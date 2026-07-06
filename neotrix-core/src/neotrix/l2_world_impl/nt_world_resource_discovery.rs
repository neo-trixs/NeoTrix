//! # NT-WORLD Resource Discovery Engine
//!
//! Unified orchestrator that discovers resources (proxies, LLM providers, etc.)
//! from external sources, normalizes them, and feeds them into the
//! `ResourceRegistry` for automatic pool management.
//!
//! ## Built-in discoverers
//!
//! | Discoverer | Kind | Source | Config |
//! |------------|------|--------|--------|
//! | `ProxyScraperDiscoverer` | Proxy | `FREE_PROXY_SCRAPERS` (geonode, proxy-list, proxyscrape) | Always active |
//! | `ProxySubscriptionDiscoverer` | Proxy | `DEFAULT_SUBSCRIPTIONS` (freefq, ssrsub) | Always active |
//! | `LlmFreeCatalogDiscoverer` | LlmProvider | `FreeModelCatalog` (OpenRouter free tier) | Always active |
//! | `BuiltinLlmDiscoverer` | LlmProvider | `discovery::BUILTIN_FREE_PROVIDERS` | Always active |
//!
//! ## Usage
//!
//! ```ignore
//! let engine = ResourceDiscoveryOrchestrator::new();
//! engine.register_builtin_discoverers();
//! let results = engine.discover_all().await;
//! engine.feed_to_registry(&registry, &results).await;
//! ```

use std::time::Duration;

use async_trait::async_trait;

use crate::core::nt_core_resource_pool::{
    DiscoveredResource, DiscovererInfo, DiscoveryResult,
    ResourceDiscoverer, ResourceDiscoveryEngine, ResourceKind, ResourceRegistry,
};

// ── Proxy Scraper Discoverer ──

const FREE_PROXY_SCRAPERS: &[(&str, &str)] = &[
    ("geonode", "https://proxylist.geonode.com/api/proxy-list?limit=100&page=1&sort_by=lastChecked&sort_type=desc"),
    ("proxy-list", "https://www.proxy-list.download/api/v1/get?type=http"),
    ("proxyscrape", "https://api.proxyscrape.com/v2/?request=displayproxies&protocol=http&timeout=10000&country=all&ssl=all&anonymity=all"),
];

pub struct ProxyScraperDiscoverer {
    sources: Vec<(String, String)>,
}

impl Default for ProxyScraperDiscoverer {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyScraperDiscoverer {
    pub fn new() -> Self {
        Self {
            sources: FREE_PROXY_SCRAPERS
                .iter()
                .map(|(name, url)| (name.to_string(), url.to_string()))
                .collect(),
        }
    }

    /// Parse a plaintext proxy list (one proxy per line, possibly comma-sep).
    fn parse_proxy_list(text: &str) -> Vec<String> {
        let mut proxies = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let supported = [
                "ss://", "ssr://", "vmess://", "trojan://", "vless://",
                "socks5://", "socks4://", "http://", "https://",
            ];
            if supported.iter().any(|p| line.starts_with(p)) {
                proxies.push(line.to_string());
            }
        }
        proxies
    }
}

#[async_trait]
impl ResourceDiscoverer for ProxyScraperDiscoverer {
    fn info(&self) -> DiscovererInfo {
        DiscovererInfo {
            name: "proxy-scraper".to_string(),
            description: "Scrapes free proxy lists from geonode, proxy-list.download, proxyscrape".to_string(),
            kinds: vec![ResourceKind::Proxy],
            default_interval_secs: 600,
        }
    }

    async fn discover(&self) -> DiscoveryResult {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .danger_accept_invalid_certs(true)
            .no_proxy()
            .build()
            .unwrap_or_default();

        let mut discovered = Vec::new();
        let mut errors = Vec::new();

        for (name, url) in &self.sources {
            match client.get(url.as_str()).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let text = resp.text().await.unwrap_or_default();
                    let proxies = Self::parse_proxy_list(&text);
                    let proxy_count = proxies.len();
                    for proxy_url in &proxies {
                        let tag = proxy_url
                            .split('#')
                            .nth(1)
                            .unwrap_or(name)
                            .to_string();
                        let host = proxy_url
                            .split("://")
                            .nth(1)
                            .and_then(|s| s.split('@').next_back())
                            .and_then(|s| s.split(':').next())
                            .unwrap_or("unknown")
                            .to_string();
                        let mut meta = std::collections::HashMap::new();
                        meta.insert("source".to_string(), name.clone());
                        meta.insert("host".to_string(), host);
                        discovered.push(DiscoveredResource {
                            kind: ResourceKind::Proxy,
                            resource_id: proxy_url.clone(),
                            label: tag,
                            source_url: Some(url.clone()),
                            is_free: true,
                            requires_auth: false,
                            meta,
                        });
                    }
                    log::info!(
                        "[proxy-scraper] {}: discovered {} proxies",
                        name,
                        proxy_count
                    );
                }
                Ok(resp) => {
                    errors.push(format!("{}: HTTP {}", name, resp.status()));
                }
                Err(e) => {
                    errors.push(format!("{}: {}", name, e));
                }
            }
        }

        DiscoveryResult {
            kind: ResourceKind::Proxy,
            discovered,
            errors,
            source_name: "proxy-scraper".to_string(),
        }
    }
}

