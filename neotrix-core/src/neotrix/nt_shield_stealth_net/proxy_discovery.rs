use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use log;
use tokio::sync::RwLock;

use super::http_client::StealthHttpClient;
use super::proxy_sourcing::{
    self, batch_validate, ProxyProtocol, RawProxy,
};

const CRAWL_TIMEOUT_SECS: u64 = 30;
const CRAWL_DELAY_MS: u64 = 3000;
const MAX_DISCOVERED_PER_TARGET: usize = 200;
const MIN_PROXY_PORT: u16 = 80;
const MAX_PROXY_PORT: u16 = 65535;

const MAX_TARGET_FAILURES: u32 = 5;
const TARGET_COOLDOWN_SECS: u64 = 900;
const DISCOVERY_INTERVAL_HOURS: u64 = 6;

#[derive(Debug, Clone)]
struct DiscoveryTarget {
    name: &'static str,
    url: &'static str,
    hint_protocol: Option<ProxyProtocol>,
}

static DISCOVERY_TARGETS: &[DiscoveryTarget] = &[
    DiscoveryTarget {
        name: "free-proxy-list",
        url: "https://free-proxy-list.net/",
        hint_protocol: Some(ProxyProtocol::Http),
    },
    DiscoveryTarget {
        name: "sslproxies",
        url: "https://www.sslproxies.org/",
        hint_protocol: Some(ProxyProtocol::Https),
    },
    DiscoveryTarget {
        name: "us-proxy",
        url: "https://www.us-proxy.org/",
        hint_protocol: Some(ProxyProtocol::Http),
    },
    DiscoveryTarget {
        name: "socks-proxy",
        url: "https://www.socks-proxy.net/",
        hint_protocol: Some(ProxyProtocol::Socks5),
    },
    DiscoveryTarget {
        name: "spys-one",
        url: "https://spys.one/en/",
        hint_protocol: None,
    },
    DiscoveryTarget {
        name: "proxynova",
        url: "https://www.proxynova.com/proxy-server-list/",
        hint_protocol: Some(ProxyProtocol::Http),
    },
    DiscoveryTarget {
        name: "hidemy-name",
        url: "https://hidemy.name/en/proxy-list/",
        hint_protocol: None,
    },
    DiscoveryTarget {
        name: "proxylist-mobi",
        url: "https://www.proxylist.mobi/",
        hint_protocol: None,
    },
];

#[derive(Debug, Clone, Default)]
pub struct DiscoveryTargetHealth {
    consecutive_failures: u32,
    total_fetches: u64,
    total_proxies: u64,
    total_valid: u64,
    #[allow(dead_code)]
    last_attempt: Option<Instant>,
    cooldown_until: Option<Instant>,
}

impl DiscoveryTargetHealth {
    fn is_on_cooldown(&self) -> bool {
        self.cooldown_until
            .map(|t| Instant::now() < t)
            .unwrap_or(false)
    }

    fn record_success(&mut self, count: usize) {
        self.consecutive_failures = 0;
        self.total_fetches += 1;
        self.total_proxies += count as u64;
        self.cooldown_until = None;
    }

    fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.total_fetches += 1;
        if self.consecutive_failures >= MAX_TARGET_FAILURES {
            self.cooldown_until =
                Some(Instant::now() + Duration::from_secs(TARGET_COOLDOWN_SECS));
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_fetches == 0 {
            return 0.5;
        }
        let total_valid = self.total_valid as f64;
        let total = self.total_proxies.max(1) as f64;
        (total_valid / total).min(1.0)
    }
}

pub struct ProxyDiscoveryEngine {
    client: StealthHttpClient,
    target_health: RwLock<HashMap<&'static str, DiscoveryTargetHealth>>,
    discovered_targets: RwLock<Vec<String>>,
    last_run: RwLock<Option<Instant>>,
}

impl ProxyDiscoveryEngine {
    pub fn new() -> Self {
        let client = StealthHttpClient::new();
        Self {
            client,
            target_health: RwLock::new(HashMap::new()),
            discovered_targets: RwLock::new(Vec::new()),
            last_run: RwLock::new(None),
        }
    }

