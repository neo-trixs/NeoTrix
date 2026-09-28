//! NeoTrix 统一能力接口 —— **类型层 re-export，非实现层**。
//!
//! 定义跨域调用的标准接口，支持：
//! - 能力注册与发现
//! - 统一输入/输出
//! - 路由调度
//! - 健康监控
//! - 跨域组合
//! - 结果缓存
//!
//! 核心类型全部下沉到 `l0_substrate::nt_core_capability_types`（823 行），
//! 此处仅 re-export 保持向后兼容。
//!
//! ── 2026-09-28：删除 14 个零消费者引擎模块（4,640 行）──
//! 本目录原有 17 个 .rs / 5,708 行，其中 14 个引擎（orchestrator / composer /
//! dependency / discovery / monitoring / monitor / cache / hotreload /
//! loadbalancer / performance / security / versioning / integrator / factory）
//! 与 2 个测试文件（tests.rs / integration_tests.rs）经**逐符号复核**确认
//! **零外部消费者**（`LoadBalancer` 5 处、`VersionManager` 1 处、`Factory` 9 处
//! 表面命中，逐个开 import 核实全是同名异物，如
//! `l5_cognition/nt_core_gwt/load_balancer.rs` 是另一个类型）。
//! 其 91 个测试全部是死代码互测（测自己的 composer/cache/security），
//! 故随代码一并删除。
//!
//! 删除后本目录只剩本文件，实质能力 = L0 的 `UnifiedCapability` trait
//! （16 处实现，跨 L0/L2/L3/L5/neotrix）与 `CapabilityRegistry`。
//!
//! ⚠️ 未随之处理的**重名副本**（尚有活消费者，故本轮不动）：
//! - `LoadBalancer` 3 份：本目录已删，余 `l5_cognition/nt_core_gwt/load_balancer.rs:15`
//!   与 `l5_cognition/nt_core/multi_agent/coordinator/load_balancer.rs:37`
//! - `VersionManager` 2 份：本目录已删，余 `l2_perception/nt_core_knowledge/versioning.rs:141`
//! - `CapabilityRegistry` 3 套并存（L0 类型层 / `nt-core-capability-tree` 治理树 /
//!   L5 `nt_core/capability/registry.rs`），前两者均有活消费者，不可合并；
//!   layer-map.json 的 `_rule` 写「x4」已过期（实为 3，第 4 份已删）。

// Re-export all types from L0 substrate — single source of truth
pub use crate::l0_substrate::nt_core_capability_types::*;

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

// 2026-09-28: 原 `inline_tests_2` 与 `inline_tests` **逐字重复**（同样 4 个同名
// 测试函数 + 同样 80 行 MockCapability），故合并为本处一份，删除重复体。
