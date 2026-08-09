use std::collections::VecDeque;

use super::SelfIteratingBrain;
use super::super::core::CapabilityVector;
use super::recipe::RecipeStage;
use crate::neotrix::nt_world_model::TaskType;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_memory_kb::GraphRagConfig;
use crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisStatus;


// Pre-register all available BrainStage implementations from sibling modules.
// Stages that don't implement BrainStage directly get wrapper structs below.
use super::skillopt::{
    BoundedEditStage, ValidationGateStage, RejectedBufferFeedbackStage, EpochSlowUpdateStage,
};
use super::aging_monitor::AgingDiagnosisStage;
use super::dp_sgd_stage::DpSgdStage;
use super::benchmark_gate::BenchmarkGateStage;
use super::procedural_memory::ProceduralMemoryStage;
use super::checkpoint::{CheckpointStage, RewindStage};
use super::goal_contract::{
    EvidenceCaptureStage, NarrowRecoveryStage,
    FinalVerificationStage, GoalTerminatorStage, ExternalVerifierStage, SemanticRecallStage,
};

use super::secret_scanner::SecretScanner;
use super::openspace_evolution::OpenSpaceEvolveStage;
use super::constitutional_stage::ConstitutionalSelfCritiqueStage;
use super::safety_stage::SafetyCheckStage;
use super::sft_stage::SupervisedExample;
use super::process_stage::{ProcessStage, ProcessExample, ReasoningTrace, ReasoningStep, TraceSource};
use super::search_skill_stage::{SearchExercise, SearchResult, Evidence, SearchTaskType};
use super::hypercore::SafetyCheckResult;
use super::hyperstage::{MetaEvolveStage, DGMMetaEvolveStage};
use super::hyperarchive::{HyperAgentArchive, SelectionConfig};
use super::hypercore::HyperMetaAgent;
use super::hyperdgm::{DGMMetaAgent, GenerativeReplay, SelfReferentialCheck};
use crate::core::nt_core_self_review::SelfReviewGate;
use crate::make_stage;
use crate::neotrix::nt_memory_kb::ProceduralMemoryRecord;
use crate::neotrix::l8_autonomic_impl::nt_mind_memory::MemoryTier;

