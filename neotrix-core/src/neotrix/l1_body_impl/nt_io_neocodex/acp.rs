// ── ACP Server (Agent Client Protocol for editor integration) ──

use std::sync::Arc;

use tokio::sync::Mutex;

use super::agent::NeoCodexAgent;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpRequest {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpResponse {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AcpError>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpNotification {
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpError {
    pub code: i32,
    pub message: String,
}

pub struct AcpServer {
    pub agent: Arc<Mutex<NeoCodexAgent>>,
}

impl AcpServer {
    pub fn new(agent: Arc<Mutex<NeoCodexAgent>>) -> Self {
        Self { agent }
    }

    pub async fn handle_request(&self, req: AcpRequest) -> AcpResponse {
        let AcpRequest { id, method, params } = req;
        if !matches!(
            method.as_str(),
            "ping" | "agent/process" | "agent/status" | "agent/mode" | "tools/list" | "shutdown"
        ) {
            // Protocol errors belong in `error`, not smuggled into `result` —
            // clients keying on the error field otherwise see a success.
            return AcpResponse {
                id,
                result: None,
                error: Some(AcpError {
                    code: -32601,
                    message: format!("unknown method: {}", method),
                }),
            };
        }
        let result = self.dispatch(method, params).await;
        AcpResponse {
            id,
            result: Some(result),
            error: None,
        }
    }

    async fn dispatch(&self, method: String, params: serde_json::Value) -> serde_json::Value {
        match method.as_str() {
            "ping" => serde_json::json!({"pong": true}),
            "agent/process" => {
                let input = params.get("input").and_then(|v| v.as_str()).unwrap_or("");
                let mut agent = self.agent.lock().await;
                let response = agent.process(input).await;
                serde_json::json!({"response": response, "turn": agent.state.turn_count})
            }
            "agent/status" => {
                let agent = self.agent.lock().await;
                serde_json::json!({
                    "mode": agent.state.mode,
                    "turn": agent.state.turn_count,
                    "tools": agent.state.tool_call_count,
                    "tokens": agent.state.tokens_used,
                })
            }
            "agent/mode" => {
                let mode_name = params
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("agent");
                let mut agent = self.agent.lock().await;
                match mode_name {
                    "shell" => {
                        agent.toggle_mode();
                    }
                    "plan" => {
                        agent.set_plan_mode();
                    }
                    _ => {}
                }
                serde_json::json!({"mode": agent.state.mode})
            }
            "tools/list" => serde_json::json!({
                "tools": [
                    {"name": "read", "description": "Read files"},
                    {"name": "search", "description": "Search codebase"},
                    {"name": "shell", "description": "Execute shell commands"},
                    {"name": "edit", "description": "Replace a unique old substring with new. Args: <path>|<old>|<new>"},
                    {"name": "write", "description": "Write or overwrite a file. Args: <path>|<content>"},
                    {"name": "plan", "description": "Create/edit plans"},
                ]
            }),
            "shutdown" => {
                serde_json::json!({"shutdown": true})
            }
            _ => serde_json::json!({"error": format!("unknown method: {}", method)}),
        }
    }

    /// Run ACP server over stdio (JSON-RPC 2.0 line protocol)
    pub async fn run_stdio(&self) {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        loop {
            let mut line = String::new();
            if stdin.read_line(&mut line).is_err() || line.trim().is_empty() {
                break;
            }
            if let Ok(req) = serde_json::from_str::<AcpRequest>(&line) {
                let resp = self.handle_request(req).await;
                if let Ok(json) = serde_json::to_string(&resp) {
                    use std::io::Write;
                    let mut out = stdout.lock();
                    let _ = writeln!(out, "{}", json);
                    let _ = out.flush();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::stream::AgentStream;
    use super::*;

    #[test]
    fn test_acp_ping() {
        let agent = NeoCodexAgent::new("acp-test");
        let stream = AgentStream::new(agent, 10.0);
        let server = AcpServer::new(stream.agent.clone());
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let req = AcpRequest {
                id: 1,
                method: "ping".into(),
                params: serde_json::json!({}),
            };
            let resp = server.handle_request(req).await;
            if let Some(result) = resp.result {
                assert_eq!(result.get("pong").and_then(|v| v.as_bool()), Some(true));
            } else {
                panic!("expected response, got error: {:?}", resp.error);
            }
        });
    }
}
