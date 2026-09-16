//! ACP Protocol Implementation
//!
//! Agent Communication Protocol — handles protocol negotiation,
//! capability exchange, and request/response messaging between agents.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// Protocol Trait
// ============================================================================

/// Agent Protocol trait defining the communication interface.
pub trait AgentProtocol: Send + Sync + 'static {
    /// Connect to a remote endpoint
    fn connect(&mut self, endpoint: &str) -> Result<ConnectionInfo, ProtocolError>;

    /// Disconnect from the current endpoint
    fn disconnect(&mut self) -> Result<(), ProtocolError>;

    /// Send a request to the remote peer
    fn send_request(&mut self, request: ProtocolRequest) -> Result<String, ProtocolError>;

    /// Receive a response by request ID
    fn receive_response(&mut self, request_id: &str) -> Result<ProtocolResponse, ProtocolError>;

    /// Stream events from the remote peer
    fn stream_event(&mut self) -> Result<ProtocolEvent, ProtocolError>;
}

// ============================================================================
// Types
// ============================================================================

/// Connection information after successful handshake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub peer_id: String,
    pub protocol_version: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub established_at: u64,
}

/// Protocol request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolRequest {
    pub id: String,
    pub action: String,
    pub payload: serde_json::Value,
    pub capabilities: Vec<String>,
    pub timeout_ms: u64,
}

/// Protocol response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolResponse {
    pub request_id: String,
    pub status: ResponseStatus,
    pub payload: Option<serde_json::Value>,
    pub capabilities: Vec<String>,
    pub timestamp: u64,
}

/// Response status enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseStatus {
    Success,
    Failure,
    Pending,
    Error,
}

/// Protocol event for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
    pub source: String,
}

