//! nt_shield_comm — NT-SHIELD 通信伪装层
//!
//! Port of the retired `scripts/nt_comm_router.py` (IdentityPool / HeaderObfuscator /
//! TimingObfuscator / GeoCoherence / CommRouter) + `nt_api_client.py` 的
//! persona 选择。目标: 使外部请求呈现为随机全球真实浏览器用户, 剥离内部
//! NeoTrix 指纹。R-P79: 通过 `StealthHttpClient::with_persona` 接入生产网络
//! 出口, 并提供 `comm` CLI 观测。
//!
//! 范围说明: TLS/H2/tcp 窗口等 JA3 级指纹需 curl_cffi/真实引擎, 纯 Rust
//! reqwest 无法伪造, 与 Python 版一致(urllib 也不设置这些字段) — 仅保留
//! header 级伪装。

use rand::Rng;
use rusqlite::{Connection, params};
use std::time::{Duration, Instant};

pub const COMM_DB_TABLES: &str = "\
CREATE TABLE IF NOT EXISTS identity_pool (
    id TEXT PRIMARY KEY,
    persona_key TEXT NOT NULL,
    created_at REAL NOT NULL,
    last_used REAL,
    success_count INTEGER DEFAULT 0,
    fail_count INTEGER DEFAULT 0,
    last_ip TEXT,
    last_ip_geo TEXT
);
CREATE TABLE IF NOT EXISTS persona_stats (
    persona_key TEXT PRIMARY KEY,
    total_uses INTEGER DEFAULT 0,
    total_success INTEGER DEFAULT 0,
    total_fail INTEGER DEFAULT 0,
    avg_latency_ms REAL DEFAULT 0.0
);
CREATE TABLE IF NOT EXISTS failure_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp REAL NOT NULL,
    persona_id TEXT,
    domain TEXT,
    status_code INTEGER,
    error TEXT,
    proxy_used TEXT
);
";

// ============================================================================
// Persona 目录 (2026 浏览器档案) — 与已退役 nt_comm_router.py PERSONAS 一致
// ============================================================================

#[derive(Clone, Copy)]
pub struct Persona {
    pub key: &'static str,
    pub label: &'static str,
    pub weight: f64,
    pub ua: &'static str,
    pub accept: &'static str,
    pub accept_encoding: &'static str,
    pub accept_language: &'static str,
    pub sec_ch_ua: Option<&'static str>,
    pub sec_ch_ua_mobile: Option<&'static str>,
    pub sec_ch_ua_platform: Option<&'static str>,
    pub sec_fetch_dest: &'static str,
    pub sec_fetch_mode: &'static str,
    pub sec_fetch_site: &'static str,
    pub sec_fetch_user: Option<&'static str>,
    pub upgrade_insecure_requests: Option<&'static str>,
    pub header_order: &'static [&'static str],
    pub geo_regions: &'static [&'static str],
}

const CHROME_ORDER: &[&str] = &[
    ":method", ":path", ":scheme", ":authority", "accept-encoding",
    "accept-language", "user-agent", "accept", "sec-ch-ua", "sec-ch-ua-mobile",
    "sec-ch-ua-platform", "sec-fetch-dest", "sec-fetch-mode", "sec-fetch-site",
    "sec-fetch-user", "upgrade-insecure-requests", "referer",
];

const FIREFOX_ORDER: &[&str] = &[
    ":method", ":path", ":scheme", ":authority", "user-agent", "accept",
    "accept-language", "accept-encoding", "referer", "upgrade-insecure-requests",
    "sec-fetch-dest", "sec-fetch-mode", "sec-fetch-site", "cache-control", "pragma",
];

const SAFARI_ORDER: &[&str] = &[
    ":method", ":path", ":scheme", ":authority", "accept-encoding",
    "accept-language", "user-agent", "accept", "sec-fetch-dest", "sec-fetch-mode",
    "sec-fetch-site", "referer",
];

const CHROME_SEC_CH_UA: &str = r#""Not A(Brand";v="8", "Chromium";v="132", "Google Chrome";v="132""#;
const EDGE_SEC_CH_UA: &str = r#""Not A(Brand";v="8", "Chromium";v="132", "Microsoft Edge";v="132""#;
const CHROME_ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8";

