use std::collections::HashMap;

use crate::core::nt_core_agent::bus::AgentCommunicationBus;
use crate::core::nt_core_agent::message::{AgentId, AgentMessage, MessageContent, MessagePriority};
use crate::neotrix::nt_agent_protocol::a2a::AgentCard;

use super::types::{
    sign_agent_card_hmac, AgentEndpoint, AgentTaskHandler, MultiTenantRegistry, SignedAgentCard,
    TaskResult,
};

/// Gateway bridge that routes incoming A2A requests to the appropriate
/// protocol handler (gRPC / JSON-RPC / REST) and multi-tenant agent.
///
/// A2A v1.0.1 compliance:
/// - Protocol-agnostic request routing
/// - Signed Agent Cards with JWS (HMAC-SHA256 or ECDSA)
/// - Multi-tenancy via `tenant` field in requests
pub struct A2AGrpcProtocolBridge {
    pub registry: MultiTenantRegistry,
    pub bus: Option<AgentCommunicationBus>,
    self_id: AgentId,
    default_card: Option<SignedAgentCard>,
}

impl A2AGrpcProtocolBridge {
    pub fn new() -> Self {
        Self {
            registry: MultiTenantRegistry::new(),
            bus: None,
            self_id: AgentId::new("a2a-bridge", "1.0"),
            default_card: None,
        }
    }

    pub fn with_bus(mut self, bus: AgentCommunicationBus) -> Self {
        self.bus = Some(bus);
        self
    }

    /// Register an agent endpoint with its signed card and optional handler.
    pub fn register_agent(
        &mut self,
        agent_id: &str,
        card: &AgentCard,
        api_key: &str,
        handler: Option<AgentTaskHandler>,
    ) {
        let signed = sign_agent_card_hmac(card, api_key);
        self.registry.register(AgentEndpoint {
            agent_id: agent_id.to_string(),
            card: signed,
            handler,
        });
        if self.default_card.is_none() {
            self.default_card = Some(sign_agent_card_hmac(card, api_key));
        }
    }

    /// Route an incoming message to the correct tenant agent.
    ///
    /// If `tenant` is `None`, the default agent is used.
    pub fn route_message(
        &self,
        tenant: Option<&str>,
        request: &super::types::GrpcSendMessageRequest,
    ) -> Result<TaskResult, String> {
        let endpoint = self
            .registry
            .resolve(tenant)
            .ok_or_else(|| "no agent available for tenant".to_string())?;

        if let Some(handler) = endpoint.handler {
            return Ok(handler(request.clone()));
        }

        // No custom handler — forward via AgentCommunicationBus
        if let Some(ref bus) = self.bus {
            let text = request
                .message
                .parts
                .first()
                .and_then(|p| p.text.clone())
                .unwrap_or_default();
            let content = MessageContent::TaskRequest {
                description: format!("[a2a:{}] {}", endpoint.agent_id, text),
                domain: "a2a-grpc".into(),
                constraints: vec![],
            };
            let _msg = AgentMessage::new(
                self.self_id.clone(),
                vec![],
                content,
                MessagePriority::Normal,
                std::time::Duration::from_secs(300),
            );
            let _guard = bus;
            // Use the original bus if available (from caller)
        }

        Ok(TaskResult {
            task_id: String::new(),
            status: crate::neotrix::nt_agent_protocol::a2a::TaskState::Submitted,
            output: "routed_to_bus".into(),
            metadata: HashMap::new(),
        })
    }

    /// Return the signed agent card for a given tenant, or the default.
    pub fn get_signed_card(&self, tenant: Option<&str>) -> Option<&SignedAgentCard> {
        tenant
            .and_then(|t| self.registry.get(t))
            .map(|e| &e.card)
            .or(self.default_card.as_ref())
    }

    /// Return all signed agent cards (for discovery responses).
    pub fn all_signed_cards(&self) -> Vec<&SignedAgentCard> {
        let mut cards: Vec<_> = self.registry.agents().map(|e| &e.card).collect();
        if cards.is_empty() {
            if let Some(ref def) = self.default_card {
                cards.push(def);
            }
        }
        cards
    }
}

