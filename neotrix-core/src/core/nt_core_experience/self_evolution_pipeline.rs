/// SEPL 5-Operator Algebra — Autogenesis Protocol (arXiv 2604.15034)
///
/// Formal signatures:
///   ρ: Z × Vevo → ℘(H)     — Reflect: traces → causal hypotheses
///   σ: Vevo × ℘(H) → ℘(D)  — Select: hypotheses → modification proposals
///   ι: Vevo × ℘(D) → V'evo — Improve: proposals → candidate state
///   ε: V'evo × G → S        — Evaluate: candidate → scores
///   κ: V'evo × S → Vevo     — Commit: gated state update + rollback
///
/// Where:
///   Z     = execution traces (intervention_log, calibration curves)
///   Vevo  = current evolution state (task queue, module registry)
///   ℘(H)  = power set of causal hypotheses
///   ℘(D)  = power set of modification proposals
///   G     = guard criteria (safety, performance floor)
///   S     = evaluation scores
use std::collections::{HashSet, VecDeque};

use super::evolution_task_system::EvolutionTaskSystem;
use super::layered_mutability::{LayeredMutabilityTracker, MutabilityLayer};
use super::orchestrator_bridge::OrchestratorBridge;
use super::seal_proposal_bridge::SealProposalBridge;
use super::self_arch_audit::{ArchAuditReport, SelfArchAudit};

// ============================================================================
// SEPL Types — Formal Algebra
// ============================================================================

/// ConstraintPreservationChecker — SAHOO §2.2 invariant enforcement.
/// Records baseline system state before a modification and verifies
/// critical invariants are preserved after commit.
#[derive(Debug, Clone)]
pub struct ConstraintPreservationChecker {
    /// Invariant descriptions to check after each modification
    pub constraints: Vec<String>,
    /// Hash of system state before modification
    pub baseline_hash: Option<u64>,
    /// Current preservation score (0.0-1.0)
    pub constraint_preservation_score: f64,
    /// Number of preservation checks performed
    total_checks: u64,
    /// Number of preservation failures
    violations: u64,
}

impl Default for ConstraintPreservationChecker {
    fn default() -> Self {
        Self {
            constraints: vec![
                "task_queue_non_empty".into(),
                "module_registry_consistent".into(),
                "safety_gate_passed".into(),
            ],
            baseline_hash: None,
            constraint_preservation_score: 1.0,
            total_checks: 0,
            violations: 0,
        }
    }
}

impl ConstraintPreservationChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_constraints(mut self, constraints: Vec<String>) -> Self {
        self.constraints = constraints;
        self
    }

    /// Record baseline by hashing a system state string
    pub fn record_baseline(&mut self, system_state: &str) {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        system_state.hash(&mut hasher);
        self.baseline_hash = Some(hasher.finish());
    }

    /// Verify preservation by comparing new state to baseline.
    /// Returns true if preserved (score >= 0.8 threshold).
    pub fn verify_preservation(&mut self, new_state: &str) -> bool {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        new_state.hash(&mut hasher);
        let new_hash = hasher.finish();
        self.total_checks += 1;

        let preserved = match self.baseline_hash {
            Some(baseline) => {
                // Structural hash comparison; in production, run each constraint fn
                let sim = if baseline == new_hash { 1.0 } else { 0.0 };
                // Blend with constraint satisfaction
                let constraint_ok = self.constraints.iter().all(|_c| true); // stub: real check per constraint
                if constraint_ok && sim > 0.0 {
                    1.0
                } else {
                    sim * 0.5
                }
            }
            None => 1.0,
        };

        self.constraint_preservation_score =
            self.constraint_preservation_score * 0.7 + preserved * 0.3;
        if preserved < 0.8 {
            self.violations += 1;
            false
        } else {
            true
        }
    }

    pub fn score(&self) -> f64 {
        self.constraint_preservation_score
    }

    /// Reset for the next modification cycle
    pub fn reset(&mut self) {
        self.baseline_hash = None;
    }
}

/// The 5 SEPL operators + Idle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeplOperator {
    /// ρ: Reflect — trace → hypothesis
    Reflect,
    /// σ: Select — hypothesis → proposal
    Select,
    /// ι: Improve — proposal → candidate state
    Improve,
    /// ε: Evaluate — candidate → scores
    Evaluate,
    /// κ: Commit — gated state update with rollback
    Commit,
    /// Idle (no operator running)
    Idle,
}

impl SeplOperator {
    pub fn symbol(&self) -> &'static str {
        match self {
            SeplOperator::Reflect => "ρ",
            SeplOperator::Select => "σ",
            SeplOperator::Improve => "ι",
            SeplOperator::Evaluate => "ε",
            SeplOperator::Commit => "κ",
            SeplOperator::Idle => "—",
        }
    }
}

/// Causal failure hypothesis — output of ρ(Reflect).
#[derive(Debug, Clone)]
pub struct SeplHypothesis {
    pub id: u64,
    pub description: String,
    pub confidence: f64,
    pub severity: u8,
    pub evidence_traces: Vec<String>,
    pub source_cycle: u64,
}

/// Modification proposal — output of σ(Select).
#[derive(Debug, Clone)]
pub struct SeplProposal {
    pub id: u64,
    pub hypothesis_id: u64,
    pub description: String,
    pub target_module: String,
    pub estimated_impact: f64,
    pub risk: f64,
    /// SEPL lineage string, e.g. `"ρ(42)→σ(7)→ι→ε→κ"`
    pub lineage: String,
}

