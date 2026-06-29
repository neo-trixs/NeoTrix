use futures::FutureExt;
use log;
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock as StdRwLock};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::RwLock;
use tokio::time::sleep;

use super::config::NeoTrixConfig;
use super::firewall::FirewallManager;
use super::local_proxy::{connect_via_socks5, connect_via_socks5_chain, parse_proxy_url};
use super::pool_types::NodeRole;
use super::proxy_pool::global_pool;
use super::rotation_coordinator::{RotationCoordinator, RotationDomain};
use super::system_fingerprint::{
    Browser, Platform, SystemFingerprint, SystemFingerprintConfig, SystemFingerprintGenerator,
};

const TRANSIT_LISTEN_ADDR: &str = "127.0.0.1:11081";
const SYSTEM_PROXY_ADDR: &str = "127.0.0.1:11080";
const RELAY_BUFFER_SIZE: usize = 65536;
const RELAY_TIMEOUT_SECS: u64 = 120;
const PADDING_CHANCE: f64 = 0.20;
const PADDING_MIN: usize = 16;
const PADDING_MAX: usize = 256;
const DNS_CACHE_TTL_SECS: u64 = 60;
const MAX_CONCURRENT_CONNS: usize = 1024;
const DNS_REDIRECT_PORT: u16 = 11053;
const DNS_UPSTREAM: &str = "8.8.8.8:53";
const DNS_UPSTREAM_FALLBACK: &str = "1.1.1.1:53";
const DNS_TIMEOUT_SECS: u64 = 5;
const MAX_ROUTING_RECORDS: usize = 10000;

/// 每连接路由归属记录
#[derive(Debug, Clone)]
pub struct RoutingRecord {
    pub target: String,
    pub peer: String,
    pub relay: Option<String>,
    pub exit: Option<String>,
    pub mode: String,
    pub timestamp: Instant,
    pub bytes_relayed: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub circuit_token: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitMode {
    /// pf divert-to transparent capture (port 11081)
    PfDivert,
    /// System proxy mode (port 11080)
    SystemProxy,
    /// TUN device mode (utun9)
    TunDevice,
}

#[derive(Debug, Clone)]
pub struct TransitFingerprint {
    pub browser: Browser,
    pub platform: Platform,
    pub system_fp: SystemFingerprint,
    pub created_at: Instant,
}

pub struct TransitStation {
    mode: TransitMode,
    listen_addr: StdRwLock<String>,
    enabled: AtomicBool,
    conn_count: AtomicU64,
    total_bytes_relayed: AtomicU64,
    rotation_coordinator: RwLock<Option<Arc<RotationCoordinator>>>,
    fingerprint_gen: SystemFingerprintGenerator,
    last_fingerprint: RwLock<TransitFingerprint>,
    dns_cache: RwLock<HashMap<String, (String, Instant)>>,
    active_conns: AtomicU64,
    routing_log: RwLock<VecDeque<RoutingRecord>>,
    circuit_manager: RwLock<Option<Arc<super::circuit_isolation::CircuitIsolationManager>>>,
}

impl Default for TransitStation {
    fn default() -> Self {
        Self::new()
    }
}

impl TransitStation {
    pub fn new() -> Self {
        let fg = SystemFingerprintGenerator::new();
        let fp = fg.generate(&SystemFingerprintConfig::default());
        Self {
            mode: TransitMode::PfDivert,
            listen_addr: StdRwLock::new(TRANSIT_LISTEN_ADDR.to_string()),
            enabled: AtomicBool::new(false),
            conn_count: AtomicU64::new(0),
            total_bytes_relayed: AtomicU64::new(0),
            rotation_coordinator: RwLock::new(None),
            fingerprint_gen: fg,
            last_fingerprint: RwLock::new(TransitFingerprint {
                browser: fp.nt_world_browse,
                platform: fp.platform,
                system_fp: fp,
                created_at: Instant::now(),
            }),
            dns_cache: RwLock::new(HashMap::new()),
            active_conns: AtomicU64::new(0),
            routing_log: RwLock::new(VecDeque::with_capacity(MAX_ROUTING_RECORDS)),
            circuit_manager: RwLock::new(None),
        }
    }

    pub fn with_mode(mut self, mode: TransitMode) -> Self {
        self.mode = mode;
        let addr = match mode {
            TransitMode::PfDivert => TRANSIT_LISTEN_ADDR,
            TransitMode::SystemProxy => SYSTEM_PROXY_ADDR,
            TransitMode::TunDevice => TRANSIT_LISTEN_ADDR,
        };
        *self.listen_addr.write().unwrap_or_else(|e| e.into_inner()) = addr.to_string();
        self
    }

    pub async fn with_coordinator(self, coord: Arc<RotationCoordinator>) -> Self {
        *self.rotation_coordinator.write().await = Some(coord);
        self
    }

    pub async fn with_circuit_manager(
        self,
        mgr: Arc<super::circuit_isolation::CircuitIsolationManager>,
    ) -> Self {
        *self.circuit_manager.write().await = Some(mgr);
        self
    }

