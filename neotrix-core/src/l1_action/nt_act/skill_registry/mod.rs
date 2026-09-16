//! Secure Skill Registry — NT-ACT
//!
//! Implements progressive disclosure for skill loading:
//! - SKILL-SPEC.md (top-level contract, <200 lines) loads first
//! - references/ and scripts/ loaded on demand
//! - Content hashing, security scanning, lockfile integrity
//!
//! Domain: NT-ACT (行动执行者)
//! Layer: L1 Action

pub mod manifest;
pub mod registry;
pub mod loader;
pub mod validator;

pub use manifest::SkillManifest;
pub use registry::SkillRegistry;
pub use loader::SkillLoader;
pub use validator::SkillValidator;

use std::path::PathBuf;
use sha2::{Digest, Sha256};

/// Security scan result for skill content
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityScan {
    pub accepted: bool,
    pub rejected: bool,
    pub flags: Vec<SecurityFlag>,
}

/// A single security flag detected in skill content
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityFlag {
    pub category: String,
    pub pattern: String,
    pub reason: String,
    pub severity: SecuritySeverity,
}

/// Security severity levels
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Info,
    Warning,
    Critical,
}

/// Content hash for integrity verification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash {
    pub algorithm: String,
    pub value: String,
    pub size_bytes: u64,
}

impl ContentHash {
    pub fn sha256(data: &[u8]) -> Self {
        let hash = Sha256::digest(data);
        let size = data.len() as u64;
        ContentHash {
            algorithm: "sha256".to_string(),
            value: hex::encode(hash),
            size_bytes: size,
        }
    }
}

/// Lockfile entry for skill integrity
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillLockfile {
    pub version: String,
    pub skills: Vec<SkillLockfileEntry>,
}

/// Single entry in the skill lockfile
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillLockfileEntry {
    pub skill_id: String,
    pub content_hash: String,
    pub manifest_path: PathBuf,
    pub references: Vec<PathBuf>,
    pub scripts: Vec<PathBuf>,
    pub pinned_version: String,
}

/// Default security rule set for skill content scanning
#[derive(Debug, Clone)]
pub struct SecurityRules {
    rules: Vec<SecurityRule>,
}

/// A single security rule
#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub id: String,
    pub category: String,
    pub pattern: regex::Regex,
    pub reason: String,
    pub severity: SecuritySeverity,
}

impl SecurityRules {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: SecurityRule) {
        self.rules.push(rule);
    }

    /// Scan skill content against all rules
    pub fn scan(&self, content: &str, title: &str, description: &str) -> SecurityScan {
        let mut flags = Vec::new();
        let mut rejected = false;
        let combined = format!("{}\n{}\n{}", title, description, content);

        for rule in &self.rules {
            if rule.pattern.is_match(&combined) {
                flags.push(SecurityFlag {
                    category: rule.category.clone(),
                    pattern: rule.id.clone(),
                    reason: rule.reason.clone(),
                    severity: rule.severity.clone(),
                });
                if rule.severity == SecuritySeverity::Critical {
                    rejected = true;
                }
            }
        }

        SecurityScan {
            accepted: !rejected && !flags.iter().any(|f| f.severity == SecuritySeverity::Critical),
            rejected,
            flags,
        }
    }
}

/// Skill registry configuration
#[derive(Debug, Clone)]
pub struct SkillRegistryConfig {
    pub skills_dir: PathBuf,
    pub lockfile_path: PathBuf,
    pub security_rules: SecurityRules,
    pub enable_progressive_disclosure: bool,
    pub cache_size_limit: usize,
}

impl Default for SkillRegistryConfig {
    fn default() -> Self {
        SkillRegistryConfig {
            skills_dir: PathBuf::from("skills"),
            lockfile_path: PathBuf::from("skills/.ntx-manifest.json"),
            security_rules: SecurityRules::new(),
            enable_progressive_disclosure: true,
            cache_size_limit: 128,
        }
    }
}

/// Error types for skill registry operations
#[derive(Debug, thiserror::Error)]
pub enum SkillRegistryError {
    #[error("Security scan rejected skill: {0}")]
    SecurityRejected(String),
    #[error("Manifest validation failed: {0}")]
    ManifestValidation(String),
    #[error("Content hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("Lockfile integrity check failed: {0}")]
    LockfileIntegrity(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_content_hash() {
        let data = b"test skill content";
        let hash = ContentHash::sha256(data);
        assert_eq!(hash.size_bytes, 18);
        assert_eq!(hash.algorithm, "sha256");
        assert_eq!(hash.value.len(), 64);
    }

    #[test]
    fn test_security_scan_accepted() {
        let rules = SecurityRules::new();
        let scan = rules.scan("safe content", "Test Skill", "A safe skill");
        assert!(scan.accepted);
        assert!(!scan.rejected);
        assert!(scan.flags.is_empty());
    }

    #[test]
    fn test_security_scan_with_rules() {
        let mut rules = SecurityRules::new();
        rules.add_rule(SecurityRule {
            id: "R001".to_string(),
            category: "code-injection".to_string(),
            pattern: regex::Regex::new(r"exec\(").unwrap(),
            reason: "Potential code injection".to_string(),
            severity: SecuritySeverity::Critical,
        });
        let scan = rules.scan("exec('rm -rf /')", "Evil", "Bad skill");
        assert!(scan.rejected);
        assert!(!scan.accepted);
        assert_eq!(scan.flags.len(), 1);
    }

    #[test]
    fn test_lockfile_roundtrip() {
        let dir = tempdir().unwrap();
        let lockfile = SkillLockfile {
            version: "1.0.0".to_string(),
            skills: vec![SkillLockfileEntry {
                skill_id: "test_skill".to_string(),
                content_hash: "abc123".to_string(),
                manifest_path: dir.path().join("SKILL.md"),
                references: vec![],
                scripts: vec![],
                pinned_version: "1.0.0".to_string(),
            }],
        };
        let json = serde_json::to_string(&lockfile).unwrap();
        let parsed: SkillLockfile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.skills.len(), 1);
        assert_eq!(parsed.skills[0].skill_id, "test_skill");
    }

    #[test]
    fn test_hash_mismatch_detection() {
        let data1 = b"content A";
        let data2 = b"content B";
        let h1 = ContentHash::sha256(data1);
        let h2 = ContentHash::sha256(data2);
        assert_ne!(h1, h2);
        assert_eq!(h1.value.len(), 64);
        assert_eq!(h2.value.len(), 64);
    }

    #[test]
    fn test_security_severity_ordering() {
        assert!(SecuritySeverity::Info < SecuritySeverity::Warning);
        assert!(SecuritySeverity::Warning < SecuritySeverity::Critical);
    }

    #[test]
    fn test_default_config() {
        let config = SkillRegistryConfig::default();
        assert_eq!(config.skills_dir, PathBuf::from("skills"));
        assert!(config.enable_progressive_disclosure);
        assert_eq!(config.cache_size_limit, 128);
    }
}