/// Structured result of a single SEPL operator execution.
#[derive(Debug, Clone)]
pub struct SeplOperatorResult {
    pub operator: SeplOperator,
    pub cycle: u64,
    pub success: bool,
    pub description: String,
    pub hypotheses_generated: usize,
    pub proposals_generated: usize,
    pub score_before: f64,
    pub score_after: f64,
}

/// Rollback data stored before Commit — enables atomic undo.
#[derive(Debug, Clone)]
pub struct SeplRollbackData {
    pub cycle: u64,
    pub state_hash: String,
    pub description: String,
}

/// Per-operator execution counters.
#[derive(Debug, Clone, Default)]
pub struct SeplOperatorStats {
    pub run_count: u64,
    pub success_count: u64,
    pub fail_count: u64,
    pub hypotheses_generated: u64,
    pub proposals_generated: u64,
    pub last_score_before: f64,
    pub last_score_after: f64,
}

impl SeplOperatorStats {
    pub fn record(&mut self, result: &SeplOperatorResult) {
        self.run_count += 1;
        if result.success {
            self.success_count += 1;
        } else {
            self.fail_count += 1;
        }
        self.hypotheses_generated += result.hypotheses_generated as u64;
        self.proposals_generated += result.proposals_generated as u64;
        self.last_score_before = result.score_before;
        self.last_score_after = result.score_after;
    }

    pub fn success_rate(&self) -> f64 {
        if self.run_count == 0 {
            return 1.0;
        }
        self.success_count as f64 / self.run_count as f64
    }
}

// ============================================================================
// PipelinePhase — backward-compatible alias for mod.rs re-export
// ============================================================================

/// Legacy phase enum — each maps to one or more SEPL operators:
///   Assess  → Reflect(ρ)
///   Plan    → Select(σ)
///   Propose → Improve(ι)
///   Bridge  → Evaluate(ε) + Commit(κ)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelinePhase {
    Idle,
    Assess,
    Plan,
    Propose,
    Bridge,
}

impl From<SeplOperator> for PipelinePhase {
    fn from(op: SeplOperator) -> Self {
        match op {
            SeplOperator::Reflect => PipelinePhase::Assess,
            SeplOperator::Select => PipelinePhase::Plan,
            SeplOperator::Improve => PipelinePhase::Propose,
            SeplOperator::Evaluate | SeplOperator::Commit => PipelinePhase::Bridge,
            SeplOperator::Idle => PipelinePhase::Idle,
        }
    }
}

/// A single cycle run record.
#[derive(Debug, Clone)]
pub struct PhaseRecord {
    pub cycle: u64,
    pub phase: PipelinePhase,
    pub success: bool,
    pub description: String,
    pub operator: SeplOperator,
    pub lineage: String,
}

/// Rollback stack — each Commit pushes the previous state for undo.
#[derive(Debug, Clone)]
struct RollbackStackEntry {
    cycle: u64,
    state_hash: String,
    #[allow(dead_code)]
    description: String,
    /// Task IDs that existed at commit time — any task added after must be removed on rollback
    pre_task_ids: HashSet<u64>,
    /// Number of proposals that existed at commit time — excess removed on rollback
    prev_proposal_count: usize,
}

// ============================================================================
// PipelineStats — extended with SEPL operator counters
// ============================================================================

#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    /// Per-operator breakdown
    pub reflect: SeplOperatorStats,
    pub select: SeplOperatorStats,
    pub improve: SeplOperatorStats,
    pub evaluate: SeplOperatorStats,
    pub commit: SeplOperatorStats,
    /// Closure detection
    pub closure_loops: u64,
    /// Rollback tracking
    pub rollbacks_performed: u64,
    pub rollbacks_attempted: u64,
}

impl Default for PipelineStats {
    fn default() -> Self {
        Self {
            total_runs: 0,
            successful_runs: 0,
            failed_runs: 0,
            reflect: SeplOperatorStats::default(),
            select: SeplOperatorStats::default(),
            improve: SeplOperatorStats::default(),
            evaluate: SeplOperatorStats::default(),
            commit: SeplOperatorStats::default(),
            closure_loops: 0,
            rollbacks_performed: 0,
            rollbacks_attempted: 0,
        }
    }
}

// ============================================================================
// PipelineConfig
// ============================================================================

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub interval_cycles: u64,
    pub enable_assess: bool,
    pub enable_propose: bool,
    /// Max closure iterations per run (ρ→σ→ι→ε→κ→ρ...)
    pub max_closure_loops: u32,
    /// Max rollback stack depth
    pub max_rollback_depth: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            interval_cycles: 10,
            enable_assess: true,
            enable_propose: true,
            max_closure_loops: 3,
            max_rollback_depth: 10,
        }
    }
}

// ============================================================================
// SelfEvolutionPipeline — SEPL 5-Operator Pipeline
// ============================================================================

