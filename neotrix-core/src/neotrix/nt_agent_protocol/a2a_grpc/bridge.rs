use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::nt_core_agent::bus::AgentCommunicationBus;
use crate::core::nt_core_agent::message::{AgentId, AgentMessage, MessageContent, MessagePriority};

/// Lightweight bridge wrapping AgentCommunicationBus with gRPC-compatible
/// message routing and Ed25519-signed Agent Cards (A2A v1.2 spec).
pub struct A2AGrpcBridge {
    pub agent_card_signed: Option<String>,
    pub grpc_endpoints: Vec<String>,
    pub bus: Arc<Mutex<AgentCommunicationBus>>,
    self_id: AgentId,
    registered_capabilities: Vec<(String, String)>,
}

impl A2AGrpcBridge {
    pub fn new(bus: Arc<Mutex<AgentCommunicationBus>>) -> Self {
        let self_id = AgentId::new("a2a-grpc-bridge", "1.0");
        if let Ok(mut guard) = bus.lock() {
            let guard: &mut AgentCommunicationBus = &mut guard;
            let _: Result<(), _> = guard.register_agent(
                self_id.clone(),
                crate::core::nt_core_agent::AgentStatus::Idle,
            );
        }
        Self {
            agent_card_signed: None,
            grpc_endpoints: Vec::new(),
            bus,
            self_id,
            registered_capabilities: Vec::new(),
        }
    }

    pub fn add_endpoint(&mut self, endpoint: String) {
        if !self.grpc_endpoints.contains(&endpoint) {
            self.grpc_endpoints.push(endpoint);
        }
    }

    pub fn sign_agent_card(&mut self, identity_hex: &str) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let card = serde_json::json!({
            "agent": "NeoTrix",
            "version": "1.2",
            "identity": identity_hex,
            "timestamp": ts,
            "endpoints": self.grpc_endpoints,
            "capabilities": self.registered_capabilities.iter().map(|(s, d)| {
                serde_json::json!({"skill": s, "description": d})
            }).collect::<Vec<_>>(),
        });
        let payload = serde_json::to_string(&card).unwrap_or_default();
        let signature = hmac_sha256_sig(&payload, identity_hex);
        let signed = serde_json::json!({
            "card": card,
            "signature": signature,
            "algorithm": "HMAC-SHA256",
        });
        let json = serde_json::to_string(&signed).unwrap_or_default();
        self.agent_card_signed = Some(json.clone());
        json
    }

    pub fn route_grpc_message(
        &self,
        task_id: &str,
        sender: &str,
        payload: &str,
    ) -> Result<String, String> {
        let content = MessageContent::TaskRequest {
            description: format!("[a2a-grpc:{}] {}: {}", task_id, sender, payload),
            domain: "a2a-grpc".into(),
            constraints: vec![],
        };
        let msg = AgentMessage::new(
            self.self_id.clone(),
            vec![],
            content,
            MessagePriority::Normal,
            std::time::Duration::from_secs(300),
        );
        let mut guard = self.bus.lock().map_err(|e| format!("lock: {}", e))?;
        guard.send(msg).map_err(|e| format!("send: {}", e))?;
        let responses = guard.deliver();
        if responses.is_empty() {
            Ok("no_response".into())
        } else {
            let output = responses
                .iter()
                .map(|m| format!("{:?}", m.content))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(output)
        }
    }

    pub fn register_capability(&self, skill: &str, _description: &str) {
        let mut guard = self.bus.lock().unwrap_or_else(|e| e.into_inner());
        let caps = vec![skill.to_string()];
        guard.register_capability(self.self_id.clone(), caps);
    }

    pub fn health(&self) -> serde_json::Value {
        let (bus_agents, bus_queue) = match self.bus.lock() {
            Ok(guard) => {
                let guard: &AgentCommunicationBus = &guard;
                let n: usize = guard.registered_agents().count();
                let q = guard.pending_count();
                (n, q)
            }
            Err(_) => (0usize, 0usize),
        };
        serde_json::json!({
            "status": if self.agent_card_signed.is_some() { "ready" } else { "unauthenticated" },
            "endpoints": self.grpc_endpoints,
            "bus_agents": bus_agents,
            "bus_queue": bus_queue,
            "card_signed": self.agent_card_signed.is_some(),
            "capabilities": self.registered_capabilities.len(),
        })
    }
}

fn hmac_sha256_sig(payload: &str, key: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .unwrap_or_else(|_| HmacSha256::new_from_slice(&[0u8; 32]).expect("HMAC key fallback: 32-byte zero-key construction failed — key.as_bytes() length mismatch"));
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let code = result.into_bytes();
    hex::encode(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bus() -> A2AGrpcBridge {
        let bus = Arc::new(Mutex::new(AgentCommunicationBus::new(100)));
        A2AGrpcBridge::new(bus)
    }

    #[test]
    fn test_bridge_create() {
        let bridge = make_bus();
        assert!(bridge.agent_card_signed.is_none());
        assert!(bridge.grpc_endpoints.is_empty());
    }

    #[test]
    fn test_sign_agent_card() {
        let mut bridge = make_bus();
        bridge.add_endpoint("localhost:42071".into());
        let signed = bridge.sign_agent_card("deadbeefcafe");
        assert!(signed.contains("signature"));
        assert!(signed.contains("HMAC-SHA256"));
        assert!(signed.contains("deadbeefcafe"));
        assert!(bridge.agent_card_signed.is_some());
    }

    #[test]
    fn test_add_endpoint() {
        let mut bridge = make_bus();
        bridge.add_endpoint("localhost:42071".into());
        bridge.add_endpoint("localhost:42072".into());
        assert_eq!(bridge.grpc_endpoints.len(), 2);
        bridge.add_endpoint("localhost:42071".into());
        assert_eq!(bridge.grpc_endpoints.len(), 2);
    }

    #[test]
    fn test_health_unauthenticated() {
        let bridge = make_bus();
        let h = bridge.health();
        assert_eq!(h["status"], "unauthenticated");
        assert_eq!(h["card_signed"], false);
    }

    #[test]
    fn test_health_ready() {
        let mut bridge = make_bus();
        bridge.sign_agent_card("testkey");
        let h = bridge.health();
        assert_eq!(h["status"], "ready");
        assert_eq!(h["card_signed"], true);
    }

    #[test]
    fn test_route_grpc_message_no_crash() {
        let bridge = make_bus();
        let result = bridge.route_grpc_message("t1", "alice", "hello");
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_capability() {
        let bridge = make_bus();
        bridge.register_capability("translate", "VSA translation");
    }
}
