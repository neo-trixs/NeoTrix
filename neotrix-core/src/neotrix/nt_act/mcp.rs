//! MCP (Model Context Protocol) Client

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use std::process::Stdio;

use super::{McpConfig, McpTransport, McpTool, McpResource, McpPrompt, McpResult};

/// MCP Client
pub struct McpClient {
    config: crate::nt_act::McpConfig,
    process: Option<tokio::process::Child>,
    stdin: Option<Arc<tokio::sync::Mutex<BufWriter<tokio::process::ChildStdin>>>>,
    stdout: Option<Arc<Mutex<BufReader<tokio::process::ChildStdout>>>>,
    request_id: Arc<std::sync::atomic::AtomicU64>,
    pending: Arc<Mutex<HashMap<u64, tokio::sync::oneshot::Sender<serde_json::Value>>>>,
}

impl McpClient {
    pub async fn new(config: crate::nt_act::McpConfig) -> Result<Self, String> {
        let mut client = Self {
            config,
            process: None,
            stdin: None,
            stdout: None,
            request_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            pending: Arc::new(Mutex::new(HashMap::new())),
        };
        
        client.connect().await?;
        Ok(client)
    }

    async fn connect(&mut self) -> Result<(), String> {
        match self.config.transport {
            crate::nt_act::McpTransport::Stdio => self.connect_stdio().await,
            crate::nt_act::McpTransport::Http => self.connect_http().await,
            crate::nt_act::McpTransport::WebSocket => self.connect_ws().await,
            crate::nt_act::McpTransport::Sse => self.connect_sse().await,
        }
    }

    async fn connect_stdio(&mut self) -> Result<(), String> {
        let server_url = self.config.server_url.clone()
            .ok_or("MCP stdio transport requires server_url (command to execute)")?;
        
        let parts: Vec<&str> = server_url.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Invalid server command".to_string());
        }
        
        let mut cmd = Command::new(parts[0]);
        cmd.args(&parts[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        let mut child = Command::spawn(cmd)
            .map_err(|e| format!("Failed to spawn MCP server: {}", e))?;
        
        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        
        let stdin = Arc::new(Mutex::new(BufWriter::new(stdin)));
        let stdout = Arc::new(Mutex::new(BufReader::new(child.stdout.take().unwrap())));
        
        self.process = Some(child);
        self.stdin = Some(stdin);
        self.stdout = Some(stdout);
        
        // Start reading responses
        self.spawn_reader().await;
        
        // Initialize MCP session
        self.initialize().await?;
        
        Ok(())
    }

    async fn connect_http(&mut self) -> Result<(), String> {
        Err("HTTP transport not yet implemented".to_string())
    }

    async fn connect_ws(&mut self) -> Result<(), String> {
        Err("WebSocket transport not yet implemented".to_string())
    }

    async fn connect_sse(&mut self) -> Result<(), String> {
        Err("SSE transport not yet implemented".to_string())
    }

    async fn spawn_reader(&mut self) {
        let stdout = self.stdout.clone().unwrap();
        let pending = self.pending.clone();
        
        tokio::spawn(async move {
            let mut reader = stdout.lock().await;
            let mut line = String::new();
            
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                            if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
                                if let Some(sender) = pending.lock().await.remove(&id) {
                                    let _ = sender.send(msg);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("MCP read error: {}", e);
                        break;
                    }
                }
            }
        });
    }

    async fn initialize(&mut self) -> Result<(), String> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "neotrix",
                    "version": "0.19.0-rc1"
                }
            })),
        };
        
        let response = self.send_request(request).await?;
        // Process initialize response
        Ok(())
    }

    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    async fn send_request(&self, request: McpRequest) -> Result<serde_json::Value, String> {
        let id = request.id;
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        {
            let mut pending = self.pending.lock().await;
            pending.insert(request.id, tx);
        }
        
        let stdin = self.stdin.as_ref().ok_or("Not connected")?;
        let mut stdin = stdin.lock().await;
        
        let request_json = serde_json::to_string(&request)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        
        stdin.write_all(request_json.as_bytes()).await
            .map_err(|e| format!("Write failed: {}", e))?;
        stdin.write_all(b"\n").await
            .map_err(|e| format!("Write newline failed: {}", e))?;
        stdin.flush().await
            .map_err(|e| format!("Flush failed: {}", e))?;
        
        // Wait for response with timeout
        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| "Request timeout".to_string())?
            .map_err(|_| "Channel closed".to_string())
    }

    /// Call a tool on the MCP server
    pub async fn call_tool(&self, tool_name: &str, args: Option<serde_json::Value>) -> Result<McpResult, String> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": tool_name,
                "arguments": args.unwrap_or(serde_json::json!({}))
            })),
        };
        
        let response = self.send_request(request).await?;
        
        if let Some(error) = response.get("error") {
            return Err(format!("MCP error: {}", error));
        }
        
        Ok(McpResult {
            content: response.get("result")
                .and_then(|r| r.get("content"))
                .cloned()
                .unwrap_or(serde_json::json!([])),
            is_error: false,
        })
    }

    /// List available tools
    pub async fn list_tools(&self) -> Result<Vec<McpTool>, String> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id(),
            method: "tools/list".to_string(),
            params: None,
        };
        
        let response = self.send_request(request).await?;
        
        if let Some(result) = response.get("result") {
            serde_json::from_value(result.clone())
                .map_err(|e| format!("Failed to parse tools: {}", e))
        } else {
            Err("No result in response".to_string())
        }
    }

    /// List resources
    pub async fn list_resources(&self) -> Result<Vec<McpResource>, String> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id(),
            method: "resources/list".to_string(),
            params: None,
        };
        
        let response = self.send_request(request).await?;
        
        if let Some(result) = response.get("result") {
            serde_json::from_value(result.clone())
                .map_err(|e| format!("Failed to parse resources: {}", e))
        } else {
            Err("No result in response".to_string())
        }
    }
}

/// MCP Request
#[derive(Debug, Serialize)]
struct McpRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_default() {
        let config = crate::nt_act::McpConfig::default();
        assert_eq!(config.transport, crate::nt_act::McpTransport::Stdio);
    }
}


#[derive(Debug, Clone)]
pub struct McpServer {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct McpConfig {
    pub server_url: String,
}
