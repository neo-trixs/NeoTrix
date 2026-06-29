use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::timeout;

use super::geo_proxy::domain_resolves_to_china;
use super::proxy_control::DaemonMode;
use super::proxy_pool::global_pool;
use super::rules::{OutboundAction, RuleEngine};
use super::self_iterating::FingerprintManager;
use log;

pub(crate) const LOCAL_PROXY_ADDR: &str = "127.0.0.1:11080";
pub(crate) use crate::core::nt_core_util::TOR_SOCKS_ADDR;

type HttpRequestParts = (String, String, String, Vec<(String, String)>);

fn parse_http_request(data: &[u8]) -> Option<HttpRequestParts> {
    let text = std::str::from_utf8(data).ok()?;
    let mut lines = text.lines();
    let first = lines.next()?;
    let parts: Vec<&str> = first.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return None;
    }
    let method = parts[0].to_string();
    let url_ = parts[1].to_string();

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some(pos) = line.find(':') {
            headers.push((
                line[..pos].trim().to_string(),
                line[pos + 1..].trim().to_string(),
            ));
        }
    }
    let body_start = text.find("\r\n\r\n").map(|p| p + 4).unwrap_or(text.len());
    let body = text[body_start..].to_string();
    Some((method, url_, body, headers))
}

fn extract_host(data: &[u8]) -> Option<(String, u16)> {
    let text = std::str::from_utf8(data).ok()?;
    if text.starts_with("CONNECT ") {
        let rest = text.strip_prefix("CONNECT ")?;
        let target = rest.split(' ').next()?;
        let parts: Vec<&str> = target.rsplitn(2, ':').collect();
        if parts.len() == 2 {
            let host = parts[1].to_string();
            let port: u16 = parts[0].parse().ok()?;
            return Some((host, port));
        }
        return Some((target.to_string(), 443));
    }
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

pub(crate) fn parse_proxy_url(url: &str) -> Option<(&str, &str, u16)> {
    // Returns (scheme, host, port)
    if let Some(rest) = url.strip_prefix("socks5://") {
        if let Some(colon) = rest.rfind(':') {
            let host = &rest[..colon];
            let port: u16 = rest[colon + 1..].parse().ok()?;
            return Some(("socks5", host, port));
        }
        return Some(("socks5", rest, 1080));
    }
    if let Some(rest) = url.strip_prefix("http://") {
        if let Some(colon) = rest.rfind(':') {
            let host = &rest[..colon];
            let port: u16 = rest[colon + 1..].parse().ok()?;
            return Some(("http", host, port));
        }
        return Some(("http", rest, 80));
    }
    if let Some(rest) = url.strip_prefix("https://") {
        if let Some(colon) = rest.rfind(':') {
            let host = &rest[..colon];
            let port: u16 = rest[colon + 1..].parse().ok()?;
            return Some(("https", host, port));
        }
        return Some(("https", rest, 443));
    }
    None
}

async fn connect_via_proxy_pool(host: &str, port: u16) -> Result<TcpStream, String> {
    let pool = global_pool();
    let strategy = pool.current_strategy().await;
    let is_adaptive_or_auto = strategy == super::proxy_pool::NodeSelectionStrategy::Adaptive
        || strategy == super::proxy_pool::NodeSelectionStrategy::Auto;
    let proxy = if is_adaptive_or_auto {
        pool.select_node_for_host(host).await
    } else {
        pool.select_node().await
    }
    .ok_or_else(|| "no proxy available".to_string())?;
    let parsed =
        parse_proxy_url(&proxy.url).ok_or_else(|| format!("bad proxy url: {}", proxy.url))?;

    let result = match parsed.0 {
        "socks5" => connect_via_socks5(&format!("{}:{}", parsed.1, parsed.2), host, port).await,
        "http" | "https" => {
            let proxy_addr = format!("{}:{}", parsed.1, parsed.2);
            let mut stream = TcpStream::connect(&proxy_addr)
                .await
                .map_err(|e| format!("http proxy connect: {}", e))?;
            let connect_req = format!(
                "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
                host, port, host, port
            );
            stream
                .write_all(connect_req.as_bytes())
                .await
                .map_err(|e| format!("http proxy write: {}", e))?;
            let mut buf = [0u8; 4096];
            let n = stream
                .read(&mut buf)
                .await
                .map_err(|e| format!("http proxy read: {}", e))?;
            let resp = std::str::from_utf8(&buf[..n]).unwrap_or("");
            if !resp.starts_with("HTTP/1.1 200") {
                Err(format!(
                    "http proxy rejected: {}",
                    resp.lines().next().unwrap_or("")
                ))
            } else {
                Ok(stream)
            }
        }
        _ => Err(format!("unsupported proxy scheme: {}", parsed.0)),
    };

    // RL 奖励信号
    pool.record_strategy_result(host, result.is_ok()).await;

    result
}

pub(crate) async fn connect_via_socks5(
    proxy_addr: &str,
    host: &str,
    port: u16,
) -> Result<TcpStream, String> {
    let mut stream = tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(proxy_addr))
        .await
        .map_err(|_| "SOCKS5 connect timeout".to_string())?
        .map_err(|e| format!("SOCKS5 connect: {}", e))?;

    // SOCKS5 greet
    let greet = vec![5u8, 1, 0];
    tokio::time::timeout(Duration::from_secs(5), stream.write_all(&greet))
        .await
        .map_err(|_| "socks5 greet write timeout".to_string())?
        .map_err(|e| format!("socks5 greet: {}", e))?;
    let mut buf = [0u8; 2];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut buf))
        .await
        .map_err(|_| "socks5 greet read timeout".to_string())?
        .map_err(|e| format!("socks5 greet resp: {}", e))?;
    if buf[1] != 0 {
        return Err(format!("socks5 auth required: {:?}", buf));
    }

    // SOCKS5 connect
    let host_bytes = host.as_bytes();
    let mut msg = Vec::with_capacity(7 + host_bytes.len());
    msg.extend_from_slice(&[5u8, 1, 0, 3]);
    msg.push(host_bytes.len() as u8);
    msg.extend_from_slice(host_bytes);
    msg.extend_from_slice(&port.to_be_bytes());
    tokio::time::timeout(Duration::from_secs(10), stream.write_all(&msg))
        .await
        .map_err(|_| "socks5 connect write timeout".to_string())?
        .map_err(|e| format!("socks5 connect: {}", e))?;
    let mut resp = [0u8; 4];
    tokio::time::timeout(Duration::from_secs(10), stream.read_exact(&mut resp))
        .await
        .map_err(|_| "socks5 connect read timeout".to_string())?
        .map_err(|e| format!("socks5 resp: {}", e))?;
    if resp[1] != 0 {
        return Err(format!("socks5 rejected: {}", resp[1]));
    }

    let bound_len = match resp[3] {
        1 => 4usize,
        3 => {
            let mut lb = [0u8; 1];
            tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut lb))
                .await
                .map_err(|_| "socks5 bound len timeout".to_string())?
                .map_err(|e| format!("bound len: {}", e))?;
            lb[0] as usize
        }
        4 => 16usize,
        _ => return Err("unknown socks5 addr type".to_string()),
    };
    let mut _addr = vec![0u8; bound_len];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut _addr))
        .await
        .map_err(|_| "socks5 bound addr timeout".to_string())?
        .map_err(|e| format!("bound addr: {}", e))?;
    let mut _pb = [0u8; 2];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut _pb))
        .await
        .map_err(|_| "socks5 bound port timeout".to_string())?
        .map_err(|e| format!("bound port: {}", e))?;

    Ok(stream)
}

