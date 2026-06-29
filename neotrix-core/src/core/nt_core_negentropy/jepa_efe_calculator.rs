use super::efe_minimizer::{EFEResult, Policy, PolicyEvaluation};
use crate::core::nt_core_e8_model::E8WorldModel;
use crate::core::nt_core_infer::ActiveInferenceEngine;

/// Unified EFE calculator using `ActiveInferenceEngine::compute_free_energy`.
///
/// Wraps the Friston free energy principle into:
///   - `compute_act_efe()` — single EFE value for AcT's UCB1
///   - `compute_efe_full()` — full Risk/Ambiguity/InfoGain decomposition for EFEMinimizer
pub struct JepaEfeCalculator {
    ai_engine: ActiveInferenceEngine,
    e8: E8WorldModel,
}

impl JepaEfeCalculator {
    pub fn new(ai_engine: ActiveInferenceEngine, e8: E8WorldModel) -> Self {
        Self { ai_engine, e8 }
    }

    pub fn with_defaults() -> Self {
        Self {
            ai_engine: ActiveInferenceEngine::new(),
            e8: E8WorldModel::new(),
        }
    }

    /// Single EFE value for AcT's UCB1 selection.
    ///
    /// Computes F = β·E_JEPA − H(E8)/T + γ·|∇E8|
    /// where E_JEPA is the MSE between state and next_state as a proxy for
    /// prediction energy.
    pub fn compute_act_efe(&mut self, state: &[f64], next_state: &[f64]) -> f64 {
        let jepa_energy = Self::prediction_energy(state, next_state);
        let e8_entropy = self.e8.entropy();
        let prev_energy = self.e8.energy();
        let e8_gradient = Self::compute_energy_difference(prev_energy, jepa_energy);
        let report = self
            .ai_engine
            .compute_free_energy(jepa_energy, e8_entropy, e8_gradient);
        let efe = report.variational_fe;
        if !efe.is_finite() {
            return f64::MAX;
        }
        efe
    }

