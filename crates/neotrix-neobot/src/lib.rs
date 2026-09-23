//! NeoBot — 本地独立 agent App 核心.
//!
//! 吸收来源 (2026-09-23, `/tmp neobot-absorb` 快照):
//! - OpenMuse (CopilotKit/OpenMuse, MIT): durable TaskWorker 租约状态机 +
//!   computer 沙箱 (`operationId` 幂等 + `workspacePath` 越狱拦截) +
//!   `WORKSPACE_MODE=sample` 零模型可跑.
//! - openbot (CopilotKit/openbot, MIT): fail-closed 网关
//!   (resolve → policy.evaluate → 先写 audit → 再执行, deny 优先/缺省拒绝) +
//!   append-only audit(含脱敏) + computer 工具契约 + Tauri desktop 引擎生命周期.
//! - cumora (yetone/cumora, MIT): `EngineAdapter` + daemon
//!   (debounce/coalesce/triage/steer) + 极简 tool schema
//!   (`bash` + `set_turn_status` + 3×FS) + CLI 即协议 + `llm_calls` 账本 + outbox.
//!
//! 本地独立保证: 默认零网络/零 Docker/零 Postgres/零云端 Intelligence.
//! 持久化只用 SQLite(`~/.neobot/neobot.db`), 模型经 `EngineAdapter` 可换
//! (默认 `Echo` 本地回显引擎, `--engine cli --command claude` 走本机 CLI).

#![forbid(unsafe_code)]

pub mod nt_agent;
pub mod nt_audit;
pub mod nt_cli;
pub mod nt_config;
pub mod nt_daemon;
pub mod nt_engine;
pub mod nt_error;
pub mod nt_policy;
pub mod nt_store;
pub mod nt_types;

pub use nt_agent::run_local_turn;
pub use nt_audit::{AuditDecision, AuditEvent, redact_detail};
pub use nt_cli::{format_side_effect, parse_side_effect};
pub use nt_config::{EngineKind, NeobotConfig, PolicyMode};
pub use nt_daemon::{DaemonGate, SteerMsg};
pub use nt_engine::{CliEngine, EngineAdapter, EngineTurn, LocalEchoEngine};
pub use nt_error::NtBotError;
pub use nt_policy::{Actor, PolicyContext, PolicyDecision, evaluate_policy};
pub use nt_store::NeobotStore;
pub use nt_types::{AgentTask, TaskStatus, ToolCall, ToolName, ToolResult, TurnStatus};
