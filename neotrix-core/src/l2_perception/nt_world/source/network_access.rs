//! Blocked network access abstraction
//!
//! Unified interface for accessing networks that may be blocked/restricted:
//! corporate firewalls, government censorship, geo-restrictions.
//!
//! ## Strategy hierarchy
//!
//! ```text
//! NetworkAccessClient
//!   +-- Direct          -> reqwest::Client (no proxy)
//!   +-- HttpProxy       -> reqwest::Client + HTTP proxy
//!   +-- Socks5Proxy     -> reqwest::Client + SOCKS5 proxy (reqwest/socks feature)
//!   +-- Tor             -> SOCKS5 to 127.0.0.1:9050 (local Tor daemon)
//!   +-- Camofox         -> anti-detection browser via camofox CLI
//!   +-- Stealth         -> fingerprint rotation + proxy pool
//!   +-- Fallback(vec!)  -> try each strategy in order
//! ```

use std::time::Duration;

use reqwest::Client;

// ============================================================================
// NetworkStrategy
// ============================================================================

/// Network access strategy for blocked/restricted networks
#[derive(Debug, Clone)]
pub enum NetworkStrategy {
    /// Direct connection (no blocking)
    Direct,
    /// HTTP/HTTPS proxy (e.g. "http://proxy.corp:8080")
    HttpProxy(String),
    /// SOCKS5 proxy (e.g. "socks5://127.0.0.1:1080")
    Socks5Proxy(String),
    /// Tor network -- routes through local Tor daemon on 127.0.0.1:9050
    Tor,
    /// Anti-detection browser (Camofox)
    Camofox,
    /// Full stealth -- fingerprint spoofing + proxy rotation
    Stealth,
    /// Multiple strategies with automatic fallback
    Fallback(Vec<NetworkStrategy>),
}

// ============================================================================
// NetworkAccessClient
// ============================================================================

/// Unified network access client
pub struct NetworkAccessClient {
    strategy: NetworkStrategy,
    timeout: Duration,
    retry_count: u32,
}

