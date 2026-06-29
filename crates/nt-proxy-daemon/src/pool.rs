use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::dns::dns_cache;
use crate::metrics::{rate_limiter, NodeMetrics};

const CONSECUTIVE_FAIL_LIMIT: u32 = 3;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub(crate) struct PoolEntry {
    pub(crate) url: String,
    pub(crate) healthy: bool,
    pub(crate) name: String,
    pub(crate) region: String,
    pub(crate) score: f64,
    pub(crate) ping_ms: u32,
    pub(crate) consecutive_failures: u32,
    pub(crate) routing_failures: u32,
    pub(crate) circuit_state: CircuitState,
    pub(crate) circuit_until: Option<Instant>,
    pub(crate) last_checked: Option<Instant>,
    pub(crate) metrics: NodeMetrics,
    pub(crate) conn_count: AtomicU64,
}

pub(crate) struct ProxyPool {
    pub(crate) entries: Mutex<Vec<PoolEntry>>,
    conf_path: PathBuf,
    last_mtime: Mutex<Option<std::time::SystemTime>>,
    generation: AtomicU64,
}

pub(crate) fn extract_region(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.len() >= 2 {
        trimmed[..2].to_uppercase()
    } else {
        "XX".to_string()
    }
}

pub(crate) fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub(crate) fn parse_upstreams(content: &str) -> Vec<PoolEntry> {
    let lines: Vec<&str> = content.lines().collect();
    let mut entries = Vec::new();
    let mut pending_meta: Option<(String, f64, u32)> = None;

    for line in &lines {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        if let Some(meta) = l.strip_prefix('#') {
            let meta = meta.trim();
            let name = meta
                .split("score=")
                .next()
                .unwrap_or("unknown")
                .trim()
                .to_string();
            let score = meta
                .split("score=")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.5);
            let ping = meta
                .split("ping=")
                .nth(1)
                .and_then(|s| s.split("ms").next())
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(999);
            pending_meta = Some((name, score, ping));
            continue;
        }
        if l.starts_with("socks5://") || l.starts_with("socks5h://") {
            let is_local = l.contains("127.0.0.1") || l.contains("localhost");
            let (name, score, ping) = if is_local {
                pending_meta.take();
                ("local-proxy".to_string(), 1.0, 0)
            } else {
                pending_meta
                    .take()
                    .unwrap_or_else(|| ("unknown".into(), 0.5, 999))
            };
            let region = extract_region(&name);
            entries.push(PoolEntry {
                url: l.to_string(),
                healthy: score >= 0.5,
                name,
                region,
                score,
                ping_ms: ping,
                consecutive_failures: 0,
                routing_failures: 0,
                circuit_state: CircuitState::Closed,
                circuit_until: None,
                last_checked: None,
                metrics: NodeMetrics::default(),
                conn_count: AtomicU64::new(0),
            });
        }
    }
    entries
}

impl ProxyPool {
    pub(crate) fn from_file(path: &str) -> Self {
        let conf_path = PathBuf::from(path);
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let entries = parse_upstreams(&content);
        let mtime = std::fs::metadata(path).ok().and_then(|m| m.modified().ok());
        log::error!("[pool] loaded {} nodes from {}", entries.len(), path);
        ProxyPool {
            entries: Mutex::new(entries),
            conf_path,
            last_mtime: Mutex::new(mtime),
            generation: AtomicU64::new(0),
        }
    }

