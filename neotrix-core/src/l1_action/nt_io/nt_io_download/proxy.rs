/// 代理配置模块
/// 支持 SOCKS5 和 HTTP 代理，集成到 reqwest 客户端

use reqwest::Proxy;
use std::net::SocketAddr;

/// 代理类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyType {
    Socks5,   // SOCKS5 (如 Clash Verge)
    Socks4,   // SOCKS4
    Http,     // HTTP 代理
    Https,    // HTTPS 代理
}

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 代理类型
    pub proxy_type: ProxyType,
    /// 代理地址 (主机:端口 或 IP:端口)
    pub address: String,
    /// 用户名 (如有认证)
    pub username: Option<String>,
    /// 密码 (如有认证)
    pub password: Option<String>,
}

impl ProxyConfig {
    /// 创建新的代理配置
    pub fn new(proxy_type: ProxyType, address: String) -> Self {
        ProxyConfig {
            proxy_type,
            address,
            username: None,
            password: None,
        }
    }

    /// 将配置转换为 reqwest Proxy
    pub fn to_reqwest(&self) -> Result<Proxy, String> {
        let proxy_url = match self.proxy_type {
            ProxyType::Socks5 => format!("socks5://{}", self.address),
            ProxyType::Socks4 => format!("socks4://{}", self.address),
            ProxyType::Http => format!("http://{}", self.address),
            ProxyType::Https => format!("https://{}", self.address),
        };

        Proxy::all(proxy_url.as_str()).map_err(|e| format!("Invalid proxy URL: {}", e))
    }

    /// 检查代理是否有效
    pub fn is_valid(&self) -> bool {
        !self.address.is_empty() && matches!(self.proxy_type, ProxyType::Socks5 | ProxyType::Http)
    }
}

/// 从环境变量或配置创建代理
pub fn from_env() -> Option<ProxyConfig> {
    // 检查常见环境变量
    let http_proxy = std::env::var("HTTP_PROXY").ok()
        .or_else(|| std::env::var("http_proxy").ok());
    let https_proxy = std::env::var("HTTPS_PROXY").ok()
        .or_else(|| std::env::var("https_proxy").ok());
    let socks_proxy = std::env::var("SOCKS5_PROXY").ok()
        .or_else(|| std::env::var("socks5_proxy").ok());

    // 优先 HTTPS_PROXY，其次 HTTP_PROXY，最后 SOCKS5
    if let Some(url) = https_proxy {
        // 简单解析，实际使用中可引入 url 解析库
        Some(ProxyConfig::new(ProxyType::Http, url))
    } else if let Some(url) = http_proxy {
        Some(ProxyConfig::new(ProxyType::Http, url))
    } else if let Some(url) = socks_proxy {
        // 判断是 socks5 还是 socks4
        if url.to_lowercase().contains("socks5") {
            Some(ProxyConfig::new(ProxyType::Socks5, url.split("://").nth(1).unwrap_or("").to_string()))
        } else {
            Some(ProxyConfig::new(ProxyType::Socks4, url.split("://").nth(1).unwrap_or("").to_string()))
        }
    } else {
        None
    }
}