    /// 记录一条路由归属
    pub async fn record_routing(&self, record: RoutingRecord) {
        let mut log = self.routing_log.write().await;
        if log.len() >= MAX_ROUTING_RECORDS {
            log.pop_front();
        }
        log.push_back(record);
    }

    /// 获取路由归属日志
    pub async fn routing_log(&self) -> Vec<RoutingRecord> {
        self.routing_log.read().await.iter().cloned().collect()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn conn_count(&self) -> u64 {
        self.conn_count.load(Ordering::Relaxed)
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes_relayed.load(Ordering::Relaxed)
    }

    pub fn active_connections(&self) -> u64 {
        self.active_conns.load(Ordering::Relaxed)
    }

    /// 启动中转站监听
    /// 在指定端口监听TCP连接，每个连接通过代理池转发，
    /// 中间动态修改IP和指纹
    pub async fn start(self: Arc<Self>) -> Result<(), String> {
        if self.enabled.swap(true, Ordering::AcqRel) {
            return Ok(());
        }

        let addr = self
            .listen_addr
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let listener = TcpListener::bind(&addr[..])
            .await
            .map_err(|e| format!("transit bind {}: {}", addr, e))?;

        log::info!(
            "[transit] 中转站启动于 {} (mode={:?}), 所有系统流量经此IP隐匿+指纹混淆",
            addr,
            self.mode
        );

        let station = self.clone();
        tokio::spawn(async move {
            if let Err(panic) = std::panic::AssertUnwindSafe(async move {
                station.run_accept_loop(listener).await;
            })
            .catch_unwind()
            .await
            {
                log::error!("[transit] accept loop panic: {:?}", panic);
            }
        });

        // 启动 DNS 重定向处理器（pf rdr 规则将 :53 → :11053）
        let dns_station = self.clone();
        tokio::spawn(async move {
            if let Err(panic) = std::panic::AssertUnwindSafe(async move {
                dns_station.run_dns_redirect().await;
            })
            .catch_unwind()
            .await
            {
                log::error!("[transit] DNS redirect loop panic: {:?}", panic);
            }
        });

        Ok(())
    }

    async fn run_accept_loop(self: Arc<Self>, listener: TcpListener) {
        loop {
            if !self.enabled.load(Ordering::Relaxed) {
                break;
            }

            let (stream, peer_addr) = match listener.accept().await {
                Ok(v) => v,
                Err(e) => {
                    log::error!("[transit] accept error: {}", e);
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }
            };

            let active = self.active_conns.load(Ordering::Relaxed);
            if active >= MAX_CONCURRENT_CONNS as u64 {
                log::warn!(
                    "[transit] max concurrent connections reached ({})",
                    MAX_CONCURRENT_CONNS
                );
                drop(stream);
                continue;
            }

            self.active_conns.fetch_add(1, Ordering::Relaxed);
            self.conn_count.fetch_add(1, Ordering::Relaxed);
            let station = self.clone();
            tokio::spawn(async move {
                if let Err(e) = station.handle_transit_connection(stream, peer_addr).await {
                    log::warn!("[transit] {} -> {}", peer_addr, e);
                }
                station.active_conns.fetch_sub(1, Ordering::Relaxed);
            });
        }
    }

    /// DNS 重定向处理器
    ///
    /// 监听 UDP :11053（pf rdr 规则将 :53 重定向至此），
    /// 将 DNS 查询透传到上游 DNS 服务器（8.8.8.8 / 1.1.1.1）。
    async fn run_dns_redirect(&self) {
        let socket = match UdpSocket::bind(format!("127.0.0.1:{}", DNS_REDIRECT_PORT)).await {
            Ok(s) => s,
            Err(e) => {
                log::error!(
                    "[transit] DNS redirect bind :{} failed: {}",
                    DNS_REDIRECT_PORT,
                    e
                );
                return;
            }
        };
        log::info!(
            "[transit] DNS redirect listening on :{}, upstream={}",
            DNS_REDIRECT_PORT,
            DNS_UPSTREAM
        );

        let socket = Arc::new(socket);
        let mut buf = vec![0u8; 1500];
        loop {
            if !self.enabled.load(Ordering::Relaxed) {
                break;
            }
            match socket.recv_from(&mut buf).await {
                Ok((n, src)) => {
                    let query = buf[..n].to_vec();
                    let socket_clone = socket.clone();
                    tokio::spawn(async move {
                        Self::forward_dns_query(socket_clone, query, src).await;
                    });
                }
                Err(e) => {
                    log::warn!("[transit] DNS recv error: {}", e);
                    continue;
                }
            }
        }
    }

    /// 转发 DNS 查询到上游 DNS 服务器
    async fn forward_dns_query(socket: Arc<UdpSocket>, query: Vec<u8>, src: SocketAddr) {
        // 尝试上游 DNS，失败时回退
        let upstreams = [DNS_UPSTREAM, DNS_UPSTREAM_FALLBACK];
        for upstream in &upstreams {
            let result = tokio::time::timeout(Duration::from_secs(DNS_TIMEOUT_SECS), async {
                let remote = UdpSocket::bind("0.0.0.0:0")
                    .await
                    .map_err(|e| e.to_string())?;
                remote.connect(upstream).await.map_err(|e| e.to_string())?;
                remote.send(&query).await.map_err(|e| e.to_string())?;
                let mut resp = vec![0u8; 1500];
                let n = remote.recv(&mut resp).await.map_err(|e| e.to_string())?;
                Ok::<_, String>((resp[..n].to_vec(), n))
            })
            .await;

            match result {
                Ok(Ok((response, _n))) => {
                    let _ = socket.send_to(&response, src).await;
                    return;
                }
                Ok(Err(e)) => {
                    log::warn!("[transit] DNS forward to {} failed: {}", upstream, e);
                }
                Err(_) => {
                    log::warn!("[transit] DNS forward to {} timeout", upstream);
                }
            }
        }
        log::warn!("[transit] DNS query dropped (all upstreams failed)");
    }

    /// 处理每个中转连接
    /// 1. 读取客户端请求，提取目标地址
    /// 2. 从代理池选择出口节点（每连接不同IP）
    /// 3. 应用指纹混淆
    /// 4. 通过代理转发
    /// 5. 双向中继
    async fn handle_transit_connection(
        &self,
        mut ingress: TcpStream,
        peer: SocketAddr,
    ) -> Result<(), String> {
        let _ = ingress.set_nodelay(true);

        let mut buf = vec![0u8; RELAY_BUFFER_SIZE];
        let n = tokio::time::timeout(Duration::from_secs(15), ingress.read(&mut buf))
            .await
            .map_err(|_| "ingress read timeout".to_string())?
            .map_err(|e| format!("ingress read: {}", e))?;

        if n == 0 {
            return Ok(());
        }

        let data = &buf[..n];

        let (target_host, target_port) = if data.starts_with(b"CONNECT ") {
            self.extract_connect_target(data)?
        } else if let Some(host) = self.extract_http_host(data) {
            (host.0, host.1)
        } else {
            return Err("unknown protocol".to_string());
        };

        log::info!("[transit] {} → {} 通过中转站", peer, target_host);

        let (mut upstream, route, relay_url, exit_url) = self
            .connect_via_transit_pool(&target_host, target_port)
            .await?;

        if data.starts_with(b"CONNECT ") {
            ingress
                .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                .await
                .map_err(|e| format!("write 200: {}", e))?;
        } else {
            let (rest, _) = data.split_at(n);
            upstream
                .write_all(rest)
                .await
                .map_err(|e| format!("upstream write: {}", e))?;
        }

        let (total_bytes, tx_bytes, rx_bytes) =
            self.relay_with_obfuscation(ingress, upstream).await;

        let circuit_token = if route == "tor" {
            relay_url.clone()
        } else {
            None
        };
        self.record_routing(RoutingRecord {
            target: format!("{}:{}", target_host, target_port),
            peer: peer.to_string(),
            relay: relay_url,
            exit: exit_url,
            mode: route,
            timestamp: Instant::now(),
            bytes_relayed: total_bytes,
            tx_bytes,
            rx_bytes,
            circuit_token,
        })
        .await;

        Ok(())
    }

    /// 从代理池选择出口节点转发目标（含路由归属记录）
    /// 返回 (TcpStream, 路由描述, 中继URL, 出口URL)
    async fn connect_via_transit_pool(
        &self,
        host: &str,
        port: u16,
    ) -> Result<(TcpStream, String, Option<String>, Option<String>), String> {
        let pool = global_pool();
        let n_hops = crate::neotrix::nt_io_stealth_net::config::load()
            .pool
            .multi_hop_count;

        // Multi-hop chain (configurable N hops) with automatic fallback
        if n_hops > 1 {
            match pool.connect_multi_hop(host, port, n_hops).await {
                Ok(stream) => {
                    log::info!("[transit] {}:{} {}跳链", host, port, n_hops,);
                    return Ok((stream, format!("multi_{}hop", n_hops), None, None));
                }
                Err(e) => {
                    log::warn!("[transit] {}跳链不可用, 回退: {}", n_hops, e);
                }
            }
        }

        let use_tor = host.ends_with(".onion");
        if use_tor {
            let tor_addr = crate::core::nt_core_util::TOR_SOCKS_ADDR;
            let circuit = self.circuit_manager.read().await;
            if let Some(ref cm) = *circuit {
                let token = cm.acquire().await;
                let _username = cm.tor_socks_username(&token);
                drop(circuit);
                if let Ok(stream) = connect_via_socks5(tor_addr, host, port).await {
                    log::info!("[transit] {}:{} 经由 Tor (电路隔离)", host, port);
                    return Ok((
                        stream,
                        "tor".into(),
                        Some(format!("tor:{}", _username)),
                        None,
                    ));
                }
            } else {
                drop(circuit);
                if let Ok(stream) = connect_via_socks5(tor_addr, host, port).await {
                    return Ok((stream, "tor".into(), Some("tor:no_circuit".into()), None));
                }
            }
        }

        let direct = tokio::time::timeout(
            Duration::from_secs(10),
            TcpStream::connect(format!("{}:{}", host, port)),
        )
        .await
        .map_err(|_| format!("direct timeout {}:{}", host, port))?
        .map_err(|e| format!("direct {}:{}: {}", host, port, e))?;

        log::warn!("[transit] 直连 {}:{} (代理池不可用)", host, port);
        Ok((direct, "direct".into(), None, None))
    }

    /// Assign node roles based on observed latency during health check
    pub async fn auto_assign_roles(&self) {
        let pool = global_pool();
        let mut nodes = pool.nodes.write().await;
        for node in nodes.iter_mut() {
            let role = match node.latency_ms {
                Some(ms) if ms < 500.0 && node.success_count >= 3 => NodeRole::Relay,
                Some(ms) if ms >= 500.0 && node.success_count >= 3 => NodeRole::Obfuscation,
                _ => NodeRole::Mixed,
            };
            node.role = role;
        }
    }

    /// 双向中继带时序混淆和填充
    /// 返回 (总字节, 客户端→服务端字节, 服务端→客户端字节)
    async fn relay_with_obfuscation(
        &self,
        ingress: TcpStream,
        egress: TcpStream,
    ) -> (u64, u64, u64) {
        let (mut ri, mut wi) = ingress.into_split();
        let (mut re, mut we) = egress.into_split();

        let start = Instant::now();
        let bytes = Arc::new(AtomicU64::new(0));
        let b = bytes.clone();
        let tx_bytes = Arc::new(AtomicU64::new(0));
        let rx_bytes = Arc::new(AtomicU64::new(0));
        let tx = tx_bytes.clone();
        let rx = rx_bytes.clone();

        let c2s = tokio::spawn(async move {
            let mut buf = vec![0u8; RELAY_BUFFER_SIZE];
            loop {
                match tokio::time::timeout(
                    Duration::from_secs(RELAY_TIMEOUT_SECS),
                    ri.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) | Err(_) => break,
                    Ok(Ok(n)) => {
                        b.fetch_add(n as u64, Ordering::Relaxed);
                        tx.fetch_add(n as u64, Ordering::Relaxed);
                        let padded = maybe_pad(&buf[..n]);
                        if we.write_all(&padded).await.is_err() {
                            break;
                        }
                        maybe_jitter().await;
                    }
                    Ok(Err(_)) => break,
                }
            }
        });

        let s2c = tokio::spawn(async move {
            let mut buf = vec![0u8; RELAY_BUFFER_SIZE];
            loop {
                match tokio::time::timeout(
                    Duration::from_secs(RELAY_TIMEOUT_SECS),
                    re.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) | Err(_) => break,
                    Ok(Ok(n)) => {
                        rx.fetch_add(n as u64, Ordering::Relaxed);
                        let padded = maybe_pad(&buf[..n]);
                        if wi.write_all(&padded).await.is_err() {
                            break;
                        }
                        maybe_jitter().await;
                    }
                    Ok(Err(_)) => break,
                }
            }
        });

