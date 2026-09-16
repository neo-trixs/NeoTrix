//! Channel-aware adapter wrapper
//!
//! Wraps a SocialPlatformAdapter with multi-backend channel routing.
//! Automatically probes channels and routes requests through the active backend.

use std::sync::{Arc, RwLock};

use super::channel::{ChannelRegistry, ProbeResult};
use super::feed::PlatformAdapterRegistry;
use super::traits::*;

/// Adapter wrapper with channel awareness
pub struct ChannelAdapter {
    inner: Arc<dyn SocialPlatformAdapter>,
    channel_registry: Arc<RwLock<ChannelRegistry>>,
}

impl ChannelAdapter {
    pub fn new(
        adapter: Arc<dyn SocialPlatformAdapter>,
        registry: Arc<RwLock<ChannelRegistry>>,
    ) -> Self {
        Self {
            inner: adapter,
            channel_registry: registry,
        }
    }

    /// Get the channel name for this adapter
    fn channel_name(&self) -> Option<&str> {
        self.inner.channel_name()
    }

    /// Probe the channel and return active backend info
    pub fn probe_channel(&self) -> Option<ProbeResult> {
        let channel_name = self.channel_name()?;
        let registry = self.channel_registry.read().ok()?;
        let channel = registry.get(channel_name)?;

        // Return the active backend status if known
        if let Some(ref active) = channel.active_backend {
            return Some(ProbeResult::ok(
                format!("active backend: {}", active),
                0,
            ));
        }

        None
    }

    /// Get the underlying adapter
    pub fn inner(&self) -> &dyn SocialPlatformAdapter {
        self.inner.as_ref()
    }

    /// Get the channel registry
    pub fn registry(&self) -> &Arc<RwLock<ChannelRegistry>> {
        &self.channel_registry
    }
}

impl SocialPlatformAdapter for ChannelAdapter {
    fn id(&self) -> PlatformId {
        self.inner.id()
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn auth_flow(&self) -> AuthFlow {
        self.inner.auth_flow()
    }

    fn api_base_url(&self) -> &'static str {
        self.inner.api_base_url()
    }

    fn channel_name(&self) -> Option<&'static str> {
        self.inner.channel_name()
    }

    fn get_recommended(
        &self,
        session: &SessionEntry,
        limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        // Try channel-aware path first
        if let Some(channel_name) = self.channel_name() {
            let registry = self.channel_registry.read().map_err(|e| {
                SocialAccessError::Platform(format!("registry lock: {}", e))
            })?;

            if let Some(channel) = registry.get(channel_name) {
                if channel.active_backend.is_some() {
                    // Channel is healthy, delegate to inner adapter
                    return self.inner.get_recommended(session, limit);
                }
            }
        }

        // Fallback to direct adapter
        self.inner.get_recommended(session, limit)
    }

    fn get_following(
        &self,
        session: &SessionEntry,
        limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        self.inner.get_following(session, limit)
    }

    fn get_trending(&self) -> Result<Vec<TrendingTopic>, SocialAccessError> {
        self.inner.get_trending()
    }

    fn search(
        &self,
        session: &SessionEntry,
        query: &str,
        limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        self.inner.search(session, query, limit)
    }

    fn get_from_url(
        &self,
        session: &SessionEntry,
        url: &str,
    ) -> Result<ExtractorResult, SocialAccessError> {
        self.inner.get_from_url(session, url)
    }
}

/// Registry that manages channel-aware adapters
pub struct ChannelAdapterRegistry {
    adapters: Vec<Arc<ChannelAdapter>>,
    channel_registry: Arc<RwLock<ChannelRegistry>>,
}

impl ChannelAdapterRegistry {
    pub fn new(channel_registry: Arc<RwLock<ChannelRegistry>>) -> Self {
        Self {
            adapters: Vec::new(),
            channel_registry,
        }
    }

    /// Register a channel-aware adapter
    pub fn register(&mut self, adapter: Arc<dyn SocialPlatformAdapter>) {
        let channel_adapter = Arc::new(ChannelAdapter::new(adapter, self.channel_registry.clone()));
        self.adapters.push(channel_adapter);
    }

    /// Get all adapters
    pub fn adapters(&self) -> &[Arc<ChannelAdapter>] {
        &self.adapters
    }

    /// Find adapter by platform ID
    pub fn find(&self, platform_id: &str) -> Option<&ChannelAdapter> {
        self.adapters.iter().find(|a| a.id() == platform_id).map(|a| a.as_ref())
    }

    /// Probe all channels and return results
    pub fn probe_all(&self) -> Vec<(String, Option<ProbeResult>)> {
        self.adapters
            .iter()
            .map(|a| {
                let name = a.id();
                let result = a.probe_channel();
                (name, result)
            })
            .collect()
    }

    /// Convert to PlatformAdapterRegistry for backward compatibility
    pub fn to_platform_registry(&self) -> PlatformAdapterRegistry {
        let mut registry = PlatformAdapterRegistry::new();
        for adapter in &self.adapters {
            registry.register(adapter.clone() as Arc<dyn SocialPlatformAdapter>);
        }
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockAdapter {
        id: String,
        channel: Option<&'static str>,
    }

    impl SocialPlatformAdapter for MockAdapter {
        fn id(&self) -> PlatformId { self.id.clone() }
        fn name(&self) -> &'static str { "Mock" }
        fn auth_flow(&self) -> AuthFlow { AuthFlow::OAuth2PKCE }
        fn api_base_url(&self) -> &'static str { "https://mock.com" }
        fn channel_name(&self) -> Option<&'static str> { self.channel }
        fn get_recommended(&self, _: &SessionEntry, _: usize) -> Result<FeedResult, SocialAccessError> {
            Ok(FeedResult { items: vec![], total: 0 })
        }
        fn get_following(&self, _: &SessionEntry, _: usize) -> Result<FeedResult, SocialAccessError> {
            Ok(FeedResult { items: vec![], total: 0 })
        }
        fn get_trending(&self) -> Result<Vec<TrendingTopic>, SocialAccessError> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_channel_adapter_channel_name() {
        let registry = Arc::new(RwLock::new(super::super::channel::default_channels()));
        let adapter = Arc::new(MockAdapter {
            id: "twitter".into(),
            channel: Some("twitter"),
        });

        let channel_adapter = ChannelAdapter::new(adapter, registry);
        assert_eq!(channel_adapter.channel_name(), Some("twitter"));
    }

    #[test]
    fn test_channel_adapter_registry() {
        let channel_registry = Arc::new(RwLock::new(super::super::channel::default_channels()));
        let mut registry = ChannelAdapterRegistry::new(channel_registry);

        registry.register(Arc::new(MockAdapter {
            id: "twitter".into(),
            channel: Some("twitter"),
        }));

        assert_eq!(registry.adapters.len(), 1);
        assert!(registry.find("twitter").is_some());
        assert!(registry.find("nonexistent").is_none());
    }
}
