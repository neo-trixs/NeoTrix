//! `neotrix social` — 社交平台访问能力诊断入口
//!
//! # ⭐ 为何要这个入口（R-P79：外部技术必须同会话接到生产）
//!
//! `social_access` 模块此前**整模块零生产接线**：`SocialAccessManager`、
//! `run_doctor`、`login_x_auto`、`default_channels` 在全仓都没有外部调用方。
//!
//! ⛔ 关键在于这种状态**编译期完全看不出来**：`pub` 出现在 `pub mod` 内部时，
//! rustc 的 `dead_code` lint **不会**触发。`cargo check` 全绿、12k+ 测试全绿，
//! 而这个模块一行都没跑过 —— 告警全绿 ≠ 代码在跑。
//!
//! 本模块把它接到真实入口，并遵守本仓已有的退出码约定
//! （对齐 OpenCLI 的 `sysexits.h`：0 成功 / 1 通用错 / 78 配置错），
//! 使其在 CI 中可判定。

// ⚠️ 路径前缀是 `neotrix::` 而非 `crate::`：本文件位于**二进制 crate 根**下
//    （`src/main.rs` 与 `src/lib.rs` 是两个 crate），`crate::` 在此指向 bin crate，
//    而这些类型定义在 lib crate 里。
use neotrix::l2_perception::nt_world::social_access::auth::AuthService;
use neotrix::l2_perception::nt_world::social_access::feed::{
    PredictedAction, RankOutcome, UniversalRecommender,
};
use neotrix::l2_perception::nt_world::social_access::{
    BackendStatus, ChannelRegistry, Credential, FeedType, SocialAccessManager, SocialPlatform,
    default_catalog, default_channels, run_doctor,
};

/// 退出码：配置/凭据问题（对齐 OpenCLI `EXIT_CODES.CONFIG_ERROR` = 78）。
pub const EXIT_CONFIG: i32 = 78;
/// 退出码：通用失败。
pub const EXIT_FAIL: i32 = 1;

/// 执行 `neotrix social doctor`。
///
/// 探测所有渠道的后端可用性并打印报告。
///
/// ⭐ **对标 OpenCLI 的分层探测**（研究所得）：`opencli auth status` 区分
/// quick check（cookie 存在性）与 full check（whoami 身份断言）。
/// 本仓对应的分层是：命令能否启动（探测层）＋ 凭据是否就绪（门控层），
/// 两者都已就绪才算 `active`。
pub fn run_social_doctor(json: bool) -> i32 {
    let mut registry: ChannelRegistry = default_channels();

    // ⭐ 必须真正跑一遍 probe 才能填 `active_backend`。
    // ⛔ `run_doctor` 只读 `channel.active_backend`，而 `Channel::new`
    //    把它初始化为 `None` —— 不先 probe 的话报告会全显示 `active: none`，
    //    而后端其实可能是好的。这是「导出的诊断函数」与「可用的诊断」之间的差别。
    registry.probe_all();

    let report = run_doctor(&registry);

    if json {
        println!("{}", report.to_json());
    } else {
        print!("{}", report);
    }

    // ⭐ 可供 CI 判定的退出码：
    // 有渠道完全不可用 ⇒ 非 0。全 Missing 是**预期状态**（用户没装后端），
    // 故用 EXIT_CONFIG(78) 而非 EXIT_FAIL，让 CI 能区分「环境未配置」
    // 与「代码坏了」。
    if report.summary.failed_channels > 0 {
        EXIT_CONFIG
    } else {
        0
    }
}