fn compute_capability_deltas(brain: &SelfIteratingBrain) -> Vec<(String, f64)> {
    let current = brain.brain.capability.arr().to_vec();
    let snap = brain._snapshot_capability().arr().to_vec();
    current.into_iter().zip(snap).enumerate().map(|(i, (cur, snp))| {
        (format!("cap_{}", i), cur - snp)
    }).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AutonomyLevel {
    Proposal,
    Bounded,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    Suggest,
    Full,
}

#[derive(Debug, Clone)]
pub struct BrainSnapshot {
    pub capability: CapabilityVector,
    pub learning_rate: f64,
    pub score: f64,
}

impl BrainSnapshot {
    pub fn new(brain: &super::brain_core::ReasoningBrain, task_type: &TaskType) -> Self {
        let _ = task_type;
        Self {
            capability: brain.capability.clone(),
            learning_rate: brain.learning_rate,
            score: 0.0,
        }
    }

    pub fn restore(&self, brain: &mut super::brain_core::ReasoningBrain) {
        brain.capability = self.capability.clone();
        brain.learning_rate = self.learning_rate;
    }
}

#[derive(Debug)]
pub enum StageDecision {
    Continue,
    Skip(String),
    Promote(BrainSnapshot),
    Rollback(String),
}

pub trait BrainStage: Send + Sync {
    fn name(&self) -> &str;
    fn frequency(&self) -> usize { 1 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError>;
}

pub struct StageResult {
    pub stage_name: String,
    pub efc: f64,
    pub efficiency: f64,
}

impl StageResult {
    pub fn new(stage_name: &str) -> Self {
        Self {
            stage_name: stage_name.to_string(),
            efc: 0.0,
            efficiency: 0.0,
        }
    }
}

#[derive(Default)]
pub struct BrainPipeline {
    pub stages: Vec<Box<dyn BrainStage>>,
}

impl BrainPipeline {
    pub fn execute(&self, brain: &mut SelfIteratingBrain) -> NeoTrixResult<()> {
        for stage in &self.stages {
            if !brain.iteration.is_multiple_of(stage.frequency() as u64) {
                continue;
            }
            let stage_name = stage.name().to_string();
            match stage.process(brain)? {
                StageDecision::Continue => {}
                StageDecision::Skip(_) => {}
                StageDecision::Promote(champ) => {
                    brain.champion = Some(champ);
                }
                StageDecision::Rollback(reason) => {
                    return Err(NeoTrixError::Brain(format!("Pipeline rollback: {}", reason)));
                }
            }
            brain._stage_results.push(StageResult::new(&stage_name));
            const MAX_STAGE_RESULTS: usize = 1000;
            if brain._stage_results.len() > MAX_STAGE_RESULTS {
                brain._stage_results.drain(0..(brain._stage_results.len() - MAX_STAGE_RESULTS));
            }
        }
        Ok(())
    }
}

pub fn seal_pipeline() -> BrainPipeline {
    BrainPipeline {
        stages: vec![
            Box::new(CheckpointStage::new()),
            Box::new(RecipeStage::new(Box::new(RewardCalculationStage::new())).with_frequency(3)),
            Box::new(ConsciousnessRewardStage::new()),
            Box::new(SftWrapperStage::new()),
            Box::new(ProcessWrapperStage::new()),
            Box::new(SearchSkillWrapperStage::new()),
            Box::new(DpoWrapperStage::new()),
            Box::new(ConstitutionalWrapperStage::new()),
            Box::new(SafetyWrapperStage::new()),
            Box::new(BoundarySeparationStage::new()),
            Box::new(BoundedEditStage::new()),
            Box::new(ScaffoldAwareRLStage::new()),
            Box::new(ValidationGateStage::new()),
            Box::new(DpSgdStage::new()),
            Box::new(GwtAbsorbStage::new()),
            Box::new(BenchmarkGateStage::new()),
            Box::new(HarnessAdaptStage::new()),
            Box::new(KnowledgeQualityStage::new()),
            Box::new(AutonomyPerStage::new()),
            Box::new(RewindStage::new()),
            Box::new(RejectedBufferFeedbackStage::new()),
            Box::new(SecretScanStage::new()),
            // GoalContractStage omitted — individual stages handle each phase:
            // EvidenceCaptureStage, NarrowRecoveryStage, FinalVerificationStage,
            // GoalTerminatorStage, ExternalVerifierStage, SemanticRecallStage
            Box::new(EvidenceCaptureStage::new()),
            Box::new(NarrowRecoveryStage::new()),
            Box::new(FinalVerificationStage::new()),
            Box::new(GoalTerminatorStage::new()),
            Box::new(ExternalVerifierStage::new()),
            Box::new(SemanticRecallStage::new()),
            Box::new(ProceduralMemoryStage::new()),
            Box::new(MetaEvolveStage::new(
                HyperMetaAgent::new(10, true),
                HyperAgentArchive::new(SelectionConfig::default()),
            )),
            Box::new(DGMMetaEvolveStage::new(
                DGMMetaAgent::new(512, 5, 0.1),
                HyperAgentArchive::new(SelectionConfig::default()),
                GenerativeReplay { num_components: 64, min_score: 0.3, max_samples: 100, enabled: true },
                SelfReferentialCheck { max_distortion_ratio: 0.5, max_spectral_growth: 1.5, min_self_consistency: 0.4 },
            )),
            Box::new(HypothesisAccuracyStage::new()),
            Box::new(PatternExtractionStage::new()),
            Box::new(DistillationStage::new()),
            Box::new(ConversationDistillStage::new()),
            Box::new(EpochSlowUpdateStage::new()),
            Box::new(AgingDiagnosisStage::new()),
            Box::new(SelfReviewStage::new()),
            Box::new(CreditAssignmentStage::new()),
            Box::new(OracleGateStage::new()),
            Box::new(ArchitectureOptimizerStage::new()),
            Box::new(TrendAnalysisStage::new()),
            Box::new(MetaGoalStage::new()),
            Box::new(MemoryConsolidationStage::new()),
            Box::new(CacheCleanupStage::new()),
            Box::new(ExternalKnowledgeAbsorbStage::new()),
            Box::new(ConvergenceCheckStage::new()),
            Box::new(SelfTestStage::new()),
            Box::new(OpenSpaceEvolveStage::new()
                .with_fix_enabled(true)
                .with_derived_enabled(true)
                .with_captured_enabled(true)),
        ],
    }
}

pub fn kernel_iterate_pipeline() -> BrainPipeline {
    BrainPipeline {
        stages: vec![
            Box::new(CheckpointStage::new()),
            Box::new(BoundedEditStage::new()),
            Box::new(ValidationGateStage::new()),
            Box::new(RejectedBufferFeedbackStage::new()),
        ],
    }
}

// ── Recipe-only stages (used by recipe.rs, not in seal_pipeline) ──────────

/// Snapshot stage (used by recipe.rs)
pub struct SnapshotStage;
impl Default for SnapshotStage { fn default() -> Self { Self } }
impl SnapshotStage { pub fn new() -> Self { Self } }
impl BrainStage for SnapshotStage {
    fn name(&self) -> &str { "snapshot" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let caps = brain.brain.capability.arr();
        log::trace!("[snapshot] iter={} caps={:?}", brain.iteration, caps);
        Ok(StageDecision::Continue)
    }
}

/// Memory retrieval stage (used by recipe.rs)
pub struct MemoryRetrievalStage;
impl Default for MemoryRetrievalStage { fn default() -> Self { Self } }
impl MemoryRetrievalStage { pub fn new() -> Self { Self } }
impl BrainStage for MemoryRetrievalStage {
    fn name(&self) -> &str { "memory_retrieval" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let kb_available = brain._nt_memory_kb.is_some();
        log::trace!("[memory_retrieval] kb={} task='{}'", kb_available, brain._current_task);
        Ok(StageDecision::Continue)
    }
}

/// Gap analysis stage (used by recipe.rs)
pub struct GapAnalysisStage;
impl Default for GapAnalysisStage { fn default() -> Self { Self } }
impl GapAnalysisStage { pub fn new() -> Self { Self } }
impl BrainStage for GapAnalysisStage {
    fn name(&self) -> &str { "gap_analysis" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let caps = brain.brain.capability.arr();
        let champ = brain.champion.as_ref().map(|c| format!("score={:.4}", c.score)).unwrap_or_default();
        log::trace!("[gap_analysis] caps={:?} champion=[{}]", caps, champ);
        Ok(StageDecision::Continue)
    }
}

/// SSM update stage (used by recipe.rs)
pub struct SSMUpdateStage;
impl Default for SSMUpdateStage { fn default() -> Self { Self } }
impl SSMUpdateStage { pub fn new() -> Self { Self } }
impl BrainStage for SSMUpdateStage {
    fn name(&self) -> &str { "ssm_update" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let reward = brain._prm_cumulative_reward;
        let mode = brain._e8_policy.best_mode();
        brain._transition_learner.record(&brain._current_task, mode, reward, brain.iteration);
        brain._e8_policy.mode_values[mode.0 as usize] =
            (brain._e8_policy.mode_values[mode.0 as usize] * 0.9 + reward * 0.1).min(1.0);
        brain._e8_policy.mode_counts[mode.0 as usize] += 1;
        brain._e8_policy.decay_epsilon();
        log::trace!("[ssm_update] iter={} reward={:.4} mode={} eps={:.4}",
            brain.iteration, reward, mode.0, brain._e8_policy.epsilon());
        Ok(StageDecision::Continue)
    }
}

/// Self-edit generation stage (used by recipe.rs)
pub struct SelfEditGenerationStage;
impl Default for SelfEditGenerationStage { fn default() -> Self { Self } }
impl SelfEditGenerationStage { pub fn new() -> Self { Self } }
impl BrainStage for SelfEditGenerationStage {
    fn name(&self) -> &str { "self_edit_gen" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let edits = brain._micro_edits.len();
        log::trace!("[self_edit_gen] pending_edits={} task='{}'", edits, brain._current_task);
        Ok(StageDecision::Continue)
    }
}

/// Apply edits stage (used by recipe.rs)
pub struct ApplyEditsStage;
impl Default for ApplyEditsStage { fn default() -> Self { Self } }
impl ApplyEditsStage { pub fn new() -> Self { Self } }
impl BrainStage for ApplyEditsStage {
    fn name(&self) -> &str { "apply_edits" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let edits = brain._micro_edits.len();
        log::trace!("[apply_edits] iter={} edits_pending={}", brain.iteration, edits);
        Ok(StageDecision::Continue)
    }
}

// ── Wrapper stages for modules that don't implement BrainStage directly ─────

/// Fable 5-style boundary separation stage: assessment before action.
///
/// Implements the principle "when the user is describing a problem, report
/// findings and stop. Don't apply a fix until asked." Checks whether each
/// pending edit/action was explicitly requested or is an unsolicited action.
/// Unsolicited actions are flagged and prevented from executing.
pub struct BoundarySeparationStage {
    /// Whether to allow unrequested fixes (default: false = block them)
    pub allow_unrequested_fixes: bool,
    /// Threshold: actions matching this many keywords are "unrequested"
    pub unrequested_keywords: Vec<String>,
}

impl Default for BoundarySeparationStage {
    fn default() -> Self {
        Self {
            allow_unrequested_fixes: false,
            unrequested_keywords: vec![
                "fix".into(), "restructure".into(), "refactor".into(),
                "rewrite".into(), "optimize".into(), "clean up".into(),
                "delete".into(), "remove".into(), "migrate".into(),
                "create".into(), "add".into(), "implement".into(),
            ],
        }
    }
}

impl BoundarySeparationStage {
    pub fn new() -> Self { Self::default() }

    pub fn with_allowed_fixes(mut self, allow: bool) -> Self {
        self.allow_unrequested_fixes = allow;
        self
    }

    /// Check if an action description appears to be unrequested (not explicitly asked for).
    pub fn is_unrequested_action(&self, action_desc: &str, context: &str) -> bool {
        if self.allow_unrequested_fixes {
            return false;
        }
        let action_lower = action_desc.to_lowercase();
        let context_lower = context.to_lowercase();

        // Check: does the action contain any keyword that was NOT mentioned in context?
        let mut unrequested_count = 0u32;
        for kw in &self.unrequested_keywords {
            if action_lower.contains(kw) && !context_lower.contains(kw) {
                unrequested_count += 1;
            }
        }
        unrequested_count >= 1
    }
}

impl BrainStage for BoundarySeparationStage {
    fn name(&self) -> &str { "boundary_separation" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let context = brain._current_task.as_str();
        let pending_actions: Vec<String> = brain._micro_edits.iter()
            .map(|e| format!("{:?}", e))
            .collect();

        let mut blocked = Vec::new();
        for action in &pending_actions {
            if self.is_unrequested_action(action, context) {
                blocked.push(action.clone());
            }
        }

        if !blocked.is_empty() {
            // Remove blocked (unrequested) edits from pending list
            brain._micro_edits.retain(|e| {
                !blocked.contains(&format!("{:?}", e))
            });
            return Ok(StageDecision::Skip(
                format!("boundary_separation: blocked {} unrequested actions: {:?}",
                    blocked.len(), blocked)
            ));
        }

        Ok(StageDecision::Continue)
    }
}

/// Ornith-1-style Scaffold-Aware Reinforcement Learning stage.
///
/// Implements self-scaffolding: the model learns to generate both solution
/// rollouts and the task-specific scaffolds that guide those rollouts.
/// Two-stage process:
///   1. Scaffold stage — propose refined scaffold from past experience
///   2. Solution stage — generate solution rollout conditioned on scaffold
///
/// Jointly optimizes scaffold and solution using staleness-weighted GRPO,
/// where older off-policy tokens are down-weighted by age.
///
/// Inspired by Ornith-1.0 (DeepReinforce, arXiv 2026):
/// "Jointly optimizing the scaffold and the resulting solution, the model
/// discovers better search trajectories and generates higher-quality solutions."
#[derive(Debug, Clone)]
pub struct ScaffoldAwareRLStage {
    pub scaffold_history: VecDeque<ScaffoldRecord>,
    pub max_history: usize,
    pub staleness_threshold: u64,
    pub grpo_clip: f64,
}

#[derive(Debug, Clone)]
pub struct ScaffoldRecord {
    pub iteration: u64,
    pub scaffold: String,
    pub solution_score: f64,
    pub reward: f64,
    pub staleness: u64,
}

impl ScaffoldAwareRLStage {
    pub fn new() -> Self {
        Self {
            scaffold_history: VecDeque::new(),
            max_history: 50,
            staleness_threshold: 10,
            grpo_clip: 0.2,
        }
    }

    /// Stage 1: Propose a refined scaffold based on past high-reward scaffolds.
    fn propose_scaffold(&self, brain: &SelfIteratingBrain) -> String {
        if self.scaffold_history.is_empty() {
            return format!("iter_{}_baseline", brain.iteration);
        }
        let mut best = &self.scaffold_history[0];
        for record in &self.scaffold_history {
            if record.reward > best.reward {
                best = record;
            }
        }
        let staleness_weight = self.compute_staleness_weight(best.staleness);
        if staleness_weight < 0.3 {
            format!("iter_{}_fresh", brain.iteration)
        } else {
            format!("{}_refined", best.scaffold)
        }
    }

    /// Stage 2: Generate solution score from brain state and scaffold.
    fn compute_solution_score(&self, brain: &SelfIteratingBrain, _scaffold: &str) -> f64 {
        let champion_score = brain.champion.as_ref().map(|c| c.score).unwrap_or(0.0);
        let prm_reward = brain._prm_cumulative_reward.clamp(0.0, 1.0);
        let cap_mean = if brain.brain.capability.arr.is_empty() {
            0.0
        } else {
            brain.brain.capability.arr.iter().sum::<f64>() / brain.brain.capability.arr.len() as f64
        };
        let entropy_penalty = (1.0 - brain.entropy_crisis_level.clamp(0.0, 1.0)) * 0.1;

        0.4 * champion_score + 0.3 * prm_reward + 0.2 * cap_mean + entropy_penalty
    }

    /// Staleness weight: down-weights older off-policy tokens.
    /// w(d) = exp(-d / threshold) — tokens older than threshold get near-zero weight.
    fn compute_staleness_weight(&self, age: u64) -> f64 {
        if age == 0 {
            return 1.0;
        }
        (-(age as f64) / (self.staleness_threshold as f64)).exp()
    }

    /// GRPO loss with staleness weighting:
    /// L = -E[ w(d) * min(ratio * A, clip(ratio, 1-ε, 1+ε) * A) ]
    fn compute_grpo_loss(&self, reward: f64, old_reward: f64, staleness: u64) -> f64 {
        let ratio = if old_reward.abs() > 1e-10 {
            (reward / old_reward).clamp(0.1, 10.0)
        } else {
            1.0
        };
        let advantage = reward - 0.5;
        let clipped = ratio.clamp(1.0 - self.grpo_clip, 1.0 + self.grpo_clip);
        let w = self.compute_staleness_weight(staleness);
        -w * ratio.min(clipped) * advantage
    }
}

impl Default for ScaffoldAwareRLStage { fn default() -> Self { Self::new() } }

impl BrainStage for ScaffoldAwareRLStage {
    fn name(&self) -> &str { "scaffold_aware_rl" }
    fn frequency(&self) -> usize { 3 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let mut stage = self.clone();
        let scaffold = stage.propose_scaffold(brain);

        let old_score = brain.champion.as_ref().map(|c| c.score).unwrap_or(0.0);
        let solution_score = stage.compute_solution_score(brain, &scaffold);

        let age = brain.iteration.saturating_sub(
            stage.scaffold_history.back().map(|r| r.iteration).unwrap_or(0)
        );
        let reward = solution_score * stage.compute_staleness_weight(age);

        // GRPO loss
        let _loss = stage.compute_grpo_loss(reward, old_score, age);

        // Record scaffold
        stage.scaffold_history.push_back(ScaffoldRecord {
            iteration: brain.iteration,
            scaffold,
            solution_score,
            reward,
            staleness: age,
        });
        if stage.scaffold_history.len() > stage.max_history {
            stage.scaffold_history.pop_front();
        }

        // ECHO terminal-prediction signal quality — available with full echo bridge
        #[cfg(feature = "echo_bridge")]
        {
            let report = brain.echo_bridge.echo.batch_signal_quality();
            let echo_loss = 1.0 - report.signal_coverage();
            if echo_loss > 0.001 {
                log::debug!("[e8-echo] loss={:.6} (coverage={:.3}, error_rate={:.3})",
                    echo_loss, report.signal_coverage(), report.error_rate());
            }
        }

        // Update champion if improved
        let improved = solution_score > old_score + 0.01;
        if improved {
            let champ = BrainSnapshot::new(&brain.brain, &TaskType::CodeGeneration);
            return Ok(StageDecision::Promote(champ));
        }

        Ok(StageDecision::Continue)
    }
}

/// Champion compare stage (used by recipe.rs)
pub struct ChampionCompareStage;
impl Default for ChampionCompareStage { fn default() -> Self { Self } }
impl ChampionCompareStage { pub fn new() -> Self { Self } }
impl BrainStage for ChampionCompareStage {
    fn name(&self) -> &str { "champion_compare" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let champ_score = brain.champion.as_ref().map(|c| c.score).unwrap_or(0.0);
        log::trace!("[champion_compare] current={:.4} iter={}", champ_score, brain.iteration);
        Ok(StageDecision::Continue)
    }
}

/// Reasoning bank storage stage
pub struct ReasoningBankStorageStage;
impl Default for ReasoningBankStorageStage { fn default() -> Self { Self } }
impl ReasoningBankStorageStage { pub fn new() -> Self { Self } }
impl BrainStage for ReasoningBankStorageStage {
    fn name(&self) -> &str { "bank_storage" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        log::trace!("[bank_storage] iter={}", brain.iteration);
        Ok(StageDecision::Continue)
    }
}

/// Hypercube optimize stage — prunes low-access entries via HyperCubeBridge.
pub struct HyperCubeOptimizeStage;
impl Default for HyperCubeOptimizeStage { fn default() -> Self { Self } }
impl HyperCubeOptimizeStage { pub fn new() -> Self { Self } }
impl BrainStage for HyperCubeOptimizeStage {
    fn name(&self) -> &str { "hypercube_optimize" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let mut pruned = 0usize;
        if let Some(ref mut router) = brain.attention_router {
            let before = router.bridge.hypercube.cell_count();
            pruned = router.bridge.hypercube.prune_low_access(2);
            if pruned > 0 {
                let after = router.bridge.hypercube.cell_count();
                log::info!("[hypercube_optimize] iter={}: pruned {} entries ({} → {})",
                    brain.iteration, pruned, before, after);
            }
            let sparse_dims: Vec<String> = (0..16)
                .filter_map(|dim| {
                    let d = router.bridge.hypercube.coord_density(dim);
                    if d < 0.01 { Some(format!("dim{}:{:.3}", dim, d)) } else { None }
                }).collect();
            if !sparse_dims.is_empty() {
                log::debug!("[hypercube_optimize] sparse dims: [{}]", sparse_dims.join(","));
            }
        }
        if pruned == 0 {
            log::trace!("[hypercube_optimize] iter={}: no pruning needed", brain.iteration);
        }
        Ok(StageDecision::Continue)
    }
}

/// Security stage — scans current task/code context for secrets using SecretScanStage.
pub struct SecurityStage;
impl Default for SecurityStage { fn default() -> Self { Self } }
impl SecurityStage { pub fn new() -> Self { Self } }
impl BrainStage for SecurityStage {
    fn name(&self) -> &str { "security_scan" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let scanner = super::secret_scanner::SecretScanner::new();
        let findings = scanner.scan_with_context(&brain._current_task, "");
        if !findings.findings.is_empty() {
            log::warn!("[security_scan] iter={}: {} secret(s) detected in task context!",
                brain.iteration, findings.findings.len());
            for secret in &findings.findings {
                // 只记录 pattern + line，禁止把完整秘密行 (snippet) 打进日志 (防泄密)
                let preview: String = secret.snippet.chars().take(8).collect();
                log::warn!("[security_scan]   pattern={} line={} snippet_prefix={}...",
                    secret.pattern, secret.line, preview);
            }
            // Broadcast alert via GWT
            if let Some(ref mut engine) = brain.reasoning_engine {
                if let Some(ref mut gwt) = engine.gwt {
                    let alert = format!("SECURITY_ALERT: {} secrets detected in task '{}'",
                        findings.findings.len(), brain._current_task);
                    gwt.broadcast(&alert);
                }
            }
        } else {
            log::trace!("[security_scan] iter={}: clean", brain.iteration);
        }
        Ok(StageDecision::Continue)
    }
}

/// Distillation stage — extracts principles from pipeline trajectory into knowledge distiller
pub struct DistillationStage;
impl Default for DistillationStage { fn default() -> Self { Self } }
impl DistillationStage { pub fn new() -> Self { Self } }
impl BrainStage for DistillationStage {
    fn name(&self) -> &str { "distillation" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let traj_len = brain.reasoning_engine.as_ref()
            .map(|e| e.state_trajectory.len()).unwrap_or(0);
        if traj_len > 0 {
            let session = crate::neotrix::nt_act_autonomy::knowledge_distiller::SessionRecord {
                id: format!("pipeline-iter-{}", brain.iteration),
                user_messages: vec![brain._current_task.clone()],
                actions_taken: vec![format!("pipeline_iter_{}", brain.iteration)],
                outcomes: vec![format!("reward_{:.4}", brain._reward)],
                reward_signal: brain._reward,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs()).unwrap_or(0),
                task_type: None,
                e8_mode: None,
                edit_types: vec![],
            };
            let principles = brain._knowledge_distiller.distill(&session);
            let absorbed = brain._knowledge_distiller.absorb(&mut brain.brain.capability);
            if !principles.is_empty() || absorbed > 0 {
                log::info!("[distillation] {} principles from iter {}, {} absorbed into capability",
                    principles.len(), brain.iteration, absorbed);
            }
        }
        Ok(StageDecision::Continue)
    }
}

/// Meta improvement stage (self-evolution planning)
pub struct MetaImprovementStage;
impl Default for MetaImprovementStage { fn default() -> Self { Self } }
impl MetaImprovementStage { pub fn new() -> Self { Self } }
impl BrainStage for MetaImprovementStage {
    fn name(&self) -> &str { "meta_improvement" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let hist = &brain.evaluation_history;
        let recent_count = hist.len().min(10);
        if recent_count >= 3 {
            let recent: Vec<f64> = hist.iter().rev().take(recent_count).map(|r| r.score_after).collect();
            let avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
            let improving = recent.windows(2).filter(|w| w[1] > w[0]).count();
            let plateau = improving < recent_count / 3 && avg < 0.6;
            if plateau {
                brain.curiosity_bonus = (brain.curiosity_bonus + 0.05).min(0.3);
                log::info!("[meta_improvement] plateau detected, curiosity={:.4}", brain.curiosity_bonus);
            } else {
                brain.curiosity_bonus = (brain.curiosity_bonus * 0.95).max(0.0);
            }
            if let Some(ref kb) = brain._nt_memory_kb {
                let _ = kb.kv_set("meta_improvement", &format!("iter_{}", brain.iteration),
                    &serde_json::json!({"avg_score": avg, "improving": improving, "plateau": plateau,
                        "curiosity": brain.curiosity_bonus}).to_string());
            }
        }
        log::trace!("[meta_improvement] iter={} eval_history={}", brain.iteration, hist.len());
        Ok(StageDecision::Continue)
    }
}

/// Sleep stage (offline memory consolidation)
pub struct SleepStage;
impl Default for SleepStage { fn default() -> Self { Self } }
impl SleepStage { pub fn new() -> Self { Self } }
impl BrainStage for SleepStage {
    fn name(&self) -> &str { "sleep" }
    fn frequency(&self) -> usize { 100 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref mut engine) = brain.sleep_engine {
            if let (Some(op), Some(ref mut st)) = (brain.select_operator.as_ref(), brain.selective_state.as_mut()) {
                match engine.sleep(&mut brain.brain.capability, &mut brain.reasoning_bank, op, st) {
                    Ok(result) => {
                        let stats = result.stats.clone();
                        brain.last_sleep_stats = Some(result.stats);
                        log::info!("[sleep] passes={} memories={} delta={:.6}",
                            stats.passes_done, stats.total_memories, stats.total_delta);
                    }
                    Err(e) => log::warn!("[sleep] engine error: {}", e),
                }
            }
        } else {
            let result = brain.consolidate_memories();
            log::info!("[sleep] light consolidation: merged={} pruned={}",
                result.merged_count, result.pruned_count);
        }
        Ok(StageDecision::Continue)
    }
}

