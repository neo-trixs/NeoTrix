use crate::core::nt_core_hcube::cube::KnowledgeHyperCube;
use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use crate::core::nt_core_gwt::module_def::{SpecialistModule, SpecialistType};
use crate::core::nt_core_bank::ReasoningMemory;
use crate::core::nt_core_edit::MicroEdit;
use crate::neotrix::nt_world_model::{WorldModelV2, TaskType};
use crate::neotrix::nt_world_infer::FreeEnergyReport;
use crate::neotrix::l5_consciousness_impl::nt_core_iit_phi::PhiReport;
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
    pub horizon_divergence: Vec<f64>,
}

/// Panorama 报告
#[derive(Debug, Clone)]
pub struct PanoramaReport {
    pub cycle: u64,
    pub total_anomalies: u64,
    pub repairs_triggered: u64,
    pub repair_triggered_this_cycle: bool,
    pub anomaly: bool,
}

/// 全景管道 — 连接 GWT + 世界模型 + 预测皮层 + 超立方体记忆
pub struct PanoramaPipeline {
    pub cycle: u64,
    pub hypercube: KnowledgeHyperCube,
    pub gwt: GlobalWorkspace,
    pub cortex: CortexMemory,
    pub predictive_cortex: PredictiveCortex,
    pub last_features: Vec<f64>,
    pub last_forecast: Option<HorizonForecast>,
    pub total_anomalies: u64,
    pub repairs_triggered: u64,
}

impl PanoramaPipeline {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            hypercube: KnowledgeHyperCube::new(),
            gwt: GlobalWorkspace::new(13.0),
            cortex: CortexMemory::new(100, 1000),
            predictive_cortex: PredictiveCortex::new(32, 64),
            last_features: Vec::new(),
            last_forecast: None,
            total_anomalies: 0,
            repairs_triggered: 0,
        }
    }

    /// 运行全景循环: 预测 → 存储 → GWT广播 → 目标生成
    pub fn run_cycle(
        &mut self,
        brain: &mut SelfIteratingBrain,
        goal_loop: &mut GoalLoop,
        nt_world_model: &mut WorldModelV2,
    ) -> PanoramaReport {
        self.cycle += 1;
        
        let mut repair_triggered_this_cycle = false;

        // 1. PredictiveCortex 前瞻预测
        if !self.last_features.is_empty() {
            let forecast = self.predictive_cortex.predict_horizon(
                &self.last_features.iter().take(32).copied().collect::<Vec<_>>(),
                3,
            );
            let lookahead_anomaly = forecast.anomaly_predicted;
            if lookahead_anomaly {
                let desc = format!(
                    "preventive: PredictiveCortex forecasts anomaly — cumul_fe={:.3} divergence={:?}",
                    forecast.cumulative_fe, forecast.divergence_step
                );
                goal_loop.enqueue_goal(brain, &desc, Some(GoalConfig::default()));
            }
        }

        // 2. 世界模型预测
        let features: Vec<f64> = brain.brain.capability.arr.iter().take(64).copied().collect();
        let (fe_report, phi_value, anomaly) = nt_world_model.run_prediction_cycle(&features);
        let latent = nt_world_model.jepa.encode(&features);
        let phi_report = PhiReport {
            phi: phi_value,
            phi_raw: phi_value,
            total_resonance: phi_value,
            state_energy: fe_report.prediction_energy,
            effective_dims: 8,
            max_resonance_pair: (0, 0),
            phi_trend: 0.0,
            is_conscious_like: phi_value > 0.33,
        };
        self.store_prediction(&latent, &fe_report, &phi_report);

        // 3. PredictiveCortex 结果记录 + 退化检测 (自修复回路)
        if let Some(ref last_forecast) = self.last_forecast.clone() {
            self.predictive_cortex.record_outcome(last_forecast, &latent);
            if let Some(repair) = self.predictive_cortex.detect_degradation() {
                self.repairs_triggered += 1;
                repair_triggered_this_cycle = true;

                let desc = format!(
                    "SELF-REPAIR: {} [severity={:.2}]",
                    repair.diagnosis, repair.severity
                );
                goal_loop.enqueue_goal(brain, &desc, Some(GoalConfig::default()));

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
        let new_forecast = self.predictive_cortex.predict_horizon(
            &features.iter().take(32).copied().collect::<Vec<_>>(),
            3,
        );
        self.last_forecast = Some(new_forecast);

        ConsciousnessBridge::from_seal(brain, &mut self.gwt);
        let _gwt_count = self.gwt.active_specialists().len();

        if anomaly {
            self.create_anomaly_goal(goal_loop, brain, &fe_report);
            self.total_anomalies += 1;
        }
        if phi_report.is_conscious_like {
            self.create_phi_goal(goal_loop, brain, &phi_report);
        }

        let mem = ReasoningMemory::new(
            &format!("panorama_cycle_{}", self.cycle),
            TaskType::Learning,
            &[
                MicroEdit::AdjustDimension("prediction_energy".into(), fe_report.prediction_energy.min(1.0)),
                MicroEdit::AdjustDimension("phi".into(), phi_report.phi.min(1.0)),
            ],
            0.5 + phi_report.phi.min(0.5),
        );
        brain.reasoning_bank.store(mem);

        PanoramaReport {
            cycle: self.cycle,
            total_anomalies: self.total_anomalies,
            repairs_triggered: self.repairs_triggered,
            repair_triggered_this_cycle,
            anomaly,
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

        let spec_type = if phi.phi > 0.5 {
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

impl Default for PanoramaPipeline {
    fn default() -> Self {
        Self::new()
    }
}