    pub async fn run_discovery_cycle(&self) {
        log::info!("[proxy-discovery] starting discovery cycle");

        let mut total = 0usize;
        let mut valid = 0usize;

        for target in DISCOVERY_TARGETS {
            let health = self.target_health.read().await;
            let h = health.get(target.name).cloned().unwrap_or_default();
            drop(health);

            if h.is_on_cooldown() {
                log::debug!("[proxy-discovery] {} on cooldown, skipping", target.name);
                continue;
            }

            match self.crawl_target(target).await {
                Ok(proxies) => {
                    let count = proxies.len();
                    total += count;
                    let valid_count = self.validate_and_feed(target, proxies).await;
                    valid += valid_count;
                    log::info!(
                        "[proxy-discovery] {}: {} found, {} valid",
                        target.name,
                        count,
                        valid_count
                    );
                    let mut health = self.target_health.write().await;
                    let entry = health.entry(target.name).or_default();
                    entry.record_success(count);
                    entry.total_valid += valid_count as u64;
                }
                Err(e) => {
                    log::warn!("[proxy-discovery] {} failed: {}", target.name, e);
                    let mut health = self.target_health.write().await;
                    health.entry(target.name).or_default().record_failure();
                }
            }

            tokio::time::sleep(Duration::from_millis(CRAWL_DELAY_MS)).await;
        }

        let discovered = self.discovered_targets.read().await.clone();
        for target_url in &discovered {
            match self.crawl_custom_target(target_url).await {
                Ok(proxies) => {
                    let count = proxies.len();
                    total += count;
                    let valid_count = self.validate_and_feed_raw(proxies).await;
                    valid += valid_count;
                    log::info!(
                        "[proxy-discovery] custom {}: {} found, {} valid",
                        target_url,
                        count,
                        valid_count
                    );
                }
                Err(e) => {
                    log::debug!("[proxy-discovery] custom {}: {}", target_url, e);
                }
            }
            tokio::time::sleep(Duration::from_millis(CRAWL_DELAY_MS)).await;
        }

        *self.last_run.write().await = Some(Instant::now());
        log::info!(
            "[proxy-discovery] cycle complete: {} total, {} valid across {} targets + {} custom",
            total,
            valid,
            DISCOVERY_TARGETS.len(),
            discovered.len(),
        );
    }

    async fn crawl_target(&self, target: &DiscoveryTarget) -> Result<Vec<RawProxy>, String> {
        let resp = tokio::time::timeout(
            Duration::from_secs(CRAWL_TIMEOUT_SECS),
            self.client.fetch(target.url),
        )
        .await
        .map_err(|_| format!("timeout fetching {}", target.url))?
        .map_err(|e| format!("fetch error: {}", e))?;

        if resp.status != 200 {
            return Err(format!("HTTP {}", resp.status));
        }

        let text = resp.text().map_err(|e| format!("utf8 error: {}", e))?;
        let proxies = extract_proxies_from_html(&text, target.hint_protocol);
        Ok(proxies)
    }

    async fn crawl_custom_target(&self, url: &str) -> Result<Vec<RawProxy>, String> {
        let resp = tokio::time::timeout(
            Duration::from_secs(CRAWL_TIMEOUT_SECS),
            self.client.fetch(url),
        )
        .await
        .map_err(|_| format!("timeout fetching {}", url))?
        .map_err(|e| format!("fetch error: {}", e))?;

        if resp.status != 200 {
            return Err(format!("HTTP {}", resp.status));
        }

        let text = resp.text().map_err(|e| format!("utf8 error: {}", e))?;

        if text.contains('\n') && text.lines().count() > 2 {
            let proxies = proxy_sourcing::parse_plain_text(&text, &[
                ProxyProtocol::Http,
                ProxyProtocol::Https,
                ProxyProtocol::Socks4,
                ProxyProtocol::Socks5,
            ]);
            if !proxies.is_empty() {
                return Ok(proxies);
            }
        }

        let proxies = extract_proxies_from_html(&text, None);
        Ok(proxies)
    }

    async fn validate_and_feed(&self, target: &DiscoveryTarget, proxies: Vec<RawProxy>) -> usize {
        if proxies.is_empty() {
            return 0;
        }

        let validated =
            batch_validate(&proxies, 50, 5, DEFAULT_VALIDATION_TARGETS).await;

        let valid_count = validated
            .iter()
            .filter(|(_, r)| r.connect_ok)
            .count();

        if valid_count > 0 {
            let valid_proxies: Vec<RawProxy> = validated
                .into_iter()
                .filter(|(_, r)| r.connect_ok)
                .map(|(p, _)| p)
                .collect();

            let _ = self.push_to_pool(target, valid_proxies).await;
        }

        valid_count
    }

    async fn validate_and_feed_raw(&self, proxies: Vec<RawProxy>) -> usize {
        if proxies.is_empty() {
            return 0;
        }

        let validated =
            batch_validate(&proxies, 50, 5, DEFAULT_VALIDATION_TARGETS).await;

        let valid_count = validated
            .iter()
            .filter(|(_, r)| r.connect_ok)
            .count();

        valid_count
    }

