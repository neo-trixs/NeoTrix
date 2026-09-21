//! CLI Plugin — CLI 命令执行
//!
//! 通过异步子进程执行命令（`tokio::process`，不阻塞运行时）。

use super::common::stub_action;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;

pub struct CliPlugin;

impl CliPlugin {
    async fn run_command(args: &[String]) -> Result<serde_json::Value, DomainError> {
        if args.is_empty() {
            return Err(DomainError {
                code: "INVALID_ARGS".into(),
                message: "缺少 command 参数".into(),
                recoverable: true,
            });
        }
        let output = tokio::process::Command::new(&args[0])
            .args(&args[1..])
            .output()
            .await
            .map_err(|e| DomainError {
                code: "CLI_ERROR".into(),
                message: format!("执行失败: {e}"),
                recoverable: true,
            })?;
        Ok(serde_json::json!({
            "success": output.status.success(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        }))
    }
}

#[async_trait]
impl DomainPlugin for CliPlugin {
    fn name(&self) -> &str {
        "cli"
    }
    fn description(&self) -> &str {
        "CLI 命令执行"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["exec", "list", "run", "history", "clear"]
            .iter()
            .map(|a| stub_action(a))
            .collect()
    }
    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "exec" | "run" => {
                let command = args
                    .get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 command 参数".into(),
                        recoverable: true,
                    })?;
                let cmd_args: Vec<String> = args
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                let mut full_args = vec![command.to_string()];
                full_args.extend(cmd_args);
                Self::run_command(&full_args).await
            }
            "list" => {
                // Return available CLI commands
                Ok(serde_json::json!({
                    "commands": [
                        {"name": "neotrix", "description": "NeoTrix CLI"},
                        {"name": "help", "description": "显示帮助"},
                        {"name": "version", "description": "显示版本"},
                    ]
                }))
            }
            "history" => {
                // TODO: Implement CLI command history tracking
                Ok(serde_json::json!({ "history": [], "count": 0 }))
            }
            "clear" => {
                // TODO: Implement CLI history clear
                Ok(serde_json::json!({ "ok": true }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
