//! JEV Core Primitives — the three decision types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Decision status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    /// Model selected this answer
    Selected,
    /// Model scored this answer
    Scored,
    /// Needs human review
    Review,
    /// Error occurred
    Error,
}

/// Noul — boolean probability (0.0–1.0)
///
/// Use for: "Is this safe?", "Should we proceed?", "Is this hallucination?"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoulAnswer {
    /// Probability that the statement is true (0.0–1.0)
    pub noul: f64,
    /// Whether this needs human review
    pub needs_review: bool,
    /// Optional explanation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Decision status
    pub status: DecisionStatus,
}

impl NoulAnswer {
    /// Create with auto-computed confidence from probability
    pub fn new(noul: f64) -> Self {
        let confidence = noul.max(1.0 - noul); // max(p, 1-p)
        Self {
            noul,
            needs_review: confidence < 0.7,
            reason: if confidence < 0.7 {
                Some(format!("low confidence: {:.4}", confidence))
            } else {
                None
            },
            status: if confidence < 0.7 {
                DecisionStatus::Review
            } else {
                DecisionStatus::Selected
            },
        }
    }
}

/// Choice — select one option from a fixed set
///
/// Use for: "Which tool?", "Which agent?", "Which policy?"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceAnswer {
    /// Selected option key
    pub choice: String,
    /// Full probability distribution over all options
    pub probabilities: HashMap<String, f64>,
    /// Confidence: how concentrated the distribution is (0.0–1.0)
    pub confidence: f64,
    /// Margin: top-1 minus top-2 probability
    pub margin: f64,
    /// Whether this needs human review
    pub needs_review: bool,
    /// Optional explanation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Decision status
    pub status: DecisionStatus,
}

impl ChoiceAnswer {
    /// Create with auto-computed confidence and margin from probabilities
    pub fn new(choice: String, probabilities: HashMap<String, f64>) -> Self {
        let confidence = choice_confidence(&probabilities);
        let mut sorted: Vec<f64> = probabilities.values().cloned().collect();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let margin = if sorted.len() >= 2 {
            sorted[0] - sorted[1]
        } else {
            sorted.first().copied().unwrap_or(0.0)
        };
        Self {
            choice,
            probabilities,
            confidence,
            margin,
            needs_review: confidence < 0.7,
            reason: if confidence < 0.7 {
                Some(format!("low confidence: {:.4}", confidence))
            } else {
                None
            },
            status: if confidence < 0.7 {
                DecisionStatus::Review
            } else {
                DecisionStatus::Selected
            },
        }
    }
}

/// Score — rate on an ordered scale
///
/// Use for: "Severity level?", "Maturity stage?", "Risk level?"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreAnswer {
    /// Selected score (index into legend)
    pub score: f64,
    /// Full probability distribution over all levels
    pub probabilities: HashMap<String, f64>,
    /// Confidence: how concentrated the distribution is (0.0–1.0)
    pub confidence: f64,
    /// Legend: human-readable labels for each score level
    pub legend: Vec<String>,
    /// Whether this needs human review
    pub needs_review: bool,
    /// Optional explanation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Decision status
    pub status: DecisionStatus,
}

impl ScoreAnswer {
    /// Create with auto-computed confidence from probabilities
    pub fn new(score: f64, probabilities: HashMap<String, f64>, legend: Vec<String>) -> Self {
        let confidence = score_confidence(&probabilities);
        Self {
            score,
            probabilities,
            confidence,
            legend,
            needs_review: confidence < 0.7,
            reason: if confidence < 0.7 {
                Some(format!("low confidence: {:.4}", confidence))
            } else {
                None
            },
            status: if confidence < 0.7 {
                DecisionStatus::Review
            } else {
                DecisionStatus::Scored
            },
        }
    }
}

/// Unified JEV decision — wraps all three primitives
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JevDecision {
    /// Boolean probability
    Noul(NoulAnswer),
    /// Selection from fixed options
    Choice(ChoiceAnswer),
    /// Rating on ordered scale
    Score(ScoreAnswer),
}

impl JevDecision {
    /// Does this decision need human review?
    pub fn needs_review(&self) -> bool {
        match self {
            Self::Noul(n) => n.needs_review,
            Self::Choice(c) => c.needs_review,
            Self::Score(s) => s.needs_review,
        }
    }

