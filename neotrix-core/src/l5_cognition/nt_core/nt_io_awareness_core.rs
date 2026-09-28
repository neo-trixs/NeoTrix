//! # AwarenessCore — 意识核心命令面 (命令桥接已退役, 保留类型与契约)
//!
//! src/cli/commands 删除后: 命令字符串执行面不再可用
//! (`execute_command` 恒返 Err, `neotrix_command_tools()` 恒为空)。
//! 意图路由改走 `l6_meta::nt_auto_orchestrator::classify` + MCP NativeTools;
//! AgentLoop 的工具面由 `entry` 经 MCP 注册表装配, 人类交互走 Tauri。
//!
//! 本模块保留 `CommandNativeTool` 类型与"已移除"契约测试, 供历史调用方
//! 以类型兼容方式迁移; 不再提供执行能力。

use serde_json::Value;

use neotrix_types::traits::{NativeTool, ToolOutput};

/// 把命令行字符串转成可执行的完整命令 (补前导 `/`)。
/// 兼容 `/file read x` 与 `file read x` 两种写法。
fn normalize_command(command: &str) -> String {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{}", trimmed)
    }
}

/// 进程内执行 NeoTrix 命令, 返回文本输出。
fn execute_command(command: &str) -> Result<String, String> {
    let input = normalize_command(command);
    if input.is_empty() {
        return Err("Empty command".to_string());
    }
    // cli::commands removed — command execution unavailable.
    // Intent-based routing is handled by l6_meta::nt_auto_orchestrator.
    let _ = input;
    Err("CommandRegistry removed".to_string())
}

/// 单个命令 → NativeTool 适配。LLM 通过工具名 (`neotrix_<cmd>`) 调用。
pub struct CommandNativeTool {
    name: String,
    description: String,
}

impl CommandNativeTool {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}

impl NativeTool for CommandNativeTool {
    fn id(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "NeoTrix 命令文本 (可带或不带前导 '/'), 如 'file read src/main.rs' 或 '/memory search kb'"
                }
            },
            "required": ["command"]
        })
    }

    fn capability_tags(&self) -> Vec<&'static str> {
        vec!["neotrix_command"]
    }

    fn execute(&self, args: &Value) -> Result<ToolOutput, String> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing required field: command".to_string())?;
        let content = execute_command(command)?;
        Ok(ToolOutput {
            success: true,
            content,
        })
    }
}

/// 从 CommandRegistry 生成全部命令工具 (意识能力面)。
pub fn neotrix_command_tools() -> Vec<Box<dyn NativeTool>> {
    // cli::commands removed — no command tools available
    Vec::new()
}

/// 便捷入口: 供 entry (TUI/agent) 装配 AwarenessCore 工具面。
pub fn awareness_core_tools() -> Vec<Box<dyn NativeTool>> {
    neotrix_command_tools()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_command() {
        assert_eq!(normalize_command("/help"), "/help");
        assert_eq!(normalize_command("help"), "/help");
        assert_eq!(normalize_command("  file read x "), "/file read x");
        assert_eq!(normalize_command("   "), "");
    }

    #[test]
    fn test_execute_command_help_removed() {
        // 命令执行面已随 src/cli/commands 删除而移除
        let err = execute_command("/help").unwrap_err();
        assert!(err.contains("CommandRegistry removed"), "unexpected: {err}");
    }

    #[test]
    fn test_execute_command_no_slash_removed() {
        let err = execute_command("help").unwrap_err();
        assert!(err.contains("CommandRegistry removed"), "unexpected: {err}");
    }

    #[test]
    fn test_execute_command_memory_aggregator_removed() {
        let err = execute_command("/memory").unwrap_err();
        assert!(err.contains("CommandRegistry removed"), "unexpected: {err}");
    }

    #[test]
    fn test_execute_command_empty_err() {
        assert!(execute_command("   ").is_err());
    }

    #[test]
    fn test_command_tool_execute_removed() {
        // 工具类型保留 (类型兼容), 执行恒 Err
        // (ToolOutput 无 Debug, 不用 unwrap_err, 走 match 断言)
        let tool = CommandNativeTool::new("neotrix_cmd_help", "help");
        match tool.execute(&serde_json::json!({"command": "/help"})) {
            Ok(_) => assert!(false, "execution surface removed, must err"),
            Err(e) => assert!(e.contains("CommandRegistry removed"), "unexpected: {e}"),
        }
    }

    #[test]
    fn test_command_tool_missing_field() {
        let tool = CommandNativeTool::new("neotrix_cmd_help", "help");
        assert!(tool.execute(&serde_json::json!({})).is_err());
    }

    #[test]
    fn test_neotrix_command_tools_empty_by_design() {
        // 能力面改走 MCP NativeTools + nt_auto_orchestrator 意图分类,
        // 不再由命令字符串桥接: 两处入口恒为空。
        assert!(neotrix_command_tools().is_empty());
        assert!(awareness_core_tools().is_empty());
    }
}
