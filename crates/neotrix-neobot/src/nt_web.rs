//! `nt_web` — 对话侧联网读写（客户端直调，不经服务端）。
//!
//! 对话即 crystal 全能力外表：chat 路径此前只供本地工具（bash/文件），
//! 联网能力只活在服务端 agents/run。`web_search`/`web_fetch` 补上这块。
//!
//! 搜索链（2026-09-26 实测）：Bing RSS 主路（200 + 10 条含描述/日期）→
//! Wikipedia 回退；DDG html/lite 自本机 202 空墙，已摘出默认链
//! （`parse_ddg` 解析器保留复用 + 单测）。
//!
//! 无新依赖（ureq 现成）；纯解析函数单测覆盖，网络不出单测。

use std::time::Duration;

use crate::nt_error::NtBotError;

const TIMEOUT: Duration = Duration::from_secs(15);
const FETCH_MAX_CHARS: usize = 4000;
const UA: &str = "neobot/1.0 (+local-first agent)";

/// 代理出口（环境）：`HTTPS_PROXY>HTTP_PROXY>ALL_PROXY`（大小写皆可）；
/// `NO_PROXY/no_proxy` 命中走直连（精确 + `.域` 后缀通配 + `*` 全放）。
/// 与 core（fetch.rs / nt_world_search）同律；未配即直连。
pub fn proxy_for_url(url: &str) -> Option<String> {
    let no_proxy = ["NO_PROXY", "no_proxy"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .unwrap_or_default();
    if no_proxy_hit_url(url, &no_proxy) {
        return None;
    }
    ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

/// NO_PROXY 命中判定（纯函数）：空表永不命中；`*` 全放；`.foo` 匹配 `*.foo`；余精确。
pub fn no_proxy_hit_url(url: &str, no_proxy: &str) -> bool {
    let host = host_of(url).to_lowercase();
    if host.is_empty() {
        return false;
    }
    no_proxy.split(',').map(str::trim).filter(|s| !s.is_empty()).any(|rule| {
        let r = rule.to_lowercase();
        if r == "*" {
            true
        } else if let Some(suffix) = r.strip_prefix('.') {
            host == suffix || host.ends_with(&format!(".{suffix}"))
        } else {
            host == r
        }
    })
}

fn host_of(url: &str) -> &str {
    let s = url.split("://").nth(1).unwrap_or(url);
    let s = s.split('/').next().unwrap_or(s);
    s.split(':').next().unwrap_or(s).split('?').next().unwrap_or(s)
}

/// 带代理的 Agent（按 URL 判定；代理非法回落直连，不炸）。
fn agent_for(url: &str) -> ureq::Agent {
    let mut builder = ureq::AgentBuilder::new().timeout(TIMEOUT);
    if let Some(proxy) = proxy_for_url(url) {
        if let Ok(p) = ureq::Proxy::new(&proxy) {
            builder = builder.proxy(p);
        }
    }
    builder.build()
}

/// 联网搜索：Bing RSS 主路，空/错回退 Wikipedia opensearch。返回证据行文本。
/// 新闻类查询（见 `is_news_query`）先走中文热点直连（百度热搜 + 新浪滚动），
/// Bing 对中文新闻常回无关页（2026-09-26 实测：问“今日要点”回 YouTube 帮助页）。
/// 百科类查询（见 `is_factual_query`）先走 Wikipedia（权威稳定），再 Bing 补新鲜。
pub fn web_search(query: &str, count: usize) -> Result<String, NtBotError> {
    let q = query.trim();
    if q.is_empty() {
        return Err(NtBotError::Invalid("web_search requires {query}".to_owned()));
    }
    let n = count.clamp(1, 10);
    if is_news_query(q) {
        match hot_news(n) {
            Ok(rows) if !rows.is_empty() => return Ok(format_rows(&rows)),
            _ => {}
        }
    }
    if is_factual_query(q) {
        match wiki_search(q, n) {
            Ok(text) => return Ok(text),
            _ => {}
        }
    }
    match bing_search(q, n) {
        Ok(rows) if !rows.is_empty() => Ok(format_rows(&rows)),
        _ => wiki_search(q, n),
    }
}

/// 百科意图判定（纯函数）：定义/人物/概念类先查百科（稳），再补全网（新）。
pub fn is_factual_query(q: &str) -> bool {
    let lower = q.to_lowercase();
    ["是什么", "什么是", "定义", "是谁", "简介", "介绍", "含义",
     "what is", "what's", "who is", "define", "meaning of", "introduction"]
        .iter()
        .any(|k| lower.contains(&k.to_lowercase()))
}

/// 新闻意图判定（纯函数）：中英热点关键词命中即走热点直连。
pub fn is_news_query(q: &str) -> bool {
    let lower = q.to_lowercase();
    ["新闻", "要闻", "热点", "头条", "热搜", "快讯", "头版", "大事", "要事",
     "消息", "快报", "动态", "资讯",
     "headline", "headlines", "breaking", "top news", "hot", "latest news"]
        .iter()
        .any(|k| lower.contains(&k.to_lowercase()))
}

/// 中文热点直连：百度热搜 + 头条热榜 + 新浪滚动（国内 2510/国际 2511/财经 2509）+
/// V2EX 热议合并，按发布时间倒序（榜单=抓取时刻，天然置顶；未知沉底），标题去重。
/// 五源并发抓（`thread::scope`，串行最坏 75s+ → 并行 ~15s 上限）。
pub fn hot_news(count: usize) -> Result<Vec<Row>, String> {
    let n = count.clamp(1, 10);
    let (b, t, s10, s11, s09, v) = std::thread::scope(|scope| {
        let b = scope.spawn(|| baidu_top(n));
        let t = scope.spawn(|| toutiao_hot(n));
        let s10 = scope.spawn(|| sina_roll(n, "2510"));
        let s11 = scope.spawn(|| sina_roll(n, "2511"));
        let s09 = scope.spawn(|| sina_roll(n, "2509"));
        let v = scope.spawn(|| v2ex_hot(n));
        // 线程 panic（网络栈崩）按源失败计，不炸整批。
        let flat = |h: std::thread::ScopedJoinHandle<'_, Result<Vec<Row>, String>>| {
            h.join().ok().and_then(|r| r.ok()).unwrap_or_default()
        };
        (flat(b), flat(t), flat(s10), flat(s11), flat(s09), flat(v))
    });
    let mut rows: Vec<Row> = Vec::new();
    for list in [b, t, s10, s11, s09, v] {
        push_unique(&mut rows, list);
    }
    if rows.is_empty() {
        return Err("hot news: all sources empty".to_owned());
    }
    // 新鲜度排序（ts=0 沉底），再截断。
    rows.sort_by(|a, b| b.ts.cmp(&a.ts));
    rows.truncate(n);
    Ok(rows)
}

/// 标题去重并入（纯逻辑，借用冲突外置）。
fn push_unique(rows: &mut Vec<Row>, more: Vec<Row>) {
    for r in more {
        if !rows.iter().any(|x| x.title == r.title) {
            rows.push(r);
        }
    }
}

/// 头条热榜（免 key；Title + trending 直链 + 热度值）。
fn toutiao_hot(count: usize) -> Result<Vec<Row>, String> {
    let body = agent_for("https://www.toutiao.com")
        .get("https://www.toutiao.com/hot-event/hot-board/")
        .query("origin", "toutiao_pc")
        .set("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/126.0 Safari/537.36")
        .call()
        .map_err(|e| format!("toutiao: {e}"))?
        .into_string()
        .map_err(|e| format!("toutiao body: {e}"))?;
    parse_toutiao_hot(&body, count)
}

/// 头条热榜解析（纯函数）。
pub fn parse_toutiao_hot(body: &str, count: usize) -> Result<Vec<Row>, String> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("json: {e}"))?;
    let data = v.get("data").and_then(|d| d.as_array()).ok_or("no data")?;
    let mut rows = Vec::new();
    for it in data.iter().take(count.max(20)) {
        let title = it.get("Title").and_then(|t| t.as_str()).unwrap_or("").trim().to_owned();
        if title.is_empty() {
            continue;
        }
        let url = it.get("Url").and_then(|u| u.as_str()).unwrap_or("").trim().to_owned();
        let hot = it.get("HotValue").and_then(|h| h.as_i64()).unwrap_or(0);
        rows.push(Row {
            title: title.clone(),
            url: if url.is_empty() {
                format!("https://so.toutiao.com/search?keyword={}", percent_encode(&title))
            } else {
                url
            },
            snippet: if hot > 0 {
                format!("头条热榜 · 热度{hot}")
            } else {
                "头条热榜".to_owned()
            },
            // 榜单即实时：ts 取抓取时刻。
            ts: chrono::Utc::now().timestamp(),
        });
        if rows.len() >= count {
            break;
        }
    }
    if rows.is_empty() {
        return Err("toutiao: no rows".to_owned());
    }
    Ok(rows)
}

