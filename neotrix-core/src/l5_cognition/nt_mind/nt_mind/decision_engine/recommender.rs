//! Decision recommender — full pipeline from Decision to Recommendation
//!
//! R-P123: 按认知域拆分决策流程
//! R-P124: 配置集中管理，支持 Default trait

use serde::{Deserialize, Serialize};

use super::analyzer::{Analysis, DecisionAnalyzer};
use super::decision::Decision;
use super::scorer::WeightedScorer;

/// Final recommendation output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Top recommended option identifier
    pub top_option: String,
    /// Alternative option identifiers, ordered by score descending
    pub alternatives: Vec<String>,
    /// Overall confidence (0.0..1.0)
    pub confidence: f64,
    /// Human-readable reasoning
    pub reasoning: String,
}

/// End-to-end decision recommender
///
/// Pipeline: Decision → WeightedScorer → ScoredOptions → DecisionAnalyzer → Analysis → Recommendation
pub struct DecisionRecommender {
    scorer: WeightedScorer,
    analyzer: DecisionAnalyzer,
}

impl Default for DecisionRecommender {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl DecisionRecommender {
    pub fn new(analyzer: DecisionAnalyzer) -> Self {
        Self {
            scorer: WeightedScorer::new(),
            analyzer,
        }
    }

    /// Create with default analyzer
    pub fn with_defaults() -> Self {
        Self::new(DecisionAnalyzer::new())
    }

    /// Produce a full Recommendation from a Decision
    pub fn recommend(&self, decision: &Decision) -> Recommendation {
        let scored = self.scorer.score(decision);
        let analysis = self.analyzer.analyze(&scored);

        let alternatives: Vec<String> = scored
            .iter()
            .skip(1)
            .map(|s| s.option_id.clone())
            .collect();

        let reasoning = self.build_reasoning(decision, &analysis, &alternatives);

        Recommendation {
            top_option: analysis.recommendation,
            alternatives,
            confidence: analysis.confidence,
            reasoning,
        }
    }

