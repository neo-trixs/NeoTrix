use crate::core::nt_core_ssm::SelectiveState;
use crate::core::{strategy_matrix, E8TransitionLearner, ReasoningHexagram};
use crate::neotrix::nt_act_autonomy::knowledge_distiller::KnowledgeDistiller;
use crate::neotrix::nt_act_crypto::CryptoAgent;
use crate::neotrix::nt_core_signal::select::SelectableOperator;
use crate::neotrix::nt_expert_routing::TaskType;
use crate::neotrix::nt_io_plugin::PluginRegistry;
use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_mind::attention_router::AttentionRouter;
use crate::neotrix::nt_mind::change_archive::ChangeArchive;
use crate::neotrix::nt_mind::core::{
    CapabilityVector, KnowledgeSource, PerformanceEvaluator, RewardSource,
};
use crate::neotrix::nt_mind::cortex_memory::CortexMemory;
use crate::neotrix::nt_mind::goal_loop::GoalLoop;
use crate::neotrix::nt_mind::goal_register::GoalRegister;
use crate::neotrix::nt_mind::memory::ReasoningBank;
use crate::neotrix::nt_mind::multi_brain::MultiBrainManager;
use crate::neotrix::nt_mind::perception_evolution::PerceptionEvolution;
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;
use crate::neotrix::nt_mind::self_edit::MicroEdit;
use crate::neotrix::nt_mind::self_iterating::aging_monitor::AgingMonitor;
use crate::neotrix::nt_mind::self_iterating::brain_impl::{
    AbsorbValidator, EvaluationRecord, ReasoningBrain,
};
use crate::neotrix::nt_mind::self_iterating::curvature_rl::CurvaturePolicy;
use crate::neotrix::nt_mind::self_iterating::harness_adapter::HarnessAdapter;
use crate::neotrix::nt_mind::self_iterating::pipeline::{
    seal_pipeline, AutonomyLevel, BrainPipeline, BrainSnapshot,
};
use crate::neotrix::nt_mind::self_iterating::recipe::RecipeRegistry;
use crate::neotrix::nt_mind::self_iterating::sib_state::{
    ConsciousnessState, EvoPipelineState, GoalContractState, SealRlState, TaskScratch,
};
use crate::neotrix::nt_mind::self_iterating::skillopt::{LrScheduler, ValidationGate};
use crate::neotrix::nt_mind::self_iterating::vsi_verifier::VsiVerifier;
use crate::neotrix::nt_mind::side_git::SideGit;
use crate::neotrix::nt_mind::skill_evolution::SkillEvolver;
use crate::neotrix::nt_mind::sleep::{SleepEngine, SleepStats};
use crate::neotrix::nt_mind::stagnation::StagnationDetector;
use crate::neotrix::nt_mind_ingestion::scratchpad::IngestionScratchpad;
use crate::neotrix::nt_mind_ingestion::self_preservation::SelfPreservation;
use crate::neotrix::nt_world_jepa::JepaWorldModel;
use std::sync::{Arc, Mutex};

