//! Model Router — 智能模型选择
//!
//! 吸收 KB 经验:
//! - 任务类型路由
//! - 成本优化
//! - 延迟优化
//! - 负载均衡
//! - 故障转移
//!
//! P0-3 MoE Capability Routing: 路由到最便宜的胜任模型
//! - 任务难度感知：简单任务用廉价模型，复杂任务用强力模型
//! - 成本权重动态调整：根据任务难度调整 cost_weights 中 lambda 系数

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 模型路由器
pub struct ModelRouter {
    providers: HashMap<String, Provider>,
    models: HashMap<String, ModelInfo>,
    routing_table: RoutingTable,
    stats: RouterStats,
}

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTable {
    pub task_routes: HashMap<String, String>,
    pub fallback_chain: Vec<String>,
    pub cost_weights: CostWeights,
    pub latency_weights: LatencyWeights,
}

/// 成本权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostWeights {
    pub input_token_cost: f64,
    pub output_token_cost: f64,
    pub fixed_cost: f64,
}

/// 延迟权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyWeights {
    pub first_token_latency: f64,
    pub throughput: f64,
    pub p99_latency: f64,
}

/// 提供商
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub rate_limit: u32,
    pub cost_per_1k_input: f64,
    pub cost_per_1k_output: f64,
    pub status: ProviderStatus,
}

/// 提供商状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Active,
    Inactive,
    RateLimited,
    Error,
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub capabilities: Vec<String>,
    pub max_tokens: usize,
    pub cost_per_1k_input: f64,
    pub cost_per_1k_output: f64,
    pub avg_latency_ms: u64,
    pub quality_score: f64,
}

/// 路由结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    pub model_id: String,
    pub provider: String,
    pub estimated_cost: f64,
    pub estimated_latency: u64,
    pub confidence: f64,
    pub reason: String,
}

/// 路由统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterStats {
    pub total_requests: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
    pub avg_cost: f64,
    pub avg_latency: u64,
    pub provider_usage: HashMap<String, u64>,
}

impl ModelRouter {
    /// 创建新的模型路由器
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            models: HashMap::new(),
            routing_table: RoutingTable {
                task_routes: HashMap::new(),
                fallback_chain: vec!["openai".into(), "anthropic".into(), "local".into()],
                cost_weights: CostWeights {
                    input_token_cost: 1.0,
                    output_token_cost: 1.0,
                    fixed_cost: 0.0,
                },
                latency_weights: LatencyWeights {
                    first_token_latency: 1.0,
                    throughput: 0.5,
                    p99_latency: 0.3,
                },
            },
            stats: RouterStats {
                total_requests: 0,
                successful_routes: 0,
                failed_routes: 0,
                avg_cost: 0.0,
                avg_latency: 0,
                provider_usage: HashMap::new(),
            },
        }
    }

    /// 注册提供商
    pub fn register_provider(&mut self, provider: Provider) {
        self.providers.insert(provider.name.clone(), provider);
    }

    /// 注册模型
    pub fn register_model(&mut self, model: ModelInfo) {
        self.models.insert(model.id.clone(), model);
    }

    /// 路由请求
    pub fn route(&mut self, task_type: &str, requirements: &RoutingRequirements) -> Result<RoutingResult, String> {
        self.stats.total_requests += 1;

        // 查找适合任务类型的模型
        let candidates: Vec<&ModelInfo> = self.models.values()
            .filter(|m| {
                // 检查能力匹配
                requirements.required_capabilities.iter()
                    .all(|cap| m.capabilities.contains(cap))
            })
            .collect();

        if candidates.is_empty() {
            self.stats.failed_routes += 1;
            return Err("No suitable model found".into());
        }

        // 评分和排序
        let mut scored_candidates: Vec<(&ModelInfo, f64)> = candidates.iter()
            .map(|model| {
                let score = self.calculate_score(model, requirements);
                (*model, score)
            })
            .collect();

        scored_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if let Some((best_model, score)) = scored_candidates.first() {
            let result = RoutingResult {
                model_id: best_model.id.clone(),
                provider: best_model.provider.clone(),
                estimated_cost: self.estimate_cost(best_model, requirements),
                estimated_latency: best_model.avg_latency_ms,
                confidence: *score,
                reason: format!("Best match for task type: {}", task_type),
            };

            self.stats.successful_routes += 1;
            *self.stats.provider_usage.entry(best_model.provider.clone()).or_insert(0) += 1;

            Ok(result)
        } else {
            self.stats.failed_routes += 1;
            Err("No suitable model found after scoring".into())
        }
    }

    /// 计算模型分数
    fn calculate_score(&self, model: &ModelInfo, requirements: &RoutingRequirements) -> f64 {
        let mut score = 0.0;

        // 质量分数
        score += model.quality_score * 0.4;

        // 成本分数 (越低越好)
        let cost = self.estimate_cost(model, requirements);
        let max_cost = self.models.values().map(|m| self.estimate_cost(m, requirements)).fold(f64::INFINITY, f64::min);
        if max_cost > 0.0 {
            score += (1.0 - cost / max_cost) * 0.3;
        }

        // 延迟分数 (越低越好)
        let max_latency = self.models.values().map(|m| m.avg_latency_ms).max().unwrap_or(1) as f64;
        if max_latency > 0.0 {
            score += (1.0 - model.avg_latency_ms as f64 / max_latency) * 0.3;
        }

        score
    }

    /// 估算成本
    fn estimate_cost(&self, model: &ModelInfo, requirements: &RoutingRequirements) -> f64 {
        let input_tokens = requirements.estimated_input_tokens as f64;
        let output_tokens = requirements.estimated_output_tokens as f64;

        (input_tokens / 1000.0 * model.cost_per_1k_input) +
        (output_tokens / 1000.0 * model.cost_per_1k_output)
    }

    /// 获取统计信息
    pub fn stats(&self) -> &RouterStats {
        &self.stats
    }
}

