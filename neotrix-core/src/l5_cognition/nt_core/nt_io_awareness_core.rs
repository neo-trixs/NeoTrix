//! # AwarenessCore — 意识核心命令面 (命令 → 意识能力桥接)
//!
//! 对外只暴露一个意识核心：人类交互 (TUI/CLI/headless) 只接触基础控制命令,
//! 领域操作全部由意识核心 (AgentLoop + AttentionRouter) 智能调度。
//!
//! 本模块把 CommandRegistry 的全部命令 (file/git/session/agent/memory/
//! crypto/kb/wiki/...) 桥接为 NativeTool, 注入 AgentLoop 的能力面。
//! LLM 意识核心通过 AttentionRouter 判断意图 → 调用对应命令工具 → 回传结果。
//!
//! 进程内执行 (CommandRegistry::execute), 无需 spawn 子进程, 复用 registry 的
//! sandbox/shield/hook/approval 治理。命令仍可被人类直接输入, 但不占一级
//! 认知面 (is_primary=false)。

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
    let reg = crate::cli::commands::registry::default_registry();
    let out = reg.execute(&input, None);

    let mut result = String::new();
    if out.success {
        result.push_str(&out.message);
    } else {
        result.push_str(&format!("Error: {}", out.message));
    }
    if let Some(json) = &out.json {
        if let Ok(s) = serde_json::to_string(json) {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&s);
        }
    }
    Ok(result)
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
///
/// 每个已注册命令生成一个 `neotrix_<cmd名去slash>` 工具, 描述引用命令原文,
/// 让 LLM 意识核心能智能调度任意能力。附带 agent_all 兜底工具。
pub fn neotrix_command_tools() -> Vec<Box<dyn NativeTool>> {
    use crate::cli::commands::registry::default_registry;
    let reg = default_registry();
    let mut tools: Vec<Box<dyn NativeTool>> = Vec::new();

    // 兜底: 通过单个工具执行任意命令 (LLM 一次只能看到有限工具时启用)。
    tools.push(Box::new(CommandNativeTool::new(
        "neotrix_command",
        "Execute any NeoTrix command in-process (agent 后端自我调度通道). \
         command 为完整命令文本, 如 'file read src/main.rs' 或 '/memory search kb'.",
    )));

    for name in reg.list() {
        if name.is_empty() {
            continue;
        }
        let tool_id = format!("neotrix_cmd_{}", name.trim_start_matches('/').replace('/', "_"));
        let desc = if let Some(cmd) = reg.get(name) {
            format!("{} — {}", name, cmd.description())
        } else {
            format!("Execute NeoTrix command {}", name)
        };
        tools.push(Box::new(CommandNativeTool::new(&tool_id, &desc)));
    }
    tools
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
    fn test_execute_command_help() {
        let out = execute_command("/help").unwrap();
        assert!(!out.is_empty());
    }

    #[test]
    fn test_execute_command_no_slash() {
        let out = execute_command("help").unwrap();
        assert!(out.contains("help") || out.contains("命令"));
    }

    #[test]
    fn test_execute_command_memory_aggregator() {
        let out = execute_command("/memory").unwrap();
        assert!(out.contains("evidence"), "aggregator 应可执行: {}", out);
    }

    #[test]
    fn test_execute_command_empty_err() {
        assert!(execute_command("   ").is_err());
    }

    #[test]
    fn test_command_tool_execute() {
        let tool = CommandNativeTool::new("neotrix_cmd_help", "help");
        let out = tool.execute(&serde_json::json!({"command": "/help"})).unwrap();
        assert!(out.success);
        assert!(!out.content.is_empty());
    }

    #[test]
    fn test_command_tool_missing_field() {
        let tool = CommandNativeTool::new("neotrix_cmd_help", "help");
        assert!(tool.execute(&serde_json::json!({})).is_err());
    }

    #[test]
    fn test_neotrix_command_tools_nonempty() {
        let tools = neotrix_command_tools();
        assert!(tools.len() > 10, "应有 10+ 工具面, got {}", tools.len());
        // 兜底工具存在
        assert!(tools.iter().any(|t| t.id() == "neotrix_command"));
        // 特定命令工具存在
        let ids: Vec<&str> = tools.iter().map(|t| t.id()).collect();
        assert!(ids.contains(&"neotrix_cmd_help"), "应有 help 命令工具");
        assert!(ids.contains(&"neotrix_cmd_memory"), "应有 memory 聚合器工具 / 兜底");
    }
}