pub fn personas() -> &'static [Persona] {
    &PERSONAS
}

pub static PERSONAS: [Persona; 6] = [
    Persona {
        key: "chrome_win", label: "Chrome 132 / Windows 11", weight: 0.35,
        ua: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        accept: CHROME_ACCEPT, accept_encoding: "gzip, deflate, br, zstd",
        accept_language: "en-US,en;q=0.9",
        sec_ch_ua: Some(CHROME_SEC_CH_UA), sec_ch_ua_mobile: Some("?0"),
        sec_ch_ua_platform: Some("\"Windows\""),
        sec_fetch_dest: "document", sec_fetch_mode: "navigate", sec_fetch_site: "none",
        sec_fetch_user: Some("?1"), upgrade_insecure_requests: Some("1"),
        header_order: CHROME_ORDER,
        geo_regions: &["US", "GB", "CA", "AU", "DE", "FR"],
    },
    Persona {
        key: "chrome_mac", label: "Chrome 132 / macOS 15", weight: 0.15,
        ua: "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        accept: CHROME_ACCEPT, accept_encoding: "gzip, deflate, br, zstd",
        accept_language: "en-US,en;q=0.9",
        sec_ch_ua: Some(CHROME_SEC_CH_UA), sec_ch_ua_mobile: Some("?0"),
        sec_ch_ua_platform: Some("\"macOS\""),
        sec_fetch_dest: "document", sec_fetch_mode: "navigate", sec_fetch_site: "none",
        sec_fetch_user: Some("?1"), upgrade_insecure_requests: Some("1"),
        header_order: CHROME_ORDER,
        geo_regions: &["US", "GB", "CA", "AU", "JP", "KR"],
    },
    Persona {
        key: "chrome_linux", label: "Chrome 132 / Linux x86_64", weight: 0.03,
        ua: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        accept: CHROME_ACCEPT, accept_encoding: "gzip, deflate, br, zstd",
        accept_language: "en-US,en;q=0.9",
        sec_ch_ua: Some(CHROME_SEC_CH_UA), sec_ch_ua_mobile: Some("?0"),
        sec_ch_ua_platform: Some("\"Linux\""),
        sec_fetch_dest: "document", sec_fetch_mode: "navigate", sec_fetch_site: "none",
        sec_fetch_user: Some("?1"), upgrade_insecure_requests: Some("1"),
        header_order: CHROME_ORDER,
        geo_regions: &["US", "DE", "GB", "NL", "FR", "CA"],
    },
    Persona {
        key: "firefox_win", label: "Firefox 127 / Windows 11", weight: 0.05,
        ua: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:127.0) Gecko/20100101 Firefox/127.0",
        accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        accept_encoding: "gzip, deflate, br, zstd",
        accept_language: "en-US,en;q=0.5",
        sec_ch_ua: None, sec_ch_ua_mobile: None, sec_ch_ua_platform: None,
        sec_fetch_dest: "document", sec_fetch_mode: "navigate", sec_fetch_site: "none",
        sec_fetch_user: None, upgrade_insecure_requests: Some("1"),
        header_order: FIREFOX_ORDER,
        geo_regions: &["US", "DE", "GB", "FR", "CA", "NL"],
    },
    Persona {
        key: "safari_mac", label: "Safari 17.4 / macOS 15", weight: 0.10,
        ua: "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_0) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15",
        accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        accept_encoding: "gzip, deflate, br",
        accept_language: "en-US,en;q=0.9",
        sec_ch_ua: None, sec_ch_ua_mobile: None, sec_ch_ua_platform: None,
        sec_fetch_dest: "document", sec_fetch_mode: "navigate", sec_fetch_site: "none",
        sec_fetch_user: None, upgrade_insecure_requests: None,
        header_order: SAFARI_ORDER,
        geo_regions: &["US", "GB", "CA", "AU", "JP"],
    },
    Persona {
        key: "edge_win", label: "Edge 132 / Windows 11", weight: 0.05,
        ua: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 Edg/132.0.0.0",
        accept: CHROME_ACCEPT, accept_encoding: "gzip, deflate, br, zstd",
        accept_language: "en-US,en;q=0.9",
        sec_ch_ua: Some(EDGE_SEC_CH_UA), sec_ch_ua_mobile: Some("?0"),
        sec_ch_ua_platform: Some("\"Windows\""),
        sec_fetch_dest: "document", sec_fetch_mode: "navigate", sec_fetch_site: "none",
        sec_fetch_user: Some("?1"), upgrade_insecure_requests: Some("1"),
        header_order: CHROME_ORDER,
        geo_regions: &["US", "GB", "DE", "FR", "JP", "CA"],
    },
];

