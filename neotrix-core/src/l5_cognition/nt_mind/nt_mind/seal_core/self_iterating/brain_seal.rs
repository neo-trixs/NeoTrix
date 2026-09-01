use super::brain_core::ReasoningBrain;
use super::super::self_edit::{select_relevant_dimensions, calculate_adjustment_magnitude, infer_task_type};

/// SEAL edit strategy — 可替换的自我编辑策略
pub trait SealEditStrategy: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn generate_edit(&self, brain: &ReasoningBrain, task: &str) -> Vec<CapabilityDelta>;
}

/// 单维度能力变更描述
#[derive(Debug, Clone)]
pub struct CapabilityDelta {
    pub dimension: String,
    pub delta: f64,
    pub confidence: f64,
}

/// 默认策略：规则驱动
#[derive(Debug, Clone)]
pub struct DefaultSealStrategy;

impl SealEditStrategy for DefaultSealStrategy {
    fn name(&self) -> &'static str { "default" }
    fn generate_edit(&self, _brain: &ReasoningBrain, task: &str) -> Vec<CapabilityDelta> {
        let task_type = infer_task_type(task);
        let magnitude = calculate_adjustment_magnitude(&task_type);
        let dims = select_relevant_dimensions(&task_type);
        dims.into_iter().map(|dim| CapabilityDelta {
            dimension: dim,
            delta: magnitude,
            confidence: 0.8,
        }).collect()
    }
}

/// 保守策略
#[derive(Debug, Clone)]
pub struct ConservativeSealStrategy {
    pub max_delta: f64,
}

impl SealEditStrategy for ConservativeSealStrategy {
    fn name(&self) -> &'static str { "conservative" }
    fn generate_edit(&self, _brain: &ReasoningBrain, task: &str) -> Vec<CapabilityDelta> {
        let task_type = infer_task_type(task);
        let magnitude = calculate_adjustment_magnitude(&task_type).min(self.max_delta);
        let dims = select_relevant_dimensions(&task_type);
        dims.into_iter().map(|dim| CapabilityDelta {
            dimension: dim,
            delta: magnitude,
            confidence: 0.6,
        }).collect()
    }
}

/// 激进策略
#[derive(Debug, Clone)]
pub struct AggressiveSealStrategy {
    pub max_delta: f64,
}

impl SealEditStrategy for AggressiveSealStrategy {
    fn name(&self) -> &'static str { "aggressive" }
    fn generate_edit(&self, _brain: &ReasoningBrain, task: &str) -> Vec<CapabilityDelta> {
        let task_type = infer_task_type(task);
        let magnitude = calculate_adjustment_magnitude(&task_type).max(self.max_delta);
        let dims = select_relevant_dimensions(&task_type);
        dims.into_iter().map(|dim| CapabilityDelta {
            dimension: dim,
            delta: magnitude,
            confidence: 0.9,
        }).collect()
    }
}