    /// Build reasoning narrative
    fn build_reasoning(
        &self,
        decision: &Decision,
        analysis: &Analysis,
        alternatives: &[String],
    ) -> String {
        let mut parts = Vec::new();

        parts.push(format!(
            "Decision '{}' evaluated {} option(s) against {} criteria.",
            decision.id,
            decision.options.len(),
            decision.criteria.len()
        ));

        parts.push(format!(
            "Recommendation: '{}' (confidence: {:.0}%).",
            analysis.recommendation,
            analysis.confidence * 100.0
        ));

        if !alternatives.is_empty() {
            parts.push(format!(
                "Alternatives considered: {}.",
                alternatives.join(", ")
            ));
        }

        parts.push(format!("Rationale: {}", analysis.rationale));
        parts.push(format!("Risk: {}", analysis.risk_assessment));

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_mind::nt_mind::decision_engine::decision::{Criteria, DecisionOption};

    fn make_test_decision() -> Decision {
        Decision::new("deploy-choice")
            .with_context("project", "backend")
            .with_criteria(Criteria::new("cost", 0.5, "Deployment cost"))
            .with_criteria(Criteria::new("reliability", 0.3, "System reliability"))
            .with_criteria(Criteria::new("speed", 0.2, "Deployment speed"))
            .with_option(
                DecisionOption::new("kubernetes", "Kubernetes")
                    .with_score("cost", 0.4)
                    .with_score("reliability", 0.9)
                    .with_score("speed", 0.5),
            )
            .with_option(
                DecisionOption::new("serverless", "Serverless")
                    .with_score("cost", 0.8)
                    .with_score("reliability", 0.6)
                    .with_score("speed", 0.9),
            )
            .with_option(
                DecisionOption::new("vm", "Traditional VM")
                    .with_score("cost", 0.6)
                    .with_score("reliability", 0.7)
                    .with_score("speed", 0.3),
            )
    }

    #[test]
    fn test_recommendation_pipeline() {
        let recommender = DecisionRecommender::with_defaults();
        let d = make_test_decision();
        let rec = recommender.recommend(&d);

        assert!(!rec.top_option.is_empty());
        assert!(rec.confidence > 0.0 && rec.confidence <= 1.0);
        assert!(!rec.reasoning.is_empty());

        // Should have 2 alternatives
        assert_eq!(rec.alternatives.len(), 2);
    }

    #[test]
    fn test_recommendation_top_is_best() {
        let recommender = DecisionRecommender::with_defaults();
        let d = make_test_decision();
        let rec = recommender.recommend(&d);

        // serverless: 0.8*0.5 + 0.6*0.3 + 0.9*0.2 = 0.40+0.18+0.18 = 0.76
        // kubernetes: 0.4*0.5 + 0.9*0.3 + 0.5*0.2 = 0.20+0.27+0.10 = 0.57
        // vm:         0.6*0.5 + 0.7*0.3 + 0.3*0.2 = 0.30+0.21+0.06 = 0.57
        assert_eq!(rec.top_option, "serverless");
    }

    #[test]
    fn test_recommendation_empty_decision() {
        let recommender = DecisionRecommender::with_defaults();
        let d = Decision::new("empty");
        let rec = recommender.recommend(&d);

        assert!(rec.top_option.is_empty());
        assert_eq!(rec.confidence, 0.0);
    }

    #[test]
    fn test_recommendation_single_option() {
        let d = Decision::new("solo")
            .with_criteria(Criteria::new("x", 1.0, "only"))
            .with_option(DecisionOption::new("only", "Only Choice").with_score("x", 0.7));

        let recommender = DecisionRecommender::with_defaults();
        let rec = recommender.recommend(&d);

        assert_eq!(rec.top_option, "only");
        assert!(rec.alternatives.is_empty());
        assert!(rec.confidence > 0.0);
    }

    #[test]
    fn test_recommendation_reasoning_content() {
        let recommender = DecisionRecommender::with_defaults();
        let d = make_test_decision();
        let rec = recommender.recommend(&d);

        assert!(rec.reasoning.contains("deploy-choice"));
        assert!(rec.reasoning.contains("3 option(s)"));
        assert!(rec.reasoning.contains("3 criteria"));
        assert!(rec.reasoning.contains("Recommendation"));
    }

    #[test]
    fn test_default_recommender() {
        let rec = DecisionRecommender::default();
        let d = Decision::new("test")
            .with_criteria(Criteria::new("a", 1.0, "A"))
            .with_option(DecisionOption::new("o1", "O1").with_score("a", 0.5));
        let r = rec.recommend(&d);
        assert_eq!(r.top_option, "o1");
    }

    #[test]
    fn test_recommendation_serialization() {
        let rec = DecisionRecommender::with_defaults();
        let d = make_test_decision();
        let r = rec.recommend(&d);
        let json = serde_json::to_string(&r).unwrap();
        let back: Recommendation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.top_option, r.top_option);
        assert_eq!(back.alternatives.len(), r.alternatives.len());
    }

    #[test]
    fn test_recommendation_confidence_range() {
        let rec = DecisionRecommender::with_defaults();
        let d = make_test_decision();
        let r = rec.recommend(&d);
        assert!(r.confidence >= 0.0 && r.confidence <= 1.0);
    }

    #[test]
    fn test_recommendation_reasoning_contains_risk() {
        let rec = DecisionRecommender::with_defaults();
        let d = make_test_decision();
        let r = rec.recommend(&d);
        assert!(r.reasoning.contains("Risk"));
    }

    #[test]
    fn test_custom_analyzer() {
        let analyzer = DecisionAnalyzer::new();
        let rec = DecisionRecommender::new(analyzer);
        let d = Decision::new("custom")
            .with_criteria(Criteria::new("x", 1.0, "X"))
            .with_option(DecisionOption::new("o1", "O1").with_score("x", 0.8));
        let r = rec.recommend(&d);
        assert_eq!(r.top_option, "o1");
    }

    #[test]
    fn test_recommendation_two_options() {
        let d = Decision::new("two")
            .with_criteria(Criteria::new("c", 1.0, "only"))
            .with_option(DecisionOption::new("better", "Better").with_score("c", 0.9))
            .with_option(DecisionOption::new("worse", "Worse").with_score("c", 0.3));
        let rec = DecisionRecommender::with_defaults();
        let r = rec.recommend(&d);
        assert_eq!(r.top_option, "better");
        assert_eq!(r.alternatives, vec!["worse"]);
    }
}