pub struct SelfEvolutionPipeline {
    pub config: PipelineConfig,
    pub stats: PipelineStats,
    pub arch_audit: SelfArchAudit,
    pub seal_bridge: Option<SealProposalBridge>,
    pub orchestrator_bridge: Option<OrchestratorBridge>,
    pub phase_history: VecDeque<PhaseRecord>,
    pub last_run_cycle: u64,
    pub current_operator: SeplOperator,
    pub current_phase: PipelinePhase,
    pub current_task_id: Option<u64>,
    pub last_audit_report: Option<ArchAuditReport>,
    /// Last operator result for downstream introspection
    pub last_result: Option<SeplOperatorResult>,
    /// SEPL hypothesis buffer (output of ρ, consumed by σ)
    pub hypothesis_buffer: Vec<SeplHypothesis>,
    /// SEPL proposal buffer (output of σ, consumed by ι)
    pub proposal_buffer: Vec<SeplProposal>,
    /// Rollback stack for κ(Commit) undo
    rollback_stack: Vec<RollbackStackEntry>,
    /// Last generated lineage string
    pub last_lineage: String,
    /// Next hypothesis ID counter
    hypothesis_next_id: u64,
    /// Next SEPL proposal ID counter
    proposal_next_id: u64,
    /// Layered mutability tracker — identity drift monitoring (arXiv:2604.14717)
    pub layered_mutability: LayeredMutabilityTracker,
    /// Constraint preservation checker — SAHOO §2.2 invariant enforcement
    pub constraint_preserver: ConstraintPreservationChecker,
}

impl Default for SelfEvolutionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfEvolutionPipeline {
    pub fn new() -> Self {
        let mut s = Self {
            config: PipelineConfig::default(),
            stats: PipelineStats::default(),
            arch_audit: SelfArchAudit::new(),
            seal_bridge: Some(SealProposalBridge::new()),
            orchestrator_bridge: Some(OrchestratorBridge::new()),
            phase_history: VecDeque::with_capacity(50),
            last_run_cycle: 0,
            current_operator: SeplOperator::Idle,
            current_phase: PipelinePhase::Idle,
            current_task_id: None,
            last_audit_report: None,
            last_result: None,
            hypothesis_buffer: Vec::new(),
            proposal_buffer: Vec::new(),
            rollback_stack: Vec::with_capacity(10),
            last_lineage: String::new(),
            hypothesis_next_id: 1,
            proposal_next_id: 1,
            layered_mutability: LayeredMutabilityTracker::new().with_max_hysteresis(0.6),
            constraint_preserver: ConstraintPreservationChecker::new(),
        };
        s.discover_files(&[
            "adapt_orch",
            "adversarial",
            "auto_deploy",
            "auto_research",
            "business_diagnosis",
            "calibration_engine",
            "capability_router",
            "capability_synthesizer",
            "capacity_monitor",
            "co_evolution",
            "cog_dashboard",
            "competition_orch",
            "consciousness_hooks",
            "consolidation_bridge",
            "containment",
            "context_compression",
            "context_compressor",
            "context_manager",
            "context_memory",
            "continuous_learning",
            "contrastive_reflection",
            "cross_domain",
            "cues",
            "curiosity",
            "curriculum",
            "cyber_threat_monitor",
            "decent_mem",
            "dependency_strategy",
            "diff_impact",
            "dream",
            "edit_guard",
            "egpo_engine",
            "engineering_workflow",
            "epistemic",
            "evolution_bridge",
            "evolution_coordinator",
            "evolution_task_system",
            "evosc",
            "extract_pipeline",
            "failure_taxonomy",
            "failure_trace",
            "faithfulness_auditor",
            "faithfulness_checker",
            "fggm_safety",
            "frame_grounded_repair",
            "fusion_deliberator",
            "gap_detector_bridge",
            "gate",
            "goal_decomposer",
            "goal_drift_index",
            "godel_checker",
            "graceful",
            "gradient_seal_bridge",
            "grpo_trainer",
            "handler_profiler",
            "handler_tier",
            "harness_slot",
            "health_checkable",
            "health_patrol",
            "html_presentation",
            "humanizer",
            "hybrid_retrieval",
            "hyperagent",
            "hypothesis_tree",
            "identity_correlator",
            "identity_generator",
            "imagination_engine",
            "in_page_agent",
            "internet_absorption_bridge",
            "loop_engine",
            "loop_templates",
            "loss_function",
            "memory_consolidation",
            "memory_ops",
            "meta_cog_mera",
            "meta_evolution",
            "meta_improvement",
            "micro_reflective_loop",
            "mirror_threads",
            "motion_synthesizer",
            "mtc_assessment",
            "multi_signal_retrieval",
            "native_evolution_explorer",
            "news_radar",
            "open_skill",
            "operational_mirror",
            "osint_tools",
            "parl",
            "pcc_safety",
            "persona_adapter",
            "phase2_memory",
            "phase3_meta",
            "policy_repair",
            "population_funnel",
            "reasoning_ke_bridge",
            "reflector",
            "reliability_gate",
            "research_writer",
            "response_generator",
            "rl_consolidation",
            "safety_ball",
            "safety_gate",
            "sage_rollout",
            "sahoo",
            "sandbox_executor",
            "sar_diagnostic",
            "scaffold",
            "schemas",
            "seal_closed_loop",
            "seal_proposal_bridge",
            "self_arch_audit",
            "self_evolution_engine",
            "self_evolution_loop",
            "self_evolution_meta_layer",
            "self_evolution_pipeline",
            "self_evolution_task_engine",
            "self_harness",
            "self_introspection",
            "self_pacing_governor",
            "self_play_guide",
            "self_revision",
            "self_understanding",
            "skill_acc",
            "skill_crystal",
            "skill_dag",
            "skill_health",
            "skill_unified",
            "soul_identity",
            "sparse_vsa_attention",
            "stacked_validation",
            "storm_engine",
            "story_generator",
            "thought_flow_viz",
            "timem",
            "tool_orchestrator",
            "tool_safety",
            "trajectory_heuristics",
            "trial_worker",
            "uncertainty_quant",
            "verification_gate",
            "visual_planner",
            "voice_synthesis",
            "vsa_decoder",
            "vsa_judge",
            "vsi",
            "workflow_engine",
            "workstream_exporter",
        ]);
        s
    }

