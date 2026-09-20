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
    /// Boolean decision value (true/false)
    pub value: bool,
    /// Probability that the proposition is true (0.0 ~ 1.0)
    pub probability: f64,
    /// Decision status
    pub status: DecisionStatus,
}

/// Answer to a Choice question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceAnswer {
    /// Decision status
    pub status: DecisionStatus,
    /// Selected option
    pub value: String,
    /// Probability of the selected option (0.0 ~ 1.0)
    pub probability: f64,
    /// Margin: gap between top-1 and top-2 probabilities
    pub margin: f64,
}

/// Answer to a Score question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreAnswer {
    /// Decision status
    pub status: DecisionStatus,
    /// Score value (0 ~ max_level)
    pub value: f64,
    /// Probability distribution across all levels
    pub probabilities: HashMap<String, f64>,
}

/// Unified answer type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Answer {
    Noul(NoulAnswer),
    Choice(ChoiceAnswer),
    Score(ScoreAnswer),
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
            value: true,
            probability: 0.85,
            status: DecisionStatus::Selected,
        });
        let json = serde_json::to_string(&answer).unwrap();
        let back: Answer = serde_json::from_str(&json).unwrap();
        match back {
            Answer::Noul(n) => {
                assert!(n.value);
                assert!((n.probability - 0.85).abs() < 0.001);
            }
            _ => panic!("Expected Noul"),
        }
    }

    #[test]
    fn test_answer_serialization_choice() {
        let answer = Answer::Choice(ChoiceAnswer {
            status: DecisionStatus::Selected,
            value: "a".to_string(),
            probability: 0.9,
            margin: 0.7,
        });
        let json = serde_json::to_string(&answer).unwrap();
        let back: Answer = serde_json::from_str(&json).unwrap();
        match back {
            Answer::Choice(c) => {
                assert_eq!(c.value, "a");
                assert!((c.probability - 0.9).abs() < 0.001);
                assert!((c.margin - 0.7).abs() < 0.001);
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
            status: DecisionStatus::Scored,
            value: 0.7,
            probabilities: probs,
        });
        let json = serde_json::to_string(&answer).unwrap();
        let back: Answer = serde_json::from_str(&json).unwrap();
        match back {
            Answer::Score(s) => {
                assert!((s.value - 0.7).abs() < 0.001);
            }
            _ => panic!("Expected Score"),
        }
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
            value: true,
            probability: 0.5,
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
