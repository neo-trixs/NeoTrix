use crate::agent::tool::mcp::{McpRegistry, McpToolDef, McpTransport};
use crate::neotrix::l1_body_impl::nt_agent_mcp_gateway::fold_tool_specs_from_defs;

/// Return a list of built-in NeoTrix MCP tool definitions.
pub fn neotrix_mcp_tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "neotrix_search".into(),
            description: "Search NeoTrix knowledge base".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["kb".into(), "search".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {"query": {"type": "string"}}}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_reason".into(),
            description: "Invoke NeoTrix reasoning engine".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["reason".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {"task": {"type": "string"}}}),
            schema_version: None,
        },
    ]
}

/// Register NeoTrix's built-in MCP tool servers into the given registry.
/// Returns the N→4 folded tool specs computed at registration time (production
/// folding: bootstrap registration and the gateway surface both use it).
pub fn register_neotrix_tools(
    registry: &mut McpRegistry,
) -> crate::neotrix::l1_body_impl::nt_agent_mcp_gateway::FoldedSpecs {
    let tools = neotrix_mcp_tools();
    registry.register_stdio("built-in", "neotrix", &["mcp"], tools);
    fold_tool_specs_from_defs(registry.list_tools())
}
