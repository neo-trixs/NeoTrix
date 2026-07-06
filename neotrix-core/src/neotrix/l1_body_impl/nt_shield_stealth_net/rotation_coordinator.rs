use rand::Rng;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use super::config::load as cfg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RotationDomain {
    TlsFingerprint,
    HttpHeaders,
    ProxyChain,
    SourceIp,
    TorCircuit,
    TimingPattern,
}

impl RotationDomain {
    pub fn all() -> &'static [RotationDomain] {
        &[
            RotationDomain::TlsFingerprint,
            RotationDomain::HttpHeaders,
            RotationDomain::ProxyChain,
            RotationDomain::SourceIp,
            RotationDomain::TorCircuit,
            RotationDomain::TimingPattern,
        ]
    }

    fn mean_multiplier(&self) -> f64 {
        match self {
            RotationDomain::TlsFingerprint => 1.0,
            RotationDomain::HttpHeaders => 0.75,
            RotationDomain::ProxyChain => 1.0,
            RotationDomain::SourceIp => 1.3,
            RotationDomain::TorCircuit => 1.5,
            RotationDomain::TimingPattern => 1.2,
        }
    }
}

struct DomainState {
    mean_ms: f64,
    std_dev_ms: f64,
    #[allow(dead_code)]
    phase_offset_ms: u64,
    last_rotation: Instant,
    rotation_count: AtomicU64,
    next_interval_ms: u64,
}

pub struct RotationCoordinator {
    domains: RwLock<Vec<(RotationDomain, DomainState)>>,
    global_tick: AtomicU64,
}

impl std::fmt::Debug for RotationCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RotationCoordinator")
            .field("global_tick", &self.global_tick.load(Ordering::Relaxed))
            .finish()
    }
}

fn gaussian_sample(mean: f64, std_dev: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    mean + z * std_dev
}

fn clamp_interval(val: f64, min_ms: u64, max_ms: u64) -> u64 {
    val.round().max(min_ms as f64).min(max_ms as f64) as u64
}

impl RotationCoordinator {
    /// 从 TOML 配置创建（使用 config.rotation.gaussian_mean_secs）
    pub fn from_config() -> Arc<Self> {
        let c = cfg();
        let mean_ms = c.rotation.gaussian_mean_secs * 1000.0;
        let std_ms = c.rotation.gaussian_std_dev_secs * 1000.0;
        let max_ms = (c.rotation.max_interval_secs * 1000.0) as u64;
        let mut rng = rand::thread_rng();
        let mut domains = Vec::new();

        for &domain in RotationDomain::all() {
            let mult = domain.mean_multiplier();
            let dmean = mean_ms * mult;
            let dstd = std_ms * mult;
            let offset = rng.gen_range(0..max_ms);
            let interval = Self::sample_interval(dmean, dstd);
            domains.push((domain, DomainState {
                mean_ms: dmean,
                std_dev_ms: dstd,
                phase_offset_ms: offset,
                last_rotation: Instant::now(),
                rotation_count: AtomicU64::new(0),
                next_interval_ms: interval,
            }));
        }

        Arc::new(Self {
            domains: RwLock::new(domains),
            global_tick: AtomicU64::new(0),
        })
    }

    pub fn new() -> Arc<Self> {
        Self::from_config()
    }

    pub async fn next_interval_ms(&self, domain: RotationDomain) -> u64 {
        let domains = self.domains.read().await;
        for (d, state) in domains.iter() {
            if *d == domain {
                return state.next_interval_ms;
            }
        }
        Self::sample_interval(7500.0, 2500.0)
    }

    pub async fn should_rotate(&self, domain: RotationDomain) -> bool {
        let c = cfg();
        let max_ms = (c.rotation.max_interval_secs * 1000.0) as u64;
        let domains = self.domains.read().await;
        for (d, state) in domains.iter() {
            if *d == domain {
                let elapsed = state.last_rotation.elapsed().as_millis() as u64;
                let interval = state.next_interval_ms;
                if elapsed >= interval || elapsed >= max_ms {
                    return true;
                }
                return false;
            }
        }
        false
    }

    pub async fn seconds_until_rotation(&self, domain: RotationDomain) -> f64 {
        let domains = self.domains.read().await;
        for (d, state) in domains.iter() {
            if *d == domain {
                let elapsed = state.last_rotation.elapsed().as_millis() as u64;
                let remaining = state.next_interval_ms.saturating_sub(elapsed);
                return remaining as f64 / 1000.0;
            }
        }
        60.0
    }

