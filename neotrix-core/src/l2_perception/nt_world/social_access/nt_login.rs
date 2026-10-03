//! 通用站点登录 — 与平台无关的会话建立
//!
//! # ⭐ 为什么要有这一层（2026-10-03）
//!
//! 修复前的登录能力**只对 X 有效**，且是四处硬编码：
//!
//! - [`crate::l1_action::nt_io::universal_browser::PlatformConfig`] 只有
//!   `twitter()` / `futong()` / `custom()` 三个构造器，且 `custom()` 的
//!   `success_url` 恒为 `None` ⇒ 只能等超时，**无法判定登录成功**。
//! - `login_manual()` 的成功判据写死 `cookie.name == "auth_token"`
//!   —— 那是 X 的 cookie 名。GitHub / Reddit / 任一站点都不叫这个。
//! - [`super::auth::AuthService`] 的 OAuth 注册是 `with_x_oauth()` /
//!   `with_reddit_oauth()` 两个各自硬编码端点的方法。
//! - cookie 路径硬编码 `~/.neotrix/cookies/twitter.json`。
//!
//! ⇒ 本模块把这些收敛成**一份注册表 + 一个通用流程**。
//!
//! # 设计要点
//!
//! ## 1. 成功判据是「注册表数据」，不是代码里的 if
//!
//! [`LoginTarget`] 用 [`SuccessProbe`] 描述「怎么算登录成功」：
//! cookie 存在性 / URL 前缀 / DOM 标记。每个平台只贡献数据，不写控制流。
//!
//! ⭐ 这直接对上同类项目的教训：bird 与 OpenCLI **都不读浏览器 cookie
//! 数据库**，只做 `auth_token` 的**存在性检查**，然后让浏览器自己附送 ——
//! 因为 `auth_token` 是 HttpOnly，**拿不到值也不需要值**。
//! 本模块据此把 cookie 探测设计成「只看名字、不读值」。
//!
//! ## 2. 凭据永不出现在输出里
//!
//! [`ProbeOutcome`] 只报告「哪些 cookie 名字在」，**不返回值**。
//! 否则 doctor 的输出会把 token 写进终端与 CI 日志。

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// 一个可登录目标的完整描述。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginTarget {
    /// 稳定标识（同时用作 cookie 存储的域键）。
    pub id: String,
    /// 人类可读名。
    pub display_name: String,
    /// 登录页 URL。
    pub login_url: String,
    /// 登录成功后应当出现的 URL 前缀（可空）。
    pub success_url: Option<String>,
    /// ⭐ 判定登录成功的探针（可多个，任一命中即成功）。
    pub probes: Vec<SuccessProbe>,
    /// cookie 存储文件名（相对 `~/.neotrix/cookies/`）。
    pub cookie_file: String,
    /// 登录等待超时。
    pub timeout: Duration,
    /// 该站点是否**必须**有 cookie 才能工作。
    ///
    /// ⭐ 用于诊断文案：`false` 表示「登录只是加分项，没它也能用」，
    /// 避免把「没登录」报成致命错误。
    pub requires_session: bool,
}

/// 登录成功的判据。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SuccessProbe {
    /// ⭐ 存在名为 `name` 的非空 cookie。**不读值**（参照 bird/OpenCLI：
    /// HttpOnly cookie 拿不到值也不需要值）。
    CookiePresent { name: String },
    /// 当前 URL 含此前缀。
    UrlContains { needle: String },
    /// 页面正文含此标记（用于「已登录但无标志性 cookie」的站点）。
    BodyContains { needle: String },
}

impl SuccessProbe {
    /// 该探针的简短人类描述（供 doctor 输出）。
    pub fn describe(&self) -> String {
        match self {
            SuccessProbe::CookiePresent { name } => format!("cookie `{}` present", name),
            SuccessProbe::UrlContains { needle } => format!("url contains `{}`", needle),
            SuccessProbe::BodyContains { needle } => format!("body contains `{}`", needle),
        }
    }
}

/// 一次探测的结论。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeOutcome {
    /// 哪个探针命中了（未命中则为 `None`）。
    pub matched_by: Option<String>,
    /// 已观测到的 cookie **名字**（⛔ 不含值）。
    pub cookie_names: Vec<String>,
    /// 观测到的当前 URL。
    pub current_url: Option<String>,
    /// 观测到的正文（截断）。
    pub body_excerpt: Option<String>,
}

impl ProbeOutcome {
    /// 是否判定为已登录。
    pub fn is_authenticated(&self) -> bool {
        self.matched_by.is_some()
    }
}

