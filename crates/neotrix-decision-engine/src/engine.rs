//! Core DecisionEngine implementation
//!
//! Provides structured decision-making with calibrated probabilities,
//! inspired by TypeSafe JEV.

use std::collections::HashMap;

use crate::types::*;
use crate::error::{Error, Result};

/// Trait for model inference backends
pub trait InferenceBackend {
    /// Generate a response from a prompt
    fn generate(&self, prompt: &str) -> Result<String>;
    
    /// Get model name for tracking
    fn model_name(&self) -> &str;
}

/// The main decision engine
pub struct DecisionEngine {
    backend: Box<dyn InferenceBackend>,
}

impl DecisionEngine {
    /// Create a new decision engine with a custom backend
    pub fn new(backend: Box<dyn InferenceBackend>) -> Self {
        Self { backend }
    }
    
    /// Evaluate state against questions and return answers
    pub fn evaluate(&self, state: &State, question_set: &QuestionSet) -> Result<EvaluationResult> {
        // Build prompt
        let prompt = self.build_prompt(state, &question_set.questions);
        
        // Generate response
        let response = self.backend.generate(&prompt)?;
        
        // Parse structured output
        let answers = self.parse_response(&response, &question_set.questions)?;
        
        Ok(EvaluationResult {
            answers,
            model: Some(self.backend.model_name().to_string()),
            usage: None,  // TODO: extract from response
        })
    }
    
    /// Build a prompt for the model
    fn build_prompt(&self, state: &State, questions: &[Question]) -> String {
        let mut prompt = String::from(
            "You are a structured decision engine. Given the state below, answer the questions.\n\
             Return ONLY a JSON object with answers keyed by question ID.\n\n"
        );
        
        // Add state
        prompt.push_str("## State\n");
        prompt.push_str(&state.content);
        if let Some(data) = &state.data {
            prompt.push_str("\n\n## Structured Data\n```json\n");
            prompt.push_str(&serde_json::to_string_pretty(data).unwrap_or_default());
            prompt.push_str("\n```");
        }
        prompt.push_str("\n\n## Questions\n");
        
        // Add questions
        for q in questions {
            prompt.push_str(&format!("\n### Question: {}\n", q.id));
            
            match &q.question_type {
                QuestionType::Noul { instructions, criteria } => {
                    prompt.push_str("Type: noul (boolean probability)\n");
                    prompt.push_str(&format!("Instructions: {}\n", instructions));
                    if let Some(c) = criteria {
                        if let Some(true_desc) = &c.r#true {
                            prompt.push_str(&format!("True means: {}\n", true_desc));
                        }
                        if let Some(false_desc) = &c.false_ {
                            prompt.push_str(&format!("False means: {}\n", false_desc));
                        }
                    }
                    prompt.push_str("Return: {\"noul\": <0.0-1.0>}\n");
                }
                
                QuestionType::Choice { instructions, criteria } => {
                    prompt.push_str("Type: choice (select one option)\n");
                    prompt.push_str(&format!("Instructions: {}\n", instructions));
                    prompt.push_str("Options:\n");
                    for (opt, desc) in criteria {
                        let desc_str = desc.as_deref().unwrap_or("No description");
                        prompt.push_str(&format!("  - {}: {}\n", opt, desc_str));
                    }
                    prompt.push_str("Return: {\"choice\": \"<selected>\", \"confidence\": <0.0-1.0>}\n");
                }
                
                QuestionType::Score { instructions, criteria } => {
                    prompt.push_str("Type: score (rate on scale)\n");
                    prompt.push_str(&format!("Instructions: {}\n", instructions));
                    prompt.push_str("Scale levels:\n");
                    for (i, level) in criteria.iter().enumerate() {
                        prompt.push_str(&format!("  {}: {}\n", i, level));
                    }
                    prompt.push_str(&format!("Return: {{\"score\": <0-{}>, \"confidence\": <0.0-1.0>}}\n", criteria.len() - 1));
                }
            }
        }
        
        prompt.push_str("\n## Response\nReturn a JSON object with keys matching question IDs.\n");
        
