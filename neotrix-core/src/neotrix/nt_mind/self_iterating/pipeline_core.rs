use super::checkpoint::{CheckpointStage, RewindStage};
use super::goal_contract::{
    EvidenceCaptureStage, ExternalVerifierStage, FinalVerificationStage, GoalContractStage,
    GoalTerminatorStage, NarrowRecoveryStage, SemanticRecallStage,
};
use super::SelfIteratingBrain;
pub(crate) use crate::neotrix::nt_core_error::NeoTrixError;
use crate::neotrix::nt_expert_routing::TaskType;
use crate::neotrix::nt_world_vision::VisionStage;
use std::path::Path;
use std::sync::Mutex;

// Stage types from sibling sub-modules (used in seal_pipeline / kernel_iterate_pipeline)
use super::pipeline_awareness::{
    AuraIntentStage, AutonomyGateStage, ConflictResolutionStage, DegradationGateStage,
    KnowledgeQualityStage, MotivationStage, NegentropyStage, PerceptionEvolutionStage, PhiStage,
    PluginDiscoveryStage, SelfPreservationStage, SleepStage, SnapshotStage, SpectralMonitorStage,
    StatsSignificanceStage, UQCalibrationStage,
};
use super::pipeline_code::{RemoteSyncStage, SecurityStage, SideGitStage};
use super::pipeline_evolution::{
    AdaptiveLRStage, ApplyEditsStage, ChampionCompareStage, ConversationDistillStage,
    DistillationStage, E8ExperimentStage, EvaluationStage, GwtAbsorbStage, HarnessAdaptStage,
    HyperCubeOptimizeStage, RewardCalculationStage, RollbackDecisionStage, SSMUpdateStage,
    SelfEditGenerationStage, SelfReviewStage, TaskAffinityStage,
};
use super::pipeline_memory::{
    CoachAndUpdateStage, MemoryStorageStage, MetaImprovementStage, ReasoningBankStorageStage,
    TrajectoryCollectStage,
};
use super::pipeline_search::{
    CodeSearchStage, EmbeddingRefreshStage, GapAnalysisStage, KnowledgeAbsorbStage,
    MemoryRetrievalStage, OpenSourceCompareStage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutonomyLevel {
    Proposal,
    Bounded,
    #[default]
    Full,
}

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
    if raw_cost > 0.0 {
        efc / raw_cost
    } else {
        0.0
    }
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
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError>;
    fn verify_step(
        &self,
        _brain: &SelfIteratingBrain,
    ) -> Option<super::vsi_verifier::VsiStepVerdict> {
        None
    }
}

pub struct BrainPipeline {
    pub stages: Vec<Box<dyn BrainStage>>,
}

