//! agent — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。
//!
//! F1 死码清理（2026-09-27）：`run_agent_mode` 全仓零调用已删（REPL 由 dialog 面承接）；
//! 本文件仅剩 TUI 转接桩。

use super::err;

/// NT-AGENT TUI 模式 — 基于 ratatui 的完整对话终端.
///
/// **STUB** — cli::tui 模块已移除：指引到对话面，不再是死胡同。
pub fn run_agent_tui(_profile: &str) {
    eprintln!(
        "{} TUI 模块已移除，请使用 `neotrix dialog say/agent/...` 对话面，或 --headless/web 模式",
        err("Error")
    );
}