/// 路由需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequirements {
    pub task_type: String,
    pub required_capabilities: Vec<String>,
    pub max_cost: Option<f64>,
    pub max_latency: Option<u64>,
    pub estimated_input_tokens: usize,
    pub estimated_output_tokens: usize,
}

/// Task difficulty level — drives cost-aware model selection.
///
/// Implements Axiom A1 (Cost-Aware Routing): not all tasks need the strongest
/// model. Simple tasks route to cheap models, complex tasks to powerful ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskDifficulty {
    /// Simple factual queries, format conversions, short answers.
    /// Target: cheapest capable model.
    Trivial,
    /// Standard Q&A, simple code generation, summarization.
    /// Target: mid-tier model with good cost/quality ratio.
    Easy,
    /// Multi-step reasoning, moderate code tasks, analysis.
    /// Target: balanced cost/quality.
    Medium,
    /// Complex reasoning, long-context tasks, architecture design.
    /// Target: high-quality model, cost secondary.
    Hard,
    /// Critical tasks: security audit, production debugging, novel research.
    /// Target: strongest model regardless of cost.
    Expert,
}

impl TaskDifficulty {
    /// Cost sensitivity factor: 0.0 = ignore cost, 1.0 = maximize savings.
    /// Higher difficulty → lower sensitivity (quality matters more).
    pub fn cost_sensitivity(&self) -> f64 {
        match self {
            Self::Trivial => 0.9,
            Self::Easy => 0.7,
            Self::Medium => 0.5,
            Self::Hard => 0.3,
            Self::Expert => 0.1,
        }
    }

    /// Maximum acceptable cost multiplier relative to cheapest option.
    pub fn max_cost_multiplier(&self) -> f64 {
        match self {
            Self::Trivial => 1.2,
            Self::Easy => 2.0,
            Self::Medium => 4.0,
            Self::Hard => 8.0,
            Self::Expert => f64::INFINITY,
        }
    }

    /// Minimum required quality score (0.0 - 1.0).
    pub fn min_quality(&self) -> f64 {
        match self {
            Self::Trivial => 0.3,
            Self::Easy => 0.5,
            Self::Medium => 0.7,
            Self::Hard => 0.85,
            Self::Expert => 0.95,
        }
    }

