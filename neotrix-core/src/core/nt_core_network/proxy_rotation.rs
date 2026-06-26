use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ProxyProtocol {
    Http,
    Https,
    Socks4,
    Socks5,
}

impl ProxyProtocol {
    pub fn scheme(&self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Socks4 => "socks4",
            Self::Socks5 => "socks5",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
    pub protocol: ProxyProtocol,
    pub auth: Option<ProxyAuth>,
    pub label: String,
    pub region: Option<String>,
    pub weight: u32,
    pub max_failures: u32,
}

impl ProxyConfig {
    pub fn new(host: &str, port: u16, protocol: ProxyProtocol) -> Self {
        Self {
            host: host.to_string(),
            port,
            protocol,
            auth: None,
            label: format!("{}://{}:{}", protocol.scheme(), host, port),
            region: None,
            weight: 1,
            max_failures: 3,
        }
    }

    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.auth = Some(ProxyAuth {
            username: username.to_string(),
            password: password.to_string(),
        });
        self
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn with_region(mut self, region: &str) -> Self {
        self.region = Some(region.to_string());
        self
    }

    pub fn url(&self) -> String {
        let base = format!("{}://{}:{}", self.protocol.scheme(), self.host, self.port);
        if let Some(ref auth) = self.auth {
            format!(
                "{}://{}:{}@{}:{}",
                self.protocol.scheme(),
                auth.username,
                auth.password,
                self.host,
                self.port
            )
        } else {
            base
        }
    }

    pub fn to_reqwest_proxy(&self) -> Result<reqwest::Proxy, String> {
        let url = self.url();
        reqwest::Proxy::all(&url).map_err(|e| format!("invalid proxy {}: {}", url, e))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RotationStrategy {
    RoundRobin,
    Random,
    LeastUsed,
}

impl RotationStrategy {
    pub fn variants() -> &'static [Self] {
        &[Self::RoundRobin, Self::Random, Self::LeastUsed]
    }
}

#[derive(Debug, Clone)]
struct ProxyState {
    config: ProxyConfig,
    failure_count: u32,
    last_used: Option<Instant>,
    use_count: u64,
    consecutive_failures: u32,
    success_count: u64,
}

#[derive(Debug, Clone)]
pub struct ProxyPool {
    proxies: Vec<ProxyState>,
    capacity: usize,
}

impl ProxyPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            proxies: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add(&mut self, config: ProxyConfig) -> Result<(), String> {
        if self.proxies.len() >= self.capacity {
            return Err(format!("proxy pool full (capacity: {})", self.capacity));
        }
        if self.proxies.iter().any(|p| p.config.label == config.label) {
            return Err(format!("proxy '{}' already exists", config.label));
        }
        self.proxies.push(ProxyState {
            config,
            failure_count: 0,
            last_used: None,
            use_count: 0,
            consecutive_failures: 0,
            success_count: 0,
        });
        Ok(())
    }

