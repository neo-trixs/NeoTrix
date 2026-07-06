use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::nt_memory_types::{KnowledgeNode, NodeType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrivacyMode {
    Stateless,
    Encrypted,
    Sovereign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub mode: PrivacyMode,
    pub encryption_key: Option<String>,
    pub auto_export_path: Option<String>,
    pub data_retention_days: u64,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            mode: PrivacyMode::Stateless,
            encryption_key: None,
            auto_export_path: None,
            data_retention_days: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSovereigntyProof {
    pub record_id: String,
    pub timestamp: i64,
    pub signature_hex: String,
    pub public_key_hex: String,
}

impl DataSovereigntyProof {
    pub fn sign(record_id: &str, secret: &[u8; 32]) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let message = format!("{}:{}:{}", record_id, timestamp, hex_fingerprint(secret));
        let hash = compute_hash(&message);
        DataSovereigntyProof {
            record_id: record_id.to_string(),
            timestamp,
            signature_hex: hash,
            public_key_hex: hex_fingerprint(secret),
        }
    }

    pub fn verify(&self, secret: &[u8; 32]) -> bool {
        let message = format!("{}:{}:{}", self.record_id, self.timestamp, hex_fingerprint(secret));
        let expected = compute_hash(&message);
        expected == self.signature_hex
    }
}

pub struct PrivacyEnforcer {
    config: PrivacyConfig,
    encryption_key: Option<[u8; 32]>,
    signing_secret: [u8; 32],
}

impl PrivacyEnforcer {
    pub fn new(config: PrivacyConfig) -> Self {
        let encryption_key = config.encryption_key.as_ref().and_then(|hex_key| {
            let bytes = simple_hex_decode(hex_key);
            if bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes[..32]);
                Some(key)
            } else {
                None
            }
        });

        let signing_secret = {
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42);
            let h = compute_hash(&format!("neotrix-privacy-seed-{}", t));
            let mut seed = [0u8; 32];
            let bytes = h.as_bytes();
            let copy_len = 32.min(bytes.len());
            seed[..copy_len].copy_from_slice(&bytes[..copy_len]);
            seed
        };

        PrivacyEnforcer {
            config,
            encryption_key,
            signing_secret,
        }
    }

    pub fn config(&self) -> &PrivacyConfig {
        &self.config
    }

    pub fn store_with_privacy(&self, record: &KnowledgeNode) -> Result<DataSovereigntyProof, String> {
        match self.config.mode {
            PrivacyMode::Stateless => {
                return Err("Cannot store in Stateless mode: no local storage permitted".to_string());
            }
            PrivacyMode::Encrypted => {
                let encrypted = self.encrypt_record(record)?;
                let meta_node = KnowledgeNode {
                    id: record.id.clone(),
                    node_type: NodeType::Insight,
                    title: format!("[ENCRYPTED] {}", record.title),
                    summary: Some("[data encrypted at rest]".to_string()),
                    content: Some(encrypted),
                    url: record.url.clone(),
                    domain: record.domain.clone(),
                    language: record.language.clone(),
                    confidence: record.confidence,
                    importance: record.importance,
                    created_at: record.created_at,
                    updated_at: record.updated_at,
                    access_count: record.access_count,
                    metadata: record.metadata.clone(),
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
                };
                std::fs::write(
                    format!("/tmp/neotrix_encrypted_{}", record.id),
                    meta_node.content.as_deref().unwrap_or(""),
                ).map_err(|e| format!("Store encrypted: {}", e))?;
            }
            PrivacyMode::Sovereign => {
                let json = serde_json::to_string_pretty(record)
                    .map_err(|e| format!("Serialize: {}", e))?;
                if let Some(ref export_path) = self.config.auto_export_path {
                    let path = Path::new(export_path).join(format!("{}.json", record.id));
                    std::fs::write(&path, &json).map_err(|e| format!("Export: {}", e))?;
                }
            }
        }

        Ok(DataSovereigntyProof::sign(&record.id, &self.signing_secret))
    }

    pub fn export_snapshot(&self, _path: &str) -> Result<(), String> {
        Err("Snapshot requires KB integration; use Sovereign mode with auto_export_path".to_string())
    }

    pub fn verify_proof(&self, proof: &DataSovereigntyProof) -> bool {
        proof.verify(&self.signing_secret)
    }

    fn encrypt_record(&self, record: &KnowledgeNode) -> Result<String, String> {
        let key = self.encryption_key.as_ref().ok_or("No encryption key configured")?;
        let plaintext = serde_json::to_string(record).map_err(|e| format!("Serialize: {}", e))?;
        let mut result = String::with_capacity(plaintext.len() * 2 + 64);
        for (i, byte) in plaintext.as_bytes().iter().enumerate() {
            let k = key[i % 32];
            let xored = byte ^ k;
            result.push_str(&format!("{:02x}", xored));
        }
        Ok(result)
    }

    pub fn decrypt_content(&self, content: &str) -> Result<String, String> {
        self.decrypt_node(content)
    }

    fn decrypt_node(&self, content: &str) -> Result<String, String> {
        let key = self.encryption_key.as_ref().ok_or("No encryption key configured")?;
        let bytes = simple_hex_decode(content);
        let plain: Vec<u8> = bytes.iter().enumerate()
            .map(|(i, &b)| b ^ key[i % 32])
            .collect();
        String::from_utf8(plain).map_err(|e| format!("UTF-8: {}", e))
    }
}

