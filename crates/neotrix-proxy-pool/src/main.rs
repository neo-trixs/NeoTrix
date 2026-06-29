use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use futures::FutureExt;
use tokio::sync::Notify;

#[derive(Clone, Debug)]
struct ShutdownSignal {
    triggered: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl ShutdownSignal {
    fn new() -> Self { Self { triggered: Arc::new(AtomicBool::new(false)), notify: Arc::new(Notify::new()) } }
    fn trigger(&self, _reason: &str) { self.triggered.store(true, Ordering::SeqCst); self.notify.notify_one(); }
    fn is_shutdown(&self) -> bool { self.triggered.load(Ordering::SeqCst) }
    async fn wait_shutdown(&self) {
        if self.triggered.load(Ordering::SeqCst) { return; }
        self.notify.notified().await;
    }
}

use chrono::Local;
use serde::{Deserialize, Serialize};
use base64::Engine;
use url::Url;

lazy_static::lazy_static! {
    static ref BRIDGE_HANDLES: Mutex<HashMap<u16, tokio::task::JoinHandle<()>>> = Mutex::new(HashMap::new());
}

fn control_socket() -> String {
    format!("{}/.neotrix/neotrix-proxy.sock", home_dir().display())
}
const UPSTREAMS_CONF: &str = ".neotrix/proxy-upstreams.conf";
const POOL_STATE_FILE: &str = ".neotrix/proxy-pool-state.json";
const CONFIG_FILE: &str = ".neotrix/proxy-pool.json";
const PROTOCOL_TEST_TARGET: &str = "opencode.ai:443";

fn home_dir() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"))
}

fn config_path() -> PathBuf {
    home_dir().join(CONFIG_FILE)
}

fn upstreams_conf_path() -> PathBuf {
    home_dir().join(UPSTREAMS_CONF)
}

