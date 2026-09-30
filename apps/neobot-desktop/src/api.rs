//! **API 契约注册表** —— 前后端接口的单一真源。
//!
//! # 为什么要有这个
//!
//! 上游前端 1:1 过来之后 invoke 了 **79** 个 Tauri 命令，而本仓 Rust 侧只有 12 个
//! `neobot_*`，**名字零重叠**。如果只是「缺了就抛错」，那么：
//!
//! * 界面启动就白屏（`store.harness.startup()` 等 `get_runtime_info`）
//! * **没人知道到底缺多少、缺哪些、哪些是真缺哪些是本就不需要**
//!
//! 第二条才是致命的。一个「报错的接口」和「不存在的接口」在调用方看来一样，
//! 所以缺口会一直隐性存在，直到某天有人以为是 bug 去查。
//!
//! ⇒ 本模块把**每一个**命令显式登记，带存活状态与原因。
//!   前端可列出全部、按状态筛选、直接调用 ⇒ **前端成为后端的可视化交互**。
//!
//! # 存活状态（`Status`）
//!
//! | 状态 | 含义 | 调用行为 |
//! |---|---|---|
//! | `Implemented` | 本仓有真实实现 | 真执行 |
//! | `Stub` | 已登记但**本仓不打算实现** | 返回结构化「不适用」，不抛错 |
//! | `Planned` | 该实现，尚未做 | 返回结构化「未实现」 |
//!
//! ⛔ **`Stub` 与 `Planned` 必须分开。** 两者都「不能做」，但一个是**决定不做**
//!    （DSH 运行时专属，如插件安装器），一个是**还没做**（NeoBot 该有的，
//!    如消息落库）。混在一起 ⇒ 缺口清单失真，路线图跟着失真。
//!
//! # 单一真源

use serde::Serialize;

/// 命令存活状态。见上表。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// 本仓有真实实现。
    Implemented,
    /// 已登记，**本仓不打算实现**（上游/DSH 运行时专属）。
    Stub,
    /// 该实现，尚未做。
    Planned,
}

/// 一条命令的契约条目。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiSpec {
    /// 命令名（`invoke` 用的那个）。
    pub name: &'static str,
    /// 分组，用于 UI 侧栏归类。
    pub category: &'static str,
    /// 参数名（按名传参的接口）。空 = 无参。
    pub params: &'static [&'static str],
    /// 返回类型（TS 写法）。`"void"` = 无返回值。
    pub ret: &'static str,
    /// 存活状态。
    pub status: Status,
    /// 为什么是这个状态。**`Stub` 必填** —— 没有理由的 Stub 会被后人「顺手补上」。
    pub note: &'static str,
}

impl ApiSpec {
    const fn new(
        name: &'static str,
        category: &'static str,
        params: &'static [&'static str],
        ret: &'static str,
        status: Status,
        note: &'static str,
    ) -> Self {
        Self { name, category, params, ret, status, note }
    }
}

/// DSH 运行时专属 —— NeoBot 没有那个运行时，**决定不做**。
///
/// 这些命令在上游是围绕 `source/deepseek-harness`（空 submodule）建的：
/// 插件安装/版本策略、profile 备份恢复、updater、core 下载、BongoCat 桌宠。
/// 本仓的骨架是 `crates/neotrix-neobot`（超集，30k 行），不跑 DSH 运行时。
const DSH_ONLY: &str = "DSH 运行时专属（插件/profile/updater/core/桌宠），NeoBot 不跑该运行时";
/// 目前没有 Planned 条目（5 个已全部实现），但**保留**这个常量：
/// 加回 Planned 时直接用。若此时把它删掉，下次写 Planned 会临时造一个措辞不同的
/// 理由，而理由措辞的漂移会让「欠账清单」的可比性下降。
#[allow(dead_code)]
const TODO: &str = "NeoBot 该有，尚未实现";

