use crate::neotrix::nt_mind::core::MaturityLevel;
use std::collections::HashMap;

/// Pairs a KnowledgeSource with its assessed maturity level.
#[derive(Debug, Clone)]
pub struct MatureKnowledgeSource {
    pub name: String,
    pub maturity: MaturityLevel,
}

/// Tracks maturity levels of named knowledge sources.
///
/// Provides multi-fidelity filtering: consolidated queries only return
/// sources that have reached at least `Validated`.
#[derive(Debug, Clone)]
pub struct KnowledgeMaturityTracker {
    levels: HashMap<String, MaturityLevel>,
}

impl Default for KnowledgeMaturityTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeMaturityTracker {
    pub fn new() -> Self {
        Self {
            levels: HashMap::new(),
        }
    }

    /// Register a source at a given maturity level.
    pub fn register(&mut self, name: &str, level: MaturityLevel) {
        self.levels.insert(name.to_string(), level);
    }

    /// Promote a source by one maturity level.
    /// Returns `true` if the level actually changed.
    pub fn promote(&mut self, name: &str) -> bool {
        let entry = self.levels.get(name);
        match entry {
            Some(current) => {
                if let Some(next) = current.promote() {
                    self.levels.insert(name.to_string(), next);
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    /// Get the confidence score for a named source.
    /// Returns `0.0` if the source is not registered.
    pub fn get_confidence(&self, name: &str) -> f64 {
        self.levels.get(name).map_or(0.0, |m| m.confidence())
    }

    /// Return only sources that have reached at least `Validated` maturity,
    /// paired with their confidence score.
    pub fn consolidated_knowledge(&self, sources: &[String]) -> Vec<(String, f64)> {
        sources
            .iter()
            .filter_map(|name| {
                let level = self.levels.get(name.as_str())?;
                if *level >= MaturityLevel::Validated {
                    Some((name.clone(), level.confidence()))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promote_chain() {
        let mut tracker = KnowledgeMaturityTracker::new();
        tracker.register("test_source", MaturityLevel::Candidate);

        assert_eq!(tracker.get_confidence("test_source"), 0.25);

        assert!(tracker.promote("test_source"));
        assert_eq!(tracker.get_confidence("test_source"), 0.5);

        assert!(tracker.promote("test_source"));
        assert_eq!(tracker.get_confidence("test_source"), 0.75);

        assert!(tracker.promote("test_source"));
        assert_eq!(tracker.get_confidence("test_source"), 1.0);

        assert!(!tracker.promote("test_source"));
        assert_eq!(tracker.get_confidence("test_source"), 1.0);
    }

    #[test]
    fn test_confidence_values() {
        let mut tracker = KnowledgeMaturityTracker::new();
        tracker.register("a", MaturityLevel::Candidate);
        tracker.register("b", MaturityLevel::Reviewed);
        tracker.register("c", MaturityLevel::Validated);
        tracker.register("d", MaturityLevel::GroundTruth);

        assert_eq!(tracker.get_confidence("a"), 0.25);
        assert_eq!(tracker.get_confidence("b"), 0.5);
        assert_eq!(tracker.get_confidence("c"), 0.75);
        assert_eq!(tracker.get_confidence("d"), 1.0);
        assert_eq!(tracker.get_confidence("unknown"), 0.0);
    }

    #[test]
    fn test_consolidated_knowledge_filters() {
        let mut tracker = KnowledgeMaturityTracker::new();
        tracker.register("unreviewed", MaturityLevel::Candidate);
        tracker.register("validated_one", MaturityLevel::Validated);
        tracker.register("ground_truth", MaturityLevel::GroundTruth);
        tracker.register("reviewed_only", MaturityLevel::Reviewed);

        let all = vec![
            "unreviewed".to_string(),
            "validated_one".to_string(),
            "ground_truth".to_string(),
            "reviewed_only".to_string(),
        ];

        let consolidated = tracker.consolidated_knowledge(&all);
        let names: Vec<&str> = consolidated.iter().map(|(n, _)| n.as_str()).collect();

        assert_eq!(consolidated.len(), 2);
        assert!(names.contains(&"validated_one"));
        assert!(names.contains(&"ground_truth"));
        assert!(!names.contains(&"unreviewed"));
        assert!(!names.contains(&"reviewed_only"));

        for (_, conf) in &consolidated {
            assert!(*conf >= 0.75);
        }
    }

    #[test]
    fn test_promote_unregistered_returns_false() {
        let mut tracker = KnowledgeMaturityTracker::new();
        assert!(!tracker.promote("nonexistent"));
    }
}
