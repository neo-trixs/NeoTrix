use std::sync::Arc;

use super::handler::AcpHandler;
use super::protocol::JsonRpcMessage;
use super::session::SessionManager;
use super::transport::StdioTransport;

/// Routes incoming messages to the correct handler for the session.
/// For P0, this is a simple pass-through to AcpHandler.
pub struct AcpRouter {
    handler: Arc<AcpHandler>,
}

impl AcpRouter {
    pub fn new(sessions: Arc<SessionManager>, transport: Arc<StdioTransport>) -> Self {
        Self {
            handler: Arc::new(AcpHandler::new(sessions, transport)),
        }
    }

    /// Route a message — for P0, delegates directly to the handler.
    /// Future: fan-out to per-session handler tasks, load balancing, etc.
    pub fn route(&self, msg: JsonRpcMessage) -> Option<JsonRpcMessage> {
        self.handler.dispatch(msg)
    }

    pub fn handler(&self) -> &AcpHandler {
        &self.handler
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_router() -> AcpRouter {
        let sessions = Arc::new(SessionManager::new());
        let transport = Arc::new(StdioTransport::new());
        AcpRouter::new(sessions, transport)
    }

    #[test]
    fn test_route_ping() {
        let router = make_router();
        let msg = JsonRpcMessage::Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "ping".into(),
            params: json!({}),
        };
        let response = router.route(msg);
        assert!(response.is_some());
    }

    #[test]
    fn test_route_unknown() {
        let router = make_router();
        let msg = JsonRpcMessage::Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "nonexistent".into(),
            params: json!({}),
        };
        let response = router.route(msg);
        match response.unwrap() {
            JsonRpcMessage::Response { error: Some(e), .. } => {
                assert_eq!(e.code, -32601);
            }
            _ => panic!("expected error"),
        }
    }
}
