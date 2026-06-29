//! HTTP/2 multiplexing layer for nt-proxy-daemon.
//!
//! ## Motivation
//!
//! USENIX Security '24 ("Fingerprinting Obfuscated Proxy Traffic with
//! Encapsulated TLS Handshakes") demonstrated that passive observers detect
//! proxy usage at 77%+ TPR by fingerprinting encapsulated TLS handshakes.
//! The paper found that stream multiplexing drops detection rate to **14.84%**
//! (a 5× reduction) because multiplexed connections share a single upstream
//! TLS handshake instead of creating one per request.
//!
//! ## Design
//!
//! Instead of establishing one SOCKS5 connection per downstream CONNECT
//! (each triggering a fresh TLS handshake to the upstream), we maintain a
//! pool of persistent HTTP/2 connections to upstream proxies.  Each
//! downstream CONNECT becomes an HTTP/2 CONNECT stream (RFC 7540 §8.3) on
//! an existing connection.  Multiple streams share one TLS connection →
//! no per-request TLS handshake → no encapsulated TLS fingerprint.
//!
//! ```text
//! ┌──────────────┐   HTTP CONNECT   ┌──────────────────┐  H2 CONNECT  ┌──────────────┐
//! │  Client A    │ ───────────────► │                  │ ───────────► │   Upstream   │
//! └──────────────┘                  │  nt-proxy-daemon  │              │   Proxy      │
//! ┌──────────────┐   HTTP CONNECT   │  (HTTP/2 MUX)    │  H2 CONNECT  │  (h2 server) │
//! │  Client B    │ ───────────────► │                  │ ───────────► │              │
//! └──────────────┘                  └──────────────────┘              └──────────────┘
//!                                              │
//!                                     persistent TLS (1 handshake)
//! ```
//!
//! ## Feature gate
//!
//! Compiled only when `feature = "mux"` is enabled.  Falls back to regular
//! SOCKS5 when the upstream doesn't support HTTP/2 or when the connection
//! pool is exhausted.

#![cfg(feature = "mux")]

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bytes::Bytes;
use futures_util::StreamExt;
use h2::{RecvStream, SendStream};
use http::Request;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;

use crate::dns::dns_cache;
use crate::obfuscation::jitter_sleep;
use crate::tls::{tls_config_random, UpstreamScheme};

// ─── Constants ───────────────────────────────────────────────────────────

/// Default max concurrent streams per HTTP/2 connection.
/// RFC 7540 §5.1.2 mandates a minimum of 100.
const DEFAULT_MAX_STREAMS: u32 = 100;

/// Default idle timeout before closing an unused upstream connection.
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/// Connection-pool eviction interval.
const EVICTION_INTERVAL: Duration = Duration::from_secs(60);

/// TLS server name for upstream verification.
/// Using the hostname from the upstream URL.
///
/// Note: Some upstreams may use non-standard SNI.  The `tls_config_random()`
/// config does NOT pin certificates beyond root-CA trust, so this is fine
/// for the proxy use case.

// ─── Public API ──────────────────────────────────────────────────────────

/// Result of attempting an HTTP/2 multiplexed connection.
#[must_use = "check if MUX succeeded before falling back to SOCKS5"]
pub enum MuxAttempt {
    /// MUX succeeded; caller must use the stream for relay.
    Connected(MuxStream),
    /// MUX not available (no existing connection, upstream doesn't speak
    /// HTTP/2, or pool exhausted).  Caller should fall back to SOCKS5.
    Fallback,
}

/// A single multiplexed HTTP/2 CONNECT stream.
///
/// Implements `Read + Write` and can be used transparently in relay code.
pub struct MuxStream {
    /// Synchronous end of a Unix socket pair bridged to the h2 stream.
    local: std::os::unix::net::UnixStream,
    /// Keeps the async bridge task alive until the stream drops.
    _liveness: Arc<tokio::sync::Notify>,
    /// When this stream was created (for latency accounting).
    #[allow(dead_code)]
    created: Instant,
}

impl MuxStream {
    /// Time elapsed since stream creation.
    #[allow(dead_code)]
    pub fn elapsed(&self) -> Duration {
        self.created.elapsed()
    }
}

impl Read for MuxStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.local.read(buf)
    }
}

impl Write for MuxStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.local.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.local.flush()
    }
}

/// Multi-threaded HTTP/2 connection pool manager.
///
/// Maintains persistent upstream HTTP/2 connections keyed by `(host, port)`.
/// Each connection serves up to `max_streams_per_connection` concurrent
/// CONNECT streams.  Idle connections are expired after
/// `connection_idle_timeout`.
pub struct MuxManager {
    rt: Arc<Runtime>,
    inner: Arc<Mutex<Inner>>,
    shutdown: Arc<AtomicBool>,
}

