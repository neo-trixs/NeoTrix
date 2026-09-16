//! PatternCatalog — Catalog of recognized patterns with corresponding skills

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{BehaviorPattern, BehaviorPatternType, TrustBoundary, RouteResult};
use super::semantic_routing::{BehaviorPatternType, TrustBoundary, RouteResult};

/// Catalog of recognized behavior patterns mapped to skills
pub struct PatternCatalog {
    /// Patterns by type
    patterns_by_type: HashMap<BehaviorPatternType, Vec<BehaviorPattern>>,
    /// Patterns by skill ID
    patterns_by_skill: HashMap<String, Vec<BehaviorPattern>>,
    /// Skill to pattern mapping
    skill_patterns: HashMap<String, Vec<String>>,
    /// Registered skills and their associated patterns
    registered_skills: HashMap<String, RegisteredSkill>,
}

/// Registered skill metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredSkill {
    pub skill_id: String,
    pub skill_name: String,
    pub domain: String,
    pub tier: String,
    pub patterns: Vec<String>,
    pub manifest_path: PathBuf,
}

impl PatternCatalog {
    /// Create a new empty catalog
    pub fn new() -> Self {
        Self {
            patterns_by_type: HashMap::new(),
            patterns_by_skill: HashMap::new(),
            skill_patterns: HashMap::new(),
            registered_skills: HashMap::new(),
        }
    }

    /// Register a behavior pattern
    pub fn register(&mut self, pattern: BehaviorPattern) {
        let pattern_type = pattern.pattern_type.clone();
        self.patterns_by_type
            .entry(pattern_type.clone())
            .or_insert_with(Vec::new)
            .push(pattern.clone());

        for skill_id in &pattern.associated_skills {
            self.patterns_by_skill
                .entry(skill_id.clone())
                .or_insert_with(Vec::new)
                .push(pattern.clone());
        }
    }

    /// Register a skill with its associated patterns
    pub fn register_skill(&mut self, skill_id: String, skill_name: String, domain: String, tier: String, manifest_path: PathBuf) {
        self.registered_skills.insert(skill_id.clone(), RegisteredSkill {
            skill_id: skill_id.clone(),
            skill_name,
            domain,
            tier,
            patterns: Vec::new(),
            manifest_path,
        });
    }

    /// Add a pattern to a registered skill
    pub fn add_skill_pattern(&mut self, skill_id: &str, pattern_id: String) {
        if let Some(skill) = self.registered_skills.get_mut(skill_id) {
            skill.patterns.push(pattern_id);
        }
        self.skill_patterns
            .entry(skill_id.to_string())
            .or_insert_with(Vec::new)
            .push(pattern_id);
    }

    /// Find patterns by type
    pub fn find_by_type(&self, pattern_type: &BehaviorPatternType) -> Vec<&BehaviorPattern> {
        self.patterns_by_type.get(pattern_type)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Find patterns associated with a skill
    pub fn find_by_skill(&self, skill_id: &str) -> Vec<&BehaviorPattern> {
        self.patterns_by_skill.get(skill_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Find all patterns matching a query across all types
    pub fn search_patterns(&self, query: &str) -> Vec<&BehaviorPattern> {
        let mut results = Vec::new();
        for patterns in self.patterns_by_type.values() {
            for pattern in patterns {
                if pattern.matches(query) {
                    results.push(pattern);
                }
            }
        }
        results
    }

    /// Get all registered skills
    pub fn registered_skills(&self) -> Vec<&RegisteredSkill> {
        self.registered_skills.values().collect()
    }

    /// Get patterns for a specific skill
    pub fn skill_pattern_ids(&self, skill_id: &str) -> Vec<String> {
        self.skill_patterns.get(skill_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Build route result from pattern and skill matching
    pub fn build_route_result(&self, skill_id: &str, pattern_id: &str, score: f64) -> Option<RouteResult> {
        let skill = self.registered_skills.get(skill_id)?;
        Some(RouteResult {
            skill_id: skill_id.to_string(),
            skill_name: skill.skill_name.clone(),
            score: super::semantic_routing::SimilarityScore { value: score, method: "catalog_match".to_string() },
            policy_trace: super::semantic_routing::PolicyTrace {
                skill_id: skill_id.to_string(),
                pattern_id: pattern_id.to_string(),
                confidence: score,
                trust_boundary: TrustBoundary::Internal,
                timestamp: now_timestamp(),
                path: vec![skill_id.to_string(), pattern_id.to_string()],
            },
            references_loaded: Vec::new(),
            scripts_loaded: Vec::new(),
        })
    }

    /// Get catalog statistics
    pub fn stats(&self) -> CatalogStats {
        let total_patterns: usize = self.patterns_by_type.values().map(|v| v.len()).sum();
        CatalogStats {
            total_patterns,
            total_skills: self.registered_skills.len(),
            total_types: self.patterns_by_type.len(),
            pattern_type_breakdown: self.patterns_by_type.iter()
                .map(|(k, v)| (format!("{:?}", k), v.len()))
                .collect(),
        }
    }
}

/// Catalog statistics
#[derive(Debug, Clone, Default)]
pub struct CatalogStats {
    pub total_patterns: usize,
    pub total_skills: usize,
    pub total_types: usize,
    pub pattern_type_breakdown: HashMap<String, usize>,
}

/// Get current timestamp
fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use super::semantic_routing::BehaviorPatternType;

    #[test]
    fn test_register_and_find() {
        let mut catalog = PatternCatalog::new();
        let mut pattern = BehaviorPattern::new("p1".to_string(), "Research".to_string(), BehaviorPatternType::Research);
        pattern.add_keyword("research".to_string());
        pattern.add_skill("nt-act-research".to_string());
        catalog.register(pattern);
        let found = catalog.find_by_type(&BehaviorPatternType::Research);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_search_patterns() {
        let mut catalog = PatternCatalog::new();
        let mut pattern = BehaviorPattern::new("p1".to_string(), "Research".to_string(), BehaviorPatternType::Research);
        pattern.add_keyword("research".to_string());
        pattern.add_skill("sk1".to_string());
        catalog.register(pattern);
        let results = catalog.search_patterns("research paper");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_register_skill() {
        let mut catalog = PatternCatalog::new();
        catalog.register_skill("sk1".to_string(), "Test Skill".to_string(), "NT-ACT".to_string(), "notable".to_string(), PathBuf::from("skills/test/SKILL.md"));
        assert_eq!(catalog.registered_skills().len(), 1);
    }

    #[test]
    fn test_stats() {
        let mut catalog = PatternCatalog::new();
        catalog.register_skill("sk1".to_string(), "Skill1".to_string(), "NT-ACT".to_string(), "small".to_string(), PathBuf::from("sk1.md"));
        catalog.register_skill("sk2".to_string(), "Skill2".to_string(), "NT-WORLD".to_string(), "notable".to_string(), PathBuf::from("sk2.md"));
        let stats = catalog.stats();
        assert_eq!(stats.total_skills, 2);
    }

    #[test]
    fn test_skill_pattern_ids() {
        let mut catalog = PatternCatalog::new();
        catalog.register_skill("sk1".to_string(), "Skill".to_string(), "NT".to_string(), "small".to_string(), PathBuf::from("sk1.md"));
        catalog.add_skill_pattern("sk1", "pattern-1".to_string());
        let ids = catalog.skill_pattern_ids("sk1");
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "pattern-1");
    }
}