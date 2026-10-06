#![forbid(unsafe_code)]

//! URL 主机判定原语 —— 供各层做「这个 URL 属于哪个站点」
//!
//! # 为何抽到这里（L0）
//!
//! 审计（2026-10-03）发现同一类缺陷散落 11 个文件、12 处：
//!
//! ```text
//! if url.contains("github.com") { … }
//! else if lower.contains("bilibili.com") { … }
//! ```
//!
//! 裸 `contains` 会把下面这些**全部误判**：
//!
//! ```text
//! https://phishing-github.com/        ← 钓鱼
//! https://github.com.evil.net/        ← 前缀伪装
//! https://github.com@evil.net/        ← userinfo 伪装
//! ```
//!
//! ⛔ 且这不是「洁癖」问题：`AbsorbSource::from_url`（`osint/mod.rs`）
//! 与 `PlaylistSource::from_url`（`source/playlist.rs`）**据此选择抓取
//! 处理器**。认领错 ⇒ 用错适配器去请求攻击者控制的域名。
//!
//! 本模块是**纯函数**（无 IO、无状态），故放 L0 substrate ——
//! L1/L2/L4/L5 都可调用，不产生跨层违规。
//!
//! # 与既有实现的关系
//!
//! [`crate::l2_perception::nt_world::social_access::nt_catalog`] 早前已
//! 内联了一份同逻辑。现改为**委托到本模块**，避免两份判定分叉 ——
//! 这正是本仓反复吃亏的「同一件事两处实现」。

/// 从 URL 提取**主机名**（小写、不含 userinfo / 端口 / path）。
///
/// ⛔ **不**用 `url::Url::parse` 作为唯一路径：本仓多处拿到的是
/// 残缺或非绝对 URL（`github.com/owner/repo`、`//host/path`），
/// 解析失败时若回退到"整串 contains"就等于退回原缺陷。
/// 故此处做**容错的词法提取**，对残缺输入仍给出正确 host。
///
/// 处理的形态：
/// - `https://user:pass@Host.Example:8443/a/b?c=d#e` → `host.example`
/// - `github.com/owner/repo`                      → `github.com`
/// - `//www.x.com/home`                            → `www.x.com`
/// - `HTTPS://X.COM`                               → `x.com`
pub fn host_of(url: &str) -> String {
    let mut rest = url.trim();

    // 1) 去 scheme（找 `://`；无 scheme 时 rest 不变）
    if let Some((_, after)) = rest.split_once("://") {
        rest = after;
    } else if let Some(after) = rest.strip_prefix("//") {
        rest = after;
    }

    // 2) 去 path / query / fragment
    rest = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(rest);

    // 3) 去 userinfo（`user:pass@host`）—— 取最后一个 `@` 之后
    if let Some((_, host)) = rest.rsplit_once('@') {
        rest = host;
    }

    // 4) 去端口 —— 只在冒号后是**纯数字**时才切，否则可能是 IPv6 字面量
    if let Some((head, port)) = rest.rsplit_once(':') {
        if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) {
            rest = head;
        }
    }

    // 5) 小写化（DNS 大小写不敏感）
    rest.to_ascii_lowercase()
}

/// 判断 `host` 是否真属于 `domain`。
///
/// # 规则（刻意保守）
///
/// `domain` 匹配当且仅当：
/// - `host == domain`，或
/// - `host` 以 `domain` 结尾**且**其前缀以 `.` 结束（子域边界）。
///
/// # ⛔ 明确**不**匹配的情形（这些正是裸 contains 会误判的）
///
/// | host | domain | 结果 | 理由 |
/// |---|---|---|---|
/// | `notx.com` | `x.com` | ❌ | `x.com` 只是子串，不是后缀 |
/// | `phishing-x.com` | `x.com` | ❌ | 同上 |
/// | `x.com.evil.net` | `x.com` | ❌ | `x.com` 在**前**缀侧，域名归 `evil.net` |
/// | `www.x.com` | `x.com` | ✅ | 合法子域 |
/// | `x.com:8080` | `x.com` | ✅ | 端口已由 [`host_of`] 剥离 |
pub fn host_matches(host: &str, domain: &str) -> bool {
    let domain = domain.trim().trim_start_matches('.').to_ascii_lowercase();
    if domain.is_empty() || host.is_empty() {
        return false;
    }
    let host = host.to_ascii_lowercase();
    if host == domain {
        return true;
    }
    match host.strip_suffix(&domain) {
        Some(prefix) => prefix.ends_with('.'),
        None => false,
    }
}

/// 便捷判定：URL 是否属于该域名。
///
/// 这是**应当替换裸 `contains` 的那个函数**。
pub fn url_matches_domain(url: &str, domain: &str) -> bool {
    host_matches(&host_of(url), domain)
}

