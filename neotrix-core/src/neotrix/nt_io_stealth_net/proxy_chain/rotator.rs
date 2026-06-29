use rand::Rng;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum RotationStrategy {
    Cyclic,
    Random,
    Performance,
    Adaptive,
}

#[derive(Debug, Clone)]
pub struct ProxyEntry {
    pub url: String,
    pub protocol: String,
    pub region: Option<String>,
    pub health: f64,
    pub last_used: Instant,
    pub latency_ms: u64,
    pub failures: u8,
    pub max_failures: u8,
}

impl ProxyEntry {
    pub fn new(url: &str, protocol: &str) -> Self {
        ProxyEntry {
            url: url.to_string(),
            protocol: protocol.to_string(),
            region: None,
            health: 1.0,
            last_used: Instant::now(),
            latency_ms: 0,
            failures: 0,
            max_failures: 3,
        }
    }

    pub fn with_region(mut self, region: &str) -> Self {
        self.region = Some(region.to_string());
        self
    }

    pub fn with_max_failures(mut self, max: u8) -> Self {
        self.max_failures = max;
        self
    }
}

#[derive(Debug, Clone)]
pub struct RotatorMetrics {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub average_latency_ms: f64,
    pub proxy_switches: usize,
}

impl Default for RotatorMetrics {
    fn default() -> Self {
        RotatorMetrics {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_latency_ms: 0.0,
            proxy_switches: 0,
        }
    }
}

pub struct ProxyRotator {
    pub proxies: Vec<ProxyEntry>,
    pub strategy: RotationStrategy,
    pub current_index: usize,
    pub metrics: RotatorMetrics,
}

impl ProxyRotator {
    pub fn new(proxies: Vec<ProxyEntry>, strategy: RotationStrategy) -> Self {
        let count = proxies.len();
        ProxyRotator {
            proxies,
            strategy,
            current_index: if count == 0 { 0 } else { count - 1 },
            metrics: RotatorMetrics::default(),
        }
    }

    pub fn next(&mut self) -> Option<&ProxyEntry> {
        if self.proxies.is_empty() {
            return None;
        }

        self.current_index = match self.strategy {
            RotationStrategy::Cyclic => (self.current_index + 1) % self.proxies.len(),
            RotationStrategy::Random => {
                let mut rng = rand::thread_rng();
                rng.gen_range(0..self.proxies.len())
            }
            RotationStrategy::Performance => {
                let mut best_idx = 0;
                let mut best_lat = u64::MAX;
                for (i, p) in self.proxies.iter().enumerate() {
                    if p.latency_ms < best_lat {
                        best_lat = p.latency_ms;
                        best_idx = i;
                    }
                }
                best_idx
            }
            RotationStrategy::Adaptive => {
                if self.current_index >= self.proxies.len() {
                    0
                } else {
                    self.current_index
                }
            }
        };

        self.proxies[self.current_index].last_used = Instant::now();
        self.metrics.total_requests += 1;

        Some(&self.proxies[self.current_index])
    }

    pub fn record_success(&mut self, latency_ms: u64) {
        self.metrics.successful_requests += 1;
        if self.proxies.is_empty() {
            return;
        }
        if let Some(entry) = self.proxies.get_mut(self.current_index) {
            entry.latency_ms = (entry.latency_ms * 3 + latency_ms) / 4;
            entry.health = (entry.health * 9.0 + 1.0) / 10.0;
            entry.health = entry.health.min(1.0);
            entry.failures = 0;
        }
        let total = self.metrics.successful_requests + self.metrics.failed_requests;
        self.metrics.average_latency_ms = (self.metrics.average_latency_ms
            * (total.saturating_sub(1)) as f64
            + latency_ms as f64)
            / total as f64;
    }

