//! ResourceRegistry — a central registry that maps `ResourceKind` to pool
//! implementations. Lets the supervisor query "all proxy pools" or "all LLM
//! provider pools" without knowing concrete types at compile time.
//!
//! Supports:
//! - `register_pool(kind, Arc<dyn AnyPool>)` — type-erased registration
//! - `get_pool(kind)` — get the pool for a resource kind
//! - `discover_and_feed(kind)` — run discoverers and feed results to the pool

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::resource_types::{DiscoveredResource, ResourceKind};

/// Type-erased pool handle. Concrete pools (ProxyPool, NetworkResourcePool)
/// are cast back via `downcast_ref` when accessed through typed helpers.
#[async_trait]
pub trait AnyPool: Send + Sync {
    fn pool_name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;

    /// Feed discovered resources into this pool.
    /// Returns the number of resources successfully added.
    async fn feed(&self, resources: &[DiscoveredResource]) -> usize;
}

/// Central registry of resource kinds to their pool implementations.
pub struct ResourceRegistry {
    pools: RwLock<HashMap<ResourceKind, Arc<dyn AnyPool>>>,
    name_to_kind: RwLock<HashMap<String, ResourceKind>>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            name_to_kind: RwLock::new(HashMap::new()),
        }
    }

    /// Register a pool for a resource kind.
    pub async fn register(&self, kind: ResourceKind, pool: Arc<dyn AnyPool>) {
        let name = pool.pool_name().to_string();
        self.pools.write().await.insert(kind.clone(), pool);
        self.name_to_kind.write().await.insert(name, kind);
    }

    /// Get the pool for a resource kind.
    pub async fn get(&self, kind: &ResourceKind) -> Option<Arc<dyn AnyPool>> {
        self.pools.read().await.get(kind).cloned()
    }

    /// List all registered resource kinds.
    pub async fn registered_kinds(&self) -> Vec<ResourceKind> {
        self.pools.read().await.keys().cloned().collect()
    }

    /// Count registered pools.
    pub async fn pool_count(&self) -> usize {
        self.pools.read().await.len()
    }

    /// Check if a pool is registered for a given kind.
    pub async fn has_kind(&self, kind: &ResourceKind) -> bool {
        self.pools.read().await.contains_key(kind)
    }

    /// List all pools with their names.
    pub async fn list_pools(&self) -> Vec<(ResourceKind, String)> {
        let pools = self.pools.read().await;
        pools.iter().map(|(k, p)| (k.clone(), p.pool_name().to_string())).collect()
    }

    /// Feed discovered resources into the appropriate pool.
    /// Returns (fed_count, skipped_count).
    pub async fn feed(&self, kind: &ResourceKind, resources: &[DiscoveredResource]) -> (usize, usize) {
        let pool = match self.pools.read().await.get(kind) {
            Some(p) => p.clone(),
            None => return (0, resources.len()),
        };
        let fed = pool.feed(resources).await;
        (fed, resources.len().saturating_sub(fed))
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
