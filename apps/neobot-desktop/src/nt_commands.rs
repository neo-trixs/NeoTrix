//! `nt_commands` — NeoBot 独立壳的唯一 IPC 面.
//!
//! 薄封装 `neotrix-neobot` (SQLite + 网关 + 引擎), 不含业务逻辑.
//! 复制 `src-tauri/src/commands/neobot.rs` 范式并收敛到独立进程:
//! 读操作走同步命令, `run` (可长达分钟级) 经 `spawn_blocking`
//! 避免占住 Tauri 异步运行时. 错误统一为 `String` 给前端.
//! DTO 在此脱敏 (核心 `AuditEvent.detail` 永不外发).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use neotrix_neobot::{CliEngine, EngineKind, HttpEngine, NeobotConfig, NeobotStore, OpencodeEngine};

/// 前端任务 DTO (含认领 + 可见性 + 会话归属, Team U3.5).
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotTaskItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub claimed_by: Option<String>,
    pub visibility: String,
    pub conversation_id: Option<String>,
    pub attempts: i64,
}

/// 前端模型 DTO（含来源端点）.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotModelItem {
    pub id: String,
    pub owner: String,
    pub source: String,
}

/// 前端端点 DTO（key 永不出前端，只出变量名）.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotProviderItem {
    pub name: String,
    pub base_url: String,
    pub key_env: String,
    pub model: String,
    pub enabled: bool,
}

/// 前端审计 DTO (明细已由核心脱敏, 此处只出四列).
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotAuditItem {
    pub at: String,
    pub actor: String,
    pub tool: String,
    pub decision: String,
    pub rule: Option<String>,
}

/// 成本账聚合行 (按 engine+model).
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotCostRow {
    pub engine: String,
    pub model: String,
    pub in_tokens: i64,
    pub out_tokens: i64,
    pub cost_usd: f64,
}

/// 成本账行 (按 engine+model+actor).
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotCostActorRow {
    pub engine: String,
    pub model: String,
    pub actor: String,
    pub in_tokens: i64,
    pub out_tokens: i64,
    pub cost_usd: f64,
}

/// 例行 DTO.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotRoutineItem {
    pub name: String,
    pub interval_secs: i64,
    pub owner: String,
    pub failures: i64,
    pub disabled: i64,
    pub next_run_at: i64,
    pub instruction: String,
}

/// 技能 DTO（描述行；全文不进前端，省上下文）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotSkillItem {
    pub name: String,
    pub description: String,
}

/// 成员 DTO.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotMemberItem {
    pub id: String,
    pub kind: String,
    pub owner: bool,
}

/// presence 行 (派生状态, 非落盘).
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotPresenceItem {
    pub id: String,
    pub kind: String,
    pub presence: String,
    pub last_seen_secs: u64,
}

fn load_config() -> Result<NeobotConfig, String> {
    NeobotConfig::from_env().map_err(|err| err.to_string())
}

fn open_store(config: &NeobotConfig) -> Result<NeobotStore, String> {
    let path = config.db_path().to_string_lossy().into_owned();
    NeobotStore::open(&path).map_err(|err| err.to_string())
}

enum EngineSelection {
    Echo,
    Cli(CliEngine),
    Opencode(OpencodeEngine),
    Http(HttpEngine),
}