pub fn persona_by_key(key: &str) -> Option<&'static Persona> {
    PERSONAS.iter().find(|p| p.key == key)
}

/// 按市场占有率加权随机选择 persona (与 Python random() 相同逻辑)。
pub fn select_persona_weighted() -> &'static Persona {
    let mut rng = rand::thread_rng();
    let total: f64 = PERSONAS.iter().map(|p| p.weight).sum();
    let r: f64 = rng.gen_range(0.0..total);
    let mut cum = 0.0;
    for p in &PERSONAS {
        cum += p.weight;
        if r <= cum {
            return p;
        }
    }
    &PERSONAS[PERSONAS.len() - 1]
}

/// 选择 geo_regions 兼容指定区域(或 Accept-Language 为 en-*)的 persona。
pub fn select_persona_for_geo(geo_region: &str) -> &'static Persona {
    let upper = geo_region.to_uppercase();
    let mut en_fallback: Option<&'static Persona> = None;
    for p in &PERSONAS {
        if p.geo_regions.iter().any(|r| r.eq_ignore_ascii_case(&upper)) {
            return p;
        }
        if en_fallback.is_none() && p.accept_language.starts_with("en") {
            en_fallback = Some(p);
        }
    }
    en_fallback.unwrap_or(&PERSONAS[0])
}

// ============================================================================
// GeoCoherence — 地理一致性
// ============================================================================

/// 从 Accept-Language 主语言映射区域 (port of LANG_REGION_MAP 常用项)。
pub fn region_for_language(accept_language: &str) -> &'static str {
    let primary = accept_language.split(',').next().unwrap_or("").trim();
    match primary {
        "en-GB" => "GB", "en-CA" => "CA", "en-AU" => "AU",
        "de-DE" => "DE", "de-AT" => "AT", "de-CH" => "CH",
        "fr-FR" => "FR", "fr-CA" => "CA", "fr-BE" => "BE", "fr-CH" => "CH",
        "ja-JP" => "JP", "ko-KR" => "KR",
        "zh-CN" => "CN", "zh-TW" => "TW", "zh-HK" => "HK",
        "es-ES" => "ES", "es-MX" => "MX", "es-AR" => "AR",
        "pt-BR" => "BR", "pt-PT" => "PT",
        "it-IT" => "IT", "it-CH" => "CH",
        "nl-NL" => "NL", "nl-BE" => "BE",
        "sv-SE" => "SE", "no-NO" => "NO", "da-DK" => "DK", "fi-FI" => "FI",
        "pl-PL" => "PL", "cs-CZ" => "CZ", "sk-SK" => "SK",
        "ru-RU" => "RU", "tr-TR" => "TR",
        "ar-SA" => "SA", "he-IL" => "IL", "hi-IN" => "IN",
        "th-TH" => "TH", "vi-VN" => "VN", "id-ID" => "ID",
        _ => "US",
    }
}

fn continent(region: &str) -> &'static str {
    match region {
        "US" | "CA" | "MX" => "NA",
        "GB" | "DE" | "FR" | "IT" | "ES" | "BE" | "CH" | "AT" | "NL" | "SE" | "NO" | "DK" | "FI" | "PL" | "CZ" | "SK" | "PT" | "IE" => "EU",
        "JP" | "KR" | "CN" | "TW" | "HK" | "IN" | "TH" | "VN" | "ID" | "SG" => "AS",
        "AU" | "NZ" => "OC",
        "BR" | "AR" => "SA",
        _ => "",
    }
}

/// 0.3~1.0: Accept-Language 与 IP 区域的匹配度 (port of score_coherence)。
pub fn score_coherence(accept_language: &str, ip_region: &str) -> f64 {
    let lang_region = region_for_language(accept_language);
    if lang_region.eq_ignore_ascii_case(ip_region) {
        return 1.0;
    }
    let lc = continent(lang_region);
    let ic = continent(ip_region);
    if !lc.is_empty() && lc == ic {
        return 0.6;
    }
    0.3
}

