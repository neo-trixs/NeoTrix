//! # Model Gateway — 统一模型网关
//!
//! 所有模型调用经过此处，实现:
//! - 单一入口: `ModelGateway::route()`
//! - 成本门控: 超预算自动降级
//! - 降级链: 主模型失败自动切换
//! - Provider 池: 管理连接和复用
//!
//! 基于 `RealTimeModelRouter` 增强，不替换现有路由逻辑。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model_router::{
    BudgetConstraint, LatencyTolerance, ModelProvider, ModelTier, QueryContext,
    RealTimeModelRouter, RoutingDecision, RoutingStrategy, TaskType, UserPreferences,
};

// ─── Time helper ────────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ─── Gateway Request/Response ───────────────────────────────────────────────

/// 统一网关请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    /// 请求 ID
    pub id: String,
    /// 用户输入
    pub input: String,
    /// 任务类型 (可选, 由 classifier 自动推断)
    pub task_type: Option<TaskType>,
    /// 指定模型 (可选, 覆盖自动路由)
    pub preferred_model: Option<String>,
    /// 成本预算 (可选)
    pub budget: Option<f64>,
    /// 最大延迟 (ms, 可选)
    pub max_latency_ms: Option<u64>,
    /// 会话上下文
    pub context: Option<String>,
}

/// 统一网关响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayResponse {
    /// 请求 ID
    pub request_id: String,
    /// 实际使用的模型
    pub model_used: String,
    /// 模型输出
    pub output: String,
    /// 成本
    pub cost: f64,
    /// 延迟 (ms)
    pub latency_ms: u64,
    /// 是否降级
    pub degraded: bool,
    /// 降级原因
    pub degradation_reason: Option<String>,
    /// 路由决策
    pub routing: RoutingDecision,
}

/// 网关错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatewayError {
    /// 所有模型都不可用
    AllProvidersUnavailable,
    /// 成本超限
    BudgetExceeded { budget: f64, estimated: f64 },
    /// 延迟超限
    LatencyExceeded { max_ms: u64, estimated: u64 },
    /// Provider 错误
    ProviderError { provider: String, error: String },
    /// 无匹配路由
    NoMatchingRoute { task_type: String },
}

// ─── CostGate ───────────────────────────────────────────────────────────────

/// 成本门控 — 超预算自动降级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostGate {
    /// 每模型预算 (USD)
    pub model_budgets: HashMap<String, f64>,
    /// 已消耗
    pub consumed: HashMap<String, f64>,
    /// 全局月预算
    pub monthly_budget: Option<f64>,
    /// 本月已消耗
    pub monthly_consumed: f64,
}

impl Default for CostGate {
    fn default() -> Self {
        Self {
            model_budgets: HashMap::new(),
            consumed: HashMap::new(),
            monthly_budget: Some(100.0),
            monthly_consumed: 0.0,
        }
    }
}

impl CostGate {
    /// 判断是否允许调用指定模型
    pub fn allow(&self, model_id: &str, estimated_cost: f64) -> CostGateResult {
        // 检查月预算
        if let Some(monthly) = self.monthly_budget {
            if self.monthly_consumed + estimated_cost > monthly {
                return CostGateResult::Denied {
                    reason: format!(
                        "月预算超限: 已用 ${:.2} + 本次 ${:.2} > 预算 ${:.2}",
                        self.monthly_consumed, estimated_cost, monthly
                    ),
                };
            }
        }

        // 检查模型预算
        if let Some(budget) = self.model_budgets.get(model_id) {
            let used = self.consumed.get(model_id).copied().unwrap_or(0.0);
            if used + estimated_cost > *budget {
                return CostGateResult::Denied {
                    reason: format!(
                        "模型预算超限 {}: 已用 ${:.2} + 本次 ${:.2} > 预算 ${:.2}",
                        model_id, used, estimated_cost, budget
                    ),
                };
            }
        }

        CostGateResult::Allowed
    }

    /// 记录消耗
    pub fn record(&mut self, model_id: &str, cost: f64) {
        *self.consumed.entry(model_id.to_string()).or_insert(0.0) += cost;
        self.monthly_consumed += cost;
    }

