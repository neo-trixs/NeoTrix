use std::sync::{Arc, OnceLock};
use std::time::Duration;

use log;

/// TLS 变体 — 改变 JA3/h2 指纹
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TlsVariant {
    ModernH2,
    LegacyHttp11,
    StrictVerify,
    LegacyStrict,
}

impl TlsVariant {
    pub fn all() -> &'static [TlsVariant] {
        &[
            TlsVariant::ModernH2,
            TlsVariant::LegacyHttp11,
            TlsVariant::StrictVerify,
            TlsVariant::LegacyStrict,
        ]
    }
}

/// H2 SETTINGS 参数组合
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum H2SettingsProfile {
    ChromeDefault,
    FirefoxDefault,
    SafariDefault,
    EdgeDefault,
}

impl H2SettingsProfile {
    pub fn all() -> &'static [H2SettingsProfile] {
        &[
            H2SettingsProfile::ChromeDefault,
            H2SettingsProfile::FirefoxDefault,
            H2SettingsProfile::SafariDefault,
            H2SettingsProfile::EdgeDefault,
        ]
    }
}

/// 连接池配置
pub const POOL_MAX_IDLE_PER_HOST: usize = 32;
pub const POOL_IDLE_TIMEOUT_SECS: u64 = 90;
pub const TCP_KEEPALIVE_SECS: u64 = 15;
pub const CONNECT_TIMEOUT_SECS: u64 = 10;
pub const REQUEST_TIMEOUT_SECS: u64 = 60;

/// 全局异步 HTTP 客户端（惰性初始化，自带连接池）
pub fn global_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| build_async_client_with_tls(TlsVariant::ModernH2, None))
}

/// 全局阻塞 HTTP 客户端
pub fn global_blocking_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
            .pool_idle_timeout(Duration::from_secs(POOL_IDLE_TIMEOUT_SECS))
            .tcp_keepalive(Duration::from_secs(TCP_KEEPALIVE_SECS))
            .build()
            .expect("global blocking reqwest Client build failed")
    })
}

/// 构建自定义异步客户端
pub fn build_async_client() -> reqwest::Client {
    build_async_client_with_tls(TlsVariant::ModernH2, None)
}

/// 按 TLS 变体 + 可选源地址构建异步客户端
pub fn build_async_client_with_tls(
    variant: TlsVariant,
    local_addr: Option<std::net::IpAddr>,
) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
        .pool_idle_timeout(Duration::from_secs(POOL_IDLE_TIMEOUT_SECS))
        .tcp_keepalive(Duration::from_secs(TCP_KEEPALIVE_SECS));

    match variant {
        TlsVariant::ModernH2 => {
            builder = builder.danger_accept_invalid_certs(true);
        }
        TlsVariant::LegacyHttp11 => {
            builder = builder.http1_only().danger_accept_invalid_certs(true);
        }
        TlsVariant::StrictVerify => {}
        TlsVariant::LegacyStrict => {
            builder = builder.http1_only();
        }
    }

    if let Some(addr) = local_addr {
        builder = builder.local_address(addr);
    }

    builder.build().unwrap_or_else(|_| global_client().clone())
}

/// 构建自定义阻塞客户端
pub fn build_blocking_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
        .pool_idle_timeout(Duration::from_secs(POOL_IDLE_TIMEOUT_SECS))
        .tcp_keepalive(Duration::from_secs(TCP_KEEPALIVE_SECS))
        .build()
        .unwrap_or_else(|_| global_blocking_client().clone())
}

// ============================================================================
// Browser TLS Fingerprints — used by both Simple and Stealth HTTP clients
// ============================================================================

/// Browser TLS fingerprints for HTTP client impersonation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsFingerprint {
    Chrome116,
    Chrome120,
    Firefox117,
    Firefox120,
    Safari17,
    Edge120,
    Custom(&'static str),
}

impl Default for TlsFingerprint {
    fn default() -> Self {
        Self::Chrome116
    }
}

