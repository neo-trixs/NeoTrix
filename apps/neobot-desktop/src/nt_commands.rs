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
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotRunResult {
    pub status: String,
    pub labels: neotrix_neobot::TurnLabels,
}

fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub mod nt_cmd_convo;
pub mod nt_cmd_core;
pub mod nt_cmd_run;
pub mod nt_cmd_sys;
pub mod nt_cmd_tasks;
