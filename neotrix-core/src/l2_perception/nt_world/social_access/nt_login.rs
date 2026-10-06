//! 通用站点登录 — 与平台无关的会话建立
//!
//! # 为什么要有这一层（2026-10-03）
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
//! 这直接对上同类项目的教训：bird 与 OpenCLI **都不读浏览器 cookie
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
    /// 判定登录成功的探针（可多个，任一命中即成功）。
    pub probes: Vec<SuccessProbe>,
    /// cookie 存储文件名（相对 `~/.neotrix/cookies/`）。
    pub cookie_file: String,
    /// 登录等待超时。
    pub timeout: Duration,
    /// 该站点是否**必须**有 cookie 才能工作。
    ///
    /// 用于诊断文案：`false` 表示「登录只是加分项，没它也能用」，
    /// 避免把「没登录」报成致命错误。
    pub requires_session: bool,
}

/// 登录成功的判据。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SuccessProbe {
    /// 存在名为 `name` 的非空 cookie。**不读值**（参照 bird/OpenCLI：
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

/// 构造默认登录注册表 —— **由 [`PlatformCatalog`] 派生，不重复维护**。
///
/// # ⛔ 2026-10-03：此前这里是**第二份手写平台清单**
///
/// 上一个版本本函数硬编码 5 个 [`LoginTarget`]，而
/// [`crate::l2_perception::nt_world::social_access::nt_catalog::default_catalog`]
/// 有 10 个。审计实证的漂移后果：
///
/// ```text
/// $ neotrix social catalog --json  → 10 个平台（含 instagram/tiktok/…）
/// $ neotrix social sites    --json  →  5 个平台
/// ```
///
/// ⛔ 即 `catalog` 声称支持某平台，`auth` 却拒接它 —— 用户看到的是
/// 「自相矛盾的工具」，而非「明确不支持」。
///
/// ⇒ 现改为**从目录派生**：登录能力是平台的**属性**，不是独立清单。
/// 新增平台只需 [`PlatformCatalog::register`]，登录表自动跟随。
pub fn default_registry() -> LoginRegistry {
    let catalog = crate::l2_perception::nt_world::social_access::nt_catalog::default_catalog();
    let mut r = LoginRegistry::new();
    for spec in catalog.all() {
        // ⛔ 无探针 / 无登录 URL 的平台**不注册**到登录表：
        //    「没有可验证的登录判据」与「有判据但未登录」必须可区分。
        let (Some(login_url), false) = (spec.login_url.clone(), spec.login_probes.is_empty()) else {
            continue;
        };
        let cookie_file = spec.cookie_file.clone().unwrap_or_else(|| format!("{}.json", spec.id));
        r.register(LoginTarget {
            id: spec.id.clone(),
            display_name: spec.display_name.clone(),
            login_url,
            success_url: spec.success_url_for_channel(),
            probes: spec.login_probes.clone(),
            cookie_file,
            timeout: spec.login_timeout.unwrap_or(DEFAULT_LOGIN_TIMEOUT),
            requires_session: spec.requires_session,
        });
    }
    r
}

/// 默认登录超时（目录未指定时）。
pub const DEFAULT_LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

