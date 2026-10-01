//! **API 契约注册表** —— 前后端接口的单一真源。
//!
//! # 为什么要有这个
//!
//! 上游前端 1:1 过来之后 invoke 上游命令，而本仓 Rust 侧起步只有 14 个
//! `neobot_*`。如果只是「缺了就抛错」，那么：
//!
//! * 界面启动就白屏（`store.harness.startup()` 等 `get_runtime_info`）
//! * **没人知道到底缺多少、缺哪些、哪些是真缺哪些是本就不需要**
//!
//! 第二条才是致命的。一个「报错的接口」和「不存在的接口」在调用方看来一样，
//! 所以缺口会一直隐性存在，直到某天有人以为是 bug 去查。
//!
//! ⇒ 本模块把上游 96 个命令**逐条**登记（`desktop/builder.rs` 的
//! `generate_handler!` 为分母），带存活状态与原因。
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
    /// 已登记，**本仓不打算实现**（上游/DSH 运行时专属），且**刻意未注册**。
    ///
    /// ⛔ 与 `Refused` 的区别不是「做没做」，是**注册了没有**：
    ///   未注册 ⇒ 调用方拿到 `command not found`，那读起来像 bug；
    ///   `Refused` ⇒ 拿到一句「本仓不提供 X」，那读起来像决定。
    /// 凡是「界面上真的会调到」的 DSH 专属命令，都走 `Refused` 而不是 `Stub`。
    Stub,
    /// 已登记并**已注册**，但实现是一句明确的拒绝（上游/DSH 运行时专属）。
    ///
    /// 上游有 11 处 `invoke` 落在本仓不提供的体系上（插件/档案/CLI 链接）。
    /// 不注册它们时界面**静默失效**（react-query 回落到 `[]` 画出空列表），
    /// 点了才报 `command not found` —— 而没人会为空白面板点按钮。
    /// 注册成 `Refused` 后同一个操作给出一句人话。
    Refused,
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
/// `Refused` 专用理由：与 DSH_ONLY 的区别是**已注册**，调用方拿到的是一句拒绝。
const REFUSED: &str = "本仓不提供：已注册为显式拒绝，调用会得到一句原因而非 command not found";
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
    ApiSpec::new("neobot_member_list", "会话", &[], "MemberView[]", Status::Implemented,
        "成员清单（id/kind）；建会话要求成员已登记，故配套 neobot_member_add"),
    ApiSpec::new("neobot_member_add", "会话", &["id", "kind"], "null", Status::Implemented,
        "登记成员（human|agent，幂等）；会话创建的前置条件"),
    ApiSpec::new("neobot_send", "会话", &["convo_id", "text"], "AgentRunResult", Status::Implemented,
        "会话内一轮：问落库 → 跑 → 答落库；convo 缺席 = 脱离会话手动跑"),
    ApiSpec::new("neobot_convo_messages", "会话", &["convo_id"], "ChatMessage[]", Status::Implemented,
        "切会话时读历史；⛔ 之前只换标题不换消息流（串台）"),
    ApiSpec::new("neobot_skill_list", "其它", &[], "SkillListView", Status::Implemented,
        "技能清单（顶替上游插件页签）；skipped>0 = 有技能包装坏了"),
    ApiSpec::new("neobot_skill_install", "其它", &["path"], "null", Status::Implemented,
        "从绝对路径安装技能；路径不存在时 Err（不建空目录充成功）"),
    ApiSpec::new("neobot_usage_summary", "其它", &["days"], "UsageSummary", Status::Implemented,
        "日/周/月三档传 days（1/7/30），缺席=有史以来；文件缺席=零，坏文件=Err"),
    ApiSpec::new("neobot_memory_list", "其它", &[], "MemoryView", Status::Implemented,
        "读 MEMORY.md（逐轮注入 prompt）；带 bytes/cap/revisions，坏历史=Err"),
    ApiSpec::new("neobot_memory_add", "其它", &["text"], "bool", Status::Implemented,
        "记一行；true=记下/false=已有；密钥行库侧拒绝；⛔ 上限 8KiB"),
    ApiSpec::new("neobot_memory_undo", "其它", &[], "bool", Status::Implemented,
        "撤一版（可再撤）；true=撤了/false=无可撤"),
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
        "对话区顶栏消费（模型 · 工具数）；调不到就不渲染，不设静态默认"),
    // ── API 平台自身 ──
    ApiSpec::new("neobot_api_specs", "API", &[], "ApiSpec[]", Status::Implemented,
        "契约全量清单，界面据此渲染"),
    ApiSpec::new("neobot_api_call", "API", &["name", "args"], "ApiCallResult", Status::Implemented,
        "按名字分派；Stub/Planned 返回结构化说明而不抛错"),
    // ── 核心/备份/配置（优先级 2：接线，neotrix 库已有底层能力） ──
    ApiSpec::new("get_cores", "核心", &[], "HarnessCore[]", Status::Implemented,
        "⚠️ neotrix 的 core = provider+model，**不是** DSH 的引擎二进制"),
    ApiSpec::new("set_active_core", "核心", &["core"], "void", Status::Implemented,
        "按 id 查库，⛔ 不用界面回传整包（那份可能已过期）"),
    ApiSpec::new("remove_core", "核心", &["id"], "void", Status::Implemented, ""),
    ApiSpec::new("list_backups", "备份", &[], "BackupInfo[]", Status::Implemented,
        "⛔ 必须排序：目录遍历顺序不保证是时间序"),
    ApiSpec::new("delete_backup", "备份", &["path"], "void", Status::Implemented,
        "⛔ 只允许删备份目录内的路径，否则「删备份」=「删任意文件」"),
    ApiSpec::new("update_app_config", "配置", &["config"], "void", Status::Implemented,
        "⛔ **合并**写入。覆盖会把调用方没带的字段清成默认值"),
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
    // ── 上游 96 个：分状态登记 ──
    //
    // 分母是上游 `src-tauri` 的 96 个 `#[tauri::command]`
    //（`desktop/builder.rs` 的 `generate_handler!`，`builder.rs:1055` 起）。
    // 本仓已逐条登记 96 个上游同名 + 14 个 `neobot_*` + `neobot_api_*` 2 个（计入会话/API 类），
    // 无未展开项（`UPSTREAM_UNLISTED` 为空，见下）。
    // ── harness 生命周期：自持模式只留空操作，进程编排类决定不做 ──
    ApiSpec::new("runtime_ready", "harness", &[], "boolean", Status::Implemented,
        "恒 true：返回 false 会把壳推进 installing 去装不需要的依赖"),
    ApiSpec::new("get_runtime_info", "harness", &[], "RuntimeInfo", Status::Implemented,
        "⛔ 刻意返回**空** service_url + has_service=false：NeoBot 不跑 DSH 运行时。
        壳据此切自持界面；编造 URL 会把失败搬到离原因很远的地方"),
    ApiSpec::new("install_dependencies", "harness", &[], "void", Status::Implemented,
        "空操作：NeoBot 无外部依赖。壳在 !installed 时会调到，缺命令会卡在 installing"),
    ApiSpec::new("launch_harness", "harness", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("shutdown_harness", "harness", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("restart_harness", "harness", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("copy_service_url", "harness", &[], "void", Status::Refused,
        "自持形态无 iframe 服务地址可复制；应用面板有该按钮 ⇒ 注册为显式拒绝"),
    ApiSpec::new("get_dsh_status", "harness", &[], "DshStatus", Status::Stub, DSH_ONLY),
    ApiSpec::new("proxy_health_check", "harness", &[], "string", Status::Stub, DSH_ONLY),
    ApiSpec::new("enter_safe_mode", "harness", &[], "QuarantineReport", Status::Stub, DSH_ONLY),
    ApiSpec::new("quarantine_broken_patch_layers", "harness", &[], "QuarantineReport", Status::Stub, DSH_ONLY),
    ApiSpec::new("strip_unresolved_patch_entries", "harness", &[], "void", Status::Stub, DSH_ONLY),
    // ── 核心通道：版本分发类决定不做（本地优先架构无二进制通道） ──
    ApiSpec::new("check_dsh_update", "核心", &[], "DshUpdateInfo", Status::Stub, DSH_ONLY),
    ApiSpec::new("download_core", "核心", &["id"], "void", Status::Refused, REFUSED),
    ApiSpec::new("update_local_core", "核心", &[], "string", Status::Refused, REFUSED),
    // ── 插件（14 个，DSH 运行时经 dsh CLI / pnpm 执行，本仓无该运行时） ──
    ApiSpec::new("get_dsh_plugins", "插件", &[], "DshPlugin[]", Status::Refused, REFUSED),
    ApiSpec::new("refresh_plugin_updates", "插件", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("update_dsh_plugin", "插件", &["id"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("remove_dsh_plugin", "插件", &["id"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("disable_dsh_plugin", "插件", &["id"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("enable_dsh_plugin", "插件", &["id"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("snapshot_plugin", "插件", &["id"], "SnapshotInfo", Status::Stub, DSH_ONLY),
    ApiSpec::new("snapshot_plugins", "插件", &[], "SnapshotInfo[]", Status::Stub, DSH_ONLY),
    ApiSpec::new("get_plugin_backup", "插件", &["id"], "PluginBackup", Status::Refused, REFUSED),
    ApiSpec::new("restore_plugin", "插件", &["id"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("delete_plugin_backup", "插件", &["id"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("report_plugin_error", "插件", &["id", "error"], "void", Status::Refused, REFUSED),
    ApiSpec::new("detect_plugin_recovery", "插件", &[], "RecoveryInfo", Status::Stub, DSH_ONLY),
    ApiSpec::new("recover_plugin", "插件", &["id"], "void", Status::Stub, DSH_ONLY),
    // ── 预装 / 内置插件自愈（10 个，同上） ──
    ApiSpec::new("get_preinstall_plugins", "预装", &[], "PreinstallPlugin[]", Status::Stub, DSH_ONLY),
    ApiSpec::new("get_preinstall_pending", "预装", &[], "boolean", Status::Stub, DSH_ONLY),
    ApiSpec::new("install_preinstall_plugins", "预装", &["ids"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("cancel_preinstall_plugins", "预装", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("skip_preinstall_plugins", "预装", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("open_preinstall_repo", "预装", &["url"], "void", Status::Refused, REFUSED),
    ApiSpec::new("allow_plugin_versions", "预装", &["ids"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("allow_plugin_policy_versions", "预装", &["ids"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("ensure_internal_plugins", "预装", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("cancel_internal_plugins", "预装", &[], "void", Status::Stub, DSH_ONLY),
    // ── 档案 profile（8 个：$DSH_HOME/profiles 语义，本仓会话在 sqlite 里） ──
    ApiSpec::new("get_profiles", "档案", &[], "Profile[]", Status::Refused, REFUSED),
    ApiSpec::new("create_profile", "档案", &["name"], "Profile", Status::Refused, REFUSED),
    ApiSpec::new("set_active_profile", "档案", &["id"], "Profile", Status::Refused, REFUSED),
    ApiSpec::new("remove_profile", "档案", &["id"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("reset_profile", "档案", &["id"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("clone_profile", "档案", &["id", "name"], "Profile", Status::Stub, DSH_ONLY),
    ApiSpec::new("backup_profile", "档案", &["id"], "string", Status::Stub, DSH_ONLY),
    ApiSpec::new("restore_profile", "档案", &["id", "path"], "void", Status::Stub, DSH_ONLY),
    // ── 桌宠：开关/选择/大小/能力位已接（壳设置面）；清单与资源待接 ──
    ApiSpec::new("get_pet_status", "桌宠", &[], "PetStatus", Status::Implemented,
        "缺省关闭未选择；读失败回默认而不报错（pet 窗挂载即调，报错即白屏）"),
    ApiSpec::new("set_pet_enabled", "桌宠", &["enabled"], "PetStatus", Status::Implemented,
        "落盘 app-config.json 并推 pet://status"),
    ApiSpec::new("set_active_pet", "桌宠", &["id"], "PetStatus", Status::Implemented,
        "空串 = 清除选择；⛔ 不校验 id 是否存在（清单命令还是 Planned）"),
    ApiSpec::new("set_pet_size", "桌宠", &["size"], "PetStatus", Status::Implemented,
        "⛔ 越界拒绝不收敛（50–200），收敛会让用户误以为设上了"),
    ApiSpec::new("get_pet_overlay_supported", "桌宠", &[], "boolean", Status::Implemented,
        "macOS/Windows 恒 true；Linux 看 WAYLAND_DISPLAY/GDK_BACKEND（抄上游判定）"),
    ApiSpec::new("get_force_xwayland", "桌宠", &[], "boolean", Status::Implemented, "缺省 false"),
    ApiSpec::new("set_force_xwayland", "桌宠", &["enabled"], "boolean", Status::Implemented,
        "返回新值，调用方据此渲染开关"),
    ApiSpec::new("set_pet_ignore_cursor_events", "桌宠", &["ignore"], "boolean", Status::Implemented,
        "⛔ 仅 pet 窗可调；隐藏窗口的穿透请求吞掉报 false（上游 issue #437）"),
    ApiSpec::new("move_pet_window", "桌宠", &["delta_x", "delta_y"], "void", Status::Implemented,
        "⛔ 相对增量（与上游同形）。第一版是绝对定位，pet 窗自调 `{deltaX,deltaY}` 对不上而静默失败"),
    ApiSpec::new("list_pets", "桌宠", &["source"], "PetListItem[]", Status::Implemented,
        "扫 pets/<source>/；坏 manifest 的目录跳过；codex 首次列出播种内置海豚"),
    ApiSpec::new("import_pet", "桌宠", &["name", "data"], "PetListItem", Status::Implemented,
        "base64 zip；三道 cap + 链接拒绝 + 单 pet.json + 重名拒绝（走库侧同一条路）"),
    ApiSpec::new("get_pet_asset", "桌宠", &["id"], "PetAsset", Status::Implemented,
        "`source:id` 限定；内置海豚走静态单帧 1×1（偏离声明，待目检），其余走动画网格"),
    ApiSpec::new("list_preset_pets", "桌宠", &[], "PresetPetItem[]", Status::Implemented,
        "恒空：无远端源可对；本地走导入+种子。调用方对空走 PET_NOT_FOUND 可见诊断"),
    ApiSpec::new("push_pet_session", "桌宠", &["action", "session"], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("start_pet_mouse_stream", "桌宠", &[], "void", Status::Planned,
        "出口：rdev 光标线程 + 系统权限。无头环境不可验（做了也验不了），穿透开关已先接"),
    // ── 平台补齐（纯壳能力，无 DSH 依赖） ──
    ApiSpec::new("quit_app", "平台", &[], "void", Status::Implemented, "调用后进程即退出"),
    ApiSpec::new("reveal_data_dir", "平台", &[], "void", Status::Implemented,
        "目录不存在先建（全新安装），再揭示"),
    ApiSpec::new("reveal_in_folder", "平台", &["path"], "void", Status::Implemented,
        "⛔ 只揭示存在的路径；揭示不读内容"),
    ApiSpec::new("open_dir", "平台", &["path"], "void", Status::Implemented,
        "⛔ 必须是已存在的目录；文件/不存在的路径各平台行为不一"),
    ApiSpec::new("open_in_browser", "平台", &[], "void", Status::Stub,
        "自持模式无服务地址可打开；该按钮仅 DSH-runtime 分支可达"),
    ApiSpec::new("toggle_sidebar", "桌面", &[], "boolean", Status::Stub,
        "上游本体即 Ok(true) 空操作（布局状态在前端）；且无任何前端调用方（只有 iframe 内协议，无 iframe 即无调用）"),
    // ── 配置/桌面/日志/远端/更新 ──
    ApiSpec::new("get_app_config", "配置", &[], "AppConfig", Status::Implemented,
        "update 的读侧；缺文件回 {}，坏文件报错不回空对象"),
    ApiSpec::new("set_language", "其它", &["lang"], "void", Status::Implemented, "持久化界面语言"),
    ApiSpec::new("get_dsh_theme", "桌面", &[], "'dark' | 'light' | 'system'", Status::Implemented, ""),
    ApiSpec::new("get_cli_link_status", "其它", &[], "CliLinkStatus", Status::Refused,
        "真功能 = shim 生成 + PATH 注册（改 shell rc / 注册表，跨平台且有不可逆风险），
        且本仓无 bundle CLI 二进制可指。应用面板会调它 ⇒ 注册为显式拒绝（Refused）"),
    ApiSpec::new("is_dev_build", "桌面", &[], "boolean", Status::Implemented,
        "编译期事实，**不**读环境变量（打包后误判会让开发菜单出现在正式版）"),
    ApiSpec::new("log_frontend", "日志", &["level", "target", "message"], "void", Status::Implemented,
        "⚠️ 3 个参数。契约最初写成 2 个而门没抓到 —— 门当时只对命令名不对参数"),
    ApiSpec::new("read_run_logs", "日志", &[], "string", Status::Implemented,
        "⛔ 不返空串：前端直接塞 <pre>，空串会被读成「没有日志」"),
    ApiSpec::new("read_service_logs", "日志", &[], "string", Status::Stub, DSH_ONLY),
    ApiSpec::new("clear_service_logs", "日志", &[], "void", Status::Stub, DSH_ONLY),
    ApiSpec::new("get_desktop_about", "桌面", &[], "DesktopAboutInfo", Status::Implemented, ""),
    ApiSpec::new("remote_bridge_ping", "远端", &[], "string", Status::Planned,
        "出口：SSH 后端（远端起停 + 隧道）。ssh2 在 devDeps 只是引子，无后端的心跳是假心跳"),
    ApiSpec::new("check_desktop_update", "更新", &[], "DesktopUpdateInfo", Status::Planned,
        "出口：发布通道存在（releases.atom + 平台包命名）+ ureq/semver/atom 解析。
        未接前回 None 与抛错在界面同效果（静默无提示），且 desktop 从未发版，无物可对"),
    ApiSpec::new("download_desktop_update", "更新", &[], "DesktopUpdateInfo", Status::Planned,
        "与 check 成对；check 落地后跟进（进度事件 + AppData 落盘 + 退出时安装）"),
    ApiSpec::new("open_desktop_installer", "更新", &["path"], "void", Status::Planned,
        "同上；落地时路径守卫只开 updates 目录内（否则开任意文件）"),
];

/// 上游未逐条登记的命令。本轮已毕业：**为空**。
///
/// ⛔ 为空不是「删了」，是「95 个已全部逐条进 SPECS」。
/// 保留这个常量与 `neobot_api_call` 的 `unlisted` 分支：
/// 上游加命令时先落这里（平台保证不漏），逐条展开后再搬进 SPECS。
/// 若删掉它，下次上游加命令会直接变成「契约外调用」，门 ④ 只对
/// `neobot_` 报错，上游名会静默漏网。
pub const UPSTREAM_UNLISTED: &[&str] = &[];

/// 汇总视图：实现数 / 决定不做 / 待做 / 上游未逐条登记。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiSummary {
    pub total: usize,
    pub implemented: usize,
    pub stub: usize,
    pub planned: usize,
    pub upstream_unlisted: usize,
    /// 上游 `src-tauri` 的命令总数（`desktop/builder.rs` 的 `generate_handler!`）。
    /// ⚠️ 这个数字被 `summary` 自洽测试与 nt_check_api.mjs ⑤ 对账，写错门会 FAIL。
    /// 已注册、但实现是一句明确拒绝的命令数。
    pub refused: usize,
    /// 当前值 96（2026-09-30 上游 0.19.1 实数；上游加命令时同步加）。
    pub upstream_total: usize,
}

pub fn summary() -> ApiSummary {
    let count = |s: Status| SPECS.iter().filter(|x| x.status == s).count();
    ApiSummary {
        total: SPECS.len(),
        implemented: count(Status::Implemented),
        // ⛔ `Refused` 单列一档，不并入 `stub`：「未注册」与「已注册但拒绝」
        // 是两种不同的界面行为（前者 command not found，后者一句人话），
        // 合成一档就看不出这 11 条到底注册了没有。
        refused: count(Status::Refused),
        stub: count(Status::Stub),
        planned: count(Status::Planned),
        upstream_unlisted: UPSTREAM_UNLISTED.len(),
        upstream_total: 96,
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
            s.implemented + s.refused + s.stub + s.planned,
            s.total,
            "implemented+refused+stub+planned != total"
        );
    }

    /// ⛔ 上游口径自洽：非 `neobot_` 条目 + 未展开 = upstream_total。
    /// 这是「一个都不漏」的 Rust 侧半边（另半边是 nt_check_api.mjs ⑤）。
    /// 若上游加了命令而这里没跟，`upstream_total` 与分项之和先分叉 ——
    /// 分叉本身即信号，比静默漏登记好。
    #[test]
    fn 上游口径自洽() {
        let s = summary();
        let non_neobot = SPECS.iter().filter(|x| !x.name.starts_with("neobot_")).count();
        assert_eq!(
            non_neobot + s.upstream_unlisted,
            s.upstream_total,
            "非 neobot_ 条目({non_neobot}) + 未展开({}) != upstream_total({})",
            s.upstream_unlisted,
            s.upstream_total
        );
    }

    /// 未展开清单若非空，名字不得与 SPECS 重复 ——
    /// 重名会让 `neobot_api_call` 的「SPECS 优先、UNLISTED 兜底」含糊
    /// （同名永远走 SPECS，UNLISTED 那份不可达）。
    #[test]
    fn 未展开不与已登记重名() {
        for u in UPSTREAM_UNLISTED {
            assert!(
                !SPECS.iter().any(|s| s.name == *u),
                "未展开 {u} 与 SPECS 重名，UPSTREAM_UNLISTED 那份永远不可达"
            );
        }
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
