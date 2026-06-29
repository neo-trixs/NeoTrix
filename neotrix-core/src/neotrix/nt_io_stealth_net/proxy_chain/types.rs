//! 多跳代理链 — 全节点动态轮转 (每15秒)
//!
//! 对标开源项目:
//! - **ProxyChains-NG**: 多跳代理链协议
//! - **Obscura**: 代理池与健康检测
//! - **V2Ray/Xray**: 多协议代理路由
//!
//! 核心能力:
//! - 多协议代理节点 (HTTP/HTTPS/SOCKS4/SOCKS5)
//! - 可配置多跳链 (entry → middle → exit)
//! - **每15秒动态轮转所有节点**
//! - 健康检测 (延迟/成功率)
//! - 故障自动跳转下一节点
//! - Geo 标签 (区域感知路由)

use std::time::Instant;

use rand::Rng;

pub(crate) const DEFAULT_ROTATION_INTERVAL_SECS: u64 = 9;
pub(crate) const CONNECT_TIMEOUT_SECS: u64 = 5;
pub(crate) const PROBE_INTERVAL_MS: u64 = 500;
pub(crate) const QUICK_PROBE_TIMEOUT_MS: u64 = 500;
pub(crate) const FAILOVER_THRESHOLD_SUCCESS_RATE: f64 = 0.3;
pub(crate) const FAILOVER_LATENCY_THRESHOLD_MS: f64 = 3000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum ProxyProtocol {
    Http,
    Https,
    Socks4,
    Socks5,
    /// CDN Relay: 通过 Vercel/Netlify/Cloudflare Workers 中继 (Client↔CDN↔Server)
    CdnRelay,
}

impl ProxyProtocol {
    pub fn as_url_scheme(&self) -> &'static str {
        match self {
            ProxyProtocol::Http => "http",
            ProxyProtocol::Https => "https",
            ProxyProtocol::Socks4 => "socks4",
            ProxyProtocol::Socks5 => "socks5",
            ProxyProtocol::CdnRelay => "https+relay",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            ProxyProtocol::Http => 8080,
            ProxyProtocol::Https => 8443,
            ProxyProtocol::Socks4 => 1080,
            ProxyProtocol::Socks5 => 1080,
            ProxyProtocol::CdnRelay => 443,
        }
    }

    /// CDN Relay 头部注入: 将真实目标地址编码到请求头中
    pub fn relay_headers(&self, host: &str) -> Vec<(&'static str, String)> {
        if matches!(self, ProxyProtocol::CdnRelay) {
            vec![
                ("X-Relay-Target", host.to_string()),
                ("X-Relay-Protocol", "https".to_string()),
            ]
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyNode {
    pub protocol: ProxyProtocol,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub geo_tag: Option<String>,
    pub label: String,
    pub weight: f64,
}

impl ProxyNode {
    pub fn new(protocol: ProxyProtocol, host: &str, port: u16) -> Self {
        Self {
            protocol,
            host: host.to_string(),
            port,
            username: None,
            password: None,
            geo_tag: None,
            label: format!("{}://{}:{}", protocol.as_url_scheme(), host, port),
            weight: 1.0,
        }
    }

    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }

    pub fn with_geo(mut self, tag: &str) -> Self {
        self.geo_tag = Some(tag.to_string());
        self
    }

    /// 安全 URL — 不包含认证信息（用于日志、展示、健康检查）
    /// 对标 OWASP ASVS V3.4: 凭证不出现于日志/错误信息
    pub fn display_url(&self) -> String {
        format!(
            "{}://{}:{}",
            self.protocol.as_url_scheme(),
            self.host,
            self.port
        )
    }

    /// 完整 URL — 包含认证信息（仅用于 reqwest::Proxy::all 内部传输）
    pub fn secret_url(&self) -> String {
        let scheme = self.protocol.as_url_scheme();
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            format!("{}://{}:{}@{}:{}", scheme, user, pass, self.host, self.port)
        } else {
            self.display_url()
        }
    }
}

/// 代理节点健康状态
#[derive(Debug, Clone)]
pub struct ProxyHealth {
    pub node_label: String,
    pub last_check: Option<Instant>,
    pub success_count: u64,
    pub fail_count: u64,
    pub avg_latency_ms: f64,
}

impl ProxyHealth {
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.fail_count;
        if total == 0 {
            0.5
        } else {
            self.success_count as f64 / total as f64
        }
    }

    pub fn is_healthy(&self, min_success_rate: f64) -> bool {
        self.success_rate() >= min_success_rate
    }
}

/// 代理选择策略 (动态变化)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProxySelectionRule {
    /// 完全随机
    Random,
    /// 按权重选择 (weight 越高概率越大)
    Weighted,
    /// 按最低延迟 (选最近的)
    LowestLatency,
    /// 按最高成功率
    HighestSuccess,
    /// Geo 轮转 (每步换区域)
    GeoRoundRobin,
}

impl ProxySelectionRule {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..5) {
            0 => ProxySelectionRule::Random,
            1 => ProxySelectionRule::Weighted,
            2 => ProxySelectionRule::LowestLatency,
            3 => ProxySelectionRule::HighestSuccess,
            _ => ProxySelectionRule::GeoRoundRobin,
        }
    }
}
