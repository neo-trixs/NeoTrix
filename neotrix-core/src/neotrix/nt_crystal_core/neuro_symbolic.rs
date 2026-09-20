//! NeuroSymbolicEngine — 神经符号融合推理
//!
//! 基于 AAAI 2026 的 Neuro-Symbolic 趋势:
//! - 神经组件: 模式识别、感知
//! - 符号组件: 逻辑推理、规划
//! - 桥接层: 神经 <-> 符号双向转换

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// 神经组件
// ═══════════════════════════════════════════════════════════════

/// 神经感知结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralPerception {
    pub embedding: Vec<f64>,
    pub confidence: f64,
    pub features: HashMap<String, f64>,
    pub attention_weights: Vec<f64>,
}

/// 神经组件 — 模式识别 + 感知
pub struct NeuralComponent {
    pub input_dim: usize,
    pub hidden_dim: usize,
    pub output_dim: usize,
}

impl NeuralComponent {
    pub fn new(input_dim: usize, hidden_dim: usize, output_dim: usize) -> Self {
        Self { input_dim, hidden_dim, output_dim }
    }

    /// 感知输入，产生神经表示
    pub fn perceive(&self, input: &[f64]) -> NeuralPerception {
        let mut hidden = vec![0.0; self.hidden_dim];
        for (i, h) in hidden.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (j, &x) in input.iter().enumerate() {
                let w = ((i * self.input_dim + j) as f64).sin();
                sum += x * w;
            }
            *h = sum.tanh();
        }

        let mut output = vec![0.0; self.output_dim];
        for (i, o) in output.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (j, &h) in hidden.iter().enumerate() {
                let w = ((i * self.hidden_dim + j) as f64).cos();
                sum += h * w;
            }
            *o = sum.tanh();
        }

        let confidence = output.iter().map(|x| x * x).sum::<f64>().sqrt() / self.output_dim as f64;

        let features: HashMap<String, f64> = output.iter().enumerate()
            .map(|(i, &v)| (format!("f{}", i), v))
            .collect();

        let attention_weights: Vec<f64> = hidden.iter().map(|h| h.abs()).collect();
        let att_sum: f64 = attention_weights.iter().sum();
        let attention_weights: Vec<f64> = if att_sum > 0.0 {
            attention_weights.iter().map(|a| a / att_sum).collect()
        } else {
            vec![1.0 / self.hidden_dim as f64; self.hidden_dim]
        };

        NeuralPerception {
            embedding: output,
            confidence,
            features,
            attention_weights,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 符号组件
// ═══════════════════════════════════════════════════════════════

/// 符号命题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicProposition {
    pub predicate: String,
    pub args: Vec<String>,
    pub truth_value: f64,
}

/// 符号规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicRule {
    pub name: String,
    pub premises: Vec<SymbolicProposition>,
    pub conclusion: SymbolicProposition,
    pub confidence: f64,
}

/// 符号推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicResult {
    pub conclusions: Vec<SymbolicProposition>,
    pub applied_rules: Vec<String>,
    pub confidence: f64,
}

/// 符号组件 — 逻辑推理 + 规划
pub struct SymbolicComponent {
    pub rules: Vec<SymbolicRule>,
    pub knowledge_base: Vec<SymbolicProposition>,
}

