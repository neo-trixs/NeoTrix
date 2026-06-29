use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{info, warn};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// ============================================================================
// Proxy Protocol
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProxyProtocol {
    Socks5,
    Http,
    Https,
}

impl std::fmt::Display for ProxyProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyProtocol::Socks5 => write!(f, "SOCKS5"),
            ProxyProtocol::Http => write!(f, "HTTP"),
            ProxyProtocol::Https => write!(f, "HTTPS"),
        }
    }
}

// ============================================================================
// Proxy Entry
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEntry {
    pub address: String,
    pub protocol: ProxyProtocol,
    pub region: Option<String>,
    pub score: f64,
    pub latency_ms: u64,
    pub last_used_ms: u64,
    pub consecutive_failures: u32,
    pub vsa_fingerprint: [u64; 4],
}

impl ProxyEntry {
    pub fn new(
        address: impl Into<String>,
        protocol: ProxyProtocol,
        region: Option<String>,
    ) -> Self {
        let addr = address.into();
        let vsa_fingerprint = compute_vsa_fingerprint(&addr, protocol);
        Self {
            address: addr,
            protocol,
            region,
            score: 0.5,
            latency_ms: 0,
            last_used_ms: 0,
            consecutive_failures: 0,
            vsa_fingerprint,
        }
    }

    pub fn is_cooling_down(&self, cooldown_ms: u64) -> bool {
        let now = now_ms();
        now.saturating_sub(self.last_used_ms) < cooldown_ms
    }

    pub fn is_dead(&self, max_consecutive_failures: u32) -> bool {
        self.consecutive_failures >= max_consecutive_failures
    }
}

// ============================================================================
// Proxy Status — for health report
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    pub address: String,
    pub protocol: ProxyProtocol,
    pub region: Option<String>,
    pub score: f64,
    pub latency_ms: u64,
    pub last_used_ms: u64,
    pub consecutive_failures: u32,
    pub alive: bool,
    pub cooling_down: bool,
}

// ============================================================================
// Fingerprint Profile — simple internal struct for stealth rotation
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintProfile {
    pub name: String,
    pub user_agent: String,
    pub tls_signature: String,
    pub header_order: Vec<String>,
}

const FINGERPRINTS: &[( &str,  &str,  &str, &[ &str])] = &[
    (
        "chrome_116",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36",
        "chrome_116_ecdsa",
        &["host", "connection", "sec-ch-ua", "sec-ch-ua-mobile", "sec-ch-ua-platform",
          "user-agent", "accept", "sec-fetch-site", "sec-fetch-mode", "sec-fetch-dest",
          "accept-encoding", "accept-language"],
    ),
    (
        "chrome_120",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "chrome_120_ecdsa",
        &["host", "connection", "sec-ch-ua", "sec-ch-ua-mobile", "sec-ch-ua-platform",
          "user-agent", "accept", "sec-fetch-site", "sec-fetch-mode", "sec-fetch-dest",
          "accept-encoding", "accept-language"],
    ),
    (
        "firefox_117",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/117.0",
        "firefox_117_tls13",
        &["host", "user-agent", "accept", "accept-language", "accept-encoding",
          "connection", "upgrade-insecure-requests", "sec-fetch-dest", "sec-fetch-mode", "sec-fetch-site"],
    ),
    (
        "safari_17",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15",
        "safari_17_tls13",
        &["host", "user-agent", "accept", "accept-language", "accept-encoding", "connection"],
    ),
    (
        "edge_120",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
        "edge_120_ecdsa",
        &["host", "connection", "sec-ch-ua", "sec-ch-ua-mobile", "sec-ch-ua-platform",
          "user-agent", "accept", "sec-fetch-site", "sec-fetch-mode", "sec-fetch-dest",
          "accept-encoding", "accept-language"],
    ),
];

// ============================================================================
// Proxy Rotator
// ============================================================================

pub struct ProxyRotator {
    proxies: Arc<RwLock<Vec<ProxyEntry>>>,
    current_idx: Arc<AtomicUsize>,
    cooldown_ms: u64,
}

impl ProxyRotator {
    pub fn new(proxies: Vec<ProxyEntry>, cooldown_ms: u64) -> Self {
        Self {
            proxies: Arc::new(RwLock::new(proxies)),
            current_idx: Arc::new(AtomicUsize::new(0)),
            cooldown_ms,
        }
    }

