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

use std::collections::HashMap;
use std::sync::Arc;

// Re-export all types from L0 substrate — single source of truth
pub use crate::l0_substrate::nt_core_capability_types::*;

pub mod cache;
pub mod composer;
pub mod dependency;
pub mod discovery;
pub mod factory;
pub mod hotreload;
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

/// 能力注册中心
#[derive(Clone)]
pub struct CapabilityRegistry {
    /// 已注册的能力
    capabilities: HashMap<String, Arc<dyn UnifiedCapability>>,
    /// 按域索引
    by_domain: HashMap<Domain, Vec<String>>,
    /// 按层级索引
    by_layer: HashMap<Layer, Vec<String>>,
}

impl CapabilityRegistry {
    /// 创建新的注册中心
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            by_domain: HashMap::new(),
            by_layer: HashMap::new(),
        }
    }

    /// 注册能力
    pub fn register(&mut self, cap: Arc<dyn UnifiedCapability>) {
        let meta = cap.meta();
        let id = meta.id.clone();

        // 索引
        self.by_domain
            .entry(meta.domain)
            .or_default()
            .push(id.clone());
        self.by_layer
            .entry(meta.layer)
            .or_default()
            .push(id.clone());

        self.capabilities.insert(id, cap);
    }

    /// 获取能力
    pub fn get(&self, id: &str) -> Option<Arc<dyn UnifiedCapability>> {
        self.capabilities.get(id).cloned()
    }

    /// 按域获取能力列表
    pub fn by_domain(&self, domain: Domain) -> Vec<Arc<dyn UnifiedCapability>> {
        self.by_domain
            .get(&domain)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.capabilities.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 按层级获取能力列表
    pub fn by_layer(&self, layer: Layer) -> Vec<Arc<dyn UnifiedCapability>> {
        self.by_layer
            .get(&layer)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.capabilities.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 列出所有能力
    pub fn list_all(&self) -> Vec<CapabilityMeta> {
        self.capabilities.values().map(|cap| cap.meta()).collect()
    }

    /// 获取所有健康状态
    pub fn health_all(&self) -> Vec<(CapabilityMeta, CapabilityHealth)> {
        self.capabilities
            .values()
            .map(|cap| (cap.meta(), cap.health()))
            .collect()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 路由调度器
pub struct CapabilityRouter {
    /// 能力注册中心
    registry: Arc<CapabilityRegistry>,
    /// 路由规则
    rules: Vec<Box<dyn Fn(&CapabilityInput) -> Option<String>>>,
    /// MoE routing strategy: route to cheapest capable model (Spotify Portal Shunt pattern)
    routing_strategy: MoERoutingStrategy,
}

/// MoE (Mixture of Experts) routing strategy for cost-aware capability selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoERoutingStrategy {
    /// Route to cheapest capable model (default, ~90% token savings)
    CostOptimized,
    /// Route to highest quality model regardless of cost
    QualityFirst,
    /// Round-robin across capable models
    RoundRobin,
    /// Load-balanced across capable models
    LoadBalanced,
}

impl CapabilityRouter {
    /// 创建新的路由器 (default: CostOptimized)
    pub fn new(registry: Arc<CapabilityRegistry>) -> Self {
        Self {
            registry,
            rules: Vec::new(),
            routing_strategy: MoERoutingStrategy::CostOptimized,
        }
    }

    /// 创建路由器 with explicit MoE routing strategy
    pub fn with_strategy(registry: Arc<CapabilityRegistry>, strategy: MoERoutingStrategy) -> Self {
        Self {
            registry,
            rules: Vec::new(),
            routing_strategy: strategy,
        }
    }

    /// 添加路由规则
    pub fn add_rule<F>(&mut self, rule: F)
    where
        F: Fn(&CapabilityInput) -> Option<String> + 'static,
    {
        self.rules.push(Box::new(rule));
    }

    /// 路由调用
    pub fn route(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        // 尝试路由规则
        for rule in &self.rules {
            if let Some(cap_id) = rule(&input) {
                if let Some(cap) = self.registry.get(&cap_id) {
                    return cap.execute(input);
                }
            }
        }

        // 默认路由: 根据输入类型
        match &input {
            CapabilityInput::Text(_) => {
                // 默认路由到NLP
                self.route_to_domain(input, Domain::NtWorld)
            }
            CapabilityInput::Network(_) => self.route_to_domain(input, Domain::NtShield),
            CapabilityInput::Security(_) => self.route_to_domain(input, Domain::NtShield),
            CapabilityInput::Nlp(_) => self.route_to_domain(input, Domain::NtWorld),
            CapabilityInput::Asset(_) => self.route_to_domain(input, Domain::NtWorld),
            CapabilityInput::FileEnhance(_) => self.route_to_domain(input, Domain::NtFileAbility),
            CapabilityInput::Kv(_) => self.route_to_domain(input, Domain::NtMemory),
        }
    }

    /// 按域路由 — MoE cost-aware: sorts by cost_weight when CostOptimized
    fn route_to_domain(
        &self,
        input: CapabilityInput,
        domain: Domain,
    ) -> Result<CapabilityOutput, CapabilityError> {
        let mut caps: Vec<Arc<dyn UnifiedCapability>> = self.registry.by_domain(domain);
        let strategy = self.routing_strategy.clone();

        // MoE routing: sort by cost_weight (cheapest first) for CostOptimized strategy
        // Source: Spotify Portal Shunt — route I/O to cheapest capable model (~90% savings)
        match strategy {
            MoERoutingStrategy::CostOptimized => {
                caps.sort_by(|a, b| {
                    let a_cost = a.meta().cost_weight;
                    let b_cost = b.meta().cost_weight;
                    a_cost.partial_cmp(&b_cost).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            MoERoutingStrategy::QualityFirst => {
                // Reverse: highest cost_weight (highest quality) first
                caps.sort_by(|a, b| {
                    let a_cost = a.meta().cost_weight;
                    let b_cost = b.meta().cost_weight;
                    b_cost.partial_cmp(&a_cost).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            MoERoutingStrategy::RoundRobin | MoERoutingStrategy::LoadBalanced => {
                // No pre-sort; round-robin or load-based handled externally
            }
        }

        for cap in caps {
            if cap.supports(&input) {
                return cap.execute(input);
            }
        }
        Err(CapabilityError::UnsupportedInput("无匹配能力".into()))
    }

    /// 获取注册中心引用
    pub fn registry(&self) -> &Arc<CapabilityRegistry> {
        &self.registry
    }

    /// Set the MoE routing strategy
    pub fn set_routing_strategy(&mut self, strategy: MoERoutingStrategy) {
        self.routing_strategy = strategy;
    }

    /// Get the current MoE routing strategy
    pub fn routing_strategy(&self) -> &MoERoutingStrategy {
        &self.routing_strategy
    }
}

#[cfg(test)]
mod inline_tests {
    use super::*;

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

        let router = CapabilityRouter::with_strategy(
            Arc::new(registry),
            MoERoutingStrategy::CostOptimized,
        );
        assert_eq!(*router.routing_strategy(), MoERoutingStrategy::CostOptimized);
    }

    #[test]
    fn moe_strategy_default_is_cost_optimized() {
        let registry = Arc::new(CapabilityRegistry::new());
        let router = CapabilityRouter::new(registry);
        assert_eq!(*router.routing_strategy(), MoERoutingStrategy::CostOptimized);
    }

    struct MockCapability {
        id: String,
        domain: Domain,
        cost_weight: f64,
    }

    impl MockCapability {
        fn new(id: &str, domain: Domain, cost_weight: f64) -> Self {
            Self { id: id.to_string(), domain, cost_weight }
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
