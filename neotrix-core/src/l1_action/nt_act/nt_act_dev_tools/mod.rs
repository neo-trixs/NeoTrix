//! Development Tools — 开发工具集
//!
//! 从 scripts/ 移植到 Rust 的开发工具:
//! - build-desktop.sh → DesktopBuilder
//! - daemon-monitor.sh → DaemonMonitor
//! - git-hook.sh → GitHook

pub mod build_desktop;
pub mod daemon_monitor;
pub mod git_hook;
pub mod interactive_cli;

pub use build_desktop::DesktopBuilder;
pub use daemon_monitor::DaemonMonitor;
pub use git_hook::GitHook;
pub use interactive_cli::{freebuff_cli, InteractiveAgentCli};