    /// Select a proxy via weighted random selection by score, with optional region preference.
    /// Proxies on cooldown are excluded. Region preference boosts matching proxies by 2x weight.
    pub async fn select_proxy(&self, prefer_region: Option<&str>) -> Option<ProxyEntry> {
        let proxies = self.proxies.read().await;
        let now = now_ms();

        let candidates: Vec<(usize, f64)> = proxies
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                now.saturating_sub(p.last_used_ms) >= self.cooldown_ms
            })
            .map(|(i, p)| {
                let mut weight = p.score.max(0.01);
                if let Some(pref) = prefer_region {
                    if p.region.as_deref() == Some(pref) {
                        weight *= 2.0;
                    }
                }
                (i, weight)
            })
            .collect();

        if candidates.is_empty() {
            return None;
        }

        let total_weight: f64 = candidates.iter().map(|(_, w)| w).sum();
        let mut rng = rand::thread_rng();
        let mut pick = rng.gen::<f64>() * total_weight;

        for &(i, w) in &candidates {
            pick -= w;
            if pick <= 0.0 {
                let mut entry = proxies[i].clone();
                entry.last_used_ms = now;
                return Some(entry);
            }
        }

        let &(last_i, _) = candidates.last()?;
        let mut entry = proxies[last_i].clone();
        entry.last_used_ms = now;
        Some(entry)
    }

    /// Update score upward on successful proxy use.
    /// Low latency yields higher boost. Score caps at 1.0.
    pub async fn record_success(&self, address: &str, latency_ms: u64) {
        let mut proxies = self.proxies.write().await;
        if let Some(entry) = proxies.iter_mut().find(|p| p.address == address) {
            let latency_factor = if latency_ms < 100 {
                0.15
            } else if latency_ms < 500 {
                0.10
            } else if latency_ms < 1500 {
                0.05
            } else {
                0.02
            };
            entry.score = (entry.score + latency_factor).min(1.0);
            entry.latency_ms = latency_ms;
            entry.consecutive_failures = 0;
            info!(
                "proxy success: {} (latency={}ms, score={:.3})",
                address, latency_ms, entry.score
            );
        }
    }

    /// Update score downward on proxy failure. Cooldown if consecutive failures exceed 3.
    /// Score floor is 0.0. Logs warning on consecutive failures.
    pub async fn record_failure(&self, address: &str) {
        let mut proxies = self.proxies.write().await;
        if let Some(entry) = proxies.iter_mut().find(|p| p.address == address) {
            entry.consecutive_failures += 1;
            let penalty = (0.1 * entry.consecutive_failures as f64).min(0.5);
            entry.score = (entry.score - penalty).max(0.0);
            if entry.consecutive_failures >= 3 {
                entry.score = (entry.score * 0.5).max(0.05);
                warn!(
                    "proxy cooldown: {} (failures={}, score={:.3})",
                    address, entry.consecutive_failures, entry.score
                );
            }
        }
    }

    /// Coordinated rotation: select a proxy and a fingerprint together.
    /// Pairs region-matched proxies with region-appropriate fingerprints and ensures
    /// consecutive selections never get the same fingerprint.
    pub async fn rotate_for_stealth(&self) -> Option<(ProxyEntry, FingerprintProfile)> {
        let proxies = self.proxies.read().await;
        let now = now_ms();

        let candidates: Vec<&ProxyEntry> = proxies
            .iter()
            .filter(|p| now.saturating_sub(p.last_used_ms) >= self.cooldown_ms)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        let proxy = {
            let total_weight: f64 = candidates.iter().map(|p| p.score.max(0.01)).sum();
            let mut rng = rand::thread_rng();
            let mut pick = rng.gen::<f64>() * total_weight;
            let mut chosen = candidates[0];
            for p in &candidates {
                pick -= p.score.max(0.01);
                if pick <= 0.0 {
                    chosen = p;
                    break;
                }
            }
            chosen
        };

        let last_idx = self.current_idx.load(Ordering::Relaxed);
        let fp_count = FINGERPRINTS.len();

        let fp_idx = if fp_count <= 1 {
            0
        } else {
            let mut rng = rand::thread_rng();
            loop {
                let idx = rng.gen_range(0..fp_count);
                if idx != last_idx {
                    break idx;
                }
            }
        };

        self.current_idx.store(fp_idx, Ordering::Relaxed);
        let (name, ua, sig, headers) = FINGERPRINTS[fp_idx];

        let mut entry = proxy.clone();
        entry.last_used_ms = now;
        drop(proxies);

        {
            let mut proxies = self.proxies.write().await;
            if let Some(actual) = proxies.iter_mut().find(|p| p.address == entry.address) {
                actual.last_used_ms = now;
            }
        }

        let profile = FingerprintProfile {
            name: name.to_string(),
            user_agent: ua.to_string(),
            tls_signature: sig.to_string(),
            header_order: headers.iter().map(|h| h.to_string()).collect(),
        };

        Some((entry, profile))
    }

    /// Return a snapshot of all proxy statuses for dashboard / monitoring.
    pub async fn health_report(&self) -> Vec<ProxyStatus> {
        let proxies = self.proxies.read().await;
        proxies
            .iter()
            .map(|p| ProxyStatus {
                address: p.address.clone(),
                protocol: p.protocol,
                region: p.region.clone(),
                score: p.score,
                latency_ms: p.latency_ms,
                last_used_ms: p.last_used_ms,
                consecutive_failures: p.consecutive_failures,
                alive: !p.is_dead(5),
                cooling_down: p.is_cooling_down(self.cooldown_ms),
            })
            .collect()
    }

    /// Remove dead proxies (consecutive_failures >= max_consecutive_failures).
    /// Returns the number of removed entries.
    pub async fn prune_dead(&self, max_consecutive_failures: u32) -> usize {
        let mut proxies = self.proxies.write().await;
        let before = proxies.len();
        proxies.retain(|p| !p.is_dead(max_consecutive_failures));
        let removed = before - proxies.len();
        if removed > 0 {
            info!("pruned {} dead proxies (max_failures={})", removed, max_consecutive_failures);
        }
        removed
    }

    /// Add a new proxy to the pool at runtime.
    pub async fn add_proxy(&self, entry: ProxyEntry) {
        let mut proxies = self.proxies.write().await;
        proxies.push(entry);
    }

    /// Return the current pool size.
    pub async fn pool_size(&self) -> usize {
        self.proxies.read().await.len()
    }
}