impl TlsFingerprint {
    pub fn user_agent(&self) -> &'static str {
        match self {
            TlsFingerprint::Chrome116 => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36"
            }
            TlsFingerprint::Chrome120 => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
            }
            TlsFingerprint::Firefox117 => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/117.0"
            }
            TlsFingerprint::Firefox120 => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/120.0"
            }
            TlsFingerprint::Safari17 => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15"
            }
            TlsFingerprint::Edge120 => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0"
            }
            TlsFingerprint::Custom(_) => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
            }
        }
    }

    pub fn default_header_order(&self) -> Vec<&'static str> {
        match self {
            TlsFingerprint::Chrome116 | TlsFingerprint::Chrome120 | TlsFingerprint::Edge120 => {
                vec![
                    "host",
                    "connection",
                    "sec-ch-ua",
                    "sec-ch-ua-mobile",
                    "sec-ch-ua-platform",
                    "user-agent",
                    "accept",
                    "sec-fetch-site",
                    "sec-fetch-mode",
                    "sec-fetch-dest",
                    "accept-encoding",
                    "accept-language",
                    "cookie",
                ]
            }
            TlsFingerprint::Firefox117 | TlsFingerprint::Firefox120 => {
                vec![
                    "host",
                    "user-agent",
                    "accept",
                    "accept-language",
                    "accept-encoding",
                    "connection",
                    "cookie",
                    "upgrade-insecure-requests",
                    "sec-fetch-dest",
                    "sec-fetch-mode",
                    "sec-fetch-site",
                    "priority",
                ]
            }
            TlsFingerprint::Safari17 => {
                vec![
                    "host",
                    "user-agent",
                    "accept",
                    "accept-language",
                    "accept-encoding",
                    "connection",
                    "cookie",
                ]
            }
            TlsFingerprint::Custom(_) => {
                vec![
                    "host",
                    "user-agent",
                    "accept",
                    "accept-language",
                    "accept-encoding",
                    "connection",
                ]
            }
        }
    }

    /// Reorder headers to match the target browser's ordering
    pub fn apply_to_headers(&self, headers: &mut [(&str, String)]) {
        let order = self.default_header_order();
        headers.sort_by(|a, b| {
            let a_pos = order
                .iter()
                .position(|h| h.eq_ignore_ascii_case(a.0))
                .unwrap_or(usize::MAX);
            let b_pos = order
                .iter()
                .position(|h| h.eq_ignore_ascii_case(b.0))
                .unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos)
        });
    }
}

// ============================================================================
// Unified HTTP Client Factory
// ============================================================================

/// Select HTTP backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpClientBackend {
    /// Simple reqwest client (current default, may be detectable)
    Simple,
    /// Stealth client with TLS fingerprint + proxy rotation
    Stealth,
}

/// Unified HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub backend: HttpClientBackend,
    pub tls_fingerprint: TlsFingerprint,
    pub proxy_url: Option<String>,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub extra_headers: Vec<(String, String)>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            backend: HttpClientBackend::Simple,
            tls_fingerprint: TlsFingerprint::Chrome116,
            proxy_url: None,
            timeout_secs: 30,
            max_retries: 3,
            extra_headers: vec![],
        }
    }
}

/// Synchronous adapter: wraps an async HttpClient to provide blocking methods.
/// Uses `Handle::current().block_on()` — requires a running tokio runtime.
pub struct BlockingHttpClientAdapter {
    inner: Arc<dyn HttpClient>,
    config: HttpClientConfig,
}

impl std::fmt::Debug for BlockingHttpClientAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockingHttpClientAdapter")
            .field("config", &self.config)
            .field("inner_type", &"dyn HttpClient")
            .finish()
    }
}

impl Clone for BlockingHttpClientAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            config: self.config.clone(),
        }
    }
}

impl BlockingHttpClientAdapter {
    pub fn new(inner: Box<dyn HttpClient>, config: HttpClientConfig) -> Self {
        Self { inner: Arc::from(inner), config }
    }

    pub fn get_blocking(&self, url: &str) -> Result<HttpClientResult, String> {
        let config = self.config.clone();
        tokio::runtime::Handle::current().block_on(self.inner.get(url, &config))
    }

    pub fn post_blocking(&self, url: &str, body: &[u8], content_type: &str) -> Result<HttpClientResult, String> {
        let config = self.config.clone();
        tokio::runtime::Handle::current().block_on(
            self.inner.post(url, body, content_type, &config)
        )
    }
}

/// Create a blocking HTTP client that uses the specified backend.
pub fn create_blocking_http_client(config: HttpClientConfig) -> BlockingHttpClientAdapter {
    let inner = create_http_client(config.clone());
    BlockingHttpClientAdapter::new(inner, config)
}

/// Result from any HTTP client
#[derive(Debug)]
pub struct HttpClientResult {
    pub status_code: u16,
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
    pub latency_ms: u64,
    pub url: String,
}

/// The trait that unifies all HTTP backends
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str, config: &HttpClientConfig) -> Result<HttpClientResult, String>;
    async fn post(
        &self,
        url: &str,
        body: &[u8],
        content_type: &str,
        config: &HttpClientConfig,
    ) -> Result<HttpClientResult, String>;
    fn config(&self) -> &HttpClientConfig;
}

