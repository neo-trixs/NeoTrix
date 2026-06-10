use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

use super::local_proxy::TorManager;
pub use super::tor_client::IsolationToken;

const DEFAULT_TOKENS: usize = 4;
const DEFAULT_ROTATION_INTERVAL: usize = 30;
const NEWNYM_COOLDOWN_SECS: u64 = 10;

#[derive(Debug, Clone)]
pub struct CircuitIsolationConfig {
    pub pool_size: usize,
    pub rotation_interval: usize,
    pub newnym_cooldown: Duration,
}

impl Default for CircuitIsolationConfig {
    fn default() -> Self {
        Self {
            pool_size: DEFAULT_TOKENS,
            rotation_interval: DEFAULT_ROTATION_INTERVAL,
            newnym_cooldown: Duration::from_secs(NEWNYM_COOLDOWN_SECS),
        }
    }
}

pub struct CircuitIsolationManager {
    tokens: RwLock<Vec<IsolationToken>>,
    current: AtomicUsize,
    request_count: AtomicUsize,
    config: CircuitIsolationConfig,
    tor_manager: RwLock<Option<Arc<TorManager>>>,
    last_newnym: Mutex<Instant>,
}

pub fn global_circuit_manager() -> &'static CircuitIsolationManager {
    static CM: OnceLock<CircuitIsolationManager> = OnceLock::new();
    CM.get_or_init(|| CircuitIsolationManager::new(CircuitIsolationConfig::default(), None))
}

impl CircuitIsolationManager {
    pub fn new(config: CircuitIsolationConfig, tor_manager: Option<Arc<TorManager>>) -> Self {
        let tokens: Vec<IsolationToken> = (0..config.pool_size).map(|_| IsolationToken::new()).collect();
        Self {
            tokens: RwLock::new(tokens),
            current: AtomicUsize::new(0),
            request_count: AtomicUsize::new(0),
            config,
            tor_manager: RwLock::new(tor_manager),
            last_newnym: Mutex::new(Instant::now()),
        }
    }

    pub async fn set_tor_manager(&self, tm: Arc<TorManager>) {
        *self.tor_manager.write().await = Some(tm);
    }

    pub async fn acquire(&self) -> IsolationToken {
        let count = self.request_count.fetch_add(1, Ordering::Relaxed) + 1;

        if count % self.config.rotation_interval == 0 {
            self.rotate().await;
        }

        let tokens = self.tokens.read().await;
        if tokens.is_empty() {
            return IsolationToken::new();
        }
        let idx = self.current.fetch_add(1, Ordering::Relaxed) % tokens.len();
        tokens[idx]
    }

    pub async fn acquire_batch(&self, n: usize) -> Vec<IsolationToken> {
        let mut tokens = Vec::with_capacity(n);
        for _ in 0..n {
            tokens.push(self.acquire().await);
        }
        tokens
    }

    pub async fn rotate(&self) {
        let mut pool = self.tokens.write().await;
        let new_pool: Vec<IsolationToken> = (0..self.config.pool_size)
            .map(|_| IsolationToken::new())
            .collect();
        *pool = new_pool;
        self.current.store(0, Ordering::Relaxed);

        if let Some(ref tm) = *self.tor_manager.read().await {
            let mut last = self.last_newnym.lock().unwrap();
            if last.elapsed() >= self.config.newnym_cooldown {
                let tm_clone = tm.clone();
                tokio::spawn(async move {
                    if let Err(e) = tm_clone.new_identity().await {
                        log::warn!("[circuit] NEWNYM failed: {}", e);
                    } else {
                        log::info!("[circuit] rotated {} tokens, NEWNYM issued", self.config.pool_size);
                    }
                });
                *last = Instant::now();
            }
        }
    }

    pub async fn force_rotate(&self) {
        self.request_count.store(0, Ordering::Relaxed);
        self.rotate().await;
    }

    pub fn tor_socks_username(&self, token: &IsolationToken) -> String {
        format!("neotrix-{:x}", token.value())
    }

    pub fn tor_socks_password(&self, _token: &IsolationToken) -> String {
        String::new()
    }

    pub fn token_count(&self) -> usize {
        self.config.pool_size
    }

    pub async fn tokens(&self) -> Vec<IsolationToken> {
        self.tokens.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_config_defaults() {
        let cfg = CircuitIsolationConfig::default();
        assert_eq!(cfg.pool_size, 4);
        assert_eq!(cfg.rotation_interval, 30);
        assert_eq!(cfg.newnym_cooldown, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_acquire_returns_token() {
        let manager = CircuitIsolationManager::new(CircuitIsolationConfig::default(), None);
        let token = manager.acquire().await;
        let token2 = manager.acquire().await;
        assert_ne!(token, token2, "sequential acquires should differ");
    }

    #[tokio::test]
    async fn test_acquire_round_robin() {
        let config = CircuitIsolationConfig {
            pool_size: 3,
            rotation_interval: 100,
            newnym_cooldown: Duration::from_secs(1),
        };
        let manager = CircuitIsolationManager::new(config, None);

        let a = manager.acquire().await;
        let b = manager.acquire().await;
        let c = manager.acquire().await;
        let d = manager.acquire().await;

        assert_eq!(a, d, "round-robin should cycle after pool_size acquires");
        assert_ne!(a, b);
        assert_ne!(b, c);
    }

    #[tokio::test]
    async fn test_rotate_generates_new_tokens() {
        let config = CircuitIsolationConfig {
            pool_size: 2,
            rotation_interval: 100,
            newnym_cooldown: Duration::from_secs(0),
        };
        let manager = CircuitIsolationManager::new(config, None);

        let before: Vec<IsolationToken> = manager.tokens().await;
        manager.rotate().await;
        let after: Vec<IsolationToken> = manager.tokens().await;

        assert_eq!(before.len(), after.len());
        for t in &before {
            assert!(!after.contains(t), "all tokens should be new after rotation");
        }
    }

    #[tokio::test]
    async fn test_force_rotate_resets_counter() {
        let config = CircuitIsolationConfig {
            pool_size: 2,
            rotation_interval: 100,
            newnym_cooldown: Duration::from_secs(0),
        };
        let manager = CircuitIsolationManager::new(config, None);
        manager.acquire().await;
        assert_eq!(manager.request_count.load(Ordering::Relaxed), 1);

        manager.force_rotate().await;
        assert_eq!(manager.request_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_tor_socks_username_format() {
        let config = CircuitIsolationConfig::default();
        let manager = CircuitIsolationManager::new(config, None);
        let token = IsolationToken::new();
        let username = manager.tor_socks_username(&token);
        assert!(username.starts_with("neotrix-"));
        assert!(username.len() > 8);
    }

    #[tokio::test]
    async fn test_acquire_batch() {
        let manager = CircuitIsolationManager::new(CircuitIsolationConfig::default(), None);
        let batch = manager.acquire_batch(5).await;
        assert_eq!(batch.len(), 5);
        let mut unique = batch.clone();
        unique.sort();
        unique.dedup();
        assert!(unique.len() > 1, "batch should contain diverse tokens");
    }
}
