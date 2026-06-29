use std::sync::Arc;

use rand::Rng;

use super::super::proxy_chain::DynamicProxyChain;
use crate::core::nt_core_agent::UserAgentRotation;

pub(super) const DEFAULT_TIMEOUT_SECS: u64 = 5;
pub(super) const MAX_REDIRECTS: usize = 10;
pub(super) const STEALTH_CONNECT_TIMEOUT_SECS: u64 = 3;
pub(super) const STEALTH_POOL_MAX_IDLE: usize = 16;
pub(super) const STEALTH_POOL_IDLE_TIMEOUT_SECS: u64 = 20;

pub(crate) use crate::core::nt_core_util::TOR_SOCKS_ADDR;

pub(super) const DOH_SERVERS: &[&str] = &[
    "https://cloudflare-dns.com/dns-query",
    "https://dns.google/dns-query",
    "https://dns.quad9.net/dns-query",
];

pub fn stealth_user_agent() -> &'static str {
    UserAgentRotation::default().next()
}

#[derive(Debug, Clone)]
pub enum ProxyConfig {
    None,
    Static(String),
    Tor,
    DynamicChain(Arc<DynamicProxyChain>),
}

#[derive(Debug, Clone)]
pub struct EntropyBudget {
    pub consumed: f64,
    pub limit: f64,
    pub fingerprint_rotations: u64,
    pub chain_rotations: u64,
    pub total_requests: u64,
    pub last_full_reset: std::time::Instant,
}

impl EntropyBudget {
    pub fn new(limit: f64) -> Self {
        Self {
            consumed: 0.0,
            limit,
            fingerprint_rotations: 0,
            chain_rotations: 0,
            total_requests: 0,
            last_full_reset: std::time::Instant::now(),
        }
    }

    pub fn record_fingerprint_rotation(&mut self) {
        self.fingerprint_rotations += 1;
        self.consumed += 0.5;
    }

    pub fn record_chain_rotation(&mut self) {
        self.chain_rotations += 1;
        self.consumed += 0.3;
    }

    pub fn record_request(&mut self) {
        self.total_requests += 1;
        self.consumed += 0.02;
    }

    pub fn is_exhausted(&self) -> bool {
        self.consumed >= self.limit
    }

    pub fn reset(&mut self) {
        self.consumed = 0.0;
        self.fingerprint_rotations = 0;
        self.chain_rotations = 0;
        self.total_requests = 0;
        self.last_full_reset = std::time::Instant::now();
    }
}

pub(super) fn proxy_url_string(config: &ProxyConfig) -> String {
    match config {
        ProxyConfig::Static(url) => url.clone(),
        ProxyConfig::Tor => format!("socks5://{}", TOR_SOCKS_ADDR),
        _ => String::new(),
    }
}

pub(super) fn proxy_config_from_url(url: &str) -> ProxyConfig {
    if url.is_empty() {
        ProxyConfig::None
    } else {
        ProxyConfig::Static(url.to_string())
    }
}

