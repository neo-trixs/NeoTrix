//! Capability Routing - 模型能力路由
//!
//! Smart Friend 模式: 根据任务复杂度选择合适的模型
//! - 简单/快速任务 → 小模型
//! - 复杂/推理任务 → 大模型
//! - 成本与性能平衡

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelSize {
    Small,
    Medium,
    Large,
    Frontier,
}

impl ModelSize {
    pub fn is_small(&self) -> bool {
        matches!(self, Self::Small)
    }

    pub fn debug_name(&self) -> &'static str {
        match self {
            ModelSize::Small => "Small",
            ModelSize::Medium => "Medium",
            ModelSize::Large => "Large",
            ModelSize::Frontier => "Frontier",
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            Self::Small => 1,
            Self::Medium => 2,
            Self::Large => 3,
            Self::Frontier => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapability {
    pub model_id: String,
    pub provider: String,
    pub size: ModelSize,
    pub cost_per_1k_tokens: f64,
    pub latency_ms_estimate: u64,
    pub supported_tasks: Vec<String>,
    pub max_tokens: u32,
    pub reasoning_effort: Option<String>,
}

impl ModelCapability {
    pub fn new(model_id: impl Into<String>, provider: impl Into<String>, size: ModelSize) -> Self {
        Self {
            model_id: model_id.into(),
            provider: provider.into(),
            size,
            cost_per_1k_tokens: 0.0,
            latency_ms_estimate: 0,
            supported_tasks: Vec::new(),
            max_tokens: 128000,
            reasoning_effort: None,
        }
    }

    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost_per_1k_tokens = cost;
        self
    }

    pub fn with_latency(mut self, ms: u64) -> Self {
        self.latency_ms_estimate = ms;
        self
    }

    pub fn supports_task(&self, task: &str) -> bool {
        self.supported_tasks.is_empty() || self.supported_tasks.contains(&task.to_string())
    }
}

pub struct RouterDecision {
    pub primary: ModelCapability,
    pub fallbacks: Vec<ModelCapability>,
    pub reasoning: String,
}

pub struct CapabilityRouter {
    models: Vec<ModelCapability>,
}

impl CapabilityRouter {
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    pub fn register(&mut self, model: ModelCapability) {
        self.models.push(model);
    }

    pub fn select(&self, task_complexity: &str, budget: Option<f64>) -> RouterDecision {
        let target_size = match task_complexity {
            "simple" | "quick" => ModelSize::Small,
            "standard" => ModelSize::Medium,
            "complex" | "reasoning" => ModelSize::Large,
            "frontier" => ModelSize::Frontier,
            _ => ModelSize::Medium,
        };

        let primary = self
            .models
            .iter()
            .find(|m| m.size == target_size && m.cost_per_1k_tokens <= budget.unwrap_or(f64::MAX))
            .cloned()
            .unwrap_or_else(|| {
                self.models
                    .iter()
                    .find(|m| m.size == ModelSize::Medium)
                    .cloned()
                    .unwrap_or_else(|| ModelCapability::new(
                        "claude-sonnet-4-5",
                        "anthropic",
                        ModelSize::Medium,
                    ))
            });

        let fallbacks: Vec<ModelCapability> = self
            .models
            .iter()
            .filter(|m| m.size.priority() < target_size.priority())
            .take(2)
            .cloned()
            .collect();

        let reasoning = format!(
            "Selected {} ({}) for {} task, fallbacks: {:?}",
            primary.model_id,
            primary.size.debug_name(),
            task_complexity,
            fallbacks.iter().map(|m| &m.model_id).collect::<Vec<_>>()
        );

        RouterDecision {
            primary,
            fallbacks,
            reasoning,
        }
    }
}

impl Default for CapabilityRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelCapability {
    pub fn debug_name(&self) -> String {
        format!("{}:{}/{:?}", self.provider, self.model_id, self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_simple_task() {
        let mut router = CapabilityRouter::new();
        router.register(ModelCapability::new("haiku-4-5", "anthropic", ModelSize::Small).with_cost(0.0002));
        router.register(ModelCapability::new("sonnet-4-5", "anthropic", ModelSize::Medium).with_cost(0.003));
        router.register(ModelCapability::new("opus-4-5", "anthropic", ModelSize::Large).with_cost(0.015));

        let decision = router.select("simple", None);
        assert_eq!(decision.primary.model_id, "haiku-4-5");
    }

    #[test]
    fn test_router_complex_task() {
        let mut router = CapabilityRouter::new();
        router.register(ModelCapability::new("haiku-4-5", "anthropic", ModelSize::Small).with_cost(0.0002));
        router.register(ModelCapability::new("sonnet-4-5", "anthropic", ModelSize::Medium).with_cost(0.003));
        router.register(ModelCapability::new("opus-4-5", "anthropic", ModelSize::Large).with_cost(0.015));

        let decision = router.select("complex", Some(0.01));
        assert_eq!(decision.primary.model_id, "sonnet-4-5");
    }

    #[test]
    fn test_model_size_priority() {
        assert!(ModelSize::Small.priority() < ModelSize::Medium.priority());
        assert!(ModelSize::Medium.priority() < ModelSize::Large.priority());
        assert!(ModelSize::Large.priority() < ModelSize::Frontier.priority());
    }
}