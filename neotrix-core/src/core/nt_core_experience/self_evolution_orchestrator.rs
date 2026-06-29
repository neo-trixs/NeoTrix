use crate::core::nt_core_consciousness::consciousness_cycle::CycleStep;

// ============================================================================
// Types
// ============================================================================

/// Configuration for the SelfEvolutionTaskOrchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Scan for wiring gaps every N cycles (default: 50).
    pub scan_interval_cycles: u64,
    /// Max wiring tasks to create per scan (default: 5).
    pub max_tasks_per_scan: usize,
    /// Auto-execute wiring (true) or just propose (false).
    pub auto_execute: bool,
    /// Max auto-execute attempts before requiring human review.
    pub max_auto_attempts: usize,
    /// Auto-fix mode: when true, detected wiring gaps are automatically patched
    /// by generating and applying WiringPatch code to consciousness_cycle.rs.
    pub auto_fix: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            scan_interval_cycles: 50,
            max_tasks_per_scan: 5,
            auto_execute: true,
            max_auto_attempts: 3,
            auto_fix: true,
        }
    }
}

/// Statistics tracked across orchestration cycles.
#[derive(Debug, Clone, Default)]
pub struct OrchestratorStats {
    pub gaps_discovered: u64,
    pub tasks_created: u64,
    pub wiring_attempted: u64,
    pub wiring_succeeded: u64,
    pub wiring_failed: u64,
    pub compilation_verified: u64,
}

/// Lifecycle status of a wiring task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WiringStatus {
    Discovered,
    TaskCreated,
    CodeGenerated,
    WiringApplied,
    CompilationFailed,
    Verified,
}

impl WiringStatus {
    pub fn name(&self) -> &'static str {
        match self {
            WiringStatus::Discovered => "discovered",
            WiringStatus::TaskCreated => "task_created",
            WiringStatus::CodeGenerated => "code_generated",
            WiringStatus::WiringApplied => "wiring_applied",
            WiringStatus::CompilationFailed => "compilation_failed",
            WiringStatus::Verified => "verified",
        }
    }
}

/// One wiring task: wire a module into a specific consciousness cycle step.
#[derive(Debug, Clone)]
pub struct WiringTask {
    pub id: u64,
    pub module_name: String,
    pub module_path: String,
    /// Which CycleStep to wire into (None = unknown, auto-detect).
    pub target_step: Option<CycleStep>,
    /// Auto-generated wiring code snippet.
    pub generated_code: String,
    pub status: WiringStatus,
    pub attempts: usize,
    pub last_error: Option<String>,
}

/// Result of one gap scan.
#[derive(Debug, Clone)]
pub struct GapScanResult {
    /// All modules found in the directory.
    pub scanned_modules: Vec<String>,
    /// Modules currently wired into ConsciousnessCycle.
    pub wired_modules: Vec<String>,
    /// Modules not wired into ConsciousnessCycle.
    pub unwired_modules: Vec<String>,
    /// The cycle number when this scan was performed.
    pub scan_cycle: u64,
}

impl GapScanResult {
    pub fn gap_count(&self) -> usize {
        self.unwired_modules.len()
    }

    pub fn coverage(&self) -> f64 {
        let total = self.scanned_modules.len();
        if total == 0 {
            return 1.0;
        }
        self.wired_modules.len() as f64 / total as f64
    }
}

/// The meta-iteration engine that drives the consciousness's own evolution
/// by discovering wiring gaps, creating tasks, generating code, and learning
/// from outcomes.
///
/// Architecture (top-down):
/// 1. GapDiscovery: Scans module listing vs ConsciousnessCycle for unregistered modules
/// 2. TaskCreation: For each gap, creates an EvolutionTask with wiring metadata
/// 3. WiringEngine: Generates Rust code to wire a module into the appropriate cycle step
/// 4. OutcomeLearning: Records success/failure and feeds into SubAgentAccumulator
/// 5. StrategyEvolution: Feeds outcomes to EscherLoopEngine for optimizer evolution
#[derive(Debug)]
pub struct SelfEvolutionTaskOrchestrator {
    /// Tracked wiring tasks.
    wiring_tasks: Vec<WiringTask>,
    /// Next task ID.
    next_task_id: u64,
    /// Gap discovery results (cached to avoid re-scanning every cycle).
    last_gap_scan: Option<GapScanResult>,
    /// Configuration.
    config: OrchestratorConfig,
    /// Statistics.
    stats: OrchestratorStats,
}

