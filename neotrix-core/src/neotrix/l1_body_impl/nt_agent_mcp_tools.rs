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
        McpToolDef {
            name: "neotrix_code_graph".into(),
            description: "Deterministic code-graph retrieval (G1, codebase-memory-mcp): search symbols, file stats, graph topology, get node".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["code-graph".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "action": {"type": "string", "enum": ["search_symbols", "file_stats", "graph_topology", "get_node"]},
                "query": {"type": "string"},
                "root": {"type": "string"}
            }, "required": ["action"]}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_judge".into(),
            description: "C2 judge: run a program against test cases and produce verdict (passed/wrong_answer/timeout/runtime_error/compile_error) via the sandbox. Verdict is machine-readable JSON".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["judge".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "runtime": {"type": "string", "enum": ["python3", "node18", "rust", "go", "linux"]},
                "code": {"type": "string"},
                "expected": {"type": "string"},
                "input": {"type": "string"}
            }, "required": ["runtime", "code", "expected"]}),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_tools_are_valid_mcp_defs() {
        let tools = neotrix_mcp_tools();
        assert_eq!(tools.len(), 4, "built-in tool registry must expose 4 tools");
        for t in &tools {
            assert!(!t.name.is_empty(), "tool name must be non-empty");
            assert!(!t.description.is_empty(), "tool description must be non-empty");
            assert_eq!(t.server_name, "built-in");
            assert!(t.input_schema.is_object(), "input schema must be an object");
            assert!(t.schema_version.is_none(), "built-ins carry no schema version");
        }
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"neotrix_search"));
        assert!(names.contains(&"neotrix_reason"));
        assert!(names.contains(&"neotrix_code_graph"));
        assert!(names.contains(&"neotrix_judge"));
    }

    #[test]
    fn registration_folds_n_to_4() {
        let mut registry = McpRegistry::new();
        let folded = register_neotrix_tools(&mut registry);
        assert_eq!(folded.categories.len(), 4, "N→4 fold must produce exactly 4 categories");
        assert!(folded.saved_tokens > 0, "folding must reduce token budget vs raw specs");
        assert!(folded.savings_percent > 0.0);
        let registered = registry.list_tools();
        assert_eq!(registered.len(), 4, "all built-in tools must register");
    }

    #[test]
    fn registration_is_idempotent() {
        let mut registry = McpRegistry::new();
        let first = register_neotrix_tools(&mut registry);
        let second = register_neotrix_tools(&mut registry);
        assert_eq!(registry.list_tools().len(), 4, "re-registration must not duplicate");
        assert_eq!(first.folded_chars, second.folded_chars, "fold result must be stable");
    }
}