/// V2EX 热议（免 key；标题 + 帖链 + 回复数）。
fn v2ex_hot(count: usize) -> Result<Vec<Row>, String> {
    let body = agent_for("https://www.v2ex.com")
        .get("https://www.v2ex.com/api/topics/hot.json")
        .set("User-Agent", "Mozilla/5.0")
        .call()
        .map_err(|e| format!("v2ex: {e}"))?
        .into_string()
        .map_err(|e| format!("v2ex body: {e}"))?;
    parse_v2ex_hot(&body, count)
}

/// V2EX 解析（纯函数；created epoch 秒直入 ts）。
pub fn parse_v2ex_hot(body: &str, count: usize) -> Result<Vec<Row>, String> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("json: {e}"))?;
    let arr = v.as_array().ok_or("not array")?;
    let mut rows = Vec::new();
    for it in arr.iter().take(count.max(20)) {
        let title = it.get("title").and_then(|t| t.as_str()).unwrap_or("").trim().to_owned();
        if title.is_empty() {
            continue;
        }
        let id = it.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
        let url = it
            .get("url")
            .and_then(|u| u.as_str())
            .map(str::trim)
            .filter(|u| !u.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("https://www.v2ex.com/t/{id}"));
        let replies = it.get("replies").and_then(|r| r.as_i64()).unwrap_or(0);
        let node = it
            .get("node")
            .and_then(|n| n.get("title"))
            .and_then(|t| t.as_str())
            .unwrap_or("");
        rows.push(Row {
            title,
            url,
            snippet: if replies > 0 {
                format!("V2EX热议 · {replies}回复{node}", node = if node.is_empty() { String::new() } else { format!(" · {node}") })
            } else {
                "V2EX热议".to_owned()
            },
            ts: it.get("created").and_then(|c| c.as_i64()).unwrap_or(0),
        });
        if rows.len() >= count {
            break;
        }
    }
    if rows.is_empty() {
        return Err("v2ex: no rows".to_owned());
    }
    Ok(rows)
}

