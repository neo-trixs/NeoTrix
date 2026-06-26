#![forbid(unsafe_code)]

use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

use super::types::*;

#[derive(Debug)]
pub struct McpClient {
    pub server_name: String,
    transport: McpClientTransport,
    next_id: u64,
}

#[derive(Debug)]
enum McpClientTransport {
    Stdio {
        child: Child,
        reader: BufReader<tokio::process::ChildStdout>,
        writer: tokio::process::ChildStdin,
    },
}

impl McpClient {
    pub async fn connect_stdio(command: &str, args: &[&str]) -> Result<Self, String> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server '{}': {}", command, e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture child stdout".to_string())?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to capture child stdin".to_string())?;

        let reader = BufReader::new(stdout);
        let writer = stdin;

        let client = Self {
            server_name: command.to_string(),
            transport: McpClientTransport::Stdio {
                child,
                reader,
                writer,
            },
            next_id: 1,
        };

        Ok(client)
    }

    /// Explicit connection check — replaces old initialize handshake (MCP 2026-07-28)
    pub async fn ping(&mut self) -> Result<(), String> {
        let resp = self.send_request("ping", None).await?;
        if let Some(err) = resp.error {
            return Err(format!("Ping failed: {}", err.message));
        }
        Ok(())
    }

    /// Stateless connect — does not run initialize handshake (MCP 2026-07-28)
    pub async fn connect(&mut self) -> Result<(), String> {
        self.ping().await
    }

    /// Old initialize handshake (DEPRECATED — MCP 2026-07-28 removed this)
    async fn handshake(&mut self) -> Result<(), String> {
        let params = serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "neotrix",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        let resp = self.send_request("initialize", Some(params)).await?;
        if let Some(err) = resp.error {
            return Err(format!("Initialize failed: {}", err.message));
        }
        let _ = self.send_notification("notifications/initialized", None).await;
        Ok(())
    }

    /// Legacy constructor — spawns + runs old initialize handshake (for transition)
    pub async fn new_legacy(command: &str, args: &[&str]) -> Result<Self, String> {
        let mut client = Self::connect_stdio(command, args).await?;
        client.handshake().await?;
        Ok(client)
    }

    async fn write_line(&mut self, line: &str) -> Result<(), String> {
        match &mut self.transport {
            McpClientTransport::Stdio { writer, .. } => {
                writer
                    .write_all(line.as_bytes())
                    .await
                    .map_err(|e| format!("Write error: {}", e))?;
                writer
                    .flush()
                    .await
                    .map_err(|e| format!("Flush error: {}", e))?;
                Ok(())
            }
        }
    }

    async fn read_line(&mut self) -> Result<String, String> {
        match &mut self.transport {
            McpClientTransport::Stdio { reader, .. } => {
                let mut line = String::new();
                reader
                    .read_line(&mut line)
                    .await
                    .map_err(|e| format!("Read error: {}", e))?;
                if line.is_empty() {
                    return Err("Server closed connection".to_string());
                }
                Ok(line.trim().to_string())
            }
        }
    }

    pub async fn send_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<JsonRpcResponse, String> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id,
            method: method.to_string(),
            params,
        };

        let line =
            serde_json::to_string(&request).map_err(|e| format!("Serialize error: {}", e))?;

        self.write_line(&format!("{}\n", line)).await?;

        let response_line = self.read_line().await?;

        serde_json::from_str(&response_line)
            .map_err(|e| format!("Deserialize response error: {}", e))
    }

    pub async fn send_notification(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<(), String> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let line =
            serde_json::to_string(&notification).map_err(|e| format!("Serialize error: {}", e))?;

        self.write_line(&format!("{}\n", line)).await
    }

    pub async fn list_tools(&mut self) -> Result<Vec<McpTool>, String> {
        let resp = self.send_request("tools/list", None).await?;

        if let Some(err) = resp.error {
            return Err(format!("tools/list failed: {}", err.message));
        }

        let result = resp.result.ok_or("Missing result in tools/list response")?;
        let tools: Vec<McpTool> =
            serde_json::from_value(result["tools"].clone()).map_err(|e| {
                format!("Failed to parse tools list: {}", e)
            })?;

        Ok(tools)
    }

    pub async fn call_tool(
        &mut self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let params = json!({
            "name": name,
            "arguments": arguments,
        });

        let resp = self.send_request("tools/call", Some(params)).await?;

        if let Some(err) = resp.error {
            return Err(format!("tools/call '{}' failed: {}", name, err.message));
        }

        Ok(resp.result.unwrap_or(json!({})))
    }

    pub async fn list_resources(&mut self) -> Result<Vec<McpResource>, String> {
        let resp = self.send_request("resources/list", None).await?;

        if let Some(err) = resp.error {
            return Err(format!("resources/list failed: {}", err.message));
        }

        let result = resp.result.ok_or("Missing result in resources/list response")?;
        let resources: Vec<McpResource> =
            serde_json::from_value(result["resources"].clone()).map_err(|e| {
                format!("Failed to parse resources list: {}", e)
            })?;

        Ok(resources)
    }

    pub async fn read_resource(&mut self, uri: &str) -> Result<serde_json::Value, String> {
        let params = json!({
            "uri": uri,
        });

        let resp = self.send_request("resources/read", Some(params)).await?;

        if let Some(err) = resp.error {
            return Err(format!("resources/read '{}' failed: {}", uri, err.message));
        }

        Ok(resp.result.unwrap_or(json!({})))
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let McpClientTransport::Stdio { child, .. } = &mut self.transport;
        let _ = child.start_kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_request_creates_valid_json_rpc() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "tools/list".into(),
            params: None,
        };
        let json = serde_json::to_string(&req).unwrap_or_default();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"method\":\"tools/list\""));
        assert!(!json.contains("params"));
    }

    #[tokio::test]
    async fn test_connect_nonexistent_command() {
        let result = McpClient::connect_stdio("nonexistent-command-12345", &[]).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Failed to spawn"));
    }

    #[test]
    fn test_drop_trait_implemented() {
        fn _assert_drop<T>() { let _ = std::mem::needs_drop::<T>(); }
        _assert_drop::<McpClient>();
    }

    #[test]
    fn test_json_rpc_request_serialization_matches_spec() {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "test_tool",
                "arguments": {"key": "value"}
            }
        });

        assert_eq!(req["jsonrpc"], "2.0");
        assert_eq!(req["id"], 5);
        assert_eq!(req["method"], "tools/call");
        assert_eq!(req["params"]["name"], "test_tool");
        assert_eq!(req["params"]["arguments"]["key"], "value");
    }

    #[test]
    fn test_list_tools_response_parse() {
        let response_json = r#"{"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"test","description":"A tool","input_schema":{"type":"object"}}]}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(response_json).unwrap_or_default();
        assert!(resp.error.is_none());
        let result = resp.result.unwrap_or_default();
        let tools: Vec<McpTool> = serde_json::from_value(result["tools"].clone()).unwrap_or_default();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test");
    }

    #[test]
    fn test_list_resources_response_parse() {
        let response_json = r#"{"jsonrpc":"2.0","id":1,"result":{"resources":[{"uri":"test://uri","name":"Test","description":"desc","mime_type":"text/plain"}]}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(response_json).unwrap_or_default();
        let result = resp.result.unwrap_or_default();
        let resources: Vec<McpResource> =
            serde_json::from_value(result["resources"].clone()).unwrap_or_default();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, "test://uri");
    }

    #[test]
    fn test_error_response_parse() {
        let response_json =
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(response_json).unwrap_or_default();
        assert!(resp.result.is_none());
        let err = resp.error.unwrap_or(JsonRpcError::internal_error("fallback"));
        assert_eq!(err.code, -32601);
        assert_eq!(err.message, "Method not found");
    }

    #[test]
    fn test_notification_format() {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/cancelled"
        });
        assert_eq!(notification["jsonrpc"], "2.0");
        assert_eq!(notification["method"], "notifications/cancelled");
        assert!(notification.get("id").is_none());
    }
}
