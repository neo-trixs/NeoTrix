//! nt_core_abduction — 归纳推理（最佳解释）
//!
//! 基于证据的最佳解释推理，用于假设生成和理论选择
//! 节点: nt_core_abduction (L4)
//! Provides: abduction, hypothesis_generation
//! Requires: nt_core_traits, serde
//! Rune: Crimson, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AbductionConfig {
    /// 候选假设数量
    pub candidate_count: usize,
    /// 置信度阈值
    pub confidence_threshold: f32,
}

impl Default for AbductionConfig {
    fn default() -> Self {
        Self {
            candidate_count: 5,
            confidence_threshold: 0.6,
        }
    }
}

/// 候选假设
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub explanation: String,
    pub confidence: f32,
    pub evidence_support: f32,
}

/// 归纳推理引擎
pub struct AbductionEngine {
    config: AbductionConfig,
    hypotheses: Vec<Hypothesis>,
    evidence: Vec<String>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl AbductionEngine {
    pub fn new(config: AbductionConfig) -> Self {
        Self {
            config,
            hypotheses: Vec::new(),
            evidence: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_evidence(&mut self, evidence: String) {
        self.evidence.push(evidence);
    }

    /// 生成最佳解释（简化实现：基于证据关键词匹配）
    pub fn generate_hypotheses(&self) -> Vec<Hypothesis> {
        let mut hypotheses = Vec::new();

        for (i, ev) in self.evidence.iter().enumerate() {
            let confidence = if ev.len() > 5 { 0.8 } else { 0.5 };
            if confidence >= self.config.confidence_threshold {
                hypotheses.push(Hypothesis {
                    id: format!("hyp_{}", i),
                    explanation: ev.clone(),
                    confidence,
                    evidence_support: 1.0,
                });
            }
        }

        // 按置信度排序
        hypotheses.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 返回前 candidate_count 个
        hypotheses
            .into_iter()
            .take(self.config.candidate_count)
            .collect()
    }

    pub fn config(&self) -> &AbductionConfig {
        &self.config
    }
}

impl CapabilityNode for AbductionEngine {
    fn node_id(&self) -> &str {
        "nt_core_abduction"
    }
    fn provides(&self) -> Vec<String> {
        vec!["abduction".into(), "hypothesis_generation".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["nt_core_traits".into(), "serde".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for AbductionEngine {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut ab = AbductionEngine::new(AbductionConfig::default());

            ab.add_evidence("用户报告系统崩溃".into());
            ab.add_evidence("数据库连接超时".into());

            let hypotheses = ab.generate_hypotheses();
            assert!(!hypotheses.is_empty());

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_core_abduction_engine"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_abduction_engine_self_test() {
        let ab = AbductionEngine::new(AbductionConfig::default());
        assert!(ab.self_test().is_ok());
    }
}
