//! 平台目录 — 社交平台知识的**单一真源**
//!
//! # ⭐ 审计发现的同类问题（2026-10-03）
//!
//! 上一轮把登录通用化了，但审计全仓后发现「平台知识」散在**三处互不相通**的地方：
//!
//! | 位置 | 存了什么 | 问题 |
//! |---|---|---|
//! | `traits::SocialPlatform` | 6 个变体 + `Other(String)` | ⛔ **封闭枚举**；`all()` 只返回 6 个，`Other` 永不出现在遍历里 |
//! | `channel::Channel::can_handle` | `match self.platform { … }` 穷举 6 个 URL 模式 | ⛔ 加平台**必须改这个 match**，否则新平台无 URL 路由 |
//! | `nt_login::default_registry` | 5 个登录目标 | ⚠️ 与上面两处**各维护一份**，可漂移 |
//!
//! ⇒ **「加平台不用改代码」此前只做到一半**：登录不用改了，但
//! `SocialPlatform` 枚举 + `can_handle` 的 match 仍在封闭。
//!
//! # 本模块的解法
//!
//! 一个 [`PlatformCatalog`]，把「这个平台是什么」的全部知识收在一处：
//! id、显示名、URL 模式、登录探针、渠道名。
//!
//! - [`SocialPlatform::Other(String)`] 之外的平台走 `catalog` 查表，
//!   **不再需要改枚举**；
//! - `can_handle` 改为**查表**（O(n) over catalog），不再 `match` 穷举；
//! - 登录目标与 URL 模式**同源** —— 不可能再漂移。
//!
//! # ⚠️ 兼容性：既有枚举变体保留
//!
//! [`SocialPlatform`] 的 6 个变体**不删**（它们是既有 API 的一部分，
//! `traits.rs` 的 `From`/`as_str` 依赖它们）。本目录提供**并行的**
//! 字符串键路径，让新平台无需触碰枚举。
//! ⛔ 但 `SocialPlatform::all()` 仍只含那 6 个 —— 用
//! [`PlatformCatalog::all_ids`] 取完整列表，不要用 `all()`。

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::nt_login::SuccessProbe;
use super::traits::SocialPlatform;

/// 一个平台的完整描述。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformSpec {
    /// 稳定标识（渠道名、cookie 域键、CLI 参数）。
    pub id: String,
    /// 人类可读名。
    pub display_name: String,
    /// 归属该平台的 URL 片段（任一命中即认领）。
    ///
    /// ⛔ 必须是**域名或路径片段**，不是完整 URL —— 否则
    /// `https://notyoutube.com.evil/` 这类构造会被误认领。
    pub url_patterns: Vec<String>,
    /// 登录成功判据（无则表示该平台暂不支持交互登录）。
    pub login_probes: Vec<SuccessProbe>,
    /// 登录页 URL。
    pub login_url: Option<String>,
    /// 登录等待超时。
    pub login_timeout: Option<Duration>,
    /// cookie 存储文件名。
    pub cookie_file: Option<String>,
    /// 该平台是否必须有会话才能工作。
    ///
    /// ⭐ `false` 表示「不登录也能用（如只读公开数据）」，
    /// 避免把「没登录」报成致命错误。
    pub requires_session: bool,
}

impl PlatformSpec {
    /// 是否支持交互登录。
    pub fn supports_login(&self) -> bool {
        !self.login_probes.is_empty()
    }

    /// 本平台对应的 [`SocialPlatform`](super::traits::SocialPlatform)。
    ///
    /// ⭐ 用于把目录条目关联到既有渠道。⚠️ `x` 映射回 `Twitter` ——
    /// 历史渠道名是 `"twitter"` 但枚举变体叫 `Twitter`，二者都保留。
    pub fn platform_id(&self) -> SocialPlatform {
        SocialPlatform::from_str(&self.id)
    }
}

