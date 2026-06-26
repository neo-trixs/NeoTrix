use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream, UnixListener};
use tokio::sync::RwLock;

use neotrix_proxy_kernel::BoxedStream;
use neotrix_proxy_kernel::node::{ProtocolKind, ProxyNode};
use neotrix_proxy_kernel::telemetry::{ConnectStats, TelemetryCollector};

const LISTEN_ADDR: &str = "127.0.0.1:11080";
const UPSTREAM_CONFIG_PATH: &str = ".neotrix/proxy-upstreams.conf";

fn socket_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.neotrix/neotrix-proxy.sock", home)
}

fn pid_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.neotrix/neotrix-proxy.pid", home)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ProxyMode {
    Direct,
    Stealth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FingerprintProfile {
    ChromeMac,
    FirefoxMac,
    SafariMac,
    ChromeLinux,
    EdgeWin,
}

static FINGERPRINT_PROFILES: &[FingerprintProfile] = &[
    FingerprintProfile::ChromeMac,
    FingerprintProfile::FirefoxMac,
    FingerprintProfile::SafariMac,
    FingerprintProfile::ChromeLinux,
    FingerprintProfile::EdgeWin,
];

impl FingerprintProfile {
    fn user_agent(&self) -> &'static str {
        match self {
            FingerprintProfile::ChromeMac => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36"
            }
            FingerprintProfile::FirefoxMac => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:127.0) Gecko/20100101 Firefox/127.0"
            }
            FingerprintProfile::SafariMac => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15"
            }
            FingerprintProfile::ChromeLinux => {
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36"
            }
            FingerprintProfile::EdgeWin => {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Edg/125.0.0.0"
            }
        }
    }

    fn random() -> Self {
        let i = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            / 1000) as usize
            % FINGERPRINT_PROFILES.len();
        FINGERPRINT_PROFILES[i]
    }
}

#[derive(Debug, Clone)]
struct UpstreamProxy {
    addr: String,
    username: Option<String>,
    password: Option<String>,
    is_socks5: bool,
    native_node: Option<ProxyNode>,
}

impl UpstreamProxy {
    fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();
        if input.is_empty() || input.starts_with('#') {
            return Err("skip".into());
        }
        if input.starts_with("native://") {
            return parse_native_upstream(input);
        }
        if let Some(rest) = input.strip_prefix("socks5://") {
            let (user, pass, addr) = parse_user_pass_addr(rest);
            Ok(Self { addr, username: user, password: pass, is_socks5: true, native_node: None })
        } else if let Some(rest) = input.strip_prefix("http://") {
            let (user, pass, addr) = parse_user_pass_addr(rest);
            Ok(Self { addr, username: user, password: pass, is_socks5: false, native_node: None })
        } else if let Some(rest) = input.strip_prefix("socks5h://") {
            let (user, pass, addr) = parse_user_pass_addr(rest);
            Ok(Self { addr, username: user, password: pass, is_socks5: true, native_node: None })
        } else {
            Err(format!("unsupported scheme: {}", input))
        }
    }

    fn to_kernel_node(&self) -> ProxyNode {
        if let Some(ref node) = self.native_node {
            return node.clone();
        }
        let proto = if self.is_socks5 { ProtocolKind::Socks5 } else { ProtocolKind::Http };
        let (server, port) = split_host_port(&self.addr, 1080);
        ProxyNode {
            uri: format!("{}://{}", if self.is_socks5 { "socks5" } else { "http" }, self.addr),
            protocol: proto,
            server: server.to_string(),
            port,
            name: String::new(),
            method: None,
            password: self.password.clone(),
            username: self.username.clone(),
            uuid: None,
            cipher: None,
            sni: None,
            tls: false,
            skip_cert_verify: false,
            network: None,
            path: None,
            host: None,
            alpn: None,
            fingerprint: None,
            flow: None,
            public_key: None,
            short_id: None,
            obfs: None,
            obfs_password: None,
        }
    }

    fn display(&self) -> String {
        if let Some(node) = &self.native_node {
            return format!("native://{}@{}:{}", node.protocol.name(), node.server, node.port);
        }
        let scheme = if self.is_socks5 { "socks5" } else { "http" };
        if let Some(user) = &self.username {
            format!("{}://{}:****@{}", scheme, user, self.addr)
        } else {
            format!("{}://{}", scheme, self.addr)
        }
    }
}

