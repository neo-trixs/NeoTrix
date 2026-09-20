#![forbid(unsafe_code)]

//! # Provider Manager — 统一 Provider 管理
//!
//! 集成 Circuit Breaker、Cost Tracker、Failover Chain。
//! 基于 Azure APIM Unified Model API 和 oxllm 路由模式。

use crate::service::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager};
use crate::service::cost_tracker::CostTracker;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Provider 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Vec<String>,
    pub enabled: bool,
    pub priority: u32,
    pub failover_group: Option<String>,
}

/// Provider 健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub provider_id: String,
    pub available: bool,
    pub latency_ms: f64,
    pub last_check: String,
    pub error: Option<String>,
}

/// Failover 链配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverChain {
    pub name: String,
    pub providers: Vec<String>,
    pub task_type: String,
}

/// Provider Manager
pub struct ProviderManager {
    providers: HashMap<String, ProviderConfig>,
    circuit_breakers: CircuitBreakerManager,
    cost_tracker: CostTracker,
    failover_chains: Vec<FailoverChain>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            circuit_breakers: CircuitBreakerManager::new(CircuitBreakerConfig::default()),
            cost_tracker: CostTracker::new(),
            failover_chains: Vec::new(),
        }
    }

    /// 注册 Provider
    pub fn register(&mut self, config: ProviderConfig) {
        self.providers.insert(config.id.clone(), config);
    }

    /// 移除 Provider
    pub fn unregister(&mut self, provider_id: &str) -> bool {
        self.providers.remove(provider_id).is_some()
    }

    /// 获取可用 Provider（考虑 circuit breaker）
    pub fn available_providers(&self) -> Vec<&ProviderConfig> {
        self.providers
            .values()
            .filter(|p| p.enabled && self.circuit_breakers.can_request(&p.id))
            .collect()
    }

    /// 获取 failover 链
    pub fn get_failover_chain(&self, task_type: &str) -> Vec<&ProviderConfig> {
        // 查找匹配的 failover chain
        if let Some(chain) = self
            .failover_chains
            .iter()
            .find(|c| c.task_type == task_type)
        {
            return chain
                .providers
                .iter()
                .filter_map(|id| self.providers.get(id))
                .filter(|p| p.enabled && self.circuit_breakers.can_request(&p.id))
                .collect();
        }

        // 默认：按 priority 排序
        let mut providers: Vec<&ProviderConfig> = self
            .providers
            .values()
            .filter(|p| p.enabled && self.circuit_breakers.can_request(&p.id))
            .collect();
        providers.sort_by_key(|p| p.priority);
        providers
    }

    /// 记录请求成功
    pub fn record_success(&mut self, provider_id: &str) {
        self.circuit_breakers.record_success(provider_id);
    }

    /// 记录请求失败
    pub fn record_failure(&mut self, provider_id: &str) {
        self.circuit_breakers.record_failure(provider_id);
    }

    /// 获取 Circuit Breaker 快照
    pub fn circuit_breaker_snapshot(
        &self,
    ) -> Vec<crate::service::circuit_breaker::ProviderCircuitBreaker> {
        self.circuit_breakers.snapshot()
    }

    /// 获取成本摘要
    pub fn cost_summaries(&self) -> Vec<&crate::service::cost_tracker::ProviderCostSummary> {
        self.cost_tracker.summaries()
    }

    /// 注册 failover chain
    pub fn register_failover_chain(&mut self, chain: FailoverChain) {
        self.failover_chains.push(chain);
    }

    /// 列出所有 Provider
    pub fn list_providers(&self) -> Vec<&ProviderConfig> {
        self.providers.values().collect()
    }

    /// 获取 Provider 配置
    pub fn get_provider(&self, provider_id: &str) -> Option<&ProviderConfig> {
        self.providers.get(provider_id)
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 共享 Provider Manager 状态
pub type SharedProviderManager = Arc<RwLock<ProviderManager>>;

pub fn new_shared_provider_manager() -> SharedProviderManager {
    Arc::new(RwLock::new(ProviderManager::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_provider() {
        let mut manager = ProviderManager::new();
        let config = ProviderConfig {
            id: "openai".into(),
            name: "OpenAI".into(),
            provider_type: "openai".into(),
            api_key: None,
            base_url: None,
            models: vec!["gpt-4".into()],
            enabled: true,
            priority: 1,
            failover_group: None,
        };
        manager.register(config);
        assert_eq!(manager.list_providers().len(), 1);
    }

    #[test]
    fn test_failover_chain() {
        let mut manager = ProviderManager::new();

        // 注册 providers
        for (id, priority) in [("openai", 1), ("anthropic", 2), ("ollama", 3)] {
            manager.register(ProviderConfig {
                id: id.into(),
                name: id.into(),
                provider_type: "openai".into(),
                api_key: None,
                base_url: None,
                models: vec![],
                enabled: true,
                priority,
                failover_group: None,
            });
        }

        // 注册 failover chain
        manager.register_failover_chain(FailoverChain {
            name: "code".into(),
            providers: vec!["openai".into(), "anthropic".into(), "ollama".into()],
            task_type: "code".into(),
        });

        let chain = manager.get_failover_chain("code");
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].id, "openai");
    }
}
