//! Model Router — 智能模型选择
//!
//! 吸收 KB 经验:
//! - 任务类型路由
//! - 成本优化
//! - 延迟优化
//! - 负载均衡
//! - 故障转移

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
