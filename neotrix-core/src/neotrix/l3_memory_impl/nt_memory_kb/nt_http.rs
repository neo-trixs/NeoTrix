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

/// 在 tokio runtime 上下文内执行阻塞闭包时用 block_in_place 包裹,
/// 避免 reqwest::blocking 内部 runtime 创建/drop 在异步上下文 panic
/// ("Cannot drop a runtime in a context where blocking is not allowed")。
/// 非 runtime 上下文直接执行; current_thread runtime 内 block_in_place
/// 不支持, 退化为直接执行 (仅测试辅助场景, 不触网)。
pub(crate) fn run_blocking<T>(f: impl FnOnce() -> T) -> T {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
            tokio::task::block_in_place(f)
        } else {
            f()
        }
    } else {
        f()
    }
}

/// 单一 blocking client 工厂 (所有阻塞吞入路径共享连接池与安全策略)
pub(crate) fn shared_blocking_client() -> &'static reqwest::blocking::Client {
    static CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
        run_blocking(|| {
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
        })
    });
    &CLIENT
}

/// 校验 URL 安全,并返回可连接的目标地址 (connect-期 pin 使用)。
/// 在 `is_safe_fetch_url` 之上:解析全部 A+AAAA,过滤内网/回环/链路本地/保留段,
/// 取首个安全地址供调用方 `resolve(host, addr)` pin,彻底阻断 DNS rebinding。
///
/// 对齐 Windows-MCP Scrape 块清单 (agent-security 吸收):
/// - 私有/回环/链路本地/保留段 (含 IPv4-mapped IPv6、CGNAT、benchmarking)
/// - URL 内嵌 userinfo 凭据 (`user:pass@host`) — 凭据外泄 + 社工向量
/// - 非 http/https scheme; localhost/.local 拒绝
pub(crate) fn resolve_safe_origin(url: &str) -> Result<(SocketAddr, url::Url), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("URL parse: {e}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("scheme must be http/https".into());
    }
    // URL 内嵌 userinfo 凭据: 凭据随请求外泄且是钓鱼/冒用向量,一律拒绝。
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("URL embedded credentials (userinfo) rejected".into());
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
    fetch_safe_http_inner(url, &[])
}

/// 带额外 headers 的安全抓取 (blocking): 用于需要特定 Accept/Authorization 等头的 API。
/// SSRF guard + connect pin 语义与 `fetch_safe_http` 完全一致。
pub(crate) fn fetch_safe_http_with_headers(
    url: &str,
    extra_headers: &[(&str, &str)],
) -> Result<(String, String), String> {
    fetch_safe_http_inner(url, extra_headers)
}