struct Inner {
    connections: HashMap<UpstreamKey, ConnState>,
    config: MuxConfig,
}

#[derive(Clone, Copy)]
struct MuxConfig {
    max_streams_per_connection: u32,
    connection_idle_timeout: Duration,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct UpstreamKey {
    host: String,
    port: u16,
}

struct ConnState {
    send_request: h2::client::SendRequest<Bytes>,
    active_streams: Arc<AtomicUsize>,
    last_used: Instant,
}

// ─── Implementation ──────────────────────────────────────────────────────

impl MuxConfig {
    fn default_with_overrides(
        max_streams: Option<u32>,
        idle_timeout: Option<Duration>,
    ) -> Self {
        Self {
            max_streams_per_connection: max_streams.unwrap_or(DEFAULT_MAX_STREAMS),
            connection_idle_timeout: idle_timeout.unwrap_or(DEFAULT_IDLE_TIMEOUT),
        }
    }
}

impl MuxManager {
    /// Create a new MUX manager with a dedicated tokio runtime.
    ///
    /// Spawns a background eviction thread that periodically purges stale
    /// connections from the pool.
    pub fn new(max_streams: Option<u32>, idle_timeout: Option<Duration>) -> Self {
        let rt = Arc::new(
            Runtime::new().expect("mux: failed to create tokio runtime"),
        );
        let config = MuxConfig::default_with_overrides(max_streams, idle_timeout);
        let inner = Arc::new(Mutex::new(Inner {
            connections: HashMap::new(),
            config,
        }));
        let shutdown = Arc::new(AtomicBool::new(false));

        // Background eviction thread
        let evict_inner = Arc::clone(&inner);
        let evict_rt = Arc::clone(&rt);
        let evict_shutdown = Arc::clone(&shutdown);
        std::thread::Builder::new()
            .name("mux-evict".into())
            .spawn(move || {
                evict_rt.block_on(async move {
                    while !evict_shutdown.load(Ordering::Relaxed) {
                        tokio::time::sleep(EVICTION_INTERVAL).await;
                        if evict_shutdown.load(Ordering::Relaxed) { break; }
                        Self::evict_stale(&evict_inner);
                    }
                });
            })
            .expect("mux: eviction thread");

        Self { rt, inner, shutdown }
    }

    /// Attempt to establish a multiplexed CONNECT stream to `target`
    /// through the upstream proxy at `upstream_url`.
    ///
    /// Returns `MuxAttempt::Connected(stream)` on success.
    /// Returns `MuxAttempt::Fallback` when:
    ///   - The upstream is a plain (non-TLS) SOCKS5 proxy
    ///   - The connection pool is exhausted
    ///   - Any error occurs during setup
    ///
    /// The caller should fall back to SOCKS5 on `Fallback`.
    pub fn try_mux_connect(
        &self,
        target: &str,
        upstream_url: &str,
    ) -> MuxAttempt {
        // Parse upstream — requires TLS for HTTP/2
        let (upstream_host, upstream_port, scheme) =
            match Self::parse_upstream(upstream_url) {
                Some(v) => v,
                None => return MuxAttempt::Fallback,
            };

        if scheme != UpstreamScheme::Tls {
            log::error!("[mux] skipping: non-TLS upstream {upstream_url}");
            return MuxAttempt::Fallback;
        }

        // SSRF check
        if crate::ssrf::is_private_target(target) {
            return MuxAttempt::Fallback;
        }

        self.rt.block_on(async move {
            let conn = match self
                .get_or_create_connection(&upstream_host, upstream_port)
                .await
            {
                Ok(c) => c,
                Err(e) => {
                    log::error!("[mux] conn failed: {e}");
                    return MuxAttempt::Fallback;
                }
            };

            match Self::create_connect_stream(&conn, target).await {
                Ok((send_stream, recv_stream)) => {
                    log::error!("[mux] CONNECT {target} via {upstream_url}");
                    let stream =
                        Self::bridge_to_sync(send_stream, recv_stream, self.rt.clone());
                    MuxAttempt::Connected(stream)
                }
                Err(e) => {
                    log::error!("[mux] stream failed: {e}");
                    MuxAttempt::Fallback
                }
            }
        })
    }

    // ─── Private helpers ───────────────────────────────────────────

    fn parse_upstream(url: &str) -> Option<(String, u16, UpstreamScheme)> {
        use crate::socks5::parse_socks5_url;
        let (host, port, _is_local, scheme) = parse_socks5_url(url)?;
        Some((host, port, scheme))
    }

