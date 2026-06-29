use super::error::AgentError;
use std::collections::HashMap;
use std::num::NonZeroUsize;

use k256::ecdsa::{signature::Signer, signature::Verifier, Signature, SigningKey, VerifyingKey};
use lru::LruCache;
use rand::rngs::OsRng;

/// Generate a new ECDSA (k256/secp256k1) keypair.
/// Returns (secret_key_bytes, public_key_bytes).
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let sk = SigningKey::random(&mut OsRng);
    let pk = VerifyingKey::from(&sk);
    (sk.to_bytes().to_vec(), pk.to_sec1_bytes().to_vec())
}

/// Sign a message with the given secret key bytes.
pub fn sign(message: &[u8], sk_bytes: &[u8]) -> Result<Vec<u8>, AgentError> {
    let sk = SigningKey::from_slice(sk_bytes)
        .map_err(|e| AgentError::ToolExecutionFailed(format!("invalid secret key: {}", e)))?;
    let sig: Signature = sk.sign(message);
    Ok(sig.to_bytes().to_vec())
}

/// Verify a signature against a message and public key bytes (SEC1-encoded).
pub fn verify(message: &[u8], signature: &[u8], pk_bytes: &[u8]) -> bool {
    let pk = match VerifyingKey::from_sec1_bytes(pk_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };
    let sig = match Signature::from_slice(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };
    pk.verify(message, &sig).is_ok()
}

/// Generate a new identity with a fresh keypair.
pub fn generate_identity(agent_id: &str) -> (AgentIdentity, Vec<u8>) {
    let (sk, pk) = generate_keypair();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let identity = AgentIdentity {
        agent_id: agent_id.to_string(),
        public_key: pk,
        created_at: now,
        attestations: Vec::new(),
    };
    (identity, sk)
}

/// A cryptographic identity for an agent using ECDSA (k256/secp256k1).
#[derive(Debug, Clone)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub public_key: Vec<u8>,
    pub created_at: u64,
    pub attestations: Vec<Attestation>,
}

impl AgentIdentity {
    /// Recover identity from an existing secret key.
    pub fn recover(agent_id: &str, sk_bytes: &[u8]) -> Result<Self, AgentError> {
        let sk = SigningKey::from_slice(sk_bytes)
            .map_err(|e| AgentError::ToolExecutionFailed(format!("invalid key: {e}")))?;
        let pk = VerifyingKey::from(&sk);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Ok(AgentIdentity {
            agent_id: agent_id.to_string(),
            public_key: pk.to_sec1_bytes().to_vec(),
            created_at: now,
            attestations: Vec::new(),
        })
    }
}

/// A signed attestation claim.
#[derive(Debug, Clone)]
pub struct Attestation {
    pub claim: String,
    pub signature: Vec<u8>,
    pub timestamp: u64,
    pub issuer: String,
}

/// Identity statistics.
#[derive(Debug, Clone, Default)]
pub struct IdentityStats {
    pub total_identities: usize,
    pub total_attestations: usize,
    pub verified_claims: u64,
    pub failed_claims: u64,
}

/// Identity manager with verification cache.
pub struct IdentityManager {
    pub identities: HashMap<String, AgentIdentity>,
    max_attestations: usize,
    verification_cache: LruCache<(String, String), bool>,
}

impl IdentityManager {
    /// Create an empty identity manager.
    pub fn new() -> Self {
        IdentityManager {
            identities: HashMap::new(),
            max_attestations: 100,
            verification_cache: LruCache::new(NonZeroUsize::new(1024).unwrap()),
        }
    }

    /// Create an identity manager pre-loaded with an identity recovered from a secret key.
    pub fn from_key(agent_id: &str, sk_bytes: &[u8]) -> Result<Self, AgentError> {
        let identity = AgentIdentity::recover(agent_id, sk_bytes)?;
        let mut mgr = IdentityManager::new();
        mgr.add_identity(identity);
        Ok(mgr)
    }