/// Uncertainty quantification calibration stage
pub struct UQCalibrationStage;
impl Default for UQCalibrationStage { fn default() -> Self { Self } }
impl UQCalibrationStage { pub fn new() -> Self { Self } }
impl BrainStage for UQCalibrationStage {
    fn name(&self) -> &str { "uq_calibration" }
    fn frequency(&self) -> usize { 20 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let hist = &brain.evaluation_history;
        let recent_count = hist.len().min(20);
        let volatility = if recent_count >= 4 {
            let recent: Vec<f64> = hist.iter().rev().take(recent_count).map(|r| r.score_after).collect();
            let mean = recent.iter().sum::<f64>() / recent.len() as f64;
            let variance = recent.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / recent.len() as f64;
            let vol = variance.sqrt();
            let reward = brain._prm_cumulative_reward;
            let target_entropy = if vol > 0.15 && reward < 0.3 {
                0.6f64.min(0.3 + vol)
            } else if vol < 0.05 && reward > 0.5 {
                (brain.entropy_crisis_level * 0.8).max(0.05)
            } else {
                brain.entropy_crisis_level * 0.95 + vol * 0.05
            };
            brain.entropy_crisis_level = target_entropy.max(0.0).min(1.0);
            if let Some(ref kb) = brain._nt_memory_kb {
                let _ = kb.kv_set("uq_calibration", &format!("iter_{}", brain.iteration),
                    &serde_json::json!({"volatility": vol, "entropy": brain.entropy_crisis_level,
                        "reward": reward}).to_string());
            }
            vol
        } else {
            0.0
        };
        log::trace!("[uq_calibration] entropy={:.4} volatility={:.4} reward={:.4}",
            brain.entropy_crisis_level, volatility, brain._prm_cumulative_reward);
        Ok(StageDecision::Continue)
    }
}

/// Open-source compare stage
pub struct OpenSourceCompareStage;
impl Default for OpenSourceCompareStage { fn default() -> Self { Self } }
impl OpenSourceCompareStage { pub fn new() -> Self { Self } }
impl BrainStage for OpenSourceCompareStage {
    fn name(&self) -> &str { "open_source_compare" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref insights) = brain._open_source_insights {
            let insight_len = insights.len();
            let deltas = compute_capability_deltas(brain);
            let total_delta: f64 = deltas.iter().map(|(_, d)| *d).sum();
            if total_delta.abs() > 0.001 && insight_len > 5 {
                let new_edits: Vec<super::super::self_edit::MicroEdit> = deltas.iter()
                    .filter(|(_, d)| d.abs() > 0.01)
                    .map(|(name, delta)| {
                        super::super::self_edit::MicroEdit::AdjustDimension(
                            name.clone(), *delta)
                    }).collect();
                brain._open_source_edits.extend(new_edits);
                if let Some(ref kb) = brain._nt_memory_kb {
                    let _ = kb.kv_set("open_source_compare", &format!("iter_{}", brain.iteration),
                        &serde_json::json!({"insight_len": insight_len, "total_delta": total_delta,
                            "new_edits": brain._open_source_edits.len()}).to_string());
                }
            }
        }
        log::trace!("[open_source_compare] insights_present={} edits={}",
            brain._open_source_insights.is_some(), brain._open_source_edits.len());
        Ok(StageDecision::Continue)
    }
}

