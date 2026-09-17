//! Resource discovery framework.
//!
//! A `ResourceDiscoverer` knows how to find resources of a given kind from
//! external sources (HTTP APIs, web scraping, GitHub, etc.). The
//! `ResourceNormalizer` converts raw discovered items into `DiscoveredResource`.
//!
//! The `ResourceDiscoveryEngine` orchestrates multiple discoverers, deduplicates
//! results, and feeds them into the `ResourceRegistry`'s pools.

use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::RwLock;

use async_trait::async_trait;

use super::resource_types::{DiscoveredResource, ResourceKind};

/// How a discoverer identifies itself.
#[derive(Debug, Clone)]
pub struct DiscovererInfo {
    pub name: String,
    pub description: String,
    pub kinds: Vec<ResourceKind>,
    /// How often (in seconds) this discoverer should run
    pub default_interval_secs: u64,
}

/// Result of a single discovery run.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryResult {
    pub kind: ResourceKind,
    pub discovered: Vec<DiscoveredResource>,
    pub errors: Vec<String>,
    pub source_name: String,
}

impl DiscoveryResult {
    pub fn success_count(&self) -> usize {
        self.discovered.len()
    }
}

/// A discoverer knows how to find resources of one or more `ResourceKind`s.
#[async_trait]
pub trait ResourceDiscoverer: Send + Sync {
    fn info(&self) -> DiscovererInfo;

    /// Run one discovery cycle. Returns all discovered resources.
    async fn discover(&self) -> DiscoveryResult;
}

/// Deduplication cache to avoid re-discovering the same resource.
pub struct DiscoveryCache {
    seen: RwLock<HashSet<(ResourceKind, String)>>,
    max_entries: usize,
}

impl DiscoveryCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            seen: RwLock::new(HashSet::with_capacity(max_entries)),
            max_entries,
        }
    }

    /// Returns `true` if the resource is new (not yet seen).
    pub async fn check_and_mark(&self, kind: &ResourceKind, id: &str) -> bool {
        let mut seen = self.seen.write().await;
        let key = (kind.clone(), id.to_string());
        if seen.contains(&key) {
            return false;
        }
        seen.insert(key);
        if seen.len() > self.max_entries {
            seen.clear();
        }
        true
    }
}

/// Unified discovery engine.
pub struct ResourceDiscoveryEngine {
    discoverers: Vec<Box<dyn ResourceDiscoverer>>,
    cache: Arc<DiscoveryCache>,
}

impl ResourceDiscoveryEngine {
    pub fn new() -> Self {
        Self {
            discoverers: Vec::new(),
            cache: Arc::new(DiscoveryCache::new(10_000)),
        }
    }

    /// Register a discoverer.
    pub fn register(&mut self, discoverer: Box<dyn ResourceDiscoverer>) {
        log::info!(
            "[resource-discovery] registered discoverer: {}",
            discoverer.info().name
        );
        self.discoverers.push(discoverer);
    }

    /// Run all discoverers, returning aggregated results.
    pub async fn discover_all(&self) -> Vec<DiscoveryResult> {
        let mut results = Vec::new();
        for discoverer in &self.discoverers {
            let mut result = discoverer.discover().await;
            // Deduplicate via cache
            let info = discoverer.info();
            result.source_name = info.name.clone();
            result.discovered.retain(|r| {
                let is_new =
                    futures::executor::block_on(self.cache.check_and_mark(&r.kind, &r.resource_id));
                is_new
            });
            results.push(result);
        }
        results
    }

    /// Run discoverers for a specific resource kind only.
    pub async fn discover_kind(&self, kind: &ResourceKind) -> Vec<DiscoveryResult> {
        let mut results = Vec::new();
        for discoverer in &self.discoverers {
            let info = discoverer.info();
            if !info.kinds.contains(kind) {
                continue;
            }
            let mut result = discoverer.discover().await;
            result.source_name = info.name.clone();
            result.discovered.retain(|r| {
                futures::executor::block_on(self.cache.check_and_mark(&r.kind, &r.resource_id))
            });
            results.push(result);
        }
        results
    }

    /// How many discoverers are registered.
    pub fn discoverer_count(&self) -> usize {
        self.discoverers.len()
    }

    /// Cache reference for shared use.
    pub fn cache(&self) -> Arc<DiscoveryCache> {
        self.cache.clone()
    }
}

impl Default for ResourceDiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}