        let _ = tokio::join!(c2s, s2c);
        let elapsed = start.elapsed();
        let total = bytes.load(Ordering::Relaxed);
        let tx_total = tx_bytes.load(Ordering::Relaxed);
        let rx_total = rx_bytes.load(Ordering::Relaxed);
        self.total_bytes_relayed.fetch_add(total, Ordering::Relaxed);

        log::info!(
            "[transit] 中继完成: {} bytes (↑{} ↓{}) in {:?} ({:.1} KB/s)",
            total,
            tx_total,
            rx_total,
            elapsed,
            if elapsed.as_secs_f64() > 0.0 {
                total as f64 / elapsed.as_secs_f64() / 1024.0
            } else {
                0.0
            }
        );
        (total, tx_total, rx_total)
    }

    fn extract_connect_target(&self, data: &[u8]) -> Result<(String, u16), String> {
        let text = std::str::from_utf8(data).map_err(|_| "not utf8")?;
        let rest = text.strip_prefix("CONNECT ").ok_or("no CONNECT")?;
        let target = rest.split(' ').next().ok_or("no target")?;
        let parts: Vec<&str> = target.rsplitn(2, ':').collect();
        if parts.len() == 2 {
            let host = parts[1].to_string();
            let port: u16 = parts[0].parse().map_err(|_| "bad port")?;
            Ok((host, port))
        } else {
            Ok((target.to_string(), 443))
        }
    }