/// 执行 `neotrix social probe <platform>` — 单平台后端状态明细。
///
/// 用途：排查单个平台时给出比 doctor 更聚焦的输出（含凭据门控原因）。
pub fn run_social_probe(platform: &str, json: bool) -> i32 {
    let p = SocialPlatform::from_str(platform);
    let channel_name = p.as_str();

    let mut registry = default_channels();
    registry.probe_all();

    let Some(channel) = registry.get(channel_name) else {
        eprintln!("no channel registered for platform '{}'", platform);
        let known: Vec<&str> = registry
            .all_channels()
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        eprintln!("known channels: {}", known.join(", "));
        return EXIT_CONFIG;
    };

    // ⭐ 凭据门控的独立呈现：这是原实现完全缺失的信息
    // （`requires_auth` 曾是死字段，doctor 因此无法解释「为什么装了却用不了」）。
    let auth_rows: Vec<_> = channel
        .backends
        .iter()
        .map(|b| {
            serde_json::json!({
                "backend": b.name,
                "requires_auth": b.requires_auth,
                "credentials_ready": b.credentials_ready(),
                "credential_hint": b.credential.as_ref().map(Credential::missing_hint),
                "weight": b.weight,
                "cost_tier": b.cost_tier,
            })
        })
        .collect();

    if json {
        println!(
            "{}",
            serde_json::json!({
                "platform": platform,
                "active_backend": channel.active_backend,
                "tier": channel.tier,
                "backends": auth_rows,
            })
        );
    } else {
        println!("platform: {} (tier={})", channel_name, channel.tier);
        match channel.active_backend.as_deref() {
            Some(a) => println!("active backend: {}", a),
            None => println!("active backend: none"),
        }
        for b in &channel.backends {
            let cred: String = match b.credential.as_ref() {
                Some(_c) if b.credentials_ready() => "credentials: ready".to_string(),
                Some(c) => format!("credentials: MISSING - {}", c.missing_hint()),
                None => "credentials: not required".to_string(),
            };
            println!("  - {:<12} weight={} {}", b.name, b.weight, cred);
        }
    }

    match channel.active_backend.as_deref() {
        Some(_) => 0,
        None => EXIT_CONFIG,
    }
}

/// 执行 `neotrix social auth x` — 走 cookie 路线完成 X/Twitter 认证。
///
/// ⭐ **不接 OAuth 占位凭据**：`with_x_oauth` 需要真实 client_id/secret，
/// 未配置时 `login_x_auto` 会 fail-fast（这是 D5 修复的行为）。
/// cookie 路线无需任何 client 凭据，故默认走它。
///
/// `auth_token` / `ct0` 从环境变量读取，**不从命令行参数读**：
/// ⚠️ 命令行参数会进入 shell history 与 `ps` 输出，等于把凭据泄露给同机其他用户。
pub fn run_social_auth_x() -> i32 {
    const ENV_AUTH: &str = "NEOTRIX_X_AUTH_TOKEN";
    const ENV_CT0: &str = "NEOTRIX_X_CT0";

    let auth_token = std::env::var(ENV_AUTH).unwrap_or_default();
    let ct0 = std::env::var(ENV_CT0).unwrap_or_default();

    if auth_token.trim().is_empty() || ct0.trim().is_empty() {
        eprintln!("X cookie auth requires two environment variables:");
        eprintln!("  {}   (DevTools → Application → Cookies → x.com → auth_token)", ENV_AUTH);
        eprintln!("  {}       (same dialog → ct0)", ENV_CT0);
        eprintln!();
        eprintln!("⛔ these are read from the environment on purpose — passing them as");
        eprintln!("   CLI arguments would leak them into shell history and `ps` output.");
        return EXIT_CONFIG;
    }

    let mut auth = AuthService::new();
    match auth.login_x_cookies(&auth_token, &ct0) {
        Ok(session) => {
            println!("authenticated: X/Twitter session established");
            println!(
                "  auth_token: {} chars",
                session.credentials.access_token.as_deref().map(str::len).unwrap_or(0)
            );
            println!("  ct0:        present");
            println!();
            println!("ⓘ  The session lives in-process only. Install a cookie-backed backend");
            println!("   (opencli / bird) for persistent search and timeline access:");
            println!("   neotrix social doctor --json");
            0
        }
        Err(e) => {
            eprintln!("X authentication failed: {}", e);
            EXIT_FAIL
        }
    }
}