/// 已注册目标。
#[derive(Debug, Clone, Default)]
pub struct LoginRegistry {
    targets: HashMap<String, LoginTarget>,
}

impl LoginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个目标（按 `id` 覆盖）。
    pub fn register(&mut self, target: LoginTarget) {
        self.targets.insert(target.id.clone(), target);
    }

    pub fn get(&self, id: &str) -> Option<&LoginTarget> {
        self.targets.get(id)
    }

    pub fn ids(&self) -> Vec<&str> {
        self.targets.keys().map(String::as_str).collect()
    }

    pub fn all(&self) -> Vec<&LoginTarget> {
        self.targets.values().collect()
    }

    /// cookie 存储的绝对路径。
    ///
    /// ⛔ 不使用 `dirs::home_dir().unwrap_or_default()` ——
    /// 那样在无 HOME 时会静默落到当前目录，把凭据写进意外位置。
    /// 无 HOME 时**返回错误**，让调用方显式处理。
    pub fn cookie_path(&self, id: &str) -> Result<std::path::PathBuf, String> {
        let target = self
            .targets
            .get(id)
            .ok_or_else(|| format!("no login target registered for `{}`", id))?;
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .ok_or_else(|| {
                "neither HOME nor USERPROFILE is set; refusing to guess a cookie \
                 storage location (would silently write credentials somewhere unexpected)"
                    .to_string()
            })?;
        Ok(std::path::Path::new(&home)
            .join(".neotrix")
            .join("cookies")
            .join(&target.cookie_file))
    }
}

/// 构造默认注册表。
///
/// ⭐ 每个平台的探针都取自**实测**而非猜测：
///
/// | 平台 | 判据 | 依据 |
/// |---|---|---|
/// | x | cookie `auth_token` + URL `/home` | bird/gobird README 记录 `auth_token`(40 hex) 与 `ct0`；`login_manual` 原实现也用它 |
/// | github | cookie `user_session` + URL 段 | GitHub 会话 cookie 名 |
/// | reddit | cookie `reddit_session` + URL 段 | Reddit 会话 cookie 名 |
/// | bilibili | cookie `SESSDATA` + URL 段 | B 站标准会话 cookie |
/// | zhihu | cookie `z_c0` + URL 段 | 知乎的 `z_c0` 即登录态 cookie |
///
/// ⚠️ 除 x 外，其余 cookie 名**未经本机实测**（那些站点本机未登录）。
/// 它们是各自社区的公开惯例，但若将来发现不对，
/// 修 [`LoginRegistry::register`] 里的一条数据即可 —— 无需改控制流。
pub fn default_registry() -> LoginRegistry {
    let mut r = LoginRegistry::new();

    r.register(LoginTarget {
        id: "x".into(),
        display_name: "X (Twitter)".into(),
        login_url: "https://x.com/login".into(),
        success_url: Some("https://x.com/home".into()),
        probes: vec![
            // ⭐ 存在性检查，不读值（HttpOnly）
            SuccessProbe::CookiePresent { name: "auth_token".into() },
            SuccessProbe::UrlContains { needle: "/home".into() },
        ],
        cookie_file: "x.json".into(),
        timeout: Duration::from_secs(300),
        requires_session: true,
    });

    r.register(LoginTarget {
        id: "github".into(),
        display_name: "GitHub".into(),
        login_url: "https://github.com/login".into(),
        success_url: Some("https://github.com/".into()),
        probes: vec![
            SuccessProbe::CookiePresent { name: "user_session".into() },
            SuccessProbe::UrlContains { needle: "github.com/".into() },
        ],
        cookie_file: "github.json".into(),
        timeout: Duration::from_secs(300),
        requires_session: true,
    });

    r.register(LoginTarget {
        id: "reddit".into(),
        display_name: "Reddit".into(),
        login_url: "https://www.reddit.com/login".into(),
        success_url: Some("https://www.reddit.com/".into()),
        probes: vec![
            SuccessProbe::CookiePresent { name: "reddit_session".into() },
            SuccessProbe::UrlContains { needle: "reddit.com/".into() },
        ],
        cookie_file: "reddit.json".into(),
        timeout: Duration::from_secs(300),
        requires_session: true,
    });

    r.register(LoginTarget {
        id: "bilibili".into(),
        display_name: "Bilibili".into(),
        login_url: "https://passport.bilibili.com/login".into(),
        success_url: Some("https://www.bilibili.com/".into()),
        probes: vec![
            SuccessProbe::CookiePresent { name: "SESSDATA".into() },
            SuccessProbe::UrlContains { needle: "bilibili.com/".into() },
        ],
        cookie_file: "bilibili.json".into(),
        timeout: Duration::from_secs(300),
        requires_session: true,
    });

    r.register(LoginTarget {
        id: "zhihu".into(),
        display_name: "知乎".into(),
        login_url: "https://www.zhihu.com/signin".into(),
        success_url: Some("https://www.zhihu.com/".into()),
        probes: vec![
            SuccessProbe::CookiePresent { name: "z_c0".into() },
            SuccessProbe::UrlContains { needle: "zhihu.com/".into() },
        ],
        cookie_file: "zhihu.json".into(),
        timeout: Duration::from_secs(300),
        requires_session: true,
    });

    r
}