/// 平台目录。
#[derive(Debug, Clone, Default)]
pub struct PlatformCatalog {
    specs: HashMap<String, PlatformSpec>,
}

impl PlatformCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个平台（按 `id` 覆盖）。
    ///
    /// ⭐ 这是「加平台」的唯一入口 —— 不改枚举、不改 `can_handle`、不改登录流程。
    pub fn register(&mut self, spec: PlatformSpec) {
        self.specs.insert(spec.id.clone(), spec);
    }

    pub fn get(&self, id: &str) -> Option<&PlatformSpec> {
        self.specs.get(id)
    }

    /// ⭐ 全部平台 id（**排序稳定**）。
    ///
    /// ⛔ 不要用 `SocialPlatform::all()` —— 它只含 6 个枚举变体，
    /// 经 `register` 加入的平台不在其中。HashMap 迭代序不稳定，
    /// 故此处显式排序以免造成 diff 噪音。
    pub fn all_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.specs.keys().map(String::as_str).collect();
        ids.sort_unstable();
        ids
    }

    pub fn all(&self) -> Vec<&PlatformSpec> {
        self.all_ids()
            .into_iter()
            .filter_map(|id| self.specs.get(id))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    /// 认领一个 URL。
    ///
    /// ⭐ 取代旧 `Channel::can_handle` 的 `match` 穷举。
    ///
    /// # ⚠️ 判据必须是域名/路径片段，不是 `contains` 裸匹配
    ///
    /// ⛔ `url.contains("x.com")` 会把
    /// `https://phishing-x.com.attacker.net/` 认领成 X。
    /// 故要求「片段两侧是非域名字符或字符串边界」——
    /// 即片段前是 `/`、`.`、`:`, 或位于 host 起始；
    /// 片段后是 `/`、`.`、`?`、`#`、端口，或位于 host 末尾。
    pub fn match_url(&self, url: &str) -> Option<&PlatformSpec> {
        // 提取得 host（去 scheme、去 userinfo、去 path/query/fragment）
        let after_scheme = match url.split_once("://") {
            Some((_, rest)) => rest,
            None => url,
        };
        let host_and_path = after_scheme
            .split(['/', '?', '#'])
            .next()
            .unwrap_or(after_scheme);
        // 去 userinfo（`user:pass@host`）
        let without_userinfo = match host_and_path.rsplit_once('@') {
            Some((_, h)) => h,
            None => host_and_path,
        };
        // ⛔ 去端口：`x.com:8080` 的 host 是 `x.com`，不是 `x.com:8080`。
        //    漏掉这一步会让 `https://x.com:8080/` 完全无法认领 ——
        //    自测 `url_parsing_handles_userinfo_and_ports` 抓到的。
        //    ⚠️ 必须**从右**切且只切一次：`::1` 这类 IPv6 字面量
        //    含多个冒号，右切一次得到 `[::1]`，已足够（平台不会用 IPv6 匹配）。
        let without_port = match without_userinfo.rsplit_once(':') {
            Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() => h,
            _ => without_userinfo,
        };
        let host = without_port
        .to_ascii_lowercase();

        for id in self.all_ids() {
            let spec = &self.specs[id];
            for pattern in &spec.url_patterns {
                let p = pattern.to_ascii_lowercase();
                if matches_domain_fragment(&host, &p) {
                    return Some(spec);
                }
            }
        }
        None
    }

    /// 由 cookie 域键（= id）构造 cookie 路径。
    ///
    /// ⛔ 无 HOME 时返回错误，**不猜路径**（否则凭据会落到意外位置）。
    pub fn cookie_path(&self, id: &str) -> Result<std::path::PathBuf, String> {
        let spec = self
            .specs
            .get(id)
            .ok_or_else(|| format!("no platform registered for `{}`", id))?;
        let file = spec
            .cookie_file
            .clone()
            .unwrap_or_else(|| format!("{}.json", spec.id));
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .ok_or_else(|| {
                "neither HOME nor USERPROFILE is set; refusing to guess where to \
                 persist credentials"
                    .to_string()
            })?;
        Ok(std::path::Path::new(&home)
            .join(".neotrix")
            .join("cookies")
            .join(file))
    }

    /// 凭据环境变量名（若该平台需要）。
    ///
    /// ⭐ **统一规则**：`NEOTRIX_<大写ID>_AUTH_TOKEN`。
    /// 此前只有 `NEOTRIX_X_AUTH_TOKEN`，加平台就要加一个常量。
    pub fn env_var_names(id: &str) -> (String, String) {
        let upper: String = id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' })
            .collect();
        (
            format!("NEOTRIX_{}_AUTH_TOKEN", upper),
            format!("NEOTRIX_{}_CT0", upper),
        )
    }
}

