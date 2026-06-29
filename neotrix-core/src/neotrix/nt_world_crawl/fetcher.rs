use log;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use reqwest::blocking::Client;

use crate::core::nt_core_agent::UserAgentRotation;
use crate::neotrix::nt_world_scrape::{RequestScraper, ScraperConfig};

use super::config::CrawlStrategy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetcherProtocol {
    Http,
    Tor,
    Browser,
}

impl FetcherProtocol {
    pub fn name(&self) -> &'static str {
        match self {
            FetcherProtocol::Http => "http",
            FetcherProtocol::Tor => "tor",
            FetcherProtocol::Browser => "nt_world_browse",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchError {
    pub url: String,
    pub protocol: FetcherProtocol,
    pub status_code: u16,
    pub message: String,
    pub duration_ms: u64,
    pub retries: u32,
}

#[derive(Debug, Clone)]
pub struct FetchResult {
    pub url: String,
    pub status_code: u16,
    pub body: Option<String>,
    pub text: Option<String>,
    pub content_length: usize,
    pub duration_ms: u64,
    pub protocol: FetcherProtocol,
    pub error: Option<FetchError>,
}

impl FetchResult {
    pub fn is_success(&self) -> bool {
        self.error.is_none() && self.status_code == 200
    }

    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status_code)
    }

    pub fn is_ratelimited(&self) -> bool {
        self.status_code == 429
    }

    pub fn is_blocked(&self) -> bool {
        self.status_code == 403 || self.status_code == 401
    }

    pub fn is_error_status(&self) -> bool {
        self.status_code >= 400
    }

    pub fn text_snippet(&self) -> &str {
        self.text.as_deref().unwrap_or("")
    }
}

pub struct FetcherPool {
    http_nt_world_scrape: RequestScraper,
    strategy: CrawlStrategy,
    nt_world_browse_client: Option<Client>,
    errors: Vec<FetchError>,
    total_requests: u64,
    total_success: u64,
    total_timeouts: u64,
    total_blocked: u64,
    total_bytes: u64,
    last_network_check: Option<Instant>,
    network_available: bool,
    network_check_interval_secs: u64,
}

impl FetcherPool {
    const MAX_ERRORS: usize = 10000;

    pub fn new(config: &ScraperConfig, strategy: CrawlStrategy) -> Self {
        FetcherPool {
            http_nt_world_scrape: RequestScraper::new(config.clone()),
            strategy,
            nt_world_browse_client: None,
            errors: Vec::new(),
            total_requests: 0,
            total_success: 0,
            total_timeouts: 0,
            total_blocked: 0,
            total_bytes: 0,
            last_network_check: None,
            network_available: true,
            network_check_interval_secs: 30,
        }
    }

    pub fn fetch(&mut self, url: &str) -> FetchResult {
        self.total_requests += 1;
        let start = Instant::now();

        let scrape = self.http_nt_world_scrape.get(url);
        let duration_ms = start.elapsed().as_millis() as u64;

        let fetch_result = if let Some(err) = scrape.error {
            let fetch_err = FetchError {
                url: url.to_string(),
                protocol: FetcherProtocol::Http,
                status_code: scrape.status_code,
                message: err,
                duration_ms,
                retries: 0,
            };
            self.errors.push(fetch_err.clone());
            if self.errors.len() > Self::MAX_ERRORS {
                self.errors.drain(0..Self::MAX_ERRORS / 5);
            }
            if duration_ms > self.strategy.delay_ms() * 2 {
                self.total_timeouts += 1;
            }
            if scrape.status_code == 403 || scrape.status_code == 401 {
                self.total_blocked += 1;
            }
            FetchResult {
                url: url.to_string(),
                status_code: scrape.status_code,
                body: scrape.html,
                text: scrape.text,
                content_length: 0,
                duration_ms,
                protocol: FetcherProtocol::Http,
                error: Some(fetch_err),
            }
        } else {
            let text_len = scrape.text.as_ref().map_or(0, |t| t.len());
            self.total_success += 1;
            self.total_bytes += text_len as u64;
            FetchResult {
                url: url.to_string(),
                status_code: scrape.status_code,
                body: scrape.html,
                text: scrape.text,
                content_length: text_len,
                duration_ms,
                protocol: FetcherProtocol::Http,
                error: None,
            }
        };

        if duration_ms < self.strategy.delay_ms() {
            tokio::runtime::Handle::current().block_on(tokio::time::sleep(Duration::from_millis(
                self.strategy.delay_ms() - duration_ms,
            )));
        }

        fetch_result
    }