pub(super) fn gaussian_delay_ms(mean: f64, std: f64, min: u64, max: u64) -> u64 {
    let mut rng = rand::thread_rng();
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    let sample = mean + z * std;
    (sample.round() as i64).clamp(min as i64, max as i64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_budget_new() {
        let budget = EntropyBudget::new(10.0);
        assert_eq!(budget.consumed, 0.0);
        assert_eq!(budget.limit, 10.0);
        assert!(!budget.is_exhausted());
    }

    #[test]
    fn test_entropy_budget_record_rotation() {
        let mut budget = EntropyBudget::new(10.0);
        budget.record_fingerprint_rotation();
        assert_eq!(budget.fingerprint_rotations, 1);
        assert_eq!(budget.consumed, 0.5);
    }

    #[test]
    fn test_entropy_budget_record_chain() {
        let mut budget = EntropyBudget::new(10.0);
        budget.record_chain_rotation();
        assert_eq!(budget.chain_rotations, 1);
        assert_eq!(budget.consumed, 0.3);
    }

    #[test]
    fn test_entropy_budget_record_request() {
        let mut budget = EntropyBudget::new(10.0);
        budget.record_request();
        assert_eq!(budget.total_requests, 1);
        assert_eq!(budget.consumed, 0.02);
    }

    #[test]
    fn test_entropy_budget_is_exhausted() {
        let mut budget = EntropyBudget::new(1.0);
        budget.record_fingerprint_rotation();
        budget.record_fingerprint_rotation();
        assert!(budget.is_exhausted());
    }

    #[test]
    fn test_entropy_budget_reset() {
        let mut budget = EntropyBudget::new(10.0);
        budget.record_fingerprint_rotation();
        budget.record_chain_rotation();
        budget.record_request();
        budget.reset();
        assert_eq!(budget.consumed, 0.0);
        assert_eq!(budget.fingerprint_rotations, 0);
        assert_eq!(budget.chain_rotations, 0);
        assert_eq!(budget.total_requests, 0);
    }

    #[test]
    fn test_proxy_url_string_static() {
        let url = proxy_url_string(&ProxyConfig::Static("http://127.0.0.1:8080".into()));
        assert_eq!(url, "http://127.0.0.1:8080");
    }

    #[test]
    fn test_proxy_url_string_tor() {
        let url = proxy_url_string(&ProxyConfig::Tor);
        assert_eq!(url, format!("socks5://{}", TOR_SOCKS_ADDR));
    }

    #[test]
    fn test_proxy_url_string_none() {
        let url = proxy_url_string(&ProxyConfig::None);
        assert_eq!(url, "");
    }

    #[test]
    fn test_proxy_config_from_url_empty() {
        assert!(matches!(proxy_config_from_url(""), ProxyConfig::None));
    }

    #[test]
    fn test_proxy_config_from_url_static() {
        let cfg = proxy_config_from_url("http://proxy:8080");
        assert!(matches!(cfg, ProxyConfig::Static(_)));
    }

    #[test]
    fn test_gaussian_delay_within_bounds() {
        for _ in 0..100 {
            let delay = gaussian_delay_ms(100.0, 20.0, 10, 200);
            assert!(delay >= 10, "delay {} < min", delay);
            assert!(delay <= 200, "delay {} > max", delay);
        }
    }

    #[test]
    fn test_gaussian_delay_clamped() {
        for _ in 0..100 {
            let delay = gaussian_delay_ms(500.0, 500.0, 100, 600);
            assert!(delay <= 600, "delay {} > max 600", delay);
            assert!(delay >= 100, "delay {} < min 100", delay);
        }
    }

    #[test]
    fn test_stealth_user_agent_contains_chrome() {
        let ua = stealth_user_agent();
        assert!(ua.contains("Chrome/") || ua.contains("Firefox/") || ua.contains("Safari/"));
        assert!(ua.starts_with("Mozilla/"));
    }

    #[test]
    fn test_entropy_budget_zero_limit_exhausted() {
        let budget = EntropyBudget::new(0.0);
        assert!(budget.is_exhausted());
    }

    #[test]
    fn test_proxy_config_from_url_https() {
        let cfg = proxy_config_from_url("https://proxy:8443");
        assert!(matches!(cfg, ProxyConfig::Static(url) if url == "https://proxy:8443"));
    }

    #[test]
    fn test_doh_servers_all_https() {
        for server in DOH_SERVERS {
            assert!(
                server.starts_with("https://"),
                "DOH server should start with https://: {}",
                server
            );
        }
    }

    #[test]
    fn test_gaussian_delay_min_equals_max() {
        for _ in 0..20 {
            let delay = gaussian_delay_ms(100.0, 5.0, 100, 100);
            assert_eq!(delay, 100);
        }
    }

    #[test]
    fn test_proxy_config_variants_debug() {
        let none = ProxyConfig::None;
        let static_cfg = ProxyConfig::Static("http://p:8080".into());
        let tor = ProxyConfig::Tor;
        assert!(format!("{:?}", none).contains("None"));
        assert!(format!("{:?}", static_cfg).contains("http://p:8080"));
        assert!(format!("{:?}", tor).contains("Tor"));
    }
}