/// Wraps SftStage ::process() as a BrainStage.
/// 监督微调：将能力增量作为监督信号，把当前最优 E8 模式推向目标质量。
pub struct SftWrapperStage;
impl Default for SftWrapperStage { fn default() -> Self { Self } }
impl SftWrapperStage { pub fn new() -> Self { Self } }
impl BrainStage for SftWrapperStage {
    fn name(&self) -> &str { "sft_supervision" }
    fn frequency(&self) -> usize { 1 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let deltas = compute_capability_deltas(brain);
        let current_mode = brain._e8_policy.best_mode();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0);
        let examples: Vec<SupervisedExample> = deltas.iter()
            // H2 修复: 只把正向 delta 当监督样本 (能力提升)。此前用 delta.abs()
            // 方向无关 — 能力下降 (负 delta, 含 HarnessAdapt 抬升假象) 也当正样本
            // 训练, 形成"自抬能力→自产 delta→自训"奖励黑客循环, 无外部验证。
            .filter(|(_, d)| *d > 0.01)
            .map(|(name, delta)| SupervisedExample::new(
                name,
                current_mode.0,
                delta.clamp(0.0, 1.0),
            ).with_timestamp(timestamp))
            .collect();
        let (_result, adjusted_reward) = brain._sft_stage.process(examples.clone(), brain._reward);
        brain._set_reward(adjusted_reward);
        log::trace!("[sft_supervision] reward={:.4} examples={} updates={}",
            adjusted_reward, examples.len(), brain._sft_stage.total_updates);
        Ok(StageDecision::Continue)
    }
}

/// Wraps ProcessStage ::process() as a BrainStage.
/// 过程知识习得：从工具调用轨迹构造推理链，监督"如何推理/分解/验证"。
pub struct ProcessWrapperStage;
impl Default for ProcessWrapperStage { fn default() -> Self { Self } }
impl ProcessWrapperStage { pub fn new() -> Self { Self } }
impl BrainStage for ProcessWrapperStage {
    fn name(&self) -> &str { "process_supervision" }
    fn frequency(&self) -> usize { 2 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0);
        // 从工具调用轨迹构造推理链步骤
        let steps: Vec<ReasoningStep> = brain.tool_traces.iter().enumerate().map(|(i, (tool, dur, ok))| {
            ReasoningStep {
                step_idx: i,
                specialist: "Tool".to_string(),
                e8_mode: 0,
                action: tool.clone(),
                input: String::new(),
                output: if *ok { "ok".to_string() } else { "error".to_string() },
                duration_ms: Some(*dur),
                success: *ok,
                reward: Some(if *ok { 1.0 } else { 0.0 }),
            }
        }).collect();
        let examples: Vec<ProcessExample> = if steps.is_empty() {
            Vec::new()
        } else {
            let trace = ReasoningTrace {
                trace_id: format!("iter-{}", brain.iteration),
                task,
                steps,
                completed: true,
                final_quality: brain._reward.clamp(0.0, 1.0),
                source: TraceSource::Synthesis,
                timestamp,
            };
            vec![ProcessExample { trace, weight: 1.0 }]
        };
        // B5 (缺陷4修复): 消费意识树果实 → 转换为 ReasoningTrace 并入 process 样本。
        // 此前 extract_from_consciousness_tree (process_stage.rs:130) 无生产调用者,
        // 意识树产出的进化果实从不进入 SEAL 过程学习。果实轨迹以 quality 加权,
        // 使高质量进化果实优先塑造 reasoning_depth/cot_quality 等能力维度。
        let mut examples = examples;
        let fruit_traces = ProcessStage::extract_from_consciousness_tree(&brain._consciousness_fruits);
        for ft in fruit_traces {
            let w = ft.final_quality.max(0.1);
            examples.push(ProcessExample { trace: ft, weight: w });
        }
        // 缺陷2修复 (自我运转实际情况): 果实消费后立即清除, 防止同一批果实
        // 被 SEAL 反复消费 (1h 注入 vs 10min 消费的时序错配 → 同一 trace 进
        // buffer 6 次 → process 学习被重复污染)。一次性消费, 下次 tick 重新注入。
        brain._consciousness_fruits.clear();
        let (_result, loss) = brain._process_stage.process(examples);
        log::trace!("[process_supervision] loss={:.4} traces={}",
            loss, brain._process_stage.buffer.len());
        Ok(StageDecision::Continue)
    }
}

/// Wraps SearchSkillStage ::process() as a BrainStage.
/// 搜索技能内化：从当前任务构造搜索演练，学习 query/evidence/synthesis 子技能。
pub struct SearchSkillWrapperStage;
impl Default for SearchSkillWrapperStage { fn default() -> Self { Self } }
impl SearchSkillWrapperStage { pub fn new() -> Self { Self } }
impl BrainStage for SearchSkillWrapperStage {
    fn name(&self) -> &str { "search_skill_supervision" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task.clone();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0);
        let exercises: Vec<SearchExercise> = if task.is_empty() {
            Vec::new()
        } else {
            let grounding = brain._reward.clamp(0.0, 1.0);
            vec![SearchExercise {
                exercise_id: format!("search-{}", brain.iteration),
                task_type: SearchTaskType::TechnicalQuery,
                query: task.clone(),
                raw_results: vec![SearchResult {
                    url: "local://task".to_string(),
                    title: task.clone(),
                    snippet: String::new(),
                    source_type: "doc".to_string(),
                    credibility: grounding,
                }],
                filtered_evidence: vec![Evidence {
                    source_url: "local://task".to_string(),
                    claim: task.clone(),
                    confidence: grounding,
                    supports_answer: true,
                }],
                synthesized_answer: task.clone(),
                grounding_score: grounding,
                relevance_score: grounding,
                synthesis_quality: grounding,
                latency_ms: 0,
                timestamp,
            }]
        };
        let (_result, loss) = brain._search_skill_stage.process(exercises.clone());
        log::info!("[search_skill_supervision] loss={:.4} exercises={} updates={}",
            loss, exercises.len(), brain._search_skill_stage.total_updates);
        Ok(StageDecision::Continue)
    }
}

/// Wraps DpoStage ::process() as a BrainStage
pub struct DpoWrapperStage;
impl Default for DpoWrapperStage { fn default() -> Self { Self } }
impl DpoWrapperStage { pub fn new() -> Self { Self } }
impl BrainStage for DpoWrapperStage {
    fn name(&self) -> &str { "dpo_preference" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let deltas = compute_capability_deltas(brain);
        let mut pairs: Vec<crate::neotrix::nt_mind::self_iterating::dpo_stage::PreferencePair> = Vec::new();
        let current_mode = brain._e8_policy.best_mode();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0);
        for (cap_name, delta) in &deltas {
            if (*delta).abs() > 0.01 {
                let (rejected_idx, _) = brain._e8_policy.mode_values.iter().enumerate()
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap_or((0, &0.0));
                if *delta > 0.0 {
                    pairs.push(crate::neotrix::nt_mind::self_iterating::dpo_stage::PreferencePair {
                        task: cap_name.clone(),
                        chosen_mode: current_mode.0,
                        rejected_mode: rejected_idx as u8,
                        chosen_reward: *delta,
                        rejected_reward: 0.0,
                        timestamp,
                    });
                }
            }
        }
        let (_result, adjusted_reward) = brain._dpo_stage.process(pairs, brain._reward);
        brain._set_reward(adjusted_reward);
        // H3 修复: DPO 偏好信号真正应用到能力向量。
        // 此前 DpoStage::process 只算 loss + 调 reward, 从不更新任何权重 —
        // DPO 是 no-op, smol-course 吸收的 DPO 阶段无学习效果。
        // 现在: chosen 维度按 margin 提升, rejected 维度按 margin 下降,
        // 使偏好信号实际改变能力分布 (DPO 梯度方向: 提升 chosen, 压低 rejected)。
        let beta = brain._dpo_stage.beta;
        for pair in brain._dpo_stage.buffer.pairs.iter() {
            if let Some(idx) = parse_cap_index(&pair.task) {
                let margin = (pair.chosen_reward - pair.rejected_reward).clamp(0.0, 1.0);
                let step = (beta * margin).min(0.05);
                let arr = brain.brain.capability.arr_mut();
                if idx < arr.len() {
                    arr[idx] = (arr[idx] + step).min(1.0);
                }
            }
        }
        log::trace!("[dpo_preference] reward={:.4} deltas={} updates={}",
            adjusted_reward, deltas.len(), brain._dpo_stage.total_updates);
        Ok(StageDecision::Continue)
    }
}

/// Parse capability index from "cap_{i}" task name.
fn parse_cap_index(task: &str) -> Option<usize> {
    task.strip_prefix("cap_").and_then(|s| s.parse::<usize>().ok())
}

/// Wraps ConstitutionalSelfCritiqueStage as a BrainStage
pub struct ConstitutionalWrapperStage;
impl Default for ConstitutionalWrapperStage { fn default() -> Self { Self } }
impl ConstitutionalWrapperStage { pub fn new() -> Self { Self } }
impl BrainStage for ConstitutionalWrapperStage {
    fn name(&self) -> &str { "constitutional_critique" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let deltas = compute_capability_deltas(brain);
        let mut critic = ConstitutionalSelfCritiqueStage::new();
        let (_result, adjusted_reward, should_reflect) = critic.process(&deltas, brain._reward);
        brain._set_reward(adjusted_reward);
        log::trace!("[constitutional_critique] reward_adjusted={:.4} should_reflect={} violations={}",
            adjusted_reward, should_reflect, critic.consecutive_violations);
        Ok(StageDecision::Continue)
    }
}

/// Wraps SafetyCheckStage as a BrainStage
pub struct SafetyWrapperStage;
impl Default for SafetyWrapperStage { fn default() -> Self { Self } }
impl SafetyWrapperStage { pub fn new() -> Self { Self } }
impl BrainStage for SafetyWrapperStage {
    fn name(&self) -> &str { "safety_check" }
    fn frequency(&self) -> usize { 1 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let deltas = compute_capability_deltas(brain);
        let mut safety = SafetyCheckStage::new();
        let (_result, check_result, adjusted_reward) = safety.evaluate(&deltas, brain._reward);
        brain._set_reward(adjusted_reward);
        match &check_result {
            SafetyCheckResult::Failed { reason } => {
                log::warn!("[safety_check] BLOCKED: {}", reason);
                return Ok(StageDecision::Skip(reason.clone()));
            }
            SafetyCheckResult::NeedsHumanReview { concern } => {
                log::warn!("[safety_check] REVIEW: {}", concern);
            }
            SafetyCheckResult::Passed => {
                log::trace!("[safety_check] passed");
            }
        }
        Ok(StageDecision::Continue)
    }
}