    /// 重置月度消耗
    pub fn reset_monthly(&mut self) {
        self.monthly_consumed = 0.0;
        self.consumed.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostGateResult {
    Allowed,
    Denied { reason: String },
}

// ─── FallbackChain ──────────────────────────────────────────────────────────

/// 降级链 — 主模型失败时自动切换
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackChain {
    /// 按任务类型的降级序列
    pub chains: HashMap<String, Vec<String>>,
    /// 默认降级序列 (当任务类型无匹配时)
    pub default_chain: Vec<String>,
}

impl Default for FallbackChain {
    fn default() -> Self {
        let mut chains = HashMap::new();

        // 编码任务降级链
        chains.insert(
            "coding".to_string(),
            vec![
                "claude-code".to_string(),
                "codex".to_string(),
                "gpt-4o".to_string(),
                "ollama:local".to_string(),
            ],
        );

        // 推理任务降级链
        chains.insert(
            "reasoning".to_string(),
            vec![
                "claude-opus".to_string(),
                "gpt-4o".to_string(),
                "gemini-pro".to_string(),
                "ollama:local".to_string(),
            ],
        );

        // 简单任务降级链
        chains.insert(
            "simple".to_string(),
            vec![
                "gemini-flash".to_string(),
                "gpt-4o-mini".to_string(),
                "ollama:local".to_string(),
            ],
        );

        Self {
            chains,
            default_chain: vec![
                "gpt-4o".to_string(),
                "claude-sonnet".to_string(),
                "ollama:local".to_string(),
            ],
        }
    }
}

impl FallbackChain {
    /// 获取降级链
    pub fn get_chain(&self, task_type: &str) -> &[String] {
        self.chains
            .get(task_type)
            .map(|c| c.as_slice())
            .unwrap_or(&self.default_chain)
    }

    /// 获取下一个降级模型
    pub fn next(&self, current_model: &str, task_type: &str) -> Option<String> {
        let chain = self.get_chain(task_type);
        if let Some(pos) = chain.iter().position(|m| m == current_model) {
            chain.get(pos + 1).cloned()
        } else {
            chain.first().cloned()
        }
    }
}

// ─── ProviderPool ───────────────────────────────────────────────────────────

/// Provider 连接池 — 管理活跃连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPool {
    /// 活跃连接数
    pub active_connections: HashMap<String, u32>,
    /// 最大连接数
    pub max_connections: HashMap<String, u32>,
    /// Provider 可用性
    pub availability: HashMap<String, bool>,
}

impl Default for ProviderPool {
    fn default() -> Self {
        let mut max_connections = HashMap::new();
        max_connections.insert("claude-code".to_string(), 3);
        max_connections.insert("codex".to_string(), 2);
        max_connections.insert("gpt-4o".to_string(), 5);
        max_connections.insert("ollama:local".to_string(), 1);

        let mut availability = HashMap::new();
        availability.insert("claude-code".to_string(), true);
        availability.insert("codex".to_string(), true);
        availability.insert("gpt-4o".to_string(), true);
        availability.insert("ollama:local".to_string(), true);

        Self {
            active_connections: HashMap::new(),
            max_connections,
            availability,
        }
    }
}

impl ProviderPool {
    /// 检查 provider 是否可用
    pub fn is_available(&self, provider_id: &str) -> bool {
        *self.availability.get(provider_id).unwrap_or(&false)
    }

    /// 尝试获取连接
    pub fn acquire(&mut self, provider_id: &str) -> bool {
        if !self.is_available(provider_id) {
            return false;
        }

        let active = self.active_connections.get(provider_id).unwrap_or(&0);
        let max = self.max_connections.get(provider_id).unwrap_or(&1);

        if active < max {
            *self
                .active_connections
                .entry(provider_id.to_string())
                .or_insert(0) += 1;
            true
        } else {
            false
        }
    }

    /// 释放连接
    pub fn release(&mut self, provider_id: &str) {
        if let Some(active) = self.active_connections.get_mut(provider_id) {
            if *active > 0 {
                *active -= 1;
            }
        }
    }