/// 百度热搜实时榜（免 key；标题 + 搜索链接）。
fn baidu_top(count: usize) -> Result<Vec<Row>, String> {
    let body = agent_for("https://top.baidu.com")
        .get("https://top.baidu.com/board")
        .query("tab", "realtime")
        .set("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/126.0 Safari/537.36")
        .call()
        .map_err(|e| format!("baidu: {e}"))?
        .into_string()
        .map_err(|e| format!("baidu body: {e}"))?;
    Ok(parse_baidu_top(&body, count))
}

/// 百度榜单解析（纯函数）：`c-single-text-ellipsis` 标题，链接配 `s?wd=` 搜索页。
pub fn parse_baidu_top(html: &str, count: usize) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut rest = html;
    while rows.len() < count {
        let Some(p) = rest.find("c-single-text-ellipsis") else {
            break;
        };
        let seg = &rest[p..];
        let Some(gt) = seg.find('>').map(|i| i + 1) else {
            break;
        };
        let tail = &seg[gt..];
        let Some(e) = tail.find('<') else {
            break;
        };
        let title = tail[..e].trim().to_owned();
        rest = &tail[e..];
        if title.chars().count() < 4 || !seen.insert(title.clone()) {
            continue;
        }
        rows.push(Row {
            url: format!("https://www.baidu.com/s?wd={}", percent_encode(&title)),
            title,
            snippet: "百度热搜实时榜".to_owned(),
            // 榜单即实时：ts 取抓取时刻，新鲜度置顶。
            ts: chrono::Utc::now().timestamp(),
        });
    }
    rows
}