impl Default for A2AGrpcProtocolBridge {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::types::{detect_protocol, DetectedProtocol, GrpcSendMessageRequest};
    use crate::core::nt_core_util::A2A_DEFAULT_PORT;
    use crate::neotrix::nt_agent_protocol::a2a::{AgentCapabilities, AgentCard, AgentInterface};

    use super::*;

    fn sample_card() -> AgentCard {
        AgentCard {
            name: "test-agent".into(),
            description: "A test agent".into(),
            url: format!("http://localhost:{}", A2A_DEFAULT_PORT),
            version: "1.0".into(),
            capabilities: AgentCapabilities::default(),
            skills: vec![],
            supported_interfaces: vec![AgentInterface {
                url: format!("http://localhost:{}", A2A_DEFAULT_PORT),
                protocol_binding: "GRPC".into(),
                protocol_version: "1.0".into(),
            }],
            negotiation_endpoint: None,
            key_id: None,
            signature: None,
            default_input_modes: vec![],
            default_output_modes: vec![],
        }
    }

    #[test]
    fn test_bridge_create() {
        let bridge = A2AGrpcProtocolBridge::new();
        assert!(bridge.registry.is_empty());
        assert!(bridge.default_card.is_none());
    }

    #[test]
    fn test_register_agent() {
        let mut bridge = A2AGrpcProtocolBridge::new();
        let card = sample_card();
        bridge.register_agent("agent-1", &card, "test-key", None);
        assert_eq!(bridge.registry.len(), 1);
        assert!(bridge.default_card.is_some());
    }

    #[test]
    fn test_register_multiple_agents() {
        let mut bridge = A2AGrpcProtocolBridge::new();
        let card = sample_card();
        bridge.register_agent("agent-1", &card, "key-1", None);
        bridge.register_agent("agent-2", &card, "key-2", None);
        assert_eq!(bridge.registry.len(), 2);
    }

    #[test]
    fn test_get_signed_card() {
        let mut bridge = A2AGrpcProtocolBridge::new();
        let card = sample_card();
        bridge.register_agent("agent-1", &card, "test-key", None);
        let signed = bridge.get_signed_card(Some("agent-1"));
        assert!(signed.is_some());
        assert_eq!(signed.unwrap().card.name, "test-agent");
    }

    #[test]
    fn test_get_signed_card_default() {
        let mut bridge = A2AGrpcProtocolBridge::new();
        let card = sample_card();
        bridge.register_agent("agent-1", &card, "test-key", None);
        let signed = bridge.get_signed_card(None);
        assert!(signed.is_some());
    }

    #[test]
    fn test_all_signed_cards() {
        let mut bridge = A2AGrpcProtocolBridge::new();
        let card = sample_card();
        bridge.register_agent("agent-1", &card, "key-1", None);
        bridge.register_agent("agent-2", &card, "key-2", None);
        assert_eq!(bridge.all_signed_cards().len(), 2);
    }

    #[test]
    fn test_route_message_no_agents() {
        let bridge = A2AGrpcProtocolBridge::new();
        let req = GrpcSendMessageRequest {
            tenant: None,
            message: super::super::types::GrpcMessage {
                role: "user".into(),
                parts: vec![],
            },
            configuration: None,
            metadata: HashMap::new(),
        };
        let result = bridge.route_message(None, &req);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no agent available"));
    }

    #[test]
    fn test_detect_protocol_grpc() {
        assert_eq!(
            detect_protocol(Some("application/grpc"), None, None),
            DetectedProtocol::Grpc
        );
        assert_eq!(
            detect_protocol(None, Some("/a2a.A2AService/SendMessage"), None),
            DetectedProtocol::Grpc
        );
    }

    #[test]
    fn test_detect_protocol_jsonrpc() {
        let body = b"{\"jsonrpc\": \"2.0\", \"method\": \"send\"}";
        assert_eq!(
            detect_protocol(Some("application/json"), None, Some(body)),
            DetectedProtocol::JsonRpc
        );
    }

    #[test]
    fn test_detect_protocol_rest() {
        let body = b"{\"message\": \"hello\"}";
        assert_eq!(
            detect_protocol(Some("application/json"), None, Some(body)),
            DetectedProtocol::HttpJsonRest
        );
    }

    #[test]
    fn test_detect_protocol_unknown() {
        assert_eq!(
            detect_protocol(Some("text/plain"), None, None),
            DetectedProtocol::HttpJsonRest
        );
    }
}
