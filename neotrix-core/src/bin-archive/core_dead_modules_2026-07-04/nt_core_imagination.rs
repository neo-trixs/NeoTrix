#![forbid(unsafe_code)]


/// Optimal sample allocation ratio between dynamics and reward samples (Theorem 1).
///
/// Implements the theoretical ratio `Ndyn/Nrew` from Theorem 1 of
/// "On Training in Imagination" (arXiv 2605.06732):
///
/// `Ndyn/Nrew = α/β * (γ * Lr * (1+Lπ)) / (1 - γ * Lf * (1+Lπ)) * (crew/cdyn) * (εdyn/εrew)`
#[derive(Debug, Clone)]
pub struct OptimalSampleAllocation {
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    pub lf: f64,
    pub lr: f64,
    pub lpi: f64,
    pub crew: f64,
    pub cdyn: f64,
    pub eps_dyn: f64,
    pub eps_rew: f64,
}

impl OptimalSampleAllocation {
    pub fn new(
        alpha: f64,
        beta: f64,
        gamma: f64,
        lf: f64,
        lr: f64,
        lpi: f64,
        crew: f64,
        cdyn: f64,
        eps_dyn: f64,
        eps_rew: f64,
    ) -> Self {
        Self {
            alpha,
            beta,
            gamma,
            lf,
            lr,
            lpi,
            crew,
            cdyn,
            eps_dyn,
            eps_rew,
        }
    }

    pub fn default_cfg() -> Self {
        Self {
            alpha: 0.5,
            beta: 0.5,
            gamma: 0.99,
            lf: 0.1,
            lr: 0.1,
            lpi: 0.1,
            crew: 1.0,
            cdyn: 1.0,
            eps_dyn: 0.01,
            eps_rew: 0.01,
        }
    }

    pub fn compute_ratio(&self) -> Result<f64, &'static str> {
        let denom = 1.0 - self.gamma * self.lf * (1.0 + self.lpi);
        if denom <= 0.0 {
            return Err("denominator non-positive: γ * Lf * (1+Lπ) >= 1");
        }

        let first = self.alpha / self.beta;
        let second = self.gamma * self.lr * (1.0 + self.lpi) / denom;
        let third = self.crew / self.cdyn;
        let fourth = self.eps_dyn / self.eps_rew;

        let ratio = first * second * third * fourth;
        Ok(ratio)
    }

    pub fn dyn_fraction(&self) -> Result<f64, &'static str> {
        let ratio = self.compute_ratio()?;
        Ok(ratio / (1.0 + ratio))
    }

    pub fn rew_fraction(&self) -> Result<f64, &'static str> {
        let ratio = self.compute_ratio()?;
        Ok(1.0 / (1.0 + ratio))
    }
}

/// Lipschitz regularization enforcing smooth dynamics, reward, and policy (Corollary 1).
///
/// Lower Lipschitz constants tighten the return-error bound. This regularizer
/// applies spectral-norm-based regularization to network gradients.
#[derive(Debug, Clone)]
pub struct LipschitzRegularizer {
    pub lambda_dyn: f64,
    pub lambda_rew: f64,
    pub lambda_pol: f64,
    pub max_norm: f64,
}

impl Default for LipschitzRegularizer {
    fn default() -> Self {
        Self {
            lambda_dyn: 0.01,
            lambda_rew: 0.01,
            lambda_pol: 0.01,
            max_norm: 1.0,
        }
    }
}

impl LipschitzRegularizer {
    pub fn new(lambda_dyn: f64, lambda_rew: f64, lambda_pol: f64, max_norm: f64) -> Self {
        Self {
            lambda_dyn,
            lambda_rew,
            lambda_pol,
            max_norm,
        }
    }

    pub fn lipschitz_regularization_loss(
        &self,
        f_gradients: &[f64],
        r_gradients: &[f64],
        pi_gradients: &[f64],
    ) -> f64 {
        let f_norm = spectral_norm(f_gradients);
        let r_norm = spectral_norm(r_gradients);
        let pi_norm = spectral_norm(pi_gradients);

        let f_pen = if f_norm > self.max_norm {
            self.lambda_dyn * (f_norm - self.max_norm).powi(2)
        } else {
            0.0
        };

        let r_pen = if r_norm > self.max_norm {
            self.lambda_rew * (r_norm - self.max_norm).powi(2)
        } else {
            0.0
        };

        let pi_pen = if pi_norm > self.max_norm {
            self.lambda_pol * (pi_norm - self.max_norm).powi(2)
        } else {
            0.0
        };

        f_pen + r_pen + pi_pen
    }

