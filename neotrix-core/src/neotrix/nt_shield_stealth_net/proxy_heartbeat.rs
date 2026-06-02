use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::proxy_pool::{ProxyPool, ProxyNode};
use super::self_iterating::FingerprintManager;

/// One heartbeat tick record — the coordinated rotation snapshot
#[derive(Debug, Clone)]
pub struct HeartbeatRecord {
    pub tick: u64,
    pub timestamp: Instant,
    pub proxy_url: String,
    pub proxy_geo: Option<String>,
    pub proxy_latency_ms: f64,
    pub fingerprint_id: usize,
    pub dns_flushed: bool,
    pub success: bool,
}

/// Summary of current heartbeat state
#[derive(Debug, Clone)]
pub struct HeartbeatSummary {
    pub total_ticks: u64,
    pub last_proxy_url: String,
    pub last_proxy_geo: String,
    pub last_fingerprint_id: usize,
    pub avg_rotation_interval_ms: f64,
    pub recent_success_rate: f64,
    pub rotation_count_last_hour: u64,
}

/// Heartbeat-driven proxy + fingerprint rotator.
///
/// On each tick:
/// 1. Selects a different proxy node (preferring a different geographic region)
/// 2. Rotates the nt_world_browse fingerprint (atomic_rotate → fingerprint + TLS + timing)
/// 3. Flushes OS DNS cache for clean resolution through the new egress
/// 4. Records telemetry for monitoring
pub struct ProxyHeartbeatEngine {
    pool: Arc<ProxyPool>,
    fingerprint_manager: RwLock<FingerprintManager>,
    #[allow(dead_code)]
    heartbeat_interval: Duration,
    last_heartbeat: RwLock<Instant>,
    rotation_count: AtomicU64,
    current_proxy_url: RwLock<String>,
    current_fingerprint_id: RwLock<usize>,
    history: RwLock<VecDeque<HeartbeatRecord>>,
    max_history: usize,
}

impl ProxyHeartbeatEngine {
    pub fn new(
        pool: Arc<ProxyPool>,
        fingerprint_manager: FingerprintManager,
        heartbeat_interval_secs: u64,
    ) -> Self {
        Self {
            pool,
            fingerprint_manager: RwLock::new(fingerprint_manager),
            heartbeat_interval: Duration::from_secs(heartbeat_interval_secs),
            last_heartbeat: RwLock::new(Instant::now()),
            rotation_count: AtomicU64::new(0),
            current_proxy_url: RwLock::new(String::new()),
            current_fingerprint_id: RwLock::new(0),
            history: RwLock::new(VecDeque::new()),
            max_history: 1000,
        }
    }

    /// Execute one heartbeat tick.
    /// Returns a HeartbeatReport with the rotation result.
    pub async fn tick(&self) -> HeartbeatRecord {
        let tick = self.rotation_count.fetch_add(1, Ordering::Relaxed) + 1;
        let timestamp = Instant::now();

        // 1. Rotate proxy — select a different node from the pool
        let (proxy_url, proxy_geo, proxy_latency) = self.rotate_proxy().await;

        // 2. Rotate fingerprint — atomic_rotate switches fingerprint + TLS + timing
        let fingerprint_id = self.rotate_fingerprint().await;

        // 3. Flush DNS cache
        let dns_flushed = flush_dns_cache().await;

        // 4. Record the heartbeat
        let success = !proxy_url.is_empty();
        let record = HeartbeatRecord {
            tick,
            timestamp,
            proxy_url: proxy_url.clone(),
            proxy_geo: proxy_geo.clone(),
            proxy_latency_ms: proxy_latency,
            fingerprint_id,
            dns_flushed,
            success,
        };

        let mut current_url = self.current_proxy_url.write().await;
        *current_url = proxy_url.clone();

        let mut current_fp = self.current_fingerprint_id.write().await;
        *current_fp = fingerprint_id;

        let mut history = self.history.write().await;
        history.push_back(record.clone());
        while history.len() > self.max_history {
            history.pop_front();
        }

        record
    }