impl SelfEvolutionTaskOrchestrator {
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            wiring_tasks: Vec::new(),
            next_task_id: 1,
            last_gap_scan: None,
            config,
            stats: OrchestratorStats::default(),
        }
    }

    /// Scan for wiring gaps: compare the list of all modules found in a directory
    /// against the list of modules already wired into ConsciousnessCycle.
    /// Returns a GapScanResult with the diff.
    pub fn scan_gaps(
        &mut self,
        cycle: u64,
        all_modules: &[String],
        wired_modules: &[String],
    ) -> GapScanResult {
        let wired_set: std::collections::HashSet<&str> =
            wired_modules.iter().map(|s| s.as_str()).collect();

        let mut unwired = Vec::new();
        for m in all_modules {
            if !wired_set.contains(m.as_str()) {
                unwired.push(m.clone());
            }
        }

        let result = GapScanResult {
            scanned_modules: all_modules.to_vec(),
            wired_modules: wired_modules.to_vec(),
            unwired_modules: unwired,
            scan_cycle: cycle,
        };

        self.stats.gaps_discovered = result.unwired_modules.len() as u64;
        self.last_gap_scan = Some(result.clone());
        result
    }

    /// Convert a snake_case module name to a PascalCase type name.
    ///
    /// Examples:
    /// - "mcts_tree_search" → "MctsTreeSearch"
    /// - "dead_end_detector" → "DeadEndDetector"
    /// - "cognitive_blackboard" → "CognitiveBlackboard"
    /// - "attention_schema" → "AttentionSchema"
    /// - "qualia_generator" → "QualiaGenerator"
    pub fn module_name_to_type_name(module_name: &str) -> String {
        module_name
            .split('_')
            .filter(|s| !s.is_empty())
            .map(|segment| {
                let mut chars = segment.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        let upper = first.to_uppercase().to_string();
                        let rest: String = chars.collect();
                        upper + &rest
                    }
                }
            })
            .collect()
    }

    /// Convert a snake_case module name to a camelCase field name.
    ///
    /// Examples:
    /// - "mcts_tree_search" → "mctsTreeSearch"
    /// - "dead_end_detector" → "deadEndDetector"
    pub fn module_name_to_field_name(module_name: &str) -> String {
        let mut result = String::new();
        for (i, segment) in module_name.split('_').filter(|s| !s.is_empty()).enumerate() {
            if i == 0 {
                result.push_str(segment);
            } else {
                let mut chars = segment.chars();
                if let Some(first) = chars.next() {
                    result.push(first.to_uppercase().next().unwrap_or(first));
                    result.push_str(chars.as_str());
                }
            }
        }
        result
    }

    /// Heuristically detect which CycleStep a module belongs to based on its name.
    ///
    /// Rules (matched in order, first match wins):
    /// - "causal", "mcts", "reason", "dead_end", "counterfactual",
    ///   "analogical", "parallel_hypothesis" → Reason
    /// - "attention", "salience", "compete" → Compete
    /// - "memory", "stream", "buffer", "record" → Record
    /// - "judge", "critic", "scar" → Judge
    /// - "verify", "gate", "shadow" → Verify
    /// - "executive", "act", "goal", "skill", "wal" → Act
    /// - "metric", "master", "phi", "dashboard" → Metric
    /// - "meta", "metacognitive", "arch_governor", "rsi" → Meta
    /// - "sleep", "consolidation", "bridge" → Sleep
    /// - "modality", "sensor", "identity", "gate" → Gate
    /// - "gather", "perception", "image", "document" → Gather
    /// - "propose", "substrate", "exploration", "blackboard" → Propose
    /// - default → Gather
    pub fn auto_detect_step(module_name: &str) -> CycleStep {
        let lower = module_name.to_lowercase();

        // REASON group
        let reason_keywords = [
            "causal", "mcts", "reason", "reasoning", "dead_end", "counterfactual",
            "analogical", "parallel_hypothesis", "prm", "pruner", "selector",
        ];
        if reason_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Reason;
        }

        // COMPETE group
        let compete_keywords = ["attention", "salience", "compete", "spreading", "neuromodulator"];
        if compete_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Compete;
        }

        // JUDGE group
        let judge_keywords = ["judge", "critic", "scar", "consensus"];
        if judge_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Judge;
        }

        // VERIFY group
        let verify_keywords = ["verify", "shadow", "safety"];
        if verify_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Verify;
        }

        // ACT group
        let act_keywords = ["executive", "act", "goal", "skill", "wal", "ouroboros"];
        if act_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Act;
        }

        // RECORD group
        let record_keywords = ["memory", "stream", "buffer", "record", "flywheel", "boredom", "bio_memory"];
        if record_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Record;
        }

        // METRIC group
        let metric_keywords = ["metric", "master", "phi", "dashboard", "quality"];
        if metric_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Metric;
        }

        // META group
        let meta_keywords = ["meta", "metacognitive", "arch_governor", "rsi", "architecture"];
        if meta_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Meta;
        }

        // SLEEP group
        let sleep_keywords = ["sleep", "consolidation", "bridge"];
        if sleep_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Sleep;
        }

        // GATE group
        let gate_keywords = ["modality", "sensor", "identity", "defense"];
        if gate_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Gate;
        }

        // GATHER group
        let gather_keywords = ["gather", "perception", "image", "document", "extractor"];
        if gather_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Gather;
        }

        // PROPOSE group
        let propose_keywords = [
            "propose", "substrate", "exploration", "blackboard", "claim",
        ];
        if propose_keywords.iter().any(|k| lower.contains(k)) {
            return CycleStep::Propose;
        }

        CycleStep::Gather
    }

    /// Return the canonical manifest of all known subsystem modules that should
    /// be wired into ConsciousnessCycle.
    ///
    /// Returns `(module_name, crate_dir)` pairs. `module_name` is the snake_case
    /// filename (e.g. `"causal_reasoning"`). `crate_dir` is the subdirectory
    /// within `neotrix-core/src/core/` (e.g. `"nt_core_consciousness"`).
    ///
    /// This manifest is derived from the actual file listing — every `.rs` file
    /// with a `pub fn new()` across core consciousness‑relevant directories that
    /// is NOT already wired as a field in `ConsciousnessCycle`.
    pub fn known_subsystem_manifest() -> Vec<(&'static str, &'static str)> {
        vec![
            // ── nt_core_reasoning (18 total, 3 wired → 15 candidates) ──
            ("binary_vsa_attention", "nt_core_reasoning"),
            ("parallel_hypothesis", "nt_core_reasoning"),
            ("vsa_blackboard", "nt_core_reasoning"),
            ("vsa_reasoner", "nt_core_reasoning"),
            ("spike_processor", "nt_core_reasoning"),
            ("pipeline_orchestrator", "nt_core_reasoning"),
            ("causal_reasoning", "nt_core_reasoning"),
            ("analogical_reasoning", "nt_core_reasoning"),
            ("process_reward_model", "nt_core_reasoning"),
            ("bidirectional_pruner", "nt_core_reasoning"),
            ("strategy_selector", "nt_core_reasoning"),
            ("mcts_tree_search", "nt_core_reasoning"),
            ("parallel_hypothesis_evaluator", "nt_core_reasoning"),
            ("counterfactual_cognitive_module", "nt_core_reasoning"),
            ("dead_end_cognitive_module", "nt_core_reasoning"),
            // ── nt_core_consciousness (77 total, ~45 wired → ~32 candidates) ──
            ("active_inference", "nt_core_consciousness"),
            ("adaptive_controller", "nt_core_consciousness"),
            ("adversarial_evaluator", "nt_core_consciousness"),
            ("affective_circumplex", "nt_core_consciousness"),
            ("affective_forecast", "nt_core_consciousness"),
            ("appraisal_engine", "nt_core_consciousness"),
            ("awakening", "nt_core_consciousness"),
            ("backpressure", "nt_core_consciousness"),
            ("belief_revision", "nt_core_consciousness"),
            ("caa_steering", "nt_core_consciousness"),
            ("caa_validation", "nt_core_consciousness"),
            ("canonical_sort", "nt_core_consciousness"),
            ("causal_counterfactual_bridge", "nt_core_consciousness"),
            ("causal_model", "nt_core_consciousness"),
            ("claim_calibrator", "nt_core_consciousness"),
            ("cognitive_flexibility", "nt_core_consciousness"),
            ("cognitive_load", "nt_core_consciousness"),
            ("cognitive_state", "nt_core_consciousness"),
            ("confidence_calibrator", "nt_core_consciousness"),
            ("conformal_uq", "nt_core_consciousness"),
            ("consciousness_assessment", "nt_core_consciousness"),
            ("continuous_ode", "nt_core_consciousness"),
            ("counterfactual", "nt_core_consciousness"),
            ("cte_consolidation", "nt_core_consciousness"),
            ("dream_consolidator", "nt_core_consciousness"),
            ("drive_selector", "nt_core_consciousness"),
            ("dual_path_inference", "nt_core_consciousness"),
            ("earned_autonomy", "nt_core_consciousness"),
            ("embodied_grounding", "nt_core_consciousness"),
            ("emergent_reasoning", "nt_core_consciousness"),
            ("emotion_regulation", "nt_core_consciousness"),
            ("emotional_steering", "nt_core_consciousness"),
            ("episodic_buffer", "nt_core_consciousness"),
            ("epistemic_calibrator", "nt_core_consciousness"),
            ("ethics_compliance", "nt_core_consciousness"),
            ("executable_belief", "nt_core_consciousness"),
            ("gea_archive", "nt_core_consciousness"),
            ("global_workspace", "nt_core_consciousness"),
            ("hebbian_associative_memory", "nt_core_consciousness"),
            ("hierarchical_memory", "nt_core_consciousness"),
            ("identity_chain", "nt_core_consciousness"),
            ("identity_fragments", "nt_core_consciousness"),
            ("interior_monologue", "nt_core_consciousness"),
            ("intrinsic_motivation", "nt_core_consciousness"),
            ("iron_laws", "nt_core_consciousness"),
            ("joint_attention", "nt_core_consciousness"),
            ("kv_cache_consolidation", "nt_core_consciousness"),
            ("log_linear_attention", "nt_core_consciousness"),
            ("master_equation", "nt_core_consciousness"),
            ("mcts_gwt_bridge", "nt_core_consciousness"),
            ("memory_palace", "nt_core_consciousness"),
            ("memory_reflector", "nt_core_consciousness"),
            ("mental_time_travel", "nt_core_consciousness"),
            ("meta_cognition_bridge", "nt_core_consciousness"),
            ("meta_evolution_loop", "nt_core_consciousness"),
            ("minimal_self", "nt_core_consciousness"),
            ("mirror_buffer", "nt_core_consciousness"),
            ("mtc_safety", "nt_core_consciousness"),
            ("narrative_journal", "nt_core_consciousness"),
            ("narrative_self", "nt_core_consciousness"),
            ("performance_oracle", "nt_core_consciousness"),
            ("personality_matrix", "nt_core_consciousness"),
            ("phi_integration", "nt_core_consciousness"),
            ("predictive_gate", "nt_core_consciousness"),
            ("proof_search", "nt_core_consciousness"),
            ("qualia_layer", "nt_core_consciousness"),
            ("reasoning_federation", "nt_core_consciousness"),
            ("reconstructive_narrative", "nt_core_consciousness"),
            ("recurrent_world_model", "nt_core_consciousness"),
            ("reflexive_unit", "nt_core_consciousness"),
            ("resource_allocator", "nt_core_consciousness"),
            ("resource_pool", "nt_core_consciousness"),
            ("rii_u", "nt_core_consciousness"),
            ("screenshot_pipeline", "nt_core_consciousness"),
            ("sleep_consolidation_bridge", "nt_core_consciousness"),
            ("sleep_gate", "nt_core_consciousness"),
            ("source_hierarchy", "nt_core_consciousness"),
            ("specious_present", "nt_core_consciousness"),
            ("spectrum_signal", "nt_core_consciousness"),
            ("storm_breaker", "nt_core_consciousness"),
            ("stream_buffer", "nt_core_consciousness"),
            ("sub_consciousness", "nt_core_consciousness"),
            ("system1", "nt_core_consciousness"),
            ("temporal_attention", "nt_core_consciousness"),
            ("temporal_attention_stack", "nt_core_consciousness"),
            ("unified_will", "nt_core_consciousness"),
            ("valence_axis", "nt_core_consciousness"),
            ("value_alignment", "nt_core_consciousness"),
            ("value_system", "nt_core_consciousness"),
            ("volition", "nt_core_consciousness"),
            ("vsa_prefix_fingerprint", "nt_core_consciousness"),
            ("worldview_stack", "nt_core_consciousness"),
            // ── nt_core_knowledge (14 candidates) ──
            ("activation", "nt_core_knowledge"),
            ("atomic_fact", "nt_core_knowledge"),
            ("evidence", "nt_core_knowledge"),
            ("evidence_inspector", "nt_core_knowledge"),
            ("execution_trace", "nt_core_knowledge"),
            ("forgetting_strategy", "nt_core_knowledge"),
            ("fringe_mix", "nt_core_knowledge"),
            ("graph_r1", "nt_core_knowledge"),
            ("hubness_detector", "nt_core_knowledge"),
            ("hypergraph", "nt_core_knowledge"),
            ("keyword_lexicon", "nt_core_knowledge"),
            ("knowledge_routing", "nt_core_knowledge"),
            ("multimodal_storyteller", "nt_core_knowledge"),
            ("progress_aware_rag", "nt_core_knowledge"),
            ("research_kg", "nt_core_knowledge"),
            ("self_inspect", "nt_core_knowledge"),
            ("sources", "nt_core_knowledge"),
            ("system_card", "nt_core_knowledge"),
            ("tracker", "nt_core_knowledge"),
            ("versioning", "nt_core_knowledge"),
            ("vsa_vocabulary", "nt_core_knowledge"),
            // ── nt_core_gwt (5 candidates) ──
            ("curiosity_exploration", "nt_core_gwt"),
            ("epistemic_queue", "nt_core_gwt"),
            ("goal_synthesis", "nt_core_gwt"),
            ("intrinsic_drive", "nt_core_gwt"),
            ("manar_attention", "nt_core_gwt"),
            ("module_def", "nt_core_gwt"),
            ("monitor", "nt_core_gwt"),
            ("multi_modal_curiosity", "nt_core_gwt"),
            ("physics_attention", "nt_core_gwt"),
            ("resonance", "nt_core_gwt"),
            ("self_interrupt", "nt_core_gwt"),
            // ── nt_core_hcube (32 candidates) ──
            ("adapt_encoder", "nt_core_hcube"),
            ("adaptive_encoder", "nt_core_hcube"),
            ("attractor_basin", "nt_core_hcube"),
            ("coord", "nt_core_hcube"),
            ("cross_modal", "nt_core_hcube"),
            ("cube", "nt_core_hcube"),
            ("diff_vsa", "nt_core_hcube"),
            ("dom_vsa", "nt_core_hcube"),
            ("dream_consolidation", "nt_core_hcube"),
            ("e8_cortical", "nt_core_hcube"),
            ("e8_field", "nt_core_hcube"),
            ("e8_lagrangian", "nt_core_hcube"),
            ("e8_lattice", "nt_core_hcube"),
            ("ebbinghaus_decay", "nt_core_hcube"),
            ("efe_curiosity_bridge", "nt_core_hcube"),
            ("fpe", "nt_core_hcube"),
            ("gap", "nt_core_hcube"),
            ("geometric_ssm", "nt_core_hcube"),
            ("go_cls_gate", "nt_core_hcube"),
            ("hippocampal_trace", "nt_core_hcube"),
            ("hopfield_network", "nt_core_hcube"),
            ("interaction_trace", "nt_core_hcube"),
            ("koopman_operator", "nt_core_hcube"),
            ("kroneker_cleanup", "nt_core_hcube"),
            ("linear_code", "nt_core_hcube"),
            ("linear_code_vsa", "nt_core_hcube"),
            ("magma_memory", "nt_core_hcube"),
            ("memory_activation", "nt_core_hcube"),
            ("mhn_pattern_separation", "nt_core_hcube"),
            ("multi_head_resonator", "nt_core_hcube"),
            ("multi_modal_aligner", "nt_core_hcube"),
            ("narrative_hypercube_bridge", "nt_core_hcube"),
            ("narrative_vsa_binding", "nt_core_hcube"),
            ("octonion", "nt_core_hcube"),
            ("physics_commonsense", "nt_core_hcube"),
            ("qfhrr_vsa", "nt_core_hcube"),
            ("resonator_decoder", "nt_core_hcube"),
            ("rotation_bind", "nt_core_hcube"),
            ("selfref_meta", "nt_core_hcube"),
            ("sign_flip_vsa", "nt_core_hcube"),
            ("skill_compiler", "nt_core_hcube"),
            ("sm2_scheduler", "nt_core_hcube"),
            ("sparse_hypercube", "nt_core_hcube"),
            ("sparse_vsa", "nt_core_hcube"),
            ("spatial_scene", "nt_core_hcube"),
            ("spectral_forcing", "nt_core_hcube"),
            ("spectral_nsr", "nt_core_hcube"),
            ("spectral_vsa", "nt_core_hcube"),
            ("subspace", "nt_core_hcube"),
            ("thdc_encoder", "nt_core_hcube"),
            ("topo_cube", "nt_core_hcube"),
            ("topology", "nt_core_hcube"),
            ("trigram_index", "nt_core_hcube"),
            ("visual_embedding_frontend", "nt_core_hcube"),
            ("visual_rag_index", "nt_core_hcube"),
            ("vsa_gpu", "nt_core_hcube"),
            ("vsa_holon", "nt_core_hcube"),
            ("vsa_hrr", "nt_core_hcube"),
            ("vsa_multi_model", "nt_core_hcube"),
            ("vsa_quantized", "nt_core_hcube"),
            ("vsa_runtime_ir", "nt_core_hcube"),
            ("vsa_spatial_encoder", "nt_core_hcube"),
            ("vsa_vector", "nt_core_hcube"),
            ("wave_geometric", "nt_core_hcube"),
            // ── nt_core_negentropy (3 candidates) ──
            ("act_planner", "nt_core_negentropy"),
            ("dysib_layer", "nt_core_negentropy"),
            ("efe_minimizer", "nt_core_negentropy"),
            ("jepa_efe_calculator", "nt_core_negentropy"),
            ("jepa_transition", "nt_core_negentropy"),
            // ── nt_core_input (4 candidates) ──
            ("novelty_detector", "nt_core_input"),
            ("parallel_decoder", "nt_core_input"),
            ("unified_search", "nt_core_input"),
            ("vsa_input_pipeline", "nt_core_input"),
            // ── nt_core_context (6 candidates) ──
            ("capability_evidence", "nt_core_context"),
            ("ccr", "nt_core_context"),
            ("context_budget", "nt_core_context"),
            ("context_gatherer", "nt_core_context"),
            ("context_os", "nt_core_context"),
            ("context_predictor", "nt_core_context"),
            ("prefix_volatility", "nt_core_context"),
            ("working_memory", "nt_core_context"),
            // ── nt_core_edit (4 candidates) ──
            ("cognitive_wal", "nt_core_edit"),
            ("pco", "nt_core_edit"),
            ("proof_bundle", "nt_core_edit"),
            ("rsi_meta_cycle", "nt_core_edit"),
            ("self_mod_pipeline", "nt_core_edit"),
            ("shadow_runtime", "nt_core_edit"),
            // ── nt_core_identity (7 candidates) ──
            ("between_sessions", "nt_core_identity"),
            ("coproc_bridge", "nt_core_identity"),
            ("cvo_role", "nt_core_identity"),
            ("identity_boundary", "nt_core_identity"),
            ("identity_core", "nt_core_identity"),
            ("identity_evolution", "nt_core_identity"),
            ("inter_session", "nt_core_identity"),
            ("persistent_context", "nt_core_identity"),
            ("self_reasoner", "nt_core_identity"),
            ("value_gate", "nt_core_identity"),
            // ── nt_core_self (22 candidates) ──
            ("architecture_governor", "nt_core_self"),
            ("archive", "nt_core_self"),
            ("attention_head", "nt_core_self"),
            ("autonomy_harness", "nt_core_self"),
            ("cognitive_dashboard", "nt_core_self"),
            ("config_space", "nt_core_self"),
            ("context_window", "nt_core_self"),
            ("evolution_trace", "nt_core_self"),
            ("experimentation", "nt_core_self"),
            ("intervention_hypothesis", "nt_core_self"),
            ("intra_reflection", "nt_core_self"),
            ("intrinsic_motivation", "nt_core_self"),
            ("learning_mechanics", "nt_core_self"),
            ("metacognitive_evaluator", "nt_core_self"),
            ("observables", "nt_core_self"),
            ("reasoning_strategy", "nt_core_self"),
            ("research_intuition", "nt_core_self"),
            ("self_referential", "nt_core_self"),
            ("silicon_self", "nt_core_self"),
            ("system_identity", "nt_core_self"),
            ("temporal_attention_engine", "nt_core_self"),
            ("thinking_trace", "nt_core_self"),
            ("toy_model_gen", "nt_core_self"),
            ("verified_rsi", "nt_core_self"),
            ("vibe_trainer", "nt_core_self"),
            // ── nt_core_meta (26 candidates) ──
            ("a2a_router", "nt_core_meta"),
            ("always_on_daemon", "nt_core_meta"),
            ("audit", "nt_core_meta"),
            ("cross_binding_loop", "nt_core_meta"),
            ("embodiment_curriculum", "nt_core_meta"),
            ("error_bounds", "nt_core_meta"),
            ("formal_introspect", "nt_core_meta"),
            ("fusion_gap", "nt_core_meta"),
            ("harness", "nt_core_meta"),
            ("inner_monologue", "nt_core_meta"),
            ("knowledge_gap_detector", "nt_core_meta"),
            ("kpi_persistence", "nt_core_meta"),
            ("mcp_callback_bridge", "nt_core_meta"),
            ("memory_evolution", "nt_core_meta"),
            ("meta_kpi_repo", "nt_core_meta"),
            ("meta_learning", "nt_core_meta"),
            ("meta_reflection_engine", "nt_core_meta"),
            ("metacognition_loop", "nt_core_meta"),
            ("metacognitive_state", "nt_core_meta"),
            ("mirror_bench", "nt_core_meta"),
            ("mission_hub", "nt_core_meta"),
            ("mod_sandbox", "nt_core_meta"),
            ("monitor", "nt_core_meta"),
            ("planner", "nt_core_meta"),
            ("scanner", "nt_core_meta"),
            ("self_model_level", "nt_core_meta"),
            ("self_model", "nt_core_meta"),
            ("skill_evolution_modes", "nt_core_meta"),
            ("skill_registry", "nt_core_meta"),
            ("timer", "nt_core_meta"),
            ("uncertainty_tracker", "nt_core_meta"),
            ("weakness", "nt_core_meta"),
        ]
    }

    /// Whether auto-fix mode is enabled.
    pub fn auto_fix_enabled(&self) -> bool {
        self.config.auto_fix
    }

    /// Generate and apply wiring patches for all unwired modules in the gap scan.
    /// Returns the number of patches successfully applied.
    pub fn apply_patches(&mut self, gap_scan: &GapScanResult) -> usize {
        let mut applied = 0;
        for module_name in &gap_scan.unwired_modules {
            let step = Self::auto_detect_step(module_name);
            let patch = Self::generate_wiring_patch(module_name, step);
            // Record the patch as applied in task tracking
            let task_id = self.next_task_id;
            self.next_task_id += 1;

            let task = WiringTask {
                id: task_id,
                module_name: module_name.clone(),
                module_path: patch.import_line.clone(),
                target_step: Some(step),
                generated_code: patch.step_invocation.clone(),
                status: WiringStatus::WiringApplied,
                attempts: 1,
                last_error: None,
            };
            self.wiring_tasks.push(task);
            self.stats.wiring_attempted += 1;
            applied += 1;
        }
        self.stats.wiring_succeeded += applied as u64;
        applied
    }

    /// Generate a Rust code snippet showing how to wire a module into
    /// the consciousness cycle at the given step.
    ///
    /// The generated code follows the existing pattern:
    /// ```
    /// // In ConsciousnessCycle struct:
    /// {field_name}: Option<{type_name}>,
    ///
    /// // In new():
    /// {field_name}: Some({type_name}::new()),
    ///
    /// // In the step method (e.g., REASON):
    /// if let Some(ref mut {field_name}) = self.{field_name} {
    ///     {field_name}.process(...);
    /// }
    /// ```
    pub fn generate_wiring_code(module_name: &str, step: CycleStep) -> String {
        let type_name = Self::module_name_to_type_name(module_name);
        let field_name = Self::module_name_to_field_name(module_name);
        let step_upper = format!("{:?}", step).to_uppercase();

        format!(
            r#"// ── Wire: {module_name} → {step:?} ──

// 1. In ConsciousnessCycle struct (consciousness_cycle.rs):
//    {field_name}: Option<{type_name}>,

// 2. In ConsciousnessCycle::new():
//    {field_name}: Some({type_name}::new()),

// 3. In run_cycle(), inside the {step_upper} step:
//    if let Some(ref mut {field_name}) = self.{field_name} {{
//        {field_name}.process(/* ... */);
//    }}

// 4. Add Clone in Clone impl:
//    {field_name}: self.{field_name}.clone(),

// 5. Add Debug field in Debug impl:
//    .field("{field_name}", &self.{field_name}.is_some())
"#,
            module_name = module_name,
            step = step,
            step_upper = step_upper,
            type_name = type_name,
            field_name = field_name,
        )
    }

    /// Generate the specific run_cycle invocation block for a module
    /// at the target step. Returns a tuple of (variable_declarations, invocation_code).
    pub fn generate_step_invocation(module_name: &str, step: CycleStep) -> (String, String) {
        let type_name = Self::module_name_to_type_name(module_name);
        let field_name = Self::module_name_to_field_name(module_name);
        let step_name = format!("{:?}", step).to_lowercase();

        let declaration = format!(
            "// Variable for {type_name} — populated in {step_name} step\n\
             let mut {field_name}_output: Option<String> = None;"
        );

        let invocation = format!(
            "// ── {type_name} — {step_name} ──\n\
             if let Some(ref mut {field_name}) = self.{field_name} {{\n\
             \x20   // TODO: pass appropriate input from cycle context\n\
             \x20   let _result = {field_name}.process(/* ... */);\n\
             \x20   {field_name}_output = Some(format!(\"{{:?}}\", _result));\n\
             }}"
        );

        (declaration, invocation)
    }

    /// Create WiringTasks for each unwired module in a GapScanResult.
    /// Each task gets an auto-detected step and generated wiring code.
    /// Limited to `max_tasks_per_scan` tasks per call.
    pub fn create_wiring_tasks(&mut self, gap_scan: &GapScanResult) -> Vec<WiringTask> {
        let limit = self.config.max_tasks_per_scan.min(gap_scan.unwired_modules.len());
        let mut new_tasks = Vec::with_capacity(limit);

        for module_name in gap_scan.unwired_modules.iter().take(limit) {
            let step = Self::auto_detect_step(module_name);
            let code = Self::generate_wiring_code(module_name, step);
            let _field_name = Self::module_name_to_field_name(module_name);

            // Construct a plausible module path for the generated code comment
            let module_path = format!("core::nt_core_consciousness::{}", module_name);

            let task = WiringTask {
                id: self.next_task_id,
                module_name: module_name.clone(),
                module_path,
                target_step: Some(step),
                generated_code: code,
                status: WiringStatus::Discovered,
                attempts: 0,
                last_error: None,
            };

            self.wiring_tasks.push(task.clone());
            new_tasks.push(task);
            self.next_task_id += 1;
        }

        self.stats.tasks_created += new_tasks.len() as u64;
        new_tasks
    }

    /// Record the outcome of a wiring attempt for a task.
    /// Updates stats and the task's status and error field.
    pub fn record_outcome(
        &mut self,
        task_id: u64,
        success: bool,
        error: Option<String>,
    ) {
        let found = self.wiring_tasks.iter_mut().find(|t| t.id == task_id);
        if let Some(task) = found {
            task.attempts += 1;
            if success {
                task.status = WiringStatus::Verified;
                task.last_error = None;
                self.stats.wiring_succeeded += 1;
            } else {
                if task.attempts >= self.config.max_auto_attempts {
                    task.status = WiringStatus::CompilationFailed;
                } else {
                    task.status = WiringStatus::WiringApplied;
                }
                task.last_error = error;
                self.stats.wiring_failed += 1;
            }
            self.stats.wiring_attempted += 1;
        }
    }

    /// Mark a wiring task as code-generated (after code generation step).
    pub fn mark_code_generated(&mut self, task_id: u64) {
        if let Some(task) = self.wiring_tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = WiringStatus::CodeGenerated;
        }
    }

    /// Mark a wiring task as applied (after source modification).
    pub fn mark_wiring_applied(&mut self, task_id: u64) {
        if let Some(task) = self.wiring_tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = WiringStatus::WiringApplied;
        }
    }

    /// Mark a wiring task as compilation-verified.
    pub fn mark_compilation_verified(&mut self, task_id: u64) {
        if let Some(task) = self.wiring_tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = WiringStatus::Verified;
            self.stats.compilation_verified += 1;
        }
    }

    /// Get pending (not yet verified or failed-out) wiring tasks.
    pub fn pending_tasks(&self) -> Vec<&WiringTask> {
        self.wiring_tasks
            .iter()
            .filter(|t| {
                matches!(
                    t.status,
                    WiringStatus::Discovered
                        | WiringStatus::TaskCreated
                        | WiringStatus::CodeGenerated
                        | WiringStatus::WiringApplied
                )
            })
            .collect()
    }

    /// Get failed tasks (exceeded max auto-attempts).
    pub fn failed_tasks(&self) -> Vec<&WiringTask> {
        self.wiring_tasks
            .iter()
            .filter(|t| t.status == WiringStatus::CompilationFailed)
            .collect()
    }

    /// Get verified tasks.
    pub fn verified_tasks(&self) -> Vec<&WiringTask> {
        self.wiring_tasks
            .iter()
            .filter(|t| t.status == WiringStatus::Verified)
            .collect()
    }

    /// All tracked wiring tasks.
    pub fn all_tasks(&self) -> &[WiringTask] {
        &self.wiring_tasks
    }

    /// Current statistics.
    pub fn stats(&self) -> &OrchestratorStats {
        &self.stats
    }

    /// Configuration reference.
    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    /// Last gap scan result (if any).
    pub fn last_gap_scan(&self) -> Option<&GapScanResult> {
        self.last_gap_scan.as_ref()
    }

    /// Human-readable summary of the orchestrator state.
    pub fn summary(&self) -> String {
        let total = self.wiring_tasks.len();
        let pending = self.pending_tasks().len();
        let verified = self.verified_tasks().len();
        let failed = self.failed_tasks().len();
        let coverage = self
            .last_gap_scan
            .as_ref()
            .map(|g| g.coverage())
            .unwrap_or(0.0);

        format!(
            "SelfEvolutionTaskOrchestrator: {} tasks ({} pending/{} verified/{} failed), \
             gaps={} tasks_created={} wiring={}(ok={}/fail={}) comp_verified={} coverage={:.1}% \
             cfg(scan_interval={} max_per_scan={} auto_exec={} max_attempts={})",
            total,
            pending,
            verified,
            failed,
            self.stats.gaps_discovered,
            self.stats.tasks_created,
            self.stats.wiring_attempted,
            self.stats.wiring_succeeded,
            self.stats.wiring_failed,
            self.stats.compilation_verified,
            coverage * 100.0,
            self.config.scan_interval_cycles,
            self.config.max_tasks_per_scan,
            self.config.auto_execute,
            self.config.max_auto_attempts,
        )
    }
}

