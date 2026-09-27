//! fetch — 从 `nt_io_browser_engine.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::time::Duration;

use reqwest::Client;
use scraper::{Html, Selector};
use super::cookies::CookieJar;
use super::{MAX_FIELDS_PER_FORM, MAX_FORMS, MAX_LINKS, MAX_RAW_HTML_CHARS, MAX_TEXT_CHARS, STRIP_RE, UA_POOL, WS_RE};
use super::error::BrowserError;
use super::policy::parse_retry_after_secs;
use super::session::BrowserConfig;
use super::types::{FormField, FormSpec, LinkRef, PageSnapshot};

pub(crate) fn pick_ua(config_ua: Option<&str>) -> String {
    if let Some(ua) = config_ua {
        return ua.to_string();
    }
    use rand::Rng;
    let mut rng = rand::thread_rng();
    UA_POOL[rng.gen_range(0..UA_POOL.len())].to_string()
}

pub(crate) fn select_first(doc: &Html, css: &str) -> Option<String> {
    let sel = Selector::parse(css).ok()?;
    doc.select(&sel).next().map(|el| {
        el.text().collect::<Vec<_>>().join(" ").trim().to_string()
    })
}

/// 从 HTML 提取结构化快照（纯函数）
pub(crate) fn parse_page(
    base_url: &str,
    body: &str,
) -> Result<(Option<String>, String, Vec<LinkRef>, Vec<FormSpec>, String), BrowserError> {
    let cleaned = STRIP_RE.replace_all(body, " ");
    let collapsed = WS_RE.replace_all(&cleaned, " ").to_string();
    let doc = Html::parse_document(&collapsed);
    let base = url::Url::parse(base_url)
        .map_err(|e| BrowserError::ActionFailed(format!("bad base url: {e}")))?;

    let title = select_first(&doc, "title").filter(|t| !t.is_empty());

    let text = select_first(&doc, "body")
        .unwrap_or_else(|| {
            doc.root_element()
                .text()
                .collect::<Vec<_>>()
                .join(" ")
        });
    let text: String = WS_RE
        .replace_all(text.trim(), " ")
        .chars()
        .take(MAX_TEXT_CHARS)
        .collect();

    let mut links = Vec::new();
    if let Ok(a_sel) = Selector::parse("a[href]") {
        for el in doc.select(&a_sel).take(MAX_LINKS * 2) {
            if links.len() >= MAX_LINKS {
                break;
            }
            let href = el.value().attr("href").unwrap_or("").trim();
            if href.is_empty()
                || href.starts_with('#')
                || href.starts_with("javascript:")
                || href.starts_with("mailto:")
                || href.starts_with("tel:")
                || href.starts_with("data:")
            {
                continue;
            }
            let abs = match base.join(href) {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            };
            let label = el
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .chars()
                .take(120)
                .collect::<String>();
            links.push(LinkRef {
                index: links.len(),
                text: if label.is_empty() {
                    abs.clone()
                } else {
                    label
                },
                href: abs,
            });
        }
    }

    let mut forms = Vec::new();
    if let Ok(form_sel) = Selector::parse("form") {
        for (fi, form) in doc.select(&form_sel).enumerate().take(MAX_FORMS) {
            let action = form
                .value()
                .attr("action")
                .unwrap_or("")
                .trim()
                .to_string();
            let action = if action.is_empty() {
                base_url.to_string()
            } else {
                base.join(&action)
                    .map(|u| u.to_string())
                    .unwrap_or_else(|_| base_url.to_string())
            };
            let method = form
                .value()
                .attr("method")
                .unwrap_or("get")
                .trim()
                .to_uppercase();
            let mut fields = Vec::new();
            if let Ok(field_sel) =
                Selector::parse("input[name], select[name], textarea[name]")
            {
                for el in form.select(&field_sel).take(MAX_FIELDS_PER_FORM * 2) {
                    if fields.len() >= MAX_FIELDS_PER_FORM {
                        break;
                    }
                    let tag = el.value().name();
                    let name = el.value().attr("name").unwrap_or("").to_string();
                    if name.is_empty() {
                        continue;
                    }
                    let (kind, value) = match tag {
                        "input" => {
                            let t = el
                                .value()
                                .attr("type")
                                .unwrap_or("text")
                                .to_lowercase();
                            match t.as_str() {
                                "submit" | "button" | "image" | "reset" | "file" => {
                                    (t, String::new())
                                }
                                "checkbox" | "radio" => {
                                    let checked =
                                        el.value().attr("checked").is_some();
                                    let v = el
                                        .value()
                                        .attr("value")
                                        .unwrap_or("on")
                                        .to_string();
                                    (t, if checked { v } else { String::new() })
                                }
                                _ => (
                                    t,
                                    el.value().attr("value").unwrap_or("").to_string(),
                                ),
                            }
                        }
                        "textarea" => (
                            "textarea".to_string(),
                            el.text().collect::<Vec<_>>().join(" "),
                        ),
                        _ => {
                            // select：取选中项，否则首项
                            let mut picked = String::new();
                            let mut first = String::new();
                            if let Ok(opt_sel) = Selector::parse("option") {
                                for opt in el.select(&opt_sel) {
                                    let v = opt
                                        .value()
                                        .attr("value")
                                        .map(str::to_string)
                                        .unwrap_or_else(|| {
                                            opt.text().collect::<Vec<_>>().join(" ")
                                        });
                                    if first.is_empty() {
                                        first.clone_from(&v);
                                    }
                                    if opt.value().attr("selected").is_some() {
                                        picked = v;
                                        break;
                                    }
                                }
                            }
                            (
                                "select".to_string(),
                                if picked.is_empty() { first } else { picked },
                            )
                        }
                    };
                    fields.push(FormField { name, kind, value });
                }
            }
            forms.push(FormSpec {
                index: fi,
                action,
                method,
                fields,
            });
        }
    }

    let raw_html: String = collapsed.chars().take(MAX_RAW_HTML_CHARS).collect();
    Ok((title, text, links, forms, raw_html))
}

