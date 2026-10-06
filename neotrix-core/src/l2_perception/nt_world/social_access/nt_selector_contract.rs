//! DOM 选择器契约 — 吸收 open-slide「约束画布而非内容」+ lightpanda「声明能力面」
//!
//! # 动机：治我自己的一个假成功缺陷
//!
//! [`super::nt_x_browser`] 用 `article[data-testid='tweet']` 这类选择器抽取。
//! X 一旦改版，选择器失效 ⇒ 抽到 **0 条** ⇒ 而 `Ok(ExtractorResult{ total: 0 })`
//! 与「该查询真的没有结果」**在类型上无法区分**。
//!
//! ⛔ 这正是本会话反复打击的同一个失败模式：静默的假成功比报错坏得多。
//!
//! # 两条被吸收的设计
//!
//! 1. **open-slide「约束画布而非内容」**（`open-slide/open-slide`，MIT）
//!    它的思路是：不试图约束 agent 产出的**内容**，而是给定一个受约束的**画布**
//!    （1920×1080 + inspector），让越界在结构上不可能。
//!    ⇒ 移植为：不给「抽取结果」加内容校验，而是约束**抽取面** ——
//!    用**探针**证明选择器当前仍然有效，把「选择器漂移」变成类型化错误。
//!
//! 2. **lightpanda「声明能力面」**（AGPL，仅吸收设计不吸收代码）
//!    它的 `Page.enable` / `Target.setDiscoverTargets` 是**空实现**，
//!    却仍返回成功 —— 这类「能力声明与实际不符」正是它不可作反检测后端的原因之一。
//!    ⇒ 移植为：[`SelectorContract`] **显式声明**每个选择器适配哪些页面形态，
//!    并由 [`SelectorContract::verify`] 在运行时核对，不核对就不许抽取。
//!
//! # 与既有 `BrowserCaps` 的关系
//!
//// 核实更正（2026-10-03）：`nt_io_browser_engine::types::BackendCaps`
//! **已经**是一个后端能力契约（`javascript`/`screenshot`/`form_submit`/`cookie_persist`），
//! 即 lightpanda 那条「CDP 契约」设计**早已吸收**，只是命名不同。
//! （此前我在 LICENSES.md 记为「未落地」，是**搜错了字符串**，已更正。）
//! 本模块补的是它**没有**的一层：**选择器层面的契约**——
//! 后端能执行 JS ≠ JS 里的选择器还能命中。

use super::traits::{SocialAccessError, SocialAccessResult};

/// 页面形态。
///
/// ⛔ 不是「页面 URL」而是「DOM 形态」—— 同一条 URL 在未登录、
/// 已登录、搜索结果三种状态下 DOM 结构完全不同，而这三者恰恰决定
/// 选择器是否有效。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageShape {
    /// 未登录：只有登录墙，无推文。
    LoginWall,
    /// 已登录的搜索结果 / 时间线。
    Feed,
    /// 页面已加载但形态不明（改版、空结果、错误页）。
    Unknown,
}

/// 一个选择器的契约声明。
#[derive(Debug, Clone, Copy)]
pub struct SelectorProbe {
    /// 选择器名（用于错误信息定位）。
    pub name: &'static str,
    /// 该选择器在**哪种**页面形态下应当命中。
    pub required_for: PageShape,
    /// 该选择器是否为整次抽取的**必要条件**。
    ///
    /// ⛔ `true` ⇒ 零命中即判定漂移并报错。
    /// `false` ⇒ 零命中只记警告（如正文 `tweetText` 可能是纯图片推文）。
    pub essential: bool,
}

/// X 抽取器的选择器契约。
///
/// 全部选择器集中在这一处，是「约束抽取面」的落点：
/// 改版时只改这里，且 [`Self::verify`] 会强制每个改动都被验证。
pub const X_SELECTOR_CONTRACT: &[SelectorProbe] = &[
    SelectorProbe {
        name: "tweet",
        required_for: PageShape::Feed,
        essential: true,
    },
    SelectorProbe {
        name: "tweetText",
        required_for: PageShape::Feed,
        essential: false,
    },
    SelectorProbe {
        name: "User-Name",
        required_for: PageShape::Feed,
        essential: false,
    },
];

/// 一次抽取的观测结果：每个选择器各命中多少元素。
pub type SelectorCounts = Vec<(&'static str, usize)>;

/// 契约验证结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractVerdict {
    /// 契约成立。
    Satisfied,
    /// 选择器漂移：essential 选择器零命中。
    Drifted {
        shape: PageShape,
        missing: Vec<&'static str>,
    },
}

impl ContractVerdict {
    /// 是否可继续抽取。
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Satisfied)
    }
}