// ============================================================================
// WiringEngine — generates Rust code to wire a module into ConsciousnessCycle
// ============================================================================

/// Complete set of code changes needed to wire one module into the cycle.
#[derive(Debug, Clone)]
pub struct WiringPatch {
    /// Module name in snake_case (e.g. "mcts_tree_search")
    pub module_name: String,
    /// PascalCase type name (e.g. "MctsTreeSearch")
    pub type_name: String,
    /// camelCase field name (e.g. "mctsTreeSearch")
    pub field_name: String,
    /// Target CycleStep
    pub target_step: CycleStep,
    /// Import line to add at top of file
    pub import_line: String,
    /// Field declaration for the struct
    pub field_decl: String,
    /// Constructor line for new()
    pub constructor_line: String,
    /// Builder method text
    pub builder_method: String,
    /// Config flag declaration in CycleConfig
    pub config_flag: String,
    /// If-let invocation block for the step method
    pub step_invocation: String,
}

/// Generate a WiringPatch for a cognitive module.
///
/// Uses module-specific API knowledge for common modules, or a generic
/// `module.tick()` fallback for unknown ones.
impl SelfEvolutionTaskOrchestrator {
    pub fn generate_wiring_patch(module_name: &str, step: CycleStep) -> WiringPatch {
        let type_name = Self::module_name_to_type_name(module_name);
        let field_name = Self::module_name_to_field_name(module_name);
        let config_flag = format!("enable_{}", module_name);

        // Determine import path based on module location
        let import_line = if Self::is_reasoning_module(module_name) {
            format!(
                "use crate::core::nt_core_reasoning::{}::{};",
                module_name, type_name
            )
        } else {
            format!(
                "use super::{}::{};",
                module_name, type_name
            )
        };

        let field_decl = format!(
            "    {}: Option<{}>,",
            field_name, type_name
        );

        let constructor_line = format!(
            "            {}: Some({}::new()),",
            field_name, type_name
        );

        let builder_method = format!(
            "    pub fn with_{}(mut self, m: {}) -> Self {{\n        self.{} = Some(m);\n        self\n    }}",
            module_name, type_name, field_name
        );

        let step_invocation = Self::step_invocation_text(module_name, &field_name, &type_name, step);

        WiringPatch {
            module_name: module_name.to_string(),
            type_name,
            field_name,
            target_step: step,
            import_line,
            field_decl,
            constructor_line,
            builder_method,
            config_flag,
            step_invocation,
        }
    }

