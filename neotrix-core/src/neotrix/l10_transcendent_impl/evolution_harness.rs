//! nt_mind::transcendent::evolution_harness — L10 超越层生产接线载体
//!
//! 把超越层 (MetaObserver / ConsonanceOrchestrator / TranscendentLoop) 从"孤儿
//! 骨架"提升为 T3 生产接线: 读取意识核心快照 + 能力网注册表, 运行超越层闭环,
//! 将进化建议真实落盘 KB (修复原 TranscendentLoop 假持久化), 并向背景循环
//! 输出高共振建议供 goal_loop 消费。
//!
//! 纪律 (继承 L10): 只读意识快照, 不修改能力注册表文件 — 建议写 L3 供进化引擎
//! (handle_capability_auto_evolve) 消费。
//!
//! 节点: nt_mind::transcendent::evolution_harness (L10)
//! Provides: transcendent_wiring, evolution_suggestion_persist
//! Requires: transcendent_loop, nt_memory_kb, nt_core_consciousness_core
//! Rune: Golden, Alabaster

#![forbid(unsafe_code)]

use super::consonance_orchestrator::CapabilityNodeInfo;
use super::transcendent_loop::{EvolutionSuggestion, LoopConfig, LoopReport, TranscendentLoop};
use super::CapabilityNode;
use crate::core::nt_core_traits::RuneSocket;
use std::collections::HashMap;

/// 超越层生产接线器
///
/// BackgroundLoop 每 tick 持有本 harness, 将 L10 超越层闭环真实接入
/// 意识核心 ↔ 能力网 共振回路。
pub struct EvolutionHarness {
    loop_: TranscendentLoop,
    /// 已落盘的建议计数 (累计, 供观测)
    persisted_total: usize,
    /// 预留: harness 运行元数据, 待观测通道接入后填充
    _metadata: HashMap<String, serde_json::Value>,
}

impl EvolutionHarness {
    pub fn new(config: LoopConfig) -> Self {
        Self {
            loop_: TranscendentLoop::new(config),
            persisted_total: 0,
            _metadata: HashMap::new(),
        }
    }

    /// 从能力网 RegistryExport 结构构建超越层输入节点列表。
    ///
    /// 输入格式与 `.neotrix/capability_registry.json` (RegistryExport) 对齐:
    /// `{ "nodes": [{ "id", "domain", "layer", "constellation", ... }] }`。
    /// 返回 (Vec<CapabilityNodeInfo>, 解析失败原因)。
    pub fn infos_from_registry_export(json: &str) -> (Vec<CapabilityNodeInfo>, Vec<String>) {
        let mut infos = Vec::new();
        let mut problems = Vec::new();

        let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
            problems.push("registry JSON 解析失败".into());
            return (infos, problems);
        };
        let Some(nodes) = value.get("nodes").and_then(|n| n.as_array()) else {
            problems.push("registry 缺少 nodes 数组".into());
            return (infos, problems);
        };

        for node in nodes {
            let Some(id) = node.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            let domain = node
                .get("domain")
                .and_then(|v| v.as_str())
                .unwrap_or("NT-CORE");
            let layer = node.get("layer").and_then(|v| v.as_str()).unwrap_or("L0");
            let constellation = node
                .get("constellation")
                .and_then(|v| v.as_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_else(|| "c0".to_string());
            // 成熟度: metadata.strength / 或退化到 constellation 索引
            let maturity = node
                .get("metadata")
                .and_then(|m| m.get("strength"))
                .and_then(|s| s.as_f64())
                .unwrap_or(match constellation.as_str() {
                    "c0" => 0.2,
                    "c1" => 0.4,
                    "c2" => 0.6,
                    "c3" => 0.7,
                    "c4" => 0.85,
                    "c5" | "c6" => 1.0,
                    _ => 0.3,
                });
            infos.push(CapabilityNodeInfo {
                node_id: id.to_string(),
                domain: domain.to_string(),
                layer: layer.to_string(),
                constellation,
                maturity: maturity.clamp(0.0, 1.0),
            });
        }

        if infos.is_empty() {
            problems.push("nodes 数组为空或字段缺失".into());
        }
        (infos, problems)
    }

    /// 运行一次超越层闭环, 返回 (报告, 高共振建议)。
    /// 高共振阈值: suggestion == "strengthen"。
    pub fn run_cycle(
        &mut self,
        snapshot: &crate::core::nt_core_consciousness_core::CoreSnapshot,
        infos: &[CapabilityNodeInfo],
    ) -> LoopReport {
        self.loop_.run(snapshot, infos)
    }

