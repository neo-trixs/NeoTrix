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
        McpToolDef {
            name: "neotrix_kb_get".into(),
            description: "Read a single KB node by id (read-only). Returns id/type/title/summary/url/domain/confidence/importance".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["kb".into(), "get".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "id": {"type": "string", "description": "KB node id"}
            }, "required": ["id"]}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_kb_stats".into(),
            description: "Read KB statistics (node/edge counts, type distribution). Read-only".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["kb".into(), "stats".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {}, "required": []}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_kb_query".into(),
            description: "Advanced hybrid-rerank query over the KB (read-only). Returns ranked nodes by text".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["kb".into(), "query".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "text": {"type": "string", "description": "query text"},
                "limit": {"type": "integer", "description": "max results (default 10)"}
            }, "required": ["text"]}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_kb_write".into(),
            description: "Guarded KB write (node:create/update, edge:upsert, kv:set, node:delete, edge:delete). \
                         All writes pass the deterministic kb_write_guard (title/url/weight/protected-namespace checks); \
                         deletes and bulk ops require explicit force (operator approval). Evidence is recorded to kv_store write_guard".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["kb".into(), "write".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "action": {"type": "string", "enum": ["node:create", "node:update", "edge:upsert", "node:delete", "edge:delete", "kv:set"], "description": "write operation"},
                "id": {"type": "string"},
                "title": {"type": "string"},
                "node_type": {"type": "string"},
                "summary": {"type": "string"},
                "content": {"type": "string"},
                "url": {"type": "string"},
                "domain": {"type": "string"},
                "source_id": {"type": "string"},
                "target_id": {"type": "string"},
                "relation_type": {"type": "string"},
                "weight": {"type": "number"},
                "description": {"type": "string"},
                "namespace": {"type": "string"},
                "key": {"type": "string"},
                "value": {"type": "string"},
                "force": {"type": "boolean", "description": "explicit approval for delete/backfill (Tier3/4)"}
            }, "required": ["action"]}),
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
        assert_eq!(tools.len(), 8, "built-in tool registry must expose 8 tools");
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
        assert!(names.contains(&"neotrix_kb_get"));
        assert!(names.contains(&"neotrix_kb_stats"));
        assert!(names.contains(&"neotrix_kb_query"));
        assert!(names.contains(&"neotrix_kb_write"));
    }

    #[test]
    fn registration_folds_n_to_8() {
        let mut registry = McpRegistry::new();
        let folded = register_neotrix_tools(&mut registry);
        assert_eq!(folded.categories.len(), 4, "N→4 fold must produce exactly 4 categories");
        assert!(folded.saved_tokens > 0, "folding must reduce token budget vs raw specs");
        assert!(folded.savings_percent > 0.0);
        let registered = registry.list_tools();
        assert_eq!(registered.len(), 8, "all 8 built-in tools must register");
        assert!(
            registered.iter().any(|t| t.name == "neotrix_kb_write"),
            "kb write tool must be registered"
        );
    }

    #[test]
    fn registration_is_idempotent() {
        let mut registry = McpRegistry::new();
        let first = register_neotrix_tools(&mut registry);
        let second = register_neotrix_tools(&mut registry);
        assert_eq!(registry.list_tools().len(), 8, "re-registration must not duplicate");
        assert_eq!(first.folded_chars, second.folded_chars, "fold result must be stable");
    }
}
