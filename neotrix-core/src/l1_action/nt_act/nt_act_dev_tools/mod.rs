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
pub mod external_cli_plugins;

pub use build_desktop::DesktopBuilder;
pub use daemon_monitor::DaemonMonitor;
pub use git_hook::GitHook;
pub use interactive_cli::InteractiveAgentCli;
pub use external_cli_plugins::{ExternalCliPlugin, find_external_cli_plugin, load_external_cli_plugins, plugins_dir, CliDescriptorPlugin, external_cli_as_plugins, load_external_cli_into};