/// 为指定区域找合理的 Accept-Language (port of language_for_region)。
pub fn language_for_region(region: &str) -> String {
    for (lang, reg) in [
        ("en-US", "US"), ("en-GB", "GB"), ("en-CA", "CA"), ("en-AU", "AU"),
        ("de-DE", "DE"), ("de-AT", "AT"), ("de-CH", "CH"),
        ("fr-FR", "FR"), ("fr-CA", "CA"), ("fr-BE", "BE"), ("fr-CH", "CH"),
        ("ja-JP", "JP"), ("ko-KR", "KR"),
        ("zh-CN", "CN"), ("zh-TW", "TW"), ("zh-HK", "HK"),
        ("es-ES", "ES"), ("es-MX", "MX"), ("es-AR", "AR"),
        ("pt-BR", "BR"), ("pt-PT", "PT"),
        ("it-IT", "IT"), ("it-CH", "CH"),
        ("nl-NL", "NL"), ("nl-BE", "BE"),
        ("sv-SE", "SE"), ("no-NO", "NO"), ("da-DK", "DK"), ("fi-FI", "FI"),
        ("pl-PL", "PL"), ("cs-CZ", "CZ"), ("sk-SK", "SK"),
        ("ru-RU", "RU"), ("tr-TR", "TR"),
        ("ar-SA", "SA"), ("he-IL", "IL"), ("hi-IN", "IN"),
        ("th-TH", "TH"), ("vi-VN", "VN"), ("id-ID", "ID"),
    ] {
        if reg.eq_ignore_ascii_case(region) {
            let base = lang.split('-').next().unwrap_or(lang);
            return format!("{},{base};q=0.9,en;q=0.5", lang);
        }
    }
    "en-US,en;q=0.9".to_string()
}

// ============================================================================
// HeaderObfuscator — 头部伪装
// ============================================================================

/// 剥离内部 NeoTrix 指纹 (port of _strip_internal_headers / INTERNAL_PATTERNS)。
pub fn strip_internal(value: &str) -> String {
    let mut v = value.to_string();
    // "nt_" 独立成词才替换 (词边界, 避免命中 "client_" 尾部), 先于 "neotrix"
    let nt_re = regex::Regex::new(r"(?i)\bnt_").unwrap();
    v = nt_re.replace_all(&v, "sys_").into_owned();
    // case-insensitive neotrix → client, 保留输入大小写
    let neotrix_re = regex::Regex::new(r"(?i)neotrix").unwrap();
    v = neotrix_re
        .replace_all(&v, |caps: &regex::Captures| {
            if caps[0].chars().all(|c| c.is_uppercase()) {
                "CLIENT".to_string()
            } else {
                "client".to_string()
            }
        })
        .into_owned();
    v = v.replace("NEOTRIX_", "CLIENT_").replace("x-neotrix-", "x-client-").replace("x-nt-", "x-client-");
    // UUID-like → zeros
    let uuid_re = regex::Regex::new(r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b").unwrap();
    v = uuid_re.replace_all(&v, "00000000-0000-0000-0000-000000000000").into_owned();
    // /Users/<name>/ → /home/user/
    let path_re = regex::Regex::new(r"/Users/[^/]+/").unwrap();
    v = path_re.replace_all(&v, "/home/user/").into_owned();
    v
}

fn random_referer(url: &str) -> String {
    let host = url::Url::parse(url).map(|u| u.host_str().unwrap_or("").to_string()).unwrap_or_default();
    let choices: Vec<String> = vec![
        "https://www.google.com/search?q=research".into(),
        "https://www.google.com/search?q=paper".into(),
        "https://www.google.com/search?q=documentation".into(),
        "https://www.google.com/search?q=api".into(),
        "https://www.google.com/search?q=tutorial".into(),
        format!("https://{host}/"),
        "https://scholar.google.com/scholar?q=machine+learning".into(),
        "https://github.com/search?q=neural+network".into(),
        "https://en.wikipedia.org/wiki/Artificial_intelligence".into(),
    ];
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx].clone()
}