impl BrainPipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn with_stages(stages: Vec<Box<dyn BrainStage>>) -> Self {
        Self { stages }
    }

    pub fn register(&mut self, stage: Box<dyn BrainStage>) {
        self.stages.push(stage);
    }

    pub fn execute(&self, brain: &mut SelfIteratingBrain) -> Result<(), NeoTrixError> {
        brain.seal_rl.stage_results.clear();
        for stage in &self.stages {
            let freq = stage.frequency();
            if freq > 1 && !brain.iteration.is_multiple_of(freq as u64) {
                continue;
            }

            {
                let task_type = brain.task_scratch.current_task_type;
                let snap = BrainSnapshot::new(&brain.brain, &task_type);
                let iteration = brain.iteration;
                let permission = brain.permission;
                let autonomy = brain.autonomy;
                let reward = brain.task_scratch.reward;
                brain.checkpoint_manager.push(
                    iteration,
                    &snap,
                    permission,
                    autonomy,
                    reward,
                    stage.name(),
                );
            }

            let _span = tracing::info_span!(
                "pipeline_stage",
                stage.name = %stage.name(),
                iteration = brain.iteration,
            )
            .entered();

            let stage_name = stage.name().to_string();
            let before_cap = brain.brain.capability.clone();
            brain.vsi_verifier.snapshot_before(&stage_name, &before_cap);

            let decision = stage.process(brain)?;

            if let Some(verdict) = stage.verify_step(brain) {
                match &verdict {
                    super::vsi_verifier::VsiStepVerdict::Pass(s) => {
                        log::trace!("[vsi] {} passed (score={:.4})", stage.name(), s);
                    }
                    super::vsi_verifier::VsiStepVerdict::Fail(reason, s) => {
                        log::warn!("[vsi] {} FAILED: {} (score={:.4})", stage.name(), reason, s);
                    }
                    super::vsi_verifier::VsiStepVerdict::Skip => {}
                }
            }

            let after_cap = brain.brain.capability.clone();
            let vsi_verdict = brain.vsi_verifier.verify(&stage_name, &after_cap);
            if let super::vsi_verifier::VsiStepVerdict::Fail(reason, s) = &vsi_verdict {
                log::warn!(
                    "[vsi] capability delta check FAILED for {}: {} (magnitude={:.4})",
                    stage.name(),
                    reason,
                    s
                );
            }

            let mut stage_result = StageResult::new(stage.name());
            if matches!(decision, StageDecision::Skip(_)) {
                stage_result.efc *= 0.1;
                stage_result.raw_cost *= 0.1;
            } else if matches!(decision, StageDecision::Rollback(_)) {
                stage_result.efc *= 0.3;
            }
            stage_result.efficiency =
                compute_stage_efficiency(stage_result.efc, stage_result.raw_cost);
            brain.seal_rl.stage_results.push(stage_result);

            match decision {
                StageDecision::Continue => continue,
                StageDecision::Skip(reason) => {
                    log::trace!("Stage '{}' aborted remaining: {}", stage.name(), reason);
                    return Ok(());
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
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! make_stage {
    ($name:ident) => {
        pub struct $name;
        impl Default for $name {
            fn default() -> Self {
                Self
            }
        }
        impl $name {
            pub fn new() -> Self {
                Self
            }
        }
    };
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

pub fn run_conversation_distill(
    kb: &crate::neotrix::nt_memory_kb::KnowledgeBase,
) -> Result<DistillationResult, NeoTrixError> {
    let records = match kb.get_evolution_history(10) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("[conv-distill] query failed: {}", e);
            return Ok(DistillationResult {
                total: 0,
                successes: 0,
                failures: 0,
                avg_eff: 0.0,
                error_rate: 0.0,
                patterns_created: false,
                total_gain: 0.0,
            });
        }
    };
    if records.is_empty() {
        return Ok(DistillationResult {
            total: 0,
            successes: 0,
            failures: 0,
            avg_eff: 0.0,
            error_rate: 0.0,
            patterns_created: false,
            total_gain: 0.0,
        });
    }

    let total = records.len();
    let successes = records.iter().filter(|r| r.outcome == "success").count();
    let failures = total - successes;
    let avg_eff: f64 = records.iter().map(|r| r.effectiveness).sum::<f64>() / total as f64;

    let mut by_strategy: std::collections::HashMap<&str, Vec<f64>> =
        std::collections::HashMap::new();
    for r in &records {
        by_strategy
            .entry(r.strategy_used.as_str())
            .or_default()
            .push(r.effectiveness);
    }
    let mut strategy_ratings: Vec<(&str, f64)> = by_strategy
        .iter()
        .map(|(k, v)| (*k, v.iter().sum::<f64>() / v.len() as f64))
        .collect();
    strategy_ratings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let error_patterns: Vec<&str> = records
        .iter()
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
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut created_any = false;

    if error_rate > 0.0 {
        let most_common_error = error_patterns.first().unwrap_or(&"unknown");
        let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
            id: format!("conv_pat_err_{}", timestamp),
            source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
            pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::RecurringError,
            description: format!(
                "Error rate {:.2}, {} failures: {}",
                error_rate, failures, most_common_error
            ),
            before_behavior: "".into(),
            after_behavior: "".into(),
            effectiveness_gain: avg_eff * error_rate,
            applied_to: vec![],
            verified: false,
            timestamp,
        };
        if kb.store_evolution_record(&evolution).is_ok() {
            created_any = true;
            log::info!(
                "[conv-distill] recorded RecurringError pattern: {}",
                evolution.description
            );
        }
    }

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
                    log::info!(
                        "[conv-distill] recorded CommunicationOptimization pattern: {}",
                        evolution.description
                    );
                }
            }
        }
    }

    let high_eff_low_err = records
        .iter()
        .filter(|r| r.effectiveness > 0.7 && r.error_count == 0)
        .count();
    if high_eff_low_err as f64 > total as f64 * 0.3 {
        let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
            id: format!("conv_pat_dec_{}", timestamp),
            source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
            pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::ProblemDecomposition,
            description: format!(
                "{}/{} records high-effectiveness zero-error",
                high_eff_low_err, total
            ),
            before_behavior: "".into(),
            after_behavior: "".into(),
            effectiveness_gain: avg_eff,
            applied_to: vec![],
            verified: false,
            timestamp,
        };
        if kb.store_evolution_record(&evolution).is_ok() {
            created_any = true;
            log::info!(
                "[conv-distill] recorded ProblemDecomposition pattern: {}",
                evolution.description
            );
        }
    }

    if fix_patterns_present {
        let fix_count: usize = records.iter().map(|r| r.fix_patterns.len()).sum();
        let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
            id: format!("conv_pat_ver_{}", timestamp),
            source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
            pattern_type:
                crate::neotrix::nt_memory_kb::EvolutionPatternType::VerificationImprovement,
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
            log::info!(
                "[conv-distill] recorded VerificationImprovement pattern: {}",
                evolution.description
            );
        }
    }

    let successful_strategies: Vec<&str> = records
        .iter()
        .filter(|r| r.outcome == "success" && !r.actions_taken.is_empty())
        .map(|r| r.strategy_used.as_str())
        .collect();
    if !successful_strategies.is_empty() && successful_strategies.len() > total / 2 {
        let mut tool_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for t in &successful_strategies {
            *tool_counts.entry(t).or_insert(0) += 1;
        }
        if let Some((best_tool, _)) = tool_counts.iter().max_by_key(|(_, c)| **c) {
            let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
                id: format!("conv_pat_tool_{}", timestamp),
                source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
                pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::ToolUsagePattern,
                description: format!(
                    "Strategy '{}' used in {}/{} successes",
                    best_tool, tool_counts[best_tool], successes
                ),
                before_behavior: "".into(),
                after_behavior: best_tool.to_string(),
                effectiveness_gain: avg_eff,
                applied_to: vec![],
                verified: false,
                timestamp,
            };
            if kb.store_evolution_record(&evolution).is_ok() {
                created_any = true;
                log::info!(
                    "[conv-distill] recorded ToolUsagePattern pattern: {}",
                    evolution.description
                );
            }
        }
    }

    if let Some((best_strat, best_eff)) = strategy_ratings.first() {
        if *best_strat != "auto" && *best_eff > avg_eff * 1.1 {
            let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
                id: format!("conv_pat_strat_{}", timestamp),
                source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
                pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::StrategyDiscovery,
                description: format!(
                    "New strategy '{}' outperforms average ({:.2} vs {:.2})",
                    best_strat, best_eff, avg_eff
                ),
                before_behavior: "auto".into(),
                after_behavior: best_strat.to_string(),
                effectiveness_gain: best_eff - avg_eff,
                applied_to: vec![],
                verified: false,
                timestamp,
            };
            if kb.store_evolution_record(&evolution).is_ok() {
                created_any = true;
                log::info!(
                    "[conv-distill] recorded StrategyDiscovery pattern: {}",
                    evolution.description
                );
            }
        }
    }

    if avg_eff > 0.8 {
        let all_above_threshold = by_strategy
            .iter()
            .all(|(_, v)| v.iter().sum::<f64>() / v.len() as f64 > 0.7);
        if all_above_threshold && strategy_ratings.len() > 1 {
            let evolution = crate::neotrix::nt_memory_kb::EvolutionRecord {
                id: format!("conv_pat_princ_{}", timestamp),
                source_conversation_id: records.last().map(|r| r.id.clone()).unwrap_or_default(),
                pattern_type: crate::neotrix::nt_memory_kb::EvolutionPatternType::PrincipleUpdate,
                description: format!(
                    "Consistent high effectiveness ({:.2}) across {} strategies",
                    avg_eff,
                    strategy_ratings.len()
                ),
                before_behavior: "".into(),
                after_behavior: "".into(),
                effectiveness_gain: avg_eff,
                applied_to: vec![],
                verified: false,
                timestamp,
            };
            if kb.store_evolution_record(&evolution).is_ok() {
                created_any = true;
                log::info!(
                    "[conv-distill] recorded PrincipleUpdate pattern: {}",
                    evolution.description
                );
            }
        }
    }

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
        Box::new(
            crate::neotrix::nt_mind::self_iterating::skillopt::RejectedBufferFeedbackStage::new(),
        ),
        Box::new(ChampionCompareStage::new()),
        Box::new(ReasoningBankStorageStage::new()),
        Box::new(HyperCubeOptimizeStage::new()),
        Box::new(E8ExperimentStage::new()),
        Box::new(SelfReviewStage::new()),
        Box::new(crate::neotrix::nt_mind::self_iterating::skillopt::EpochSlowUpdateStage::new()),
        Box::new(SecurityStage::new()),
        Box::new(DistillationStage::new()),
        Box::new(ConversationDistillStage::new()),
        Box::new(
            crate::neotrix::nt_mind::self_iterating::aging_monitor::AgingDiagnosisStage::new(),
        ),
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