    /// Create a signed attestation for the given claim.
    /// Returns None if the agent identity is not registered.
    pub fn attest(&mut self, agent_id: &str, claim: &str, sk: &[u8]) -> Option<Attestation> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let signature = sign(claim.as_bytes(), sk).ok()?;
        let attestation = Attestation {
            claim: claim.to_string(),
            signature,
            timestamp: now,
            issuer: agent_id.to_string(),
        };
        let identity = self.identities.get_mut(agent_id)?;
        if identity.attestations.len() < self.max_attestations {
            identity.attestations.push(attestation.clone());
        }
        Some(attestation)
    }

    /// Verify that an attestation was signed by the given agent.
    pub fn verify(&mut self, agent_id: &str, attestation: &Attestation) -> bool {
        let identity = match self.identities.get(agent_id) {
            Some(id) => id,
            None => return false,
        };
        let cache_key = (agent_id.to_string(), attestation.claim.clone());
        if let Some(result) = self.verification_cache.get(&cache_key) {
            return *result;
        }
        let result = attestation.issuer == agent_id
            && verify(
                attestation.claim.as_bytes(),
                &attestation.signature,
                &identity.public_key,
            );
        self.verification_cache.put(cache_key, result);
        result
    }

    /// Verify a chain of claims all signed by the same agent.
    /// For each claim, a matching attestation must exist in the agent's record.
    pub fn chain_verify(&self, agent_id: &str, claims: &[&str]) -> bool {
        let identity = match self.identities.get(agent_id) {
            Some(id) => id,
            None => return false,
        };
        for claim in claims {
            let found = identity.attestations.iter().any(|att| {
                att.claim == *claim
                    && att.issuer == agent_id
                    && verify(claim.as_bytes(), &att.signature, &identity.public_key)
            });
            if !found {
                return false;
            }
        }
        true
    }

    /// Register an identity.
    pub fn add_identity(&mut self, identity: AgentIdentity) {
        let agent_id = identity.agent_id.clone();
        self.identities.insert(agent_id, identity);
    }

    /// Look up an identity by agent_id.
    pub fn get_identity(&self, agent_id: &str) -> Option<&AgentIdentity> {
        self.identities.get(agent_id)
    }

    /// Identity statistics.
    pub fn stats(&self) -> IdentityStats {
        let total_identities = self.identities.len();
        let total_attestations: usize = self
            .identities
            .values()
            .map(|id| id.attestations.len())
            .sum();
        let (verified, failed) =
            self.verification_cache
                .iter()
                .fold(
                    (0u64, 0u64),
                    |(v, f), (_, &ok)| {
                        if ok {
                            (v + 1, f)
                        } else {
                            (v, f + 1)
                        }
                    },
                );
        IdentityStats {
            total_identities,
            total_attestations,
            verified_claims: verified,
            failed_claims: failed,
        }
    }
}