/// 按 persona 构建有序伪装头 (port of build_headers + _apply_header_order)。
pub fn build_headers(
    persona: &Persona,
    url: &str,
    extra: &[(&str, &str)],
) -> Vec<(String, String)> {
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut push = |k: &str, v: String| {
        if !headers.iter().any(|(hk, _)| hk == k) {
            headers.push((k.to_string(), v));
        }
    };

    push("accept", persona.accept.to_string());
    push("accept-language", persona.accept_language.to_string());
    push("accept-encoding", persona.accept_encoding.to_string());
    if let Some(v) = persona.sec_ch_ua {
        push("sec-ch-ua", v.to_string());
    }
    if let Some(v) = persona.sec_ch_ua_mobile {
        push("sec-ch-ua-mobile", v.to_string());
    }
    if let Some(v) = persona.sec_ch_ua_platform {
        push("sec-ch-ua-platform", v.to_string());
    }
    push("sec-fetch-dest", persona.sec_fetch_dest.to_string());
    push("sec-fetch-mode", persona.sec_fetch_mode.to_string());
    push("sec-fetch-site", persona.sec_fetch_site.to_string());
    if let Some(v) = persona.sec_fetch_user {
        push("sec-fetch-user", v.to_string());
    }
    if let Some(v) = persona.upgrade_insecure_requests {
        push("upgrade-insecure-requests", v.to_string());
    }
    push("user-agent", persona.ua.to_string());
    push("referer", random_referer(url));

    // extra headers (不得覆盖关键身份头)
    for (k, v) in extra {
        let kl = k.to_lowercase();
        if !["user-agent", "accept", "accept-language", "accept-encoding", "referer"].contains(&kl.as_str()) {
            push(&kl, (*v).to_string());
        }
    }

    // strip internal
    let cleaned: Vec<(String, String)> = headers
        .into_iter()
        .map(|(k, v)| (strip_internal(&k), strip_internal(&v)))
        .collect();

    // apply header_order
    let ordered = apply_header_order(cleaned, persona);
    ordered
}

fn apply_header_order(headers: Vec<(String, String)>, persona: &Persona) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for key in persona.header_order {
        let kl = key.trim_start_matches(':').to_lowercase();
        if let Some((k, v)) = headers.iter().find(|(hk, _)| *hk == kl) {
            if !seen.contains(&kl) {
                result.push((k.clone(), v.clone()));
                seen.insert(kl);
            }
        }
    }
    for (k, v) in headers {
        if !seen.contains(&k) {
            result.push((k.clone(), v));
            seen.insert(k);
        }
    }
    result
}

// ============================================================================
// TimingObfuscator — 时序抖动
// ============================================================================

#[derive(Default)]
pub struct TimingObfuscator {
    last_request: Option<Instant>,
}

impl TimingObfuscator {
    /// 计算人类浏览节奏的等待秒数 (gauss(2.5, 1.0) clamp [0.3, 10], 减去已逝时间)。
    pub fn next_wait_secs(&mut self) -> f64 {
        let now = Instant::now();
        let elapsed = self.last_request.map(|t| now.duration_since(t).as_secs_f64()).unwrap_or(0.0);
        self.last_request = Some(now);
        if elapsed == 0.0 {
            return 0.0;
        }
        let target = gauss(2.5, 1.0).clamp(0.3, 10.0);
        (target - elapsed).max(0.0)
    }

    /// 页面渲染抖动 (ms), gauss(200, 100)。
    pub fn page_load_jitter_ms(&self) -> f64 {
        gauss(200.0, 100.0)
    }
}

