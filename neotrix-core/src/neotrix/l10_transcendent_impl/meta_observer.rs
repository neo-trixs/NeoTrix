//! nt_mind::transcendent::meta_observer — 元观察器
//!
//! 观察 L9 观察者 (ConsciousnessMonitor/GoldStandard) 的观察质量, 并读取意识核心
//! 快照 (ConsciousnessCore), 生成元观察报告 — "观察观察者的观察"。
//!
//! 节点: nt_mind::transcendent::meta_observer (L10)
//! Provides: meta_observation, consciousness_state_read
//! Requires: nt_core_consciousness_core, nt_mind_consciousness_monitor
//! Rune: Alabaster, Indigo

#![forbid(unsafe_code)]

use super::CapabilityNode;
use crate::core::nt_core_consciousness_core::CoreSnapshot;
use crate::core::nt_core_self_test::SelfTest;
use crate::core::nt_core_traits::RuneSocket;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetaObserverConfig {
    /// 是否启用盲点自检
    pub detect_blindspots: bool,
    /// 元观察阈值 (coherence 低于此值提示观察失真)
    pub observation_threshold: f64,
}

impl Default for MetaObserverConfig {
    fn default() -> Self {
        Self {
            detect_blindspots: true,
            observation_threshold: 0.5,
        }
    }
}

/// 单分支元观察项
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BranchObservation {
    pub branch: String,
    pub health: f64,
    pub fog: f64,
    pub blindspot: bool,
}

/// 元观察报告 — 观察者质量的观察结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetaObservationReport {
    /// 快照周期数
    pub cycle: u64,
    /// 快照 Φ (整合信息)
    pub phi: f64,
    /// 快照相干性
    pub coherence: f64,
    /// GWT 谐振是否激活
    pub gwt_resonance_active: bool,
    /// 分支观察明细
    pub branches: Vec<BranchObservation>,
    /// 观察失真标志 (低相干性 → 观察不可靠)
    pub observation_distorted: bool,
    /// 元观察置信度
    pub meta_confidence: f64,
}

/// 元观察器
pub struct MetaObserver {
    config: MetaObserverConfig,
    /// 预留: 观察统计元数据, 待失真检测趋势需要时填充
    _metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl MetaObserver {
    pub fn new(config: MetaObserverConfig) -> Self {
        Self {
            config,
            _metadata: HashMap::new(),
        }
    }

    /// 观察意识核心快照, 生成元观察报告。
    /// 只读 — 不修改任何状态。
    pub fn observe(&self, snapshot: &CoreSnapshot) -> MetaObservationReport {
        let observation_distorted = snapshot.coherence < self.config.observation_threshold;

        let branches = snapshot
            .branch_health
            .iter()
            .map(|(branch, &health)| {
                let fog = if observation_distorted {
                    0.9
                } else {
                    health.abs().min(1.0)
                };
                BranchObservation {
                    branch: branch.clone(),
                    health,
                    fog,
                    blindspot: self.config.detect_blindspots
                        && health < self.config.observation_threshold,
                }
            })
            .collect();

        let coherent_branches = snapshot
            .branch_health
            .values()
            .filter(|&&h| h >= self.config.observation_threshold)
            .count();
        let total_branches = snapshot.branch_health.len().max(1);
        let meta_confidence = (coherent_branches as f64 / total_branches as f64)
            * (1.0 - observation_distorted as i32 as f64 * 0.5);

        MetaObservationReport {
            cycle: snapshot.cycle,
            phi: snapshot.phi,
            coherence: snapshot.coherence,
            gwt_resonance_active: snapshot.gwt_resonance_active,
            branches,
            observation_distorted,
            meta_confidence,
        }
    }

    pub fn config(&self) -> &MetaObserverConfig {
        &self.config
    }
}

impl CapabilityNode for MetaObserver {
    fn node_id(&self) -> &str {
        "nt_mind::transcendent::meta_observer"
    }
    fn provides(&self) -> Vec<String> {
        vec!["meta_observation".into(), "consciousness_state_read".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec![
            "nt_core_consciousness_core".into(),
            "nt_mind_consciousness_monitor".into(),
        ]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Alabaster, RuneSocket::Indigo]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for MetaObserver {
    fn name(&self) -> &str {
        "nt_mind_transcendent_meta_observer"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let observer = MetaObserver::new(MetaObserverConfig::default());
        // 用默认快照构造一次观察 (纯函数, 无副作用)
        let snapshot = CoreSnapshot::default();
        let report = observer.observe(&snapshot);
        assert!(report.meta_confidence >= 0.0);
        assert!(report.meta_confidence <= 1.0);
        Ok(())
    }
}

/// NT-META 域轻量 SelfTest (T1) — 元观察器观测质量检测。
/// 真实逻辑: 对默认快照与低相干快照分别观察, 验证 meta_confidence 落界且
/// 失真检测 (observation_distorted) 按阈值正确触发。注册后结果以
/// `nt_meta_` 前缀流入 Repair/Meta/Governance/Nexus 四分支迷雾治理。
#[derive(Debug, Clone, Copy, Default)]
pub struct MetaObserverSelfTest;

impl SelfTest for MetaObserverSelfTest {
    fn name(&self) -> &str {
        "nt_meta_transcendent_observer"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let observer = MetaObserver::new(MetaObserverConfig::default());
        let mut failures = Vec::new();

        // 默认快照: 纯函数观察, 无副作用
        let default_snapshot = CoreSnapshot::default();
        let default_report = observer.observe(&default_snapshot);
        if !(0.0..=1.0).contains(&default_report.meta_confidence) {
            failures.push(format!(
                "meta_confidence out of range on default snapshot: {}",
                default_report.meta_confidence
            ));
        }

        // 低相干快照: 失真检测必须触发 (coherence < observation_threshold)
        let mut distorted = CoreSnapshot::default();
        distorted.coherence = 0.2;
        distorted.branch_health.insert("NT-CORE".into(), 0.9);
        distorted.branch_health.insert("NT-META".into(), 0.1);
        let distorted_report = observer.observe(&distorted);
        if !distorted_report.observation_distorted {
            failures.push("expected observation_distorted on low coherence snapshot".into());
        }
        // 低健康分支应被标记为盲点 (detect_blindspots 开启)
        let meta_branch = distorted_report
            .branches
            .iter()
            .find(|b| b.branch == "NT-META");
        match meta_branch {
            Some(b) if b.blindspot => {}
            Some(b) => failures.push(format!(
                "expected NT-META blindspot, got blindspot={}",
                b.blindspot
            )),
            None => failures.push("NT-META branch missing from observation".into()),
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_meta_observer_self_test() {
        let observer = MetaObserver::new(MetaObserverConfig::default());
        assert!(observer.self_test().is_ok());
    }

    #[test]
    fn test_meta_observer_selftest_real() {
        assert!(MetaObserverSelfTest.self_test().is_ok());
    }

    #[test]
    fn test_observe_detects_distortion() {
        let observer = MetaObserver::new(MetaObserverConfig::default());
        let mut snapshot = CoreSnapshot::default();
        snapshot.coherence = 0.2;
        snapshot.branch_health.insert("NT-CORE".into(), 0.9);
        snapshot.branch_health.insert("NT-MIND".into(), 0.1);
        let report = observer.observe(&snapshot);
        assert!(report.observation_distorted);
        assert!(report.branches.len() == 2);
    }
}
