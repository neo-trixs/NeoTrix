//! # NT-CORE Resource Pool — Unified Adaptive Resource Pool
//!
//! Provides the foundational traits and generic supervisor for all
//! resource pool implementations across the NeoTrix stack.
//!
//! ## PooledResource trait
//! Any entity that can be managed by a pool (proxy node, LLM provider,
//! DNS server, IP resource, etc.)
//!
//! ## ResourcePool trait
//! The common interface for all pool implementations: registration,
//! selection, health-checking, and reporting.
//!
//! ## PoolSupervisor
//! Generic background loop that periodically health-checks and
//! auto-replenishes any `ResourcePool`.
//!
//! ## Architecture unification
//! - `ProxyPool` → implements `ResourcePool<Resource=ProxyNode>`
//! - `NetworkResourcePool` → implements `ResourcePool<Resource=IpResource>`
//!   (and separately for DnsServer / RouteNode)
//! - `GatewayV2` → implements `ResourcePool<Resource=LlmProviderState>`

mod discovery;
mod normalizer;
mod pool_trait;
mod pooled_resource;
mod resource_registry;
mod resource_types;
mod selection_strategy;

pub use discovery::{
    DiscovererInfo, DiscoveryCache, DiscoveryResult, ResourceDiscoverer, ResourceDiscoveryEngine,
};
pub use normalizer::{NormalizedEntry, ProxyUrlNormalizer, ResourceNormalizer};
pub use pool_trait::{PoolHealthReport, PoolSnapshot, PoolSupervisor, ResourcePool};
pub use pooled_resource::PooledResource;
pub use resource_registry::{AnyPool, ResourceRegistry};
pub use resource_types::{DiscoveredResource, ResourceKind, ResourceMeta};
pub use selection_strategy::PoolSelectionStrategy;
