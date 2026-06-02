use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use serde::{Serialize, Deserialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{interval, Duration};
use crate::neotrix::nt_core_error::{NeoTrixResult, NeoTrixError};

/// Agent connection health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Online,
    Offline,
    Unknown,
}

/// A connected agent session
#[derive(Debug, Clone)]
pub struct AgentSession {
    pub agent_id: String,
    pub info: super::discovery::AgentInfo,
    pub status: AgentStatus,
    pub last_heartbeat: Instant,
    pub connected_since: Instant,
}

/// TCP-based agent communication server
pub struct AgentServer {
    port: u16,
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
}

impl AgentServer {
    /// Create new server, trying port + up to 5 fallbacks if port is taken
    pub fn new(preferred_port: u16) -> Self {
        Self {
            port: preferred_port,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start the TCP listener
    pub async fn start(self: Arc<Self>) -> NeoTrixResult<u16> {
        let mut port = self.port;
        let listener = loop {
            match TcpListener::bind(("0.0.0.0", port)).await {
                Ok(l) => break l,
                Err(_) if port < self.port + 5 => { port += 1; }
                Err(e) => return Err(NeoTrixError::Network(format!("TCP bind failed: {}", e))),
            }
        };
        let actual_port = port;
        let sessions = self.sessions.clone();

        // Accept loop
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        let sessions = sessions.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, sessions).await {
                                log::warn!("[agent-server] connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("[agent-server] accept error: {}", e);
                        break;
                    }
                }
            }
        });

        // Heartbeat monitor
        let sessions_heartbeat = self.sessions.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(15));
            loop {
                ticker.tick().await;
                let mut sessions = sessions_heartbeat.lock().expect("result");
                let now = Instant::now();
                sessions.retain(|_, s| {
                    if s.status == AgentStatus::Online && now.duration_since(s.last_heartbeat) > Duration::from_secs(90) {
                        s.status = AgentStatus::Offline;
                        log::info!("[agent-server] agent {} went offline (heartbeat timeout)", s.agent_id);
                    }
                    true
                });
            }
        });

        Ok(actual_port)
    }

    pub fn connected_agents(&self) -> Vec<AgentSession> {
        self.sessions.lock().expect("result").values().cloned().collect()
    }

    pub fn agent_count(&self) -> usize {
        self.sessions.lock().expect("result").len()
    }
}

async fn handle_connection(mut stream: TcpStream, sessions: Arc<Mutex<HashMap<String, AgentSession>>>) -> NeoTrixResult<()> {
    let mut buf = vec![0u8; 4096];
    loop {
        let n = stream.read(&mut buf).await
            .map_err(|e| NeoTrixError::Network(format!("read error: {}", e)))?;
        if n == 0 { break; }

        let msg = String::from_utf8_lossy(&buf[..n]);
        let trimmed = msg.trim();

        if trimmed == "HEARTBEAT" {
            stream.write_all(b"HEARTBEAT_ACK\n").await
                .map_err(|e| NeoTrixError::Network(format!("write error: {}", e)))?;
        } else if let Ok(info) = serde_json::from_str::<super::discovery::AgentInfo>(trimmed) {
            let session = AgentSession {
                agent_id: info.id.clone(),
                info,
                status: AgentStatus::Online,
                last_heartbeat: Instant::now(),
                connected_since: Instant::now(),
            };
            sessions.lock().expect("result").insert(session.agent_id.clone(), session);
            stream.write_all(b"REGISTERED\n").await
                .map_err(|e| NeoTrixError::Network(format!("write error: {}", e)))?;
        } else {
            stream.write_all(b"UNKNOWN\n").await
                .map_err(|e| NeoTrixError::Network(format!("write error: {}", e)))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_defaults() {
        assert_eq!(AgentStatus::Online, AgentStatus::Online);
        assert_ne!(AgentStatus::Online, AgentStatus::Offline);
    }

    #[test]
    fn test_agent_server_creation() {
        let server = AgentServer::new(42070);
        assert_eq!(server.agent_count(), 0);
        assert!(server.connected_agents().is_empty());
    }
}
