//! # Pipeline factories — build complete data pipelines for each resource type.
//!
//! Each factory wires Acquisition → Normalization → Dedup → Storage → Health
//! stages into a `PipelineOrchestrator` that can be run from the background loop.

use async_trait::async_trait;

use crate::core::nt_core_data_pipeline::{PipelineOrchestrator, PipelineStage, StageResult};
use crate::core::nt_core_resource_pool::ResourceDiscoverer;
#[cfg(feature = "stealth-net")]
use crate::neotrix::nt_shield_stealth_net::proxy_pool::{global_pool, ProxyPool};
use crate::neotrix::l2_world_impl::nt_world_resource_discovery::{
    LlmFreeProviderDiscoverer, ProxyScraperDiscoverer, ProxySubscriptionDiscoverer,
};

// ── Proxy Scraper Pipeline Stage ──

pub struct ProxyScraperAcquireStage {
    discoverer: ProxyScraperDiscoverer,
}

impl Default for ProxyScraperAcquireStage {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyScraperAcquireStage {
    pub fn new() -> Self {
        Self {
            discoverer: ProxyScraperDiscoverer::new(),
        }
    }
}

#[async_trait]
impl PipelineStage for ProxyScraperAcquireStage {
    fn name(&self) -> &str {
        "proxy-scraper-acquire"
    }

    async fn execute(&self, context: &str) -> StageResult {
        let mut result = StageResult::new(self.name());
        let discovery_result = self.discoverer.discover().await;
        let count = discovery_result.discovered.len();
        result.items_processed = count;
        result.items_succeeded = count;
        let err_count = discovery_result.errors.len();
        result.errors = discovery_result.errors;
        result.items_failed = err_count;
        log::info!(
            "[pipeline] proxy-scraper({}): discovered {} proxies",
            context,
            count
        );
        result
    }
}

// ── Proxy Subscription Pipeline Stage ──

pub struct ProxySubscriptionAcquireStage {
    discoverer: ProxySubscriptionDiscoverer,
}

impl Default for ProxySubscriptionAcquireStage {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxySubscriptionAcquireStage {
    pub fn new() -> Self {
        Self {
            discoverer: ProxySubscriptionDiscoverer::new(),
        }
    }
}

#[async_trait]
impl PipelineStage for ProxySubscriptionAcquireStage {
    fn name(&self) -> &str {
        "proxy-subscription-acquire"
    }

    async fn execute(&self, context: &str) -> StageResult {
        let mut result = StageResult::new(self.name());
        let discovery_result = self.discoverer.discover().await;
        let count = discovery_result.discovered.len();
        result.items_processed = count;
        result.items_succeeded = count;
        let err_count = discovery_result.errors.len();
        result.errors = discovery_result.errors;
        result.items_failed = err_count;
        log::info!(
            "[pipeline] proxy-subscription({}): discovered {} proxies",
            context,
            count
        );
        result
    }
}

// ── Proxy Pool Store Stage ──

#[cfg(feature = "stealth-net")]
pub struct ProxyPoolStoreStage {
    pool: std::sync::Arc<ProxyPool>,
}

#[cfg(feature = "stealth-net")]
impl ProxyPoolStoreStage {
    pub fn new(pool: std::sync::Arc<ProxyPool>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "stealth-net")]
#[async_trait]
impl PipelineStage for ProxyPoolStoreStage {
    fn name(&self) -> &str {
        "proxy-pool-store"
    }

    async fn execute(&self, context: &str) -> StageResult {
        let mut result = StageResult::new(self.name());
        let before: usize = self.pool.total_count().await;
        self.pool.heal_if_needed().await;
        self.pool.health_check().await;
        let after: usize = self.pool.total_count().await;
        result.items_processed = after.saturating_sub(before);
        result.items_succeeded = self.pool.available_count().await;
        log::info!(
            "[pipeline] proxy-pool-store({}): {} total, {} available",
            context,
            after,
            result.items_succeeded
        );
        result
    }
}

// ── LLM Provider Discovery Pipeline Stage ──

pub struct LlmProviderAcquireStage {
    discoverer: LlmFreeProviderDiscoverer,
}

impl Default for LlmProviderAcquireStage {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmProviderAcquireStage {
    pub fn new() -> Self {
        Self {
            discoverer: LlmFreeProviderDiscoverer::new(),
        }
    }
}

#[async_trait]
impl PipelineStage for LlmProviderAcquireStage {
    fn name(&self) -> &str {
        "llm-provider-acquire"
    }

    async fn execute(&self, _context: &str) -> StageResult {
        let mut result = StageResult::new(self.name());
        let discovery_result = self.discoverer.discover().await;
        let count = discovery_result.discovered.len();
        result.items_processed = count;
        result.items_succeeded = count;
        log::info!(
            "[pipeline] llm-provider-acquire: discovered {} providers",
            count
        );
        result
    }
}

// ── Pipeline Factories ──

/// Build a complete proxy resource lifecycle pipeline.
#[cfg(feature = "stealth-net")]
pub fn create_proxy_pipeline() -> PipelineOrchestrator {
    let mut pipeline = PipelineOrchestrator::new("proxy-lifecycle");

    pipeline.register(Box::new(ProxyScraperAcquireStage::new()));
    pipeline.register(Box::new(ProxySubscriptionAcquireStage::new()));
    pipeline.register(Box::new(ProxyPoolStoreStage::new(global_pool())));

    pipeline
}

/// Build a complete LLM provider lifecycle pipeline.
pub fn create_llm_provider_pipeline() -> PipelineOrchestrator {
    let mut pipeline = PipelineOrchestrator::new("llm-provider-lifecycle");

    pipeline.register(Box::new(LlmProviderAcquireStage::new()));

    pipeline
}

/// Build all resource pipelines and configure them.
#[cfg(feature = "stealth-net")]
pub fn create_all_pipelines() -> Vec<PipelineOrchestrator> {
    vec![
        create_proxy_pipeline(),
        create_llm_provider_pipeline(),
    ]
}

/// Build all pipelines without stealth-net (LLM-only).
#[cfg(not(feature = "stealth-net"))]
pub fn create_all_pipelines() -> Vec<PipelineOrchestrator> {
    vec![
        create_llm_provider_pipeline(),
    ]
}
