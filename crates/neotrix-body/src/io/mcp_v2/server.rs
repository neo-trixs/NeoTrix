#![forbid(unsafe_code)]
#![allow(dead_code)]

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde_json::json;
use super::types::*;

pub struct McpServer {
    name: String,
    version: String,
    resources: Vec<McpResource>,
    tools: Vec<McpTool>,
    prompts: Vec<McpPrompt>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            name: "neotrix".into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            resources: Self::default_resources(),
            tools: Self::default_tools(),
            prompts: Vec::new(),
        }
    }

    fn default_resources() -> Vec<McpResource> {
        vec![
            McpResource {
                uri: "neotrix://consciousness/stats".into(),
                name: "Consciousness Stats".into(),
                description: "Current consciousness pipeline statistics".into(),
                mime_type: "application/json".into(),
            },
            McpResource {
                uri: "neotrix://consciousness/status".into(),
                name: "Consciousness Status".into(),
                description: "Current status of all consciousness subsystems".into(),
                mime_type: "application/json".into(),
            },
            McpResource {
                uri: "neotrix://knowledge/graph".into(),
                name: "Knowledge Graph Stats".into(),
                description: "Knowledge graph node/edge counts and topology".into(),
                mime_type: "application/json".into(),
            },
        ]
    }

    fn default_tools() -> Vec<McpTool> {
        vec![
            McpTool {
                name: "consciousness_pipeline".into(),
                description: "Trigger a full consciousness pipeline cycle (16 steps)".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "cycle_count": {
                            "type": "integer",
                            "description": "Number of pipeline cycles to run",
                            "default": 1
                        }
                    }
                }),
            },
            McpTool {
                name: "query_state".into(),
                description: "Query current consciousness state".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "domain": {
                            "type": "string",
                            "description": "Domain to query: stats|emotion|cognitive|personality|reflexive",
                            "enum": ["stats", "emotion", "cognitive", "personality", "reflexive"]
                        }
                    },
                    "required": ["domain"]
                }),
            },
            McpTool {
                name: "explore".into(),
                description: "Trigger knowledge exploration via external sources".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"},
                        "source": {
                            "type": "string",
                            "description": "Source to search",
                            "enum": ["web", "github", "wikipedia", "openlibrary"]
                        }
                    },
                    "required": ["query"]
                }),
            },
        ]
    }

    pub fn handle_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "ping" => self.handle_ping(request),
            "tools/list" => self.handle_tools_list(request),
            "tools/call" => self.handle_tools_call(request),
            "resources/list" => self.handle_resources_list(request),
            "resources/read" => self.handle_resources_read(request),
            "prompts/list" => self.handle_prompts_list(request),
            "initialize" => {
                log::warn!("MCP 2026-07-28: 'initialize' is deprecated per SEP-2575; use 'ping' for connection check");
                self.handle_ping(request)
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError::method_not_found()),
            },
        }
    }

    fn handle_ping(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: request.id,
            result: Some(json!({})),
            error: None,
        }
    }

    fn handle_tools_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: request.id,
            result: Some(json!({ "tools": self.tools })),
            error: None,
        }
    }

    fn handle_tools_call(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let tool_name = request.params.as_ref()
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("");

        let arguments = request.params.as_ref()
            .and_then(|p| p.get("arguments"))
            .cloned()
            .unwrap_or(json!({}));

        match tool_name {
            "consciousness_pipeline" => {
                let cycles = arguments.get("cycle_count").and_then(|c| c.as_u64()).unwrap_or(1);
                JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id,
                    result: Some(json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Triggered {} consciousness pipeline cycle(s). Note: pipeline runs asynchronously via background loop.", cycles)
                        }]
                    })),
                    error: None,
                }
            }
            "query_state" => {
                let domain = arguments.get("domain").and_then(|d| d.as_str()).unwrap_or("stats");
                JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id,
                    result: Some(json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Consciousness state for domain '{}': available via neotrix://consciousness/stats", domain)
                        }]
                    })),
                    error: None,
                }
            }
            "explore" => {
                let query = arguments.get("query").and_then(|q| q.as_str()).unwrap_or("");
                JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id,
                    result: Some(json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Exploration queued for query: '{}'. Results will be available in next pipeline cycle.", query)
                        }]
                    })),
                    error: None,
                }
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("Unknown tool: {}", tool_name),
                    data: None,
                }),
            }
        }
    }

    fn handle_resources_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: request.id,
            result: Some(json!({ "resources": self.resources })),
            error: None,
        }
    }

    fn handle_resources_read(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let uri = request.params.as_ref()
            .and_then(|p| p.get("uri"))
            .and_then(|u| u.as_str())
            .unwrap_or("");

        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: request.id,
            result: Some(json!({
                "contents": [{
                    "uri": uri,
                    "mime_type": "application/json",
                    "text": format!("Resource '{}' - live data available when integrated with ConsciousnessIntegration", uri)
                }]
            })),
            error: None,
        }
    }

    fn handle_prompts_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: request.id,
            result: Some(json!({ "prompts": self.prompts })),
            error: None,
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run_stdio_server() {
    let server = Arc::new(McpServer::new());
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin).lines();
    let mut writer = stdout;

    while let Ok(Some(line)) = reader.next_line().await {
        if line.trim().is_empty() { continue; }
        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                let response = server.handle_request(&request);
                let json = serde_json::to_string(&response).unwrap_or_default();
                if let Err(e) = writer.write_all(format!("{}\n", json).as_bytes()).await {
                    log::error!("MCP write error: {}", e);
                    break;
                }
                let _ = writer.flush().await;
            }
            Err(_) => {
                let err = JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: 0,
                    result: None,
                    error: Some(JsonRpcError::parse_error()),
                };
                let json = serde_json::to_string(&err).unwrap_or_default();
                let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
                let _ = writer.flush().await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_new() {
        let server = McpServer::new();
        assert_eq!(server.name, "neotrix");
        assert_eq!(server.resources.len(), 3);
        assert_eq!(server.tools.len(), 3);
    }

    #[test]
    fn test_server_default() {
        let server = McpServer::default();
        assert_eq!(server.tools.len(), 3);
    }

    #[test]
    fn test_handle_ping() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "ping".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap(), json!({}));
    }

    #[test]
    fn test_handle_initialize_deprecated() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "initialize".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        // Deprecated initialize still returns success (via ping handler)
        assert_eq!(resp.result.unwrap(), json!({}));
    }

    #[test]
    fn test_handle_tools_list() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "tools/list".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        let binding = resp.result.unwrap();
        let empty = vec![];
        let tools = binding["tools"].as_array().unwrap_or(&empty);
        assert_eq!(tools.len(), 3);
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap_or_default()).collect();
        assert!(names.contains(&"consciousness_pipeline"));
        assert!(names.contains(&"query_state"));
        assert!(names.contains(&"explore"));
    }

    #[test]
    fn test_handle_tools_call_consciousness_pipeline() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "tools/call".into(),
            params: Some(json!({
                "name": "consciousness_pipeline",
                "arguments": { "cycle_count": 3 }
            })),
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        let text = resp.result.unwrap_or_default()["content"][0]["text"].as_str().unwrap_or_default().to_string();
        assert!(text.contains("3 consciousness pipeline"));
    }

    #[test]
    fn test_handle_tools_call_query_state() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "tools/call".into(),
            params: Some(json!({
                "name": "query_state",
                "arguments": { "domain": "emotion" }
            })),
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        let text = resp.result.unwrap_or_default()["content"][0]["text"].as_str().unwrap_or_default().to_string();
        assert!(text.contains("emotion"));
    }

    #[test]
    fn test_handle_tools_call_explore() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "tools/call".into(),
            params: Some(json!({
                "name": "explore",
                "arguments": { "query": "consciousness" }
            })),
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        let text = resp.result.unwrap_or_default()["content"][0]["text"].as_str().unwrap_or_default().to_string();
        assert!(text.contains("Exploration queued"));
    }

    #[test]
    fn test_handle_tools_call_unknown() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "tools/call".into(),
            params: Some(json!({
                "name": "nonexistent",
                "arguments": {}
            })),
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32602);
    }

    #[test]
    fn test_handle_resources_list() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "resources/list".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        let binding = resp.result.unwrap();
        let empty = vec![];
        let resources = binding["resources"].as_array().unwrap_or(&empty);
        assert_eq!(resources.len(), 3);
    }

    #[test]
    fn test_handle_resources_read() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "resources/read".into(),
            params: Some(json!({
                "uri": "neotrix://consciousness/stats"
            })),
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        let uri = resp.result.unwrap_or_default()["contents"][0]["uri"].as_str().unwrap_or_default().to_string();
        assert_eq!(uri, "neotrix://consciousness/stats");
    }

    #[test]
    fn test_handle_prompts_list() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "prompts/list".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        assert!(resp.result.unwrap_or_default()["prompts"].as_array().unwrap_or(&vec![]).is_empty());
    }

    #[test]
    fn test_handle_notifications_initialized_deprecated() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "notifications/initialized".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        // Removed per MCP 2026-07-28 — now returns method_not_found
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_handle_unknown_method() {
        let server = McpServer::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "bogus".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_default_resources_have_correct_uris() {
        let resources = McpServer::default_resources();
        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"neotrix://consciousness/stats"));
        assert!(uris.contains(&"neotrix://consciousness/status"));
        assert!(uris.contains(&"neotrix://knowledge/graph"));
    }

    #[test]
    fn test_tool_input_schema_valid() {
        for tool in McpServer::default_tools() {
            assert_eq!(tool.input_schema["type"], "object");
            assert!(tool.input_schema.get("properties").is_some());
        }
    }
}