fn parse_native_upstream(input: &str) -> Result<UpstreamProxy, String> {
    let rest = input.strip_prefix("native://").ok_or("not native")?;
    let (proto_name, query) = rest.split_once('?').ok_or("missing query string")?;
    let params: HashMap<&str, &str> = query.split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let val = parts.next().unwrap_or("");
            Some((key, val))
        })
        .collect();

    let protocol = match proto_name {
        "ss" | "shadowsocks" => ProtocolKind::Shadowsocks,
        "trojan" => ProtocolKind::Trojan,
        "hysteria2" | "hy2" => ProtocolKind::Hysteria2,
        _ => return Err(format!("unsupported native protocol: {}", proto_name)),
    };

    let server = params.get("server").ok_or("missing server")?.to_string();
    let port: u16 = params.get("port").ok_or("missing port")?
        .parse().map_err(|_| "invalid port".to_string())?;
    let password = params.get("password").map(|s| s.to_string());
    let method = params.get("method").map(|s| s.to_string());
    let sni = params.get("sni").map(|s| s.to_string());
    let skip_cert = params.get("insecure").is_some_and(|v| *v != "false" && *v != "0");

    let node = ProxyNode {
        uri: input.to_string(),
        protocol,
        server: server.clone(),
        port,
        name: String::new(),
        method,
        password,
        username: None,
        uuid: None,
        cipher: None,
        sni,
        tls: matches!(protocol, ProtocolKind::Trojan | ProtocolKind::Hysteria2),
        skip_cert_verify: skip_cert,
        network: None,
        path: None,
        host: None,
        alpn: None,
        fingerprint: None,
        flow: None,
        public_key: None,
        short_id: None,
        obfs: None,
        obfs_password: None,
    };

    Ok(UpstreamProxy {
        addr: format!("{}:{}", server, port),
        username: None,
        password: None,
        is_socks5: false,
        native_node: Some(node),
    })
}

fn split_host_port(addr: &str, default_port: u16) -> (String, u16) {
    if let Some(bracket_end) = addr.rfind(']') {
        let host = &addr[..=bracket_end];
        let after = &addr[bracket_end + 1..];
        if let Some(port_str) = after.strip_prefix(':') {
            let port: u16 = port_str.parse().unwrap_or(default_port);
            (host.to_string(), port)
        } else {
            (addr.to_string(), default_port)
        }
    } else if let Some(pos) = addr.rfind(':') {
        let host = &addr[..pos];
        let port_str = &addr[pos + 1..];
        let port: u16 = port_str.parse().unwrap_or(default_port);
        (host.to_string(), port)
    } else {
        (addr.to_string(), default_port)
    }
}

fn parse_user_pass_addr(input: &str) -> (Option<String>, Option<String>, String) {
    if let Some(at_pos) = input.rfind('@') {
        let user_pass = &input[..at_pos];
        let addr = input[at_pos + 1..].to_string();
        if let Some(colon_pos) = user_pass.find(':') {
            let user = user_pass[..colon_pos].to_string();
            let pass = user_pass[colon_pos + 1..].to_string();
            (Some(user), Some(pass), addr)
        } else {
            (Some(user_pass.to_string()), None, addr)
        }
    } else {
        (None, None, input.to_string())
    }
}

fn default_upstream_config_path() -> String {
    if let Ok(home) = std::env::var("HOME") {
        format!("{}/{}", home, UPSTREAM_CONFIG_PATH)
    } else {
        format!("/tmp/{}", UPSTREAM_CONFIG_PATH)
    }
}

fn load_upstreams_from_file(path: &str) -> Vec<UpstreamProxy> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content.lines().filter_map(|line| {
        UpstreamProxy::parse(line).ok()
    }).collect()
}

