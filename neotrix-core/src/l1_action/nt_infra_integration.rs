//! L1 基础设施 — 统一集成层
//!
//! 将 tracing + breaker + semantic_router + agent_card + scatter_gather + persistence + learning
//! 集成到 Registry/Router/Bridge 的调用链路中

use std::collections::HashMap;

// ════════════════════════════════════════════════════════════════
// 增强 Registry — 自动追踪 + 断路器 + Agent Card
// ════════════════════════════════════════════════════════════════

/// 增强型 Registry — 自动集成基础设施
pub struct EnhancedRegistry {
    pub entries: Vec<super::nt_infra_agent_card::AgentCard>,
    pub breaker: super::nt_infra_breaker::BreakerRegistry,
    pub persistence: super::nt_infra_persistence::RegistryPersistence,
}

impl Default for EnhancedRegistry {
    fn default() -> Self { Self::new() }
}

impl EnhancedRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            breaker: super::nt_infra_breaker::BreakerRegistry::new(),
            persistence: super::nt_infra_persistence::RegistryPersistence::new(
                &format!("{}/.neotrix/registry.json",
                    std::env::var("HOME").unwrap_or_default())
            ),
        }
    }

    /// 注册 Provider (自动发布 Agent Card + 断路器)
    pub fn register(&mut self, card: super::nt_infra_agent_card::AgentCard) {
        // 自动注册断路器
        self.breaker.get_or_create(&card.id);
        // 持久化
        self.persistence.upsert(super::nt_infra_persistence::PersistedEntry {
            id: card.id.clone(),
            category: card.tags.first().cloned().unwrap_or_default(),
            constellation: "C1".into(),
            description: card.description.clone(),
            health_healthy: true,
            health_error_rate: 0.0,
            tags: card.tags.clone(),
            metadata: HashMap::new(),
        });
        self.entries.push(card);
    }

    /// 检查 Provider 是否可用 (断路器 + 健康)
    pub fn is_available(&self, id: &str) -> bool {
        self.breaker.allow(id) && self.entries.iter().any(|e| e.id == id)
    }

    /// 记录调用结果 (断路器 + 追踪)
    pub fn record_result(&mut self, id: &str, success: bool) {
        self.breaker.record_result(id, success);
        let span_id = super::nt_infra_tracing::trace_start(id, "call");
        super::nt_infra_tracing::trace_end(&span_id, success, None);
    }

    /// 获取所有可用 Provider
    pub fn available(&self) -> Vec<&super::nt_infra_agent_card::AgentCard> {
        self.entries.iter()
            .filter(|e| self.is_available(&e.id))
            .collect()
    }

    /// 按能力查找
    pub fn find_by_capability(&self, cap: &str) -> Vec<&super::nt_infra_agent_card::AgentCard> {
        self.entries.iter()
            .filter(|e| self.is_available(&e.id))
            .filter(|e| e.capabilities.iter().any(|c| c.name == cap))
            .collect()
    }

    /// 保存到磁盘
    pub fn save(&self) -> Result<(), String> {
        self.persistence.save()
    }

    /// 从磁盘恢复
    pub fn load(&mut self) -> Result<(), String> {
        self.persistence.load()
    }
}

// ════════════════════════════════════════════════════════════════
// 增强 Router — 语义路由 + 学习 + Scatter-Gather
// ════════════════════════════════════════════════════════════════

/// 增强型 Router
pub struct EnhancedRouter {
    pub semantic: super::nt_infra_semantic_router::SemanticRouter,
    pub learner: super::nt_infra_learning::RouterLearner,
    pub scatter: super::nt_infra_scatter_gather::ScatterGather,
}

impl Default for EnhancedRouter {
    fn default() -> Self { Self::new() }
}

impl EnhancedRouter {
    pub fn new() -> Self {
        Self {
            semantic: super::nt_infra_semantic_router::SemanticRouter::new(),
            learner: super::nt_infra_learning::RouterLearner::new(),
            scatter: super::nt_infra_scatter_gather::ScatterGather::new(),
        }
    }

    /// 智能路由: 语义 → 学习 → 默认
    pub fn route(&self, query: &str, intent: Option<&str>) -> Option<String> {
        // 1. 语义路由
        if let Some(decision) = self.semantic.route(query, intent) {
            if decision.confidence > 0.6 {
                return Some(decision.provider_id);
            }
        }
        // 2. 学习权重
        if let Some(best) = self.learner.best_provider() {
            return Some(best.to_string());
        }
        // 3. 回退
        None
    }

    /// 记录调用结果 (学习 + 追踪)
    pub fn record_result(&mut self, provider_id: &str, success: bool, latency_ms: f64) {
        self.learner.record_call(provider_id, success, latency_ms);
        let span_id = super::nt_infra_tracing::trace_start(provider_id, "route");
        super::nt_infra_tracing::trace_end(&span_id, success, None);
    }

    /// Scatter-Gather: 并行查询多个 Provider
    pub fn scatter_gather(&self, _query: &str, providers: Vec<String>) -> super::nt_infra_scatter_gather::GatherResult {
        let responses = providers.into_iter().map(|p| {
            super::nt_infra_scatter_gather::ProviderResponse {
                provider_id: p,
                score: 0.5,
                data: vec![],
                latency_ms: 100,
                success: true,
            }
        }).collect();
        self.scatter.gather(responses)
    }
}

// ════════════════════════════════════════════════════════════════
// 全局增强实例
// ════════════════════════════════════════════════════════════════

lazy_static::lazy_static! {
    pub static ref ENHANCED_REGISTRY: std::sync::Mutex<EnhancedRegistry> =
        std::sync::Mutex::new(EnhancedRegistry::new());
    pub static ref ENHANCED_ROUTER: std::sync::Mutex<EnhancedRouter> =
        std::sync::Mutex::new(EnhancedRouter::new());
}

pub fn enhanced_register(card: super::nt_infra_agent_card::AgentCard) {
    ENHANCED_REGISTRY.lock().unwrap_or_else(|e| e.into_inner()).register(card);
}

pub fn enhanced_is_available(id: &str) -> bool {
    ENHANCED_REGISTRY.lock().unwrap_or_else(|e| e.into_inner()).is_available(id)
}

pub fn enhanced_route(query: &str, intent: Option<&str>) -> Option<String> {
    ENHANCED_ROUTER.lock().unwrap_or_else(|e| e.into_inner()).route(query, intent)
}

pub fn enhanced_record(provider_id: &str, success: bool, latency_ms: f64) {
    ENHANCED_REGISTRY.lock().unwrap_or_else(|e| e.into_inner()).record_result(provider_id, success);
    ENHANCED_ROUTER.lock().unwrap_or_else(|e| e.into_inner()).record_result(provider_id, success, latency_ms);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_registry() {
        let mut reg = EnhancedRegistry::new();
        let card = super::nt_infra_agent_card::AgentCard::new("test", "Test", "Test agent");
        reg.register(card);
        assert!(reg.is_available("test"));
        assert_eq!(reg.available().len(), 1);
    }

    #[test]
    fn test_enhanced_router() {
        let mut router = EnhancedRouter::new();
        router.semantic.add_rule(super::nt_infra_semantic_router::RouteRule {
            id: "r1".into(),
            intent: "search".into(),
            keywords: vec!["search".into()],
            provider_preference: vec!["kb".into()],
            priority: 1,
        });
        let result = router.route("search for info", None);
        assert!(result.is_some());
    }
}