pub struct SelfIteratingBrain {
    pub brain: ReasoningBrain,
    pub iteration: u64,
    pub quality_threshold: f64,
    pub auto_absorb: bool,
    pub evaluation_history: Vec<EvaluationRecord>,
    pub reasoning_bank: ReasoningBank,
    pub policy_learning_rate: f64,
    pub regularization_weight: f64,
    pub auto_memory_iteration: bool,
    pub memory_iteration_interval: u64,
    pub select_operator: Option<SelectableOperator>,
    pub selective_state: Option<SelectiveState>,
    pub group_manager: Option<MultiBrainManager>,
    pub reasoning_engine: Option<ReasoningEngine>,
    pub attention_router: Option<AttentionRouter>,
    pub cortex: CortexMemory,
    pub pipeline: BrainPipeline,
    pub archive: ChangeArchive,
    pub champion: Option<BrainSnapshot>,
    pub autonomy: AutonomyLevel,
    pub permission: crate::neotrix::nt_mind::self_iterating::pipeline::PermissionLevel,
    pub goal_loop: GoalLoop,
    pub goal_register: GoalRegister,
    pub entropy_crisis_level: f64,
    pub curiosity_bonus: f64,
    pub(crate) task_scratch: TaskScratch,
    pub tool_call_count: usize,
    pub tool_traces: Vec<(String, u64, bool)>,
    pub skill_evolver: SkillEvolver,
    pub stagnation: StagnationDetector,
    pub sleep_engine: Option<SleepEngine>,
    pub last_sleep_stats: Option<SleepStats>,
    pub dgm_strategy:
        Option<crate::neotrix::nt_mind::self_iterating::brain_dgm::DgmSelfEditStrategy>,
    pub(crate) seal_rl: SealRlState,
    pub(crate) _transition_learner: E8TransitionLearner,
    pub(crate) _e8_policy: crate::core::E8Policy,
    pub(crate) _knowledge_distiller: KnowledgeDistiller,
    pub(crate) _nt_memory_kb: Option<KnowledgeBase>,
    pub(crate) _strategy_matrix: [[ReasoningHexagram; 8]; 8],
    pub(crate) _trajectory_collector: crate::core::nt_core_prm::TrajectoryCollector,
    pub(crate) _coach: Option<Box<dyn crate::core::nt_core_prm::Coach>>,
    pub(crate) _ingestion_scratchpad: Option<IngestionScratchpad>,
    pub nt_act_crypto: Option<Arc<Mutex<CryptoAgent>>>,
    pub nt_world_jepa: Option<JepaWorldModel>,
    pub(crate) evo_pipeline: EvoPipelineState,
    // Consciousness system
    pub(crate) consciousness_state: ConsciousnessState,
    pub _meta_agent: Option<crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAgent>,
    pub meta_additions: Vec<Box<dyn crate::neotrix::nt_mind::self_iterating::pipeline::BrainStage>>,
    pub checkpoint_manager: crate::neotrix::nt_mind::self_iterating::checkpoint::CheckpointManager,
    pub code_search_cache: Option<String>,
    pub recipe_registry: RecipeRegistry,
    pub curvature_policy: CurvaturePolicy,
    pub self_preservation: SelfPreservation,
    pub plugin_registry: PluginRegistry,
    pub perception_evolution: PerceptionEvolution,
    pub side_git: SideGit,
    pub(crate) goal_state: GoalContractState,
    // remote-control field removed: nt_act_remote_control module was deleted (dead)
    pub(crate) _tom: crate::neotrix::nt_mind::theory_of_mind::TheoryOfMind,
    pub(crate) _negentropy: crate::neotrix::nt_core_negentropy::NegentropyCalculator,
    pub(crate) _negentropy_nvsa_pool: Vec<Vec<u8>>,
    pub(crate) _open_source_insights: Option<String>,
    pub vsi_verifier: VsiVerifier,
}