/// 对一次观测执行全部探针。
///
/// ⭐ **纯函数** —— 不碰浏览器、不碰文件系统。真实浏览器观测由
/// [`observe_and_verify`] 负责。这样判别逻辑可完整单测。
pub fn evaluate(target: &LoginTarget, observation: &Observation) -> ProbeOutcome {
    let mut cookie_names: Vec<String> = observation.cookie_names.clone();
    cookie_names.sort();
    cookie_names.dedup();

    let mut matched_by = None;

    for probe in &target.probes {
        let hit = match probe {
            SuccessProbe::CookiePresent { name } => observation
                .cookie_names
                .iter()
                .any(|c| c == name && observation.cookie_has_value(name)),
            SuccessProbe::UrlContains { needle } => observation
                .current_url
                .as_deref()
                .map_or(false, |u| u.contains(needle.as_str())),
            SuccessProbe::BodyContains { needle } => observation
                .body_excerpt
                .as_deref()
                .map_or(false, |b| b.contains(needle.as_str())),
        };
        if hit {
            matched_by = Some(probe.describe());
            break;
        }
    }

    ProbeOutcome {
        matched_by,
        cookie_names,
        current_url: observation.current_url.clone(),
        body_excerpt: observation.body_excerpt.as_deref().map(|b| {
            let t: String = b.chars().take(120).collect();
            t
        }),
    }
}

/// 浏览器侧的观测快照。
#[derive(Debug, Clone, Default)]
pub struct Observation {
    /// 观测到的 cookie **名字**列表。
    pub cookie_names: Vec<String>,
    /// ⭐ 哪些 cookie 有**非空值**（只记布尔，不记值本身）。
    ///
    /// ⛔ 存「值是否非空」而非值：前者足以判定登录态，
    /// 后者会把凭据带进内存与调试输出。
    pub non_empty_cookies: Vec<String>,
    /// 当前 URL。
    pub current_url: Option<String>,
    /// 正文（用于 `BodyContains` 探针）。
    pub body_excerpt: Option<String>,
}

impl Observation {
    /// 某 cookie 是否有非空值。
    pub fn cookie_has_value(&self, name: &str) -> bool {
        self.non_empty_cookies.iter().any(|c| c == name)
    }

    /// 从 `(name, value)` 对构造观测 —— ⛔ **只保留 name 与「值非空」**。
    pub fn from_cookies(pairs: &[(String, String)], url: Option<String>, body: Option<String>) -> Self {
        let mut cookie_names = Vec::new();
        let mut non_empty_cookies = Vec::new();
        for (n, v) in pairs {
            cookie_names.push(n.clone());
            if !v.is_empty() {
                non_empty_cookies.push(n.clone());
            }
        }
        Self { cookie_names, non_empty_cookies, current_url: url, body_excerpt: body }
    }
}

