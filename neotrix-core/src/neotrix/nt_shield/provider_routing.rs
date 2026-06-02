use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, PartialEq)]
pub enum ModelProvider {
    OpenAI,
    Anthropic,
    Google,
    Azure,
    Local,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelCapability {
    Chat,
    CodeGeneration,
    Reasoning,
    Embedding,
    ImageGeneration,
    AudioTranscription,
}

#[derive(Debug, Clone)]
pub struct ModelDefinition {
    pub name: String,
    pub provider: ModelProvider,
    pub capabilities: Vec<ModelCapability>,
    pub context_window: usize,
    pub cost_per_1k_input: f64,
    pub cost_per_1k_output: f64,
    pub max_output_tokens: usize,
    pub is_available: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoutingStrategy {
    LowestCost,
    FastestResponse,
    HighestCapability,
    FallbackChain,
    LoadBalanced,
}

#[derive(Debug, Clone)]
pub struct RoutingRecord {
    pub task: String,
    pub selected_model: String,
    pub strategy: RoutingStrategy,
    pub success: bool,
    pub latency_ms: u64,
    pub cost: f64,
}

#[derive(Debug)]
pub struct ProviderRouter {
    pub models: Vec<ModelDefinition>,
    pub strategy: RoutingStrategy,
    pub routing_history: Vec<RoutingRecord>,
    pub health_checks: HashMap<String, bool>,
    round_robin_counter: AtomicUsize,
}

impl ProviderRouter {
    pub fn new(models: Vec<ModelDefinition>, strategy: RoutingStrategy) -> Self {
        let health_checks: HashMap<String, bool> =
            models.iter().map(|m| (m.name.clone(), m.is_available)).collect();
        Self {
            models,
            strategy,
            routing_history: Vec::new(),
            health_checks,
            round_robin_counter: AtomicUsize::new(0),
        }
    }

    pub fn register_model(&mut self, model: ModelDefinition) {
        let available = model.is_available;
        self.health_checks.insert(model.name.clone(), available);
        self.models.push(model);
    }