impl Clone for SelfIteratingBrain {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl SelfIteratingBrain {
    pub fn new() -> Self {
        let (init_kb, init_adapter): (Option<KnowledgeBase>, HarnessAdapter) = {
            let kb = KnowledgeBase::open(None).ok();
            let adapter = kb
                .as_ref()
                .and_then(|k| HarnessAdapter::load_from_kb(k).ok())
                .unwrap_or_default();
            (kb, adapter)
        };
        Self {
            brain: ReasoningBrain::new(),
            iteration: 0,
            quality_threshold: match std::env::var("NEOTRIX_QUALITY_THRESHOLD") {
                Ok(v) => v.parse().unwrap_or(0.85),
                Err(_) => 0.85,
            },
            auto_absorb: true,
            evaluation_history: Vec::new(),
            reasoning_bank: ReasoningBank::new(100),
            policy_learning_rate: match std::env::var("NEOTRIX_POLICY_LR") {
                Ok(v) => v.parse().unwrap_or(0.01),
                Err(_) => 0.01,
            },
            regularization_weight: 0.001,
            auto_memory_iteration: true,
            memory_iteration_interval: match std::env::var("NEOTRIX_MEMORY_INTERVAL") {
                Ok(v) => v.parse().unwrap_or(10),
                Err(_) => 10,
            },
            select_operator: None,
            selective_state: None,
            group_manager: None,
            reasoning_engine: None,
            attention_router: None,
            cortex: CortexMemory::new(50, 500),
            pipeline: seal_pipeline(),
            archive: ChangeArchive::new(),
            champion: None,
            autonomy: AutonomyLevel::Full,
            permission: crate::neotrix::nt_mind::self_iterating::pipeline::PermissionLevel::Full,
            task_scratch: TaskScratch::default(),
            tool_call_count: 0,
            tool_traces: Vec::new(),
            skill_evolver: SkillEvolver::new(),
            stagnation: StagnationDetector::new(),
            sleep_engine: None,
            last_sleep_stats: None,
            dgm_strategy: None,
            goal_loop: GoalLoop::new(),
            goal_register: GoalRegister::new(),
            entropy_crisis_level: 0.0,
            curiosity_bonus: 0.0,
            seal_rl: {
                let mut s = SealRlState::default();
                s.harness_adapter = init_adapter;
                s
            },
            _transition_learner: E8TransitionLearner::default(),
            _e8_policy: crate::core::E8Policy::default(),
            _knowledge_distiller: KnowledgeDistiller::new(),
            _nt_memory_kb: init_kb,
            _strategy_matrix: strategy_matrix(),
            _trajectory_collector: crate::core::nt_core_prm::TrajectoryCollector::new(),
            _coach: Some(Box::new(crate::core::nt_core_prm::HeuristicCoach::default())),
            _ingestion_scratchpad: None,
            nt_act_crypto: None,
            nt_world_jepa: None,
            evo_pipeline: EvoPipelineState::default(),
            consciousness_state: ConsciousnessState::default(),
            _meta_agent: {
                let mut agent =
                    crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAgent::new();
                agent.meta_layer_can_rewrite_self = true;
                Some(agent)
            },
            meta_additions: Vec::new(),
            checkpoint_manager:
                crate::neotrix::nt_mind::self_iterating::checkpoint::CheckpointManager::new(),
            code_search_cache: None,
            recipe_registry: {
                let mut reg = RecipeRegistry::new();
                reg.register(crate::neotrix::nt_mind::self_iterating::recipe::preset_standard());
                reg.register(crate::neotrix::nt_mind::self_iterating::recipe::preset_kernel());
                reg.register(crate::neotrix::nt_mind::self_iterating::recipe::preset_debug());
                reg.register(crate::neotrix::nt_mind::self_iterating::recipe::preset_design());
                reg
            },
            curvature_policy: CurvaturePolicy::new(0.1, 0.01, 0.5, 0.2, 20),
            self_preservation: SelfPreservation::new(10),
            plugin_registry: {
                let reg = PluginRegistry::new();
                reg
            },
            perception_evolution: PerceptionEvolution::new(),
            side_git: SideGit::new(),
            goal_state: GoalContractState::default(),
            _open_source_insights: None,
            _tom: crate::neotrix::nt_mind::theory_of_mind::TheoryOfMind::new(),
            _negentropy: crate::neotrix::nt_core_negentropy::NegentropyCalculator::new(),
            _negentropy_nvsa_pool: Vec::new(),
            vsi_verifier: VsiVerifier::new(),
        }
    }

    /// Auto-tune thresholds and learning rates from archive activity.
    /// Call after the archive has accumulated at least a few entries.
    /// Uses change count as a proxy for maturity — more changes mean
    /// tighter thresholds and shorter memory intervals.
    pub fn auto_tune_from_archive(&mut self) {
        let n = self.archive.len();
        if n < 5 {
            return;
        }
        let maturity = (n as f64 / 100.0).min(1.0);
        self.quality_threshold = (0.85 * maturity).max(0.5).min(0.99);
        self.policy_learning_rate = (0.01 * (1.0 - maturity * 0.5)).max(0.001).min(0.1);
        self.memory_iteration_interval = (10u64.saturating_sub(n as u64 / 20)).max(3);
    }

    // ========== CryptoAgent integration (shared Arc<Mutex<>>) ==========
    pub fn with_nt_act_crypto(mut self, crypto: Arc<Mutex<CryptoAgent>>) -> Self {
        self.nt_act_crypto = Some(crypto);
        self
    }

    pub fn with_nt_world_jepa(mut self, jepa: JepaWorldModel) -> Self {
        self.nt_world_jepa = Some(jepa);
        self
    }

    pub fn nt_act_crypto_arc(&self) -> Option<Arc<Mutex<CryptoAgent>>> {
        self.nt_act_crypto.clone()
    }