/// 判断 host 是否真的包含该域名片段。
///
/// # 规则（刻意保守）
///
/// `x.com` 应匹配 `x.com` / `www.x.com` / `x.com.evil` 的**前缀侧**不对，
/// 但 `notx.com` 不该命中。判据：
///
/// - 片段出现在 host **起始**，或
/// - 片段前一个字符是 `.`（子域边界）。
///
/// ⚠️ 只查 host，不查 path —— `url_patterns` 里带 `/` 的（如 `youtu.be`
/// 实际是域名）由调用方保证语义。此函数按域名处理。
fn matches_domain_fragment(host: &str, pattern: &str) -> bool {
    if pattern.is_empty() || host.is_empty() {
        return false;
    }
    if host == pattern {
        return true;
    }
    // 片段必须是 host 的**后缀**，或被 `.` 包围的子域
    if let Some(rest) = host.strip_suffix(pattern) {
        // rest 非空 ⇒ 前面必须有边界字符；rest 为空 ⇒ 上面已处理
        return rest.ends_with('.');
    }
    // 或片段作为子域前缀：`x.com.evil.net` —— ⛔ **不**认为命中，
    // 因为那是攻击者控制的域名。故此处刻意不匹配。
    false
}

/// 内置平台目录。
///
/// ⭐ URL 模式全部取自实测的既有 `can_handle` 实现，未凭空添加。
pub fn default_catalog() -> PlatformCatalog {
    let mut c = PlatformCatalog::new();

    c.register(PlatformSpec {
        id: "x".into(),
        display_name: "X (Twitter)".into(),
        // ⚠️ `twitter.com` 与 `x.com` 并列（X 未登录时会 302 到它）
        url_patterns: vec!["x.com".into(), "twitter.com".into()],
        login_probes: vec![
            // ⭐ 只看存在性，不读值（HttpOnly）
            SuccessProbe::CookiePresent { name: "auth_token".into() },
            SuccessProbe::UrlContains { needle: "/home".into() },
        ],
        login_url: Some("https://x.com/login".into()),
        login_timeout: Some(Duration::from_secs(300)),
        cookie_file: Some("x.json".into()),
        requires_session: true,
    });

    c.register(PlatformSpec {
        id: "reddit".into(),
        display_name: "Reddit".into(),
        url_patterns: vec!["reddit.com".into(), "redd.it".into()],
        login_probes: vec![
            SuccessProbe::CookiePresent { name: "reddit_session".into() },
        ],
        login_url: Some("https://www.reddit.com/login".into()),
        login_timeout: Some(Duration::from_secs(300)),
        cookie_file: Some("reddit.json".into()),
        requires_session: true,
    });

    c.register(PlatformSpec {
        id: "instagram".into(),
        display_name: "Instagram".into(),
        url_patterns: vec!["instagram.com".into()],
        // ⚠️ 未提供探针 ⇒ `supports_login()` 为 false。
        //    据实留空而非编造 cookie 名 —— instagram 的会话 cookie
        //    机制未在本仓实测过。
        login_probes: vec![],
        login_url: None,
        login_timeout: None,
        cookie_file: None,
        requires_session: true,
    });

    c.register(PlatformSpec {
        id: "tiktok".into(),
        display_name: "TikTok".into(),
        url_patterns: vec!["tiktok.com".into(), "vm.tiktok.com".into()],
        login_probes: vec![],
        login_url: None,
        login_timeout: None,
        cookie_file: None,
        requires_session: true,
    });

    c.register(PlatformSpec {
        id: "youtube".into(),
        display_name: "YouTube".into(),
        url_patterns: vec!["youtube.com".into(), "youtu.be".into()],
        login_probes: vec![],
        login_url: None,
        login_timeout: None,
        cookie_file: None,
        // ⭐ YouTube 靠 yt-dlp **零配置**可用（实测 `yt-dlp --version` rc=0）
        requires_session: false,
    });

    c.register(PlatformSpec {
        id: "linkedin".into(),
        display_name: "LinkedIn".into(),
        url_patterns: vec!["linkedin.com".into()],
        login_probes: vec![],
        login_url: None,
        login_timeout: None,
        cookie_file: None,
        requires_session: true,
    });

    // ── 非社交但既有渠道（保持与 default_channels() 一致）────────────
    c.register(PlatformSpec {
        id: "github".into(),
        display_name: "GitHub".into(),
        url_patterns: vec!["github.com".into()],
        login_probes: vec![
            SuccessProbe::CookiePresent { name: "user_session".into() },
        ],
        login_url: Some("https://github.com/login".into()),
        login_timeout: Some(Duration::from_secs(300)),
        cookie_file: Some("github.json".into()),
        requires_session: true,
    });

    c.register(PlatformSpec {
        id: "bilibili".into(),
        display_name: "Bilibili".into(),
        url_patterns: vec!["bilibili.com".into(), "b23.tv".into()],
        login_probes: vec![
            SuccessProbe::CookiePresent { name: "SESSDATA".into() },
        ],
        login_url: Some("https://passport.bilibili.com/login".into()),
        login_timeout: Some(Duration::from_secs(300)),
        cookie_file: Some("bilibili.json".into()),
        requires_session: true,
    });

    c.register(PlatformSpec {
        id: "zhihu".into(),
        display_name: "知乎".into(),
        url_patterns: vec!["zhihu.com".into()],
        login_probes: vec![
            SuccessProbe::CookiePresent { name: "z_c0".into() },
        ],
        login_url: Some("https://www.zhihu.com/signin".into()),
        login_timeout: Some(Duration::from_secs(300)),
        cookie_file: Some("zhihu.json".into()),
        requires_session: true,
    });

    c.register(PlatformSpec {
        id: "web".into(),
        display_name: "通用网页".into(),
        url_patterns: vec![],
        login_probes: vec![],
        login_url: None,
        login_timeout: None,
        cookie_file: None,
        // ⭐ 通用网页靠 Jina Reader，零配置
        requires_session: false,
    });

    c
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 通用性：加平台只改数据 ──────────────────────────────────

    #[test]
    fn custom_platform_needs_no_code_change() {
        // ⭐ 这是「通用性」的验收：不经由 SocialPlatform 枚举
        let mut c = PlatformCatalog::new();
        c.register(PlatformSpec {
            id: "mastodon".into(),
            display_name: "Mastodon".into(),
            url_patterns: vec!["mastodon.social".into()],
            login_probes: vec![],
            login_url: None,
            login_timeout: None,
            cookie_file: None,
            requires_session: true,
        });

        assert!(c.get("mastodon").is_some());
        assert_eq!(c.all_ids(), vec!["mastodon"]);
        // ⭐ 不用碰 SocialPlatform 枚举就能被 URL 路由认领
        assert_eq!(c.match_url("https://mastodon.social/@x").map(|s| s.id.as_str()), Some("mastodon"));
    }

    #[test]
    fn catalog_covers_more_than_the_enum() {
        // ⛔ `SocialPlatform::all()` 只有 6 个；目录必须有更多，
        //    否则「通用」是假的。
        let c = default_catalog();
        assert!(
            c.len() > 6,
            "catalog ({} platforms) must exceed SocialPlatform::all() (6)",
            c.len()
        );
    }

    #[test]
    fn all_ids_is_sorted_and_stable() {
        // ⭐ HashMap 迭代序不稳定 ⇒ 会造成 diff 噪音
        let c = default_catalog();
        let a = c.all_ids();
        let b = c.all_ids();
        assert_eq!(a, b);
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(a, sorted, "all_ids must be sorted");
    }

    // ── URL 认领：域名边界 ──────────────────────────────────────

    #[test]
    fn matches_real_platform_urls() {
        let c = default_catalog();
        for (url, want) in [
            ("https://x.com/user/status/1", "x"),
            ("https://twitter.com/user", "x"),
            ("https://www.reddit.com/r/rust", "reddit"),
            ("https://redd.it/abc", "reddit"),
            ("https://youtu.be/dQw4w9WgXcQ", "youtube"),
            ("https://www.youtube.com/watch?v=x", "youtube"),
            ("https://github.com/foo/bar", "github"),
            ("https://instagram.com/p/", "instagram"),
            ("https://www.linkedin.com/in/x", "linkedin"),
        ] {
            assert_eq!(c.match_url(url).map(|s| s.id.as_str()), Some(want), "url: {}", url);
        }
    }

    #[test]
    fn does_not_claim_lookalike_domains() {
        // ⭐⭐ 安全判据：`x.com` 是 host **后缀**才认领。
        //    裸 `contains` 会把 phishing-x.com / x.com.evil.net 误判成 X。
        let c = default_catalog();
        for hostile in [
            "https://phishing-x.com/",
            "https://notx.com/",
            "https://x.com.evil.net/",
            "https://myreddit.com/",
            "https://github.com.evil.io/",
        ] {
            let got = c.match_url(hostile).map(|s| s.id.clone());
            assert_ne!(
                got.as_deref(),
                Some("x"),
                "hostile url {} must not be claimed as x (got {:?})",
                hostile,
                got
            );
            assert_ne!(got.as_deref(), Some("reddit"), "hostile {} claimed as reddit", hostile);
            assert_ne!(got.as_deref(), Some("github"), "hostile {} claimed as github", hostile);
        }
    }

    #[test]
    fn subdomain_of_platform_is_claimed() {
        // ⭐ `www.x.com` 是 X 的合法子域，必须认领
        let c = default_catalog();
        assert_eq!(
            c.match_url("https://www.x.com/home").map(|s| s.id.as_str()),
            Some("x")
        );
        assert_eq!(
            c.match_url("https://old.reddit.com/r/rust").map(|s| s.id.as_str()),
            Some("reddit")
        );
    }

    #[test]
    fn url_parsing_handles_userinfo_and_ports() {
        let c = default_catalog();
        // ⛔ userinfo 里的伪域名不得被认领
        assert_ne!(
            c.match_url("https://x.com@evil.net/").map(|s| s.id.as_str()),
            Some("x")
        );
        // ⛔ 端口必须被剥掉：`x.com:8080` 的 host 是 `x.com`。
        //    ⛔ 漏掉这一步会让带端口的 URL 完全无法认领
        //    （自测独立 harness 抓到的实际 bug）。
        for url in [
            "https://x.com:8080/home",
            "https://www.x.com:443/",
            "https://reddit.com:8443/r/rust",
        ] {
            let got = c.match_url(url).map(|s| s.id.clone());
            assert!(got.is_some(), "port URL {} must still be claimed (got None)", url);
        }
        assert_eq!(
            c.match_url("https://x.com:8080/home").map(|s| s.id.as_str()),
            Some("x")
        );
        // 查询串/fragment 不影响
        assert_eq!(
            c.match_url("https://x.com/home?a=1#frag").map(|s| s.id.as_str()),
            Some("x")
        );
    }

    #[test]
    fn unknown_url_is_not_claimed() {
        let c = default_catalog();
        assert!(c.match_url("https://example.org/page").is_none());
        assert!(c.match_url("not a url").is_none());
        assert!(c.match_url("").is_none());
    }

    // ── 凭据路径与 env 命名 ────────────────────────────────────

    #[test]
    fn cookie_paths_use_their_own_file() {
        let c = default_catalog();
        for (id, file) in [("x", "x.json"), ("github", "github.json"), ("zhihu", "zhihu.json")] {
            let p = c.cookie_path(id).expect("HOME is set on this machine");
            assert!(p.ends_with(file), "{} should end with {}, got {:?}", id, file, p);
        }
    }

    #[test]
    fn cookie_path_errors_on_unknown_platform() {
        let c = default_catalog();
        assert!(c.cookie_path("nope").is_err());
    }

    #[test]
    fn env_var_names_follow_one_uniform_rule() {
        // ⭐ 统一规则而非每平台一个常量
        assert_eq!(
            PlatformCatalog::env_var_names("x"),
            ("NEOTRIX_X_AUTH_TOKEN".to_string(), "NEOTRIX_X_CT0".to_string())
        );
        // ⛔ 非字母数字必须被替换，否则会产生非法 env 名
        let (auth, ct0) = PlatformCatalog::env_var_names("my-site.v2");
        assert_eq!(auth, "NEOTRIX_MY_SITE_V2_AUTH_TOKEN");
        assert_eq!(ct0, "NEOTRIX_MY_SITE_V2_CT0");
        assert!(auth.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()));
    }

    // ── 诚实性：没实测过的探针不编造 ──────────────────────────

    #[test]
    fn platforms_without_verified_probes_do_not_claim_login_support() {
        // ⛔ 编造 cookie 名比不声明更糟：会让人以为登录能work
        let c = default_catalog();
        for id in ["instagram", "tiktok", "linkedin", "youtube", "web"] {
            let spec = c.get(id).expect("spec exists");
            assert!(
                !spec.supports_login(),
                "{} must not claim login support without a verified probe",
                id
            );
        }
    }

    #[test]
    fn login_capable_platforms_have_probes_and_urls() {
        let c = default_catalog();
        for id in ["x", "reddit", "github", "bilibili", "zhihu"] {
            let spec = c.get(id).expect("spec exists");
            assert!(spec.supports_login(), "{} should support login", id);
            assert!(spec.login_url.is_some(), "{} needs a login_url", id);
            assert!(spec.cookie_file.is_some(), "{} needs a cookie_file", id);
            assert!(spec.login_timeout.is_some(), "{} needs a timeout", id);
        }
    }

    #[test]
    fn every_spec_has_id_and_display_name() {
        let c = default_catalog();
        for spec in c.all() {
            assert!(!spec.id.is_empty());
            assert!(!spec.display_name.is_empty(), "{} has empty display_name", spec.id);
            assert!(spec.id == spec.id.to_lowercase(), "{} id must be lowercase", spec.id);
        }
    }

    #[test]
    fn requires_session_false_only_where_session_is_optional() {
        let c = default_catalog();
        // ⭐ 这两个实测可零配置工作（yt-dlp / Jina Reader）
        assert!(!c.get("youtube").unwrap().requires_session);
        assert!(!c.get("web").unwrap().requires_session);
    }

    #[test]
    fn domain_fragment_matcher_edge_cases() {
        assert!(matches_domain_fragment("x.com", "x.com"));
        assert!(matches_domain_fragment("www.x.com", "x.com"));
        assert!(!matches_domain_fragment("notx.com", "x.com"));
        assert!(!matches_domain_fragment("", "x.com"));
        assert!(!matches_domain_fragment("x.com", ""));
        // 大小写不敏感由调用方小写化保证
        assert!(!matches_domain_fragment("x.com.evil.net", "x.com"));
    }
}