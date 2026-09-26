//! # Real-time Model Router
//!
//! Inspired by GPT-5.6's real-time router that decides which model to use
//! based on conversation type, complexity, tool needs, and explicit intent.
//!
//! ## Design Principles
//! - Cost-aware routing: Consider token cost when selecting models
//! - Complexity-based: Route simple queries to fast models, complex to powerful
//! - Fallback chains: Graceful degradation when primary model unavailable
//! - Learning router: Improve routing decisions based on user feedback

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model tier classification (GPT-5.6 pattern: Sol/Terra/Luna)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelTier {
    /// Fast, cheap, good for simple tasks (Luna/Terra)
    Fast,
    /// Balanced cost/quality for everyday work (Terra)
    Balanced,
    /// Most capable, expensive for complex tasks (Sol)
    Powerful,
    /// Specialized for specific domains (e.g., code, math)
    Specialized(String),
}

/// Model provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    pub id: String,
    pub name: String,
    pub tier: ModelTier,
    pub cost_per_1k_input: f64,
    pub cost_per_1k_output: f64,
    pub max_context: u32,
    pub max_output: u32,
    pub capabilities: Vec<String>,
    pub latency_ms: u64,
    pub reliability: f32, // 0.0 - 1.0
}

/// Query context for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    pub query: String,
    pub conversation_history: Vec<String>,
    pub user_preferences: UserPreferences,
    pub task_type: TaskType,
    pub complexity_hint: Option<f32>, // 0.0 - 1.0
    pub budget_constraint: Option<BudgetConstraint>,
}

/// User preferences for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_tier: Option<ModelTier>,
    pub max_cost_per_query: Option<f64>,
    pub latency_tolerance: LatencyTolerance,
    pub quality_priority: f32, // 0.0 = cost priority, 1.0 = quality priority
}

/// Latency tolerance levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LatencyTolerance {
    /// Must be fast (< 1s)
    Low,
    /// Can wait a few seconds (1-5s)
    Medium,
    /// Can wait for complex processing (5-30s)
    High,
    /// Can wait for deep reasoning (30s+)
    VeryHigh,
}

/// Task type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    /// Simple chat, quick answers
    Chat,
    /// Code generation, debugging
    Coding,
    /// Mathematical reasoning
    Math,
    /// Research, analysis
    Research,
    /// Creative writing
    Creative,
    /// Multi-step agent tasks
    Agent,
    /// Data processing
    DataProcessing,
    /// Image/video understanding
    Multimodal,
}

/// Budget constraint for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConstraint {
    pub max_total_cost: f64,
    pub spent_so_far: f64,
    pub remaining_budget: f64,
}

/// Routing decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub selected_model: ModelProvider,
    pub reason: String,
    pub estimated_cost: CostEstimate,
    pub estimated_latency: LatencyEstimate,
    pub confidence: f32,
    pub alternatives: Vec<ModelProvider>,
    pub routing_strategy: RoutingStrategy,
}

/// Cost estimate for the decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub input_cost: f64,
    pub output_cost: f64,
    pub total_cost: f64,
    pub budget_remaining_after: f64,
}

/// Latency estimate for the decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyEstimate {
    pub estimated_ms: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

/// Routing strategy used
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// Direct match to tier
    DirectMatch,
    /// Cost-optimized selection
    CostOptimized,
    /// Quality-optimized selection
    QualityOptimized,
    /// Fallback due to availability
    Fallback,
    /// User preference override
    UserPreference,
    /// Learning-based optimization
    LearningBased,
}

/// Real-time Model Router
pub struct RealTimeModelRouter {
    providers: Vec<ModelProvider>,
    routing_history: Vec<RoutingHistoryEntry>,
    cost_tracker: CostTracker,
}

/// Routing history entry for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingHistoryEntry {
    pub query_hash: u64,
    pub selected_model: String,
    pub actual_cost: f64,
    pub actual_latency_ms: u64,
    pub user_satisfaction: Option<f32>, // 0.0 - 1.0
    pub timestamp: String,
}

/// Cost tracker for budget management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTracker {
    pub total_spent: f64,
    pub cost_by_model: HashMap<String, f64>,
    pub cost_by_task_type: HashMap<String, f64>,
    pub daily_budget: Option<f64>,
    pub monthly_budget: Option<f64>,
}