    /// Get confidence (0.0 for Noul which uses probability instead)
    pub fn confidence(&self) -> f64 {
        match self {
            Self::Noul(n) => n.noul,
            Self::Choice(c) => c.confidence,
            Self::Score(s) => s.confidence,
        }
    }

    /// Get reason
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Noul(n) => n.reason.as_deref(),
            Self::Choice(c) => c.reason.as_deref(),
            Self::Score(s) => s.reason.as_deref(),
        }
    }

    /// Get status
    pub fn status(&self) -> DecisionStatus {
        match self {
            Self::Noul(n) => n.status,
            Self::Choice(c) => c.status,
            Self::Score(s) => s.status,
        }
    }
}

/// Normalized Shannon entropy confidence: 1 - H(p) / log(k)
/// Ported from Laya's `confidence_from_probs(p, k)`.
/// k is the number of options (must be >= 2, otherwise returns 1.0).
pub fn confidence_from_probs(probs: &[f64], k: usize) -> f64 {
    if k < 2 {
        return 1.0;
    }
    let log_k = (k as f64).ln();
    let entropy: f64 = probs
        .iter()
        .take(k)
        .map(|&p| if p > 0.0 { -p * p.ln() } else { 0.0 })
        .sum();
    (1.0 - entropy / log_k).clamp(0.0, 1.0)
}

/// Compute confidence from a HashMap<String, f64> of probabilities
pub fn confidence_from_map(probs: &HashMap<String, f64>) -> f64 {
    let values: Vec<f64> = probs.values().cloned().collect();
    confidence_from_probs(&values, values.len())
}

/// Temperature bucket: maps (question_type, option_count) → bucket string
/// Ported from Laya's `temp_bucket(qtype, k)`.
pub fn temp_bucket(qtype: &str, k: usize) -> String {
    let size = if k <= 2 {
        "2"
    } else if k <= 5 {
        "3-5"
    } else if k <= 10 {
        "6-10"
    } else {
        "11+"
    };
    format!("{}:{}", qtype, size)
}

/// Upstream Jev API confidence: rescaled top-probability `(k·p_top − 1)/(k − 1)`.
///
/// Live-API measurements (~1M answers, 2026-09) show the hosted Jev `confidence`
/// field follows this formula within rounding error — it summarizes the returned
/// distribution (distance of the winner from pure chance), it is NOT a separate
/// model-uncertainty signal. Use [`confidence_from_probs`] (normalized entropy,
/// full distribution) as the primary gating signal; use this only when
/// cross-comparing with upstream API values.
pub fn confidence_top_prob(probs: &HashMap<String, f64>) -> f64 {
    let k = probs.len();
    if k < 2 {
        return 1.0;
    }
    let top = probs.values().cloned().fold(0.0f64, f64::max);
    ((k as f64 * top - 1.0) / (k as f64 - 1.0)).clamp(0.0, 1.0)
}

/// Contested margin: when top-1 and top-2 are closer than this, the winning
/// label is unstable across repeated calls (measured flips when the gap is
/// within ~0.05). Contested answers must go to review, never auto-act.
pub const CONTESTED_MARGIN: f64 = 0.05;

/// Returns true if a choice is contested (margin below [`CONTESTED_MARGIN`]).
pub fn choice_is_contested(choice: &ChoiceAnswer) -> bool {
    choice.margin < CONTESTED_MARGIN
}

/// Compute choice confidence from a HashMap of option probabilities
pub fn choice_confidence(probs: &HashMap<String, f64>) -> f64 {
    let values: Vec<f64> = probs.values().cloned().collect();
    confidence_from_probs(&values, values.len())
}

/// Compute score confidence from a HashMap of level probabilities
pub fn score_confidence(probs: &HashMap<String, f64>) -> f64 {
    let values: Vec<f64> = probs.values().cloned().collect();
    confidence_from_probs(&values, values.len())
}

/// A set of JEV decisions keyed by question ID
pub type JevResultSet = HashMap<String, JevDecision>;

/// Evaluation result with JEV decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JevResult {
    /// Decisions keyed by question ID
    pub decisions: JevResultSet,
    /// Model used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Token usage (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<JevUsage>,
    /// Overall exit code
    pub exit_code: ExitCode,
}

/// Token usage for JEV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JevUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Exit codes for process integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ExitCode {
    /// All decisions successful
    Success = 0,
    /// Evaluation error
    Error = 1,
    /// One or more answers need human review
    NeedsReview = 2,
}