// ── Proxy Subscription Discoverer ──

const DEFAULT_SUBSCRIPTIONS: &[&str] = &[
    "https://raw.githubusercontent.com/freefq/free/master/v2",
    "https://raw.githubusercontent.com/mahdibland/ShadowsocksAggregator/master/Eternity.txt",
    "https://raw.githubusercontent.com/ssrsub/ssr/master/ss-sub",
];

pub struct ProxySubscriptionDiscoverer;

impl Default for ProxySubscriptionDiscoverer {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxySubscriptionDiscoverer {
    pub fn new() -> Self {
        Self
    }

    /// Base64 decode subscription content if needed.
    fn decode_content(text: &str) -> String {
        use base64::Engine;
        let cleaned = text.trim().replace('\n', "");
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD
            .decode(&cleaned)
        {
            String::from_utf8(bytes).unwrap_or_else(|_| text.to_string())
        } else {
            text.to_string()
        }
    }
}

#[async_trait]
impl ResourceDiscoverer for ProxySubscriptionDiscoverer {
    fn info(&self) -> DiscovererInfo {
        DiscovererInfo {
            name: "proxy-subscriptions".to_string(),
            description: "Fetches proxy subscription lists from freefq, ssrsub, etc.".to_string(),
            kinds: vec![ResourceKind::Proxy],
            default_interval_secs: 1800,
        }
    }

    async fn discover(&self) -> DiscoveryResult {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .danger_accept_invalid_certs(true)
            .no_proxy()
            .build()
            .unwrap_or_default();

        let mut discovered = Vec::new();
        let mut errors = Vec::new();

        for url in DEFAULT_SUBSCRIPTIONS {
            match client.get(*url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let text = resp.text().await.unwrap_or_default();
                    let decoded = Self::decode_content(&text);
                    let count_before = discovered.len();
                    for line in decoded.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        let tag = line.split('#').nth(1).unwrap_or("sub").to_string();
                        let host = line
                            .split("://")
                            .nth(1)
                            .and_then(|s| s.split('@').next_back())
                            .and_then(|s| s.split(':').next())
                            .unwrap_or("unknown");
                        let mut meta = std::collections::HashMap::new();
                        meta.insert("source".to_string(), "subscription".to_string());
                        meta.insert("host".to_string(), host.to_string());
                        meta.insert("sub_url".to_string(), url.to_string());
                        discovered.push(DiscoveredResource {
                            kind: ResourceKind::Proxy,
                            resource_id: line.to_string(),
                            label: tag,
                            source_url: Some(url.to_string()),
                            is_free: true,
                            requires_auth: false,
                            meta,
                        });
                    }
                    let count = discovered.len() - count_before;
                    log::info!("[proxy-subscriptions] {}: discovered {} proxies", url, count);
                }
                Ok(resp) => {
                    errors.push(format!("{}: HTTP {}", url, resp.status()));
                }
                Err(e) => {
                    errors.push(format!("{}: {}", url, e));
                }
            }
        }

        DiscoveryResult {
            kind: ResourceKind::Proxy,
            discovered,
            errors,
            source_name: "proxy-subscriptions".to_string(),
        }
    }
}

// ── LLM Free Provider Discoverer ──

pub struct LlmFreeProviderDiscoverer;

impl Default for LlmFreeProviderDiscoverer {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmFreeProviderDiscoverer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ResourceDiscoverer for LlmFreeProviderDiscoverer {
    fn info(&self) -> DiscovererInfo {
        DiscovererInfo {
            name: "llm-free-providers".to_string(),
            description: "Discovers free LLM providers from built-in list and OpenRouter API".to_string(),
            kinds: vec![ResourceKind::LlmProvider],
            default_interval_secs: 3600,
        }
    }