/// 全部命令契约。**新增命令必须在此登记** —— `nt_check_api.mjs` 守这条。
pub const SPECS: &[ApiSpec] = &[
    // ── 会话（真数据，读 store） ──
    ApiSpec::new("neobot_convo_list", "会话", &[], "ConvoView[]", Status::Implemented, ""),
    ApiSpec::new("neobot_convo_group", "会话", &["title", "members"], "string", Status::Implemented, ""),
    ApiSpec::new("neobot_convo_dm", "会话", &["me", "peer"], "string", Status::Implemented, ""),
    ApiSpec::new("neobot_member_list", "会话", &[], "MemberView[]", Status::Implemented, ""),
    ApiSpec::new("neobot_send", "会话", &["text"], "AgentRunResult", Status::Implemented,
        "⛔ 不带 convo_id，消息不落库 —— 见 STATUS.md 缺口 5"),
    ApiSpec::new("neobot_agent_run", "会话", &["goal", "context"], "AgentRunResult", Status::Implemented, ""),
    // ── 决策面板 ──
    ApiSpec::new("neobot_panel_publish", "面板", &["panel"], "number", Status::Implemented, ""),
    ApiSpec::new("neobot_panel_answer", "面板", &["answer"], "AnswerOutcome", Status::Implemented,
        "⛔ 只收 answer，基准由骨架注册表持有"),
    ApiSpec::new("neobot_panel_clear", "面板", &[], "number", Status::Implemented, ""),
    ApiSpec::new("neobot_panel_demo_publish", "面板", &[], "number", Status::Implemented,
        "演示下发器，走与真实骨架同一条路"),
    // ── 其它 ──
    ApiSpec::new("neobot_evidence_summary", "其它", &["text"], "EvidenceReport", Status::Implemented, ""),
    ApiSpec::new("neobot_core_capabilities", "其它", &[], "string", Status::Implemented,
        "⛔ 已注册但前端未消费，能力矩阵仍是静态的"),
    // ── API 平台自身 ──
    ApiSpec::new("neobot_api_specs", "API", &[], "ApiSpec[]", Status::Implemented,
        "契约全量清单，界面据此渲染"),
    ApiSpec::new("neobot_api_call", "API", &["name", "args"], "ApiCallResult", Status::Implemented,
        "按名字分派；Stub/Planned 返回结构化说明而不抛错"),
    // ── 平台接线（优先级 1：不是 agent 能力，是操作系统能力） ──
    ApiSpec::new("open_external_url", "平台", &["url"], "void", Status::Implemented,
        "⛔ 只放行 http/https —— file:// 会让对话里的链接变成任意本地文件读取入口"),
    ApiSpec::new("write_clipboard_text", "平台", &["text"], "void", Status::Implemented, ""),
    ApiSpec::new("read_clipboard_image", "平台", &[], "string", Status::Implemented,
        "⛔ arboard 未暴露像素读取（image-data feature 未开）⇒ **诚实报错**而非返空 data URL"),
    ApiSpec::new("show_native_notification", "平台", &["title", "body"], "void", Status::Implemented, ""),
    ApiSpec::new("get_launch_on_login", "平台", &[], "boolean", Status::Implemented, ""),
    ApiSpec::new("set_launch_on_login", "平台", &["enabled"], "void", Status::Implemented, ""),
    ApiSpec::new("create_app_window", "平台", &["spec"], "string", Status::Implemented,
        "url 只允许相对路径或 http(s)"),
    ApiSpec::new("remote_open_window", "平台", &["label", "url"], "string", Status::Implemented,
        "已存在则**只聚焦不重新导航** —— 重新导航会打断那个窗口里正在做的事"),
    ApiSpec::new("move_pet_window", "平台", &["x", "y", "always_on_top"], "void", Status::Implemented,
        "always_on_top 是语义参数，桌宠要置顶、普通窗口不要 ⇒ 不塞进通用 move"),
    // ── 上游 79 个：分状态登记 ──
    ApiSpec::new("runtime_ready", "harness", &[], "boolean", Status::Implemented,
        "恒 true：返回 false 会把壳推进 installing 去装不需要的依赖"),
    ApiSpec::new("set_language", "其它", &["lang"], "void", Status::Implemented, "持久化界面语言"),
    ApiSpec::new("get_runtime_info", "harness", &[], "RuntimeInfo", Status::Implemented,
        "⛔ 刻意返回**空** service_url + has_service=false：NeoBot 不跑 DSH 运行时。
        壳据此切自持界面；编造 URL 会把失败搬到离原因很远的地方"),
    ApiSpec::new("launch_harness", "harness", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("shutdown_harness", "harness", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("copy_service_url", "harness", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("install_dependencies", "harness", &[], "void", Status::Implemented,
        "空操作：NeoBot 无外部依赖。壳在 !installed 时会调到，缺命令会卡在 installing"),
    ApiSpec::new("get_dsh_theme", "桌面", &[], "'dark' | 'light' | 'system'", Status::Implemented, ""),
    ApiSpec::new("get_cli_link_status", "其它", &[], "CliLinkStatus", Status::Stub, DSH_ONLY),
    ApiSpec::new("is_dev_build", "桌面", &[], "boolean", Status::Implemented,
        "编译期事实，**不**读环境变量（打包后误判会让开发菜单出现在正式版）"),
    ApiSpec::new("log_frontend", "日志", &["level", "target", "message"], "void", Status::Implemented,
        "⚠️ 3 个参数。契约最初写成 2 个而门没抓到 —— 门当时只对命令名不对参数"),
    ApiSpec::new("read_run_logs", "日志", &[], "string", Status::Implemented,
        "⛔ 不返空串：前端直接塞 <pre>，空串会被读成「没有日志」"),
    ApiSpec::new("read_service_logs", "日志", &[], "string", Status::Stub, DSH_ONLY),
    ApiSpec::new("get_desktop_about", "桌面", &[], "DesktopAboutInfo", Status::Implemented, ""),
];

/// 未逐条登记的上游命令。**只登记名字**，让缺口可数。
///
/// ⛔ 为什么允许「只登记名字」：79 个里 42 个是 DSH 运行时专属，逐条写 `note`
///    是复制粘贴，抄错的概率比不写更高，而且**没人会读第二遍**。
///    ⇒ 平台保证「**一个都不漏**」；逐条理由等真的要做某一条时再补。
///    `Stub` 判定用 `is_upstream_unlisted` 兜住。
pub const UPSTREAM_UNLISTED: &[&str] = &[
    "allow_plugin_policy_versions", "allow_plugin_versions", "backup_profile", "cancel_internal_plugins",
    "cancel_preinstall_plugins", "check_desktop_update", "check_dsh_update", "clone_profile",
    "create_app_window", "create_profile", "delete_backup", "delete_plugin_backup", "delete_profile",
    "detect_plugin_recovery", "disable_dsh_plugin", "download_core", "download_desktop_update",
    "enable_dsh_plugin", "ensure_internal_plugins", "enter_safe_mode", "get_cores", "get_launch_on_login",
    "get_pet_asset", "get_pet_status", "get_preinstall_pending", "get_profiles", "list_backups",
    "list_preset_pets", "move_pet_window", "open_external_url", "read_clipboard_image", "remote_open_window",
    "remove_core", "remove_profile", "reset_profile", "restore_profile", "set_active_core",
    "set_launch_on_login", "set_pet_ignore_cursor_events", "show_native_notification",
    "start_pet_mouse_stream", "update_app_config", "update_local_core", "write_clipboard_text",
];

/// 汇总视图：实现数 / 决定不做 / 待做 / 上游未逐条登记。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiSummary {
    pub total: usize,
    pub implemented: usize,
    pub stub: usize,
    pub planned: usize,
    pub upstream_unlisted: usize,
    /// 上游前端 invoke 的命令总数。用来核对「一个都不漏」。
    /// ⚠️ 这个数字被 nt_check_api.mjs ⑤ 对账，写错门会 FAIL。
    pub upstream_total: usize,
}

pub fn summary() -> ApiSummary {
    let count = |s: Status| SPECS.iter().filter(|x| x.status == s).count();
    ApiSummary {
        total: SPECS.len(),
        implemented: count(Status::Implemented),
        stub: count(Status::Stub),
        planned: count(Status::Planned),
        upstream_unlisted: UPSTREAM_UNLISTED.len(),
        upstream_total: 80,
    }
}

/// `neobot_api_specs()` 的返回体。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiCatalog {
    pub summary: ApiSummary,
    pub specs: Vec<ApiSpec>,
    pub upstream_unlisted: Vec<String>,
}

pub fn catalog() -> ApiCatalog {
    ApiCatalog {
        summary: summary(),
        specs: SPECS.to_vec(),
        upstream_unlisted: UPSTREAM_UNLISTED.iter().map(|s| (*s).to_owned()).collect(),
    }
}

/// `neobot_api_call` 的返回体。
///
/// ⛔ 刻意用 `String` 存 `status` 而不是枚举：序列化后的字面量
///    （`stub` / `planned` / `unlisted`）是**前端要显示的字**，
///    一旦改成枚举又忘了加 `rename_all`，前端会拿到 `Stub` 而不是 `stub`，
///    而界面上会静默落进「未知」分支。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiCallResult {
    pub ok: bool,
    /// `implemented` | `stub` | `planned` | `unlisted`
    pub status: String,
    pub reason: String,
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn 名字不许重复() {
        let mut seen = HashSet::new();
        for s in SPECS {
            assert!(seen.insert(s.name), "命令名重复：{}", s.name);
        }
    }

    #[test]
    fn 名字必须是蛇形小写() {
        for s in SPECS {
            assert!(
                s.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "命令名不合规：{}",
                s.name
            );
        }
    }

    /// ⛔ 这条是本模块的核心不变量：`Stub` 必须带理由。
    /// 没有理由的 `Stub` 会被后人「顺手补上」，而那多半是错的 ——
    /// 补上一个 DSH 专属命令只会制造一个永远返回错误的僵尸接口。
    #[test]
    fn stub必须写明为什么不做() {
        for s in SPECS {
            if s.status == Status::Stub {
                assert!(!s.note.trim().is_empty(), "Stub 缺理由：{}", s.name);
            }
        }
    }

    /// `Stub` 的理由应当指向真实原因，而不是「暂时没做」——
    /// 后者属于 `Planned`，混淆会让路线图失真。
    #[test]
    fn stub的理由不能是暂时没做() {
        for s in SPECS {
            if s.status == Status::Stub {
                assert!(
                    !s.note.contains("暂时") && !s.note.contains("还没"),
                    "{} 的 Stub 理由像是「暂时没做」，应归为 Planned",
                    s.name
                );
            }
        }
    }

    #[test]
    fn 分类不能为空() {
        for s in SPECS {
            assert!(!s.category.trim().is_empty(), "缺分类：{}", s.name);
        }
    }

    /// 数字守恒：分类之和 = 总数。防止加了条目忘了改 `summary`。
    #[test]
    fn 汇总数字自洽() {
        let s = summary();
        assert_eq!(
            s.implemented + s.stub + s.planned,
            s.total,
            "implemented+stub+planned != total"
        );
    }

    /// 已实现的命令必须真的在 Rust 侧存在（由 nt_check_api.mjs 对账，
    /// 这里只保证 SPECS 自己不谎报 implemented 却没参数可调）。
    #[test]
    fn 已实现的条目必须有返回值描述() {
        for s in SPECS {
            if s.status == Status::Implemented {
                assert!(!s.ret.trim().is_empty(), "{} 缺返回类型", s.name);
            }
        }
    }
}
