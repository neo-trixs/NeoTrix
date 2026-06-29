use std::collections::HashMap;

use base64::Engine as _;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

/// A2A v1.2 signed Agent Card — JWT-wrapped Agent Card
/// Per the A2A v1.2 spec (Linux Foundation governance, June 2025):
/// Agent Cards MUST be signed with HMAC-SHA256 and wrapped in JWT format.
pub const A2A_V12_PROTOCOL_VERSION: &str = "1.2";

// ── JWT Components ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
}

impl JwtHeader {
    pub fn hs256(kid: Option<String>) -> Self {
        Self {
            alg: "HS256".into(),
            typ: "JWT".into(),
            kid,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCardPayload {
    pub name: String,
    pub description: String,
    pub url: String,
    pub version: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub auth: AuthConfig,
    #[serde(default)]
    pub skills: Vec<SkillEntryV12>,
    pub expires_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub scheme: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            scheme: "bearer".into(),
            credentials: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntryV12 {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub input_schema: HashMap<String, String>,
    #[serde(default)]
    pub output_schema: HashMap<String, String>,
}

/// Full JWT-signed Agent Card — the wire format for A2A v1.2 discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAgentCardV12 {
    pub header: JwtHeader,
    pub payload: AgentCardPayload,
    pub signature: String,
}

impl SignedAgentCardV12 {
    pub fn sign(payload: AgentCardPayload, secret: &[u8]) -> Result<String, String> {
        let header = JwtHeader::hs256(None);
        let header_json =
            serde_json::to_string(&header).map_err(|e| format!("serialize header: {e}"))?;
        let payload_json =
            serde_json::to_string(&payload).map_err(|e| format!("serialize payload: {e}"))?;

        let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header_b64 = b64.encode(header_json.as_bytes());
        let payload_b64 = b64.encode(payload_json.as_bytes());

        let signing_input = format!("{header_b64}.{payload_b64}");

        type HmacSha256 = Hmac<Sha256>;
        let mut mac =
            HmacSha256::new_from_slice(secret).map_err(|e| format!("invalid key length: {e}"))?;
        mac.update(signing_input.as_bytes());
        let sig = mac.finalize().into_bytes();
        let sig_b64 = b64.encode(&sig);

        let jwt = format!("{signing_input}.{sig_b64}");
        Ok(jwt)
    }

    pub fn verify(jwt: &str, secret: &[u8]) -> Result<Self, String> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err("JWT must have 3 dot-separated parts".into());
        }

        let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;

        let header_bytes = b64
            .decode(parts[0])
            .map_err(|e| format!("decode header: {e}"))?;
        let payload_bytes = b64
            .decode(parts[1])
            .map_err(|e| format!("decode payload: {e}"))?;

        let header: JwtHeader =
            serde_json::from_slice(&header_bytes).map_err(|e| format!("parse header: {e}"))?;
        let payload: AgentCardPayload =
            serde_json::from_slice(&payload_bytes).map_err(|e| format!("parse payload: {e}"))?;

        let signing_input = format!("{}.{}", parts[0], parts[1]);

        type HmacSha256 = Hmac<Sha256>;
        let mut mac =
            HmacSha256::new_from_slice(secret).map_err(|e| format!("invalid key length: {e}"))?;
        mac.update(signing_input.as_bytes());
        let expected_sig = mac.finalize().into_bytes();
        let expected_b64 = b64.encode(&expected_sig);

        let sig_match = constant_time_eq(parts[2].as_bytes(), expected_b64.as_bytes());

        if !sig_match {
            return Err("signature mismatch".into());
        }

        Ok(Self {
            header,
            payload,
            signature: parts[2].to_string(),
        })
    }

    pub fn to_jwt_string(&self) -> String {
        let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header_json = serde_json::to_string(&self.header).unwrap_or_default();
        let payload_json = serde_json::to_string(&self.payload).unwrap_or_default();
        let header_b64 = b64.encode(header_json.as_bytes());
        let payload_b64 = b64.encode(payload_json.as_bytes());
        format!("{header_b64}.{payload_b64}.{}", self.signature)
    }

    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.payload.expires_at
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

// ── Builder helpers ─────────────────────────────────────────────────────────

pub fn build_agent_card_payload(
    name: &str,
    description: &str,
    url: &str,
    version: &str,
    capabilities: Vec<String>,
    skills: Vec<SkillEntryV12>,
    auth: AuthConfig,
    ttl_secs: u64,
) -> AgentCardPayload {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    AgentCardPayload {
        name: name.to_string(),
        description: description.to_string(),
        url: url.to_string(),
        version: version.to_string(),
        capabilities,
        auth,
        skills,
        expires_at: now + ttl_secs,
        iss: Some(name.to_string()),
        sub: Some(name.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload() -> AgentCardPayload {
        AgentCardPayload {
            name: "NeoTrix".into(),
            description: "Cognitive agent".into(),
            url: "http://localhost:42072".into(),
            version: "1.2".into(),
            capabilities: vec!["streaming".into(), "a2a-v12".into()],
            auth: AuthConfig {
                scheme: "bearer".into(),
                credentials: None,
            },
            skills: vec![],
            expires_at: 9999999999,
            iss: Some("NeoTrix".into()),
            sub: Some("NeoTrix".into()),
        }
    }

    #[test]
    fn test_jwt_sign_verify_roundtrip() {
        let secret = b"test-secret-key-32-bytes-long!!";
        let payload = sample_payload();
        let jwt = SignedAgentCardV12::sign(payload, secret).expect("sign");
        let verified = SignedAgentCardV12::verify(&jwt, secret).expect("verify");
        assert_eq!(verified.payload.name, "NeoTrix");
        assert_eq!(verified.header.alg, "HS256");
        assert_eq!(verified.header.typ, "JWT");
    }

    #[test]
    fn test_jwt_tampered_fails() {
        let secret = b"test-secret-key-32-bytes-long!!";
        let payload = sample_payload();
        let jwt = SignedAgentCardV12::sign(payload, secret).expect("sign");
        let mut bytes: Vec<u8> = jwt.into_bytes();
        bytes[10] ^= 0xFF;
        let tampered = String::from_utf8(bytes).unwrap();
        let result = SignedAgentCardV12::verify(&tampered, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_wrong_secret_fails() {
        let payload = sample_payload();
        let jwt = SignedAgentCardV12::sign(payload, b"correct-key-32-bytes-long!!!").expect("sign");
        let result = SignedAgentCardV12::verify(&jwt, b"wrong-key-32-bytes-long!!!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_to_string_roundtrip() {
        let secret = b"test-secret-key-32-bytes-long!!";
        let payload = sample_payload();
        let jwt = SignedAgentCardV12::sign(payload, secret).expect("sign");
        let card = SignedAgentCardV12::verify(&jwt, secret).expect("verify");
        let jwt2 = card.to_jwt_string();
        let card2 = SignedAgentCardV12::verify(&jwt2, secret).expect("verify");
        assert_eq!(card2.payload.name, "NeoTrix");
    }

    #[test]
    fn test_jwt_malformed() {
        let result = SignedAgentCardV12::verify("not.a.jwt", b"key");
        assert!(result.is_err());
        let result = SignedAgentCardV12::verify("too.many.parts.here", b"key");
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_card() {
        let payload = AgentCardPayload {
            expires_at: 1,
            ..sample_payload()
        };
        let jwt =
            SignedAgentCardV12::sign(payload, b"test-secret-key-32-bytes-long!!").expect("sign");
        let card =
            SignedAgentCardV12::verify(&jwt, b"test-secret-key-32-bytes-long!!").expect("verify");
        assert!(card.is_expired());
    }

    #[test]
    fn test_build_agent_card_payload() {
        let skills = vec![SkillEntryV12 {
            id: "s1".into(),
            name: "Test".into(),
            description: "A test skill".into(),
            input_schema: [("text".into(), "string".into())].into(),
            output_schema: [("result".into(), "string".into())].into(),
        }];
        let payload = build_agent_card_payload(
            "test-agent",
            "desc",
            "http://localhost:9999",
            "1.2",
            vec!["streaming".into()],
            skills,
            AuthConfig::default(),
            3600,
        );
        assert_eq!(payload.name, "test-agent");
        assert_eq!(payload.skills.len(), 1);
        assert_eq!(payload.skills[0].input_schema["text"], "string");
    }
}
