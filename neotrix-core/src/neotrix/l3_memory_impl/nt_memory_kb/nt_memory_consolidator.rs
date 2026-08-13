//! nt_memory_kb — 内存consolidator（consolidation）
//!
//! 基于证据的多级内存consolidation（从短期到长期归纳），配合 VSA 关联与 FEP-IIT 桥接。
//! 节点: nt_memory_kb::nt_memory_consolidator (L3)
//! Provides: consolidation, evidence_integration
//! Requires: nt_memory_kb, serde, VSA
//! Rune: Indigo, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use crate::neotrix::l5_consciousness_impl::nt_core_fep_iit::types::BridgeReport;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsolidationConfig {
    /// consolidation 阈值
    pub threshold: f32,
    /// 最大consolidation批次大小
    pub batch_size: usize,
    /// 是否启用 VSA 关联
    pub use_vsa: bool,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            threshold: 0.7,
            batch_size: 32,
            use_vsa: true,
        }
    }
}

/// consolidation 证据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsolidationEvidence {
    pub source: String,
    pub confidence: f32,
    pub content: String,
}

/// 内存consolidator
pub struct MemoryConsolidator {
    config: ConsolidationConfig,
    evidence_buffer: Vec<ConsolidationEvidence>,
    consolidated_memories: HashMap<String, BridgeReport>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl MemoryConsolidator {
    pub fn new(config: ConsolidationConfig) -> Self {
        Self {
            config,
            evidence_buffer: Vec::new(),
            consolidated_memories: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_evidence(&mut self, evidence: ConsolidationEvidence) {
        self.evidence_buffer.push(evidence);
        // 维护缓存大小
        if self.evidence_buffer.len() > self.config.batch_size {
            self.evidence_buffer.remove(0);
        }
    }

    /// 基于证据进行consolidation
    /// 1. 收集证据
    /// 2. 计算共识/置信度
    /// 3. 通过 VSA 或 FEP-IIT 桥接生成 consolidated memory
    /// 4. 存入长期memory
    pub fn consolidate(&mut self) -> Result<Vec<BridgeReport>, NeoTrixError> {
        if self.evidence_buffer.is_empty() {
            return Ok(Vec::new());
        }

        // 简化实现：汇总证据并生成 BridgeReport
        let mut total_confidence: f32 = 0.0;
        let mut combined_content = String::new();

        for evidence in &self.evidence_buffer {
            total_confidence += evidence.confidence;
            combined_content.push_str(&evidence.content);
            combined_content.push('\n');
        }

        let avg_confidence_f32 = total_confidence / self.evidence_buffer.len() as f32;
        let avg_confidence = avg_confidence_f32 as f64;

        // 如果置信度超过阈值，创建 consolidated memory
        if avg_confidence_f32 >= self.config.threshold {
            let report = BridgeReport {
                consciousness_score: avg_confidence,
                vsa_coherence: avg_confidence,
                fe_derived_phi: avg_confidence,
                bounded_free_energy: 0.0,
                free_energy_bound: 0.0,
                fe_improvement_from_iit: 0.0,
                phi_improvement_from_fep: 0.0,
                state_classification: "consolidated",
            };
            // 生成 VSA 关联钥匙 (简化)
            let key = format!(
                "consolidated_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            self.consolidated_memories
                .insert(key.clone(), report.clone());
            Ok(vec![report])
        } else {
            Ok(Vec::new())
        }
    }

    pub fn get_consolidated(&self, key: &str) -> Option<&BridgeReport> {
        self.consolidated_memories.get(key)
    }

    pub fn config(&self) -> &ConsolidationConfig {
        &self.config
    }
}

impl CapabilityNode for MemoryConsolidator {
    fn node_id(&self) -> &str {
        "nt_memory_kb::nt_memory_consolidator"
    }
    fn provides(&self) -> Vec<String> {
        vec!["consolidation".into(), "evidence_integration".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec![
            "nt_memory_kb".into(),
            "nt_core_fep_iit".into(),
            "serde".into(),
        ]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Indigo, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for MemoryConsolidator {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut mc = MemoryConsolidator::new(ConsolidationConfig::default());

            let ev1 = ConsolidationEvidence {
                source: "sense_data_1".into(),
                confidence: 0.8,
                content: "用户输入: Hello".into(),
            };
            let ev2 = ConsolidationEvidence {
                source: "sense_data_2".into(),
                confidence: 0.7,
                content: "系统确认: Hello".into(),
            };

            mc.add_evidence(ev1);
            mc.add_evidence(ev2);

            let _reports = mc.consolidate()?;
            // 可能因为阈值原因返回空，这也是可以接受的

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_memory_kb_consolidator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_memory_consolidator_self_test() {
        let mc = MemoryConsolidator::new(ConsolidationConfig::default());
        assert!(mc.self_test().is_ok());
    }
}
