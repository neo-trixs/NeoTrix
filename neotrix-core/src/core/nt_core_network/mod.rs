//! # NeoTrix Network Awareness Layer (v10 W1)
//!
//! TLS fingerprinting + proxy rotation + unified HTTP client factory.
//! Combines existing nt_io_network (sensor/dns_cache) with new TLS and proxy modules.

// ── Re-export legacy nt_io_network (backward compat) ──
pub use crate::neotrix::nt_io_network::dns_cache;
pub use crate::neotrix::nt_io_network::sensor;
pub use crate::neotrix::nt_io_network::*;

// ── New sub-modules ──
pub mod proxy_rotation;
pub mod tls_fingerprint;

use std::sync::Arc;
use std::time::Duration;

pub use proxy_rotation::{ProxyConfig, ProxyPool, ProxyRotator, ProxyProtocol, RotationStrategy};
pub use tls_fingerprint::{TlsFingerprintConfig, TlsFingerprintProfile, HttpVersionPref};

/// HTTP method abstraction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
}

/// Result from any HTTP request
#[derive(Debug, Clone)]
pub struct NetworkResponse {
    pub status_code: u16,
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
    pub latency_ms: u64,
    pub url: String,
    pub proxy_used: Option<String>,
    pub fingerprint_used: String,
}

impl NetworkResponse {
    pub fn text(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, String> {
        serde_json::from_slice(&self.body).map_err(|e| format!("json parse: {}", e))
    }

    pub fn status_is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
}

/// Combined network client with TLS fingerprinting + proxy rotation
pub struct NetworkClient {
    tls_config: TlsFingerprintConfig,
    proxy_rotator: Option<Arc<ProxyRotator>>,
    client: reqwest::Client,
    extra_headers: Vec<(String, String)>,
    request_timeout: Duration,
    max_retries: u32,
}

impl NetworkClient {
    pub fn builder() -> NetworkClientBuilder {
        NetworkClientBuilder::new()
    }

    pub async fn get(&self, url: &str) -> Result<NetworkResponse, String> {
        self.request(HttpMethod::Get, url, None, None).await
    }

    pub async fn get_with_headers(
        &self,
        url: &str,
        headers: Vec<(String, String)>,
    ) -> Result<NetworkResponse, String> {
        self.request(HttpMethod::Get, url, None, Some(headers)).await
    }

    pub async fn post(
        &self,
        url: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<NetworkResponse, String> {
        self.request(HttpMethod::Post, url, Some((body, content_type)), None)
            .await
    }

    async fn request(
        &self,
        method: HttpMethod,
        url: &str,
        body: Option<(Vec<u8>, &str)>,
        extra_headers: Option<Vec<(String, String)>>,
    ) -> Result<NetworkResponse, String> {
        let start = std::time::Instant::now();
        let proxy_used;

        let client = if let Some(ref rotator) = self.proxy_rotator {
            let proxy = rotator.next().await;
            match proxy {
                Some(ref p) => {
                    proxy_used = Some(p.label.clone());
                    let rp = p.to_reqwest_proxy()?;
                    self.tls_config
                        .build_reqwest_client(Some(rp))?
                }
                None => {
                    proxy_used = None;
                    self.client.clone()
                }
            }
        } else {
            proxy_used = None;
            self.client.clone()
        };

        let mut req = client.request(
            match method {
                HttpMethod::Get => reqwest::Method::GET,
                HttpMethod::Post => reqwest::Method::POST,
                HttpMethod::Put => reqwest::Method::PUT,
                HttpMethod::Delete => reqwest::Method::DELETE,
                HttpMethod::Head => reqwest::Method::HEAD,
            },
            url,
        );

        if let Some((ref b, ct)) = body {
            req = req.header("content-type", ct).body(b.clone());
        }

        for (k, v) in &self.extra_headers {
            req = req.header(k.as_str(), v.as_str());
        }
        if let Some(ref headers) = extra_headers {
            for (k, v) in headers {
                req = req.header(k.as_str(), v.as_str());
            }
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("{} request failed: {}", url, e))?;

        let status_code = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
            .collect();
        let body_bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("read body failed: {}", e))?
            .to_vec();
        let latency_ms = start.elapsed().as_millis() as u64;

        if let Some(ref rotator) = self.proxy_rotator {
            if status_code < 500 {
                if let Some(ref p) = proxy_used {
                    rotator.record_success(p).await;
                }
            } else {
                if let Some(ref p) = proxy_used {
                    rotator.record_failure(p).await;
                }
            }
        }

        Ok(NetworkResponse {
            status_code,
            body: body_bytes,
            headers,
            latency_ms,
            url: url.to_string(),
            proxy_used,
            fingerprint_used: self.tls_config.profile.to_string(),
        })
    }

