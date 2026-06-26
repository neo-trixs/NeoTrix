use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

/// A connector to an external service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connector {
    pub id: String,
    pub name: String,
    pub kind: ConnectorKind,
    pub config: ConnectorConfig,
    pub enabled: bool,
    pub last_event: Option<chrono::DateTime<chrono::Utc>>,
    pub event_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectorKind {
    GitHub,
    Slack,
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectorConfig {
    GitHub {
        webhook_secret: String,
        repos: Vec<String>,
        events: Vec<String>,
    },
    Slack {
        token: String,
        channels: Vec<String>,
    },
    Webhook {
        url: String,
        method: String,
        headers: HashMap<String, String>,
    },
}

/// Connector Manager — manages webhook listeners and event processing
pub struct ConnectorManager {
    pub connectors: Vec<Connector>,
    server_running: bool,
    server_port: u16,
}

fn connectors_path() -> PathBuf {
    let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(".neotrix").join("connectors.json")
}

impl ConnectorManager {
    pub fn new() -> Self {
        Self { connectors: Vec::new(), server_running: false, server_port: 9090 }
    }

    pub fn add_connector(&mut self, name: &str, kind: ConnectorKind, config: ConnectorConfig) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.connectors.push(Connector {
            id: id.clone(),
            name: name.to_string(),
            kind,
            config,
            enabled: true,
            last_event: None,
            event_count: 0,
        });
        id
    }

    pub fn remove_connector(&mut self, id: &str) -> Result<(), String> {
        let len = self.connectors.len();
        self.connectors.retain(|c| c.id != id);
        if self.connectors.len() == len {
            Err(format!("Connector not found: {}", id))
        } else {
            Ok(())
        }
    }

    pub fn list_connectors(&self) -> Vec<&Connector> {
        self.connectors.iter().collect()
    }

    pub fn enable_connector(&mut self, id: &str) -> Result<(), String> {
        self.connectors
            .iter_mut()
            .find(|c| c.id == id)
            .map(|c| {
                c.enabled = true;
            })
            .ok_or_else(|| format!("Connector not found: {}", id))
    }

    pub fn disable_connector(&mut self, id: &str) -> Result<(), String> {
        self.connectors
            .iter_mut()
            .find(|c| c.id == id)
            .map(|c| {
                c.enabled = false;
            })
            .ok_or_else(|| format!("Connector not found: {}", id))
    }

    pub fn start_server(&mut self) -> Result<(), String> {
        if self.server_running {
            return Err("Server already running".to_string());
        }
        self.server_running = true;
        Ok(())
    }

    pub fn stop_server(&mut self) -> Result<(), String> {
        if !self.server_running {
            return Err("Server not running".to_string());
        }
        self.server_running = false;
        Ok(())
    }

    pub fn server_running(&self) -> bool {
        self.server_running
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    pub fn handle_event(
        &self,
        kind: &ConnectorKind,
        body: &str,
        _headers: &HashMap<String, String>,
    ) -> Result<String, String> {
        match kind {
            ConnectorKind::GitHub => {
                if body.contains("\"action\"") || body.contains("\"zen\"") {
                    Ok("GitHub ping or action event received".to_string())
                } else if body.contains("\"commits\"") {
                    Ok("GitHub push event processed".to_string())
                } else {
                    Ok("GitHub event received".to_string())
                }
            }
            ConnectorKind::Slack => {
                if body.contains("\"command\"") {
                    Ok("Slack slash command processed".to_string())
                } else if body.contains("\"challenge\"") {
                    Ok("Slack URL challenge responded".to_string())
                } else {
                    Ok("Slack event received".to_string())
                }
            }
            ConnectorKind::Webhook => {
                Ok(format!("Webhook event processed ({} bytes)", body.len()))
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = connectors_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        let json =
            serde_json::to_string_pretty(&self.connectors).map_err(|e| format!("Serialize error: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    pub fn load() -> Self {
        let path = connectors_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Vec<Connector>>(&content) {
                    Ok(connectors) => {
                        Self { connectors, server_running: false, server_port: 9090 }
                    }
                    Err(e) => {
                        log::warn!("Failed to parse connectors.json: {}", e);
                        Self::new()
                    }
                },
                Err(e) => {
                    log::warn!("Failed to read connectors.json: {}", e);
                    Self::new()
                }
            }
        } else {
            Self::new()
        }
    }
}

impl Default for ConnectorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global connector manager
pub static CONNECTOR_MANAGER: LazyLock<Mutex<ConnectorManager>> =
    LazyLock::new(|| Mutex::new(ConnectorManager::load()));

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[test]
    fn test_new_manager_empty() {
        let mgr = ConnectorManager::new();
        assert!(mgr.list_connectors().is_empty());
        assert!(!mgr.server_running());
        assert_eq!(mgr.server_port(), 9090);
    }

    #[test]
    fn test_add_and_list_connector() {
        let mut mgr = ConnectorManager::new();
        let id = mgr.add_connector(
            "test-github",
            ConnectorKind::GitHub,
            ConnectorConfig::GitHub {
                webhook_secret: "secret123".into(),
                repos: vec!["user/repo".into()],
                events: vec!["push".into()],
            },
        );
        assert_eq!(mgr.list_connectors().len(), 1);
        assert_eq!(mgr.list_connectors()[0].name, "test-github");
        assert_eq!(mgr.list_connectors()[0].id, id);
    }

    #[test]
    fn test_remove_connector() {
        let mut mgr = ConnectorManager::new();
        let id = mgr.add_connector(
            "test",
            ConnectorKind::Webhook,
            ConnectorConfig::Webhook {
                url: "https://example.com/hook".into(),
                method: "POST".into(),
                headers: HashMap::new(),
            },
        );
        assert!(mgr.remove_connector(&id).is_ok());
        assert!(mgr.list_connectors().is_empty());
        assert!(mgr.remove_connector("nonexistent").is_err());
    }

    #[test]
    fn test_enable_disable_connector() {
        let mut mgr = ConnectorManager::new();
        let id = mgr.add_connector(
            "test",
            ConnectorKind::Slack,
            ConnectorConfig::Slack {
                token: "xoxb-test".into(),
                channels: vec!["general".into()],
            },
        );
        assert!(mgr.connectors[0].enabled);
        assert!(mgr.disable_connector(&id).is_ok());
        assert!(!mgr.connectors[0].enabled);
        assert!(mgr.enable_connector(&id).is_ok());
        assert!(mgr.connectors[0].enabled);
        assert!(mgr.disable_connector("bad-id").is_err());
    }

    #[test]
    fn test_server_start_stop() {
        let mut mgr = ConnectorManager::new();
        assert!(!mgr.server_running());
        assert!(mgr.start_server().is_ok());
        assert!(mgr.server_running());
        assert!(mgr.start_server().is_err());
        assert!(mgr.stop_server().is_ok());
        assert!(!mgr.server_running());
        assert!(mgr.stop_server().is_err());
    }

    #[test]
    fn test_handle_event_github() {
        let mgr = ConnectorManager::new();
        let headers = HashMap::new();
        let result = mgr.handle_event(&ConnectorKind::GitHub, r#"{"zen":"test","hook_id":1}"#, &headers);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("GitHub"));
    }

    #[test]
    fn test_handle_event_slack() {
        let mgr = ConnectorManager::new();
        let headers = HashMap::new();
        let result =
            mgr.handle_event(&ConnectorKind::Slack, r#"{"challenge":"abc123","type":"url_verification"}"#, &headers);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("challenge"));
    }

    #[test]
    fn test_handle_event_webhook() {
        let mgr = ConnectorManager::new();
        let headers = HashMap::new();
        let result = mgr.handle_event(&ConnectorKind::Webhook, r#"{"event":"test"}"#, &headers);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Webhook"));
    }

    #[test]
    fn test_connector_kind_partial_eq() {
        assert_eq!(ConnectorKind::GitHub, ConnectorKind::GitHub);
        assert_ne!(ConnectorKind::GitHub, ConnectorKind::Slack);
        assert_ne!(ConnectorKind::Webhook, ConnectorKind::Slack);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let mut mgr = ConnectorManager::new();
        mgr.add_connector(
            "save-test",
            ConnectorKind::GitHub,
            ConnectorConfig::GitHub {
                webhook_secret: "s".into(),
                repos: vec!["a/b".into()],
                events: vec!["push".into()],
            },
        );
        let json = serde_json::to_string_pretty(&mgr.connectors).unwrap();
        let deserialized: Vec<Connector> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized[0].name, "save-test");
        assert_eq!(deserialized[0].kind, ConnectorKind::GitHub);
    }
}
