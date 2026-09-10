#![forbid(unsafe_code)]

//! 成本感知模型路由器 (Cost-Aware Model Router)
//!
//! 将任务路由到最便宜的有能力模型，支持复杂度估计、质量-成本权衡优化、
//! 提供者故障回退链
//!
//! 参考: Spotify Portal Shunt (~90% token 节省) + Cost-Aware Routing 公理

use serde::{Deserialize, Serialize};

/// 模型能力等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CapabilityTier {
    /// 轻量级 — 简单 I/O 任务
    Lightweight,
    /// 中等 — 一般推理
    Medium,
    /// 强力 — 复杂推理/代码
    Strong,
    /// 旗舰 — 最高质量
    Flagship,
}

impl CapabilityTier {
    /// 基础成本因子
    pub fn cost_factor(&self) -> f64 {
        match self {
            CapabilityTier::Lightweight => 0.01,
            CapabilityTier::Medium => 0.05,
            CapabilityTier::Strong => 0.20,
            CapabilityTier::Flagship => 1.00,
        }
    }
}

/// 任务复杂度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaskComplexity {
    /// 简单 — 单步、模板化
    Simple,
    /// 中等 — 多步、需要理解
    Moderate,
    /// 复杂 — 推理链、代码生成
    Complex,
    /// 旗舰 — 创新、多域综合
    Expert,
}

impl TaskComplexity {
    /// 最低能力需求
    pub fn min_capability(&self) -> CapabilityTier {
        match self {
            TaskComplexity::Simple => CapabilityTier::Lightweight,
            TaskComplexity::Moderate => CapabilityTier::Medium,
            TaskComplexity::Complex => CapabilityTier::Strong,
            TaskComplexity::Expert => CapabilityTier::Flagship,
        }
    }

    /// 质量权重
    pub fn quality_weight(&self) -> f64 {
        match self {
            TaskComplexity::Simple => 0.2,
            TaskComplexity::Moderate => 0.5,
            TaskComplexity::Complex => 0.8,
            TaskComplexity::Expert => 1.0,
        }
    }
}

/// 模型提供者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    /// 提供者 ID
    pub id: String,
    /// 提供者名称
    pub name: String,
    /// 模型名称
    pub model: String,
    /// 能力等级
    pub capability: CapabilityTier,
    /// 每 1K token 成本
    pub cost_per_1k_tokens: f64,
    /// 每次请求基础成本
    pub base_cost: f64,
    /// 平均延迟 (毫秒)
    pub avg_latency_ms: u64,
    /// 质量分数 (0.0 - 1.0)
    pub quality_score: f64,
    /// 可用性 (0.0 - 1.0)
    pub availability: f64,
    /// 当前使用量
    pub total_calls: u64,
    /// 最大 QPS
    pub max_qps: f64,
    /// 是否可用
    pub enabled: bool,
}

/// 路由请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRouteRequest {
    /// 请求 ID
    pub id: String,
    /// 任务描述
    pub task_description: String,
    /// 估计的 token 数
    pub estimated_tokens: u32,
    /// 质量要求 (0.0 - 1.0)
    pub quality_requirement: f64,
    /// 预算约束 (美元)
    pub budget: Option<f64>,
    /// 延迟要求 (毫秒)
    pub latency_requirement: Option<u64>,
    /// 是否允许降级
    pub allow_degradation: bool,
}

/// 路由结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRouteResult {
    /// 结果 ID
    pub id: String,
    /// 选择的提供者
    pub selected_provider: ModelProvider,
    /// 路由分数
    pub route_score: f64,
    /// 预估成本
    pub estimated_cost: f64,
    /// 预估延迟
    pub estimated_latency_ms: u64,
    /// 任务复杂度
    pub complexity: TaskComplexity,
    /// 路由原因
    pub reason: String,
    /// 回退链 (如果主选失败)
    pub fallback_chain: Vec<ModelProvider>,
    /// 估计的 token 数
    pub estimated_tokens: u32,
}

/// 路由记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRecord {
    /// 记录 ID
    pub id: String,
    /// 请求
    pub request: CostRouteRequest,
    /// 结果
    pub result: CostRouteResult,
    /// 实际成本
    pub actual_cost: Option<f64>,
    /// 实际延迟
    pub actual_latency_ms: Option<u64>,
    /// 是否成功
    pub success: Option<bool>,
    /// 时间戳
    pub timestamp: String,
}

