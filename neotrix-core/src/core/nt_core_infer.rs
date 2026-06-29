//! Active Inference Engine — Friston Free Energy Principle
//!
//! Normative drive for the consciousness kernel:
//!   variational free energy = prediction error - epistemic value
//!
//! Core equation:
//!   F = β·E_JEPA - H(E8)/T + γ·|∇E8|
//!
//! Where:
//!   - β: sensory precision (how much we trust observations)
//!   - E_JEPA: JEPA prediction energy (prediction error in latent space)
//!   - H(E8): E8 state entropy (uncertainty about hidden states)
//!   - T: temperature (exploration vs. exploitation balance)
//!   - γ·|∇E8|: novelty/transient detection (energy gradient penalty)
//!
//! Policy selection via expected free energy minimization:
//!   G(π) = -E_q[I(o;s|π)] + E_q[E_JEPA(o|s,π)]
//!        = -epistemic_value + expected_prediction_error
//!
//! Low free energy = well-calibrated world model + optimal exploration.

use serde::{Deserialize, Serialize};

/// Default sensory precision (β)
pub const DEFAULT_SENSORY_PRECISION: f64 = 1.0;

/// Default temperature (T) — high = more exploration
pub const DEFAULT_TEMPERATURE: f64 = 0.5;

/// Default gradient weight (γ)
pub const DEFAULT_GRADIENT_WEIGHT: f64 = 0.1;

/// Free energy moving average window
pub const FE_WINDOW_SIZE: usize = 20;

/// Free energy downward trend threshold — below this, model is considered converged
pub const FE_CONVERGENCE_THRESHOLD: f64 = 0.01;

/// Free energy analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeEnergyReport {
    /// Current variational free energy F
    pub variational_fe: f64,
    /// JEPA prediction energy contribution (β·E_JEPA)
    pub prediction_energy: f64,
    /// E8 entropy contribution (-H(E8)/T) — negative = uncertainty reduction
    pub entropy_contribution: f64,
    /// Energy gradient contribution (γ·|∇E8|) — transient detection
    pub gradient_penalty: f64,
    /// Epistemic value — positive component of information gain
    pub epistemic_value: f64,
    /// Free energy trend (negative = decreasing = learning)
    pub fe_trend: f64,
    /// Whether model has converged
    pub is_converged: bool,
}

/// Active Inference Engine — Friston Free Energy Principle implementation
///
/// Provides normative drive: minimize free energy = maximize world model prediction accuracy + optimal exploration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveInferenceEngine {
    /// Sensory precision β — observation confidence (high = trust observations more)
    pub nt_world_sense_precision: f64,
    /// Temperature T — exploration/exploitation balance
    pub temperature: f64,
    /// Gradient weight γ — transient sensitivity
    pub gradient_weight: f64,
    /// Free energy history (for trend analysis)
    pub fe_history: Vec<f64>,
    /// Current free energy
    pub current_fe: f64,
    /// Learning step count
    pub step: usize,
}

impl Default for ActiveInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveInferenceEngine {
    pub fn new() -> Self {
        Self {
            nt_world_sense_precision: DEFAULT_SENSORY_PRECISION,
            temperature: DEFAULT_TEMPERATURE,
            gradient_weight: DEFAULT_GRADIENT_WEIGHT,
            fe_history: Vec::with_capacity(FE_WINDOW_SIZE),
            current_fe: 0.0,
            step: 0,
        }
    }

    /// Compute variational free energy:
    ///   F = β·E_JEPA - H(E8)/T + γ·|∇E8|
    pub fn compute_free_energy(
        &mut self,
        jepa_energy: f64,
        e8_entropy: f64,
        e8_energy_gradient: f64,
    ) -> FreeEnergyReport {
        let prediction_energy = self.nt_world_sense_precision * jepa_energy;
        let entropy_contribution = -e8_entropy / self.temperature.max(1e-8);
        let gradient_penalty = self.gradient_weight * e8_energy_gradient.abs();
        let epistemic_value = e8_entropy * (1.0 - (-self.nt_world_sense_precision).exp());

        let fe = prediction_energy + entropy_contribution + gradient_penalty;
        self.current_fe = fe;
        self.step += 1;

        self.fe_history.push(fe);
        if self.fe_history.len() > FE_WINDOW_SIZE {
            self.fe_history.remove(0);
        }

        let fe_trend = if self.fe_history.len() >= 5 {
            let n = self.fe_history.len();
            let recent = &self.fe_history[n - 5..];
            let half = recent.len() / 2;
            let first_half_avg: f64 = recent[..half].iter().sum::<f64>() / half as f64;
            let second_half_avg: f64 =
                recent[half..].iter().sum::<f64>() / (recent.len() - half) as f64;
            second_half_avg - first_half_avg
        } else {
            0.0
        };

        let is_converged = fe_trend.abs() < FE_CONVERGENCE_THRESHOLD && fe < 1.0;

        FreeEnergyReport {
            variational_fe: fe,
            prediction_energy,
            entropy_contribution,
            gradient_penalty,
            epistemic_value,
            fe_trend,
            is_converged,
        }
    }