struct ProxyState {
    mode: ProxyMode,
    current_profile: FingerprintProfile,
    rotation_interval: Duration,
    last_rotation: SystemTime,
    connections_served: u64,
    upstreams: Vec<UpstreamProxy>,
    upstream_index: usize,
    telemetry: TelemetryCollector,
}

impl ProxyState {
    fn new() -> Self {
        let config_path = default_upstream_config_path();
        let upstreams = load_upstreams_from_file(&config_path);
        if !upstreams.is_empty() {
            log::error!("[proxy] loaded {} upstream proxies from {}", upstreams.len(), config_path);
        }
        Self {
            mode: ProxyMode::Direct,
            current_profile: FingerprintProfile::ChromeMac,
            rotation_interval: Duration::from_secs(300),
            last_rotation: SystemTime::now(),
            connections_served: 0,
            upstreams,
            upstream_index: 0,
            telemetry: TelemetryCollector::new(),
        }
    }

    fn maybe_rotate(&mut self) {
        if self.mode != ProxyMode::Stealth {
            return;
        }
        if SystemTime::now()
            .duration_since(self.last_rotation)
            .unwrap_or_default()
            >= self.rotation_interval
        {
            self.current_profile = FingerprintProfile::random();
            self.last_rotation = SystemTime::now();
            log::error!("[proxy] rotated fingerprint: {:?}", self.current_profile);
        }
    }

    fn rotate_now(&mut self) {
        self.current_profile = FingerprintProfile::random();
        self.last_rotation = SystemTime::now();
        log::error!("[proxy] forced fingerprint rotation: {:?}", self.current_profile);
    }

    fn next_upstream(&mut self) -> Option<UpstreamProxy> {
        if self.upstreams.is_empty() {
            return None;
        }
        let idx = self.upstream_index % self.upstreams.len();
        self.upstream_index = self.upstream_index.wrapping_add(1);
        Some(self.upstreams[idx].clone())
    }

    fn rotate_ip_now(&mut self) -> Option<String> {
        if self.upstreams.is_empty() {
            return None;
        }
        self.upstream_index = self.upstream_index.wrapping_add(1);
        let idx = (self.upstream_index.wrapping_sub(1)) % self.upstreams.len();
        Some(self.upstreams[idx].display())
    }

    fn reload_upstreams(&mut self) -> usize {
        let config_path = default_upstream_config_path();
        self.upstreams = load_upstreams_from_file(&config_path);
        self.upstream_index = 0;
        self.upstreams.len()
    }
}

async fn handle_connect(
    mut client: TcpStream,
    addr: String,
    state: Arc<RwLock<ProxyState>>,
) {
    let _ = client.set_nodelay(true);
    let (_profile, upstream) = {
        let mut s = state.write().await;
        s.connections_served += 1;
        s.maybe_rotate();
        let p = s.current_profile;
        let u = s.next_upstream();
        (p, u)
    };

    match upstream {
        None => {
            match tokio::time::timeout(Duration::from_secs(15), TcpStream::connect(&addr)).await {
                Ok(Ok(mut remote)) => {
                    let _ = remote.set_nodelay(true);
                    let _ = client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await;
                    let _ = tokio::time::timeout(
                        Duration::from_secs(300),
                        tokio::io::copy_bidirectional(&mut client, &mut remote),
                    ).await;
                }
                _ => {
                    let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
                }
            }
        }
        Some(up) => {
            let (target_host, target_port) = split_host_port(&addr, 443);
            // Strip IPv6 brackets for protocol-level targets (SOCKS5/HTTP CONNECT)
            let target_host = target_host.strip_prefix('[')
                .and_then(|h| h.strip_suffix(']'))
                .unwrap_or(&target_host)
                .to_string();
            if let Some(ref node) = up.native_node {
                log::error!("[proxy] native connect via {} to {}:{}",
                    node.protocol.name(), target_host, target_port);
                let result = neotrix_proxy_kernel::connect_through(node, &target_host, target_port).await
                    .map_err(|e| format!("native connect failed for {}: {}", addr, e));
                complete_tunnel(&mut client, &state, result).await;
            } else {
                let node = up.to_kernel_node();
                let result = neotrix_proxy_kernel::connect_through(&node, &target_host, target_port).await
                    .map_err(|e| format!("kernel chain failed for {}: {}", addr, e));
                complete_tunnel(&mut client, &state, result).await;
            }
        }
    }
}