/// 成本路由器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRouterConfig {
    /// 最大历史记录
    pub max_history: usize,
    /// 成本权重 (vs 质量)
    pub cost_weight: f64,
    /// 质量权重 (vs 成本)
    pub quality_weight: f64,
    /// 延迟权重
    pub latency_weight: f64,
    /// 最小质量阈值
    pub min_quality_threshold: f64,
    /// 最大成本阈值
    pub max_cost_threshold: f64,
    /// 启用复杂度估计
    pub enable_complexity_estimation: bool,
}

impl Default for CostRouterConfig {
    fn default() -> Self {
        Self {
            max_history: 10_000,
            cost_weight: 0.4,
            quality_weight: 0.4,
            latency_weight: 0.2,
            min_quality_threshold: 0.6,
            max_cost_threshold: 1.0,
            enable_complexity_estimation: true,
        }
    }
}

/// 成本感知模型路由器
pub struct CostRouter {
    /// 提供者池
    providers: Vec<ModelProvider>,
    /// 路由历史
    history: Vec<RouteRecord>,
    /// 配置
    config: CostRouterConfig,
    /// 统计
    stats: CostRouterStats,
}

/// 路由器统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostRouterStats {
    /// 总路由次数
    pub total_routes: u64,
    /// 成功路由次数
    pub successful_routes: u64,
    /// 总成本
    pub total_cost: f64,
    /// 平均成本
    pub avg_cost: f64,
    /// 节省成本 (vs 旗舰模型)
    pub cost_savings: f64,
    /// 平均延迟
    pub avg_latency_ms: f64,
    /// 回退次数
    pub fallback_count: u64,
}

impl CostRouter {
    /// 创建新的成本路由器
    pub fn new(config: CostRouterConfig) -> Self {
        Self {
            providers: Vec::new(),
            history: Vec::new(),
            config,
            stats: CostRouterStats::default(),
        }
    }

    /// 添加模型提供者
    pub fn add_provider(&mut self, provider: ModelProvider) {
        self.providers.push(provider);
    }

    /// 估计任务复杂度
    pub fn estimate_complexity(&self, request: &CostRouteRequest) -> TaskComplexity {
        if !self.config.enable_complexity_estimation {
            return TaskComplexity::Moderate;
        }

        let desc_lower = request.task_description.to_lowercase();
        let token_estimate = request.estimated_tokens as f64;

        // 基于关键词的复杂度估计
        let keyword_score: f64 = if desc_lower.contains("简单") || desc_lower.contains("simple") {
            0.0
        } else if desc_lower.contains("复杂") || desc_lower.contains("complex")
            || desc_lower.contains("推理") || desc_lower.contains("reasoning")
        {
            0.8
        } else if desc_lower.contains("创新") || desc_lower.contains("creative")
            || desc_lower.contains("综合") || desc_lower.contains("comprehensive")
        {
            1.0
        } else {
            0.4
        };

        // 基于 token 数的复杂度
        let token_score: f64 = if token_estimate < 100.0 {
            0.0
        } else if token_estimate < 500.0 {
            0.3
        } else if token_estimate < 2000.0 {
            0.6
        } else {
            0.9
        };

        // 综合评分
        let score: f64 = (keyword_score * 0.6 + token_score * 0.4).min(1.0);

        match score {
            s if s < 0.25 => TaskComplexity::Simple,
            s if s < 0.50 => TaskComplexity::Moderate,
            s if s < 0.75 => TaskComplexity::Complex,
            _ => TaskComplexity::Expert,
        }
    }

    /// 计算提供者路由分数
    fn calculate_score(
        &self,
        provider: &ModelProvider,
        complexity: &TaskComplexity,
        request: &CostRouteRequest,
    ) -> f64 {
        let min_cap = complexity.min_capability();

        // 能力匹配检查
        if provider.capability < min_cap {
            return -1.0; // 不满足最低要求
        }

        // 成本分数 (越低成本越好)
        let estimated_cost = provider.base_cost
            + (request.estimated_tokens as f64 / 1000.0) * provider.cost_per_1k_tokens;
        let max_cost = self.config.max_cost_threshold;
        let cost_score = if estimated_cost > max_cost {
            0.0 // 超出预算
        } else {
            1.0 - (estimated_cost / max_cost)
        };

        // 质量分数
        let quality_score = provider.quality_score;

        // 延迟分数
        let latency_score = if let Some(max_latency) = request.latency_requirement {
            if provider.avg_latency_ms <= max_latency {
                1.0 - (provider.avg_latency_ms as f64 / max_latency as f64)
            } else {
                0.0
            }
        } else {
            0.5 // 无延迟要求
        };

        // 加权综合
        let score = cost_score * self.config.cost_weight
            + quality_score * self.config.quality_weight
            + latency_score * self.config.latency_weight;

        score
    }