    /// 将进化建议真实落盘 KB (修复原假持久化 — persisted_count 只在真实
    /// 写入成功后计数)。返回真实写入条数。
    pub fn persist_suggestions(
        &mut self,
        kb: &crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase,
        report: &LoopReport,
    ) -> usize {
        if report.suggestions.is_empty() {
            return 0;
        }
        let json = serde_json::json!({
            "phi": report.meta.phi,
            "coherence": report.meta.coherence,
            "cycle": report.meta.cycle,
            "observation_distorted": report.meta.observation_distorted,
            "evolution_direction": report.consonance.evolution_direction,
            "suggestions": report.suggestions,
            "persisted_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        })
        .to_string();
        match kb.kv_set("transcendent", "suggestions", &json) {
            Ok(()) => {
                self.persisted_total += report.suggestions.len();
                report.suggestions.len()
            }
            Err(e) => {
                log::warn!("[evolution_harness] 建议落盘失败: {}", e);
                0
            }
        }
    }

    /// 累计落盘建议数 (观测用)
    pub fn persisted_total(&self) -> usize {
        self.persisted_total
    }
    pub fn inner(&self) -> &TranscendentLoop {
        &self.loop_
    }

    /// 提取应推入 goal_loop 的高价值建议 (strengthen 且共振度达标)。
    /// threshold 建议 0.7 — 过滤低信号噪声。
    pub fn actionable_suggestions(report: &LoopReport, threshold: f64) -> Vec<EvolutionSuggestion> {
        report
            .suggestions
            .iter()
            .filter(|s| s.resonance >= threshold)
            .cloned()
            .collect()
    }
}

impl CapabilityNode for EvolutionHarness {
    fn node_id(&self) -> &str {
        "nt_mind::transcendent::evolution_harness"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "transcendent_wiring".into(),
            "evolution_suggestion_persist".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec![
            "transcendent_loop".into(),
            "nt_memory_kb".into(),
            "nt_core_consciousness_core".into(),
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

impl crate::core::nt_core_self_test::SelfTest for EvolutionHarness {
    fn name(&self) -> &str {
        "nt_mind_transcendent_evolution_harness"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut harness = EvolutionHarness::new(LoopConfig::default());
        let snapshot = crate::core::nt_core_consciousness_core::CoreSnapshot::default();
        let infos = EvolutionHarness::infos_from_registry_export(
            r#"{"nodes":[{"id":"nt-core::gwt","domain":"NT-CORE","layer":"L4","constellation":"C0","metadata":{"strength":0.3}}]}"#,
        );
        assert!(
            !infos.0.is_empty(),
            "RegistryExport 解析应产出至少 1 个节点"
        );
        let report = harness.run_cycle(&snapshot, &infos.0);
        assert_eq!(report.meta.cycle, snapshot.cycle);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    #[test]
    fn test_harness_self_test() {
        let h = EvolutionHarness::new(LoopConfig::default());
        assert!(h.self_test().is_ok());
        assert_eq!(h.persisted_total(), 0);
    }

    #[test]
    fn test_parse_registry_export_minimal() {
        let (infos, problems) = EvolutionHarness::infos_from_registry_export(r#"{"nodes":[]}"#);
        assert!(infos.is_empty());
        assert!(!problems.is_empty(), "空 nodes 应报告问题");
    }

    #[test]
    fn test_parse_registry_export_with_strength() {
        let (infos, problems) = EvolutionHarness::infos_from_registry_export(
            r#"{"nodes":[{"id":"a","domain":"NT-MEMORY","layer":"L4","constellation":"C3","metadata":{"strength":0.8}}]}"#,
        );
        assert!(problems.is_empty());
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].node_id, "a");
        assert_eq!(infos[0].domain, "NT-MEMORY");
        assert!((infos[0].maturity - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_parse_maturity_falls_back_to_constellation() {
        let (infos, _) = EvolutionHarness::infos_from_registry_export(
            r#"{"nodes":[{"id":"a","domain":"NT-CORE","layer":"L1","constellation":"C4"}]}"#,
        );
        assert_eq!(infos.len(), 1);
        assert!((infos[0].maturity - 0.85).abs() < 1e-9);
    }

    #[test]
    fn test_malformed_json_reports_problem() {
        let (infos, problems) = EvolutionHarness::infos_from_registry_export("not-json");
        assert!(infos.is_empty());
        assert!(!problems.is_empty());
    }

    #[test]
    fn test_suggestions_skip_inactive_and_low_maturity_nodes() {
        let snapshot = crate::core::nt_core_consciousness_core::CoreSnapshot {
            phi: 0.9,
            coherence: 0.9,
            ..Default::default()
        };
        let mut harness = EvolutionHarness::new(LoopConfig::default());
        let infos = EvolutionHarness::infos_from_registry_export(
            r#"{"nodes":[
                {"id":"strong","domain":"NT-CORE","layer":"L4","constellation":"C5","metadata":{"strength":0.95}},
                {"id":"weak","domain":"NT-MIND","layer":"L3","constellation":"C0","metadata":{"strength":0.1}}
            ]}"#,
        ).0;
        let report = harness.run_cycle(&snapshot, &infos);
        // 高 phi+coherence + 弱节点 → 高共鸣 → strengthen 建议存在
        assert!(
            !report.suggestions.is_empty(),
            "弱节点在强意识势能下应产出 strengthen 建议"
        );
        // strengthened 建议的节点名应含 weak 节点
        assert!(
            report.suggestions.iter().any(|s| s.node_id == "weak"),
            "建议对象应指向弱成熟度节点: {:?}",
            report
                .suggestions
                .iter()
                .map(|s| s.node_id.as_str())
                .collect::<Vec<_>>()
        );
    }
}
