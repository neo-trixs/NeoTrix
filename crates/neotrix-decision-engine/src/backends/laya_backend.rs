//! Laya backend using Python FFI
//!
//! This module provides a backend that calls the Python Laya library via FFI.

use std::collections::HashMap;
use crate::engine::InferenceBackend;
use crate::error::{Error, Result};
use crate::types::*;

/// Python FFI backend using Laya
pub struct LayaBackend {
    model_path: String,
}

impl LayaBackend {
    /// Create a new Laya backend
    pub fn new(model_path: &str) -> Self {
        Self {
            model_path: model_path.to_string(),
        }
    }
    
    /// Evaluate using Python subprocess (simpler than FFI)
    fn evaluate_via_python(&self, state: &State, questions: &[Question]) -> Result<String> {
        // 构造 Python 脚本
        let script = self.build_python_script(state, questions)?;
        
        // 执行 Python 脚本
        let output = std::process::Command::new("python3")
            .arg("-c")
            .arg(&script)
            .output()
            .map_err(|e| Error::InferenceError(format!("Python execution failed: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::InferenceError(format!("Python error: {}", stderr)));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
    
    /// Build Python script for evaluation
    fn build_python_script(&self, state: &State, questions: &[Question]) -> Result<String> {
        let state_json = serde_json::to_string(&state)?;
        let questions_json = serde_json::to_string(questions)?;
        
        let script = format!(
            r#"
import json
import sys
sys.path.insert(0, '{model_path}')

import laya

# Load state
state = json.loads('{state_json}')

# Load questions
questions = json.loads('{questions_json}')

# Load model
agent = laya.load('{model_path}')

# Evaluate
result = agent.predict(state, questions)

# Output as JSON
print(json.dumps(result))
"#,
            model_path = self.model_path,
            state_json = state_json.replace('\'', "\\'"),
            questions_json = questions_json.replace('\'', "\\'"),
        );
        
        Ok(script)
    }
}

impl InferenceBackend for LayaBackend {
    fn generate(&self, _prompt: &str) -> Result<String> {
        // Laya doesn't generate text, it returns structured decisions
        Err(Error::InferenceError(
            "LayaBackend does not support text generation. Use evaluate() instead.".to_string()
        ))
    }
    
    fn model_name(&self) -> &str {
        "laya-python"
    }
}

/// Direct evaluation using Laya
impl LayaBackend {
    pub fn evaluate_direct(&self, state: &State, questions: &[Question]) -> Result<EvaluationResult> {
        let output = self.evaluate_via_python(state, questions)?;
        
        // 解析 Python 输出
        let result: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| Error::ParseError(format!("Failed to parse Laya output: {}", e)))?;
        
        // 转换为 EvaluationResult
        self.parse_laya_result(&result)
    }
    
    fn parse_laya_result(&self, result: &serde_json::Value) -> Result<EvaluationResult> {
        let mut answers = HashMap::new();
        
        if let Some(answers_obj) = result.get("answers") {
            for (qid, answer_val) in answers_obj.as_object().unwrap_or(&serde_json::Map::new()) {
                let answer = self.parse_answer(answer_val)?;
                answers.insert(qid.clone(), answer);
            }
        }
        
        let model = result.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("laya")
            .to_string();
        
        let usage = result.get("usage").map(|u| {
            Usage {
                input_tokens: u.get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                output_tokens: u.get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
            }
        });
        
        Ok(EvaluationResult {
            answers,
            model: Some(model),
            usage,
        })
    }
    
    fn parse_answer(&self, val: &serde_json::Value) -> Result<Answer> {
        let answer_type = val.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::ParseError("Missing answer type".to_string()))?;
        
        match answer_type {
            "choice" => {
                let choice = val.get("choice")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                
                let probabilities: HashMap<String, f64> = val.get("probabilities")
                    .and_then(|v| v.as_object())
                    .map(|m| m.iter()
                        .map(|(k, v)| (k.clone(), v.as_f64().unwrap_or(0.0)))
                        .collect()
                    )
                    .unwrap_or_default();
                
                let confidence = val.get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or_else(|| crate::types::choice_confidence(&probabilities));
                
                let margin = val.get("margin")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                
                let needs_review = val.get("needs_review")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(confidence < 0.5 || margin < 0.1);
                
                let reason = val.get("reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                Ok(Answer::Choice(ChoiceAnswer {
                    choice,
                    probabilities,
                    confidence,
                    margin,
                    needs_review,
                    reason,
                    status: if needs_review {
                        DecisionStatus::Review
                    } else {
                        DecisionStatus::Selected
                    },
                }))
            }
            
            "score" => {
                let score = val.get("score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                
                let probabilities: HashMap<String, f64> = val.get("probabilities")
                    .and_then(|v| v.as_object())
                    .map(|m| m.iter()
                        .map(|(k, v)| (k.clone(), v.as_f64().unwrap_or(0.0)))
                        .collect()
                    )
                    .unwrap_or_default();
                
                let confidence = val.get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or_else(|| crate::types::score_confidence(&probabilities));
                
                let legend: Vec<String> = val.get("legend")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                
                let needs_review = val.get("needs_review")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(confidence < 0.4);
                
                let reason = val.get("reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                Ok(Answer::Score(ScoreAnswer {
                    score,
                    probabilities,
                    confidence,
                    legend,
                    needs_review,
                    reason,
                    status: if needs_review {
                        DecisionStatus::Review
                    } else {
                        DecisionStatus::Scored
                    },
                }))
            }
            
            "noul" => {
                let prob = val.get("noul")
                    .or_else(|| val.get("probability"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);
                
                let needs_review = val.get("needs_review")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(!(0.1..=0.9).contains(&prob));
                
                let reason = val.get("reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                Ok(Answer::Noul(NoulAnswer {
                    noul: prob,
                    needs_review,
                    reason,
                    status: if needs_review {
                        DecisionStatus::Review
                    } else {
                        DecisionStatus::Selected
                    },
                }))
            }
            
            _ => Err(Error::ParseError(format!("Unknown answer type: {}", answer_type)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_laya_result() {
        let backend = LayaBackend::new("/tmp/test");
        
        let result = serde_json::json!({
            "model": "laya",
            "answers": {
                "dept": {
                    "type": "choice",
                    "choice": "billing",
                    "confidence": 0.94,
                    "probabilities": {
                        "billing": 0.94,
                        "technical": 0.06
                    }
                }
            }
        });
        
        let eval_result = backend.parse_laya_result(&result).unwrap();
        assert!(eval_result.answers.contains_key("dept"));
        
        match &eval_result.answers["dept"] {
            Answer::Choice(c) => {
                assert_eq!(c.choice, "billing");
                assert!((c.confidence - 0.94).abs() < 0.01);
            }
            _ => panic!("Expected Choice answer"),
        }
    }
}