    async fn push_to_pool(
        &self,
        target: &DiscoveryTarget,
        proxies: Vec<RawProxy>,
    ) -> Result<usize, String> {
        let pool = super::proxy_pool::global_pool();
        let mut count = 0usize;
        for proxy in proxies {
            let url = proxy.to_proxy_url();
            let tag = format!("discovered/{}", target.name);
            pool.add(&url, &tag).await;
            count += 1;
        }
        log::info!(
            "[proxy-discovery] pushed {} proxies to pool from {}",
            count,
            target.name
        );
        Ok(count)
    }

    pub fn add_discovery_target(&self, url: &str) {
        let mut targets = tokio::task::block_in_place(|| {
            self.discovered_targets.blocking_write()
        });
        if !targets.contains(&url.to_string()) {
            targets.push(url.to_string());
            log::info!("[proxy-discovery] added custom target: {}", url);
        }
    }

    pub async fn target_health_snapshot(&self) -> Vec<(&'static str, DiscoveryTargetHealth)> {
        let health = self.target_health.read().await;
        DISCOVERY_TARGETS
            .iter()
            .map(|t| {
                (
                    t.name,
                    health.get(t.name).cloned().unwrap_or_default(),
                )
            })
            .collect()
    }

    pub async fn last_run_elapsed(&self) -> Option<Duration> {
        self.last_run
            .read()
            .await
            .map(|t| t.elapsed())
    }

    pub async fn start_discovery_loop(self: Arc<Self>) {
        loop {
            let interval = Duration::from_secs(DISCOVERY_INTERVAL_HOURS * 3600);
            let elapsed = self.last_run_elapsed().await;

            if elapsed.map(|e| e >= interval).unwrap_or(true) {
                self.run_discovery_cycle().await;
            }

            tokio::time::sleep(Duration::from_secs(300)).await;
        }
    }
}

const DEFAULT_VALIDATION_TARGETS: &[(&str, u16)] = &[
    ("httpbin.org", 80),
    ("example.com", 80),
    ("google.com", 80),
];

fn extract_proxies_from_html(
    html: &str,
    hint_protocol: Option<ProxyProtocol>,
) -> Vec<RawProxy> {
    let mut proxies = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in html.lines() {
        if proxies.len() >= MAX_DISCOVERED_PER_TARGET {
            break;
        }

        let found = find_ip_port_pairs(line);
        for (ip, port) in found {
            let key = (ip.clone(), port);
            if !seen.insert(key) {
                continue;
            }
            if !is_valid_ip(&ip) || port < MIN_PROXY_PORT || port > MAX_PROXY_PORT {
                continue;
            }

            let protocol = detect_protocol(line, hint_protocol, port);
            proxies.push(RawProxy {
                ip,
                port,
                protocol,
                source: None,
            });
        }
    }

    proxies
}

fn find_ip_port_pairs(text: &str) -> Vec<(String, u16)> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }

            if i < len && bytes[i] == b':' {
                let ip_candidate = &text[start..i];

                let port_start = i + 1;
                let mut port_end = port_start;
                while port_end < len && bytes[port_end].is_ascii_digit() {
                    port_end += 1;
                }

                if port_end > port_start {
                    let port_str = &text[port_start..port_end];
                    if let Ok(port) = port_str.parse::<u16>() {
                        results.push((ip_candidate.to_string(), port));
                    }
                }
            }
            continue;
        }
        i += 1;
    }

    results
}

fn is_valid_ip(ip: &str) -> bool {
    let octets: Vec<&str> = ip.split('.').collect();
    if octets.len() != 4 {
        return false;
    }
    for octet in &octets {
        let val: u16 = match octet.parse() {
            Ok(v) => v,
            Err(_) => return false,
        };
        if val > 255 {
            return false;
        }
        if octet.len() > 1 && octet.starts_with('0') {
            return false;
        }
    }
    true
}