    pub fn init_nt_act_crypto(&mut self) -> Arc<Mutex<CryptoAgent>> {
        if self.nt_act_crypto.is_none() {
            self.nt_act_crypto = Some(Arc::new(Mutex::new(CryptoAgent::new())));
        }
        self.nt_act_crypto.clone().unwrap()
    }

    pub fn run_crypto_iteration(&mut self) -> Option<f64> {
        let crypto_lock = self.init_nt_act_crypto();
        let mut crypto = crypto_lock.lock().unwrap_or_else(|e| e.into_inner());
        crypto.run_iteration();
        let opps = crypto.scan_opportunities();
        let total_value: f64 = opps.iter().map(|o| o.estimated_value_usd).sum();
        let insights = crypto.learn_and_adapt();
        let insights_str = insights.join("; ");
        if !insights_str.is_empty() {
            let v = self.brain.capability.clone();
            let boost = (total_value * 0.001).min(0.3).max(0.01);
            let mut evolved = v;
            evolved.set_analysis((evolved.analysis() + boost * 0.1).min(1.0));
            evolved.set_synthesis((evolved.synthesis() + boost * 0.05).min(1.0));
            self.brain
                .register_knowledge_source("nt_act_crypto::insights", evolved);
            let _ = self.brain.absorb_from_custom("nt_act_crypto::insights");
        }
        Some(total_value)
    }

    // ========== Pipeline scratchpad helpers ==========
    pub(crate) fn _current_task(&self) -> String {
        self.task_scratch.current_task.clone()
    }
    pub(crate) fn _current_task_type(&self) -> TaskType {
        self.task_scratch.current_task_type
    }
    pub(crate) fn _task_embedding(&self) -> Option<Vec<f64>> {
        self.task_scratch.task_embedding.clone()
    }
    pub(crate) fn _take_task_embedding(&mut self) -> Option<Vec<f64>> {
        self.task_scratch.task_embedding.take()
    }
    pub(crate) fn _external_reward(&self) -> Option<f64> {
        self.task_scratch.external_reward
    }
    pub(crate) fn _snapshot_score(&self) -> f64 {
        self.task_scratch
            .snapshot
            .as_ref()
            .map(|s| s.score)
            .unwrap_or(0.0)
    }
    pub(crate) fn _snapshot_lr(&self) -> f64 {
        self.task_scratch
            .snapshot
            .as_ref()
            .map(|s| s.learning_rate)
            .unwrap_or(0.01)
    }
    pub fn pipeline_status(&self) -> String {
        format!(
            "iter={}, reward={:.3}, task='{}'",
            self.iteration, self.task_scratch.reward, self.task_scratch.current_task
        )
    }
    pub(crate) fn _snapshot_capability(&self) -> CapabilityVector {
        self.task_scratch
            .snapshot
            .as_ref()
            .map(|s| s.capability.clone())
            .unwrap_or_default()
    }
    pub(crate) fn _set_snapshot(&mut self, s: BrainSnapshot) {
        self.task_scratch.snapshot = Some(s);
    }
    pub(crate) fn _snapshot_restore(&mut self) {
        if let Some(snap) = self.task_scratch.snapshot.clone() {
            snap.restore(&mut self.brain);
        }
    }
    pub(crate) fn _reward(&self) -> f64 {
        self.task_scratch.reward
    }
    pub(crate) fn _reward_source(&self) -> RewardSource {
        self.task_scratch.reward_source
    }
    pub(crate) fn _set_reward(&mut self, r: f64) {
        self.task_scratch.reward = r;
    }
    pub(crate) fn _set_reward_source(&mut self, s: RewardSource) {
        self.task_scratch.reward_source = s;
    }
    pub(crate) fn _micro_edits(&self) -> Vec<MicroEdit> {
        self.task_scratch.micro_edits.clone()
    }
    pub(crate) fn _set_micro_edits(&mut self, edits: Vec<MicroEdit>) {
        self.task_scratch.micro_edits = edits;
    }
    pub(crate) fn _take_micro_edits(&mut self) -> Vec<MicroEdit> {
        std::mem::take(&mut self.task_scratch.micro_edits)
    }
    pub(crate) fn _set_open_source_edits(&mut self, edits: Vec<MicroEdit>) {
        self.evo_pipeline.open_source_edits = edits;
    }
    pub(crate) fn _take_open_source_edits(&mut self) -> Vec<MicroEdit> {
        std::mem::take(&mut self.evo_pipeline.open_source_edits)
    }
    pub(crate) fn _set_lr_scheduler(&mut self, s: LrScheduler) {
        self.seal_rl.lr_scheduler = s;
    }
    pub(crate) fn _set_validation_gate(&mut self, g: ValidationGate) {
        self.seal_rl.validation_gate = g;
    }
    pub(crate) fn _set_aging_monitor(&mut self, m: AgingMonitor) {
        self.seal_rl.aging_monitor = m;
    }

