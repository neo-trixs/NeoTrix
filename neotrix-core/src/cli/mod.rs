//! # NeoTrix CLI — 终端 UI 层
//!
//! 基于 ratatui 的交互式终端界面 + 命令系统。
//! 依赖 `agent` + `core` 层。
//!
//! ## 子模块
//!
//! - `tui` — ratatui 交互式终端
//! - `commands` — 命令注册和执行

pub mod approval;
pub mod commands;
pub mod cost_tracker;
pub mod jsonl_stream;
pub mod permission_profiles;
pub mod sandbox;
pub mod tui;
pub use approval::{global_approval, ActionType, ApprovalEngine, ApprovalMode, PendingAction};
pub use commands::CommandRegistry;
pub use cost_tracker::CostTracker;
pub use permission_profiles::{
    action_type_to_key, active_profile_name, global_profile_manager, is_action_allowed,
    is_action_denied, list_profiles, switch_profile, PermissionProfile, ProfileDecision,
    ProfileStore,
};
pub use sandbox::{check_sandbox, global_sandbox, init_sandbox, CliSandboxMode, SandboxEnforcer};
pub use tui::TuiApp;
pub mod laws;
pub mod sandboxed_shell;
pub mod shield_enforcer;
pub use laws::{LawSeverity, LawViolation, ProjectLaws};
pub use sandboxed_shell::{check_file_operation, execute_guarded, is_shell_allowed};
pub use shield_enforcer::{global_shield, init_shield, ShieldDecision, ShieldEnforcer};
pub mod cli_interface;
pub mod progress;