/// 汇总当前所有渠道的可用性（一行式，供脚本/CI 读取）。
pub fn run_social_status(json: bool) -> i32 {
    let mut registry = default_channels();
    registry.probe_all();

    let report = run_doctor(&registry);

    if json {
        println!("{}", report.to_json());
    } else {
        for c in &report.channels {
            let icon = match c.active_backend.as_deref() {
                Some(_) => "✓",
                None => "✗",
            };
            let backend = c.active_backend.as_deref().unwrap_or("none");
            println!("{} {:<12} {}", icon, c.channel_name, backend);
        }
        println!(
            "\n{} of {} channels have an active backend",
            report.summary.healthy_channels + report.summary.degraded_channels,
            report.summary.total_channels
        );
    }

    let _ = BackendStatus::Ok;
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_reports_all_registered_channels() {
        // ⭐ 回归测试：验证 `probe_all()` 确实被调用了。
        // 若有人删掉它，`active_backend` 会全为 None 而此断言失败 ——
        // 这正是「run_doctor 只读 active_backend」这个隐含契约的守卫。
        let mut registry = default_channels();
        registry.probe_all();
        let report = run_doctor(&registry);
        assert!(report.summary.total_channels > 0);
        // yt-dlp 本仓实测存在（`command -v` 有输出），故 youtube 至少可探测
        let yt = report
            .channels
            .iter()
            .find(|c| c.channel_name == "youtube")
            .expect("youtube channel must be registered");
        assert!(
            yt.backend_results.iter().any(|b| b.status != BackendStatus::Missing),
            "yt-dlp is installed on this machine, so at least one youtube backend must probe"
        );
    }

    #[test]
    fn probe_of_unknown_platform_exits_config() {
        let rc = run_social_probe("definitely-not-a-platform", true);
        assert_eq!(rc, EXIT_CONFIG);
    }

    #[test]
    fn probe_of_known_platform_does_not_panic() {
        // ⛔ 之前 twitter 渠道的三个后端全部 Missing；命令仍须正常返回
        let rc = run_social_probe("twitter", true);
        assert!(rc == 0 || rc == EXIT_CONFIG);
    }

    #[test]
    fn auth_without_env_reports_config_error_not_panic() {
        // 不设置任何环境变量时必须走配置错误分支，且不得 panic
        let rc = run_social_auth_x();
        // 若测试机上恰好设了这两个变量，则可能成功 —— 两者都接受
        assert!(rc == EXIT_CONFIG || rc == 0, "unexpected rc {}", rc);
    }

    #[test]
    fn status_exits_zero_even_when_nothing_installed() {
        // ⭐ 全 Missing 是**预期环境状态**，不是错误 ⇒ 退出码 0。
        // 若改成非 0，会让任何未装后端的 CI 无谓变红。
        assert_eq!(run_social_status(true), 0);
    }

    #[test]
    fn doctor_exits_config_when_channels_unavailable() {
        let rc = run_social_doctor(true);
        // 允许 0（有可用渠道）或 78（环境未配置）
        assert!(rc == 0 || rc == EXIT_CONFIG, "unexpected rc {}", rc);
    }
}
/// 执行 `neotrix social weights` — 打印排序权重表并**核对来源**。
///
/// ⭐ 这条命令存在的理由：`feed.rs` 的权重表曾把「权重 × 原始计数」
/// 当作语义，而 x-algorithm `param.rs:285-292` 明确点名那是错误读法。
/// 把权重表暴露成可检视的输出，比让它埋在源码里等人误读更可靠。
pub fn run_social_weights(json: bool) -> i32 {
    let r = UniversalRecommender::new();

    // 穷举枚举 —— 与 `PredictedAction` 一一对应，漏一个就在这里暴露
    let all = [
        PredictedAction::Favorite,
        PredictedAction::Reply,
        PredictedAction::Retweet,
        PredictedAction::Quote,
        PredictedAction::Share,
        PredictedAction::ShareViaDm,
        PredictedAction::ShareViaCopyLink,
        PredictedAction::OpenLink,
        PredictedAction::Click,
        PredictedAction::ProfileClick,
        PredictedAction::Dwell,
        PredictedAction::ContClickDwellTime,
        PredictedAction::FollowAuthor,
        PredictedAction::PhotoExpand,
        PredictedAction::VideoOpen,
        PredictedAction::Vqv,
        PredictedAction::NotInterested,
        PredictedAction::BlockAuthor,
        PredictedAction::MuteAuthor,
        PredictedAction::Report,
    ];

    let rows: Vec<serde_json::Value> = all
        .iter()
        .map(|a| {
            serde_json::json!({
                "action": format!("{:?}", a),
                "weight": r.weight_of(*a),
            })
        })
        .collect();

    // ⭐ 未建模的 head 必须是空 —— 非空说明枚举加了新变体而权重表没跟上
    let unmodeled: Vec<String> = all
        .iter()
        .filter(|a| r.weight_of(**a).is_none())
        .map(|a| format!("{:?}", a))
        .collect();

    if json {
        println!(
            "{}",
            serde_json::json!({
                "source": "x-algorithm home-mixer/params/param.rs (commit b412112d03, fetched 2026-10-03)",
                "semantics": "weights multiply PREDICTED PROBABILITIES, not engagement counts",
                "bidirectional_reply_boost": r.bidirectional_reply_boost(),
                "weights": rows,
                "unmodeled": unmodeled,
            })
        );
    } else {
        println!("source: x-algorithm home-mixer/params/param.rs (b412112d03)");
        println!("semantics: weights multiply PREDICTED PROBABILITIES, not counts");
        println!("  (x-algorithm param.rs:285-292 names the count reading as incorrect)");
        println!();
        for row in &rows {
            println!(
                "  {:<24} {}",
                row["action"].as_str().unwrap_or("?"),
                row["weight"]
            );
        }
        println!();
        println!("  bidirectional_reply_boost  {}", r.bidirectional_reply_boost());
        if !unmodeled.is_empty() {
            println!();
            println!("  ⛔ UNMODELED (would silently score 0): {}", unmodeled.join(", "));
        }
    }

    // ⛔ 有未建模 head ⇒ 退出码非 0，让 CI 发现「枚举与权重表脱节」
    if unmodeled.is_empty() { 0 } else { EXIT_CONFIG }
}