impl RealTimeModelRouter {
    /// Create a new router with default providers
    pub fn new() -> Self {
        let providers = vec![
            // Fast tier (Luna/Terra equivalent)
            ModelProvider {
                id: "deepseek-r1:7b".into(),
                name: "DeepSeek R1 7B".into(),
                tier: ModelTier::Fast,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
                max_context: 32768,
                max_output: 8192,
                capabilities: vec!["chat".into(), "simple_qa".into()],
                latency_ms: 100,
                reliability: 0.95,
            },
            ModelProvider {
                id: "gpt-4o-mini".into(),
                name: "GPT-4o Mini".into(),
                tier: ModelTier::Fast,
                cost_per_1k_input: 0.00015,
                cost_per_1k_output: 0.0006,
                max_context: 128000,
                max_output: 16384,
                capabilities: vec!["chat".into(), "code".into(), "vision".into()],
                latency_ms: 200,
                reliability: 0.98,
            },
            // Balanced tier (Terra equivalent)
            ModelProvider {
                id: "deepseek-r1:14b".into(),
                name: "DeepSeek R1 14B".into(),
                tier: ModelTier::Balanced,
                cost_per_1k_input: 0.001,
                cost_per_1k_output: 0.002,
                max_context: 65536,
                max_output: 16384,
                capabilities: vec!["chat".into(), "code".into(), "reasoning".into()],
                latency_ms: 500,
                reliability: 0.93,
            },
            ModelProvider {
                id: "claude-sonnet-4-20250514".into(),
                name: "Claude Sonnet 4".into(),
                tier: ModelTier::Balanced,
                cost_per_1k_input: 0.003,
                cost_per_1k_output: 0.015,
                max_context: 200000,
                max_output: 64000,
                capabilities: vec![
                    "chat".into(),
                    "code".into(),
                    "reasoning".into(),
                    "agents".into(),
                ],
                latency_ms: 800,
                reliability: 0.97,
            },
            // Powerful tier (Sol equivalent)
            ModelProvider {
                id: "claude-opus-4-20250514".into(),
                name: "Claude Opus 4".into(),
                tier: ModelTier::Powerful,
                cost_per_1k_input: 0.015,
                cost_per_1k_output: 0.075,
                max_context: 200000,
                max_output: 64000,
                capabilities: vec![
                    "chat".into(),
                    "code".into(),
                    "reasoning".into(),
                    "agents".into(),
                    "research".into(),
                ],
                latency_ms: 2000,
                reliability: 0.96,
            },
            ModelProvider {
                id: "gpt-5.6-sol".into(),
                name: "GPT-5.6 Sol".into(),
                tier: ModelTier::Powerful,
                cost_per_1k_input: 0.01,
                cost_per_1k_output: 0.03,
                max_context: 128000,
                max_output: 32768,
                capabilities: vec![
                    "chat".into(),
                    "code".into(),
                    "reasoning".into(),
                    "agents".into(),
                    "research".into(),
                ],
                latency_ms: 1500,
                reliability: 0.99,
            },
            // Specialized tier
            ModelProvider {
                id: "codestral".into(),
                name: "Codestral".into(),
                tier: ModelTier::Specialized("code".into()),
                cost_per_1k_input: 0.001,
                cost_per_1k_output: 0.003,
                max_context: 32768,
                max_output: 8192,
                capabilities: vec!["code".into()],
                latency_ms: 300,
                reliability: 0.94,
            },
        ];

        Self {
            providers,
            routing_history: Vec::new(),
            cost_tracker: CostTracker {
                total_spent: 0.0,
                cost_by_model: HashMap::new(),
                cost_by_task_type: HashMap::new(),
                daily_budget: None,
                monthly_budget: None,
            },
        }
    }

