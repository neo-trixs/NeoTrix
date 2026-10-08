//! Layered fetcher abstraction with anti-detection.

/// Fetcher backend type
#[derive(Debug, Clone)]
pub enum FetcherBackend {
    Http,
    Browser,
    Stealthy,
}

impl Default for FetcherBackend {
    fn default() -> Self {
        FetcherBackend::Http
    }
}

/// Configuration for a web fetch
#[derive(Debug, Clone, Default)]
pub struct FetchConfig {
    pub backend: FetcherBackend,
    pub timeout_ms: u64,
    pub max_retries: usize,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub respect_robots: bool,
}

/// A web fetcher with layered backend support
pub struct WebFetcher {
    config: FetchConfig,
}

impl WebFetcher {
    pub fn new(config: FetchConfig) -> Self { Self { config } }

    pub fn http() -> Self {
        Self::new(FetchConfig { backend: FetcherBackend::Http, timeout_ms: 30000, max_retries: 3, respect_robots: true })
    }

    pub fn stealthy() -> Self {
        Self::new(FetchConfig { backend: FetcherBackend::Stealthy, timeout_ms: 60000, max_retries: 3, respect_robots: true })
    }

    pub async fn fetch(&self, _url: &str) -> Result<String, String> {
        // Placeholder -- will be wired to actual HTTP/browser fetching
        Ok(String::new())
    }
}
