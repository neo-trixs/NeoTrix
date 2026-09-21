//! Question and Answer type definitions
//!
//! Inspired by TypeSafe JEV's three question primitives:
//! - Noul: Boolean probability (0.0 ~ 1.0)
//! - Choice: Select one option from a fixed set
//! - Score: Rate on an ordered scale

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;

/// State sent to the decision engine for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// State as string (e.g., user message, tool call arguments)
    pub content: String,
    
    /// Optional structured data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl From<&str> for State {
    fn from(s: &str) -> Self {
        Self {
            content: s.to_string(),
            data: None,
        }
    }
}

impl From<String> for State {
    fn from(s: String) -> Self {
        Self {
            content: s,
            data: None,
        }
    }
}

impl From<serde_json::Value> for State {
    fn from(v: serde_json::Value) -> Self {
        Self {
            content: v.to_string(),
            data: Some(v),
        }
    }
}

/// Question type - one of three primitives
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QuestionType {
    /// Boolean probability question
    /// Returns a value between 0.0 (false) and 1.0 (true)
    Noul {
        instructions: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        criteria: Option<NoulCriteria>,
    },
    
    /// Select one option from a fixed set
    Choice {
        instructions: String,
        criteria: HashMap<String, Option<String>>,
    },
    
    /// Rate on an ordered scale
    Score {
        instructions: String,
        criteria: Vec<String>,
    },
}

/// Optional criteria for Noul questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoulCriteria {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#true: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub false_: Option<String>,
}

/// A question to be evaluated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    /// Unique identifier for this question
    pub id: String,
    
    /// Question type and configuration
    #[serde(flatten)]
    pub question_type: QuestionType,
}

/// A question set for a single evaluation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionSet {
    /// Questions to evaluate
    pub questions: Vec<Question>,
}

/// Decision status (matches JEV output format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Selected,
    Scored,
    Review,
    Error,
}

/// Answer to a Noul question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoulAnswer {
    /// Probability that the proposition is true (0.0 ~ 1.0)
    pub noul: f64,
    /// Whether this answer needs human review (abstention signal)
    #[serde(default)]
    pub needs_review: bool,
    /// Short evidence-based reason for the decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Decision status
    #[serde(default = "default_selected")]
    pub status: DecisionStatus,
}

fn default_selected() -> DecisionStatus {
    DecisionStatus::Selected
}

/// Answer to a Choice question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceAnswer {
    /// Selected option label
    pub choice: String,
    /// Full probability distribution across all labels
    pub probabilities: HashMap<String, f64>,
    /// Distribution concentration (NOT correctness probability)
    /// Formula: (max(p) - 1/K) / (1 - 1/K) where K = number of labels
    pub confidence: f64,
    /// Margin: gap between top-1 and top-2 probabilities
    pub margin: f64,
    /// Whether this answer needs human review
    #[serde(default)]
    pub needs_review: bool,
    /// Short evidence-based reason for the decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Decision status
    #[serde(default = "default_selected")]
    pub status: DecisionStatus,
}

/// Answer to a Score question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreAnswer {
    /// Score value (fractional expected rubric index, e.g., 1.4)
    pub score: f64,
    /// Full probability distribution across all levels
    pub probabilities: HashMap<String, f64>,
    /// Distribution concentration (NOT correctness probability)
    pub confidence: f64,
    /// Level descriptions (legend), indexed from 0
    #[serde(default)]
    pub legend: Vec<String>,
    /// Whether this answer needs human review
    #[serde(default)]
    pub needs_review: bool,
    /// Short evidence-based reason for the decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Decision status
    #[serde(default = "default_scored")]
    pub status: DecisionStatus,
}

fn default_scored() -> DecisionStatus {
    DecisionStatus::Scored
}

/// Unified answer type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Answer {
    Noul(NoulAnswer),
    Choice(ChoiceAnswer),
    Score(ScoreAnswer),
}

/// Compute choice confidence from probability distribution
///
/// Formula: `(max(p) - 1/K) / (1 - 1/K)` where K = number of labels
/// Returns 0.0 for uniform distribution, 1.0 for certain.
pub fn choice_confidence(probs: &HashMap<String, f64>) -> f64 {
    let k = probs.len() as f64;
    if k <= 1.0 {
        return 1.0;
    }
    let max_p = probs.values().cloned().fold(f64::NEG_INFINITY, f64::max);
    let uniform = 1.0 / k;
    let denominator = 1.0 - uniform;
    if denominator <= 0.0 {
        return 0.0;
    }
    ((max_p - uniform) / denominator).clamp(0.0, 1.0)
}

