//! # NeoTrix CLI — 终端 UI 层
//!
//! 基于 ratatui 的交互式终端界面 + 命令系统。
//! 依赖 `agent` + `core` 层。
//!
//! ## 子模块
//!
//! - `tui` — ratatui 交互式终端
//! - `commands` — 命令注册和执行

pub mod tui;
pub mod commands;
pub mod approval;
pub mod permission_profiles;
pub mod cost_tracker;
pub mod sandbox;
pub mod jsonl_stream;
pub use tui::TuiApp;
pub use commands::CommandRegistry;
pub use approval::{ApprovalMode, ApprovalEngine, ActionType, PendingAction, global_approval};
pub use permission_profiles::{
    PermissionProfile, ProfileDecision, ProfileStore, global_profile_manager,
    active_profile_name, switch_profile, list_profiles, is_action_denied, is_action_allowed,
    action_type_to_key,
};
pub use cost_tracker::CostTracker;
pub use sandbox::{SandboxMode, SandboxEnforcer, global_sandbox, check_sandbox, init_sandbox};
pub mod cli_interface;
