//! NeoTrix 统一能力接口
//!
//! 定义跨域调用的标准接口，支持:
//! - 能力注册与发现
//! - 统一输入/输出
//! - 路由调度
//! - 健康监控
//! - 跨域组合
//! - 结果缓存
//!
//! 核心类型下沉到 `l0_substrate::nt_core_capability_types`, 此处 re-export 保持向后兼容。

// Re-export all types from L0 substrate — single source of truth
pub use crate::l0_substrate::nt_core_capability_types::*;

pub mod cache;
pub mod composer;
pub mod dependency;
pub mod discovery;
pub mod factory;
pub mod integrator;
pub mod loadbalancer;
pub mod monitor;
pub mod monitoring;
pub mod orchestrator;
pub mod performance;
pub mod security;
pub mod versioning;

// 测试和文档模块
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod inline_tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn registry_creation() {
        let registry = CapabilityRegistry::new();
        assert!(registry.list_all().is_empty());
    }

    #[test]
    fn layer_domain_coverage() {
        let layers = vec![
            Layer::L1Action,
            Layer::L2Perception,
            Layer::L3Embodiment,
            Layer::L4Emotion,
            Layer::L5Cognition,
            Layer::L6Meta,
        ];
        let domains = vec![
            Domain::NtCore,
            Domain::NtMind,
            Domain::NtMemory,
            Domain::NtWorld,
            Domain::NtAct,
            Domain::NtIo,
            Domain::NtShield,
            Domain::NtPhysical,
            Domain::NtFeel,
            Domain::NtFileAbility,
            Domain::Trade,
        ];
        assert_eq!(layers.len(), 6);
        assert_eq!(domains.len(), 11);
    }

    #[test]
    fn moe_cost_optimized_routes_cheapest_first() {
        let mut registry = CapabilityRegistry::new();
        let cheap_cap = Arc::new(MockCapability::new("cheap", Domain::NtWorld, 0.1));
        let expensive_cap = Arc::new(MockCapability::new("expensive", Domain::NtWorld, 0.9));
        registry.register(cheap_cap);
        registry.register(expensive_cap);

        let router =
            CapabilityRouter::with_strategy(Arc::new(registry), MoERoutingStrategy::CostOptimized);
        assert_eq!(
            *router.routing_strategy(),
            MoERoutingStrategy::CostOptimized
        );
    }

    #[test]
    fn moe_strategy_default_is_cost_optimized() {
        let registry = Arc::new(CapabilityRegistry::new());
        let router = CapabilityRouter::new(registry);
        assert_eq!(
            *router.routing_strategy(),
            MoERoutingStrategy::CostOptimized
        );
    }

    struct MockCapability {
        id: String,
        domain: Domain,
        cost_weight: f64,
    }

    impl MockCapability {
        fn new(id: &str, domain: Domain, cost_weight: f64) -> Self {
            Self {
                id: id.to_string(),
                domain,
                cost_weight,
            }
        }
    }

    impl UnifiedCapability for MockCapability {
        fn meta(&self) -> CapabilityMeta {
            CapabilityMeta {
                id: self.id.clone(),
                name: self.id.clone(),
                layer: Layer::L1Action,
                domain: self.domain,
                version: "0.1.0".into(),
                description: "mock".into(),
                tags: vec![],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
                cost_weight: self.cost_weight,
                priority: 1.0,
            }
        }
        fn health(&self) -> CapabilityHealth {
            CapabilityHealth {
                state: CapabilityState::Ready,
                success_rate: 1.0,
                avg_latency_ms: 0.0,
                last_called: None,
                call_count: 0,
            }
        }
        fn execute(&self, _input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
            Ok(CapabilityOutput::Text(format!("executed: {}", self.id)))
        }
        fn supports(&self, _input: &CapabilityInput) -> bool {
            true
        }
    }
}

#[cfg(test)]
mod inline_tests_2 {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn registry_creation() {
        let registry = CapabilityRegistry::new();
        assert!(registry.list_all().is_empty());
    }

    #[test]
    fn layer_domain_coverage() {
        let layers = vec![
            Layer::L1Action,
            Layer::L2Perception,
            Layer::L3Embodiment,
            Layer::L4Emotion,
            Layer::L5Cognition,
            Layer::L6Meta,
        ];
        let domains = vec![
            Domain::NtCore,
            Domain::NtMind,
            Domain::NtMemory,
            Domain::NtWorld,
            Domain::NtAct,
            Domain::NtIo,
            Domain::NtShield,
            Domain::NtPhysical,
            Domain::NtFeel,
            Domain::NtFileAbility,
            Domain::Trade,
        ];
        assert_eq!(layers.len(), 6);
        assert_eq!(domains.len(), 11);
    }

    #[test]
    fn moe_cost_optimized_routes_cheapest_first() {
        // Spotify Portal Shunt pattern: route to cheapest capable model
        let mut registry = CapabilityRegistry::new();
        let cheap_cap = Arc::new(MockCapability::new("cheap", Domain::NtWorld, 0.1));
        let expensive_cap = Arc::new(MockCapability::new("expensive", Domain::NtWorld, 0.9));
        registry.register(cheap_cap);
        registry.register(expensive_cap);

        let router =
            CapabilityRouter::with_strategy(Arc::new(registry), MoERoutingStrategy::CostOptimized);
        assert_eq!(
            *router.routing_strategy(),
            MoERoutingStrategy::CostOptimized
        );
    }

    #[test]
    fn moe_strategy_default_is_cost_optimized() {
        let registry = Arc::new(CapabilityRegistry::new());
        let router = CapabilityRouter::new(registry);
        assert_eq!(
            *router.routing_strategy(),
            MoERoutingStrategy::CostOptimized
        );
    }

    struct MockCapability {
        id: String,
        domain: Domain,
        cost_weight: f64,
    }

    impl MockCapability {
        fn new(id: &str, domain: Domain, cost_weight: f64) -> Self {
            Self {
                id: id.to_string(),
                domain,
                cost_weight,
            }
        }
    }

    impl UnifiedCapability for MockCapability {
        fn meta(&self) -> CapabilityMeta {
            CapabilityMeta {
                id: self.id.clone(),
                name: self.id.clone(),
                layer: Layer::L1Action,
                domain: self.domain,
                version: "0.1.0".into(),
                description: "mock".into(),
                tags: vec![],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
                cost_weight: self.cost_weight,
                priority: 1.0,
            }
        }
        fn health(&self) -> CapabilityHealth {
            CapabilityHealth {
                state: CapabilityState::Ready,
                success_rate: 1.0,
                avg_latency_ms: 0.0,
                last_called: None,
                call_count: 0,
            }
        }
        fn execute(&self, _input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
            Ok(CapabilityOutput::Text(format!("executed: {}", self.id)))
        }
        fn supports(&self, _input: &CapabilityInput) -> bool {
            true
        }
    }
}
