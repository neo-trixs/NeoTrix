//! # MindModules — 意识核心命令桥接 (命令 → 意识能力桥接)
//!
//! 对外只暴露一个意识核心：人类交互 (TUI/CLI/headless) 只接触基础控制命令,
//! 领域操作全部由意识核心 (AgentLoop + AttentionRouter) 智能调度。
//!
//! 本模块把命令注册表的全部命令桥接为 NativeTool, 注入 AgentLoop 的能力面。
//! LLM 意识核心通过 AttentionRouter 判断意图 → 调用对应命令工具 → 回传结果。
//!
//! # Design
//! Gateway crate defines the `CommandExecutor` trait. The core crate provides
//! the concrete implementation that wraps `CommandRegistry`. This keeps the
//! gateway decoupled from the CLI internals.

use serde_json::Value;

/// Trait for executing NeoTrix commands in-process.
///
/// Core crate implements this by wrapping `CommandRegistry::execute`.
/// Gateway crate uses this trait to bridge commands to LLM tool interfaces.
pub trait CommandExecutor: Send + Sync {
    /// Execute a command string (with or without leading `/`).
    /// Returns the text output of the command.
    fn execute(&self, command: &str) -> Result<String, String>;

    /// List all available command names (with leading `/`).
    fn list_commands(&self) -> Vec<String>;

    /// Get description for a command.
    fn describe_command(&self, name: &str) -> Option<String>;
}

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

/// 单个命令 → NativeTool 适配。LLM 通过工具名 (`neotrix_<cmd>`) 调用。
pub struct CommandNativeTool {
    name: String,
    description: String,
    executor: std::sync::Arc<dyn CommandExecutor>,
}

impl CommandNativeTool {
    pub fn new(name: &str, description: &str, executor: std::sync::Arc<dyn CommandExecutor>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            executor,
        }
    }
}

impl CommandNativeTool {
    pub fn id(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn input_schema(&self) -> Value {
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

    pub fn capability_tags(&self) -> Vec<&'static str> {
        vec!["neotrix_command"]
    }

    pub fn execute_tool(&self, args: &Value) -> Result<String, String> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing required field: command".to_string())?;
        let input = normalize_command(command);
        if input.is_empty() {
            return Err("Empty command".to_string());
        }
        self.executor.execute(&input)
    }
}

/// 从命令注册表生成全部命令工具 (意识能力面)。
///
/// 每个已注册命令生成一个 `neotrix_<cmd名去slash>` 工具, 描述引用命令原文,
/// 让 LLM 意识核心能智能调度任意能力。附带 agent_all 兜底工具。
pub fn neotrix_command_tools(executor: std::sync::Arc<dyn CommandExecutor>) -> Vec<CommandNativeTool> {
    let mut tools: Vec<CommandNativeTool> = Vec::new();

    // 兜底: 通过单个工具执行任意命令 (LLM 一次只能看到有限工具时启用)。
    tools.push(CommandNativeTool::new(
        "neotrix_command",
        "Execute any NeoTrix command in-process (agent 后端自我调度通道). \
         command 为完整命令文本, 如 'file read src/main.rs' 或 '/memory search kb'.",
        executor.clone(),
    ));

    for name in executor.list_commands() {
        if name.is_empty() {
            continue;
        }
        let tool_id = format!("neotrix_cmd_{}", name.trim_start_matches('/').replace('/', "_"));
        let desc = if let Some(desc) = executor.describe_command(&name) {
            format!("{} — {}", name, desc)
        } else {
            format!("Execute NeoTrix command {}", name)
        };
        tools.push(CommandNativeTool::new(&tool_id, &desc, executor.clone()));
    }
    tools
}

/// 便捷入口: 供 entry (TUI/agent) 装配 MindModules 工具面。
pub fn awareness_core_tools(executor: std::sync::Arc<dyn CommandExecutor>) -> Vec<CommandNativeTool> {
    neotrix_command_tools(executor)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock executor for testing
    struct MockExecutor;

    impl CommandExecutor for MockExecutor {
        fn execute(&self, command: &str) -> Result<String, String> {
            Ok(format!("executed: {}", command))
        }

        fn list_commands(&self) -> Vec<String> {
            vec![
                "/help".to_string(),
                "/file".to_string(),
                "/memory".to_string(),
            ]
        }

        fn describe_command(&self, name: &str) -> Option<String> {
            match name {
                "/help" => Some("Show help".to_string()),
                "/file" => Some("File operations".to_string()),
                "/memory" => Some("Memory operations".to_string()),
                _ => None,
            }
        }
    }

    #[test]
    fn test_normalize_command() {
        assert_eq!(normalize_command("/help"), "/help");
        assert_eq!(normalize_command("help"), "/help");
        assert_eq!(normalize_command("  file read x "), "/file read x");
        assert_eq!(normalize_command("   "), "");
    }

    #[test]
    fn test_command_tool_execute() {
        let executor: std::sync::Arc<dyn CommandExecutor> = std::sync::Arc::new(MockExecutor);
        let tool = CommandNativeTool::new("neotrix_cmd_help", "help", executor);
        let out = tool.execute_tool(&serde_json::json!({"command": "/help"})).unwrap();
        assert!(out.contains("executed"));
    }

    #[test]
    fn test_command_tool_missing_field() {
        let executor: std::sync::Arc<dyn CommandExecutor> = std::sync::Arc::new(MockExecutor);
        let tool = CommandNativeTool::new("neotrix_cmd_help", "help", executor);
        assert!(tool.execute_tool(&serde_json::json!({})).is_err());
    }

    #[test]
    fn test_neotrix_command_tools_nonempty() {
        let executor: std::sync::Arc<dyn CommandExecutor> = std::sync::Arc::new(MockExecutor);
        let tools = neotrix_command_tools(executor);
        assert!(tools.len() > 1, "应有 1+ 工具面, got {}", tools.len());
        // 兜底工具存在
        assert!(tools.iter().any(|t| t.id() == "neotrix_command"));
        // 特定命令工具存在
        let ids: Vec<&str> = tools.iter().map(|t| t.id()).collect();
        assert!(ids.contains(&"neotrix_cmd_help"), "应有 help 命令工具");
    }
}
