use std::sync::Arc;

use super::protocol::{
    CancelNotification, InitializeRequest, InitializeResponse,
    JsonRpcMessage, NewSessionRequest, NewSessionResponse, PromptRequest,
    PromptResponse, ContentBlock, StopReason,
};
use super::session::SessionManager;
use super::transport::StdioTransport;

const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const INTERNAL_ERROR: i32 = -32603;

/// ACP method dispatcher
pub struct AcpHandler {
    sessions: Arc<SessionManager>,
    #[allow(dead_code)]
    transport: Arc<StdioTransport>,
}

impl AcpHandler {
    pub fn new(sessions: Arc<SessionManager>, transport: Arc<StdioTransport>) -> Self {
        Self {
            sessions,
            transport,
        }
    }

    /// Dispatch an incoming JSON-RPC message.
    pub fn dispatch(&self, msg: JsonRpcMessage) -> Option<JsonRpcMessage> {
        match msg {
            JsonRpcMessage::Request {
                id,
                method,
                params,
                ..
            } => {
                let result = self.handle_method(&method, &params, id);
                Some(result)
            }
            JsonRpcMessage::Notification {
                method,
                params,
                ..
            } => {
                self.handle_notification(&method, &params);
                None
            }
            JsonRpcMessage::Response { .. } => {
                log::warn!("[acp] unexpected response message (agent should not receive responses)");
                None
            }
        }
    }

    fn handle_method(&self, method: &str, params: &serde_json::Value, id: u64) -> JsonRpcMessage {
        match method {
            "initialize" => self.handle_initialize(params, id),
            "session/new" => self.handle_session_new(params, id),
            "session/prompt" => self.handle_session_prompt(params, id),
            "ping" => JsonRpcMessage::success(id, serde_json::json!({"pong": true})),
            _ => JsonRpcMessage::error(id, METHOD_NOT_FOUND, format!("unknown method: {}", method)),
        }
    }

    fn handle_notification(&self, method: &str, params: &serde_json::Value) {
        match method {
            "session/cancel" => {
                if let Ok(cancel) = serde_json::from_value::<CancelNotification>(params.clone()) {
                    let _ = self.sessions.close_session(&cancel.session_id);
                    log::info!("[acp] cancelled session {}", cancel.session_id);
                } else {
                    log::warn!("[acp] invalid cancel notification params");
                }
            }
            _ => {
                log::debug!("[acp] unhandled notification: {}", method);
            }
        }
    }

    fn handle_initialize(&self, params: &serde_json::Value, id: u64) -> JsonRpcMessage {
        let _init: InitializeRequest = match serde_json::from_value(params.clone()) {
            Ok(req) => req,
            Err(e) => {
                return JsonRpcMessage::error(
                    id,
                    INVALID_PARAMS,
                    format!("invalid initialize params: {}", e),
                );
            }
        };
        let response = InitializeResponse::default();
        match serde_json::to_value(&response) {
            Ok(val) => JsonRpcMessage::success(id, val),
            Err(e) => JsonRpcMessage::error(id, INTERNAL_ERROR, format!("serialization error: {}", e)),
        }
    }