/// Compute score confidence from probability distribution
///
/// Measures distribution concentration around the mode.
/// Higher = more concentrated = more confident.
pub fn score_confidence(probs: &HashMap<String, f64>) -> f64 {
    if probs.is_empty() {
        return 0.0;
    }
    // Find mode (level with highest probability)
    let mode = probs.iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(k, _)| k.parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);
    
    // Compute D_K (max possible divergence) as K-1
    let k = probs.len() as f64;
    let d_k = k - 1.0;
    if d_k <= 0.0 {
        return 1.0;
    }
    
    // sum(p[i] * |i - mode|)
    let weighted_distance: f64 = probs.iter()
        .map(|(level, p)| {
            let i = level.parse::<f64>().unwrap_or(0.0);
            p * (i - mode).abs()
        })
        .sum();
    
    (1.0 - weighted_distance / d_k).clamp(0.0, 1.0)
}

/// Validate probability distribution: total mass must be within 0.05 of 1.0
pub fn validate_probabilities(probs: &HashMap<String, f64>) -> std::result::Result<(), String> {
    let total: f64 = probs.values().sum();
    let error = (total - 1.0).abs();
    if error > 0.05 {
        Err(format!(
            "Probability mass error {:.4} exceeds threshold 0.05 (total: {:.4})",
            error, total
        ))
    } else {
        Ok(())
    }
}

/// Complete evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Answers keyed by question ID
    pub answers: HashMap<String, Answer>,
    
    /// Model used (for tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    
    /// Token usage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Decision result for guardrail checks
#[derive(Debug, Clone)]
pub enum Decision {
    /// Allow the action
    Allow,
    
    /// Block the action with reason
    Block { reason: String },
    
    /// Requires human approval
    Pending { reason: String },
}