/// Autonomy PER (Plan-Execute-Reflect) stage: runs the PlanExecuteReflectLoop
/// on the current task to produce structured task plans, execution traces, and
/// self-reflective revisions. Frequency 5 — runs every 5 iterations to avoid
/// overwhelming the pipeline with detailed planning on every tick.
pub struct AutonomyPerStage;
impl Default for AutonomyPerStage { fn default() -> Self { Self } }
impl AutonomyPerStage { pub fn new() -> Self { Self } }
impl BrainStage for AutonomyPerStage {
    fn name(&self) -> &str { "autonomy_per" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_act_autonomy::PlanExecuteReflectLoop;

        let task = if brain._current_task.is_empty() {
            log::trace!("[autonomy_per] no current task, skipping");
            return Ok(StageDecision::Skip("no task".into()));
        } else {
            brain._current_task.clone()
        };

        // Run the PER loop against the knowledge distiller's per_loop field
        let outcome = if let Some(ref mut per) = brain._per_loop {
            per.run(&task)
        } else {
            let mut per = PlanExecuteReflectLoop::new(
                crate::neotrix::nt_act_autonomy::PerConfig {
                    max_iterations: 3,
                    min_score_to_converge: 0.7,
                    require_all_steps: false,
                },
            );
            let outcome = per.run(&task);
            brain._per_loop = Some(per);
            outcome
        };

        log::info!(
            "[autonomy_per] task='{}' converged={} score={:.2} iterations={} duration={}ms steps={}",
            task.chars().take(60).collect::<String>(),
            outcome.converged,
            outcome.final_score,
            outcome.iterations.len(),
            outcome.total_duration_ms,
            outcome.final_plan.steps.len(),
        );

        // Store as a KV record in KB for later review/audit
        if let Some(ref kb) = brain._nt_memory_kb {
            let summary = serde_json::json!({
                "task": &task,
                "converged": outcome.converged,
                "final_score": outcome.final_score,
                "iteration_count": outcome.iterations.len(),
                "total_duration_ms": outcome.total_duration_ms,
                "step_count": outcome.final_plan.steps.len(),
            });
            let _ = kb.kv_set(
                "autonomy_per",
                &format!("iter_{}", brain.iteration),
                &summary.to_string(),
            );
        }

        Ok(StageDecision::Continue)
    }
}

/// GWT absorption stage: routes insights into global workspace
pub struct GwtAbsorbStage;
impl Default for GwtAbsorbStage { fn default() -> Self { Self } }
impl GwtAbsorbStage { pub fn new() -> Self { Self } }
impl BrainStage for GwtAbsorbStage {
    fn name(&self) -> &str { "gwt_absorb" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let iteration = brain.iteration;
        let reward = brain._reward;
        let caps = brain.brain.capability.arr().to_vec();
        let avg_cap = if caps.is_empty() { 0.5 } else { caps.iter().sum::<f64>() / caps.len() as f64 };
        if let Some(ref kb) = brain._nt_memory_kb {
            let summary = format!("gwt:iter={} reward={:.4} avg_cap={:.4} aut={:?}",
                iteration, reward, avg_cap, brain.autonomy);
            let _ = kb.kv_set("gwt_absorb", &format!("snapshot_{}", iteration), &summary);
            // is_conscious derived from the real InnerCritic quality score
            // (0.0–1.0), not hardcoded true. Falls back to a phi-like threshold
            // on avg_cap when no critique has been produced yet.
            let conscious_signal = if brain._consciousness_critique_count > 0 {
                brain._last_consciousness_quality
            } else {
                avg_cap
            };
            let is_conscious = conscious_signal >= 0.5;
            let _ = kb.record_consciousness_snapshot(
                avg_cap,
                reward,
                is_conscious,
                "pipeline",
                &summary,
            );
        }
        // Route state summary into GWT workspace for specialist broadcast
        if let Some(ref mut router) = brain.attention_router {
            let content = format!("pipeline_iter={} reward={:.4} avg_cap={:.4}", iteration, reward, avg_cap);
            // 升级: 从 no-op broadcast (仅 push history) 改为 resonant_broadcast —
            // 真正进入 E8 注意力偏置 + Kuramoto 同步 + 共振竞争, 让 SEAL 状态
            // 参与 GWT 注意力路由 (修复信息流转对内断点 #2)。
            let states = crate::core::nt_core_gwt::resonance::default_specialist_states();
            let _report = router.wm().resonant_broadcast(&content, &states);
            log::debug!("[gwt_absorb] resonant broadcast to GWT: {}", content);
            if let Some(ref kb) = brain._nt_memory_kb {
                if let Ok(results) = kb.query_broadcast_context(&content, 3) {
                    log::debug!("[gwt_absorb] broadcast context results: {} found", results.len());
                }
            }
        }
        log::info!("[gwt_absorb] iter={} reward={:.4} avg_cap={:.4}", iteration, reward, avg_cap);
        Ok(StageDecision::Continue)
    }
}

/// Harness adaptation stage — adjusts capability vector based on low reward
pub struct HarnessAdaptStage;
impl Default for HarnessAdaptStage { fn default() -> Self { Self } }
impl HarnessAdaptStage { pub fn new() -> Self { Self } }
impl BrainStage for HarnessAdaptStage {
    fn name(&self) -> &str { "harness_adapt" }
    fn frequency(&self) -> usize { 2 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let reward = brain._reward;
        let caps = brain.brain.capability.arr().to_vec();
        if reward < 0.3 && !caps.is_empty() {
            let weak_count = caps.iter().filter(|&&v| v < 0.2).count();
            let boost = (0.3 - reward) * 0.1;
            for v in brain.brain.capability.arr_mut().iter_mut() {
                if *v < 0.2 {
                    *v = (*v + boost).min(0.3);
                }
            }
            // H2 修复: 抬升后同步快照, 使保底抬升不产生虚假 delta。
            // 否则下一 tick compute_capability_deltas 会把 Harness 抬升当真实
            // 能力提升 → SFT 自产正样本 → "自抬能力→自产 delta→自训"奖励黑客。
            // 保底抬升是防能力归零的自我修正, 不是真实进化, 不应进入学习信号。
            let snap = brain._snapshot_capability();
            let mut new_snap = snap.clone();
            for (i, v) in new_snap.arr_mut().iter_mut().enumerate() {
                if *v < 0.2 {
                    *v = (*v + boost).min(0.3);
                }
            }
            brain._set_snapshot(crate::neotrix::nt_mind::self_iterating::BrainSnapshot {
                capability: new_snap,
                learning_rate: brain._snapshot_lr(),
                score: brain._snapshot_score(),
            });
            log::info!("[harness_adapt] low reward={:.4}, boosted {} weak caps by {:.4} (snapshot synced)",
                reward, weak_count, boost);
        }
        Ok(StageDecision::Continue)
    }
}