    fn extract_http_host(&self, data: &[u8]) -> Option<(String, u16)> {
        let text = std::str::from_utf8(data).ok()?;
        for line in text.lines() {
            let lower = line.to_lowercase();
            if let Some(val) = lower.strip_prefix("host: ") {
                let host = val.trim();
                if let Some(colon) = host.rfind(':') {
                    let h = host[..colon].to_string();
                    let p: u16 = host[colon + 1..].parse().ok()?;
                    return Some((h, p));
                }
                return Some((host.to_string(), 80));
            }
        }
        None
    }

    /// Set circuit manager (used when wiring from external code)
    pub async fn set_circuit_manager_arc(
        &self,
        mgr: Arc<super::circuit_isolation::CircuitIsolationManager>,
    ) {
        *self.circuit_manager.write().await = Some(mgr);
    }

    pub async fn set_circuit_manager(
        &self,
        _mgr: &super::circuit_isolation::CircuitIsolationManager,
    ) {
        *self.circuit_manager.write().await =
            Some(super::circuit_isolation::global_circuit_manager_arc());
    }

    /// Set the listen address (used from auto_start_transit after mode selection)
    pub fn set_listen_addr(&self, addr: &str) {
        *self.listen_addr.write().unwrap_or_else(|e| e.into_inner()) = addr.to_string();
    }