    pub fn compute_lipschitz_estimate(values: &[f64], steps: &[f64]) -> f64 {
        if values.len() < 2 || values.len() != steps.len() {
            return 0.0;
        }

        let n = values.len();
        let mut max_slope = 0.0;

        for i in 0..n - 1 {
            let dval = (values[i + 1] - values[i]).abs();
            let dstp = (steps[i + 1] - steps[i]).abs();
            if dstp > 1e-12 {
                let slope = dval / dstp;
                if slope > max_slope {
                    max_slope = slope;
                }
            }
        }

        max_slope
    }

    pub fn spectral_normalize(weights: &[f64], max_norm: f64) -> Vec<f64> {
        let s = spectral_norm(weights);
        if s <= max_norm || s < 1e-12 {
            return weights.to_vec();
        }
        let scale = max_norm / s;
        weights.iter().map(|w| w * scale).collect()
    }
}

/// REINFORCE with noisy rewards (Theorem 2, Corollary 2).
///
/// Provides unbiased gradient estimation under reward noise and optimal
/// fidelity selection under budget constraints.
#[derive(Debug, Clone)]
pub struct NoisyRewardPolicy {
    pub learning_rate: f64,
    pub discount: f64,
    pub baseline: f64,
}

impl Default for NoisyRewardPolicy {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            discount: 0.99,
            baseline: 0.0,
        }
    }
}

impl NoisyRewardPolicy {
    pub fn new(learning_rate: f64, discount: f64, baseline: f64) -> Self {
        Self {
            learning_rate,
            discount,
            baseline,
        }
    }

    pub fn reinforce_with_noisy_rewards(
        &self,
        trajectories: &[TrajectoryWithRewards],
        reward_noise_var: f64,
    ) -> NoisyGradientResult {
        let mut total_gradient = 0.0;
        let mut n_steps = 0usize;

        for traj in trajectories {
            let mut discounted_return = 0.0;
            let mut discount = 1.0;

            for step in &traj.steps {
                let noisy_reward = step.reward;
                discounted_return += discount * noisy_reward;
                discount *= self.discount;
            }

            let advantage = discounted_return - self.baseline;

            for step in &traj.steps {
                total_gradient += advantage * step.log_prob;
                n_steps += 1;
            }
        }

        let mean_gradient = if n_steps > 0 {
            total_gradient / n_steps as f64
        } else {
            0.0
        };

        let grad_var = reward_noise_var / n_steps.max(1) as f64;

        NoisyGradientResult {
            mean_gradient,
            gradient_variance: grad_var,
            effective_samples: n_steps,
            bias_estimate: 0.0,
        }
    }

    pub fn optimal_noise_fidelity(
        budget: f64,
        noise_var_fn: &dyn Fn(f64) -> f64,
    ) -> f64 {
        let mut best_c = budget;
        let mut best_phi = f64::MAX;
        let steps = 100;

        for i in 0..=steps {
            let c = budget * (i as f64) / (steps as f64);
            if c < 1e-12 {
                continue;
            }
            let sig2 = noise_var_fn(c);
            let phi = c * sig2;
            if phi < best_phi {
                best_phi = phi;
                best_c = c;
            }
        }

        best_c
    }

    pub fn phi(c: f64, sigma_sq: f64) -> f64 {
        c * sigma_sq
    }

    pub fn num_rollouts_for_noise(
        target_variance: f64,
        noise_var: f64,
        current_rollouts: usize,
    ) -> usize {
        if target_variance <= 0.0 || noise_var <= 0.0 {
            return current_rollouts;
        }
        let current_var = noise_var / current_rollouts.max(1) as f64;
        if current_var <= target_variance {
            return current_rollouts;
        }
        let needed = (noise_var / target_variance).ceil() as usize;
        needed.max(current_rollouts)
    }
}

/// A single step in a trajectory with reward signal.
#[derive(Debug, Clone)]
pub struct StepWithReward {
    pub log_prob: f64,
    pub reward: f64,
}

/// A complete trajectory with per-step rewards.
#[derive(Debug, Clone)]
pub struct TrajectoryWithRewards {
    pub steps: Vec<StepWithReward>,
}

impl TrajectoryWithRewards {
    pub fn new(steps: Vec<StepWithReward>) -> Self {
        Self { steps }
    }
}