    /// Estimate difficulty from task type string and token counts.
    pub fn estimate(task_type: &str, input_tokens: usize, output_tokens: usize) -> Self {
        let lower = task_type.to_lowercase();
        let total = input_tokens + output_tokens;

        if lower.contains("security") || lower.contains("audit") || lower.contains("critical") {
            return Self::Expert;
        }
        if lower.contains("architecture") || lower.contains("research") || total > 50_000 {
            return Self::Hard;
        }
        if lower.contains("code") || lower.contains("reason") || lower.contains("analysis")
            || lower.contains("multi") || total > 10_000
        {
            return Self::Medium;
        }
        if lower.contains("simple") || lower.contains("qa") || lower.contains("fact")
            || total < 1_000
        {
            return Self::Trivial;
        }
        Self::Easy
    }
}

impl ModelRouter {
    /// Route request with task-difficulty-aware cost optimization.
    ///
    /// Filters models by minimum quality threshold, then scores with
    /// difficulty-adjusted cost sensitivity. Cheap models are preferred
    /// for trivial tasks; powerful models for expert tasks.
    pub fn difficulty_route(
        &mut self,
        _task_type: &str,
        requirements: &RoutingRequirements,
        difficulty: TaskDifficulty,
    ) -> Result<RoutingResult, String> {
        self.stats.total_requests += 1;

        let min_quality = difficulty.min_quality();
        let max_cost_mult = difficulty.max_cost_multiplier();
        let cost_sensitivity = difficulty.cost_sensitivity();

        // Find candidates meeting minimum quality
        let candidates: Vec<&ModelInfo> = self.models.values()
            .filter(|m| {
                m.quality_score >= min_quality
                    && requirements.required_capabilities.iter()
                        .all(|cap| m.capabilities.contains(cap))
            })
            .collect();

        if candidates.is_empty() {
            self.stats.failed_routes += 1;
            return Err(format!(
                "No model meets quality >= {:.2} for difficulty {:?}",
                min_quality, difficulty
            ));
        }

        // Find cheapest model for cost multiplier baseline
        let cheapest_cost = candidates.iter()
            .map(|m| self.estimate_cost(m, requirements))
            .fold(f64::INFINITY, f64::min);

        // Score with difficulty-adjusted cost sensitivity
        let mut scored: Vec<(&ModelInfo, f64)> = candidates.iter()
            .filter_map(|model| {
                let cost = self.estimate_cost(model, requirements);
                let cost_ratio = if cheapest_cost > 0.0 { cost / cheapest_cost } else { 1.0 };

                // Reject if cost exceeds difficulty budget
                if cost_ratio > max_cost_mult {
                    return None;
                }

                // Score = quality × (1 - cost_sensitivity × normalized_cost)
                let normalized_cost = cost_ratio.min(max_cost_mult) / max_cost_mult;
                let score = model.quality_score * (1.0 - cost_sensitivity * normalized_cost);

                Some((*model, score))
            })
            .collect();

        if scored.is_empty() {
            self.stats.failed_routes += 1;
            return Err(format!(
                "No model within cost budget (max {}× cheapest) for difficulty {:?}",
                max_cost_mult, difficulty
            ));
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if let Some((best_model, score)) = scored.first() {
            let result = RoutingResult {
                model_id: best_model.id.clone(),
                provider: best_model.provider.clone(),
                estimated_cost: self.estimate_cost(best_model, requirements),
                estimated_latency: best_model.avg_latency_ms,
                confidence: *score,
                reason: format!(
                    "Difficulty {:?}: quality>={:.2}, cost_sensitivity={:.1}, model={}",
                    difficulty, min_quality, cost_sensitivity, best_model.name
                ),
            };

            self.stats.successful_routes += 1;
            *self.stats.provider_usage.entry(best_model.provider.clone()).or_insert(0) += 1;

            Ok(result)
        } else {
            self.stats.failed_routes += 1;
            Err("No model found after difficulty filtering".into())
        }
    }

    /// Auto-estimate difficulty and route.
    pub fn auto_route(
        &mut self,
        task_type: &str,
        requirements: &RoutingRequirements,
    ) -> Result<RoutingResult, String> {
        let difficulty = TaskDifficulty::estimate(
            task_type,
            requirements.estimated_input_tokens,
            requirements.estimated_output_tokens,
        );
        self.difficulty_route(task_type, requirements, difficulty)
    }
}

#[cfg(test)]
mod difficulty_tests {
    use super::*;