/// 新浪滚动快讯（免 key；lid 2510 国内 / 2511 国际 / 2509 财经，标题 + 直链 + 摘要）。
fn sina_roll(count: usize, lid: &str) -> Result<Vec<Row>, String> {
    let body = agent_for("https://feed.mix.sina.com.cn")
        .get("https://feed.mix.sina.com.cn/api/roll/get")
        .query("pageid", "153")
        .query("lid", lid)
        .query("num", &count.to_string())
        .set("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/126.0 Safari/537.36")
        .call()
        .map_err(|e| format!("sina: {e}"))?
        .into_string()
        .map_err(|e| format!("sina body: {e}"))?;
    parse_sina_roll(&body)
}

/// 新浪滚动解析（纯函数）。
pub fn parse_sina_roll(body: &str) -> Result<Vec<Row>, String> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("json: {e}"))?;
    let data = v
        .get("result")
        .and_then(|r| r.get("data"))
        .and_then(|d| d.as_array())
        .ok_or("no result.data")?;
    let mut rows = Vec::new();
    for it in data {
        let title = it.get("title").and_then(|t| t.as_str()).unwrap_or("").trim().to_owned();
        let url = it.get("url").and_then(|u| u.as_str()).unwrap_or("").trim().to_owned();
        if title.is_empty() || url.is_empty() {
            continue;
        }
        let media = it.get("media_name").and_then(|m| m.as_str()).unwrap_or("");
        let summary = it.get("summary").and_then(|s| s.as_str()).unwrap_or("").trim();
        let snippet = if media.is_empty() {
            summary.chars().take(120).collect::<String>()
        } else {
            format!("{media}：{}", summary.chars().take(100).collect::<String>())
        };
        // ctime epoch 秒；非法回 0（排序沉底）。
        let ts = it
            .get("ctime")
            .and_then(|c| c.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        rows.push(Row { title, url, snippet, ts });
    }
    if rows.is_empty() {
        return Err("sina: no rows".to_owned());
    }
    Ok(rows)
}