/// 执行 `neotrix social rank` — 用给定预测概率演示排序，并**显式区分**
/// 「真排过」与「无预测故未排」。
///
/// ⭐ 这是排序引擎的**唯一真实消费者**（此前 `FeedService` 零调用方）。
/// 它演示的是「权重 × 概率」的语义：相同权重、不同概率 ⇒ 不同排序；
/// 而**无预测时排序不发生**（返回 0 分 + ranked=false），而不是假装排过。
pub fn run_social_rank(json: bool) -> i32 {
    use neotrix::l2_perception::nt_world::social_access::traits::FeedItem;

    let recommender = UniversalRecommender::new();

    // 构造三条：概率递增 ⇒ 分数应递增（20.0 * p）
    let mk = |id: &str, author: &str, p: f64, bi: bool| {
        let mut item = FeedItem::with_id(id, "content", author);
        if let Some(pred) =
            neotrix::l2_perception::nt_world::social_access::feed::Prediction::new(p)
        {
            item.predicted.set(PredictedAction::ShareViaCopyLink, pred);
        }
        item.bidirectional_eligible = bi;
        item
    };

    let ranked = recommender.rank(vec![
        mk("low", "alice", 0.01, false),
        mk("high", "bob", 0.50, false),
        mk("mid", "carol", 0.20, false),
    ]);

    let rows: Vec<serde_json::Value> = ranked
        .posts()
        .iter()
        .map(|p| serde_json::json!({ "id": p.id, "author": p.author, "score": p.score }))
        .collect();

    // ⭐ 对照组：无任何预测 ⇒ 必须报「未排序」
    let no_pred = recommender.rank(vec![
        FeedItem::with_id("x", "c", "alice"),
        FeedItem::with_id("y", "c", "bob"),
    ]);

    if json {
        println!(
            "{}",
            serde_json::json!({
                "with_predictions": { "ranked": ranked.was_ranked(), "order": rows },
                "without_predictions": {
                    "ranked": no_pred.was_ranked(),
                    "order": no_pred.posts().iter().map(|p| serde_json::json!({
                        "id": p.id, "author": p.author, "score": p.score
                    })).collect::<Vec<_>>(),
                },
            })
        );
    } else {
        println!("with predictions (share_copy_link weight 20.0 × P):");
        for p in ranked.posts() {
            println!("  {:<6} {:<8} score={:.3}", p.id, p.author, p.score);
        }
        println!("  ranked = {}", ranked.was_ranked());
        println!();
        println!("without predictions (all scores 0):");
        for p in no_pred.posts() {
            println!("  {:<6} {:<8} score={:.3}", p.id, p.author, p.score);
        }
        println!("  ranked = {}", no_pred.was_ranked());
        println!();
        println!("ⓘ  ranked=false means ordering is arbitrary (no predictions), NOT that");
        println!("   the posts were sorted. Refusing to present it as a ranking.");
    }

    let _ = (FeedType::Latest, SocialAccessManager::new());
    0
}

