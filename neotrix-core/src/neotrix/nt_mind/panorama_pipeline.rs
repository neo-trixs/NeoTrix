use crate::core::nt_core_hcube::cube::KnowledgeHyperCube;
use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use crate::core::nt_core_gwt::module_def::{SpecialistModule, SpecialistType};
use crate::core::nt_core_bank::ReasoningMemory;
use crate::core::edit::MicroEdit;
use crate::neotrix::nt_world_model::{WorldModelV2, TaskType};
use crate::neotrix::nt_world_infer::FreeEnergyReport;
use crate::neotrix::iit_phi::PhiReport;
use super::cortex_memory::{CortexMemory, MemoryTrace, DimensionTag, Modality};
use super::consciousness_bridge::ConsciousnessBridge;
use super::self_iterating::SelfIteratingBrain;
use super::goal_loop::GoalLoop;
use super::goal_loop::GoalConfig;
use super::predictive_cortex::{PredictiveCortex, HorizonForecast};

/// 前瞻预测标签
#[derive(Debug, Clone)]
pub struct LookaheadTag {
    pub horizon_anomaly_prob: f64,
    pub horizon_best_fe: f64,
    pub horizon_divergence: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct PanoramaReport {
    pub cycle: u64,
    pub hypercube_entries: usize,
    pub cortex_traces: usize,
    pub gwt_broadcasts: usize,
    pub bank_memories: usize,
    pub goals_created: usize,
    pub anomaly: bool,
    pub fe_energy: f64,
    pub phi: f64,
    pub lookahead: Option<LookaheadTag>,
    pub forecast_quality: f64,
    pub repairs_triggered: u64,
    pub repair_triggered_this_cycle: bool,
}

pub struct PanoramaPipeline {
    pub hypercube: KnowledgeHyperCube,
    pub cortex: CortexMemory,
    pub consciousness: ConsciousnessBridge,
    pub gwt: GlobalWorkspace,
    pub predictive_cortex: PredictiveCortex,
    pub cycle: u64,
    pub total_anomalies: u64,
    pub repairs_triggered: u64,
    last_forecast: Option<HorizonForecast>,
    last_features: Vec<f64>,
}

impl Default for PanoramaPipeline {
    fn default() -> Self { Self::new() }
}

impl PanoramaPipeline {
    pub fn new() -> Self {
        Self {
            hypercube: KnowledgeHyperCube::new(),
            cortex: CortexMemory::new(20, 500),
            consciousness: ConsciousnessBridge::new(),
            gwt: GlobalWorkspace::new(0.5),
            predictive_cortex: PredictiveCortex::new(32, 64),
            cycle: 0,
            total_anomalies: 0,
            repairs_triggered: 0,
            last_forecast: None,
            last_features: Vec::new(),
        }
    }

    /// 前瞻预扫描: 在 run_cycle 之前预测 horizon 内的异常概率
    pub fn lookahead_scan(
        &mut self,
        brain: &SelfIteratingBrain,
        horizon: usize,
    ) -> LookaheadTag {
        let features: Vec<f64> = brain.brain.capability.arr.iter().take(32).copied().collect();
        let prob = self.predictive_cortex.lookahead_anomaly_prob(&features, horizon);
        let forecast = self.predictive_cortex.predict_horizon(&features, horizon);
        let best_fe = forecast.cumulative_fe;
        LookaheadTag {
            horizon_anomaly_prob: prob,
            horizon_best_fe: best_fe,
            horizon_divergence: forecast.divergence_step,
        }
    }