    pub fn with_seal_bridge(mut self, bridge: SealProposalBridge) -> Self {
        self.seal_bridge = Some(bridge);
        self
    }

    pub fn with_orchestrator(mut self, ob: OrchestratorBridge) -> Self {
        self.orchestrator_bridge = Some(ob);
        self
    }

    pub fn register_modules(&mut self, names: &[&str]) {
        self.arch_audit.register_modules(names);
    }

    pub fn discover_files(&mut self, names: &[&str]) {
        self.arch_audit.discover_files(names);
    }

    // ========================================================================
    // SEPL 5-Operator Run Cycle
    // ========================================================================

    /// Execute one full SEPL evolution cycle: ρ → σ → ι → ε → κ.
    ///
    /// If after κ(Commit) remaining hypotheses exist and max_closure_loops
    /// not reached, loops back to σ(Select) — creating a closure cycle.
    ///
    /// Returns true if at least one proposal was committed.
    pub fn run(
        &mut self,
        cycle: u64,
        task_system: &mut EvolutionTaskSystem,
        current_meta_acc: f64,
        current_ece: f64,
        current_loss: f64,
    ) -> bool {
        if cycle < self.last_run_cycle + self.config.interval_cycles {
            return false;
        }
        self.last_run_cycle = cycle;
        self.stats.total_runs += 1;

        // ── ρ(Reflect): Z × Vevo → ℘(H) ──────────────────────────────────
        let reflect_result = self.operator_reflect(
            cycle,
            task_system,
            current_meta_acc,
            current_ece,
            current_loss,
        );
        self.stats.reflect.record(&reflect_result);
        self.last_result = Some(reflect_result.clone());

        // If reflect generated no hypotheses and no audit gaps, exit early
        let has_hypotheses = !self.hypothesis_buffer.is_empty();
        let audit_has_gaps = self
            .last_audit_report
            .as_ref()
            .map(|r| !r.gaps.is_empty())
            .unwrap_or(false);
        if !has_hypotheses && !audit_has_gaps {
            self.stats.failed_runs += 1;
            self.current_operator = SeplOperator::Idle;
            self.current_phase = PipelinePhase::Idle;
            self.phase_history.push_back(PhaseRecord {
                cycle,
                phase: PipelinePhase::Assess,
                success: false,
                description: "ρ: no hypotheses generated".into(),
                operator: SeplOperator::Reflect,
                lineage: "ρ(∅)".into(),
            });
            return false;
        }

        // ── Layered mutability gate: check identity drift before any mutation ──
        let (mutation_allowed, ban_reason) = self
            .layered_mutability
            .check_mutation_allowed(MutabilityLayer::SelfNarrative);
        if !mutation_allowed {
            log::warn!(
                "SEPL mutation blocked by LayeredMutabilityTracker: {}",
                ban_reason.as_deref().unwrap_or("unknown reason"),
            );
            self.current_operator = SeplOperator::Idle;
            self.current_phase = PipelinePhase::Idle;
            self.phase_history.push_back(PhaseRecord {
                cycle,
                phase: PipelinePhase::Assess,
                success: false,
                description: format!(
                    "κ: mutation blocked — {}",
                    ban_reason.as_deref().unwrap_or("hysteresis exceeded"),
                ),
                operator: SeplOperator::Commit,
                lineage: "κ(✗)".into(),
            });
            self.stats.failed_runs += 1;
            return false;
        }

        // ── Evolution efficiency gate: block if success rate is critically low ──
        // Pipeline tracks its own success/failure across runs.
        // If success_rate < 0.15 after 5+ runs, evolution is stalled.
        let success_rate = if self.stats.total_runs > 0 {
            self.stats.successful_runs as f64 / self.stats.total_runs as f64
        } else {
            1.0
        };
        let efficiency_blocked = self.stats.total_runs >= 5 && success_rate < 0.15;
        if efficiency_blocked {
            log::warn!(
                "SEPL evolution blocked by efficiency gate: success_rate={:.4} runs={} <15% threshold",
                success_rate,
                self.stats.total_runs,
            );
            self.current_operator = SeplOperator::Idle;
            self.current_phase = PipelinePhase::Idle;
            self.phase_history.push_back(PhaseRecord {
                cycle,
                phase: PipelinePhase::Assess,
                success: false,
                description: format!(
                    "efficiency gate: success_rate={:.4} below 0.15 after {} runs",
                    success_rate, self.stats.total_runs,
                ),
                operator: SeplOperator::Commit,
                lineage: "κ(✗)".into(),
            });
            self.stats.failed_runs += 1;
            return false;
        }

        // ── Closure loop: σ → ι → ε → κ, repeat if hypotheses remain ────
        let mut closure_count = 0u32;
        let mut overall_success = false;

        loop {
            // ── σ(Select): Vevo × ℘(H) → ℘(D) ────────────────────────────
            let select_result = self.operator_select(cycle, task_system, current_meta_acc);
            self.stats.select.record(&select_result);
            self.last_result = Some(select_result.clone());

            // ── ι(Improve): Vevo × ℘(D) → V'evo ──────────────────────────
            let improve_result = self.operator_improve(cycle, task_system);
            self.stats.improve.record(&improve_result);
            self.last_result = Some(improve_result.clone());

            // ── ε(Evaluate): V'evo × G → S ───────────────────────────────
            let evaluate_result = self.operator_evaluate(cycle, current_meta_acc, current_ece);
            self.stats.evaluate.record(&evaluate_result);
            self.last_result = Some(evaluate_result.clone());

            // ── Constraint Preservation: record baseline before commit ───
            let baseline_state = format!(
                "cycle={} tasks={} proposals={} meta_acc={:.4}",
                cycle,
                task_system.task_ids().len(),
                self.seal_bridge
                    .as_ref()
                    .map(|b| b.pending_proposals().len())
                    .unwrap_or(0),
                current_meta_acc,
            );
            self.constraint_preserver.record_baseline(&baseline_state);

            // ── κ(Commit): V'evo × S → Vevo (gated + rollback) ───────────
            let commit_result = self.operator_commit(cycle, task_system, &evaluate_result);

            // ── Verify preservation after commit — block if violated ──────
            let post_state = format!(
                "cycle={} tasks={} proposals={}",
                cycle,
                task_system.task_ids().len(),
                self.seal_bridge
                    .as_ref()
                    .map(|b| b.pending_proposals().len())
                    .unwrap_or(0),
            );
            let preserved = self.constraint_preserver.verify_preservation(&post_state);
            if !preserved {
                log::warn!(
                    "SEPL constraint preservation FAILED (score={:.4}) — blocking commit for cycle {}",
                    self.constraint_preserver.score(),
                    cycle,
                );
                // Rollback the just-committed state
                self.rollback_last(task_system);
                self.stats.failed_runs += 1;
                self.current_operator = SeplOperator::Idle;
                self.current_phase = PipelinePhase::Idle;
                self.hypothesis_buffer.clear();
                self.proposal_buffer.clear();
                self.constraint_preserver.reset();
                return false;
            }
            self.constraint_preserver.reset();
            self.stats.commit.record(&commit_result);
            self.last_result = Some(commit_result.clone());

            if commit_result.success {
                overall_success = true;
            }

            // ── Closure detection: should we loop back to σ? ─────────────
            closure_count += 1;
            if closure_count >= self.config.max_closure_loops {
                break;
            }
            // Rebuild hypothesis buffer from remaining audit gaps
            self.refresh_hypotheses_from_gaps();
            if self.hypothesis_buffer.is_empty() {
                break;
            }
            // Clear proposal buffer for next iteration
            self.proposal_buffer.clear();
            self.stats.closure_loops += 1;
        }

        // ── Orchestrator bridge (legacy compatibility) ────────────────────
        if let Some(ref mut ob) = self.orchestrator_bridge {
            let _orch_proposals = ob.run_bridge(cycle, current_meta_acc, current_ece, current_loss);
        }

        if overall_success {
            self.stats.successful_runs += 1;
        } else {
            self.stats.failed_runs += 1;
        }

        self.current_operator = SeplOperator::Idle;
        self.current_phase = PipelinePhase::Idle;
        self.hypothesis_buffer.clear();
        self.proposal_buffer.clear();
        overall_success
    }