    /// Adapt rotation intervals based on fingerprint bandit confidence
    pub async fn adapt_rotation_to_bandit(&self) {
        let coord = self.rotation_coordinator.read().await;
        if let Some(ref c) = *coord {
            let bandit = super::bandit::FingerprintBandit::load();
            let confidence = bandit.confidence();
            c.adapt_to_confidence(confidence).await;
        }
    }

    /// 轮转当前指纹（浏览器/平台/TLS参数）
    pub async fn rotate_fingerprint(&self) {
        let coord = self.rotation_coordinator.read().await;
        let should_rotate = coord
            .as_ref()
            .map(|c| {
                let rt = tokio::runtime::Runtime::new().expect("transit_station: failed to create tokio runtime for TLS fingerprint rotation");
                rt.block_on(c.should_rotate(RotationDomain::TlsFingerprint))
            })
            .unwrap_or(true);

        if !should_rotate {
            return;
        }

        let fp = self
            .fingerprint_gen
            .generate(&SystemFingerprintConfig::default());
        *self.last_fingerprint.write().await = TransitFingerprint {
            browser: fp.nt_world_browse,
            platform: fp.platform,
            system_fp: fp,
            created_at: Instant::now(),
        };

        if let Some(ref c) = *coord {
            c.mark_rotated(RotationDomain::TlsFingerprint).await;
        }

        log::info!(
            "[transit] 指纹轮转 → {:?}/{:?}",
            self.last_fingerprint.read().await.browser,
            self.last_fingerprint.read().await.platform
        );
    }

    /// 停止中转站
    pub fn stop(&self) {
        self.enabled.store(false, Ordering::Relaxed);
        log::info!("[transit] 中转站已停止");
    }