impl Default for IdentityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (sk, pk) = generate_keypair();
        assert_eq!(sk.len(), 32, "secret key must be 32 bytes (secp256k1)");
        assert_eq!(
            pk.len(),
            33,
            "public key must be 33 bytes (SEC1 compressed)"
        );
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        let (sk, pk) = generate_keypair();
        let msg = b"hello world";
        let sig = sign(msg, &sk).unwrap();
        assert!(verify(msg, &sig, &pk), "valid signature must verify");
    }

    #[test]
    fn test_attestation_has_valid_signature() {
        let (identity, sk) = generate_identity("agent-1");
        let mut mgr = IdentityManager::new();
        mgr.add_identity(identity);

        let att = mgr.attest("agent-1", "capability:hypergraph-rag", &sk);
        assert!(att.is_some(), "attestation must succeed");
        let att = att.unwrap();
        assert_eq!(att.claim, "capability:hypergraph-rag");
        assert_eq!(att.issuer, "agent-1");
        assert!(att.timestamp > 0, "timestamp must be set");
        // Verify the signature directly
        assert!(
            verify(
                att.claim.as_bytes(),
                &att.signature,
                &mgr.identities["agent-1"].public_key
            ),
            "attestation signature must be valid"
        );
    }

    #[test]
    fn test_verify_authentic_returns_true() {
        let (identity, sk) = generate_identity("agent-1");
        let mut mgr = IdentityManager::new();
        mgr.add_identity(identity);

        let att = mgr.attest("agent-1", "role:explorer", &sk).unwrap();
        assert!(
            mgr.verify("agent-1", &att),
            "authentic attestation must verify"
        );
    }

    #[test]
    fn test_verify_tampered_returns_false() {
        let (identity, sk) = generate_identity("agent-1");
        let mut mgr = IdentityManager::new();
        mgr.add_identity(identity);

        let mut att = mgr.attest("agent-1", "role:explorer", &sk).unwrap();
        att.claim = "role:attacker".to_string();
        assert!(
            !mgr.verify("agent-1", &att),
            "tampered attestation must fail verification"
        );
    }

    #[test]
    fn test_verify_wrong_signer_returns_false() {
        let (identity_a, sk_a) = generate_identity("agent-a");
        let (identity_b, _sk_b) = generate_identity("agent-b");
        let mut mgr = IdentityManager::new();
        mgr.add_identity(identity_a);
        mgr.add_identity(identity_b);

        let att = mgr.attest("agent-a", "role:explorer", &sk_a).unwrap();
        // Verify using agent-b's id — the attestation says issuer is agent-a, so it should fail
        assert!(
            !mgr.verify("agent-b", &att),
            "wrong agent_id must fail verification"
        );
    }

    #[test]
    fn test_add_and_retrieve_identity() {
        let (identity, _sk) = generate_identity("agent-x");
        let mut mgr = IdentityManager::new();
        mgr.add_identity(identity);

        let retrieved = mgr.get_identity("agent-x");
        assert!(retrieved.is_some(), "must retrieve by agent_id");
        assert_eq!(retrieved.unwrap().agent_id, "agent-x");
        assert_eq!(retrieved.unwrap().public_key.len(), 33);
    }

    #[test]
    fn test_stats() {
        let (id1, sk1) = generate_identity("agent-1");
        let (id2, _sk2) = generate_identity("agent-2");
        let mut mgr = IdentityManager::new();
        mgr.add_identity(id1);
        mgr.add_identity(id2);

        // No attestations yet
        let s = mgr.stats();
        assert_eq!(s.total_identities, 2);
        assert_eq!(s.total_attestations, 0);
        assert_eq!(s.verified_claims, 0);

        // Attest once
        let att = mgr.attest("agent-1", "capability:rag", &sk1).unwrap();
        mgr.verify("agent-1", &att); // populate cache

        let s = mgr.stats();
        assert_eq!(s.total_identities, 2);
        assert_eq!(s.total_attestations, 1);
        assert_eq!(s.verified_claims, 1);
    }

    #[test]
    fn test_chain_verify_valid() {
        let (identity, sk) = generate_identity("agent-1");
        let mut mgr = IdentityManager::new();
        mgr.add_identity(identity);

        mgr.attest("agent-1", "capability:rag", &sk);
        mgr.attest("agent-1", "role:explorer", &sk);
        mgr.attest("agent-1", "trust-level:high", &sk);

        assert!(
            mgr.chain_verify("agent-1", &["capability:rag", "role:explorer"]),
            "chain of valid claims must verify"
        );
        assert!(
            mgr.chain_verify(
                "agent-1",
                &["capability:rag", "role:explorer", "trust-level:high"]
            ),
            "all three claims must verify"
        );
    }

    #[test]
    fn test_chain_verify_mismatched_signer() {
        let (id_a, sk_a) = generate_identity("agent-a");
        let (id_b, _sk_b) = generate_identity("agent-b");
        let mut mgr = IdentityManager::new();
        mgr.add_identity(id_a);
        mgr.add_identity(id_b);

        mgr.attest("agent-a", "capability:rag", &sk_a);

        assert!(
            !mgr.chain_verify("agent-b", &["capability:rag"]),
            "agent-b has no attestations, chain must fail"
        );
    }

    #[test]
    fn test_attest_nonexistent_identity() {
        let mut mgr = IdentityManager::new();
        let result = mgr.attest("ghost", "claim:test", &[0u8; 32]);
        assert!(
            result.is_none(),
            "attest for unknown identity must return None"
        );
    }

    #[test]
    fn test_recover_identity_from_key() {
        let (original_id, sk) = generate_identity("agent-1");
        let recovered = AgentIdentity::recover("agent-1", &sk).unwrap();
        assert_eq!(original_id.public_key, recovered.public_key);
        assert_eq!(original_id.agent_id, recovered.agent_id);
    }

    #[test]
    fn test_verify_wrong_signature_bytes() {
        let (identity, sk) = generate_identity("alice");
        let mut mgr = IdentityManager::new();
        mgr.add_identity(identity);

        let att = mgr.attest("alice", "claim:hello", &sk).unwrap();
        let mut tampered = att.clone();
        tampered.signature = vec![0u8; 64]; // garbage signature
        assert!(
            !mgr.verify("alice", &tampered),
            "garbage signature must fail"
        );
    }

    #[test]
    fn test_stats_tracks_failures() {
        let (identity, sk) = generate_identity("alice");
        let mut mgr = IdentityManager::new();
        mgr.add_identity(identity);

        let att = mgr.attest("alice", "claim:x", &sk).unwrap();
        let mut bad = att.clone();
        bad.claim = "claim:y".to_string();
        mgr.verify("alice", &bad); // should fail

        let s = mgr.stats();
        assert!(s.failed_claims >= 1, "failed claims must be counted");
    }
}