        prompt
    }
    
    /// Parse model response into structured answers
    fn parse_response(&self, response: &str, questions: &[Question]) -> Result<HashMap<String, Answer>> {
        // Try to extract JSON from response
        let json_str = self.extract_json(response)?;
        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| Error::ParseError(format!("Invalid JSON: {}", e)))?;
        
        let mut answers = HashMap::new();
        
        for q in questions {
            let answer_value = parsed.get(&q.id)
                .ok_or_else(|| Error::MissingAnswer(q.id.clone()))?;
            
            let answer = self.parse_answer(answer_value, &q.question_type)?;
            answers.insert(q.id.clone(), answer);
        }
        
        Ok(answers)
    }
    
    /// Extract JSON from response (handles markdown code blocks)
    fn extract_json(&self, response: &str) -> Result<String> {
        // Try to find JSON in code block
        if let Some(start) = response.find("```json") {
            let json_start = start + 7;
            if let Some(end) = response[json_start..].find("```") {
                return Ok(response[json_start..json_start + end].trim().to_string());
            }
        }
        
        // Try to find raw JSON (starts with { or [)
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                return Ok(response[start..=end].to_string());
            }
        }
        
        Err(Error::ParseError("No JSON found in response".to_string()))
    }
    
    /// Parse a single answer value
    fn parse_answer(&self, value: &serde_json::Value, question_type: &QuestionType) -> Result<Answer> {
        match question_type {
            QuestionType::Noul { .. } => {
                let prob = value.get("probability")
                    .or_else(|| value.get("noul"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);
                let val = value.get("value")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(prob >= 0.5);
                
                Ok(Answer::Noul(NoulAnswer {
                    value: val,
                    probability: prob.clamp(0.0, 1.0),
                    status: DecisionStatus::Selected,
                }))
            }
            
            QuestionType::Choice { criteria: _, .. } => {
                let selected = value.get("value")
                    .or_else(|| value.get("choice"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let probability = value.get("probability")
                    .or_else(|| value.get("confidence"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);
                
                let margin = value.get("margin")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                
                Ok(Answer::Choice(ChoiceAnswer {
                    status: DecisionStatus::Selected,
                    value: selected,
                    probability: probability.clamp(0.0, 1.0),
                    margin,
                }))
            }
            
            QuestionType::Score { .. } => {
                let val = value.get("value")
                    .or_else(|| value.get("score"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                
                let probabilities = value.get("probabilities")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                
                Ok(Answer::Score(ScoreAnswer {
                    status: DecisionStatus::Scored,
                    value: val,
                    probabilities,
                }))
            }
        }
    }
}

/// Helper to create a decision engine with default backend
impl Default for DecisionEngine {
    fn default() -> Self {
        // Use mock backend for now
        Self::new(Box::new(MockBackend))
    }
}

/// Mock backend for testing and development
struct MockBackend;

impl InferenceBackend for MockBackend {
    fn generate(&self, _prompt: &str) -> Result<String> {
        // Return a mock response
        Ok(r#"{
            "mock_question": {"noul": 0.5},
            "mock_choice": {"choice": "option_a", "confidence": 0.7},
            "mock_score": {"score": 1, "confidence": 0.6}
        }"#.to_string())
    }
    
    fn model_name(&self) -> &str {
        "mock-backend"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_prompt() {
        let engine = DecisionEngine::default();
        let state: State = "Test state".into();
        let questions = vec![
            Question::noul("q1", "Is this true?"),
        ];
        
        let prompt = engine.build_prompt(&state, &questions);
        
        assert!(prompt.contains("Test state"));
        assert!(prompt.contains("q1"));
        assert!(prompt.contains("noul"));
    }

    #[test]
    fn test_build_prompt_choice() {
        let engine = DecisionEngine::default();
        let state: State = "Database error".into();
        let mut opts = HashMap::new();
        opts.insert("restart".to_string(), Some("Restart DB".to_string()));
        let questions = vec![
            Question::choice("action", "What to do?", opts).unwrap(),
        ];

        let prompt = engine.build_prompt(&state, &questions);
        assert!(prompt.contains("choice"));
        assert!(prompt.contains("restart"));
        assert!(prompt.contains("Restart DB"));
    }

    #[test]
    fn test_build_prompt_score() {
        let engine = DecisionEngine::default();
        let state: State = "API error".into();
        let questions = vec![
            Question::score("severity", "Rate it", vec!["Low".into(), "High".into()]).unwrap(),
        ];

        let prompt = engine.build_prompt(&state, &questions);
        assert!(prompt.contains("score"));
        assert!(prompt.contains("Low"));
        assert!(prompt.contains("High"));
    }

    #[test]
    fn test_build_prompt_with_data() {
        let engine = DecisionEngine::default();
        let state = State {
            content: "Metrics snapshot".to_string(),
            data: Some(serde_json::json!({"cpu": 95})),
        };
        let questions = vec![Question::noul("q1", "test?")];

        let prompt = engine.build_prompt(&state, &questions);
        assert!(prompt.contains("Structured Data"));
        assert!(prompt.contains("cpu"));
    }

    #[test]
    fn test_build_prompt_multi_question() {
        let engine = DecisionEngine::default();
        let state: State = "Deploy ready".into();
        let questions = vec![
            Question::noul("ready", "Ready?"),
            Question::score("risk", "Risk level", vec!["L".into(), "H".into()]).unwrap(),
        ];

        let prompt = engine.build_prompt(&state, &questions);
        assert!(prompt.contains("ready"));
        assert!(prompt.contains("risk"));
    }
    
    #[test]
    fn test_parse_json_from_code_block() {
        let engine = DecisionEngine::default();
        let response = r#"
Some text before
```json
{"test": 123}
```
Some text after
"#;
        
        let json = engine.extract_json(response).unwrap();
        assert_eq!(json, r#"{"test": 123}"#);
    }

    #[test]
    fn test_parse_json_from_code_block_no_lang() {
        let engine = DecisionEngine::default();
        let response = r#"
```
{"key": "val"}
```
"#;
        let json = engine.extract_json(response).unwrap();
        assert_eq!(json, r#"{"key": "val"}"#);
    }
    
    #[test]
    fn test_parse_raw_json() {
        let engine = DecisionEngine::default();
        let response = "Here is the result: {\"key\": \"value\"} done.";
        
        let json = engine.extract_json(response).unwrap();
        assert_eq!(json, r#"{"key": "value"}"#);
    }

    #[test]
    fn test_parse_json_nested() {
        let engine = DecisionEngine::default();
        let response = r#"{"outer": {"inner": [1, 2, 3]}}"#;
        let json = engine.extract_json(response).unwrap();
        assert!(json.contains("outer"));
        assert!(json.contains("inner"));
    }

    #[test]
    fn test_extract_json_no_json() {
        let engine = DecisionEngine::default();
        let result = engine.extract_json("no json here at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_noul_answer() {
        let engine = DecisionEngine::default();
        let value = serde_json::json!({"noul": 0.85});
        let qt = QuestionType::Noul {
            instructions: "test".to_string(),
            criteria: None,
        };
        
        let answer = engine.parse_answer(&value, &qt).unwrap();
        match answer {
            Answer::Noul(n) => assert!((n.probability - 0.85).abs() < 0.001),
            _ => panic!("Expected Noul answer"),
        }
    }

    #[test]
    fn test_parse_noul_clamped() {
        let engine = DecisionEngine::default();
        let value = serde_json::json!({"noul": 1.5});
        let qt = QuestionType::Noul { instructions: "test".into(), criteria: None };
        let answer = engine.parse_answer(&value, &qt).unwrap();
        match answer {
            Answer::Noul(n) => assert!(n.probability <= 1.0),
            _ => panic!("Expected Noul"),
        }
    }

    #[test]
    fn test_parse_choice_answer() {
        let engine = DecisionEngine::default();
        let mut criteria = HashMap::new();
        criteria.insert("a".to_string(), None);
        criteria.insert("b".to_string(), None);
        let value = serde_json::json!({"choice": "a", "confidence": 0.9});
        let qt = QuestionType::Choice { instructions: "test".into(), criteria };
        let answer = engine.parse_answer(&value, &qt).unwrap();
        match answer {
            Answer::Choice(c) => {
                assert_eq!(c.value, "a");
                assert!((c.probability - 0.9).abs() < 0.001);
            }
            _ => panic!("Expected Choice"),
        }
    }

    #[test]
    fn test_parse_score_answer() {
        let engine = DecisionEngine::default();
        let value = serde_json::json!({"score": 2, "confidence": 0.8});
        let qt = QuestionType::Score {
            instructions: "test".into(),
            criteria: vec!["L".into(), "M".into(), "H".into()],
        };
        let answer = engine.parse_answer(&value, &qt).unwrap();
        match answer {
            Answer::Score(s) => {
                assert!((s.value - 2.0).abs() < 0.001);
                // probabilities from input JSON; empty if not provided
            }
            _ => panic!("Expected Score"),
        }
    }

    #[test]
    fn test_parse_answer_missing_field() {
        let engine = DecisionEngine::default();
        let value = serde_json::json!({});
        let qt = QuestionType::Noul { instructions: "test".into(), criteria: None };
        let result = engine.parse_answer(&value, &qt);
        // Missing fields get defaults (matches JEV: probability defaults to 0.5)
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_mock() {
        let engine = DecisionEngine::default();
        let state: State = "Test state".into();
        let question_set = QuestionSet {
            questions: vec![
                Question::noul("mock_question", "Is this true?"),
            ],
        };
        
        let result = engine.evaluate(&state, &question_set).unwrap();
        assert!(result.answers.contains_key("mock_question"));
    }

    #[test]
    fn test_evaluate_mock_multi() {
        let engine = DecisionEngine::default();
        let state: State = "Test state".into();
        let mut opts = HashMap::new();
        opts.insert("a".to_string(), None);
        let question_set = QuestionSet {
            questions: vec![
                Question::noul("mock_question", "test?"),
                Question::choice("mock_choice", "pick", opts).unwrap(),
                Question::score("mock_score", "rate", vec!["L".into(), "H".into()]).unwrap(),
            ],
        };

        let result = engine.evaluate(&state, &question_set).unwrap();
        // Mock backend returns "mock_question", "mock_choice", "mock_score"
        assert_eq!(result.answers.len(), 3);
    }
}