fn detect_protocol(
    line: &str,
    hint: Option<ProxyProtocol>,
    _port: u16,
) -> ProxyProtocol {
    if line.to_lowercase().contains("socks5") {
        return ProxyProtocol::Socks5;
    }
    if line.to_lowercase().contains("socks4") {
        return ProxyProtocol::Socks4;
    }
    if line.to_lowercase().contains("https") || line.to_lowercase().contains("ssl") {
        return ProxyProtocol::Https;
    }

    hint.unwrap_or(ProxyProtocol::Http)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_proxies_from_html_simple() {
        let html = r#"
        <tr><td>1.2.3.4</td><td>8080</td><td>HTTP</td></tr>
        <tr><td>5.6.7.8</td><td>3128</td><td>HTTPS</td></tr>
        "#;
        let proxies = extract_proxies_from_html(html, Some(ProxyProtocol::Http));
        assert!(proxies.len() >= 2);
        let has_1_2_3_4 = proxies.iter().any(|p| p.ip == "1.2.3.4" && p.port == 8080);
        let has_5_6_7_8 = proxies.iter().any(|p| p.ip == "5.6.7.8" && p.port == 3128);
        assert!(has_1_2_3_4, "should find 1.2.3.4:8080");
        assert!(has_5_6_7_8, "should find 5.6.7.8:3128");
    }

    #[test]
    fn test_extract_proxies_ip_port_inline() {
        let html = "proxy 1.2.3.4:8080 and 5.6.7.8:3128 are available";
        let proxies = extract_proxies_from_html(html, None);
        assert_eq!(proxies.len(), 2);
        assert_eq!(proxies[0].ip, "1.2.3.4");
        assert_eq!(proxies[0].port, 8080);
    }

    #[test]
    fn test_is_valid_ip_valid() {
        assert!(is_valid_ip("1.2.3.4"));
        assert!(is_valid_ip("192.168.0.1"));
        assert!(is_valid_ip("255.255.255.255"));
        assert!(is_valid_ip("0.0.0.0"));
    }

    #[test]
    fn test_is_valid_ip_invalid() {
        assert!(!is_valid_ip("256.1.2.3"));
        assert!(!is_valid_ip("1.2.3"));
        assert!(!is_valid_ip("1.2.3.4.5"));
        assert!(!is_valid_ip("abc.def.ghi.jkl"));
        assert!(!is_valid_ip(""));
    }

    #[test]
    fn test_detect_protocol_from_text() {
        let line = "socks5://1.2.3.4:1080";
        assert_eq!(
            detect_protocol(line, None, 1080),
            ProxyProtocol::Socks5
        );

        let line = "socks4://1.2.3.4:1080";
        assert_eq!(
            detect_protocol(line, None, 1080),
            ProxyProtocol::Socks4
        );

        let line = "https://1.2.3.4:443";
        assert_eq!(
            detect_protocol(line, None, 443),
            ProxyProtocol::Https
        );
    }

    #[test]
    fn test_detect_protocol_hint_fallback() {
        let line = "1.2.3.4:8080";
        assert_eq!(
            detect_protocol(line, Some(ProxyProtocol::Http), 8080),
            ProxyProtocol::Http
        );
        assert_eq!(
            detect_protocol(line, Some(ProxyProtocol::Https), 8080),
            ProxyProtocol::Https
        );
    }

    #[test]
    fn test_find_ip_port_pairs() {
        let text = "1.2.3.4:8080 some text 5.6.7.8:3128 end";
        let pairs = find_ip_port_pairs(text);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("1.2.3.4".to_string(), 8080));
        assert_eq!(pairs[1], ("5.6.7.8".to_string(), 3128));
    }

    #[test]
    fn test_find_ip_port_pairs_no_false_positives() {
        let text = "no port here 1.2.3.4 or 5.6.7.8:notaport";
        let pairs = find_ip_port_pairs(text);
        assert!(pairs.is_empty() || pairs.iter().all(|p| p.0 == "5.6.7.8" && p.1 == 0));
    }

    #[test]
    fn test_target_health_default() {
        let health = DiscoveryTargetHealth::default();
        assert!(!health.is_on_cooldown());
        assert!((health.success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_target_health_after_failures() {
        let mut health = DiscoveryTargetHealth::default();
        for _ in 0..MAX_TARGET_FAILURES {
            health.record_failure();
        }
        assert!(health.is_on_cooldown());
    }

    #[test]
    fn test_target_health_recovery() {
        let mut health = DiscoveryTargetHealth::default();
        for _ in 0..MAX_TARGET_FAILURES {
            health.record_failure();
        }
        assert!(health.is_on_cooldown());
        health.record_success(10);
        assert!(!health.is_on_cooldown());
        assert_eq!(health.total_proxies, 10);
    }

    #[test]
    fn test_extract_proxies_handles_empty() {
        let proxies = extract_proxies_from_html("", None);
        assert!(proxies.is_empty());
    }

    #[test]
    fn test_extract_proxies_skips_invalid() {
        let html = "256.256.256.256:8080 1.2.3.4:0 1.2.3.4:99999";
        let proxies = extract_proxies_from_html(html, None);
        assert!(proxies.is_empty());
    }

    #[test]
    fn test_extract_proxies_deduplicates() {
        let html = "1.2.3.4:8080 and 1.2.3.4:8080 again";
        let proxies = extract_proxies_from_html(html, None);
        assert_eq!(proxies.len(), 1);
    }

    #[test]
    fn test_discovery_targets_defined() {
        assert!(!DISCOVERY_TARGETS.is_empty());
        for target in DISCOVERY_TARGETS {
            assert!(!target.name.is_empty());
            assert!(target.url.starts_with("http"));
        }
    }
}