    pub(crate) fn _ingestion_scratchpad(&self) -> Option<&IngestionScratchpad> {
        self._ingestion_scratchpad.as_ref()
    }
    pub(crate) fn _ingestion_scratchpad_mut(&mut self) -> Option<&mut IngestionScratchpad> {
        self._ingestion_scratchpad.as_mut()
    }

    pub fn record_tool_call(&mut self, name: &str, duration_ms: u64, success: bool) {
        self.tool_call_count += 1;
        self.tool_traces
            .push((name.to_string(), duration_ms, success));
        if self.tool_traces.len() > 100 {
            self.tool_traces.remove(0);
        }
    }

    /// 预览 absorb 效果：计算吸收后的向量但不修改状态
    pub fn preview_absorb(
        &self,
        source: KnowledgeSource,
    ) -> (CapabilityVector, CapabilityVector, f64) {
        let before = self.brain.capability.clone();
        let source_vector = source.capability_vector();
        let mut simulated = before.clone();
        simulated.update_from_other(&source_vector, self.brain.learning_rate);
        simulated.normalize();
        let delta = crate::neotrix::nt_core_signal::ops::cosine_similarity(
            &simulated.to_full_vector(),
            &before.to_full_vector(),
        );
        (before, simulated, delta)
    }

    /// 预览 MicroEdit 序列的效果
    pub fn preview_edit(&self, _edits: &[MicroEdit]) -> (CapabilityVector, f64) {
        let mut simulated = self.brain.capability.clone();
        let dummy_source = KnowledgeSource::DesignPhilosophy;
        simulated.update_from_other(&dummy_source.capability_vector(), self.brain.learning_rate);
        simulated.normalize();
        let delta = crate::neotrix::nt_core_signal::ops::cosine_similarity(
            &simulated.to_full_vector(),
            &self.brain.capability.to_full_vector(),
        );
        (simulated, delta)
    }

    /// 安全吸收：三阶段法
    pub fn safe_absorb(
        &mut self,
        source: KnowledgeSource,
        validator: Option<&dyn AbsorbValidator>,
    ) -> bool {
        let snapshot = self.brain.capability.clone();
        let snapshot_lr = self.brain.learning_rate;
        let (_before, _after, delta) = self.preview_absorb(source);
        if delta < 0.001 {
            return false;
        }
        self.brain.absorb(source);
        if let Some(v) = validator {
            if !v.validate_absorb(&self.brain.capability) {
                self.brain.capability = snapshot;
                self.brain.learning_rate = snapshot_lr;
                return false;
            }
        }
        let edits = vec![
            crate::neotrix::nt_mind::self_edit::MicroEdit::AdjustDimension(
                format!("source_{:?}", source),
                delta,
            ),
        ];
        self.archive
            .record(&format!("absorb_{:?}", source), "safe_absorb", &edits);
        true
    }

    /// 显示能力向量变化量（类似 git diff）
    pub fn diff(&self, before: &CapabilityVector) -> String {
        let after = &self.brain.capability;
        let mut changes = Vec::new();
        for (i, name) in crate::neotrix::nt_mind::core::FIELD_NAMES
            .iter()
            .enumerate()
        {
            let diff = after.arr()[i] - before.arr()[i];
            if diff.abs() > 0.001 {
                let sign = if diff > 0.0 { "+" } else { "" };
                changes.push(format!(
                    "  {}: {:.3} → {:.3} ({} {:.3})",
                    name,
                    before.arr()[i],
                    after.arr()[i],
                    sign,
                    diff
                ));
            }
        }
        for (name, val) in &after.extension {
            let before_val = before
                .extension
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            let diff = val - before_val;
            if diff.abs() > 0.001 {
                let sign = if diff > 0.0 { "+" } else { "" };
                changes.push(format!(
                    "  ext.{}: {:.3} → {:.3} ({} {:.3})",
                    name, before_val, val, sign, diff
                ));
            }
        }
        changes.join("\n")
    }

