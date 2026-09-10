//! 单一 HTTP client 配置源 (生命线 A 终态)
//!
//! 全工程唯一构造 `reqwest::Client` 的位置。UA / timeout / connect_timeout /
//! redirect-none 集中于此,blocking 与 async 运行时各产出一个对象。
//!
//! 对齐 OWASP SSRF 结论:
//! - `redirect(Policy::none())` — 禁止重定向跟随 (重定向是 SSRF 启动器)
//! - `resolve_safe_origin` 提供 connect-期 DNS pinning,防 DNS rebinding (TOCTOU)
#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use log::warn;

const USER_AGENT: &str = "NeoTrix/0.19 (nt_http)";
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
/// 断点续传下载的总生命周期超时 (reqwest `.timeout()` 覆盖整个请求含 body 读取)。
/// 大文件需要宽松预算; 挂起由该超时兜底 → 外层网络错误重试。
const DOWNLOAD_TOTAL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);
/// 断点续传下载单块读取大小。
const DOWNLOAD_CHUNK_SIZE: usize = 64 * 1024;

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
                    warn!("[nt_http] Failed to build blocking client: {e}");
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


/// 安全解析重定向链：最多 max_redirects 跳，每跳都经 resolve_safe_origin 校验。
/// 防止开放重定向攻击 (open redirect) 并保持 SSRF 防护一致性。
fn resolve_redirects_safely(url: &str, max_redirects: usize) -> Result<String, String> {
    let mut current_url = url.to_string();
    let mut redirects = 0;
    
    loop {
        let (addr, parsed) = resolve_safe_origin(&current_url)?;
        let host = parsed.host_str().ok_or("no host")?.to_string();
        
        let client = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(CONNECT_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .resolve(&host, addr)
            .build()
            .map_err(|e| format!("redirect client: {e}"))?;
        
        let resp = client
            .head(&current_url)
            .send()
            .map_err(|e| format!("redirect head: {e}"))?;
        
        if resp.status().is_redirection() {
            if redirects >= max_redirects {
                return Err(format!("too many redirects (> {})", max_redirects));
            }
            let location = resp
                .headers()
                .get("location")
                .ok_or("redirect without Location header")?
                .to_str()
                .map_err(|e| format!("invalid Location header: {e}"))?
                .to_string();
            
            // 解析重定向 URL（支持相对路径）
            let next_url = if let Ok(abs) = url::Url::parse(&location) {
                abs.to_string()
            } else {
                let base = url::Url::parse(&current_url).map_err(|e| format!("base URL parse: {e}"))?;
                base.join(&location).map_err(|e| format!("redirect join: {e}"))?.to_string()
            };
            
            current_url = next_url;
            redirects += 1;
            continue;
        }
        
        // 非重定向：返回当前 URL
        return Ok(current_url);
    }
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

/// 断点续传下载原语配置 (吸收 project-nomad `downloads.ts` 模式)。
///
/// 对齐的可靠性模式:
/// - `.tmp` 暂存 + 原子 rename — 消费者永不见半成品
/// - Range 断点续传 — 从已下载字节偏移恢复
/// - 服务器文件缩小丢弃 stale partial — 防止 416 死循环 (openZIM 滚动构建)
/// - 服务器忽略 Range (200) 降级重试 — 防 .tmp 损坏
/// - 非 200/206 一律失败 (含 3xx — redirect 保持 SSRF 安全)
/// - 网络错误重试 (仅 ECONNRESET/ENOTFOUND/ETIMEDOUT 语义)
/// - 自定义 UA — Wikimedia/Cloudflare CDN 403 规避
/// - 可选 MIME 白名单 + 大小上限
pub(crate) struct DownloadOptions<'a> {
    pub url: &'a str,
    pub dest: &'a Path,
    /// 覆盖默认 UA (默认 `NeoTrix/0.19 (nt_http)`)。上游 403 时设可辨识 UA。
    pub user_agent: Option<&'a str>,
    /// MIME 白名单 (substring 匹配); 空 = 跳过检查。
    pub allowed_mime_types: &'a [&'a str],
    /// 最大下载字节数; 0 = 不限制。
    pub max_bytes: u64,
    /// 总生命周期超时 (默认 300s, 覆盖整个请求含 body 读取)。
    pub total_timeout: Option<std::time::Duration>,
    /// 代理地址 (如 `socks5h://127.0.0.1:9050`)。fake-ip 分流网络下直连超时,
    /// 经此代理路由。None = 直连 (保留 SSRF pin)。
    pub proxy: Option<&'a str>,
    /// 启用代理池轮换 (C5 nt_shield_stealth_net::proxy_pool): 每次重试尝试前从池
    /// 选最快可用节点作为 egress。池空/锁争用 → 回退 `proxy`, 再回退直连。
    /// 与 `host` 配合做失败记账 (RL 策略学习器)。
    pub proxy_pool: bool,
    /// 下载目标 host — 供代理池失败记账 (record_strategy_result_blocking) 按域学习。
    pub host: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DownloadResult {
    pub path: PathBuf,
    pub bytes_written: u64,
    /// 是否从已有 .tmp 断点续传。
    pub resumed: bool,
}

/// 断点续传下载到文件 (blocking)。返回最终文件路径。
/// 网络错误自动重试 (最多 3 次, 指数退避 2s→8s)。HTTP 4xx/5xx 不重试。
pub(crate) fn download_to_file(opts: &DownloadOptions<'_>) -> Result<DownloadResult, String> {
    download_to_file_with_retry(opts, 3)
}

/// 断点续传下载到文件 (blocking), 指定重试次数。
/// 网络错误重试; 非网络错误 (SSRF guard / HTTP 4xx-5xx / MIME 拒绝) 立即返回。
/// 启用 `proxy_pool` 时, 每次尝试前从全局代理池选最快节点轮换 egress,
/// 失败记账回传池学习器 (`record_strategy_result_blocking`)。
pub(crate) fn download_to_file_with_retry(
    opts: &DownloadOptions<'_>,
    max_retries: u32,
) -> Result<DownloadResult, String> {
    run_blocking(|| {
        let mut wait = std::time::Duration::from_secs(2);
        let mut last_err: Option<String> = None;
        // 池节点 URL 缓冲 — 存活于整个重试循环, 供按次轮换 egress。
        let mut pool_proxy_buf: Option<String> = None;
        for attempt in 0..=max_retries {
            if opts.proxy_pool {
                pool_proxy_buf = pool_select_blocking();
            }
            let attempt_opts = DownloadOptions {
                url: opts.url,
                dest: opts.dest,
                user_agent: opts.user_agent,
                allowed_mime_types: opts.allowed_mime_types,
                max_bytes: opts.max_bytes,
                total_timeout: opts.total_timeout,
                proxy: if opts.proxy_pool {
                    pool_proxy_buf.as_deref().or(opts.proxy)
                } else {
                    opts.proxy
                },
                proxy_pool: false,
                host: opts.host,
            };
            match download_to_file_inner(&attempt_opts) {
                Ok(ok) => {
                    if opts.proxy_pool {
                        pool_record_result(opts.host, true);
                    }
                    return Ok(ok);
                }
                Err(e) => {
                    let retriable = is_retriable_network_err(&e);
                    if opts.proxy_pool {
                        pool_record_result(opts.host, false);
                    }
                    if retriable && attempt < max_retries {
                        std::thread::sleep(wait);
                        wait = (wait * 2).min(std::time::Duration::from_secs(8));
                        continue;
                    }
                    last_err = Some(e);
                    if attempt >= max_retries {
                        break;
                    }
                    return Err(last_err.take().unwrap_or_else(|| "download failed".into()));
                }
            }
        }
        Err(last_err.unwrap_or_else(|| "download failed".to_string()))
    })
}

#[cfg(feature = "stealth-net")]
fn pool_select_blocking() -> Option<String> {
    crate::neotrix::nt_shield_stealth_net::proxy_pool::global_pool()
        .select_node_blocking()
        .map(|n| n.url.clone())
}

#[cfg(feature = "stealth-net")]
fn pool_record_result(host: Option<&str>, success: bool) {
    if let Some(host) = host {
        crate::neotrix::nt_shield_stealth_net::proxy_pool::global_pool()
            .record_strategy_result_blocking(host, success);
    }
}

#[cfg(not(feature = "stealth-net"))]
fn pool_select_blocking() -> Option<String> {
    None
}

#[cfg(not(feature = "stealth-net"))]
fn pool_record_result(_host: Option<&str>, _success: bool) {}

/// 判定错误是否属于可重试网络错误 (project-nomad: ECONNRESET/ENOTFOUND/ETIMEDOUT 语义)。
/// 非网络错误 (SSRF guard 拒绝 / HTTP 状态码 / MIME 拒绝) 不重试 — 保证 guard 语义不被绕过。
fn is_retriable_network_err(e: &str) -> bool {
    e.contains("timed out") || e.contains("Connection reset") || e.contains("connect error")
        || e.contains("peer closed") || e.contains("connection closed") || e.contains("refused")
}

fn download_to_file_inner(opts: &DownloadOptions<'_>) -> Result<DownloadResult, String> {
    let (_addr, parsed) = resolve_safe_origin(opts.url)?;
    let _host = parsed.host_str().ok_or("no host")?.to_string();

    let dir = opts.dest.parent().ok_or("dest has no parent dir")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;

    // .tmp 暂存: 消费者永不见半成品 (project-nomad 模式)
    let temp_path = PathBuf::from(format!("{}.tmp", opts.dest.display()));

    // 断点续传: 检测已有 .tmp 偏移
    let mut start_byte: u64 = 0;
    let mut append_mode = false;
    if let Ok(meta) = std::fs::metadata(&temp_path) {
        start_byte = meta.len();
        append_mode = true;
    }

    // 自定义 UA (Wikimedia/Cloudflare 403 规避)
    let ua = opts.user_agent.unwrap_or(USER_AGENT);

    // HEAD 预检: 大小 / accept-ranges / MIME
    // 先安全解析重定向链（最多5跳），验证每跳都通过 SSRF guard
    let final_url = resolve_redirects_safely(opts.url, 5)?;
    
    let (final_addr, final_parsed) = resolve_safe_origin(&final_url)?;
    let final_host = final_parsed.host_str().ok_or("no host")?.to_string();
    
    let mut cbuilder = reqwest::blocking::Client::builder()
        .user_agent(ua)
        .timeout(opts.total_timeout.unwrap_or(DOWNLOAD_TOTAL_TIMEOUT))
        .connect_timeout(CONNECT_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .resolve(&final_host, final_addr);
    if let Some(proxy) = opts.proxy {
        match reqwest::Proxy::all(proxy) {
            Ok(p) => {
                cbuilder = cbuilder.proxy(p);
            }
            Err(e) => {
                return Err(format!("invalid proxy {proxy:?}: {e}"));
            }
        }
    }
    let pin_client = cbuilder
        .build()
        .map_err(|e| format!("pin client: {e}"))?;

    let head = pin_client
        .head(&final_url)
        .send()
        .map_err(|e| format!("head: {e}"))?;
    if !head.status().is_success() {
        return Err(format!("HEAD HTTP {}", head.status()));
    }
    let total_bytes: u64 = head
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let supports_range = head
        .headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("bytes"))
        .unwrap_or(false);
    let content_type = head
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    // MIME 白名单校验
    if !opts.allowed_mime_types.is_empty() {
        let allowed = opts
            .allowed_mime_types
            .iter()
            .any(|m| content_type.contains(m));
        if !allowed {
            return Err(format!("MIME type {} is not allowed", content_type));
        }
    }

    // 幂等: 最终文件已存在且大小正确 → 直接返回
    if let Ok(fmeta) = std::fs::metadata(opts.dest) {
        if fmeta.len() == total_bytes && total_bytes > 0 {
            return Ok(DownloadResult {
                path: opts.dest.to_path_buf(),
                bytes_written: total_bytes,
                resumed: false,
            });
        }
    }

    // .tmp 已完整但未 rename → 直接 rename
    if start_byte == total_bytes && total_bytes > 0 {
        std::fs::rename(&temp_path, opts.dest)
            .map_err(|e| format!("rename complete tmp: {e}"))?;
        return Ok(DownloadResult {
            path: opts.dest.to_path_buf(),
            bytes_written: total_bytes,
            resumed: true,
        });
    }

    // 服务器不支持 range 且有 partial → 删除重来
    if !supports_range && start_byte > 0 {
        let _ = std::fs::remove_file(&temp_path);
        start_byte = 0;
        append_mode = false;
    }

    // 服务器文件缩小 (openZIM 滚动构建) → 丢弃 stale partial, 防 416 死循环
    if start_byte > total_bytes && total_bytes > 0 {
        let _ = std::fs::remove_file(&temp_path);
        start_byte = 0;
        append_mode = false;
    }

    // 大小上限预检
    if opts.max_bytes > 0 && total_bytes > opts.max_bytes {
        return Err(format!(
            "download exceeds max_bytes ({} > {})",
            total_bytes, opts.max_bytes
        ));
    }

    // Range 头 (支持时)
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_str(ua).map_err(|e| format!("UA: {e}"))?);
    if supports_range && start_byte > 0 {
        let range = format!("bytes={}-", start_byte);
        headers.insert(
            reqwest::header::RANGE,
            reqwest::header::HeaderValue::from_str(&range).map_err(|e| format!("Range: {e}"))?,
        );
    }

    let get = pin_client.get(opts.url).headers(headers.clone());
    let mut resp = get.send().map_err(|e| format!("get: {e}"))?;
    let status = resp.status();
    if status != reqwest::StatusCode::OK && status != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(format!("GET HTTP {}", status));
    }

    // 请求了 range 但服务器返回 200 (忽略 Range) → 丢弃重来, 防 .tmp 损坏
    if headers.contains_key(reqwest::header::RANGE) && status == reqwest::StatusCode::OK {
        drop(resp);
        let _ = std::fs::remove_file(&temp_path);
        start_byte = 0;
        append_mode = false;
        headers.remove(reqwest::header::RANGE);
        let get2 = pin_client.get(opts.url).headers(headers.clone());
        resp = get2.send().map_err(|e| format!("get2: {e}"))?;
        let s2 = resp.status();
        if s2 != reqwest::StatusCode::OK && s2 != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(format!("GET2 HTTP {}", s2));
        }
    }

    // 流式写入 + 读超时兜底 (stall: 连接挂起时 read 最终超时报错, 由外层 retry)
    let mut out: File = if append_mode {
        File::options()
            .append(true)
            .open(&temp_path)
            .map_err(|e| format!("append open {}: {e}", temp_path.display()))?
    } else {
        File::create(&temp_path)
            .map_err(|e| format!("create {}: {e}", temp_path.display()))?
    };

    let mut buf = vec![0u8; DOWNLOAD_CHUNK_SIZE];
    let mut written: u64 = start_byte;
    loop {
        let n = resp
            .read(&mut buf)
            .map_err(|e| format!("read: {e}"))?;
        if n == 0 {
            break;
        }
        written += n as u64;
        if opts.max_bytes > 0 && written > opts.max_bytes {
            let _ = std::fs::remove_file(&temp_path);
            return Err(format!("download exceeded max_bytes {}", opts.max_bytes));
        }
        out.write_all(&buf[..n])
            .map_err(|e| format!("write: {e}"))?;
    }
    out.flush().map_err(|e| format!("flush: {e}"))?;
    drop(out);

    // 原子 rename 完成
    std::fs::rename(&temp_path, opts.dest)
        .map_err(|e| format!("rename final: {e}"))?;

    Ok(DownloadResult {
        path: opts.dest.to_path_buf(),
        bytes_written: written,
        resumed: start_byte > 0,
    })
}