pub struct LspDiagnosticsStage {
    manager: Mutex<crate::neotrix::nt_mind::lsp_client::LspManager>,
}

impl LspDiagnosticsStage {
    pub fn new() -> Self {
        Self {
            manager: Mutex::new(crate::neotrix::nt_mind::lsp_client::LspManager::new()),
        }
    }
}

impl Default for LspDiagnosticsStage {
    fn default() -> Self {
        Self::new()
    }
}

impl BrainStage for LspDiagnosticsStage {
    fn name(&self) -> &str {
        "lsp_diagnostics"
    }
    fn frequency(&self) -> usize {
        5
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let workspace = std::env::var("NEOTRIX_WORKSPACE").unwrap_or_else(|_| ".".to_string());

        let files = match recently_modified_sources(&workspace, 5) {
            Ok(f) => f,
            Err(e) => {
                log::debug!("[lsp_diag] scan failed: {}", e);
                return Ok(StageDecision::Continue);
            }
        };

        if files.is_empty() {
            return Ok(StageDecision::Continue);
        }

        let mut manager = match self.manager.lock() {
            Ok(m) => m,
            Err(e) => {
                log::warn!("[lsp_diag] lock: {}", e);
                return Ok(StageDecision::Continue);
            }
        };

        let cargo = format!("{}/Cargo.toml", workspace);
        let server = match manager.detect_and_start(&cargo) {
            Some(s) => s,
            None => {
                return Ok(StageDecision::Continue);
            }
        };

        let mut total_errors = 0;
        let mut total_warnings = 0;

        for file in &files {
            let uri = format!("file://{}", file);
            if manager.send_request(
                &server,
                "textDocument/didOpen",
                serde_json::json!({
                    "textDocument": { "uri": uri, "languageId": "rust", "version": 1, "text": "" }
                }),
            ).is_none() {
                log::warn!("[lsp_diag] didOpen failed for {file}");
            }

            let resp = manager.send_request(
                &server,
                "textDocument/diagnostic",
                serde_json::json!({
                    "textDocument": { "uri": uri }
                }),
            );

            if let Some(val) = resp {
                let items = val
                    .pointer("/result/items")
                    .or_else(|| val.pointer("/result"))
                    .and_then(|r| r.as_array());
                if let Some(diags) = items {
                    for d in diags {
                        let sev = d.get("severity").and_then(|s| s.as_i64()).unwrap_or(0);
                        let msg = d.get("message").and_then(|m| m.as_str()).unwrap_or("");
                        match sev {
                            1 => {
                                total_errors += 1;
                                log::info!("[lsp_diag] ERR {}:{}", file, msg);
                            }
                            2 => {
                                total_warnings += 1;
                                log::debug!("[lsp_diag] WARN {}:{}", file, msg);
                            }
                            _ => {}
                        }
                    }
                } else if let Some(err) = val.get("error") {
                    log::debug!("[lsp_diag] LSP err {}: {}", file, err);
                }
            }
        }

        if total_errors > 0 || total_warnings > 0 {
            let penalty =
                (total_errors as f64).min(5.0) * -0.05 + (total_warnings as f64).min(5.0) * -0.01;
            brain.task_scratch.reward = (brain.task_scratch.reward + penalty).max(-1.0);
            log::info!(
                "[lsp_diag] errors={} warnings={} penalty={:.3}",
                total_errors,
                total_warnings,
                penalty
            );
        }

        Ok(StageDecision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_stage_produces_valid_struct() {
        use super::super::pipeline_awareness::SnapshotStage;
        let s = SnapshotStage::new();
        assert_eq!(s.name(), "snapshot");
        assert_eq!(s.frequency(), 1);
    }

    #[test]
    fn test_stage_result_new_known_name() {
        let r = StageResult::new("gap_analysis");
        assert_eq!(r.stage_name, "gap_analysis");
        assert!((r.efc - 0.85).abs() < 1e-6);
        assert!((r.raw_cost - 1000.0).abs() < 1e-6);
        assert!((r.efficiency - 0.85 / 1000.0).abs() < 1e-6);
    }

    #[test]
    fn test_stage_result_new_unknown_name() {
        let r = StageResult::new("some_unknown_stage");
        assert_eq!(r.efc, 0.30);
        assert_eq!(r.raw_cost, 200.0);
    }

    #[test]
    fn test_autonomy_level_default() {
        assert_eq!(AutonomyLevel::default(), AutonomyLevel::Full);
    }

    #[test]
    fn test_permission_level_default() {
        assert_eq!(PermissionLevel::default(), PermissionLevel::Full);
    }

    #[test]
    fn test_permission_level_to_approval_mode() {
        assert_eq!(
            PermissionLevel::Review.to_approval_mode(),
            crate::cli::approval::ApprovalMode::Suggest,
        );
        assert_eq!(
            PermissionLevel::Suggest.to_approval_mode(),
            crate::cli::approval::ApprovalMode::AutoEdit,
        );
        assert_eq!(
            PermissionLevel::Full.to_approval_mode(),
            crate::cli::approval::ApprovalMode::FullAuto,
        );
    }

    #[test]
    fn test_brain_pipeline_new_is_empty() {
        let p = BrainPipeline::new();
        assert!(p.stages.is_empty());
    }

    #[test]
    fn test_brain_pipeline_default_is_empty() {
        let p = BrainPipeline::default();
        assert!(p.stages.is_empty());
    }

    #[test]
    fn test_brain_pipeline_with_stages() {
        use super::super::pipeline_awareness::SnapshotStage;
        let stages: Vec<Box<dyn BrainStage>> = vec![Box::new(SnapshotStage)];
        let p = BrainPipeline::with_stages(stages);
        assert_eq!(p.stages.len(), 1);
        assert_eq!(p.stages[0].name(), "snapshot");
    }

    #[test]
    fn test_brain_pipeline_register() {
        use super::super::pipeline_awareness::SnapshotStage;
        let mut p = BrainPipeline::new();
        p.register(Box::new(SnapshotStage));
        assert_eq!(p.stages.len(), 1);
    }

    #[test]
    fn test_brain_pipeline_execute_empty_does_not_panic() {
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let p = BrainPipeline::new();
        assert!(p.execute(&mut brain).is_ok());
    }

    #[test]
    fn test_estimate_stage_efc_known() {
        assert_eq!(estimate_stage_efc("gap_analysis"), (0.85, 1000.0));
        assert_eq!(estimate_stage_efc("snapshot"), (0.15, 50.0));
        assert_eq!(estimate_stage_efc("autonomy_gate"), (0.50, 200.0));
    }

    #[test]
    fn test_estimate_stage_efc_unknown() {
        assert_eq!(estimate_stage_efc("nonexistent"), (0.30, 200.0));
    }

    #[test]
    fn test_compute_stage_efficiency_zero() {
        assert_eq!(compute_stage_efficiency(1.0, 0.0), 0.0);
    }

    #[test]
    fn test_compute_stage_efficiency_normal() {
        let eff = compute_stage_efficiency(0.5, 200.0);
        assert!((eff - 0.0025).abs() < 1e-10);
    }

    #[test]
    fn test_stage_decision_variants() {
        assert!(matches!(StageDecision::Continue, StageDecision::Continue));
        assert!(matches!(
            StageDecision::Skip("x".into()),
            StageDecision::Skip(_)
        ));
        assert!(matches!(
            StageDecision::Rollback("x".into()),
            StageDecision::Rollback(_)
        ));
    }

    #[test]
    fn test_stage_trait_frequency_default() {
        use super::super::pipeline_awareness::SnapshotStage;
        let s = SnapshotStage;
        assert_eq!(s.frequency(), 1);
    }

    #[test]
    fn test_stage_trait_verify_step_default() {
        use super::super::pipeline_awareness::AuraIntentStage;
        let s = AuraIntentStage;
        assert!(s
            .verify_step(&crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new())
            .is_none());
    }
}

pub(crate) fn recently_modified_sources(root: &str, max: usize) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let now = std::time::SystemTime::now();

    fn walk(
        dir: &Path,
        files: &mut Vec<(String, std::time::SystemTime)>,
        max: usize,
        now: std::time::SystemTime,
    ) -> Result<(), String> {
        if files.len() >= max {
            return Ok(());
        }
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
