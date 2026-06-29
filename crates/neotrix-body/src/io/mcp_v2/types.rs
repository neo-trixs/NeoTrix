#![forbid(unsafe_code)]

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub fn parse_error() -> Self {
        Self { code: -32700, message: "Parse error".into(), data: None }
    }
    pub fn invalid_request() -> Self {
        Self { code: -32600, message: "Invalid Request".into(), data: None }
    }
    pub fn method_not_found() -> Self {
        Self { code: -32601, message: "Method not found".into(), data: None }
    }
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self { code: -32603, message: msg.into(), data: None }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct McpResourceContents {
    pub uri: String,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<McpPromptArgument>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct McpServerCapabilities {
    pub tools: Option<HashMap<String, bool>>,
    pub resources: Option<HashMap<String, bool>>,
    pub prompts: Option<HashMap<String, bool>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct McpInitializeResult {
    pub protocol_version: String,
    pub capabilities: McpServerCapabilities,
    pub server_info: McpServerInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_error_codes() {
        assert_eq!(JsonRpcError::parse_error().code, -32700);
        assert_eq!(JsonRpcError::invalid_request().code, -32600);
        assert_eq!(JsonRpcError::method_not_found().code, -32601);
        assert_eq!(JsonRpcError::internal_error("err").code, -32603);
    }

    #[test]
    fn test_json_rpc_error_message() {
        let e = JsonRpcError::internal_error("something broke");
        assert_eq!(e.message, "something broke");
    }

    #[test]
    fn test_mcp_tool_serde() {
        let tool = McpTool {
            name: "test".into(),
            description: "A test tool".into(),
            input_schema: serde_json::json!({"type": "object"}),
        };
        let json = serde_json::to_string(&tool).unwrap();
        let back: McpTool = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test");
    }

    #[test]
    fn test_mcp_resource_serde() {
        let r = McpResource {
            uri: "neotrix://test".into(),
            name: "Test".into(),
            description: "desc".into(),
            mime_type: "text/plain".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: McpResource = serde_json::from_str(&json).unwrap();
        assert_eq!(back.uri, "neotrix://test");
    }

    #[test]
    fn test_mcp_initialize_result() {
        let result = McpInitializeResult {
            protocol_version: "2025-03-26".into(),
            capabilities: McpServerCapabilities {
                tools: Some([("list".into(), true)].into()),
                resources: None,
                prompts: None,
            },
            server_info: McpServerInfo {
                name: "neotrix".into(),
                version: "1.0.0".into(),
            },
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("2025-03-26"));
        assert!(json.contains("neotrix"));
    }

    #[test]
    fn test_mcp_prompt_serde() {
        let p = McpPrompt {
            name: "greet".into(),
            description: "Greet the user".into(),
            arguments: vec![McpPromptArgument {
                name: "name".into(),
                description: "User name".into(),
                required: true,
            }],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: McpPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "greet");
        assert_eq!(back.arguments.len(), 1);
        assert!(back.arguments[0].required);
    }

    #[test]
    fn test_json_rpc_request_no_params() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "ping".into(),
            params: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("params")); // skip_serializing_if
        let back: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.method, "ping");
        assert!(back.params.is_none());
    }

    #[test]
    fn test_json_rpc_response_with_error() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: 1,
            result: None,
            error: Some(JsonRpcError::method_not_found()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("-32601") || json.contains("method"), "json should contain error info: {json}");
        let back: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert!(back.error.is_some());
        assert!(back.result.is_none());
    }

    #[test]
    fn test_mcp_resource_contents_text() {
        let c = McpResourceContents {
            uri: "neotrix://hello".into(),
            mime_type: "text/plain".into(),
            text: Some("world".into()),
            blob: None,
        };
        assert_eq!(c.text.as_deref(), Some("world"));
    }
}