    fn sample_interval(mean_ms: f64, std_dev_ms: f64) -> u64 {
        let c = cfg();
        let min_ms = (c.rotation.min_interval_secs * 1000.0) as u64;
        let max_ms = (c.rotation.max_interval_secs * 1000.0) as u64;
        let sample = gaussian_sample(mean_ms, std_dev_ms);
        clamp_interval(sample, min_ms, max_ms)
    }

    /// 动态更新域的均值/std（供 bandit 置信度自适应耦合）
    pub async fn set_domain_params(&self, domain: RotationDomain, mean_ms: f64, std_dev_ms: f64) {
        let mut domains = self.domains.write().await;
        for (d, state) in domains.iter_mut() {
            if *d == domain {
                state.mean_ms = mean_ms;
                state.std_dev_ms = std_dev_ms;
                state.next_interval_ms = Self::sample_interval(mean_ms, std_dev_ms);
                return;
            }
        }
    }

    pub async fn mark_rotated(&self, domain: RotationDomain) {
        let mut domains = self.domains.write().await;
        for (d, state) in domains.iter_mut() {
            if *d == domain {
                state.rotation_count.fetch_add(1, Ordering::Relaxed);
                state.last_rotation = Instant::now();
                state.next_interval_ms = Self::sample_interval(state.mean_ms, state.std_dev_ms);
                return;
            }
        }
    }

    pub async fn rotation_count(&self, domain: RotationDomain) -> u64 {
        let domains = self.domains.read().await;
        for (d, state) in domains.iter() {
            if *d == domain {
                return state.rotation_count.load(Ordering::Relaxed);
            }
        }
        0
    }

    pub fn tick(&self) -> u64 {
        self.global_tick.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn summary(&self) -> Vec<(RotationDomain, u64, u64, u64)> {
        let domains = self.domains.read().await;
        domains.iter().map(|(d, state)| {
            let elapsed = state.last_rotation.elapsed().as_millis() as u64;
            (*d, state.mean_ms as u64, elapsed, state.rotation_count.load(Ordering::Relaxed))
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let coord = RotationCoordinator::new();
        let summary = coord.summary().await;
        assert_eq!(summary.len(), 6);
    }

    #[tokio::test]
    async fn test_gaussian_interval_bounds() {
        let coord = RotationCoordinator::new();
        for _ in 0..200 {
            let i = coord.next_interval_ms(RotationDomain::TlsFingerprint).await;
            assert!(i >= 2000, "interval {} below 2000ms", i);
            assert!(i <= 15000, "interval {} above 15000ms", i);
        }
    }

    #[tokio::test]
    async fn test_gaussian_distribution_center() {
        let coord = RotationCoordinator::new();
        let mut sum = 0u64;
        let n = 2000;
        for _ in 0..n {
            sum += coord.next_interval_ms(RotationDomain::TlsFingerprint).await;
        }
        let mean = sum as f64 / n as f64;
        assert!(mean > 5000.0, "mean {} too low", mean);
        assert!(mean < 10000.0, "mean {} too high", mean);
    }

    #[tokio::test]
    async fn test_should_rotate_fresh() {
        let coord = RotationCoordinator::new();
        assert!(!coord.should_rotate(RotationDomain::TlsFingerprint).await);
    }

    #[tokio::test]
    async fn test_mark_rotated() {
        let coord = RotationCoordinator::new();
        coord.mark_rotated(RotationDomain::HttpHeaders).await;
        assert_eq!(coord.rotation_count(RotationDomain::HttpHeaders).await, 1);
    }

    #[tokio::test]
    async fn test_domain_phase_independence() {
        let coord = RotationCoordinator::new();
        coord.mark_rotated(RotationDomain::ProxyChain).await;
        coord.mark_rotated(RotationDomain::TlsFingerprint).await;
        assert_eq!(coord.rotation_count(RotationDomain::ProxyChain).await, 1);
        assert_eq!(coord.rotation_count(RotationDomain::TlsFingerprint).await, 1);
        assert_eq!(coord.rotation_count(RotationDomain::SourceIp).await, 0);
    }

    #[tokio::test]
    async fn test_all_domains_within_15s() {
        let coord = RotationCoordinator::new();
        for &domain in RotationDomain::all() {
            for _ in 0..100 {
                let i = coord.next_interval_ms(domain).await;
                assert!(i <= 15000, "domain {:?} interval {} exceeds 15s", domain, i);
                assert!(i >= 2000, "domain {:?} interval {} below 2s", domain, i);
            }
        }
    }
}