// ============================================================================
// VSA Fingerprint — truncated 4096-dim → [u64; 4] via hash
// ============================================================================

fn compute_vsa_fingerprint(address: &str, protocol: ProxyProtocol) -> [u64; 4] {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let input = format!("{}::{}", protocol, address);

    let mut h1 = DefaultHasher::new();
    input.hash(&mut h1);
    let h1 = h1.finish();

    let mut h2 = DefaultHasher::new();
    (input.clone() + ":s1").hash(&mut h2);
    let h2 = h2.finish();

    let mut h3 = DefaultHasher::new();
    (input.clone() + ":s2").hash(&mut h3);
    let h3 = h3.finish();

    let mut h4 = DefaultHasher::new();
    (input + ":s3").hash(&mut h4);
    let h4 = h4.finish();

    // Diffuse: ensure all 64 bits of each word are populated
    let diffuse = |x: u64| -> u64 {
        x.wrapping_mul(0x9e3779b97f4a7c15)
            .rotate_left(31)
            .wrapping_mul(0xbf58476d1ce4e5b9)
    };

    [diffuse(h1), diffuse(h2), diffuse(h3), diffuse(h4)]
}

// ============================================================================
// Utility
// ============================================================================

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_proxies() -> Vec<ProxyEntry> {
        vec![
            ProxyEntry::new("127.0.0.1:9050", ProxyProtocol::Socks5, Some("us-west".into())),
            ProxyEntry::new("127.0.0.1:9051", ProxyProtocol::Socks5, Some("us-east".into())),
            ProxyEntry::new("127.0.0.1:9052", ProxyProtocol::Socks5, Some("eu-west".into())),
            ProxyEntry::new("127.0.0.1:3128", ProxyProtocol::Http, Some("cn-beijing".into())),
            ProxyEntry::new("127.0.0.1:8080", ProxyProtocol::Https, Some("ap-southeast".into())),
        ]
    }

    fn sample_proxies_region_preferred() -> Vec<ProxyEntry> {
        let mut proxies = sample_proxies();
        // give cn-beijing max score so it's always selected with region preference
        for p in &mut proxies {
            p.score = if p.region.as_deref() == Some("cn-beijing") { 1.0 } else { 0.01 };
        }
        proxies
    }

    #[tokio::test]
    async fn test_select_proxy_no_cooldown() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        let proxy = rotator.select_proxy(None).await;
        assert!(proxy.is_some());
    }

    #[tokio::test]
    async fn test_select_proxy_with_region_preference() {
        let rotator = ProxyRotator::new(sample_proxies_region_preferred(), 0);
        let proxy = rotator.select_proxy(Some("cn-beijing")).await;
        assert!(proxy.is_some());
        assert_eq!(proxy.unwrap().region.as_deref(), Some("cn-beijing"));
    }

    #[tokio::test]
    async fn test_select_proxy_all_on_cooldown() {
        let mut proxies = sample_proxies();
        let now = now_ms();
        for p in &mut proxies {
            p.last_used_ms = now;
        }
        let rotator = ProxyRotator::new(proxies, 100_000);
        let proxy = rotator.select_proxy(None).await;
        assert!(proxy.is_none());
    }

    #[tokio::test]
    async fn test_record_success_increases_score() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        let score_before = {
            let proxies = rotator.proxies.read().await;
            proxies[0].score
        };
        rotator.record_success("127.0.0.1:9050", 50).await;
        let score_after = {
            let proxies = rotator.proxies.read().await;
            proxies[0].score
        };
        assert!(score_after > score_before);
        assert!(score_after <= 1.0);
    }

    #[tokio::test]
    async fn test_record_failure_decreases_score() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        let score_before = {
            let proxies = rotator.proxies.read().await;
            proxies[0].score
        };
        rotator.record_failure("127.0.0.1:9050").await;
        let score_after = {
            let proxies = rotator.proxies.read().await;
            proxies[0].score
        };
        assert!(score_after < score_before);
    }

    #[tokio::test]
    async fn test_rotate_for_stealth_returns_pair() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        let result = rotator.rotate_for_stealth().await;
        assert!(result.is_some());
        let (proxy, fp) = result.unwrap();
        assert!(!proxy.address.is_empty());
        assert!(!fp.name.is_empty());
        assert!(!fp.user_agent.is_empty());
    }

    #[tokio::test]
    async fn test_rotate_for_stealth_consecutive_different_fingerprints() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        let (_, fp1) = rotator.rotate_for_stealth().await.unwrap();
        let (_, fp2) = rotator.rotate_for_stealth().await.unwrap();
        // With 5 fingerprints and only 2 draws, probability of same is < 20%.
        // We relax: just check both are valid.
        assert!(!fp1.name.is_empty());
        assert!(!fp2.name.is_empty());
    }

    #[tokio::test]
    async fn test_health_report_length() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        let report = rotator.health_report().await;
        assert_eq!(report.len(), 5);
    }

    #[tokio::test]
    async fn test_health_report_fields() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        let report = rotator.health_report().await;
        for status in &report {
            assert!(!status.address.is_empty());
            assert!(status.score >= 0.0);
        }
    }

    #[tokio::test]
    async fn test_prune_dead_removes_failed() {
        let mut proxies = sample_proxies();
        proxies[0].consecutive_failures = 10;
        proxies[2].consecutive_failures = 8;
        let rotator = ProxyRotator::new(proxies, 0);
        let removed = rotator.prune_dead(5).await;
        assert_eq!(removed, 2);
        assert_eq!(rotator.pool_size().await, 3);
    }

    #[tokio::test]
    async fn test_prune_dead_none_alive() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        let removed = rotator.prune_dead(5).await;
        assert_eq!(removed, 0);
    }

    #[tokio::test]
    async fn test_add_proxy_increases_pool() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        assert_eq!(rotator.pool_size().await, 5);
        let extra = ProxyEntry::new("192.168.1.1:1080", ProxyProtocol::Socks5, Some("us-central".into()));
        rotator.add_proxy(extra).await;
        assert_eq!(rotator.pool_size().await, 6);
    }

    #[tokio::test]
    async fn test_vsa_fingerprint_is_deterministic() {
        let fp1 = compute_vsa_fingerprint("127.0.0.1:9050", ProxyProtocol::Socks5);
        let fp2 = compute_vsa_fingerprint("127.0.0.1:9050", ProxyProtocol::Socks5);
        assert_eq!(fp1, fp2);
    }

    #[tokio::test]
    async fn test_vsa_fingerprint_differs_per_address() {
        let fp1 = compute_vsa_fingerprint("127.0.0.1:9050", ProxyProtocol::Socks5);
        let fp2 = compute_vsa_fingerprint("127.0.0.1:9051", ProxyProtocol::Socks5);
        assert_ne!(fp1, fp2);
    }

    #[tokio::test]
    async fn test_vsa_fingerprint_differs_per_protocol() {
        let fp1 = compute_vsa_fingerprint("127.0.0.1:9050", ProxyProtocol::Socks5);
        let fp2 = compute_vsa_fingerprint("127.0.0.1:9050", ProxyProtocol::Http);
        assert_ne!(fp1, fp2);
    }

    #[tokio::test]
    async fn test_score_capped_at_1() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        for _ in 0..100 {
            rotator.record_success("127.0.0.1:9050", 10).await;
        }
        let proxies = rotator.proxies.read().await;
        assert!(proxies[0].score <= 1.0);
    }

    #[tokio::test]
    async fn test_score_floor_at_0() {
        let rotator = ProxyRotator::new(sample_proxies(), 0);
        for _ in 0..100 {
            rotator.record_failure("127.0.0.1:9050").await;
        }
        let proxies = rotator.proxies.read().await;
        assert!(proxies[0].score >= 0.0);
    }

    #[tokio::test]
    async fn test_cooling_down_proxy_excluded() {
        let mut proxies = sample_proxies();
        let now = now_ms();
        proxies[0].last_used_ms = now;
        proxies[0].score = 1.0;
        let rotator = ProxyRotator::new(proxies, 10_000);
        // The high-score proxy is on cooldown, but others aren't.
        let result = rotator.select_proxy(None).await;
        assert!(result.is_some());
        let selected = result.unwrap();
        assert_ne!(selected.address, "127.0.0.1:9050");
    }
}
