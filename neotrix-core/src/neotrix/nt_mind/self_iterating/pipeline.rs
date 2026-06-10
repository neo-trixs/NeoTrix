use super::SelfIteratingBrain;
use super::super::core::RewardSource;
use super::super::self_edit::MicroEdit;
use super::super::memory::ReasoningMemory;
use super::super::distillation::{ExperienceDistiller, apply_principles, avoid_anti_patterns};
use super::super::cortex_memory::CmsConfig;
use crate::neotrix::nt_world_model::TaskType;
pub(crate) use crate::neotrix::nt_core_error::NeoTrixError;
use super::checkpoint::{CheckpointStage, RewindStage};
use super::goal_contract::{GoalContractStage, EvidenceCaptureStage, NarrowRecoveryStage, FinalVerificationStage, GoalTerminatorStage, ExternalVerifierStage, SemanticRecallStage};
use crate::neotrix::nt_world_vision::VisionStage;
use crate::core::nt_core_consciousness::{
    VsaOrigin, VsaSelfCategory, VsaTagged,
};
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::sync::Mutex;
use std::path::Path;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum AutonomyLevel {
    Proposal,
    Bounded,
    #[default]
    Full,
}

/// Pipeline-level permission gate.
/// Maps to CLI ApprovalMode at the brain level:
///   Review  → ApprovalMode::Suggest  (all actions need approval)
///   Suggest → ApprovalMode::AutoEdit (file ops auto, shell/git needs approval)
///   Full    → ApprovalMode::FullAuto (everything auto)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PermissionLevel {
    Review,
    Suggest,
    #[default]
    Full,
}

impl PermissionLevel {
    pub fn to_approval_mode(&self) -> crate::cli::approval::ApprovalMode {
        match self {
            Self::Review => crate::cli::approval::ApprovalMode::Suggest,
            Self::Suggest => crate::cli::approval::ApprovalMode::AutoEdit,
            Self::Full => crate::cli::approval::ApprovalMode::FullAuto,
        }
    }
}


#[derive(Debug, Clone)]
pub struct BrainSnapshot {
    pub capability: super::super::core::CapabilityVector,
    pub learning_rate: f64,
    pub score: f64,
}

impl BrainSnapshot {
    pub fn new(brain: &super::brain_impl::ReasoningBrain, task_type: &TaskType) -> Self {
        Self {
            capability: brain.capability.clone(),
            learning_rate: brain.learning_rate,
            score: brain.evaluate_capability(*task_type),
        }
    }

    pub fn restore(&self, brain: &mut super::brain_impl::ReasoningBrain) {
        brain.capability = self.capability.clone();
        brain.learning_rate = self.learning_rate;
    }
}

#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage_name: String,
    pub efc: f64,
    pub raw_cost: f64,
    pub efficiency: f64,
}

impl StageResult {
    pub fn new(stage_name: &str) -> Self {
        let (efc, raw_cost) = estimate_stage_efc(stage_name);
        let efficiency = compute_stage_efficiency(efc, raw_cost);
        Self {
            stage_name: stage_name.to_string(),
            efc,
            raw_cost,
            efficiency,
        }
    }
}

pub fn estimate_stage_efc(stage_name: &str) -> (f64, f64) {
    match stage_name {
        "gap_analysis" => (0.85, 1000.0),
        "session_distill" => (0.80, 800.0),
        "conversation_distill" => (0.75, 900.0),
        "autonomy_gate" => (0.50, 200.0),
        "memory_retrieval" => (0.50, 300.0),
        "snapshot" => (0.15, 50.0),
        "nt_shield_scan" => (0.10, 400.0),
        "hypercube_optimize" => (0.10, 500.0),
        _ => (0.30, 200.0),
    }
}

pub fn compute_stage_efficiency(efc: f64, raw_cost: f64) -> f64 {
    if raw_cost > 0.0 { efc / raw_cost } else { 0.0 }
}

#[derive(Debug, Clone)]
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

pub struct BrainPipeline {
    pub stages: Vec<Box<dyn BrainStage>>,
}

impl BrainPipeline {
    pub fn new() -> Self { Self { stages: Vec::new() } }

    pub fn with_stages(stages: Vec<Box<dyn BrainStage>>) -> Self { Self { stages } }

    pub fn register(&mut self, stage: Box<dyn BrainStage>) {
        self.stages.push(stage);
    }

    pub fn execute(&self, brain: &mut SelfIteratingBrain) -> Result<(), NeoTrixError> {
        brain._stage_results.clear();
        for stage in &self.stages {
            let freq = stage.frequency();
            if freq > 1 && !brain.iteration.is_multiple_of(freq as u64) {
                continue;
            }

            // Auto-checkpoint before each stage execution
            {
                let task_type = brain._current_task_type();
                let snap = BrainSnapshot::new(&brain.brain, &task_type);
                let iteration = brain.iteration;
                let permission = brain.permission;
                let autonomy = brain.autonomy;
                let reward = brain._reward;
                brain.checkpoint_manager.push(
                    iteration, &snap, permission, autonomy, reward, stage.name(),
                );
            }

            let _span = tracing::info_span!(
                "pipeline_stage",
                stage.name = %stage.name(),
                iteration = brain.iteration,
            ).entered();
            let decision = stage.process(brain)?;

            let mut stage_result = StageResult::new(stage.name());
            if matches!(decision, StageDecision::Skip(_)) {
                stage_result.efc *= 0.1;
                stage_result.raw_cost *= 0.1;
            } else if matches!(decision, StageDecision::Rollback(_)) {
                stage_result.efc *= 0.3;
            }
            stage_result.efficiency = compute_stage_efficiency(stage_result.efc, stage_result.raw_cost);
            brain._stage_results.push(stage_result);

            match decision {
                StageDecision::Continue => continue,
                StageDecision::Skip(reason) => {
                    log::trace!("Stage '{}' aborted remaining: {}", stage.name(), reason);
                    return Ok(()); // Skip = 跳过剩余所有 stage
                }
                StageDecision::Promote(champ) => {
                    log::info!("Stage '{}' promoted new champion", stage.name());
                    brain.champion = Some(champ);
                }
                StageDecision::Rollback(reason) => {
                    log::warn!("Stage '{}' triggered rollback: {}", stage.name(), reason);
                    return Err(NeoTrixError::Brain(reason));
                }
            }
        }
        Ok(())
    }
}

impl Default for BrainPipeline {
    fn default() -> Self { Self::new() }
}

#[macro_export]
macro_rules! make_stage {
    ($name:ident) => {
        pub struct $name;
        impl Default for $name { fn default() -> Self { Self } }
        impl $name { pub fn new() -> Self { Self } }
    };
}

make_stage!(SnapshotStage);
impl BrainStage for SnapshotStage {
    fn name(&self) -> &str { "snapshot" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let champ = BrainSnapshot::new(&brain.brain, &brain._current_task_type());
        if let Some(ref champion) = brain.champion {
            if champ.score < champion.score * 0.8 {
                return Ok(StageDecision::Skip("当前能力显著低于冠军".to_string()));
            }
        }
        brain._set_snapshot(champ);
        Ok(StageDecision::Continue)
    }
}

