use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub transport: McpTransport,
    pub tools: Vec<McpToolInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McpTransport {
    Stdio { command: String, args: Vec<String> },
    Http { url: String },
    Ws { url: String },
    Sse { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub tool: String,
    pub content: serde_json::Value,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarPulseMessage {
    pub vector: Vec<f64>,
    pub metadata: HashMap<String, String>,
    pub confidence: f64,
    pub source: String,
}

pub trait McpBridgeAdapter: Send + Sync {
    fn ingest_server(&mut self, server: McpServerInfo);
    fn mcp_to_vsa(&self, response: &McpResponse) -> StarPulseMessage;
    fn vsa_to_mcp_request(&self, message: &StarPulseMessage, tool_name: &str) -> serde_json::Value;
    fn list_servers(&self) -> &[BridgeServerState];
    fn list_tools(&self) -> Vec<McpToolInfo>;
}

#[derive(Debug, Clone)]
pub struct BridgeServerState {
    pub server: McpServerInfo,
    pub is_healthy: bool,
    pub translation_confidence: f64,
}

pub struct StarPulseMcpBridge {
    servers: Vec<BridgeServerState>,
}

impl StarPulseMcpBridge {
    pub fn new() -> Self {
        Self { servers: Vec::new() }
    }

    pub fn register_server(&mut self, server: McpServerInfo) {
        let confidence = match &server.transport {
            McpTransport::Stdio { .. } => 0.90,
            McpTransport::Http { .. } => 0.85,
            McpTransport::Ws { .. } => 0.80,
            McpTransport::Sse { .. } => 0.75,
        };
        self.servers.push(BridgeServerState {
            server,
            is_healthy: true,
            translation_confidence: confidence,
        });
    }

    pub fn mcp_to_vsa(&self, response: &McpResponse) -> StarPulseMessage {
        let content_str = match &response.content {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let mut vector = vec![0.0; 23];
        let lower = content_str.to_lowercase();
        if lower.contains("image") || lower.contains("png") || lower.contains("jpg") { vector[4] = 0.8; }
        if lower.contains("audio") || lower.contains("wav") || lower.contains("mp3") { vector[5] = 0.8; }
        if lower.contains("code") || lower.contains("function") || lower.contains("class") { vector[1] = 0.8; vector[8] = 0.7; }
        if lower.contains("data") || lower.contains("result") || lower.contains("output") { vector[11] = 0.7; }
        if lower.contains("error") || lower.contains("fail") { vector[10] = 0.3; }
        let mut metadata = HashMap::new();
        metadata.insert("tool".into(), response.tool.clone());
        metadata.insert("vsa_generated".into(), "mcp_bridge".into());
        StarPulseMessage {
            vector,
            metadata,
            confidence: response.confidence * self.default_translation_confidence(),
            source: format!("mcp:{}", response.tool),
        }
    }

    pub fn vsa_to_jsonrpc(&self, message: &StarPulseMessage, tool_name: &str) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": {
                    "query_vector": message.vector,
                    "source": message.source,
                    "metadata": message.metadata,
                }
            },
            "id": 1,
        })
    }

    pub fn default_translation_confidence(&self) -> f64 {
        if self.servers.is_empty() {
            return 0.5;
        }
        self.servers.iter().map(|s| s.translation_confidence).sum::<f64>() / self.servers.len() as f64
    }

    pub fn server_count(&self) -> usize {
        self.servers.len()
    }

    pub fn health_check(&self, server_name: &str) -> Option<bool> {
        self.servers.iter()
            .find(|s| s.server.name == server_name)
            .map(|s| s.is_healthy)
    }

    pub fn all_tools(&self) -> Vec<McpToolInfo> {
        self.servers.iter()
            .flat_map(|s| s.server.tools.clone())
            .collect()
    }
}

impl Default for StarPulseMcpBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mcp_tool(name: &str) -> McpToolInfo {
        McpToolInfo {
            name: name.into(),
            description: format!("Tool {} for testing", name),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        }
    }

    #[test]
    fn test_bridge_new() {
        let bridge = StarPulseMcpBridge::new();
        assert_eq!(bridge.server_count(), 0);
    }

    #[test]
    fn test_register_server() {
        let mut bridge = StarPulseMcpBridge::new();
        bridge.register_server(McpServerInfo {
            name: "test-server".into(),
            transport: McpTransport::Stdio { command: "tool".into(), args: vec![] },
            tools: vec![sample_mcp_tool("test_tool")],
        });
        assert_eq!(bridge.server_count(), 1);
        assert_eq!(bridge.all_tools().len(), 1);
    }

    #[test]
    fn test_mcp_to_vsa_image_response() {
        let bridge = StarPulseMcpBridge::new();
        let response = McpResponse {
            tool: "txt2img".into(),
            content: serde_json::json!({"data": "image/png binary data here"}),
            confidence: 0.9,
        };
        let msg = bridge.mcp_to_vsa(&response);
        assert!(msg.vector[4] > 0.5);
        assert!(msg.confidence > 0.0);
        assert_eq!(msg.source, "mcp:txt2img");
    }

    #[test]
    fn test_mcp_to_vsa_code_response() {
        let bridge = StarPulseMcpBridge::new();
        let response = McpResponse {
            tool: "analyze".into(),
            content: serde_json::json!({"result": "function hello() { return 42; }"}),
            confidence: 0.95,
        };
        let msg = bridge.mcp_to_vsa(&response);
        assert!(msg.vector[1] > 0.5);
        assert!(msg.vector[8] > 0.5);
    }

    #[test]
    fn test_vsa_to_jsonrpc_format() {
        let bridge = StarPulseMcpBridge::new();
        let msg = StarPulseMessage {
            vector: vec![0.5; 23],
            metadata: HashMap::new(),
            confidence: 0.85,
            source: "e8_state_machine".into(),
        };
        let json = bridge.vsa_to_jsonrpc(&msg, "txt2img");
        assert_eq!(json["method"], "tools/call");
        assert_eq!(json["params"]["name"], "txt2img");
    }

    #[test]
    fn test_health_check_unknown_server() {
        let bridge = StarPulseMcpBridge::new();
        assert!(bridge.health_check("nonexistent").is_none());
    }

    #[test]
    fn test_multiple_servers_avg_confidence() {
        let mut bridge = StarPulseMcpBridge::new();
        bridge.register_server(McpServerInfo {
            name: "s1".into(), transport: McpTransport::Stdio { command: "c".into(), args: vec![] },
            tools: vec![sample_mcp_tool("t1")],
        });
        bridge.register_server(McpServerInfo {
            name: "s2".into(), transport: McpTransport::Http { url: "http://localhost".into() },
            tools: vec![sample_mcp_tool("t2")],
        });
        let avg = bridge.default_translation_confidence();
        assert!((avg - 0.875).abs() < 0.001);
    }

    #[test]
    fn test_default_confidence_empty() {
        let bridge = StarPulseMcpBridge::new();
        assert!((bridge.default_translation_confidence() - 0.5).abs() < 1e-10);
    }
}
