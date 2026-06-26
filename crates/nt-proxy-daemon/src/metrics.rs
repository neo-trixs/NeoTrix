use std::collections::HashMap;
use std::net::IpAddr;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::guard::PerClientGuard;
use crate::pool::json_escape;

const MAX_PER_CLIENT: u32 = 16;
const RATE_PRUNE_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Default, Clone)]
pub(crate) struct NodeMetrics {
    pub(crate) total_conns: u64,
    pub(crate) failed_conns: u64,
    pub(crate) avg_latency_ms: f64,
    pub(crate) sample_count: u64,
}

impl NodeMetrics {
    pub(crate) fn record(&mut self, ok: bool, latency_ms: f64) {
        self.total_conns += 1;
        if !ok {
            self.failed_conns += 1;
        }
        self.avg_latency_ms = (self.avg_latency_ms * self.sample_count as f64 + latency_ms)
            / (self.sample_count + 1) as f64;
        self.sample_count += 1;
    }
    pub(crate) fn success_rate(&self) -> f64 {
        if self.total_conns == 0 {
            return 1.0;
        }
        1.0 - self.failed_conns as f64 / self.total_conns as f64
    }
}

pub(crate) struct ClientRateLimiter {
    entries: Mutex<HashMap<IpAddr, u32>>,
    last_prune: Mutex<Instant>,
    max_per_client: u32,
}

impl ClientRateLimiter {
    pub(crate) fn new(max: u32) -> Self {
        ClientRateLimiter {
            entries: Mutex::new(HashMap::new()),
            last_prune: Mutex::new(Instant::now()),
            max_per_client: max,
        }
    }

    pub(crate) fn try_register(self: &Arc<Self>, addr: IpAddr) -> Result<PerClientGuard, ()> {
        self.maybe_prune();
        let mut e = self.entries.lock().unwrap_or_else(|p| p.into_inner());
        let count = e.entry(addr).or_insert(0);
        if *count >= self.max_per_client {
            return Err(());
        }
        *count += 1;
        Ok(PerClientGuard {
            limiter: Arc::clone(self),
            addr,
        })
    }

    pub(crate) fn release(&self, addr: &IpAddr) {
        if let Ok(mut e) = self.entries.lock() {
            if let Some(count) = e.get_mut(addr) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    e.remove(addr);
                }
            }
        }
    }

    pub(crate) fn active_count(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }

    fn maybe_prune(&self) {
        if let Ok(mut last) = self.last_prune.lock() {
            if last.elapsed() < RATE_PRUNE_INTERVAL {
                return;
            }
            *last = Instant::now();
        }
        if let Ok(mut e) = self.entries.lock() {
            e.retain(|_, count| *count > 0);
        }
    }
}

pub(crate) fn rate_limiter() -> &'static Arc<ClientRateLimiter> {
    static LIMITER: std::sync::OnceLock<Arc<ClientRateLimiter>> = std::sync::OnceLock::new();
    LIMITER.get_or_init(|| Arc::new(ClientRateLimiter::new(MAX_PER_CLIENT)))
}

pub(crate) struct AccessLogEntry<'a> {
    pub(crate) method: String,
    pub(crate) target: String,
    pub(crate) upstream_idx: isize,
    pub(crate) success: bool,
    pub(crate) latency_ms: f64,
    pub(crate) circuit_state: &'a str,
}

pub(crate) fn log_access(entry: &AccessLogEntry) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let status = if entry.success { "ok" } else { "fail" };
    log::error!(
        "{{\"ts\":{:.3},\"method\":\"{}\",\"target\":\"{}\",\"upstream\":{},\"status\":\"{}\",\"latency_ms\":{:.0},\"circuit\":\"{}\"}}",
        ts,
        json_escape(&entry.method),
        json_escape(&entry.target),
        entry.upstream_idx,
        status,
        entry.latency_ms,
        entry.circuit_state,
    );
}