fn pool_state_path() -> PathBuf {
    home_dir().join(POOL_STATE_FILE)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RandomIntervalConfig {
    mean_secs: f64,
    std_secs: f64,
    min_secs: f64,
    max_secs: f64,
}

impl RandomIntervalConfig {
    fn duration(&self) -> Duration {
        let u1 = fast_random().clamp(1e-15, 1.0 - 1e-15);
        let u2 = fast_random().clamp(1e-15, 1.0 - 1e-15);
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let secs = (self.mean_secs + z * self.std_secs).clamp(self.min_secs, self.max_secs);
        Duration::from_secs_f64(secs)
    }
}

impl Default for RandomIntervalConfig {
    fn default() -> Self {
        Self { mean_secs: 4.5, std_secs: 2.0, min_secs: 0.0, max_secs: 9.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PoolConfig {
    subscription_urls: Vec<String>,
    check_interval: RandomIntervalConfig,
    tcp_timeout: RandomIntervalConfig,
    protocol_timeout: RandomIntervalConfig,
    discovery_interval: RandomIntervalConfig,
    max_upstreams: usize,
    min_health_score: f64,
    consecutive_fail_limit: u32,
    test_target: String,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            subscription_urls: vec!["https://234.qzz.io/fsllist64".into()],
            check_interval: RandomIntervalConfig { mean_secs: 4.5, std_secs: 2.0, min_secs: 0.0, max_secs: 9.0 },
            tcp_timeout: RandomIntervalConfig { mean_secs: 4.0, std_secs: 1.5, min_secs: 1.0, max_secs: 8.0 },
            protocol_timeout: RandomIntervalConfig { mean_secs: 12.0, std_secs: 4.0, min_secs: 5.0, max_secs: 30.0 },
            discovery_interval: RandomIntervalConfig { mean_secs: 30.0, std_secs: 10.0, min_secs: 10.0, max_secs: 60.0 },
            max_upstreams: 15,
            min_health_score: 0.35,
            consecutive_fail_limit: 3,
            test_target: PROTOCOL_TEST_TARGET.into(),
        }
    }
}

fn load_config() -> PoolConfig {
    let path = config_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProxyNode {
    host: String,
    port: u16,
    tag: String,
    scheme: String,
    is_direct_upstream: bool,
    is_socks5: bool,
    is_http: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    original_uri: Option<String>,
}

impl ProxyNode {
    fn upstream_uri(&self) -> String {
        if self.is_socks5 {
            format!("socks5://{}:{}", self.host, self.port)
        } else {
            format!("http://{}:{}", self.host, self.port)
        }
    }

    fn display(&self) -> String {
        if self.tag.is_empty() {
            format!("{}:{} ({})", self.host, self.port, self.scheme)
        } else {
            format!("{} ({}:{} {})", self.tag, self.host, self.port, self.scheme)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PoolEntry {
    node: ProxyNode,
    tcp_ping_ms: f64,
    protocol_ok: bool,
    health_score: f64,
    consecutive_fails: u32,
    last_checked: String,
    success_count: u64,
    fail_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
struct PoolStats {
    total_checks: u64,
    fetch_errors: u64,
    last_check_time: String,
    direct_usable_count: u32,
    encrypted_count: u32,
    upstreams_fed: u32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
struct PoolState {
    entries: Vec<PoolEntry>,
    stats: PoolStats,
}


fn now_str() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn url_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8()
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| s.to_string())
}

fn fetch_subscription_blocking(url: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client build failed: {}", e))?;
    let resp = client.get(url).send()
        .map_err(|e| format!("HTTP GET failed: {}", e))?;
    let bytes = resp.bytes()
        .map_err(|e| format!("read body failed: {}", e))?;
    let raw = bytes.to_vec();

    let engine = base64::engine::general_purpose::STANDARD;

    if let Ok(decoded) = engine.decode(&raw) {
        let text = String::from_utf8_lossy(&decoded).to_string();
        if text.contains("://") {
            return Ok(text);
        }
    }
    let padded = format!("{}=", String::from_utf8_lossy(&raw));
    if let Ok(decoded) = engine.decode(padded.as_bytes()) {
        let text = String::from_utf8_lossy(&decoded).to_string();
        if text.contains("://") {
            return Ok(text);
        }
    }
    Ok(String::from_utf8_lossy(&raw).to_string())
}

async fn fetch_subscription(url: &str) -> Result<String, String> {
    let url = url.to_string();
    tokio::task::spawn_blocking(move || {
        fetch_subscription_blocking(&url)
    }).await.map_err(|e| format!("spawn blocking: {}", e))?
}

fn parse_proxy_nodes(text: &str) -> Vec<ProxyNode> {
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .filter_map(parse_uri)
        .collect()
}

fn parse_uri(input: &str) -> Option<ProxyNode> {
    let original = Some(input.to_string());
    if input.starts_with("ss://") {
        let mut n = parse_ss(input)?;
        n.original_uri = original;
        return Some(n);
    }
    if input.starts_with("vless://") {
        let mut n = parse_vless(input)?;
        n.original_uri = original;
        return Some(n);
    }
    if input.starts_with("vmess://") {
        let mut n = parse_vmess(input)?;
        n.original_uri = original;
        return Some(n);
    }
    if input.starts_with("hysteria2://") || input.starts_with("hy2://") {
        let mut n = parse_hysteria2(input)?;
        n.original_uri = original;
        return Some(n);
    }
    if input.starts_with("socks5://") {
        let mut n = parse_socks5(input)?;
        n.original_uri = original;
        return Some(n);
    }
    if input.starts_with("https://") || input.starts_with("http://") {
        let mut n = parse_http_proxy(input)?;
        n.original_uri = original;
        return Some(n);
    }
    // Fallback: try as URL
    let u = Url::parse(input).ok()?;
    let host = u.host_str()?.to_string();
    let port = u.port().unwrap_or(443);
    let tag = u.fragment().unwrap_or("");
    Some(ProxyNode {
        host,
        port,
        tag: url_decode(tag),
        scheme: u.scheme().into(),
        is_direct_upstream: false,
        is_socks5: false,
        is_http: false,
        original_uri: None,
    })
}

fn parse_ss(input: &str) -> Option<ProxyNode> {
    // ss://base64(method:password)@host:port#tag
    // SIP008 format: ss://base64(config) without @ — falls through to kernel
    let rest = input.strip_prefix("ss://")?;
    let (_, after_at) = match rest.split_once('@') {
        Some(pair) => pair,
        None => {
            log::error!("[pool] WARN: SIP008-style ss:// URI without '@' — try upgrading to kernel-based parsing: {}", &input[..30.min(input.len())]);
            return None;
        }
    };
    let (host, after_host) = after_at.split_once(':')?;
    let port_str = after_host.split('?').next()
        .or_else(|| after_host.split('#').next())?;
    let port: u16 = port_str.parse().unwrap_or(443);
    let tag = after_host.split('#').nth(1).unwrap_or("");
    Some(ProxyNode {
        host: host.to_string(),
        port,
        tag: url_decode(tag),
        scheme: "ss".into(),
        is_direct_upstream: false,
        is_socks5: false,
        is_http: false,
        original_uri: None,
    })
}

fn parse_vless(input: &str) -> Option<ProxyNode> {
    // vless://uuid@host:port?params#tag
    let rest = input.strip_prefix("vless://")?;
    let (_, after_at) = rest.split_once('@')?;
    let (host, after_host) = after_at.split_once(':')?;
    let port_str = after_host.split('?').next()
        .or_else(|| after_host.split('#').next())?;
    let port: u16 = port_str.parse().unwrap_or(443);
    let tag = after_host.split('#').nth(1).unwrap_or("");
    Some(ProxyNode {
        host: host.to_string(),
        port,
        tag: url_decode(tag),
        scheme: "vless".into(),
        is_direct_upstream: false,
        is_socks5: false,
        is_http: false,
        original_uri: None,
    })
}

fn parse_vmess(input: &str) -> Option<ProxyNode> {
    // vmess://base64({json})
    let b64 = input.strip_prefix("vmess://")?;
    let engine = base64::engine::general_purpose::STANDARD;
    let json_bytes = engine.decode(b64).ok()?;
    let json_str = String::from_utf8_lossy(&json_bytes);
    let v: HashMap<String, serde_json::Value> = serde_json::from_str(&json_str).ok()?;
    let host = v.get("add")?.as_str()?.to_string();
    let port: u16 = v.get("port")
        .and_then(|p| p.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(443);
    let tag = v.get("ps").and_then(|s| s.as_str()).unwrap_or("");
    Some(ProxyNode {
        host,
        port,
        tag: tag.to_string(),
        scheme: "vmess".into(),
        is_direct_upstream: false,
        is_socks5: false,
        is_http: false,
        original_uri: None,
    })
}

fn parse_hysteria2(input: &str) -> Option<ProxyNode> {
    // hysteria2://password@host:port?params#tag
    let rest = if input.starts_with("hysteria2://") {
        input.strip_prefix("hysteria2://")?
    } else {
        input.strip_prefix("hy2://")?
    };
    let after_at = if let Some((_, a)) = rest.split_once('@') {
        a
    } else {
        rest
    };
    let (host, after_host) = after_at.split_once(':')?;
    let port_str = after_host.split('?').next()
        .or_else(|| after_host.split('#').next())?;
    let port: u16 = port_str.parse().unwrap_or(443);
    let tag = after_host.split('#').nth(1).unwrap_or("");
    Some(ProxyNode {
        host: host.to_string(),
        port,
        tag: url_decode(tag),
        scheme: "hysteria2".into(),
        is_direct_upstream: false,
        is_socks5: false,
        is_http: false,
        original_uri: None,
    })
}

fn parse_socks5(input: &str) -> Option<ProxyNode> {
    let u = Url::parse(input).ok()?;
    let host = u.host_str()?.to_string();
    let port = u.port().unwrap_or(1080);
    let tag = u.fragment().unwrap_or("");
    Some(ProxyNode {
        host,
        port,
        tag: url_decode(tag),
        scheme: "socks5".into(),
        is_direct_upstream: true,
        is_socks5: true,
        is_http: false,
        original_uri: None,
    })
}

fn parse_http_proxy(input: &str) -> Option<ProxyNode> {
    let u = Url::parse(input).ok()?;
    let host = u.host_str()?.to_string();
    let port = u.port().unwrap_or(80);
    let tag = u.fragment().unwrap_or("");
    Some(ProxyNode {
        host,
        port,
        tag: url_decode(tag),
        scheme: "http".into(),
        is_direct_upstream: true,
        is_socks5: false,
        is_http: true,
        original_uri: None,
    })
}

fn load_pool_state() -> PoolState {
    let path = pool_state_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_pool_state(state: &PoolState) {
    let path = pool_state_path();
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(&path, json);
    }
}

async fn tcp_ping(host: &str, port: u16, timeout_secs: u64) -> Result<f64, String> {
    let addr = format!("{}:{}", host, port);
    let start = Instant::now();
    match tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        tokio::net::TcpStream::connect(&addr),
    ).await {
        Ok(Ok(_)) => Ok(start.elapsed().as_secs_f64() * 1000.0),
        Ok(Err(e)) => Err(format!("TCP connect failed: {}", e)),
        Err(_) => Err("TCP timeout".into()),
    }
}

async fn test_socks5(host: &str, port: u16, target: &str, timeout_secs: u64) -> bool {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    let addr = format!("{}:{}", host, port);
    let Ok(Ok(mut sock)) = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        tokio::net::TcpStream::connect(&addr),
    ).await else { return false; };

    // Greeting: SOCKS5, 1 method, no auth
    if sock.write_all(&[0x05, 0x01, 0x00]).await.is_err() { return false; }
    let mut buf = [0u8; 2];
    if sock.read_exact(&mut buf).await.is_err() { return false; }
    if buf != [0x05, 0x00] { return false; }

    // CONNECT to target
    let (t_host, t_port_str) = target.split_once(':').unwrap_or((target, "443"));
    let t_port: u16 = t_port_str.parse().unwrap_or(443);
    let hb = t_host.as_bytes();
    let mut req = vec![0x05, 0x01, 0x00, 0x03, hb.len() as u8];
    req.extend(hb);
    req.extend(&t_port.to_be_bytes());
    if sock.write_all(&req).await.is_err() { return false; }

    let mut header = [0u8; 4];
    if sock.read_exact(&mut header).await.is_err() { return false; }
    header[0] == 0x05 && header[1] == 0x00
}

async fn test_http_connect(host: &str, port: u16, target: &str, timeout_secs: u64) -> bool {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    let addr = format!("{}:{}", host, port);
    let Ok(Ok(mut sock)) = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        tokio::net::TcpStream::connect(&addr),
    ).await else { return false; };

    let req = format!("CONNECT {} HTTP/1.1\r\nHost: {}\r\n\r\n", target, target);
    if sock.write_all(req.as_bytes()).await.is_err() { return false; }

    let mut buf = [0u8; 4096];
    let n = sock.read(&mut buf).await.unwrap_or(0);
    if n == 0 { return false; }

    let resp = String::from_utf8_lossy(&buf[..n]);
    resp.contains("200") || resp.contains("Connection Established")
}

fn compute_health_score(tcp_ping_ms: f64, protocol_ok: bool, prev_score: f64) -> f64 {
    let mut score = 0.0;
    if tcp_ping_ms > 0.0 {
        let latency_score = (1.0 - (tcp_ping_ms / 5000.0).clamp(0.0, 1.0)) * 0.6;
        score += latency_score;
    }
    score += if protocol_ok { 0.4 } else { 0.1 };
    if prev_score > 0.0 {
        score = 0.7 * score + 0.3 * prev_score;
    }
    score.clamp(0.0, 1.0)
}

fn feed_upstreams(entries: &[PoolEntry]) -> Result<(), String> {
    let path = upstreams_conf_path();
    let mut content = String::new();
    content.push_str("# Auto-generated by neotrix-proxy-pool\n");
    content.push_str(&format!("# Generated at: {}\n", now_str()));
    content.push_str(&format!("# Healthy upstreams: {}\n\n", entries.len()));

    for entry in entries {
        let uri = entry.node.upstream_uri();
        let tag = if entry.node.tag.is_empty() {
            format!("{}:{}", entry.node.host, entry.node.port)
        } else {
            entry.node.tag.clone()
        };
        content.push_str(&format!(
            "# {} score={:.2} ping={:.0}ms\n{}\n",
            tag, entry.health_score, entry.tcp_ping_ms, uri
        ));
    }

    std::fs::write(&path, &content)
        .map_err(|e| format!("write upstreams.conf: {}", e))?;

    // Signal neotrix-proxy to reload
    let mut sock = UnixStream::connect(control_socket())
        .map_err(|e| format!("connect control socket: {}", e))?;
    sock.write_all(b"upstream=reload\n")
        .map_err(|e| format!("send reload: {}", e))?;
    let mut reader = BufReader::new(&mut sock);
    let mut resp = String::new();
    let _ = reader.read_line(&mut resp);
    log::error!("[pool] proxy reload: {}", resp.trim());

    Ok(())
}

async fn check_node(node: &ProxyNode, config: &PoolConfig, prev: Option<&PoolEntry>) -> PoolEntry {
    let mut tcp_ping_ms = 9999.0;
    let mut tcp_ok = false;
    let mut protocol_ok = false;

    if node.is_direct_upstream {
        // Direct upstreams: TCP ping + protocol handshake
        let tcp_timeout = config.tcp_timeout.duration().as_secs();
        let tcp_result = tcp_ping(&node.host, node.port, tcp_timeout).await;
        let (ms, ok) = match tcp_result {
            Ok(ms) => (ms, true),
            Err(_) => (9999.0, false),
        };
        tcp_ping_ms = ms;
        tcp_ok = ok;
        if tcp_ok {
            let proto_timeout = config.protocol_timeout.duration().as_secs();
            if node.is_socks5 {
                protocol_ok = test_socks5(&node.host, node.port, &config.test_target, proto_timeout).await;
            } else if node.is_http {
                protocol_ok = test_http_connect(&node.host, node.port, &config.test_target, proto_timeout).await;
            }
        }
    } else if let Some(ref uri) = node.original_uri {
        // Encrypted nodes: kernel protocol handshake only (covers TCP + protocol in one trip)
        let start = Instant::now();
        let test_target = &config.test_target;
        let (target_host, port_str) = test_target.split_once(':').unwrap_or((test_target, "443"));
        let target_port: u16 = port_str.parse().unwrap_or(443);

        if let Some(kernel_node) = neotrix_proxy_kernel::node::ProxyNode::parse_with_vpn_link(uri) {
            let kernel_timeout = config.protocol_timeout.duration().as_secs().max(15);
            match tokio::time::timeout(
                Duration::from_secs(kernel_timeout),
                neotrix_proxy_kernel::connect_through(&kernel_node, target_host, target_port),
            ).await {
                Ok(Ok((_, _))) => {
                    tcp_ok = true;
                    protocol_ok = true;
                    tcp_ping_ms = start.elapsed().as_secs_f64() * 1000.0;
                    log::error!("[pool] kernel handshake OK for {} — {:.0}ms", node.display(), tcp_ping_ms);
                }
                Ok(Err(e)) => {
                    log::error!("[pool] kernel handshake failed for {}: {:?}", node.display(), e);
                }
                Err(_) => {
                    log::error!("[pool] kernel handshake timeout for {}", node.display());
                }
            }
        }
    }

    let prev_score = prev.map(|e| e.health_score).unwrap_or(0.0);
    let health_score = if tcp_ok {
        compute_health_score(tcp_ping_ms, protocol_ok, prev_score)
    } else {
        let decayed = prev_score * 0.5;
        if decayed < 0.05 { 0.0 } else { decayed }
    };

    let consecutive_fails = prev.map(|e| {
        if tcp_ok && protocol_ok { 0 } else { e.consecutive_fails + 1 }
    }).unwrap_or(if tcp_ok && protocol_ok { 0 } else { 1 });

    let (success_count, fail_count) = match (prev, tcp_ok && protocol_ok) {
        (Some(e), true) => (e.success_count + 1, e.fail_count),
        (Some(e), false) => (e.success_count, e.fail_count + 1),
        (None, true) => (1, 0),
        (None, false) => (0, 1),
    };

    PoolEntry {
        node: node.clone(),
        tcp_ping_ms,
        protocol_ok,
        health_score,
        consecutive_fails,
        last_checked: now_str(),
        success_count,
        fail_count,
    }
}

async fn run_check_cycle(config: &PoolConfig, shutdown: &ShutdownSignal) {
    let mut pool = load_pool_state();
    log::error!("[pool] fetching subscription...");

    // Fetch from all configured subscription URLs
    let mut all_nodes: Vec<ProxyNode> = Vec::new();
    for url in &config.subscription_urls {
        log::error!("[pool] fetching: {}", url);
        match fetch_subscription(url).await {
            Ok(text) => {
                let nodes = parse_proxy_nodes(&text);
                all_nodes.extend(nodes);
            }
            Err(e) => {
                log::error!("[pool] fetch {} failed: {}", url, e);
                pool.stats.fetch_errors += 1;
            }
        }
    }

    // Deduplicate across multiple sources
    let mut seen: HashSet<(String, u16)> = HashSet::new();
    all_nodes.retain(|n| seen.insert((n.host.clone(), n.port)));

    let direct_count = all_nodes.iter().filter(|n| n.is_direct_upstream).count();
    let encrypted_count = all_nodes.len() - direct_count;
    log::error!("[pool] parsed {} nodes: {} direct (SOCKS5/HTTP), {} encrypted",
        all_nodes.len(), direct_count, encrypted_count);

    let prev_map: HashMap<(String, u16), PoolEntry> = pool.entries.drain(..)
        .map(|e| ((e.node.host.clone(), e.node.port), e))
        .collect();

    let mut handles = Vec::new();
    for node in all_nodes {
        let cfg = config.clone();
        let prev = prev_map.get(&(node.host.clone(), node.port)).cloned();
        handles.push(tokio::spawn(async move {
            check_node(&node, &cfg, prev.as_ref()).await
        }));
    }

    let mut checked = Vec::new();
    for h in handles {
        if let Ok(entry) = h.await {
            checked.push(entry);
        }
    }

    // Keep stale entries from prev pool (not in current subscription) with decay
    let checked_keys: HashSet<(String, u16)> = checked.iter()
        .map(|e| (e.node.host.clone(), e.node.port))
        .collect();
    for (key, entry) in &prev_map {
        if !checked_keys.contains(key) {
            let mut stale = entry.clone();
            stale.health_score *= 0.9;
            stale.consecutive_fails += 1;
            if stale.health_score > 0.1 {
                checked.push(stale);
            }
        }
    }

    // Score filter
    let mut healthy: Vec<PoolEntry> = checked.into_iter()
        .filter(|e| e.health_score >= config.min_health_score || e.consecutive_fails < config.consecutive_fail_limit)
        .collect();
    healthy.sort_by(|a, b| b.health_score.partial_cmp(&a.health_score).unwrap_or(std::cmp::Ordering::Equal));

    let hc = healthy.iter().filter(|e| e.health_score >= config.min_health_score).count();
    let uc = healthy.len() - hc;
    log::error!("[pool] check complete: {} healthy, {} low-score (of {})", hc, uc, healthy.len());

    // Deduplicate by host:port
    let mut seen: HashSet<(String, u16)> = HashSet::new();
    let deduped: Vec<PoolEntry> = healthy.iter()
        .filter(|e| seen.insert((e.node.host.clone(), e.node.port)))
        .cloned()
        .collect();

    // Feed directly usable upstreams
    let mut feed: Vec<PoolEntry> = deduped.iter()
        .filter(|e| e.node.is_direct_upstream && e.protocol_ok && e.health_score >= config.min_health_score)
        .take(config.max_upstreams)
        .cloned()
        .collect();

    // Spawn local SOCKS5 bridges for healthy encrypted nodes
    let healthy_encrypted: Vec<&PoolEntry> = deduped.iter()
        .filter(|e| !e.node.is_direct_upstream && e.protocol_ok && e.health_score >= config.min_health_score)
        .collect();
    if !healthy_encrypted.is_empty() {
        log::error!("[pool] spawning {} encrypted node bridges", healthy_encrypted.len());
    }
    for (i, entry) in healthy_encrypted.iter().enumerate() {
        if let Some(ref uri) = entry.node.original_uri {
            if let Some(knode) = neotrix_proxy_kernel::node::ProxyNode::parse_with_vpn_link(uri) {
                let listen_port = ENCRYPTED_BRIDGE_BASE + i as u16;
                let node = entry.node.clone();
                let kn = knode;
                // Cancel previous bridge on same port if any
                if let Some(old) = BRIDGE_HANDLES.lock().unwrap_or_else(|e| e.into_inner()).remove(&listen_port) {
                    old.abort();
                }
                let sd = shutdown.clone();
                let handle = tokio::spawn(async move {
                    encrypted_socks5_bridge(node, kn, listen_port, sd).await;
                });
                BRIDGE_HANDLES.lock().unwrap_or_else(|e| e.into_inner()).insert(listen_port, handle);
                feed.push(PoolEntry {
                    node: ProxyNode {
                        host: "127.0.0.1".into(),
                        port: listen_port,
                        tag: format!("bridge-{}", entry.node.tag),
                        scheme: "socks5".into(),
                        is_direct_upstream: true,
                        is_socks5: true,
                        is_http: false,
                        original_uri: None,
                    },
                    tcp_ping_ms: entry.tcp_ping_ms,
                    protocol_ok: true,
                    health_score: entry.health_score,
                    consecutive_fails: 0,
                    last_checked: now_str(),
                    success_count: entry.success_count,
                    fail_count: entry.fail_count,
                });
            }
        }
    }

    let bridge_count = feed.iter().filter(|e| e.node.host == "127.0.0.1").count();
    let direct_count_in_feed = feed.len() - bridge_count;
    if !feed.is_empty() {
        log::error!("[pool] feeding {} upstreams ({} direct + {} bridge) to nt-proxy-daemon",
            feed.len(), direct_count_in_feed, bridge_count);
        if let Err(e) = feed_upstreams(&feed) {
            log::error!("[pool] feed failed: {}", e);
        }
    }

    pool.entries = healthy;
    pool.stats.total_checks += 1;
    pool.stats.last_check_time = now_str();
    pool.stats.direct_usable_count = direct_count_in_feed as u32;
    pool.stats.encrypted_count = encrypted_count as u32;
    pool.stats.upstreams_fed = feed.len() as u32;
    save_pool_state(&pool);

    log::error!("[pool] cycle done. next check via normal({:.1}, {:.1})s",
        config.check_interval.mean_secs, config.check_interval.std_secs);
}

async fn show_status() {
    let pool = load_pool_state();
    log::info!("=== NeoTrix Proxy Pool ===");
    log::info!("  Last check:  {}", pool.stats.last_check_time);
    log::info!("  Total runs:  {}", pool.stats.total_checks);
    log::info!("  Fetch errs:  {}", pool.stats.fetch_errors);
    log::info!("  Direct/HTTP: {}", pool.stats.direct_usable_count);
    log::info!("  Encrypted:   {}", pool.stats.encrypted_count);
    log::info!("  Upstreams:   {}", pool.stats.upstreams_fed);
    log::info!("  Timing:      all intervals use normal distribution (RandomIntervalConfig)");

    let up_path = upstreams_conf_path();
    match std::fs::read_to_string(&up_path) {
        Ok(c) => {
            let n = c.lines().filter(|l| l.contains("://")).count();
            log::info!("  Config file: {} ({} entries)", up_path.display(), n);
        }
        Err(_) => log::info!("  Config file: {} (not found)", up_path.display()),
    }

    if pool.entries.is_empty() {
        log::info!("  Pool: empty");
        return;
    }

    log::info!("\n  Top 10 (by health score):");
    let mut sorted = pool.entries.clone();
    sorted.sort_by(|a, b| b.health_score.partial_cmp(&a.health_score).unwrap_or(std::cmp::Ordering::Equal));
    for (i, e) in sorted.iter().take(10).enumerate() {
        let tag = e.node.display();
        log::info!("  {}. {} [{:.0}ms] score={:.2} proto={} s={} f={}",
            i + 1, tag, e.tcp_ping_ms, e.health_score,
            if e.protocol_ok { "Y" } else { "N" },
            e.success_count, e.fail_count);
    }
    log::info!("\nConfig: {}", config_path().display());
}

fn send_heartbeat() -> bool {
    match UnixStream::connect(control_socket()) {
        Ok(mut sock) => {
            if let Err(e) = sock.write_all(b"status\n") {
                log::error!("[pool] heartbeat write FAIL: {}", e);
                return false;
            }
            let mut resp = String::new();
            let mut reader = BufReader::new(&sock);
            if let Err(e) = reader.read_line(&mut resp) {
                log::error!("[pool] heartbeat read FAIL: {}", e);
                return false;
            }
            log::error!("[pool] heartbeat OK: {}", resp.trim());
            true
        }
        Err(e) => {
            log::error!("[pool] heartbeat FAIL: {}", e);
            false
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("error")).init();
    rustls::crypto::ring::default_provider().install_default().expect("rustls ring provider");
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--status" | "-s" => {
                show_status().await;
                return;
            }
            "--oneshot" | "-1" => {
                let config = load_config();
                let dummy = ShutdownSignal::new();
                run_check_cycle(&config, &dummy).await;
                return;
            }
            "--help" | "-h" => {
                log::info!("neotrix-proxy-pool - Subscription-based proxy pool manager");
                log::info!("");
                log::info!("Usage:");
                log::info!("  neotrix-proxy-pool           Run daemon");
                log::info!("  neotrix-proxy-pool --oneshot  Single check cycle");
                log::info!("  neotrix-proxy-pool --status   Show pool status");
                log::info!("  neotrix-proxy-pool --help     This help");
                log::info!("");
                log::info!("Config: {} (auto-created on first run)", config_path().display());
                log::info!("Subscription: edit subscription_urls (array) in config file");
                return;
            }
            other => {
                log::error!("Unknown option: {}. Use --help for usage.", other);
                return;
            }
        }
    }

    log::error!("[pool] NeoTrix Proxy Pool Manager v0.18.0");
    let config = load_config();
    log::error!("[pool] subscription urls: {:?}", config.subscription_urls);
    log::error!("[pool] check interval: normal({:.1}, {:.1}) clamped [{:.1}, {:.1}]s",
        config.check_interval.mean_secs, config.check_interval.std_secs,
        config.check_interval.min_secs, config.check_interval.max_secs);
    log::error!("[pool] tcp timeout: normal({:.1}, {:.1}) clamped [{:.1}, {:.1}]s",
        config.tcp_timeout.mean_secs, config.tcp_timeout.std_secs,
        config.tcp_timeout.min_secs, config.tcp_timeout.max_secs);
    log::error!("[pool] max upstreams: {}", config.max_upstreams);

    // Write default config if not exists
    let cfg_path = config_path();
    if !cfg_path.exists() {
        if let Ok(json) = serde_json::to_string_pretty(&config) {
            let _ = std::fs::write(&cfg_path, &json);
            log::error!("[pool] created default config at {}", cfg_path.display());
        }
    }

    let shutdown = ShutdownSignal::new();

    // First cycle immediately
    run_check_cycle(&config, &shutdown).await;

    // Daemon loop — 所有时间参数统一使用 RandomIntervalConfig，正态分布防冲撞
    let mut discovery_count: u64 = 0;
    let mut heartbeat_count: u64 = 0;
    let mut discovery_next = config.discovery_interval.duration();
    let mut heartbeat_next = Duration::from_secs(15);
    let signal = shutdown.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("proxy-pool daemon: failed to wait for Ctrl+C shutdown signal");
        shutdown.trigger("ctrl-c received");
    });

    while !signal.is_shutdown() {
        let check_wait = config.check_interval.duration();
        let mut shortest_wait = check_wait;

        // Schedule heartbeat check (every 15s base)
        if heartbeat_next < shortest_wait {
            shortest_wait = heartbeat_next;
        }
        // Schedule discovery check (config.discovery_interval)
        if discovery_next < shortest_wait {
            shortest_wait = discovery_next;
        }

        log::error!("[pool] next event in {:.1}s (check:{:.1}s hb:{}s disc:{}s)",
            shortest_wait.as_secs_f64(), check_wait.as_secs_f64(),
            heartbeat_next.as_secs_f64(), discovery_next.as_secs_f64());
        tokio::time::sleep(shortest_wait).await;

        // Countdown all timers
        heartbeat_next = if heartbeat_next <= shortest_wait {
            heartbeat_count += 1;
            // 心跳: 验证 neotrix-proxy 存活
            let alive = send_heartbeat();
            if !alive {
                log::error!("[pool] proxy not responding after {} beats, attempting restart...", heartbeat_count);
                // Try restart proxy (launcher handles this)
            }
            Duration::from_secs(15) // Reset heartbeat to base 15s
        } else {
            heartbeat_next - shortest_wait
        };

        discovery_next = if discovery_next <= shortest_wait {
            discovery_count += 1;
            log::error!("[pool] discovery cycle #{}...", discovery_count);
            run_check_cycle(&config, &signal).await;
            config.discovery_interval.duration()
        } else {
            discovery_next - shortest_wait
        };
    }

    log::error!("[pool] shutdown signal received, exiting daemon loop");
}

