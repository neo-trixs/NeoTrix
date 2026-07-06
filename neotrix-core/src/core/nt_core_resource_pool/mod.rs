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

mod pooled_resource;
mod pool_trait;
mod selection_strategy;
mod resource_types;
mod resource_registry;
mod discovery;
mod normalizer;

pub use pooled_resource::PooledResource;
pub use pool_trait::{PoolHealthReport, PoolSnapshot, PoolSupervisor, ResourcePool};
pub use selection_strategy::PoolSelectionStrategy;
pub use resource_types::{ResourceKind, DiscoveredResource, ResourceMeta};
pub use resource_registry::{AnyPool, ResourceRegistry};
pub use discovery::{DiscovererInfo, DiscoveryResult, DiscoveryCache, ResourceDiscoverer, ResourceDiscoveryEngine};
pub use normalizer::{ResourceNormalizer, NormalizedEntry, ProxyUrlNormalizer};