    fn test_router() -> ModelRouter {
        let mut router = ModelRouter::new();
        router.register_model(ModelInfo {
            id: "cheap-v1".into(),
            name: "Cheap Model".into(),
            provider: "local".into(),
            capabilities: vec!["text".into()],
            max_tokens: 4096,
            cost_per_1k_input: 0.001,
            cost_per_1k_output: 0.002,
            avg_latency_ms: 50,
            quality_score: 0.6,
        });
        router.register_model(ModelInfo {
            id: "mid-v1".into(),
            name: "Mid Model".into(),
            provider: "openai".into(),
            capabilities: vec!["text".into()],
            max_tokens: 16384,
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.02,
            avg_latency_ms: 200,
            quality_score: 0.8,
        });
        router.register_model(ModelInfo {
            id: "premium-v1".into(),
            name: "Premium Model".into(),
            provider: "anthropic".into(),
            capabilities: vec!["text".into(), "code".into()],
            max_tokens: 100_000,
            cost_per_1k_input: 0.05,
            cost_per_1k_output: 0.10,
            avg_latency_ms: 500,
            quality_score: 0.95,
        });
        router
    }

    fn req() -> RoutingRequirements {
        RoutingRequirements {
            task_type: "qa".into(),
            required_capabilities: vec!["text".into()],
            max_cost: None,
            max_latency: None,
            estimated_input_tokens: 100,
            estimated_output_tokens: 50,
        }
    }

    #[test]
    fn test_trivial_uses_cheapest() {
        let mut router = test_router();
        let result = router.difficulty_route("qa", &req(), TaskDifficulty::Trivial).unwrap();
        assert_eq!(result.model_id, "cheap-v1");
    }

    #[test]
    fn test_expensive_task_uses_best() {
        let mut router = test_router();
        let result = router.difficulty_route("security audit", &req(), TaskDifficulty::Expert).unwrap();
        assert_eq!(result.model_id, "premium-v1");
    }

    #[test]
    fn test_medium_balances_cost_quality() {
        let mut router = test_router();
        let result = router.difficulty_route("code", &req(), TaskDifficulty::Medium).unwrap();
        // Should pick mid or premium based on quality threshold
        assert!(result.model_id == "mid-v1" || result.model_id == "premium-v1");
    }

    #[test]
    fn test_difficulty_estimate_trivial() {
        let d = TaskDifficulty::estimate("simple_qa", 50, 20);
        assert_eq!(d, TaskDifficulty::Trivial);
    }

    #[test]
    fn test_difficulty_estimate_expert() {
        let d = TaskDifficulty::estimate("security audit", 1000, 500);
        assert_eq!(d, TaskDifficulty::Expert);
    }

    #[test]
    fn test_difficulty_estimate_by_token_count() {
        let d = TaskDifficulty::estimate("general", 60_000, 10_000);
        assert_eq!(d, TaskDifficulty::Hard);
    }

    #[test]
    fn test_auto_route_picks_cheap_for_simple() {
        let mut router = test_router();
        let mut r = req();
        r.task_type = "simple_qa".into();
        let result = router.auto_route("simple_qa", &r).unwrap();
        assert_eq!(result.model_id, "cheap-v1");
    }

    #[test]
    fn test_cost_sensitivity_decreases_with_difficulty() {
        assert!(TaskDifficulty::Trivial.cost_sensitivity() > TaskDifficulty::Hard.cost_sensitivity());
        assert!(TaskDifficulty::Easy.cost_sensitivity() > TaskDifficulty::Expert.cost_sensitivity());
    }
}
