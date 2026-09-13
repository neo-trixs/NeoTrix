//! 资源路由器 (ResourceRouter)
//! 
//! 根据任务类型和资源能力，智能分配任务到最合适的资源
//! 
//! 参考: 成本感知路由 (Cost-Aware Routing) — 不是所有任务都需要最强模型

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 资源路由器
pub struct ResourceRouter {
    /// 资源池
    pub resource_pool: Vec<_ResourceEntry>,
    /// 路由历史
    pub routing_history: Vec<RoutingRecord>,
    /// 路由配置
    pub config: RouterConfig,
    /// 资源使用统计
    pub usage_stats: HashMap<String, ResourceUsage>,
}

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// 最大历史记录
    pub max_history: usize,
    /// 最大重试次数
    pub max_retries: u32,
    /// 负载均衡策略
    pub load_balance_strategy: LoadBalanceStrategy,
    /// 成本优化策略
    pub cost_optimization: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            max_retries: 3,
            load_balance_strategy: LoadBalanceStrategy::RoundRobin,
            cost_optimization: true,
        }
    }
}

/// 负载均衡策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalanceStrategy {
    /// 轮询
    RoundRobin,
    /// 最少连接
    LeastConnections,
    /// 最低成本
    LowestCost,
    /// 最高质量
    HighestQuality,
    /// 加权轮询
    WeightedRoundRobin,
}

/// 资源条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ResourceEntry {
    /// 资源ID
    pub id: String,
    /// 资源名称
    pub name: String,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 能力列表
    pub capabilities: Vec<_TaskCapability>,
    /// 成本模型
    pub cost_model: _CostModel,
    /// 质量评分
    pub quality_score: f64,
    /// 当前负载
    pub current_load: f64,
    /// 最大负载
    pub max_load: f64,
    /// 可用性
    pub availability: f64,
    /// 优先级权重
    pub weight: f64,
}

/// 资源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// LLM 推理
    LLM,
    /// 代码生成
    CodeGeneration,
    /// 知识检索
    KnowledgeRetrieval,
    /// 论文搜索
    PaperSearch,
    /// 模型仓库
    ModelRepository,
    /// 自定义
    Custom(String),
}

/// 任务能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _TaskCapability {
    /// 任务类型
    pub task_type: TaskType,
    /// 能力等级 (0.0 - 1.0)
    pub capability_level: f64,
    /// 延迟 (毫秒)
    pub latency_ms: u64,
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// 简单问答
    SimpleQA,
    /// 复杂推理
    ComplexReasoning,
    /// 代码生成
    CodeGeneration,
    /// 创意写作
    CreativeWriting,
    /// 知识检索
    KnowledgeRetrieval,
    /// 数据分析
    DataAnalysis,
    /// 自定义任务
    Custom(String),
}

/// 成本模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CostModel {
    /// 每1K token 成本
    pub cost_per_1k_tokens: f64,
    /// 每次请求基础成本
    pub base_cost: f64,
    /// 免费额度
    pub free_quota: u32,
    /// 当前使用量
    pub current_usage: u32,
}

/// 路由请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequest {
    /// 请求ID
    pub id: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 任务描述
    pub description: String,
    /// 优先级
    pub priority: TaskPriority,
    /// 预算约束
    pub budget: Option<f64>,
    /// 质量要求
    pub quality_requirement: f64,
    /// 延迟要求 (毫秒)
    pub latency_requirement: Option<u64>,
}

/// 任务优先级
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// 路由结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    /// 结果ID
    pub id: String,
    /// 选择的资源
    pub selected_resource: _ResourceEntry,
    /// 路由分数
    pub routing_score: f64,
    /// 预估成本
    pub estimated_cost: f64,
    /// 预估延迟
    pub estimated_latency: u64,
    /// 路由原因
    pub routing_reason: String,
}

/// 路由记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRecord {
    /// 记录ID
    pub id: String,
    /// 周期
    pub cycle: u32,
    /// 请求
    pub request: RoutingRequest,
    /// 结果
    pub result: RoutingResult,
    /// 时间戳
    pub timestamp: String,
    /// 实际耗时 (毫秒)
    pub actual_duration_ms: Option<u64>,
    /// 是否成功
    pub success: Option<bool>,
}

/// 资源使用统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// 总请求数
    pub total_requests: u32,
    /// 成功请求数
    pub successful_requests: u32,
    /// 总成本
    pub total_cost: f64,
    /// 平均延迟
    pub avg_latency_ms: f64,
}

impl ResourceRouter {
    /// 创建新的资源路由器
    pub fn new(config: RouterConfig) -> Self {
        Self {
            resource_pool: Vec::new(),
            routing_history: Vec::new(),
            config,
            usage_stats: HashMap::new(),
        }
    }

    /// 添加资源
    pub(crate) fn _add_resource(&mut self, resource: _ResourceEntry) {
        self.resource_pool.push(resource);
    }

