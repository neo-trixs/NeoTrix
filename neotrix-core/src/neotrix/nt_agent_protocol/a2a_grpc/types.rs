use std::collections::HashMap;

use base64::Engine as _;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::super::a2a::{AgentCard, TaskState};

// ── JWS Signature (HMAC-SHA256, A2A v1.0.1) ──────────────────────────────

/// A single JWS signature entry for a signed AgentCard (RFC 7515).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwsSignature {
    /// Signature algorithm identifier, e.g. "HS256"
    pub alg: String,
    /// Key identifier — a hex fingerprint or label
    pub kid: String,
    /// Base64url-encoded HMAC-SHA256 signature
    pub signature: String,
}

/// Signed agent card with one or more JWS signatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAgentCard {
    pub card: AgentCard,
    #[serde(default)]
    pub signatures: Vec<JwsSignature>,
}

impl SignedAgentCard {
    /// Verify that at least one signature matches the given API key.
    pub fn verify(&self, api_key: &str) -> bool {
        let card_json = serde_json::to_string(&self.card).unwrap_or_default();
        self.signatures.iter().any(|sig| {
            let expected = hmac_sha256_base64url(&card_json, api_key);
            sig.signature == expected
        })
    }
}

/// Compute HMAC-SHA256, base64url-encoded (no-pad), for JWS signing.
pub fn hmac_sha256_base64url(payload: &str, key: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mac = HmacSha256::new_from_slice(key.as_bytes())
        .unwrap_or_else(|_| HmacSha256::new_from_slice(&[0u8; 32]).expect("HMAC key fallback: 32-byte zero-key construction failed — key.as_bytes() length mismatch"));
    let code = mac.chain_update(payload.as_bytes()).finalize().into_bytes();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&code)
}

/// Sign an AgentCard using HMAC-SHA256, returning a `SignedAgentCard` with
/// JWS-compliant signature (alg="HS256").
pub fn sign_agent_card_hmac(card: &AgentCard, api_key: &str) -> SignedAgentCard {
    let kid = {
        let hash = sha2::Sha256::digest(api_key.as_bytes());
        hex::encode(&hash[..8])
    };
    let card_json = serde_json::to_string(card).unwrap_or_default();
    let signature = hmac_sha256_base64url(&card_json, api_key);
    SignedAgentCard {
        card: card.clone(),
        signatures: vec![JwsSignature {
            alg: "HS256".into(),
            kid,
            signature,
        }],
    }
}

// ── Multi-Tenancy ─────────────────────────────────────────────────────────

/// Result returned by an agent's task handler.
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub status: TaskState,
    pub output: String,
    pub metadata: HashMap<String, String>,
}

/// Handler function signature for a tenant agent.
pub type AgentTaskHandler =
    fn(crate::neotrix::nt_agent_protocol::a2a_grpc::types::GrpcSendMessageRequest) -> TaskResult;

/// An endpoint hosted by the multi-tenant registry.
#[derive(Debug, Clone)]
pub struct AgentEndpoint {
    pub agent_id: String,
    pub card: SignedAgentCard,
    pub handler: Option<AgentTaskHandler>,
}

/// Registry that maps agent IDs to their endpoints, supporting multi-tenancy.
/// Incoming gRPC requests carry a `tenant` field that selects the target agent.
#[derive(Debug, Clone)]
pub struct MultiTenantRegistry {
    agents: HashMap<String, AgentEndpoint>,
    default_agent: Option<String>,
}

impl MultiTenantRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            default_agent: None,
        }
    }

    pub fn register(&mut self, endpoint: AgentEndpoint) {
        let id = endpoint.agent_id.clone();
        if self.agents.is_empty() {
            self.default_agent = Some(id.clone());
        }
        self.agents.insert(id, endpoint);
    }

    pub fn get(&self, agent_id: &str) -> Option<&AgentEndpoint> {
        self.agents.get(agent_id)
    }

    pub fn get_default(&self) -> Option<&AgentEndpoint> {
        self.default_agent
            .as_ref()
            .and_then(|id| self.agents.get(id))
    }

    pub fn set_default(&mut self, agent_id: &str) {
        if self.agents.contains_key(agent_id) {
            self.default_agent = Some(agent_id.to_string());
        }
    }

    pub fn agents(&self) -> impl Iterator<Item = &AgentEndpoint> {
        self.agents.values()
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Clear all registered agents and reset the default.
    pub fn clear(&mut self) {
        self.agents.clear();
        self.default_agent = None;
    }

    /// Resolve an endpoint by optional tenant ID. Falls back to default.
    pub fn resolve(&self, tenant: Option<&str>) -> Option<&AgentEndpoint> {
        tenant
            .and_then(|t| self.agents.get(t))
            .or_else(|| self.get_default())
    }
}

