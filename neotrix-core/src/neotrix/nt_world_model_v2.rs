//! WorldModelV2 — 下一代世界预测引擎
//!
//! 集成 JEPA + E8 + Active Inference + IIT Φ 的统一预测接口。
//!
//! 全链路: JEPA(latent预测) → E8(64卦演化) → ActiveInference(自由能)
//!                                             → IIT(集成信息Φ)
//!
//! 从 nt_world_model.rs 拆分 (Session 30: 880→660行)

use crate::neotrix::nt_world_model::{WorldModel, Context};
use crate::neotrix::nt_world_jepa::JepaWorldModel;
use crate::neotrix::nt_world_e8::E8WorldModel;
use crate::neotrix::nt_world_infer::{ActiveInferenceEngine, FreeEnergyReport};
use crate::neotrix::iit_phi::{IITPhiCalculator, PhiReport};

/// V2 世界模型: 传统 WorldModel + JEPA + E8 + Active Inference + IIT
pub struct WorldModelV2 {
    pub legacy: WorldModel,
    pub jepa: JepaWorldModel,
    pub e8: E8WorldModel,
    pub nt_world_infer: ActiveInferenceEngine,
    pub iit: IITPhiCalculator,
    pub prediction_cache: Vec<f64>,
    pub last_jepa_energy: f64,
    pub last_e8_energy: f64,
    pub prediction_cycles: u64,
}

impl WorldModelV2 {
    pub fn new(num_experts: usize, input_dim: usize) -> Self {
        Self {
            legacy: WorldModel::new(num_experts),
            jepa: JepaWorldModel::new(input_dim),
            e8: E8WorldModel::new(),
            nt_world_infer: ActiveInferenceEngine::new(),
            iit: IITPhiCalculator::new(),
            prediction_cache: Vec::new(),
            last_jepa_energy: 0.0,
            last_e8_energy: 0.0,
            prediction_cycles: 0,
        }
    }

    pub fn predict_all(&self, context_features: &[f64]) -> (f64, f64, f64) {
        let (z_pred, jepa_energy) = self.jepa.predict(context_features);
        let e8_energy = {
            let mut e8 = self.e8.clone();
            e8.from_jepa_latent(&z_pred);
            e8.evolve_n(3, 1.0);
            e8.energy()
        };
        let ctx = Context::from_task_description("predict");
        let _expert_scores = self.legacy.predict_expert_performance(
            &vec![0.0; 64], &ctx, &[0],
        );
        (jepa_energy, e8_energy, 0.5)
    }

    pub fn run_prediction_cycle(&mut self, context_features: &[f64]) -> (FreeEnergyReport, PhiReport, bool) {
        self.prediction_cycles += 1;
        let (z_pred, jepa_energy) = self.jepa.predict(context_features);
        self.last_jepa_energy = jepa_energy;
        self.e8.from_jepa_latent(&z_pred);
        self.e8.evolve_n(3, 1.0);
        let e8_entropy = self.e8.entropy();
        let e8_energy = self.e8.energy();
        let e8_energy_gradient = e8_energy - self.last_e8_energy;
        self.last_e8_energy = e8_energy;
        let fe_report = self.nt_world_infer.compute_free_energy(
            jepa_energy, e8_entropy, e8_energy_gradient,
        );
        let phi_report = self.iit.analyze_e8(&self.e8);
        let anomaly = self.jepa.detect_anomaly(context_features, 2.0);
        self.prediction_cache = z_pred;
        (fe_report, phi_report, anomaly)
    }

    pub fn train_jepa(&mut self, x: &[f64], y: &[f64]) -> f64 {
        let (loss, _, _, _) = self.jepa.train_step(x, y);
        loss
    }

    pub fn detect_anomaly(&self, features: &[f64], threshold: f64) -> bool {
        self.jepa.detect_anomaly(features, threshold)
    }

    pub fn free_energy_report(&mut self) -> FreeEnergyReport {
        self.nt_world_infer.compute_free_energy(
            self.last_jepa_energy, 0.0, 0.0,
        )
    }

    pub fn phi_report(&self) -> PhiReport {
        self.iit.compute_from_e8(&self.e8)
    }

    pub fn with_precision(mut self, precision: f64) -> Self {
        self.nt_world_infer.nt_world_sense_precision = precision;
        self
    }

    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.nt_world_infer.temperature = temperature;
        self
    }

    pub fn with_phi_sigma(mut self, sigma: f64) -> Self {
        self.iit = self.iit.with_sigma(sigma);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nt_world_model_v2_new() {
        let wm = WorldModelV2::new(4, 64);
        assert_eq!(wm.prediction_cycles, 0);
    }

    #[test]
    fn test_predict_all_returns_valid() {
        let wm = WorldModelV2::new(4, 64);
        let features = (0..64).map(|i| (i as f64) / 64.0).collect::<Vec<_>>();
        let (jepa_energy, e8_energy, _) = wm.predict_all(&features);
        assert!(jepa_energy >= 0.0);
        assert!(e8_energy >= 0.0);
    }

    #[test]
    fn test_prediction_cycle_runs() {
        let mut wm = WorldModelV2::new(4, 64);
        let features = (0..64).map(|i| (i as f64) / 64.0).collect::<Vec<_>>();
        let (fe, phi, anomaly) = wm.run_prediction_cycle(&features);
        assert!(fe.variational_fe.is_finite());
        assert!(phi.phi >= 0.0);
        assert_eq!(wm.prediction_cycles, 1);
        // anomaly could be true or false, just check no panic
        let _ = anomaly;
    }

    #[test]
    fn test_configure_methods() {
        let wm = WorldModelV2::new(4, 64)
            .with_precision(2.0)
            .with_temperature(1.0)
            .with_phi_sigma(0.2);
        assert!((wm.nt_world_infer.nt_world_sense_precision - 2.0).abs() < 1e-10);
        assert!((wm.nt_world_infer.temperature - 1.0).abs() < 1e-10);
        assert!((wm.iit.sigma - 0.2).abs() < 1e-10);
    }
}
