use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// WebRTC ICE server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

/// WebRTC session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcConfig {
    pub ice_servers: Vec<IceServer>,
    pub ice_transport_policy: String,
    pub audio_codec: String,
    pub turn_port: u16,
    pub stun_only: bool,
}

impl Default for WebRtcConfig {
    fn default() -> Self {
        Self {
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".into()],
                username: None,
                credential: None,
            }],
            ice_transport_policy: "all".into(),
            audio_codec: "opus".into(),
            turn_port: 8443,
            stun_only: true,
        }
    }
}

/// Represents an active WebRTC peer connection
#[derive(Debug, Clone)]
pub struct PeerConnection {
    pub session_id: String,
    pub peer_id: String,
    pub connected_at: u64,
    pub audio_enabled: bool,
    pub ice_state: String,
}

/// WebRTC session manager
pub struct WebRtcManager {
    pub config: WebRtcConfig,
    pub active_peers: Arc<RwLock<HashMap<String, PeerConnection>>>,
    pub turn_credentials: Arc<RwLock<HashMap<String, (String, u64)>>>,
}

impl WebRtcManager {
    pub fn new(config: WebRtcConfig) -> Self {
        Self {
            config,
            active_peers: Arc::new(RwLock::new(HashMap::new())),
            turn_credentials: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate TURN credentials for a peer
    pub fn generate_turn_credentials(&self, peer_id: &str, ttl_secs: u64) -> (String, String) {
        let username = format!("{}:{}", peer_id, Uuid::new_v4());
        let credential = Uuid::new_v4().to_string();
        let _expiry = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + ttl_secs;
        (username, credential)
    }

    /// Register a new peer connection
    pub async fn register_peer(&self, peer_id: &str) -> PeerConnection {
        let session_id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let peer = PeerConnection {
            session_id: session_id.clone(),
            peer_id: peer_id.to_string(),
            connected_at: now,
            audio_enabled: true,
            ice_state: "new".into(),
        };
        self.active_peers
            .write()
            .await
            .insert(session_id.clone(), peer.clone());
        peer
    }

    /// Remove a peer connection
    pub async fn remove_peer(&self, session_id: &str) {
        self.active_peers.write().await.remove(session_id);
    }

    /// Get ICE configuration for client-side initialization
    pub fn ice_servers_config(&self) -> Vec<IceServer> {
        self.config.ice_servers.clone()
    }

    /// Count active sessions
    pub async fn active_count(&self) -> usize {
        self.active_peers.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WebRtcConfig::default();
        assert!(!config.ice_servers.is_empty());
        assert_eq!(
            config.ice_servers[0].urls[0],
            "stun:stun.l.google.com:19302"
        );
        assert_eq!(config.turn_port, 8443);
    }

    #[test]
    fn test_turn_credentials() {
        let manager = WebRtcManager::new(WebRtcConfig::default());
        let (user, cred) = manager.generate_turn_credentials("test-peer", 3600);
        assert!(user.contains("test-peer"));
        assert!(!cred.is_empty());
    }

    #[tokio::test]
    async fn test_peer_registration() {
        let manager = WebRtcManager::new(WebRtcConfig::default());
        let peer = manager.register_peer("alice").await;
        assert_eq!(peer.peer_id, "alice");
        assert_eq!(manager.active_count().await, 1);
        manager.remove_peer(&peer.session_id).await;
        assert_eq!(manager.active_count().await, 0);
    }

    #[test]
    fn test_ice_servers_config() {
        let manager = WebRtcManager::new(WebRtcConfig::default());
        let servers = manager.ice_servers_config();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].urls[0], "stun:stun.l.google.com:19302");
    }
}