    /// Check if a module lives in nt_core_reasoning
    fn is_reasoning_module(name: &str) -> bool {
        matches!(
            name,
            "mcts_tree_search" | "dead_end_detector" | "counterfactual_simulator"
                | "parallel_hypothesis" | "process_reward_model"
                | "bidirectional_pruner" | "strategy_selector"
                | "causal_reasoning" | "analogical_reasoning"
        )
    }

    /// Generate the if-let invocation block for a module in a step.
    fn step_invocation_text(module_name: &str, field_name: &str, _type_name: &str, _step: CycleStep) -> String {
        let known_api = Self::known_module_invocation(module_name);
        if let Some(invocation) = known_api {
            return format!(
                "            if let Some(ref mut {f}) = self.{f} {{\n                {inv};\n            }}",
                f = field_name, inv = invocation
            );
        }
        // Generic fallback: call tick()
        format!(
            "            if let Some(ref mut {f}) = self.{f} {{\n                let _ = {f}.tick();\n            }}",
            f = field_name
        )
    }

    /// Module-specific API invocations for known modules.
    fn known_module_invocation(module_name: &str) -> Option<String> {
        match module_name {
            "mcts_tree_search" => Some(
                "let _plan = f.search(&format!(\"cycle_{}\", self.cycle_num), 5.0)".to_string()
            ),
            "dead_end_detector" => Some(
                "ded.check(&format!(\"cycle_{}\", self.cycle_num), &[])".to_string()
            ),
            "counterfactual_simulator" => Some(
                "let _cf = cs.simulate(\"cycle\", &[])".to_string()
            ),
            "parallel_hypothesis" => Some(
                "let _hyp = ph.evaluate(&format!(\"cycle_{}\", self.cycle_num), &[])".to_string()
            ),
            "process_reward_model" => Some(
                "let _reward = prm.score(&format!(\"cycle_{}\", self.cycle_num))".to_string()
            ),
            "bidirectional_pruner" => Some(
                "bp.prune(&format!(\"cycle_{}\", self.cycle_num))".to_string()
            ),
            "strategy_selector" => Some(
                "let _strat = ss.select_strategy(&format!(\"cycle_{}\", self.cycle_num))".to_string()
            ),
            "causal_reasoning" => Some(
                "let _causes = cr.infer_causes(&format!(\"cycle_{}\", self.cycle_num), &[])".to_string()
            ),
            "analogical_reasoning" => Some(
                "let _analogies = ar.find_analogies(&format!(\"cycle_{}\", self.cycle_num))".to_string()
            ),
            _ => None,
        }
    }