    pub(crate) fn reload_if_changed(&self) -> bool {
        let new_mtime = match std::fs::metadata(&self.conf_path) {
            Ok(m) => match m.modified() {
                Ok(t) => t,
                Err(_) => return false,
            },
            Err(_) => return false,
        };
        let changed = {
            let last = self.last_mtime.lock().unwrap_or_else(|p| p.into_inner());
            last.map(|t| new_mtime != t).unwrap_or(true)
        };
        if !changed {
            return false;
        }
        let content = match std::fs::read_to_string(&self.conf_path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let new_entries = parse_upstreams(&content);
        let entry_count = new_entries.len();
        {
            let mut entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
            *entries = new_entries;
        }
        {
            let mut mtime = self.last_mtime.lock().unwrap_or_else(|p| p.into_inner());
            *mtime = Some(new_mtime);
        }
        self.generation.fetch_add(1, Ordering::Relaxed);
        log::error!(
            "[pool] reloaded {} nodes from {}",
            entry_count,
            self.conf_path.display()
        );
        true
    }

    pub(crate) fn prune_dead_nodes(&self) -> usize {
        if let Ok(mut e) = self.entries.lock() {
            let before = e.len();
            e.retain(|entry| entry.consecutive_failures < CONSECUTIVE_FAIL_LIMIT * 10);
            let pruned = before - e.len();
            if pruned > 0 {
                log::error!(
                    "[pool] pruned {pruned} dead nodes (consecutive_failures >= {})",
                    CONSECUTIVE_FAIL_LIMIT * 10
                );
            }
            pruned
        } else {
            0
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }

    pub(crate) fn healthy_count(&self) -> usize {
        self.entries
            .lock()
            .map(|e| e.iter().filter(|e| e.healthy).count())
            .unwrap_or(0)
    }

    pub(crate) fn set_health(&self, idx: usize, healthy: bool, latency_ms: f64) -> bool {
        if let Ok(mut e) = self.entries.lock() {
            if idx < e.len() {
                let entry = &mut e[idx];
                let old = entry.healthy;
                entry.metrics.record(healthy, latency_ms);
                if healthy {
                    entry.consecutive_failures = 0;
                    entry.healthy = true;
                    entry.routing_failures = 0;
                    if entry.circuit_state != CircuitState::Closed {
                        entry.circuit_state = CircuitState::Closed;
                        entry.circuit_until = None;
                    }
                } else {
                    entry.consecutive_failures += 1;
                    entry.healthy = entry.consecutive_failures < CONSECUTIVE_FAIL_LIMIT;
                    if entry.circuit_state == CircuitState::HalfOpen {
                        entry.circuit_state = CircuitState::Open;
                        entry.circuit_until = Some(Instant::now() + Duration::from_secs(30));
                    }
                }
                entry.last_checked = Some(Instant::now());
                return old;
            }
        }
        false
    }

    pub(crate) fn bootstrap_health(&self, idx: usize, healthy: bool) {
        if let Ok(mut e) = self.entries.lock() {
            if idx < e.len() {
                let entry = &mut e[idx];
                entry.healthy = healthy;
            }
        }
    }

    pub(crate) fn get_info(&self, idx: usize) -> Option<(String, String, String, f64, u32)> {
        self.entries.lock().ok().and_then(|e| {
            e.get(idx).map(|entry| {
                (
                    entry.name.clone(),
                    entry.url.clone(),
                    entry.region.clone(),
                    entry.score,
                    entry.ping_ms,
                )
            })
        })
    }

    pub(crate) fn record_result(&self, url: &str, ok: bool, latency_ms: f64) {
        if let Ok(mut e) = self.entries.lock() {
            if let Some(entry) = e.iter_mut().find(|e| e.url == url) {
                entry.metrics.record(ok, latency_ms);
                if ok {
                    entry.routing_failures = 0;
                    if entry.circuit_state == CircuitState::HalfOpen {
                        entry.circuit_state = CircuitState::Closed;
                        entry.circuit_until = None;
                    }
                } else {
                    entry.routing_failures = entry.routing_failures.saturating_add(1);
                    if entry.routing_failures >= 2 && entry.circuit_state == CircuitState::Closed {
                        entry.circuit_state = CircuitState::Open;
                        entry.circuit_until = Some(Instant::now() + Duration::from_secs(30));
                    }
                    if entry.circuit_state == CircuitState::HalfOpen {
                        entry.circuit_state = CircuitState::Open;
                        entry.circuit_until = Some(Instant::now() + Duration::from_secs(30));
                    }
                }
            }
        }
    }

    pub(crate) fn status_json(&self) -> String {
        let e = match self.entries.lock() {
            Ok(g) => g,
            Err(_) => return "{}".into(),
        };
        let parts: Vec<String> = e.iter().map(|entry| {
            let cb_state = match entry.circuit_state {
                CircuitState::Closed => "closed",
                CircuitState::Open => "open",
                CircuitState::HalfOpen => "half_open",
            };
            format!("\"{}\":{{\"healthy\":{},\"region\":\"{}\",\"score\":{},\"ping_ms\":{},\"failures\":{},\"conns\":{},\"failed_conns\":{},\"conn_count\":{},\"success_rate\":{:.3},\"circuit\":\"{}\"}}",
                json_escape(&entry.name),
                entry.healthy,
                entry.region,
                entry.score,
                entry.ping_ms,
                entry.consecutive_failures,
                entry.metrics.total_conns,
                entry.metrics.failed_conns,
                entry.conn_count.load(Ordering::Relaxed),
                entry.metrics.success_rate(),
                cb_state,
            )
        }).collect();
        let (dns_hits, dns_misses) = dns_cache().stats();
        let dns_size = dns_cache().len();
        let rate_active = rate_limiter().active_count();
        format!("{{\"total\":{},\"healthy\":{},\"dns_hits\":{},\"dns_misses\":{},\"dns_cache_size\":{},\"rate_limit_active\":{},\"nodes\":{{{}}}}}",
            e.len(), e.iter().filter(|e| e.healthy).count(),
            dns_hits, dns_misses, dns_size, rate_active,
            parts.join(","))
    }
}