/// URL 是否属于给定域名中的**任一个**。
pub fn url_matches_any(url: &str, domains: &[&str]) -> bool {
    let host = host_of(url);
    domains.iter().any(|d| host_matches(&host, d))
}

/// URL 是否属于给定域名中的**任一个**（`String` 版本，避免调用方分配）。
pub fn url_matches_any_owned(url: &str, domains: &[String]) -> bool {
    let host = host_of(url);
    domains.iter().any(|d| host_matches(&host, d))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── host_of 提取 ──────────────────────────────────────────

    #[test]
    fn extracts_host_from_full_urls() {
        assert_eq!(host_of("https://github.com/owner/repo"), "github.com");
        assert_eq!(host_of("http://x.com/home"), "x.com");
        assert_eq!(host_of("https://www.reddit.com/r/rust"), "www.reddit.com");
    }

    #[test]
    fn strips_userinfo_and_port() {
        // ⛔ 这两种是裸 contains 最容易误判的伪装形态
        assert_eq!(host_of("https://user:pass@github.com/x"), "github.com");
        assert_eq!(host_of("https://github.com@evil.net/x"), "evil.net");
        assert_eq!(host_of("https://x.com:8443/home"), "x.com");
        assert_eq!(host_of("https://user@x.com:443/a"), "x.com");
    }

    #[test]
    fn strips_query_and_fragment() {
        assert_eq!(host_of("https://x.com/home?a=1#frag"), "x.com");
        assert_eq!(host_of("https://x.com#top"), "x.com");
    }

    #[test]
    fn is_case_insensitive() {
        assert_eq!(host_of("HTTPS://X.COM/Home"), "x.com");
        assert_eq!(host_of("WWW.Reddit.COM"), "www.reddit.com");
    }

    #[test]
    fn handles_schemeless_and_protocol_relative() {
        // 残缺输入在真实数据里很常见（`github.com/owner/repo`）
        assert_eq!(host_of("github.com/owner/repo"), "github.com");
        assert_eq!(host_of("//www.x.com/home"), "www.x.com");
    }

    #[test]
    fn tolerates_empty_and_garbage_without_panicking() {
        assert_eq!(host_of(""), "");
        assert_eq!(host_of("   "), "");
        assert_eq!(host_of("not a url at all"), "not a url at all");
        assert_eq!(host_of("://"), "");
    }

    // ── host_matches 域名边界（核心安全语义）───────────────────

    #[test]
    fn matches_exact_and_subdomain() {
        assert!(host_matches("x.com", "x.com"));
        assert!(host_matches("www.x.com", "x.com"));
        assert!(host_matches("a.b.x.com", "x.com"));
        assert!(host_matches("X.COM", "x.com"), "case-insensitive");
    }

    #[test]
    fn rejects_lookalike_domains() {
        // 这三条就是裸 contains 会误判的全部形态
        assert!(!host_matches("notx.com", "x.com"));
        assert!(!host_matches("phishing-x.com", "x.com"));
        assert!(!host_matches("x.com.evil.net", "x.com"));
        assert!(!host_matches("myreddit.com", "reddit.com"));
        assert!(!host_matches("github.com.evil.io", "github.com"));
    }

    #[test]
    fn empty_inputs_never_match() {
        assert!(!host_matches("", "x.com"));
        assert!(!host_matches("x.com", ""));
        assert!(!host_matches("", ""));
    }

    #[test]
    fn leading_dot_in_pattern_is_tolerated() {
        // 调用方常写 ".x.com" 表示子域
        assert!(host_matches("www.x.com", ".x.com"));
        assert!(host_matches("x.com", ".x.com"));
    }

    // ── url_matches_domain：应当替换 contains 的那个 ────────────

    #[test]
    fn url_matching_is_host_based_not_substring() {
        assert!(url_matches_domain("https://github.com/a/b", "github.com"));
        // ⛔ 以下三条裸 contains 都会返回 true
        assert!(!url_matches_domain("https://github.com.evil.net/", "github.com"));
        assert!(!url_matches_domain("https://notgithub.com/", "github.com"));
        // ⚠️ `raw.githubusercontent.com` 是 `githubusercontent.com` 的子域，
        //    **不是** `github.com` 的 —— 裸 contains 会误判，这里如实为 false。
        assert!(!url_matches_domain("https://raw.githubusercontent.com/x", "github.com"));
    }

    #[test]
    fn any_variant_behaves_like_iteration() {
        assert!(url_matches_any("https://youtu.be/abc", &["youtube.com", "youtu.be"]));
        assert!(!url_matches_any("https://vimeo.com/1", &["youtube.com", "youtu.be"]));
        assert!(url_matches_any_owned(
            "https://b23.tv/xyz",
            &["bilibili.com".to_string(), "b23.tv".to_string()]
        ));
    }

    #[test]
    fn empty_domain_list_never_matches() {
        assert!(!url_matches_any("https://x.com/", &[]));
    }
}