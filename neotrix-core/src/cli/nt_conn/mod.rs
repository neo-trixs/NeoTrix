//! Stub — deleted module, kept for CLI compilation only.
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ConnectorKind { GitHub, Slack, Webhook, Api, Database, Custom(String) }

impl Default for ConnectorKind {
    fn default() -> Self { ConnectorKind::GitHub }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ConnectorConfig {
    GitHub { webhook_secret: String, repos: Vec<String>, events: Vec<String> },
    Slack { token: String, channels: Vec<String> },
    Webhook { url: String, method: String, headers: HashMap<String, String> },
    Api { endpoint: String, api_key: Option<String> },
    Database { url: String },
}

#[derive(Clone, Debug, Default)]
pub struct ConnectorEntry {
    pub id: String,
    pub kind: ConnectorKind,
    pub name: String,
    pub enabled: bool,
    pub event_count: u64,
    pub last_event: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct ConnectorManager {
    entries: Vec<ConnectorEntry>,
    server_running: bool,
    server_port: u16,
}

impl ConnectorManager {
    pub fn load() -> Self {
        Self { entries: Vec::new(), server_running: false, server_port: 9090 }
    }
    pub fn list_connectors(&self) -> &[ConnectorEntry] { &self.entries }
    pub fn add_connector(&mut self, name: &str, kind: ConnectorKind, _cfg: ConnectorConfig) -> String {
        let id = format!("conn-{}-{}", name, self.entries.len());
        self.entries.push(ConnectorEntry {
            id: id.clone(), kind, name: name.to_string(), enabled: true, event_count: 0, last_event: None,
        });
        id
    }
    pub fn remove_connector(&mut self, id: &str) -> Result<(), String> {
        self.entries.retain(|e| e.id != id); Ok(())
    }
    pub fn enable_connector(&mut self, id: &str) -> Result<(), String> {
        if let Some(e) = self.entries.iter_mut().find(|e| e.id == id) { e.enabled = true; } Ok(())
    }
    pub fn disable_connector(&mut self, id: &str) -> Result<(), String> {
        if let Some(e) = self.entries.iter_mut().find(|e| e.id == id) { e.enabled = false; } Ok(())
    }
    pub fn save(&self) -> Result<(), String> { Ok(()) }
    pub fn start_server(&mut self) -> Result<(), String> { self.server_running = true; Ok(()) }
    pub fn stop_server(&mut self) -> Result<(), String> { self.server_running = false; Ok(()) }
    pub fn server_running(&self) -> bool { self.server_running }
    pub fn server_port(&self) -> u16 { self.server_port }
}

lazy_static! {
    pub static ref CONNECTOR_MANAGER: Mutex<ConnectorManager> = Mutex::new(ConnectorManager::load());
}
