use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Server - exposes NeoTrix domain plugins as MCP tools.
/// Allows external AI agents (Claude Desktop, Cursor, etc.) to use NeoTrix capabilities.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

pub struct McpServer {
    name: String,
    version: String,
    tools: HashMap<String, McpTool>,
}

impl McpServer {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, name: impl Into<String>, description: impl Into<String>, input_schema: serde_json::Value) {
        let tool_name = name.into();
        self.tools.insert(tool_name.clone(), McpTool {
            name: tool_name,
            description: description.into(),
            input_schema,
        });
    }

    pub fn handle_request(&self, request: &McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request),
            "tools/list" => self.handle_list_tools(request),
            "tools/call" => self.handle_call_tool(request),
            "ping" => McpResponse {
                jsonrpc: "2.0".into(),
                id: request.id.clone(),
                result: Some(serde_json::json!({})),
                error: None,
            },
            _ => McpResponse {
                jsonrpc: "2.0".into(),
                id: request.id.clone(),
                result: None,
                error: Some(McpError { code: -32601, message: format!("Method not found: {}", request.method) }),
            },
        }
    }

    fn handle_initialize(&self, request: &McpRequest) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".into(),
            id: request.id.clone(),
            result: Some(serde_json::json!({
                "protocolVersion": "2025-03-26",
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": self.name, "version": self.version }
            })),
            error: None,
        }
    }

    fn handle_list_tools(&self, request: &McpRequest) -> McpResponse {
        let tools: Vec<&McpTool> = self.tools.values().collect();
        McpResponse {
            jsonrpc: "2.0".into(),
            id: request.id.clone(),
            result: Some(serde_json::json!({ "tools": tools })),
            error: None,
        }
    }

    fn handle_call_tool(&self, request: &McpRequest) -> McpResponse {
        let params = request.params.as_ref().and_then(|p| p.as_object());
        let tool_name = params.and_then(|o| o.get("name")).and_then(|n| n.as_str()).unwrap_or("");
        let arguments = params.and_then(|o| o.get("arguments")).cloned().unwrap_or(serde_json::json!({}));

        if let Some(_tool) = self.tools.get(tool_name) {
            // Tool call would be dispatched to domain registry here
            // For now return a placeholder
            McpResponse {
                jsonrpc: "2.0".into(),
                id: request.id.clone(),
                result: Some(serde_json::json!({
                    "content": [{ "type": "text", "text": format!("Tool {} executed (placeholder)", tool_name) }]
                })),
                error: None,
            }
        } else {
            McpResponse {
                jsonrpc: "2.0".into(),
                id: request.id.clone(),
                result: None,
                error: Some(McpError { code: -32602, message: format!("Tool not found: {}", tool_name) }),
            }
        }
    }
}