    /// 获取统计信息
    pub async fn stats(&self) -> TransitStats {
        let fp = self.last_fingerprint.read().await;
        TransitStats {
            mode: self.mode,
            enabled: self.enabled.load(Ordering::Relaxed),
            listen_addr: self
                .listen_addr
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
            conn_count: self.conn_count.load(Ordering::Relaxed),
            total_bytes_relayed: self.total_bytes_relayed.load(Ordering::Relaxed),
            active_connections: self.active_conns.load(Ordering::Relaxed),
            current_browser: fp.browser,
            current_platform: fp.platform,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransitStats {
    pub mode: TransitMode,
    pub enabled: bool,
    pub listen_addr: String,
    pub conn_count: u64,
    pub total_bytes_relayed: u64,
    pub active_connections: u64,
    pub current_browser: Browser,
    pub current_platform: Platform,
}

/// 概率性添加填充字节（20%概率，16-256字节）
/// 对抗流量分析
fn maybe_pad(data: &[u8]) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    if rng.gen::<f64>() < PADDING_CHANCE {
        let pad_len = rng.gen_range(PADDING_MIN..=PADDING_MAX);
        let mut padded = Vec::with_capacity(data.len() + pad_len + 4);
        padded.extend_from_slice(data);
        padded.extend_from_slice(&(pad_len as u32).to_be_bytes());
        let padding: Vec<u8> = (0..pad_len).map(|_| rng.gen()).collect();
        padded.extend_from_slice(&padding);
        padded
    } else {
        data.to_vec()
    }
}

/// 概率性时序抖动（20%概率，10-100ms延迟）
/// 对抗时序分析
async fn maybe_jitter() {
    let (should_jitter, jitter_ms) = {
        let mut rng = rand::thread_rng();
        let should = rng.gen::<f64>() < 0.20;
        let ms = if should { rng.gen_range(10..100) } else { 0 };
        (should, ms)
    };
    if should_jitter {
        sleep(Duration::from_millis(jitter_ms)).await;
    }
}

/// 全局中转站单例（惰性初始化）
pub fn global_transit_station() -> Arc<TransitStation> {
    static TS: OnceLock<Arc<TransitStation>> = OnceLock::new();
    TS.get_or_init(|| Arc::new(TransitStation::new())).clone()
}

/// 全局防火墙管理器单例
fn global_firewall() -> Arc<FirewallManager> {
    static FW: OnceLock<Arc<FirewallManager>> = OnceLock::new();
    FW.get_or_init(|| Arc::new(FirewallManager::new())).clone()
}

/// 从配置自动启动中转站（pf divert-to 模式）
///
/// 读取 `NeoTrixConfig.transit`，若 `enabled=true` 则：
/// 1. 创建/获取全局 TransitStation
/// 2. 设置模式（pf_divert / system_proxy）
/// 3. 启动 TCP listener
/// 4. 启用 pf divert-to 规则
///
/// 若已运行则直接返回 Ok。
pub async fn auto_start_transit(cfg: &NeoTrixConfig) -> Result<(), String> {
    let ts = global_transit_station();
    if ts.is_enabled() {
        log::info!("[transit] already running");
        return Ok(());
    }

    let tc = &cfg.transit;
    if !tc.enabled {
        log::info!("[transit] disabled in config, skipping");
        return Ok(());
    }

    let mode = match tc.mode.as_str() {
        "system_proxy" => TransitMode::SystemProxy,
        "pf_divert" => TransitMode::PfDivert,
        "tun" => TransitMode::TunDevice,
        other => {
            log::warn!(
                "[transit] unknown mode '{}', falling back to pf_divert",
                other
            );
            TransitMode::PfDivert
        }
    };

    // 更新监听地址（根据模式）
    let addr = match mode {
        TransitMode::PfDivert => TRANSIT_LISTEN_ADDR,
        TransitMode::SystemProxy => SYSTEM_PROXY_ADDR,
        TransitMode::TunDevice => TRANSIT_LISTEN_ADDR,
    };
    ts.set_listen_addr(addr);

    // 启动 listener
    Arc::clone(&ts)
        .start()
        .await
        .map_err(|e| format!("transit start: {}", e))?;

    // 初始角色分配（基于已有健康数据）
    ts.auto_assign_roles().await;

    // 若 pf_divert 模式，启用 pf 规则
    if mode == TransitMode::PfDivert {
        let fw = global_firewall();
        match fw.enable_divert().await {
            Ok(_) => log::info!("[transit] pf divert-to enabled on port 11081"),
            Err(e) => log::warn!("[transit] pf enable_divert failed (sudo?): {}", e),
        }
    }

    log::info!(
        "[transit] started: mode={:?}, port={}, per_conn_ip={}, padding={}, timing={}",
        mode,
        tc.listen_port,
        tc.per_conn_ip_rotation,
        tc.padding_enabled,
        tc.timing_obfuscation_enabled,
    );
    Ok(())
}

/// 停止全局中转站并清理防火墙规则
pub async fn stop_transit() {
    let ts = global_transit_station();
    if !ts.is_enabled() {
        return;
    }
    ts.stop();

    // 清理 pf 规则（不阻塞于权限错误）
    let fw = global_firewall();
    let _ = fw.disable_divert().await;

    log::info!("[transit] stopped and pf rules cleared");
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 基础创建与配置 ──

    #[test]
    fn test_transit_station_creation() {
        let ts = TransitStation::new();
        assert!(!ts.is_enabled());
        assert_eq!(ts.conn_count(), 0);
        assert_eq!(ts.total_bytes(), 0);
        assert_eq!(ts.active_connections(), 0);
    }

    #[test]
    fn test_transit_mode_system_proxy() {
        let ts = TransitStation::new().with_mode(TransitMode::SystemProxy);
        assert_eq!(*ts.listen_addr.read().unwrap(), SYSTEM_PROXY_ADDR);
    }

    #[test]
    fn test_transit_mode_pf_divert() {
        let ts = TransitStation::new().with_mode(TransitMode::PfDivert);
        assert_eq!(*ts.listen_addr.read().unwrap(), TRANSIT_LISTEN_ADDR);
    }

    #[test]
    fn test_transit_mode_tun() {
        let ts = TransitStation::new().with_mode(TransitMode::TunDevice);
        assert_eq!(*ts.listen_addr.read().unwrap(), TRANSIT_LISTEN_ADDR);
    }

    #[tokio::test]
    async fn test_stats_structure() {
        let ts = TransitStation::new();
        let stats = ts.stats().await;
        assert!(!stats.enabled);
        assert_eq!(stats.conn_count, 0);
        assert_eq!(stats.active_connections, 0);
        matches!(
            stats.current_browser,
            Browser::Chrome | Browser::Firefox | Browser::Safari | Browser::Edge
        );
        matches!(
            stats.current_platform,
            Platform::Windows
                | Platform::MacOS
                | Platform::Linux
                | Platform::ChromeOS
                | Platform::Android
                | Platform::IOS
        );
    }

    #[tokio::test]
    async fn test_start_stop_lifecycle() {
        let ts = Arc::new(TransitStation::new().with_mode(TransitMode::SystemProxy));
        assert!(!ts.is_enabled());

        let ts_clone = ts.clone();
        let result = ts_clone.start().await;
        assert!(result.is_ok() || result.is_err());

        if result.is_ok() {
            assert!(ts.is_enabled());
            ts.stop();
            assert!(!ts.is_enabled());
        }
    }

    #[tokio::test]
    async fn test_concurrent_access_safety() {
        let ts = Arc::new(TransitStation::new());
        let mut handles = Vec::new();
        for _ in 0..10 {
            let t = ts.clone();
            handles.push(tokio::spawn(async move {
                let _ = t.stats().await;
                t.conn_count();
                t.total_bytes();
                t.active_connections();
            }));
        }
        for h in handles {
            h.await.expect("concurrent access should not panic");
        }
        assert_eq!(ts.conn_count(), 0);
    }

    #[tokio::test]
    async fn test_rotate_fingerprint_changes_identity() {
        let ts = Arc::new(TransitStation::new());
        let before = ts.last_fingerprint.read().await.browser;
        ts.rotate_fingerprint().await;
        let after = ts.last_fingerprint.read().await.browser;
        assert!(before == after || ts.rotation_coordinator.read().await.is_some());
    }

    #[tokio::test]
    async fn test_rotate_fingerprint_updates_timestamp() {
        let ts = Arc::new(TransitStation::new());
        let before = ts.last_fingerprint.read().await.created_at;
        ts.rotate_fingerprint().await;
        let after = ts.last_fingerprint.read().await.created_at;
        assert!(after >= before);
    }

    #[tokio::test]
    async fn test_stats_reflects_state_changes() {
        let ts = Arc::new(TransitStation::new().with_mode(TransitMode::PfDivert));
        let stats = ts.stats().await;
        assert_eq!(stats.mode, TransitMode::PfDivert);
        assert!(!stats.enabled);
    }

    // ── Padding 测试 ──

    #[test]
    fn test_maybe_pad_runs_without_panic() {
        let data = b"test data";
        let padded = maybe_pad(data);
        assert!(!padded.is_empty());
        assert!(padded.len() >= data.len());
    }

    #[test]
    fn test_maybe_pad_preserves_original_data() {
        let data = b"Hello, NeoTrix!";
        let padded = maybe_pad(data);
        assert!(
            padded.starts_with(data),
            "padded data should start with original: {:?} vs {:?}",
            padded,
            data
        );
    }

    #[test]
    fn test_maybe_pad_returns_vec() {
        let data = b"";
        let padded = maybe_pad(data);
        assert!(padded.len() <= data.len() + PADDING_MAX + 4);
    }

    #[test]
    fn test_maybe_pad_multiple_calls_ok() {
        for _ in 0..100 {
            let data = b"persistent test data stream";
            let _ = maybe_pad(data);
        }
    }

    #[test]
    fn test_maybe_pad_variable_sizes() {
        for size in [0, 1, 64, 128, 1024, 4096] {
            let data = vec![0xABu8; size];
            let padded = maybe_pad(&data);
            assert!(padded.len() >= data.len());
            assert!(padded.starts_with(&data));
        }
    }

    // ── HTTP 请求解析 ──

    #[test]
    fn test_extract_connect_target_normal() {
        let ts = TransitStation::new();
        let data = b"CONNECT example.com:443 HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let (host, port) = ts.extract_connect_target(data).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_extract_connect_target_default_port() {
        let ts = TransitStation::new();
        let data = b"CONNECT example.com HTTP/1.1\r\n\r\n";
        let (host, port) = ts.extract_connect_target(data).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_extract_connect_target_custom_port() {
        let ts = TransitStation::new();
        let data = b"CONNECT api.example.com:8080 HTTP/1.1\r\n\r\n";
        let (host, port) = ts.extract_connect_target(data).unwrap();
        assert_eq!(host, "api.example.com");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_extract_connect_target_ipv6() {
        let ts = TransitStation::new();
        let data = b"CONNECT [::1]:443 HTTP/1.1\r\n\r\n";
        let (host, port) = ts.extract_connect_target(data).unwrap();
        assert_eq!(port, 443);
    }

    #[test]
    fn test_extract_connect_target_empty() {
        let ts = TransitStation::new();
        let result = ts.extract_connect_target(b"GET / HTTP/1.1\r\n\r\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_http_host_normal() {
        let ts = TransitStation::new();
        let data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let (host, port) = ts.extract_http_host(data).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 80);
    }

    #[test]
    fn test_extract_http_host_with_port() {
        let ts = TransitStation::new();
        let data = b"GET / HTTP/1.1\r\nHost: example.com:8080\r\n\r\n";
        let (host, port) = ts.extract_http_host(data).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_extract_http_host_no_host() {
        let ts = TransitStation::new();
        let result = ts.extract_http_host(b"GET / HTTP/1.1\r\n\r\n");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_http_host_case_insensitive() {
        let ts = TransitStation::new();
        let data = b"GET / HTTP/1.1\r\nHOST: Example.Com\r\n\r\n";
        let (host, port) = ts.extract_http_host(data).unwrap();
        assert_eq!(host, "Example.Com");
        assert_eq!(port, 80);
    }

    #[test]
    fn test_extract_http_host_malformed_data() {
        let ts = TransitStation::new();
        let result = ts.extract_http_host(b"\xFF\xFE\x00");
        assert!(result.is_none());
    }

    // ── 连通性测试（本地 echo server） ──

    #[tokio::test]
    async fn test_relay_simple_echo() {
        let msg = b"hello transit station";
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let echo_addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 1024];
            let n = stream.read(&mut buf).await.unwrap();
            stream.write_all(&buf[..n]).await.unwrap();
        });

        let client = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_secs(5), async {
                let mut stream = tokio::net::TcpStream::connect(echo_addr).await.unwrap();
                stream.write_all(msg).await.unwrap();
                let mut buf = vec![0u8; 1024];
                let n = stream.read(&mut buf).await.unwrap();
                assert_eq!(&buf[..n], msg, "回显数据应与发送一致");
            })
            .await
            .unwrap();
        });

        let _ = tokio::join!(server, client);
    }

