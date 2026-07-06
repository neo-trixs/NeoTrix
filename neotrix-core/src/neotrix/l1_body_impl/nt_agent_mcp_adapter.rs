use serde_json::Value;

use crate::agent::tool::mcp::McpToolDef;
use neotrix_types::traits::{NativeTool, ToolOutput};
use crate::neotrix::nt_agent_mcp_transport::{mcp_call_tool, TransportMode};

/// McpToolAdapter — 将外部 MCP 服务器包装为 NativeTool
///
/// 吸收管线的一部分：外部 MCP 服务器 → McpToolAdapter → ToolOrchestrator
/// 上层（GWT、SEAL、nt_cap）看到的只是一个普通 NativeTool。
pub struct McpToolAdapter {
    server_name: String,
    transport: TransportMode,
    tools: Vec<McpToolDef>,
}

impl McpToolAdapter {
    pub fn new(server_name: &str, transport: TransportMode, tools: Vec<McpToolDef>) -> Self {
        Self {
            server_name: server_name.to_string(),
            transport,
            tools,
        }
    }
}

impl NativeTool for McpToolAdapter {
    fn id(&self) -> &str {
        // 如果只有一个工具，直接用它名字
        if self.tools.len() == 1 {
            &self.tools[0].name
        } else {
            &self.server_name
        }
    }

    fn description(&self) -> &str {
        if self.tools.len() == 1 {
            &self.tools[0].description
        } else {
            "Absorbed MCP server"
        }
    }

    fn input_schema(&self) -> Value {
        if self.tools.len() == 1 {
            self.tools[0].input_schema.clone()
        } else {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "tool_name": {
                        "type": "string",
                        "enum": self.tools.iter().map(|t| t.name.clone()).collect::<Vec<_>>(),
                        "description": "Tool to call on this server"
                    }
                },
                "required": ["tool_name"]
            })
        }
    }

    fn capability_tags(&self) -> Vec<&'static str> {
        vec!["mcp_absorbed"]
    }

    fn execute(&self, args: &Value) -> Result<ToolOutput, String> {
        let tool_name = if self.tools.len() == 1 {
            self.tools[0].name.clone()
        } else {
            args.get("tool_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing 'tool_name' for multi-tool server".to_string())?
                .to_string()
        };

        let (content, _cache) = mcp_call_tool(&self.transport, &tool_name, args)
            .map_err(|e| format!("MCP tool '{}' failed: {}", tool_name, e))?;

        Ok(ToolOutput {
            success: true,
            content,
        })
    }
}
