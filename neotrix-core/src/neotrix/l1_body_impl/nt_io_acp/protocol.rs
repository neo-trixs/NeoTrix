use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── JSON-RPC Envelope ──

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request {
        jsonrpc: String,
        id: u64,
        method: String,
        #[serde(default)]
        params: serde_json::Value,
    },
    Notification {
        jsonrpc: String,
        method: String,
        #[serde(default)]
        params: serde_json::Value,
    },
    Response {
        jsonrpc: String,
        id: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<JsonRpcError>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcMessage {
    pub fn success(id: u64, result: serde_json::Value) -> Self {
        JsonRpcMessage::Response {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: u64, code: i32, message: impl Into<String>) -> Self {
        JsonRpcMessage::Response {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    pub fn notify(method: impl Into<String>, params: serde_json::Value) -> Self {
        JsonRpcMessage::Notification {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params,
        }
    }
}

// ── ACP Protocol V1 Types ──

pub const ACP_PROTOCOL_VERSION: i32 = 1;

// ── initialize ──

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeRequest {
    pub protocol_version: i32,
    #[serde(default)]
    pub capabilities: ClientCapabilities,
    #[serde(default)]
    pub client_info: ClientInfo,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(default)]
    pub session: SessionClientCapabilities,
    #[serde(default)]
    pub tools: ToolsClientCapabilities,
    #[serde(default)]
    pub streaming: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionClientCapabilities {
    #[serde(default)]
    pub load: bool,
    #[serde(default)]
    pub close: bool,
    #[serde(default)]
    pub list: bool,
    #[serde(default)]
    pub delete: bool,
    #[serde(default)]
    pub set_mode: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ToolsClientCapabilities {
    #[serde(default)]
    pub request_permission: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClientInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeResponse {
    pub protocol_version: i32,
    pub capabilities: AgentCapabilities,
    pub agent_info: AgentInfoResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentCapabilities {
    #[serde(default)]
    pub session: SessionAgentCapabilities,
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub push_notifications: bool,
    #[serde(default)]
    pub mcp_servers: bool,
    #[serde(default)]
    pub e8_reasoning_modes: Vec<u8>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionAgentCapabilities {
    #[serde(default)]
    pub resume: bool,
    #[serde(default)]
    pub close: bool,
    #[serde(default)]
    pub list: bool,
    #[serde(default)]
    pub set_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfoResponse {
    pub name: String,
    pub version: String,
}

impl Default for InitializeResponse {
    fn default() -> Self {
        Self {
            protocol_version: ACP_PROTOCOL_VERSION,
            capabilities: AgentCapabilities {
                session: SessionAgentCapabilities {
                    resume: true,
                    close: true,
                    list: true,
                    set_mode: true,
                },
                streaming: true,
                push_notifications: true,
                mcp_servers: true,
                e8_reasoning_modes: (0..64).collect(),
            },
            agent_info: AgentInfoResponse {
                name: "neotrix".into(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

// ── session/new ──

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSessionRequest {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSessionResponse {
    pub session_id: String,
    pub created: bool,
}

// ── session/prompt ──

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptRequest {
    pub session_id: String,
    pub message: PromptMessage,
    #[serde(default)]
    pub mode: Option<u8>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_call_id: String,
        content: Vec<ContentBlock>,
        #[serde(default)]
        is_error: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptResponse {
    pub session_id: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: StopReason,
    #[serde(default)]
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    StopRequested,
    ToolUse,
    Error,
    MaxTokens,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

// ── session/cancel ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelNotification {
    pub session_id: String,
}

// ── session/update (notifications from agent to client) ──

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionUpdate {
    #[serde(rename = "agent_message_chunk")]
    AgentMessageChunk { content: String },
    #[serde(rename = "tool_call")]
    ToolCallUpdate { id: String, name: String, arguments: serde_json::Value },
    #[serde(rename = "tool_call_update")]
    ToolCallResult { id: String, result: serde_json::Value },
    #[serde(rename = "status")]
    StatusUpdate { message: String },
    #[serde(rename = "usage_update")]
    UsageUpdate { input_tokens: u32, output_tokens: u32 },
    #[serde(rename = "current_mode_update")]
    ModeUpdate { mode: u8 },
    #[serde(rename = "error")]
    ErrorUpdate { message: String, recoverable: bool },
}

// ── session/request_permission (from agent to client) ──

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub session_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub approved: bool,
}

// ── helper: build session/update notification ──

pub fn build_update(session_id: &str, update: SessionUpdate) -> JsonRpcMessage {
    let params = serde_json::json!({
        "session_id": session_id,
        "update": update,
    });
    JsonRpcMessage::notify("session/update", params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_response_serde() {
        let resp = InitializeResponse::default();
        let json = serde_json::to_string(&resp).unwrap();
        let back: InitializeResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.protocol_version, 1);
        assert_eq!(back.agent_info.name, "neotrix");
        assert!(back.capabilities.streaming);
        assert_eq!(back.capabilities.e8_reasoning_modes.len(), 64);
    }

    #[test]
    fn test_json_rpc_request_serde() {
        let req = JsonRpcMessage::Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "initialize".into(),
            params: serde_json::json!({"protocol_version": 1}),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"initialize\""));
        let back: JsonRpcMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, JsonRpcMessage::Request { .. }));
    }

    #[test]
    fn test_json_rpc_notification_no_id() {
        let notif = JsonRpcMessage::notify("session/update", serde_json::json!({"x": 1}));
        let json = serde_json::to_string(&notif).unwrap();
        assert!(!json.contains("\"id\""));
        assert!(json.contains("\"session/update\""));
    }

    #[test]
    fn test_json_rpc_error_response() {
        let err = JsonRpcMessage::error(1, -32601, "method not found");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"code\":-32601"));
        let back: JsonRpcMessage = serde_json::from_str(&json).unwrap();
        match back {
            JsonRpcMessage::Response { error: Some(e), .. } => {
                assert_eq!(e.code, -32601);
            }
            _ => panic!("expected error response"),
        }
    }

    #[test]
    fn test_content_block_text_serde() {
        let block = ContentBlock::Text { text: "hello".into() };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, ContentBlock::Text { .. }));
    }

    #[test]
    fn test_content_block_tool_call_serde() {
        let block = ContentBlock::ToolCall {
            id: "call_1".into(),
            name: "bash".into(),
            arguments: serde_json::json!({"cmd": "ls"}),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"tool_call\""));
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        match back {
            ContentBlock::ToolCall { ref name, .. } => assert_eq!(name, "bash"),
            _ => panic!("expected tool_call"),
        }
    }

    #[test]
    fn test_session_update_serde() {
        let update = SessionUpdate::AgentMessageChunk { content: "Hello".into() };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"agent_message_chunk\""));
        assert!(json.contains("\"Hello\""));
    }

    #[test]
    fn test_prompt_request_serde() {
        let req = PromptRequest {
            session_id: "sess-1".into(),
            message: PromptMessage {
                role: "user".into(),
                content: vec![ContentBlock::Text { text: "hello".into() }],
            },
            mode: Some(42),
            max_tokens: Some(4096),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: PromptRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.session_id, "sess-1");
        assert_eq!(back.mode, Some(42));
        assert_eq!(back.max_tokens, Some(4096));
    }

    #[test]
    fn test_stop_reason_serde() {
        let cases = vec![
            (StopReason::EndTurn, "end_turn"),
            (StopReason::ToolUse, "tool_use"),
            (StopReason::Error, "error"),
        ];
        for (reason, expected) in cases {
            let json = serde_json::to_string(&reason).unwrap();
            assert!(json.contains(expected));
        }
    }

    #[test]
    fn test_new_session_serde() {
        let req = NewSessionRequest {
            session_id: Some("my-session".into()),
            cwd: Some("/home/user".into()),
            metadata: Some([("key".into(), "val".into())].into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: NewSessionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.session_id.unwrap(), "my-session");

        let resp = NewSessionResponse { session_id: "abc".into(), created: true };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"created\":true"));
    }

    #[test]
    fn test_permission_serde() {
        let req = PermissionRequest {
            session_id: "s-1".into(),
            tool_name: "bash".into(),
            arguments: serde_json::json!({"cmd": "ls"}),
            description: "list files".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: PermissionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tool_name, "bash");
    }
}