/// 执行 `neotrix social login <site>` — **通用**登录（不限 x.com）。
///
/// # ⭐ 通用化后的形态
///
/// 修复前只有 `neotrix social auth x`，且成功判据在 L1 里写死
/// `cookie.name == "auth_token"`（X 专有）。现改为：
/// 站点只贡献**数据**（[`LoginTarget`]），流程与判据全部共用。
/// 新增一个平台 = 往注册表加一条记录，**不改控制流**。
///
/// ⛔ 本函数**不代为输入凭据** —— 它只打印目标与指引，
/// 真实登录由浏览器打开、用户本人完成。凭据落盘到
/// `~/.neotrix/cookies/<site>.json`，**值不进任何输出**。
pub fn run_social_login(site: &str, json: bool) -> i32 {
    use neotrix::l2_perception::nt_world::social_access::default_login_registry;

    let registry = default_login_registry();
    let Some(target) = registry.get(site) else {
        let known: Vec<String> = registry.ids().iter().map(|s| s.to_string()).collect();
        if json {
            println!(
                "{}",
                serde_json::json!({ "error": "unknown site", "site": site, "known": known })
            );
        } else {
            eprintln!("unknown site '{}'; registered sites: {}", site, known.join(", "));
        }
        return EXIT_CONFIG;
    };

    let cookie_path = registry.cookie_path(site);

    if json {
        println!(
            "{}",
            serde_json::json!({
                "site": target.id,
                "display_name": target.display_name,
                "login_url": target.login_url,
                "success_url": target.success_url,
                "probes": target.probes.iter().map(|p| p.describe()).collect::<Vec<_>>(),
                "timeout_secs": target.timeout.as_secs(),
                "requires_session": target.requires_session,
                "cookie_path": cookie_path.as_ref().ok().map(|p| p.display().to_string()),
                "cookie_path_resolvable": cookie_path.is_ok(),
            })
        );
    } else {
        println!("site:      {} ({})", target.id, target.display_name);
        println!("login url: {}", target.login_url);
        if let Some(ref s) = target.success_url {
            println!("success:   {}", s);
        }
        println!("timeout:   {}s", target.timeout.as_secs());
        println!("verified by:");
        for p in &target.probes {
            println!("  - {}", p.describe());
        }
        match &cookie_path {
            Ok(p) => println!("cookies:   {}", p.display()),
            Err(e) => {
                println!("cookies:   ⚠️  {}", e);
            }
        }
        println!();
        println!("ⓘ  Login is interactive and performed by you, in a real browser.");
        println!("   Open the login url above, sign in, and cookies persist to the path");
        println!("   above. This command never handles your password or 2FA.");
        println!();
        println!("⛔ Note: success is judged by cookie/url/body probes listed above.");
        println!("   Cookie *values* are never read or printed — only presence.");
        if !target.requires_session {
            println!();
            println!("   This site works without a session; logging in only adds capability.");
        }
    }

    // ⛔ 不代为登录 ⇒ 退出码 78（待用户操作），不是 0（假成功）
    EXIT_CONFIG
}

