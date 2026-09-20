//! Weighted scoring engine — WeightedScorer
//!
//! R-P123: 按认知域拆分决策流程
//! R-P124: 配置集中管理，支持 Default trait

use serde::{Deserialize, Serialize};

use super::decision::Decision;

/// A scored option with its weighted score and rank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredOption {
    /// Option identifier from the original DecisionOption
    pub option_id: String,
    /// Computed weighted score: sum(score * weight) for each criteria
    pub weighted_score: f64,
    /// Rank position (1 = best)
    pub rank: usize,
}

/// Weighted scoring engine
///
/// Scores each option by computing: sum(score_i * weight_i) for all criteria.
/// Options are ranked by weighted_score descending.
#[derive(Debug, Clone)]
pub struct WeightedScorer;

impl Default for WeightedScorer {
    fn default() -> Self {
        Self
    }
}

impl WeightedScorer {
    pub fn new() -> Self {
        Self
    }

    /// Score all options in a decision, returning ranked ScoredOptions
    pub fn score(&self, decision: &Decision) -> Vec<ScoredOption> {
        if decision.criteria.is_empty() || decision.options.is_empty() {
            return Vec::new();
        }

        let weights = decision.normalized_weights();

        let mut scored: Vec<ScoredOption> = decision
            .options
            .iter()
            .enumerate()
            .map(|(idx, option)| {
                let weighted_score: f64 = decision
                    .criteria
                    .iter()
                    .enumerate()
                    .map(|(ci, criteria)| {
                        let score = option.score_for(&criteria.name);
                        score * weights[ci]
                    })
                    .sum();

                ScoredOption {
                    option_id: option.id.clone(),
                    weighted_score,
                    rank: idx + 1, // temporary, will be reassigned
                }
            })
            .collect();

        // Sort descending by weighted_score
        scored.sort_by(|a, b| b.weighted_score.partial_cmp(&a.weighted_score).unwrap_or(std::cmp::Ordering::Equal));

        // Assign ranks
        for (i, s) in scored.iter_mut().enumerate() {
            s.rank = i + 1;
        }

        scored
    }

