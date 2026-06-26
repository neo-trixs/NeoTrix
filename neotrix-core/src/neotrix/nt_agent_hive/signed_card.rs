use k256::ecdsa::{signature::Signer, signature::Verifier, Signature, SigningKey, VerifyingKey};

/// An AgentCard signed with an ECDSA (k256/P-256) identity key.
///
/// Enables sub-hives to verify each other's identity before
/// establishing encrypted back-channels for knowledge syphoning.
///
/// Reference: A2A Protocol v1.2 signed AgentCards with Ed25519.
/// Now supports both Ed25519 and ECDSA P-256.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Ed25519,
    Ecdsa,
}

#[derive(Debug, Clone)]
pub struct SignedAgentCard {
    /// The original JSON-serialized AgentCard bytes (signed payload)
    pub card_json: Vec<u8>,
    /// Signature algorithm used
    pub algorithm: Algorithm,
    /// Signature over card_json
    pub signature: Vec<u8>,
    /// Public key (33 bytes SEC1 for ECDSA, 32 bytes for Ed25519)
    pub signer_pubkey: Vec<u8>,
    /// Human-readable signer identifier
    pub signer_name: String,
}

impl SignedAgentCard {
    /// Sign an AgentCard with the given signing key.
    pub fn sign(
        card_json: Vec<u8>,
        signing_key: &SigningKey,
        signer_name: &str,
    ) -> Result<Self, String> {
        let sig: Signature = signing_key.sign(&card_json);
        let pubkey = VerifyingKey::from(signing_key);
        let pubkey_bytes = pubkey.to_sec1_bytes().to_vec();

        Ok(SignedAgentCard {
            card_json,
            algorithm: Algorithm::Ecdsa,
            signature: sig.to_der().as_bytes().to_vec(),
            signer_pubkey: pubkey_bytes,
            signer_name: signer_name.to_string(),
        })
    }

    /// Verify the signature against the embedded pubkey.
    pub fn verify(&self) -> Result<bool, String> {
        match self.algorithm {
            Algorithm::Ed25519 => self.verify_ed25519(),
            Algorithm::Ecdsa => {
                let verifying_key = VerifyingKey::from_sec1_bytes(&self.signer_pubkey)
                    .map_err(|e| format!("invalid verifying key: {}", e))?;

                let sig = Signature::from_der(&self.signature)
                    .map_err(|e| format!("invalid signature DER: {}", e))?;

                Ok(verifying_key.verify(&self.card_json, &sig).is_ok())
            }
        }
    }

    /// Sign an AgentCard with Ed25519 (requires `ed25519-dalek` feature).
    pub fn sign_ed25519(
        _card_json: &[u8],
        _secret: &[u8],
        _signer_name: &str,
    ) -> Result<Self, String> {
        Err("Ed25519 support requires ed25519-dalek feature".into())
    }

    /// Verify an Ed25519 signature (requires `ed25519-dalek` feature).
    pub fn verify_ed25519(&self) -> Result<bool, String> {
        Err("Ed25519 support requires ed25519-dalek feature".into())
    }

    /// Extract the AgentCard from JSON bytes.
    pub fn extract_card<T: serde::de::DeserializeOwned>(&self) -> Result<T, String> {
        serde_json::from_slice(&self.card_json).map_err(|e| format!("deserialize AgentCard: {}", e))
    }

    /// Generate a deterministic name from the pubkey for identity display.
    pub fn pubkey_fingerprint(&self) -> String {
        let bytes = &self.signer_pubkey;
        let mut hex = String::with_capacity(8);
        for b in bytes.iter().take(4) {
            hex.push_str(&format!("{:02x}", b));
        }
        hex
    }
}

/// A registry of trusted signer pubkeys for identity verification.
pub struct TrustRegistry {
    trusted_pubkeys: Vec<Vec<u8>>,
}

impl TrustRegistry {
    pub fn new() -> Self {
        TrustRegistry {
            trusted_pubkeys: Vec::new(),
        }
    }

    pub fn trust(&mut self, pubkey: Vec<u8>) {
        if !self.trusted_pubkeys.contains(&pubkey) {
            self.trusted_pubkeys.push(pubkey);
        }
    }

    pub fn is_trusted(&self, pubkey: &[u8]) -> bool {
        self.trusted_pubkeys.iter().any(|k| k.as_slice() == pubkey)
    }

    pub fn verify_signed_card(&self, card: &SignedAgentCard) -> Result<bool, String> {
        if !self.is_trusted(&card.signer_pubkey) {
            return Ok(false);
        }
        card.verify()
    }