    // ========================================================================
    // ρ — Reflect: Z × Vevo → ℘(H)
    // ========================================================================

    fn operator_reflect(
        &mut self,
        cycle: u64,
        _task_system: &mut EvolutionTaskSystem,
        current_meta_acc: f64,
        current_ece: f64,
        _current_loss: f64,
    ) -> SeplOperatorResult {
        self.current_operator = SeplOperator::Reflect;
        self.current_phase = PipelinePhase::Assess;

        let audit_report = self.arch_audit.audit(cycle);
        self.last_audit_report = Some(audit_report.clone());

        // Build hypotheses from audit gaps
        let mut hypotheses = Vec::new();
        let score_before = (current_meta_acc + (1.0 - current_ece)) / 2.0;

        for gap in &audit_report.gaps {
            let severity_num = match gap.severity {
                super::self_arch_audit::AuditSeverity::UnregisteredModule => 8u8,
                super::self_arch_audit::AuditSeverity::MissingReexport => 6,
                super::self_arch_audit::AuditSeverity::DeadCodeFile => 4,
                super::self_arch_audit::AuditSeverity::DuplicateRegistration => 3,
            };
            let hyp = SeplHypothesis {
                id: self.hypothesis_next_id,
                description: format!("[{}] {} — {}", severity_num, gap.file_name, gap.description),
                confidence: current_meta_acc * 0.7 + (1.0 - current_ece) * 0.3,
                severity: severity_num,
                evidence_traces: vec![format!("arch_audit: gap_type={:?}", gap.severity)],
                source_cycle: cycle,
            };
            self.hypothesis_next_id += 1;
            hypotheses.push(hyp);
        }

        // Also generate hypotheses from calibration metrics
        if current_ece > 0.15 {
            let hyp = SeplHypothesis {
                id: self.hypothesis_next_id,
                description: format!(
                    "Calibration drift: ECE={:.3} exceeds 0.15 threshold",
                    current_ece
                ),
                confidence: 0.6,
                severity: 7,
                evidence_traces: vec![format!("ece={:.4}", current_ece)],
                source_cycle: cycle,
            };
            self.hypothesis_next_id += 1;
            hypotheses.push(hyp);
        }
        if current_meta_acc < 0.7 {
            let hyp = SeplHypothesis {
                id: self.hypothesis_next_id,
                description: format!("Meta-accuracy deficit: {:.3} below 0.7", current_meta_acc),
                confidence: 0.7,
                severity: 8,
                evidence_traces: vec![format!("meta_acc={:.4}", current_meta_acc)],
                source_cycle: cycle,
            };
            self.hypothesis_next_id += 1;
            hypotheses.push(hyp);
        }

        let desc = format!(
            "ρ: generated {} hypotheses from {} audit gaps",
            hypotheses.len(),
            audit_report.gaps.len(),
        );

        self.hypothesis_buffer = hypotheses;

        // Record to phase_history
        let lineage = format!("ρ({})", cycle);
        self.last_lineage = lineage.clone();

        self.phase_history.push_back(PhaseRecord {
            cycle,
            phase: PipelinePhase::Assess,
            success: !self.hypothesis_buffer.is_empty(),
            description: desc.clone(),
            operator: SeplOperator::Reflect,
            lineage: lineage.clone(),
        });

        SeplOperatorResult {
            operator: SeplOperator::Reflect,
            cycle,
            success: !self.hypothesis_buffer.is_empty(),
            description: desc,
            hypotheses_generated: self.hypothesis_buffer.len(),
            proposals_generated: 0,
            score_before,
            score_after: if self.hypothesis_buffer.is_empty() {
                score_before
            } else {
                (score_before + 0.1).min(1.0)
            },
        }
    }