impl Question {
    /// Create a Noul question
    pub fn noul(id: impl Into<String>, instructions: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            question_type: QuestionType::Noul {
                instructions: instructions.into(),
                criteria: None,
            },
        }
    }
    
    /// Create a Noul question with criteria
    pub fn noul_with_criteria(
        id: impl Into<String>,
        instructions: impl Into<String>,
        true_desc: impl Into<String>,
        false_desc: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            question_type: QuestionType::Noul {
                instructions: instructions.into(),
                criteria: Some(NoulCriteria {
                    r#true: Some(true_desc.into()),
                    false_: Some(false_desc.into()),
                }),
            },
        }
    }
    
    /// Create a Choice question (validates non-empty options)
    pub fn choice(
        id: impl Into<String>,
        instructions: impl Into<String>,
        options: HashMap<String, Option<String>>,
    ) -> Result<Self> {
        if options.is_empty() {
            return Err(crate::error::Error::InvalidQuestionType(
                "Choice question requires at least one option".into()
            ));
        }
        Ok(Self {
            id: id.into(),
            question_type: QuestionType::Choice {
                instructions: instructions.into(),
                criteria: options,
            },
        })
    }
    
    /// Create a Score question (validates non-empty levels)
    pub fn score(
        id: impl Into<String>,
        instructions: impl Into<String>,
        levels: Vec<String>,
    ) -> Result<Self> {
        if levels.is_empty() {
            return Err(crate::error::Error::InvalidQuestionType(
                "Score question requires at least one level".into()
            ));
        }
        Ok(Self {
            id: id.into(),
            question_type: QuestionType::Score {
                instructions: instructions.into(),
                criteria: levels,
            },
        })
    }

    /// Validate this question has sensible structure
    pub fn validate(&self) -> Result<()> {
        match &self.question_type {
            QuestionType::Noul { instructions, .. } => {
                if instructions.is_empty() {
                    return Err(crate::error::Error::InvalidQuestionType(
                        format!("Question '{}' has empty instructions", self.id)
                    ));
                }
            }
            QuestionType::Choice { instructions, criteria } => {
                if instructions.is_empty() {
                    return Err(crate::error::Error::InvalidQuestionType(
                        format!("Question '{}' has empty instructions", self.id)
                    ));
                }
                if criteria.is_empty() {
                    return Err(crate::error::Error::InvalidQuestionType(
                        format!("Choice question '{}' requires at least one option", self.id)
                    ));
                }
            }
            QuestionType::Score { instructions, criteria } => {
                if instructions.is_empty() {
                    return Err(crate::error::Error::InvalidQuestionType(
                        format!("Question '{}' has empty instructions", self.id)
                    ));
                }
                if criteria.is_empty() {
                    return Err(crate::error::Error::InvalidQuestionType(
                        format!("Score question '{}' requires at least one level", self.id)
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_noul_question() {
        let q = Question::noul("is_urgent", "Does this convey urgency?");
        assert_eq!(q.id, "is_urgent");
        assert!(matches!(q.question_type, QuestionType::Noul { .. }));
    }
    
    #[test]
    fn test_noul_with_criteria() {
        let q = Question::noul_with_criteria(
            "healthy", "Is the system healthy?",
            "System is responding normally",
            "System has issues",
        );
        match &q.question_type {
            QuestionType::Noul { instructions, criteria } => {
                assert_eq!(instructions, "Is the system healthy?");
                let c = criteria.as_ref().unwrap();
                assert_eq!(c.r#true.as_deref(), Some("System is responding normally"));
                assert_eq!(c.false_.as_deref(), Some("System has issues"));
            }
            _ => panic!("Expected Noul"),
        }
    }

    #[test]
    fn test_choice_question() {
        let mut options = HashMap::new();
        options.insert("billing".to_string(), Some("Payments".to_string()));
        options.insert("technical".to_string(), Some("Bugs".to_string()));
        
        let q = Question::choice("department", "Which team handles this?", options).unwrap();
        assert_eq!(q.id, "department");
        match &q.question_type {
            QuestionType::Choice { criteria, .. } => {
                assert_eq!(criteria.len(), 2);
                assert!(criteria.contains_key("billing"));
                assert!(criteria.contains_key("technical"));
            }
            _ => panic!("Expected Choice"),
        }
    }

    #[test]
    fn test_choice_no_descriptions() {
        let mut options = HashMap::new();
        options.insert("a".to_string(), None);
        options.insert("b".to_string(), None);

        let q = Question::choice("pick", "Choose one", options).unwrap();
        match &q.question_type {
            QuestionType::Choice { criteria, .. } => {
                assert_eq!(criteria.len(), 2);
                assert_eq!(criteria["a"], None);
                assert_eq!(criteria["b"], None);
            }
            _ => panic!("Expected Choice"),
        }
    }
    
    #[test]
    fn test_score_question() {
        let q = Question::score(
            "severity",
            "How severe is this?",
            vec!["Low".to_string(), "Medium".to_string(), "High".to_string()],
        ).unwrap();
        assert_eq!(q.id, "severity");
        match &q.question_type {
            QuestionType::Score { criteria, .. } => {
                assert_eq!(criteria.len(), 3);
                assert_eq!(criteria[0], "Low");
                assert_eq!(criteria[1], "Medium");
                assert_eq!(criteria[2], "High");
            }
            _ => panic!("Expected Score"),
        }
    }

    #[test]
    fn test_score_single_level() {
        let q = Question::score("binary", "Rate", vec!["No".into()]).unwrap();
        match &q.question_type {
            QuestionType::Score { criteria, .. } => {
                assert_eq!(criteria.len(), 1);
            }
            _ => panic!("Expected Score"),
        }
    }
    
    #[test]
    fn test_state_from_str() {
        let state: State = "test content".into();
        assert_eq!(state.content, "test content");
        assert!(state.data.is_none());
    }

    #[test]
    fn test_state_from_string() {
        let state: State = "owned string".to_string().into();
        assert_eq!(state.content, "owned string");
        assert!(state.data.is_none());
    }

    #[test]
    fn test_state_from_json() {
        let json = serde_json::json!({"key": "value", "num": 42});
        let state: State = json.into();
        assert!(state.content.contains("key"));
        assert!(state.data.is_some());
    }

    // ── Serialization Roundtrips ───────────────────────────────

    #[test]
    fn test_noul_serialization() {
        let q = Question::noul("q1", "test?");
        let json = serde_json::to_string(&q).unwrap();
        let back: Question = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "q1");
        assert!(matches!(back.question_type, QuestionType::Noul { .. }));
    }

    #[test]
    fn test_choice_serialization() {
        let mut opts = HashMap::new();
        opts.insert("x".to_string(), Some("X desc".to_string()));
        let q = Question::choice("q2", "Pick x?", opts).unwrap();
        let json = serde_json::to_string(&q).unwrap();
        let back: Question = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "q2");
        match &back.question_type {
            QuestionType::Choice { criteria, .. } => {
                assert_eq!(criteria["x"].as_deref(), Some("X desc"));
            }
            _ => panic!("Expected Choice"),
        }
    }

    #[test]
    fn test_score_serialization() {
        let q = Question::score("q3", "Rate", vec!["A".into(), "B".into()]).unwrap();
        let json = serde_json::to_string(&q).unwrap();
        let back: Question = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "q3");
        match &back.question_type {
            QuestionType::Score { criteria, .. } => {
                assert_eq!(criteria, &vec!["A".to_string(), "B".to_string()]);
            }
            _ => panic!("Expected Score"),
        }
    }

    #[test]
    fn test_question_set_serialization() {
        let qs = QuestionSet {
            questions: vec![
                Question::noul("q1", "test1?"),
                Question::score("q2", "test2?", vec!["L".into(), "H".into()]).unwrap(),
            ],
        };
        let json = serde_json::to_string_pretty(&qs).unwrap();
        let back: QuestionSet = serde_json::from_str(&json).unwrap();
        assert_eq!(back.questions.len(), 2);
        assert_eq!(back.questions[0].id, "q1");
        assert_eq!(back.questions[1].id, "q2");
    }

    #[test]
    fn test_answer_serialization_noul() {
        let answer = Answer::Noul(NoulAnswer {
            noul: 0.85,
            needs_review: false,
            reason: Some("test reason".to_string()),
            status: DecisionStatus::Selected,
        });
        let json = serde_json::to_string(&answer).unwrap();
        let back: Answer = serde_json::from_str(&json).unwrap();
        match back {
            Answer::Noul(n) => {
                assert!((n.noul - 0.85).abs() < 0.001);
                assert!(!n.needs_review);
                assert_eq!(n.reason.as_deref(), Some("test reason"));
            }
            _ => panic!("Expected Noul"),
        }
    }

    #[test]
    fn test_answer_serialization_choice() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.9);
        probs.insert("b".to_string(), 0.1);
        let answer = Answer::Choice(ChoiceAnswer {
            choice: "a".to_string(),
            probabilities: probs,
            confidence: 0.8,
            margin: 0.8,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        });
        let json = serde_json::to_string(&answer).unwrap();
        let back: Answer = serde_json::from_str(&json).unwrap();
        match back {
            Answer::Choice(c) => {
                assert_eq!(c.choice, "a");
                assert!((c.confidence - 0.8).abs() < 0.001);
                assert!((c.margin - 0.8).abs() < 0.001);
            }
            _ => panic!("Expected Choice"),
        }
    }

    #[test]
    fn test_answer_serialization_score() {
        let mut probs = HashMap::new();
        probs.insert("0".to_string(), 0.3);
        probs.insert("1".to_string(), 0.7);
        let answer = Answer::Score(ScoreAnswer {
            score: 0.7,
            probabilities: probs,
            confidence: 0.6,
            legend: vec!["Low".into(), "High".into()],
            needs_review: false,
            reason: None,
            status: DecisionStatus::Scored,
        });
        let json = serde_json::to_string(&answer).unwrap();
        let back: Answer = serde_json::from_str(&json).unwrap();
        match back {
            Answer::Score(s) => {
                assert!((s.score - 0.7).abs() < 0.001);
                assert_eq!(s.legend.len(), 2);
            }
            _ => panic!("Expected Score"),
        }
    }

    // ── Confidence Computation ────────────────────────────────

    #[test]
    fn test_choice_confidence_certain() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 1.0);
        probs.insert("b".to_string(), 0.0);
        assert!((choice_confidence(&probs) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_choice_confidence_uniform() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.5);
        probs.insert("b".to_string(), 0.5);
        assert!((choice_confidence(&probs) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_score_confidence_concentrated() {
        let mut probs = HashMap::new();
        probs.insert("0".to_string(), 0.05);
        probs.insert("1".to_string(), 0.9);
        probs.insert("2".to_string(), 0.05);
        let conf = score_confidence(&probs);
        assert!(conf > 0.8, "Concentrated distribution should have high confidence: {}", conf);
    }

    #[test]
    fn test_validate_probabilities_ok() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.6);
        probs.insert("b".to_string(), 0.4);
        assert!(validate_probabilities(&probs).is_ok());
    }

    #[test]
    fn test_validate_probabilities_error() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.6);
        probs.insert("b".to_string(), 0.6);
        assert!(validate_probabilities(&probs).is_err());
    }

    // ── Edge Cases ─────────────────────────────────────────────

    #[test]
    fn test_noul_criteria_optional() {
        let q = Question::noul("q", "test?");
        match &q.question_type {
            QuestionType::Noul { criteria, .. } => {
                assert!(criteria.is_none());
            }
            _ => panic!("Expected Noul"),
        }
    }

    #[test]
    fn test_choice_empty_options_rejected() {
        let result = Question::choice("q", "test?", HashMap::new());
        assert!(result.is_err(), "Empty options should be rejected");
    }

    #[test]
    fn test_evaluation_result_with_usage() {
        let mut answers = HashMap::new();
        answers.insert("q1".to_string(), Answer::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        }));
        let result = EvaluationResult {
            answers,
            model: Some("test-model".to_string()),
            usage: Some(Usage { input_tokens: 100, output_tokens: 0 }),
        };
        let json = serde_json::to_string_pretty(&result).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_evaluation_result_minimal() {
        let result = EvaluationResult {
            answers: HashMap::new(),
            model: None,
            usage: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        // model and usage should be skipped
        assert!(!json.contains("model"));
        assert!(!json.contains("usage"));
    }
}