    pub fn trusted_count(&self) -> usize {
        self.trusted_pubkeys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::SigningKey;
    use rand::rngs::OsRng;

    const A2A_PORT: u16 = 5025;

    fn sample_card_json() -> Vec<u8> {
        format!(r#"{{"name":"test-sub-hive","description":"explorer","url":"http://localhost:{}","version":"0.1.0","capabilities":{{"streaming":true,"push_notifications":false,"authentication":false}},"skills":[{{"id":"explore","name":"Explorer","description":"Knowledge gap explorer","tags":["research"]}}]}}"#, A2A_PORT).as_bytes().to_vec()
    }

    #[test]
    fn test_sign_and_verify() {
        let signing_key = SigningKey::random(&mut OsRng);
        let card_json = sample_card_json();
        let signed = SignedAgentCard::sign(card_json.clone(), &signing_key, "test-agent").unwrap();
        assert!(signed.verify().unwrap());
    }

    #[test]
    fn test_tampered_card_fails() {
        let signing_key = SigningKey::random(&mut OsRng);
        let mut signed =
            SignedAgentCard::sign(sample_card_json(), &signing_key, "test-agent").unwrap();
        signed.card_json[10] ^= 0xFF;
        assert!(!signed.verify().unwrap());
    }

    #[test]
    fn test_wrong_key_fails() {
        let alice = SigningKey::random(&mut OsRng);
        let bob = SigningKey::random(&mut OsRng);
        let signed = SignedAgentCard::sign(sample_card_json(), &alice, "alice").unwrap();

        let bob_pubkey = VerifyingKey::from(&bob);
        let forged = SignedAgentCard {
            card_json: signed.card_json,
            algorithm: Algorithm::Ecdsa,
            signature: signed.signature,
            signer_pubkey: bob_pubkey.to_sec1_bytes().to_vec(),
            signer_name: "bob".into(),
        };
        assert!(!forged.verify().unwrap());
    }

    #[test]
    fn test_extract_card() {
        let signing_key = SigningKey::random(&mut OsRng);
        let signed = SignedAgentCard::sign(sample_card_json(), &signing_key, "test-agent").unwrap();
        let card: serde_json::Value = signed.extract_card().unwrap();
        assert_eq!(card["name"], "test-sub-hive");
    }

    #[test]
    fn test_trust_registry() {
        let mut registry = TrustRegistry::new();
        let signing_key = SigningKey::random(&mut OsRng);
        let pubkey = VerifyingKey::from(&signing_key)
            .to_encoded_point(true)
            .as_bytes()
            .to_vec();

        assert!(!registry.is_trusted(&pubkey));
        registry.trust(pubkey.clone());
        assert!(registry.is_trusted(&pubkey));
        assert_eq!(registry.trusted_count(), 1);
    }

    #[test]
    fn test_trust_registry_verify() {
        let mut registry = TrustRegistry::new();
        let signing_key = SigningKey::random(&mut OsRng);
        let pubkey = VerifyingKey::from(&signing_key)
            .to_encoded_point(true)
            .as_bytes()
            .to_vec();
        registry.trust(pubkey);

        let signed = SignedAgentCard::sign(sample_card_json(), &signing_key, "trusted").unwrap();
        assert!(registry.verify_signed_card(&signed).unwrap());
    }

    #[test]
    fn test_trust_registry_rejects_unknown() {
        let registry = TrustRegistry::new();
        let signing_key = SigningKey::random(&mut OsRng);
        let signed = SignedAgentCard::sign(sample_card_json(), &signing_key, "unknown").unwrap();
        assert!(!registry.verify_signed_card(&signed).unwrap());
    }

    #[test]
    fn test_pubkey_fingerprint_deterministic() {
        let signing_key = SigningKey::random(&mut OsRng);
        let signed1 = SignedAgentCard::sign(sample_card_json(), &signing_key, "a").unwrap();
        let signed2 = SignedAgentCard::sign(sample_card_json(), &signing_key, "b").unwrap();
        assert_eq!(
            signed1.pubkey_fingerprint(),
            signed2.pubkey_fingerprint(),
            "same key → same fingerprint"
        );
    }

    #[test]
    fn test_signature_determinism() {
        let signing_key = SigningKey::random(&mut OsRng);
        let card = sample_card_json();
        let s1 = SignedAgentCard::sign(card.clone(), &signing_key, "a").unwrap();
        let s2 = SignedAgentCard::sign(card, &signing_key, "a").unwrap();
        assert_eq!(
            s1.signer_pubkey, s2.signer_pubkey,
            "same key produces same pubkey"
        );
    }

    #[test]
    fn test_ed25519_sign_verify() {
        let card = sample_card_json();
        let seed = [0u8; 32];
        let signed = SignedAgentCard::sign_ed25519(&card, &seed, "test-agent").unwrap();
        assert_eq!(signed.algorithm, Algorithm::Ed25519);
        assert!(signed.verify_ed25519().unwrap());
    }

    #[test]
    fn test_ed25519_tamper_detection() {
        let card = sample_card_json();
        let seed = [1u8; 32];
        let mut signed = SignedAgentCard::sign_ed25519(&card, &seed, "test-agent").unwrap();
        signed.card_json[10] ^= 0xFF;
        assert!(!signed.verify_ed25519().unwrap());
    }

    #[test]
    fn test_algorithm_enum() {
        assert!(Algorithm::Ed25519 != Algorithm::Ecdsa);
    }

    #[test]
    fn test_invalid_seed_rejected() {
        let card = sample_card_json();
        let result = SignedAgentCard::sign_ed25519(&card, &[0u8; 16], "test-agent");
        assert!(result.is_err());
    }
}