    pub fn fetch_with_retry(&mut self, url: &str, max_retries: u32) -> FetchResult {
        let mut result = self.fetch(url);
        let mut retries = 0;

        while result.is_error_status() && retries < max_retries {
            retries += 1;
            let backoff = Duration::from_secs(2u64.pow(retries));
            tokio::runtime::Handle::current().block_on(tokio::time::sleep(backoff));
            result = self.fetch(url);
            if let Some(ref mut err) = result.error {
                err.retries = retries;
            }
        }

        result
    }

    pub fn fetch_tor_safe(&mut self, url: &str) -> FetchResult {
        let mut result = self.fetch(url);
        if result.is_blocked() {
            tokio::runtime::Handle::current().block_on(tokio::time::sleep(Duration::from_secs(5)));
            result = self.fetch(url);
        }
        result
    }

    fn ensure_nt_world_browse_client(&mut self) -> &Client {
        self.nt_world_browse_client.get_or_insert_with(|| {
            Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent(UserAgentRotation::default().next())
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(reqwest::header::ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".parse().expect("Header::from_str failed: ACCEPT - should be valid ASCII"));
                    headers.insert(reqwest::header::ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().expect("Header::from_str failed: ACCEPT_LANGUAGE - should be valid ASCII"));
                    headers.insert(reqwest::header::DNT, "1".parse().expect("Header::from_str failed: DNT - should be valid ASCII"));
                    headers.insert(reqwest::header::CONNECTION, "keep-alive".parse().expect("Header::from_str failed: CONNECTION - should be valid ASCII"));
                    headers.insert(reqwest::header::UPGRADE_INSECURE_REQUESTS, "1".parse().expect("Header::from_str failed: UPGRADE_INSECURE_REQUESTS - should be valid ASCII"));
                    headers
                })
                .build()
                .expect("Failed to build reqwest blocking client")
        })
    }

    pub fn fetch_nt_world_browse_mode(&mut self, url: &str) -> FetchResult {
        self.total_requests += 1;
        let start = Instant::now();
        let client = self.ensure_nt_world_browse_client();

        let response = client.get(url).header(reqwest::header::REFERER, url).send();

        let duration_ms = start.elapsed().as_millis() as u64;

        let fetch_result = match response {
            Ok(resp) => {
                let status_code = resp.status().as_u16();
                let body_text = resp.text().unwrap_or_default();
                let text_len = body_text.len();

                if status_code >= 400 {
                    let fetch_err = FetchError {
                        url: url.to_string(),
                        protocol: FetcherProtocol::Browser,
                        status_code,
                        message: format!("HTTP {}", status_code),
                        duration_ms,
                        retries: 0,
                    };
                    self.errors.push(fetch_err.clone());
                    if self.errors.len() > Self::MAX_ERRORS {
                        self.errors.drain(0..Self::MAX_ERRORS / 5);
                    }
                    if duration_ms > self.strategy.delay_ms() * 2 {
                        self.total_timeouts += 1;
                    }
                    if status_code == 403 || status_code == 401 {
                        self.total_blocked += 1;
                    }
                    FetchResult {
                        url: url.to_string(),
                        status_code,
                        body: None,
                        text: Some(body_text),
                        content_length: 0,
                        duration_ms,
                        protocol: FetcherProtocol::Browser,
                        error: Some(fetch_err),
                    }
                } else {
                    self.total_success += 1;
                    self.total_bytes += text_len as u64;
                    FetchResult {
                        url: url.to_string(),
                        status_code,
                        body: Some(body_text.clone()),
                        text: Some(body_text),
                        content_length: text_len,
                        duration_ms,
                        protocol: FetcherProtocol::Browser,
                        error: None,
                    }
                }
            }
            Err(e) => {
                let status_code = e.status().map_or(0, |s| s.as_u16());
                let fetch_err = FetchError {
                    url: url.to_string(),
                    protocol: FetcherProtocol::Browser,
                    status_code,
                    message: e.to_string(),
                    duration_ms,
                    retries: 0,
                };
                self.errors.push(fetch_err.clone());
                if self.errors.len() > Self::MAX_ERRORS {
                    self.errors.drain(0..Self::MAX_ERRORS / 5);
                }
                if duration_ms > self.strategy.delay_ms() * 2 {
                    self.total_timeouts += 1;
                }
                FetchResult {
                    url: url.to_string(),
                    status_code,
                    body: None,
                    text: None,
                    content_length: 0,
                    duration_ms,
                    protocol: FetcherProtocol::Browser,
                    error: Some(fetch_err),
                }
            }
        };

        if duration_ms < self.strategy.delay_ms() {
            tokio::runtime::Handle::current().block_on(tokio::time::sleep(Duration::from_millis(
                self.strategy.delay_ms() - duration_ms,
            )));
        }

        fetch_result
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.total_requests - self.total_success) as f64 / self.total_requests as f64
    }

    pub fn summary(&self) -> FetcherSummary {
        FetcherSummary {
            total_requests: self.total_requests,
            total_success: self.total_success,
            total_timeouts: self.total_timeouts,
            total_blocked: self.total_blocked,
            total_bytes: self.total_bytes,
            error_rate: self.error_rate(),
            recent_errors: self.errors.iter().rev().take(10).cloned().collect(),
        }
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn adjust_strategy(&mut self, new_strategy: CrawlStrategy) {
        self.strategy = new_strategy;
    }

    /// 快速网络连通性检测: TCP connect to common hosts
    /// 超时 2s, 缓存 30s. 用于 nt_world_crawl 避免傻等 15s timeout
    pub fn check_connectivity(&mut self) -> bool {
        if let Some(last) = self.last_network_check {
            if last.elapsed().as_secs() < self.network_check_interval_secs {
                return self.network_available;
            }
        }
        let hosts = &["8.8.8.8:53", "1.1.1.1:53", "208.67.222.222:53"];
        let mut available = false;
        for host in hosts {
            if let Ok(mut addr) = host.to_socket_addrs() {
                if let Some(addr) = addr.find(|a| a.is_ipv4()) {
                    if TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok() {
                        available = true;
                        break;
                    }
                }
            }
        }
        self.last_network_check = Some(Instant::now());
        self.network_available = available;
        if !available {
            log::warn!("[fetcher] ⛔ 无网络连接, 进入离线模式");
        }
        available
    }

    pub fn is_network_available(&self) -> bool {
        self.network_available
    }
}

