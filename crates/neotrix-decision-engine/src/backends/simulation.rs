//! Agent Simulation backend
//!
//! Fallback when the model is unavailable. Returns JEV-compatible simulation output
//! with `probability: null`, `confidence: null`, `needs_review: true`.
//!
//! # JEV Simulation Mode Contract
//!
//! - `mode: "agent_simulation"`, `jev_called: false`
//! - Uses same question IDs, criteria, and candidate definitions
//! - For `choice`: selects a supplied label (first match or random)
//! - For `noul`: returns a boolean based on simple heuristics
//! - For `score`: chooses an anchored rubric level
//! - If evidence insufficient: `value: null`, `needs_review: true`
//! - **Never** invents calibrated probabilities or distributions
//! - Consent is per-task, not permanent

use std::collections::HashMap;
use crate::types::*;
use crate::error::Result;
use crate::engine::InferenceBackend;

/// Simulation backend for agent fallback mode
pub struct SimulationBackend {
    /// Mode label for output
    pub mode: String,
}

impl SimulationBackend {
    pub fn new() -> Self {
        Self {
            mode: "agent_simulation".to_string(),
        }
    }
}

impl Default for SimulationBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceBackend for SimulationBackend {
    fn generate(&self, _prompt: &str) -> Result<String> {
        // Simulation mode doesn't generate — it uses heuristics
        Ok("{}".to_string())
    }

    fn model_name(&self) -> &str {
        "agent-simulation"
    }
}

/// Generate simulation answers for questions using simple heuristics
///
/// Returns JEV-compatible output with `probability: null`, `confidence: null`
pub fn simulate_answers(questions: &[Question]) -> HashMap<String, Answer> {
    let mut answers = HashMap::new();

    for q in questions {
        let answer = match &q.question_type {
            QuestionType::Noul { instructions, .. } => {
                // Heuristic: if instructions contain negative words, return false
                let lower = instructions.to_lowercase();
                let is_false = lower.contains("not")
                    || lower.contains("fail")
                    || lower.contains("error")
                    || lower.contains("missing");

                Answer::Noul(NoulAnswer {
                    noul: if is_false { 0.2 } else { 0.8 },
                    needs_review: true, // Always needs review in simulation
                    reason: Some("Agent simulation: heuristic based on instruction keywords".to_string()),
                    status: DecisionStatus::Review,
                })
            }
            QuestionType::Choice { criteria, .. } => {
                // Select first option as default
                let first_key = criteria.keys().next()
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());

                Answer::Choice(ChoiceAnswer {
                    choice: first_key.clone(),
                    probabilities: HashMap::new(), // No distribution in simulation
                    confidence: 0.0, // No confidence in simulation
                    margin: 0.0,
                    needs_review: true,
                    reason: Some("Agent simulation: first option selected".to_string()),
                    status: DecisionStatus::Review,
                })
            }
            QuestionType::Score { criteria, .. } => {
                // Select middle level
                let mid = criteria.len() / 2;

                Answer::Score(ScoreAnswer {
                    score: mid as f64,
                    probabilities: HashMap::new(), // No distribution in simulation
                    confidence: 0.0,
                    legend: criteria.iter().enumerate()
                        .map(|(i, c)| format!("Level {}: {}", i, c))
                        .collect(),
                    needs_review: true,
                    reason: Some("Agent simulation: middle level selected".to_string()),
                    status: DecisionStatus::Review,
                })
            }
        };
        answers.insert(q.id.clone(), answer);
    }

    answers
}

/// JEV simulation response envelope
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimulationResponse {
    /// Always "agent_simulation"
    pub mode: String,
    /// Always false
    pub jev_called: bool,
    /// Answers keyed by question ID
    pub answers: HashMap<String, serde_json::Value>,
}

impl SimulationResponse {
    /// Create from questions (produces JEV-compatible simulation output)
    pub fn from_questions(questions: &[Question]) -> Self {
        let answers = simulate_answers(questions);

        // Convert to JEV simulation format (probability/confidence as null)
        let mut output_answers = HashMap::new();
        for (id, answer) in answers {
            let mut val = serde_json::to_value(&answer).unwrap_or(serde_json::Value::Null);
            // Force probability/confidence to null for simulation mode
            if let Some(obj) = val.as_object_mut() {
                obj.insert("probability".to_string(), serde_json::Value::Null);
                obj.insert("confidence".to_string(), serde_json::Value::Null);
            }
            output_answers.insert(id, val);
        }

        Self {
            mode: "agent_simulation".to_string(),
            jev_called: false,
            answers: output_answers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_noul() {
        let questions = vec![
            Question::noul("q1", "Is this critical?"),
            Question::noul("q2", "Does this not work?"),
        ];
        let answers = simulate_answers(&questions);
        assert_eq!(answers.len(), 2);

        if let Answer::Noul(n) = &answers["q1"] {
            assert!(n.needs_review); // Always true in sim
            assert!(n.reason.is_some());
        } else {
            panic!("Expected Noul");
        }
    }

    #[test]
    fn test_simulation_choice() {
        let mut opts = HashMap::new();
        opts.insert("a".to_string(), None);
        opts.insert("b".to_string(), None);
        let questions = vec![
            Question::choice("q1", "Pick one", opts).unwrap(),
        ];
        let answers = simulate_answers(&questions);

        if let Answer::Choice(c) = &answers["q1"] {
            assert_eq!(c.choice, "a"); // First option
            assert!(c.needs_review);
            assert!(c.probabilities.is_empty()); // No distribution
        } else {
            panic!("Expected Choice");
        }
    }

    #[test]
    fn test_simulation_response_format() {
        let questions = vec![
            Question::noul("q1", "Test?"),
        ];
        let resp = SimulationResponse::from_questions(&questions);
        assert_eq!(resp.mode, "agent_simulation");
        assert!(!resp.jev_called);
        assert!(resp.answers.contains_key("q1"));
    }
}
