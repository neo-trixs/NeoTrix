use std::fmt::Debug;

/// Trait for any resource managed by a pooled system.
///
/// Implementations exist for:
/// - `ProxyNode` (proxy pool)
/// - `IpResource` / `DnsServer` / `RouteNode` (network pool)
/// - LLM provider states (gateway pool)
pub trait PooledResource: Clone + Send + Sync + Debug + 'static {
    /// Unique identifier within the pool
    fn resource_id(&self) -> &str;

    /// Whether the resource is currently usable
    fn is_effective(&self) -> bool;

    /// Composite score (higher = better), used for selection ranking
    fn effective_score(&self) -> f64;

    /// Human-readable label for logging/diagnostics
    fn resource_label(&self) -> &str {
        self.resource_id()
    }

    /// Geo tag if available
    fn geo_tag(&self) -> Option<&str> {
        None
    }
}
