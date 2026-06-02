use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::neotrix::nt_core_error::{NeoTrixResult, NeoTrixError};

/// State of a remote connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState {
    Pending,
    Authenticated,
    Active,
    Idle,
    Disconnected,
}

/// Statistics for a single connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub commands_received: u64,
    pub commands_sent: u64,
    pub last_activity: String,
    pub connected_at: String,
    pub state: ConnectionState,
}

/// A tracked connection
#[derive(Debug, Clone)]
pub struct Connection {
    pub id: String,
    pub peer_addr: String,
    pub state: ConnectionState,
    pub commands_received: u64,
    pub commands_sent: u64,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub sender: Option<mpsc::UnboundedSender<String>>,
}

impl Connection {
    pub fn new(id: String, peer_addr: String) -> Self {
        Self {
            id,
            peer_addr,
            state: ConnectionState::Pending,
            commands_received: 0,
            commands_sent: 0,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            sender: None,
        }
    }

    pub fn stats(&self) -> ConnectionStats {
        ConnectionStats {
            commands_received: self.commands_received,
            commands_sent: self.commands_sent,
            last_activity: format!("{:?}", self.last_activity.elapsed()),
            connected_at: format!("{:?}", self.connected_at.elapsed()),
            state: self.state.clone(),
        }
    }
}

/// Manages active connections with rate limiting and heartbeat
pub struct ConnectionManager {
    connections: Arc<Mutex<HashMap<String, Connection>>>,
    rate_limit_per_minute: usize,
    max_payload_size: usize,
    command_whitelist: Option<Vec<String>>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            rate_limit_per_minute: 60,
            max_payload_size: 64 * 1024,
            command_whitelist: None,
        }
    }

    pub fn with_rate_limit(mut self, limit: usize) -> Self {
        self.rate_limit_per_minute = limit;
        self
    }

    pub fn with_max_payload(mut self, bytes: usize) -> Self {
        self.max_payload_size = bytes;
        self
    }

    pub fn with_command_whitelist(mut self, commands: Vec<String>) -> Self {
        self.command_whitelist = Some(commands);
        self
    }

    pub fn rate_limit_per_minute(&self) -> usize {
        self.rate_limit_per_minute
    }

    pub fn max_payload_size(&self) -> usize {
        self.max_payload_size
    }

    pub fn command_whitelist(&self) -> Option<&Vec<String>> {
        self.command_whitelist.as_ref()
    }

    pub fn register(&self, id: String, peer_addr: String) -> String {
        let mut map = self.connections.lock().expect("lock");
        let conn = Connection::new(id.clone(), peer_addr);
        map.insert(id.clone(), conn);
        id
    }

    pub fn remove(&self, id: &str) {
        let mut map = self.connections.lock().expect("lock");
        map.remove(id);
    }

    pub fn get(&self, id: &str) -> Option<Connection> {
        let map = self.connections.lock().expect("lock");
        map.get(id).cloned()
    }

    pub fn update_state(&self, id: &str, state: ConnectionState) {
        if let Some(conn) = self.connections.lock().expect("lock").get_mut(id) {
            conn.state = state;
            conn.last_activity = Instant::now();
        }
    }

    pub fn record_command(&self, id: &str) {
        if let Some(conn) = self.connections.lock().expect("lock").get_mut(id) {
            conn.commands_received += 1;
            conn.last_activity = Instant::now();
        }
    }

    pub fn set_sender(&self, id: &str, sender: mpsc::UnboundedSender<String>) {
        if let Some(conn) = self.connections.lock().expect("lock").get_mut(id) {
            conn.sender = Some(sender);
            conn.state = ConnectionState::Authenticated;
        }
    }

    pub fn active_count(&self) -> usize {
        let map = self.connections.lock().expect("lock");
        map.values().filter(|c| c.state != ConnectionState::Disconnected).count()
    }

    pub fn list_connections(&self) -> Vec<Connection> {
        let map = self.connections.lock().expect("lock");
        map.values().cloned().collect()
    }

    pub fn list_stats(&self) -> Vec<(String, ConnectionStats)> {
        let map = self.connections.lock().expect("lock");
        map.iter().map(|(k, v)| (k.clone(), v.stats())).collect()
    }

    pub fn check_rate_limit(&self, id: &str) -> bool {
        if let Some(conn) = self.connections.lock().expect("lock").get(id) {
            let elapsed_secs = conn.last_activity.elapsed().as_secs_f64();
            let allowed_interval = 60.0 / self.rate_limit_per_minute as f64;
            elapsed_secs >= allowed_interval
        } else {
            true
        }
    }

    pub fn validate_payload(&self, data: &[u8]) -> NeoTrixResult<()> {
        if data.len() > self.max_payload_size {
            return Err(NeoTrixError::Network(format!(
                "Payload too large: {} bytes (max {})", data.len(), self.max_payload_size
            )));
        }
        Ok(())
    }

    pub fn validate_command(&self, command: &str) -> bool {
        match &self.command_whitelist {
            None => true,
            Some(whitelist) => whitelist.iter().any(|c| command.starts_with(c)),
        }
    }

    /// Start heartbeat monitor that disconnects stale connections
    pub fn start_heartbeat_monitor(manager: Arc<ConnectionManager>, interval_secs: u64, timeout_secs: u64) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            loop {
                ticker.tick().await;
                let stale_ids: Vec<String> = {
                    let map = manager.connections.lock().expect("lock");
                    map.iter()
                        .filter(|(_, c)| {
                            c.state != ConnectionState::Disconnected
                                && c.last_activity.elapsed() > Duration::from_secs(timeout_secs)
                        })
                        .map(|(k, _)| k.clone())
                        .collect()
                };
                for id in stale_ids {
                    log::info!("[remote-control] heartbeat timeout for connection {}", id);
                    manager.update_state(&id, ConnectionState::Disconnected);
                }
            }
        });
    }
}
