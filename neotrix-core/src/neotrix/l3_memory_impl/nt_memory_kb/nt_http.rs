//! 单一 HTTP client 配置源 (生命线 A 终态)
//!
//! 全工程唯一构造 `reqwest::Client` 的位置。UA / timeout / connect_timeout /
//! redirect-none 集中于此,blocking 与 async 运行时各产出一个对象。
//!
//! 对齐 OWASP SSRF 结论:
//! - `redirect(Policy::none())` — 禁止重定向跟随 (重定向是 SSRF 启动器)
//! - `resolve_safe_origin` 提供 connect-期 DNS pinning,防 DNS rebinding (TOCTOU)
#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::sync::LazyLock;

const USER_AGENT: &str = "NeoTrix/0.19 (nt_http)";
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// 单一 blocking client 工厂 (所有阻塞吞入路径共享连接池与安全策略)
pub(crate) fn shared_blocking_client() -> &'static reqwest::blocking::Client {
    static CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
        reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_else(|e| {
                eprintln!("[nt_http] WARNING: Failed to build blocking client: {e}");
                reqwest::blocking::Client::new()
            })
    });
    &CLIENT
}

/// 校验 URL 安全,并返回可连接的目标地址 (connect-期 pin 使用)。
/// 在 `is_safe_fetch_url` 之上:解析全部 A+AAAA,过滤内网/回环/链路本地/保留段,
/// 取首个安全地址供调用方 `resolve(host, addr)` pin,彻底阻断 DNS rebinding。
pub(crate) fn resolve_safe_origin(url: &str) -> Result<(SocketAddr, url::Url), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("URL parse: {e}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("scheme must be http/https".into());
    }
    let host = parsed.host_str().ok_or("no host")?.to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return Err("localhost/.local rejected".into());
    }
    let port = parsed.port_or_known_default().ok_or("no port")?;

    // IP 字面量直接校验
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if is_private_ip(ip) {
            return Err("private/reserved IP rejected".into());
        }
        return Ok((SocketAddr::new(ip, port), parsed));
    }

    // 域名: 解析全部,任一私有即拒绝,取首个安全地址
    let addrs: Vec<SocketAddr> = std::net::ToSocketAddrs::to_socket_addrs(&(host.clone(), port))
        .map_err(|e| format!("DNS resolve: {e}"))?
        .collect();
    if addrs.is_empty() {
        return Err("DNS resolve: empty".into());
    }
    for sa in &addrs {
        if is_private_ip(sa.ip()) {
            return Err("private/reserved resolved IP rejected".into());
        }
    }
    Ok((addrs[0], parsed))
}

/// 单一「安全抓取」原语 (blocking): guard → pin → fetch → (body, final_host)。
/// 所有阻塞吞入路径统一委托此处。
pub(crate) fn fetch_safe_http(url: &str) -> Result<(String, String), String> {
    let (addr, parsed) = resolve_safe_origin(url)?;
    let host = parsed.host_str().ok_or("no host")?.to_string();

    // connect-期 pin: 用已校验 IP 建立临时 client,阻断 DNS rebinding。
    // 因 `resolve` 是 per-client 的,不能复用共享单例,故按调用临时构造。
    let pin_client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .resolve(&host, addr)
        .build()
        .map_err(|e| format!("pin client: {e}"))?;

    let resp = pin_client
        .get(url)
        .send()
        .map_err(|e| format!("fetch: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let body = resp.text().map_err(|e| format!("read: {e}"))?;
    Ok((body, host))
}

/// 单一「安全抓取」原语 (async): guard → pin → fetch → (body, final_host)。
/// 所有异步吞入路径统一委托此处。
pub(crate) async fn fetch_safe_http_async(url: &str) -> Result<(String, String), String> {
    let (addr, parsed) = resolve_safe_origin(url)?;
    let host = parsed.host_str().ok_or("no host")?.to_string();

    let pin_client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .resolve(&host, addr)
        .build()
        .map_err(|e| format!("pin client: {e}"))?;

    let resp = tokio::time::timeout(TIMEOUT, pin_client.get(url).send())
        .await
        .map_err(|_| "fetch timed out".to_string())?
        .map_err(|e| format!("fetch: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let body = tokio::time::timeout(TIMEOUT, resp.text())
        .await
        .map_err(|_| "read timed out".to_string())?
        .map_err(|e| format!("read: {e}"))?;
    Ok((body, host))
}

fn is_private_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_broadcast()
                || v4.is_unspecified() || v4.is_documentation()
        }
        std::net::IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_private_ip(std::net::IpAddr::V4(v4));
            }
            v6.is_loopback() || v6.is_unspecified() || v6.is_unique_local()
                || v6.is_unicast_link_local() || v6.is_multicast()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_safe_origin_rejects_loopback() {
        assert!(resolve_safe_origin("http://127.0.0.1:8080/").is_err());
        assert!(resolve_safe_origin("http://localhost/").is_err());
        assert!(resolve_safe_origin("http://[::1]/").is_err());
        assert!(resolve_safe_origin("http://10.0.0.1/").is_err());
        assert!(resolve_safe_origin("http://192.168.1.1/").is_err());
        assert!(resolve_safe_origin("http://[::ffff:127.0.0.1]/").is_err());
        assert!(resolve_safe_origin("http://169.254.169.254/latest/meta-data/").is_err());
    }

    #[test]
    fn resolve_safe_origin_rejects_bad_scheme() {
        assert!(resolve_safe_origin("ftp://example.com/x").is_err());
        assert!(resolve_safe_origin("file:///etc/passwd").is_err());
        assert!(resolve_safe_origin("").is_err());
        assert!(resolve_safe_origin("not a url").is_err());
    }

    #[test]
    fn resolve_safe_origin_accepts_public_literal() {
        let (addr, _parsed) = resolve_safe_origin("http://8.8.8.8:80/").expect("public IP ok");
        assert_eq!(addr.ip().to_string(), "8.8.8.8");
        assert_eq!(addr.port(), 80);
    }
}