    pub fn record_failure(&mut self) {
        self.metrics.failed_requests += 1;
        if self.proxies.is_empty() {
            return;
        }
        if let Some(entry) = self.proxies.get_mut(self.current_index) {
            entry.failures += 1;
            entry.health = (entry.health * 10.0 - 1.0) / 10.0;
            entry.health = entry.health.max(0.0);
            if entry.failures >= entry.max_failures {
                entry.health = 0.0;
            }
        }
        if self.strategy == RotationStrategy::Adaptive {
            if !self.proxies.is_empty() {
                self.current_index = (self.current_index + 1) % self.proxies.len();
            }
            self.metrics.proxy_switches += 1;
        }
    }

    pub fn health_report(&self) -> Vec<(String, f64)> {
        self.proxies
            .iter()
            .map(|p| (p.url.clone(), p.health))
            .collect()
    }

    pub fn prune_dead(&mut self) -> usize {
        let before = self.proxies.len();
        self.proxies.retain(|p| p.health > 0.0);
        if self.current_index >= self.proxies.len() && !self.proxies.is_empty() {
            self.current_index = 0;
        }
        before - self.proxies.len()
    }

    pub fn add_proxy(&mut self, url: &str, protocol: &str, region: Option<&str>) {
        let mut entry = ProxyEntry::new(url, protocol);
        if let Some(r) = region {
            entry = entry.with_region(r);
        }
        self.proxies.push(entry);
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.metrics.successful_requests + self.metrics.failed_requests;
        if total == 0 {
            1.0
        } else {
            self.metrics.successful_requests as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_cyclic_round_robin() {
        let mut rotator = ProxyRotator::new(make_proxies(), RotationStrategy::Cyclic);
        let first = rotator
            .next()
            .expect("rotator should return proxy")
            .url
            .clone();
        let second = rotator
            .next()
            .expect("rotator should return proxy")
            .url
            .clone();
        let third = rotator
            .next()
            .expect("rotator should return proxy")
            .url
            .clone();
        let fourth = rotator
            .next()
            .expect("rotator should return proxy")
            .url
            .clone();
        assert_ne!(first, second, "Cyclic should advance");
        assert_eq!(first, fourth, "Cyclic should wrap around after 3 calls");
        assert_eq!(rotator.metrics.total_requests, 4);
    }

    #[test]
    fn test_random_returns_valid_proxy() {
        let mut rotator = ProxyRotator::new(make_proxies(), RotationStrategy::Random);
        for _ in 0..20 {
            let proxy = rotator.next();
            assert!(proxy.is_some());
            let entry = proxy.expect("proxy should be Some");
            assert!(entry.url.contains("proxy-"));
            assert!([
                "http://proxy-a:8080",
                "http://proxy-b:8080",
                "http://proxy-c:8080"
            ]
            .contains(&entry.url.as_str()));
        }
    }

    #[test]
    fn test_performance_picks_lowest_latency() {
        let mut rotator = ProxyRotator::new(make_proxies(), RotationStrategy::Performance);
        let proxy = rotator.next().expect("rotator should return proxy");
        assert_eq!(proxy.url, "http://proxy-c:8080");
        assert_eq!(proxy.latency_ms, 50);
    }

    #[test]
    fn test_adaptive_stays_on_same_proxy() {
        let mut rotator = ProxyRotator::new(make_proxies(), RotationStrategy::Adaptive);
        let first = rotator
            .next()
            .expect("rotator should return proxy")
            .url
            .clone();
        let second = rotator
            .next()
            .expect("rotator should return proxy")
            .url
            .clone();
        assert_eq!(first, second, "Adaptive should stay on same proxy");
    }

    #[test]
    fn test_record_success_updates_metrics() {
        let mut rotator = ProxyRotator::new(make_proxies(), RotationStrategy::Cyclic);
        rotator.next();
        rotator.record_success(150);
        assert_eq!(rotator.metrics.successful_requests, 1);
        assert_eq!(rotator.metrics.total_requests, 1);
        assert!((rotator.metrics.average_latency_ms - 150.0).abs() < 0.1);
    }

    #[test]
    fn test_record_failure_decrements_health() {
        let mut rotator = ProxyRotator::new(make_proxies(), RotationStrategy::Cyclic);
        let proxy = rotator.next().expect("rotator should return proxy");
        let initial_health = proxy.health;
        drop(proxy);
        rotator.record_failure();
        let updated = &rotator.proxies[rotator.current_index];
        assert!(
            updated.health < initial_health,
            "Health should decrease after failure"
        );
        assert_eq!(rotator.metrics.failed_requests, 1);
    }

    #[test]
    fn test_adaptive_switches_on_failure() {
        let mut rotator = ProxyRotator::new(make_proxies(), RotationStrategy::Adaptive);
        let first = rotator
            .next()
            .expect("rotator should return proxy")
            .url
            .clone();
        rotator.record_failure();
        let second = rotator
            .next()
            .expect("rotator should return proxy")
            .url
            .clone();
        assert_ne!(first, second, "Adaptive should switch proxy on failure");
        assert_eq!(rotator.metrics.proxy_switches, 1);
    }

    #[test]
    fn test_health_report_format() {
        let rotator = ProxyRotator::new(make_proxies(), RotationStrategy::Cyclic);
        let report = rotator.health_report();
        assert_eq!(report.len(), 3);
        for (url, health) in &report {
            assert!(url.contains("proxy-"));
            assert!(*health > 0.0 && *health <= 1.0);
        }
    }

    #[test]
    fn test_prune_dead_removes_health_zero() {
        let mut proxies = make_proxies();
        proxies[0].health = 0.0;
        proxies[1].health = 0.0;
        let mut rotator = ProxyRotator::new(proxies, RotationStrategy::Cyclic);
        let removed = rotator.prune_dead();
        assert_eq!(removed, 2);
        assert_eq!(rotator.proxies.len(), 1);
        assert_eq!(rotator.proxies[0].url, "http://proxy-c:8080");
    }

    #[test]
    fn test_add_new_proxy() {
        let mut rotator = ProxyRotator::new(Vec::new(), RotationStrategy::Cyclic);
        assert!(rotator.next().is_none());
        rotator.add_proxy("http://new-proxy:3128", "http", Some("us-west"));
        assert_eq!(rotator.proxies.len(), 1);
        let p = rotator.next().expect("rotator should return new proxy");
        assert_eq!(p.url, "http://new-proxy:3128");
        assert_eq!(p.health, 1.0);
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut rotator = ProxyRotator::new(make_proxies(), RotationStrategy::Cyclic);
        assert!((rotator.success_rate() - 1.0).abs() < 0.01);
        rotator.next();
        rotator.record_success(100);
        assert!((rotator.success_rate() - 1.0).abs() < 0.01);
        rotator.next();
        rotator.record_failure();
        assert!((rotator.success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_health_drops_to_zero_on_max_failures() {
        let entry = ProxyEntry::new("http://bad-proxy:8080", "http").with_max_failures(2);
        let mut rotator = ProxyRotator::new(vec![entry], RotationStrategy::Cyclic);
        rotator.next();
        rotator.record_failure();
        rotator.record_failure();
        assert_eq!(rotator.proxies[0].health, 0.0);
    }

    #[test]
    fn test_empty_proxy_list_returns_none() {
        let mut rotator = ProxyRotator::new(Vec::new(), RotationStrategy::Cyclic);
        assert!(rotator.next().is_none());
    }

    #[test]
    fn test_cyclic_returns_all_proxies() {
        let mut rotator = ProxyRotator::new(make_proxies(), RotationStrategy::Cyclic);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..6 {
            let url = rotator
                .next()
                .expect("rotator should return proxy")
                .url
                .clone();
            seen.insert(url);
        }
        assert_eq!(seen.len(), 3);
    }
}