    /// 获取池状态
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            total_active: self.active_connections.values().sum(),
            by_provider: self.active_connections.clone(),
            availability: self.availability.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_active: u32,
    pub by_provider: HashMap<String, u32>,
    pub availability: HashMap<String, bool>,
}

// ─── ModelGateway ───────────────────────────────────────────────────────────

/// 统一模型网关 — 所有模型调用经过此处
pub struct ModelGateway {
    /// 底层路由
    router: RealTimeModelRouter,
    /// 成本门控
    cost_gate: CostGate,
    /// 降级链
    fallback_chain: FallbackChain,
    /// Provider 池
    pool: ProviderPool,
    /// 路由日志
    route_log: Vec<RouteLogEntry>,
}

/// 路由日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteLogEntry {
    pub request_id: String,
    pub input_preview: String,
    pub selected_model: String,
    pub cost: f64,
    pub latency_ms: u64,
    pub degraded: bool,
    pub timestamp: u64,
}

impl ModelGateway {
    /// 创建新网关
    pub fn new() -> Self {
        Self {
            router: RealTimeModelRouter::new(),
            cost_gate: CostGate::default(),
            fallback_chain: FallbackChain::default(),
            pool: ProviderPool::default(),
            route_log: Vec::new(),
        }
    }

    /// 配置成本门控
    pub fn with_cost_gate(mut self, gate: CostGate) -> Self {
        self.cost_gate = gate;
        self
    }

    /// 配置降级链
    pub fn with_fallback_chain(mut self, chain: FallbackChain) -> Self {
        self.fallback_chain = chain;
        self
    }

    /// 统一路由入口
    pub async fn route(&mut self, request: GatewayRequest) -> Result<GatewayResponse, GatewayError> {
        let start = now_secs();
        let task_type_str = request
            .task_type
            .as_ref()
            .map(|t| format!("{:?}", t))
            .unwrap_or_else(|| "Chat".to_string());

        // 1. 构建查询上下文
        let context = QueryContext {
            query: request.input.clone(),
            conversation_history: Vec::new(),
            user_preferences: UserPreferences {
                preferred_tier: None,
                max_cost_per_query: request.budget,
                latency_tolerance: LatencyTolerance::Medium,
                quality_priority: 0.5,
            },
            task_type: request.task_type.unwrap_or(TaskType::Chat),
            complexity_hint: None,
            budget_constraint: request.budget.map(|b| BudgetConstraint {
                max_total_cost: b,
                spent_so_far: 0.0,
                remaining_budget: b,
            }),
        };

        // 2. 获取路由决策
        let mut decision = self.router.route_query(&context);

        // 3. 成本门控检查
        if let Some(_budget) = request.budget {
            let estimated = decision.estimated_cost.total_cost;
            if let CostGateResult::Denied { reason } =
                self.cost_gate.allow(&decision.selected_model.id, estimated)
            {
                // 降级到更便宜的模型
                if let Some(cheaper) =
                    self.fallback_chain.next(&decision.selected_model.id, &task_type_str)
                {
                    decision.selected_model = ModelProvider {
                        id: cheaper.clone(),
                        name: cheaper.clone(),
                        tier: ModelTier::Fast,
                        cost_per_1k_input: 0.001,
                        cost_per_1k_output: 0.002,
                        max_context: 128000,
                        max_output: 4096,
                        capabilities: Vec::new(),
                        latency_ms: 500,
                        reliability: 0.9,
                    };
                    decision.reason = format!("Cost gate: {}. Fallback to {}", reason, cheaper);
                    decision.routing_strategy = RoutingStrategy::Fallback;
                }
            }
        }

        // 4. Provider 池检查
        let model_id = decision.selected_model.id.clone();
        if !self.pool.is_available(&model_id) {
            // 降级到下一个可用模型
            let chain = self.fallback_chain.get_chain(&task_type_str);
            for alt in chain {
                if self.pool.is_available(alt) {
                    decision.selected_model = ModelProvider {
                        id: alt.clone(),
                        name: alt.clone(),
                        tier: ModelTier::Fast,
                        cost_per_1k_input: 0.001,
                        cost_per_1k_output: 0.002,
                        max_context: 128000,
                        max_output: 4096,
                        capabilities: Vec::new(),
                        latency_ms: 500,
                        reliability: 0.9,
                    };
                    decision.reason = format!("Provider unavailable: {}. Using {}", model_id, alt);
                    decision.routing_strategy = RoutingStrategy::Fallback;
                    break;
                }
            }
        }

        // 5. 获取连接
        if !self.pool.acquire(&decision.selected_model.id) {
            return Err(GatewayError::AllProvidersUnavailable);
        }

        // 6. 模拟执行 (实际调用在 Tauri 层)
        let output = format!(
            "[Gateway] Processed by {}: {}",
            decision.selected_model.id, request.input
        );
        let cost = decision.estimated_cost.total_cost;
        let latency = (now_secs() - start) * 1000;

        // 7. 释放连接
        self.pool.release(&decision.selected_model.id);

        // 8. 记录消耗
        self.cost_gate.record(&decision.selected_model.id, cost);

        // 9. 记录日志
        let log_entry = RouteLogEntry {
            request_id: request.id.clone(),
            input_preview: request.input.chars().take(50).collect(),
            selected_model: decision.selected_model.id.clone(),
            cost,
            latency_ms: latency,
            degraded: matches!(decision.routing_strategy, RoutingStrategy::Fallback),
            timestamp: now_secs(),
        };
        self.route_log.push(log_entry);

        let is_degraded = matches!(decision.routing_strategy, RoutingStrategy::Fallback);
        let model_used = decision.selected_model.id.clone();
        let degradation_reason = if is_degraded {
            Some(decision.reason.clone())
        } else {
            None
        };

        Ok(GatewayResponse {
            request_id: request.id,
            model_used,
            output,
            cost,
            latency_ms: latency,
            degraded: is_degraded,
            degradation_reason,
            routing: decision,
        })
    }