pub struct FetcherSummary {
    pub total_requests: u64,
    pub total_success: u64,
    pub total_timeouts: u64,
    pub total_blocked: u64,
    pub total_bytes: u64,
    pub error_rate: f64,
    pub recent_errors: Vec<FetchError>,
}

impl std::fmt::Display for FetcherSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Fetcher: req={} ok={} timeout={} blocked={} bytes={} err_rate={:.2}%",
            self.total_requests,
            self.total_success,
            self.total_timeouts,
            self.total_blocked,
            self.total_bytes,
            self.error_rate * 100.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_world_scrape::ScraperConfig;

    fn test_config() -> ScraperConfig {
        ScraperConfig {
            proxy: None,
            headless: true,
            block_images: true,
            user_agent: Some("NeoTrixTest/1.0".into()),
            timeout_secs: 5,
            max_retries: 0,
            profile_name: None,
            use_tiny_profile: false,
        }
    }

    #[test]
    fn test_fetcher_pool_creation() {
        let pool = FetcherPool::new(&test_config(), CrawlStrategy::Polite);
        assert_eq!(pool.total_requests, 0);
        assert_eq!(pool.total_success, 0);
    }

    #[test]
    fn test_fetch_result_methods() {
        let ok = FetchResult {
            url: "https://example.com".into(),
            status_code: 200,
            body: Some("<html>ok</html>".into()),
            text: Some("ok".into()),
            content_length: 2,
            duration_ms: 100,
            protocol: FetcherProtocol::Http,
            error: None,
        };
        assert!(ok.is_success());
        assert!(!ok.is_error_status());
        assert_eq!(ok.text_snippet(), "ok");

        let blocked = FetchResult {
            status_code: 403,
            ..ok.clone()
        };
        assert!(blocked.is_blocked());
        assert!(!blocked.is_success());

        let ratelimited = FetchResult {
            status_code: 429,
            ..ok
        };
        assert!(ratelimited.is_ratelimited());
    }

    #[test]
    fn test_error_rate() {
        let mut pool = FetcherPool::new(&test_config(), CrawlStrategy::Balanced);
        assert_eq!(pool.error_rate(), 0.0);
        pool.total_requests = 100;
        pool.total_success = 85;
        assert!((pool.error_rate() - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_fetch_strategy_delays() {
        assert_eq!(CrawlStrategy::Polite.delay_ms(), 2000);
        assert_eq!(CrawlStrategy::Balanced.delay_ms(), 500);
        assert_eq!(CrawlStrategy::Aggressive.delay_ms(), 100);
    }

    #[test]
    fn test_fetcher_summary_display() {
        let mut pool = FetcherPool::new(&test_config(), CrawlStrategy::Polite);
        pool.total_requests = 50;
        pool.total_success = 48;
        pool.total_timeouts = 1;
        pool.total_blocked = 1;
        pool.total_bytes = 100000;
        let summary = pool.summary();
        let display = format!("{}", summary);
        assert!(display.contains("req=50"));
        assert!(display.contains("ok=48"));
        assert!(display.contains("timeout=1"));
    }
}
