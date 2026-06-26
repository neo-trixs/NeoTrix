//! 网络资源池 — DNS / 路由 / IP 池 + 自动有效性更新
//!
//! 对标:
//! - **ProxyChains-NG**: 动态 DNS 解析 + 路由切换
//! - **V2Ray/Xray**: 负载均衡 + 健康检测
//! - **Tor**: 节点 liveliness 检测
//!
//! 核心:
//! - DNS 服务器池 + 延迟检测
//! - 路由/出口节点池 + 健康计数
//! - IP 资源池 + 有效性状态
//! - 自动定时刷新 (默认每15秒)
//! - 有效数量 tracking

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::Rng;
use tokio::sync::RwLock;
use tokio::time::sleep;

use super::proxy_chain::{ProxyNode, ProxyProtocol};

const POOL_REFRESH_INTERVAL_SECS: u64 = 9;
const DNS_CHECK_TIMEOUT_SECS: u64 = 5;
const ROUTE_CHECK_TIMEOUT_SECS: u64 = 5;

#[derive(Debug, Clone)]
pub struct DnsServer {
    pub addr: String,
    pub label: String,
    pub effective: bool,
    pub latency_ms: f64,
    pub protocol: DnsProtocol,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DnsProtocol {
    Udp,
    Tcp,
    Tls,
    Https,
}

impl DnsProtocol {
    pub fn default_port(&self) -> u16 {
        match self {
            DnsProtocol::Udp => 53,
            DnsProtocol::Tcp => 53,
            DnsProtocol::Tls => 853,
            DnsProtocol::Https => 443,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteNode {
    pub id: String,
    pub addr: String,
    pub geo: String,
    pub effective: bool,
    pub latency_ms: f64,
    pub success_count: u64,
    pub fail_count: u64,
    pub last_check: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct IpResource {
    pub ip: String,
    pub port: u16,
    pub geo: String,
    pub protocol: ProxyProtocol,
    pub effective: bool,
    pub latency_ms: f64,
    pub last_verified: Option<Instant>,
    pub success_count: u64,
    pub fail_count: u64,
}

impl IpResource {
    pub fn to_proxy_node(&self) -> ProxyNode {
        ProxyNode {
            protocol: self.protocol,
            host: self.ip.clone(),
            port: self.port,
            username: None,
            password: None,
            geo_tag: Some(self.geo.clone()),
            label: format!(
                "{}://{}:{}",
                self.protocol.as_url_scheme(),
                self.ip,
                self.port
            ),
            weight: if self.effective { 1.0 } else { 0.0 },
        }
    }
}

/// 网络资源池 — 自动管理 DNS/路由/IP 有效性
pub struct NetworkResourcePool {
    dns_servers: RwLock<Vec<DnsServer>>,
    route_nodes: RwLock<Vec<RouteNode>>,
    ip_resources: RwLock<Vec<IpResource>>,
    refresh_interval_secs: AtomicU64,
    running: AtomicBool,
    refresh_count: AtomicU64,
}

impl Default for NetworkResourcePool {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkResourcePool {
    pub fn new() -> Self {
        Self {
            dns_servers: RwLock::new(Vec::new()),
            route_nodes: RwLock::new(Vec::new()),
            ip_resources: RwLock::new(Vec::new()),
            refresh_interval_secs: AtomicU64::new(POOL_REFRESH_INTERVAL_SECS),
            running: AtomicBool::new(false),
            refresh_count: AtomicU64::new(0),
        }
    }

    pub fn with_refresh_interval(self, secs: u64) -> Self {
        self.refresh_interval_secs.store(secs, Ordering::Relaxed);
        self
    }

    // ── DNS 池 ──

    pub async fn add_dns(&self, addr: &str, protocol: DnsProtocol) {
        let mut servers = self.dns_servers.write().await;
        let label = format!("dns-{}", servers.len() + 1);
        servers.push(DnsServer {
            addr: addr.to_string(),
            label,
            effective: false,
            latency_ms: 0.0,
            protocol,
        });
    }

    pub async fn add_dns_list(&self, addrs: &[(&str, DnsProtocol)]) {
        for (addr, protocol) in addrs {
            self.add_dns(addr, *protocol).await;
        }
    }

    pub async fn effective_dns_count(&self) -> usize {
        self.dns_servers
            .read()
            .await
            .iter()
            .filter(|d| d.effective)
            .count()
    }

    pub async fn total_dns_count(&self) -> usize {
        self.dns_servers.read().await.len()
    }

    pub async fn get_effective_dns(&self) -> Vec<DnsServer> {
        self.dns_servers
            .read()
            .await
            .iter()
            .filter(|d| d.effective)
            .cloned()
            .collect()
    }

    /// 获取最快 DNS
    pub async fn fastest_dns(&self) -> Option<DnsServer> {
        let servers = self.dns_servers.read().await;
        servers
            .iter()
            .filter(|d| d.effective)
            .min_by(|a, b| {
                a.latency_ms
                    .partial_cmp(&b.latency_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    // ── 路由池 ──

    pub async fn add_route(&self, addr: &str, geo: &str) {
        let mut nodes = self.route_nodes.write().await;
        let id = format!("route-{}", nodes.len() + 1);
        nodes.push(RouteNode {
            id,
            addr: addr.to_string(),
            geo: geo.to_string(),
            effective: false,
            latency_ms: 0.0,
            success_count: 0,
            fail_count: 0,
            last_check: None,
        });
    }

    pub async fn add_route_list(&self, routes: &[(&str, &str)]) {
        for (addr, geo) in routes {
            self.add_route(addr, geo).await;
        }
    }

    pub async fn effective_route_count(&self) -> usize {
        self.route_nodes
            .read()
            .await
            .iter()
            .filter(|r| r.effective)
            .count()
    }

    pub async fn total_route_count(&self) -> usize {
        self.route_nodes.read().await.len()
    }

    pub async fn get_effective_routes(&self) -> Vec<RouteNode> {
        self.route_nodes
            .read()
            .await
            .iter()
            .filter(|r| r.effective)
            .cloned()
            .collect()
    }

    // ── IP 池 ──

    pub async fn add_ip(&self, ip: &str, port: u16, geo: &str, protocol: ProxyProtocol) {
        let mut resources = self.ip_resources.write().await;
        resources.push(IpResource {
            ip: ip.to_string(),
            port,
            geo: geo.to_string(),
            protocol,
            effective: false,
            latency_ms: 0.0,
            last_verified: None,
            success_count: 0,
            fail_count: 0,
        });
    }

    pub async fn add_ip_list(&self, ips: &[(&str, u16, &str, ProxyProtocol)]) {
        for (ip, port, geo, protocol) in ips {
            self.add_ip(ip, *port, geo, *protocol).await;
        }
    }

    pub async fn effective_ip_count(&self) -> usize {
        self.ip_resources
            .read()
            .await
            .iter()
            .filter(|r| r.effective)
            .count()
    }

    pub async fn total_ip_count(&self) -> usize {
        self.ip_resources.read().await.len()
    }

    pub async fn get_effective_ips(&self) -> Vec<IpResource> {
        self.ip_resources
            .read()
            .await
            .iter()
            .filter(|r| r.effective)
            .cloned()
            .collect()
    }

    pub async fn get_effective_proxy_nodes(&self) -> Vec<ProxyNode> {
        self.ip_resources
            .read()
            .await
            .iter()
            .filter(|r| r.effective)
            .map(|r| r.to_proxy_node())
            .collect()
    }

    /// 随机获取一个有效 IP
    pub async fn random_effective_ip(&self) -> Option<IpResource> {
        let effective: Vec<_> = self
            .ip_resources
            .read()
            .await
            .iter()
            .filter(|r| r.effective)
            .cloned()
            .collect();
        if effective.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        Some(effective[rng.gen_range(0..effective.len())].clone())
    }

    // ── 健康检查 ──

    /// DNS 健康检查: 发送真实 DNS 查询并验证响应 (对标 ProxyChains-NG)
    async fn check_dns(server: &DnsServer) -> bool {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        let addr = format!("{}:{}", server.addr, server.protocol.default_port());
        let mut stream = match tokio::time::timeout(
            Duration::from_secs(DNS_CHECK_TIMEOUT_SECS),
            TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(s)) => s,
            _ => return false,
        };

        // 最小 DNS 查询: example.com A record over TCP
        // 12字节头 + 域名(1+7+1+3+1=13) + 类型(2) + 类(2) = 29 字节
        let query: [u8; 29] = [
            0x00, 0x01, // ID=1
            0x01, 0x00, // 标准查询, RD=1
            0x00, 0x01, // 1 个问题
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0 answers, auth, add
            0x07, b'e', b'x', b'a', b'm', b'p', b'l', b'e', // "example"
            0x03, b'c', b'o', b'm', // ".com"
            0x00, // 域名结束
            0x00, 0x01, // Type A
            0x00, 0x01, // Class IN
        ];

        // TCP DNS 前加 2 字节长度前缀 (C-v4-04: u16 防止截断)
        let len_prefixed = {
            let qlen = query.len() as u16;
            let mut buf = Vec::with_capacity(query.len() + 2);
            buf.extend_from_slice(&qlen.to_be_bytes());
            buf.extend_from_slice(&query);
            buf
        };

        if tokio::time::timeout(Duration::from_secs(2), stream.write_all(&len_prefixed))
            .await
            .is_err()
        {
            return false;
        }

        // 读响应头 (至少 2 字节长度 + 12 字节响应头)
        let mut header = [0u8; 14];
        if tokio::time::timeout(Duration::from_secs(2), stream.read_exact(&mut header))
            .await
            .is_err()
        {
            return false;
        }

        // 验证响应: QR=1 (响应标志), RCODE=0 (无错误)
        let response_flags = ((header[2 + 2] as u16) << 8) | header[2 + 3] as u16;
        (response_flags & 0x8000) != 0 && (response_flags & 0x000F) == 0
    }

    async fn check_route(node: &RouteNode) -> bool {
        tokio::time::timeout(
            Duration::from_secs(ROUTE_CHECK_TIMEOUT_SECS),
            tokio::net::TcpStream::connect(&node.addr),
        )
        .await
        .is_ok()
    }

    async fn check_ip(resource: &IpResource) -> bool {
        tokio::time::timeout(
            Duration::from_secs(ROUTE_CHECK_TIMEOUT_SECS),
            tokio::net::TcpStream::connect(format!("{}:{}", resource.ip, resource.port)),
        )
        .await
        .is_ok()
    }

    /// 刷新所有池的有效性
    pub async fn refresh_all(&self) -> PoolSnapshot {
        // 刷新 DNS
        {
            let mut servers = self.dns_servers.write().await;
            for server in servers.iter_mut() {
                let start = Instant::now();
                server.effective = Self::check_dns(server).await;
                server.latency_ms = start.elapsed().as_millis() as f64;
            }
        }

        // 刷新路由
        {
            let mut nodes = self.route_nodes.write().await;
            for node in nodes.iter_mut() {
                let start = Instant::now();
                let ok = Self::check_route(node).await;
                node.effective = ok;
                node.latency_ms = start.elapsed().as_millis() as f64;
                node.last_check = Some(Instant::now());
                if ok {
                    node.success_count += 1;
                } else {
                    node.fail_count += 1;
                }
            }
        }

        // 刷新 IP
        {
            let mut resources = self.ip_resources.write().await;
            for resource in resources.iter_mut() {
                let start = Instant::now();
                let ok = Self::check_ip(resource).await;
                resource.effective = ok;
                resource.latency_ms = start.elapsed().as_millis() as f64;
                resource.last_verified = Some(Instant::now());
                if ok {
                    resource.success_count += 1;
                } else {
                    resource.fail_count += 1;
                }
            }
        }

        self.refresh_count.fetch_add(1, Ordering::Relaxed);
        self.snapshot().await
    }

    /// 自动刷新循环（默认每15秒）
    pub async fn start_auto_refresh(self: Arc<Self>) {
        if self.running.swap(true, Ordering::AcqRel) {
            return;
        }
        loop {
            if !self.running.load(Ordering::Acquire) {
                break;
            }
            sleep(Duration::from_secs(
                self.refresh_interval_secs.load(Ordering::Relaxed),
            ))
            .await;
            self.refresh_all().await;
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    /// 获取当前池快照
    pub async fn snapshot(&self) -> PoolSnapshot {
        PoolSnapshot {
            dns_total: self.total_dns_count().await,
            dns_effective: self.effective_dns_count().await,
            route_total: self.total_route_count().await,
            route_effective: self.effective_route_count().await,
            ip_total: self.total_ip_count().await,
            ip_effective: self.effective_ip_count().await,
            refresh_count: self.refresh_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolSnapshot {
    pub dns_total: usize,
    pub dns_effective: usize,
    pub route_total: usize,
    pub route_effective: usize,
    pub ip_total: usize,
    pub ip_effective: usize,
    pub refresh_count: u64,
}

impl PoolSnapshot {
    pub fn dns_effective_rate(&self) -> f64 {
        if self.dns_total == 0 {
            0.0
        } else {
            self.dns_effective as f64 / self.dns_total as f64
        }
    }

    pub fn route_effective_rate(&self) -> f64 {
        if self.route_total == 0 {
            0.0
        } else {
            self.route_effective as f64 / self.route_total as f64
        }
    }

    pub fn ip_effective_rate(&self) -> f64 {
        if self.ip_total == 0 {
            0.0
        } else {
            self.ip_effective as f64 / self.ip_total as f64
        }
    }
}

/// 预设公共 DNS 服务器
pub fn default_public_dns() -> Vec<(&'static str, DnsProtocol)> {
    vec![
        ("1.1.1.1", DnsProtocol::Udp),
        ("1.0.0.1", DnsProtocol::Udp),
        ("8.8.8.8", DnsProtocol::Udp),
        ("8.8.4.4", DnsProtocol::Udp),
        ("9.9.9.9", DnsProtocol::Udp),
        ("208.67.222.222", DnsProtocol::Udp),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_pool() {
        let pool = NetworkResourcePool::new();
        pool.add_dns("1.1.1.1", DnsProtocol::Udp).await;
        pool.add_dns("8.8.8.8", DnsProtocol::Udp).await;
        assert_eq!(pool.total_dns_count().await, 2);
        // 初次刷新前有效数量为0
        assert_eq!(pool.effective_dns_count().await, 0);
    }

    #[tokio::test]
    async fn test_route_pool() {
        let pool = NetworkResourcePool::new();
        pool.add_route("192.168.1.1:8080", "local").await;
        pool.add_route("10.0.0.1:3128", "local").await;
        assert_eq!(pool.total_route_count().await, 2);
    }

    #[tokio::test]
    async fn test_ip_pool() {
        let pool = NetworkResourcePool::new();
        pool.add_ip("1.2.3.4", 8080, "US", ProxyProtocol::Http)
            .await;
        pool.add_ip("5.6.7.8", 1080, "JP", ProxyProtocol::Socks5)
            .await;
        assert_eq!(pool.total_ip_count().await, 2);
    }

    #[tokio::test]
    async fn test_effective_counts_start_at_zero() {
        let pool = NetworkResourcePool::new();
        pool.add_dns_list(&[("1.1.1.1", DnsProtocol::Udp), ("8.8.8.8", DnsProtocol::Udp)])
            .await;
        pool.add_route_list(&[("10.0.0.1:80", "local")]).await;
        pool.add_ip_list(&[("203.0.113.1", 8080, "US", ProxyProtocol::Http)])
            .await;

        let snap = pool.snapshot().await;
        assert_eq!(snap.dns_total, 2);
        assert_eq!(snap.dns_effective, 0);
        assert_eq!(snap.route_total, 1);
        assert_eq!(snap.route_effective, 0);
        assert_eq!(snap.ip_total, 1);
        assert_eq!(snap.ip_effective, 0);
    }

    #[tokio::test]
    async fn test_refresh_updates_effectiveness() {
        let pool = NetworkResourcePool::new();
        pool.add_dns("1.1.1.1", DnsProtocol::Udp).await;

        let snap = pool.refresh_all().await;
        // 1.1.1.1:53 可能可达
        assert_eq!(snap.dns_total, 1);
        assert!(snap.refresh_count >= 1);
    }

    #[tokio::test]
    async fn test_effective_rates() {
        let pool = NetworkResourcePool::new();
        pool.add_ip("10.0.0.1", 8080, "US", ProxyProtocol::Http)
            .await;
        pool.add_ip("10.0.0.2", 8080, "US", ProxyProtocol::Http)
            .await;

        let snap = pool.snapshot().await;
        assert_eq!(snap.ip_effective_rate(), 0.0); // 都没验证过

        // 手动标记一个有效
        {
            let mut ips = pool.ip_resources.write().await;
            ips[0].effective = true;
        }

        let snap = pool.snapshot().await;
        assert!((snap.ip_effective_rate() - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_random_effective_ip() {
        let pool = NetworkResourcePool::new();
        pool.add_ip("10.0.0.1", 8080, "US", ProxyProtocol::Http)
            .await;
        pool.add_ip("10.0.0.2", 8080, "DE", ProxyProtocol::Http)
            .await;

        // 没有有效 IP
        assert!(pool.random_effective_ip().await.is_none());

        // 标记一个有效
        {
            let mut ips = pool.ip_resources.write().await;
            ips[0].effective = true;
        }

        assert!(pool.random_effective_ip().await.is_some());
    }

    #[test]
    fn test_default_dns_list() {
        let dns = default_public_dns();
        assert!(dns.len() >= 5);
        assert!(dns.iter().any(|(addr, _)| *addr == "1.1.1.1"));
    }

    #[test]
    fn test_ip_to_proxy_node() {
        let ip = IpResource {
            ip: "203.0.113.1".into(),
            port: 8080,
            geo: "US".into(),
            protocol: ProxyProtocol::Http,
            effective: true,
            latency_ms: 50.0,
            last_verified: None,
            success_count: 10,
            fail_count: 0,
        };
        let node = ip.to_proxy_node();
        assert_eq!(node.host, "203.0.113.1");
        assert_eq!(node.port, 8080);
        assert_eq!(node.geo_tag.as_deref(), Some("US"));
    }

    #[test]
    fn test_pool_snapshot_rates() {
        let snap = PoolSnapshot {
            dns_total: 10,
            dns_effective: 7,
            route_total: 5,
            route_effective: 3,
            ip_total: 20,
            ip_effective: 15,
            refresh_count: 5,
        };
        assert!((snap.dns_effective_rate() - 0.7).abs() < 0.01);
        assert!((snap.route_effective_rate() - 0.6).abs() < 0.01);
        assert!((snap.ip_effective_rate() - 0.75).abs() < 0.01);
    }
}