impl NetworkAccessClient {
    /// Create a client with a specific strategy
    pub fn new(strategy: NetworkStrategy) -> Self {
        Self {
            strategy,
            timeout: Duration::from_secs(30),
            retry_count: 2,
        }
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set retry count
    pub fn with_retries(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    /// Direct connection (no blocking)
    pub fn direct() -> Self {
        Self::new(NetworkStrategy::Direct)
    }

    /// HTTP/HTTPS proxy
    pub fn with_proxy(proxy: &str) -> Self {
        Self::new(NetworkStrategy::HttpProxy(proxy.to_string()))
    }

    /// SOCKS5 proxy
    pub fn with_socks5(proxy: &str) -> Self {
        Self::new(NetworkStrategy::Socks5Proxy(proxy.to_string()))
    }

    /// Tor network (local daemon on 127.0.0.1:9050)
    pub fn with_tor() -> Self {
        Self::new(NetworkStrategy::Tor)
    }

    /// Anti-detection browser (Camofox)
    pub fn with_camofox() -> Self {
        Self::new(NetworkStrategy::Camofox)
    }

    /// Full stealth (fingerprint rotation + proxy rotation)
    pub fn with_stealth() -> Self {
        Self::new(NetworkStrategy::Stealth)
    }

    /// Fallback chain -- tries each strategy in order
    pub fn with_fallback(strategies: Vec<NetworkStrategy>) -> Self {
        Self::new(NetworkStrategy::Fallback(strategies))
    }

    /// Current strategy reference
    pub fn strategy(&self) -> &NetworkStrategy {
        &self.strategy
    }

    // -- HTTP verbs ----------------------------------------------------------

    /// GET request returning raw bytes
    pub async fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        self.execute(Method::Get, url, &[]).await
    }

    /// POST request returning raw bytes
    pub async fn post(&self, url: &str, body: &[u8]) -> Result<Vec<u8>, String> {
        self.execute(Method::Post, url, body).await
    }

    /// GET request returning text
    pub async fn get_text(&self, url: &str) -> Result<String, String> {
        let bytes = self.get(url).await?;
        String::from_utf8(bytes).map_err(|e| format!("utf8 decode: {e}"))
    }

    /// POST request returning text
    pub async fn post_text(&self, url: &str, body: &[u8]) -> Result<String, String> {
        let bytes = self.post(url, body).await?;
        String::from_utf8(bytes).map_err(|e| format!("utf8 decode: {e}"))
    }

    // -- Internal dispatch ---------------------------------------------------

    async fn execute(&self, method: Method, url: &str, body: &[u8]) -> Result<Vec<u8>, String> {
        let mut last_err = String::new();

        for attempt in 0..=self.retry_count {
            if attempt > 0 {
                let backoff = Duration::from_secs(2u64.saturating_pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }

            match self.try_strategy(method, url, body).await {
                Ok(bytes) => return Ok(bytes),
                Err(e) => {
                    last_err = e;
                }
            }
        }

        Err(last_err)
    }

    async fn try_strategy(
        &self,
        method: Method,
        url: &str,
        body: &[u8],
    ) -> Result<Vec<u8>, String> {
        match &self.strategy {
            NetworkStrategy::Direct => self.fetch_direct(method, url, body).await,
            NetworkStrategy::HttpProxy(proxy) => {
                self.fetch_with_proxy(method, url, body, proxy).await
            }
            NetworkStrategy::Socks5Proxy(proxy) => {
                self.fetch_with_socks5(method, url, body, proxy).await
            }
            NetworkStrategy::Tor => {
                self.fetch_with_socks5(method, url, body, "socks5://127.0.0.1:9050")
                    .await
            }
            NetworkStrategy::Camofox => self.fetch_camofox(method, url).await,
            NetworkStrategy::Stealth => self.fetch_stealth(method, url, body).await,
            NetworkStrategy::Fallback(strategies) => {
                self.fetch_fallback(method, url.to_string(), body.to_vec(), strategies.clone()).await
            }
        }
    }

    // -- Strategy implementations --------------------------------------------

    /// Direct connection via reqwest
    async fn fetch_direct(
        &self,
        method: Method,
        url: &str,
        body: &[u8],
    ) -> Result<Vec<u8>, String> {
        let client = build_client(None, self.timeout)?;
        execute_request(&client, method, url, body).await
    }

    /// HTTP/HTTPS proxy
    async fn fetch_with_proxy(
        &self,
        method: Method,
        url: &str,
        body: &[u8],
        proxy: &str,
    ) -> Result<Vec<u8>, String> {
        let proxy_obj = reqwest::Proxy::https(proxy)
            .or_else(|_| reqwest::Proxy::http(proxy))
            .map_err(|e| format!("invalid proxy {proxy}: {e}"))?;
        let client = build_client(Some(proxy_obj), self.timeout)?;
        execute_request(&client, method, url, body).await
    }

    /// SOCKS5 proxy (requires reqwest/socks feature -- enabled in Cargo.toml)
    async fn fetch_with_socks5(
        &self,
        method: Method,
        url: &str,
        body: &[u8],
        proxy: &str,
    ) -> Result<Vec<u8>, String> {
        let proxy_obj =
            reqwest::Proxy::all(proxy).map_err(|e| format!("invalid socks5 proxy {proxy}: {e}"))?;
        let client = build_client(Some(proxy_obj), self.timeout)?;
        execute_request(&client, method, url, body).await
    }

    /// Camofox anti-detection browser -- delegates to `camofox` CLI
    async fn fetch_camofox(&self, _method: Method, url: &str) -> Result<Vec<u8>, String> {
        use tokio::process::Command;

        let output = Command::new("camofox")
            .args(["fetch", url, "--format", "bytes"])
            .output()
            .await
            .map_err(|e| format!("camofox not available: {e}"))?;

        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(format!(
                "camofox fetch failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Full stealth -- rotates fingerprints per attempt
    async fn fetch_stealth(
        &self,
        method: Method,
        url: &str,
        body: &[u8],
    ) -> Result<Vec<u8>, String> {
        use crate::l2_perception::nt_world::crawl::stealth::Fingerprint;

        let fp = Fingerprint::random();
        let client = Client::builder()
            .timeout(self.timeout)
            .user_agent(&fp.user_agent)
            .build()
            .map_err(|e| format!("build stealth client: {e}"))?;

        let mut request = match method {
            Method::Get => client.get(url),
            Method::Post => client.post(url),
        };

        request = request.header("Accept-Language", &fp.language);
        request = request.header("X-Forwarded-For", "1.0.0.1");

        if !body.is_empty() {
            request = request.body(body.to_vec());
        }

        let resp = request
            .send()
            .await
            .map_err(|e| format!("stealth request: {e}"))?;

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("stealth read: {e}"))
    }

    /// Fallback -- tries each strategy in order, returns first success.
    /// Uses Box::pin to allow recursive async calls.
    fn fetch_fallback(
        &self,
        method: Method,
        url: String,
        body: Vec<u8>,
        strategies: Vec<NetworkStrategy>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send + '_>> {
        Box::pin(async move {
            let mut errors = Vec::new();
            for strategy in &strategies {
                let result = match strategy {
                    NetworkStrategy::Direct => self.fetch_direct(method, url.as_str(), &body).await,
                    NetworkStrategy::HttpProxy(proxy) => {
                        self.fetch_with_proxy(method, url.as_str(), &body, proxy).await
                    }
                    NetworkStrategy::Socks5Proxy(proxy) => {
                        self.fetch_with_socks5(method, url.as_str(), &body, proxy).await
                    }
                    NetworkStrategy::Tor => {
                        self.fetch_with_socks5(method, url.as_str(), &body, "socks5://127.0.0.1:9050")
                            .await
                    }
                    NetworkStrategy::Camofox => self.fetch_camofox(method, url.as_str()).await,
                    NetworkStrategy::Stealth => self.fetch_stealth(method, url.as_str(), &body).await,
                    NetworkStrategy::Fallback(inner) => {
                        self.fetch_fallback(method, url.clone(), body.clone(), inner.clone()).await
                    }
                };
                match result {
                    Ok(bytes) => return Ok(bytes),
                    Err(e) => errors.push(format!("[{:?}]: {}", strategy, e)),
                }
            }
            Err(format!(
                "all fallback strategies failed ({} attempts): {}",
                errors.len(),
                errors.join("; ")
            ))
        })
    }
}

// ============================================================================
// Subscription feed manager
// ============================================================================

/// Subscription feed -- a single URL source with refresh metadata
pub struct SubscriptionFeed {
    pub url: String,
    pub name: String,
    pub update_interval: Duration,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl SubscriptionFeed {
    pub fn new(url: &str, name: &str, update_interval: Duration) -> Self {
        Self {
            url: url.to_string(),
            name: name.to_string(),
            update_interval,
            last_updated: None,
        }
    }

    /// Whether this feed is due for a refresh
    pub fn is_stale(&self) -> bool {
        match self.last_updated {
            Some(last) => {
                let elapsed = chrono::Utc::now() - last;
                elapsed > chrono::Duration::from_std(self.update_interval).unwrap_or_default()
            }
            None => true,
        }
    }
}

/// Subscription feed manager -- manages multiple subscription sources
pub struct SubscriptionManager {
    client: NetworkAccessClient,
    feeds: Vec<SubscriptionFeed>,
}

impl SubscriptionManager {
    pub fn new(client: NetworkAccessClient) -> Self {
        Self {
            client,
            feeds: Vec::new(),
        }
    }

    /// Add a feed to track
    pub fn add_feed(&mut self, feed: SubscriptionFeed) {
        self.feeds.push(feed);
    }

    /// Number of registered feeds
    pub fn feed_count(&self) -> usize {
        self.feeds.len()
    }

    /// Fetch all stale feeds, returning (name, bytes) pairs
    pub async fn fetch_all(&self) -> Result<Vec<(String, Vec<u8>)>, String> {
        let mut results = Vec::new();
        for feed in &self.feeds {
            if feed.is_stale() {
                match self.client.get(&feed.url).await {
                    Ok(bytes) => results.push((feed.name.clone(), bytes)),
                    Err(e) => {
                        log::warn!("[subscription] fetch {} failed: {e}", feed.name);
                    }
                }
            }
        }
        Ok(results)
    }

    /// Fetch a single feed by URL
    pub async fn fetch_feed(&self, url: &str) -> Result<Vec<u8>, String> {
        self.client.get(url).await
    }

    /// Mark a feed as updated
    pub fn mark_updated(&mut self, name: &str) {
        if let Some(feed) = self.feeds.iter_mut().find(|f| f.name == name) {
            feed.last_updated = Some(chrono::Utc::now());
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Method {
    Get,
    Post,
}

fn build_client(proxy: Option<reqwest::Proxy>, timeout: Duration) -> Result<Client, String> {
    let mut builder = Client::builder().timeout(timeout);
    if let Some(p) = proxy {
        builder = builder.proxy(p);
    }
    builder
        .build()
        .map_err(|e| format!("build client: {e}"))
}

async fn execute_request(
    client: &Client,
    method: Method,
    url: &str,
    body: &[u8],
) -> Result<Vec<u8>, String> {
    let mut req = match method {
        Method::Get => client.get(url),
        Method::Post => client.post(url),
    };

    if !body.is_empty() {
        req = req.body(body.to_vec());
    }

    let resp = req.send().await.map_err(|e| format!("request {url}: {e}"))?;

    let status = resp.status();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("read response: {e}"))?
        .to_vec();

    if status.is_success() {
        Ok(bytes)
    } else {
        Err(format!(
            "HTTP {} from {url}: {}",
            status.as_u16(),
            String::from_utf8_lossy(&bytes)
                .chars()
                .take(200)
                .collect::<String>()
        ))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_construction() {
        let c = NetworkAccessClient::direct();
        assert!(matches!(c.strategy(), NetworkStrategy::Direct));
    }

    #[test]
    fn test_proxy_construction() {
        let c = NetworkAccessClient::with_proxy("http://proxy:8080");
        assert!(matches!(c.strategy(), NetworkStrategy::HttpProxy(_)));
    }

    #[test]
    fn test_socks5_construction() {
        let c = NetworkAccessClient::with_socks5("socks5://127.0.0.1:1080");
        assert!(matches!(c.strategy(), NetworkStrategy::Socks5Proxy(_)));
    }

    #[test]
    fn test_tor_construction() {
        let c = NetworkAccessClient::with_tor();
        assert!(matches!(c.strategy(), NetworkStrategy::Tor));
    }

    #[test]
    fn test_camofox_construction() {
        let c = NetworkAccessClient::with_camofox();
        assert!(matches!(c.strategy(), NetworkStrategy::Camofox));
    }

    #[test]
    fn test_stealth_construction() {
        let c = NetworkAccessClient::with_stealth();
        assert!(matches!(c.strategy(), NetworkStrategy::Stealth));
    }

    #[test]
    fn test_fallback_construction() {
        let c = NetworkAccessClient::with_fallback(vec![
            NetworkStrategy::Direct,
            NetworkStrategy::Tor,
        ]);
        assert!(matches!(c.strategy(), NetworkStrategy::Fallback(_)));
    }

    #[test]
    fn test_timeout_and_retry_builder() {
        let c = NetworkAccessClient::direct()
            .with_timeout(Duration::from_secs(60))
            .with_retries(5);
        assert_eq!(c.timeout, Duration::from_secs(60));
        assert_eq!(c.retry_count, 5);
    }

    #[test]
    fn test_subscription_feed_is_stale() {
        let mut feed = SubscriptionFeed::new("https://x.com/rss", "rss", Duration::from_secs(60));
        assert!(feed.is_stale(), "never-updated feed is stale");

        feed.last_updated = Some(chrono::Utc::now());
        assert!(!feed.is_stale(), "just-updated feed is not stale");
    }

    #[test]
    fn test_subscription_manager_add_feed() {
        let c = NetworkAccessClient::direct();
        let mut mgr = SubscriptionManager::new(c);
        mgr.add_feed(SubscriptionFeed::new("https://a.com", "a", Duration::from_secs(60)));
        mgr.add_feed(SubscriptionFeed::new("https://b.com", "b", Duration::from_secs(120)));
        assert_eq!(mgr.feed_count(), 2);
    }

    #[test]
    fn test_subscription_manager_mark_updated() {
        let c = NetworkAccessClient::direct();
        let mut mgr = SubscriptionManager::new(c);
        mgr.add_feed(SubscriptionFeed::new("https://a.com", "a", Duration::from_secs(60)));

        assert!(mgr.feeds[0].last_updated.is_none());
        mgr.mark_updated("a");
        assert!(mgr.feeds[0].last_updated.is_some());
    }

    #[test]
    fn test_network_strategy_debug_clone() {
        let s = NetworkStrategy::Fallback(vec![
            NetworkStrategy::Direct,
            NetworkStrategy::Tor,
        ]);
        let s2 = s.clone();
        let _ = format!("{:?}", s);
        let _ = format!("{:?}", s2);
    }
}
