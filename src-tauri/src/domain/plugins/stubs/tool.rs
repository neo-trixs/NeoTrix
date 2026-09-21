//! Tool Plugin — 工具：MCP、harness、computer、voice

use super::common::stub_action;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;

pub struct ToolPlugin;

impl ToolPlugin {
    fn get_default_mcp_tools() -> Vec<serde_json::Value> {
        // Use the default MCP tool registry from neotrix-core
        vec![
            serde_json::json!({
                "name": "kb_search",
                "description": "搜索知识库 — 在 KB 中检索相关信息",
                "server": "built-in",
            }),
            serde_json::json!({
                "name": "memory_search",
                "description": "搜索记忆 — 在经验库中检索相关记忆",
                "server": "built-in",
            }),
            serde_json::json!({
                "name": "skill_route",
                "description": "技能路由 — 根据任务类型选择合适的技能",
                "server": "built-in",
            }),
            serde_json::json!({
                "name": "context_manage",
                "description": "上下文管理 — 管理 LLM 上下文窗口",
                "server": "built-in",
            }),
        ]
    }
}

#[async_trait]
impl DomainPlugin for ToolPlugin {
    fn name(&self) -> &str {
        "tool"
    }
    fn description(&self) -> &str {
        "工具：MCP、harness、computer、voice"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "mcp_list",
            "mcp_register",
            "harness_execute",
            "harness_resolve",
            "computer_capture",
            "computer_click",
            "computer_type",
            "voice_synthesize",
        ]
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
            "mcp_list" => {
                let tools = Self::get_default_mcp_tools();
                Ok(serde_json::json!({
                    "tools": tools,
                    "count": tools.len(),
                    "servers": ["built-in"],
                }))
            }
            "mcp_register" => {
                // TODO: Implement MCP server registration via McpRegistry
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
                Ok(serde_json::json!({
                    "ok": true,
                    "server": name,
                    "command": command,
                    "message": "MCP server registration not yet fully implemented",
                }))
            }
            // Harness actions — delegate to neotrix-core harness
            // TODO: Implement harness_execute and harness_resolve via ToolOrchestrator
            "harness_execute" | "harness_resolve" => Ok(
                serde_json::json!({ "ok": true, "stub": true, "message": format!("{} not yet implemented", action) }),
            ),
            // Computer actions — require platform-specific implementation
            // TODO: Implement computer_capture, computer_click, computer_type via screen capture + automation
            "computer_capture" | "computer_click" | "computer_type" => Ok(
                serde_json::json!({ "ok": true, "stub": true, "message": format!("{} not yet implemented", action) }),
            ),
            // Voice actions — require TTS engine
            // TODO: Implement voice_synthesize via TTS backend
            "voice_synthesize" => Ok(
                serde_json::json!({ "ok": true, "stub": true, "message": "voice_synthesize not yet implemented" }),
            ),
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