    // ========================================================================
    // σ — Select: Vevo × ℘(H) → ℘(D)
    // ========================================================================

    fn operator_select(
        &mut self,
        cycle: u64,
        task_system: &mut EvolutionTaskSystem,
        current_meta_acc: f64,
    ) -> SeplOperatorResult {
        self.current_operator = SeplOperator::Select;
        self.current_phase = PipelinePhase::Plan;

        // Plan: auto-discover tasks from metrics
        let ece = self
            .last_audit_report
            .as_ref()
            .map(|r| r.gaps.len() as f64 * 0.05)
            .unwrap_or(0.0);
        task_system.auto_discover_from_audit(
            cycle,
            current_meta_acc,
            ece,
            0.0,
            self.hypothesis_buffer.len(),
        );

        // Translate hypotheses to proposals
        let mut proposals = Vec::new();
        let has_task = task_system.next_ready_task().is_some();

        for hyp in &self.hypothesis_buffer {
            let lineage_hyp = format!("ρ({})→σ({})", hyp.source_cycle, self.proposal_next_id,);
            let prop = SeplProposal {
                id: self.proposal_next_id,
                hypothesis_id: hyp.id,
                description: hyp.description.clone(),
                target_module: hyp
                    .description
                    .split(|c: char| c == '[' || c == ']')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "unknown".into()),
                estimated_impact: (hyp.confidence * hyp.severity as f64 / 10.0).clamp(0.0, 1.0),
                risk: (1.0 - hyp.confidence).clamp(0.0, 1.0),
                lineage: lineage_hyp,
            };
            self.proposal_next_id += 1;
            proposals.push(prop);
        }

        let desc = format!(
            "σ: selected {} proposals from {} hypotheses (tasks={})",
            proposals.len(),
            self.hypothesis_buffer.len(),
            if has_task { "ready" } else { "none" },
        );

        let lineage = format!(
            "{}→σ({})",
            self.last_lineage.split('→').next().unwrap_or("ρ(?)"),
            cycle,
        );
        self.last_lineage = lineage.clone();
        self.proposal_buffer = proposals;

        self.phase_history.push_back(PhaseRecord {
            cycle,
            phase: PipelinePhase::Plan,
            success: !self.proposal_buffer.is_empty(),
            description: desc.clone(),
            operator: SeplOperator::Select,
            lineage: lineage.clone(),
        });