    pub fn remove(&mut self, label: &str) -> bool {
        let idx = self.proxies.iter().position(|p| p.config.label == label);
        if let Some(i) = idx {
            self.proxies.swap_remove(i);
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.proxies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.proxies.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ProxyConfig> {
        self.proxies.iter().map(|s| &s.config)
    }

    pub fn record_success(&mut self, label: &str) {
        if let Some(p) = self.proxies.iter_mut().find(|p| p.config.label == label) {
            p.consecutive_failures = 0;
            p.success_count += 1;
        }
    }

    pub fn record_failure(&mut self, label: &str) {
        if let Some(p) = self.proxies.iter_mut().find(|p| p.config.label == label) {
            p.failure_count += 1;
            p.consecutive_failures += 1;
        }
    }

    pub fn healthy_proxies(&self) -> Vec<&ProxyConfig> {
        self.proxies
            .iter()
            .filter(|p| p.consecutive_failures < p.config.max_failures)
            .map(|p| &p.config)
            .collect()
    }

    pub fn reset_failures(&mut self, label: &str) {
        if let Some(p) = self.proxies.iter_mut().find(|p| p.config.label == label) {
            p.failure_count = 0;
            p.consecutive_failures = 0;
        }
    }

    fn weighted_random_index(&self) -> Option<usize> {
        let healthy: Vec<(usize, u32)> = self
            .proxies
            .iter()
            .enumerate()
            .filter(|(_, p)| p.consecutive_failures < p.config.max_failures)
            .map(|(i, p)| (i, p.config.weight))
            .collect();
        if healthy.is_empty() {
            return None;
        }
        let total: u32 = healthy.iter().map(|(_, w)| w).sum();
        if total == 0 {
            return Some(healthy[0].0);
        }
        let mut rng = rand::thread_rng();
        let threshold = rng.gen_range(0..total);
        let mut cumulative = 0;
        for (idx, weight) in &healthy {
            cumulative += weight;
            if threshold < cumulative {
                return Some(*idx);
            }
        }
        healthy.last().map(|(i, _)| *i)
    }
}

pub struct ProxyRotator {
    pool: Arc<RwLock<ProxyPool>>,
    strategy: RwLock<RotationStrategy>,
    rr_index: AtomicUsize,
}

impl ProxyRotator {
    pub fn new(pool: ProxyPool, strategy: RotationStrategy) -> Self {
        Self {
            pool: Arc::new(RwLock::new(pool)),
            strategy: RwLock::new(strategy),
            rr_index: AtomicUsize::new(0),
        }
    }

    pub fn pool(&self) -> &Arc<RwLock<ProxyPool>> {
        &self.pool
    }

    pub async fn set_strategy(&self, strategy: RotationStrategy) {
        *self.strategy.write().await = strategy;
    }

    pub async fn next(&self) -> Option<ProxyConfig> {
        let pool = self.pool.read().await;
        if pool.is_empty() {
            return None;
        }
        let strategy = *self.strategy.read().await;
        match strategy {
            RotationStrategy::RoundRobin => {
                let healthy: Vec<usize> = pool
                    .healthy_proxies()
                    .into_iter()
                    .filter_map(|c| pool.iter().position(|p| p.label == c.label))
                    .collect();
                if healthy.is_empty() {
                    return pool.iter().next().cloned();
                }
                let idx = self.rr_index.fetch_add(1, Ordering::Relaxed) % healthy.len();
                let actual = healthy[idx];
                pool.proxies.get(actual).map(|s| s.config.clone())
            }
            RotationStrategy::Random => {
                let healthy = pool.healthy_proxies();
                if healthy.is_empty() {
                    return pool.iter().next().cloned();
                }
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..healthy.len());
                Some(healthy[idx].clone())
            }
            RotationStrategy::LeastUsed => {
                let healthy: Vec<&ProxyState> = pool
                    .proxies
                    .iter()
                    .filter(|p| p.consecutive_failures < p.config.max_failures)
                    .collect();
                if healthy.is_empty() {
                    return pool.iter().next().cloned();
                }
                let min_use = healthy.iter().map(|p| p.use_count).min().unwrap_or(0);
                let candidates: Vec<&ProxyConfig> = healthy
                    .iter()
                    .filter(|p| p.use_count == min_use)
                    .map(|p| &p.config)
                    .collect();
                let mut rng = rand::thread_rng();
                Some(candidates[rng.gen_range(0..candidates.len())].clone())
            }
        }
    }

    pub async fn build_next_client(
        &self,
        tls_cfg: &super::tls_fingerprint::TlsFingerprintConfig,
    ) -> Result<reqwest::Client, String> {
        let proxy = self.next().await;
        let reqwest_proxy = match proxy {
            Some(ref p) => {
                let rp = p.to_reqwest_proxy()?;
                Some(rp)
            }
            None => None,
        };
        tls_cfg.build_reqwest_client(reqwest_proxy)
    }

    pub async fn record_success(&self, label: &str) {
        self.pool.write().await.record_success(label);
    }

    pub async fn record_failure(&self, label: &str) {
        self.pool.write().await.record_failure(label);
    }
}

pub fn global_proxy_rotator() -> &'static ProxyRotator {
    use std::sync::OnceLock;
    static ROTATOR: OnceLock<ProxyRotator> = OnceLock::new();
    ROTATOR.get_or_init(|| {
        ProxyRotator::new(ProxyPool::new(64), RotationStrategy::RoundRobin)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_proxy(label: &str, port: u16) -> ProxyConfig {
        ProxyConfig::new("127.0.0.1", port, ProxyProtocol::Http).with_label(label)
    }

    #[test]
    fn test_proxy_config_url() {
        let p = ProxyConfig::new("proxy.example.com", 8080, ProxyProtocol::Http);
        assert_eq!(p.url(), "http://proxy.example.com:8080");
    }

    #[test]
    fn test_proxy_config_url_with_auth() {
        let p = ProxyConfig::new("p.com", 3128, ProxyProtocol::Https)
            .with_auth("user", "pass");
        assert_eq!(p.url(), "https://user:pass@p.com:3128");
    }

    #[test]
    fn test_proxy_pool_add_and_remove() {
        let mut pool = ProxyPool::new(10);
        assert!(pool.add(test_proxy("p1", 8000)).is_ok());
        assert!(pool.add(test_proxy("p2", 8001)).is_ok());
        assert_eq!(pool.len(), 2);
        assert!(pool.remove("p1"));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_proxy_pool_duplicate_label() {
        let mut pool = ProxyPool::new(10);
        assert!(pool.add(test_proxy("p1", 8000)).is_ok());
        assert!(pool.add(test_proxy("p1", 8001)).is_err());
    }

    #[test]
    fn test_proxy_pool_full() {
        let mut pool = ProxyPool::new(2);
        assert!(pool.add(test_proxy("p1", 8000)).is_ok());
        assert!(pool.add(test_proxy("p2", 8001)).is_ok());
        assert!(pool.add(test_proxy("p3", 8002)).is_err());
    }

    #[test]
    fn test_proxy_pool_healthy_after_failures() {
        let mut pool = ProxyPool::new(10);
        let p = test_proxy("p1", 8000);
        assert!(pool.add(p).is_ok());
        assert_eq!(pool.healthy_proxies().len(), 1);
        pool.record_failure("p1");
        pool.record_failure("p1");
        pool.record_failure("p1");
        assert!(pool.healthy_proxies().is_empty());
    }

    #[test]
    fn test_proxy_pool_reset_failures() {
        let mut pool = ProxyPool::new(10);
        assert!(pool.add(test_proxy("p1", 8000)).is_ok());
        pool.record_failure("p1");
        pool.reset_failures("p1");
        assert_eq!(pool.healthy_proxies().len(), 1);
    }

    #[test]
    fn test_rotation_strategy_variants() {
        let v = RotationStrategy::variants();
        assert_eq!(v.len(), 3);
    }

    #[tokio::test]
    async fn test_rotator_round_robin() {
        let mut pool = ProxyPool::new(10);
        assert!(pool.add(test_proxy("p1", 8000)).is_ok());
        assert!(pool.add(test_proxy("p2", 8001)).is_ok());
        let rotator = ProxyRotator::new(pool, RotationStrategy::RoundRobin);
        let first = rotator.next().await;
        let second = rotator.next().await;
        assert!(first.is_some());
        assert!(second.is_some());
        assert_ne!(first.unwrap().label, second.unwrap().label);
    }

    #[tokio::test]
    async fn test_rotator_empty_pool_returns_none() {
        let pool = ProxyPool::new(10);
        let rotator = ProxyRotator::new(pool, RotationStrategy::Random);
        assert!(rotator.next().await.is_none());
    }

    #[test]
    fn test_proxy_protocol_scheme() {
        assert_eq!(ProxyProtocol::Http.scheme(), "http");
        assert_eq!(ProxyProtocol::Socks5.scheme(), "socks5");
    }

    #[test]
    fn test_proxy_config_with_region() {
        let p = ProxyConfig::new("p.com", 1080, ProxyProtocol::Socks5)
            .with_region("US");
        assert_eq!(p.region, Some("US".to_string()));
    }

    #[test]
    fn test_proxy_config_to_reqwest() {
        let p = ProxyConfig::new("127.0.0.1", 8080, ProxyProtocol::Http);
        assert!(p.to_reqwest_proxy().is_ok());
    }

    #[test]
    fn test_pool_record_success() {
        let mut pool = ProxyPool::new(10);
        assert!(pool.add(test_proxy("p1", 8000)).is_ok());
        pool.record_failure("p1");
        assert_eq!(pool.healthy_proxies().len(), 0);
        pool.record_success("p1");
        assert_eq!(pool.healthy_proxies().len(), 1);
    }

    #[test]
    fn test_proxy_state_use_count() {
        let mut pool = ProxyPool::new(10);
        assert!(pool.add(test_proxy("p1", 8000)).is_ok());
        pool.record_success("p1");
        pool.record_success("p1");
        pool.record_failure("p1");
        let proxies: Vec<_> = pool.iter().collect();
        assert_eq!(proxies.len(), 1);
    }
}
