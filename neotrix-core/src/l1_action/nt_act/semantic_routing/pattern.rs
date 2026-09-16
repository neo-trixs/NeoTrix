//! BehaviorPattern — Semantic patterns (fan-in queues, policy traces, trust boundaries)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::{BehaviorPatternType, TrustBoundary, SimilarityScore, PolicyTrace, FanInQueue, PatternRequest, RouteResult};
use super::semantic_routing::BehaviorPatternType;

/// A semantic behavior pattern recognized in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    /// Unique pattern identifier
    pub id: String,
    /// Pattern name/description
    pub name: String,
    /// Type of behavior pattern
    pub pattern_type: BehaviorPatternType,
    /// Keywords that trigger this pattern
    pub keywords: Vec<String>,
    /// Semantic embeddings for similarity matching
    pub embedding: Option<Vec<f64>>,
    /// Associated skill IDs
    pub associated_skills: Vec<String>,
    /// Trust boundary for execution
    pub trust_boundary: TrustBoundary,
    /// Priority score
    pub priority: u32,
    /// Policy trace metadata
    pub policy_trace: Option<PolicyTrace>,
    /// Fan-in queue configuration
    pub fan_in_queue: Option<FanInQueue>,
    /// Metadata for routing decisions
    pub metadata: HashMap<String, String>,
}

impl BehaviorPattern {
    /// Create a new behavior pattern
    pub fn new(id: String, name: String, pattern_type: BehaviorPatternType) -> Self {
        Self {
            id,
            name,
            pattern_type,
            keywords: Vec::new(),
            embedding: None,
            associated_skills: Vec::new(),
            trust_boundary: TrustBoundary::Internal,
            priority: 0,
            policy_trace: None,
            fan_in_queue: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a keyword trigger
    pub fn add_keyword(&mut self, keyword: String) {
        self.keywords.push(keyword);
    }

    /// Add an associated skill
    pub fn add_skill(&mut self, skill_id: String) {
        self.associated_skills.push(skill_id);
    }

    /// Compute similarity score against a query
    pub fn compute_similarity(&self, query: &str) -> SimilarityScore {
        let query_lower = query.to_lowercase();
        let keyword_matches = self.keywords.iter()
            .filter(|kw| query_lower.contains(&kw.to_lowercase()))
            .count();
        let score = if keyword_matches > 0 {
            (keyword_matches as f64 / self.keywords.len().max(1) as f64) * 0.7
                + 0.3 * self.priority as f64 / 10.0
        } else {
            0.0
        };
        SimilarityScore {
            value: score.min(1.0),
            method: "keyword_match".to_string(),
        }
    }

    /// Check if this pattern matches a query
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.keywords.iter().any(|kw| query_lower.contains(&kw.to_lowercase()))
    }

    /// Get the primary skill ID for this pattern
    pub fn primary_skill(&self) -> Option<&str> {
        self.associated_skills.first().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_creation() {
        let mut pattern = BehaviorPattern::new("p1".to_string(), "Test Pattern".to_string(), BehaviorPatternType::Research);
        pattern.add_keyword("research".to_string());
        pattern.add_keyword("analyze".to_string());
        assert_eq!(pattern.name, "Test Pattern");
        assert_eq!(pattern.keywords.len(), 2);
    }

    #[test]
    fn test_pattern_matching() {
        let mut pattern = BehaviorPattern::new("p1".to_string(), "Research".to_string(), BehaviorPatternType::Research);
        pattern.add_keyword("research".to_string());
        pattern.add_keyword("analyze".to_string());
        assert!(pattern.matches("research paper"));
        assert!(!pattern.matches("cooking recipe"));
    }

    #[test]
    fn test_similarity_score() {
        let mut pattern = BehaviorPattern::new("p1".to_string(), "Research".to_string(), BehaviorPatternType::Research);
        pattern.add_keyword("research".to_string());
        pattern.add_keyword("analyze".to_string());
        pattern.priority = 5;
        let score = pattern.compute_similarity("research analysis");
        assert!(score.value > 0.0);
        assert!(score.value <= 1.0);
    }

    #[test]
    fn test_add_skill() {
        let mut pattern = BehaviorPattern::new("p1".to_string(), "Test".to_string(), BehaviorPatternType::Sequential);
        pattern.add_skill("nt-act-skill1".to_string());
        pattern.add_skill("nt-act-skill2".to_string());
        assert_eq!(pattern.associated_skills.len(), 2);
        assert_eq!(pattern.primary_skill(), Some("nt-act-skill1"));
    }

    #[test]
    fn test_trust_boundary_assignment() {
        let mut pattern = BehaviorPattern::new("p1".to_string(), "Test".to_string(), BehaviorPatternType::Reactive);
        pattern.trust_boundary = TrustBoundary::Confidential;
        assert_eq!(pattern.trust_boundary, TrustBoundary::Confidential);
    }
}