/// 判定页面形态。
///
/// **先判形态再验契约**，顺序不能反：若先验契约，未登录页会
/// 被误判成「选择器漂移」，把「请先登录」报成「X 改版了」——
/// 后者会让维护者去改代码，而真正的问题是去登录。
///
/// # ⚠️ `body_markers` 与 `doc_title` 不可混用
///
/// 首版把登录墙标记拿去和 `document.title` 比对 —— 这个匹配基本永远不命中，
/// 因为 `Log in to X` / `Sign up` 出现在**正文**里（实测 lightpanda 抓
/// `x.com/elonmusk` 未登录输出全是这两个链接），而 `document.title`
/// 未登录时是 `X / 用户名` 之类。
///
/// ⇒ 故拆成两个参数：`body_markers` 由 JS 在 `document.body.innerText`
/// 里观测（这与 [`super::nt_x_browser`] 的 `LOGIN_WALL` 正则同源），
/// `doc_title` 只用于错误信息与诊断。
pub fn classify_shape(
    body_markers: &[&str],
    tweet_hits: usize,
    doc_title: &str,
) -> PageShape {
    // 1) 登录墙标记优先（正文观测）
    let body = body_markers.join(" ").to_ascii_lowercase();
    if body.contains("log in to x") || body.contains("sign up") || body.contains("登录") {
        return PageShape::LoginWall;
    }
    // 2) 推文容器命中 ⇒ Feed
    if tweet_hits > 0 {
        return PageShape::Feed;
    }
    let _ = doc_title; // 仅诊断用，不参与判定（见上方说明）
    PageShape::Unknown
}

/// 校验选择器契约。
///
/// ⛔ 只对 `essential == true` 的选择器施加「零命中即漂移」；
/// 非 essential 的零命中**不**判失败（图片推文没有 `tweetText` 是正常的）。
///
/// ⛔ `PageShape::Unknown` 时**不**判漂移：此时页面可能确实无数据
/// （真的没搜到），也可能是改版，无法区分 —— 此时返回
/// `Satisfied` 但由调用方决定是否报「零结果可疑」。
pub fn verify(contract: &[SelectorProbe], shape: PageShape, counts: &SelectorCounts) -> ContractVerdict {
    let missing: Vec<&'static str> = contract
        .iter()
        .filter(|p| p.essential && p.required_for == shape)
        .filter(|p| counts.iter().find(|(n, _)| *n == p.name).map_or(0, |(_, c)| *c) == 0)
        .map(|p| p.name)
        .collect();

    if missing.is_empty() {
        ContractVerdict::Satisfied
    } else {
        ContractVerdict::Drifted { shape, missing }
    }
}

/// 把契约判定翻译成错误。
pub fn verdict_to_error(v: &ContractVerdict, url: &str) -> Option<SocialAccessError> {
    match v {
        ContractVerdict::Satisfied => None,
        ContractVerdict::Drifted { shape, missing } => Some(SocialAccessError::Parse(format!(
            "DOM selector contract violated on {} (shape={:?}, zero hits for {:?}). \
             This is a selector drift, not an empty result — returning an empty list \
             here would be indistinguishable from 'no matches'. \
             Update X_SELECTOR_CONTRACT in nt_selector_contract.rs after checking \
             the current x.com markup.",
            url, shape, missing
        ))),
    }
}

