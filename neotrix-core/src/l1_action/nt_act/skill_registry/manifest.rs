//! SkillManifest — SKILL-SPEC.md contract (<200 lines)
//!
//! Represents the top-level skill manifest loaded first in progressive disclosure.
//! References and scripts are loaded on demand.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use super::{ContentHash, SecurityScan, SkillRegistryConfig, SkillRegistryError};

/// SKILL-SPEC.md contract representation — must be <200 lines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub id: String,
    pub domain: String,
    pub tier: SkillTier,
    pub constellation: ConstellationLevel,
    pub purpose: String,
    pub input_contract: Vec<ContractField>,
    pub output_contract: Vec<ContractField>,
    pub execution_steps: Vec<ExecutionStep>,
    pub dependencies: Vec<String>,
    pub failure_modes: Vec<FailureMode>,
    pub references: Vec<PathBuf>,
    pub scripts: Vec<PathBuf>,
    pub tests_path: Option<PathBuf>,
    pub manifest_path: PathBuf,
    pub content_hash: String,
    pub version: String,
    pub author: Option<String>,
    pub triggers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillTier {
    SmallPassive,
    NotablePassive,
    Keystone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractField {
    pub field: String,
    pub field_type: String,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step: usize,
    pub description: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMode {
    pub mode: String,
    pub detection: String,
    pub recovery: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstellationLevel {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
}

impl SkillManifest {
    pub fn new(name: String, domain: String, tier: SkillTier, constellation: ConstellationLevel) -> Self {
        let id = format!("{}-{}", domain.to_lowercase(), name);
        Self {
            name,
            id,
            domain,
            tier,
            constellation,
            purpose: String::new(),
            input_contract: Vec::new(),
            output_contract: Vec::new(),
            execution_steps: Vec::new(),
            dependencies: Vec::new(),
            failure_modes: Vec::new(),
            references: Vec::new(),
            scripts: Vec::new(),
            tests_path: None,
            manifest_path: PathBuf::new(),
            content_hash: String::new(),
            version: "0.0.0".to_string(),
            author: None,
            triggers: Vec::new(),
        }
    }

    pub fn is_within_line_limit(&self, content: &str) -> bool {
        content.lines().count() < 200
    }

    pub fn add_reference(&mut self, path: PathBuf) {
        self.references.push(path);
    }

    pub fn add_script(&mut self, path: PathBuf) {
        self.scripts.push(path);
    }

    pub fn set_content_hash(&mut self, hash: String) {
        self.content_hash = hash;
    }

    pub fn matches_trigger(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.triggers.iter().any(|t| query_lower.contains(&t.to_lowercase()))
    }

    pub fn token_budget(&self) -> usize {
        match self.tier {
            SkillTier::SmallPassive => 500,
            SkillTier::NotablePassive => 2000,
            SkillTier::Keystone => 5000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_manifest_creation() {
        let manifest = SkillManifest::new("tdd".to_string(), "NT-ACT".to_string(), SkillTier::NotablePassive, ConstellationLevel::C2);
        assert_eq!(manifest.name, "tdd");
        assert_eq!(manifest.domain, "NT-ACT");
        assert_eq!(manifest.id, "nt-act-tdd");
    }

    #[test]
    fn test_line_limit_validation() {
        let manifest = SkillManifest::new("test".to_string(), "NT-ACT".to_string(), SkillTier::SmallPassive, ConstellationLevel::C1);
        assert!(manifest.is_within_line_limit("Line 1\nLine 2"));
        let long = "# ".to_string() + &"Line\n".repeat(200);
        assert!(!manifest.is_within_line_limit(&long));
    }

    #[test]
    fn test_add_reference_and_script() {
        let mut manifest = SkillManifest::new("test".to_string(), "NT-ACT".to_string(), SkillTier::SmallPassive, ConstellationLevel::C1);
        manifest.add_reference(PathBuf::from("references/domain.md"));
        manifest.add_script(PathBuf::from("scripts/run.sh"));
        assert_eq!(manifest.references.len(), 1);
        assert_eq!(manifest.scripts.len(), 1);
    }

    #[test]
    fn test_token_budget() {
        let small = SkillManifest::new("s".to_string(), "NT".to_string(), SkillTier::SmallPassive, ConstellationLevel::C1);
        let notable = SkillManifest::new("n".to_string(), "NT".to_string(), SkillTier::NotablePassive, ConstellationLevel::C3);
        let keystone = SkillManifest::new("k".to_string(), "NT".to_string(), SkillTier::Keystone, ConstellationLevel::C5);
        assert_eq!(small.token_budget(), 500);
        assert_eq!(notable.token_budget(), 2000);
        assert_eq!(keystone.token_budget(), 5000);
    }

    #[test]
    fn test_trigger_matching() {
        let mut manifest = SkillManifest::new("tdd".to_string(), "NT-ACT".to_string(), SkillTier::NotablePassive, ConstellationLevel::C2);
        manifest.triggers = vec!["tdd".to_string(), "test".to_string()];
        assert!(manifest.matches_trigger("run tdd test"));
        assert!(!manifest.matches_trigger("unrelated"));
    }

    #[test]
    fn test_constellation_ordering() {
        assert!(ConstellationLevel::C0 < ConstellationLevel::C1);
    }

    #[test]
    fn test_tier_ordering() {
        assert!(SkillTier::SmallPassive < SkillTier::NotablePassive);
    }
}