    /// Compute expected free energy G(π) for a given policy:
    ///   G(π) = -epistemic_value + expected_prediction_error
    pub fn expected_free_energy(&self, epistemic_value: f64, expected_energy: f64) -> f64 {
        -epistemic_value + expected_energy
    }

    /// Select optimal policy: minimize expected free energy
    pub fn select_policy(&self, policy_costs: &[(String, f64)]) -> Option<String> {
        policy_costs
            .iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone())
    }

    /// Free energy moving average
    pub fn fe_moving_avg(&self) -> f64 {
        if self.fe_history.is_empty() {
            return 0.0;
        }
        self.fe_history.iter().sum::<f64>() / self.fe_history.len() as f64
    }

    /// Reset engine
    pub fn reset(&mut self) {
        self.fe_history.clear();
        self.current_fe = 0.0;
        self.step = 0;
    }

    /// Configure sensory precision (confidence)
    pub fn with_precision(mut self, precision: f64) -> Self {
        self.nt_world_sense_precision = precision;
        self
    }

    /// Configure temperature (exploration/exploitation)
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_energy_basic_components() {
        let mut engine = ActiveInferenceEngine::new();
        let report = engine.compute_free_energy(0.5, 2.0, 0.1);
        assert!(report.variational_fe.is_finite());
        assert!(report.prediction_energy > 0.0);
        assert!(report.entropy_contribution < 0.0);
        assert!(report.gradient_penalty >= 0.0);
    }

    #[test]
    fn test_free_energy_decreases_with_learning() {
        let mut engine = ActiveInferenceEngine::new();
        let r1 = engine.compute_free_energy(1.0, 1.0, 0.1);
        let r2 = engine.compute_free_energy(0.1, 1.0, 0.01);
        assert!(r2.variational_fe < r1.variational_fe);
    }

    #[test]
    fn test_epistemic_value_positive() {
        let engine = ActiveInferenceEngine::new();
        let fe = engine.expected_free_energy(0.5, 0.3);
        assert!(fe.is_finite());
        assert!(fe < 0.3);
    }

    #[test]
    fn test_policy_selection() {
        let engine = ActiveInferenceEngine::new();
        let policies = vec![
            ("hero_ui".to_string(), 0.8),
            ("base_ui".to_string(), 0.3),
            ("shield".to_string(), 1.2),
        ];
        let selected = engine.select_policy(&policies);
        assert_eq!(selected, Some("base_ui".to_string()));
    }

    #[test]
    fn test_fe_trend_negative_when_improving() {
        let mut engine = ActiveInferenceEngine::new();
        for i in 0..10 {
            let e = (10 - i) as f64 * 0.2;
            engine.compute_free_energy(e, 0.5, 0.01);
        }
        let last_report = engine.compute_free_energy(0.01, 0.5, 0.001);
        assert!(
            last_report.fe_trend < 0.0 || last_report.is_converged,
            "FE should trend down: trend={:.6}",
            last_report.fe_trend
        );
    }

    #[test]
    fn test_convergence_detection() {
        let mut engine = ActiveInferenceEngine::new();
        for _ in 0..10 {
            engine.compute_free_energy(0.01, 0.05, 0.001);
        }
        let report = engine.compute_free_energy(0.01, 0.05, 0.001);
        assert!(report.is_converged);
    }

    #[test]
    fn test_precision_increases_prediction_weight() {
        let mut engine = ActiveInferenceEngine::with_precision(ActiveInferenceEngine::new(), 5.0);
        let report = engine.compute_free_energy(1.0, 1.0, 0.1);
        assert!((report.prediction_energy - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_temperature_scales_entropy() {
        let mut engine =
            ActiveInferenceEngine::with_temperature(ActiveInferenceEngine::new(), 10.0);
        let report = engine.compute_free_energy(0.5, 5.0, 0.1);
        assert!(
            (report.entropy_contribution - (-0.5)).abs() < 1e-10,
            "Expected -0.5, got {}",
            report.entropy_contribution
        );
    }

    #[test]
    fn test_select_policy_empty() {
        let engine = ActiveInferenceEngine::new();
        assert!(engine.select_policy(&[]).is_none());
    }

    #[test]
    fn test_reset_clears_state() {
        let mut engine = ActiveInferenceEngine::new();
        engine.compute_free_energy(1.0, 1.0, 0.1);
        assert!(engine.step > 0);
        engine.reset();
        assert_eq!(engine.step, 0);
        assert!(engine.fe_history.is_empty());
    }
}
