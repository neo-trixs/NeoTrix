use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::dns::dns_cache;
use crate::obfuscation::socks5_greeting_standard;
use crate::pool::ProxyPool;
use crate::socks5::parse_socks5_url;

const CHECK_BASE_INTERVAL: Duration = Duration::from_secs(15);
const CHECK_MAX_INTERVAL: Duration = Duration::from_secs(60);
const CHECK_FAST_INTERVAL: Duration = Duration::from_secs(5);
const CHECK_PARALLELISM: usize = 4;
const PRUNE_INTERVAL_CYCLES: u32 = 20;

pub(crate) struct HealthChecker;

impl HealthChecker {
    pub(crate) fn spawn(pool: Arc<ProxyPool>, shutdown: Arc<AtomicBool>) {
        let _ = thread::Builder::new().stack_size(512 * 1024).spawn(move || {
            let mut interval = CHECK_BASE_INTERVAL;
            let mut stable_cycles = 0u32;
            let mut prune_counter = 0u32;
            let mut prev_healthy_count = 0usize;

            loop {
                if shutdown.load(Ordering::Relaxed) {
                    log::error!("[checker] shutdown signal received, exiting");
                    break;
                }
                let cycle_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let cycle_start = Instant::now();
                    let n = pool.len();
                    let n_parallel = CHECK_PARALLELISM.min(n);

                    let mut healthy_count = 0u32;
                    let mut unhealthy_count = 0u32;

                    let mut processed = 0usize;
                    while processed < n {
                        let batch_start = processed;
                        let batch_end = (processed + n_parallel).min(n);

                        let mut handles = Vec::with_capacity(batch_end - batch_start);
                        for idx in batch_start..batch_end {
                            let pool_ref = Arc::clone(&pool);
                            handles.push(thread::Builder::new().stack_size(128 * 1024).spawn(move || {
                                let (name, url, _region, score, ping_ms) = match pool_ref.get_info(idx) {
                                    Some(info) => info,
                                    None => return (idx, false, 0.0),
                                };
                                let probe_start = Instant::now();
                                let ok = probe_socks5(&url);
                                let lat = probe_start.elapsed().as_secs_f64() * 1000.0;
                                let old = pool_ref.set_health(idx, ok, lat);
                                if old != ok {
                                    let state = if ok { "up" } else { "down" };
                                    log::error!("[checker] {} score={} ping={}ms → {} (changed, {lat:.0}ms)", name, score, ping_ms, state);
                                }
                                (idx, ok, lat)
                            }));
                        }

                        for handle in handles.into_iter().flatten() {
                            if let Ok(result) = handle.join() {
                                if result.1 { healthy_count += 1; } else { unhealthy_count += 1; }
                            }
                        }
                        processed = batch_end;
                    }

                    let n_total = (healthy_count + unhealthy_count) as usize;
                    let healthy_now = healthy_count as usize;

                    if healthy_now == n_total {
                        stable_cycles += 1;
                        if stable_cycles >= 3 {
                            interval = CHECK_MAX_INTERVAL.min(interval.mul_f64(1.5));
                        } else {
                            interval = CHECK_BASE_INTERVAL;
                        }
                    } else if healthy_now < n_total / 2 {
                        interval = CHECK_FAST_INTERVAL;
                        stable_cycles = 0;
                    } else {
                        interval = CHECK_BASE_INTERVAL;
                        stable_cycles = 0;
                    }

                    if healthy_now != prev_healthy_count {
                        log::error!("[checker] done: {healthy_count} healthy / {unhealthy_count} unhealthy / {n_total} total (interval={:.0}s)", interval.as_secs_f64());
                        prev_healthy_count = healthy_now;
                    }

                    prune_counter += 1;
                    if prune_counter % PRUNE_INTERVAL_CYCLES == 0 {
                        pool.prune_dead_nodes();
                    }

                    let elapsed = cycle_start.elapsed();
                    if elapsed < interval {
                        thread::sleep(interval - elapsed);
                    }
                }));

                if cycle_result.is_err() {
                    log::error!("[checker] CRASH recovered, restarting cycle");
                    interval = CHECK_BASE_INTERVAL;
                }
            }
        });
    }
}

pub(crate) fn probe_socks5(upstream_url: &str) -> bool {
    let (host, port, _is_local_dns, _scheme) = match parse_socks5_url(upstream_url) {
        Some(p) => p,
        None => return false,
    };
    let socket_addr = match dns_cache().resolve(&host, port) {
        Ok(a) => a,
        Err(_) => return false,
    };
    let is_local = host.starts_with("127.") || host == "localhost";
    // Localhost bridges (encrypted node → kernel): TCP ping only (kernel CONNECT is 5-15s)
    // Remote upstreams: full SOCKS5 greeting probe
    let timeout = if is_local { 1 } else { 3 };
    let mut sock = match std::net::TcpStream::connect_timeout(&socket_addr, Duration::from_secs(timeout)) {
        Ok(s) => s,
        Err(_) => return false,
    };
    if is_local {
        drop(sock);
        return true;
    }
    sock.set_read_timeout(Some(Duration::from_secs(5))).ok();
    sock.set_write_timeout(Some(Duration::from_secs(3))).ok();
    if sock.write_all(&socks5_greeting_standard()).is_err() {
        return false;
    }
    let mut buf = [0u8; 2];
    if sock.read_exact(&mut buf).is_err() {
        return false;
    }
    buf[0] == 5
}