/// Perform SOCKS5 handshake (greet + connect) on an existing TCP stream
/// Used for proxy chaining where the TCP connection is already established
async fn socks5_handshake_on_stream(
    stream: &mut TcpStream,
    host: &str,
    port: u16,
) -> Result<(), String> {
    let greet = vec![5u8, 1, 0];
    timeout(Duration::from_secs(10), stream.write_all(&greet))
        .await
        .map_err(|_| "socks5 greet write timeout".to_string())?
        .map_err(|e| format!("socks5 greet: {}", e))?;
    let mut buf = [0u8; 2];
    timeout(Duration::from_secs(10), stream.read_exact(&mut buf))
        .await
        .map_err(|_| "socks5 greet read timeout".to_string())?
        .map_err(|e| format!("socks5 greet resp: {}", e))?;
    if buf[1] != 0 {
        return Err(format!("socks5 auth required: {:?}", buf));
    }
    let host_bytes = host.as_bytes();
    let mut msg = Vec::with_capacity(7 + host_bytes.len());
    msg.extend_from_slice(&[5u8, 1, 0, 3]);
    msg.push(host_bytes.len() as u8);
    msg.extend_from_slice(host_bytes);
    msg.extend_from_slice(&port.to_be_bytes());
    timeout(Duration::from_secs(10), stream.write_all(&msg))
        .await
        .map_err(|_| "socks5 connect write timeout".to_string())?
        .map_err(|e| format!("socks5 connect: {}", e))?;
    let mut resp = [0u8; 4];
    timeout(Duration::from_secs(10), stream.read_exact(&mut resp))
        .await
        .map_err(|_| "socks5 connect read timeout".to_string())?
        .map_err(|e| format!("socks5 resp: {}", e))?;
    if resp[1] != 0 {
        return Err(format!("socks5 rejected: code={}", resp[1]));
    }
    let bound_len = match resp[3] {
        1 => 4usize,
        3 => {
            let mut lb = [0u8; 1];
            timeout(Duration::from_secs(5), stream.read_exact(&mut lb))
                .await
                .map_err(|_| "socks5 bound len timeout".to_string())?
                .map_err(|e| format!("bound len: {}", e))?;
            lb[0] as usize
        }
        4 => 16usize,
        _ => return Err("unknown socks5 addr type".to_string()),
    };
    let mut _addr = vec![0u8; bound_len];
    timeout(Duration::from_secs(5), stream.read_exact(&mut _addr))
        .await
        .map_err(|_| "socks5 bound addr timeout".to_string())?
        .map_err(|e| format!("bound addr: {}", e))?;
    let mut _pb = [0u8; 2];
    timeout(Duration::from_secs(5), stream.read_exact(&mut _pb))
        .await
        .map_err(|_| "socks5 bound port timeout".to_string())?
        .map_err(|e| format!("bound port: {}", e))?;
    Ok(())
}

