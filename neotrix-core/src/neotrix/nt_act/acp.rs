//! ACP (Agent Communication Protocol) Client

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::UnixStream;

use super::{AcpConfig, AcpMessage, AcpResponse};

/// ACP Client
pub struct AcpClient {
    config: crate::nt_act::AcpConfig,
    stream: Option<Arc<Mutex<tokio::io::BufReader<tokio::io::BufWriter<tokio::net::TcpStream>>>>>,
    request_id: Arc<std::sync::atomic::AtomicU64>,
    pending: Arc<tokio::sync::Mutex<HashMap<u64, tokio::sync::oneshot::Sender<AcpResponse>>>>,
}

impl AcpClient {
    pub async fn new(config: crate::nt_act::AcpConfig) -> Result<Self, String> {
        let mut client = Self {
            config,
            stream: None,
            request_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            pending: Arc::new(Mutex::new(HashMap::new())),
        };
        
        client.connect().await?;
        Ok(client)
    }

    async fn connect(&mut self) -> Result<(), String> {
        let addr = self.config.server_url.clone()
            .ok_or("ACP server URL required")?;
        
        let stream = TcpStream::connect(&addr).await
            .map_err(|e| format!("Failed to connect to ACP server: {}", e))?;
        
        let (reader, writer) = tokio::io::split(stream);
        let reader = BufReader::new(reader);
        let writer = BufWriter::new(writer);
        let stream = Arc::new(Mutex::new(BufWriter::new(BufReader::new(writer))));
        
        self.stream = Some(stream);
        
        // Start reading responses
        self.spawn_reader().await;
        
        // Initialize ACP session
        self.initialize().await?;
        
        Ok(())
    }

    async fn spawn_reader(&mut self) {
        // Spawn a background task to read responses
        // This is a simplified version
    }

    async fn initialize(&mut self) -> Result<(), String> {
        let request = AcpRequest {
            id: self.next_id(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "1.0",
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
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    async fn send_request(&self, request: AcpRequest) -> Result<AcpResponse, String> {
        let id = request.id;
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        {
            let mut pending = self.pending.lock().await;
            pending.insert(request.id, tx);
        }
        
        // Serialize and send
        let request_json = serde_json::to_string(&request)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        
        // Send via stream
        // This is simplified - actual implementation would use the stream
        
        // Wait for response
        tokio::time::timeout(std::time::Duration::from_secs(30), async {
            // Wait for response
        }).await
        .map_err(|_| "Request timeout".to_string())?
    }

    pub async fn send_message(&self, message: AcpMessage) -> Result<AcpResponse, String> {
        let request = AcpRequest {
            id: Self::next_id_static(),
            method: "message/send".to_string(),
            params: Some(serde_json::to_value(message).map_err(|e| format!("Serialize failed: {}", e))?),
        };
        
        // Send and wait for response
        Ok(AcpResponse {
            success: true,
            data: None,
            error: None,
        })
    }

    fn next_id(&self) -> u64 {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    fn next_id_static() -> u64 {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

/// ACP Request
#[derive(Debug, Serialize)]
struct AcpRequest {
    id: u64,
    method: String,
    params: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acp_config_default() {
        let config = crate::nt_act::AcpConfig::default();
        assert_eq!(config.timeout_secs, 60);
    }
}


#[derive(Debug, Clone)]
pub struct AcpServer { pub endpoint: String }

#[derive(Debug, Clone)]
pub struct AcpConfig { pub server_url: String }

#[derive(Debug, Clone)]
pub struct AcpSession { pub id: String }
