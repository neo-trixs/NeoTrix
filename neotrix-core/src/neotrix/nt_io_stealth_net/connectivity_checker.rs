use crate::core::nt_core_network::VsaDnsCache;
use crate::core::ShutdownSignal;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::proxy_control::DaemonMode;
use super::proxy_pool::global_pool;
use super::proxy_sourcing::ProxySourcing;

const CHECK_INTERVAL_SECS: u64 = 30;
const DIRECT_TIMEOUT_SECS: u64 = 5;
const PROXY_THRESHOLD: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinkStatus {
    Healthy,
    Degraded,
    Dead,
}

#[derive(Debug, Clone)]
pub struct ConnectivitySnapshot {
    pub direct_reachable: bool,
    pub direct_latency_ms: Option<f64>,
    pub proxy_total_count: usize,
    pub proxy_healthy_count: usize,
    pub proxy_avg_latency_ms: Option<f64>,
    pub active_mode: DaemonMode,
    pub last_checked: Instant,
    pub consecutive_proxy_failures: u32,
}

impl ConnectivitySnapshot {
    pub fn direct_status(&self) -> LinkStatus {
        if self.direct_reachable {
            LinkStatus::Healthy
        } else {
            LinkStatus::Dead
        }
    }

    pub fn proxy_status(&self) -> LinkStatus {
        if self.proxy_healthy_count >= PROXY_THRESHOLD {
            LinkStatus::Healthy
        } else if self.proxy_healthy_count > 0 {
            LinkStatus::Degraded
        } else {
            LinkStatus::Dead
        }
    }
}

pub struct ConnectivityChecker {
    snapshot: RwLock<ConnectivitySnapshot>,
    mode: Arc<RwLock<DaemonMode>>,
    sourcing: ProxySourcing,
    auto_mode: bool,
}

impl ConnectivityChecker {
    pub fn new(mode: Arc<RwLock<DaemonMode>>) -> Self {
        Self {
            snapshot: RwLock::new(ConnectivitySnapshot {
                direct_reachable: true,
                direct_latency_ms: None,
                proxy_total_count: 0,
                proxy_healthy_count: 0,
                proxy_avg_latency_ms: None,
                active_mode: DaemonMode::Off,
                last_checked: Instant::now(),
                consecutive_proxy_failures: 0,
            }),
            mode,
            sourcing: ProxySourcing::new(),
            auto_mode: true,
        }
    }

    pub async fn snapshot(&self) -> ConnectivitySnapshot {
        self.snapshot.read().await.clone()
    }

    pub async fn tick(&self) {
        let (direct_ok, direct_latency) = self.probe_direct().await;
        let (proxy_count, proxy_healthy, _proxy_latency) = self.probe_proxy_pool().await;

        if proxy_healthy < PROXY_THRESHOLD {
            let mut s = self.snapshot.write().await;
            s.consecutive_proxy_failures += 1;
        } else {
            let mut s = self.snapshot.write().await;
            s.consecutive_proxy_failures = 0;
        };

        let best_mode = Self::select_best_mode(proxy_healthy);

        let mut s = self.snapshot.write().await;
        s.direct_reachable = direct_ok;
        s.direct_latency_ms = direct_latency;
        s.proxy_total_count = proxy_count;
        s.proxy_healthy_count = proxy_healthy;
        s.last_checked = Instant::now();

        if self.auto_mode {
            *self.mode.write().await = best_mode;
            s.active_mode = best_mode;
        } else {
            s.active_mode = *self.mode.read().await;
        }
    }

    pub async fn start_background(self: Arc<Self>, shutdown: ShutdownSignal) {
        loop {
            self.tick().await;
            if shutdown.is_shutdown() {
                log::info!("[connectivity] background loop shutting down");
                break;
            }
            let need_refill = {
                let s = self.snapshot.read().await;
                s.proxy_healthy_count < PROXY_THRESHOLD && s.direct_reachable
            };
            if need_refill {
                let pool = global_pool();
                let free_sources: Vec<&super::proxy_sourcing::ProxySourceDef> =
                    super::proxy_sourcing::ALL_FREE_SOURCES.iter().collect();
                let results = self.sourcing.fetch_all_sources(&free_sources).await;
                let mut added = 0usize;
                for (_, proxies) in &results {
                    for p in proxies {
                        pool.add(&p.to_proxy_url(), "sourced").await;
                        added += 1;
                    }
                }
                if added > 0 {
                    pool.health_check().await;
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(CHECK_INTERVAL_SECS)) => {}
                _ = shutdown.wait_shutdown() => {
                    log::info!("[connectivity] background loop shutting down");
                    break;
                }
            }
        }
    }

    async fn probe_direct(&self) -> (bool, Option<f64>) {
        self.probe_direct_cached(None).await
    }

    async fn probe_direct_cached(
        &self,
        mut dns_cache: Option<&mut VsaDnsCache>,
    ) -> (bool, Option<f64>) {
        let targets = [("1.1.1.1", 80u16), ("8.8.8.8", 80), ("example.com", 80)];
        for (host, port) in &targets {
            let start = Instant::now();
            let addr = if let Some(cache) = dns_cache.as_deref_mut() {
                if host.parse::<std::net::IpAddr>().is_err() {
                    if let Some(ip) = cache.resolve(
                        host,
                        crate::core::nt_core_network::dns_cache::AddressFamily::V4,
                    ) {
                        format!("{}:{}", ip, port)
                    } else {
                        format!("{}:{}", host, port)
                    }
                } else {
                    format!("{}:{}", host, port)
                }
            } else {
                format!("{}:{}", host, port)
            };
            let ok = tokio::time::timeout(
                Duration::from_secs(DIRECT_TIMEOUT_SECS),
                tokio::net::TcpStream::connect(&addr),
            )
            .await
            .is_ok_and(|r| r.is_ok());
            if ok {
                return (true, Some(start.elapsed().as_secs_f64() * 1000.0));
            }
        }
        (false, None)
    }

    async fn probe_proxy_pool(&self) -> (usize, usize, Option<f64>) {
        let pool = global_pool();
        let total = pool.total_count().await;
        pool.health_check().await;
        let healthy = pool.available_count().await;
        (total, healthy, None)
    }

    fn select_best_mode(proxy_healthy: usize) -> DaemonMode {
        if proxy_healthy >= PROXY_THRESHOLD {
            DaemonMode::Stealth
        } else if proxy_healthy > 0 {
            DaemonMode::Geo
        } else {
            DaemonMode::Off
        }
    }
}
