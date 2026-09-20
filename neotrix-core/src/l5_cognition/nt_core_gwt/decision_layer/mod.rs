//! # Jev DecisionLayer — 类型化快速决策层
//!
//! 借鉴 TypeSafe Jev (System One Model) 的核心模式：
//! - 输入: 结构化状态 + N 个选项
//! - 输出: 每个选项的校准概率 + 置信度
//! - 延迟: <100ms (比 LLM 路由快 40-200x)
//!
//! 用于 GWT 注意力路由的快速决策替代方案。

use serde::{Deserialize, Serialize};

/// 决策选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: String,
    pub label: String,
    pub description: String,
    /// 选项特征向量 (用于相似度计算)
    pub features: Vec<f64>,
}

/// 决策结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    /// 选中的选项 ID
    pub selected: String,
    /// 所有选项的校准概率 (和为 1.0)
    pub probabilities: Vec<(String, f64)>,
    /// 决策置信度 [0, 1]
    pub confidence: f64,
    /// 决策延迟 (ms)
    pub latency_ms: u64,
}

/// Jev-like 决策层 trait
pub trait DecisionLayer: Send + Sync {
    /// 从选项中做出快速决策
    fn decide(&self, context: &[f64], options: &[DecisionOption]) -> DecisionResult;
    
    /// 决策层名称
    fn name(&self) -> &str;
}

/// 基于注意力评分的本地决策层 (无需外部模型)
/// 使用余弦相似度 + softmax 生成校准概率
pub struct AttentionScoringDecisionLayer {
    /// 温度参数 (越小越确定性)
    temperature: f64,
}

impl AttentionScoringDecisionLayer {
    pub fn new(temperature: f64) -> Self {
        Self { temperature }
    }
    
    /// 余弦相似度
    fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }
    
    /// Softmax with temperature
    fn softmax(scores: &[f64], temperature: f64) -> Vec<f64> {
        let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_scores: Vec<f64> = scores.iter()
            .map(|s| ((s - max_score) / temperature).exp())
            .collect();
        let sum: f64 = exp_scores.iter().sum();
        exp_scores.iter().map(|s| s / sum).collect()
    }
}

impl DecisionLayer for AttentionScoringDecisionLayer {
    fn decide(&self, context: &[f64], options: &[DecisionOption]) -> DecisionResult {
        let start = std::time::Instant::now();
        
        if options.is_empty() {
            return DecisionResult {
                selected: String::new(),
                probabilities: vec![],
                confidence: 0.0,
                latency_ms: start.elapsed().as_millis() as u64,
            };
        }
        
        let scores: Vec<f64> = options.iter()
            .map(|opt| Self::cosine_similarity(context, &opt.features))
            .collect();
        
        let probabilities = Self::softmax(&scores, self.temperature);
        let probs_with_ids: Vec<(String, f64)> = options.iter()
            .zip(probabilities.iter())
            .map(|(opt, prob)| (opt.id.clone(), *prob))
            .collect();
        
        let max_prob = probabilities.iter().cloned().fold(0.0f64, f64::max);
        let selected_idx = probabilities.iter()
            .position(|p| *p == max_prob)
            .unwrap_or(0);
        
        DecisionResult {
            selected: options[selected_idx].id.clone(),
            probabilities: probs_with_ids,
            confidence: max_prob,
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }
    
    fn name(&self) -> &str {
        "attention_scoring"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decision_layer_basic() {
        let layer = AttentionScoringDecisionLayer::new(1.0);
        let context = vec![1.0, 0.0, 0.0];
        let options = vec![
            DecisionOption { id: "a".into(), label: "A".into(), description: "".into(), features: vec![1.0, 0.0, 0.0] },
            DecisionOption { id: "b".into(), label: "B".into(), description: "".into(), features: vec![0.0, 1.0, 0.0] },
        ];
        let result = layer.decide(&context, &options);
        assert_eq!(result.selected, "a");
        assert!(result.confidence > 0.5);
    }
}