fn gauss(mean: f64, std: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u1: f64 = rng.gen_range(1e-9..1.0);
    let u2: f64 = rng.gen_range(0.0..1.0);
    let z = (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos();
    mean + std * z
}

// ============================================================================
// IdentityPool — 持久身份池 (KB 表)
// ============================================================================

pub fn ensure_tables(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(COMM_DB_TABLES)
}

/// 从 persona 模板创建一个新身份实例 (md5 派生确定性 id)。
pub fn create_id(conn: &Connection, persona_key: &str, now: f64) -> rusqlite::Result<String> {
    ensure_tables(conn)?;
    use md5::{Digest, Md5};
    let mut h = Md5::new();
    h.update(format!("{persona_key}:{now}:{}", rand::random::<u64>()).as_bytes());
    let hex: String = h.finalize().iter().map(|b| format!("{:02x}", b)).collect();
    let pid = hex[..16].to_string();
    conn.execute(
        "INSERT OR IGNORE INTO identity_pool (id, persona_key, created_at) VALUES (?1, ?2, ?3)",
        params![pid, persona_key, now],
    )?;
    Ok(pid)
}

pub fn record_success(conn: &Connection, persona_id: &str, latency_ms: f64) -> rusqlite::Result<()> {
    ensure_tables(conn)?;
    conn.execute(
        "UPDATE identity_pool SET last_used=?, success_count=success_count+1 WHERE id=?",
        params![now_epoch(), persona_id],
    )?;
    if let Ok(key) = conn.query_row(
        "SELECT persona_key FROM identity_pool WHERE id=?1",
        [persona_id],
        |r| r.get::<_, String>(0),
    ) {
        conn.execute(
            "INSERT INTO persona_stats (persona_key, total_uses, total_success, avg_latency_ms)
             VALUES (?1, 1, 1, ?2)
             ON CONFLICT(persona_key) DO UPDATE SET
                total_uses = total_uses + 1,
                total_success = total_success + 1,
                avg_latency_ms = (avg_latency_ms * (total_uses) + ?2) / (total_uses + 1)",
            params![key, latency_ms],
        )?;
    }
    Ok(())
}

pub fn record_failure(
    conn: &Connection,
    persona_id: &str,
    domain: &str,
    status_code: i64,
    error: &str,
    proxy_used: &str,
) -> rusqlite::Result<()> {
    ensure_tables(conn)?;
    let now = now_epoch();
    conn.execute(
        "UPDATE identity_pool SET last_used=?, fail_count=fail_count+1 WHERE id=?",
        params![now, persona_id],
    )?;
    if let Ok(key) = conn.query_row(
        "SELECT persona_key FROM identity_pool WHERE id=?1",
        [persona_id],
        |r| r.get::<_, String>(0),
    ) {
        conn.execute(
            "INSERT INTO persona_stats (persona_key, total_uses, total_success, total_fail)
             VALUES (?1, 1, 0, 1)
             ON CONFLICT(persona_key) DO UPDATE SET
                total_uses = total_uses + 1,
                total_fail = total_fail + 1",
            params![key],
        )?;
    }
    conn.execute(
        "INSERT INTO failure_log (timestamp, persona_id, domain, status_code, error, proxy_used)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![now, persona_id, domain, status_code, error, proxy_used],
    )?;
    Ok(())
}

pub fn pool_stats(conn: &Connection) -> Result<serde_json::Value, rusqlite::Error> {
    ensure_tables(conn)?;
    let total_identities: i64 = conn.query_row("SELECT COUNT(*) FROM identity_pool", [], |r| r.get(0)).unwrap_or(0);
    let failures_24h: i64 = conn.query_row(
        "SELECT COUNT(*) FROM failure_log WHERE timestamp >= ?1",
        params![now_epoch() - 86_400.0],
        |r| r.get(0),
    )
    .unwrap_or(0);
    let mut by_persona = serde_json::Map::new();
    let mut stmt = conn.prepare("SELECT persona_key, total_uses, total_success, total_fail, avg_latency_ms FROM persona_stats")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?, r.get::<_, f64>(4)?,
        ))
    })?;
    for row in rows.flatten() {
        let (key, uses, success, fail, latency) = row;
        by_persona.insert(key, serde_json::json!({"uses": uses, "success": success, "fail": fail, "avg_latency_ms": latency}));
    }
    Ok(serde_json::json!({
        "total_identities": total_identities,
        "failures_24h": failures_24h,
        "by_persona": by_persona,
    }))
}

fn now_epoch() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

// ============================================================================
// CommRouter — 统一入口
// ============================================================================

#[derive(Debug, Default, Clone)]
pub struct RouterResult {
    pub status: u16,
    pub body: Option<String>,
    pub error: String,
    pub latency_ms: f64,
    pub persona_used: String,
    pub identity_id: String,
}