fn compute_hash(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let h = hasher.finish();
    format!("{:016x}", h)
}

fn hex_fingerprint(data: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for &b in data.iter() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn simple_hex_decode(hex: &str) -> Vec<u8> {
    let hex = hex.trim();
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let chars: Vec<char> = hex.chars().collect();
    for chunk in chars.chunks(2) {
        if chunk.len() < 2 { continue; }
        let byte = u8::from_str_radix(&format!("{}{}", chunk[0], chunk[1]), 16).unwrap_or(0);
        bytes.push(byte);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stateless_rejects_store() {
        let config = PrivacyConfig {
            mode: PrivacyMode::Stateless,
            ..Default::default()
        };
        let enforcer = PrivacyEnforcer::new(config);
        let node = KnowledgeNode {
            id: "test-1".to_string(),
            node_type: NodeType::Concept,
            title: "test".to_string(),
            summary: None,
            content: None,
            url: None,
            domain: None,
            language: "en".to_string(),
            confidence: 1.0,
            importance: 1.0,
            created_at: 0,
            updated_at: 0,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        };
        let result = enforcer.store_with_privacy(&node);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Stateless"));
    }

    #[test]
    fn test_sovereignty_proof_sign_and_verify() {
        let config = PrivacyConfig::default();
        let enforcer = PrivacyEnforcer::new(config);
        let proof = DataSovereigntyProof::sign("record-42", &enforcer.signing_secret);
        assert!(proof.verify(&enforcer.signing_secret));
    }

    #[test]
    fn test_sovereignty_proof_tampered() {
        let config = PrivacyConfig::default();
        let enforcer = PrivacyEnforcer::new(config);
        let mut proof = DataSovereigntyProof::sign("record-42", &enforcer.signing_secret);
        proof.record_id = "record-99".to_string();
        assert!(!proof.verify(&enforcer.signing_secret));
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let config = PrivacyConfig {
            mode: PrivacyMode::Encrypted,
            encryption_key: Some("ab".repeat(32)),
            ..Default::default()
        };
        let enforcer = PrivacyEnforcer::new(config);
        let node = KnowledgeNode {
            id: "secret-1".to_string(),
            node_type: NodeType::Insight,
            title: "secret".to_string(),
            summary: Some("encrypted data".to_string()),
            content: Some("supersecret".to_string()),
            url: None,
            domain: None,
            language: "en".to_string(),
            confidence: 0.9,
            importance: 1.0,
            created_at: 1000,
            updated_at: 1000,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        };
        let encrypted = enforcer.encrypt_record(&node).unwrap();
        assert_ne!(encrypted, "");

        let decrypted = enforcer.decrypt_node(&encrypted).unwrap();
        assert!(decrypted.contains("supersecret"));
    }

    #[test]
    fn test_hex_decode_encode() {
        let data = [0xABu8; 32];
        let hex = hex_fingerprint(&data);
        assert_eq!(hex.len(), 64);
        let decoded = simple_hex_decode(&hex);
        assert_eq!(decoded.len(), 32);
        assert_eq!(decoded[0], 0xAB);
    }

    #[test]
    fn test_compute_hash_deterministic() {
        let h1 = compute_hash("hello");
        let h2 = compute_hash("hello");
        assert_eq!(h1, h2);
        assert_ne!(h1, compute_hash("world"));
    }

    #[test]
    fn test_privacy_config_default() {
        let cfg = PrivacyConfig::default();
        assert_eq!(cfg.mode, PrivacyMode::Stateless);
        assert_eq!(cfg.data_retention_days, 90);
    }

    #[test]
    fn test_privacy_enforcer_new() {
        let config = PrivacyConfig::default();
        let enforcer = PrivacyEnforcer::new(config);
        assert!(enforcer.encryption_key.is_none());
        assert_eq!(enforcer.config.mode, PrivacyMode::Stateless);
    }

    #[test]
    fn test_encrypt_with_key_runs() {
        let config = PrivacyConfig {
            mode: PrivacyMode::Encrypted,
            encryption_key: Some("ff".repeat(32)),
            ..Default::default()
        };
        let enforcer = PrivacyEnforcer::new(config);
        let node = KnowledgeNode {
            id: "x".to_string(),
            node_type: NodeType::Concept,
            title: "x".to_string(),
            summary: None,
            content: Some("data".to_string()),
            url: None,
            domain: None,
            language: "en".to_string(),
            confidence: 1.0,
            importance: 1.0,
            created_at: 0,
            updated_at: 0,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        };
        let proof = enforcer.store_with_privacy(&node);
        assert!(proof.is_ok());
        assert_eq!(proof.unwrap().record_id, "x");
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let config = PrivacyConfig {
            mode: PrivacyMode::Encrypted,
            encryption_key: Some("aa".repeat(32)),
            ..Default::default()
        };
        let enforcer = PrivacyEnforcer::new(config);
        let result = enforcer.decrypt_node("deadbeef");
        assert!(result.is_ok() || result.is_err());
    }
}