    /// Select a new proxy node, preferring a different geographic region than the current one.
    async fn rotate_proxy(&self) -> (String, Option<String>, f64) {
        let current_url = self.current_proxy_url.read().await.clone();
        let nodes = self.pool.all_nodes().await;

        if nodes.is_empty() {
            return (String::new(), None, 0.0);
        }

        // Prefer a node with different geo than current
        if !current_url.is_empty() {
            let current_geo = nodes.iter()
                .find(|n| n.url == current_url)
                .and_then(|n| n.geo_tag.clone());

            if let Some(ref cg) = current_geo {
                if let Some(different) = nodes.iter()
                    .filter(|n| n.geo_tag.as_deref() != Some(cg))
                    .filter(|n| n.latency_ms.is_some())
                    .min_by(|a, b| {
                        a.latency_ms.partial_cmp(&b.latency_ms)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                {
                    let latency = different.latency_ms.unwrap_or(0.0);
                    return (different.url.clone(), different.geo_tag.clone(), latency);
                }
            }
        }

        // Fallback: pick the fastest available node
        if let Some(fastest) = self.pool.select_fastest().await {
            let latency = fastest.latency_ms.unwrap_or(0.0);
            return (fastest.url.clone(), fastest.geo_tag.clone(), latency);
        }

        (String::new(), None, 0.0)
    }

    /// Rotate the nt_world_browse fingerprint via atomic_rotate.
    async fn rotate_fingerprint(&self) -> usize {
        let mut fm = self.fingerprint_manager.write().await;
        fm.atomic_rotate();
        fm.current_index
    }

    /// Return the most recent heartbeat record, if any.
    pub async fn last_heartbeat(&self) -> Option<HeartbeatRecord> {
        self.history.read().await.back().cloned()
    }

    /// Return a summary of heartbeat state.
    pub async fn summary(&self) -> HeartbeatSummary {
        let history = self.history.read().await;
        let total_ticks = self.rotation_count.load(Ordering::Relaxed);
        let last = history.back();

        let (last_proxy_url, last_proxy_geo, last_fingerprint_id) = match last {
            Some(r) => (r.proxy_url.clone(), r.proxy_geo.clone().unwrap_or_default(), r.fingerprint_id),
            None => (String::new(), String::new(), 0),
        };

        let recent_count = history.len().min(10);
        let recent_success = history.iter().rev().take(recent_count).filter(|r| r.success).count();
        let recent_success_rate = if recent_count > 0 { recent_success as f64 / recent_count as f64 } else { 0.0 };

        let avg_interval = if history.len() >= 2 {
            let first = history.front().map(|r| r.timestamp).unwrap_or(Instant::now());
            let last_ts = history.back().map(|r| r.timestamp).unwrap_or(Instant::now());
            let duration = last_ts.duration_since(first).as_secs_f64();
            if history.len() > 1 { duration / (history.len() - 1) as f64 * 1000.0 } else { 0.0 }
        } else {
            0.0
        };

        let rotation_count_last_hour = history.iter().rev()
            .take_while(|r| r.timestamp.elapsed().as_secs() < 3600)
            .count() as u64;

        HeartbeatSummary {
            total_ticks,
            last_proxy_url,
            last_proxy_geo,
            last_fingerprint_id,
            avg_rotation_interval_ms: avg_interval,
            recent_success_rate,
            rotation_count_last_hour,
        }
    }

    /// Reset the heartbeat state
    pub async fn reset(&self) {
        self.rotation_count.store(0, Ordering::Relaxed);
        self.history.write().await.clear();
        *self.current_proxy_url.write().await = String::new();
        *self.last_heartbeat.write().await = Instant::now();
    }
}

/// Flush OS DNS cache.
/// macOS: `dscacheutil -flushcache && sudo killall -HUP mDNSResponder`
/// Linux: `systemd-resolve --flush-caches` or `rndc flush`
/// Windows: `ipconfig /flushdns`
///
/// Returns true if the command was dispatched (may not have root on macOS).
async fn flush_dns_cache() -> bool {
    #[cfg(target_os = "macos")]
    {
        let r1 = tokio::process::Command::new("dscacheutil")
            .arg("-flushcache")
            .output().await;
        let r2 = tokio::process::Command::new("killall")
            .arg("-HUP")
            .arg("mDNSResponder")
            .output().await;
        r1.is_ok() && r2.is_ok()
    }
    #[cfg(target_os = "linux")]
    {
        let r1 = tokio::process::Command::new("resolvectl")
            .arg("flush-caches")
            .output().await;
        r1.is_ok()
    }
    #[cfg(target_os = "windows")]
    {
        let r = tokio::process::Command::new("ipconfig")
            .arg("/flushdns")
            .output().await;
        r.is_ok()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        false
    }
}

impl ProxyPool {
    /// Return all nodes with their current data (cloned).
    pub async fn all_nodes(&self) -> Vec<ProxyNode> {
        self.nodes.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_shield_stealth_net::proxy_pool::ProxyPool;

    #[tokio::test]
    async fn test_heartbeat_empty_pool() {
        let pool = Arc::new(ProxyPool::new());
        let fm = FingerprintManager::new();
        let engine = ProxyHeartbeatEngine::new(pool, fm, 30);

        let record = engine.tick().await;
        assert!(!record.success);
        assert_eq!(record.proxy_url, "");
        assert_eq!(record.tick, 1);
        assert!(record.dns_flushed);
    }

    #[tokio::test]
    async fn test_heartbeat_with_nodes() {
        let pool = Arc::new(ProxyPool::new());
        pool.add("socks5://1.2.3.4:1080", "test_a").await;
        pool.add("socks5://5.6.7.8:1080", "test_b").await;

        // Set latency so they're selectable
        {
            let mut nodes = pool.nodes.write().await;
            for n in nodes.iter_mut() {
                n.latency_ms = Some(100.0);
                n.last_success = Some(Instant::now());
            }
        }

        let fm = FingerprintManager::new();
        let engine = ProxyHeartbeatEngine::new(pool, fm, 30);

        let record = engine.tick().await;
        assert!(record.success);
        assert!(!record.proxy_url.is_empty());
        assert_eq!(record.tick, 1);
    }

    #[tokio::test]
    async fn test_heartbeat_summary() {
        let pool = Arc::new(ProxyPool::new());
        pool.add("socks5://1.2.3.4:1080", "test").await;
        {
            let mut nodes = pool.nodes.write().await;
            for n in nodes.iter_mut() {
                n.latency_ms = Some(50.0);
                n.last_success = Some(Instant::now());
            }
        }

        let fm = FingerprintManager::new();
        let engine = ProxyHeartbeatEngine::new(pool, fm, 30);

        let _r1 = engine.tick().await;
        let _r2 = engine.tick().await;

        let summary = engine.summary().await;
        assert_eq!(summary.total_ticks, 2);
        assert!(summary.recent_success_rate > 0.0);
    }

    #[tokio::test]
    async fn test_heartbeat_twice_rotates() {
        let pool = Arc::new(ProxyPool::new());
        pool.add("socks5://a:1080", "a").await;
        pool.add("socks5://b:1080", "b").await;
        {
            let mut nodes = pool.nodes.write().await;
            for n in nodes.iter_mut() {
                n.latency_ms = Some(50.0);
                n.last_success = Some(Instant::now());
            }
        }

        let fm = FingerprintManager::new();
        let engine = ProxyHeartbeatEngine::new(pool, fm, 30);

        let r1 = engine.tick().await;
        let r2 = engine.tick().await;
        assert!(r1.success && r2.success);

        // Fingerprints should differ after rotation
        assert_ne!(r1.fingerprint_id, r2.fingerprint_id);
    }

    #[test]
    fn test_flush_dns_cache_runs() {
        // Just validate the function can be called without panic
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(flush_dns_cache());
        // On macOS it should succeed
        #[cfg(target_os = "macos")]
        assert!(result);
    }
}
