/// AwakeningEngine — 统一觉醒循环 (Phase 3)
///
/// measure → model → hypothesize → modify → re-measure
/// 将 SelfMeasure + SelfRepresentation + CausalPredictor + SelfModify
/// 整合为单一闭环，替代 background_loop 中分散的 awakening_ticker 逻辑。
pub mod modify;

use crate::core::self_measure::{AwakeningReport, SelfMeasure, SubsystemId, SystemSnapshot};
use crate::core::self_model::{CausalPredictor, InterventionOutcome, SelfRepresentation};
use modify::{Intervention, InterventionResult, SelfModify};

use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

/// 觉醒引擎配置
#[derive(Debug, Clone)]
pub struct AwakeningConfig {
    /// 多少 tick 重新拟合一次自我模型
    pub refit_interval: u32,
    /// 多少 tick 自动应用一次最佳干预
    pub auto_modify_interval: u32,
    /// 自动修改的最小预期 Φ 增益
    pub min_expected_gain: f64,
    /// 快照窗口大小
    pub window_size: usize,
}

impl Default for AwakeningConfig {
    fn default() -> Self {
        Self {
            refit_interval: 10,
            auto_modify_interval: 20,
            min_expected_gain: 0.005,
            window_size: 50,
        }
    }
}

/// 单次 tick 结果
#[derive(Debug)]
pub struct AwakeningTickResult {
    pub tick_count: u32,
    pub phi: f64,
    pub awakening_speed: f64,
    pub refit_done: bool,
    pub hypotheses_generated: usize,
    pub intervention: Option<InterventionResult>,
    pub bottleneck: (SubsystemId, SubsystemId),
    pub bottleneck_synergy: f64,
}

/// 统一觉醒引擎
#[derive(Clone)]
pub struct AwakeningEngine {
    pub measure: SelfMeasure,
    pub model: SelfRepresentation,
    pub predictor: Option<CausalPredictor>,
    pub modifier: SelfModify,
    pub config: AwakeningConfig,
    tick: u32,
    _last_refit_tick: u32,
    last_modify_tick: u32,
    last_report: Option<AwakeningReport>,
}

impl Default for AwakeningEngine {
    fn default() -> Self {
        Self::new(AwakeningConfig::default())
    }
}

impl AwakeningEngine {
    pub fn new(config: AwakeningConfig) -> Self {
        Self {
            measure: SelfMeasure::new(),
            model: SelfRepresentation::default(),
            predictor: None,
            modifier: SelfModify::new(),
            config,
            tick: 0,
            _last_refit_tick: 0,
            last_modify_tick: 0,
            last_report: None,
        }
    }

    /// 获取当前觉醒报告（如有）
    pub fn report(&self) -> Option<&AwakeningReport> {
        self.last_report.as_ref()
    }

    /// 采集快照到 SelfMeasure
    pub fn snapshot_from_brain(&mut self, brain: &SelfIteratingBrain) {
        let cap = &brain.brain.capability;
        let cap_sum = cap.arr.iter().sum::<f64>() / cap.arr.len() as f64;
        let extension_count = cap.extension.len() as f64 / 20.0;
        let memory_count = brain.reasoning_bank.stats().total_memories as f64 / 100.0;
        let iteration = brain.iteration as f64 / 100.0;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let snap = SystemSnapshot {
            timestamp: now,
            mood: [cap_sum, 0.2, 0.1, 0.1, 0.3, 0.5],
            persona: [0.8, 0.85, 0.55, 0.75, 0.25],
            social: [0.33, memory_count.min(1.0), 0.5],
            reflection: [0.5, 0.3],
            conversation: [iteration.min(1.0), extension_count.min(1.0)],
            behavioral: 0.8,
            law: 0.4,
        };
        self.measure.snapshot(snap);
    }

