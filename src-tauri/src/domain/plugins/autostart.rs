//! Autostart domain plugin — 开机自启管理
//!
//! Wraps the same `tauri_plugin_autostart` APIs as `AutoStartManager`,
//! but reachable via `domain_call` (the old `autostart_*` Tauri commands
//! were removed in the chat-first migration). Uses the globally stored
//! `AppHandle` (`domain::app_handle`), set during Tauri setup.

use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;
use tauri_plugin_autostart::ManagerExt;

pub struct AutostartPlugin;

impl AutostartPlugin {
    pub fn new() -> Self {
        Self
    }

    fn with_handle<T>(
        f: impl FnOnce(&tauri::AppHandle) -> Result<T, DomainError>,
    ) -> Result<T, DomainError> {
        let app = crate::domain::app_handle::get_app_handle().ok_or_else(|| DomainError {
            code: "NOT_AVAILABLE".into(),
            message: "AppHandle not yet initialized".into(),
            recoverable: true,
        })?;
        f(app)
    }
}

impl Default for AutostartPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn err(code: &str, message: String) -> DomainError {
    DomainError {
        code: code.into(),
        message,
        recoverable: true,
    }
}

#[async_trait]
impl DomainPlugin for AutostartPlugin {
    fn name(&self) -> &str {
        "autostart"
    }

    fn description(&self) -> &str {
        "开机自启：查询/启用/禁用/切换"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "is_enabled".into(),
                description: "查询开机自启是否启用".into(),
                params: vec![],
                returns: "bool".into(),
            },
            ActionSpec {
                name: "enable".into(),
                description: "启用开机自启".into(),
                params: vec![],
                returns: "void".into(),
            },
            ActionSpec {
                name: "disable".into(),
                description: "禁用开机自启".into(),
                params: vec![],
                returns: "void".into(),
            },
            ActionSpec {
                name: "toggle".into(),
                description: "切换开机自启并返回新状态".into(),
                params: vec![],
                returns: "bool".into(),
            },
        ]
    }

    async fn call(
        &self,
        action: &str,
        _args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "is_enabled" => {
                let enabled = Self::with_handle(|app| {
                    app.autolaunch()
                        .is_enabled()
                        .map_err(|e| err("AUTOSTART_QUERY_FAILED", e.to_string()))
                })?;
                Ok(serde_json::json!({ "enabled": enabled }))
            }
            "enable" => {
                Self::with_handle(|app| {
                    app.autolaunch()
                        .enable()
                        .map_err(|e| err("AUTOSTART_ENABLE_FAILED", e.to_string()))
                })?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "disable" => {
                Self::with_handle(|app| {
                    app.autolaunch()
                        .disable()
                        .map_err(|e| err("AUTOSTART_DISABLE_FAILED", e.to_string()))
                })?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "toggle" => {
                let enabled = Self::with_handle(|app| {
                    let launcher = app.autolaunch();
                    let enabled = launcher
                        .is_enabled()
                        .map_err(|e| err("AUTOSTART_QUERY_FAILED", e.to_string()))?;
                    if enabled {
                        launcher
                            .disable()
                            .map_err(|e| err("AUTOSTART_DISABLE_FAILED", e.to_string()))?;
                    } else {
                        launcher
                            .enable()
                            .map_err(|e| err("AUTOSTART_ENABLE_FAILED", e.to_string()))?;
                    }
                    Ok(enabled)
                })?;
                Ok(serde_json::json!({ "enabled": !enabled }))
            }
            _ => Err(err(
                "UNKNOWN_ACTION",
                format!("Unknown autostart action: {action}"),
            )),
        }
    }
}