    /// Get or create an HTTP/2 connection to the given upstream.
    async fn get_or_create_connection(
        &self,
        host: &str,
        port: u16,
    ) -> Result<ConnRef, MuxError> {
        let key = UpstreamKey {
            host: host.to_string(),
            port,
        };

        // Fast path: existing connection with capacity
        {
            let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(state) = inner.connections.get(&key) {
                if state.active_streams.load(Ordering::Acquire)
                    < inner.config.max_streams_per_connection as usize
                {
                    return Ok(ConnRef {
                        send_request: state.send_request.clone(),
                        active_streams: Arc::clone(&state.active_streams),
                    });
                }
            }
        }

        // Slow path: establish new connection
        let send_request = self.connect_new_upstream(host, port).await?;

        let active_streams = Arc::new(AtomicUsize::new(0));
        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.connections.insert(
                key,
                ConnState {
                    send_request: send_request.clone(),
                    active_streams: Arc::clone(&active_streams),
                    last_used: Instant::now(),
                },
            );
        }

        Ok(ConnRef {
            send_request,
            active_streams,
        })
    }

    /// Establish a new TLS + HTTP/2 connection to the upstream.
    async fn connect_new_upstream(
        &self,
        host: &str,
        port: u16,
    ) -> Result<h2::client::SendRequest<Bytes>, MuxError> {
        let addr = dns_cache()
            .resolve(host, port)
            .map_err(|e| MuxError::Resolve(e))?;

        // Jitter before connect (obfuscation)
        // Safe: jitter_sleep does not hold any lock
        jitter_sleep();

        let tcp = tokio::net::TcpStream::connect(addr)
            .await
            .map_err(|e| MuxError::Connect(e.to_string()))?;

        // TLS handshake using tokio-rustls
        let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
            .map_err(|e| MuxError::Tls(e.to_string()))?;

        let tls_config = tls_config_random();
        let connector = tokio_rustls::TlsConnector::from(tls_config.clone());
        let mut tls_stream = connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| MuxError::Tls(e.to_string()))?;

        // Flush the TLS handshake (tokio-rustls does the handshake in connect).
        // A zero-length write triggers any remaining handshake frames.
        let _ = tls_stream.write(&[]).await;

        // h2 handshake (sends client preface, waits for SETTINGS)
        let (send_request, h2_conn) = h2::client::handshake(tls_stream)
            .await
            .map_err(|e| MuxError::Http2(e.to_string()))?;

        // Spawn the connection driver on our runtime.
        tokio::spawn(h2_conn);

        log::error!("[mux] new HTTP/2 connection to {host}:{port}");
        Ok(send_request)
    }

    /// Send an HTTP/2 CONNECT request on the given connection.
    async fn create_connect_stream(
        conn: &ConnRef,
        target: &str,
    ) -> Result<
        (
            SendStream<Bytes>,
            RecvStream,
        ),
        MuxError,
    > {
        let mut sender = conn.send_request.clone();
        let request = Request::builder()
            .method("CONNECT")
            .uri(target)
            .body(())
            .map_err(|e| MuxError::Http2(e.to_string()))?;

        let (response_fut, send_stream) = sender
            .send_request(request, false)
            .map_err(|e| MuxError::Http2(e.to_string()))?;

        conn.active_streams.fetch_add(1, Ordering::Release);

        let response = response_fut
            .await
            .map_err(|e| MuxError::Http2(e.to_string()))?;
        let status = response.status();
        let recv_stream = response.into_body();
        if status != http::StatusCode::OK {
            conn.active_streams.fetch_sub(1, Ordering::Release);
            return Err(MuxError::Rejected(status));
        }

        Ok((send_stream, recv_stream))
    }

    /// Bridge an h2 stream pair to a synchronous `MuxStream` using a
    /// Unix socket pair.
    ///
    /// A background tokio task does bidirectional copy between the h2
    /// stream and one end of the socket.  The other end is returned
    /// as the sync `MuxStream`.
    fn bridge_to_sync(
        send_stream: SendStream<Bytes>,
        recv_stream: RecvStream,
        _rt: Arc<Runtime>,
    ) -> MuxStream {
        let (sync_end, async_end) =
            std::os::unix::net::UnixStream::pair().expect("mux: unix pair");
        tune_unix_socket(&sync_end);
        tune_unix_socket(&async_end);

        let async_end = tokio::net::UnixStream::from_std(async_end)
            .expect("mux: tokio unix from_std");
        let liveness = Arc::new(tokio::sync::Notify::new());
        let liveness_clone = Arc::clone(&liveness);

        tokio::spawn(async move {
            let result = Self::bridge_loop(send_stream, recv_stream, async_end).await;
            if let Err(e) = &result {
                log::error!("[mux] bridge ended: {e}");
            }
            liveness_clone.notify_one();
        });

        MuxStream {
            local: sync_end,
            _liveness: liveness,
            created: Instant::now(),
        }
    }

    /// Async loop: bidirectional copy between h2 stream and Unix socket.
    async fn bridge_loop(
        mut send_stream: SendStream<Bytes>,
        recv_stream: RecvStream,
        mut unix: tokio::net::UnixStream,
    ) -> Result<(), String> {
        let (mut unix_rd, mut unix_wr) = unix.split();

        // Direction 1: h2 recv → unix write
        let h2_to_unix = async {
            let mut stream = recv_stream;
            while let Some(chunk) = stream.next().await {
                let data = chunk.map_err(|e| format!("h2 recv: {e}"))?;
                unix_wr
                    .write_all(&data)
                    .await
                    .map_err(|e| format!("unix write: {e}"))?;
            }
            Ok::<_, String>(())
        };

        // Direction 2: unix read → h2 send
        let unix_to_h2 = async {
            let mut buf = vec![0u8; 65536];
            loop {
                let n = unix_rd
                    .read(&mut buf)
                    .await
                    .map_err(|e| format!("unix read: {e}"))?;
                if n == 0 {
                    let _ = send_stream.send_data(Bytes::new(), true);
                    return Ok::<_, String>(());
                }
                let chunk = Bytes::copy_from_slice(&buf[..n]);
                send_stream
                    .send_data(chunk, false)
                    .map_err(|e| format!("h2 send: {e}"))?;
            }
        };

        tokio::select! {
            r = h2_to_unix => r,
            r = unix_to_h2 => r,
        }
    }

    fn evict_stale(inner: &Arc<Mutex<Inner>>) {
        let Ok(mut guard) = inner.lock() else { return };
        let timeout = guard.config.connection_idle_timeout;
        let now = Instant::now();
        guard.connections.retain(|key, state| {
            let alive = now.saturating_duration_since(state.last_used) < timeout;
            if !alive {
                log::error!("[mux] evict idle {host}:{port}", host = key.host, port = key.port);
            }
            alive
        });
    }
}

