#![forbid(unsafe_code)]
#![allow(dead_code)]
// dead_code allowed: architecture components waiting for integration

use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod circuit;
mod dns;
mod guard;
mod health;
mod http;
mod metrics;
mod obfuscation;
pub mod obfuscation_test;
mod picker;
mod traffic_shape;
mod pool;
mod relay;
mod socket;
mod socks5;
mod ssrf;
mod timing_obfuscation;
mod tls;

#[cfg(feature = "mux")]
mod mux;

use crate::guard::ConnectionGuard;
use crate::health::{probe_socks5, HealthChecker};
use crate::http::{parse_http_request, HttpError, send_error, send_status};
use crate::metrics::{log_access, rate_limiter, AccessLogEntry};
use crate::picker::FastPicker;
use crate::pool::{CircuitState, ProxyPool};
use crate::relay::{relay_upstream, try_upstream};
use crate::socket::{create_listener, try_direct, tune_socket};

const UPSTREAM_FILE: &str = ".neotrix/proxy-upstreams.conf";
const MAX_CONNECTIONS: u32 = 128;

#[cfg(feature = "mux")]
static MUX_MANAGER: std::sync::OnceLock<mux::MuxManager> = std::sync::OnceLock::new();
const FALLBACK_TIMEOUT: Duration = Duration::from_secs(5);
const CONF_POLL_INTERVAL: Duration = Duration::from_secs(10);
const MAX_FALLBACK_RETRIES: u32 = 2;
const CHECK_PARALLELISM: usize = 4;

struct StderrLogger;
static LOGGER: StderrLogger = StderrLogger;
impl log::Log for StderrLogger {
    fn enabled(&self, _: &log::Metadata) -> bool { true }
    fn log(&self, record: &log::Record) {
        std::eprintln!("[nt-proxy] {} {}", record.level(), record.args());
    }
    fn flush(&self) {}
}

fn main() {
    log::set_logger(&LOGGER).ok();
    log::set_max_level(log::LevelFilter::Info);
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let path = format!("{home}/{UPSTREAM_FILE}");
    let pool = Arc::new(ProxyPool::from_file(&path));
    let total = pool.len();
    if total == 0 {
        log::error!("FATAL: no upstream proxies found in {path}");
        std::process::exit(1);
    }
    log::error!("nt-proxy-daemon ready: {total} upstreams");

    let n = pool.len();
    let batch_size = CHECK_PARALLELISM.min(n);
    let mut processed = 0usize;
    while processed < n {
        let batch_end = (processed + batch_size).min(n);
        let mut handles = Vec::with_capacity(batch_end - processed);
        for idx in processed..batch_end {
            let pool_ref = Arc::clone(&pool);
            handles.push(
                thread::Builder::new()
                    .stack_size(128 * 1024)
                    .spawn(move || {
                        let (name, url, _region, score, ping_ms) = match pool_ref.get_info(idx) {
                            Some(info) => info,
                            None => return None,
                        };
                        let ok = probe_socks5(&url);
                        pool_ref.bootstrap_health(idx, ok);
                        Some((idx, name, score, ping_ms, ok))
                    }),
            );
        }
        for handle in handles.into_iter().flatten() {
            if let Some((_idx, name, score, ping_ms, ok)) = handle.join().unwrap_or(None) {
                let state = if ok { "up" } else { "down" };
                log::error!(
                    "[init] {} score={} ping={}ms → {}",
                    name, score, ping_ms, state
                );
            }
        }
        processed = batch_end;
    }
    log::error!(
        "[init] done: {} healthy / {} total",
        pool.healthy_count(),
        pool.len()
    );

    let shutdown = Arc::new(AtomicBool::new(false));
    HealthChecker::spawn(Arc::clone(&pool), Arc::clone(&shutdown));
    let picker = Arc::new(FastPicker::new());
    let conn_count = Arc::new(AtomicU32::new(0));
    let rate_limiter = Arc::clone(rate_limiter());

    #[cfg(feature = "mux")]
    {
        let _ = MUX_MANAGER.set(mux::MuxManager::new(None, None));
        log::error!("[mux] HTTP/2 MUX enabled (defeats encapsulated TLS handshake detection)");
    }

    {
        let pool = Arc::clone(&pool);
        let sd = Arc::clone(&shutdown);
        let _ = thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || loop {
                if sd.load(Ordering::Relaxed) {
                    log::error!("[watcher] shutdown signal received, exiting");
                    break;
                }
                thread::sleep(CONF_POLL_INTERVAL);
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if pool.reload_if_changed() {
                        log::error!("[watcher] config changed, hot-reloaded upstreams");
                    }
                }));
            });
    }

    let listener = create_listener().expect("bind 127.0.0.1:11080");
    log::error!("listening on 127.0.0.1:11080");

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                log::error!("accept error: {e}");
                continue;
            }
        };
        let count = conn_count.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        if count >= MAX_CONNECTIONS {
            conn_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
            log::error!("connection limit reached ({MAX_CONNECTIONS}), dropping");
            drop(stream);
            continue;
        }
        tune_socket(&stream);
        let client_addr = match stream.peer_addr() {
            Ok(addr) => addr.ip(),
            Err(_) => continue,
        };
        let Ok(rate_guard) = rate_limiter.try_register(client_addr) else {
            conn_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
            log::error!("rate limit: {client_addr} exceeded 16 connections");
            drop(stream);
            continue;
        };
        let guard = ConnectionGuard(Arc::clone(&conn_count));
        let pool = Arc::clone(&pool);
        let picker = Arc::clone(&picker);
        let _ = thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || {
                let (_guard, _rate_guard) = (guard, rate_guard);
                handle_client(stream, &pool, &picker);
            });
    }

    let remaining = conn_count.load(std::sync::atomic::Ordering::Acquire);
    if remaining > 0 {
        log::error!("[shutdown] draining {remaining} active connections...");
        let drain_deadline = Instant::now() + Duration::from_secs(30);
        while conn_count.load(std::sync::atomic::Ordering::Acquire) > 0
            && Instant::now() < drain_deadline
        {
            thread::sleep(Duration::from_millis(100));
        }
        let still_active = conn_count.load(std::sync::atomic::Ordering::Acquire);
        if still_active > 0 {
            log::error!(
                "[shutdown] {still_active} connections still active after 30s, exiting anyway"
            );
        } else {
            log::error!("[shutdown] all connections drained");
        }
    }
    log::error!("[shutdown] goodbye");
}

