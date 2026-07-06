use std::sync::Arc;
use std::time::{Duration, Instant};

use super::pooled_resource::PooledResource;
use super::selection_strategy::PoolSelectionStrategy;

/// Snapshot of pool state at a point in time.
#[derive(Debug, Clone)]
pub struct PoolSnapshot {
    pub total: usize,
    pub effective: usize,
    pub strategy: PoolSelectionStrategy,
    pub healthy_count: usize,
    pub stale_count: usize,
}

impl PoolSnapshot {
    pub fn effective_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.effective as f64 / self.total as f64
        }
    }
}

impl Default for PoolSnapshot {
    fn default() -> Self {
        Self {
            total: 0,
            effective: 0,
            strategy: PoolSelectionStrategy::Fastest,
            healthy_count: 0,
            stale_count: 0,
        }
    }
}

/// Health report from a pool scan.
#[derive(Debug, Clone, Default)]
pub struct PoolHealthReport {
    pub total_checked: usize,
    pub healthy: usize,
    pub failed: usize,
    pub total_latency_ms: f64,
    pub duration_ms: f64,
}

/// The core trait that all resource pools implement.
///
/// This provides a unified interface for:
/// - Proxy pools (`ProxyPool`)
/// - Network resource pools (`NetworkResourcePool`)
/// - LLM provider pools (`GatewayV2`)
pub trait ResourcePool: Send + Sync {
    /// The type of resource managed by this pool.
    type Resource: PooledResource;

    /// Total number of registered resources
    fn total(&self) -> impl std::future::Future<Output = usize> + Send;

    /// Number of effective (usable) resources
    fn effective_count(&self) -> impl std::future::Future<Output = usize> + Send;

    /// Register a new resource
    fn register(&self, resource: Self::Resource) -> impl std::future::Future<Output = ()> + Send;

    /// Select the best resource for a given context (e.g. host)
    fn select_for(
        &self,
        context: Option<&str>,
    ) -> impl std::future::Future<Output = Option<Self::Resource>> + Send;

    /// Report a successful use of a resource
    fn report_success(
        &self,
        resource_id: &str,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Report a failed use of a resource
    fn report_failure(
        &self,
        resource_id: &str,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Health-check all managed resources, updating their state
    fn health_check(&self) -> impl std::future::Future<Output = PoolHealthReport> + Send;

    /// Current pool snapshot
    fn snapshot(&self) -> impl std::future::Future<Output = PoolSnapshot> + Send;
}

/// Generic background supervisor that runs health checks and auto-replenish.
pub struct PoolSupervisor {
    pool_name: String,
    health_interval: Duration,
    replenish_interval: Duration,
}

impl PoolSupervisor {
    pub fn new(pool_name: &str) -> Self {
        Self {
            pool_name: pool_name.to_string(),
            health_interval: Duration::from_secs(30),
            replenish_interval: Duration::from_secs(300),
        }
    }

    pub fn with_health_interval(mut self, secs: u64) -> Self {
        self.health_interval = Duration::from_secs(secs);
        self
    }

    pub fn with_replenish_interval(mut self, secs: u64) -> Self {
        self.replenish_interval = Duration::from_secs(secs);
        self
    }

    /// Start the supervisory loop. Runs until the pool is dropped.
    pub async fn run<P: ResourcePool + ?Sized>(
        self,
        pool: Arc<P>,
        mut replenisher: Option<Box<dyn FnMut() -> futures::future::BoxFuture<'static, ()> + Send>>,
    ) {
        let mut last_replenish = Instant::now();
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            // Health check every `health_interval`
            let report = pool.health_check().await;
            log::debug!(
                "[{}] health: {}/{} ok in {:.0}ms",
                self.pool_name,
                report.healthy,
                report.total_checked,
                report.duration_ms,
            );

            // Replenish every `replenish_interval`
            if last_replenish.elapsed() >= self.replenish_interval {
                if let Some(ref mut replenish_fn) = replenisher {
                    log::info!("[{}] auto-replenishing...", self.pool_name);
                    (replenish_fn)().await;
                }
                last_replenish = Instant::now();
            }
        }
    }
}