/// 通过伪装层执行 GET (persona 选择 + 头构建 + 成功/失败记录)。
/// 失败 (403/429/连接失败) 时用另一 persona 重试一次。
pub async fn fetch(
    conn: &Connection,
    client: &reqwest::Client,
    url: &str,
    persona_key: &str,
    extra_headers: &[(&str, &str)],
    timeout: Duration,
) -> RouterResult {
    ensure_tables(conn).ok();
    let now = now_epoch();
    let persona = if persona_key.is_empty() {
        select_persona_weighted()
    } else {
        persona_by_key(persona_key).unwrap_or_else(select_persona_weighted)
    };
    let identity_id = create_id(conn, persona.key, now).unwrap_or_else(|_| "anon".to_string());
    let headers = build_headers(persona, url, extra_headers);

    let mut result = execute(client, url, &headers, timeout).await;
    result.persona_used = persona.key.to_string();
    result.identity_id = identity_id.clone();

    if result.status == 0 || result.status == 403 || result.status == 429 {
        record_failure(conn, &identity_id, &host_of(url), result.status as i64, &result.error, "").ok();
    } else {
        record_success(conn, &identity_id, result.latency_ms).ok();
    }

    // retry with different persona
    if result.status == 403 || result.status == 429 || result.status == 0 {
        let alt = select_persona_weighted();
        if alt.key != persona.key {
            let alt_id = create_id(conn, alt.key, now).unwrap_or_else(|_| "anon".to_string());
            let headers2 = build_headers(alt, url, extra_headers);
            let r2 = execute(client, url, &headers2, timeout).await;
            if r2.status != 403 && r2.status != 429 && r2.status != 0 {
                let mut r2 = r2;
                r2.persona_used = alt.key.to_string();
                r2.identity_id = alt_id.clone();
                record_success(conn, &alt_id, r2.latency_ms).ok();
                return r2;
            }
            record_failure(conn, &alt_id, &host_of(url), r2.status as i64, &r2.error, "").ok();
        }
    }

    result
}

async fn execute(client: &reqwest::Client, url: &str, headers: &[(String, String)], timeout: Duration) -> RouterResult {
    let start = std::time::Instant::now();
    let mut req = client.get(url).timeout(timeout);
    for (k, v) in headers {
        req = req.header(k, v);
    }
    match req.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body = resp.text().await.ok();
            RouterResult {
                status,
                body,
                error: String::new(),
                latency_ms: start.elapsed().as_millis() as f64,
                persona_used: String::new(),
                identity_id: String::new(),
            }
        }
        Err(e) => RouterResult {
            status: 0,
            body: None,
            error: e.to_string(),
            latency_ms: start.elapsed().as_millis() as f64,
            persona_used: String::new(),
            identity_id: String::new(),
        },
    }
}