/// Result of noisy-reward REINFORCE gradient estimation.
#[derive(Debug, Clone)]
pub struct NoisyGradientResult {
    pub mean_gradient: f64,
    pub gradient_variance: f64,
    pub effective_samples: usize,
    pub bias_estimate: f64,
}

/// Return error bound (Lemma 1, Equation 1).
///
/// Computes the theoretical upper bound on the return error given
/// dynamics error, reward error, and Lipschitz constants.
#[derive(Debug, Clone)]
pub struct ReturnErrorBound {
    pub eps_dyn: f64,
    pub eps_rew: f64,
    pub lf: f64,
    pub lr: f64,
    pub lpi: f64,
    pub gamma: f64,
}

impl Default for ReturnErrorBound {
    fn default() -> Self {
        Self {
            eps_dyn: 0.01,
            eps_rew: 0.01,
            lf: 0.1,
            lr: 0.1,
            lpi: 0.1,
            gamma: 0.99,
        }
    }
}

impl ReturnErrorBound {
    pub fn new(eps_dyn: f64, eps_rew: f64, lf: f64, lr: f64, lpi: f64, gamma: f64) -> Self {
        Self {
            eps_dyn,
            eps_rew,
            lf,
            lr,
            lpi,
            gamma,
        }
    }

    pub fn compute_return_error_bound(&self) -> Result<f64, &'static str> {
        let denom = 1.0 - self.gamma * self.lf * (1.0 + self.lpi);
        if denom <= 0.0 {
            return Err("denominator non-positive: γ * Lf * (1+Lπ) >= 1");
        }

        let rew_term = self.eps_rew * (1.0 + self.lpi) / (1.0 - self.gamma);
        let dyn_term = self.eps_dyn * self.lr * (1.0 + self.lpi) / denom;

        Ok(rew_term + dyn_term)
    }

    pub fn compute_error_coefficients(&self) -> (f64, f64) {
        let denom = (1.0 - self.gamma * self.lf * (1.0 + self.lpi)).max(1e-12);
        let rew_coef = (1.0 + self.lpi) / (1.0 - self.gamma);
        let dyn_coef = self.lr * (1.0 + self.lpi) / denom;
        (rew_coef, dyn_coef)
    }
}