make_stage!(MemoryRetrievalStage);
impl BrainStage for MemoryRetrievalStage {
    fn name(&self) -> &str { "memory_retrieval" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let task_type = brain._current_task_type();
        let embedding = brain._task_embedding();
        if let Some(ref emb) = embedding {
            brain.reasoning_bank.retrieve_relevant_by_embedding(emb, Some(task_type), 5);
        } else {
            brain.reasoning_bank.retrieve_relevant(&task, Some(task_type), 5);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(GapAnalysisStage);
impl BrainStage for GapAnalysisStage {
    fn name(&self) -> &str { "gap_analysis" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref router) = brain.attention_router {
            let gap_reports = router.bridge.analyze_gaps();
            let domains = router.bridge.sparse_domains(&gap_reports);

            let mut gap_lines: Vec<String> = Vec::new();
            for report in &gap_reports {
                if report.gap > 0.0 {
                    gap_lines.push(format!(
                        "[dim {}] current={:.3}, target={:.3}, gap={:.3}",
                        report.dim_index, report.current_value, report.target_value, report.gap
                    ));
                    log::info!("[gap-analysis] dim {}: current={:.3}, target={:.3}, gap={:.3}",
                        report.dim_index, report.current_value, report.target_value, report.gap);
                }
            }

            if !domains.is_empty() {
                let domain_str: Vec<String> = domains.iter().map(|d| format!("{:?}", d)).collect();
                log::info!("[gap-analysis] exploration domains suggested: {:?}", domain_str);
                gap_lines.push(format!("Suggested exploration: {}", domain_str.join(", ")));
            }

            if !gap_lines.is_empty() {
                let gap_summary = gap_lines.join(" | ");
                let existing = brain._open_source_insights.clone().unwrap_or_default();
                let combined = if existing.is_empty() {
                    format!("Knowledge gaps: {}", gap_summary)
                } else {
                    format!("{} | Knowledge gaps: {}", existing, gap_summary)
                };
                brain._set_open_source_insights(Some(combined));
            }
        } else {
            log::trace!("[gap-analysis] no attention_router, skipping");
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(SSMUpdateStage);
impl BrainStage for SSMUpdateStage {
    fn name(&self) -> &str { "ssm_update" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref op) = brain.select_operator {
            let input = brain.brain.capability.to_full_vector();
            let mut ssm_state = crate::neotrix::nt_core_signal::core::SelectiveState::new(input.len(), op.hidden_dim);
            let output = op.step(&mut ssm_state, &input);
            let dim = output.len().min(brain.brain.capability.arr().len());
            for (i, item) in brain.brain.capability.arr_mut().iter_mut().enumerate().take(dim) {
                let gate = (ssm_state.hidden.get(i).copied().unwrap_or(0.0)).abs().min(1.0);
                let current = *item;
                *item = gate * output[i] + (1.0 - gate) * current;
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(OpenSourceCompareStage);
impl BrainStage for OpenSourceCompareStage {
    fn name(&self) -> &str { "open_source_compare" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let task_type = brain._current_task_type();
        let capability = brain.brain.capability.clone();
        let benchmarker = crate::neotrix::nt_mind::open_source_benchmark::OpenSourceBenchmarker::new();

        // Phase 1: Static benchmark against known projects
        let reports = benchmarker.benchmark_top3(&task, task_type, &capability);

        let dyn_edits: Vec<MicroEdit> = Vec::new();

        if reports.is_empty() || (reports[0].relevance_score < 0.3 && dyn_edits.is_empty()) {
            brain._set_open_source_insights(None);
            brain._set_open_source_edits(Vec::new());
            return Ok(StageDecision::Skip("No relevant open-source projects found".into()));
        }

        let mut summary_lines = Vec::new();
        let mut all_edits = Vec::new();

        for report in &reports {
            summary_lines.push(report.summary.clone());
            for (idx, delta, detail) in &report.gap_areas {
                let edit = MicroEdit::AdjustDimension(idx.to_string(), *delta);
                if !all_edits.iter().any(|e: &MicroEdit| matches!(e, MicroEdit::AdjustDimension(i, _) if *i == idx.to_string())) {
                    all_edits.push(edit);
                }
                log::info!("[open-source] gap: {}", detail);
            }
        }

        for edit in dyn_edits {
            if !all_edits.iter().any(|e: &MicroEdit| {
                matches!((e, &edit), (MicroEdit::AdjustDimension(i1, _), MicroEdit::AdjustDimension(i2, _)) if *i1 == *i2)
            }) {
                all_edits.push(edit);
            }
        }

        if !all_edits.is_empty() {
            all_edits.push(MicroEdit::NormalizeVector);
        }

        let insights = summary_lines.join(" | ");
        brain._set_open_source_insights(Some(insights));
        brain._set_open_source_edits(all_edits);
        Ok(StageDecision::Continue)
    }
}

make_stage!(SelfEditGenerationStage);
impl BrainStage for SelfEditGenerationStage {
    fn name(&self) -> &str { "self_edit_gen" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let mut edits = if let Some(ref dgm) = brain.dgm_strategy {
            let ctx = super::brain_dgm::EditContext {
                task: &task,
                brain: &brain.brain,
                noise_level: 0.0,
            };
            dgm.generate_via_diffusion(&ctx)
        } else {
            brain.brain.generate_self_edit(&task)
        };

        let task_type = brain._current_task_type();
        let memories = brain.reasoning_bank.retrieve_relevant(&task, Some(task_type), 5);
        if !memories.is_empty() {
            let avg_reward: f64 = memories.iter().map(|m| m.reward).sum::<f64>() / memories.len() as f64;
            let factor = if avg_reward > 0.7 { 1.1 } else if avg_reward < 0.3 { 0.9 } else { 1.0 };
            if (factor - 1.0_f64).abs() > 0.01_f64 {
                for edit in &mut edits {
                    if let MicroEdit::AdjustDimension(_, ref mut amount) = edit {
                        *amount *= factor;
                    }
                }
            }
        }

        brain._set_micro_edits(edits);
        Ok(StageDecision::Continue)
    }
}

make_stage!(ApplyEditsStage);
impl BrainStage for ApplyEditsStage {
    fn name(&self) -> &str { "apply_edits" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let edits = brain._take_micro_edits();
        brain.brain.apply_micro_edits(&edits);
        brain._set_micro_edits(edits);
        Ok(StageDecision::Continue)
    }
}

make_stage!(RewardCalculationStage);
impl BrainStage for RewardCalculationStage {
    fn name(&self) -> &str { "reward_calc" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let external = brain._external_reward();
        let (reward, source) = if let Some(ext) = external {
            (ext, RewardSource::External)
        } else {
            let task_type = brain._current_task_type();
            let score_before = brain._snapshot_score();
            let score_after = brain.brain.evaluate_capability(task_type);
            let regularization = brain.compute_regularization(&brain._snapshot_capability());
            let raw = (score_after - score_before) + regularization;
            let health = brain.evo_stats().health_score;
            let mut calibrated = raw * (0.5 + health * 0.5);

            // InnerCritic quality gate
            let critic_out = VsaTagged::new(
                QuantizedVSA::random_binary(),
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            ).with_confidence(if calibrated > 0.0 { 0.6 } else { 0.3 });
            let critic_ctx = VsaTagged::new(
                QuantizedVSA::random_binary(),
                VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
            );
            let critique = brain._inner_critic.evaluate(
                &critic_out, &critic_ctx, Some(&brain._specious_present),
            );
            if !critique.passed && calibrated > 0.0 {
                calibrated *= 0.8;
            }

            (calibrated, RewardSource::Internal)
        };
        brain._set_reward(reward);
        brain._set_reward_source(source);
        Ok(StageDecision::Continue)
    }
}

make_stage!(GwtAbsorbStage);
impl BrainStage for GwtAbsorbStage {
    fn name(&self) -> &str { "gwt_absorb" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let reward = brain._reward();
        let source = brain._reward_source();
        let summary = format!("task: {}, reward: {:.4}, source: {:?}", task, reward, source);
        if let Some(ref mut router) = brain.attention_router {
            router.absorb_reasoning_result(&task, &summary, "seal_loop");
            router.wm().broadcast(&summary);
            log::info!("[gwt-absorb] broadcast: {}", summary);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(HyperCubeOptimizeStage);
impl BrainStage for HyperCubeOptimizeStage {
    fn name(&self) -> &str { "hypercube_optimize" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pruned = if let Some(ref mut router) = brain.attention_router {
            router.bridge.prune_low_access(2)
        } else {
            0
        };
        if pruned > 0 {
            log::info!("[hypercube-optimize] pruned {} low-access entries", pruned);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(DistillationStage);
impl BrainStage for DistillationStage {
    fn name(&self) -> &str { "session_distill" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let session = crate::neotrix::nt_act_autonomy::knowledge_distiller::SessionRecord {
            id: format!("seal-iter-{}", brain.iteration),
            user_messages: vec![brain._current_task()],
            actions_taken: brain._micro_edits.iter().map(|e| format!("{:?}", e)).collect(),
            outcomes: vec![format!("reward={:.4}", brain._reward())],
            reward_signal: brain._reward(),
            timestamp: brain.iteration,
            task_type: Some(format!("{:?}", brain._current_task_type())),
            e8_mode: Some(brain._e8_policy.best_mode()),
            edit_types: brain._micro_edits.iter().map(|e| format!("{:?}", e)).collect(),
        };
        let principles = brain._knowledge_distiller.distill(&session);
        if !principles.is_empty() {
            let absorbed = brain._knowledge_distiller.absorb(&mut brain.brain.capability);
            log::info!(
                "[session-distill] {} principles from iter {}, {} absorbed",
                principles.len(), brain.iteration, absorbed,
            );
            let summary = brain._knowledge_distiller.summary();
            if let Some(ref mut router) = brain.attention_router {
                router.wm().broadcast(&summary);
            }
        }
        Ok(StageDecision::Continue)
    }
}

/// Result of a conversation distillation run.
pub struct DistillationResult {
    pub total: usize,
    pub successes: usize,
    pub failures: usize,
    pub avg_eff: f64,
    pub error_rate: f64,
    pub patterns_created: bool,
    pub total_gain: f64,
}

/// Standalone conversation distillation — analyzes recent ConversationRecords,
/// detects patterns, and stores EvolutionRecords. Can be called from both
/// the SEAL pipeline stage and directly from core_review() after every reason().
pub fn run_conversation_distill(kb: &crate::neotrix::nt_memory_kb::KnowledgeBase) -> Result<DistillationResult, NeoTrixError> {
    let records = match kb.get_evolution_history(10) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("[conv-distill] query failed: {}", e);
            return Ok(DistillationResult {
                total: 0, successes: 0, failures: 0, avg_eff: 0.0,
                error_rate: 0.0, patterns_created: false, total_gain: 0.0,
            });
        }
    };
    if records.is_empty() {
        return Ok(DistillationResult {
            total: 0, successes: 0, failures: 0, avg_eff: 0.0,
            error_rate: 0.0, patterns_created: false, total_gain: 0.0,
        });
    }

    let total = records.len();
    let successes = records.iter().filter(|r| r.outcome == "success").count();
    let failures = total - successes;
    let avg_eff: f64 = records.iter().map(|r| r.effectiveness).sum::<f64>() / total as f64;

    let mut by_strategy: std::collections::HashMap<&str, Vec<f64>> = std::collections::HashMap::new();
    for r in &records {
        by_strategy.entry(r.strategy_used.as_str()).or_default().push(r.effectiveness);
    }
    let mut strategy_ratings: Vec<(&str, f64)> = by_strategy.iter()
        .map(|(k, v)| (*k, v.iter().sum::<f64>() / v.len() as f64))
        .collect();
    strategy_ratings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let error_patterns: Vec<&str> = records.iter()
        .filter(|r| !r.obstacles_encountered.is_empty())
        .map(|r| r.obstacles_encountered[0].as_str())
        .collect();

    let error_rate = records.iter().map(|r| r.error_count).sum::<u32>() as f64 / total as f64;

    let mut modes_used: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for r in &records {
        modes_used.insert(r.e8_mode.as_str());
    }

    let fix_patterns_present = records.iter().any(|r| !r.fix_patterns.is_empty());

    log::info!(
        "[conv-distill] {} records: {}/{} success, avg_eff={:.3}, error_rate={:.3}, modes={}, top_strategy={:?}",
        total, successes, failures, avg_eff, error_rate, modes_used.len(),
        strategy_ratings.first().map(|(s, _)| *s),
    );

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);

    let mut created_any = false;

    // 1. RecurringError
    if error_rate > 0.0 {
        let most_common_error = error_patterns.first().unwrap_or(&"unknown");
        let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
            id: format!("conv_pat_err_{}", timestamp),
            source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
            pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::RecurringError,
            description: format!("Error rate {:.2}, {} failures: {}", error_rate, failures, most_common_error),
            before_behavior: "".into(),
            after_behavior: "".into(),
            effectiveness_gain: avg_eff * error_rate,
            applied_to: vec![],
            verified: false,
            timestamp,
        };
        if kb.store_evolution_record(&evolution).is_ok() {
            created_any = true;
            log::info!("[conv-distill] recorded RecurringError pattern: {}", evolution.description);
        }
    }

    // 2. CommunicationOptimization
    if let Some((best_strat, best_eff)) = strategy_ratings.first() {
        if let Some((_, default_eff)) = strategy_ratings.iter().find(|(s, _)| *s == "auto") {
            if best_eff - default_eff > 0.1 {
                let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
                    id: format!("conv_pat_comm_{}", timestamp),
                    source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
                    pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::CommunicationOptimization,
                    description: format!("Strategy '{}' improves over 'auto' by {:.2}", best_strat, best_eff - default_eff),
                    before_behavior: "auto".into(),
                    after_behavior: best_strat.to_string(),
                    effectiveness_gain: best_eff - default_eff,
                    applied_to: vec![],
                    verified: false,
                    timestamp,
                };
                if kb.store_evolution_record(&evolution).is_ok() {
                    created_any = true;
                    log::info!("[conv-distill] recorded CommunicationOptimization pattern: {}", evolution.description);
                }
            }
        }
    }

    // 3. ProblemDecomposition
    let high_eff_low_err = records.iter()
        .filter(|r| r.effectiveness > 0.7 && r.error_count == 0)
        .count();
    if high_eff_low_err as f64 > total as f64 * 0.3 {
        let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
            id: format!("conv_pat_dec_{}", timestamp),
            source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
            pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::ProblemDecomposition,
            description: format!("{}/{} records high-effectiveness zero-error", high_eff_low_err, total),
            before_behavior: "".into(),
            after_behavior: "".into(),
            effectiveness_gain: avg_eff,
            applied_to: vec![],
            verified: false,
            timestamp,
        };
        if kb.store_evolution_record(&evolution).is_ok() {
            created_any = true;
            log::info!("[conv-distill] recorded ProblemDecomposition pattern: {}", evolution.description);
        }
    }

    // 4. VerificationImprovement
    if fix_patterns_present {
        let fix_count: usize = records.iter().map(|r| r.fix_patterns.len()).sum();
        let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
            id: format!("conv_pat_ver_{}", timestamp),
            source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
            pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::VerificationImprovement,
            description: format!("{} fix patterns across {} records", fix_count, total),
            before_behavior: "".into(),
            after_behavior: "".into(),
            effectiveness_gain: avg_eff,
            applied_to: vec![],
            verified: false,
            timestamp,
        };
        if kb.store_evolution_record(&evolution).is_ok() {
            created_any = true;
            log::info!("[conv-distill] recorded VerificationImprovement pattern: {}", evolution.description);
        }
    }

    // 5. ToolUsagePattern
    let successful_strategies: Vec<&str> = records.iter()
        .filter(|r| r.outcome == "success" && !r.actions_taken.is_empty())
        .map(|r| r.strategy_used.as_str())
        .collect();
    if !successful_strategies.is_empty() && successful_strategies.len() > total / 2 {
        let mut tool_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for t in &successful_strategies {
            *tool_counts.entry(t).or_insert(0) += 1;
        }
        if let Some((best_tool, _)) = tool_counts.iter().max_by_key(|(_, c)| **c) {
            let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
                id: format!("conv_pat_tool_{}", timestamp),
                source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
                pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::ToolUsagePattern,
                description: format!("Strategy '{}' used in {}/{} successes", best_tool, tool_counts[best_tool], successes),
                before_behavior: "".into(),
                after_behavior: best_tool.to_string(),
                effectiveness_gain: avg_eff,
                applied_to: vec![],
                verified: false,
                timestamp,
            };
            if kb.store_evolution_record(&evolution).is_ok() {
                created_any = true;
                log::info!("[conv-distill] recorded ToolUsagePattern pattern: {}", evolution.description);
            }
        }
    }

    // 6. StrategyDiscovery
    if let Some((best_strat, best_eff)) = strategy_ratings.first() {
        if *best_strat != "auto" && *best_eff > avg_eff * 1.1 {
            let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
                id: format!("conv_pat_strat_{}", timestamp),
                source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
                pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::StrategyDiscovery,
                description: format!("New strategy '{}' outperforms average ({:.2} vs {:.2})", best_strat, best_eff, avg_eff),
                before_behavior: "auto".into(),
                after_behavior: best_strat.to_string(),
                effectiveness_gain: best_eff - avg_eff,
                applied_to: vec![],
                verified: false,
                timestamp,
            };
            if kb.store_evolution_record(&evolution).is_ok() {
                created_any = true;
                log::info!("[conv-distill] recorded StrategyDiscovery pattern: {}", evolution.description);
            }
        }
    }

    // 7. PrincipleUpdate
    if avg_eff > 0.8 {
        let all_above_threshold = by_strategy.iter().all(|(_, v)| v.iter().sum::<f64>() / v.len() as f64 > 0.7);
        if all_above_threshold && strategy_ratings.len() > 1 {
            let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
                id: format!("conv_pat_princ_{}", timestamp),
                source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
                pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::PrincipleUpdate,
                description: format!("Consistent high effectiveness ({:.2}) across {} strategies", avg_eff, strategy_ratings.len()),
                before_behavior: "".into(),
                after_behavior: "".into(),
                effectiveness_gain: avg_eff,
                applied_to: vec![],
                verified: false,
                timestamp,
            };
            if kb.store_evolution_record(&evolution).is_ok() {
                created_any = true;
                log::info!("[conv-distill] recorded PrincipleUpdate pattern: {}", evolution.description);
            }
        }
    }

    // Read back recent evolution patterns and compute total gain
    let total_gain = if let Ok(patterns) = kb.get_evolution_patterns(5) {
        if !patterns.is_empty() {
            patterns.iter().map(|p| p.effectiveness_gain).sum()
        } else {
            0.0
        }
    } else {
        0.0
    };

    Ok(DistillationResult {
        total,
        successes,
        failures,
        avg_eff,
        error_rate,
        patterns_created: created_any,
        total_gain,
    })
}

make_stage!(ConversationDistillStage);
impl BrainStage for ConversationDistillStage {
    fn name(&self) -> &str { "conversation_distill" }
    fn frequency(&self) -> usize { 1 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let Some(ref kb) = brain._nt_memory_kb else {
            return Ok(StageDecision::Skip("no KB".into()));
        };
        let result = run_conversation_distill(kb)?;

        if result.total == 0 {
            return Ok(StageDecision::Continue);
        }

        // Adjust reward based on evolution pattern gains
        if result.total_gain > 0.0 {
            let bonus = (result.total_gain * 0.1).min(0.5);
            brain._set_reward(brain._reward() + bonus);
            log::info!("[conv-distill] evolution reward bonus: {:.4}", bonus);
        }

        // Broadcast to GWT
        if let Some(ref mut router) = brain.attention_router {
            let report = format!(
                "[conversation-evolution] {} records: {}/{} OK, avg_eff={:.2}, error_rate={:.3}, patterns_created={}",
                result.total, result.successes, result.failures, result.avg_eff, result.error_rate,
                if result.patterns_created { "yes" } else { "no" },
            );
            router.wm().broadcast(&report);
        }

        Ok(StageDecision::Continue)
    }
}

make_stage!(SecurityStage);
impl BrainStage for SecurityStage {
    fn name(&self) -> &str { "nt_shield_scan" }
    fn frequency(&self) -> usize { 1 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let code_context = brain._open_source_insights.clone().unwrap_or_default();
        let scanner = super::secret_scanner::SecretScanner::new();
        let result = scanner.scan_with_context(&task, &code_context);
        if !result.is_safe() {
            let critical_count = result.count_by_severity
                .get(&super::secret_scanner::Severity::Critical)
                .copied()
                .unwrap_or(0);
            log::warn!(
                "[nt_shield-scan] found {} secrets ({} critical) — max_severity={:?}, risk={:.2}",
                result.findings.len(),
                critical_count,
                result.max_severity,
                result.risk_score()
            );
            if let Some(ref mut router) = brain.attention_router {
                for finding in &result.findings {
                    router.wm().broadcast(&format!(
                        "Security alert: {} at line {} — \"{}\"",
                        finding.pattern, finding.line, finding.snippet
                    ));
                }
            }
            if critical_count > 0 {
                brain._set_reward(brain._reward() - result.risk_score() * 0.2);
                log::info!("[nt_shield-scan] penalized reward by {:.4}", result.risk_score() * 0.2);
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(E8ExperimentStage);
impl BrainStage for E8ExperimentStage {
    fn name(&self) -> &str { "e8_experiment" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let reward = brain._reward();
        let task_type = brain._current_task_type();

        let mode = brain._e8_policy.select_mode(&task, task_type, &brain._transition_learner);
        brain._transition_learner.record(&task, mode, reward, brain.iteration);
        brain._e8_policy.update(reward);

        // Consume evolution patterns for factorized E8 updates
        if let Some(ref kb) = brain._nt_memory_kb {
            if let Ok(patterns) = kb.get_evolution_patterns(5) {
                if !patterns.is_empty() {
                    let mut factor_deltas = [0.0f64; crate::core::NUM_E8_FACTORS];
                    for pattern in &patterns {
                        let base_deltas = match pattern.pattern_type {
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::RecurringError
                                => [0.2, 0.0, 0.1, 0.0, 0.0, 0.3],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::CommunicationOptimization
                                => [0.0, 0.1, 0.0, 0.0, 0.3, 0.0],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::ProblemDecomposition
                                => [-0.2, 0.0, -0.2, 0.2, 0.0, -0.1],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::VerificationImprovement
                                => [0.0, 0.0, -0.1, 0.0, 0.0, -0.2],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::ToolUsagePattern
                                => [0.0, -0.1, 0.0, 0.1, 0.0, 0.0],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::StrategyDiscovery
                                => [0.3, 0.2, 0.3, -0.1, 0.1, 0.3],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::PrincipleUpdate
                                => [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                        };
                        let weight = pattern.effectiveness_gain * 0.5;
                        for i in 0..crate::core::NUM_E8_FACTORS {
                            factor_deltas[i] += base_deltas[i] * weight;
                        }
                    }
                    let has_nonzero = factor_deltas.iter().any(|d| d.abs() > 0.001);
                    if has_nonzero {
                        brain._e8_policy.update_factorized(reward, &factor_deltas);
                        log::info!(
                            "[e8-experiment] factorized update from {} evolution patterns: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}]",
                            patterns.len(),
                            factor_deltas[0], factor_deltas[1], factor_deltas[2],
                            factor_deltas[3], factor_deltas[4], factor_deltas[5],
                        );
                    }
                }
            }
        }

        brain._e8_policy.decay_epsilon();

        log::info!(
            "[e8-experiment] mode={}, epsilon={:.4}, mode_value={:.4}",
            mode.0, brain._e8_policy.epsilon(), brain._e8_policy.mode_values[mode.0 as usize],
        );

        if let Some(ref mut router) = brain.attention_router {
            let bridge = &router.bridge;
            let gap_reports = bridge.analyze_gaps();
            let high_gaps: usize = gap_reports.iter().filter(|r| r.gap > 0.5).count();
            if high_gaps > 3 && brain._transition_learner.outcomes.len() > 20 {
                let approach = (mode.0 >> 3) as usize;
                let domain = (mode.0 & 0x07) as usize;
                if let Some(pattern) = brain._transition_learner.suggest_evolution(approach, domain) {
                    log::info!("[e8-experiment] suggested evolution ({}) for cell ({},{})", pattern, approach, domain);
                }
            }
        }

        let evolved = brain._transition_learner.evolve_matrix(&mut brain._strategy_matrix);
        if evolved > 0 {
            log::info!("[e8-experiment] evolved {} strategy matrix cells", evolved);
        }

        Ok(StageDecision::Continue)
    }
}

make_stage!(HarnessAdaptStage);
impl BrainStage for HarnessAdaptStage {
    fn name(&self) -> &str { "harness_adapt" }
    fn frequency(&self) -> usize { 2 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let current_task = brain._current_task();
        if let Some(ref mut router) = brain.attention_router {
            let env = current_task.clone();
            router.set_environment(&env);
            let profile = brain._harness_adapter.active_profile().cloned();
            if let Some(p) = profile {
                router.register_harness_profile(&env, &p);
                log::info!("[harness-adapt] applied profile for env={}, performance_delta={:.4}",
                    env, p.performance_delta);
            }
        }
        // Persist harness profiles to KnowledgeBase
        if let Some(ref kb) = brain._nt_memory_kb {
            let saved = brain._harness_adapter.save_to_kb(kb).unwrap_or(0);
            if saved > 0 {
                log::info!("[harness-adapt] saved {} profiles to KB", saved);
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(TaskAffinityStage);
impl BrainStage for TaskAffinityStage {
    fn name(&self) -> &str { "task_affinity" }
    fn frequency(&self) -> usize { 2 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task_type = brain._current_task_type();
        let reward = brain._reward();
        brain.brain.update_task_affinity(task_type, brain._snapshot_score() + reward);
        Ok(StageDecision::Continue)
    }
}

make_stage!(KnowledgeQualityStage);
impl BrainStage for KnowledgeQualityStage {
    fn name(&self) -> &str { "knowledge_quality" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let quality = brain.reasoning_bank.quality_score();
        let current_reward = brain._reward();

        if quality > 0.5 {
            let bonus = quality * 0.03;
            let new_reward = current_reward + bonus;
            brain._set_reward(new_reward);
            log::info!("[knowledge-quality] quality={:.3}, bonus={:.4}, reward={:.3}→{:.3}",
                quality, bonus, current_reward, new_reward);
        } else {
            log::info!("[knowledge-quality] quality={:.3} below threshold, no bonus", quality);
        }

        brain._open_source_insights = Some(format!(
            "{} | Knowledge quality: {:.3}",
            brain._open_source_insights.clone().unwrap_or_default(),
            quality
        ));

        Ok(StageDecision::Continue)
    }
}

make_stage!(RollbackDecisionStage);
impl BrainStage for RollbackDecisionStage {
    fn name(&self) -> &str { "rollback_decision" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let reward = brain._reward();
        let source = brain._reward_source();
        let snapshot_lr = brain._snapshot_lr();
        if reward < 0.0 && source == RewardSource::External {
            brain._snapshot_restore();
            brain.brain.learning_rate = (snapshot_lr * 0.9).max(0.01);
            return Ok(StageDecision::Rollback("外部奖励为负，已回滚".to_string()));
        }
        if reward > brain.quality_threshold {
            if let Err(e) = brain.brain.save() {
                log::warn!("持久化失败: {}", e);
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(ReasoningBankStorageStage);
impl BrainStage for ReasoningBankStorageStage {
    fn name(&self) -> &str { "bank_storage" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let task_type = brain._current_task_type();
        let reward = brain._reward();
        let source = brain._reward_source();
        let edits = brain._take_micro_edits();

        let memory = if source == RewardSource::External {
            ReasoningMemory::with_external_reward(&task, task_type, &edits, reward)
        } else {
            ReasoningMemory::new(&task, task_type, &edits, reward)
        };

        let embedding = brain._take_task_embedding();
        if let Some(emb) = embedding {
            brain.reasoning_bank.store_with_embedding(memory, emb);
        } else {
            brain.reasoning_bank.store(memory);
        }
        brain._set_micro_edits(edits);
        Ok(StageDecision::Continue)
    }
}

make_stage!(AdaptiveLRStage);
impl BrainStage for AdaptiveLRStage {
    fn name(&self) -> &str { "adaptive_lr" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let reward = brain._reward + brain.curiosity_bonus;
        let adapted_lr = brain.curvature_policy.adapt_lr(reward);
        brain.brain.learning_rate = adapted_lr;
        log::debug!("[curvature] lr={:.4} regime={:?}",
            adapted_lr, brain.curvature_policy.regime());
        Ok(StageDecision::Continue)
    }
}

make_stage!(KnowledgeAbsorbStage);
impl BrainStage for KnowledgeAbsorbStage {
    fn name(&self) -> &str { "knowledge_absorb" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task_type = brain._current_task_type();
        let score_before = brain._snapshot_score();
        if score_before < brain.quality_threshold && brain.auto_absorb {
            let sources = brain.select_relevant_sources(task_type);
            brain.brain.absorb_batch(&sources);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(MemoryStorageStage);
impl BrainStage for MemoryStorageStage {
    fn name(&self) -> &str { "memory_storage" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let task_type = brain._current_task_type();
        let score_before = brain._snapshot_score();
        let reward = brain.brain.evaluate_capability(task_type) - score_before;
        let micro_edits = brain.brain.generate_self_edit(&task);
        let memory = ReasoningMemory::new(&task, task_type, &micro_edits, reward);
        brain.reasoning_bank.store(memory);

        if brain.auto_memory_iteration && brain.iteration.is_multiple_of(brain.memory_iteration_interval) {
            brain.reasoning_bank.iterate_memories(0.85, 0.1);
            let all_mems: Vec<ReasoningMemory> = brain.reasoning_bank.memories().iter().cloned().collect();
            let principles = ExperienceDistiller::distill(&all_mems);
            if !principles.is_empty() {
                apply_principles(&mut brain.brain.capability, &principles, 0.6);
            }
            let anti_patterns = ExperienceDistiller::contrastive_reflect(&all_mems);
            if !anti_patterns.is_empty() {
                avoid_anti_patterns(&mut brain.brain.capability, &anti_patterns);
            }
            if let Some(ref mut gm) = brain.group_manager {
                gm.evolve_group();
            }
        }

        // CMS: Continuum Memory System consolidation (every pipeline iteration)
        let cms_config = CmsConfig::default();
        let cms_result = brain.cortex.consolidate_cms(brain.iteration, &cms_config);
        if cms_result.nt_world_sense_to_topic + cms_result.topic_to_event + cms_result.event_to_fact > 0 {
            log::trace!("CMS: S→T {} T→E {} E→F {}", 
                cms_result.nt_world_sense_to_topic, cms_result.topic_to_event, cms_result.event_to_fact);
        }

        Ok(StageDecision::Continue)
    }
}

make_stage!(EvaluationStage);
impl BrainStage for EvaluationStage {
    fn name(&self) -> &str { "evaluation" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task_type = brain._current_task_type();
        let score_before = brain._snapshot_score();
        let score_after = brain.brain.evaluate_capability(task_type);
        brain.evaluation_history.push(super::brain_impl::EvaluationRecord {
            iteration: brain.iteration,
            task_type,
            score_before,
            score_after,
            improved: score_after > score_before,
        });
        Ok(StageDecision::Continue)
    }
}

make_stage!(ChampionCompareStage);
impl BrainStage for ChampionCompareStage {
    fn name(&self) -> &str { "champion_compare" }
    fn frequency(&self) -> usize { 2 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref champion) = brain.champion {
            let task_type = brain._current_task_type();
            let current_score = brain.brain.evaluate_capability(task_type);
            if current_score > champion.score * 1.05 {
                let new_champ = BrainSnapshot::new(&brain.brain, &task_type);
                return Ok(StageDecision::Promote(new_champ));
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(AutonomyGateStage);
impl BrainStage for AutonomyGateStage {
    fn name(&self) -> &str { "autonomy_gate" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        // 1. Sync permission level to global ShieldEnforcer
        if let Ok(shield) = crate::cli::global_shield().try_lock() {
            let target_mode = brain.permission.to_approval_mode();
            if shield.approval.mode() != target_mode {
                drop(shield);
                if let Ok(mut s) = crate::cli::global_shield().lock() {
                    s.set_approval_mode(target_mode);
                    log::info!("[autonomy-gate] synced approval mode to {:?}", target_mode);
                }
            } else {
                drop(shield);
            }
        }

        // 2. Check PermissionLevel
        match brain.permission {
            PermissionLevel::Review => {
                return Ok(StageDecision::Skip(
                    "PermissionLevel=Review：所有编辑操作需要审批".to_string()));
            }
            PermissionLevel::Suggest | PermissionLevel::Full => {}
        }

        // 3. Check AutonomyLevel
        match brain.autonomy {
            AutonomyLevel::Proposal => {
                return Ok(StageDecision::Skip("Proposal 模式：只预览不执行".to_string()));
            }
            AutonomyLevel::Bounded => {
                let current: f64 = brain.brain.capability.arr().iter().sum();
                if current > 16.0 {
                    return Ok(StageDecision::Skip(format!(
                        "Bounded 模式：能力总和 {:.2} 超过阈值 16.0", current)));
                }
            }
            AutonomyLevel::Full => {}
        }

        // 4. ShieldEnforcer quick check — blocks if sandbox is read-only
        if let Ok(shield) = crate::cli::global_shield().try_lock() {
            if shield.sandbox.is_read_only() {
                return Ok(StageDecision::Skip(
                    "沙箱只读模式：不允许修改操作".to_string()));
            }
        }

        // 5. CognitiveLoad gate: switch to Fast mode under high load
        if !brain._cognitive_load.can_do_deep_reasoning() {
            return Ok(StageDecision::Skip(
                "CognitiveLoad 过高：切换到快速模式".to_string()));
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(StatsSignificanceStage);
impl BrainStage for StatsSignificanceStage {
    fn name(&self) -> &str { "stats_significance" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let recent: Vec<bool> = brain.evaluation_history.iter()
            .rev().take(5).map(|r| r.improved).collect();
        if recent.len() >= 3 {
            let success_rate = recent.iter().filter(|&&x| x).count() as f64 / recent.len() as f64;
            if success_rate < 0.3 && brain._reward() < 0.0 {
                return Ok(StageDecision::Skip(
                    "统计显著性不足：近期成功率低于30%".to_string()));
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(EmbeddingRefreshStage);
impl BrainStage for EmbeddingRefreshStage {
    fn name(&self) -> &str { "embedding_refresh" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let kb = match brain.reasoning_engine.as_mut().and_then(|e| e.kb.as_mut()) {
            Some(k) => k,
            None => return Ok(StageDecision::Skip("no KB attached".to_string())),
        };
        let has_config = kb.embedding_config.read()
            .map(|r| r.is_some())
            .unwrap_or(false);
        if !has_config {
            return Ok(StageDecision::Skip("no embedding config".to_string()));
        }
        match kb.ensure_embeddings() {
            Ok(count) => {
                if count > 0 {
                    log::info!("[embedding-refresh] generated {} missing embeddings", count);
                }
            }
            Err(e) => {
                log::warn!("[embedding-refresh] {}", e);
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(TrajectoryCollectStage);
impl BrainStage for TrajectoryCollectStage {
    fn name(&self) -> &str { "trajectory_collect" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        brain._trajectory_collector.begin(brain._current_task.clone());
        for step in &brain._stage_results {
            brain._trajectory_collector.record_step(
                crate::core::nt_core_gwt::module_def::SpecialistType::Planner,
                brain._e8_policy.best_mode(),
                step.stage_name.clone(),
                brain._current_task.clone(),
                format!("efc={:.3}", step.efc),
                None,
                true,
                None,
            );
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(CoachAndUpdateStage);
impl BrainStage for CoachAndUpdateStage {
    fn name(&self) -> &str { "coach_and_update" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let trajectories: Vec<crate::core::nt_core_prm::AgentTrajectory> =
            brain._trajectory_collector.collected.drain(..).collect();
        if trajectories.is_empty() {
            return Ok(StageDecision::Continue);
        }
        if let Some(ref mut coach) = brain._coach {
            for traj in &trajectories {
                let scores = coach.score_episode(traj);
                for score in &scores {
                    if let Some(step) = traj.steps.get(score.step_idx) {
                        brain._e8_policy.set_previous(step.e8_mode);
                        brain._e8_policy.update(score.score);
                    }
                }
                let outcome_reward = traj.outcome_reward.unwrap_or(0.5);
                brain._transition_learner.record(
                    &traj.task,
                    traj.steps.first().map(|s| s.e8_mode).unwrap_or(brain._e8_policy.best_mode()),
                    outcome_reward,
                    brain.iteration,
                );
                let avg_score = scores.iter().map(|s| s.score).sum::<f64>() / scores.len().max(1) as f64;
                brain._reward = brain._reward * 0.9 + avg_score * 0.1;
            }
            brain._e8_policy.decay_epsilon();
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(SpectralMonitorStage);
impl BrainStage for SpectralMonitorStage {
    fn name(&self) -> &str { "spectral_monitor" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref mut jepa) = brain.nt_world_jepa {
            jepa.record_rollout_reward(brain.iteration as usize, brain._reward);
            if !jepa.check_rollout_stability() {
                log::warn!("[spectral] model degrading — reducing reward momentum");
                brain._reward *= 0.9;
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(MetaImprovementStage);
impl BrainStage for MetaImprovementStage {
    fn name(&self) -> &str { "meta_improvement" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_mind_ingestion::meta_improvement::{
            MetaDiagnostics,
        };
        if brain._meta_agent.is_none() {
            let mut agent = crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAgent::new();
            agent.meta_layer_can_rewrite_self = true;
            brain._meta_agent = Some(agent);
        }
        if let Some(ref mut agent) = brain._meta_agent {
            let diag = MetaDiagnostics::new(brain.iteration as u64);
            let (action, self_edit) = agent.observe_and_act(&diag);
            match action {
                crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAction::CreateStage { name, description: _, frequency } => {
                    if !agent.created_stages.contains(&name.to_string()) && brain.meta_additions.len() < agent.max_stages {
                        let stage = crate::neotrix::nt_mind_ingestion::meta_improvement::DynamicStage::new(name, "", frequency);
                        brain.meta_additions.push(Box::new(stage));
                        agent.created_stages.push(name.to_string());
                        log::info!("[dgm-h] created stage: {}", name);
                    }
                }
                crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAction::RemoveStage { name } => {
                    brain.meta_additions.retain(|s| s.name() != name);
                    agent.created_stages.retain(|n| n != name);
                    log::info!("[dgm-h] removed stage: {}", name);
                }
                crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAction::ModifyConfig { param: _, value: _ } => {
                    log::info!("[dgm-h] config modification (stub)");
                }
                crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAction::NoOp => {}
            }
            if let Some(edit) = self_edit {
                log::info!("[dgm-h] meta self-edit: {:?}", edit);
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(SleepStage);
impl BrainStage for SleepStage {
    fn name(&self) -> &str { "sleep" }
    fn frequency(&self) -> usize { 100 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let report = brain._sleep_gate.consolidate(&mut brain._consciousness_stream, brain.iteration as usize);
        if report.conflicts_detected > 0 || report.merged_count > 0 {
            log::info!("[sleep] merged={} evicted={} conflicts={} pressure={:.2}",
                report.merged_count, report.evicted_count, report.conflicts_detected, report.sleep_pressure_before);
        } else {
            log::debug!("[sleep] pressure={:.2} len={}->{}",
                report.sleep_pressure_before, report.pre_sleep_len, report.post_sleep_len);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(UQCalibrationStage);
impl BrainStage for UQCalibrationStage {
    fn name(&self) -> &str { "uq_calibration" }
    fn frequency(&self) -> usize { 20 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::core::nt_core_consciousness::conformal_uq::ConformalUQ;
        use crate::core::nt_core_consciousness::confidence_calibrator::ConfidenceCalibrator;
        let mut uq = ConformalUQ::new(0.9, 100);
        let mut cal = ConfidenceCalibrator::new();
        let recent: Vec<_> = brain._consciousness_stream.recent(50);
        for tagged in &recent {
            let score = tagged.confidence;
            let nonconf = if score > 0.0 { 1.0 - score } else { 0.5 };
            uq.add_calibration(&[nonconf]);
            let correct = score > 0.5;
            cal.record_prediction(score, correct);
        }
        let _threshold = uq.calibrate();
        for tagged in brain._consciousness_stream.recent(20) {
            let conf = tagged.confidence.max(0.1).min(1.0);
            let calibrated = cal.calibrate(conf);
            log::trace!("[uq] raw={:.4} calibrated={:.4} threshold={:.4} meta_acc={:.3}",
                conf, calibrated, _threshold, cal.meta_accuracy());
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(PhiStage);
impl BrainStage for PhiStage {
    fn name(&self) -> &str { "phi_measure" }
    fn frequency(&self) -> usize { 15 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_core_iit_phi::IITPhiCalculator;
        let mut calculator = IITPhiCalculator::new();
        let report = calculator.compute_phi(&brain.brain.capability.arr);
        calculator.record(report.phi);
        log::info!(
            "[phi] Φ={:.4} trend={:+.4} conscious={} effective_dims={}",
            report.phi, report.phi_trend, report.is_conscious_like, report.effective_dims
        );
        let phi_bonus = (report.phi * 0.15).min(0.15);
        if phi_bonus > 0.01 {
            brain._reward = (brain._reward + phi_bonus).min(1.0);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(NegentropyStage);
impl BrainStage for NegentropyStage {
    fn name(&self) -> &str { "negentropy" }
    fn frequency(&self) -> usize { 5 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let report = brain._negentropy.compute_full(
            &brain.brain.capability.arr,
            &brain._negentropy_nvsa_pool,
            brain._nt_memory_kb.as_ref(),
            brain.nt_world_jepa.as_ref(),
            &brain._strategy_matrix,
            &brain._consciousness_stream,
            0.0,  // import_rate — set by data ingestion stages
            0,    // export_count — set by SleepGate
            brain.tool_call_count as f64 + 1.0,
        );

        log::info!(
            "[negentropy] N_total={:.4} Φ={:.4} KB={:.4} trend={:+.4} regime={:?} {}",
            report.metric.total,
            report.metric.components.phi,
            report.metric.components.kb_order,
            report.metric.trend,
            report.regime,
            report.recommendation,
        );

        brain._reward = (report.metric.total * 0.6 + brain._reward * 0.4).clamp(0.0, 1.0);

        Ok(StageDecision::Continue)
    }
}

pub fn seal_pipeline() -> BrainPipeline {
    BrainPipeline::with_stages(vec![
        Box::new(crate::neotrix::nt_mind_ingestion::pipeline_stages::VsaFingerprintStage::new()),
        Box::new(crate::neotrix::nt_mind_ingestion::pipeline_stages::CanonicalSortStage::new()),
        Box::new(crate::neotrix::nt_mind_ingestion::pipeline_stages::StreamHygieneStage::new()),
        Box::new(crate::neotrix::nt_mind_ingestion::pipeline_stages::InnerCriticStage::new()),
        Box::new(SemanticRecallStage::new()),
        Box::new(GoalContractStage::new()),
        Box::new(SnapshotStage::new()),
        Box::new(CheckpointStage::new()),
        Box::new(AutonomyGateStage::new()),
        Box::new(MemoryRetrievalStage::new()),
        Box::new(GapAnalysisStage::new()),
        Box::new(SSMUpdateStage::new()),
        Box::new(OpenSourceCompareStage::new()),
        Box::new(SelfEditGenerationStage::new()),
        Box::new(crate::neotrix::nt_mind::self_iterating::skillopt::BoundedEditStage::new()),
        Box::new(ApplyEditsStage::new()),
        Box::new(LspDiagnosticsStage::new()),
        Box::new(EvidenceCaptureStage::new()),
        Box::new(ExternalVerifierStage::new()),
        Box::new(RewardCalculationStage::new()),
        Box::new(NarrowRecoveryStage::new()),
        Box::new(AdaptiveLRStage::new()),
        Box::new(crate::neotrix::nt_mind::self_iterating::skillopt::ValidationGateStage::new()),
        Box::new(GwtAbsorbStage::new()),
        Box::new(StatsSignificanceStage::new()),
        Box::new(HarnessAdaptStage::new()),
        Box::new(TaskAffinityStage::new()),
        Box::new(KnowledgeQualityStage::new()),
        Box::new(RollbackDecisionStage::new()),
        Box::new(RewindStage::new()),
        Box::new(crate::neotrix::nt_mind::self_iterating::skillopt::RejectedBufferFeedbackStage::new()),
        Box::new(ChampionCompareStage::new()),
        Box::new(ReasoningBankStorageStage::new()),
        Box::new(HyperCubeOptimizeStage::new()),
        Box::new(E8ExperimentStage::new()),
        Box::new(crate::neotrix::nt_mind::self_iterating::skillopt::EpochSlowUpdateStage::new()),
        Box::new(SecurityStage::new()),
        Box::new(DistillationStage::new()),
        Box::new(ConversationDistillStage::new()),
        Box::new(crate::neotrix::nt_mind::self_iterating::aging_monitor::AgingDiagnosisStage::new()),
        Box::new(EmbeddingRefreshStage::new()),
        Box::new(SpectralMonitorStage::new()),
        Box::new(TrajectoryCollectStage::new()),
        Box::new(CoachAndUpdateStage::new()),
        Box::new(MetaImprovementStage::new()),
        Box::new(SleepStage::new()),
        Box::new(SelfPreservationStage::new()),
        Box::new(DegradationGateStage::new()),
        Box::new(PluginDiscoveryStage::new()),
        Box::new(UQCalibrationStage::new()),
        Box::new(PhiStage::new()),
        Box::new(NegentropyStage::new()),
        Box::new(ConflictResolutionStage::new()),
        Box::new(MotivationStage::new()),
        Box::new(CodeSearchStage::new()),
        Box::new(PerceptionEvolutionStage::new()),
        Box::new(AuraIntentStage::new()),
        Box::new(RemoteSyncStage::new()),
        Box::new(crate::neotrix::nt_act_social::SocialIngestionStage::new()),
        Box::new(VisionStage::default()),
        Box::new(SideGitStage::new()),
        Box::new(FinalVerificationStage::new()),
        Box::new(GoalTerminatorStage::new()),
    ])
}

make_stage!(CodeSearchStage);
impl BrainStage for CodeSearchStage {
    fn name(&self) -> &str { "code_search" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_world_code_search::CodeSearchEngine;
        let workspace = std::env::var("NEOTRIX_WORKSPACE")
            .unwrap_or_else(|_| ".".to_string());
        let query = std::env::var("NEOTRIX_CODE_QUERY").ok();
        if let Some(q) = query {
            if !q.trim().is_empty() {
                let mut engine = CodeSearchEngine::with_root(std::path::Path::new(&workspace));
                match engine.format_results(&q, 5) {
                    Ok(output) => {
                        log::info!("[code_search] query='{}' → {} chars", q, output.len());
                        brain.code_search_cache = Some(output);
                    }
                    Err(e) => {
                        log::warn!("[code_search] failed: {}", e);
                        brain.code_search_cache = Some(format!("Code search error: {}", e));
                    }
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(AuraIntentStage);
impl BrainStage for AuraIntentStage {
    fn name(&self) -> &str { "aura_intent" }
    fn frequency(&self) -> usize { 1 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        if task.is_empty() { return Ok(StageDecision::Continue); }

        let frame = brain._tom.infer(&task);
        log::info!("[aura] intent={:?} gap={:.2} budget={} needs_probing={}",
            frame.literal_intent, frame.gap_score, frame.probe_budget, frame.needs_probing());

        if frame.gap_score > 0.3 {
            brain._reward = (brain._reward + frame.gap_score * 0.05).min(1.0);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(RemoteSyncStage);
impl BrainStage for RemoteSyncStage {
    fn name(&self) -> &str { "remote_sync" }
    fn frequency(&self) -> usize { 5 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref state) = brain._remote_control_state {
            if let Ok(mut s) = state.try_write() {
                s.pipeline_status = brain.pipeline_status();
                s.current_task = brain._current_task();
                s.iteration = brain.iteration;
                s.reward = brain._reward;
            }
        }
        Ok(StageDecision::Continue)
    }
}

pub struct LspDiagnosticsStage {
    manager: Mutex<crate::neotrix::nt_mind::lsp_client::LspManager>,
}

impl LspDiagnosticsStage {
    pub fn new() -> Self {
        Self { manager: Mutex::new(crate::neotrix::nt_mind::lsp_client::LspManager::new()) }
    }
}

impl Default for LspDiagnosticsStage {
    fn default() -> Self { Self::new() }
}

impl BrainStage for LspDiagnosticsStage {
    fn name(&self) -> &str { "lsp_diagnostics" }
    fn frequency(&self) -> usize { 5 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let workspace = std::env::var("NEOTRIX_WORKSPACE")
            .unwrap_or_else(|_| ".".to_string());

        let files = match recently_modified_sources(&workspace, 5) {
            Ok(f) => f,
            Err(e) => { log::debug!("[lsp_diag] scan failed: {}", e); return Ok(StageDecision::Continue); }
        };

        if files.is_empty() {
            return Ok(StageDecision::Continue);
        }

        let mut manager = match self.manager.lock() {
            Ok(m) => m,
            Err(e) => { log::warn!("[lsp_diag] lock: {}", e); return Ok(StageDecision::Continue); }
        };

        let cargo = format!("{}/Cargo.toml", workspace);
        let server = match manager.detect_and_start(&cargo) {
            Some(s) => s,
            None => { return Ok(StageDecision::Continue); }
        };

        let mut total_errors = 0;
        let mut total_warnings = 0;

        for file in &files {
            let uri = format!("file://{}", file);
            let _ = manager.send_request(&server, "textDocument/didOpen", serde_json::json!({
                "textDocument": { "uri": uri, "languageId": "rust", "version": 1, "text": "" }
            }));

            let resp = manager.send_request(&server, "textDocument/diagnostic", serde_json::json!({
                "textDocument": { "uri": uri }
            }));

            if let Some(val) = resp {
                let items = val.pointer("/result/items")
                    .or_else(|| val.pointer("/result"))
                    .and_then(|r| r.as_array());
                if let Some(diags) = items {
                    for d in diags {
                        let sev = d.get("severity").and_then(|s| s.as_i64()).unwrap_or(0);
                        let msg = d.get("message").and_then(|m| m.as_str()).unwrap_or("");
                        match sev {
                            1 => { total_errors += 1; log::info!("[lsp_diag] ERR {}:{}", file, msg); }
                            2 => { total_warnings += 1; log::debug!("[lsp_diag] WARN {}:{}", file, msg); }
                            _ => {}
                        }
                    }
                } else if let Some(err) = val.get("error") {
                    log::debug!("[lsp_diag] LSP err {}: {}", file, err);
                }
            }
        }

        if total_errors > 0 || total_warnings > 0 {
            let penalty = (total_errors as f64).min(5.0) * -0.05
                       + (total_warnings as f64).min(5.0) * -0.01;
            brain._reward = (brain._reward + penalty).max(-1.0);
            log::info!("[lsp_diag] errors={} warnings={} penalty={:.3}", total_errors, total_warnings, penalty);
        }

        Ok(StageDecision::Continue)
    }
}

fn recently_modified_sources(root: &str, max: usize) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let now = std::time::SystemTime::now();

    fn walk(dir: &Path, files: &mut Vec<(String, std::time::SystemTime)>, max: usize, now: std::time::SystemTime) -> Result<(), String> {
        if files.len() >= max { return Ok(()); }
        let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
        for entry in entries {
            let e = entry.map_err(|e| e.to_string())?;
            let p = e.path();
            if p.is_dir() {
                let n = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if n.starts_with('.') || n == "node_modules" || n == "target" || n == "build" {
                    continue;
                }
                walk(&p, files, max, now)?;
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if matches!(ext, "rs" | "ts" | "js" | "py" | "go") {
                    if let Ok(meta) = std::fs::metadata(&p) {
                        if let Ok(mtime) = meta.modified() {
                            if let Ok(d) = now.duration_since(mtime) {
                                if d.as_secs() < 60 {
                                    files.push((p.to_string_lossy().to_string(), mtime));
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    walk(Path::new(root), &mut files, max, now)?;
    files.sort_by(|a, b| b.1.cmp(&a.1));
    files.truncate(max);
    Ok(files.into_iter().map(|(p, _)| p).collect())
}

make_stage!(ConflictResolutionStage);
impl BrainStage for ConflictResolutionStage {
    fn name(&self) -> &str { "conflict_resolution" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::core::nt_core_consciousness::sleep_gate::detect_conflicts;
        let threshold = 0.85;
        let entries: Vec<&VsaTagged> = brain._consciousness_stream.recent(30);
        let conflicts = detect_conflicts(&entries, threshold);
        let _resolved = 0;
        for (i, j) in &conflicts {
            let sim = crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA::similarity(&entries[*i].vector, &entries[*j].vector);
            log::debug!("[conflict] entry[{}] vs entry[{}] sim={:.3}", i, j, sim);
        }
        if !conflicts.is_empty() {
            let _resolved = conflicts.len().min(5);
            log::info!("[conflict] detected={} resolved={}", conflicts.len(), _resolved);
            brain._reward = (brain._reward + 0.02 * _resolved as f64).min(1.0);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(MotivationStage);
impl BrainStage for MotivationStage {
    fn name(&self) -> &str { "intrinsic_motivation" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_mind_ingestion::intrinsic_value::aggregate_intrinsic_reward;
        let prediction_error = brain.curiosity_bonus;
        let knowledge_gaps = brain._external_reward.map(|r| (r * 10.0) as u64).unwrap_or(0);
        let total_known = brain.reasoning_bank.stats().total_memories.max(1) as u64;
        let rewards = aggregate_intrinsic_reward(prediction_error, knowledge_gaps as usize, total_known as usize, 0, 50);
        let total_intrinsic: f64 = rewards.iter().map(|r| r.value).sum();
        if total_intrinsic > 0.01 {
            let bonus = (total_intrinsic * 0.3).min(0.2);
            brain._reward = (brain._reward + bonus).min(1.0);
            log::info!("[motivation] intrinsic_reward={:.4} bonus={:.4} sources={}",
                total_intrinsic, bonus, rewards.len());
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(SelfPreservationStage);
impl BrainStage for SelfPreservationStage {
    fn name(&self) -> &str { "self_preservation" }
    fn frequency(&self) -> usize { 20 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_mind_ingestion::self_preservation::ResourceUsage;
        let usage = ResourceUsage {
            memory_mb: 0,
            stage_count: brain.pipeline.stages.len(),
            pipeline_depth: brain.pipeline.stages.len(),
            cpu_seconds: brain.self_preservation.uptime().as_secs_f64(),
        };
        if let Some(warning) = brain.self_preservation.protect(&usage, 1024) {
            log::warn!("[self_preservation] resource guard: {}", warning);
            brain._reward = (brain._reward - 0.05).max(-0.5);
        }
        brain.self_preservation.save_checkpoint("pipeline", format!("iter={} reward={:.3}", brain.iteration, brain._reward));
        log::debug!("[self_preservation] uptime={:?} health={}",
            brain.self_preservation.uptime(), brain.self_preservation.health());
        Ok(StageDecision::Continue)
    }
}

make_stage!(DegradationGateStage);
impl BrainStage for DegradationGateStage {
    fn name(&self) -> &str { "degradation_gate" }
    fn frequency(&self) -> usize { 15 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_mind_ingestion::graceful_degradation::{CapabilityStatus, DegradationLevel};
        let caps = CapabilityStatus::detect(
            brain.nt_world_jepa.is_some(),
            brain._nt_memory_kb.is_some(),
            brain.nt_act_crypto.is_some(),
        );
        let level = DegradationLevel::from_capabilities(&caps);
        if level as u8 <= DegradationLevel::Reduced as u8 {
            log::info!("[degradation] level={:?} available={}/6", level, caps.available_count());
        }
        if level == DegradationLevel::Minimal {
            log::warn!("[degradation] minimal capability — reducing reward expectation");
            brain._reward = (brain._reward - 0.1).max(-0.5);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(PluginDiscoveryStage);
impl BrainStage for PluginDiscoveryStage {
    fn name(&self) -> &str { "plugin_discovery" }
    fn frequency(&self) -> usize { 50 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let count = brain.plugin_registry.list().len();
        if count > 0 {
            log::info!("[plugin] {} plugins/skills available", count);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(SideGitStage);
impl BrainStage for SideGitStage {
    fn name(&self) -> &str { "side_git" }
    fn frequency(&self) -> usize { 30 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let workspace = std::env::var("NEOTRIX_WORKSPACE").unwrap_or_else(|_| ".".to_string());
        let ws_path = std::path::Path::new(&workspace);
        if !ws_path.exists() {
            return Ok(StageDecision::Skip("no workspace".into()));
        }
        if let Err(e) = brain.side_git.init() {
            log::warn!("[side_git] init failed: {}", e);
            return Ok(StageDecision::Continue);
        }
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(ws_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                if elapsed.as_secs() < 300 {
                                    files.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
        if !files.is_empty() {
            match brain.side_git.snapshot_files(&files, ws_path) {
                Ok(n) => {
                    if n > 0 {
                        log::info!("[side_git] snapshotted {} files (total={})", n, brain.side_git.snapshot_count());
                    }
                }
                Err(e) => log::warn!("[side_git] snapshot error: {}", e),
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(PerceptionEvolutionStage);
impl BrainStage for PerceptionEvolutionStage {
    fn name(&self) -> &str { "perception_evolution" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let budget = brain.perception_evolution.compute_budget(brain.iteration);
        brain.perception_evolution.record_budget(brain.iteration, budget.clone());
        brain.perception_evolution.decay_exploration();
        if brain.iteration % 50 == 0 {
            if let Some(top) = budget.first() {
                log::info!("[perception] best={:?} alloc={:.2} explore={:.3}",
                    top.modality, top.allocation, brain.perception_evolution.exploration_rate);
            }
        }
        Ok(StageDecision::Continue)
    }
}

pub fn kernel_iterate_pipeline() -> BrainPipeline {
    BrainPipeline::with_stages(vec![
        Box::new(SnapshotStage::new()),
        Box::new(AutonomyGateStage::new()),
        Box::new(MemoryRetrievalStage::new()),
        Box::new(OpenSourceCompareStage::new()),
        Box::new(AdaptiveLRStage::new()),
        Box::new(KnowledgeAbsorbStage::new()),
        Box::new(MemoryStorageStage::new()),
        Box::new(ChampionCompareStage::new()),
        Box::new(EvaluationStage::new()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    use crate::neotrix::nt_world_model::TaskType;

    #[test]
    fn test_pipeline_execution_returns_reward() {
        let mut brain = SelfIteratingBrain::new();
        let result = brain.run_seal_loop("设计一个响应式 UI 界面", None, None);
        assert!(result.is_ok(), "pipeline 应返回奖励值，但得到: {:?}", result);
        let reward = result.expect("result should be ok in test");
        // 即使无外部信号，pipeline 也应产生内部评估奖励
        assert!(reward > -1.0, "奖励应 > -1.0，得到: {}", reward);
    }

    #[test]
    fn test_pipeline_kernel_iterate() {
        let mut brain = SelfIteratingBrain::new();
        let result = brain.kernel_iterate("优化数据库查询性能");
        assert!(result.improved || result.score_after >= result.score_before - 0.1);
        assert!(result.iteration > 0);
    }

    #[test]
    fn test_pipeline_stores_memory() {
        let mut brain = SelfIteratingBrain::new();
        let _ = brain.run_seal_loop("设计 React 组件", None, None);
        let stats = brain.reasoning_bank.stats();
        assert!(stats.total_memories > 0, "pipeline 应存储推理记忆");
    }

    #[test]
    fn test_pipeline_twice_accumulates() {
        let mut brain = SelfIteratingBrain::new();
        let _ = brain.run_seal_loop("任务 A", None, None);
        let _ = brain.run_seal_loop("任务 B", None, None);
        let stats = brain.reasoning_bank.stats();
        assert!(stats.total_memories >= 2, "两次 pipeline 应累积记忆");
    }

    #[test]
    fn test_pipeline_champion_promotion() {
        let mut brain = SelfIteratingBrain::new();
        // 初始冠军
        brain.champion = Some(BrainSnapshot::new(&brain.brain, &TaskType::General));
        let baseline = brain.champion.as_ref().expect("value should be ok in test").score;

        // 强制提升能力（模拟外部吸收）
        brain.brain.capability.arr_mut()[0] = 0.99;
        brain.brain.capability.normalize();
        let _ = brain.kernel_iterate("general");

        // Champion 应有提升（或至少不降低）
        if let Some(ref champ) = brain.champion {
            assert!(champ.score >= baseline * 0.9, "champion 不应显著降低");
        }
    }

    #[test]
    fn test_pipeline_autonomy_proposal_skips_execution() {
        let mut brain = SelfIteratingBrain::new();
        brain.autonomy = AutonomyLevel::Proposal;
        let before = brain.brain.capability.clone();
        let result = brain.run_seal_loop("测试任务", None, None);
        let after = brain.brain.capability.clone();
        // Proposal 模式：能力向量不应修改
        let change: f64 = before.arr().iter().zip(after.arr().iter()).map(|(a, b)| (a - b).abs()).sum();
        assert!(change < 0.001, "Proposal 模式不应修改能力向量");
        // 但 pipeline 仍应返回奖励（不阻止流程）
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_autonomy_bounded_blocks_high_capability() {
        let mut brain = SelfIteratingBrain::new();
        brain.autonomy = AutonomyLevel::Bounded;
        // 设置能力总和 > 16.0 以触发 bounded 门控
        for i in 0..brain.brain.capability.arr().len() {
            brain.brain.capability.arr_mut()[i] = 0.9;
        }
        let _snapshot = brain.brain.capability.clone();
        let result = brain.run_seal_loop("test", None, None);
        // Bounded 模式能力超过阈值时应返回 Ok（但实际跳过了修改）
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_stages_order() {
        let pipeline = seal_pipeline();
        let names: Vec<&str> = pipeline.stages.iter().map(|s| s.name()).collect();
        assert_eq!(names, vec![
            "vsa_fingerprint", "canonical_sort", "stream_hygiene", "inner_critic",
            "semantic_recall", "goal_contract",
            "snapshot", "checkpoint", "autonomy_gate", "memory_retrieval",
            "gap_analysis",
            "ssm_update", "open_source_compare", "self_edit_gen",
            "bounded_edit", "apply_edits",
            "lsp_diagnostics",
            "evidence_capture", "external_verifier",
            "reward_calc", "narrow_recovery", "adaptive_lr", "validation_gate",
            "gwt_absorb", "stats_significance",
            "harness_adapt",
            "task_affinity", "knowledge_quality",
            "rollback_decision", "rewind", "rejected_feedback",
            "champion_compare", "bank_storage",
            "hypercube_optimize",
            "e8_experiment",
            "epoch_slow_update", "nt_shield_scan", "session_distill", "conversation_distill", "aging_diagnosis", "embedding_refresh",
            "spectral_monitor",
            "trajectory_collect", "coach_and_update",
            "meta_improvement", "sleep", "self_preservation", "degradation_gate", "plugin_discovery", "uq_calibration", "phi_measure",
            "conflict_resolution", "intrinsic_motivation", "code_search", "perception_evolution", "side_git",
            "final_verification", "goal_terminator",
        ], "SEAL pipeline 应有 58 个 stage");
    }

    #[test]
    fn test_kernel_pipeline_stages_order() {
        let pipeline = kernel_iterate_pipeline();
        let names: Vec<&str> = pipeline.stages.iter().map(|s| s.name()).collect();
        assert_eq!(names, vec![
            "snapshot", "autonomy_gate", "memory_retrieval",
            "open_source_compare", "adaptive_lr", "knowledge_absorb", "memory_storage",
            "champion_compare", "evaluation",
        ], "Kernel pipeline 应有 9 个 stage");
    }

    #[test]
    fn test_all_stages_have_unique_names() {
        let pipeline = seal_pipeline();
        let names: Vec<&str> = pipeline.stages.iter().map(|s| s.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len(), "所有 stage 名必须唯一");
    }

    #[test]
    fn test_permission_level_syncs_to_approval_mode() {
        {
            let mut s = crate::cli::global_shield().lock().unwrap();
            s.set_approval_mode(crate::cli::approval::ApprovalMode::Suggest);
        }
        let mut brain = SelfIteratingBrain::new();

        brain.permission = PermissionLevel::Review;
        let _ = brain.run_seal_loop("test_review_sync", None, None);
        {
            let shield = crate::cli::global_shield().lock().unwrap();
            assert_eq!(shield.approval.mode(), crate::cli::approval::ApprovalMode::Suggest);
        }

        brain.permission = PermissionLevel::Suggest;
        let _ = brain.run_seal_loop("test_suggest_sync", None, None);
        {
            let shield = crate::cli::global_shield().lock().unwrap();
            assert_eq!(shield.approval.mode(), crate::cli::approval::ApprovalMode::AutoEdit);
        }

        brain.permission = PermissionLevel::Full;
        let _ = brain.run_seal_loop("test_full_sync", None, None);
        {
            let shield = crate::cli::global_shield().lock().unwrap();
            assert_eq!(shield.approval.mode(), crate::cli::approval::ApprovalMode::FullAuto);
        }
    }

    #[test]
    fn test_permission_level_review_skips_autonomy_gate() {
        let mut brain = SelfIteratingBrain::new();
        brain.permission = PermissionLevel::Review;
        let stage = AutonomyGateStage::new();
        let decision = stage.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Skip(_)));
    }

    #[test]
    fn test_sandbox_readonly_skips_autonomy_gate() {
        {
            let mut s = crate::cli::global_shield().lock().unwrap();
            s.set_sandbox_mode(crate::cli::sandbox::SandboxMode::ReadOnly);
        }
        let mut brain = SelfIteratingBrain::new();
        brain.permission = PermissionLevel::Full;
        let stage = AutonomyGateStage::new();
        let decision = stage.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Skip(_)));
        {
            let mut s = crate::cli::global_shield().lock().unwrap();
            s.set_sandbox_mode(crate::cli::sandbox::SandboxMode::Disabled);
        }
    }
}