impl ExitCode {
    /// Derive from a result set
    pub fn from_decisions(decisions: &JevResultSet) -> Self {
        if decisions.values().any(|d| d.needs_review()) {
            Self::NeedsReview
        } else {
            Self::Success
        }
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noul_answer() {
        let n = NoulAnswer {
            noul: 0.9,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        assert!(!n.needs_review);
        assert_eq!(n.status, DecisionStatus::Selected);
    }

    #[test]
    fn test_choice_answer() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.8);
        probs.insert("b".to_string(), 0.2);
        let c = ChoiceAnswer {
            choice: "a".into(),
            probabilities: probs,
            confidence: 0.75,
            margin: 0.6,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        assert_eq!(c.choice, "a");
        assert!((c.confidence - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_jev_decision_needs_review() {
        let d = JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: Some("uncertain".into()),
            status: DecisionStatus::Review,
        });
        assert!(d.needs_review());
        assert_eq!(d.status(), DecisionStatus::Review);
    }

    #[test]
    fn test_exit_code() {
        let mut decisions = HashMap::new();
        decisions.insert("q1".to_string(), JevDecision::Noul(NoulAnswer {
            noul: 0.9,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        }));
        assert_eq!(ExitCode::from_decisions(&decisions), ExitCode::Success);

        decisions.insert("q2".to_string(), JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: None,
            status: DecisionStatus::Review,
        }));
        assert_eq!(ExitCode::from_decisions(&decisions), ExitCode::NeedsReview);
    }

    #[test]
    fn test_confidence_from_probs_concentrated() {
        let probs = vec![0.9, 0.1];
        let conf = confidence_from_probs(&probs, 2);
        assert!(conf > 0.5, "concentrated should have high confidence: {}", conf);
    }

    #[test]
    fn test_confidence_from_probs_uniform() {
        let probs = vec![0.5, 0.5];
        let conf = confidence_from_probs(&probs, 2);
        assert!((conf - 0.0).abs() < 0.001, "uniform should have zero confidence: {}", conf);
    }

    #[test]
    fn test_confidence_from_probs_single() {
        let probs = vec![1.0];
        let conf = confidence_from_probs(&probs, 1);
        assert!((conf - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_temp_bucket() {
        assert_eq!(temp_bucket("choice", 3), "choice:3-5");
        assert_eq!(temp_bucket("noul", 2), "noul:2");
        assert_eq!(temp_bucket("score", 7), "score:6-10");
        assert_eq!(temp_bucket("choice", 15), "choice:11+");
    }

    #[test]
    fn test_noul_answer_new() {
        let n = NoulAnswer::new(0.9);
        assert!(!n.needs_review);
        assert_eq!(n.status, DecisionStatus::Selected);

        let n = NoulAnswer::new(0.5);
        assert!(n.needs_review);
        assert_eq!(n.status, DecisionStatus::Review);
    }

    #[test]
    fn test_confidence_top_prob() {
        // 5 options, top 0.76 → (5·0.76−1)/4 = 0.70 (upstream formula)
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.76);
        probs.insert("b".to_string(), 0.06);
        probs.insert("c".to_string(), 0.06);
        probs.insert("d".to_string(), 0.06);
        probs.insert("e".to_string(), 0.06);
        let conf = confidence_top_prob(&probs);
        assert!((conf - 0.70).abs() < 0.001, "got {}", conf);
    }

    #[test]
    fn test_confidence_top_prob_chance_is_zero() {
        // Uniform over 4 → (4·0.25−1)/3 = 0
        let mut probs = HashMap::new();
        for k in ["a", "b", "c", "d"] {
            probs.insert(k.to_string(), 0.25);
        }
        assert!((confidence_top_prob(&probs)).abs() < 0.001);
    }

    #[test]
    fn test_choice_is_contested() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.51);
        probs.insert("b".to_string(), 0.49);
        let c = ChoiceAnswer::new("a".to_string(), probs);
        assert!(choice_is_contested(&c), "margin={}", c.margin);

        let mut probs2 = HashMap::new();
        probs2.insert("a".to_string(), 0.9);
        probs2.insert("b".to_string(), 0.1);
        let c2 = ChoiceAnswer::new("a".to_string(), probs2);
        assert!(!choice_is_contested(&c2));
    }
}