    /// 路由请求
    pub fn route(&mut self, request: CostRouteRequest) -> CostRouteResult {
        let complexity = self.estimate_complexity(&request);

        // 过滤可用提供者
        let mut scored_providers: Vec<(&ModelProvider, f64)> = self.providers
            .iter()
            .filter(|p| p.enabled && p.availability > 0.5)
            .map(|p| {
                let score = self.calculate_score(p, &complexity, &request);
                (p, score)
            })
            .filter(|(_, score)| *score >= 0.0)
            .collect();

        // 按分数排序 (最高分优先)
        scored_providers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 构建回退链
        let fallback_chain: Vec<ModelProvider> = scored_providers.iter()
            .skip(1)
            .take(3)
            .map(|(p, _)| (*p).clone())
            .collect();

        // 选择最佳提供者 (clone early to avoid borrow conflicts)
        let (selected, score) = if let Some((provider, score)) = scored_providers.first() {
            ((*provider).clone(), *score)
        } else {
            // 无可用提供者，返回默认
            let default_provider = ModelProvider {
                id: "default".to_string(),
                name: "Default".to_string(),
                model: "default-model".to_string(),
                capability: CapabilityTier::Medium,
                cost_per_1k_tokens: 0.01,
                base_cost: 0.001,
                avg_latency_ms: 1000,
                quality_score: 0.7,
                availability: 1.0,
                total_calls: 0,
                max_qps: 100.0,
                enabled: true,
            };
            (default_provider, 0.5)
        };

        let estimated_cost = selected.base_cost
            + (request.estimated_tokens as f64 / 1000.0) * selected.cost_per_1k_tokens;

        let result = CostRouteResult {
            id: format!("route_{}", uuid::Uuid::new_v4()),
            selected_provider: selected.clone(),
            route_score: score,
            estimated_cost,
            estimated_latency_ms: selected.avg_latency_ms,
            complexity,
            reason: format!("Score: {:.4}, Capability: {:?}", score, selected.capability),
            fallback_chain,
            estimated_tokens: request.estimated_tokens,
        };

        // 记录
        let record = RouteRecord {
            id: format!("record_{}", uuid::Uuid::new_v4()),
            request,
            result: result.clone(),
            actual_cost: None,
            actual_latency_ms: None,
            success: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.history.push(record);
        self.trim_history();

        // 更新统计
        self.stats.total_routes += 1;
        self.stats.total_cost += estimated_cost;
        self.stats.avg_cost = self.stats.total_cost / self.stats.total_routes as f64;

        // 计算节省 (vs 旗舰)
        let flagship_cost = estimated_cost / selected.capability.cost_factor();
        self.stats.cost_savings += flagship_cost - estimated_cost;

        result
    }

    /// 执行带回退的路由
    pub fn route_with_fallback(
        &mut self,
        request: CostRouteRequest,
        max_attempts: usize,
    ) -> (CostRouteResult, Vec<String>) {
        let mut result = self.route(request);
        let mut attempts = vec![result.selected_provider.id.clone()];
        let mut remaining_fallbacks = result.fallback_chain.clone();

        for _ in 1..max_attempts {
            if !remaining_fallbacks.is_empty() {
                let fallback = remaining_fallbacks.remove(0);
                result.selected_provider = fallback.clone();
                result.estimated_cost = fallback.base_cost
                    + (result.estimated_tokens as f64 / 1000.0) * fallback.cost_per_1k_tokens;
                result.reason = format!("Fallback to: {}", fallback.name);
                attempts.push(fallback.id);
                self.stats.fallback_count += 1;
            } else {
                break;
            }
        }

        (result, attempts)
    }

    /// 获取统计
    pub fn stats(&self) -> &CostRouterStats {
        &self.stats
    }

    /// 获取所有提供者
    pub fn providers(&self) -> &[ModelProvider] {
        &self.providers
    }

    /// 获取提供者按成本排序
    pub fn providers_by_cost(&self) -> Vec<&ModelProvider> {
        let mut providers: Vec<&ModelProvider> = self.providers.iter().collect();
        providers.sort_by(|a, b| a.cost_per_1k_tokens.partial_cmp(&b.cost_per_1k_tokens).unwrap());
        providers
    }

    /// 裁剪历史
    fn trim_history(&mut self) {
        while self.history.len() > self.config.max_history {
            self.history.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_provider(
        id: &str,
        capability: CapabilityTier,
        cost: f64,
        quality: f64,
    ) -> ModelProvider {
        ModelProvider {
            id: id.to_string(),
            name: format!("Provider {}", id),
            model: format!("model-{}", id),
            capability,
            cost_per_1k_tokens: cost,
            base_cost: cost * 0.1,
            avg_latency_ms: 500,
            quality_score: quality,
            availability: 1.0,
            total_calls: 0,
            max_qps: 100.0,
            enabled: true,
        }
    }

    #[test]
    fn test_router_creation() {
        let router = CostRouter::new(CostRouterConfig::default());
        assert_eq!(router.providers().len(), 0);
    }

    #[test]
    fn test_complexity_estimation() {
        let router = CostRouter::new(CostRouterConfig::default());
        
        let simple = CostRouteRequest {
            id: "1".to_string(),
            task_description: "简单问答".to_string(),
            estimated_tokens: 50,
            quality_requirement: 0.5,
            budget: None,
            latency_requirement: None,
            allow_degradation: true,
        };
        assert_eq!(router.estimate_complexity(&simple), TaskComplexity::Simple);
        
        let complex = CostRouteRequest {
            id: "2".to_string(),
            task_description: "复杂推理任务".to_string(),
            estimated_tokens: 3000,
            quality_requirement: 0.9,
            budget: None,
            latency_requirement: None,
            allow_degradation: false,
        };
        assert_eq!(router.estimate_complexity(&complex), TaskComplexity::Expert);
    }

    #[test]
    fn test_route_simple_task() {
        let mut router = CostRouter::new(CostRouterConfig::default());
        router.add_provider(create_test_provider("cheap", CapabilityTier::Lightweight, 0.001, 0.6));
        router.add_provider(create_test_provider("expensive", CapabilityTier::Flagship, 0.1, 0.95));
        
        let request = CostRouteRequest {
            id: "1".to_string(),
            task_description: "简单问答".to_string(),
            estimated_tokens: 100,
            quality_requirement: 0.5,
            budget: None,
            latency_requirement: None,
            allow_degradation: true,
        };
        
        let result = router.route(request);
        // 应该选择便宜的模型
        assert_eq!(result.selected_provider.id, "cheap");
        assert!(result.estimated_cost < 0.01);
    }

    #[test]
    fn test_route_complex_task() {
        let mut router = CostRouter::new(CostRouterConfig::default());
        router.add_provider(create_test_provider("cheap", CapabilityTier::Lightweight, 0.001, 0.6));
        router.add_provider(create_test_provider("powerful", CapabilityTier::Strong, 0.05, 0.9));
        
        let request = CostRouteRequest {
            id: "1".to_string(),
            task_description: "复杂推理代码生成".to_string(),
            estimated_tokens: 2000,
            quality_requirement: 0.8,
            budget: None,
            latency_requirement: None,
            allow_degradation: false,
        };
        
        let result = router.route(request);
        // 应该选择强力模型
        assert_eq!(result.selected_provider.id, "powerful");
    }

    #[test]
    fn test_fallback_chain() {
        let mut router = CostRouter::new(CostRouterConfig::default());
        router.add_provider(create_test_provider("p1", CapabilityTier::Medium, 0.02, 0.7));
        router.add_provider(create_test_provider("p2", CapabilityTier::Medium, 0.03, 0.75));
        router.add_provider(create_test_provider("p3", CapabilityTier::Medium, 0.04, 0.8));
        
        let request = CostRouteRequest {
            id: "1".to_string(),
            task_description: "测试任务".to_string(),
            estimated_tokens: 500,
            quality_requirement: 0.5,
            budget: None,
            latency_requirement: None,
            allow_degradation: true,
        };
        
        let result = router.route(request);
        assert!(result.fallback_chain.len() >= 2);
    }
}