    /// Score a single option against criteria, returning raw weighted sum
    pub fn score_single(
        &self,
        option: &super::decision::DecisionOption,
        criteria: &[super::decision::Criteria],
        weights: &[f64],
    ) -> f64 {
        criteria
            .iter()
            .enumerate()
            .map(|(i, c)| option.score_for(&c.name) * weights[i])
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_mind::nt_mind::decision_engine::decision::{Criteria, Decision, DecisionOption};

    fn make_test_decision() -> Decision {
        Decision::new("scorer-test")
            .with_criteria(Criteria::new("cost", 0.6, "Cost factor"))
            .with_criteria(Criteria::new("quality", 0.4, "Quality factor"))
            .with_option(
                DecisionOption::new("a", "Cheap but okay")
                    .with_score("cost", 0.9)
                    .with_score("quality", 0.4),
            )
            .with_option(
                DecisionOption::new("b", "Expensive but great")
                    .with_score("cost", 0.3)
                    .with_score("quality", 0.95),
            )
            .with_option(
                DecisionOption::new("c", "Balanced")
                    .with_score("cost", 0.7)
                    .with_score("quality", 0.7),
            )
    }

    #[test]
    fn test_score_ranking() {
        let d = make_test_decision();
        let scorer = WeightedScorer::new();
        let results = scorer.score(&d);

        assert_eq!(results.len(), 3);
        // Check ranks are assigned
        for r in &results {
            assert!(r.rank >= 1 && r.rank <= 3);
        }
        // Check ordering: highest score first
        assert!(results[0].weighted_score >= results[1].weighted_score);
        assert!(results[1].weighted_score >= results[2].weighted_score);
    }

    #[test]
    fn test_score_values() {
        let d = make_test_decision();
        let scorer = WeightedScorer::new();
        let results = scorer.score(&d);

        // Option a: 0.9*0.6 + 0.4*0.4 = 0.54 + 0.16 = 0.70
        let a = results.iter().find(|r| r.option_id == "a").unwrap();
        assert!((a.weighted_score - 0.70).abs() < 1e-10);

        // Option b: 0.3*0.6 + 0.95*0.4 = 0.18 + 0.38 = 0.56
        let b = results.iter().find(|r| r.option_id == "b").unwrap();
        assert!((b.weighted_score - 0.56).abs() < 1e-10);

        // Option c: 0.7*0.6 + 0.7*0.4 = 0.42 + 0.28 = 0.70
        let c = results.iter().find(|r| r.option_id == "c").unwrap();
        assert!((c.weighted_score - 0.70).abs() < 1e-10);
    }

    #[test]
    fn test_empty_decision() {
        let d = Decision::new("empty");
        let scorer = WeightedScorer::new();
        assert!(scorer.score(&d).is_empty());
    }

    #[test]
    fn test_single_option() {
        let d = Decision::new("single")
            .with_criteria(Criteria::new("x", 1.0, "only"))
            .with_option(DecisionOption::new("o1", "Solo").with_score("x", 0.8));
        let scorer = WeightedScorer::new();
        let results = scorer.score(&d);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rank, 1);
        assert!((results[0].weighted_score - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_score_single() {
        let scorer = WeightedScorer::new();
        let opt = DecisionOption::new("x", "Test").with_score("a", 0.8).with_score("b", 0.6);
        let criteria = vec![
            Criteria::new("a", 0.5, "A"),
            Criteria::new("b", 0.5, "B"),
        ];
        let weights = vec![0.5, 0.5];
        let s = scorer.score_single(&opt, &criteria, &weights);
        assert!((s - 0.7).abs() < 1e-10); // 0.8*0.5 + 0.6*0.5
    }

    #[test]
    fn test_score_all_criteria_zero_weight() {
        let d = Decision::new("zero")
            .with_criteria(Criteria::new("a", 0.0, "zero weight"))
            .with_criteria(Criteria::new("b", 0.0, "zero weight"))
            .with_option(DecisionOption::new("o1", "Opt").with_score("a", 0.9).with_score("b", 0.1));
        let scorer = WeightedScorer::new();
        let results = scorer.score(&d);
        assert_eq!(results.len(), 1);
        // Equal weights: 0.5*0.9 + 0.5*0.1 = 0.5
        assert!((results[0].weighted_score - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_score_ranking_ties() {
        let d = Decision::new("tie")
            .with_criteria(Criteria::new("x", 1.0, "only"))
            .with_option(DecisionOption::new("a", "A").with_score("x", 0.5))
            .with_option(DecisionOption::new("b", "B").with_score("x", 0.5));
        let scorer = WeightedScorer::new();
        let results = scorer.score(&d);
        assert_eq!(results.len(), 2);
        // Both have same score — both should be rank 1 and 2 (stable sort)
        assert_eq!(results[0].weighted_score, results[1].weighted_score);
    }

    #[test]
    fn test_scored_option_serialization() {
        let so = ScoredOption {
            option_id: "test".into(),
            weighted_score: 0.75,
            rank: 1,
        };
        let json = serde_json::to_string(&so).unwrap();
        let back: ScoredOption = serde_json::from_str(&json).unwrap();
        assert_eq!(back.option_id, "test");
        assert!((back.weighted_score - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_scorer_default() {
        let scorer = WeightedScorer::default();
        let d = Decision::new("empty");
        assert!(scorer.score(&d).is_empty());
    }

    #[test]
    fn test_score_many_options() {
        let mut d = Decision::new("many").with_criteria(Criteria::new("c", 1.0, "only"));
        for i in 0..10 {
            d = d.with_option(
                DecisionOption::new(&format!("o{}", i), &format!("Opt {}", i))
                    .with_score("c", i as f64 / 10.0),
            );
        }
        let scorer = WeightedScorer::new();
        let results = scorer.score(&d);
        assert_eq!(results.len(), 10);
        // Highest score should be rank 1
        assert_eq!(results[0].rank, 1);
        assert!(results[0].weighted_score >= results[9].weighted_score);
    }
}