/// A borrowed reference to an existing connection.
struct ConnRef {
    send_request: h2::client::SendRequest<Bytes>,
    active_streams: Arc<AtomicUsize>,
}

impl Drop for ConnRef {
    fn drop(&mut self) {
        // Decrement stream count when a MuxStream is dropped.
        self.active_streams.fetch_sub(1, Ordering::Release);
    }
}

// ─── Internal helpers ────────────────────────────────────────────────────

fn tune_unix_socket(sock: &std::os::unix::net::UnixStream) {
    let _ = sock.set_nonblocking(false);
    let _ = sock.set_read_timeout(Some(Duration::from_secs(120)));
    let _ = sock.set_write_timeout(Some(Duration::from_secs(120)));
}

#[derive(Debug)]
enum MuxError {
    Resolve(String),
    Connect(String),
    Tls(String),
    Http2(String),
    Rejected(http::StatusCode),
}

impl std::fmt::Display for MuxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MuxError::Resolve(e) => write!(f, "dns: {e}"),
            MuxError::Connect(e) => write!(f, "tcp: {e}"),
            MuxError::Tls(e) => write!(f, "tls: {e}"),
            MuxError::Http2(e) => write!(f, "h2: {e}"),
            MuxError::Rejected(s) => write!(f, "rejected {s}"),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_upstream_tls() {
        let (h, p, s) =
            MuxManager::parse_upstream("socks5+tls://proxy.x:1080").unwrap();
        assert_eq!(h, "proxy.x");
        assert_eq!(p, 1080);
        assert_eq!(s, UpstreamScheme::Tls);
    }

    #[test]
    fn test_parse_upstream_plain_skipped() {
        let (_, _, s) =
            MuxManager::parse_upstream("socks5://proxy.x:1080").unwrap();
        assert_eq!(s, UpstreamScheme::Plain);
    }

    #[test]
    fn test_parse_upstream_invalid() {
        assert!(MuxManager::parse_upstream("").is_none());
        assert!(MuxManager::parse_upstream("http://h:80").is_none());
    }

    #[test]
    fn test_config_defaults() {
        let cfg = MuxConfig::default_with_overrides(None, None);
        assert_eq!(cfg.max_streams_per_connection, DEFAULT_MAX_STREAMS);
        assert_eq!(cfg.connection_idle_timeout, DEFAULT_IDLE_TIMEOUT);

        let custom =
            MuxConfig::default_with_overrides(Some(50), Some(Duration::from_secs(60)));
        assert_eq!(custom.max_streams_per_connection, 50);
        assert_eq!(custom.connection_idle_timeout.as_secs(), 60);
    }
}