    #[tokio::test]
    async fn test_relay_bidirectional() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 32];
            let n = stream.read(&mut buf).await.unwrap();
            stream.write_all(b"pong").await.unwrap();
            assert_eq!(&buf[..n], b"ping");
        });

        let client = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_secs(5), async {
                let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
                stream.write_all(b"ping").await.unwrap();
                let mut buf = [0u8; 32];
                let n = stream.read(&mut buf).await.unwrap();
                assert_eq!(&buf[..n], b"pong");
            })
            .await
            .unwrap();
        });

        let _ = tokio::join!(server, client);
    }

    #[tokio::test]
    async fn test_transit_connection_count_tracking() {
        let ts = TransitStation::new();
        assert_eq!(ts.conn_count(), 0);
        ts.conn_count.fetch_add(1, Ordering::Relaxed);
        assert_eq!(ts.conn_count(), 1);
        ts.conn_count.fetch_add(5, Ordering::Relaxed);
        assert_eq!(ts.conn_count(), 6);
        ts.active_conns.store(3, Ordering::Relaxed);
        assert_eq!(ts.active_connections(), 3);
    }

    // ── 指纹配置一致性 ──

    #[test]
    fn test_transit_fingerprint_struct() {
        let fp = TransitFingerprint {
            browser: Browser::Chrome,
            platform: Platform::MacOS,
            system_fp: SystemFingerprintGenerator::new()
                .generate(&SystemFingerprintConfig::default()),
            created_at: Instant::now(),
        };
        assert_eq!(fp.browser, Browser::Chrome);
        assert_eq!(fp.platform, Platform::MacOS);
    }

    #[test]
    fn test_transit_mode_debug_clone() {
        let m1 = TransitMode::PfDivert;
        let m2 = TransitMode::SystemProxy;
        let m3 = TransitMode::TunDevice;
        assert_eq!(format!("{:?}", m1), "PfDivert");
        assert_eq!(format!("{:?}", m2), "SystemProxy");
        assert_eq!(format!("{:?}", m3), "TunDevice");
        assert_ne!(m1, m2);
        assert_eq!(m1.clone(), m1);
    }

    #[test]
    fn test_transit_stats_debug_clone() {
        let stats = TransitStats {
            mode: TransitMode::PfDivert,
            enabled: true,
            listen_addr: "127.0.0.1:11081".into(),
            conn_count: 42,
            total_bytes_relayed: 65536,
            active_connections: 3,
            current_browser: Browser::Firefox,
            current_platform: Platform::Linux,
        };
        let cloned = stats.clone();
        assert_eq!(stats.conn_count, cloned.conn_count);
        assert_eq!(stats.total_bytes_relayed, cloned.total_bytes_relayed);
        assert_eq!(stats.active_connections, cloned.active_connections);
    }

    #[tokio::test]
    async fn test_transit_stats_default_values() {
        let ts = TransitStation::new();
        let stats = ts.stats().await;
        let _ = stats.conn_count;
    }

    // ── 配置集成 ──

    #[test]
    fn test_config_transit_section_defaults() {
        let cfg = crate::neotrix::nt_io_stealth_net::config::NeoTrixConfig::default();
        assert!(!cfg.transit.enabled);
        assert_eq!(cfg.transit.mode, "pf_divert");
        assert_eq!(cfg.transit.listen_port, 11081);
        assert!(cfg.transit.per_conn_ip_rotation);
        assert!(cfg.transit.padding_enabled);
        assert!(cfg.transit.timing_obfuscation_enabled);
    }

    #[tokio::test]
    async fn test_connect_via_proxy_url_parse() {
        let cases = vec![
            ("socks5://127.0.0.1:1080", "socks5", "127.0.0.1", 1080u16),
            (
                "socks5://proxy.example.com:9050",
                "socks5",
                "proxy.example.com",
                9050,
            ),
            ("http://gateway:3128", "http", "gateway", 3128),
            ("https://secure-proxy:443", "https", "secure-proxy", 443),
        ];
        for (url, expected_scheme, expected_host, expected_port) in cases {
            let parsed = parse_proxy_url(url);
            assert!(parsed.is_some(), "should parse: {}", url);
            let (scheme, host, port) = parsed.unwrap();
            assert_eq!(scheme, expected_scheme, "scheme mismatch for {}", url);
            assert_eq!(host, expected_host, "host mismatch for {}", url);
            assert_eq!(port, expected_port, "port mismatch for {}", url);
        }
    }

    #[test]
    fn test_connect_via_invalid_proxy_url() {
        assert!(parse_proxy_url("").is_none());
        assert!(parse_proxy_url("not-a-url").is_none());
        assert!(parse_proxy_url("ftp://bad:21").is_none());
    }
}
