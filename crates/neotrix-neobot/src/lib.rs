//! NeoBot — 本地独立 agent App 核心.
//!
//! 设计要点：durable 任务租约状态机（lease 认领、心跳续租、失租回收、
//! attempts），fail-closed 网关（先写 audit 再执行，deny 优先、缺省拒绝、
//! 坏规则拒绝），append-only audit（含脱敏与留存清扫），routines（firing 帧、
//! 15min 地板、10 连败自停），computer 工具契约，take-the-wheel 交接审计，
//! 可换引擎（debounce/triage/steer），极简 tool schema（`bash`、
//! `set_turn_status`、3×FS），CLI 即协议（JSONL 副作用回传），调用账本
//! （purpose、measured、status），outbox（attempts、available_at 退避），
//! 引擎失败分类，认领租约。
//!
//! 本地独立保证：默认零网络/零 Docker/零 Postgres/零云端。
//! 持久化只用 SQLite（`~/.neobot/neobot.db`），模型经引擎适配可换
//! （默认 `Echo` 本地回显引擎，`--engine cli` 走本机 CLI）。
//!
//! 单机取舍（刻意）：无 Redis（seen/presence 纯内存）、
//! 无 Postgres（SKIP LOCKED 用 SQLite 原子 UPDATE 代）、无 cron
//! （routines 用 interval）、无 CEL（extra_deny 小 matcher）。

#![forbid(unsafe_code)]

pub mod nt_agent;
pub mod nt_audit;
pub mod nt_cancel;
pub mod nt_changes;
pub mod nt_channel;
pub mod nt_channel_cmd;
pub mod nt_channel_dispatch;
pub mod nt_channel_serve;
pub mod nt_channel_telegram;
pub mod nt_cli;
pub mod nt_llama;
pub mod nt_computer;
pub mod nt_config;
pub mod nt_core;
pub mod nt_cost;
pub mod nt_daemon;
pub mod nt_engine;
pub mod nt_error;
pub mod nt_export;
pub mod nt_git;
pub mod nt_http_engine;
pub mod nt_memory;
pub mod nt_policy;
pub mod nt_provider;
pub mod nt_qwen_mm;
pub mod nt_reply_tag;
pub mod nt_routine;
pub mod nt_side_chat;
pub mod nt_sidebar;
pub mod nt_skills;
pub mod nt_stale_guard;
pub mod nt_store;
pub mod nt_token_guard;
pub mod nt_types;
pub mod nt_vision;
pub mod nt_web;
pub mod nt_workspace;
/// 仅测试目标编译：测试期临时目录唯一化工具（见模块头「为什么需要它」）。
#[cfg(test)]
mod nt_testutil;

/// 对话入口（合二为一：neobot 即 neotrix 对话面；`neobot` 二进制与
/// `neotrix dialog` 同律调用，行为一字不差）。
/// 配置（`NEOBOT_DATA_DIR` 等环境覆盖在内）。
pub fn load_config() -> Result<NeobotConfig, NtBotError> {
    NeobotConfig::from_env()
}

/// 打开对话库（幂等建表；路径来自配置）。
pub fn open_store(cfg: &NeobotConfig) -> Result<NeobotStore, NtBotError> {
    let path = cfg.db_path();
    NeobotStore::open(&path.to_string_lossy())
}

pub use nt_agent::run_local_turn;
pub use nt_agent::run_local_turn_as;
pub use nt_agent::run_local_turn_stream;
pub use nt_agent::run_local_turn_stream_as;
pub use nt_audit::{AuditDecision, AuditEvent, redact_detail};
pub use nt_changes::{ChangeSink, KIND_EDIT, KIND_READ, KIND_WRITE, prune_best_effort};
pub use nt_channel::{AccessMode, ChannelAdapter, ChannelHealth, ChannelRegistry, InboundMessage, OutboundMessage};
pub use nt_channel_telegram::TelegramChannel;
pub use nt_cli::{format_side_effect, parse_side_effect, parse_side_effects_jsonl};
pub use nt_computer::{ComputerAction, ComputerBackend, ComputerCall, NoopBackend};
pub use nt_config::{EngineKind, NeobotConfig, PolicyMode};
pub use nt_core::{CORE_MODEL, CoreStatus, FreeEntry, FreeVia, TraceRow, AgentRunResult, CapabilitiesInfo, DEFAULT_TOKEN_ENV, AGENT_DEFAULT_STEPS, AGENT_MAX_STEPS, agent_run, agent_run_with_steps, bearer_for, capabilities_or_default, core_engine, core_engine_with_model, core_reload, core_status, discover_free, fetch_capabilities, fetch_capabilities_raw, normalize_token_env, pair_core, pair_free, parse_agent_run_result, parse_capabilities, summarize_reload, unpair_core};
pub use nt_cost::{CostPolicy, cost_for, price_for};
pub use nt_reply_tag::{ReplyMode, TurnLabels, TurnUsage, labels_for_turn, normalize_tools, tools_from_trace, usage_for_turn};
pub use nt_daemon::{DaemonGate, SteerMsg};
pub use nt_engine::{CliEngine, EngineAdapter, EngineTurn, LocalEchoEngine, OpencodeEngine};
pub use nt_git::{GitCommit, GitFile};
pub use nt_http_engine::{HttpEngine, HttpEngineConfig};
pub use nt_error::{EngineFailureKind, NtBotError, classify_engine_failure};
pub use nt_export::{ExportReport, export_bundle};
pub use nt_policy::{Actor, PolicyContext, PolicyDecision, evaluate_policy};
pub use nt_provider::{PoolModel, Provider, PRESETS, NEOTRIX_CORE_MODEL, pool_models};
pub use nt_routine::{Routine, fire_routine, frame_firing, read_firing, sweep_routines};
pub use nt_side_chat::{first_prompt, inherit_context};
pub use nt_sidebar::{OpenTarget, SidebarTab, TabRegistry, ViewerSpec, builtin_tabs, builtin_viewers, resolve_open, viewer_for};
pub use nt_store::{Attachment, BotRow, ChannelRow, Conversation, CorePair, FileChange, FileChangeView, NeobotStore, PathTally, PendingDelivery, CLAIM_TTL_SECS, LedgerActorSum, LedgerEntry, LedgerSum, classify_attachment};
pub use nt_types::{
    AgentTask, TaskStatus, TokenUsage, ToolCall, ToolName, ToolResult, TranscriptItem,
    TranscriptRole, TurnStatus,
};
pub use nt_workspace::{
    DirEntry, DirListing, FileText, FileWrite, SearchCap, SearchHit, SearchResults, jail_join,
    list_dir, read_text, rel_of, write_text,
};