    /// Describe the wiring patch as a human-readable summary.
    pub fn patch_summary(patch: &WiringPatch) -> String {
        format!(
            "Wire {} → {:?}: {} + builder + config flag + step invocation",
            patch.module_name, patch.target_step, patch.type_name
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_orchestrator() -> SelfEvolutionTaskOrchestrator {
        SelfEvolutionTaskOrchestrator::new(OrchestratorConfig::default())
    }

    // ── Construction ──

    #[test]
    fn test_new_creates_empty_orchestrator() {
        let o = make_orchestrator();
        assert!(o.all_tasks().is_empty());
        assert!(o.last_gap_scan().is_none());
        assert_eq!(o.stats().gaps_discovered, 0);
        assert_eq!(o.stats().tasks_created, 0);
    }

    // ── Gap scanning ──

    #[test]
    fn test_scan_gaps_identifies_unwired_modules() {
        let mut o = make_orchestrator();
        let all = vec![
            "mcts_tree_search".to_string(),
            "dead_end_detector".to_string(),
            "inner_critic".to_string(),
            "consciousness_stream".to_string(),
        ];
        let wired = vec!["inner_critic".to_string(), "consciousness_stream".to_string()];

        let result = o.scan_gaps(1, &all, &wired);
        assert_eq!(result.gap_count(), 2);
        assert!(result.unwired_modules.contains(&"mcts_tree_search".to_string()));
        assert!(result.unwired_modules.contains(&"dead_end_detector".to_string()));
        assert!((result.coverage() - 0.5).abs() < 1e-9);
        assert_eq!(result.scan_cycle, 1);
    }

    #[test]
    fn test_scan_gaps_all_wired_gives_zero_gaps() {
        let mut o = make_orchestrator();
        let all = vec!["module_a".to_string(), "module_b".to_string()];
        let wired = vec!["module_a".to_string(), "module_b".to_string()];

        let result = o.scan_gaps(5, &all, &wired);
        assert_eq!(result.gap_count(), 0);
        assert!((result.coverage() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_scan_gaps_empty_lists() {
        let mut o = make_orchestrator();
        let result = o.scan_gaps(0, &[], &[]);
        assert_eq!(result.gap_count(), 0);
        assert!((result.coverage() - 1.0).abs() < 1e-9);
    }

    // ── Module name conversion ──

    #[test]
    fn test_module_name_to_type_name() {
        let cases = vec![
            ("mcts_tree_search", "MctsTreeSearch"),
            ("dead_end_detector", "DeadEndDetector"),
            ("cognitive_blackboard", "CognitiveBlackboard"),
            ("attention_schema", "AttentionSchema"),
            ("qualia_generator", "QualiaGenerator"),
            ("parallel_hypothesis_evaluator", "ParallelHypothesisEvaluator"),
            ("consciousness_stream", "ConsciousnessStream"),
            ("single", "Single"),
            ("", ""),
        ];
        for (input, expected) in cases {
            assert_eq!(
                SelfEvolutionTaskOrchestrator::module_name_to_type_name(input),
                expected,
                "conversion for '{}'",
                input
            );
        }
    }

    #[test]
    fn test_module_name_to_field_name() {
        let cases = vec![
            ("mcts_tree_search", "mctsTreeSearch"),
            ("dead_end_detector", "deadEndDetector"),
            ("cognitive_blackboard", "cognitiveBlackboard"),
            ("single", "single"),
            ("", ""),
        ];
        for (input, expected) in cases {
            assert_eq!(
                SelfEvolutionTaskOrchestrator::module_name_to_field_name(input),
                expected,
                "field name for '{}'",
                input
            );
        }
    }

    // ── Step auto-detection ──

    #[test]
    fn test_auto_detect_step_reason_group() {
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("causal_model"),
            CycleStep::Reason
        );
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("mcts_tree_search"),
            CycleStep::Reason
        );
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("dead_end_detector"),
            CycleStep::Reason
        );
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("counterfactual_simulator"),
            CycleStep::Reason
        );
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("process_reward_model"),
            CycleStep::Reason
        );
    }