    /// Full EFE decomposition into Risk / Ambiguity / InfoGain.
    ///
    /// For each action index:
    ///   - `predicted` is the belief for that action
    ///   - `preferred` is the corresponding preferred outcome
    ///
    /// Returns `EFEResult` sorted by ascending EFE (best first).
    pub fn compute_efe_full(
        &mut self,
        belief: &[Vec<f64>],
        preferred: &[Vec<f64>],
        num_actions: usize,
    ) -> EFEResult {
        if num_actions == 0 {
            return EFEResult {
                best_policy: Policy(vec![]),
                evaluations: vec![],
                expected_free_energy: 0.0,
            };
        }
        let mut evaluations = Vec::with_capacity(num_actions);

        for a in 0..num_actions {
            let predicted = a.min(belief.len().saturating_sub(1));
            let preferred_idx = a.min(preferred.len().saturating_sub(1));

            let jepa_energy =
                Self::prediction_energy(&belief[predicted], &preferred[preferred_idx]);
            let e8_entropy = self.e8.entropy();
            let e8_gradient = Self::compute_energy_difference(self.e8.energy(), jepa_energy);

            let report = self
                .ai_engine
                .compute_free_energy(jepa_energy, e8_entropy, e8_gradient);

            let risk = Self::compute_kl(&belief[predicted], &preferred[preferred_idx]);
            let ambiguity = Self::compute_shannon_entropy(&belief[predicted]);
            let info_gain = report.epistemic_value;

            evaluations.push(PolicyEvaluation {
                policy: Policy(vec![a]),
                efe: report.variational_fe,
                risk,
                ambiguity,
                info_gain,
            });
        }

        evaluations.sort_by(|a, b| {
            a.efe
                .partial_cmp(&b.efe)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let best_policy = evaluations[0].policy.clone();
        let expected_free_energy = evaluations[0].efe;

        EFEResult {
            best_policy,
            evaluations,
            expected_free_energy,
        }
    }

    /// MSE between two vectors as prediction energy proxy.
    fn prediction_energy(a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        if n == 0 {
            return 1.0;
        }
        a.iter()
            .zip(b.iter())
            .take(n)
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            / n as f64
    }

    /// Energy difference between current E8 energy and prediction energy.
    fn compute_energy_difference(current_energy: f64, jepa_energy: f64) -> f64 {
        (current_energy - jepa_energy).abs().max(1e-8)
    }

    /// Softmax normalization: convert any vector to a probability distribution.
    fn softmax_normalize(v: &[f64]) -> Vec<f64> {
        let max = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_sum: f64 = v.iter().map(|&x| (x - max).exp()).sum();
        if exp_sum <= 0.0 || !exp_sum.is_finite() {
            return vec![1.0 / v.len() as f64; v.len()];
        }
        v.iter().map(|&x| (x - max).exp() / exp_sum).collect()
    }

    /// KL divergence KL(predicted ‖ preferred) over softmax-normalized vectors.
    fn compute_kl(predicted: &[f64], preferred: &[f64]) -> f64 {
        let p_dist = Self::softmax_normalize(predicted);
        let q_dist = Self::softmax_normalize(preferred);
        let mut kl = 0.0;
        for i in 0..p_dist.len().min(q_dist.len()) {
            let p = p_dist[i].clamp(1e-12, 1.0);
            let q = q_dist[i].clamp(1e-12, 1.0);
            kl += p * (p / q).ln();
        }
        kl
    }

    /// Shannon entropy H(p) = −Σ p·ln(p) over softmax-normalized vector.
    fn compute_shannon_entropy(belief: &[f64]) -> f64 {
        let dist = Self::softmax_normalize(belief);
        let mut h = 0.0;
        for &b in &dist {
            let p = b.clamp(1e-12, 1.0 - 1e-12);
            h -= p * p.ln();
        }
        h
    }

    pub fn ai_engine(&self) -> &ActiveInferenceEngine {
        &self.ai_engine
    }

    pub fn ai_engine_mut(&mut self) -> &mut ActiveInferenceEngine {
        &mut self.ai_engine
    }

    pub fn e8_model(&self) -> &E8WorldModel {
        &self.e8
    }

    pub fn e8_model_mut(&mut self) -> &mut E8WorldModel {
        &mut self.e8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_calculator() -> JepaEfeCalculator {
        JepaEfeCalculator::with_defaults()
    }

    #[test]
    fn test_efe_calculator_constructs() {
        let calc = make_calculator();
        assert!(calc.ai_engine.step == 0);
        assert!(calc.e8.entropy().is_finite());
    }

    #[test]
    fn test_compute_act_efe_lower_for_predictable() {
        let mut calc = make_calculator();
        let state = vec![0.5; 8];
        let identical = state.clone();
        let opposite = vec![0.0; 8];

        let efe_low = calc.compute_act_efe(&state, &identical);
        let efe_high = calc.compute_act_efe(&state, &opposite);

        assert!(
            efe_low <= efe_high,
            "identical states must give lower or equal EFE than opposite: {:.6} vs {:.6}",
            efe_low,
            efe_high
        );
    }

    #[test]
    fn test_compute_act_efe_finite() {
        let mut calc = make_calculator();
        let a = vec![0.3; 16];
        let b = vec![0.7; 16];
        let efe = calc.compute_act_efe(&a, &b);
        assert!(efe.is_finite(), "EFE must be finite, got {}", efe);
    }

    #[test]
    fn test_efe_full_returns_valid_evaluations() {
        let mut calc = make_calculator();
        let belief = vec![vec![0.5; 8], vec![0.6; 8], vec![0.4; 8]];
        let preferred = vec![vec![0.8; 8], vec![0.7; 8], vec![0.6; 8]];
        let result = calc.compute_efe_full(&belief, &preferred, 3);

        assert_eq!(result.evaluations.len(), 3);
        assert!(!result.evaluations.is_empty());

        for eval in &result.evaluations {
            assert!(eval.risk >= 0.0, "risk must be non-negative");
            assert!(eval.ambiguity >= 0.0, "ambiguity must be non-negative");
            assert!(eval.info_gain >= 0.0, "info_gain must be non-negative");
            assert!(eval.efe.is_finite(), "EFE must be finite");
        }
    }

    #[test]
    fn test_efe_full_sorted_by_efe() {
        let mut calc = make_calculator();
        let belief = vec![vec![0.2; 8], vec![0.5; 8], vec![0.9; 8]];
        let preferred = vec![vec![0.8; 8]; 3];
        let result = calc.compute_efe_full(&belief, &preferred, 3);

        let efes: Vec<f64> = result.evaluations.iter().map(|e| e.efe).collect();
        for i in 1..efes.len() {
            assert!(
                efes[i - 1] <= efes[i],
                "evaluations must be sorted by ascending EFE: {:?}",
                efes
            );
        }
        assert_eq!(result.best_policy.0.len(), 1);
    }

    #[test]
    fn test_efe_full_belief_longer_than_actions() {
        let mut calc = make_calculator();
        let belief = vec![vec![0.5; 4], vec![0.6; 4], vec![0.7; 4], vec![0.8; 4]];
        let preferred = vec![vec![0.9; 4]; 4];
        let result = calc.compute_efe_full(&belief, &preferred, 2);
        assert_eq!(result.evaluations.len(), 2);
    }

    #[test]
    fn test_efe_full_action_longer_than_beliefs() {
        let mut calc = make_calculator();
        let belief = vec![vec![0.5; 4]];
        let preferred = vec![vec![0.9; 4]];
        let result = calc.compute_efe_full(&belief, &preferred, 5);
        assert_eq!(result.evaluations.len(), 5);
    }

    #[test]
    fn test_prediction_energy_identical_zero() {
        let a = vec![0.5, 0.3, 0.2];
        let e = JepaEfeCalculator::prediction_energy(&a, &a);
        assert!((e - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_prediction_energy_different_positive() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 1.0];
        let e = JepaEfeCalculator::prediction_energy(&a, &b);
        assert!((e - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty_vector_prediction_energy() {
        let a: Vec<f64> = vec![];
        let b: Vec<f64> = vec![];
        let e = JepaEfeCalculator::prediction_energy(&a, &b);
        assert!((e - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_kl_divergence_identical_zero() {
        let a = vec![0.5, 0.3, 0.2];
        let kl = JepaEfeCalculator::compute_kl(&a, &a);
        assert!((kl - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_kl_divergence_different_positive() {
        let a = vec![0.8, 0.1, 0.1];
        let b = vec![0.2, 0.6, 0.2];
        let kl = JepaEfeCalculator::compute_kl(&a, &b);
        assert!(kl > 0.0);
    }

    #[test]
    fn test_shannon_entropy_uniform_max() {
        let uniform = vec![0.25, 0.25, 0.25, 0.25];
        let h = JepaEfeCalculator::compute_shannon_entropy(&uniform);
        let max_h = (4.0_f64).ln();
        assert!((h - max_h).abs() < 1e-10);
    }

    #[test]
    fn test_shannon_entropy_deterministic_zero() {
        let deterministic = vec![1.0, 0.0, 0.0, 0.0];
        let h = JepaEfeCalculator::compute_shannon_entropy(&deterministic);
        assert!(h < 1e-10);
    }
}