    /// 获取网关统计
    pub fn stats(&self) -> GatewayStats {
        GatewayStats {
            total_requests: self.route_log.len(),
            total_cost: self.route_log.iter().map(|e| e.cost).sum(),
            degraded_requests: self.route_log.iter().filter(|e| e.degraded).count(),
            pool_stats: self.pool.stats(),
            cost_gate: self.cost_gate.clone(),
        }
    }

    /// 获取路由日志
    pub fn log(&self) -> &[RouteLogEntry] {
        &self.route_log
    }
}

impl Default for ModelGateway {
    fn default() -> Self {
        Self::new()
    }
}

/// 网关统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStats {
    pub total_requests: usize,
    pub total_cost: f64,
    pub degraded_requests: usize,
    pub pool_stats: PoolStats,
    pub cost_gate: CostGate,
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_gate_allowed() {
        let gate = CostGate {
            monthly_budget: Some(100.0),
            monthly_consumed: 10.0,
            ..Default::default()
        };
        assert!(matches!(
            gate.allow("gpt-4o", 5.0),
            CostGateResult::Allowed
        ));
    }

    #[test]
    fn test_cost_gate_denied() {
        let gate = CostGate {
            monthly_budget: Some(100.0),
            monthly_consumed: 95.0,
            ..Default::default()
        };
        assert!(matches!(
            gate.allow("gpt-4o", 10.0),
            CostGateResult::Denied { .. }
        ));
    }

    #[test]
    fn test_fallback_chain_next() {
        let chain = FallbackChain::default();
        let next = chain.next("claude-code", "coding");
        assert_eq!(next.as_deref(), Some("codex"));
    }

    #[test]
    fn test_fallback_chain_default() {
        let chain = FallbackChain::default();
        let next = chain.next("unknown-model", "unknown-task");
        assert_eq!(next.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn test_provider_pool_acquire_release() {
        let mut pool = ProviderPool::default();
        assert!(pool.acquire("gpt-4o"));
        assert!(pool.acquire("gpt-4o"));
        assert!(pool.acquire("gpt-4o"));
        pool.release("gpt-4o");
        assert!(pool.acquire("gpt-4o"));
    }

    #[test]
    fn test_provider_pool_unavailable() {
        let mut pool = ProviderPool::default();
        pool.availability.insert("test".to_string(), false);
        assert!(!pool.acquire("test"));
    }

    #[test]
    fn test_gateway_stats() {
        let gw = ModelGateway::new();
        let stats = gw.stats();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.total_cost, 0.0);
    }
}