fn resolve_engine(config: &NeobotConfig) -> Result<EngineSelection, String> {
    match &config.engine {
        EngineKind::Echo => Ok(EngineSelection::Echo),
        EngineKind::Cli { command } => CliEngine::new(command)
            .map(EngineSelection::Cli)
            .map_err(|err| err.to_string()),
        EngineKind::Opencode { model } => OpencodeEngine::new(model)
            .map(EngineSelection::Opencode)
            .map_err(|err| err.to_string()),
        EngineKind::Http { .. } => HttpEngine::from_env()
            .map(EngineSelection::Http)
            .map_err(|err| err.to_string()),
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// presence 表 (常驻内存; 单机先行, 多租户远期).
pub struct PresenceMap(pub Mutex<HashMap<String, (String, u64)>>);

/// daemon 门控 (常驻内存; routine firing 去抖+triage, 防手动 fire 与 sweep 重叠连击).
/// DaemonGate 非 Sync 不可跨线程共享——包 Arc<Mutex> 走 Tauri State（仿 PresenceMap）。
/// commands 里先 clone 出 Arc 再进 spawn_blocking（State 本身借 app handle，非 'static）。
pub struct DaemonMap(pub std::sync::Arc<Mutex<neotrix_neobot::DaemonGate>>);

impl Default for DaemonMap {
    fn default() -> Self {
        Self(std::sync::Arc::new(Mutex::new(neotrix_neobot::DaemonGate::default())))
    }
}

impl Default for PresenceMap {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

/// 免费条目 DTO（key 是否就位一并展示；免费≠免 key，CLI 类除外）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotFreeItem {
    pub provider: String,
    pub model_id: String,
    pub display: String,
    pub key_env: String,
    pub key_ok: bool,
    pub via: String,
}

/// 晶体核心状态（设置·引擎行展示：灵魂在线/离线/未嵌入 + 版本摘要）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotCoreStatus {
    pub paired: bool,
    pub online: bool,
    pub base_url: String,
    pub model: String,
    pub models: usize,
    pub latency_ms: i64,
    pub detail: String,
    pub crystal_version: String,
    pub tool_count: usize,
}

/// 前端会话 DTO.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotConvoItem {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub members: Vec<String>,
    pub task_count: i64,
    pub last_active: String,
    pub muted: bool,
    pub unread: i64,
}

/// 前端附件 DTO（落盘绝对路径；展示层经 convertFileSrc）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotAttachItem {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub path: String,
    pub size: i64,
}

/// 跑轮结果（含回复标签；前端气泡顶部 chips 直消）。
/// `status` 沿旧口径（`TurnStatus::as_str`）；`labels` 含 model/mode/tools/usage。
///
/// ## `task_id`（2026-09-28 切片 C3）
///
/// 前端此前**拿不到**本轮任务自己的 id，只能用「窗口内新建 + 归属本会话」
/// 作间接证据（`frontend/src/turn_task.ts`）。现在后端把**本轮那一个** id
/// 直接给出，`turn_task.ts` 优先用它、缺失时才回落到间接判据。
///
/// **空串 = 没有任务**（桌面斜杠指令那条路不进跑轮，一个任务都不建）。
/// 刻意**不**填 `list_tasks(1).first()`：那是「库里最新的」，与「本轮的」
/// 没有必然关系（队内并行、后台例程、别的窗口都会让它指到别人的任务）——
/// 那正是本字段要取代的东西。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotRunResult {
    pub status: String,
    pub labels: neotrix_neobot::TurnLabels,
    /// 本轮自己的任务 id；**没有任务时是空串**（不是猜、不是编）。
    pub task_id: String,
    /// 本轮**真的**以 `TaskStatus::Cancelled` 结束（`false` = 不是被用户叫停的）。
    ///
    /// 口径是**回库核对**（`get_task(task_id).status`），不是「令牌被翻过」——
    /// 停止请求送达 ≠ 这一轮真被停掉（它可能已经跑完了）。前端拿它当
    /// 「停掉了」的**唯一**依据，不用猜。
    pub cancelled: bool,
}

fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub mod nt_cmd_channels;
pub mod nt_cmd_convo;
pub mod nt_cmd_core;
pub mod nt_cmd_files;
pub mod nt_cmd_run;
pub mod nt_cmd_sidebar;
pub mod nt_cmd_sys;
pub mod nt_cmd_tasks;

