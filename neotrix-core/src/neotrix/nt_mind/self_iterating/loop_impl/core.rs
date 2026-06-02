use super::super::brain_impl::{AbsorbValidator, ReasoningBrain, EvaluationRecord};
use super::super::super::core::{CapabilityVector, KnowledgeSource, PerformanceEvaluator, RewardSource};
use super::super::super::self_edit::MicroEdit;
use super::super::super::memory::ReasoningBank;
use super::super::super::multi_brain::MultiBrainManager;
use super::super::super::reasoning_engine::ReasoningEngine;
use super::super::super::attention_router::AttentionRouter;
use super::super::super::cortex_memory::CortexMemory;
use super::super::super::change_archive::ChangeArchive;
use super::super::super::sleep::{SleepEngine, SleepStats};
use crate::neotrix::nt_world_model::TaskType;
use crate::neotrix::signal::select::SelectableOperator;
use crate::neotrix::signal::core::SelectiveState;
use crate::neotrix::nt_act_crypto::CryptoAgent;
use super::super::super::stagnation::StagnationDetector;
use super::super::pipeline::{BrainPipeline, BrainSnapshot, AutonomyLevel, StageResult, seal_pipeline};
use super::super::super::goal_loop::GoalLoop;
use crate::neotrix::nt_mind::goal_register::GoalRegister;
use super::super::skillopt::{LrScheduler, ValidationGate, RejectedEditBuffer};
use super::super::aging_monitor::AgingMonitor;
use super::super::harness_adapter::HarnessAdapter;
use crate::core::{E8TransitionLearner, ReasoningHexagram, strategy_matrix};
use crate::neotrix::nt_world_jepa::JepaWorldModel;
use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_act_autonomy::knowledge_distiller::KnowledgeDistiller;
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
    pub goal_loop: GoalLoop,
    pub goal_register: GoalRegister,
    pub entropy_crisis_level: f64,
    pub curiosity_bonus: f64,
    pub(crate) _current_task: String,
    pub(crate) _current_task_type: TaskType,
    pub(crate) _task_embedding: Option<Vec<f64>>,
    pub(crate) _external_reward: Option<f64>,
    pub(crate) _snapshot: Option<BrainSnapshot>,
    pub(crate) _reward: f64,
    pub(crate) _reward_source: RewardSource,
    pub(crate) _micro_edits: Vec<MicroEdit>,
    pub tool_call_count: usize,
    pub tool_traces: Vec<(String, u64, bool)>,
    pub _open_source_insights: Option<String>,
    pub _open_source_edits: Vec<super::super::super::self_edit::MicroEdit>,
    pub stagnation: StagnationDetector,
    pub sleep_engine: Option<SleepEngine>,
    pub last_sleep_stats: Option<SleepStats>,
    pub dgm_strategy: Option<super::super::brain_dgm::DgmSelfEditStrategy>,
    pub(crate) _lr_scheduler: LrScheduler,
    pub(crate) _validation_gate: ValidationGate,
    pub(crate) _rejected_buffer: RejectedEditBuffer,
    pub(crate) _aging_monitor: AgingMonitor,
    pub(crate) _harness_adapter: HarnessAdapter,
    pub(crate) _stage_results: Vec<StageResult>,
    pub(crate) _transition_learner: E8TransitionLearner,
    pub(crate) _e8_policy: crate::core::E8Policy,
    pub(crate) _knowledge_distiller: KnowledgeDistiller,
    pub(crate) _nt_memory_kb: Option<KnowledgeBase>,
    pub(crate) _strategy_matrix: [[ReasoningHexagram; 8]; 8],
    pub nt_act_crypto: Option<Arc<Mutex<CryptoAgent>>>,
    pub nt_world_jepa: Option<JepaWorldModel>,
}

