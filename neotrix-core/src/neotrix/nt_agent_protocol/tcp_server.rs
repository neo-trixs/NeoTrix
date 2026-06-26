use crate::core::nt_core_shutdown::ShutdownSignal;
use crate::core::nt_core_util::A2A_METRICS_PORT;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

pub const AGENT_SERVER_PORT: u16 = A2A_METRICS_PORT;

/// Agent connection health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
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
    shutdown_signal: Option<ShutdownSignal>,
}

impl AgentServer {
    /// Create new server, trying port + up to 5 fallbacks if port is taken
    pub fn new(preferred_port: u16) -> Self {
        Self {
            port: preferred_port,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            shutdown_signal: None,
        }
    }

    /// Attach a shutdown signal for graceful termination
    pub fn with_shutdown_signal(mut self, signal: ShutdownSignal) -> Self {
        self.shutdown_signal = Some(signal);
        self
    }

    /// Start the TCP listener
    pub async fn start(self: Arc<Self>) -> NeoTrixResult<u16> {
        let mut port = self.port;
        let listener = loop {
            match TcpListener::bind(("0.0.0.0", port)).await {
                Ok(l) => break l,
                Err(_) if port < self.port + 5 => {
                    port += 1;
                }
                Err(e) => return Err(NeoTrixError::Network(format!("TCP bind failed: {}", e))),
            }
        };
        let actual_port = port;
        let sessions = self.sessions.clone();
        let shutdown = self.shutdown_signal.clone();

        // Accept loop
        tokio::spawn(async move {
            loop {
                if let Some(ref sig) = shutdown {
                    if sig.is_shutdown() {
                        log::info!("[agent-server] accept loop shutting down");
                        break;
                    }
                }
                match tokio::time::timeout(Duration::from_secs(1), listener.accept()).await {
                    Ok(Ok((stream, _addr))) => {
                        let sessions = sessions.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, sessions).await {
                                log::warn!("[agent-server] connection error: {}", e);
                            }
                        });
                    }
                    Ok(Err(e)) => {
                        log::error!("[agent-server] accept error: {}", e);
                        break;
                    }
                    Err(_) => {}
                }
            }
        });

        // Heartbeat monitor
        let sessions_heartbeat = self.sessions.clone();
        let shutdown_hb = self.shutdown_signal.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(15));
            loop {
                if let Some(ref sig) = shutdown_hb {
                    if sig.is_shutdown() {
                        log::info!("[agent-server] heartbeat loop shutting down");
                        break;
                    }
                }
                ticker.tick().await;
                let mut sessions = sessions_heartbeat.lock().await;
                let now = Instant::now();
                sessions.retain(|_, s| {
                    if s.status == AgentStatus::Online
                        && now.duration_since(s.last_heartbeat) > Duration::from_secs(90)
                    {
                        s.status = AgentStatus::Offline;
                        log::info!(
                            "[agent-server] agent {} went offline (heartbeat timeout)",
                            s.agent_id
                        );
                    }
                    s.status != AgentStatus::Offline
                });
            }
        });

        Ok(actual_port)
    }

    pub async fn connected_agents(&self) -> Vec<AgentSession> {
        self.sessions.lock().await.values().cloned().collect()
    }

    pub async fn agent_count(&self) -> usize {
        self.sessions.lock().await.len()
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
) -> NeoTrixResult<()> {
    let mut buf = vec![0u8; 4096];
    loop {
        let n = stream
            .read(&mut buf)
            .await
            .map_err(|e| NeoTrixError::Network(format!("read error: {}", e)))?;
        if n == 0 {
            break;
        }

        let msg = String::from_utf8_lossy(&buf[..n]);
        let trimmed = msg.trim();

        if trimmed == "HEARTBEAT" {
            stream
                .write_all(b"HEARTBEAT_ACK\n")
                .await
                .map_err(|e| NeoTrixError::Network(format!("write error: {}", e)))?;
        } else if let Ok(info) = serde_json::from_str::<super::discovery::AgentInfo>(trimmed) {
            let session = AgentSession {
                agent_id: info.id.clone(),
                info,
                status: AgentStatus::Online,
                last_heartbeat: Instant::now(),
                connected_since: Instant::now(),
            };
            sessions
                .lock()
                .await
                .insert(session.agent_id.clone(), session);
            stream
                .write_all(b"REGISTERED\n")
                .await
                .map_err(|e| NeoTrixError::Network(format!("write error: {}", e)))?;
        } else {
            stream
                .write_all(b"UNKNOWN\n")
                .await
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

    #[tokio::test]
    async fn test_agent_server_creation() {
        let server = AgentServer::new(AGENT_SERVER_PORT);
        assert_eq!(server.agent_count().await, 0);
        assert!(server.connected_agents().await.is_empty());
    }
}