/// 百分比编码（纯函数）：unreserved 原样，余按 UTF-8 字节 `%XX` 大写。
pub fn percent_encode(s: &str) -> String {
    const UNRESERVED: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
    let mut out = String::new();
    for b in s.as_bytes() {
        if UNRESERVED.contains(b) {
            out.push(*b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// 网页抓取：只允许 http/https；去标签后 4000 字截断。
pub fn web_fetch(url: &str) -> Result<String, NtBotError> {
    let target = url.trim();
    if !(target.starts_with("http://") || target.starts_with("https://")) {
        return Err(NtBotError::Invalid(format!(
            "web_fetch refused (scheme): {target}"
        )));
    }
    let body = agent_for(target)
        .get(target)
        .set("User-Agent", UA)
        .call()
        .map_err(|e| NtBotError::Engine {
            engine: "web_fetch".to_owned(),
            reason: format!("{e}"),
        })?
        .into_string()
        .map_err(|e| NtBotError::Engine {
            engine: "web_fetch".to_owned(),
            reason: format!("read body: {e}"),
        })?;
    let text = strip_tags(&body);
    if text.trim().is_empty() {
        return Err(NtBotError::Engine {
            engine: "web_fetch".to_owned(),
            reason: "empty text".to_owned(),
        });
    }
    Ok(truncate_chars(&text, FETCH_MAX_CHARS))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub title: String,
    pub url: String,
    pub snippet: String,
    /// 发布时间（epoch 秒；0=未知，排序沉底）。
    pub ts: i64,
}

fn format_rows(rows: &[Row]) -> String {
    rows.iter()
        .enumerate()
        .map(|(i, r)| {
            let date = if r.ts > 0 {
                format!("（{}）", fmt_ts(r.ts))
            } else {
                String::new()
            };
            format!("[{}] {} — {}{}\n{}", i + 1, r.title, r.url, date, r.snippet)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// epoch 秒 → `M月d日 HH:MM`（北京时间）。
fn fmt_ts(ts: i64) -> String {
    use chrono::{TimeZone, Utc};
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|t| t + chrono::Duration::hours(8))
        .map(|t| t.format("%-m月%-d日 %H:%M").to_string())
        .unwrap_or_default()
}

/// RFC2822 → epoch 秒（Bing pubDate；失败 0）。
fn parse_rfc2822(s: &str) -> i64 {
    use chrono::DateTime;
    DateTime::parse_from_rfc2822(s).map(|d| d.timestamp()).unwrap_or(0)
}

fn bing_search(query: &str, count: usize) -> Result<Vec<Row>, String> {
    let body = agent_for("https://www.bing.com")
        .get("https://www.bing.com/search")
        .set("User-Agent", UA)
        .query("format", "rss")
        .query("q", query)
        .call()
        .map_err(|e| format!("bing: {e}"))?
        .into_string()
        .map_err(|e| format!("bing body: {e}"))?;
    Ok(parse_bing(&body, count))
}

/// Bing RSS 解析（纯函数）：`<item>` → title/link/description（+pubDate 并入 snippet）。
pub fn parse_bing(xml: &str, count: usize) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut rest = xml;
    while rows.len() < count {
        let Some(s) = rest.find("<item>") else {
            break;
        };
        let seg = &rest[s + 6..];
        let Some(e) = seg.find("</item>") else {
            break;
        };
        let item = &seg[..e];
        let field = |tag: &str| {
            item.find(&format!("<{tag}>"))
                .and_then(|p| {
                    let r = &item[p + tag.len() + 2..];
                    r.find(&format!("</{tag}>")).map(|q| r[..q].trim().to_owned())
                })
                .unwrap_or_default()
        };
        let title = unescape(&field("title"));
        let url = unescape(&field("link"));
        let desc = unescape(&field("description"));
        let date = unescape(&field("pubDate"));
        if !title.is_empty() && !url.is_empty() {
            rows.push(Row { title, url, snippet: desc, ts: parse_rfc2822(&date) });
        }
        rest = &seg[e + 7..];
    }
    rows
}

/// XML 转义还原（Bing RSS 常用 5 种；其余原样）。
fn unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
}

/// DDG html 结果解析（纯函数）：`result__a` 标题链 + `result__snippet`。
/// 注：自本机 DDG 常回 202 空墙，不在默认链；解析器保留（换出口/IP 即复活）。
pub fn parse_ddg(html: &str, count: usize) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut rest = html;
    while rows.len() < count {
        let Some(a_pos) = rest.find("result__a") else {
            break;
        };
        let seg = &rest[a_pos..];
        let Some(href_s) = seg.find("href=\"").map(|i| i + 6) else {
            break;
        };
        let href_rest = &seg[href_s..];
        let Some(href_e) = href_rest.find('"') else {
            break;
        };
        let href = href_rest[..href_e].to_owned();
        let Some(gt) = href_rest.find('>').map(|i| i + 1) else {
            break;
        };
        let title_rest = &href_rest[gt..];
        let Some(title_e) = title_rest.find("</a>") else {
            break;
        };
        let title = strip_tags(&title_rest[..title_e]).trim().to_owned();
        let after = &title_rest[title_e..];
        let snippet = after
            .find("result__snippet")
            .and_then(|p| after[p..].find('>').map(|i| (p, i + 1)))
            .and_then(|(p, i)| {
                let s = &after[p + i..];
                s.find("</").map(|e| strip_tags(&s[..e]).trim().to_owned())
            })
            .unwrap_or_default();
        // DDG 站内跳转形（//duckduckgo.com/l/?udata=…）解出真实目标。
        let url = real_url(&href);
        if !title.is_empty() && !url.is_empty() {
            rows.push(Row { title, url, snippet, ts: 0 });
        }
        rest = after;
    }
    rows
}

/// DDG `udata` 跳转解码（取 `udata=` 后首个 http(s) 串；失败回原串）。
fn real_url(href: &str) -> String {
    let decoded = href
        .replace("&amp;", "&")
        .replace("%3A", ":")
        .replace("%2F", "/");
    if let Some(p) = decoded.find("http") {
        let tail = &decoded[p..];
        let end = tail
            .find(['&', '"', '\'', ' '])
            .unwrap_or(tail.len());
        return tail[..end].to_owned();
    }
    if href.starts_with("//") {
        return format!("https:{href}");
    }
    href.to_owned()
}

fn wiki_search(query: &str, count: usize) -> Result<String, NtBotError> {
    for api in [
        "https://zh.wikipedia.org/w/api.php",
        "https://en.wikipedia.org/w/api.php",
    ] {
        let out = agent_for(api)
            .get(api)
            .query("action", "opensearch")
            .query("search", query)
            .query("limit", &count.to_string())
            .query("format", "json")
            .call()
            .map_err(|e| format!("wiki: {e}"))
            .and_then(|r| r.into_string().map_err(|e| format!("wiki body: {e}")))
            .and_then(|b| parse_wiki(&b).map_err(|e| format!("wiki parse: {e}")));
        match out {
            Ok(rows) if !rows.is_empty() => return Ok(format_rows(&rows)),
            _ => continue,
        }
    }
    Err(NtBotError::Engine {
        engine: "web_search".to_owned(),
        reason: "no results (bing + wikipedia)".to_owned(),
    })
}

/// Wikipedia opensearch 解析（纯函数）：`[term, titles[], descs[], urls[]]`。
pub fn parse_wiki(body: &str) -> Result<Vec<Row>, String> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("json: {e}"))?;
    let arr = v.as_array().ok_or("not array")?;
    let titles = arr.get(1).and_then(|a| a.as_array()).cloned().unwrap_or_default();
    let descs = arr.get(2).and_then(|a| a.as_array()).cloned().unwrap_or_default();
    let urls = arr.get(3).and_then(|a| a.as_array()).cloned().unwrap_or_default();
    let mut rows = Vec::new();
    for i in 0..titles.len() {
        let title = titles[i].as_str().unwrap_or("").to_owned();
        if title.is_empty() {
            continue;
        }
        rows.push(Row {
            title,
            url: urls.get(i).and_then(|u| u.as_str()).unwrap_or("").to_owned(),
            snippet: descs.get(i).and_then(|d| d.as_str()).unwrap_or("").to_owned(),
            // 百科无时效语义：ts=0（百科走 factual 路，不参与新鲜度排序）。
            ts: 0,
        });
    }
    Ok(rows)
}

/// 去 HTML 标签（纯函数）：script/style 整块丢，其余标签换空格，空白折叠。
pub fn strip_tags(html: &str) -> String {
    let mut s = html.to_owned();
    for tag in ["script", "style", "noscript"] {
        loop {
            let lower = s.to_lowercase();
            let Some(start) = lower.find(&format!("<{tag}")) else {
                break;
            };
            let Some(end) = lower[start..].find(&format!("</{tag}>")).map(|i| start + i + tag.len() + 3) else {
                break;
            };
            s.replace_range(start..end, " ");
        }
    }
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 按字符截断（中文安全）。
pub fn truncate_chars(s: &str, n: usize) -> String {
    let out: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        format!("{out}…")
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_proxy_rules_match() {
        assert!(!no_proxy_hit_url("https://a.com/x", ""));
        assert!(no_proxy_hit_url("https://a.com/x", "*"));
        assert!(no_proxy_hit_url("https://a.com/x", "a.com, b.com"));
        assert!(no_proxy_hit_url("https://sub.a.com/x", ".a.com"));
        assert!(!no_proxy_hit_url("https://a.com.evil/x", ".a.com"));
        assert!(no_proxy_hit_url("http://127.0.0.1:3000/v1", "127.0.0.1,localhost"));
        assert!(!no_proxy_hit_url("https://bing.com/", "127.0.0.1"));
    }

    #[test]
    fn toutiao_parse_extracts_rows() {
        let body = r#"{"data":[{"Title":"中美达成300亿美元对等降税安排","Url":"https://www.toutiao.com/trending/1/","HotValue":151452423},{"Title":"","Url":""}]}"#;
        let rows = parse_toutiao_hot(body, 10).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].snippet.contains("151452423"));
        assert!(rows[0].ts > 1700000000);
    }

    #[test]
    fn v2ex_parse_extracts_rows() {
        let body = r#"[{"id":1244838,"title":"muse 注册方法探讨","url":"","replies":42,"created":1790411670,"node":{"title":"分享发现"}}]"#;
        let rows = parse_v2ex_hot(body, 10).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].url, "https://www.v2ex.com/t/1244838");
        assert!(rows[0].snippet.contains("42回复"));
        assert_eq!(rows[0].ts, 1790411670);
    }

    #[test]
    fn factual_query_routes_wiki_first() {
        assert!(is_factual_query("Rust 是什么语言"));
        assert!(is_factual_query("who is the president"));
        assert!(!is_factual_query("今日要点新闻"));
        assert!(!is_factual_query("Rust 最新稳定版号"));
    }

    #[test]
    fn ts_helpers_behave() {
        assert!(parse_rfc2822("Fri, 25 Sep 2026 18:59:00 GMT") > 1700000000);
        assert_eq!(parse_rfc2822("not a date"), 0);
        assert!(fmt_ts(1790411670).contains("月"));
    }

    #[test]
    fn news_query_hints_match() {
        assert!(is_news_query("获取今日要点新闻"));
        assert!(is_news_query("top headlines today"));
        assert!(!is_news_query("Rust 最新稳定版号"));
    }

    #[test]
    fn percent_encode_keeps_unreserved() {
        assert_eq!(percent_encode("abc-_.~123"), "abc-_.~123");
        assert!(percent_encode("今日要闻").starts_with('%'));
        assert!(!percent_encode("a b").contains(' '));
    }

    #[test]
    fn baidu_parse_extracts_titles() {
        let html = r#"<div class="c-single-text-ellipsis">习近平结束访美回到北京</div><div class="c-single-text-ellipsis">ab</div><div class="c-single-text-ellipsis">习近平结束访美回到北京</div>"#;
        let rows = parse_baidu_top(html, 10);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].url.starts_with("https://www.baidu.com/s?wd="));
        assert_eq!(rows[0].snippet, "百度热搜实时榜");
    }

    #[test]
    fn sina_parse_extracts_rows() {
        let body = r#"{"result":{"data":[{"title":"瑞幸杀回来了","url":"https://finance.sina.com.cn/x","media_name":"新浪财经","summary":"门店数创新高"}]}}"#;
        let rows = parse_sina_roll(body).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].snippet.contains("新浪财经"));
    }

    #[test]
    fn bing_parse_extracts_items() {
        let xml = r#"<?xml version="1.0"?><rss><channel><item><title>Rust Programming Language</title><link>https://rust-lang.org/</link><description>fast and safe</description><pubDate>Fri, 25 Sep 2026 18:59:00 GMT</pubDate></item></channel></rss>"#;
        let rows = parse_bing(xml, 10);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].title, "Rust Programming Language");
        assert_eq!(rows[0].snippet, "fast and safe");
        // 日期进 ts（排版由 format_rows 统一加），不再塞 snippet。
        assert!(rows[0].ts > 1700000000);
    }

    #[test]
    fn ddg_parse_extracts_rows() {
        let html = r#"<div class="result"><a class="result__a" href="https://example.com/a">A <b>Title</b></a><a class="result__snippet">first snippet here</a></div><div class="result"><a class="result__a" href="//duckduckgo.com/l/?udata=https%3A%2F%2Fexample.com%2Fb&amp;rut=x">B</a></div>"#;
        let rows = parse_ddg(html, 10);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title, "A Title");
        assert_eq!(rows[0].snippet, "first snippet here");
        assert_eq!(rows[1].url, "https://example.com/b");
    }

    #[test]
    fn wiki_parse_handles_opensearch() {
        let body = r#"["q",["Rust"],["systems language"],["https://en.wikipedia.org/wiki/Rust"]]"#;
        let rows = parse_wiki(body).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].title, "Rust");
        assert!(rows[0].url.contains("wikipedia"));
    }

    #[test]
    fn strip_tags_drops_script_and_collapses() {
        let html = "<html><head><script>var x=1;</script></head><body><h1>Hi</h1><p>a  b</p></body></html>";
        assert_eq!(strip_tags(html), "Hi a b");
    }

    #[test]
    fn fetch_rejects_non_http_scheme() {
        assert!(web_fetch("file:///etc/passwd").is_err());
        assert!(web_fetch("ftp://x/y").is_err());
    }

    #[test]
    fn search_rejects_empty_query() {
        assert!(web_search("  ", 5).is_err());
    }
}