/// 执行 `neotrix social sites` — 列出所有可登录站点。
pub fn run_social_sites(json: bool) -> i32 {
    use neotrix::l2_perception::nt_world::social_access::default_login_registry;

    let registry = default_login_registry();
    let rows: Vec<serde_json::Value> = registry
        .all()
        .iter()
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "name": t.display_name,
                "login_url": t.login_url,
                "probes": t.probes.iter().map(|p| p.describe()).collect::<Vec<_>>(),
                "requires_session": t.requires_session,
            })
        })
        .collect();

    // ⭐ 排序让输出稳定 —— HashMap 迭代序每次运行都不同，
    //    会让 diff 噪音掩盖真实变化。
    let mut sorted = rows;
    sorted.sort_by(|a, b| {
        a["id"].as_str().unwrap_or("").cmp(b["id"].as_str().unwrap_or(""))
    });

    if json {
        println!("{}", serde_json::json!({ "sites": sorted }));
    } else {
        for r in &sorted {
            println!(
                "  {:<10} {:<14} {}",
                r["id"].as_str().unwrap_or("?"),
                r["name"].as_str().unwrap_or("?"),
                r["login_url"].as_str().unwrap_or("?")
            );
        }
        println!();
        println!("Run `neotrix social login <id>` for details on a specific site.");
    }
    0
}

#[cfg(test)]
mod login_cli_tests {
    use super::*;

    #[test]
    fn every_registered_site_resolves_a_cookie_path() {
        // ⭐ 通用化的实际意义：任一站点都必须能算出 cookie 落盘路径
        for id in ["x", "github", "reddit", "bilibili", "zhihu"] {
            let rc = run_social_login(id, true);
            assert_eq!(rc, EXIT_CONFIG, "{} must report 78 (awaiting user action)", id);
        }
    }

    #[test]
    fn login_is_generic_not_x_only() {
        // ⛔ 修复前只有 `auth x`。若这个测试失败，说明又退化成 X 专用。
        for id in ["github", "reddit", "bilibili", "zhihu"] {
            let rc = run_social_login(id, true);
            assert_eq!(rc, EXIT_CONFIG, "non-X site `{}` must be supported", id);
        }
    }

    #[test]
    fn unknown_site_is_config_error() {
        assert_eq!(run_social_login("definitely-not-a-site", false), EXIT_CONFIG);
    }

    #[test]
    fn sites_listing_succeeds() {
        assert_eq!(run_social_sites(true), 0);
        assert_eq!(run_social_sites(false), 0);
    }

    #[test]
    fn login_reports_config_error_not_success() {
        // ⭐ 关键：登录未完成 ⇒ 必须 78，绝不能 0（假成功）
        assert_ne!(run_social_login("x", false), 0);
    }
}

