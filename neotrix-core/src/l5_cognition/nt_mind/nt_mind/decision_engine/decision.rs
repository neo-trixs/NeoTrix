//! Decision engine core types — Decision, DecisionOption, Criteria
//!
//! R-P123: 按认知域拆分决策流程
//! R-P124: 配置集中管理，支持 Default trait

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Evaluation criteria for a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criteria {
    /// Criteria name (e.g. "cost", "feasibility", "risk")
    pub name: String,
    /// Weight factor: higher = more important. Range typically 0.0..1.0
    pub weight: f64,
    /// Human-readable description of what this criteria measures
    pub description: String,
}

impl Criteria {
    pub fn new(name: &str, weight: f64, description: &str) -> Self {
        Self {
            name: name.to_string(),
            weight,
            description: description.to_string(),
        }
    }
}

/// A single option within a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    /// Unique option identifier
    pub id: String,
    /// Human-readable option name
    pub name: String,
    /// Scores per criteria name: criteria_name -> score (0.0..1.0)
    pub scores: HashMap<String, f64>,
}

impl DecisionOption {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            scores: HashMap::new(),
        }
    }

    /// Add a score for a criteria
    pub fn with_score(mut self, criteria_name: &str, score: f64) -> Self {
        self.scores.insert(criteria_name.to_string(), score);
        self
    }

    /// Get score for a criteria, returns 0.0 if not set
    pub fn score_for(&self, criteria_name: &str) -> f64 {
        self.scores.get(criteria_name).copied().unwrap_or(0.0)
    }
}

/// A decision to be evaluated with multiple options and criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Unique decision identifier
    pub id: String,
    /// Decision context (metadata, tags, etc.)
    pub context: HashMap<String, String>,
    /// Available options to choose from
    pub options: Vec<DecisionOption>,
    /// Evaluation criteria with weights
    pub criteria: Vec<Criteria>,
}

impl Decision {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            context: HashMap::new(),
            options: Vec::new(),
            criteria: Vec::new(),
        }
    }

    /// Add an option to the decision
    pub fn with_option(mut self, option: DecisionOption) -> Self {
        self.options.push(option);
        self
    }

    /// Add a criteria to the decision
    pub fn with_criteria(mut self, criteria: Criteria) -> Self {
        self.criteria.push(criteria);
        self
    }

    /// Add a context entry
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    /// Normalize criteria weights to sum to 1.0
    pub fn normalized_weights(&self) -> Vec<f64> {
        let total: f64 = self.criteria.iter().map(|c| c.weight).sum();
        if total == 0.0 {
            self.criteria.iter().map(|_| 1.0 / self.criteria.len() as f64).collect()
        } else {
            self.criteria.iter().map(|c| c.weight / total).collect()
        }
    }
}

impl Default for Decision {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_decision() -> Decision {
        Decision::new("test-decision")
            .with_criteria(Criteria::new("cost", 0.6, "Total cost"))
            .with_criteria(Criteria::new("quality", 0.4, "Quality level"))
            .with_option(
                DecisionOption::new("opt-a", "Option A")
                    .with_score("cost", 0.8)
                    .with_score("quality", 0.6),
            )
            .with_option(
                DecisionOption::new("opt-b", "Option B")
                    .with_score("cost", 0.5)
                    .with_score("quality", 0.9),
            )
    }

    #[test]
    fn test_decision_new() {
        let d = Decision::new("d1");
        assert_eq!(d.id, "d1");
        assert!(d.options.is_empty());
        assert!(d.criteria.is_empty());
    }

    #[test]
    fn test_decision_with_context() {
        let d = Decision::new("d1")
            .with_context("priority", "high")
            .with_context("domain", "engineering");
        assert_eq!(d.context.get("priority").unwrap(), "high");
        assert_eq!(d.context.get("domain").unwrap(), "engineering");
    }

    #[test]
    fn test_option_scores() {
        let opt = DecisionOption::new("o1", "Test")
            .with_score("cost", 0.5)
            .with_score("speed", 0.9);
        assert_eq!(opt.score_for("cost"), 0.5);
        assert_eq!(opt.score_for("speed"), 0.9);
        assert_eq!(opt.score_for("missing"), 0.0);
    }

    #[test]
    fn test_normalized_weights() {
        let d = make_test_decision();
        let w = d.normalized_weights();
        assert_eq!(w.len(), 2);
        assert!((w[0] - 0.6).abs() < 1e-10);
        assert!((w[1] - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_normalized_weights_zero_sum() {
        let d = Decision::new("z")
            .with_criteria(Criteria::new("a", 0.0, "desc"))
            .with_criteria(Criteria::new("b", 0.0, "desc"));
        let w = d.normalized_weights();
        assert_eq!(w.len(), 2);
        assert!((w[0] - 0.5).abs() < 1e-10);
        assert!((w[1] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_default_decision() {
        let d = Decision::default();
        assert_eq!(d.id, "default");
    }

    #[test]
    fn test_serde_roundtrip() {
        let d = make_test_decision();
        let json = serde_json::to_string(&d).unwrap();
        let back: Decision = serde_json::from_str(&json).unwrap();
        assert_eq!(d.id, back.id);
        assert_eq!(d.options.len(), back.options.len());
        assert_eq!(d.criteria.len(), back.options.len());
    }

    #[test]
    fn test_criteria_new() {
        let c = Criteria::new("speed", 0.7, "How fast");
        assert_eq!(c.name, "speed");
        assert!((c.weight - 0.7).abs() < 1e-10);
        assert_eq!(c.description, "How fast");
    }

    #[test]
    fn test_decision_with_option_and_criteria_chaining() {
        let d = Decision::new("chain")
            .with_criteria(Criteria::new("c1", 1.0, "only"))
            .with_option(DecisionOption::new("o1", "Only").with_score("c1", 0.5))
            .with_context("ctx", "val");
        assert_eq!(d.criteria.len(), 1);
        assert_eq!(d.options.len(), 1);
        assert_eq!(d.context.len(), 1);
    }

    #[test]
    fn test_option_score_for_missing() {
        let opt = DecisionOption::new("o1", "Test");
        assert_eq!(opt.score_for("nonexistent"), 0.0);
    }

    #[test]
    fn test_normalized_weights_single_criteria() {
        let d = Decision::new("single")
            .with_criteria(Criteria::new("only", 0.5, "The only one"));
        let w = d.normalized_weights();
        assert_eq!(w.len(), 1);
        assert!((w[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_decision_default() {
        let d = Decision::default();
        assert_eq!(d.id, "default");
        assert!(d.context.is_empty());
    }
}
