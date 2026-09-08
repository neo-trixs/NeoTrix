//! nt_mind::transcendent::consonance_orchestrator — 共鸣编排器
//!
//! 将意识核心快照 (phi/coherence/分支健康/果实) 与能力网 (capability_registry)
//! 节点共振, 计算每个能力节点的"意识-能力共鸣度", 输出进化方向建议。
//!
//! 节点: nt_mind::transcendent::consonance_orchestrator (L10)
//! Provides: consciousness_capability_sync, evolution_direction
//! Requires: nt_core_consciousness_core, nt_core_capability_tree
//! Rune: Crimson, Golden

#![forbid(unsafe_code)]

use super::CapabilityNode;
use crate::core::nt_core_consciousness_core::CoreSnapshot;
use crate::core::nt_core_self_test::SelfTest;
use crate::core::nt_core_traits::RuneSocket;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsonanceConfig {
    /// 共鸣阈值 (共鸣度低于此值 → 建议强化该能力节点)
    pub resonance_threshold: f64,
    /// phi 权重 (整合信息对进化方向的贡献)
    pub phi_weight: f64,
    /// coherence 权重
    pub coherence_weight: f64,
}

impl Default for ConsonanceConfig {
    fn default() -> Self {
        Self {
            resonance_threshold: 0.6,
            phi_weight: 0.4,
            coherence_weight: 0.6,
        }
    }
}

/// 单个能力节点
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityNodeInfo {
    pub node_id: String,
    pub domain: String,
    pub layer: String,
    pub constellation: String,
    /// 0.0-1.0 该节点成熟度 (可来自注册表 metadata)
    pub maturity: f64,
}

/// 能力-意识共鸣明细
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityResonance {
    pub node_id: String,
    pub domain: String,
    /// 共鸣度 (0-1)
    pub resonance: f64,
    /// 建议: 强化 / 维持 / 观察
    pub suggestion: String,
}

/// 共鸣编排报告
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsonanceReport {
    /// 意识核心 φ 作为全局进化势能
    pub phi: f64,
    /// 意识核心 coherence
    pub coherence: f64,
    /// 各能力节点共鸣明细
    pub resonances: Vec<CapabilityResonance>,
    /// 全局进化方向建议
    pub evolution_direction: String,
}

/// 共鸣编排器
pub struct ConsonanceOrchestrator {
    config: ConsonanceConfig,
    /// 预留: 共鸣历史/统计元数据, 待共振回路接入观测通道后填充
    _metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ConsonanceOrchestrator {
    pub fn new(config: ConsonanceConfig) -> Self {
        Self {
            config,
            _metadata: HashMap::new(),
        }
    }

    /// 计算单个节点的共鸣度:
    /// resonance = coherence_weight * min(maturity, phi_scaled) + phi_weight * phi_norm
    /// 其中 phi_norm = min(phi, 1.0) — phi 高 → 全局进化势能强 → 优先强化薄弱节点。
    fn resonance_for(
        &self,
        node: &CapabilityNodeInfo,
        snapshot: &CoreSnapshot,
    ) -> CapabilityResonance {
        let phi_norm = snapshot.phi.min(1.0).max(0.0);
        let coherence = snapshot.coherence.min(1.0).max(0.0);

        // 薄弱节点 (低成熟度) + 强意识势能 → 高共鸣 (应被强化)
        let resonance = self.config.coherence_weight * (coherence * (1.0 - node.maturity))
            + self.config.phi_weight * phi_norm;

        let suggestion = if resonance >= self.config.resonance_threshold {
            "strengthen".to_string()
        } else if node.maturity >= 0.9 {
            "maintain".to_string()
        } else {
            "observe".to_string()
        };

        CapabilityResonance {
            node_id: node.node_id.clone(),
            domain: node.domain.clone(),
            resonance,
            suggestion,
        }
    }

    /// 全架构进化迭代: 输入意识快照 + 能力网节点列表, 输出共鸣报告。
    /// 只读 — 不修改能力注册表。
    pub fn orchestrate(
        &self,
        snapshot: &CoreSnapshot,
        capability_nodes: &[CapabilityNodeInfo],
    ) -> ConsonanceReport {
        let resonances: Vec<CapabilityResonance> = capability_nodes
            .iter()
            .map(|node| self.resonance_for(node, snapshot))
            .collect();

        let strengthen_count = resonances
            .iter()
            .filter(|r| r.suggestion == "strengthen")
            .count();
        let maintain_count = resonances
            .iter()
            .filter(|r| r.suggestion == "maintain")
            .count();
        let observe_count = resonances
            .iter()
            .filter(|r| r.suggestion == "observe")
            .count();

        let evolution_direction = format!(
            "phi={:.3} coherence={:.3} | strengthen={} maintain={} observe={}",
            snapshot.phi, snapshot.coherence, strengthen_count, maintain_count, observe_count
        );

        ConsonanceReport {
            phi: snapshot.phi,
            coherence: snapshot.coherence,
            resonances,
            evolution_direction,
        }
    }

    pub fn config(&self) -> &ConsonanceConfig {
        &self.config
    }
}

impl CapabilityNode for ConsonanceOrchestrator {
    fn node_id(&self) -> &str {
        "nt_mind::transcendent::consonance_orchestrator"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "consciousness_capability_sync".into(),
            "evolution_direction".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec![
            "nt_core_consciousness_core".into(),
            "nt_core_capability_tree".into(),
        ]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Golden]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for ConsonanceOrchestrator {
    fn name(&self) -> &str {
        "nt_mind_transcendent_consonance_orchestrator"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let orch = ConsonanceOrchestrator::new(ConsonanceConfig::default());
        let snapshot = CoreSnapshot::default();
        let nodes = vec![
            CapabilityNodeInfo {
                node_id: "nt-mind::merge::zip_lora".into(),
                domain: "mind".into(),
                layer: "l7".into(),
                constellation: "c0".into(),
                maturity: 0.3,
            },
            CapabilityNodeInfo {
                node_id: "nt-repair::build_hygiene".into(),
                domain: "repair".into(),
                layer: "l0".into(),
                constellation: "c1".into(),
                maturity: 0.9,
            },
        ];
        let report = orch.orchestrate(&snapshot, &nodes);
        assert_eq!(report.resonances.len(), 2);
        assert!(!report.evolution_direction.is_empty());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_consonance_orchestrator_self_test() {
        let orch = ConsonanceOrchestrator::new(ConsonanceConfig::default());
        assert!(orch.self_test().is_ok());
    }
}