/// 对一次观测执行全部探针。
///
/// **纯函数** —— 不碰浏览器、不碰文件系统。真实浏览器观测由
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
    /// 哪些 cookie 有**非空值**（只记布尔，不记值本身）。
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
/// # 为何复用 [`UniversalBrowser::fetch`] 而非自己开页面
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

    // 观测输入必须被清空：它含 cookie 值（fetch 会填 value），
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
        // 报告的是**实际命中**的那个，便于诊断
        assert!(out.matched_by.unwrap().contains("user_session"));
    }

    // ── ⛔ 凭据不外泄 ────────────────────────────────────────

    #[test]
    fn outcome_never_contains_cookie_values() {
        let secret = "SUPER_SECRET_TOKEN_VALUE";
        let t = target(vec![SuccessProbe::CookiePresent { name: "auth_token".into() }]);
        let out = evaluate(&t, &obs(&[("auth_token", secret)], Some("https://x.com/home"), None));

        // 值不得出现在任何可序列化字段里（doctor 会把它打印进日志）
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
        // 这条是「通用化」的验收：不能只有 X
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
        // doctor 靠这段文案告诉用户「怎么算成功」
        let a = SuccessProbe::CookiePresent { name: "auth_token".into() };
        let b = SuccessProbe::UrlContains { needle: "/home".into() };
        let c = SuccessProbe::BodyContains { needle: "x".into() };
        assert_ne!(a.describe(), b.describe());
        assert_ne!(b.describe(), c.describe());
        assert_ne!(a.describe(), c.describe());
    }

    #[test]
    fn custom_target_can_be_registered_without_code_change() {
        // 这是「通用化」的实质：加平台**不改代码**，只加数据
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
#[cfg(test)]
mod drift_guard_tests {
    use super::*;

    /// **漂移守卫** —— 本轮审计的核心缺陷。
    ///
    /// ⛔ 此前 `default_registry()` 是**第二份手写平台清单**，与
    /// `nt_catalog` 的漂移后果已实测：
    /// ```text
    /// $ social catalog --json → 10 个平台
    /// $ social sites    --json →  5 个平台
    /// ```
    /// 即 `catalog` 声称支持、`auth` 拒接 —— 用户看到自相矛盾的工具。
    #[test]
    fn login_table_is_derived_from_catalog_not_handwritten() {
        use crate::l2_perception::nt_world::social_access::nt_catalog::default_catalog;

        let catalog = default_catalog();
        let login = default_registry();

        // 登录表 ⊆ 目录（无探针的平台理应缺席）
        for id in login.ids() {
            assert!(
                catalog.get(id).is_some(),
                "login table has `{}` but catalog does not ⇒ drifted",
                id
            );
        }

        // 目录里凡「有探针且有 login_url」的平台，登录表**必须**有
        for spec in catalog.all() {
            if !spec.login_probes.is_empty() && spec.login_url.is_some() {
                assert!(
                    login.get(&spec.id).is_some(),
                    "catalog says `{}` supports login but the login table lacks it ⇒ drifted",
                    spec.id
                );
            }
        }
    }

    #[test]
    fn platforms_without_probes_are_absent_from_login_table() {
        // 关键区分：「无可验证判据」≠「已判未登录」
        let login = default_registry();
        for id in ["instagram", "tiktok", "linkedin", "youtube", "web"] {
            assert!(
                login.get(id).is_none(),
                "{} has no probe; it must NOT appear in the login table",
                id
            );
        }
    }

    #[test]
    fn derived_login_targets_are_consistent_with_their_specs() {
        use crate::l2_perception::nt_world::social_access::nt_catalog::default_catalog;

        let catalog = default_catalog();
        let login = default_registry();
        for id in login.ids() {
            let spec = catalog.get(id).expect("checked above");
            let t = login.get(id).expect("present");
            assert_eq!(t.display_name, spec.display_name, "{} display_name drift", id);
            assert_eq!(t.probes, spec.login_probes, "{} probes drift", id);
            assert_eq!(t.login_url, spec.login_url.clone().unwrap_or_default());
            // cookie 文件名也必须同源，否则会出现两套路径
            assert_eq!(t.cookie_file, spec.cookie_file.clone().unwrap_or_else(|| format!("{}.json", spec.id)));
            assert_eq!(t.requires_session, spec.requires_session);
        }
    }

    #[test]
    fn success_url_falls_back_to_url_probe() {
        use crate::l2_perception::nt_world::social_access::nt_catalog::default_catalog;
        // x 的 success_url 显式配置为 /home
        let x = default_catalog().get("x").expect("x present").clone();
        assert_eq!(x.success_url_for_channel().as_deref(), Some("/home"));
        // 无显式配置的，probe 派生为 None（该平台探针里无 UrlContains）
        let yt = default_catalog().get("youtube").expect("yt present").clone();
        assert_eq!(yt.success_url_for_channel(), None);
    }
}