fn host_of(url: &str) -> String {
    url::Url::parse(url).map(|u| u.host_str().unwrap_or("").to_string()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personas_catalog() {
        assert_eq!(personas().len(), 6);
        assert_eq!(personas()[0].key, "chrome_win");
        assert!(personas().iter().all(|p| p.weight > 0.0));
        let total: f64 = personas().iter().map(|p| p.weight).sum();
        assert!((total - 0.73).abs() < 1e-9, "weights match python: {total}");
    }

    #[test]
    fn test_select_persona_weighted_in_range() {
        for _ in 0..20 {
            let p = select_persona_weighted();
            assert!(persona_by_key(p.key).is_some());
        }
    }

    #[test]
    fn test_region_for_language() {
        assert_eq!(region_for_language("en-US,en;q=0.9"), "US");
        assert_eq!(region_for_language("de-DE,de;q=0.9"), "DE");
        assert_eq!(region_for_language("ja-JP,ja;q=0.9"), "JP");
        assert_eq!(region_for_language("unknown-XX"), "US");
    }

    #[test]
    fn test_score_coherence() {
        assert_eq!(score_coherence("en-US,en;q=0.9", "US"), 1.0);
        assert_eq!(score_coherence("de-DE,de;q=0.9", "DE"), 1.0);
        assert!((score_coherence("en-US,en;q=0.9", "CA") - 0.6).abs() < 1e-9); // same continent
        assert!((score_coherence("en-US,en;q=0.9", "JP") - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_language_for_region_roundtrip() {
        assert_eq!(region_for_language(&language_for_region("JP")), "JP");
        assert_eq!(region_for_language(&language_for_region("DE")), "DE");
        assert_eq!(language_for_region("ZZ"), "en-US,en;q=0.9");
    }

    #[test]
    fn test_build_headers_ordered_and_coherent() {
        let persona = persona_by_key("chrome_win").unwrap();
        let headers = build_headers(persona, "https://arxiv.org/abs/1706.03762", &[]);
        let map: std::collections::HashMap<String, String> = headers.iter().cloned().collect();
        assert_eq!(map.get("user-agent").unwrap(), &persona.ua);
        assert_eq!(map.get("accept-language").unwrap(), "en-US,en;q=0.9");
        assert!(map.contains_key("sec-ch-ua"));
        assert!(map.contains_key("sec-fetch-dest"));
        assert!(map.contains_key("referer"));
        // order: accept-encoding before user-agent before accept (chrome order)
        let order: Vec<&String> = headers.iter().map(|(k, _)| k).collect();
        let pos_ae = order.iter().position(|k| *k == "accept-encoding").unwrap();
        let pos_ua = order.iter().position(|k| *k == "user-agent").unwrap();
        let pos_accept = order.iter().position(|k| *k == "accept").unwrap();
        assert!(pos_ae < pos_ua && pos_ua < pos_accept);
    }

    #[test]
    fn test_firefox_no_sec_ch_ua() {
        let persona = persona_by_key("firefox_win").unwrap();
        let headers = build_headers(persona, "https://example.com", &[]);
        assert!(!headers.iter().any(|(k, _)| k == "sec-ch-ua"));
    }

    #[test]
    fn test_strip_internal_patterns() {
        assert_eq!(strip_internal("NeoTrixBot/1.0"), "clientBot/1.0");
        assert_eq!(strip_internal("x-neotrix-session: abc"), "x-client-session: abc");
        assert_eq!(strip_internal("x-nt-key: secret"), "x-client-key: secret");
        assert_eq!(strip_internal("NEOTRIX_TOKEN"), "CLIENT_TOKEN");
        assert_eq!(strip_internal("/Users/alice/data"), "/home/user/data");
        assert_eq!(strip_internal("id 550e8400-e29b-41d4-a716-446655440000 end"), "id 00000000-0000-0000-0000-000000000000 end");
        assert_eq!(strip_internal("nt_foo_bar"), "sys_foo_bar");
    }

    #[test]
    fn test_extra_headers_not_override_identity() {
        let persona = persona_by_key("chrome_mac").unwrap();
        let headers = build_headers(
            persona,
            "https://example.com",
            &[("user-agent", "FAKE"), ("x-custom", "1")],
        );
        let map: std::collections::HashMap<String, String> = headers.iter().cloned().collect();
        assert_eq!(map.get("user-agent").unwrap(), &persona.ua);
        assert_eq!(map.get("x-custom").unwrap(), "1");
    }

    #[test]
    fn test_timing_jitter_bounds() {
        let mut t = TimingObfuscator::default();
        assert_eq!(t.next_wait_secs(), 0.0); // first call no wait
        for _ in 0..20 {
            let w = t.next_wait_secs();
            assert!((0.0..=10.0).contains(&w));
        }
        let j = t.page_load_jitter_ms();
        assert!(j > 0.0 && j < 2000.0);
    }

    #[test]
    fn test_identity_pool_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        let id = create_id(&conn, "chrome_win", 1000.0).unwrap();
        let id2 = create_id(&conn, "chrome_win", 1000.1).unwrap();
        assert_ne!(id, id2);
        record_success(&conn, &id, 123.0).unwrap();
        record_failure(&conn, &id, "example.com", 429, "rate limited", "").unwrap();
        let stats = pool_stats(&conn).unwrap();
        assert_eq!(stats["total_identities"], 2);
        let by_persona = stats["by_persona"].as_object().unwrap();
        let chrome = by_persona.get("chrome_win").unwrap();
        assert_eq!(chrome["uses"], 2);
        assert_eq!(chrome["success"], 1);
        assert_eq!(chrome["fail"], 1);
        assert!((chrome["avg_latency_ms"].as_f64().unwrap() - 123.0).abs() < 1e-6);
    }
}