/// 便利入口：观测 → 判形态 → 验契约 → 返回错误（若有）。
///
/// 顺序即语义，见 [`classify_shape`] 的说明。
///
/// # ⚠️ `expected` 是必需的，且这个参数的存在是被测试逼出来的
///
/// 首版签名没有 `expected`，只有一个 `doc_title`。自测
/// `check_reports_drift_not_empty_result` 抓到：访问 `/home`（一个**必然**
/// 有推文的 feed 页）、观测到 0 条推文时，函数返回 `Ok(PageShape::Unknown)`。
/// 而 `Ok(Unknown)` 会让调用方照常返回 `total: 0` —— **正是本模块要消灭的
/// 假成功**，只是换了个位置。
///
/// 根因有二，都不是「加个 if」能解决的：
///
/// 1. **观测不足**：`Unknown` 同时对应「真的没数据」与「选择器漂移」，
///    二者在只有 `doc_title` 时**原理上不可区分**。
/// 2. **判据用错字段**：登录墙标记（`Log in to X`）在**正文**里，
///    不在 `document.title` 里。首版拿 `doc_title` 去比对登录标记，
///    这个匹配基本永远不命中。
///
/// ⇒ 修法：显式传入 `expected`（由 URL 形态推出）与 `body_markers`
/// （由 JS 在正文里观测，见 [`classify_shape`]），三者交叉后仍不可判定时
/// **报歧义错误**，把判断权交还调用方，而不是默认放行。
pub fn check(
    contract: &[SelectorProbe],
    body_markers: &[&str],
    counts: &SelectorCounts,
    doc_title: &str,
    url: &str,
    expected: PageShape,
) -> SocialAccessResult<PageShape> {
    let tweet_hits = counts
        .iter()
        .find(|(n, _)| *n == "tweet")
        .map_or(0, |(_, c)| *c);

    let shape = classify_shape(body_markers, tweet_hits, doc_title);

    match shape {
        PageShape::LoginWall => Err(SocialAccessError::AuthFailed {
            platform: super::traits::SocialPlatform::Twitter,
            reason: format!(
                "x.com served a login page for {}. \
                 Run `neotrix social login x` once to persist cookies.",
                url
            ),
        }),
        PageShape::Feed => {
            if let Some(err) = verdict_to_error(&verify(contract, shape, counts), url) {
                return Err(err);
            }
            Ok(shape)
        }
        PageShape::Unknown => {
            // 走到这里说明「零推文容器」且「无登录墙标记」。
            //    若调用方预期是 Feed，则这是一个**不可静默**的歧义：
            //    真的没数据 与 选择器漂移 在此不可区分。
            if expected == PageShape::Feed {
                return Err(SocialAccessError::Parse(format!(
                    "ambiguous zero result on {}: expected a feed, but 0 tweet containers \
                     were found and no login-wall marker was present. This is either \
                     (a) genuinely no results, or (b) selector drift — the two are not \
                     distinguishable from this observation. Refusing to report an empty \
                     result, because that would be silently wrong in case (b). \
                     Verify the current x.com markup and update X_SELECTOR_CONTRACT \
                     in nt_selector_contract.rs if it drifted.",
                    url
                )));
            }
            // 预期本就不是 Feed（如访问一个确实为空的页面）⇒ 合法放行
            if let Some(err) = verdict_to_error(&verify(contract, shape, counts), url) {
                return Err(err);
            }
            Ok(shape)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pairs: &[(&'static str, usize)]) -> SelectorCounts {
        pairs.to_vec()
    }

    // ── 形态判定：顺序是语义 ────────────────────────────────

    #[test]
    fn login_wall_wins_over_zero_tweets() {
        // 关键顺序测试：未登录页天然 0 推文。
        //    若先验契约会误报「选择器漂移」，把「去登录」报成「X 改版」。
        //    标记来自**正文**（实测：未登录 x.com 输出全是 Log in / Sign up 链接）
        let markers = ["Log in to X", "Sign up"];
        let shape = classify_shape(&markers, 0, "X / elonmusk");
        assert_eq!(shape, PageShape::LoginWall);
    }

    #[test]
    fn login_markers_in_title_alone_do_not_trip_the_gate() {
        // 回归：首版拿 doc_title 比对登录标记，这是错的 ——
        //    title 里没有这些文案，据此判墙会永远漏判。
        let shape = classify_shape(&[], 0, "Log in to X");
        assert_eq!(shape, PageShape::Unknown);
    }

    #[test]
    fn tweets_present_means_feed() {
        // ⛔ 标记必须为空：登录墙判定优先级高于推文命中
        let shape = classify_shape(&[], 3, "X");
        assert_eq!(shape, PageShape::Feed);
    }

    #[test]
    fn login_marker_outranks_present_tweets() {
        // 未登录页不该有推文；若两者同时出现，登录墙更可信
        //    （可能是「部分渲染 + 登录提示」的中间态）
        let shape = classify_shape(&["Sign up"], 3, "X");
        assert_eq!(shape, PageShape::LoginWall);
    }

    #[test]
    fn no_markers_no_tweets_is_unknown_not_feed() {
        // ⛔ 不得把「形态不明」当成 Feed —— 那会让契约校验通过
        let shape = classify_shape(&[], 0, "Some Page");
        assert_eq!(shape, PageShape::Unknown);
    }

    #[test]
    fn marker_matching_is_case_insensitive() {
        let shape = classify_shape(&["LOG IN TO X"], 0, "X");
        assert_eq!(shape, PageShape::LoginWall);
    }

    // ── 契约校验 ───────────────────────────────────────────

    #[test]
    fn feed_with_tweets_satisfies_contract() {
        let v = verify(
            X_SELECTOR_CONTRACT,
            PageShape::Feed,
            &counts(&[("tweet", 5), ("tweetText", 5), ("User-Name", 5)]),
        );
        assert_eq!(v, ContractVerdict::Satisfied);
        assert!(v.is_usable());
    }

    #[test]
    fn feed_with_zero_tweet_elements_is_drift_not_empty() {
        // 这就是本模块存在的理由：Feed 形态却零推文容器 = 选择器漂移
        let v = verify(
            X_SELECTOR_CONTRACT,
            PageShape::Feed,
            &counts(&[("tweet", 0), ("tweetText", 0), ("User-Name", 0)]),
        );
        match v {
            ContractVerdict::Drifted { missing, .. } => {
                assert_eq!(missing, vec!["tweet"], "only the essential probe must be reported");
            }
            other => panic!("expected Drifted, got {:?}", other),
        }
    }

    #[test]
    fn non_essential_zero_hits_is_not_drift() {
        // 图片推文没有 tweetText 是正常的 —— 不得判失败
        let v = verify(
            X_SELECTOR_CONTRACT,
            PageShape::Feed,
            &counts(&[("tweet", 2), ("tweetText", 0), ("User-Name", 0)]),
        );
        assert_eq!(v, ContractVerdict::Satisfied);
    }

    #[test]
    fn unknown_shape_is_not_drift() {
        // ⛔ 形态不明时无法区分「真无数据」与「改版」，不判漂移
        let v = verify(X_SELECTOR_CONTRACT, PageShape::Unknown, &counts(&[("tweet", 0)]));
        assert_eq!(v, ContractVerdict::Satisfied);
    }

    #[test]
    fn login_wall_shape_ignores_contract() {
        // Feed 专属的 essential 探针在 LoginWall 形态下不适用
        let v = verify(X_SELECTOR_CONTRACT, PageShape::LoginWall, &counts(&[("tweet", 0)]));
        assert_eq!(v, ContractVerdict::Satisfied);
    }

    #[test]
    fn missing_probe_entry_counts_as_zero() {
        // ⛔ 观测里根本没上报该选择器 ⇒ 视为 0 命中，而非「跳过」
        let v = verify(X_SELECTOR_CONTRACT, PageShape::Feed, &counts(&[]));
        assert!(matches!(v, ContractVerdict::Drifted { .. }));
    }

    // ── check() 端到端 ─────────────────────────────────────

    #[test]
    fn check_reports_auth_error_for_login_wall() {
        let err = check(
            X_SELECTOR_CONTRACT,
            &["Log in to X", "Sign up"],
            &counts(&[("tweet", 0)]),
            "X / elonmusk",
            "https://x.com/search?q=x",
            PageShape::Feed,
        )
        .expect_err("login wall must error");
        match err {
            SocialAccessError::AuthFailed { reason, .. } => {
                assert!(reason.contains("neotrix social login x"), "must be actionable: {}", reason);
            }
            other => panic!("expected AuthFailed, got {:?}", other),
        }
    }

    #[test]
    fn ambiguous_zero_on_expected_feed_is_an_error_not_an_empty_result() {
        // 这条测试逼出了首版 API 的缺陷：/home 是必然有推文的 feed，
        //    观测到 0 条时首版返回 Ok(Unknown)，调用方会照常报 total: 0
        //    —— 假成功只是换了个位置。
        let err = check(
            X_SELECTOR_CONTRACT,
            &[],
            &counts(&[("tweet", 0)]),
            "Home / X",
            "https://x.com/home",
            PageShape::Feed,
        )
        .expect_err("ambiguous zero on an expected feed must error");
        match err {
            SocialAccessError::Parse(msg) => {
                assert!(msg.contains("ambiguous zero result"), "got: {}", msg);
                // 必须点明两种可能，否则调用方无从下手
                assert!(msg.contains("(a) genuinely no results"), "got: {}", msg);
                assert!(msg.contains("(b) selector drift"), "got: {}", msg);
                assert!(msg.contains("X_SELECTOR_CONTRACT"), "must name the fix site: {}", msg);
            }
            other => panic!("expected Parse, got {:?}", other),
        }
    }

    #[test]
    fn ambiguous_zero_is_allowed_when_feed_was_not_expected() {
        // 预期本就不是 Feed ⇒ 零结果合法，不该报错
        let shape = check(
            X_SELECTOR_CONTRACT,
            &[],
            &counts(&[("tweet", 0)]),
            "Some Empty Page",
            "https://x.com/i/u/503",
            PageShape::Unknown,
        )
        .expect("non-feed page with no tweets must pass");
        assert_eq!(shape, PageShape::Unknown);
    }

    #[test]
    fn check_passes_healthy_feed() {
        let shape = check(
            X_SELECTOR_CONTRACT,
            &[],
            &counts(&[("tweet", 4)]),
            "X",
            "https://x.com/home",
            PageShape::Feed,
        )
        .expect("healthy feed must pass");
        assert_eq!(shape, PageShape::Feed);
    }

    #[test]
    fn contract_declares_exactly_one_essential_probe() {
        // 守住「essential 是刻意收紧」这个决定，防止后来人随手加第二个
        let essential: Vec<_> = X_SELECTOR_CONTRACT.iter().filter(|p| p.essential).collect();
        assert_eq!(essential.len(), 1);
        assert_eq!(essential[0].name, "tweet");
    }
}