    /// Route a query to the optimal model
    pub fn route_query(&self, context: &QueryContext) -> RoutingDecision {
        let complexity = self.analyze_complexity(context);
        let task_tier = self.task_to_tier(&context.task_type, complexity);
        let budget_ok = self.check_budget(context);

        // Find matching providers
        let mut candidates: Vec<&ModelProvider> = self
            .providers
            .iter()
            .filter(|p| self.provider_matches_tier(p, &task_tier))
            .filter(|p| self.provider_has_capabilities(p, &context.task_type))
            .filter(|p| budget_ok || p.cost_per_1k_input <= 0.001) // Free or cheap if budget tight
            .collect();

        // Sort by cost-effectiveness
        candidates.sort_by(|a, b| {
            let cost_a = a.cost_per_1k_input + a.cost_per_1k_output;
            let cost_b = b.cost_per_1k_input + b.cost_per_1k_output;
            cost_a
                .partial_cmp(&cost_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Select best candidate
        let selected = candidates.first().copied().unwrap_or(&self.providers[0]);

        // Build alternatives
        let alternatives: Vec<ModelProvider> = candidates
            .iter()
            .skip(1)
            .take(3)
            .map(|p| (*p).clone())
            .collect();

        // Calculate estimates
        let estimated_tokens = self.estimate_tokens(&context.query);
        let input_cost = (estimated_tokens as f64 / 1000.0) * selected.cost_per_1k_input;
        let output_cost = (estimated_tokens as f64 / 4.0 / 1000.0) * selected.cost_per_1k_output; // Assume 25% output ratio
        let total_cost = input_cost + output_cost;

        let budget_remaining = context
            .budget_constraint
            .as_ref()
            .map(|b| b.remaining_budget - total_cost)
            .unwrap_or(f64::MAX);

        RoutingDecision {
            selected_model: selected.clone(),
            reason: format!(
                "Routed {} task (complexity: {:.2}) to {} tier model",
                task_type_to_string(&context.task_type),
                complexity,
                task_tier_to_string(&task_tier)
            ),
            estimated_cost: CostEstimate {
                input_cost,
                output_cost,
                total_cost,
                budget_remaining_after: budget_remaining,
            },
            estimated_latency: LatencyEstimate {
                estimated_ms: selected.latency_ms,
                p50_ms: (selected.latency_ms as f64 * 0.8) as u64,
                p95_ms: (selected.latency_ms as f64 * 1.5) as u64,
                p99_ms: (selected.latency_ms as f64 * 2.0) as u64,
            },
            confidence: selected.reliability,
            alternatives,
            routing_strategy: RoutingStrategy::CostOptimized,
        }
    }

    /// Analyze query complexity (0.0 - 1.0)
    fn analyze_complexity(&self, context: &QueryContext) -> f32 {
        let mut complexity: f32 = 0.0;
        let query = context.query.to_lowercase();

        // Length-based complexity
        let word_count = query.split_whitespace().count();
        if word_count > 100 {
            complexity += 0.2;
        } else if word_count > 50 {
            complexity += 0.1;
        }

        // Code complexity
        if query.contains("```") || query.contains("fn ") || query.contains("impl ") {
            complexity += 0.25;
        }

        // Mathematical complexity
        if query.contains("prove") || query.contains("derivative") || query.contains("integral") {
            complexity += 0.3;
        }

        // Multi-step complexity
        if query.contains("step 1") || query.contains("first") && query.contains("then") {
            complexity += 0.15;
        }

        // Domain-specific complexity
        if query.contains("architecture")
            || query.contains("algorithm")
            || query.contains("optimize")
        {
            complexity += 0.2;
        }

        // Use complexity hint if provided — treat hint as a floor, not an average
        if let Some(hint) = context.complexity_hint {
            complexity = complexity.max(hint);
        }

        complexity.min(1.0)
    }

    /// Map task type to model tier
    fn task_to_tier(&self, task_type: &TaskType, complexity: f32) -> ModelTier {
        match task_type {
            TaskType::Chat => {
                if complexity < 0.3 {
                    ModelTier::Fast
                } else {
                    ModelTier::Balanced
                }
            }
            TaskType::Coding => {
                if complexity < 0.5 {
                    ModelTier::Specialized("code".into())
                } else {
                    ModelTier::Balanced
                }
            }
            TaskType::Math | TaskType::Research => {
                if complexity > 0.7 {
                    ModelTier::Powerful
                } else {
                    ModelTier::Balanced
                }
            }
            TaskType::Agent => ModelTier::Powerful,
            TaskType::Creative => ModelTier::Balanced,
            TaskType::DataProcessing => ModelTier::Fast,
            TaskType::Multimodal => ModelTier::Balanced,
        }
    }

    /// Check if provider matches tier
    fn provider_matches_tier(&self, provider: &ModelProvider, tier: &ModelTier) -> bool {
        match tier {
            ModelTier::Fast => matches!(provider.tier, ModelTier::Fast),
            ModelTier::Balanced => matches!(provider.tier, ModelTier::Balanced),
            ModelTier::Powerful => true, // All providers can be used for powerful tasks
            ModelTier::Specialized(domain) => {
                matches!(provider.tier, ModelTier::Specialized(ref d) if d == domain)
                    || provider.capabilities.contains(domain)
            }
        }
    }

    /// Check if provider has required capabilities
    fn provider_has_capabilities(&self, provider: &ModelProvider, task_type: &TaskType) -> bool {
        let required = match task_type {
            TaskType::Chat => vec!["chat"],
            TaskType::Coding => vec!["code"],
            TaskType::Math => vec!["reasoning"],
            TaskType::Research => vec!["reasoning", "research"],
            TaskType::Creative => vec!["chat"],
            TaskType::Agent => vec!["agents"],
            TaskType::DataProcessing => vec!["chat"],
            TaskType::Multimodal => vec!["vision"],
        };

        required
            .iter()
            .all(|cap| provider.capabilities.contains(&cap.to_string()))
    }

    /// Check budget constraints
    fn check_budget(&self, context: &QueryContext) -> bool {
        if let Some(budget) = &context.budget_constraint {
            budget.remaining_budget > 0.01 // At least 1 cent remaining
        } else {
            true // No budget constraint
        }
    }

    /// Estimate token count for query
    fn estimate_tokens(&self, query: &str) -> u32 {
        // Simple estimation: ~4 characters per token
        (query.len() / 4) as u32
    }

    /// Record routing decision for learning
    pub fn record_routing(&mut self, entry: RoutingHistoryEntry) {
        self.routing_history.push(entry);
        // Keep only last 1000 entries
        if self.routing_history.len() > 1000 {
            self.routing_history.remove(0);
        }
    }

    /// Get routing statistics
    pub fn get_stats(&self) -> RoutingStats {
        let total_routes = self.routing_history.len();
        let avg_cost = if total_routes > 0 {
            self.routing_history
                .iter()
                .map(|e| e.actual_cost)
                .sum::<f64>()
                / total_routes as f64
        } else {
            0.0
        };

        let avg_latency = if total_routes > 0 {
            self.routing_history
                .iter()
                .map(|e| e.actual_latency_ms)
                .sum::<u64>()
                / total_routes as u64
        } else {
            0
        };

        let model_usage: HashMap<String, usize> =
            self.routing_history
                .iter()
                .fold(HashMap::new(), |mut acc, e| {
                    *acc.entry(e.selected_model.clone()).or_insert(0) += 1;
                    acc
                });

        RoutingStats {
            total_routes,
            avg_cost,
            avg_latency_ms: avg_latency,
            model_usage,
            total_spent: self.cost_tracker.total_spent,
        }
    }
}

/// Routing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStats {
    pub total_routes: usize,
    pub avg_cost: f64,
    pub avg_latency_ms: u64,
    pub model_usage: HashMap<String, usize>,
    pub total_spent: f64,
}

/// Helper function to convert task type to string
fn task_type_to_string(task_type: &TaskType) -> &'static str {
    match task_type {
        TaskType::Chat => "chat",
        TaskType::Coding => "coding",
        TaskType::Math => "math",
        TaskType::Research => "research",
        TaskType::Creative => "creative",
        TaskType::Agent => "agent",
        TaskType::DataProcessing => "data_processing",
        TaskType::Multimodal => "multimodal",
    }
}

/// Helper function to convert model tier to string
fn task_tier_to_string(tier: &ModelTier) -> &'static str {
    match tier {
        ModelTier::Fast => "fast",
        ModelTier::Balanced => "balanced",
        ModelTier::Powerful => "powerful",
        ModelTier::Specialized(_) => "specialized",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_simple_chat() {
        let router = RealTimeModelRouter::new();
        let context = QueryContext {
            query: "Hello, how are you?".into(),
            conversation_history: vec![],
            user_preferences: UserPreferences {
                preferred_tier: None,
                max_cost_per_query: None,
                latency_tolerance: LatencyTolerance::Low,
                quality_priority: 0.5,
            },
            task_type: TaskType::Chat,
            complexity_hint: None,
            budget_constraint: None,
        };

        let decision = router.route_query(&context);
        assert_eq!(decision.selected_model.tier, ModelTier::Fast);
    }

    #[test]
    fn test_route_complex_coding() {
        let router = RealTimeModelRouter::new();
        let context = QueryContext {
            query: "Implement a concurrent hash map with lock-free reads and fine-grained locking for writes. Include proper memory ordering and handle ABA problem.".into(),
            conversation_history: vec![],
            user_preferences: UserPreferences {
                preferred_tier: None,
                max_cost_per_query: None,
                latency_tolerance: LatencyTolerance::High,
                quality_priority: 0.8,
            },
            task_type: TaskType::Coding,
            complexity_hint: Some(0.8),
            budget_constraint: None,
        };

        let decision = router.route_query(&context);
        assert!(matches!(
            decision.selected_model.tier,
            ModelTier::Balanced | ModelTier::Powerful
        ));
    }
}