    /// 从 CriticNode 进行独立验证（吸收前的质量门控）
    pub fn absorb_with_critic(&mut self, source: KnowledgeSource) -> bool {
        let (before, _after, _delta) = self.preview_absorb(source);
        let before_score = PerformanceEvaluator::evaluate(&TaskType::General, &before);
        let did_absorb = self.safe_absorb(source, None);
        if did_absorb {
            let after_score =
                PerformanceEvaluator::evaluate(&TaskType::General, &self.brain.capability);
            if after_score < before_score - 0.01 {
                self.brain.capability = before;
                return false;
            }
            let _entry = self.archive.record(
                &format!("critic_absorb_{:?}", source),
                "absorb_with_critic",
                &[
                    crate::neotrix::nt_mind::self_edit::MicroEdit::AdjustDimension(
                        "verified".to_string(),
                        0.01,
                    ),
                ],
            );
        }
        did_absorb
    }

    pub fn select_relevant_sources(&self, task_type: TaskType) -> Vec<KnowledgeSource> {
        match task_type {
            TaskType::Design | TaskType::UIDesign => {
                vec![
                    KnowledgeSource::HeroUI,
                    KnowledgeSource::BaseUI,
                    KnowledgeSource::ArcUI,
                    KnowledgeSource::CortexUI,
                    KnowledgeSource::AgenticDS,
                ]
            }
            TaskType::CodeAnalysis | TaskType::CodeGeneration => {
                vec![
                    KnowledgeSource::DesignPhilosophy,
                    KnowledgeSource::DeepSeekTui,
                    KnowledgeSource::Codebuff,
                ]
            }
            TaskType::Security => {
                vec![KnowledgeSource::Cairn, KnowledgeSource::RedRun]
            }
            TaskType::Research | TaskType::Learning => {
                vec![
                    KnowledgeSource::OpenClaude,
                    KnowledgeSource::AutonomousSpeedrunning,
                ]
            }
            _ => vec![],
        }
    }

    pub fn get_brain_report(&self) -> crate::neotrix::nt_mind::stats::BrainReport {
        let stats = self.brain.get_statistics();
        crate::neotrix::nt_mind::stats::BrainReport {
            iteration: self.iteration,
            total_absorbed: stats.total_absorbed,
            capability_sum: stats.capability_sum,
            recent_improvement: self
                .evaluation_history
                .iter()
                .rev()
                .take(5)
                .filter(|r| r.improved)
                .count(),
        }
    }

    /// 主动触发知识库记忆迭代（合并+修剪+回放）
    pub fn consolidate_memories(&mut self) -> crate::core::nt_core_bank::MemoryIterationResult {
        self.reasoning_bank.iterate_memories(0.85, 0.1)
    }

    /// Enable DGM diffusion strategy with the given number of steps.
    pub fn use_dgm_strategy(&mut self, num_steps: usize) {
        self.dgm_strategy = Some(
            crate::neotrix::nt_mind::self_iterating::brain_dgm::DgmSelfEditStrategy::new(num_steps),
        );
    }

    /// Enable DGM with custom noise schedule.
    pub fn use_dgm_strategy_with_schedule(&mut self, num_steps: usize, schedule: Vec<f64>) {
        self.dgm_strategy = Some(
            crate::neotrix::nt_mind::self_iterating::brain_dgm::DgmSelfEditStrategy::with_schedule(
                num_steps, schedule,
            ),
        );
    }

    /// Disable DGM and revert to default `generate_self_edit()`.
    pub fn disable_dgm_strategy(&mut self) {
        self.dgm_strategy = None;
    }
}

// ==================== EvoAgentBench — 自进化评估指标 ====================

#[derive(Debug, Clone, Default)]
pub struct EvoStats {
    pub growth_slope: f64,
    pub recent_improvement_avg: f64,
    pub transfer_efficiency: f64,
    pub error_avoidance_rate: f64,
    pub tool_success_rate: f64,
    pub cms_promotion_rate: f64,
    pub health_score: f64,
}

impl SelfIteratingBrain {
    pub fn growth_curve_slope(&self) -> f64 {
        let n = self.evaluation_history.len();
        if n < 3 {
            return 0.0;
        }
        let recent: Vec<_> = self.evaluation_history.iter().rev().take(10).collect();
        let n = recent.len();
        let indices: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let scores: Vec<f64> = recent.iter().map(|r| r.score_after).collect();
        let mean_x = indices.iter().sum::<f64>() / n as f64;
        let mean_y = scores.iter().sum::<f64>() / n as f64;
        let num: f64 = indices
            .iter()
            .zip(scores.iter())
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum();
        let den: f64 = indices.iter().map(|x| (x - mean_x).powi(2)).sum();
        if den.abs() < 1e-12 {
            0.0
        } else {
            num / den
        }
    }