    pub fn run_cycle(
        &mut self,
        brain: &mut SelfIteratingBrain,
        goal_loop: &mut GoalLoop,
        nt_world_model: &mut WorldModelV2,
    ) -> PanoramaReport {
        self.cycle += 1;
        let mut repair_triggered_this_cycle = false;
        let mut goals_created = 0;

        // 1. 前瞻预扫描 (PredictiveCortex lookahead)
        let lookahead = self.lookahead_scan(brain, 3);

        // 如果前瞻预测到高风险, 提前创建预防性 goal
        if lookahead.horizon_anomaly_prob > 0.6 || lookahead.horizon_divergence.is_some() {
            let desc = format!(
                "preventive: PredictiveCortex forecasts anomaly prob={:.2} divergence={:?}",
                lookahead.horizon_anomaly_prob, lookahead.horizon_divergence
            );
            goals_created += goal_loop.enqueue_goal(brain, &desc, Some(GoalConfig::default()));
        }

        // 2. 世界模型预测
        let features: Vec<f64> = brain.brain.capability.arr.iter().take(64).copied().collect();
        let (fe_report, phi_report, anomaly) = nt_world_model.run_prediction_cycle(&features);
        let latent = nt_world_model.jepa.encode(&features);
        self.store_prediction(&latent, &fe_report, &phi_report);

        // 3. PredictiveCortex 结果记录 + 退化检测 (自修复回路)
        if let Some(ref last_forecast) = self.last_forecast.clone() {
            self.predictive_cortex.record_outcome(last_forecast, &latent);
            if let Some(repair) = self.predictive_cortex.detect_degradation() {
                self.repairs_triggered += 1;
                repair_triggered_this_cycle = true;

                // 创建自修复 goal
                let desc = format!(
                    "SELF-REPAIR: {} [severity={:.2}]",
                    repair.diagnosis, repair.severity
                );
                goals_created += goal_loop.enqueue_goal(brain, &desc, Some(GoalConfig::default()));

                // 存储修复信号到 ReasoningBank
                let edit_mem = ReasoningMemory::new(
                    &format!("repair_{}", self.cycle),
                    TaskType::Debugging,
                    &repair.suggested_edits,
                    0.5 + repair.severity * 0.5,
                );
                brain.reasoning_bank.store(edit_mem);
            }
        }

        // 保存本次特征和 forecast 供下一轮使用
        self.last_features = features.clone();
        let new_forecast = self.predictive_cortex.predict_horizon(&features.iter().take(32).copied().collect::<Vec<_>>(), 3);
        self.last_forecast = Some(new_forecast);

        ConsciousnessBridge::from_seal(brain, &mut self.gwt);
        let gwt_count = self.gwt.active_specialists().len();

        if anomaly {
            goals_created += self.create_anomaly_goal(goal_loop, brain, &fe_report);
            self.total_anomalies += 1;
        }
        if phi_report.is_conscious_like {
            goals_created += self.create_phi_goal(goal_loop, brain, &phi_report);
        }

        let mem = ReasoningMemory::new(
            &format!("panorama_cycle_{}", self.cycle),
            TaskType::Learning,
            &[
                MicroEdit::AdjustDimension("prediction_energy".into(), fe_report.prediction_energy.min(1.0)),
                MicroEdit::AdjustDimension("phi".into(), phi_report.phi.min(1.0)),
            ],
            0.7,
        );
        brain.reasoning_bank.store(mem);

        ConsciousnessBridge::to_seal(&self.gwt, brain);
        let bank_memories = brain.reasoning_bank.memories().len();

        PanoramaReport {
            cycle: self.cycle,
            hypercube_entries: self.hypercube.len(),
            cortex_traces: self.cortex.all_traces().len(),
            gwt_broadcasts: gwt_count,
            bank_memories,
            goals_created,
            anomaly,
            fe_energy: fe_report.variational_fe,
            phi: phi_report.phi,
            lookahead: Some(lookahead),
            forecast_quality: self.predictive_cortex.forecast_quality,
            repairs_triggered: self.repairs_triggered,
            repair_triggered_this_cycle,
        }
    }

    fn store_prediction(&mut self, latent: &[f64], fe: &FreeEnergyReport, phi: &PhiReport) {
        let mut coord = HyperCoord::new();
        for (i, &val) in latent.iter().enumerate().take(8) {
            if let Some(axis) = DimensionAxis::from_index(i) {
                coord.set(axis, val.abs().min(1.0));
            }
        }
        self.hypercube.insert(
            &coord,
            "nt_world_model",
            &format!("pred_{}_fe={:.3}_phi={:.3}", self.cycle, fe.variational_fe, phi.phi),
        );

        let trace = MemoryTrace::new(
            &format!("wm_pred_{}", self.cycle),
            "nt_world_model",
            &format!("JEPA latent pred energy={:.4} phi={:.4}", fe.prediction_energy, phi.phi),
            Modality::ReasoningTrace,
            vec![DimensionTag::General],
        )
        .with_importance(phi.phi.clamp(0.1, 1.0))
        .with_tags(vec![
            "nt_world_model".into(),
            "prediction".into(),
            format!("energy_{:.2}", fe.prediction_energy),
        ]);
        self.cortex.store(trace);

        let spec_type = if phi.is_conscious_like {
            SpecialistType::MetaCognitionAnalyst
        } else {
            SpecialistType::PatternMatcher
        };
        let mut module = SpecialistModule::new(spec_type, format!("wm_pred_{}", self.cycle));
        module.activate((1.0 - fe.prediction_energy.min(1.0)) * phi.phi.min(1.0));
        self.gwt.register(module);
    }

    fn create_anomaly_goal(&self, goal_loop: &mut GoalLoop, brain: &mut SelfIteratingBrain, fe: &FreeEnergyReport) -> usize {
        let desc = format!(
            "investigate world model anomaly — prediction_energy={:.3}",
            fe.prediction_energy
        );
        goal_loop.enqueue_goal(brain, &desc, Some(GoalConfig::default()))
    }

    fn create_phi_goal(&self, goal_loop: &mut GoalLoop, brain: &mut SelfIteratingBrain, phi: &PhiReport) -> usize {
        let desc = format!(
            "exploit conscious-like state — phi={:.3} energy={:.3}",
            phi.phi, phi.state_energy
        );
        goal_loop.enqueue_goal(brain, &desc, Some(GoalConfig::default()))
    }

    pub fn status(&self) -> String {
        format!(
            "Panorama: cycle={}, hypercube={}, cortex={}, gwt={}, anomalies={}",
            self.cycle,
            self.hypercube.len(),
            self.cortex.all_traces().len(),
            self.gwt.active_specialists().len(),
            self.total_anomalies,
        )
    }
}

impl DimensionAxis {
    fn from_index(i: usize) -> Option<Self> {
        let all = Self::all();
        all.get(i).copied()
    }
}