async fn complete_tunnel(
    client: &mut TcpStream,
    state: &Arc<RwLock<ProxyState>>,
    result: Result<(BoxedStream, ConnectStats), String>,
) {
    match result {
        Ok((mut remote, stat)) => {
            state.write().await.telemetry.record(stat);
            let _ = client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await;
            let _ = tokio::time::timeout(
                Duration::from_secs(300),
                tokio::io::copy_bidirectional(client, &mut remote),
            ).await;
        }
        Err(e) => {
            log::error!("[proxy] {}", e);
            let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
        }
    }
}

async fn handle_client(
    mut stream: TcpStream,
    state: Arc<RwLock<ProxyState>>,
) {
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let head = String::from_utf8_lossy(&buf[..n]);
    let head = head.trim();

    if head.starts_with("CONNECT ") {
        let rest = head.trim_start_matches("CONNECT ").trim();
        let addr = rest.split_whitespace().next().unwrap_or("");
        if addr.is_empty() {
            return;
        }
        // RFC 7231 §4.3.6: CONNECT must include port
        let has_port = if let Some(bracket_end) = addr.rfind(']') {
            addr[bracket_end + 1..].starts_with(':')
        } else {
            addr.contains(':')
        };
        if !has_port {
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\nMissing port in CONNECT\r\n").await;
            return;
        }
        handle_connect(stream, addr.to_string(), state).await;
    } else {
        let _ = stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\n\r\nOnly CONNECT supported\r\n").await;
    }
}

async fn handle_control(state: Arc<RwLock<ProxyState>>) {
    let _ = std::fs::remove_file(socket_path());
    let listener = match UnixListener::bind(socket_path()) {
        Ok(l) => l,
        Err(e) => {
            log::error!("[proxy] control socket bind failed: {}", e);
            return;
        }
    };

    while let Ok((mut stream, _)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(async move {
            let mut buf = String::new();
            let mut reader = BufReader::new(&mut stream);
            if reader.read_line(&mut buf).await.is_err() {
                return;
            }
            let cmd = buf.trim();
            let response = {
                let mut s = state.write().await;
                match cmd {
                    "mode=stealth" => {
                        s.mode = ProxyMode::Stealth;
                        s.rotate_now();
                        b"OK: stealth mode\n".to_vec()
                    }
                    "mode=direct" => {
                        s.mode = ProxyMode::Direct;
                        b"OK: direct mode\n".to_vec()
                    }
                    "mode=ip_per_request" => {
                        s.mode = ProxyMode::Stealth;
                        b"OK: ip_per_request mode (upstream round-robin)\n".to_vec()
                    }
                    "fingerprint=rotate" => {
                        s.rotate_now();
                        format!("OK: rotated to {:?}\n", s.current_profile).into_bytes()
                    }
                    "ip=rotate" => {
                        match s.rotate_ip_now() {
                            Some(desc) => {
                                format!("OK: rotated IP to upstream {}\n", desc).into_bytes()
                            }
                            None => {
                                b"ERR: no upstream proxies configured; add via upstream=add or upstream=load\n".to_vec()
                            }
                        }
                    }
                    "upstream=list" => {
                        if s.upstreams.is_empty() {
                            b"no upstream proxies configured\n".to_vec()
                        } else {
                            let msg = s.upstreams.iter().enumerate().map(|(i, u)| {
                                let marker = if i == s.upstream_index % s.upstreams.len() { " <-- current" } else { "" };
                                format!("  {}. {}{}", i + 1, u.display(), marker)
                            }).collect::<Vec<_>>().join("\n");
                            format!("upstreams ({}):\n{}\n", s.upstreams.len(), msg).into_bytes()
                        }
                    }
                    "upstream=clear" => {
                        let count = s.upstreams.len();
                        s.upstreams.clear();
                        s.upstream_index = 0;
                        format!("OK: cleared {} upstreams\n", count).into_bytes()
                    }
                    cmd if cmd.starts_with("upstream=add,") => {
                        let val = cmd.trim_start_matches("upstream=add,").trim();
                        match UpstreamProxy::parse(val) {
                            Ok(proxy) => {
                                s.upstreams.push(proxy.clone());
                                format!("OK: added {}\n", proxy.display()).into_bytes()
                            }
                            Err(e) => {
                                format!("ERR: {}\n", e).into_bytes()
                            }
                        }
                    }
                    cmd if cmd.starts_with("upstream=load,") => {
                        let path = cmd.trim_start_matches("upstream=load,").trim();
                        let loaded = load_upstreams_from_file(path);
                        if loaded.is_empty() {
                            format!("WARN: no valid upstreams in {}\n", path).into_bytes()
                        } else {
                            s.upstreams = loaded;
                            s.upstream_index = 0;
                            format!("OK: loaded {} upstreams from {}\n", s.upstreams.len(), path).into_bytes()
                        }
                    }
                    "upstream=reload" => {
                        let count = s.reload_upstreams();
                        format!("OK: reloaded {} upstreams from config\n", count).into_bytes()
                    }
                    "status" => {
                        let upstream_info = if s.upstreams.is_empty() {
                            "no upstreams (direct)".into()
                        } else {
                            let idx = s.upstream_index % s.upstreams.len();
                            format!("{} upstreams, current: {}", s.upstreams.len(), s.upstreams[idx].display())
                        };
                        let msg = format!(
                            "mode={:?}, profile={:?}, connections={}, ua={}\nupstream={}\n",
                            s.mode, s.current_profile, s.connections_served,
                            s.current_profile.user_agent(), upstream_info
                        );
                        msg.into_bytes()
                    }
                    _ => {
                        b"ERR: unknown command\n".to_vec()
                    }
                }
            };
            let _ = stream.write_all(&response).await;
        });
    }
}