const ENCRYPTED_BRIDGE_BASE: u16 = 12000;

/// Simple connection pre-warming cache for bridges.
/// Stores pre-established kernel connections for rapid reuse, reducing CONNECT timing fingerprints.
use neotrix_proxy_kernel::BoxedStream;

type BridgeCacheInner = Arc<std::sync::Mutex<HashMap<(String, u16), (BoxedStream, Instant)>>>;

struct BridgeWarmCache {
    inner: BridgeCacheInner,
    max_age: Duration,
}

impl BridgeWarmCache {
    fn new() -> Self {
        Self {
            inner: Arc::new(std::sync::Mutex::new(HashMap::new())),
            max_age: Duration::from_secs(8),
        }
    }

    fn pop(&self, host: &str, port: u16) -> Option<BoxedStream> {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let key = (host.to_string(), port);
        if let Some((stream, created)) = map.remove(&key) {
            if created.elapsed() < self.max_age {
                return Some(stream);
            }
        }
        None
    }

    fn push(&self, host: &str, port: u16, stream: BoxedStream) {
        let key = (host.to_string(), port);
        if let Ok(mut map) = self.inner.lock() {
            map.insert(key, (stream, Instant::now()));
        }
    }
}

async fn encrypted_socks5_bridge(node: ProxyNode, knode: neotrix_proxy_kernel::node::ProxyNode, port: u16, shutdown: ShutdownSignal) {
    let addr = format!("127.0.0.1:{}", port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => { log::error!("[bridge] {} listening on {}", node.display(), addr); l }
        Err(e) => { log::error!("[bridge] {} bind {} failed: {}", node.display(), addr, e); return; }
    };
    let node_name = node.display();
    let warm_cache = Arc::new(BridgeWarmCache::new());
    let prewarm_queue: Arc<tokio::sync::Mutex<Vec<(String, u16)>>> = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Background pre-warming: establishes connections to recently-used targets
    {
        let cache = warm_cache.clone();
        let kn = knode.clone();
        let pwq = prewarm_queue.clone();
        let sd_prewarm = shutdown.clone();
        tokio::spawn(async move {
            if let Err(panic) = std::panic::AssertUnwindSafe(async move {
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                        _ = sd_prewarm.wait_shutdown() => {
                            log::error!("[bridge-prewarm] shutdown signal received");
                            break;
                        }
                    }
                    if sd_prewarm.is_shutdown() { break; }
                    let target = {
                        let mut q = pwq.lock().await;
                        q.pop()
                    };
                    if let Some((host, port)) = target {
                        if cache.pop(&host, port).is_some() {
                            // Already cached, skip
                            cache.push(&host, port, {
                                // create a fresh one to replace the stale one
                                match neotrix_proxy_kernel::connect_through(&kn, &host, port).await {
                                    Ok((s, _)) => s,
                                    Err(_) => continue,
                                }
                            });
                        } else {
                            match neotrix_proxy_kernel::connect_through(&kn, &host, port).await {
                                Ok((stream, _)) => {
                                    cache.push(&host, port, stream);
                                    log::error!("[bridge-prewarm] {}:{} cached", host, port);
                                }
                                Err(e) => {
                                    log::error!("[bridge-prewarm] {}:{} failed: {:?}", host, port, e);
                                }
                            }
                        }
                    }
                }
            }).catch_unwind().await {
                log::error!("[bridge-prewarm] background loop panic: {:?}", panic);
            }
        });
    }

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (mut client, _) = match result {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let kn = knode.clone();
                let nname = node_name.clone();
                let cache = warm_cache.clone();
                let pwq = prewarm_queue.clone();
                let sd_accept = shutdown.clone();
                tokio::spawn(async move {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};
                    // SOCKS5 greeting
                    let mut buf = [0u8; 2];
                    if client.read_exact(&mut buf).await.is_err() { return; }
                    if buf[0] != 0x05 { return; }
                    let nmethods = buf[1] as usize;
                    let mut methods = vec![0u8; nmethods];
                    if client.read_exact(&mut methods).await.is_err() { return; }
                    let _ = client.write_all(&[0x05, 0x00]).await;
                    // SOCKS5 request
                    let mut hdr = [0u8; 4];
                    if client.read_exact(&mut hdr).await.is_err() { return; }
                    if hdr[0] != 0x05 || hdr[1] != 0x01 || hdr[2] != 0x00 { return; }
                    let atype = hdr[3];
                    let (target_host, target_port) = match atype {
                        0x01 => {
                            let mut ip = [0u8; 4];
                            if client.read_exact(&mut ip).await.is_err() { return; }
                            let mut port_buf = [0u8; 2];
                            if client.read_exact(&mut port_buf).await.is_err() { return; }
                            (format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]), u16::from_be_bytes(port_buf))
                        }
                        0x03 => {
                            let mut len_buf = [0u8; 1];
                            if client.read_exact(&mut len_buf).await.is_err() { return; }
                            let mut domain = vec![0u8; len_buf[0] as usize];
                            if client.read_exact(&mut domain).await.is_err() { return; }
                            let mut port_buf = [0u8; 2];
                            if client.read_exact(&mut port_buf).await.is_err() { return; }
                            (String::from_utf8_lossy(&domain).to_string(), u16::from_be_bytes(port_buf))
                        }
                        _ => return,
                    };
                    if sd_accept.is_shutdown() { return; }
                    // Try cache first, then create fresh
                    let mut upstream = match cache.pop(&target_host, target_port) {
                        Some(stream) => {
                            log::error!("[bridge-cache] HIT {}:{}", target_host, target_port);
                            stream
                        }
                        None => {
                            match neotrix_proxy_kernel::connect_through(&kn, &target_host, target_port).await {
                                Ok((s, _)) => s,
                                Err(e) => {
                                    log::error!("[bridge] {} -> {}:{} failed: {:?}", nname, target_host, target_port, e);
                                    return;
                                }
                            }
                        }
                    };
                    // Schedule pre-warming for next connection to this target
                    pwq.lock().await.push((target_host.clone(), target_port));
                    let resp = [0x05, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                    let _ = client.write_all(&resp).await;
                    let _ = tokio::io::copy_bidirectional(&mut client, &mut *upstream).await;
                });
            }
            _ = shutdown.wait_shutdown() => {
                log::error!("[bridge] {} shutdown signal received, stopping", addr);
                break;
            }
        }
    }
}

fn fast_random() -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let x = nanos.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((x >> 33) as f64) / (1u64 << 31) as f64
}