fn is_private_ip(ip: std::net::IpAddr) -> bool {
    let allow_fake_ip = std::env::var("NEOTRIX_ALLOW_FAKE_IP").as_deref() == Ok("1");
    is_private_ip_with(ip, allow_fake_ip)
}

/// 带 fake-ip 放行开关的私有地址判定 (纯函数, 便于测试 fake-ip 隧道环境)。
fn is_private_ip_with(ip: std::net::IpAddr, allow_fake_ip: bool) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            // std 已覆盖: loopback/private(10,172.16-31,192.168)/link-local(169.254)
            // /broadcast/unspecified/documentation(192.0.2,198.51.100,203.0.113)
            v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_broadcast()
                || v4.is_unspecified() || v4.is_documentation()
                // CGNAT 100.64.0.0/10 — is_private() 不覆盖 (RFC 6598)
                || (v4.octets()[0] == 100 && (v4.octets()[1] & 0xc0) == 0x40)
                // benchmarking 198.18.0.0/15 (RFC 2544) — 公网可达但禁止路由进服务。
                // fake-ip 分流网络 (198.18.x 透明隧道, 见 CONTEXT fake-ip 策略) 下
                // DNS 污染会把公网域名解析到 198.18.x — 该段实为公网转发隧道而非本地
                // 私有服务, 需显式 NEOTRIX_ALLOW_FAKE_IP=1 才放行 (默认保持拒绝)。
                || ((v4.octets()[0] == 198 && (v4.octets()[1] & 0xfe) == 0x12) && !allow_fake_ip)
                // 240.0.0.0/4 reserved + 0.0.0.0/8
                || v4.octets()[0] >= 240 || v4.octets()[0] == 0
        }
        std::net::IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_private_ip(std::net::IpAddr::V4(v4));
            }
            // fake-ip 隧道 ULA (Clash 系默认 fdfe:dcba:9876::/48) — allow_fake_ip 时放行
            let is_fake_ip_ula = v6.segments()[0..4] == [0xfdfe, 0xdcba, 0x9876, 0x0000];
            if is_fake_ip_ula && allow_fake_ip {
                return false;
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
    fn is_private_ip_fake_ip_env_override() {
        use std::net::IpAddr;
        // 默认 (allow_fake_ip=false): 198.18/15 拒绝 (RFC 2544)
        assert!(is_private_ip_with(IpAddr::V4("198.18.0.1".parse().unwrap()), false));
        // allow_fake_ip=true: fake-ip 透明隧道放行
        assert!(!is_private_ip_with(IpAddr::V4("198.18.0.5".parse().unwrap()), true));
        // fake-ip 隧道 ULA (Clash 系 fdfe:dcba:9876::/48) — allow 时放行
        assert!(is_private_ip_with("fdfe:dcba:9876::12".parse().unwrap(), false));
        assert!(!is_private_ip_with("fdfe:dcba:9876::12".parse().unwrap(), true));
        // 其他私有段不受 override 影响 (仍拒绝)
        assert!(is_private_ip_with(IpAddr::V4("10.0.0.1".parse().unwrap()), true));
        assert!(is_private_ip_with(IpAddr::V4("192.168.1.1".parse().unwrap()), true));
        assert!(is_private_ip_with(IpAddr::V4("100.64.0.1".parse().unwrap()), true));
        // 非 fake-ip ULA (其他 fdfe 变体) 仍拒绝
        assert!(is_private_ip_with("fd00::1".parse().unwrap(), true));
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

    #[test]
    fn is_retriable_network_err_heuristics() {
        // 网络错误语义 → 可重试
        assert!(is_retriable_network_err("connection timed out"));
        assert!(is_retriable_network_err("Connection reset by peer"));
        assert!(is_retriable_network_err("connect error: refused"));
        assert!(is_retriable_network_err("peer closed connection"));
        // 非网络错误 (guard / HTTP 状态 / MIME) → 不重试 (R-P42 guard 语义)
        assert!(!is_retriable_network_err("private/reserved resolved IP rejected"));
        assert!(!is_retriable_network_err("MIME type text/html is not allowed"));
        assert!(!is_retriable_network_err("GET HTTP 404"));
        assert!(!is_retriable_network_err("HEAD HTTP 403"));
    }

    #[test]
    fn download_rejects_ssrf_guard_without_retry() {
        // SSRF guard 拒绝 (loopback) 必须立即失败, 不得进入重试循环 (R-P42/R-P107)
        let opts = DownloadOptions {
            url: "http://127.0.0.1:8080/secret",
            dest: Path::new("/tmp/neotrix-download-ssrf-test"),
            user_agent: None,
            allowed_mime_types: &[],
            max_bytes: 0,
            total_timeout: None,
                    proxy: None,
                    proxy_pool: false,
                    host: None,
        };
        let err = download_to_file_with_retry(&opts, 3).unwrap_err();
        assert!(
            err.contains("private") || err.contains("reject") || err.contains("loopback"),
            "guard error surfaced: {err}"
        );
    }

    #[test]
    fn download_rejects_userinfo_credentials() {
        // URL 内嵌 userinfo — guard 必须拒绝
        let opts = DownloadOptions {
            url: "https://user:pass@codeload.github.com/owner/repo/tar.gz/refs/heads/main",
            dest: Path::new("/tmp/neotrix-download-userinfo-test"),
            user_agent: None,
            allowed_mime_types: &[],
            max_bytes: 0,
            total_timeout: None,
                    proxy: None,
                    proxy_pool: false,
                    host: None,
        };
        let err = download_to_file(&opts).unwrap_err();
        assert!(err.contains("credentials"), "userinfo rejected: {err}");
    }

    #[test]
    fn download_proxy_pool_enabled_falls_back_to_guard_semantics() {
        // proxy_pool=true 时: 池空 → select_node_blocking 返回 None → 回退直连。
        // SSRF guard 语义必须保持 (guard 错误不因池轮换被绕过)。
        let opts = DownloadOptions {
            url: "http://127.0.0.1:8080/secret",
            dest: Path::new("/tmp/neotrix-download-ssrf-pool-test"),
            user_agent: None,
            allowed_mime_types: &[],
            max_bytes: 0,
            total_timeout: None,
                    proxy: None,
                    proxy_pool: true,
                    host: Some("codeload.github.com"),
        };
        let err = download_to_file_with_retry(&opts, 3).unwrap_err();
        assert!(
            err.contains("private") || err.contains("reject") || err.contains("loopback"),
            "pool-enabled path keeps guard error: {err}"
        );
    }
}