impl Default for MultiTenantRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── gRPC Protocol Detection ──────────────────────────────────────────────

/// Protocol bindings recognized by the A2A gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedProtocol {
    Grpc,
    JsonRpc,
    HttpJsonRest,
}

/// Heuristically detect which A2A protocol binding an HTTP request uses.
///
/// - gRPC: Content-Type `application/grpc` or path matching `/a2a.A2AService/*`
/// - JSON-RPC: Content-Type `application/json` + body starts with `{"jsonrpc":`
/// - REST: Everything else with Content-Type `application/json`
/// - Unknown: Anything else
pub fn detect_protocol(
    content_type: Option<&str>,
    path: Option<&str>,
    body_prefix: Option<&[u8]>,
) -> DetectedProtocol {
    // gRPC is identified by content-type header or canonical path pattern
    if let Some(ct) = content_type {
        if ct.starts_with("application/grpc") {
            return DetectedProtocol::Grpc;
        }
    }
    if let Some(p) = path {
        if p.starts_with("/a2a.A2AService/") {
            return DetectedProtocol::Grpc;
        }
    }
    // JSON-RPC: body starts with {"jsonrpc": "2.0", ...}
    if let Some(body) = body_prefix {
        if body.len() > 20
            && body.starts_with(b"{\"jsonrpc\"")
            && body.windows(7).any(|w| w == b"\"2.0\"")
        {
            return DetectedProtocol::JsonRpc;
        }
    }
    // Default to REST JSON
    DetectedProtocol::HttpJsonRest
}

// ── gRPC Service Definition (proto documentation) ─────────────────────────
//
// The A2A v1.0.1 gRPC service is defined by the following protobuf spec.
// Full integration with tonic/prost is deferred; the HTTP/1.1 gRPC-Web
// framing layer in this module implements the same RPC semantics using JSON
// serialization over the standard gRPC wire format (5-byte header + payload).
//
// ```protobuf
// syntax = "proto3";
// package a2a;
//
// service A2AService {
//     // Send a message to an agent and create/update a task.
//     rpc SendMessage(SendMessageRequest) returns (SendMessageResponse);
//
//     // Get the current state of a task.
//     rpc GetTask(GetTaskRequest) returns (Task);
//
//     // Cancel a running task.
//     rpc CancelTask(CancelTaskRequest) returns (Task);
//
//     // Subscribe to streaming updates from a task.
//     rpc SubscribeToTask(GetTaskRequest) returns (stream TaskEvent);
//
//     // List tasks, with optional pagination.
//     rpc ListTasks(ListTasksRequest) returns (ListTasksResponse);
// }
// ```

// ── gRPC Frame (existing) ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GrpcFrame {
    pub compressed: bool,
    pub payload: Vec<u8>,
}