pub(crate) fn render_snapshot_text(snap: &PageSnapshot) -> String {
    let mut out = String::new();
    out.push_str(&format!("URL: {}\n", snap.url));
    if let Some(t) = &snap.title {
        out.push_str(&format!("Title: {t}\n"));
    }
    out.push_str(&format!("Text: {}\n", snap.text));
    out.push_str(&format!("Links ({}):\n", snap.links.len()));
    for l in snap.links.iter().take(50) {
        out.push_str(&format!("  [{}] {} -> {}\n", l.index, l.text, l.href));
    }
    out.push_str(&format!("Forms ({}):\n", snap.forms.len()));
    for f in &snap.forms {
        let names: Vec<String> =
            f.fields.iter().map(|x| x.name.clone()).collect();
        out.push_str(&format!(
            "  #{} {} {} fields=[{}]\n",
            f.index,
            f.method,
            f.action,
            names.join(",")
        ));
    }
    out
}

// ============================================================================
// 后端：HTTP（真抓取）
// ============================================================================

pub(crate) struct Fetched {
    pub(crate) final_url: String,
    pub(crate) status: u16,
    pub(crate) body: String,
    pub(crate) set_cookies: Vec<String>,
}

/// 按 Content-Type charset 解码正文（GBK 等中文站必需；未知编码回退 UTF-8 lossy）
pub(crate) fn decode_body(content_type: Option<&str>, bytes: &[u8]) -> String {
    let label = content_type.and_then(|ct| {
        ct.split(';').skip(1).find_map(|part| {
            let (k, v) = part.split_once('=')?;
            if k.trim().eq_ignore_ascii_case("charset") {
                Some(v.trim().trim_matches('"').to_string())
            } else {
                None
            }
        })
    });
    let enc = label
        .as_deref()
        .and_then(|l| encoding_rs::Encoding::for_label(l.as_bytes()))
        .unwrap_or(encoding_rs::UTF_8);
    let (text, _, _) = enc.decode(bytes);
    text.into_owned()
}

pub(crate) fn http_client_for(config: &BrowserConfig) -> Result<Client, BrowserError> {
    let mut builder =
        Client::builder().redirect(reqwest::redirect::Policy::limited(5));
    // 显式配置优先，空则读环境（HTTPS_PROXY>HTTP_PROXY>ALL_PROXY 大小写皆可），
    // 封锁区出口：配了即走代理，未配直连（与 nt_web / nt_world_search 同律）。
    // NO_PROXY（含默认回环）一律 bypass——否则 127.0.0.1 探活被自家代理掐死。
    let explicit = config
        .proxy
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned);
    // 2026-09-27 安全修复: 显式代理 URL 解析失败原先被 `.ok()` 静默吞掉 →
    // 悄悄降级为**直连**, 与本仓"出口 fail-closed"律法相悖 (mandated proxy 失效
    // 等于没有)。显式配置必须严格校验并报错; 仅环境变量保持宽容。
    let proxy_url = match explicit.as_deref() {
        Some(s) => Some(
            reqwest::Url::parse(s)
                .map_err(|e| BrowserError::ActionFailed(format!("invalid proxy url: {e}")))?,
        ),
        None => proxy_from_env().and_then(|s| reqwest::Url::parse(&s).ok()),
    };
    if let Some(base) = proxy_url {
        builder = builder.proxy(reqwest::Proxy::custom(move |url| {
            if no_proxy_hit(url.host_str().unwrap_or("")) {
                None
            } else {
                Some(base.clone())
            }
        }));
    }
    builder
        .build()
        .map_err(|e| BrowserError::ActionFailed(format!("http client: {e}")))
}

