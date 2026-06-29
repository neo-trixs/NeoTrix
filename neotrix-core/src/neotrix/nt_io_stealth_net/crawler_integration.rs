//! Connects proxy rotator infrastructure to crawler/scraper pipeline.
//!
//! Three layers:
//! 1. ProxySelector — picks best proxy for a crawl target based on geo/health/strategy
//! 2. CrawlerProxyPool — manages pool of proxies for crawling sessions
//! 3. CrawlerIntegration — high-level API for crawler code to use proxy rotation

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::RwLock;

use super::proxy_chain::rotator::{ProxyEntry, ProxyRotator, RotationStrategy};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProxySelectionStrategy {
    Cyclic,
    Random,
    Fastest,
    GeoHinted,
    Adaptive,
}

impl From<RotationStrategy> for ProxySelectionStrategy {
    fn from(s: RotationStrategy) -> Self {
        match s {
            RotationStrategy::Cyclic => ProxySelectionStrategy::Cyclic,
            RotationStrategy::Random => ProxySelectionStrategy::Random,
            RotationStrategy::Performance => ProxySelectionStrategy::Fastest,
            RotationStrategy::Adaptive => ProxySelectionStrategy::Adaptive,
        }
    }
}

impl From<ProxySelectionStrategy> for RotationStrategy {
    fn from(s: ProxySelectionStrategy) -> Self {
        match s {
            ProxySelectionStrategy::Cyclic => RotationStrategy::Cyclic,
            ProxySelectionStrategy::Random => RotationStrategy::Random,
            ProxySelectionStrategy::Fastest => RotationStrategy::Performance,
            ProxySelectionStrategy::Adaptive => RotationStrategy::Adaptive,
            ProxySelectionStrategy::GeoHinted => RotationStrategy::Cyclic,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CrawlerProxySummary {
    pub total_proxies: usize,
    pub healthy_proxies: usize,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub switches: usize,
}

pub struct CrawlerProxyPool {
    rotator: RwLock<ProxyRotator>,
    strategy: ProxySelectionStrategy,
    cursor: AtomicU64,
    switches: AtomicU64,
}

impl CrawlerProxyPool {
    pub fn new(proxies: Vec<ProxyEntry>, strategy: ProxySelectionStrategy) -> Self {
        let rot_strategy: RotationStrategy = strategy.into();
        CrawlerProxyPool {
            rotator: RwLock::new(ProxyRotator::new(proxies, rot_strategy)),
            strategy,
            cursor: AtomicU64::new(0),
            switches: AtomicU64::new(0),
        }
    }

    pub async fn select_proxy(&self, target_url: &str) -> Option<ProxyEntry> {
        let mut rotator = self.rotator.write().await;

        if rotator.proxies.is_empty() {
            return None;
        }

        // GeoHinted: pick proxy based on target URL region
        if self.strategy == ProxySelectionStrategy::GeoHinted {
            return self.select_geo(&mut rotator, target_url);
        }

        let count = rotator.proxies.len();

        match self.strategy {
            ProxySelectionStrategy::Fastest => {
                let idx = (0..count)
                    .filter(|&i| rotator.proxies[i].health > 0.0)
                    .min_by_key(|&i| rotator.proxies[i].latency_ms)?;
                rotator.current_index = idx;
                rotator.proxies[idx].last_used = Instant::now();
                rotator.metrics.total_requests += 1;
                Some(rotator.proxies[idx].clone())
            }
            ProxySelectionStrategy::Adaptive => {
                for offset in 0..count {
                    let idx = (rotator.current_index + offset) % count;
                    if rotator.proxies[idx].health > 0.0 {
                        rotator.current_index = idx;
                        rotator.proxies[idx].last_used = Instant::now();
                        rotator.metrics.total_requests += 1;
                        return Some(rotator.proxies[idx].clone());
                    }
                }
                None
            }
            _ => {
                // Cyclic / Random: try until we find a healthy proxy
                let mut attempts = 0;
                while attempts < count {
                    let idx = match self.strategy {
                        ProxySelectionStrategy::Cyclic => {
                            let next = (rotator.current_index + 1) % count;
                            rotator.current_index = next;
                            next
                        }
                        _ => {
                            let mut rng = rand::thread_rng();
                            let idx = rng.gen_range(0..count);
                            rotator.current_index = idx;
                            idx
                        }
                    };

                    if rotator.proxies[idx].health > 0.0 {
                        rotator.proxies[idx].last_used = Instant::now();
                        rotator.metrics.total_requests += 1;
                        return Some(rotator.proxies[idx].clone());
                    }
                    attempts += 1;
                }
                None
            }
        }
    }

    fn select_geo(&self, rotator: &mut ProxyRotator, target_url: &str) -> Option<ProxyEntry> {
        let hint = guess_region_hint(target_url);
        let count = rotator.proxies.len();

        let matching: Vec<usize> = rotator
            .proxies
            .iter()
            .enumerate()
            .filter(|(_, p)| p.health > 0.0)
            .filter(|(_, p)| {
                if let Some(ref hint) = hint {
                    p.region
                        .as_ref()
                        .map(|r| r.to_lowercase().contains(&hint.to_lowercase()))
                        .unwrap_or(false)
                } else {
                    false
                }
            })
            .map(|(i, _)| i)
            .collect();

        let pool = if matching.is_empty() {
            // Fallback: all healthy proxies
            let all: Vec<usize> = rotator
                .proxies
                .iter()
                .enumerate()
                .filter(|(_, p)| p.health > 0.0)
                .map(|(i, _)| i)
                .collect();
            if all.is_empty() {
                return None;
            }
            all
        } else {
            matching
        };

        let offset = self.cursor.fetch_add(1, Ordering::Relaxed) as usize;
        let idx = pool[offset % pool.len()];
        rotator.current_index = idx;
        rotator.proxies[idx].last_used = Instant::now();
        rotator.metrics.total_requests += 1;
        Some(rotator.proxies[idx].clone())
    }

    pub async fn report_success(&self, latency_ms: u64) {
        let mut rotator = self.rotator.write().await;
        rotator.record_success(latency_ms);
    }

    pub async fn report_failure(&self) {
        let mut rotator = self.rotator.write().await;
        let was_adaptive = self.strategy == ProxySelectionStrategy::Adaptive;
        rotator.record_failure();
        if !was_adaptive && !rotator.proxies.is_empty() {
            rotator.current_index = (rotator.current_index + 1) % rotator.proxies.len();
            rotator.metrics.proxy_switches += 1;
            self.switches.fetch_add(1, Ordering::Relaxed);
        } else if was_adaptive {
            self.switches.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub async fn health_summary(&self) -> CrawlerProxySummary {
        let rotator = self.rotator.read().await;
        let total_proxies = rotator.proxies.len();
        let healthy_proxies = rotator.proxies.iter().filter(|p| p.health > 0.0).count();
        let success_rate = rotator.success_rate();
        let avg_latency_ms = rotator.metrics.average_latency_ms;
        let switches = rotator.metrics.proxy_switches;
        CrawlerProxySummary {
            total_proxies,
            healthy_proxies,
            success_rate,
            avg_latency_ms,
            switches,
        }
    }

    pub async fn prune_dead(&self) -> usize {
        let mut rotator = self.rotator.write().await;
        rotator.prune_dead()
    }

    pub async fn proxy_count(&self) -> usize {
        let rotator = self.rotator.read().await;
        rotator.proxies.len()
    }

    pub async fn healthy_count(&self) -> usize {
        let rotator = self.rotator.read().await;
        rotator.proxies.iter().filter(|p| p.health > 0.0).count()
    }
}

/// Guess a region hint from a target URL based on its TLD.
fn guess_region_hint(url: &str) -> Option<&'static str> {
    let domain = url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()?;

    let tld = domain.rsplit('.').next()?;
    let tld = tld.split(':').next()?; // strip port

    match tld {
        "com" | "org" | "net" | "io" | "co" | "gov" | "edu" => Some("us"),
        "cn" | "jp" | "kr" | "in" | "sg" | "hk" | "tw" | "my" | "th" | "vn" | "ph" | "id" => {
            Some("ap")
        }
        "de" | "fr" | "uk" | "co.uk" | "nl" | "se" | "no" | "it" | "es" | "pl" | "at" | "ch"
        | "be" | "dk" | "fi" | "ie" | "pt" | "gr" | "cz" | "ro" | "hu" | "ru" | "ua" => Some("eu"),
        "br" | "ar" | "cl" | "co" | "mx" | "pe" => Some("sa"),
        "au" | "nz" => Some("oceania"),
        "za" | "eg" | "ng" | "ke" | "ma" | "tn" => Some("africa"),
        "ae" | "sa" | "il" | "qa" | "om" | "kw" | "bh" => Some("me"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn make_proxies() -> Vec<ProxyEntry> {
        let now = Instant::now();
        vec![
            ProxyEntry {
                url: "http://proxy-a:8080".into(),
                protocol: "http".into(),
                region: Some("us-east".into()),
                health: 0.9,
                last_used: now,
                latency_ms: 100,
                failures: 0,
                max_failures: 3,
            },
            ProxyEntry {
                url: "http://proxy-b:8080".into(),
                protocol: "http".into(),
                region: Some("eu-west".into()),
                health: 0.8,
                last_used: now,
                latency_ms: 200,
                failures: 1,
                max_failures: 3,
            },
            ProxyEntry {
                url: "http://proxy-c:8080".into(),
                protocol: "http".into(),
                region: Some("ap-southeast".into()),
                health: 0.7,
                last_used: now,
                latency_ms: 50,
                failures: 0,
                max_failures: 3,
            },
        ]
    }

    #[tokio::test]
    async fn test_pool_cycles_through_proxies() {
        let pool = CrawlerProxyPool::new(make_proxies(), ProxySelectionStrategy::Cyclic);
        let first = pool
            .select_proxy("http://example.com")
            .await
            .expect("should return proxy");
        let second = pool
            .select_proxy("http://example.com")
            .await
            .expect("should return proxy");
        let third = pool
            .select_proxy("http://example.com")
            .await
            .expect("should return proxy");
        let fourth = pool
            .select_proxy("http://example.com")
            .await
            .expect("should return proxy");

        assert_ne!(first.url, second.url, "Cyclic should advance each call");
        assert_eq!(
            first.url, fourth.url,
            "Cyclic should wrap around after 3 calls"
        );
        // Verify all 3 proxies were returned across 6 calls
        let mut seen = std::collections::HashSet::new();
        for _ in 0..6 {
            let p = pool.select_proxy("http://example.com").await.unwrap();
            seen.insert(p.url);
        }
        assert_eq!(seen.len(), 3);
    }

    #[tokio::test]
    async fn test_pool_skips_dead_proxies() {
        let mut proxies = make_proxies();
        proxies[0].health = 0.0;
        proxies[1].health = 0.0;
        let pool = CrawlerProxyPool::new(proxies, ProxySelectionStrategy::Cyclic);

        let proxy = pool
            .select_proxy("http://example.com")
            .await
            .expect("should return healthy proxy");
        assert_eq!(proxy.url, "http://proxy-c:8080", "Only proxy-c is healthy");

        // Even after multiple calls, should only return the healthy one
        for _ in 0..5 {
            let p = pool.select_proxy("http://example.com").await.unwrap();
            assert_eq!(p.url, "http://proxy-c:8080");
        }
    }

    #[tokio::test]
    async fn test_pool_empty_returns_none() {
        let pool = CrawlerProxyPool::new(Vec::new(), ProxySelectionStrategy::Cyclic);
        assert!(pool.select_proxy("http://example.com").await.is_none());
    }

    #[tokio::test]
    async fn test_report_failure_switches_proxy() {
        let pool = CrawlerProxyPool::new(make_proxies(), ProxySelectionStrategy::Adaptive);

        let first = pool
            .select_proxy("http://example.com")
            .await
            .expect("should return proxy");
        let url_before = first.url.clone();
        drop(first);

        pool.report_failure().await;

        let second = pool
            .select_proxy("http://example.com")
            .await
            .expect("should return proxy");
        assert_ne!(
            second.url, url_before,
            "Failure should trigger proxy switch for Adaptive"
        );

        let summary = pool.health_summary().await;
        assert!(
            summary.switches >= 1,
            "Switches should be at least 1 after failure"
        );
    }

    #[tokio::test]
    async fn test_health_summary_accuracy() {
        let pool = CrawlerProxyPool::new(make_proxies(), ProxySelectionStrategy::Cyclic);

        // Make some requests
        let _p1 = pool.select_proxy("http://example.com").await;
        pool.report_success(100).await;

        let _p2 = pool.select_proxy("http://example.com").await;
        pool.report_success(200).await;

        let _p3 = pool.select_proxy("http://example.com").await;
        pool.report_failure().await;

        let summary = pool.health_summary().await;

        assert_eq!(summary.total_proxies, 3);
        assert_eq!(summary.healthy_proxies, 3);
        assert!(summary.success_rate > 0.0);
        assert!(summary.success_rate <= 1.0);
        assert!(summary.avg_latency_ms > 0.0);
    }

    #[tokio::test]
    async fn test_report_failure_triggers_rotation() {
        let pool = CrawlerProxyPool::new(make_proxies(), ProxySelectionStrategy::Cyclic);

        let first = pool.select_proxy("http://example.com").await.unwrap();
        let url_before = first.url;
        drop(first);

        pool.report_failure().await;

        let second = pool.select_proxy("http://example.com").await.unwrap();
        assert_ne!(second.url, url_before, "Cyclic should advance on failure");
    }

    #[tokio::test]
    async fn test_geo_hinted_selects_matching_region() {
        let pool = CrawlerProxyPool::new(make_proxies(), ProxySelectionStrategy::GeoHinted);

        // .de → eu region → should pick proxy-b (eu-west)
        let proxy = pool
            .select_proxy("https://example.de/path")
            .await
            .expect("should return proxy");
        assert_eq!(proxy.url, "http://proxy-b:8080");

        // .jp → ap region → should pick proxy-c (ap-southeast)
        let proxy = pool
            .select_proxy("https://example.jp/")
            .await
            .expect("should return proxy");
        assert_eq!(proxy.url, "http://proxy-c:8080");

        // .com → us region → should pick proxy-a (us-east)
        let proxy = pool
            .select_proxy("https://example.com/")
            .await
            .expect("should return proxy");
        assert_eq!(proxy.url, "http://proxy-a:8080");
    }

    #[tokio::test]
    async fn test_geo_hinted_fallback_when_no_match() {
        let pool = CrawlerProxyPool::new(make_proxies(), ProxySelectionStrategy::GeoHinted);

        // .xyz → no region hint → fallback to all healthy proxies (cyclic)
        for _ in 0..4 {
            let proxy = pool
                .select_proxy("https://example.xyz/")
                .await
                .expect("should return proxy");
            // Should return one of the healthy proxies
            assert!(
                proxy.url == "http://proxy-a:8080"
                    || proxy.url == "http://proxy-b:8080"
                    || proxy.url == "http://proxy-c:8080"
            );
        }
    }

    #[tokio::test]
    async fn test_prune_dead_removes_unhealthy() {
        let mut proxies = make_proxies();
        proxies[0].health = 0.0;
        let pool = CrawlerProxyPool::new(proxies, ProxySelectionStrategy::Cyclic);

        assert_eq!(pool.proxy_count().await, 3);
        let removed = pool.prune_dead().await;
        assert_eq!(removed, 1);
        assert_eq!(pool.proxy_count().await, 2);
    }

    #[tokio::test]
    async fn test_fastest_picks_lowest_latency() {
        let pool = CrawlerProxyPool::new(make_proxies(), ProxySelectionStrategy::Fastest);

        let proxy = pool
            .select_proxy("http://example.com")
            .await
            .expect("should return proxy");
        assert_eq!(proxy.url, "http://proxy-c:8080");
        assert_eq!(proxy.latency_ms, 50);
    }

    #[tokio::test]
    async fn test_random_returns_valid_proxy() {
        let pool = CrawlerProxyPool::new(make_proxies(), ProxySelectionStrategy::Random);

        for _ in 0..20 {
            let proxy = pool
                .select_proxy("http://example.com")
                .await
                .expect("should return proxy");
            assert!(proxy.url.contains("proxy-"));
            assert!([
                "http://proxy-a:8080",
                "http://proxy-b:8080",
                "http://proxy-c:8080",
            ]
            .contains(&proxy.url.as_str()));
        }
    }

    #[test]
    fn test_strategy_conversion_roundtrip() {
        use super::RotationStrategy;

        let cases = vec![
            (RotationStrategy::Cyclic, ProxySelectionStrategy::Cyclic),
            (RotationStrategy::Random, ProxySelectionStrategy::Random),
            (
                RotationStrategy::Performance,
                ProxySelectionStrategy::Fastest,
            ),
            (RotationStrategy::Adaptive, ProxySelectionStrategy::Adaptive),
        ];

        for (rot, sel) in cases {
            let converted: ProxySelectionStrategy = rot.into();
            assert_eq!(converted, sel);
            let back: RotationStrategy = sel.into();
            assert_eq!(back, rot);
        }

        // GeoHinted maps to Cyclic on the RotationStrategy side
        let geo: RotationStrategy = ProxySelectionStrategy::GeoHinted.into();
        assert_eq!(geo, RotationStrategy::Cyclic);
    }

    #[test]
    fn test_guess_region_hint() {
        assert_eq!(guess_region_hint("http://example.com/page"), Some("us"));
        assert_eq!(guess_region_hint("https://example.de"), Some("eu"));
        assert_eq!(guess_region_hint("https://example.jp/"), Some("ap"));
        assert_eq!(guess_region_hint("https://example.cn/foo"), Some("ap"));
        assert_eq!(guess_region_hint("https://example.br"), Some("sa"));
        assert_eq!(guess_region_hint("https://example.au"), Some("oceania"));
        assert_eq!(guess_region_hint("https://example.za"), Some("africa"));
        assert_eq!(guess_region_hint("https://example.ae"), Some("me"));
        assert_eq!(guess_region_hint("https://example.xyz"), None);
    }
}