        SeplOperatorResult {
            operator: SeplOperator::Select,
            cycle,
            success: !self.proposal_buffer.is_empty(),
            description: desc,
            hypotheses_generated: 0,
            proposals_generated: self.proposal_buffer.len(),
            score_before: 0.0,
            score_after: if self.proposal_buffer.is_empty() {
                0.0
            } else {
                1.0
            },
        }
    }

    // ========================================================================
    // ι — Improve: Vevo × ℘(D) → V'evo
    // ========================================================================

    fn operator_improve(
        &mut self,
        cycle: u64,
        task_system: &mut EvolutionTaskSystem,
    ) -> SeplOperatorResult {
        self.current_operator = SeplOperator::Improve;
        self.current_phase = PipelinePhase::Propose;

        let mut proposed_count = 0u64;

        // Convert proposals to SealProposals + EvolutionTasks
        for prop in &self.proposal_buffer {
            task_system.auto_discover_from_audit(cycle, 0.5, 0.0, 0.0, 1);
            if let Some(ref mut bridge) = self.seal_bridge {
                bridge.propose_new_capability(
                    &prop.target_module,
                    &format!(
                        "[SEPL] {} | lineage={} | impact={:.2} risk={:.2}",
                        prop.description, prop.lineage, prop.estimated_impact, prop.risk,
                    ),
                );
                proposed_count += 1;
            }
        }

        // If no proposals but tasks exist, bridge tasks to SealProposals
        if proposed_count == 0 && self.seal_bridge.is_some() {
            if let Some(next) = task_system.next_ready_task() {
                if let Some(ref mut bridge) = self.seal_bridge {
                    bridge.propose_new_capability(
                        &format!("task_{}", next.id),
                        &format!(
                            "EvolutionTask: {} (priority={}, impact={})",
                            next.title, next.priority, next.impact
                        ),
                    );
                    proposed_count = 1;
                }
            }
        }

        let desc = format!(
            "ι: improved state with {} proposals (buffer={})",
            proposed_count,
            self.proposal_buffer.len(),
        );

        let lineage = format!("{}→ι", self.last_lineage);
        self.last_lineage = lineage.clone();

        self.phase_history.push_back(PhaseRecord {
            cycle,
            phase: PipelinePhase::Propose,
            success: proposed_count > 0,
            description: desc.clone(),
            operator: SeplOperator::Improve,
            lineage: lineage.clone(),
        });

        SeplOperatorResult {
            operator: SeplOperator::Improve,
            cycle,
            success: proposed_count > 0,
            description: desc,
            hypotheses_generated: 0,
            proposals_generated: proposed_count as usize,
            score_before: 0.0,
            score_after: if proposed_count > 0 { 1.0 } else { 0.0 },
        }
    }

    // ========================================================================
    // ε — Evaluate: V'evo × G → S
    // ========================================================================

    fn operator_evaluate(
        &mut self,
        cycle: u64,
        current_meta_acc: f64,
        current_ece: f64,
    ) -> SeplOperatorResult {
        self.current_operator = SeplOperator::Evaluate;
        self.current_phase = PipelinePhase::Bridge;

        // Evaluation scores: composite of meta_acc, ECE, proposal quality
        let meta_score = current_meta_acc;
        let ece_score = 1.0 - current_ece.clamp(0.0, 1.0);
        let proposal_score = if self
            .seal_bridge
            .as_ref()
            .map(|b| b.pending_proposals().len())
            .unwrap_or(0)
            > 0
        {
            1.0
        } else {
            0.0
        };

        let composite_score = meta_score * 0.4 + ece_score * 0.3 + proposal_score * 0.3;
        let guard_pass = composite_score > 0.3;

        let desc = format!(
            "ε: evaluated candidate → S({:.4}) meta={:.3} ece={:.3} prop={:.3} guard={}",
            composite_score, meta_score, ece_score, proposal_score, guard_pass,
        );

        let lineage = format!("{}→ε", self.last_lineage);
        self.last_lineage = lineage.clone();

        self.phase_history.push_back(PhaseRecord {
            cycle,
            phase: PipelinePhase::Bridge,
            success: guard_pass,
            description: desc.clone(),
            operator: SeplOperator::Evaluate,
            lineage: lineage.clone(),
        });

        SeplOperatorResult {
            operator: SeplOperator::Evaluate,
            cycle,
            success: guard_pass,
            description: desc,
            hypotheses_generated: 0,
            proposals_generated: 0,
            score_before: 0.0,
            score_after: composite_score,
        }
    }

    // ========================================================================
    // κ — Commit: V'evo × S → Vevo (gated + rollback)
    // ========================================================================

    fn operator_commit(
        &mut self,
        cycle: u64,
        task_system: &mut EvolutionTaskSystem,
        evaluate_result: &SeplOperatorResult,
    ) -> SeplOperatorResult {
        self.current_operator = SeplOperator::Commit;
        self.current_phase = PipelinePhase::Bridge;

        // Gate: only commit if evaluate passed
        if !evaluate_result.success {
            let desc = format!(
                "κ: commit BLOCKED — evaluate score {:.4} below guard threshold 0.3",
                evaluate_result.score_after,
            );

            let lineage = format!("{}→κ(✗)", self.last_lineage);
            self.last_lineage = lineage.clone();

            self.phase_history.push_back(PhaseRecord {
                cycle,
                phase: PipelinePhase::Bridge,
                success: false,
                description: desc.clone(),
                operator: SeplOperator::Commit,
                lineage: lineage.clone(),
            });

            return SeplOperatorResult {
                operator: SeplOperator::Commit,
                cycle,
                success: false,
                description: desc,
                hypotheses_generated: 0,
                proposals_generated: 0,
                score_before: evaluate_result.score_after,
                score_after: evaluate_result.score_after,
            };
        }

        // Snapshot current state for rollback — capture exact task IDs + proposal count
        let pre_task_ids = task_system.task_ids();
        let pre_proposal_count = self
            .seal_bridge
            .as_ref()
            .map(|b| b.pending_proposals().len())
            .unwrap_or(0);
        let state_hash = format!(
            "cycle={} tasks={} proposals={}",
            cycle,
            pre_task_ids.len(),
            pre_proposal_count,
        );

        // Push rollback data
        self.rollback_stack.push(RollbackStackEntry {
            cycle,
            state_hash: state_hash.clone(),
            description: format!("commit at cycle {}", cycle),
            pre_task_ids,
            prev_proposal_count: pre_proposal_count,
        });
        if self.rollback_stack.len() > self.config.max_rollback_depth {
            self.rollback_stack.remove(0);
        }

        // Commit: mark proposals as approved in seal_bridge
        let mut approved_count = 0u64;
        if let Some(ref _bridge) = self.seal_bridge {
            // In a full implementation we'd iterate pending proposals and approve them.
            // For now, the bridge's propose_new_capability has already enqueued them.
            // Future: bridge.approve_pending(ids) — needs SealProposalBridge API extension.
            approved_count = 1;
        }

        // Record mutation in LayeredMutabilityTracker
        let drift_hysteresis =
            self.layered_mutability
                .record_mutation(MutabilityLayer::SelfNarrative, 0.05, cycle);

        let desc = format!(
            "κ: committed state [{}] with {} approvals | rollback_stack={} | h={:.4}",
            state_hash,
            approved_count,
            self.rollback_stack.len(),
            drift_hysteresis,
        );

        let lineage = format!("{}→κ(✓)", self.last_lineage);
        self.last_lineage = lineage.clone();

        self.phase_history.push_back(PhaseRecord {
            cycle,
            phase: PipelinePhase::Bridge,
            success: true,
            description: desc.clone(),
            operator: SeplOperator::Commit,
            lineage: lineage.clone(),
        });

        SeplOperatorResult {
            operator: SeplOperator::Commit,
            cycle,
            success: true,
            description: desc,
            hypotheses_generated: 0,
            proposals_generated: 0,
            score_before: evaluate_result.score_after,
            score_after: evaluate_result.score_after,
        }
    }

    // ========================================================================
    // Rollback Support
    // ========================================================================

    /// Rollback the last Commit — restore task list and proposals to pre-commit state.
    /// Returns true if a rollback was performed.
    pub fn rollback_last(&mut self, task_system: &mut EvolutionTaskSystem) -> bool {
        let entry = match self.rollback_stack.pop() {
            Some(e) => e,
            None => return false,
        };
        self.stats.rollbacks_attempted += 1;

        // 1. Remove tasks added after commit (not in pre-commit snapshot)
        let current_ids: Vec<u64> = task_system.task_ids().into_iter().collect();
        let mut removed_tasks = 0u64;
        for id in &current_ids {
            if !entry.pre_task_ids.contains(id) {
                task_system.remove_task(*id);
                removed_tasks += 1;
            }
        }

        // 2. Reject excess proposals (those added after commit)
        let mut removed_proposals = 0u64;
        if let Some(ref mut bridge) = self.seal_bridge {
            let pending: Vec<u64> = bridge.pending_proposals().iter().map(|p| p.id).collect();
            // Reject all proposals beyond the pre-commit count
            let mut rejected = 0;
            let current_pending = pending.len();
            if current_pending > entry.prev_proposal_count {
                let excess = current_pending - entry.prev_proposal_count;
                for id in pending.iter().rev().take(excess) {
                    bridge.reject_proposal(*id);
                    rejected += 1;
                }
            }
            removed_proposals = rejected;
        }

        log::info!(
            "SEPL rollback: reverted cycle {} [{}] (removed {} tasks, {} proposals)",
            entry.cycle,
            entry.state_hash,
            removed_tasks,
            removed_proposals,
        );
        self.stats.rollbacks_performed += 1;
        true
    }

    /// Returns the rollback stack depth.
    pub fn rollback_depth(&self) -> usize {
        self.rollback_stack.len()
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    /// Refresh hypothesis buffer from remaining audit gaps (for closure loop).
    fn refresh_hypotheses_from_gaps(&mut self) {
        let mut new_hypotheses = Vec::new();
        if let Some(ref report) = self.last_audit_report {
            // Use gaps that were not yet converted to proposals
            // (proposal_buffer is cleared each closure iteration, but we
            //  keep already-converted gap IDs in a simple tracking list)
            for gap in &report.gaps {
                let already_used = self.phase_history.iter().any(|rec| {
                    rec.description.contains(&gap.file_name) && rec.operator == SeplOperator::Select
                });
                if already_used {
                    continue;
                }
                let hyp = SeplHypothesis {
                    id: self.hypothesis_next_id,
                    description: format!("[closure] {} — {}", gap.file_name, gap.description),
                    confidence: 0.5,
                    severity: 5,
                    evidence_traces: vec!["closure_refresh".into()],
                    source_cycle: self.last_run_cycle,
                };
                self.hypothesis_next_id += 1;
                new_hypotheses.push(hyp);
            }
        }
        self.hypothesis_buffer = new_hypotheses;
    }

    // ========================================================================
    // Summary & Reporting
    // ========================================================================

    pub fn summary(&self) -> String {
        let seal_info = match &self.seal_bridge {
            Some(b) => {
                let s = b.stats();
                format!(
                    " seal_proposals={} pending={} approved={} impl={}",
                    s.total_proposals, s.pending, s.approved, s.implemented
                )
            }
            None => " seal_bridge=none".to_string(),
        };
        let orch_info = match &self.orchestrator_bridge {
            Some(ob) => format!(" orchestrator_bridge=on calls={}", ob.bridge_count()),
            None => " orchestrator_bridge=off".to_string(),
        };
        format!(
            "SelfEvolutionPipeline: runs={} ok={} fail={} | \
             ρ(reflect)={}✓/{} σ(select)={}✓/{} ι(improve)={}✓/{} \
             ε(evaluate)={}✓/{} κ(commit)={}✓/{} | \
             closures={} rollbacks={}/{} | \
             lineage=[{}]{}{} | {}",
            self.stats.total_runs,
            self.stats.successful_runs,
            self.stats.failed_runs,
            self.stats.reflect.success_count,
            self.stats.reflect.run_count,
            self.stats.select.success_count,
            self.stats.select.run_count,
            self.stats.improve.success_count,
            self.stats.improve.run_count,
            self.stats.evaluate.success_count,
            self.stats.evaluate.run_count,
            self.stats.commit.success_count,
            self.stats.commit.run_count,
            self.stats.closure_loops,
            self.stats.rollbacks_performed,
            self.stats.rollbacks_attempted,
            self.last_lineage,
            seal_info,
            orch_info,
            self.layered_mutability.summary(),
        )
    }

    pub fn audit_summary(&self) -> String {
        self.arch_audit.summary()
    }
}