    /// 从 brain 状态生成 SystemSnapshot
    pub fn build_snapshot(&self, brain: &SelfIteratingBrain) -> SystemSnapshot {
        let cap = &brain.brain.capability;
        let cap_sum = cap.arr.iter().sum::<f64>() / cap.arr.len() as f64;
        let extension_count = cap.extension.len() as f64 / 20.0;
        let memory_count = brain.reasoning_bank.stats().total_memories as f64 / 100.0;
        let iteration = brain.iteration as f64 / 100.0;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        SystemSnapshot {
            timestamp: now,
            mood: [cap_sum, 0.2, 0.1, 0.1, 0.3, 0.5],
            persona: [0.8, 0.85, 0.55, 0.75, 0.25],
            social: [0.33, memory_count.min(1.0), 0.5],
            reflection: [0.5, 0.3],
            conversation: [iteration.min(1.0), extension_count.min(1.0)],
            behavioral: 0.8,
            law: 0.4,
        }
    }

    /// 执行一次觉醒 tick（需前置 snapshot_from_brain）
    pub fn tick(&mut self, sib: &mut SelfIteratingBrain) -> AwakeningTickResult {
        self.tick += 1;
        self.snapshot_from_brain(sib);
        let mut refit_done = false;
        let mut hypotheses_generated = 0;
        let mut intervention: Option<InterventionResult> = None;

        if self.tick % self.config.refit_interval == 0 && self.measure.tick_count() >= 10 {
            let traj = &self.measure.trajectory;
            self.model = SelfRepresentation::learn(traj);
            self.predictor = Some(CausalPredictor::new(traj));
            refit_done = true;
        }

        let report = self.measure.generate_report();
        self.last_report = Some(report.clone());

        if self.tick % self.config.auto_modify_interval == 0
            && self.tick > self.last_modify_tick
            && self.measure.tick_count() >= 20
        {
            if let Some(ref predictor) = self.predictor {
                let hyps = predictor.generate_hypotheses(&report);
                hypotheses_generated = hyps.len();
                if let Some(best) = hyps.first() {
                    if best.predicted_delta_phi >= self.config.min_expected_gain
                        && best.confidence > 0.3
                    {
                        let cap_idx = best.target as usize;
                        let delta = best.delta * 0.1;
                        let interventions = vec![Intervention::CapabilityDelta {
                            index: cap_idx,
                            delta,
                        }];
                        let result = self.modifier.apply_batch(&interventions, &mut sib.brain);
                        if result.applied {
                            self.modifier.commit();
                            self.last_modify_tick = self.tick;
                        }
                        intervention = Some(result);
                    }
                }
            }
        }

        let report = self.measure.generate_report();
        AwakeningTickResult {
            tick_count: self.tick,
            phi: report.phi,
            awakening_speed: report.awakening_speed,
            refit_done,
            hypotheses_generated,
            intervention,
            bottleneck: report.bottleneck,
            bottleneck_synergy: report.bottleneck_synergy,
        }
    }

    /// 获取当前最优假设列表
    pub fn suggestions(&self) -> Vec<InterventionOutcome> {
        self.predictor
            .as_ref()
            .and_then(|p| self.last_report.as_ref().map(|r| p.generate_hypotheses(r)))
            .unwrap_or_default()
    }

    /// 觉醒速度（最近 ΔΦ/步）
    pub fn awakening_speed(&self) -> f64 {
        self.measure.awakening_speed()
    }

    /// 返回最近报告文本
    pub fn report_string(&self) -> String {
        match self.last_report {
            Some(ref r) => format!("{}", r),
            None => "(等待数据)".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let engine = AwakeningEngine::new(AwakeningConfig::default());
        assert_eq!(engine.tick, 0);
    }

    #[test]
    fn test_tick_no_crash() {
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let mut engine = AwakeningEngine::new(AwakeningConfig::default());
        let result = engine.tick(&mut brain);
        assert!(result.tick_count > 0);
    }

    #[test]
    fn test_suggestions_after_enough_ticks() {
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let mut engine = AwakeningEngine::new(AwakeningConfig {
            refit_interval: 5,
            auto_modify_interval: 10,
            window_size: 30,
            ..Default::default()
        });
        for _ in 0..35 {
            engine.tick(&mut brain);
        }
        // After 35 ticks with refit_interval=5, we should have had refits
        let _suggs = engine.suggestions();
        // suggestions may be empty if trajectory is too uniform, but should not crash
    }

    #[test]
    fn test_snapshot_from_brain() {
        let brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let engine = AwakeningEngine::new(AwakeningConfig::default());
        let snap = engine.build_snapshot(&brain);
        assert_eq!(snap.mood.len(), 6);
        assert_eq!(snap.persona.len(), 5);
    }
}