fn handle_client(mut client: std::net::TcpStream, pool: &ProxyPool, picker: &FastPicker) {
    let request = match parse_http_request(&mut client) {
        Ok(r) => r,
        Err(HttpError::NoRequest) => return,
        Err(HttpError::TooLong) => {
            send_error(&mut client, 431, "Request Header Fields Too Large");
            return;
        }
        Err(HttpError::InvalidMethod) => {
            send_error(&mut client, 405, "Method Not Allowed (use CONNECT)");
            return;
        }
        Err(HttpError::InvalidTarget) => {
            send_error(&mut client, 400, "Bad Request (need host:port)");
            return;
        }
        Err(HttpError::IoError) => return,
    };

    if request.method == "GET" {
        send_status(&mut client, pool);
        return;
    }

    let conn_start = Instant::now();
    log::error!("CONNECT {}", request.target);

    let (pick_idx, pick_url, _region) = match picker.pick(pool) {
        Some(p) => p,
        None => {
            log::error!("  no healthy upstreams");
            send_error(
                &mut client,
                503,
                "Service Unavailable (no healthy upstreams)",
            );
            log_access(&AccessLogEntry {
                method: request.method,
                target: request.target,
                upstream_idx: -2,
                success: false,
                latency_ms: conn_start.elapsed().as_secs_f64() * 1000.0,
                circuit_state: "no_upstreams",
            });
            return;
        }
    };

    let mut last_err;

    // Try HTTP/2 MUX first (defeats encapsulated TLS handshake detection).
    #[cfg(feature = "mux")]
    {
        if let Some(mux_manager) = MUX_MANAGER.get() {
            match mux_manager.try_mux_connect(&request.target, &pick_url) {
                mux::MuxAttempt::Connected(mut stream) => {
                    log::error!("  ✓ #{pick_idx} (mux)");
                    let _ = client.write_all(
                        b"HTTP/1.1 200 Connection Established\r\nProxy-Connection: close\r\n\r\n",
                    );
                    crate::relay::relay_mux_stream(&mut client, &mut stream, conn_start);
                    let lat = conn_start.elapsed().as_secs_f64() * 1000.0;
                    log::error!("  done {} ({lat:.0}ms)", request.target);
                    pool.record_result(&pick_url, true, lat);
                    log_access(&AccessLogEntry {
                        method: request.method,
                        target: request.target,
                        upstream_idx: pick_idx as isize,
                        success: true,
                        latency_ms: lat,
                        circuit_state: "mux",
                    });
                    return;
                }
                mux::MuxAttempt::Fallback => {
                    log::error!("  mux fallback #{pick_idx}");
                }
            }
        }
    }

    let result = try_upstream(&pick_url, &request.target);
    match result {
        Ok(mut remote) => {
            log::error!("  ✓ #{pick_idx} (picked)");
            let _ = client.write_all(
                b"HTTP/1.1 200 Connection Established\r\nProxy-Connection: close\r\n\r\n",
            );
            relay_upstream(&mut client, &mut remote, conn_start);
            let lat = conn_start.elapsed().as_secs_f64() * 1000.0;
            log::error!("  done {} ({lat:.0}ms)", request.target);
            pool.record_result(&pick_url, true, lat);
            log_access(&AccessLogEntry {
                method: request.method,
                target: request.target,
                upstream_idx: pick_idx as isize,
                success: true,
                latency_ms: lat,
                circuit_state: "upstream",
            });
            return;
        }
        Err(e) => {
            last_err = format!("#{pick_idx}: {e}");
            log::error!("  ✗ #{pick_idx}: {e}");
            pool.record_result(&pick_url, false, conn_start.elapsed().as_secs_f64() * 1000.0);
        }
    }

    let mut fallback_attempts = 0u32;
    let fallback_deadline = Instant::now() + FALLBACK_TIMEOUT;
    loop {
        if fallback_attempts >= MAX_FALLBACK_RETRIES {
            last_err += &format!(" (max retries {MAX_FALLBACK_RETRIES})");
            break;
        }
        if Instant::now() > fallback_deadline {
            last_err += " (fallback timeout)";
            break;
        }
        let (fb_idx, fb_url) = match pool.entries.lock() {
            Ok(e) => {
                let healthy: Vec<usize> = e
                    .iter()
                    .enumerate()
                    .filter(|(i, entry)| {
                        entry.circuit_state != CircuitState::Open
                            && entry.healthy
                            && entry.routing_failures < 2
                            && *i != pick_idx
                    })
                    .map(|(i, _)| i)
                    .collect();
                if healthy.is_empty() {
                    break;
                }
                let fb_idx = healthy[fallback_attempts as usize % healthy.len()];
                let fb_url = e[fb_idx].url.clone();
                (fb_idx, fb_url)
            }
            Err(_) => break,
        };

        fallback_attempts += 1;
        match try_upstream(&fb_url, &request.target) {
            Ok(mut remote) => {
                log::error!("  ✓ #{fb_idx} (fallback)");
                let _ = client.write_all(
                    b"HTTP/1.1 200 Connection Established\r\nProxy-Connection: close\r\n\r\n",
                );
                relay_upstream(&mut client, &mut remote, conn_start);
                let lat = conn_start.elapsed().as_secs_f64() * 1000.0;
                log::error!("  done {} ({lat:.0}ms)", request.target);
                pool.record_result(&fb_url, true, lat);
                log_access(&AccessLogEntry {
                    method: request.method,
                    target: request.target,
                    upstream_idx: fb_idx as isize,
                    success: true,
                    latency_ms: lat,
                    circuit_state: "fallback",
                });
                return;
            }
            Err(e) => {
                last_err = format!("#{fb_idx}: {e}");
                log::error!("  ✗ #{fb_idx}: {e}");
                pool.record_result(&fb_url, false, conn_start.elapsed().as_secs_f64() * 1000.0);
            }
        }
    }

    log::error!("  all upstreams failed, trying direct: {last_err}");
    match try_direct(&request.target) {
        Ok(mut remote) => {
            log::error!("  ✓ direct");
            let _ = client.write_all(
                b"HTTP/1.1 200 Connection Established\r\nProxy-Connection: close\r\n\r\n",
            );
            crate::relay::relay_with_lifetime(&mut client, &mut remote, conn_start);
            let lat = conn_start.elapsed().as_secs_f64() * 1000.0;
            log::error!("  done {} ({lat:.0}ms)", request.target);
            log_access(&AccessLogEntry {
                method: request.method,
                target: request.target,
                upstream_idx: -1,
                success: true,
                latency_ms: lat,
                circuit_state: "direct",
            });
            return;
        }
        Err(e) => {
            last_err = format!("direct: {e}");
            log::error!("  ✗ direct: {e}");
        }
    }

    let lat = conn_start.elapsed().as_secs_f64() * 1000.0;
    log::error!("  all upstreams + direct failed: {last_err}");
    send_error(&mut client, 502, &format!("Bad Gateway ({last_err})"));
    log_access(&AccessLogEntry {
        method: request.method,
        target: request.target,
        upstream_idx: -2,
        success: false,
        latency_ms: lat,
        circuit_state: "all_failed",
    });
}
