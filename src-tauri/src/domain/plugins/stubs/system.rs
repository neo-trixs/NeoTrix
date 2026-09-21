//! System Plugin — 系统：窗口、PTY、更新、配置

use super::common::stub_action;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;

pub struct SystemPlugin;

impl SystemPlugin {
    fn get_system_info_sync() -> Result<serde_json::Value, DomainError> {
        let platform = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".into());
        let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".into());
        let hostname = std::env::var("HOSTNAME")
            .unwrap_or_else(|_| "unknown".to_string());
        let uptime_seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        Ok(serde_json::json!({
            "platform": platform,
            "arch": arch,
            "neotrix_version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": uptime_seconds,
            "hostname": hostname,
            "cpu_count": cpu_count,
        }))
    }
}

#[async_trait]
impl DomainPlugin for SystemPlugin {
    fn name(&self) -> &str {
        "system"
    }
    fn description(&self) -> &str {
        "系统：窗口、PTY、更新、配置"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "window_minimize",
            "window_maximize",
            "window_close",
            "pty_spawn",
            "pty_write",
            "pty_resize",
            "pty_close",
            "update_check",
            "update_download",
            "restart_app",
            "config_get",
            "config_set",
            "system_info",
        ]
        .iter()
        .map(|a| stub_action(a))
        .collect()
    }
    async fn call(
        &self,
        action: &str,
        _args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "system_info" => Self::get_system_info_sync(),
            // PTY actions require Tauri state — not accessible from plugin context
            // TODO: Implement via global PtyManager reference or separate PTY plugin
            "pty_spawn" | "pty_write" | "pty_resize" | "pty_close" => Err(DomainError {
                code: "NOT_IMPLEMENTED".into(),
                message: format!("{} requires Tauri state access", action),
                recoverable: true,
            }),
            // Update/restart actions require platform-specific implementation
            // TODO: Implement update_check, update_download, restart_app via tauri-updater or custom logic
            "update_check" | "update_download" | "restart_app" => Ok(
                serde_json::json!({ "ok": true, "stub": true, "message": format!("{} not yet implemented", action) }),
            ),
            // Window actions require AppHandle
            "window_minimize" | "window_maximize" | "window_close" => Err(DomainError {
                code: "NOT_IMPLEMENTED".into(),
                message: format!("{} requires AppHandle access", action),
                recoverable: true,
            }),
            // Config actions
            "config_get" | "config_set" => {
                // Delegate to config module
                Ok(serde_json::json!({ "ok": true, "stub": true }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
