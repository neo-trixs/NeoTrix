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
            name: "neotrix_kb_snapshot".into(),
            description: "Capture a full KB snapshot (nodes/edges/stats) to a JSON file (read-only). \
                          Returns the snapshot path plus node/edge/stats summary".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["kb".into(), "snapshot".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "out": {"type": "string", "description": "snapshot output path (default ~/.neotrix/snapshots/kb-<ts>.json)"}
            }, "required": []}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_kb_diff".into(),
            description: "Diff two KB snapshots, or one snapshot against the current KB (read-only). \
                          Returns added/removed/changed nodes and edges".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["kb".into(), "diff".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "snap_a": {"type": "string", "description": "path to the first snapshot"},
                "snap_b": {"type": "string", "description": "path to the second snapshot (omit to diff against current KB)"},
                "detail": {"type": "integer", "description": "max detail rows per section (default 10)"}
            }, "required": ["snap_a"]}),
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
        McpToolDef {
            name: "neotrix_trade_send_rfq".into(),
            description: "Send a Request for Quotation (RFQ) to suppliers for a foreign trade product. Input: product info, target countries, quantity. Returns RFQ ID and supplier list".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["trade".into(), "rfq".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "product_name": {"type": "string", "description": "product name or description"},
                "target_countries": {"type": "array", "items": {"type": "string"}, "description": "destination country codes (e.g. [\"US\",\"EU\"])"},
                "quantity": {"type": "integer", "description": "order quantity"},
                "specs": {"type": "string", "description": "optional technical specifications"}
            }, "required": ["product_name", "target_countries", "quantity"]}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_trade_gen_comparison".into(),
            description: "Generate a supplier comparison report from RFQ responses. Input: RFQ ID or response list. Returns ranked suppliers with cost/lead/risk scores".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["trade".into(), "compare".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "rfq_id": {"type": "string", "description": "RFQ identifier to fetch responses"},
                "responses": {"type": "array", "description": "inline supplier responses (alternative to rfq_id)"},
                "weight_cost": {"type": "number", "description": "weight for cost factor 0.0-1.0 (default 0.4)"},
                "weight_quality": {"type": "number", "description": "weight for quality factor 0.0-1.0 (default 0.3)"},
                "weight_delivery": {"type": "number", "description": "weight for delivery speed factor 0.0-1.0 (default 0.3)"}
            }, "required": []}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_trade_check_lc".into(),
            description: "Check a Letter of Credit document for soft clauses, discrepancies, and compliance risks. Input: LC text or structured fields. Returns risk report with soft clause list".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["trade".into(), "lc-check".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "lc_text": {"type": "string", "description": "full LC text or Swift message"},
                "lc_number": {"type": "string", "description": "LC reference number"},
                "issuer_bank": {"type": "string", "description": "issuing bank name"},
                "beneficiary": {"type": "string", "description": "beneficiary (exporter) name"},
                "amount": {"type": "number", "description": "LC amount"},
                "currency": {"type": "string", "description": "LC currency code"},
                "country": {"type": "string", "description": "destination country code"}
            }, "required": ["lc_text"]}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_trade_track_production".into(),
            description: "Track production order progress across factory milestones. Input: order ID or production plan. Returns current stage, ETA, and delay risks".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["trade".into(), "track".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "order_id": {"type": "string", "description": "production order ID"},
                "factory": {"type": "string", "description": "factory name or code"},
                "milestone": {"type": "string", "description": "milestone to check (material_purchase/cutting/sewing/finishing/packing)"},
                "eta": {"type": "string", "description": "expected delivery date (ISO 8601)"}
            }, "required": ["order_id"]}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_trade_gen_customs_doc".into(),
            description: "Generate customs declaration documents (commercial invoice, packing list, certificate of origin). Input: trade data + document type. Returns document content or file path".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["trade".into(), "customs".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "doc_type": {"type": "string", "enum": ["commercial_invoice", "packing_list", "certificate_of_origin", "bill_of_lading", "customs_declaration"], "description": "document type to generate"},
                "order_id": {"type": "string", "description": "trade order ID"},
                "buyer": {"type": "string", "description": "buyer/consignee name"},
                "seller": {"type": "string", "description": "seller/exporter name"},
                "product": {"type": "string", "description": "product description"},
                "quantity": {"type": "integer", "description": "quantity"},
                "unit_price": {"type": "number", "description": "unit price"},
                "total_value": {"type": "number", "description": "total value"},
                "hs_code": {"type": "string", "description": "HS tariff code"},
                "origin_country": {"type": "string", "description": "country of origin"},
                "dest_country": {"type": "string", "description": "destination country"},
                "incoterm": {"type": "string", "description": "Incoterm (FOB/CIF/EXW/etc.)"}
            }, "required": ["doc_type"]}),
            schema_version: None,
        },
        McpToolDef {
            name: "neotrix_trade_monitor_fx".into(),
            description: "Monitor foreign exchange rates and alert on threshold breaches. Input: currency pair + thresholds. Returns current rate, trend, and alert status".into(),
            server_name: "built-in".into(),
            transport: McpTransport::Local {
                command: "neotrix".into(),
                args: vec!["trade".into(), "fx".into()],
            },
            input_schema: serde_json::json!({"type": "object", "properties": {
                "base_currency": {"type": "string", "description": "base currency code (e.g. USD)"},
                "quote_currency": {"type": "string", "description": "quote currency code (e.g. CNY)"},
                "alert_above": {"type": "number", "description": "alert when rate exceeds this value"},
                "alert_below": {"type": "number", "description": "alert when rate drops below this value"},
                "hedge_ratio": {"type": "number", "description": "recommended hedge ratio 0.0-1.0"}
            }, "required": ["base_currency", "quote_currency"]}),
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
        assert_eq!(tools.len(), 16, "built-in tool registry must expose 16 tools (10 core + 6 trade)");
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
        assert!(names.contains(&"neotrix_kb_snapshot"));
        assert!(names.contains(&"neotrix_kb_diff"));
        assert!(names.contains(&"neotrix_kb_write"));
        assert!(names.contains(&"neotrix_trade_send_rfq"));
        assert!(names.contains(&"neotrix_trade_gen_comparison"));
        assert!(names.contains(&"neotrix_trade_check_lc"));
        assert!(names.contains(&"neotrix_trade_track_production"));
        assert!(names.contains(&"neotrix_trade_gen_customs_doc"));
        assert!(names.contains(&"neotrix_trade_monitor_fx"));
    }

    #[test]
    fn registration_folds_n_to_16() {
        let mut registry = McpRegistry::new();
        let folded = register_neotrix_tools(&mut registry);
        assert_eq!(folded.categories.len(), 4, "N→4 fold must produce exactly 4 categories");
        assert!(folded.saved_tokens > 0, "folding must reduce token budget vs raw specs");
        assert!(folded.savings_percent > 0.0);
        let registered = registry.list_tools();
        assert_eq!(registered.len(), 16, "all 16 built-in tools must register");
        assert!(
            registered.iter().any(|t| t.name == "neotrix_kb_write"),
            "kb write tool must be registered"
        );
        assert!(
            registered.iter().any(|t| t.name == "neotrix_trade_send_rfq"),
            "trade send_rfq tool must be registered"
        );
        assert!(
            registered.iter().any(|t| t.name == "neotrix_trade_check_lc"),
            "trade check_lc tool must be registered"
        );
    }

    #[test]
    fn registration_is_idempotent() {
        let mut registry = McpRegistry::new();
        let first = register_neotrix_tools(&mut registry);
        let second = register_neotrix_tools(&mut registry);
        assert_eq!(registry.list_tools().len(), 16, "re-registration must not duplicate");
        assert_eq!(first.folded_chars, second.folded_chars, "fold result must be stable");
    }
}
