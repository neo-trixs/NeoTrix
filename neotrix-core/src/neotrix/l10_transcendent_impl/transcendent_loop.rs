//! nt_mind::transcendent::transcendent_loop — 超越层进化闭环
//!
//! 融合-接线模式的核心: 读取 (意识快照) → 共振 (能力网) → 建议 (进化方向) →
//! 写 L3 (进化建议记录)。驱动"意识核心与意识能力网全架构进化迭代"。
//!
//! 节点: nt_mind::transcendent::transcendent_loop (L10)
//! Provides: transcendent_evolution_loop, self_observation
//! Requires: meta_observer, consonance_orchestrator, nt_memory_kb
//! Rune: Golden, Alabaster

#![forbid(unsafe_code)]

use super::consonance_orchestrator::{
    CapabilityNodeInfo, ConsonanceConfig, ConsonanceOrchestrator, ConsonanceReport,
};
use super::meta_observer::{MetaObservationReport, MetaObserver, MetaObserverConfig};
use super::CapabilityNode;
use crate::core::nt_core_consciousness_core::CoreSnapshot;
use crate::core::nt_core_self_test::SelfTest;
use crate::neotrix::RuneSocket;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopConfig {
    pub meta_config: MetaObserverConfig,
    pub consonance_config: ConsonanceConfig,
    /// 是否将进化建议写入 L3 (默认 true)
    pub persist_suggestions: bool,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            meta_config: MetaObserverConfig::default(),
            consonance_config: ConsonanceConfig::default(),
            persist_suggestions: true,
        }
    }
}

/// 进化建议 — 即将写入 L3 的记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvolutionSuggestion {
    pub node_id: String,
    pub domain: String,
    pub suggestion: String,
    pub resonance: f64,
    pub cycle: u64,
}

/// 超越层循环报告
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopReport {
    pub meta: MetaObservationReport,
    pub consonance: ConsonanceReport,
    pub suggestions: Vec<EvolutionSuggestion>,
    /// 写入 L3 的记录数
    pub persisted_count: usize,
}

/// 超越层进化闭环
pub struct TranscendentLoop {
    config: LoopConfig,
    observer: MetaObserver,
    orchestrator: ConsonanceOrchestrator,
    /// 本次运行的进化建议 (模拟写 L3; 真实接线经 nt_memory_kb)
    suggestions: Vec<EvolutionSuggestion>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl TranscendentLoop {
    pub fn new(config: LoopConfig) -> Self {
        let observer = MetaObserver::new(config.meta_config.clone());
        let orchestrator = ConsonanceOrchestrator::new(config.consonance_config.clone());
        Self {
            config,
            observer,
            orchestrator,
            suggestions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// 运行一次超越层闭环迭代。
    /// 1. 元观察 (只读意识快照)
    /// 2. 共鸣编排 (意识 ↔ 能力网)
    /// 3. 生成进化建议 (persist → L3)
    pub fn run(
        &mut self,
        snapshot: &CoreSnapshot,
        capability_nodes: &[CapabilityNodeInfo],
    ) -> LoopReport {
        let meta = self.observer.observe(snapshot);
        let consonance = self.orchestrator.orchestrate(snapshot, capability_nodes);

        let suggestions: Vec<EvolutionSuggestion> = consonance
            .resonances
            .iter()
            .filter(|r| r.suggestion == "strengthen")
            .map(|r| EvolutionSuggestion {
                node_id: r.node_id.clone(),
                domain: r.domain.clone(),
                suggestion: r.suggestion.clone(),
                resonance: r.resonance,
                cycle: snapshot.cycle,
            })
            .collect();

        let persisted_count = if self.config.persist_suggestions {
            suggestions.len()
        } else {
            0
        };
        self.suggestions = suggestions.clone();

        LoopReport {
            meta,
            consonance,
            suggestions,
            persisted_count,
        }
    }

    pub fn suggestions(&self) -> &[EvolutionSuggestion] {
        &self.suggestions
    }
    pub fn config(&self) -> &LoopConfig {
        &self.config
    }
}

impl CapabilityNode for TranscendentLoop {
    fn node_id(&self) -> &str {
        "nt_mind::transcendent::transcendent_loop"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "transcendent_evolution_loop".into(),
            "self_observation".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec![
            "meta_observer".into(),
            "consonance_orchestrator".into(),
            "nt_memory_kb".into(),
        ]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Golden, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for TranscendentLoop {
    fn name(&self) -> &str {
        "nt_mind_transcendent_loop"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut loop_ = TranscendentLoop::new(LoopConfig::default());
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
        let report = loop_.run(&snapshot, &nodes);
        assert_eq!(report.suggestions.len(), report.persisted_count);
        assert_eq!(report.meta.cycle, snapshot.cycle);
        assert_eq!(report.consonance.resonances.len(), 2);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_transcendent_loop_self_test() {
        let loop_ = TranscendentLoop::new(LoopConfig::default());
        assert!(loop_.self_test().is_ok());
    }
}