#[tokio::main]
async fn main() {
    // Ensure ~/.neotrix/ exists for socket and pid file
    if let Some(parent) = std::path::Path::new(&socket_path()).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(pid_path(), format!("{}", std::process::id()));

    let state = Arc::new(RwLock::new(ProxyState::new()));
    let shutdown = Arc::new(AtomicBool::new(false));

    // Graceful shutdown: clean up socket on Ctrl+C, break accept loop
    let socket_path = socket_path();
    let socket_path_clone = socket_path.clone();
    let state_clone = state.clone();
    let shutdown_clone = shutdown.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        log::error!("\n[proxy] shutting down...");
        shutdown_clone.store(true, Ordering::SeqCst);
        let s = state_clone.read().await;
        log::error!("[proxy] total connections served: {}", s.connections_served);
        drop(s);
        let _ = std::fs::remove_file(&socket_path_clone);
        let _ = std::fs::remove_file(pid_path());
        log::error!("[proxy] bye");
    });

    tokio::spawn(handle_control(state.clone()));

    log::error!("[proxy] HTTP CONNECT proxy on {}", LISTEN_ADDR);
    log::error!("[proxy] control socket at {}", socket_path);
    log::error!("[proxy] commands: mode=stealth|direct|ip_per_request, fingerprint=rotate, ip=rotate,");
    log::error!("[proxy]           upstream=add,<url>, upstream=clear|list|reload, upstream=load,<path>");

    let listener = TcpListener::bind(LISTEN_ADDR).await.unwrap_or_else(|e| {
        log::error!("[proxy] bind failed: {}", e);
        std::process::exit(1);
    });

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, _addr)) => {
                        tokio::spawn(handle_client(stream, state.clone()));
                    }
                    Err(e) => {
                        if shutdown.load(Ordering::SeqCst) {
                            break;
                        }
                        log::error!("[proxy] accept error: {}", e);
                    }
                }
            }
            _ = async {
                while !shutdown.load(Ordering::SeqCst) {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            } => {
                break;
            }
        }
    }
}