    pub fn transfer_efficiency(&self) -> f64 {
        if self.evaluation_history.len() < 2 {
            return 0.0;
        }
        let recent: Vec<_> = self.evaluation_history.iter().rev().take(20).collect();
        if recent.len() < 2 {
            return 0.0;
        }
        let mut cross_sims = Vec::new();
        for i in 1..recent.len() {
            let t1 = &recent[i - 1];
            let t2 = &recent[i];
            if t1.task_type != t2.task_type {
                let sim = 1.0 - (t1.score_before - t2.score_after).abs().min(1.0);
                cross_sims.push(sim);
            }
        }
        if cross_sims.is_empty() {
            0.0
        } else {
            cross_sims.iter().sum::<f64>() / cross_sims.len() as f64
        }
    }

    pub fn error_avoidance_rate(&self) -> f64 {
        let total = self.archive.len();
        if total < 2 {
            return 1.0;
        }
        let conflicts = self.archive.all_conflicts().len();
        let error_rate = conflicts as f64 / total as f64;
        (1.0 - error_rate).clamp(0.0, 1.0)
    }

    pub fn evo_stats(&self) -> EvoStats {
        let growth = self.growth_curve_slope();
        let transfer = self.transfer_efficiency();
        let error_avoid = self.error_avoidance_rate();
        let recent_imp: f64 = self
            .evaluation_history
            .iter()
            .rev()
            .take(5)
            .map(|r| if r.improved { 1.0 } else { 0.0 })
            .sum::<f64>()
            / 5.0_f64.max(1.0);
        let tool_success = if self.tool_call_count > 0 {
            let success = self.tool_traces.iter().filter(|(_, _, s)| *s).count() as f64;
            success / self.tool_call_count as f64
        } else {
            0.5
        };
        let cms_promote = self.cortex.stats().nt_world_sense_count.max(1) as f64;
        let cms_rate = (self.cortex.stats().long_term_count as f64 / cms_promote).min(1.0);
        let health = (growth.clamp(0.0, 1.0) * 0.2
            + recent_imp * 0.2
            + transfer * 0.15
            + error_avoid * 0.15
            + tool_success * 0.2
            + cms_rate * 0.1)
            .clamp(0.0, 1.0);
        EvoStats {
            growth_slope: growth,
            recent_improvement_avg: recent_imp,
            transfer_efficiency: transfer,
            error_avoidance_rate: error_avoid,
            tool_success_rate: tool_success,
            cms_promotion_rate: cms_rate,
            health_score: health,
        }
    }
}

impl Default for SelfIteratingBrain {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::neotrix::nt_mind::self_iterating::brain_impl::SelfIteration for SelfIteratingBrain {
    type IterationResult = crate::neotrix::nt_mind::stats::IterationResult;
    type Evaluation = f64;

    fn iterate(&mut self) -> Self::IterationResult {
        self.iterate(crate::neotrix::nt_expert_routing::TaskType::General)
    }

    fn evaluate(&self) -> Self::Evaluation {
        self.brain
            .evaluate_capability(crate::neotrix::nt_expert_routing::TaskType::General)
    }

    fn absorb_feedback(&mut self, feedback: f64) {
        if feedback > 0.5 {
            self.brain.absorb(KnowledgeSource::DesignPhilosophy);
        }
    }

    fn should_continue(&self, threshold: f64) -> bool {
        let base_score = self
            .brain
            .evaluate_capability(crate::neotrix::nt_expert_routing::TaskType::General);
        let aging_penalty = self.seal_rl.aging_monitor.overall_aging() * 0.2;
        let adjusted_threshold = threshold * (1.0 - aging_penalty);
        base_score < adjusted_threshold
    }
}