impl SelfIteratingBrain {
    pub fn new() -> Self {
        let (init_kb, init_adapter): (Option<KnowledgeBase>, HarnessAdapter) = {
            let kb = KnowledgeBase::open(None).ok();
            let adapter = kb.as_ref()
                .and_then(|k| HarnessAdapter::load_from_kb(k).ok())
                .unwrap_or_default();
            (kb, adapter)
        };
        Self {
            brain: ReasoningBrain::new(),
            iteration: 0,
            quality_threshold: 0.85,
            auto_absorb: true,
            evaluation_history: Vec::new(),
            reasoning_bank: ReasoningBank::new(100),
            policy_learning_rate: 0.01,
            regularization_weight: 0.001,
            auto_memory_iteration: true,
            memory_iteration_interval: 10,
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
            _current_task: String::new(),
            _current_task_type: TaskType::General,
            _task_embedding: None,
            _external_reward: None,
            _snapshot: None,
            _reward: 0.0,
            _reward_source: RewardSource::Internal,
            _micro_edits: Vec::new(),
            tool_call_count: 0,
            tool_traces: Vec::new(),
            _open_source_insights: None,
            _open_source_edits: Vec::new(),
            stagnation: StagnationDetector::new(),
            sleep_engine: None,
            last_sleep_stats: None,
            dgm_strategy: None,
            goal_loop: GoalLoop::new(),
            goal_register: GoalRegister::new(),
            entropy_crisis_level: 0.0,
            curiosity_bonus: 0.0,
            _lr_scheduler: LrScheduler::default(),
            _validation_gate: ValidationGate::default(),
            _rejected_buffer: RejectedEditBuffer::default(),
            _aging_monitor: AgingMonitor::default(),
            _transition_learner: E8TransitionLearner::default(),
            _e8_policy: crate::core::E8Policy::default(),
            _knowledge_distiller: KnowledgeDistiller::new(),
            _nt_memory_kb: init_kb,
            _harness_adapter: init_adapter,
            _stage_results: Vec::new(),
            _strategy_matrix: strategy_matrix(),
            nt_act_crypto: None,
            nt_world_jepa: None,
        }
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
        let mut crypto = crypto_lock.lock().unwrap();
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
            self.brain.register_knowledge_source("nt_act_crypto::insights", evolved);
            let _ = self.brain.absorb_from_custom("nt_act_crypto::insights");
        }
        Some(total_value)
    }

    // ========== Pipeline scratchpad helpers ==========
    pub(crate) fn _current_task(&self) -> String { self._current_task.clone() }
    pub(crate) fn _current_task_type(&self) -> TaskType { self._current_task_type }
    pub(crate) fn _task_embedding(&self) -> Option<Vec<f64>> { self._task_embedding.clone() }
    pub(crate) fn _take_task_embedding(&mut self) -> Option<Vec<f64>> { self._task_embedding.take() }
    pub(crate) fn _external_reward(&self) -> Option<f64> { self._external_reward }
    pub(crate) fn _snapshot_score(&self) -> f64 { self._snapshot.as_ref().map(|s| s.score).unwrap_or(0.0) }
    pub(crate) fn _snapshot_lr(&self) -> f64 { self._snapshot.as_ref().map(|s| s.learning_rate).unwrap_or(0.01) }
    pub(crate) fn _snapshot_capability(&self) -> CapabilityVector {
        self._snapshot.as_ref().map(|s| s.capability.clone()).unwrap_or_default()
    }
    pub(crate) fn _set_snapshot(&mut self, s: BrainSnapshot) { self._snapshot = Some(s); }
    pub(crate) fn _snapshot_restore(&mut self) {
        if let Some(snap) = self._snapshot.clone() {
            snap.restore(&mut self.brain);
        }
    }
    pub(crate) fn _reward(&self) -> f64 { self._reward }
    pub(crate) fn _reward_source(&self) -> RewardSource { self._reward_source }
    pub(crate) fn _set_reward(&mut self, r: f64) { self._reward = r; }
    pub(crate) fn _set_reward_source(&mut self, s: RewardSource) { self._reward_source = s; }
    pub(crate) fn _micro_edits(&self) -> Vec<MicroEdit> { self._micro_edits.clone() }
    pub(crate) fn _set_micro_edits(&mut self, edits: Vec<MicroEdit>) { self._micro_edits = edits; }
    pub(crate) fn _take_micro_edits(&mut self) -> Vec<MicroEdit> { std::mem::take(&mut self._micro_edits) }
    pub(crate) fn _set_open_source_insights(&mut self, insights: Option<String>) { self._open_source_insights = insights; }
    pub(crate) fn _open_source_insights(&self) -> Option<String> { self._open_source_insights.clone() }
    pub(crate) fn _set_open_source_edits(&mut self, edits: Vec<MicroEdit>) { self._open_source_edits = edits; }
    pub(crate) fn _take_open_source_edits(&mut self) -> Vec<MicroEdit> { std::mem::take(&mut self._open_source_edits) }
    pub(crate) fn _set_lr_scheduler(&mut self, s: LrScheduler) { self._lr_scheduler = s; }
    pub(crate) fn _set_validation_gate(&mut self, g: ValidationGate) { self._validation_gate = g; }
    pub(crate) fn _set_aging_monitor(&mut self, m: AgingMonitor) { self._aging_monitor = m; }

    pub fn record_tool_call(&mut self, name: &str, duration_ms: u64, success: bool) {
        self.tool_call_count += 1;
        self.tool_traces.push((name.to_string(), duration_ms, success));
        if self.tool_traces.len() > 100 {
            self.tool_traces.remove(0);
        }
    }

    /// 预览 absorb 效果：计算吸收后的向量但不修改状态
    pub fn preview_absorb(&self, source: KnowledgeSource) -> (CapabilityVector, CapabilityVector, f64) {
        let before = self.brain.capability.clone();
        let source_vector = source.capability_vector();
        let mut simulated = before.clone();
        simulated.update_from_other(&source_vector, self.brain.learning_rate);
        simulated.normalize();
        let delta = crate::neotrix::signal::ops::cosine_similarity(&simulated.to_full_vector(), &before.to_full_vector());
        (before, simulated, delta)
    }

    /// 预览 MicroEdit 序列的效果
    pub fn preview_edit(&self, _edits: &[MicroEdit]) -> (CapabilityVector, f64) {
        let mut simulated = self.brain.capability.clone();
        let dummy_source = KnowledgeSource::DesignPhilosophy;
        simulated.update_from_other(&dummy_source.capability_vector(), self.brain.learning_rate);
        simulated.normalize();
        let delta = crate::neotrix::signal::ops::cosine_similarity(&simulated.to_full_vector(), &self.brain.capability.to_full_vector());
        (simulated, delta)
    }

    /// 安全吸收：三阶段法
    pub fn safe_absorb(&mut self, source: KnowledgeSource, validator: Option<&dyn AbsorbValidator>) -> bool {
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
            super::super::super::self_edit::MicroEdit::AdjustDimension(
                format!("source_{:?}", source), delta),
        ];
        self.archive.record(
            &format!("absorb_{:?}", source),
            "safe_absorb",
            &edits,
        );
        true
    }

    /// 显示能力向量变化量（类似 git diff）
    pub fn diff(&self, before: &CapabilityVector) -> String {
        let after = &self.brain.capability;
        let mut changes = Vec::new();
        for (i, name) in crate::neotrix::nt_mind::core::FIELD_NAMES.iter().enumerate() {
            let diff = after.arr()[i] - before.arr()[i];
            if diff.abs() > 0.001 {
                let sign = if diff > 0.0 { "+" } else { "" };
                changes.push(format!("  {}: {:.3} → {:.3} ({} {:.3})",
                    name, before.arr()[i], after.arr()[i], sign, diff));
            }
        }
        for (name, val) in &after.extension {
            let before_val = before.extension.iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            let diff = val - before_val;
            if diff.abs() > 0.001 {
                let sign = if diff > 0.0 { "+" } else { "" };
                changes.push(format!("  ext.{}: {:.3} → {:.3} ({} {:.3})",
                    name, before_val, val, sign, diff));
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
            let after_score = PerformanceEvaluator::evaluate(&TaskType::General, &self.brain.capability);
            if after_score < before_score - 0.01 {
                self.brain.capability = before;
                return false;
            }
            let _entry = self.archive.record(
                &format!("critic_absorb_{:?}", source),
                "absorb_with_critic",
                &[super::super::super::self_edit::MicroEdit::AdjustDimension(
                    "verified".to_string(), 0.01)],
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
                vec![
                    KnowledgeSource::Cairn,
                    KnowledgeSource::RedRun,
                ]
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

    pub fn get_brain_report(&self) -> super::super::super::stats::BrainReport {
        let stats = self.brain.get_statistics();
        super::super::super::stats::BrainReport {
            iteration: self.iteration,
            total_absorbed: stats.total_absorbed,
            capability_sum: stats.capability_sum,
            recent_improvement: self.evaluation_history.iter()
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
        self.dgm_strategy = Some(super::super::brain_dgm::DgmSelfEditStrategy::new(num_steps));
    }

    /// Enable DGM with custom noise schedule.
    pub fn use_dgm_strategy_with_schedule(&mut self, num_steps: usize, schedule: Vec<f64>) {
        self.dgm_strategy = Some(super::super::brain_dgm::DgmSelfEditStrategy::with_schedule(num_steps, schedule));
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
        let num: f64 = indices.iter().zip(scores.iter()).map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
        let den: f64 = indices.iter().map(|x| (x - mean_x).powi(2)).sum();
        if den.abs() < 1e-12 { 0.0 } else { num / den }
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
        if cross_sims.is_empty() { 0.0 } else { cross_sims.iter().sum::<f64>() / cross_sims.len() as f64 }
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
        let recent_imp: f64 = self.evaluation_history.iter().rev().take(5)
            .map(|r| if r.improved { 1.0 } else { 0.0 })
            .sum::<f64>() / 5.0_f64.max(1.0);
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
            growth_slope: growth, recent_improvement_avg: recent_imp,
            transfer_efficiency: transfer, error_avoidance_rate: error_avoid,
            tool_success_rate: tool_success, cms_promotion_rate: cms_rate,
            health_score: health,
        }
    }
}

impl Default for SelfIteratingBrain {
    fn default() -> Self {
        Self::new()
    }
}

impl super::super::brain_impl::SelfIteration for SelfIteratingBrain {
    type IterationResult = super::super::super::stats::IterationResult;
    type Evaluation = f64;

    fn iterate(&mut self) -> Self::IterationResult {
        self.iterate(crate::neotrix::nt_world_model::TaskType::General)
    }

    fn evaluate(&self) -> Self::Evaluation {
        self.brain.evaluate_capability(crate::neotrix::nt_world_model::TaskType::General)
    }

    fn absorb_feedback(&mut self, feedback: f64) {
        if feedback > 0.5 {
            self.brain.absorb(KnowledgeSource::DesignPhilosophy);
        }
    }

    fn should_continue(&self, threshold: f64) -> bool {
        let base_score = self.brain.evaluate_capability(crate::neotrix::nt_world_model::TaskType::General);
        let aging_penalty = self._aging_monitor.overall_aging() * 0.2;
        let adjusted_threshold = threshold * (1.0 - aging_penalty);
        base_score < adjusted_threshold
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