impl GrpcFrame {
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            compressed: false,
            payload,
        }
    }

    pub fn encode_json<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
        let payload = serde_json::to_vec(value).map_err(|e| format!("json encode: {e}"))?;
        Ok(Self::encode_raw(&payload))
    }

    pub fn encode_raw(payload: &[u8]) -> Vec<u8> {
        let len = payload.len();
        let mut buf = Vec::with_capacity(5 + len);
        buf.push(0u8);
        buf.extend_from_slice(&(len as u32).to_be_bytes());
        buf.extend_from_slice(payload);
        buf
    }

    pub fn decode(data: &[u8]) -> Result<(Self, &[u8]), String> {
        if data.len() < 5 {
            return Err("frame too short: need at least 5 bytes".into());
        }
        let compressed_flag = data[0];
        if compressed_flag > 1 {
            return Err(format!("invalid compressed flag: {compressed_flag}"));
        }
        let len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
        let total = 5 + len;
        if data.len() < total {
            return Err(format!(
                "frame truncated: need {total} bytes, have {}",
                data.len()
            ));
        }
        let payload = data[5..total].to_vec();
        let remaining = &data[total..];
        Ok((
            Self {
                compressed: compressed_flag != 0,
                payload,
            },
            remaining,
        ))
    }

    pub fn decode_all(data: &[u8]) -> Result<Vec<Self>, String> {
        let mut frames = Vec::new();
        let mut remaining = data;
        while !remaining.is_empty() {
            let (frame, rest) = Self::decode(remaining)?;
            frames.push(frame);
            remaining = rest;
        }
        Ok(frames)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrpcMethod {
    SendMessage,
    GetTask,
    CancelTask,
    SubscribeToTask,
    ListTasks,
}

impl GrpcMethod {
    pub fn from_path(path: &str) -> Option<Self> {
        match path {
            "/a2a.A2AService/SendMessage" => Some(Self::SendMessage),
            "/a2a.A2AService/GetTask" => Some(Self::GetTask),
            "/a2a.A2AService/CancelTask" => Some(Self::CancelTask),
            "/a2a.A2AService/SubscribeToTask" => Some(Self::SubscribeToTask),
            "/a2a.A2AService/ListTasks" => Some(Self::ListTasks),
            _ => None,
        }
    }

    pub fn path(self) -> &'static str {
        match self {
            Self::SendMessage => "/a2a.A2AService/SendMessage",
            Self::GetTask => "/a2a.A2AService/GetTask",
            Self::CancelTask => "/a2a.A2AService/CancelTask",
            Self::SubscribeToTask => "/a2a.A2AService/SubscribeToTask",
            Self::ListTasks => "/a2a.A2AService/ListTasks",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrpcSendMessageRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant: Option<String>,
    pub message: GrpcMessage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration: Option<GrpcSendMessageConfiguration>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrpcMessage {
    pub role: String,
    pub parts: Vec<GrpcPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcPart {
    #[serde(rename = "type")]
    pub part_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcSendMessageConfiguration {
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcSendMessageResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<GrpcTask>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<GrpcMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcTask {
    pub id: String,
    #[serde(default)]
    pub session_id: String,
    pub status: String,
    #[serde(default)]
    pub messages: Vec<GrpcMessage>,
    #[serde(default)]
    pub artifacts: Vec<GrpcArtifact>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcArtifact {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcGetTaskRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcCancelTaskRequest {
    pub id: String,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcStatusResponse {
    pub code: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcStreamResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<GrpcTask>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<GrpcMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_update: Option<GrpcStatusUpdateEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_update: Option<GrpcArtifactUpdateEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcStatusUpdateEvent {
    pub task_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcArtifactUpdateEvent {
    pub task_id: String,
    pub artifact: GrpcArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcListTasksRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcListTasksResponse {
    pub tasks: Vec<GrpcTask>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::super::a2a::{AgentCapabilities, AgentCard, AgentInterface, TaskState};
    use super::*;

    // ── JWS / SignedAgentCard / HMAC ──────────────────────────────────────

    #[test]
    fn test_jws_signature_construction() {
        let sig = JwsSignature {
            alg: "HS256".into(),
            kid: "deadbeef".into(),
            signature: "abcd1234".into(),
        };
        assert_eq!(sig.alg, "HS256");
        assert_eq!(sig.kid, "deadbeef");
        assert_eq!(sig.signature, "abcd1234");
    }

    #[test]
    fn test_hmac_sha256_base64url_deterministic() {
        let a = hmac_sha256_base64url("hello", "key");
        let b = hmac_sha256_base64url("hello", "key");
        assert_eq!(a, b);
        let c = hmac_sha256_base64url("hello", "different-key");
        assert_ne!(a, c);
    }

    #[test]
    fn test_hmac_sha256_base64url_empty() {
        let sig = hmac_sha256_base64url("", "");
        assert!(!sig.is_empty());
    }

    fn sample_card() -> AgentCard {
        AgentCard {
            name: "signed-test".into(),
            description: "desc".into(),
            url: "http://localhost:42071".into(),
            version: "1.0.0".into(),
            capabilities: AgentCapabilities::default(),
            skills: vec![],
            supported_interfaces: vec![AgentInterface {
                url: "http://localhost:42071".into(),
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
    fn test_sign_agent_card_hmac_creates_signed_card() {
        let card = sample_card();
        let api_key = "test-api-key";
        let signed = sign_agent_card_hmac(&card, api_key);
        assert_eq!(signed.card.name, "signed-test");
        assert_eq!(signed.signatures.len(), 1);
        assert_eq!(signed.signatures[0].alg, "HS256");
        assert!(!signed.signatures[0].kid.is_empty());
        assert!(!signed.signatures[0].signature.is_empty());
    }

    #[test]
    fn test_signed_agent_card_verify_ok() {
        let card = sample_card();
        let api_key = "test-key";
        let signed = sign_agent_card_hmac(&card, api_key);
        assert!(signed.verify(api_key));
    }

    #[test]
    fn test_signed_agent_card_verify_wrong_key() {
        let card = sample_card();
        let signed = sign_agent_card_hmac(&card, "real-key");
        assert!(!signed.verify("wrong-key"));
    }

    #[test]
    fn test_signed_agent_card_verify_empty_key() {
        let card = sample_card();
        let signed = sign_agent_card_hmac(&card, "real-key");
        assert!(!signed.verify(""));
    }

    #[test]
    fn test_signed_agent_card_verify_no_signatures() {
        let card = sample_card();
        let signed = SignedAgentCard {
            card,
            signatures: vec![],
        };
        assert!(!signed.verify("any-key"));
    }

    #[test]
    fn test_sign_agent_card_different_cards_different_signatures() {
        let card_a = sample_card();
        let card_b = AgentCard {
            name: "different".into(),
            ..sample_card()
        };
        let signed_a = sign_agent_card_hmac(&card_a, "key");
        let signed_b = sign_agent_card_hmac(&card_b, "key");
        assert_ne!(
            signed_a.signatures[0].signature,
            signed_b.signatures[0].signature
        );
    }

    // ── TaskResult / AgentEndpoint ────────────────────────────────────────

    #[test]
    fn test_task_result_construction() {
        let r = TaskResult {
            task_id: "t1".into(),
            status: TaskState::Completed,
            output: "done".into(),
            metadata: HashMap::new(),
        };
        assert_eq!(r.task_id, "t1");
        assert_eq!(r.output, "done");
    }

    #[test]
    fn test_agent_endpoint_construction() {
        let card = sample_card();
        let signed = sign_agent_card_hmac(&card, "k");
        let ep = AgentEndpoint {
            agent_id: "agent-1".into(),
            card: signed,
            handler: None,
        };
        assert_eq!(ep.agent_id, "agent-1");
    }

    // ── MultiTenantRegistry ───────────────────────────────────────────────

    #[test]
    fn test_registry_empty() {
        let r = MultiTenantRegistry::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert!(r.get_default().is_none());
        assert!(r.resolve(None).is_none());
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut r = MultiTenantRegistry::new();
        let card = sample_card();
        let signed = sign_agent_card_hmac(&card, "k");
        r.register(AgentEndpoint {
            agent_id: "agent-a".into(),
            card: signed,
            handler: None,
        });
        assert_eq!(r.len(), 1);
        assert!(!r.is_empty());
        assert!(r.get("agent-a").is_some());
        assert!(r.get("agent-b").is_none());
    }

    #[test]
    fn test_registry_first_agent_is_default() {
        let mut r = MultiTenantRegistry::new();
        let card = sample_card();
        r.register(AgentEndpoint {
            agent_id: "default-agent".into(),
            card: sign_agent_card_hmac(&card, "k"),
            handler: None,
        });
        assert_eq!(r.get_default().unwrap().agent_id, "default-agent");
    }

    #[test]
    fn test_registry_set_default() {
        let mut r = MultiTenantRegistry::new();
        let card = sample_card();
        r.register(AgentEndpoint {
            agent_id: "first".into(),
            card: sign_agent_card_hmac(&card, "k"),
            handler: None,
        });
        r.register(AgentEndpoint {
            agent_id: "second".into(),
            card: sign_agent_card_hmac(&card, "k"),
            handler: None,
        });
        r.set_default("second");
        assert_eq!(r.get_default().unwrap().agent_id, "second");
    }

    #[test]
    fn test_registry_set_default_unknown() {
        let mut r = MultiTenantRegistry::new();
        r.set_default("nonexistent");
        assert!(r.get_default().is_none());
    }

    #[test]
    fn test_registry_resolve_tenant() {
        let mut r = MultiTenantRegistry::new();
        let card = sample_card();
        r.register(AgentEndpoint {
            agent_id: "alpha".into(),
            card: sign_agent_card_hmac(&card, "k"),
            handler: None,
        });
        r.register(AgentEndpoint {
            agent_id: "beta".into(),
            card: sign_agent_card_hmac(&card, "k"),
            handler: None,
        });
        assert!(r.resolve(Some("alpha")).is_some());
        assert!(r.resolve(Some("beta")).is_some());
        assert!(r.resolve(Some("gamma")).is_none());
    }

    #[test]
    fn test_registry_resolve_falls_back_to_default() {
        let mut r = MultiTenantRegistry::new();
        let card = sample_card();
        r.register(AgentEndpoint {
            agent_id: "main".into(),
            card: sign_agent_card_hmac(&card, "k"),
            handler: None,
        });
        assert!(r.resolve(None).is_some());
        assert_eq!(r.resolve(None).unwrap().agent_id, "main");
    }

    #[test]
    fn test_registry_agents_iterator() {
        let mut r = MultiTenantRegistry::new();
        let card = sample_card();
        let signed = sign_agent_card_hmac(&card, "k");
        r.register(AgentEndpoint {
            agent_id: "a".into(),
            card: signed,
            handler: None,
        });
        let card2 = AgentCard {
            name: "agent-b".into(),
            ..sample_card()
        };
        r.register(AgentEndpoint {
            agent_id: "b".into(),
            card: sign_agent_card_hmac(&card2, "k"),
            handler: None,
        });
        let ids: Vec<_> = r.agents().map(|e| e.agent_id.as_str()).collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
    }

    #[test]
    fn test_registry_clear() {
        let mut r = MultiTenantRegistry::new();
        let card = sample_card();
        r.register(AgentEndpoint {
            agent_id: "x".into(),
            card: sign_agent_card_hmac(&card, "k"),
            handler: None,
        });
        assert!(!r.is_empty());
        r.clear();
        assert!(r.is_empty());
        assert!(r.get_default().is_none());
    }

    #[test]
    fn test_registry_default_trait() {
        let r = MultiTenantRegistry::default();
        assert!(r.is_empty());
    }

    // ── DetectedProtocol ──────────────────────────────────────────────────

    #[test]
    fn test_detected_protocol_equality() {
        assert_eq!(DetectedProtocol::Grpc, DetectedProtocol::Grpc);
        assert_ne!(DetectedProtocol::Grpc, DetectedProtocol::JsonRpc);
    }

    #[test]
    fn test_detect_protocol_grpc_content_type() {
        assert_eq!(
            detect_protocol(Some("application/grpc"), None, None),
            DetectedProtocol::Grpc
        );
    }

    #[test]
    fn test_detect_protocol_grpc_path() {
        assert_eq!(
            detect_protocol(None, Some("/a2a.A2AService/SendMessage"), None),
            DetectedProtocol::Grpc
        );
    }

    #[test]
    fn test_detect_protocol_grpc_path_precedence() {
        // Path should take precedence over content-type
        assert_eq!(
            detect_protocol(Some("text/plain"), Some("/a2a.A2AService/GetTask"), None),
            DetectedProtocol::Grpc
        );
    }

    #[test]
    fn test_detect_protocol_jsonrpc() {
        let body = b"{\"jsonrpc\": \"2.0\", \"method\": \"test\"}";
        assert_eq!(
            detect_protocol(Some("application/json"), None, Some(body)),
            DetectedProtocol::JsonRpc
        );
    }

    #[test]
    fn test_detect_protocol_jsonrpc_short_body() {
        let body = b"{\"jsonrpc\"}";
        assert_eq!(
            detect_protocol(Some("application/json"), None, Some(body)),
            DetectedProtocol::HttpJsonRest
        );
    }

    #[test]
    fn test_detect_protocol_rest_default() {
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

    // ── GrpcFrame ─────────────────────────────────────────────────────────

    #[test]
    fn test_grpc_frame_new() {
        let f = GrpcFrame::new(vec![1, 2, 3]);
        assert!(!f.compressed);
        assert_eq!(f.payload, vec![1, 2, 3]);
    }

    #[test]
    fn test_grpc_frame_encode_raw_structure() {
        let payload = b"test";
        let encoded = GrpcFrame::encode_raw(payload);
        // 5-byte header: 1 byte flag + 4 bytes length
        assert_eq!(encoded.len(), 5 + payload.len());
        assert_eq!(encoded[0], 0); // uncompressed flag
                                   // length in network byte order
        assert_eq!(
            u32::from_be_bytes([encoded[1], encoded[2], encoded[3], encoded[4]]),
            payload.len() as u32
        );
        assert_eq!(&encoded[5..], payload);
    }

    #[test]
    fn test_grpc_frame_decode_empty_remaining() {
        let payload = b"grpc";
        let encoded = GrpcFrame::encode_raw(payload);
        let (frame, remaining) = GrpcFrame::decode(&encoded).unwrap();
        assert_eq!(frame.payload, payload);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_grpc_frame_decode_with_remaining() {
        let p1 = b"first";
        let p2 = b"second";
        let mut data = GrpcFrame::encode_raw(p1);
        data.extend_from_slice(&GrpcFrame::encode_raw(p2));
        let (frame, rest) = GrpcFrame::decode(&data).unwrap();
        assert_eq!(frame.payload, p1);
        assert!(!rest.is_empty());
        // The rest should be exactly the second frame
        let (frame2, rest2) = GrpcFrame::decode(rest).unwrap();
        assert_eq!(frame2.payload, p2);
        assert!(rest2.is_empty());
    }

    #[test]
    fn test_grpc_frame_encode_json_roundtrip() {
        let value = serde_json::json!({"key": "value", "num": 42});
        let encoded = GrpcFrame::encode_json(&value).unwrap();
        let (frame, _) = GrpcFrame::decode(&encoded).unwrap();
        let decoded: serde_json::Value = serde_json::from_slice(&frame.payload).unwrap();
        assert_eq!(decoded["key"], "value");
        assert_eq!(decoded["num"], 42);
    }

    #[test]
    fn test_grpc_frame_encode_json_empty() {
        let value = serde_json::json!({});
        let encoded = GrpcFrame::encode_json(&value).unwrap();
        let (frame, _) = GrpcFrame::decode(&encoded).unwrap();
        assert_eq!(frame.payload, b"{}");
    }

    #[test]
    fn test_grpc_frame_decode_all_single() {
        let encoded = GrpcFrame::encode_raw(b"single");
        let frames = GrpcFrame::decode_all(&encoded).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].payload, b"single");
    }

    #[test]
    fn test_grpc_frame_decode_all_multiple() {
        let mut data = Vec::new();
        for i in 0..3 {
            data.extend_from_slice(&GrpcFrame::encode_raw(&[i]));
        }
        let frames = GrpcFrame::decode_all(&data).unwrap();
        assert_eq!(frames.len(), 3);
        for i in 0..3 {
            assert_eq!(frames[i].payload, vec![i as u8]);
        }
    }

    #[test]
    fn test_grpc_frame_decode_all_empty() {
        let frames = GrpcFrame::decode_all(&[]).unwrap();
        assert!(frames.is_empty());
    }

    #[test]
    fn test_grpc_frame_decode_too_short() {
        let err = GrpcFrame::decode(&[0u8; 3]).unwrap_err();
        assert!(err.contains("too short"));
    }

    #[test]
    fn test_grpc_frame_truncated() {
        let buf = vec![0u8, 0, 0, 0, 10];
        // only 5 bytes header, but payload claims 10
        let err = GrpcFrame::decode(&buf).unwrap_err();
        assert!(err.contains("truncated"));
    }

    #[test]
    fn test_grpc_frame_decode_invalid_compressed_flag() {
        let err = GrpcFrame::decode(&[2u8, 0, 0, 0, 1, 0]).unwrap_err();
        assert!(err.contains("invalid compressed flag"));
    }

    #[test]
    fn test_grpc_frame_decode_zero_length_payload() {
        // flag=0, length=0, no data
        let data = vec![0u8, 0, 0, 0, 0];
        let (frame, remaining) = GrpcFrame::decode(&data).unwrap();
        assert!(frame.payload.is_empty());
        assert!(remaining.is_empty());
    }

    // ── GrpcMethod ────────────────────────────────────────────────────────

    #[test]
    fn test_grpc_method_all_paths() {
        let cases = [
            (GrpcMethod::SendMessage, "/a2a.A2AService/SendMessage"),
            (GrpcMethod::GetTask, "/a2a.A2AService/GetTask"),
            (GrpcMethod::CancelTask, "/a2a.A2AService/CancelTask"),
            (
                GrpcMethod::SubscribeToTask,
                "/a2a.A2AService/SubscribeToTask",
            ),
            (GrpcMethod::ListTasks, "/a2a.A2AService/ListTasks"),
        ];
        for (method, expected_path) in &cases {
            assert_eq!(method.path(), *expected_path);
            assert_eq!(GrpcMethod::from_path(expected_path), Some(*method));
        }
    }

    #[test]
    fn test_grpc_method_unknown_path() {
        assert_eq!(GrpcMethod::from_path("/unknown/path"), None);
        assert_eq!(GrpcMethod::from_path("/a2a.A2AService/Unknown"), None);
        assert_eq!(GrpcMethod::from_path(""), None);
    }

    // ── GrpcRequest / Response types ──────────────────────────────────────

    #[test]
    fn test_grpc_send_message_request_roundtrip() {
        let req = GrpcSendMessageRequest {
            tenant: Some("tenant-1".into()),
            message: GrpcMessage {
                role: "user".into(),
                parts: vec![
                    GrpcPart {
                        part_type: "text".into(),
                        text: Some("hello".into()),
                        mime_type: Some("text/plain".into()),
                    },
                    GrpcPart {
                        part_type: "image".into(),
                        text: None,
                        mime_type: Some("image/png".into()),
                    },
                ],
            },
            configuration: Some(GrpcSendMessageConfiguration {
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("key".into(), "val".into());
                    m
                },
            }),
            metadata: {
                let mut m = HashMap::new();
                m.insert("trace".into(), "abc".into());
                m
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: GrpcSendMessageRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tenant.as_deref(), Some("tenant-1"));
        assert_eq!(back.message.parts.len(), 2);
        assert_eq!(back.message.parts[0].text.as_deref(), Some("hello"));
        assert_eq!(
            back.message.parts[1].mime_type.as_deref(),
            Some("image/png")
        );
        assert!(back.configuration.is_some());
        assert_eq!(back.metadata.get("trace").unwrap(), "abc");
    }

    #[test]
    fn test_grpc_send_message_request_defaults() {
        let json = r#"{"message":{"role":"user","parts":[]}}"#;
        let req: GrpcSendMessageRequest = serde_json::from_str(json).unwrap();
        assert!(req.tenant.is_none());
        assert!(req.configuration.is_none());
        assert!(req.metadata.is_empty());
    }

    #[test]
    fn test_grpc_message_construction() {
        let msg = GrpcMessage {
            role: "assistant".into(),
            parts: vec![GrpcPart {
                part_type: "text".into(),
                text: Some("response".into()),
                mime_type: None,
            }],
        };
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.parts[0].text.as_deref(), Some("response"));
    }

    #[test]
    fn test_grpc_part_without_text() {
        let part = GrpcPart {
            part_type: "image".into(),
            text: None,
            mime_type: Some("image/jpeg".into()),
        };
        let json = serde_json::to_string(&part).unwrap();
        assert!(!json.contains("\"text\""));
        let back: GrpcPart = serde_json::from_str(&json).unwrap();
        assert!(back.text.is_none());
    }

    #[test]
    fn test_grpc_send_message_response_roundtrip() {
        let resp = GrpcSendMessageResponse {
            task: Some(GrpcTask {
                id: "task-1".into(),
                session_id: "sess-1".into(),
                status: "completed".into(),
                messages: vec![GrpcMessage {
                    role: "assistant".into(),
                    parts: vec![GrpcPart {
                        part_type: "text".into(),
                        text: Some("done".into()),
                        mime_type: None,
                    }],
                }],
                artifacts: vec![GrpcArtifact {
                    id: "art-1".into(),
                    name: "result.txt".into(),
                    mime_type: "text/plain".into(),
                    uri: Some("http://example.com/result.txt".into()),
                    metadata: HashMap::new(),
                }],
                metadata: HashMap::new(),
            }),
            message: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: GrpcSendMessageResponse = serde_json::from_str(&json).unwrap();
        let task = back.task.unwrap();
        assert_eq!(task.id, "task-1");
        assert_eq!(task.status, "completed");
        assert_eq!(task.artifacts.len(), 1);
        assert_eq!(task.artifacts[0].name, "result.txt");
        assert_eq!(task.messages.len(), 1);
    }

    #[test]
    fn test_grpc_task_no_artifacts() {
        let task = GrpcTask {
            id: "t1".into(),
            session_id: "".into(),
            status: "submitted".into(),
            messages: vec![],
            artifacts: vec![],
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&task).unwrap();
        let back: GrpcTask = serde_json::from_str(&json).unwrap();
        assert!(back.artifacts.is_empty());
        assert_eq!(back.status, "submitted");
    }

    #[test]
    fn test_grpc_get_task_request_roundtrip() {
        let req = GrpcGetTaskRequest {
            id: "task-42".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: GrpcGetTaskRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "task-42");
    }

    #[test]
    fn test_grpc_cancel_task_request_roundtrip() {
        let req = GrpcCancelTaskRequest {
            id: "task-99".into(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("reason".into(), "user request".into());
                m
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: GrpcCancelTaskRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "task-99");
        assert_eq!(back.metadata.get("reason").unwrap(), "user request");
    }

    #[test]
    fn test_grpc_status_response() {
        let resp = GrpcStatusResponse {
            code: 200,
            message: "OK".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: GrpcStatusResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.code, 200);
        assert_eq!(back.message, "OK");
    }

    #[test]
    fn test_grpc_status_response_error() {
        let resp = GrpcStatusResponse {
            code: 500,
            message: "internal error".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("500"));
        assert!(json.contains("internal error"));
    }

    #[test]
    fn test_grpc_stream_response_with_status_update() {
        let resp = GrpcStreamResponse {
            task: None,
            message: None,
            status_update: Some(GrpcStatusUpdateEvent {
                task_id: "t1".into(),
                status: "working".into(),
            }),
            artifact_update: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: GrpcStreamResponse = serde_json::from_str(&json).unwrap();
        let update = back.status_update.unwrap();
        assert_eq!(update.task_id, "t1");
        assert_eq!(update.status, "working");
    }

    #[test]
    fn test_grpc_stream_response_with_artifact_update() {
        let resp = GrpcStreamResponse {
            task: None,
            message: None,
            status_update: None,
            artifact_update: Some(GrpcArtifactUpdateEvent {
                task_id: "t1".into(),
                artifact: GrpcArtifact {
                    id: "a1".into(),
                    name: "output".into(),
                    mime_type: "text/plain".into(),
                    uri: Some("file:///tmp/output".into()),
                    metadata: HashMap::new(),
                },
            }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: GrpcStreamResponse = serde_json::from_str(&json).unwrap();
        let update = back.artifact_update.unwrap();
        assert_eq!(update.artifact.name, "output");
        assert!(update.artifact.uri.is_some());
    }

    #[test]
    fn test_grpc_stream_response_empty() {
        let resp = GrpcStreamResponse {
            task: None,
            message: None,
            status_update: None,
            artifact_update: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: GrpcStreamResponse = serde_json::from_str(&json).unwrap();
        assert!(back.task.is_none());
        assert!(back.message.is_none());
        assert!(back.status_update.is_none());
        assert!(back.artifact_update.is_none());
    }

    #[test]
    fn test_grpc_list_tasks_request_roundtrip() {
        let req = GrpcListTasksRequest {
            session_id: Some("sess-abc".into()),
            max_results: Some(50),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: GrpcListTasksRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.session_id.as_deref(), Some("sess-abc"));
        assert_eq!(back.max_results, Some(50));
    }

    #[test]
    fn test_grpc_list_tasks_request_default() {
        let req = GrpcListTasksRequest {
            session_id: None,
            max_results: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: GrpcListTasksRequest = serde_json::from_str(&json).unwrap();
        assert!(back.session_id.is_none());
        assert!(back.max_results.is_none());
    }

    #[test]
    fn test_grpc_list_tasks_response_roundtrip() {
        let resp = GrpcListTasksResponse {
            tasks: vec![
                GrpcTask {
                    id: "t1".into(),
                    session_id: "s1".into(),
                    status: "completed".into(),
                    messages: vec![],
                    artifacts: vec![],
                    metadata: HashMap::new(),
                },
                GrpcTask {
                    id: "t2".into(),
                    session_id: "s1".into(),
                    status: "working".into(),
                    messages: vec![],
                    artifacts: vec![],
                    metadata: HashMap::new(),
                },
            ],
            next_page_token: Some("page2".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: GrpcListTasksResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tasks.len(), 2);
        assert_eq!(back.next_page_token.as_deref(), Some("page2"));
    }

    #[test]
    fn test_grpc_list_tasks_response_no_pagination() {
        let resp = GrpcListTasksResponse {
            tasks: vec![],
            next_page_token: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: GrpcListTasksResponse = serde_json::from_str(&json).unwrap();
        assert!(back.tasks.is_empty());
        assert!(back.next_page_token.is_none());
    }

    // ── GrpcMessage / GrpcArtifact object construction ────────────────────

    #[test]
    fn test_grpc_part_serialization_skip_optional() {
        let part = GrpcPart {
            part_type: "text".into(),
            text: None,
            mime_type: None,
        };
        let json = serde_json::to_string(&part).unwrap();
        // Both `text` and `mime_type` should be absent (skip_serializing_if)
        assert!(!json.contains("\"text\""));
        assert!(!json.contains("\"mimeType\""));
        assert!(json.contains("\"type\""));
    }

    #[test]
    fn test_grpc_artifact_without_uri() {
        let art = GrpcArtifact {
            id: "a1".into(),
            name: "output".into(),
            mime_type: "text/plain".into(),
            uri: None,
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&art).unwrap();
        assert!(!json.contains("\"uri\""));
        let back: GrpcArtifact = serde_json::from_str(&json).unwrap();
        assert!(back.uri.is_none());
    }

    #[test]
    fn test_grpc_artifact_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("size".into(), "1024".into());
        metadata.insert("hash".into(), "sha256:abc".into());
        let art = GrpcArtifact {
            id: "a2".into(),
            name: "data.bin".into(),
            mime_type: "application/octet-stream".into(),
            uri: Some("file:///data.bin".into()),
            metadata,
        };
        let json = serde_json::to_string(&art).unwrap();
        let back: GrpcArtifact = serde_json::from_str(&json).unwrap();
        assert_eq!(back.metadata.get("size").unwrap(), "1024");
        assert_eq!(back.metadata.get("hash").unwrap(), "sha256:abc");
    }

    #[test]
    fn test_grpc_message_empty_parts() {
        let msg = GrpcMessage {
            role: "system".into(),
            parts: vec![],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: GrpcMessage = serde_json::from_str(&json).unwrap();
        assert!(back.parts.is_empty());
    }
}