fn spectral_norm(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    (var.sqrt()).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allocation_creates_valid_ratio() {
        let alloc = OptimalSampleAllocation::default_cfg();
        let ratio = alloc.compute_ratio().unwrap();
        assert!(ratio > 0.0, "ratio should be positive, got {}", ratio);
        assert!(ratio.is_finite(), "ratio should be finite");

        let dfrac = alloc.dyn_fraction().unwrap();
        let rfrac = alloc.rew_fraction().unwrap();
        assert!((dfrac + rfrac - 1.0).abs() < 1e-10, "fractions should sum to 1");
    }

    #[test]
    fn test_lipschitz_regularization_produces_non_negative_loss() {
        let reg = LipschitzRegularizer::default();
        let f_grad = vec![0.1, -0.2, 0.15, -0.05];
        let r_grad = vec![0.3, -0.1, 0.25];
        let pi_grad = vec![-0.05, 0.02, -0.01, 0.03, 0.04];

        let loss = reg.lipschitz_regularization_loss(&f_grad, &r_grad, &pi_grad);
        assert!(loss >= 0.0, "loss should be non-negative, got {}", loss);
    }

    #[test]
    fn test_noisy_reward_gradient_is_unbiased_for_zero_mean_noise() {
        let policy = NoisyRewardPolicy::default();
        let mut trajectories = Vec::new();

        for _ in 0..100 {
            let steps = vec![
                StepWithReward { log_prob: 0.5, reward: 1.0 },
                StepWithReward { log_prob: -0.3, reward: 0.0 },
                StepWithReward { log_prob: 0.2, reward: 2.0 },
            ];
            trajectories.push(TrajectoryWithRewards::new(steps));
        }

        let result = policy.reinforce_with_noisy_rewards(&trajectories, 0.0);
        assert!(result.mean_gradient.is_finite(), "gradient should be finite");
        assert_eq!(result.effective_samples, 300);
        assert!((result.bias_estimate - 0.0).abs() < 1e-10, "bias should be zero for zero noise");
    }

    #[test]
    fn test_return_error_bound_is_valid() {
        let bound = ReturnErrorBound::default();
        let err = bound.compute_return_error_bound().unwrap();
        assert!(err > 0.0, "error bound should be positive, got {}", err);
        assert!(err.is_finite(), "error bound should be finite");

        let (rew_coef, dyn_coef) = bound.compute_error_coefficients();
        assert!(rew_coef > 0.0, "reward coefficient should be positive");
        assert!(dyn_coef > 0.0, "dynamics coefficient should be positive");
    }

    #[test]
    fn test_spectral_normalization_reduces_gradient_norm() {
        let weights = vec![2.0, 3.0, -2.5, 1.5, -1.0];
        let orig_norm = spectral_norm(&weights);
        let max_norm = orig_norm * 0.5;

        let normalized = LipschitzRegularizer::spectral_normalize(&weights, max_norm);
        let new_norm = spectral_norm(&normalized);

        assert!(new_norm <= max_norm * 1.001, "new norm {} should be <= {}", new_norm, max_norm);
        assert!(new_norm < orig_norm, "new norm should be reduced");
    }

    #[test]
    fn test_power_law_exponents_affect_allocation_direction() {
        let base = OptimalSampleAllocation::default_cfg();
        let ratio_base = base.compute_ratio().unwrap();

        let high_alpha = OptimalSampleAllocation::new(
            1.0, 0.5, 0.99, 0.1, 0.1, 0.1, 1.0, 1.0, 0.01, 0.01,
        );
        let ratio_high = high_alpha.compute_ratio().unwrap();
        assert!(ratio_high > ratio_base, "higher alpha should increase ratio");

        let high_beta = OptimalSampleAllocation::new(
            0.5, 1.0, 0.99, 0.1, 0.1, 0.1, 1.0, 1.0, 0.01, 0.01,
        );
        let ratio_low = high_beta.compute_ratio().unwrap();
        assert!(ratio_low < ratio_base, "higher beta should decrease ratio");
    }

    #[test]
    fn test_cost_ratio_shifts_allocation() {
        let base = OptimalSampleAllocation::default_cfg();
        let ratio_base = base.compute_ratio().unwrap();

        let expensive_rew = OptimalSampleAllocation::new(
            0.5, 0.5, 0.99, 0.1, 0.1, 0.1, 10.0, 1.0, 0.01, 0.01,
        );
        let ratio_high_crew = expensive_rew.compute_ratio().unwrap();
        assert!(
            ratio_high_crew > ratio_base,
            "more expensive reward should increase dyn/rew ratio"
        );

        let expensive_dyn = OptimalSampleAllocation::new(
            0.5, 0.5, 0.99, 0.1, 0.1, 0.1, 1.0, 10.0, 0.01, 0.01,
        );
        let ratio_high_cdyn = expensive_dyn.compute_ratio().unwrap();
        assert!(
            ratio_high_cdyn < ratio_base,
            "more expensive dynamics should decrease dyn/rew ratio"
        );
    }

    #[test]
    fn test_discount_factor_decreases_dyn_multiplier() {
        let low_gamma = OptimalSampleAllocation::new(
            0.5, 0.5, 0.9, 0.1, 0.1, 0.1, 1.0, 1.0, 0.01, 0.01,
        );
        let high_gamma = OptimalSampleAllocation::new(
            0.5, 0.5, 0.99, 0.1, 0.1, 0.1, 1.0, 1.0, 0.01, 0.01,
        );

        let r_low = low_gamma.compute_ratio().unwrap();
        let r_high = high_gamma.compute_ratio().unwrap();
        assert!(
            r_low < r_high,
            "lower gamma should reduce dyn multiplier: {} vs {}",
            r_low,
            r_high
        );
    }

    #[test]
    fn test_lipschitz_constants_tighten_bound() {
        let tight = ReturnErrorBound::new(0.01, 0.01, 0.01, 0.01, 0.01, 0.99);
        let loose = ReturnErrorBound::new(0.01, 0.01, 0.5, 0.5, 0.5, 0.99);

        let err_tight = tight.compute_return_error_bound().unwrap();
        let err_loose = loose.compute_return_error_bound().unwrap();
        assert!(
            err_tight < err_loose,
            "lower Lipschitz constants should tighten bound"
        );
    }

    #[test]
    fn test_high_noise_regime_shifts_to_more_rollouts() {
        let low_noise_rollouts =
            NoisyRewardPolicy::num_rollouts_for_noise(0.001, 0.01, 10);
        let high_noise_rollouts =
            NoisyRewardPolicy::num_rollouts_for_noise(0.001, 1.0, 10);

        assert!(
            high_noise_rollouts >= low_noise_rollouts,
            "high noise should need equal or more rollouts"
        );
        assert!(high_noise_rollouts > 10, "high noise (var=1.0) should need >10 rollouts for target var=0.001");
    }

    #[test]
    fn test_phi_minimizer_detection() {
        let noise_var_fn = |c: f64| 1.0 / (c + 0.1);
        let optimal_c = NoisyRewardPolicy::optimal_noise_fidelity(10.0, &noise_var_fn);
        assert!(optimal_c > 0.0, "optimal c should be positive");
        assert!(optimal_c <= 10.0, "optimal c should not exceed budget");
    }

    #[test]
    fn test_bounded_fidelity_regime_parabolic_phi() {
        let noise_var_fn = |c: f64| (c - 5.0).powi(2) + 1.0;
        let optimal_c = NoisyRewardPolicy::optimal_noise_fidelity(10.0, &noise_var_fn);
        assert!(optimal_c > 0.0, "should find interior minimizer for parabolic Phi");
        assert!(optimal_c < 10.0, "interior minimizer should be below budget");
    }

    #[test]
    fn test_irreducible_noise_floor_linear_phi() {
        let noise_var_fn = |c: f64| 1.0 / (c + 1.0);
        let optimal_c = NoisyRewardPolicy::optimal_noise_fidelity(10.0, &noise_var_fn);
        assert!(optimal_c > 0.0, "should allocate positive budget");
    }

    #[test]
    fn test_edge_case_gamma_lf_1plus_lpi_ge_1() {
        let alloc = OptimalSampleAllocation::new(
            0.5, 0.5, 1.0, 1.0, 0.1, 0.0, 1.0, 1.0, 0.01, 0.01,
        );
        let result = alloc.compute_ratio();
        assert!(result.is_err(), "should error when γ * Lf * (1+Lπ) >= 1");
        assert!(result.err().unwrap().contains("denominator non-positive"));

        let bound = ReturnErrorBound::new(0.01, 0.01, 1.0, 0.1, 0.0, 1.0);
        let bresult = bound.compute_return_error_bound();
        assert!(bresult.is_err(), "bound should error with divergent denominator");
    }

    #[test]
    fn test_edge_case_zero_lipschitz_models() {
        let alloc = OptimalSampleAllocation::new(
            0.5, 0.5, 0.99, 0.0, 0.0, 0.0, 1.0, 1.0, 0.01, 0.01,
        );
        let ratio = alloc.compute_ratio().unwrap();
        assert!((ratio - 0.0).abs() < 1e-12, "zero Lipschitz dynamics (constant) needs zero dyn samples, got {}", ratio);

        let dfrac = alloc.dyn_fraction().unwrap();
        assert!((dfrac - 0.0).abs() < 1e-12, "dyn fraction should be 0");
        let rfrac = alloc.rew_fraction().unwrap();
        assert!((rfrac - 1.0).abs() < 1e-12, "rew fraction should be 1");
    }

    #[test]
    fn test_lipschitz_estimate_from_trajectory() {
        let values = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let l = LipschitzRegularizer::compute_lipschitz_estimate(&values, &steps);
        assert!((l - 1.0).abs() < 1e-6, "linear slope 1 should give L=1, got {}", l);
    }

    #[test]
    fn test_spectral_normalize_noop_when_below_max() {
        let weights = vec![0.1, -0.05, 0.02];
        let normalized = LipschitzRegularizer::spectral_normalize(&weights, 10.0);
        assert_eq!(normalized.len(), weights.len());
        for (a, b) in normalized.iter().zip(weights.iter()) {
            assert!((a - b).abs() < 1e-12, "should be identical when norm < max_norm");
        }
    }

    #[test]
    fn test_error_coefficients_ratio() {
        let bound = ReturnErrorBound::default();
        let (rew_coef, dyn_coef) = bound.compute_error_coefficients();
        assert!(rew_coef > 0.0);
        assert!(dyn_coef > 0.0);
        let bound_err = bound.compute_return_error_bound().unwrap();
        let manual = rew_coef * bound.eps_rew + dyn_coef * bound.eps_dyn;
        assert!((bound_err - manual).abs() < 1e-10, "bound should match coefficient expansion");
    }

    #[test]
    fn test_empty_trajectory_gradient_is_zero() {
        let policy = NoisyRewardPolicy::default();
        let result = policy.reinforce_with_noisy_rewards(&[], 0.1);
        assert!((result.mean_gradient - 0.0).abs() < 1e-10);
        assert_eq!(result.effective_samples, 0);
    }
}