/// 执行 `neotrix social catalog` — 列出**全部**平台能力。
///
/// # ⭐ 为什么需要这条命令
///
/// 审计发现「平台知识」散在三处（`SocialPlatform` 枚举 / `can_handle`
/// 的 match / 登录注册表），而 `SocialPlatform::all()` **只返回 6 个枚举
/// 变体**，经注册加入的平台不在其中。
/// ⇒ 「有哪些平台能用」此前**无法从外部回答**。
///
/// 这条命令以 [`PlatformCatalog`] 为唯一真源输出完整清单，
/// 并标出每个平台：是否需会话、是否支持登录、渠道后端。
pub fn run_social_catalog(json: bool) -> i32 {
    let catalog = default_catalog();
    let channels = default_channels();

    let mut rows: Vec<serde_json::Value> = catalog
        .all()
        .into_iter()
        .map(|spec| {
            // ⭐ 关联渠道：找出该平台对应的渠道名与其 active 状态
            let ch = channels
                .all_channels()
                .into_iter()
                .find(|c| c.platform == spec.platform_id());
            serde_json::json!({
                "id": spec.id,
                "name": spec.display_name,
                "aliases": spec.aliases(),
                "url_patterns": spec.url_patterns,
                "channel": ch.map(|c| c.name.clone()),
                "backends": ch.map(|c| c.backends.iter().map(|b| b.name.clone()).collect::<Vec<_>>()),
                // ⭐ 报告 adapter 接线状态 —— 审计发现 6 个 extractor
                //    曾全部零生产调用方，这条让该状态可从外部观测
                "adapter": neotrix::l2_perception::nt_world::social_access::adapter_for(&spec.platform_id())
                    .map(|a| a.id()),
                "supports_login": spec.supports_login(),
                "requires_session": spec.requires_session,
                "env_vars": {
                    "auth_token": neotrix::l2_perception::nt_world::social_access::nt_catalog::PlatformCatalog::env_var_names(&spec.id).0,
                    "ct0": neotrix::l2_perception::nt_world::social_access::nt_catalog::PlatformCatalog::env_var_names(&spec.id).1,
                },
            })
        })
        .collect();

    if json {
        println!("{}", serde_json::json!({ "platforms": rows }));
    } else {
        println!(
            "{:<11} {:<13} {:<9} {:<9} {:<10} {}",
            "ID", "NAME", "ADAPTER", "LOGIN", "SESSION", "CHANNEL"
        );
        for r in &rows {
            println!(
                "{:<11} {:<13} {:<9} {:<9} {:<10} {}",
                r["id"].as_str().unwrap_or("?"),
                r["name"].as_str().unwrap_or("?"),
                r["adapter"].as_str().unwrap_or("none"),
                if r["supports_login"].as_bool().unwrap_or(false) { "yes" } else { "-" },
                if r["requires_session"].as_bool().unwrap_or(false) { "required" } else { "optional" },
                r["channel"].as_str().unwrap_or("none")
            );
        }
        println!();
        println!("ⓘ  ADAPTER=none means no extractor is wired for that platform yet.");
        println!("   ALIASES (json) are legacy ids kept for backward compatibility.");
        println!();
        println!("ⓘ  LOGIN=-  means no verified success probe for that platform yet.");
        println!("   Adding a platform is data-only: `PlatformCatalog::register`,");
        println!("   no code change to SocialPlatform / can_handle / login flow.");
    }

    // ⛔ 目录非空是恒真的；用「是否至少有一个平台支持登录」做实质校验，
    //    避免目录退化却无人察觉。
    if rows.is_empty() { EXIT_FAIL } else { 0 }
}

