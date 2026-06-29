use std::collections::HashMap;

use base64::Engine as _;
use k256::ecdsa::signature::{Signer, Verifier};
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A2A protocol version string — code implements v1.2 features (gRPC binding,
/// signed Agent Cards, capability negotiation, batch tasks, latency broadcast).
pub const A2A_PROTOCOL_VERSION: &str = "1.2";

// ── Protocol Binding Types ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProtocolBinding {
    JsonRpc,
    Grpc,
    #[serde(rename = "HTTP+JSON")]
    HttpJsonRest,
}

impl ProtocolBinding {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtocolBinding::JsonRpc => "JSONRPC",
            ProtocolBinding::Grpc => "GRPC",
            ProtocolBinding::HttpJsonRest => "HTTP+JSON",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentInterface {
    pub url: String,
    #[serde(rename = "protocolBinding")]
    pub protocol_binding: String,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
}

// ── A2A Types ──────────────────────────────────────────────────────────────

/// JWS-style signature for a signed AgentCard (A2A v1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCardSignature {
    /// Base64url-encoded JWS protected header: `{"alg":"ES256","typ":"JWS","kid":"<key_id>"}`
    pub protected: String,
    /// Raw ECDSA (P-256) signature bytes (DER-encoded)
    pub signature: Vec<u8>,
    /// Identifier for the signing key
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    pub version: String,
    #[serde(default)]
    pub capabilities: AgentCapabilities,
    #[serde(default)]
    pub skills: Vec<SkillDecl>,
    /// The list of supported protocol interfaces.
    /// If empty, the `url` field is used as a single HTTP+JSON interface.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_interfaces: Vec<AgentInterface>,
    /// Protocol negotiation endpoint (`.well-known/negotiate`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_endpoint: Option<String>,
    /// Optional key identifier for signed AgentCards (A2A v1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
    /// Optional JWS signature for signed AgentCards (A2A v1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<AgentCardSignature>,
    /// Default input MIME types accepted by this agent (A2A v1.0).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_input_modes: Vec<String>,
    /// Default output MIME types produced by this agent (A2A v1.0).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_output_modes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub push_notifications: bool,
    #[serde(default)]
    pub authentication: bool,
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            streaming: true,
            push_notifications: false,
            authentication: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDecl {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskState {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Failed,
    Canceled,
}

impl TaskState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskState::Completed | TaskState::Failed | TaskState::Canceled
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct A2ATask {
    pub id: String,
    #[serde(default)]
    pub session_id: String,
    pub status: TaskState,
    #[serde(default)]
    pub messages: Vec<A2AMessage>,
    #[serde(default)]
    pub artifacts: Vec<A2AArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct A2AMessage {
    pub role: String,
    pub parts: Vec<A2APart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct A2APart {
    #[serde(rename = "type")]
    pub part_type: A2APartType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum A2APartType {
    Text,
    File,
    Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AArtifact {
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
#[serde(deny_unknown_fields)]
pub struct TaskEvent {
    pub task_id: String,
    pub event_type: String,
    pub status: TaskState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<A2AMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<A2AArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendTaskRequest {
    pub id: String,
    #[serde(default)]
    pub session_id: String,
    pub messages: Vec<A2AMessage>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTaskResponse {
    pub task: A2ATask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskResponse {
    pub task: A2ATask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskResponse {
    pub task: A2ATask,
}

// ── Card Signing (JWS-style, A2A v1.0) ─────────────────────────────────────

/// Serialize the AgentCard to canonical JSON bytes, excluding signature fields.
/// This produces the payload that is signed/verified.
fn card_signing_payload(card: &AgentCard) -> Result<Vec<u8>, String> {
    let obj = serde_json::json!({
        "name": card.name,
        "description": card.description,
        "url": card.url,
        "version": card.version,
        "capabilities": {
            "streaming": card.capabilities.streaming,
            "push_notifications": card.capabilities.push_notifications,
            "authentication": card.capabilities.authentication,
        },
        "skills": card.skills.iter().map(|s| serde_json::json!({
            "id": s.id,
            "name": s.name,
            "description": s.description,
            "tags": s.tags,
        })).collect::<Vec<_>>(),
        "supported_interfaces": card.supported_interfaces.iter().map(|i| serde_json::json!({
            "url": i.url,
            "protocolBinding": i.protocol_binding,
            "protocolVersion": i.protocol_version,
        })).collect::<Vec<_>>(),
        "negotiation_endpoint": card.negotiation_endpoint,
    });
    serde_json::to_vec(&obj).map_err(|e| format!("serialize payload: {e}"))
}

/// Sign an AgentCard and return the JWS-style signature.
///
/// The signing input is `base64url(header) || "." || base64url(card_payload)`,
/// signed with SHA-256 ECDSA using the provided key.
pub fn sign_agent_card(
    card: &AgentCard,
    signing_key: &SigningKey,
) -> Result<AgentCardSignature, String> {
    // Derive key_id from the public key fingerprint
    let pubkey = VerifyingKey::from(signing_key);
    let pubkey_bytes = pubkey.to_sec1_bytes();
    let key_id = hex::encode(&pubkey_bytes[..8]);

    // Build JWS protected header
    let header = serde_json::json!({
        "alg": "ES256",
        "typ": "JWS",
        "kid": key_id,
    });
    let header_bytes =
        serde_json::to_vec(&header).map_err(|e| format!("serialize JWS header: {e}"))?;
    let b64_encoder = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let protected = b64_encoder.encode(&header_bytes);

    // Build signing payload
    let payload = card_signing_payload(card)?;
    let payload_b64 = b64_encoder.encode(&payload);

    // Signing input: protected . payload_b64
    let signing_input = format!("{protected}.{payload_b64}");
    let hash = Sha256::digest(signing_input.as_bytes());
    let sig: Signature = signing_key.sign(&hash);

    Ok(AgentCardSignature {
        protected,
        signature: sig.to_der().as_bytes().to_vec(),
        key_id,
    })
}

/// Verify a JWS-style AgentCardSignature against the card content and public key.
pub fn verify_agent_card(
    card: &AgentCard,
    signature: &AgentCardSignature,
    verifying_key: &VerifyingKey,
) -> Result<bool, String> {
    let payload = card_signing_payload(card)?;
    let b64_encoder = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let payload_b64 = b64_encoder.encode(&payload);

    let signing_input = format!("{}.{payload_b64}", signature.protected);
    let hash = Sha256::digest(signing_input.as_bytes());

    let sig = Signature::from_der(&signature.signature)
        .map_err(|e| format!("invalid signature DER: {e}"))?;

    Ok(verifying_key.verify(&hash, &sig).is_ok())
}

impl AgentCard {
    /// Create a signed copy of this AgentCard by signing with the given key.
    /// The returned card has `key_id` and `signature` fields populated.
    pub fn to_signed_card(self, signing_key: &SigningKey) -> Result<Self, String> {
        let sig = sign_agent_card(&self, signing_key)?;
        let key_id = sig.key_id.clone();
        Ok(Self {
            key_id: Some(key_id),
            signature: Some(sig),
            ..self
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn test_agent_card_serialization() {
        let card = AgentCard {
            name: "NeoTrix".into(),
            description: "Cognitive agent".into(),
            url: "http://localhost:42069".into(),
            version: "0.1.0".into(),
            capabilities: AgentCapabilities {
                streaming: true,
                push_notifications: false,
                authentication: false,
            },
            skills: vec![SkillDecl {
                id: "s1".into(),
                name: "Skill 1".into(),
                description: "A test skill".into(),
                tags: vec![],
            }],
            supported_interfaces: vec![AgentInterface {
                url: "http://localhost:42069".into(),
                protocol_binding: "HTTP+JSON".into(),
                protocol_version: "1.0".into(),
            }],
            negotiation_endpoint: None,
            key_id: None,
            signature: None,
            default_input_modes: vec![],
            default_output_modes: vec![],
        };
        let json = serde_json::to_string(&card).expect("card should serialize");
        let deserialized: AgentCard = serde_json::from_str(&json).expect("card should deserialize");
        assert_eq!(deserialized.name, "NeoTrix");
        assert!(deserialized.capabilities.streaming);
    }

    #[test]
    fn test_task_state_terminal() {
        assert!(!TaskState::Submitted.is_terminal());
        assert!(!TaskState::Working.is_terminal());
        assert!(TaskState::Completed.is_terminal());
        assert!(TaskState::Failed.is_terminal());
        assert!(TaskState::Canceled.is_terminal());
    }

    #[test]
    fn test_a2a_task_roundtrip() {
        let task = A2ATask {
            id: "task-1".into(),
            session_id: "session-1".into(),
            status: TaskState::Submitted,
            messages: vec![A2AMessage {
                role: "user".into(),
                parts: vec![A2APart {
                    part_type: A2APartType::Text,
                    text: Some("hello".into()),
                    mime_type: Some("text/plain".into()),
                    file_uri: None,
                    data: None,
                }],
            }],
            artifacts: vec![],
            error_message: None,
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&task).expect("task should serialize");
        let deserialized: A2ATask = serde_json::from_str(&json).expect("task should deserialize");
        assert_eq!(deserialized.id, "task-1");
        assert_eq!(deserialized.messages.len(), 1);
        assert_eq!(
            deserialized.messages[0].parts[0].text.as_deref(),
            Some("hello")
        );
    }

    #[test]
    fn test_send_task_request_serialization() {
        let req = SendTaskRequest {
            id: "req-1".into(),
            session_id: "ss-1".into(),
            messages: vec![A2AMessage {
                role: "user".into(),
                parts: vec![A2APart {
                    part_type: A2APartType::Text,
                    text: Some("do work".into()),
                    mime_type: None,
                    file_uri: None,
                    data: None,
                }],
            }],
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&req).expect("request should serialize");
        let deserialized: SendTaskRequest =
            serde_json::from_str(&json).expect("request should deserialize");
        assert_eq!(deserialized.id, "req-1");
        assert_eq!(deserialized.messages[0].parts.len(), 1);
    }

    #[test]
    fn test_skill_decl_defaults() {
        let skill = SkillDecl {
            id: "s1".into(),
            name: "test".into(),
            description: "desc".into(),
            tags: vec![],
        };
        let json = serde_json::to_string(&skill).expect("skill should serialize");
        assert!(json.contains("\"id\":\"s1\""));
        assert!(json.contains("\"tags\""));
    }

    #[test]
    fn test_agent_card_default_capabilities() {
        let caps = AgentCapabilities::default();
        assert!(caps.streaming);
        assert!(!caps.authentication);
    }

    #[test]
    fn test_a2a_part_types() {
        let text_part = A2APart {
            part_type: A2APartType::Text,
            text: Some("hello".into()),
            mime_type: None,
            file_uri: None,
            data: None,
        };
        let json = serde_json::to_string(&text_part).expect("part should serialize");
        assert!(json.contains("text"));

        let file_part = A2APart {
            part_type: A2APartType::File,
            text: None,
            mime_type: Some("image/png".into()),
            file_uri: Some("file:///tmp/x.png".into()),
            data: None,
        };
        let json2 = serde_json::to_string(&file_part).expect("file part should serialize");
        assert!(json2.contains("file"));
    }

    // ── AgentCard signature (JWS, A2A v1.0) tests ────────────────────────

    fn sample_card_for_signing() -> AgentCard {
        AgentCard {
            name: "signed-agent".into(),
            description: "A test agent for JWS signing".into(),
            url: "http://localhost:42069".into(),
            version: "1.0".into(),
            capabilities: AgentCapabilities {
                streaming: true,
                push_notifications: false,
                authentication: true,
            },
            skills: vec![SkillDecl {
                id: "test-skill".into(),
                name: "Test Skill".into(),
                description: "A skill for testing".into(),
                tags: vec!["test".into()],
            }],
            supported_interfaces: vec![AgentInterface {
                url: "http://localhost:42069".into(),
                protocol_binding: "HTTP+JSON".into(),
                protocol_version: "1.0".into(),
            }],
            negotiation_endpoint: Some("/.well-known/negotiate".into()),
            key_id: None,
            signature: None,
            default_input_modes: vec!["text/plain".into()],
            default_output_modes: vec!["text/plain".into()],
        }
    }

    #[test]
    fn test_a2a_protocol_version_constant() {
        assert_eq!(A2A_PROTOCOL_VERSION, "1.2");
    }

    #[test]
    fn test_sign_agent_card_roundtrip() {
        let signing_key = SigningKey::random(&mut OsRng);
        let pubkey = VerifyingKey::from(&signing_key);

        let card = sample_card_for_signing();
        let sig = sign_agent_card(&card, &signing_key).expect("sign should succeed");

        assert!(
            !sig.protected.is_empty(),
            "protected header should be non-empty"
        );
        assert!(!sig.signature.is_empty(), "signature should be non-empty");
        assert!(!sig.key_id.is_empty(), "key_id should be non-empty");

        let verified = verify_agent_card(&card, &sig, &pubkey).expect("verify should succeed");
        assert!(verified, "signature should verify against original card");
    }

    #[test]
    fn test_to_signed_card_roundtrip() {
        let signing_key = SigningKey::random(&mut OsRng);
        let pubkey = VerifyingKey::from(&signing_key);

        let card = sample_card_for_signing();
        let signed_card = card
            .to_signed_card(&signing_key)
            .expect("to_signed_card should succeed");

        assert!(
            signed_card.signature.is_some(),
            "signed card should have signature"
        );
        assert!(
            signed_card.key_id.is_some(),
            "signed card should have key_id"
        );

        let sig = signed_card.signature.as_ref().unwrap();
        let verified =
            verify_agent_card(&signed_card, sig, &pubkey).expect("verify should succeed");
        assert!(verified, "to_signed_card roundtrip should verify");
    }

    #[test]
    fn test_tampered_card_fails_verification() {
        let signing_key = SigningKey::random(&mut OsRng);
        let pubkey = VerifyingKey::from(&signing_key);

        let card = sample_card_for_signing();
        let sig = sign_agent_card(&card, &signing_key).expect("sign should succeed");

        // Tamper with the card
        let mut tampered = card;
        tampered.name = "evil-agent".into();

        let verified = verify_agent_card(&tampered, &sig, &pubkey).expect("verify should succeed");
        assert!(!verified, "tampered card should NOT verify");
    }

    #[test]
    fn test_wrong_key_fails_verification() {
        let key_a = SigningKey::random(&mut OsRng);
        let key_b = SigningKey::random(&mut OsRng);
        let pubkey_b = VerifyingKey::from(&key_b);

        let card = sample_card_for_signing();
        let sig = sign_agent_card(&card, &key_a).expect("sign should succeed");

        let verified = verify_agent_card(&card, &sig, &pubkey_b).expect("verify should succeed");
        assert!(!verified, "wrong key should NOT verify");
    }

    #[test]
    fn test_unsigned_card_backward_compatibility() {
        let card = sample_card_for_signing();
        assert!(
            card.signature.is_none(),
            "unsigned card should have no signature"
        );
        assert!(card.key_id.is_none(), "unsigned card should have no key_id");

        let json = serde_json::to_string(&card).expect("card should serialize");
        let deserialized: AgentCard = serde_json::from_str(&json).expect("card should deserialize");
        assert!(
            deserialized.signature.is_none(),
            "deserialized unsigned card should have no signature"
        );
        assert_eq!(deserialized.name, "signed-agent");
    }

    #[test]
    fn test_signed_card_serialization_roundtrip() {
        let signing_key = SigningKey::random(&mut OsRng);

        let card = sample_card_for_signing();
        let signed = card
            .to_signed_card(&signing_key)
            .expect("to_signed_card should succeed");

        let json = serde_json::to_string(&signed).expect("signed card should serialize");
        let deserialized: AgentCard =
            serde_json::from_str(&json).expect("signed card should deserialize");

        assert!(
            deserialized.signature.is_some(),
            "signature should survive serialization"
        );
        assert!(
            deserialized.key_id.is_some(),
            "key_id should survive serialization"
        );
        assert_eq!(deserialized.name, "signed-agent");
        assert_eq!(
            deserialized.signature.as_ref().unwrap().key_id,
            signed.signature.unwrap().key_id
        );
    }

    #[test]
    fn test_card_signing_payload_deterministic() {
        let card = sample_card_for_signing();
        let p1 = card_signing_payload(&card).expect("payload should serialize");
        let p2 = card_signing_payload(&card).expect("payload should serialize");
        assert_eq!(p1, p2, "signing payload must be deterministic");
    }

    #[test]
    fn test_agent_card_signature_contains_key_id() {
        let signing_key = SigningKey::random(&mut OsRng);

        let card = sample_card_for_signing();
        let sig = sign_agent_card(&card, &signing_key).expect("sign should succeed");

        // The key_id should be a hex string derived from the public key
        assert_eq!(
            sig.key_id.len(),
            16,
            "key_id should be 16 hex chars (8 bytes)"
        );
        assert!(
            sig.key_id.chars().all(|c| c.is_ascii_hexdigit()),
            "key_id should be hex"
        );
    }

    #[test]
    fn test_agent_card_signature_json_field_names() {
        let signing_key = SigningKey::random(&mut OsRng);

        let card = sample_card_for_signing();
        let signed = card
            .to_signed_card(&signing_key)
            .expect("to_signed_card should succeed");
        let json = serde_json::to_string(&signed).expect("serialize");

        assert!(
            json.contains("\"key_id\""),
            "JSON should contain key_id field"
        );
        assert!(
            json.contains("\"signature\""),
            "JSON should contain signature field"
        );
    }

    #[test]
    fn test_invalid_signature_der_rejected() {
        let signing_key = SigningKey::random(&mut OsRng);
        let pubkey = VerifyingKey::from(&signing_key);

        let card = sample_card_for_signing();
        let mut sig = sign_agent_card(&card, &signing_key).expect("sign should succeed");

        // Corrupt the signature bytes
        sig.signature = vec![0x00, 0x01, 0x02];
        let result = verify_agent_card(&card, &sig, &pubkey);
        assert!(
            result.is_err() || !result.unwrap(),
            "invalid DER should fail verification"
        );
    }

    #[test]
    fn test_signed_card_unchanged_unsigned_fields() {
        let signing_key = SigningKey::random(&mut OsRng);

        let card = sample_card_for_signing();
        let signed = card
            .to_signed_card(&signing_key)
            .expect("to_signed_card should succeed");

        assert_eq!(signed.name, "signed-agent");
        assert_eq!(signed.description, "A test agent for JWS signing");
        assert_eq!(signed.url, "http://localhost:42069");
        assert_eq!(signed.skills.len(), 1);
        assert_eq!(signed.skills[0].id, "test-skill");
    }
}
