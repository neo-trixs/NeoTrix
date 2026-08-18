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

#[cfg(test)]
mod tests {
    use super::*;
use crate::agent::tool::mcp::{McpToolDef, McpTransport};
    use neotrix_types::traits::NativeTool;

    fn def(name: &str) -> McpToolDef {
        McpToolDef {
            name: name.into(),
            description: format!("desc {name}"),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["mcp".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {"query": {"type": "string"}}}),
            schema_version: None,
        }
    }

    fn local_mode() -> TransportMode {
        TransportMode::Local {
            command: "neotrix".into(),
            args: vec!["mcp".into()],
        }
    }

    #[test]
    fn single_tool_surface_uses_tool_identity() {
        let adapter = McpToolAdapter::new("server-a", local_mode(), vec![def("tool_x")]);
        assert_eq!(adapter.id(), "tool_x", "single-tool server uses tool name as id");
        assert_eq!(adapter.description(), "desc tool_x");
        assert_eq!(
            adapter.input_schema()["type"],
            "object",
            "single-tool server passes through its own schema"
        );
        assert!(adapter.capability_tags().contains(&"mcp_absorbed"));
    }

    #[test]
    fn multi_tool_surface_uses_server_identity() {
        let adapter = McpToolAdapter::new(
            "server-a",
            local_mode(),
            vec![def("tool_x"), def("tool_y")],
        );
        assert_eq!(adapter.id(), "server-a", "multi-tool server uses server name as id");
        assert_eq!(adapter.description(), "Absorbed MCP server");
        let schema = adapter.input_schema();
        let tool_name = schema.pointer("/properties/tool_name").unwrap();
        assert!(tool_name.get("enum").is_some(), "multi-tool schema must enumerate tools");
        assert_eq!(tool_name["enum"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn multi_tool_execute_requires_tool_name() {
        let adapter = McpToolAdapter::new(
            "server-a",
            local_mode(),
            vec![def("tool_x"), def("tool_y")],
        );
        let err = match adapter.execute(&serde_json::json!({})) {
            Err(e) => e,
            Ok(_) => panic!("missing tool_name must fail"),
        };
        assert!(err.contains("tool_name"), "missing tool_name must error with guidance");
    }
}