impl SymbolicComponent {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            knowledge_base: Vec::new(),
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: SymbolicRule) {
        self.rules.push(rule);
    }

    /// 添加命题到知识库
    pub fn add_proposition(&mut self, prop: SymbolicProposition) {
        self.knowledge_base.push(prop);
    }

    /// 前向推理
    pub fn forward_reason(&self) -> SymbolicResult {
        let mut conclusions = Vec::new();
        let mut applied_rules = Vec::new();

        for rule in &self.rules {
            let premises_met = rule.premises.iter().all(|p| {
                self.knowledge_base.iter().any(|kb| {
                    kb.predicate == p.predicate && kb.args == p.args && kb.truth_value > 0.5
                })
            });

            if premises_met {
                conclusions.push(rule.conclusion.clone());
                applied_rules.push(rule.name.clone());
            }
        }

        let confidence = if conclusions.is_empty() {
            0.0
        } else {
            conclusions.iter().map(|c| c.truth_value).sum::<f64>() / conclusions.len() as f64
        };

        SymbolicResult {
            conclusions,
            applied_rules,
            confidence,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 神经-符号桥接
// ═══════════════════════════════════════════════════════════════

/// 桥接: 神经 -> 符号
pub struct NeuralSymbolicBridge {
    /// 特征到谓词的映射
    pub feature_to_predicate: HashMap<String, String>,
    /// 阈值
    pub threshold: f64,
}

impl NeuralSymbolicBridge {
    pub fn new() -> Self {
        Self {
            feature_to_predicate: HashMap::new(),
            threshold: 0.5,
        }
    }

    /// 神经感知 -> 符号命题
    pub fn neural_to_symbolic(&self, perception: &NeuralPerception) -> Vec<SymbolicProposition> {
        let mut propositions = Vec::new();

        for (feature_name, &value) in &perception.features {
            if value.abs() > self.threshold {
                if let Some(predicate) = self.feature_to_predicate.get(feature_name) {
                    propositions.push(SymbolicProposition {
                        predicate: predicate.clone(),
                        args: vec![feature_name.clone()],
                        truth_value: value.abs(),
                    });
                } else {
                    propositions.push(SymbolicProposition {
                        predicate: format!("high_{}", feature_name),
                        args: vec![feature_name.clone()],
                        truth_value: value.abs(),
                    });
                }
            }
        }

        propositions
    }

    /// 符号结果 -> 神经嵌入
    pub fn symbolic_to_neural(&self, result: &SymbolicResult, output_dim: usize) -> Vec<f64> {
        let mut embedding = vec![0.0; output_dim];

        for (i, conclusion) in result.conclusions.iter().enumerate() {
            if i >= output_dim {
                break;
            }
            embedding[i] = conclusion.truth_value;
        }

        // 归一化
        let norm: f64 = embedding.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for e in &mut embedding {
                *e /= norm;
            }
        }

        embedding
    }
}

// ═══════════════════════════════════════════════════════════════
// 融合结果
// ═══════════════════════════════════════════════════════════════

/// 混合推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResult {
    pub neural_perception: NeuralPerception,
    pub symbolic_result: SymbolicResult,
    pub fused_embedding: Vec<f64>,
    pub fused_confidence: f64,
    pub reasoning_trace: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════
// NeuroSymbolicEngine 主引擎
// ═══════════════════════════════════════════════════════════════

/// NeuroSymbolicEngine — 神经符号融合推理
pub struct NeuroSymbolicEngine {
    pub neural: NeuralComponent,
    pub symbolic: SymbolicComponent,
    pub bridge: NeuralSymbolicBridge,
}

impl NeuroSymbolicEngine {
    pub fn new(input_dim: usize, hidden_dim: usize, output_dim: usize) -> Self {
        Self {
            neural: NeuralComponent::new(input_dim, hidden_dim, output_dim),
            symbolic: SymbolicComponent::new(),
            bridge: NeuralSymbolicBridge::new(),
        }
    }

    /// 混合推理: 神经感知 + 符号推理 + 融合
    pub fn hybrid_reasoning(&self, input: &[f64]) -> ReasoningResult {
        let mut trace = Vec::new();

        // 1. 神经感知
        let neural_perception = self.neural.perceive(input);
        trace.push(format!("Neural perception: confidence={:.3}", neural_perception.confidence));

        // 2. 神经 -> 符号
        let propositions = self.bridge.neural_to_symbolic(&neural_perception);
        trace.push(format!("Converted {} propositions to symbolic", propositions.len()));

        // 3. 符号推理
        let symbolic_result = self.symbolic.forward_reason();
        trace.push(format!("Symbolic reasoning: {} conclusions", symbolic_result.conclusions.len()));

        // 4. 符号 -> 神经
        let symbolic_embedding = self.bridge.symbolic_to_neural(&symbolic_result, neural_perception.embedding.len());

        // 5. 融合
        let fused_embedding: Vec<f64> = neural_perception.embedding.iter()
            .zip(symbolic_embedding.iter())
            .map(|(n, s)| 0.7 * n + 0.3 * s)
            .collect();

        let fused_confidence = 0.7 * neural_perception.confidence + 0.3 * symbolic_result.confidence;
        trace.push(format!("Fused confidence: {:.3}", fused_confidence));

        ReasoningResult {
            neural_perception,
            symbolic_result,
            fused_embedding,
            fused_confidence,
            reasoning_trace: trace,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_perception() {
        let neural = NeuralComponent::new(8, 16, 8);
        let input: Vec<f64> = (0..8).map(|i| i as f64).collect();
        let perception = neural.perceive(&input);
        assert!(perception.confidence > 0.0);
        assert_eq!(perception.embedding.len(), 8);
    }

    #[test]
    fn test_symbolic_reasoning() {
        let mut symbolic = SymbolicComponent::new();
        symbolic.add_proposition(SymbolicProposition {
            predicate: "has_type".to_string(),
            args: vec!["x".to_string()],
            truth_value: 0.9,
        });
        symbolic.add_rule(SymbolicRule {
            name: "type_rule".to_string(),
            premises: vec![SymbolicProposition {
                predicate: "has_type".to_string(),
                args: vec!["x".to_string()],
                truth_value: 0.9,
            }],
            conclusion: SymbolicProposition {
                predicate: "is_valid".to_string(),
                args: vec!["x".to_string()],
                truth_value: 0.85,
            },
            confidence: 0.9,
        });

        let result = symbolic.forward_reason();
        assert_eq!(result.conclusions.len(), 1);
        assert_eq!(result.conclusions[0].predicate, "is_valid");
    }

    #[test]
    fn test_hybrid_reasoning() {
        let mut engine = NeuroSymbolicEngine::new(8, 16, 8);
        engine.symbolic.add_rule(SymbolicRule {
            name: "test_rule".to_string(),
            premises: vec![],
            conclusion: SymbolicProposition {
                predicate: "always_true".to_string(),
                args: vec![],
                truth_value: 1.0,
            },
            confidence: 1.0,
        });

        let input: Vec<f64> = (0..8).map(|i| i as f64).collect();
        let result = engine.hybrid_reasoning(&input);
        assert!(result.fused_confidence > 0.0);
        assert!(!result.reasoning_trace.is_empty());
    }
}