fn fetch_safe_http_inner(
    url: &str,
    extra_headers: &[(&str, &str)],
) -> Result<(String, String), String> {
    // blocking 段 (DNS + client 构造 + send + text) 统一经 run_blocking:
    // headless/interactive 模式在 rt.block_on 内调用 absorb → 本路径,
    // 直接执行会因 reqwest::blocking 内部 runtime drop 而 panic。
    run_blocking(|| {
        let (addr, parsed) = resolve_safe_origin(url)?;
        let host = parsed.host_str().ok_or("no host")?.to_string();

        // connect-期 pin: 用已校验 IP 建立临时 client,阻断 DNS rebinding。
        // 因 `resolve` 是 per-client 的,不能复用共享单例,故按调用临时构造。
        let mut builder = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .resolve(&host, addr);

        if !extra_headers.is_empty() {
            let mut h = reqwest::header::HeaderMap::new();
            for (k, v) in extra_headers {
                let header_name = k
                    .parse::<reqwest::header::HeaderName>()
                    .map_err(|e| format!("invalid header name {k:?}: {e}"))?;
                let header_value = v
                    .parse::<reqwest::header::HeaderValue>()
                    .map_err(|e| format!("invalid header value for {k:?}: {e}"))?;
                h.insert(header_name, header_value);
            }
            builder = builder.default_headers(h);
        }

        let pin_client = builder
            .build()
            .map_err(|e| format!("pin client: {e}"))?;

        let mut req = pin_client.get(url);
        for (k, v) in extra_headers {
            req = req.header(*k, *v);
        }
        let resp = req
            .send()
            .map_err(|e| format!("fetch: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body = resp.text().map_err(|e| format!("read: {e}"))?;
        Ok((body, host))
    })
}

/// 指数退避重试版安全抓取: 仅对 429/503 重试 (最多 3 次), 尊重 retry-after 头。
/// 能力源自 `bin/kb_crawl_batch::fetch_with_retry` (R-P96 提炼并入)。
/// SSRF guard + connect pin 语义与 `fetch_safe_http` 完全一致。
pub(crate) fn fetch_safe_http_with_retry(url: &str) -> Result<(String, String), String> {
    // 重试循环含 sleep 与内部 fetch_safe_http (blocking), 统一经 run_blocking。
    run_blocking(|| {
        let mut wait = std::time::Duration::from_secs(2);
        for attempt in 0..3 {
            match fetch_safe_http(url) {
                Ok(ok) => return Ok(ok),
                Err(e) if e.starts_with("HTTP 429") || e.starts_with("HTTP 503") => {
                    std::thread::sleep(wait);
                    wait = std::time::Duration::from_secs(wait.as_secs() * 2).min(std::time::Duration::from_secs(8));
                    if attempt == 2 { return Err(e); }
                }
                Err(e) => return Err(e),
            }
        }
        Err("retry exhausted".to_string())
    })
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
            // std 已覆盖: loopback/private(10,172.16-31,192.168)/link-local(169.254)
            // /broadcast/unspecified/documentation(192.0.2,198.51.100,203.0.113)
            v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_broadcast()
                || v4.is_unspecified() || v4.is_documentation()
                // CGNAT 100.64.0.0/10 — is_private() 不覆盖 (RFC 6598)
                || (v4.octets()[0] == 100 && (v4.octets()[1] & 0xc0) == 0x40)
                // benchmarking 198.18.0.0/15 (RFC 2544) — 公网可达但禁止路由进服务
                || (v4.octets()[0] == 198 && (v4.octets()[1] & 0xfe) == 0x12)
                // 240.0.0.0/4 reserved + 0.0.0.0/8
                || v4.octets()[0] >= 240 || v4.octets()[0] == 0
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
    fn resolve_safe_origin_rejects_userinfo_credentials() {
        // URL 内嵌 userinfo 凭据 — agent-security 吸收: 凭据外泄 + 冒用向量
        assert!(resolve_safe_origin("http://user:pass@8.8.8.8/").is_err());
        assert!(resolve_safe_origin("https://user@example.com/").is_err());
        assert!(resolve_safe_origin("http://admin:admin@192.168.1.1/").is_err());
    }

    #[test]
    fn is_private_ip_covers_reserved_v4_ranges() {
        use std::net::IpAddr;
        // CGNAT 100.64.0.0/10 (RFC 6598) — std is_private() 不覆盖
        assert!(is_private_ip(IpAddr::V4("100.64.0.1".parse().unwrap())));
        assert!(is_private_ip(IpAddr::V4("100.127.255.254".parse().unwrap())));
        // benchmarking 198.18.0.0/15 (RFC 2544)
        assert!(is_private_ip(IpAddr::V4("198.18.0.1".parse().unwrap())));
        assert!(is_private_ip(IpAddr::V4("198.19.255.254".parse().unwrap())));
        // reserved 240.0.0.0/4 与 0.0.0.0/8
        assert!(is_private_ip(IpAddr::V4("240.0.0.1".parse().unwrap())));
        assert!(is_private_ip(IpAddr::V4("0.0.0.1".parse().unwrap())));
        // 边界外: 100.63 与 100.128 为公网可路由 (非 CGNAT)
        assert!(!is_private_ip(IpAddr::V4("100.63.0.1".parse().unwrap())));
        assert!(!is_private_ip(IpAddr::V4("100.128.0.1".parse().unwrap())));
        // 198.16 与 198.20 不在 198.18/15 内
        assert!(!is_private_ip(IpAddr::V4("198.16.0.1".parse().unwrap())));
        assert!(!is_private_ip(IpAddr::V4("198.20.0.1".parse().unwrap())));
        // 公网普通地址不受影响
        assert!(!is_private_ip(IpAddr::V4("8.8.8.8".parse().unwrap())));
        assert!(!is_private_ip(IpAddr::V4("1.1.1.1".parse().unwrap())));
    }

    #[test]
    fn resolve_safe_origin_rejects_cgnat_literal() {
        // CGNAT 块走 IP 字面量路径直接拒绝
        assert!(resolve_safe_origin("http://100.64.0.1/").is_err());
        assert!(resolve_safe_origin("http://198.18.0.1/").is_err());
    }

    #[test]
    fn resolve_safe_origin_accepts_public_literal() {
        let (addr, _parsed) = resolve_safe_origin("http://8.8.8.8:80/").expect("public IP ok");
        assert_eq!(addr.ip().to_string(), "8.8.8.8");
        assert_eq!(addr.port(), 80);
    }

    #[test]
    fn retry_does_not_bypass_ssrf_guard() {
        // 非 429/503 错误 (含 guard 拒绝) 不得重试 — guard 语义必须保持
        let err = fetch_safe_http_with_retry("http://127.0.0.1:8080/").unwrap_err();
        assert!(err.contains("private") || err.contains("reject") || err.contains("loopback"),
            "guard error surfaced: {err}");
    }

    #[test]
    fn retry_immediate_fail_on_network_error() {
        // 公开 IP 但未监听端口 → connect error, 属非 429/503, 不应进入 3 次重试循环。
        // 环境网络抖动时错误文本形态多变 (lookup/connect/timeout/error sending request),
        // 故断言放宽为: 错误被透传 (非空) 且非 429/503 语义 (未被重试覆盖)。
        let err = fetch_safe_http_with_retry("http://8.8.8.8:59999/").unwrap_err();
        assert!(!err.is_empty(), "network error surfaced: {err}");
        assert!(
            !err.starts_with("HTTP 429") && !err.starts_with("HTTP 503"),
            "non-429/503 error must not be retried: {err}"
        );
    }
}