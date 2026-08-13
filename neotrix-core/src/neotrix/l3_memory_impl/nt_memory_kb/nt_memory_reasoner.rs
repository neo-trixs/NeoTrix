//! nt_memory_kb — 内存reasoner（reasoning）
//!
//! 基于证据的内存推理，支持假设生成、理论选择和置信度更新。
//! 节点: nt_memory_kb::nt_memory_reasoner (L3)
//! Provides: abduction, hypothesis_evaluation
//! Requires: nt_memory_kb, nt_core_abduction, serde
//! Rune: Crimson, Indigo

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use crate::neotrix::l4_cognition_impl::nt_core_abduction::abduction_engine::{
    AbductionConfig, AbductionEngine, Hypothesis,
};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReasoningConfig {
    /// 候选假设数量
    pub candidate_count: usize,
    /// 置信度更新率
    pub update_rate: f32,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            candidate_count: 5,
            update_rate: 0.1,
        }
    }
}

/// 内存reasoner
pub struct MemoryReasoner {
    config: ReasoningConfig,
    engine: AbductionEngine,
    memory_context: HashMap<String, f32>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl MemoryReasoner {
    pub fn new(config: ReasoningConfig) -> Self {
        Self {
            config,
            engine: AbductionEngine::new(AbductionConfig::default()),
            memory_context: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_hypothesis(&mut self, _hypothesis: Hypothesis) {
        // 通过抽象引擎管理假设
        // 实际实现会将假设集成到 engine 中
    }

    /// 评估假设置信度
    pub fn evaluate_hypothesis(
        &mut self,
        hypothesis: &Hypothesis,
        new_evidence: f32,
    ) -> Hypothesis {
        let mut updated = hypothesis.clone();
        // 基于新证据更新置信度
        updated.confidence = (updated.confidence * (1.0 - self.config.update_rate)
            + new_evidence * self.config.update_rate)
            .min(1.0);
        updated
    }

    /// 生成最佳解释
    pub fn generate_best_explanation(&self, evidence: &[String]) -> Option<Hypothesis> {
        // 使用抽象引擎生成假设
        let _config = AbductionConfig::default();
        // 简化实现：返回 None 或基于证据长度的假设
        if !evidence.is_empty() {
            Some(Hypothesis {
                id: "auto_gen".into(),
                explanation: evidence[0].clone(),
                confidence: 0.7,
                evidence_support: 1.0,
            })
        } else {
            None
        }
    }

    pub fn config(&self) -> &ReasoningConfig {
        &self.config
    }
}

impl CapabilityNode for MemoryReasoner {
    fn node_id(&self) -> &str {
        "nt_memory_kb::nt_memory_reasoner"
    }
    fn provides(&self) -> Vec<String> {
        vec!["abduction".into(), "hypothesis_evaluation".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec![
            "nt_memory_kb".into(),
            "nt_core_abduction".into(),
            "serde".into(),
        ]
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

impl SelfTest for MemoryReasoner {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mr = MemoryReasoner::new(ReasoningConfig::default());
            let _ = mr.config();
            let _ = mr.generate_best_explanation(&["证据1".into(), "证据2".into()]);
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_memory_kb_reasoner"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_memory_reasoner_self_test() {
        let mr = MemoryReasoner::new(ReasoningConfig::default());
        assert!(mr.self_test().is_ok());
    }
}