/// 代理出口（环境）：`HTTPS_PROXY>HTTP_PROXY>ALL_PROXY`，大小写皆可；空串视为未配。
pub(crate) fn proxy_from_env() -> Option<String> {
    ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

/// NO_PROXY 命中（精确 + `.域` 后缀 + `*`；本机回环默认直连，配了代理也不穿代理——
/// 否则 crystal/ollama 的 127.0.0.1 探活会被自家代理掐死）。
pub(crate) fn no_proxy_hit(host: &str) -> bool {
    let h = host.trim().trim_end_matches('.').to_lowercase();
    if h.is_empty() {
        return false;
    }
    if h == "localhost" || h == "127.0.0.1" || h == "::1" {
        return true;
    }
    let rules = ["NO_PROXY", "no_proxy"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .unwrap_or_default();
    rules.split(',').map(str::trim).filter(|s| !s.is_empty()).any(|rule| {
        let r = rule.trim_end_matches('.').to_lowercase();
        if r == "*" {
            true
        } else if let Some(suffix) = r.strip_prefix('.') {
            h == suffix || h.ends_with(&format!(".{suffix}"))
        } else {
            h == r
        }
    })
}

pub(crate) async fn http_get(
    client: &Client,
    url: &str,
    jar: &CookieJar,
    ua: &str,
    auth: Option<(String, String)>,
    timeout: Duration,
) -> Result<Fetched, BrowserError> {
    let mut req = client
        .get(url)
        .timeout(timeout)
        .header(reqwest::header::USER_AGENT, ua)
        .header(
            reqwest::header::ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header(reqwest::header::ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8");
    if let Some((name, value)) = auth {
        req = req.header(name, value);
    }
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(cookie) = jar.header_for(parsed.scheme(), parsed.host_str().unwrap_or("")) {
            req = req.header(reqwest::header::COOKIE, cookie);
        }
    }
    let resp = req
        .send()
        .await
        .map_err(|e| BrowserError::NavigationFailed(format!("GET {url}: {e}")))?;
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED
        || status == reqwest::StatusCode::FORBIDDEN
    {
        return Err(BrowserError::AuthRejected(status.as_u16()));
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let secs = parse_retry_after_secs(resp.headers());
        return Err(BrowserError::RateLimited(secs));
    }
    if !status.is_success() {
        return Err(BrowserError::NavigationFailed(format!(
            "GET {url}: http {status}"
        )));
    }
    let set_cookies: Vec<String> = resp
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok().map(str::to_string))
        .collect();
    let final_url = resp.url().to_string();
    let status_code = status.as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| BrowserError::ActionFailed(format!("read body: {e}")))?;
    let body = decode_body(content_type.as_deref(), &bytes);
    Ok(Fetched {
        final_url,
        status: status_code,
        body,
        set_cookies,
    })
}

pub(crate) async fn http_submit(
    client: &Client,
    action: &str,
    method: &str,
    params: &[(String, String)],
    jar: &CookieJar,
    ua: &str,
    auth: Option<(String, String)>,
    timeout: Duration,
) -> Result<Fetched, BrowserError> {
    let with_cookie = |b: reqwest::RequestBuilder| {
        let mut b = b;
        if let Some((ref name, ref value)) = auth {
            b = b.header(name.clone(), value.clone());
        }
        if let Ok(parsed) = url::Url::parse(action) {
            if let Some(cookie) = jar.header_for(parsed.scheme(), parsed.host_str().unwrap_or("")) {
                b = b.header(reqwest::header::COOKIE, cookie);
            }
        }
        b
    };
    let resp = if method.eq_ignore_ascii_case("post") {
        let body: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect();
        with_cookie(
            client
                .post(action)
                .timeout(timeout)
                .header(reqwest::header::USER_AGENT, ua)
                .header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(body.join("&")),
        )
        .send()
        .await
    } else {
        let mut url = url::Url::parse(action)
            .map_err(|e| BrowserError::ActionFailed(format!("bad form action: {e}")))?;
        {
            let mut qp = url.query_pairs_mut();
            for (k, v) in params {
                qp.append_pair(k, v);
            }
        }
        with_cookie(
            client
                .get(url)
                .timeout(timeout)
                .header(reqwest::header::USER_AGENT, ua),
        )
        .send()
        .await
    }
    .map_err(|e| BrowserError::ActionFailed(format!("submit {action}: {e}")))?;
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED
        || status == reqwest::StatusCode::FORBIDDEN
    {
        return Err(BrowserError::AuthRejected(status.as_u16()));
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let secs = parse_retry_after_secs(resp.headers());
        return Err(BrowserError::RateLimited(secs));
    }
    if !status.is_success() {
        return Err(BrowserError::ActionFailed(format!(
            "submit {action}: http {status}"
        )));
    }
    let set_cookies: Vec<String> = resp
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok().map(str::to_string))
        .collect();
    let final_url = resp.url().to_string();
    let status_code = status.as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| BrowserError::ActionFailed(format!("read body: {e}")))?;
    let body = decode_body(content_type.as_deref(), &bytes);
    Ok(Fetched {
        final_url,
        status: status_code,
        body,
        set_cookies,
    })
}