/// Dual-hop SOCKS5 chain: connect through relay → exit → target
///
/// 1. TCP connect to relay
/// 2. SOCKS5 handshake through relay to reach exit
/// 3. SOCKS5 handshake through exit to reach target
/// 4. Return the final chained stream
pub(crate) async fn connect_via_socks5_chain(
    relay_addr: &str,
    exit_addr: &str,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, String> {
    // Step 1: TCP to relay
    let mut stream = timeout(Duration::from_secs(15), TcpStream::connect(relay_addr))
        .await
        .map_err(|_| format!("connect relay {} timeout", relay_addr))?
        .map_err(|e| format!("connect relay {}: {}", relay_addr, e))?;

    // Step 2: SOCKS5 through relay → reach exit
    let exit_parts: Vec<&str> = exit_addr.rsplitn(2, ':').collect();
    let exit_port: u16 = exit_parts[0].parse().map_err(|_| "bad exit port")?;
    socks5_handshake_on_stream(&mut stream, exit_parts[1], exit_port).await?;

    // Step 3: SOCKS5 through exit → reach target
    socks5_handshake_on_stream(&mut stream, target_host, target_port).await?;

    log::info!(
        "[socks5-chain] {}:{} via {} → {}",
        target_host,
        target_port,
        relay_addr,
        exit_addr
    );
    Ok(stream)
}

/// N-hop SOCKS5 chain: TCP → hop₁ → hop₂ → ... → hopₙ → target
///
/// Each hop performs a SOCKS5 handshake on the same stream to reach the next hop.
/// `hops` must be non-empty; the last hop connects to `target_host:target_port`.
pub(crate) async fn connect_via_multi_hop(
    hops: &[&str],
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, String> {
    if hops.is_empty() {
        // Fallback to direct
        return timeout(
            Duration::from_secs(15),
            TcpStream::connect((target_host, target_port)),
        )
        .await
        .map_err(|_| format!("direct connect {} timeout", target_host))?
        .map_err(|e| format!("direct connect {}:{}: {}", target_host, target_port, e));
    }

    // Step 1: TCP to first hop
    let mut stream = timeout(Duration::from_secs(15), TcpStream::connect(hops[0]))
        .await
        .map_err(|_| format!("connect hop {} timeout", hops[0]))?
        .map_err(|e| format!("connect hop {}: {}", hops[0], e))?;

    // Step 2-N: SOCKS5 through each hop → reach next hop
    for hop in &hops[1..] {
        let parts: Vec<&str> = hop.rsplitn(2, ':').collect();
        let port: u16 = parts[0]
            .parse()
            .map_err(|_| format!("bad hop port: {}", hop))?;
        socks5_handshake_on_stream(&mut stream, parts[1], port).await?;
    }

    // Step 3: SOCKS5 through last hop → reach target
    socks5_handshake_on_stream(&mut stream, target_host, target_port).await?;

    log::info!(
        "[multi-hop] {}:{} via {} hops",
        target_host,
        target_port,
        hops.len()
    );
    Ok(stream)
}

async fn relay(mut client: TcpStream, mut remote: TcpStream) {
    let (mut _cr, mut cw) = client.split();
    let (mut _rr, mut _rw) = remote.split();
    let _ = tokio::join!(
        tokio::io::copy(&mut _cr, &mut _rw),
        tokio::io::copy(&mut _rr, &mut cw),
    );
}

pub struct LocalProxy {
    rule_engine: Option<Arc<RwLock<RuleEngine>>>,
    fingerprint_manager: Option<Arc<std::sync::Mutex<FingerprintManager>>>,
    pub tor_manager: Option<Arc<TorManager>>,
    mode: Option<Arc<RwLock<DaemonMode>>>,
    running: Arc<AtomicBool>,
}

impl Default for LocalProxy {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalProxy {
    pub fn new() -> Self {
        Self {
            rule_engine: None,
            fingerprint_manager: None,
            tor_manager: Some(Arc::new(TorManager::new())),
            mode: None,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn with_mode_controller(mut self, mode: Arc<RwLock<DaemonMode>>) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn with_rule_engine(mut self, engine: Arc<RwLock<RuleEngine>>) -> Self {
        self.rule_engine = Some(engine);
        self
    }

    pub fn with_fingerprint_manager(
        mut self,
        fp: Arc<std::sync::Mutex<FingerprintManager>>,
    ) -> Self {
        self.fingerprint_manager = Some(fp);
        self
    }

    pub async fn start(self: Arc<Self>) -> Result<(), String> {
        let listener = TcpListener::bind(LOCAL_PROXY_ADDR)
            .await
            .map_err(|e| format!("bind: {}", e))?;
        log::info!("[proxy] HTTP CONNECT proxy on {}", LOCAL_PROXY_ADDR);

        while self.running.load(Ordering::Relaxed) {
            let (stream, addr) = listener
                .accept()
                .await
                .map_err(|e| format!("accept: {}", e))?;
            let proxy = self.clone();
            tokio::spawn(async move {
                if let Err(e) = proxy.handle_connection(stream).await {
                    log::warn!("[proxy] {} -> {}", addr, e);
                }
            });
        }
        Ok(())
    }

    pub async fn routing_decision(&self, host: &str) -> (bool, bool) {
        if host.ends_with(".onion") {
            return (false, true);
        }

        // 模式感知路由
        if let Some(mode) = &self.mode {
            match *mode.read().await {
                DaemonMode::Off => return (false, false),
                DaemonMode::Stealth => return (true, false),
                DaemonMode::Tor => return (false, true),
                DaemonMode::Geo => {}
            }
        }

        // Geo 路由（当前默认逻辑）
        let is_china = domain_resolves_to_china(host).await.unwrap_or_default();
        if is_china {
            return (false, false);
        }
        if let Some(engine) = &self.rule_engine {
            let url_s = format!("https://{}/", host);
            if let Ok(parsed) = url::Url::parse(&url_s) {
                let guard = engine.read().await;
                let action = guard.evaluate(&parsed);
                match action {
                    OutboundAction::Direct => return (false, false),
                    OutboundAction::Proxy(_) => return (true, false),
                    OutboundAction::Tor => return (false, true),
                    OutboundAction::Block => return (false, false),
                }
            }
        }
        (true, false)
    }

    async fn handle_connection(&self, mut stream: TcpStream) -> Result<(), String> {
        let mut buf = vec![0u8; 16384];
        let n = tokio::time::timeout(Duration::from_secs(15), stream.read(&mut buf))
            .await
            .map_err(|_| "read timeout".to_string())?
            .map_err(|e| format!("read: {}", e))?;
        if n == 0 {
            return Ok(());
        }
        let data = &buf[..n];

        if data.starts_with(b"CONNECT ") {
            self.handle_connect(stream, data).await
        } else if let Some(method) = parse_http_request(data).map(|r| r.0) {
            let url_s = parse_http_request(data).map(|r| r.1).unwrap_or_default();
            self.handle_http_get(stream, data, &method, &url_s).await
        } else {
            Err("unknown request type".to_string())
        }
    }

    async fn handle_connect(&self, mut stream: TcpStream, data: &[u8]) -> Result<(), String> {
        let (host, port) = extract_host(data).ok_or_else(|| "parse CONNECT".to_string())?;

        let start = Instant::now();
        let remote = self.connect_with_fallback(&host, port).await.map_err(|e| {
            log::error!("[proxy] CONNECT {}:{} failed: {}", host, port, e);
            format!("connect {}:{}: {}", host, port, e)
        })?;
        let elapsed = start.elapsed();
        log::info!("[proxy] CONNECT {}:{} OK {:?}", host, port, elapsed);

        stream
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await
            .map_err(|e| format!("write 200: {}", e))?;
        relay(stream, remote).await;
        Ok(())
    }

    async fn handle_http_get(
        &self,
        mut stream: TcpStream,
        data: &[u8],
        _method: &str,
        url_s: &str,
    ) -> Result<(), String> {
        let parsed = url::Url::parse(url_s).map_err(|e| format!("parse url {}: {}", url_s, e))?;
        let host = parsed.host_str().unwrap_or("").to_string();
        let port = parsed.port().unwrap_or(80);

        let start = Instant::now();
        let mut remote = self.connect_with_fallback(&host, port).await.map_err(|e| {
            log::error!("[proxy] HTTP {}:{} failed: {}", host, port, e);
            format!("connect {}:{}: {}", host, port, e)
        })?;
        let elapsed = start.elapsed();
        log::info!("[proxy] HTTP {}:{} OK {:?}", host, port, elapsed);

        remote
            .write_all(data)
            .await
            .map_err(|e| format!("relay write: {}", e))?;
        let (mut _rr, mut _rw) = remote.split();
        let (mut _cr, mut cw) = stream.split();
        let _ = tokio::io::copy(&mut _rr, &mut cw).await;
        Ok(())
    }

    async fn connect_with_fallback(&self, host: &str, port: u16) -> Result<TcpStream, String> {
        let (use_proxy, use_tor) = self.routing_decision(host).await;
        let pool = global_pool();
        let pool_available = pool.available_count().await;
        let pool_total = pool.total_count().await;
        let pool_ready = pool_available > 0 || pool_total > 0;

        if use_proxy && pool_ready {
            let before = pool.available_count().await;
            if before > 0 {
                match connect_via_proxy_pool(host, port).await {
                    Ok(s) => return Ok(s),
                    Err(e) => log::error!("[proxy] pool failed {}:{}: {}", host, port, e),
                }
            } else {
                log::warn!(
                    "[proxy] pool {} nodes (pending health-check), skip",
                    pool_total
                );
            }
        }

        if use_tor || (port == 443) || (!use_proxy && !pool_ready) {
            if Self::tor_reachable().await {
                match connect_via_socks5(TOR_SOCKS_ADDR, host, port).await {
                    Ok(s) => return Ok(s),
                    Err(e) => log::error!("[proxy] Tor failed {}:{}: {}", host, port, e),
                }
            } else {
                log::warn!("[proxy] Tor not reachable on {}", TOR_SOCKS_ADDR);
            }
        }

        if use_proxy && pool_available == 0 && pool_total > 0 {
            log::info!("[proxy] pool retry for {}:{}", host, port);
            tokio::time::sleep(Duration::from_secs(5)).await;
            let after = pool.available_count().await;
            if after > 0 {
                if let Ok(s) = connect_via_proxy_pool(host, port).await {
                    return Ok(s);
                }
            }
            if Self::tor_reachable().await {
                if let Ok(s) = connect_via_socks5(TOR_SOCKS_ADDR, host, port).await {
                    return Ok(s);
                }
            }
        }

        tokio::time::timeout(
            Duration::from_secs(10),
            TcpStream::connect(format!("{}:{}", host, port)),
        )
        .await
        .map_err(|_| format!("direct timeout {}:{}", host, port))?
        .map_err(|e| format!("direct {}:{}: {}", host, port, e))
    }

    async fn tor_reachable() -> bool {
        tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(TOR_SOCKS_ADDR))
            .await
            .is_ok()
    }
}

pub struct TorManager {
    state: Arc<RwLock<TorState>>,
    child: Arc<std::sync::Mutex<Option<std::process::Child>>>,
    shutdown: Arc<AtomicBool>,
}

impl Drop for TorManager {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TorState {
    Stopped,
    Starting,
    Running,
    Error(String),
}

impl Default for TorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TorManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(TorState::Stopped)),
            child: Arc::new(std::sync::Mutex::new(None)),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn auto_install_and_start(&self) {
        let mut state = self.state.write().await;
        *state = TorState::Starting;
        drop(state);

        match std::process::Command::new("tor")
            .arg("--SOCKSPort")
            .arg(crate::core::nt_core_util::TOR_SOCKS_PORT.to_string())
            .arg("--ControlPort")
            .arg(crate::core::nt_core_util::TOR_CONTROL_PORT.to_string())
            .arg("--quiet")
            .spawn()
        {
            Ok(mut child) => {
                tokio::time::sleep(Duration::from_millis(1500)).await;
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let mut s = self.state.write().await;
                        *s = TorState::Error(format!("tor exited: {}", status));
                        log::error!("[tor] failed to start: {}", status);
                    }
                    Ok(None) => {
                        let mut s = self.state.write().await;
                        *s = TorState::Running;
                        *self.child.lock().unwrap_or_else(|e| e.into_inner()) = Some(child);
                        log::info!("[tor] running on {}", TOR_SOCKS_ADDR);
                    }
                    Err(e) => {
                        let mut s = self.state.write().await;
                        *s = TorState::Error(e.to_string());
                        log::error!("[tor] spawn error: {}", e);
                    }
                }
            }
            Err(e) => {
                let mut s = self.state.write().await;
                *s = TorState::Error(e.to_string());
                log::error!("[tor] not installed: {}", e);
            }
        }
    }

    pub async fn stop(&self) {
        let mut guard = self.child.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        *self.state.write().await = TorState::Stopped;
    }

    pub async fn start_health_monitor(&self) {
        while !self.shutdown.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_secs(15)).await;
            let reachable = Self::socks5_reachable().await;
            let mut state = self.state.write().await;
            if reachable {
                if *state != TorState::Running {
                    *state = TorState::Running;
                }
            } else {
                *state = TorState::Error("unreachable".to_string());
            }
        }
    }

    pub async fn is_running(&self) -> bool {
        let s = self.state.read().await;
        if *s == TorState::Running {
            return true;
        }
        drop(s);
        Self::socks5_reachable().await
    }

    pub async fn socks5_reachable() -> bool {
        tokio::time::timeout(Duration::from_secs(3), async {
            TcpStream::connect(TOR_SOCKS_ADDR).await.is_ok()
        })
        .await
        .unwrap_or(false)
    }
}

pub fn tor_connect(_target: &str, _port: u16) -> Result<String, String> {
    Ok(format!("socks5://{}", TOR_SOCKS_ADDR))
}