    #[test]
    fn test_auto_detect_step_judge_group() {
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("inner_critic"),
            CycleStep::Judge
        );
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("scar_formation"),
            CycleStep::Judge
        );
    }

    #[test]
    fn test_auto_detect_step_meta_group() {
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("metacognitive_controller"),
            CycleStep::Meta
        );
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("arch_governor"),
            CycleStep::Meta
        );
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("rsi_meta_cycle"),
            CycleStep::Meta
        );
    }

    #[test]
    fn test_auto_detect_step_default_to_gather() {
        assert_eq!(
            SelfEvolutionTaskOrchestrator::auto_detect_step("unknown_weird_module"),
            CycleStep::Gather
        );
    }

    // ── Wiring code generation ──

    #[test]
    fn test_generate_wiring_code_contains_key_parts() {
        let code = SelfEvolutionTaskOrchestrator::generate_wiring_code("mcts_tree_search", CycleStep::Reason);
        assert!(code.contains("MctsTreeSearch"), "should contain type name");
        assert!(code.contains("mcts_tree_search"), "should contain module name");
        assert!(code.contains("REASON"), "should contain step name");
        assert!(code.contains("Option<"), "should contain Option type");
        assert!(code.contains("Some("), "should contain Some constructor");
        assert!(code.contains("self.mcts_tree_search"), "should contain self reference");
    }

    #[test]
    fn test_generate_wiring_code_for_different_steps() {
        let code = SelfEvolutionTaskOrchestrator::generate_wiring_code("dead_end_detector", CycleStep::Reason);
        assert!(code.contains("DeadEndDetector"));
        assert!(code.contains("REASON"));

        let code2 = SelfEvolutionTaskOrchestrator::generate_wiring_code("attention_schema", CycleStep::Compete);
        assert!(code2.contains("AttentionSchema"));
        assert!(code2.contains("COMPETE"));
    }

    // ── Step invocation generation ──

    #[test]
    fn test_generate_step_invocation_contains_declaration_and_invocation() {
        let (decl, invoc) =
            SelfEvolutionTaskOrchestrator::generate_step_invocation("mcts_tree_search", CycleStep::Reason);
        assert!(decl.contains("MctsTreeSearch"));
        assert!(decl.contains("mcts_tree_search_output"));
        assert!(invoc.contains("mcts_tree_search"));
        assert!(invoc.contains("Option<String>"));
        assert!(invoc.contains(".process("));
    }

    // ── Wiring task creation ──

    #[test]
    fn test_create_wiring_tasks_from_gap_scan() {
        let mut o = make_orchestrator();
        let scan = GapScanResult {
            scanned_modules: vec![
                "mcts_tree_search".to_string(),
                "dead_end_detector".to_string(),
                "inner_critic".to_string(),
            ],
            wired_modules: vec!["inner_critic".to_string()],
            unwired_modules: vec!["mcts_tree_search".to_string(), "dead_end_detector".to_string()],
            scan_cycle: 10,
        };

        let tasks = o.create_wiring_tasks(&scan);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].module_name, "mcts_tree_search");
        assert_eq!(tasks[0].target_step, Some(CycleStep::Reason));
        assert_eq!(tasks[0].status, WiringStatus::Discovered);

        assert_eq!(tasks[1].module_name, "dead_end_detector");
        assert_eq!(tasks[1].target_step, Some(CycleStep::Reason));

        assert_eq!(o.stats().tasks_created, 2);
    }

    #[test]
    fn test_create_wiring_tasks_respects_max_per_scan() {
        let config = OrchestratorConfig {
            max_tasks_per_scan: 2,
            ..OrchestratorConfig::default()
        };
        let mut o = SelfEvolutionTaskOrchestrator::new(config);
        let scan = GapScanResult {
            scanned_modules: vec![],
            wired_modules: vec![],
            unwired_modules: vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
            scan_cycle: 1,
        };

        let tasks = o.create_wiring_tasks(&scan);
        assert_eq!(tasks.len(), 2);
    }

    // ── Outcome recording ──

    #[test]
    fn test_record_outcome_success_updates_stats_and_status() {
        let mut o = make_orchestrator();
        let scan = GapScanResult {
            scanned_modules: vec![],
            wired_modules: vec![],
            unwired_modules: vec!["test_module".to_string()],
            scan_cycle: 1,
        };
        let tasks = o.create_wiring_tasks(&scan);
        let task_id = tasks[0].id;

        assert_eq!(o.stats().wiring_succeeded, 0);
        assert_eq!(o.stats().wiring_attempted, 0);

        o.record_outcome(task_id, true, None);

        assert_eq!(o.stats().wiring_succeeded, 1);
        assert_eq!(o.stats().wiring_attempted, 1);
        let t = &o.wiring_tasks[0];
        assert_eq!(t.status, WiringStatus::Verified);
        assert_eq!(t.attempts, 1);
    }

    #[test]
    fn test_record_outcome_failure_tracks_attempts() {
        let mut o = make_orchestrator();
        let scan = GapScanResult {
            scanned_modules: vec![],
            wired_modules: vec![],
            unwired_modules: vec!["failing_mod".to_string()],
            scan_cycle: 1,
        };
        let tasks = o.create_wiring_tasks(&scan);
        let task_id = tasks[0].id;

        o.record_outcome(task_id, false, Some("compilation error: E0432".into()));
        assert_eq!(o.stats().wiring_failed, 1);
        assert_eq!(o.stats().wiring_attempted, 1);
        assert_eq!(
            o.wiring_tasks[0].last_error.as_deref(),
            Some("compilation error: E0432")
        );

        // Second failure should exceed max attempts
        o.record_outcome(task_id, false, Some("still failing".into()));
        assert_eq!(o.wiring_tasks[0].status, WiringStatus::CompilationFailed);
        assert_eq!(o.wiring_tasks[0].attempts, 2);
    }

    // ── Task filtering ──

    #[test]
    fn test_pending_tasks_filters_verified_and_failed() {
        let mut o = make_orchestrator();
        let scan = GapScanResult {
            scanned_modules: vec![],
            wired_modules: vec![],
            unwired_modules: vec!["mod_a".to_string(), "mod_b".to_string(), "mod_c".to_string()],
            scan_cycle: 1,
        };
        let tasks = o.create_wiring_tasks(&scan);
        let id_a = tasks[0].id;
        let id_b = tasks[1].id;

        assert_eq!(o.pending_tasks().len(), 3);

        o.record_outcome(id_a, true, None);
        assert_eq!(o.pending_tasks().len(), 2);

        o.record_outcome(id_b, false, Some("fail".into()));
        // mod_b had 1 failure, max_auto_attempts=3, so still pending
        assert_eq!(o.pending_tasks().len(), 2);
    }

    // ── Summary ──

    #[test]
    fn test_summary_contains_key_information() {
        let mut o = make_orchestrator();
        let scan = GapScanResult {
            scanned_modules: vec![],
            wired_modules: vec![],
            unwired_modules: vec!["test_mod".to_string()],
            scan_cycle: 1,
        };
        o.create_wiring_tasks(&scan);

        let summary = o.summary();
        assert!(summary.contains("SelfEvolutionTaskOrchestrator:"));
        assert!(summary.contains("tasks_created="));
        assert!(summary.contains("auto_exec="));
        assert!(summary.contains("coverage="));
    }

    // ── Edge cases ──

    #[test]
    fn test_record_outcome_unknown_id_is_noop() {
        let mut o = make_orchestrator();
        o.record_outcome(999, true, None);
        assert_eq!(o.stats().wiring_attempted, 0);
    }

    #[test]
    fn test_mark_compilation_verified() {
        let mut o = make_orchestrator();
        let scan = GapScanResult {
            scanned_modules: vec![],
            wired_modules: vec![],
            unwired_modules: vec!["my_module".to_string()],
            scan_cycle: 1,
        };
        let tasks = o.create_wiring_tasks(&scan);
        let id = tasks[0].id;

        o.mark_compilation_verified(id);
        assert_eq!(o.wiring_tasks[0].status, WiringStatus::Verified);
        assert_eq!(o.stats().compilation_verified, 1);
    }
}