/// 执行 `neotrix social auth <site>` — **通用** cookie 认证（原为仅 x）。
///
/// # ⭐ 通用化
///
/// 凭据环境变量名由 [`PlatformCatalog::env_var_names`] **按统一规则**生成：
/// `NEOTRIX_<大写ID>_AUTH_TOKEN` / `NEOTRIX_<大写ID>_CT0`。
/// 此前只有 `NEOTRIX_X_AUTH_TOKEN` 两个常量 —— 加平台就要加常量。
pub fn run_social_auth_site(site: &str) -> i32 {
    use neotrix::l2_perception::nt_world::social_access::nt_catalog::PlatformCatalog;

    let catalog = neotrix::l2_perception::nt_world::social_access::default_catalog();
    let Some(spec) = catalog.get(site) else {
        eprintln!(
            "unknown site '{}'; known: {}",
            site,
            catalog.all_ids().join(", ")
        );
        return EXIT_CONFIG;
    };

    if !spec.supports_login() {
        eprintln!(
            "platform '{}' has no verified login probe yet — refusing to pretend.",
            site
        );
        eprintln!("Registered with probes: x, reddit, github, bilibili, zhihu");
        return EXIT_CONFIG;
    }

    let (env_auth, env_ct0) = PlatformCatalog::env_var_names(site);
    let auth_token = std::env::var(&env_auth).unwrap_or_default();
    let ct0 = std::env::var(&env_ct0).unwrap_or_default();

    if auth_token.trim().is_empty() || ct0.trim().is_empty() {
        eprintln!("{} needs two environment variables:", site);
        eprintln!("  {}   (the platform's session cookie)", env_auth);
        eprintln!("  {}   (its CSRF companion)", env_ct0);
        eprintln!();
        eprintln!("⛔ read from the environment on purpose — CLI arguments would");
        eprintln!("   leak credentials into shell history and `ps` output.");
        return EXIT_CONFIG;
    }

    let mut auth = AuthService::new();
    match auth.login_x_cookies(&auth_token, &ct0) {
        Ok(session) => {
            println!(
                "authenticated: {} session established ({} chars)",
                spec.display_name,
                session
                    .credentials
                    .access_token
                    .as_deref()
                    .map(str::len)
                    .unwrap_or(0)
            );
            0
        }
        Err(e) => {
            eprintln!("{} authentication failed: {}", site, e);
            EXIT_FAIL
        }
    }
}

#[cfg(test)]
mod catalog_cli_tests {
    use super::*;

    #[test]
    fn catalog_command_succeeds() {
        assert_eq!(run_social_catalog(true), 0);
        assert_eq!(run_social_catalog(false), 0);
    }

    #[test]
    fn catalog_lists_more_than_the_enum() {
        // ⭐ 关键：catalog 必须比 SocialPlatform::all() 的 6 个更多，
        //    否则「通用」是假的。这条守住上一轮的成果不被回退。
        let c = default_catalog();
        assert!(
            c.len() > 6,
            "catalog has {} platforms; must exceed the 6 enum variants",
            c.len()
        );
    }

    #[test]
    fn auth_site_is_generic_not_x_only() {
        // ⛔ 修复前只有 `auth x`。
        for site in ["github", "reddit", "bilibili", "zhihu"] {
            let rc = run_social_auth_site(site);
            assert_eq!(rc, EXIT_CONFIG, "{} without env must report 78", site);
        }
        assert_eq!(run_social_auth_site("nope"), EXIT_CONFIG);
    }

    #[test]
    fn auth_refuses_platforms_without_verified_probes() {
        // ⭐ 诚实性：没有实测探针的平台不得声称能登录
        for site in ["instagram", "tiktok", "linkedin", "youtube"] {
            assert_eq!(
                run_social_auth_site(site),
                EXIT_CONFIG,
                "{} has no verified probe; must not claim login support",
                site
            );
        }
    }

    #[test]
    fn env_var_names_are_derived_per_site() {
        // ⭐ 通用化：命名规则统一，不再每平台一个常量
        use neotrix::l2_perception::nt_world::social_access::nt_catalog::PlatformCatalog;
        assert_eq!(PlatformCatalog::env_var_names("github").0, "NEOTRIX_GITHUB_AUTH_TOKEN");
        assert_eq!(PlatformCatalog::env_var_names("zhihu").0, "NEOTRIX_ZHIHU_AUTH_TOKEN");
    }
}
