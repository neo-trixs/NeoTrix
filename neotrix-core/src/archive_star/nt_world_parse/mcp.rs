//! MCP 工具 — 文档解析 MCP 服务器
//!
//! 提供 MCP 工具:
//! - parse_pdf(path) → markdown
//! - parse_pdf_json(path) → json
//! - ingest_pdf(path) → KB store

pub struct DocParseMcp;

impl DocParseMcp {
    pub fn new() -> Self {
        Self
    }

    pub fn tool_definitions(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "parse_pdf",
                "description": "Parse a PDF document to Markdown",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Path to PDF file"}
                    },
                    "required": ["path"]
                }
            }),
            serde_json::json!({
                "name": "parse_pdf_json",
                "description": "Parse a PDF document to JSON tree",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Path to PDF file"}
                    },
                    "required": ["path"]
                }
            }),
        ]
    }
}

impl Default for DocParseMcp {
    fn default() -> Self {
        Self::new()
    }
}