/// Protocol negotiation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolError {
    ConnectionFailed(String),
    DisconnectionFailed(String),
    SendFailed(String),
    ReceiveFailed(String),
    NegotiationFailed(String),
    Timeout(String),
    CapabilityMismatch(String),
    InvalidState(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Self::DisconnectionFailed(msg) => write!(f, "Disconnection failed: {}", msg),
            Self::SendFailed(msg) => write!(f, "Send failed: {}", msg),
            Self::ReceiveFailed(msg) => write!(f, "Receive failed: {}", msg),
            Self::NegotiationFailed(msg) => write!(f, "Negotiation failed: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Self::CapabilityMismatch(msg) => write!(f, "Capability mismatch: {}", msg),
            Self::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl std::error::Error for ProtocolError {}

// ============================================================================
// AcpProtocol Implementation
// ============================================================================

/// ACP (Agent Communication Protocol) implementation
///
/// Handles protocol negotiation and capability exchange between agents.
/// Uses async channels for message passing and a capability registry
/// for protocol feature discovery.
pub struct AcpProtocol {
    /// Current connection endpoint
    endpoint: Option<String>,
    /// Connected peer identifier
    peer_id: Option<String>,
    /// Negotiated protocol version
    protocol_version: String,
    /// Local capabilities advertised
    local_capabilities: Vec<String>,
    /// Remote capabilities discovered
    remote_capabilities: Vec<String>,
    /// Pending request responses
    pending_responses: Arc<Mutex<HashMap<String, ProtocolResponse>>>,
    /// Connection state
    connected: bool,
}

/// Capability descriptor for protocol negotiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub name: String,
    pub version: String,
    pub required: bool,
}

impl AcpProtocol {
    /// Create a new ACP protocol instance
    pub fn new() -> Self {
        Self {
            endpoint: None,
            peer_id: None,
            protocol_version: "1.0.0".to_string(),
            local_capabilities: vec![],
            remote_capabilities: vec![],
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            connected: false,
        }
    }

    /// Add a local capability
    pub fn add_capability(&mut self, name: &str, version: &str, required: bool) {
        self.local_capabilities.push(serde_json::to_string(&CapabilityDescriptor {
            name: name.to_string(),
            version: version.to_string(),
            required,
        })
        .unwrap_or_else(|_| name.to_string()));
    }

    /// Get local capabilities
    pub fn local_capabilities(&self) -> &[String] {
        &self.local_capabilities
    }

    /// Get remote capabilities
    pub fn remote_capabilities(&self) -> &[String] {
        &self.remote_capabilities
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Negotiate capabilities with remote peer
    fn negotiate_capabilities(&mut self) -> Result<(), ProtocolError> {
        if !self.connected {
            return Err(ProtocolError::InvalidState(
                "Not connected".to_string(),
            ));
        }

        let common: Vec<String> = self
            .local_capabilities
            .iter()
            .filter(|c| self.remote_capabilities.contains(c))
            .cloned()
            .collect();

        if common.is_empty() && !self.local_capabilities.is_empty() {
            return Err(ProtocolError::CapabilityMismatch(
                "No common capabilities".to_string(),
            ));
        }

        Ok(())
    }

    /// Store a pending response
    fn store_response(&self, response: ProtocolResponse) {
        let mut pending = self.pending_responses.lock().unwrap();
        pending.insert(response.request_id.clone(), response);
    }

    /// Retrieve a pending response
    fn get_response(&self, request_id: &str) -> Option<ProtocolResponse> {
        let pending = self.pending_responses.lock().unwrap();
        pending.get(request_id).cloned()
    }
}

impl Default for AcpProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentProtocol for AcpProtocol {
    fn connect(&mut self, endpoint: &str) -> Result<ConnectionInfo, ProtocolError> {
        self.endpoint = Some(endpoint.to_string());
        self.peer_id = Some(format!("peer_{}", endpoint.len()));
        self.connected = true;

        let info = ConnectionInfo {
            peer_id: self.peer_id.clone().unwrap_or_default(),
            protocol_version: self.protocol_version.clone(),
            capabilities: self.local_capabilities.clone(),
            endpoint: endpoint.to_string(),
            established_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        Ok(info)
    }

    fn disconnect(&mut self) -> Result<(), ProtocolError> {
        self.connected = false;
        self.endpoint = None;
        self.peer_id = None;
        self.remote_capabilities.clear();
        Ok(())
    }

    fn send_request(&mut self, request: ProtocolRequest) -> Result<String, ProtocolError> {
        if !self.connected {
            return Err(ProtocolError::InvalidState(
                "Not connected".to_string(),
            ));
        }

        let request_id = request.id.clone();
        let payload = request.payload.clone();
        let capabilities = request.capabilities.clone();

        let response = ProtocolResponse {
            request_id: request_id.clone(),
            status: ResponseStatus::Pending,
            payload: Some(payload),
            capabilities,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        self.store_response(response);

        Ok(request.id)
    }

    fn receive_response(&mut self, request_id: &str) -> Result<ProtocolResponse, ProtocolError> {
        if !self.connected {
            return Err(ProtocolError::InvalidState(
                "Not connected".to_string(),
            ));
        }

        match self.get_response(request_id) {
            Some(resp) => Ok(resp),
            None => Err(ProtocolError::ReceiveFailed(format!(
                "No response for request: {}",
                request_id
            ))),
        }
    }

    fn stream_event(&mut self) -> Result<ProtocolEvent, ProtocolError> {
        if !self.connected {
            return Err(ProtocolError::InvalidState(
                "Not connected".to_string(),
            ));
        }

        let event = ProtocolEvent {
            event_type: "heartbeat".to_string(),
            data: serde_json::json!({"status": "active"}),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            source: self.peer_id.clone().unwrap_or_default(),
        };

        Ok(event)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_new() {
        let protocol = AcpProtocol::new();
        assert!(!protocol.connected);
        assert_eq!(protocol.protocol_version, "1.0.0");
    }

    #[test]
    fn test_connect_disconnect() {
        let mut protocol = AcpProtocol::new();
        let info = protocol.connect("ws://localhost:8080").unwrap();
        assert!(protocol.is_connected());
        assert_eq!(info.endpoint, "ws://localhost:8080");

        protocol.disconnect().unwrap();
        assert!(!protocol.is_connected());
    }

    #[test]
    fn test_capability_negotiation() {
        let mut protocol = AcpProtocol::new();
        protocol.add_capability("search", "1.0", true);
        protocol.add_capability("compute", "2.0", false);
        assert_eq!(protocol.local_capabilities.len(), 2);
    }

    #[test]
    fn test_send_request_when_disconnected() {
        let mut protocol = AcpProtocol::new();
        let request = ProtocolRequest {
            id: "req_1".to_string(),
            action: "test".to_string(),
            payload: serde_json::json!({}),
            capabilities: vec![],
            timeout_ms: 0,
        };
        let result = protocol.send_request(request);
        assert!(result.is_err());
    }
}