    pub fn select(&self, task: &str, capability: ModelCapability) -> Option<&ModelDefinition> {
        let _ = task;
        let available: Vec<&ModelDefinition> = self
            .models
            .iter()
            .filter(|m| m.is_available && m.capabilities.contains(&capability))
            .collect();

        if available.is_empty() {
            return None;
        }

        match self.strategy {
            RoutingStrategy::LowestCost => {
                available
                    .into_iter()
                    .min_by(|a, b| {
                        a.cost_per_1k_input
                            .partial_cmp(&b.cost_per_1k_input)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
            }
            RoutingStrategy::FastestResponse => {
                available
                    .into_iter()
                    .min_by(|a, b| {
                        a.cost_per_1k_output
                            .partial_cmp(&b.cost_per_1k_output)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
            }
            RoutingStrategy::HighestCapability => {
                available
                    .into_iter()
                    .max_by(|a, b| a.context_window.cmp(&b.context_window))
            }
            RoutingStrategy::FallbackChain => available.into_iter().next(),
            RoutingStrategy::LoadBalanced => {
                let count = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
                let len = available.len();
                available.into_iter().nth(count % len)
            }
        }
    }

    pub fn record_outcome(&mut self, task: &str, model: &str, success: bool, latency_ms: u64) {
        let cost = self
            .models
            .iter()
            .find(|m| m.name == model)
            .map(|m| m.cost_per_1k_input * (latency_ms as f64 / 1000.0))
            .unwrap_or(0.0);

        self.routing_history.push(RoutingRecord {
            task: task.to_string(),
            selected_model: model.to_string(),
            strategy: self.strategy.clone(),
            success,
            latency_ms,
            cost,
        });
    }

    pub fn estimate_cost(
        &self,
        model: &str,
        input_tokens: usize,
        output_tokens: usize,
    ) -> f64 {
        self.models
            .iter()
            .find(|m| m.name == model)
            .map(|m| {
                let input_cost = (input_tokens as f64 / 1000.0) * m.cost_per_1k_input;
                let output_cost = (output_tokens as f64 / 1000.0) * m.cost_per_1k_output;
                input_cost + output_cost
            })
            .unwrap_or(0.0)
    }

    pub fn health_report(&self) -> Vec<(String, bool)> {
        self.models
            .iter()
            .map(|m| (m.name.clone(), m.is_available))
            .collect()
    }

    pub fn set_strategy(&mut self, strategy: RoutingStrategy) {
        self.strategy = strategy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cheap_model() -> ModelDefinition {
        ModelDefinition {
            name: "gpt-4o-mini".to_string(),
            provider: ModelProvider::OpenAI,
            capabilities: vec![ModelCapability::Chat, ModelCapability::CodeGeneration],
            context_window: 128_000,
            cost_per_1k_input: 0.15,
            cost_per_1k_output: 0.60,
            max_output_tokens: 16_384,
            is_available: true,
        }
    }

    fn expensive_model() -> ModelDefinition {
        ModelDefinition {
            name: "gpt-4o".to_string(),
            provider: ModelProvider::OpenAI,
            capabilities: vec![
                ModelCapability::Chat,
                ModelCapability::CodeGeneration,
                ModelCapability::Reasoning,
            ],
            context_window: 128_000,
            cost_per_1k_input: 2.50,
            cost_per_1k_output: 10.00,
            max_output_tokens: 16_384,
            is_available: true,
        }
    }

    fn claude_model() -> ModelDefinition {
        ModelDefinition {
            name: "claude-3.5-sonnet".to_string(),
            provider: ModelProvider::Anthropic,
            capabilities: vec![
                ModelCapability::Chat,
                ModelCapability::Reasoning,
                ModelCapability::CodeGeneration,
            ],
            context_window: 200_000,
            cost_per_1k_input: 3.00,
            cost_per_1k_output: 15.00,
            max_output_tokens: 8_192,
            is_available: true,
        }
    }

    fn unavailable_model() -> ModelDefinition {
        ModelDefinition {
            name: "gpt-4".to_string(),
            provider: ModelProvider::OpenAI,
            capabilities: vec![ModelCapability::Chat, ModelCapability::Reasoning],
            context_window: 8_192,
            cost_per_1k_input: 30.00,
            cost_per_1k_output: 60.00,
            max_output_tokens: 4_096,
            is_available: false,
        }
    }

    #[test]
    fn test_select_lowest_cost_model() {
        let models = vec![cheap_model(), expensive_model()];
        let router =
            ProviderRouter::new(models, RoutingStrategy::LowestCost);
        let selected = router.select("chat", ModelCapability::Chat);
        assert!(selected.is_some());
        assert_eq!(selected.expect("lowest-cost select should return gpt-4o-mini").name, "gpt-4o-mini");
    }

    #[test]
    fn test_select_by_capability_match() {
        let models = vec![
            ModelDefinition {
                name: "embedding-model".to_string(),
                provider: ModelProvider::OpenAI,
                capabilities: vec![ModelCapability::Embedding],
                context_window: 8_192,
                cost_per_1k_input: 0.02,
                cost_per_1k_output: 0.02,
                max_output_tokens: 8_192,
                is_available: true,
            },
            cheap_model(),
        ];
        let router =
            ProviderRouter::new(models, RoutingStrategy::LowestCost);
        let selected = router.select("embed", ModelCapability::Embedding);
        assert!(selected.is_some());
        assert_eq!(selected.expect("select for Embedding capability should return embedding-model").name, "embedding-model");
    }

    #[test]
    fn test_fallback_chain_primary_unavailable() {
        let models = vec![unavailable_model(), cheap_model()];
        let router =
            ProviderRouter::new(models, RoutingStrategy::FallbackChain);
        let selected = router.select("chat", ModelCapability::Chat);
        assert!(selected.is_some());
        assert_eq!(selected.expect("fallback chain should return gpt-4o-mini").name, "gpt-4o-mini");
    }

    #[test]
    fn test_register_new_model() {
        let mut router =
            ProviderRouter::new(vec![], RoutingStrategy::LowestCost);
        assert!(router.select("chat", ModelCapability::Chat).is_none());

        router.register_model(cheap_model());
        assert!(router.select("chat", ModelCapability::Chat).is_some());
    }

    #[test]
    fn test_cost_estimation_formula() {
        let models = vec![cheap_model()];
        let router =
            ProviderRouter::new(models, RoutingStrategy::LowestCost);

        let cost = router.estimate_cost("gpt-4o-mini", 1000, 500);
        let expected = 1.0 * 0.15 + 0.5 * 0.60;
        assert!((cost - expected).abs() < 0.001);
    }

    #[test]
    fn test_health_report_format() {
        let models = vec![
            ModelDefinition {
                name: "available-model".to_string(),
                provider: ModelProvider::OpenAI,
                capabilities: vec![ModelCapability::Chat],
                context_window: 4_096,
                cost_per_1k_input: 1.0,
                cost_per_1k_output: 2.0,
                max_output_tokens: 4_096,
                is_available: true,
            },
            ModelDefinition {
                name: "down-model".to_string(),
                provider: ModelProvider::Anthropic,
                capabilities: vec![ModelCapability::Chat],
                context_window: 4_096,
                cost_per_1k_input: 1.0,
                cost_per_1k_output: 2.0,
                max_output_tokens: 4_096,
                is_available: false,
            },
        ];
        let router =
            ProviderRouter::new(models, RoutingStrategy::LowestCost);
        let report = router.health_report();

        assert_eq!(report.len(), 2);
        let status_map: HashMap<&str, bool> =
            report.iter().map(|(n, s)| (n.as_str(), *s)).collect();
        assert_eq!(status_map.get("available-model"), Some(&true));
        assert_eq!(status_map.get("down-model"), Some(&false));
    }

    #[test]
    fn test_record_outcome_updates_metrics() {
        let models = vec![cheap_model()];
        let mut router =
            ProviderRouter::new(models, RoutingStrategy::LowestCost);

        router.record_outcome("test-task", "gpt-4o-mini", true, 100);
        assert_eq!(router.routing_history.len(), 1);

        let record = &router.routing_history[0];
        assert_eq!(record.task, "test-task");
        assert!(record.success);
        assert_eq!(record.latency_ms, 100);
        assert_eq!(record.selected_model, "gpt-4o-mini");
    }

    #[test]
    fn test_switch_routing_strategy() {
        let models = vec![
            ModelDefinition {
                name: "small-model".to_string(),
                provider: ModelProvider::Local,
                capabilities: vec![ModelCapability::Chat],
                context_window: 4_096,
                cost_per_1k_input: 0.10,
                cost_per_1k_output: 0.20,
                max_output_tokens: 2_048,
                is_available: true,
            },
            ModelDefinition {
                name: "large-model".to_string(),
                provider: ModelProvider::OpenAI,
                capabilities: vec![ModelCapability::Chat],
                context_window: 128_000,
                cost_per_1k_input: 2.50,
                cost_per_1k_output: 10.00,
                max_output_tokens: 16_384,
                is_available: true,
            },
        ];
        let mut router =
            ProviderRouter::new(models, RoutingStrategy::LowestCost);

        let selected_lowest = router.select("chat", ModelCapability::Chat);
        assert_eq!(selected_lowest.expect("lowest-cost should pick small-model").name, "small-model");

        router.set_strategy(RoutingStrategy::HighestCapability);
        let selected_highest = router.select("chat", ModelCapability::Chat);
        assert_eq!(selected_highest.expect("highest-capability should pick large-model").name, "large-model");
    }

    #[test]
    fn test_no_matching_model_returns_none() {
        let models = vec![cheap_model()]; // only Chat + CodeGeneration
        let router =
            ProviderRouter::new(models, RoutingStrategy::LowestCost);

        let selected = router.select("image-gen", ModelCapability::ImageGeneration);
        assert!(selected.is_none());
    }

    #[test]
    fn test_load_balanced_distribution() {
        let models = vec![
            ModelDefinition {
                name: "model-a".to_string(),
                provider: ModelProvider::OpenAI,
                capabilities: vec![ModelCapability::Chat],
                context_window: 4_096,
                cost_per_1k_input: 1.0,
                cost_per_1k_output: 2.0,
                max_output_tokens: 4_096,
                is_available: true,
            },
            ModelDefinition {
                name: "model-b".to_string(),
                provider: ModelProvider::Anthropic,
                capabilities: vec![ModelCapability::Chat],
                context_window: 4_096,
                cost_per_1k_input: 1.0,
                cost_per_1k_output: 2.0,
                max_output_tokens: 4_096,
                is_available: true,
            },
        ];
        let router =
            ProviderRouter::new(models, RoutingStrategy::LoadBalanced);

        let first = router.select("chat", ModelCapability::Chat);
        let second = router.select("chat", ModelCapability::Chat);
        assert!(first.is_some());
        assert!(second.is_some());
        assert_ne!(first.expect("first load-balanced select should return Some").name, second.expect("second load-balanced select should return Some").name);
    }

    #[test]
    fn test_fastest_response_selection() {
        let models = vec![expensive_model(), cheap_model()];
        let router =
            ProviderRouter::new(models, RoutingStrategy::FastestResponse);
        let selected = router.select("chat", ModelCapability::Chat);
        assert!(selected.is_some());
        assert_eq!(selected.expect("fastest-response should pick gpt-4o-mini").name, "gpt-4o-mini");
    }

    #[test]
    fn test_multiple_providers_same_capability() {
        let models = vec![cheap_model(), claude_model()];
        let router =
            ProviderRouter::new(models, RoutingStrategy::LowestCost);

        let selected = router.select("reason", ModelCapability::Reasoning);
        assert!(selected.is_some());
        assert_eq!(selected.expect("lowest-cost select for Reasoning should return claude-3.5-sonnet").name, "claude-3.5-sonnet");
    }

    #[test]
    fn test_estimate_cost_no_match() {
        let models = vec![cheap_model()];
        let router =
            ProviderRouter::new(models, RoutingStrategy::LowestCost);
        let cost = router.estimate_cost("nonexistent", 100, 100);
        assert!((cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_record_outcome_zero_latency() {
        let models = vec![cheap_model()];
        let mut router =
            ProviderRouter::new(models, RoutingStrategy::LowestCost);
        router.record_outcome("fast", "gpt-4o-mini", true, 0);
        assert_eq!(router.routing_history.len(), 1);
        assert_eq!(router.routing_history[0].latency_ms, 0);
    }

    #[test]
    fn test_strategy_clone_and_equality() {
        let a = RoutingStrategy::LowestCost;
        let b = RoutingStrategy::LowestCost;
        let c = RoutingStrategy::FallbackChain;
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_model_provider_variants() {
        let providers = vec![
            ModelProvider::OpenAI,
            ModelProvider::Anthropic,
            ModelProvider::Google,
            ModelProvider::Azure,
            ModelProvider::Local,
            ModelProvider::Custom("ollama".to_string()),
        ];
        assert_eq!(providers.len(), 6);
        assert_ne!(ModelProvider::OpenAI, ModelProvider::Anthropic);
        assert_eq!(
            ModelProvider::Custom("ollama".to_string()),
            ModelProvider::Custom("ollama".to_string())
        );
    }
}