// ─── 命令注册完整性 ───
//
// 为什么需要它：**漏注册一个 `#[tauri::command]` 编译得过、clippy 干净、
// `cargo test` 全绿**，只是那个命令从前端永远调不到 —— 运行时才发现「没这条线」。
//
// 两个方向都要查（第一版只写了反向，变异测试立刻证明它抓不到正向）：
// - **正向** 声明 ⊆ 注册：写好了却没挂上 → 上面那个静默缺陷；
// - **反向** 注册 ⊆ 声明：表里拼错了模块/函数名（那种编译器会抓一半，
//   但同名不同模块的情况抓不到）。
//
// 读源码而不是反射：本 crate 只有 bin 目标、没有 lib 可 introspect ——
// 为了测试而加 lib 目标，等于把 `main.rs` 的注册表搬一次家，不值得。
#[cfg(test)]
mod registration_tests {
    /// 从源码文本里抓出全部 `#[tauri::command]` 声明的函数名。
    ///
    /// 逐行扫：遇到 `#[tauri::command]` 就记住，下一个非空、非注释且以
    /// `pub fn ` / `fn ` 开头的行即函数名（跳过 `pub async fn`）。
    fn declared_commands(source: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut armed = false;
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed == "#[tauri::command]" {
                armed = true;
                continue;
            }
            if !armed {
                continue;
            }
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            // 下一个函数定义；若是别的属性则继续等。
            let candidate = trimmed
                .strip_prefix("pub async fn ")
                .or_else(|| trimmed.strip_prefix("pub fn "))
                .or_else(|| trimmed.strip_prefix("async fn "))
                .or_else(|| trimmed.strip_prefix("fn "));
            match candidate {
                Some(rest) => {
                    if let Some(name) = rest.split('(').next() {
                        out.push(name.to_owned());
                    }
                    armed = false;
                }
                None => {
                    if trimmed.starts_with('#') || trimmed.starts_with("///") {
                        continue;
                    }
                    armed = false;
                }
            }
        }
        out
    }

    /// `generate_handler!` 里注册的命令名。
    fn registered_commands(main: &str) -> Vec<String> {
        main.lines()
            .filter_map(|line| line.trim().strip_prefix("nt_commands::"))
            .filter_map(|rest| rest.strip_suffix(','))
            .map(|entry| entry.rsplit("::").next().unwrap_or("").to_owned())
            .collect()
    }

    const MAIN: &str = include_str!("main.rs");
    const CONVO: &str = include_str!("nt_commands/nt_cmd_convo.rs");
    const CORE: &str = include_str!("nt_commands/nt_cmd_core.rs");
    const FILES: &str = include_str!("nt_commands/nt_cmd_files.rs");
    const RUN: &str = include_str!("nt_commands/nt_cmd_run.rs");
    const SIDEBAR: &str = include_str!("nt_commands/nt_cmd_sidebar.rs");
    const SYS: &str = include_str!("nt_commands/nt_cmd_sys.rs");
    const TASKS: &str = include_str!("nt_commands/nt_cmd_tasks.rs");
    const CHANNELS: &str = include_str!("nt_commands/nt_cmd_channels.rs");

    fn all_command_sources() -> [(&'static str, &'static str); 8] {
        [
            ("nt_cmd_convo", CONVO),
            ("nt_cmd_core", CORE),
            ("nt_cmd_files", FILES),
            ("nt_cmd_run", RUN),
            ("nt_cmd_sidebar", SIDEBAR),
            ("nt_cmd_sys", SYS),
            ("nt_cmd_tasks", TASKS),
            ("nt_cmd_channels", CHANNELS),
        ]
    }

    /// 解析器自检：这两条抓不到东西的话，后面的断言全是空的。
    #[test]
    fn the_parser_actually_finds_commands() {
        let declared: Vec<String> = all_command_sources()
            .iter()
            .flat_map(|(_, src)| declared_commands(src))
            .collect();
        assert!(
            declared.len() >= 90,
            "只认出 {} 个命令 —— 解析规则失效，下面的断言会变成空转",
            declared.len()
        );
        let registered = registered_commands(MAIN);
        assert!(registered.len() >= 90, "只认出 {} 个注册项", registered.len());
    }

    /// **正向**：每个声明都必须出现在注册表里。
    #[test]
    fn every_declared_command_is_registered() {
        let registered: std::collections::BTreeSet<String> =
            registered_commands(MAIN).into_iter().collect();
        let mut missing = Vec::new();
        for (module, src) in all_command_sources() {
            for name in declared_commands(src) {
                if !registered.contains(&name) {
                    missing.push(format!("{module}::{name}"));
                }
            }
        }
        assert!(
            missing.is_empty(),
            "这些命令声明了却没进 generate_handler!，从前端永远调不到：{missing:?}"
        );
    }

    /// **反向**：注册表里不能有不存在的命令。
    #[test]
    fn every_registered_command_exists() {
        let declared: std::collections::BTreeSet<String> = all_command_sources()
            .iter()
            .flat_map(|(_, src)| declared_commands(src))
            .collect();
        let mut unknown = Vec::new();
        for name in registered_commands(MAIN) {
            if !declared.contains(&name) {
                unknown.push(name);
            }
        }
        assert!(unknown.is_empty(), "注册表里有不存在的命令：{unknown:?}");
    }

    /// 注册表自身不许重复。
    #[test]
    fn registrations_are_unique() {
        let all = registered_commands(MAIN);
        let mut sorted = all.clone();
        sorted.sort();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(before, sorted.len(), "generate_handler! 里有重复注册");
    }
}
