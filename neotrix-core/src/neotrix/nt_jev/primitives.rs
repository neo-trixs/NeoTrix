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
}
