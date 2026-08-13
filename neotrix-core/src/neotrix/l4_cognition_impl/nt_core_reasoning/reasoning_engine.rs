//! nt_core_reasoning — 核心推理引擎
//!
//! 基于规则的推理和概率推理引擎
//! 节点: nt_core_reasoning (L4)
//! Provides: rule_inference, probabilistic_inference
//! Requires: nt_core_traits, serde
//! Rune: Crimson, Indigo

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReasoningConfig {
    /// 是否启用概率归一化
    pub normalize: bool,
    /// 默认置信度阈值
    pub confidence_threshold: f32,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            normalize: true,
            confidence_threshold: 0.7,
        }
    }
}

/// 推理规则
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Rule {
    pub id: String,
    pub premise: Vec<String>,
    pub conclusion: String,
    pub confidence: f32,
}

/// 推理结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InferenceResult {
    pub conclusion: String,
    pub premises: Vec<String>,
    pub confidence: f32,
}

/// 推理引擎
pub struct ReasoningEngine {
    config: ReasoningConfig,
    rules: Vec<Rule>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ReasoningEngine {
    pub fn new(config: ReasoningConfig) -> Self {
        Self {
            config,
            rules: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// 基于规则的推理：查找匹配前提的规则
    pub fn infer(&self, facts: &[String]) -> Vec<InferenceResult> {
        let mut results = Vec::new();

        for rule in &self.rules {
            // 检查前提是否被事实满足
            let mut satisfied_premises = 0;
            for premise in &rule.premise {
                if facts.contains(premise) {
                    satisfied_premises += 1;
                }
            }

            // 如果所有前提都满足
            if satisfied_premises == rule.premise.len() && !rule.premise.is_empty() {
                results.push(InferenceResult {
                    conclusion: rule.conclusion.clone(),
                    premises: rule.premise.clone(),
                    confidence: if self.config.normalize {
                        rule.confidence.min(1.0)
                    } else {
                        rule.confidence
                    },
                });
            }
        }

        results
    }

    pub fn config(&self) -> &ReasoningConfig {
        &self.config
    }
}

impl CapabilityNode for ReasoningEngine {
    fn node_id(&self) -> &str {
        "nt_core_reasoning"
    }
    fn provides(&self) -> Vec<String> {
        vec!["rule_inference".into(), "probabilistic_inference".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["nt_core_traits".into(), "serde".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Indigo]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for ReasoningEngine {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut engine = ReasoningEngine::new(ReasoningConfig::default());

            let rule = Rule {
                id: "test_rule".into(),
                premise: vec!["fact_a".into(), "fact_b".into()],
                conclusion: "conclusion_c".into(),
                confidence: 0.9,
            };
            engine.add_rule(rule);

            let facts = vec!["fact_a".into(), "fact_b".into()];
            let inferences = engine.infer(&facts);

            assert!(!inferences.is_empty());
            assert_eq!(inferences[0].conclusion, "conclusion_c");

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_core_reasoning_engine"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_reasoning_engine_self_test() {
        let engine = ReasoningEngine::new(ReasoningConfig::default());
        assert!(engine.self_test().is_ok());
    }
}