/// Factory to create appropriate HTTP client based on config
pub fn create_http_client(config: HttpClientConfig) -> Box<dyn HttpClient> {
    match config.backend {
        HttpClientBackend::Simple => Box::new(SimpleHttpClient::new(config)),
        #[cfg(feature = "stealth-net")]
        HttpClientBackend::Stealth => Box::new(StealthHttpClientImpl::new(config)),
        #[cfg(not(feature = "stealth-net"))]
        HttpClientBackend::Stealth => {
            log::warn!("Stealth backend requested but stealth-net feature disabled; falling back to Simple");
            Box::new(SimpleHttpClient::new(config))
        }
    }
}

// ============================================================================
// SimpleHttpClient — thin wrapper around reqwest::Client
// ============================================================================

pub struct SimpleHttpClient {
    config: HttpClientConfig,
    client: reqwest::Client,
}

impl SimpleHttpClient {
    pub fn new(config: HttpClientConfig) -> Self {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .user_agent(config.tls_fingerprint.user_agent());

        if let Some(ref proxy_url) = config.proxy_url {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                builder = builder.proxy(proxy);
            }
        }

        let client = builder.build().unwrap_or_else(|_| global_client().clone());
        Self { config, client }
    }

    async fn do_request(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&[u8]>,
        content_type: Option<&str>,
        config: &HttpClientConfig,
    ) -> Result<HttpClientResult, String> {
        let start = std::time::Instant::now();

        let mut req = self.client.request(method, url);

        if let Some(body) = body {
            req = req.body(body.to_vec());
        }
        if let Some(ct) = content_type {
            req = req.header("content-type", ct);
        }

        let mut header_vec: Vec<(&str, String)> = Vec::new();
        for (k, v) in &config.extra_headers {
            header_vec.push((k.as_str(), v.clone()));
        }
        config.tls_fingerprint.apply_to_headers(&mut header_vec);
        for (k, v) in &header_vec {
            req = req.header(*k, v.as_str());
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("{} failed: {}", url, e))?;
        let status_code = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
            .collect();
        let body = resp
            .bytes()
            .await
            .map_err(|e| format!("Read body failed: {}", e))?
            .to_vec();
        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(HttpClientResult {
            status_code,
            body,
            headers,
            latency_ms,
            url: url.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl HttpClient for SimpleHttpClient {
    async fn get(&self, url: &str, config: &HttpClientConfig) -> Result<HttpClientResult, String> {
        self.do_request(reqwest::Method::GET, url, None, None, config)
            .await
    }

    async fn post(
        &self,
        url: &str,
        body: &[u8],
        content_type: &str,
        config: &HttpClientConfig,
    ) -> Result<HttpClientResult, String> {
        self.do_request(
            reqwest::Method::POST,
            url,
            Some(body),
            Some(content_type),
            config,
        )
        .await
    }

    fn config(&self) -> &HttpClientConfig {
        &self.config
    }
}

// ============================================================================
// StealthHttpClientImpl — wraps the existing StealthHttpClient
// ============================================================================

#[cfg(feature = "stealth-net")]
pub struct StealthHttpClientImpl {
    config: HttpClientConfig,
    inner: std::sync::Arc<crate::neotrix::nt_io_stealth_net::http_client::StealthHttpClient>,
}

#[cfg(feature = "stealth-net")]
impl StealthHttpClientImpl {
    pub fn new(config: HttpClientConfig) -> Self {
        let fp = config.tls_fingerprint;
        let inner = std::sync::Arc::new(
            crate::neotrix::nt_io_stealth_net::http_client::StealthHttpClient::with_proxy(
                config.proxy_url.as_deref(),
            ),
        );
        Self { config, inner }
    }
}

#[cfg(feature = "stealth-net")]
#[async_trait::async_trait]
impl HttpClient for StealthHttpClientImpl {
    async fn get(&self, url: &str, _config: &HttpClientConfig) -> Result<HttpClientResult, String> {
        let start = std::time::Instant::now();
        let resp = self.inner.fetch(url).await?;
        let latency_ms = start.elapsed().as_millis() as u64;
        let headers: Vec<(String, String)> = resp.headers.into_iter().collect();
        Ok(HttpClientResult {
            status_code: resp.status,
            body: resp.body,
            headers,
            latency_ms,
            url: url.to_string(),
        })
    }

    async fn post(
        &self,
        url: &str,
        body: &[u8],
        content_type: &str,
        _config: &HttpClientConfig,
    ) -> Result<HttpClientResult, String> {
        let start = std::time::Instant::now();
        let resp = self.inner.post(url, body.to_vec(), content_type).await?;
        let latency_ms = start.elapsed().as_millis() as u64;
        let headers: Vec<(String, String)> = resp.headers.into_iter().collect();
        Ok(HttpClientResult {
            status_code: resp.status,
            body: resp.body,
            headers,
            latency_ms,
            url: url.to_string(),
        })
    }

    fn config(&self) -> &HttpClientConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_fingerprint_default() {
        assert_eq!(TlsFingerprint::default(), TlsFingerprint::Chrome116);
    }

    #[test]
    fn test_tls_fingerprint_user_agent_not_empty() {
        let cases = vec![
            TlsFingerprint::Chrome116,
            TlsFingerprint::Chrome120,
            TlsFingerprint::Firefox117,
            TlsFingerprint::Firefox120,
            TlsFingerprint::Safari17,
            TlsFingerprint::Edge120,
            TlsFingerprint::Custom("dummy"),
        ];
        for fp in cases {
            let ua = fp.user_agent();
            assert!(!ua.is_empty(), "UA for {:?} should not be empty", fp);
            assert!(ua.starts_with("Mozilla/"), "UA should start with Mozilla/");
        }
    }

    #[test]
    fn test_tls_fingerprint_chrome_header_order() {
        let order = TlsFingerprint::Chrome116.default_header_order();
        assert!(order.contains(&"sec-ch-ua"));
        assert!(order.contains(&"user-agent"));
        assert!(order.contains(&"accept"));
        // Firefox-specific headers should not appear in Chrome order
        assert!(!order.contains(&"priority"));
    }

    #[test]
    fn test_tls_fingerprint_firefox_header_order() {
        let order = TlsFingerprint::Firefox120.default_header_order();
        assert!(order.contains(&"priority"));
        assert!(!order.contains(&"sec-ch-ua"));
    }

    #[test]
    fn test_apply_to_headers_reorders() {
        let fp = TlsFingerprint::Chrome116;
        let mut headers = vec![
            ("accept", "text/html".into()),
            ("user-agent", "test".into()),
            ("host", "example.com".into()),
        ];
        fp.apply_to_headers(&mut headers);
        // host should now be first
        assert_eq!(headers[0].0, "host");
        // user-agent should come before accept
        let ua_pos = headers
            .iter()
            .position(|(k, _)| *k == "user-agent")
            .unwrap();
        let accept_pos = headers.iter().position(|(k, _)| *k == "accept").unwrap();
        assert!(ua_pos < accept_pos, "user-agent should come before accept");
    }

    #[test]
    fn test_http_client_config_default() {
        let cfg = HttpClientConfig::default();
        assert_eq!(cfg.backend, HttpClientBackend::Simple);
        assert_eq!(cfg.tls_fingerprint, TlsFingerprint::Chrome116);
        assert_eq!(cfg.timeout_secs, 30);
        assert_eq!(cfg.max_retries, 3);
        assert!(cfg.proxy_url.is_none());
        assert!(cfg.extra_headers.is_empty());
    }

    #[test]
    fn test_create_http_client_simple() {
        let config = HttpClientConfig::default();
        let client = create_http_client(config);
        assert_eq!(client.config().backend, HttpClientBackend::Simple);
    }

    #[test]
    fn test_http_client_result_construction() {
        let result = HttpClientResult {
            status_code: 200,
            body: b"hello".to_vec(),
            headers: vec![("content-type".into(), "text/plain".into())],
            latency_ms: 42,
            url: "https://example.com".into(),
        };
        assert_eq!(result.status_code, 200);
        assert_eq!(result.body, b"hello");
        assert_eq!(result.latency_ms, 42);
    }

    #[test]
    fn test_custom_fingerprint_fallback_ua() {
        let fp = TlsFingerprint::Custom("test");
        let ua = fp.user_agent();
        assert!(ua.contains("Chrome/120"));
    }

    #[test]
    fn test_safari_header_order_compact() {
        let order = TlsFingerprint::Safari17.default_header_order();
        // Safari should NOT have sec-ch-ua headers
        assert!(!order.contains(&"sec-ch-ua"));
        assert!(!order.contains(&"sec-fetch-site"));
    }

    #[test]
    fn test_edge_header_order_matches_chrome() {
        let chrome = TlsFingerprint::Chrome120.default_header_order();
        let edge = TlsFingerprint::Edge120.default_header_order();
        assert_eq!(chrome, edge, "Edge should have same header order as Chrome");
    }
}