/// multipart 提交（含文件上传；其余字段按文本 part）
pub(crate) async fn multipart_submit(
    client: &Client,
    action: &str,
    params: &[(String, String)],
    uploads: &[(String, Vec<String>)],
    jar: &CookieJar,
    ua: &str,
    auth: Option<(String, String)>,
    timeout: Duration,
) -> Result<Fetched, BrowserError> {
    let mut form = reqwest::multipart::Form::new();
    for (k, v) in params {
        form = form.text(k.clone(), v.clone());
    }
    for (name, paths) in uploads {
        for path in paths {
            let data = tokio::fs::read(path).await.map_err(|e| {
                BrowserError::ActionFailed(format!("upload 读文件 {path}: {e}"))
            })?;
            let filename = std::path::Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("upload")
                .to_string();
            let part = reqwest::multipart::Part::bytes(data).file_name(filename);
            form = form.part(name.clone(), part);
        }
    }
    let mut req = client
        .post(action)
        .timeout(timeout)
        .header(reqwest::header::USER_AGENT, ua)
        .multipart(form);
    if let Some((name, value)) = auth {
        req = req.header(name, value);
    }
    if let Ok(parsed) = url::Url::parse(action) {
        if let Some(cookie) = jar.header_for(parsed.scheme(), parsed.host_str().unwrap_or("")) {
            req = req.header(reqwest::header::COOKIE, cookie);
        }
    }
    let resp = req.send().await.map_err(|e| {
        BrowserError::ActionFailed(format!("multipart {action}: {e}"))
    })?;
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED
        || status == reqwest::StatusCode::FORBIDDEN
    {
        return Err(BrowserError::AuthRejected(status.as_u16()));
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let secs = parse_retry_after_secs(resp.headers());
        return Err(BrowserError::RateLimited(secs));
    }
    if !status.is_success() {
        return Err(BrowserError::ActionFailed(format!(
            "multipart {action}: http {status}"
        )));
    }
    let set_cookies: Vec<String> = resp
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok().map(str::to_string))
        .collect();
    let final_url = resp.url().to_string();
    let status_code = status.as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| BrowserError::ActionFailed(format!("read body: {e}")))?;
    let body = decode_body(content_type.as_deref(), &bytes);
    Ok(Fetched {
        final_url,
        status: status_code,
        body,
        set_cookies,
    })
}

/// 解析 robots.txt：仅取 `User-agent: *` 组下的 Disallow 前缀（纯函数）
pub(crate) fn parse_robots_disallows(body: &str) -> Vec<String> {
    let mut in_wildcard = false;
    let mut out = Vec::new();
    for raw in body.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            match k.trim().to_lowercase().as_str() {
                "user-agent" => {
                    in_wildcard = v.trim() == "*";
                }
                "disallow" if in_wildcard => {
                    let path = v.trim();
                    if !path.is_empty() {
                        out.push(path.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    out
}

pub(crate) fn robots_denied(disallows: &[String], path: &str) -> bool {
    disallows
        .iter()
        .any(|d| path.starts_with(d.as_str()))
}