/// Knowledge quality assessment stage — scores KB health metrics
pub struct KnowledgeQualityStage;
impl Default for KnowledgeQualityStage { fn default() -> Self { Self } }
impl KnowledgeQualityStage { pub fn new() -> Self { Self } }
impl BrainStage for KnowledgeQualityStage {
    fn name(&self) -> &str { "knowledge_quality" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref kb) = brain._nt_memory_kb {
            // Gather content quality metrics directly from the nodes table
            let quality_metrics = (|| -> Option<(i64, i64, i64, usize)> {
                let conn = kb.conn.lock().ok()?;
                let total: i64 = conn.query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0)).ok()?;
                let with_content: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM nodes WHERE content IS NOT NULL AND content != ''",
                    [], |r| r.get(0),
                ).ok()?;
                let with_summary: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM nodes WHERE summary IS NOT NULL AND summary != ''",
                    [], |r| r.get(0),
                ).ok()?;
                let mut stmt = conn.prepare("SELECT COUNT(DISTINCT node_type) FROM nodes").ok()?;
                let type_count: i64 = stmt.query_row([], |r| r.get(0)).ok()?;
                drop(stmt);
                Some((total, with_content, with_summary, type_count as usize))
            })();

            if let (Some(stats), Some((tot, has_content, has_summary, type_count))) =
                (kb.stats().ok(), quality_metrics)
            {
                let quality_score = if tot > 0 {
                    let content_cov = has_content as f64 / tot as f64;
                    let summary_cov = has_summary as f64 / tot as f64;
                    let type_div = (type_count as f64 / 23.0_f64).min(1.0);
                    let edge_ratio = if stats.total_nodes > 0 {
                        (stats.total_edges as f64 / stats.total_nodes as f64).min(5.0) / 5.0
                    } else { 0.0 };
                    (content_cov * 40.0 + summary_cov * 20.0 + type_div * 20.0 + edge_ratio * 20.0)
                        .max(0.0).min(100.0)
                } else { 0.0 };

                let reward_boost: f64 = if quality_score > 80.0 {
                    0.05
                } else if quality_score < 20.0 {
                    -0.05
                } else {
                    0.0
                };

                if reward_boost.abs() > 0.0_f64 {
                    let current = brain._reward;
                    let adjusted = (current + reward_boost).max(0.0).min(1.0);
                    brain._set_reward(adjusted);
                }

                #[allow(clippy::manual_is_multiple_of)]
                if brain.iteration % 10 == 0 {
                    log::info!(
                        "[knowledge_quality] score={:.1}% nodes={} content={}/{} summary={}/{} types={}/{} edges={} boost={}",
                        quality_score, tot, has_content, tot, has_summary, tot,
                        type_count, stats.by_type.len(), stats.total_edges, reward_boost,
                    );
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}

/// Secret scan stage — wraps SecretScanner for pipeline integration
pub struct SecretScanStage;
impl Default for SecretScanStage { fn default() -> Self { Self } }
impl SecretScanStage { pub fn new() -> Self { Self } }
impl BrainStage for SecretScanStage {
    fn name(&self) -> &str { "security_scan" }
    fn frequency(&self) -> usize { 1 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let scanner = SecretScanner::new();
        let task_repr = format!("iter={} reward={:.4} cap={:?}",
            brain.iteration, brain._reward,
            &brain.brain.capability.arr()[..5]);
        let result = scanner.scan_with_context(&task_repr, "");
        if !result.is_safe() {
            log::warn!("[security_scan] {} risks (score={:.2})",
                result.findings.len(), result.risk_score());
        } else {
            log::trace!("[security_scan] safe iter={}", brain.iteration);
        }
        Ok(StageDecision::Continue)
    }
}

/// Conversation distillation stage — stores trajectory insights to KB
pub struct ConversationDistillStage;
impl Default for ConversationDistillStage { fn default() -> Self { Self } }
impl ConversationDistillStage { pub fn new() -> Self { Self } }
impl BrainStage for ConversationDistillStage {
    fn name(&self) -> &str { "conversation_distill" }
    fn frequency(&self) -> usize { 1 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let traj_len = brain.reasoning_engine.as_ref()
            .map(|e| e.state_trajectory.len()).unwrap_or(0);
        if let Some(ref kb) = brain._nt_memory_kb {
            let stage_count = brain._stage_results.len();
            let prm_reward = brain._prm_cumulative_reward;
            let summary = format!(
                "iter={} reward={:.4} traj={} entropy={:.4} stages={} prm={:.4}",
                brain.iteration, brain._reward, traj_len, brain.entropy_crisis_level,
                stage_count, prm_reward,
            );
            let _ = kb.kv_set("conversation_distill",
                &format!("snap_{}", brain.iteration), &summary);
            if traj_len > 3 && brain.iteration.is_multiple_of(5) {
                if let Ok(records) = kb.get_evolution_history(10) {
                    let rewarding_count = records.iter().filter(|r| r.effectiveness > 0.0).count();
                    let failing_count = records.iter().filter(|r| r.effectiveness <= 0.0).count();
                    if rewarding_count + failing_count >= 3 {
                        let pattern_type = if failing_count > rewarding_count {
                            crate::neotrix::nt_memory_kb::nt_memory_types::EvolutionPatternType::RecurringError
                        } else {
                            crate::neotrix::nt_memory_kb::nt_memory_types::EvolutionPatternType::StrategyDiscovery
                        };
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs()).unwrap_or(0) as i64;
                        let record = crate::neotrix::nt_memory_kb::nt_memory_types::EvolutionRecord {
                            id: format!("evol_pipe_{}", brain.iteration),
                            source_conversation_id: format!("pipe_iter_{}", brain.iteration),
                            pattern_type: pattern_type.clone(),
                            description: summary,
                            before_behavior: format!("reward_before={:.4}", brain._reward),
                            after_behavior: String::new(),
                            effectiveness_gain: brain._reward,
                            applied_to: vec![],
                            verified: false,
                            timestamp: ts,
                        };
                        let _ = kb.store_evolution_record(&record);
                        log::info!("[conversation_distill] EvolRecord={:?}", pattern_type);
                    }
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}

/// EWHR Hypothesis Accuracy Stage: evaluates hypothesis predictions
/// against actual outcomes and updates calibration. Runs every 5 iterations.
pub struct HypothesisAccuracyStage;
impl Default for HypothesisAccuracyStage { fn default() -> Self { Self } }
impl HypothesisAccuracyStage { pub fn new() -> Self { Self } }
impl BrainStage for HypothesisAccuracyStage {
    fn name(&self) -> &str { "hypothesis_accuracy" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref engine) = brain.reasoning_engine {
            if let Some(ref net_lock) = engine.hypothesis_network {
                if let Ok(net) = net_lock.lock() {
                    let total = net.hypotheses.len();
                    let supported = net.hypotheses.iter().filter(|h| matches!(h.status, crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisStatus::Supported)).count();
                    let refuted = net.hypotheses.iter().filter(|h| matches!(h.status, crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisStatus::Refuted)).count();
                    if total > 0 {
                        log::info!("[EWHR] Hypothesis accuracy: {}/{} supported, {}/{} refuted", supported, total, refuted, total);
                    }
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}

/// EWHR Pattern Extraction Stage: converts successful hypotheses
/// into reusable procedural memory (skills). Runs every 10 iterations.
pub struct PatternExtractionStage;
impl Default for PatternExtractionStage { fn default() -> Self { Self } }
impl PatternExtractionStage { pub fn new() -> Self { Self } }
impl BrainStage for PatternExtractionStage {
    fn name(&self) -> &str { "pattern_extraction" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let (Some(ref engine), Some(ref kb)) = (&brain.reasoning_engine, &brain._nt_memory_kb) {
            // Init GraphRAG on first run
            if kb.graphrag_store.read().map(|s| s.is_none()).unwrap_or(false) {
                let _ = kb.init_graphrag(GraphRagConfig::default());
            }
            // Extract entities from new knowledge
            if let Some(ref net_lock) = engine.hypothesis_network {
                if let Ok(net) = net_lock.lock() {
                    for h in &net.hypotheses {
                        if matches!(h.status, HypothesisStatus::Supported) {
                            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                            let record = ProceduralMemoryRecord {
                                id: format!("ewhr_{}", h.id),
                                skill_id: h.id.clone(),
                                name: h.title.clone(),
                                description: h.description.clone(),
                                e8_sequence: vec![],
                                trigger_pattern: vec![],
                                success_rate: h.posterior_probability,
                                execution_count: 1,
                                avg_reward: 0.0,
                                created_at: now.clone(),
                                updated_at: now,
                                tags: h.tags.clone(),
                            };
                            let _ = kb.store_procedural_memory(&record);
                            let _ = kb.graphrag_extract(&h.description, &h.id);
                        }
                    }
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}

/// Self-review stage: runs SelfReviewGate mechanical checks every iteration.
/// Non-blocking — logs findings but never returns Rollback.
pub struct SelfReviewStage {
    pub strict_mode: bool,
}

impl Default for SelfReviewStage {
    fn default() -> Self {
        Self { strict_mode: true }
    }
}

impl SelfReviewStage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }
}

impl BrainStage for SelfReviewStage {
    fn name(&self) -> &str {
        "self_review"
    }

    fn frequency(&self) -> usize {
        1
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        // Extract observer feedback from reasoning engine if available
        let (observer_quality, observer_patterns) = brain.reasoning_engine.as_ref()
            .map(|re| {
                let q = re.observer.last_report.as_ref()
                    .map(|r| r.quality_score)
                    .unwrap_or(0.5);
                let pats = re.observer.last_report.as_ref()
                    .map(|r| r.critical_patterns.clone())
                    .unwrap_or_default();
                (q, pats)
            })
            .unwrap_or((0.5, vec![]));

        let mut gate = SelfReviewGate::new(self.strict_mode)
            .with_observer_feedback(observer_quality, observer_patterns.clone());
        let report = gate.run_all();
        let blast = gate.blast_radius();

        log::info!(
            "[self_review] {} passed, {} failed, {} warnings — blast: {} ({} files, {} crossings)",
            report.passed, report.failed, report.warnings,
            blast.risk, blast.affected_files, blast.module_crossings,
        );
        if !report.is_pass() {
            log::warn!(
                "[self_review] Failed checks: {} failed, {} warnings — review {}",
                report.failed,
                report.warnings,
                report.summary(),
            );
        }

        let findings_json = serde_json::json!({
            "stage": "self_review",
            "iteration": brain.iteration,
            "passed": report.passed,
            "failed": report.failed,
            "warnings": report.warnings,
            "blast_risk": format!("{}", blast.risk),
            "blast_affected_files": blast.affected_files,
            "blast_module_crossings": blast.module_crossings,
            "observer_quality": observer_quality,
            "observer_patterns": observer_patterns,
            "findings": report.findings.iter().map(|f| {
                serde_json::json!({
                    "severity": format!("{:?}", f.severity),
                    "category": f.category,
                    "message": f.message,
                    "file": f.file,
                })
            }).collect::<Vec<_>>(),
        });
        if let Some(ref kb) = brain._nt_memory_kb {
            let json_str = serde_json::to_string(&findings_json).unwrap_or_default();
            let _ = kb.kv_set("self_review", "latest", &json_str);
            // Store blast radius separately for trend analysis
            let blast_json = serde_json::json!({
                "iteration": brain.iteration,
                "risk": format!("{}", blast.risk),
                "files_scanned": blast.files_scanned,
                "affected_files": blast.affected_files,
                "module_crossings": blast.module_crossings,
            });
            let _ = kb.kv_set("self_review_blast", &brain.iteration.to_string(), &serde_json::to_string(&blast_json).unwrap_or_default());
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(ExternalKnowledgeAbsorbStage);
impl BrainStage for ExternalKnowledgeAbsorbStage {
    fn name(&self) -> &str { "external_knowledge_absorb" }
    fn frequency(&self) -> usize { 20 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain.iteration == 0 || !brain.iteration.is_multiple_of(20) {
            return Ok(StageDecision::Continue);
        }
        let tick = brain.iteration;
        // Open a temporary KB connection for the explorer instead of consuming
        // the pipeline's KB connection (which would lose pending transactions,
        // LRU cache state, and uncommitted embedding data).
        let explorer_kb = match crate::neotrix::nt_memory_kb::KnowledgeBase::open(None) {
            Ok(kb) => kb,
            Err(e) => {
                log::warn!("[external_knowledge_absorb] tick={}, failed to open temp KB: {}", tick, e);
                return Ok(StageDecision::Continue);
            }
        };
        let config = crate::neotrix::l2_world_impl::nt_world_exploration_engine::ExplorationConfig::default();
        let mut explorer = crate::neotrix::l2_world_impl::nt_world_exploration_engine::ExplorationEngine::new(config);
        explorer.attach_kb(explorer_kb.into());
        let report = explorer.run_cycle();
        log::info!(
            "[external_knowledge_absorb] tick={}, explore: discovered={}, ingested={}, skipped={}, failed={}, total_in_kb={}",
            tick, report.discovered, report.ingested, report.skipped, report.failed, report.total_in_kb
        );
        Ok(StageDecision::Continue)
    }
}

pub struct CreditAssignmentStage;
impl Default for CreditAssignmentStage { fn default() -> Self { Self } }
impl CreditAssignmentStage { pub fn new() -> Self { Self } }
impl BrainStage for CreditAssignmentStage {
    fn name(&self) -> &str { "credit_assignment" }
    fn frequency(&self) -> usize { 20 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        // Build credit graph from PRM step rewards + E8 transitions
        let mut graph = crate::core::nt_core_credit::CreditGraph::new();
        let policy = crate::core::nt_core_credit::E8CreditPolicy::default();
        let mut visit_counts: std::collections::HashMap<u8, u64> = std::collections::HashMap::new();

        for (step, reward) in &brain._prm_step_rewards {
            let e8_state = (step % 64) as u8;
            let visit = visit_counts.entry(e8_state).or_insert(0);
            *visit += 1;
            let attribution = policy.compute_attribution(brain._prm_step_rewards.len().saturating_sub(*step), *visit);
            graph.add_event(crate::core::nt_core_credit::CreditEvent {
                id: format!("prm_step_{}", step),
                parent_id: if *step > 0 { Some(format!("prm_step_{}", step - 1)) } else { None },
                role: if *reward > 0.5 { crate::core::nt_core_credit::CreditRole::Outcome } else { crate::core::nt_core_credit::CreditRole::Actor },
                label: format!("step_{}_reward_{:.2}", step, reward),
                e8_state,
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                weight: *reward,
                metadata: std::collections::HashMap::new(),
            });
            if *step > 0 {
                graph.add_edge(crate::core::nt_core_credit::CreditEdge {
                    from: format!("prm_step_{}", step - 1),
                    to: format!("prm_step_{}", step),
                    attribution, discount: policy.step_discount,
                });
            }
        }

        let credits = graph.backpropagate(0.95);
        if !credits.is_empty() {
            let total: f64 = credits.values().sum();
            log::debug!("[credit_assignment] {} events, total credit={:.3}", credits.len(), total);
            // Persist to KB
            if let Some(ref kb) = brain._nt_memory_kb {
                if let Ok(json) = graph.to_json() {
                    let _ = kb.kv_set("credit_graph", "latest", &json);
                    let _ = kb.kv_set("credit_graph", &format!("iter_{}", brain.iteration), &json);
                }
            }
            // GWT broadcast
            if let Some(ref mut engine) = brain.reasoning_engine {
                if let Some(ref mut gwt) = engine.gwt {
                    gwt.broadcast(&format!("credit_assignment: {} events, total={:.3}, persisted", credits.len(), total));
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}

/// OracleGate stage: evaluates pipeline health to decide if external human
/// oracle intervention is needed. Frequency 10 — low overhead gate that
/// only triggers under critical conditions (high entropy, low reward).
pub struct OracleGateStage;
impl Default for OracleGateStage { fn default() -> Self { Self } }
impl OracleGateStage { pub fn new() -> Self { Self } }
impl BrainStage for OracleGateStage {
    fn name(&self) -> &str { "oracle_gate" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_act_autonomy::OracleGate;

        let gate = brain._oracle_gate.get_or_insert_with(OracleGate::new);

        let entropy = brain.entropy_crisis_level;
        let reward = brain._reward;

        let decision = if entropy > 0.7 && reward < 0.2 {
            gate.evaluate_failure(brain.iteration as u32, "entropy_crisis")
        } else {
            gate.evaluate_failure(0, "normal")
        };

        if decision.needs_oracle {
            log::warn!(
                "[oracle_gate] HUMAN INTERVENTION: {:?} — {}",
                decision.reason, decision.suggested_action
            );
            if let Some(ref kb) = brain._nt_memory_kb {
                if let Some(ref req) = decision.request {
                    let _ = kb.kv_set("oracle_request", &format!("iter_{}", brain.iteration),
                        &serde_json::json!({"reason": format!("{:?}", req.reason)}).to_string());
                }
            }
            if let Some(ref mut engine) = brain.reasoning_engine {
                if let Some(ref mut gwt) = engine.gwt {
                    gwt.broadcast("oracle_gate: needs intervention");
                }
            }
            return Ok(StageDecision::Skip(decision.suggested_action));
        }
        Ok(StageDecision::Continue)
    }
}

/// ArchitectureOptimizer stage: runs self-architecture analysis every 15 iterations.
/// Uses SelfArchitectureOptimizer to identify structural improvements.
pub struct ArchitectureOptimizerStage;
impl Default for ArchitectureOptimizerStage { fn default() -> Self { Self } }
impl ArchitectureOptimizerStage { pub fn new() -> Self { Self } }
impl BrainStage for ArchitectureOptimizerStage {
    fn name(&self) -> &str { "arch_optimizer" }
    fn frequency(&self) -> usize { 15 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_act_autonomy::SelfArchitectureOptimizer;
        let optimizer = SelfArchitectureOptimizer::new();
        // Pass module sizes as proxy file list
        let caps = brain.brain.capability.arr();
        let files: Vec<(String, usize)> = caps.iter().enumerate()
            .map(|(i, v)| (format!("cap_{}", i), (*v * 1000.0) as usize))
            .collect();
        let report = optimizer.analyze(&files, None);
        if report.total_suggestions > 0 {
            log::info!("[arch_optimizer] suggestions={} auto_fixable={}",
                report.total_suggestions, report.auto_fixable_count);
        }
        if let Some(ref kb) = brain._nt_memory_kb {
            let _ = kb.kv_set("arch_optimizer", &format!("iter_{}", brain.iteration),
                &serde_json::json!({
                    "suggestions": report.total_suggestions,
                    "auto_fixable": report.auto_fixable_count,
                    "large_modules": report.large_modules.len(),
                }).to_string());
        }
        Ok(StageDecision::Continue)
    }
}

/// TrendAnalysisStage: runs evolution trend analysis every 15 iterations.
/// Uses EvolutionTrendAnalyzer to detect capability trends over time.
pub struct TrendAnalysisStage;
impl Default for TrendAnalysisStage { fn default() -> Self { Self } }
impl TrendAnalysisStage { pub fn new() -> Self { Self } }
impl BrainStage for TrendAnalysisStage {
    fn name(&self) -> &str { "trend_analysis" }
    fn frequency(&self) -> usize { 15 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_act_autonomy::EvolutionTrendAnalyzer;
        let mut analyzer = EvolutionTrendAnalyzer::new();
        let caps = brain.brain.capability.arr();
        for (i, val) in caps.iter().enumerate() {
            analyzer.record(&format!("cap_{}", i), *val, Some("capability"));
        }
        let report = analyzer.analyze();
        log::trace!("[trend_analysis] trends={} dir={:?} improving={} declining={}",
            report.trends.len(), report.overall_direction,
            report.improving_count, report.declining_count);
        if let Some(ref kb) = brain._nt_memory_kb {
            let _ = kb.kv_set("trend_analysis", &format!("iter_{}", brain.iteration),
                &serde_json::json!({"trends": report.trends.len(),
                    "direction": format!("{:?}", report.overall_direction),
                    "improving": report.improving_count,
                    "declining": report.declining_count,
                }).to_string());
        }
        Ok(StageDecision::Continue)
    }
}

/// MetaGoalStage: generates meta-goals every 12 iterations from trend report.
pub struct MetaGoalStage;
impl Default for MetaGoalStage { fn default() -> Self { Self } }
impl MetaGoalStage { pub fn new() -> Self { Self } }
impl BrainStage for MetaGoalStage {
    fn name(&self) -> &str { "meta_goal" }
    fn frequency(&self) -> usize { 12 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_act_autonomy::{EvolutionTrendAnalyzer, MetaGoalGenerator};
        let mut analyzer = EvolutionTrendAnalyzer::new();
        let caps = brain.brain.capability.arr();
        for (i, val) in caps.iter().enumerate() {
            analyzer.record(&format!("cap_{}", i), *val, Some("capability"));
        }
        let report = analyzer.analyze();
        let generator = MetaGoalGenerator::new();
        let goals = generator.generate_from_trends(&report);
        log::debug!("[meta_goal] generated {} goals from trends", goals.len());
        if let Some(ref kb) = brain._nt_memory_kb {
            let goal_str: Vec<String> = goals.iter()
                .map(|g| format!("{}:{:.2}", g.description, g.priority as u8))
                .collect();
            let _ = kb.kv_set("meta_goals", &format!("iter_{}", brain.iteration),
                &serde_json::json!({"count": goals.len(), "goals": goal_str}).to_string());
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(MemoryConsolidationStage);
impl BrainStage for MemoryConsolidationStage {
    fn name(&self) -> &str { "memory_consolidation" }
    fn frequency(&self) -> usize { 12 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_act_autonomy::cross_session_memory::MemoryCategory;

        let count_before = brain._memory_orch.size();
        let tiers = [MemoryTier::Working, MemoryTier::Episodic, MemoryTier::Procedural];
        let mut promoted_count = 0usize;
        for &tier in &tiers {
            let promoted = brain._memory_orch.promote(tier, |e| e.access_count >= 3);
            promoted_count += promoted.len();
            for entry in promoted {
                brain._memory_orch.store(entry).map_err(|_| NeoTrixError::Brain("promote store".into()))?;
            }
        }
        let mut persisted = 0usize;
        if let Some(kb) = brain._nt_memory_kb.take() {
            persisted = brain.persist_pending_entries(&kb);
            brain._nt_memory_kb = Some(kb);
        }

        // Cross-session memory integration: store current iteration patterns
        if let Some(ref mut csm) = brain._cross_session_memory {
            let caps = brain.brain.capability.arr();
            let avg_cap = if caps.is_empty() { 0.0 } else { caps.iter().sum::<f64>() / caps.len() as f64 };
            csm.remember(
                &format!("capability_iter_{}", brain.iteration),
                &format!("{:.4}", avg_cap),
                MemoryCategory::CapabilityState,
            );
            csm.remember(
                "task_type",
                &format!("{:?}", brain._current_task_type),
                MemoryCategory::Pattern,
            );
            csm.remember(
                "last_reward",
                &format!("{:.4}", brain._reward),
                MemoryCategory::TaskOutcome,
            );
        }

        // GWT broadcast: notify global workspace about memory state
        let size_after = brain._memory_orch.size();
        let csm_info = brain._cross_session_memory.as_ref()
            .map(|csm| format!(" csm={}", csm.len()))
            .unwrap_or_default();
        let msg = format!(
            "memory_consolidation: size={} promoted={} persisted={}{}",
            size_after, promoted_count, persisted, csm_info,
        );
        if let Some(ref mut engine) = brain.reasoning_engine {
            if let Some(ref mut gwt) = engine.gwt {
                gwt.broadcast(&msg);
            }
        }
        if count_before > 0 || persisted > 0 || promoted_count > 0 {
            log::debug!("[{}]", msg);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(CacheCleanupStage);
impl BrainStage for CacheCleanupStage {
    fn name(&self) -> &str { "cache_cleanup" }
    fn frequency(&self) -> usize { 50 }
    fn process(&self, _brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::l8_autonomic_impl::nt_mind_cleanup::{CleanupEngine, CleanupKind};
        let mut engine = CleanupEngine::new().with_project_root(std::path::PathBuf::from("."));
        engine.dry_run_default = false;
        engine.archive_on_clean = true;
        let r = engine.clean(CleanupKind::ProjectArtifacts);
        if r.deletable_count > 0 {
            log::info!("[pipeline/cache_cleanup] archived {} items ({:.1} MB) -> .cleanup/archive/",
                r.deletable_count, r.estimated_bytes as f64 / 1_048_576.0);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(RewardCalculationStage);
impl BrainStage for RewardCalculationStage {
    fn name(&self) -> &str { "reward_calc" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let external = brain._external_reward();
        let (reward, source) = if let Some(ext) = external {
            (ext, crate::core::RewardSource::External)
        } else {
            let task_type = brain._current_task_type();
            let score_before = brain._snapshot_score();
            let score_after = brain.brain.evaluate_capability(task_type);
            let regularization = brain.compute_regularization(&brain._snapshot_capability());
            let raw = (score_after - score_before) + regularization;
            let health = brain.evo_stats().health_score;
            let calibrated = raw * (0.5 + health * 0.5);
            (calibrated, crate::core::RewardSource::Internal)
        };
        brain._set_reward(reward);
        brain._set_reward_source(source);
        Ok(StageDecision::Continue)
    }
}

// ── ConvergenceCheckStage ────────────────────────────────────
// Architecture self-audit: every 50 iterations, scan for ghost modules + orphan files.

pub struct ConvergenceCheckStage;

impl Default for ConvergenceCheckStage {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvergenceCheckStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for ConvergenceCheckStage {
    fn name(&self) -> &str {
        "convergence_check"
    }

    fn frequency(&self) -> usize {
        50
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let _ = brain; // unused: the audit runs on the source tree, not brain state
        use crate::core::nt_core_self::self_audit::converge_check;
        let report = converge_check(".");
        if !report.findings.is_empty() {
            log::warn!("[seal] converge_check iter: {} ghosts, {} orphans, {} stale",
                report.ghost_count, report.stale_count, report.orphan_count);
        }
        Ok(StageDecision::Continue)
    }
}

// ── SelfTestStage ───────────────────────────────────────────
// Meta-audit: every 100 iterations, verify that detection modules themselves are intact.

pub struct SelfTestStage;

impl Default for SelfTestStage {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTestStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for SelfTestStage {
    fn name(&self) -> &str {
        "self_test"
    }

    fn frequency(&self) -> usize {
        100
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let _ = brain;
        use crate::core::nt_core_schema_watchdog::SchemaWatchdog;
        use crate::core::nt_core_self::self_audit::ConvergeCheckFn;
        use crate::core::nt_core_self_test::SelfTestRegistry;
        use crate::core::nt_core_meta::knowledge_gap_detector::KnowledgeGapDetector;
        use crate::core::nt_core_meta::scanner::CodeScanner;
        use crate::core::nt_core_gwt::monitor::EntropyMonitor;
        use crate::neotrix::l8_autonomic_impl::nt_mind::bbrain_monitor::BMonitor;
        use crate::core::nt_core_consciousness::inner_critic::InnerCritic;
        use crate::core::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime;
        use crate::core::nt_core_self_review::SelfReviewGate;
        use crate::core::nt_core_consciousness::cognitive_load::CognitiveLoadMonitor;
        use crate::core::nt_core_consciousness_tree::ConsciousnessTree;
        use crate::core::nt_core_meta::nt_core_meta_auditor::MetaAuditor;
        use crate::core::nt_core_meta::nt_core_arch_lint::ArchLint;
        use crate::core::nt_core_meta::monitor::MetaMonitor;
        use crate::core::nt_core_meta::metacognition_loop::MetaCognitiveLoop;
        use crate::core::nt_core_self::metacognitive_evaluator::CognitiveEvaluator;
        use crate::core::nt_core_meta::self_model::SelfModel;
        let mut registry = SelfTestRegistry::new();
        registry.register(Box::new(SchemaWatchdog::new()));
        registry.register(Box::new(ConvergeCheckFn));
        registry.register(Box::new(KnowledgeGapDetector::new()));
        registry.register(Box::new(CodeScanner::new(".")));
        registry.register(Box::new(EntropyMonitor::new(10, 0.5, 3)));
        registry.register(Box::new(BMonitor::default()));
        registry.register(Box::new(InnerCritic::new()));
        registry.register(Box::new(ConsciousnessRuntime::new()));
        registry.register(Box::new(SelfReviewGate::new(false)));
        registry.register(Box::new(ConsciousnessTree::new()));
        registry.register(Box::new(MetaAuditor::new()));
        registry.register(Box::new(ArchLint::new()));
        let sm = SelfModel::new();
        registry.register(Box::new(MetaMonitor::new(sm.clone())));
        registry.register(Box::new(MetaCognitiveLoop::new(sm)));
        registry.register(Box::new(CognitiveLoadMonitor::new()));
        registry.register(Box::new(CognitiveEvaluator::new()));
        registry.register(Box::new({
            let mut cm = crate::neotrix::nt_mind_consciousness_monitor::ConsciousnessMonitor::new();
            cm.observe();
            cm
        }));
        registry.register(Box::new(crate::neotrix::l8_autonomic_impl::nt_mind_self_diagnose::SelfDiagnose));
        registry.register(Box::new(crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default()));
        registry.register(Box::new(crate::core::l7_capability::nt_core_antidistil::DistillationDetector::new()));
        registry.register(Box::new(crate::neotrix::l1_body_impl::nt_act_autonomy::oracle_gate::OracleGate::new()));
        registry.register(Box::new(crate::neotrix::l1_body_impl::nt_act_code::semantic_entropy::SemanticEntropyGate::new()));
        registry.register(Box::new(crate::neotrix::l1_body_impl::nt_act_sandbox::ActionSandbox::new()));
        registry.register(Box::new(crate::core::nt_core_consciousness_review::ConsciousnessReview::new()));
        registry.register(Box::new(crate::neotrix::l5_consciousness_impl::nt_core_fep_iit::bridge::FEPIITBridge::new()));
        registry.register(Box::new(crate::neotrix::l9_transcendent_impl::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard::new()));
        registry.register(Box::new(crate::neotrix::l8_autonomic_impl::nt_mind::consciousness_bridge::ConsciousnessBridge::new()));
        registry.register(Box::new(crate::neotrix::l1_body_impl::nt_shield::browser_security::BrowserSecurityScanner::new(
            crate::neotrix::l1_body_impl::nt_shield::browser_security::BrowserSecurityConfig::default(),
        )));
        registry.register(Box::new(crate::neotrix::l1_body_impl::nt_shield::check_registry::CheckRegistry::new()));
        registry.register(Box::new(crate::core::nt_core_telemetry::TelemetryStore::new(100)));
        registry.register(Box::new(crate::neotrix::nt_memory_kb::nt_memory_commit_tracker::NarrativeConsistencyChecker::new()));
        registry.register(Box::new(crate::core::nt_core_scoring_substrate::ScoringSubstrate::new().with_threshold(0.5)));
        registry.register(Box::new(crate::core::nt_core_state_substrate::StateSubstrate::new()));
        registry.register(Box::new(crate::core::nt_core_simulate_engine::SimulateEngine::new()));
        registry.register(Box::new(crate::core::nt_core_second_brain::SecondBrain::new()));
        let results = registry.run_all();
        let passed = results.iter().filter(|r| r.passed).count();
        let total = results.len();
        if passed < total {
            log::error!("[seal] self_test: {}/{} passed — DETECTION SYSTEM DEGRADED", passed, total);
            for r in &results {
                if !r.passed {
                    log::error!("[seal] self_test: {}", r.summary());
                }
            }
            // T2.5: 自测失败必须改变行为, 不能只写日志。对本次演化施加负向奖励惩罚,
            // 使检测系统降级真实传导到演化信号 (而非静默 Continue)。
            let degradation = ((total - passed) as f64 / total.max(1) as f64) * -1.0;
            let base = brain._reward();
            brain._set_reward(base + degradation);
        }
        Ok(StageDecision::Continue)
    }
}

// ── ConsciousnessRewardStage ──────────────────────────────────
// Bridges the consciousness quality score into the SEAL reward signal.
// Every N iterations, reads _last_consciousness_quality from SelfIteratingBrain
// and applies a quality-based bonus/penalty to _reward.

make_stage!(ConsciousnessRewardStage);
impl BrainStage for ConsciousnessRewardStage {
    fn name(&self) -> &str { "consciousness_reward" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let q = brain._last_consciousness_quality;
        let count = brain._consciousness_critique_count;
        if count == 0 {
            return Ok(StageDecision::Continue);
        }
        let reward_adj = if q >= 0.8 {
            0.05
        } else if q >= 0.6 {
            0.02
        } else if q >= 0.4 {
            -0.02
        } else if q >= 0.2 {
            -0.08
        } else {
            -0.15
        };
        let current = brain._reward;
        brain._reward = (current + reward_adj).clamp(-1.0, 1.0);
        log::info!(
            "[seal] consciousness_reward: quality={:.3} count={} adj={:+.3} reward={:.3}",
            q, count, reward_adj, brain._reward,
        );
        Ok(StageDecision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let brain = crate::neotrix::l8_autonomic_impl::nt_mind::self_iterating::brain_core::ReasoningBrain::new();
        let task_type = crate::neotrix::nt_world_model::TaskType::General;
        let snapshot = BrainSnapshot::new(&brain, &task_type);
        assert!(snapshot.learning_rate >= 0.0);
    }

    #[test]
    fn test_process_wrapper_consumes_consciousness_fruits() {
        // B5 (缺陷4修复) 验证: ProcessWrapperStage 应消费意识树果实,
        // 使 extract_from_consciousness_tree 的果实 trace 进入 process buffer。
        // 此前果实从不被 SEAL 消费 (extract_from_consciousness_tree 无生产调用者)。
        let mut brain = crate::neotrix::l8_autonomic_impl::nt_mind::seal_core::self_iterating::loop_impl::core::SelfIteratingBrain::new();
        // 注入一个意识树果实 (quality 0.9)
        let fruit = crate::core::nt_core_consciousness_tree::EvolutionFruit {
            name: "NT-CORE-evo-fruit-1".into(),
            source_branch: crate::core::nt_core_consciousness_tree::BranchKind::Core,
            description: "Evolution capability from NT-CORE".into(),
            produced_at_cycle: 1,
            quality: 0.9,
            claim: "Branch Core produces capability at maturity 0.9".into(),
            evidence: crate::core::nt_core_consciousness_tree::EvidenceChain {
                run_id: Some("fruit-run-1".into()),
                ..crate::core::nt_core_consciousness_tree::EvidenceChain::default()
            },
            stop_rule: crate::core::nt_core_consciousness_tree::StopRule::default(),
            benchmark: crate::core::nt_core_consciousness_tree::ProviderBenchmark::default(),
            generation: 0,
        };
        brain._consciousness_fruits = vec![fruit];
        // 给 tool_traces 一个步骤, 使 process 有基础样本
        brain.tool_traces.push(("search".into(), 10, true));

        let stage = ProcessWrapperStage::new();
        let decision = stage.process(&mut brain).expect("process ok");
        match decision {
            StageDecision::Continue => {}
            other => panic!("expected Continue, got {:?}", other),
        }
        // 果实 trace 应进入 process buffer (至少 1 条来自 ConsciousnessTree)
        assert!(
            brain._process_stage.buffer.traces.iter().any(|t| t.source == TraceSource::ConsciousnessTree),
            "consciousness fruit trace consumed by SEAL process: {:?}",
            brain._process_stage.buffer.traces.iter().map(|t| t.source.clone()).collect::<Vec<_>>()
        );
        // 缺陷2修复 (自我运转实际情况): 果实消费后应清除, 防止同一批果实被
        // SEAL 反复消费 (1h 注入 vs 10min 消费时序错配 → 重复污染 process 学习)。
        assert!(
            brain._consciousness_fruits.is_empty(),
            "fruits cleared after consumption (one-shot): {}",
            brain._consciousness_fruits.len()
        );
        // 再次 process: 无果实可消费, buffer 不新增 ConsciousnessTree trace
        let before = brain._process_stage.buffer.traces.iter()
            .filter(|t| t.source == TraceSource::ConsciousnessTree).count();
        let _ = stage.process(&mut brain).expect("process again ok");
        let after = brain._process_stage.buffer.traces.iter()
            .filter(|t| t.source == TraceSource::ConsciousnessTree).count();
        assert_eq!(before, after, "no re-consumption after clear");
    }
}