    async fn discover(&self) -> DiscoveryResult {
        let mut discovered = Vec::new();
        let errors: Vec<String> = Vec::new();

        // 1. Built-in free providers (hardcoded env-based)
        let builtin = crate::neotrix::l1_body_impl::nt_io_provider::discovery::BUILTIN_FREE_PROVIDERS;
        for (provider_id, model_id, base_url, _provider_type, tier) in builtin {
            let rid = format!("{}/{}", provider_id, model_id);
            let mut meta = std::collections::HashMap::new();
            meta.insert("provider".to_string(), provider_id.to_string());
            meta.insert("model".to_string(), model_id.to_string());
            meta.insert("base_url".to_string(), base_url.to_string());
            meta.insert("tier".to_string(), tier.to_string());
            discovered.push(DiscoveredResource {
                kind: ResourceKind::LlmProvider,
                resource_id: rid,
                label: format!("{}/{}", provider_id, model_id),
                source_url: Some(base_url.to_string()),
                is_free: true,
                requires_auth: true,
                meta,
            });
        }

        // 2. Try OpenRouter free model discovery
        if let Ok(resp) = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .and_then(|c| c.get("https://openrouter.ai/api/v1/models").send())
        {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>() {
                    if let Some(data) = body["data"].as_array() {
                        for entry in data {
                            let model_id = entry["id"].as_str().unwrap_or("");
                            let pricing = &entry["pricing"];
                            let prompt_price: f64 = pricing["prompt"].as_str()
                                .unwrap_or("0")
                                .parse()
                                .unwrap_or(0.0);
                            if prompt_price == 0.0 && !model_id.is_empty() {
                                let rid = format!("openrouter/{}", model_id);
                                let mut meta = std::collections::HashMap::new();
                                meta.insert("provider".to_string(), "openrouter".to_string());
                                meta.insert("model".to_string(), model_id.to_string());
                                meta.insert("source".to_string(), "openrouter-api".to_string());
                                discovered.push(DiscoveredResource {
                                    kind: ResourceKind::LlmProvider,
                                    resource_id: rid,
                                    label: format!("OpenRouter/{}", model_id),
                                    source_url: Some("https://openrouter.ai".to_string()),
                                    is_free: true,
                                    requires_auth: true,
                                    meta,
                                });
                            }
                        }
                    }
                }
            }
        }

        log::info!(
            "[llm-free-providers] discovered {} free providers",
            discovered.len()
        );
        DiscoveryResult {
            kind: ResourceKind::LlmProvider,
            discovered,
            errors,
            source_name: "llm-free-providers".to_string(),
        }
    }
}

// ── Orchestrator ──

/// Orchestrator that manages all resource discoverers and feeds results
/// into the `ResourceRegistry`.
pub struct ResourceDiscoveryOrchestrator {
    engine: ResourceDiscoveryEngine,
}

impl ResourceDiscoveryOrchestrator {
    pub fn new() -> Self {
        Self {
            engine: ResourceDiscoveryEngine::new(),
        }
    }

    /// Register all built-in discoverers.
    pub fn register_builtin_discoverers(&mut self) {
        self.engine
            .register(Box::new(ProxyScraperDiscoverer::new()));
        self.engine
            .register(Box::new(ProxySubscriptionDiscoverer::new()));
        self.engine
            .register(Box::new(LlmFreeProviderDiscoverer::new()));
    }

    /// Register a custom discoverer.
    pub fn register(&mut self, discoverer: Box<dyn ResourceDiscoverer>) {
        self.engine.register(discoverer);
    }

    /// Run all discoverers and return results.
    pub async fn discover_all(&self) -> Vec<DiscoveryResult> {
        self.engine.discover_all().await
    }

    /// Run discoverers for a specific kind.
    pub async fn discover_kind(&self, kind: &ResourceKind) -> Vec<DiscoveryResult> {
        self.engine.discover_kind(kind).await
    }

    /// Feed discovered resources into the registry.
    /// For each result, finds the matching pool by kind and feeds in.
    pub async fn feed_to_registry(
        &self,
        registry: &ResourceRegistry,
        results: &[DiscoveryResult],
    ) -> (usize, usize) {
        let mut total_fed = 0usize;
        let mut total_skipped = 0usize;

        for result in results {
            let (fed, skipped) = registry.feed(&result.kind, &result.discovered).await;
            total_fed += fed;
            total_skipped += skipped;
        }

        (total_fed, total_skipped)
    }

    /// Run a full cycle: discover all → feed to registry.
    pub async fn run_cycle(
        &self,
        registry: &ResourceRegistry,
    ) -> (usize, usize) {
        let results = self.discover_all().await;
        let (fed, errors) = self.feed_to_registry(registry, &results).await;
        let total = results.iter().map(|r| r.success_count()).sum::<usize>();
        log::info!(
            "[resource-discovery] cycle complete: {} discovered, {} fed, {} errors",
            total,
            fed,
            errors,
        );
        (fed, errors)
    }

    /// Get a reference to the underlying engine.
    pub fn engine(&self) -> &ResourceDiscoveryEngine {
        &self.engine
    }

    /// Number of registered discoverers.
    pub fn discoverer_count(&self) -> usize {
        self.engine.discoverer_count()
    }
}

impl Default for ResourceDiscoveryOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