    /// 路由任务
    pub fn route(&mut self, cycle: u32, request: RoutingRequest) -> RoutingResult {
        // 计算每个资源的路由分数
        let mut scored_resources: Vec<(usize, f64)> = self.resource_pool.iter()
            .enumerate()
            .filter(|(_, r)| r.availability > 0.5 && r.current_load < r.max_load)
            .map(|(i, r)| {
                let score = self.calculate_routing_score(r, &request);
                (i, score)
            })
            .collect();

        if scored_resources.is_empty() {
            return RoutingResult {
                id: format!("route_{}", uuid::Uuid::new_v4()),
                selected_resource: _ResourceEntry {
                    id: "default".to_string(),
                    name: "Default Resource".to_string(),
                    resource_type: ResourceType::LLM,
                    capabilities: vec![],
                    cost_model: _CostModel {
                        cost_per_1k_tokens: 0.01,
                        base_cost: 0.001,
                        free_quota: 1000,
                        current_usage: 0,
                    },
                    quality_score: 0.7,
                    current_load: 0.0,
                    max_load: 1.0,
                    availability: 1.0,
                    weight: 1.0,
                },
                routing_score: 0.5,
                estimated_cost: 0.001,
                estimated_latency: 1000,
                routing_reason: "No available resources".to_string(),
            };
        }

        scored_resources.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let (idx, score) = scored_resources[0];
        let selected = self.resource_pool[idx].clone();

        let result = RoutingResult {
            id: format!("route_{}", uuid::Uuid::new_v4()),
            selected_resource: selected.clone(),
            routing_score: score,
            estimated_cost: selected.cost_model.base_cost,
            estimated_latency: selected.capabilities.iter()
                .find(|c| format!("{:?}", c.task_type) == format!("{:?}", request.task_type))
                .map(|c| c.latency_ms)
                .unwrap_or(1000),
            routing_reason: format!("Best score: {:.4}", score),
        };

        let record = RoutingRecord {
            id: format!("routing_{}", uuid::Uuid::new_v4()),
            cycle,
            request: request.clone(),
            result: result.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            actual_duration_ms: None,
            success: None,
        };

        self.routing_history.push(record);
        self.trim_history();

        let usage = self.usage_stats.entry(selected.id.clone()).or_insert_with(ResourceUsage::default);
        usage.total_requests += 1;

        result
    }

    /// 计算路由分数
    fn calculate_routing_score(&self, resource: &_ResourceEntry, request: &RoutingRequest) -> f64 {
        let mut score = 0.0;

        // 质量分数 (权重: 0.3)
        score += 0.3 * resource.quality_score;

        // 能力匹配 (权重: 0.3)
        if let Some(capability) = resource.capabilities.iter()
            .find(|c| format!("{:?}", c.task_type) == format!("{:?}", request.task_type)) {
            score += 0.3 * capability.capability_level;
        }

        // 成本分数 (权重: 0.2)
        let cost_score = 1.0 - (resource.cost_model.cost_per_1k_tokens / 0.1).min(1.0);
        score += 0.2 * cost_score;

        // 负载分数 (权重: 0.1)
        let load_score = 1.0 - (resource.current_load / resource.max_load);
        score += 0.1 * load_score;

        // 可用性分数 (权重: 0.1)
        score += 0.1 * resource.availability;

        score
    }

    /// 获取路由统计
    pub fn stats(&self) -> RouterStats {
        let total_routings = self.routing_history.len();
        let successful_routings = self.routing_history.iter()
            .filter(|r| r.success.unwrap_or(false))
            .count();

        let avg_cost = if total_routings > 0 {
            self.routing_history.iter()
                .map(|r| r.result.estimated_cost)
                .sum::<f64>() / total_routings as f64
        } else {
            0.0
        };

        let avg_latency = if total_routings > 0 {
            self.routing_history.iter()
                .map(|r| r.result.estimated_latency as f64)
                .sum::<f64>() / total_routings as f64
        } else {
            0.0
        };

        RouterStats {
            total_routings,
            successful_routings,
            success_rate: if total_routings > 0 {
                successful_routings as f64 / total_routings as f64
            } else {
                0.0
            },
            avg_cost,
            avg_latency_ms: avg_latency,
            available_resources: self.resource_pool.iter()
                .filter(|r| r.availability > 0.5)
                .count(),
        }
    }

    /// 裁剪历史
    fn trim_history(&mut self) {
        while self.routing_history.len() > self.config.max_history {
            self.routing_history.remove(0);
        }
    }
}

/// 路由统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterStats {
    pub total_routings: usize,
    pub successful_routings: usize,
    pub success_rate: f64,
    pub avg_cost: f64,
    pub avg_latency_ms: f64,
    pub available_resources: usize,
}

impl std::fmt::Display for RouterStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        ResourceRouter 统计")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "总路由次数:      {}", self.total_routings)?;
        writeln!(f, "成功次数:        {}", self.successful_routings)?;
        writeln!(f, "成功率:          {:.2}%", self.success_rate * 100.0)?;
        writeln!(f, "平均成本:        ${:.6}", self.avg_cost)?;
        writeln!(f, "平均延迟:        {:.2}ms", self.avg_latency_ms)?;
        writeln!(f, "可用资源:        {}", self.available_resources)?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = ResourceRouter::new(RouterConfig::default());
        assert_eq!(router.resource_pool.len(), 0);
        assert_eq!(router.routing_history.len(), 0);
    }

    #[test]
    fn test_route_task() {
        let mut router = ResourceRouter::new(RouterConfig::default());
        let request = RoutingRequest {
            id: "req_1".to_string(),
            task_type: TaskType::SimpleQA,
            description: "测试任务".to_string(),
            priority: TaskPriority::Medium,
            budget: None,
            quality_requirement: 0.7,
            latency_requirement: None,
        };
        
        let result = router.route(0, request);
        assert_eq!(router.routing_history.len(), 1);
        assert!(result.routing_score > 0.0);
    }
}
