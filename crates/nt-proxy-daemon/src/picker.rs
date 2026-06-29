use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::obfuscation::rand_u64_splitmix64;
use crate::pool::{CircuitState, PoolEntry, ProxyPool};

const FORCE_ROTATE_AFTER: u64 = 5;
const EPSILON: f64 = 0.15;

pub(crate) struct FastPicker {
    counter: AtomicU64,
    last_idx: Mutex<usize>,
    same_count: Mutex<u64>,
}

impl FastPicker {
    pub(crate) fn new() -> Self {
        FastPicker {
            counter: AtomicU64::new(0),
            last_idx: Mutex::new(usize::MAX),
            same_count: Mutex::new(0),
        }
    }

    fn node_weight(entry: &PoolEntry) -> f64 {
        let success = entry.metrics.success_rate();
        let latency_ms = if entry.metrics.avg_latency_ms > 0.0 {
            entry.metrics.avg_latency_ms
        } else {
            entry.ping_ms as f64
        };
        let latency_factor = 1.0 + latency_ms / 10.0;
        entry.score * success / latency_factor
    }

    pub(crate) fn pick(&self, pool: &ProxyPool) -> Option<(usize, String, String)> {
        let entries = match pool.entries.lock() {
            Ok(e) => e,
            Err(_) => return None,
        };
        let candidates: Vec<(f64, usize)> = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                if entry.circuit_state == CircuitState::Open {
                    return false;
                }
                entry.healthy && entry.routing_failures < 2
            })
            .map(|(i, entry)| (Self::node_weight(entry), i))
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let len = candidates.len();

        // Epsilon-greedy: 15% of the time pick a uniformly random node for exploration
        let explore = len > 1 && (rand_u64_splitmix64() as f64 / u64::MAX as f64) < EPSILON;
        let (_, entry_idx) = if explore {
            let idx = (rand_u64_splitmix64() as usize) % len;
            candidates[idx]
        } else {
            // window-of-3 max-weight from offset
            let offset = self.counter.fetch_add(1, Ordering::Relaxed) as usize % len;
            let window = len.min(3);
            let best = (0..window)
                .map(|w| (offset + w) % len)
                .max_by(|&a, &b| {
                    candidates[a]
                        .0
                        .partial_cmp(&candidates[b].0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(offset);
            candidates[best]
        };

        // Force rotate if same node picked too many consecutive times
        {
            let mut last = self.last_idx.lock().unwrap_or_else(|p| p.into_inner());
            let mut count = self.same_count.lock().unwrap_or_else(|p| p.into_inner());
            if entry_idx == *last {
                *count += 1;
                if *count >= FORCE_ROTATE_AFTER && len > 1 {
                    *count = 0;
                    let forced = (0..len)
                        .filter(|&i| candidates[i].1 != entry_idx)
                        .max_by(|&a, &b| {
                            candidates[a]
                                .0
                                .partial_cmp(&candidates[b].0)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .unwrap_or(0);
                    let (_, forced_idx) = candidates[forced];
                    *last = forced_idx;
                    let entry = &entries[forced_idx];
                    entry.conn_count.fetch_add(1, Ordering::Relaxed);
                    let url = entry.url.clone();
                    let region = entry.region.clone();
                    drop(entries);
                    return Some((forced_idx, url, region));
                }
            } else {
                *last = entry_idx;
                *count = 0;
            }
        }

        let entry = &entries[entry_idx];
        entry.conn_count.fetch_add(1, Ordering::Relaxed);
        let url = entry.url.clone();
        let region = entry.region.clone();
        drop(entries);
        Some((entry_idx, url, region))
    }
}