    pub fn tls_config(&self) -> &TlsFingerprintConfig {
        &self.tls_config
    }
}

pub struct NetworkClientBuilder {
    tls_config: TlsFingerprintConfig,
    proxy_rotator: Option<Arc<ProxyRotator>>,
    extra_headers: Vec<(String, String)>,
    request_timeout: Duration,
    connect_timeout: Duration,
    max_retries: u32,
    accept_invalid_certs: bool,
}

impl Default for NetworkClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkClientBuilder {
    pub fn new() -> Self {
        Self {
            tls_config: TlsFingerprintConfig::default(),
            proxy_rotator: None,
            extra_headers: Vec::new(),
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_retries: 3,
            accept_invalid_certs: false,
        }
    }

    pub fn with_tls_profile(mut self, profile: TlsFingerprintProfile) -> Self {
        self.tls_config = profile.default_config();
        self
    }

    pub fn with_tls_config(mut self, config: TlsFingerprintConfig) -> Self {
        self.tls_config = config;
        self
    }

    pub fn with_proxy_rotator(mut self, rotator: Arc<ProxyRotator>) -> Self {
        self.proxy_rotator = Some(rotator);
        self
    }

    pub fn with_proxy_pool(mut self, pool: ProxyPool, strategy: RotationStrategy) -> Self {
        self.proxy_rotator = Some(Arc::new(ProxyRotator::new(pool, strategy)));
        self
    }

    pub fn with_single_proxy(mut self, proxy: ProxyConfig) -> Self {
        let mut pool = ProxyPool::new(1);
        let _ = pool.add(proxy);
        self.proxy_rotator = Some(Arc::new(ProxyRotator::new(pool, RotationStrategy::RoundRobin)));
        self
    }

    pub fn with_extra_header(mut self, key: &str, value: &str) -> Self {
        self.extra_headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_extra_headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.extra_headers = headers;
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.request_timeout = Duration::from_secs(secs);
        self
    }

    pub fn with_connect_timeout(mut self, secs: u64) -> Self {
        self.connect_timeout = Duration::from_secs(secs);
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn with_accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    pub fn build(self) -> Result<NetworkClient, String> {
        let mut tls_cfg = self.tls_config;
        tls_cfg.accept_invalid_certs = self.accept_invalid_certs;
        tls_cfg.connect_timeout_secs = self.connect_timeout.as_secs();
        tls_cfg.request_timeout_secs = self.request_timeout.as_secs();

        let client = tls_cfg.build_reqwest_client(None)?;

        Ok(NetworkClient {
            tls_config: tls_cfg,
            proxy_rotator: self.proxy_rotator,
            client,
            extra_headers: self.extra_headers,
            request_timeout: self.request_timeout,
            max_retries: self.max_retries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = NetworkClientBuilder::new();
        assert_eq!(builder.max_retries, 3);
        assert_eq!(builder.tls_config.profile, TlsFingerprintProfile::Chrome);
    }

    #[test]
    fn test_builder_with_tls_profile() {
        let builder = NetworkClientBuilder::new().with_tls_profile(TlsFingerprintProfile::Firefox);
        assert_eq!(builder.tls_config.profile, TlsFingerprintProfile::Firefox);
    }

    #[test]
    fn test_builder_builds_client() {
        let client = NetworkClientBuilder::new()
            .with_timeout(5)
            .build();
        assert!(client.is_ok());
    }

    #[test]
    fn test_builder_with_extra_headers() {
        let client = NetworkClientBuilder::new()
            .with_extra_header("X-Custom", "test")
            .build()
            .unwrap();
        assert!(client.extra_headers.contains(&("X-Custom".into(), "test".into())));
    }

    #[test]
    fn test_network_response_helpers() {
        let resp = NetworkResponse {
            status_code: 200,
            body: b"hello".to_vec(),
            headers: vec![],
            latency_ms: 10,
            url: "https://example.com".into(),
            proxy_used: None,
            fingerprint_used: "Chrome".into(),
        };
        assert!(resp.status_is_success());
        assert_eq!(resp.text(), Ok("hello"));
    }

    #[test]
    fn test_network_response_status_failure() {
        let resp = NetworkResponse {
            status_code: 404,
            body: vec![],
            headers: vec![],
            latency_ms: 5,
            url: "https://example.com".into(),
            proxy_used: None,
            fingerprint_used: "Chrome".into(),
        };
        assert!(!resp.status_is_success());
    }

    #[test]
    fn test_builder_with_timeout() {
        let builder = NetworkClientBuilder::new().with_timeout(15);
        assert_eq!(builder.request_timeout, Duration::from_secs(15));
    }

    #[test]
    fn test_builder_with_connect_timeout() {
        let builder = NetworkClientBuilder::new().with_connect_timeout(5);
        assert_eq!(builder.connect_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_builder_with_proxy_pool() {
        let mut pool = ProxyPool::new(5);
        let _ = pool.add(ProxyConfig::new("127.0.0.1", 8080, ProxyProtocol::Http));
        let builder = NetworkClientBuilder::new()
            .with_proxy_pool(pool, RotationStrategy::RoundRobin);
        assert!(builder.proxy_rotator.is_some());
    }
}