    fn handle_session_new(&self, params: &serde_json::Value, id: u64) -> JsonRpcMessage {
        let req: NewSessionRequest = match serde_json::from_value(params.clone()) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcMessage::error(
                    id,
                    INVALID_PARAMS,
                    format!("invalid session/new params: {}", e),
                );
            }
        };
        let session_id = req.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        match self.sessions.create_session(session_id.clone(), req.cwd, req.metadata) {
            Ok(sid) => {
                let response = NewSessionResponse {
                    session_id: sid,
                    created: true,
                };
                match serde_json::to_value(&response) {
                    Ok(val) => JsonRpcMessage::success(id, val),
                    Err(e) => JsonRpcMessage::error(id, INTERNAL_ERROR, format!("serialization error: {}", e)),
                }
            }
            Err(e) => JsonRpcMessage::error(id, INVALID_PARAMS, e),
        }
    }

    fn handle_session_prompt(&self, params: &serde_json::Value, id: u64) -> JsonRpcMessage {
        let req: PromptRequest = match serde_json::from_value(params.clone()) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcMessage::error(
                    id,
                    INVALID_PARAMS,
                    format!("invalid session/prompt params: {}", e),
                );
            }
        };

        // Verify session exists
        if !self.sessions.session_exists(&req.session_id) {
            return JsonRpcMessage::error(
                id,
                INVALID_PARAMS,
                format!("session {} not found", req.session_id),
            );
        }

        // For P0, echo back the user message as a simple response
        let text = req
            .message
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");

        let response = PromptResponse {
            session_id: req.session_id.clone(),
            content: vec![ContentBlock::Text {
                text: format!("Echo: {}", text),
            }],
            stop_reason: StopReason::EndTurn,
            usage: None,
        };

        match serde_json::to_value(&response) {
            Ok(val) => JsonRpcMessage::success(id, val),
            Err(e) => JsonRpcMessage::error(id, INTERNAL_ERROR, format!("serialization error: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_handler() -> AcpHandler {
        let sessions = Arc::new(SessionManager::new());
        let transport = Arc::new(StdioTransport::new());
        AcpHandler::new(sessions, transport)
    }

    #[test]
    fn test_handle_initialize() {
        let handler = make_handler();
        let params = json!({"protocol_version": 1, "capabilities": {}, "client_info": {}});
        let response = handler.handle_method("initialize", &params, 1);
        match &response {
            JsonRpcMessage::Response { result: Some(r), .. } => {
                assert_eq!(r["protocol_version"], 1);
                assert_eq!(r["agent_info"]["name"], "neotrix");
            }
            _ => panic!("expected success response"),
        }
    }

    #[test]
    fn test_handle_unknown_method() {
        let handler = make_handler();
        let response = handler.handle_method("bogus", &json!({}), 1);
        match &response {
            JsonRpcMessage::Response { error: Some(e), .. } => {
                assert_eq!(e.code, -32601);
            }
            _ => panic!("expected error response"),
        }
    }

    #[test]
    fn test_handle_session_new() {
        let handler = make_handler();
        let params = json!({"session_id": "test-session"});
        let response = handler.handle_method("session/new", &params, 1);
        match &response {
            JsonRpcMessage::Response { result: Some(r), .. } => {
                assert_eq!(r["session_id"], "test-session");
                assert_eq!(r["created"], true);
            }
            _ => panic!("expected success response"),
        }
    }

    #[test]
    fn test_handle_session_new_auto_id() {
        let handler = make_handler();
        let params = json!({});
        let response = handler.handle_method("session/new", &params, 1);
        match &response {
            JsonRpcMessage::Response { result: Some(r), .. } => {
                assert!(r["session_id"].as_str().unwrap().len() > 10);
                assert_eq!(r["created"], true);
            }
            _ => panic!("expected success response"),
        }
    }

    #[test]
    fn test_handle_session_duplicate() {
        let handler = make_handler();
        handler.handle_method("session/new", &json!({"session_id": "dup"}), 1);
        let response = handler.handle_method("session/new", &json!({"session_id": "dup"}), 2);
        match &response {
            JsonRpcMessage::Response { error: Some(e), .. } => {
                assert_eq!(e.code, -32602);
            }
            _ => panic!("expected error response"),
        }
    }

    #[test]
    fn test_handle_ping() {
        let handler = make_handler();
        let response = handler.handle_method("ping", &json!({}), 1);
        match &response {
            JsonRpcMessage::Response { result: Some(r), .. } => {
                assert_eq!(r["pong"], true);
            }
            _ => panic!("expected pong"),
        }
    }

    #[test]
    fn test_handle_cancel_notification() {
        let handler = make_handler();
        handler.handle_method("session/new", &json!({"session_id": "to-cancel"}), 1);
        assert_eq!(handler.sessions.session_count(), 1);
        let notif = JsonRpcMessage::Notification {
            jsonrpc: "2.0".into(),
            method: "session/cancel".into(),
            params: json!({"session_id": "to-cancel"}),
        };
        handler.dispatch(notif);
        assert_eq!(handler.sessions.session_count(), 0);
    }

    #[test]
    fn test_dispatch_request_returns_response() {
        let handler = make_handler();
        let msg = JsonRpcMessage::Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "ping".into(),
            params: json!({}),
        };
        let result = handler.dispatch(msg);
        assert!(result.is_some());
    }

    #[test]
    fn test_dispatch_notification_returns_none() {
        let handler = make_handler();
        let msg = JsonRpcMessage::Notification {
            jsonrpc: "2.0".into(),
            method: "session/cancel".into(),
            params: json!({"session_id": "x"}),
        };
        let result = handler.dispatch(msg);
        assert!(result.is_none());
    }
}