/// 真实浏览器观测 + 校验。
///
/// ⛔ **需要 `stealth-net` feature**（chromiumoxide）。
///
/// # ⭐ 为何复用 [`UniversalBrowser::fetch`] 而非自己开页面
///
/// 跨层调用必须走「消费方自己那层」的 facade（AGENTS.md 硬规则）。
/// `UniversalBrowser::new_page` 是 **private**，故本模块不自建页面；
/// 而 [`fetch`] 已返回 `content` + `cookies`，恰好覆盖
/// [`SuccessProbe`] 的三种判据所需的全部输入。
/// ⇒ 不新增 L1 API，也就不制造新的跨层违规。
#[cfg(feature = "stealth-net")]
pub async fn observe_and_verify(
    browser: &crate::l2_perception::nt_world::l1_facade::UniversalBrowser,
    target: &LoginTarget,
) -> Result<ProbeOutcome, String> {
    let result = browser
        .fetch(&target.login_url)
        .await
        .map_err(|e| format!("fetch {}: {}", target.login_url, e))?;

    let pairs: Vec<(String, String)> = result
        .cookies
        .iter()
        .map(|c| (c.name.clone(), c.value.clone()))
        .collect();

    // ⚠️ `BrowserResult` 无 URL 字段（已核实），故 UrlContains 探针
    //    在此路径上无法求值。诚实处理：只用 cookie 与 body 判据，
    //    并在返回里标注 URL 未知 —— 而不是假装 URL 已核对。
    let observation = Observation {
        cookie_names: pairs.iter().map(|(n, _)| n.clone()).collect(),
        non_empty_cookies: pairs
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(n, _)| n.clone())
            .collect(),
        current_url: None,
        body_excerpt: result.content.clone(),
    };

    let mut outcome = evaluate(target, &observation);

    // ⛔ 若目标**只**配了 URL 探针，此处必然判不出成功 —— 明确告知，
    //    否则用户会看到「一直登录不上」却不知原因。
    if !outcome.is_authenticated() && target.probes.iter().all(|p| matches!(p, SuccessProbe::UrlContains { .. })) {
        return Err(format!(
            "target `{}` only has URL-based success probes, but this observation path              cannot read the current URL (BrowserResult exposes no url field).              Add a CookiePresent or BodyContains probe so login can be verified.",
            target.id
        ));
    }

    // ⭐ 观测输入必须被清空：它含 cookie 值（fetch 会填 value），
    //    而 outcome 不含。显式截断 body 避免把大段 HTML 带出去。
    outcome.body_excerpt = outcome.body_excerpt.map(|b| b.chars().take(120).collect());
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(cookies: &[(&str, &str)], url: Option<&str>, body: Option<&str>) -> Observation {
        let pairs: Vec<(String, String)> = cookies
            .iter()
            .map(|(n, v)| (n.to_string(), v.to_string()))
            .collect();
        Observation::from_cookies(&pairs, url.map(String::from), body.map(String::from))
    }

    fn target(probes: Vec<SuccessProbe>) -> LoginTarget {
        LoginTarget {
            id: "t".into(),
            display_name: "T".into(),
            login_url: "https://t/login".into(),
            success_url: None,
            probes,
            cookie_file: "t.json".into(),
            timeout: Duration::from_secs(30),
            requires_session: true,
        }
    }

    // ── 探针求值（纯函数，可完整覆盖）─────────────────────────

    #[test]
    fn cookie_present_probe_needs_non_empty_value() {
        let t = target(vec![SuccessProbe::CookiePresent { name: "auth_token".into() }]);

        // 有值 ⇒ 命中
        let hit = evaluate(&t, &obs(&[("auth_token", "abc")], None, None));
        assert!(hit.is_authenticated());
        assert!(hit.matched_by.unwrap().contains("auth_token"));

        // ⛔ 存在但**空值** ⇒ 不算登录（X 的 auth_token 为空即未登录）
        let empty = evaluate(&t, &obs(&[("auth_token", "")], None, None));
        assert!(!empty.is_authenticated(), "an empty cookie value must not count as logged in");

        // 完全不存在
        let none = evaluate(&t, &obs(&[("other", "x")], None, None));
        assert!(!none.is_authenticated());
    }

    #[test]
    fn url_probe_matches_and_tolerates_none() {
        let t = target(vec![SuccessProbe::UrlContains { needle: "/home".into() }]);
        assert!(evaluate(&t, &obs(&[], Some("https://x.com/home"), None)).is_authenticated());
        assert!(evaluate(&t, &obs(&[], Some("https://x.com/login"), None)).is_authenticated() == false);
        // ⛔ URL 未知不得 panic
        assert!(!evaluate(&t, &obs(&[], None, None)).is_authenticated());
    }

    #[test]
    fn body_probe_works_for_sites_without_marker_cookies() {
        let t = target(vec![SuccessProbe::BodyContains { needle: "我的主页".into() }]);
        assert!(evaluate(&t, &obs(&[], None, Some("欢迎回来 我的主页"))).is_authenticated());
        assert!(!evaluate(&t, &obs(&[], None, Some("请登录"))).is_authenticated());
        assert!(!evaluate(&t, &obs(&[], None, None)).is_authenticated());
    }

    #[test]
    fn first_matching_probe_wins_and_is_reported() {
        let t = target(vec![
            SuccessProbe::CookiePresent { name: "absent".into() },
            SuccessProbe::CookiePresent { name: "user_session".into() },
            SuccessProbe::UrlContains { needle: "/home".into() },
        ]);
        let out = evaluate(&t, &obs(&[("user_session", "v")], Some("https://github.com/home"), None));
        assert!(out.is_authenticated());
        // ⭐ 报告的是**实际命中**的那个，便于诊断
        assert!(out.matched_by.unwrap().contains("user_session"));
    }

    // ── ⛔ 凭据不外泄 ────────────────────────────────────────

    #[test]
    fn outcome_never_contains_cookie_values() {
        let secret = "SUPER_SECRET_TOKEN_VALUE";
        let t = target(vec![SuccessProbe::CookiePresent { name: "auth_token".into() }]);
        let out = evaluate(&t, &obs(&[("auth_token", secret)], Some("https://x.com/home"), None));

        // ⭐ 值不得出现在任何可序列化字段里（doctor 会把它打印进日志）
        let dumped = serde_json::to_string(&out).expect("outcome must serialize");
        assert!(
            !dumped.contains(secret),
            "cookie value leaked into ProbeOutcome: {}",
            dumped
        );
        assert!(out.cookie_names.contains(&"auth_token".to_string()));
    }

    #[test]
    fn observation_stores_emptiness_not_values() {
        let o = obs(&[("a", "secret1"), ("b", "")], None, None);
        assert!(o.cookie_has_value("a"));
        assert!(!o.cookie_has_value("b"), "empty value must not be recorded as non-empty");
        // ⛔ 结构里根本没有存放值的字段
        let dumped = format!("{:?}", o);
        assert!(!dumped.contains("secret1"));
    }

    // ── 注册表 ────────────────────────────────────────────────

    #[test]
    fn default_registry_covers_multiple_platforms() {
        // ⭐ 这条是「通用化」的验收：不能只有 X
        let r = default_registry();
        for id in ["x", "github", "reddit", "bilibili", "zhihu"] {
            assert!(r.get(id).is_some(), "target `{}` must be registered", id);
        }
        assert!(r.ids().len() >= 5);
    }

    #[test]
    fn every_target_has_at_least_one_probe_and_cookie_file() {
        // ⛔ 没有探针的目标永远判不出登录成功 —— 注册时就该拦住
        let r = default_registry();
        for t in r.all() {
            assert!(!t.probes.is_empty(), "{} has no success probe", t.id);
            assert!(t.cookie_file.ends_with(".json"), "{} cookie file must be .json", t.id);
            assert!(!t.login_url.is_empty(), "{} has empty login_url", t.id);
            assert!(t.timeout.as_secs() > 0, "{} has zero timeout", t.id);
        }
    }

    #[test]
    fn probe_descriptions_are_distinguishable_in_diagnostics() {
        // ⭐ doctor 靠这段文案告诉用户「怎么算成功」
        let a = SuccessProbe::CookiePresent { name: "auth_token".into() };
        let b = SuccessProbe::UrlContains { needle: "/home".into() };
        let c = SuccessProbe::BodyContains { needle: "x".into() };
        assert_ne!(a.describe(), b.describe());
        assert_ne!(b.describe(), c.describe());
        assert_ne!(a.describe(), c.describe());
    }

    #[test]
    fn custom_target_can_be_registered_without_code_change() {
        // ⭐ 这是「通用化」的实质：加平台**不改代码**，只加数据
        let mut r = LoginRegistry::new();
        let mut t = target(vec![SuccessProbe::CookiePresent { name: "MY_COOKIE".into() }]);
        t.id = "mysite".into();
        t.cookie_file = "mysite.json".into();
        r.register(t);
        let t = r.get("mysite").expect("custom target must be retrievable");
        let out = evaluate(t, &obs(&[("MY_COOKIE", "v")], None, None));
        assert!(out.is_authenticated());
    }

    #[test]
    fn unknown_id_is_an_error_not_a_panic() {
        let r = default_registry();
        assert!(r.get("nope").is_none());
        assert!(r.cookie_path("nope").is_err());
    }

    #[test]
    fn cookie_path_is_under_neotrix_cookies() {
        let r = default_registry();
        // 本机有 HOME，故应成功且路径可预期
        if std::env::var_os("HOME").is_some() {
            let p = r.cookie_path("github").expect("HOME is set on this machine");
            assert!(p.ends_with("github.json"), "got {:?}", p);
            assert!(p.to_string_lossy().contains(".neotrix"));
        }
    }
}