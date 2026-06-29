use std::collections::VecDeque;

use fastrand::Rng;

#[derive(Debug, Clone)]
pub struct FreeEnergyCuriosityEngine {
    transition_matrix: Vec<Vec<f64>>,
    likelihood_matrix: Vec<Vec<f64>>,
    beliefs: Vec<f64>,
    prediction_errors: VecDeque<f64>,
    max_history: usize,
    n_states: usize,
    n_observations: usize,
    exploration_weight: f64,
    cycle: u64,
    // Active Inference coupling
    pub active_inference_coupling: bool,
    pub joint_optimization_enabled: bool,
    pub policy_entropy_weight: f64,
    // Variational Free Energy fields
    last_prior: Vec<f64>,
    last_observation: Option<usize>,
    preferred_observations: Vec<f64>,
}

impl FreeEnergyCuriosityEngine {
    pub fn new(n_states: usize, n_observations: usize) -> Self {
        let uniform_belief = 1.0 / n_states as f64;
        let beliefs = vec![uniform_belief; n_states];
        let transition_matrix = vec![vec![uniform_belief; n_states]; n_states];
        let uniform_likelihood = 1.0 / n_observations as f64;
        let likelihood_matrix = vec![vec![uniform_likelihood; n_observations]; n_states];
        let prior = vec![uniform_belief; n_states];
        let preferred = vec![1.0 / n_observations as f64; n_observations];

        Self {
            transition_matrix,
            likelihood_matrix,
            beliefs,
            prediction_errors: VecDeque::with_capacity(100),
            max_history: 100,
            n_states,
            n_observations,
            exploration_weight: 1.0,
            cycle: 0,
            active_inference_coupling: true,
            joint_optimization_enabled: true,
            policy_entropy_weight: 0.3,
            last_prior: prior,
            last_observation: None,
            preferred_observations: preferred,
        }
    }

    pub fn step(&mut self, observation: usize) -> f64 {
        self.cycle += 1;
        self.last_observation = Some(observation);

        let likelihood: Vec<f64> = (0..self.n_states)
            .map(|s| self.likelihood_matrix[s][observation])
            .collect();

        let mut prior = vec![0.0; self.n_states];
        for s_next in 0..self.n_states {
            for s_prev in 0..self.n_states {
                prior[s_next] += self.transition_matrix[s_prev][s_next] * self.beliefs[s_prev];
            }
        }
        self.last_prior = prior.clone();

        let mut posterior = vec![0.0; self.n_states];
        let mut evidence = 0.0;
        for s in 0..self.n_states {
            posterior[s] = likelihood[s] * prior[s];
            evidence += posterior[s];
        }

        if evidence > 1e-30 {
            for s in 0..self.n_states {
                posterior[s] /= evidence;
            }
        } else {
            let uniform = 1.0 / self.n_states as f64;
            for s in 0..self.n_states {
                posterior[s] = uniform;
            }
        }

        let confidence = posterior.iter().cloned().fold(0.0_f64, f64::max);
        let prediction_error = 1.0 - confidence;

        self.prediction_errors.push_back(prediction_error);
        while self.prediction_errors.len() > self.max_history {
            self.prediction_errors.pop_front();
        }

        self.beliefs = posterior;

        prediction_error
    }

    pub fn curiosity_score(&self) -> f64 {
        let len = self.prediction_errors.len();
        if len == 0 {
            return 1.0;
        }
        let alpha = 0.3;
        let mut ema = self.prediction_errors[0];
        for &err in self.prediction_errors.iter().skip(1) {
            ema = alpha * err + (1.0 - alpha) * ema;
        }
        ema
    }

    pub fn select_action(&self, n_actions: usize) -> usize {
        let policies: Vec<Vec<usize>> = (0..n_actions).map(|a| vec![a]).collect();
        let post = self.policy_posterior(&policies);

        let temperature = self.policy_entropy_weight.max(0.1);
        let probs = Self::softmax(&post, temperature);

        let mut rng = Rng::new();
        let r: f64 = rng.f64();
        let mut cumulative = 0.0;
        for (i, &p) in probs.iter().enumerate() {
            cumulative += p;
            if r < cumulative {
                return i;
            }
        }
        n_actions - 1
    }

    pub fn action_curiosity_bonus(&self, _action_idx: usize) -> f64 {
        let max_state = self
            .beliefs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        let transition_slice = &self.transition_matrix[max_state];
        let next_state = transition_slice
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        let obs_dist = &self.likelihood_matrix[next_state];
        let obs_entropy: f64 = obs_dist
            .iter()
            .map(|&p| if p > 1e-10 { -p * p.ln() } else { 0.0 })
            .sum();
        let max_entropy = (self.n_observations as f64).ln();
        let normalized_entropy = if max_entropy > 1e-10 {
            obs_entropy / max_entropy
        } else {
            0.0
        };
        self.exploration_weight * normalized_entropy + self.curiosity_score()
    }

    pub fn reset(&mut self, new_n_states: usize, new_n_observations: usize) {
        let uniform_belief = 1.0 / new_n_states as f64;
        self.beliefs = vec![uniform_belief; new_n_states];
        self.transition_matrix = vec![vec![uniform_belief; new_n_states]; new_n_states];
        let uniform_likelihood = 1.0 / new_n_observations as f64;
        self.likelihood_matrix = vec![vec![uniform_likelihood; new_n_observations]; new_n_states];
        self.last_prior = vec![uniform_belief; new_n_states];
        self.last_observation = None;
        self.preferred_observations = vec![uniform_likelihood; new_n_observations];
        self.prediction_errors.clear();
        self.max_history = 100;
        self.n_states = new_n_states;
        self.n_observations = new_n_observations;
        self.exploration_weight = 1.0;
        self.cycle = 0;
    }

    /// Variational free energy: F = -∑ Q(s) ln P(o|s) + KL[Q(s) || P(s)]
    ///   - Accuracy: negative expected log-likelihood of observation under posterior
    ///   - Complexity: KL divergence between posterior and prior predictive
    pub fn variational_free_energy(&self) -> f64 {
        let obs_idx = match self.last_observation {
            Some(o) if o < self.n_observations => o,
            _ => return self.kl_divergence(&self.beliefs, &self.last_prior),
        };

        let accuracy: f64 = self
            .beliefs
            .iter()
            .enumerate()
            .map(|(s, &q)| {
                let ll = self.likelihood_matrix[s][obs_idx].ln().max(-20.0);
                -q * ll
            })
            .sum();

        let complexity = self.kl_divergence(&self.beliefs, &self.last_prior);

        accuracy + complexity
    }

    /// Expected free energy for a sequence of actions (policy).
    /// G(π) = -𝔼[IG] - 𝔼[ln P(o|C)]
    ///   - Epistemic value: expected information gain (KL reduction)
    ///   - Pragmatic value: expected log preference for predicted observations
    pub fn expected_free_energy(&self, policy: &[usize]) -> f64 {
        let mut beliefs = self.beliefs.clone();
        let mut epistemic = 0.0;
        let mut pragmatic = 0.0;

        for _action in policy {
            let mut pred_state = vec![0.0; self.n_states];
            for s_next in 0..self.n_states {
                for s_prev in 0..self.n_states {
                    pred_state[s_next] += self.transition_matrix[s_prev][s_next] * beliefs[s_prev];
                }
            }

            let mut pred_obs = vec![0.0; self.n_observations];
            for o in 0..self.n_observations {
                for s in 0..self.n_states {
                    pred_obs[o] += self.likelihood_matrix[s][o] * pred_state[s];
                }
            }

            // Epistemic: expected KL[Q(s|o) || Q(s)] over observations
            for o in 0..self.n_observations {
                if pred_obs[o] < 1e-10 {
                    continue;
                }
                let mut post = vec![0.0; self.n_states];
                let mut evidence = 0.0;
                for s in 0..self.n_states {
                    post[s] = self.likelihood_matrix[s][o] * pred_state[s];
                    evidence += post[s];
                }
                if evidence > 1e-30 {
                    for s in 0..self.n_states {
                        post[s] /= evidence;
                    }
                }
                epistemic += pred_obs[o] * self.kl_divergence(&post, &pred_state);
            }

            // Pragmatic: expected log preference for predicted observations
            for o in 0..self.n_observations {
                if pred_obs[o] > 1e-10 && self.preferred_observations[o] > 1e-10 {
                    pragmatic += pred_obs[o] * self.preferred_observations[o].ln();
                }
            }

            beliefs = pred_state;
        }

        -(epistemic + pragmatic)
    }

    /// Posterior over policies: P(π|o) ∝ P(o|π) · P(π)
    ///   - Prior: uniform over policies
    ///   - Likelihood: softmin(expected free energy) — lower EF = higher prob
    ///   - Posterior: normalized prior × likelihood
    pub fn policy_posterior(&self, policies: &[Vec<usize>]) -> Vec<f64> {
        let n = policies.len();
        if n == 0 {
            return vec![];
        }

        let ef_values: Vec<f64> = policies
            .iter()
            .map(|p| self.expected_free_energy(p))
            .collect();
        let prior = 1.0 / n as f64;

        let neg_ef: Vec<f64> = ef_values.iter().map(|&ef| -ef).collect();
        let likelihood = Self::softmax(&neg_ef, 1.0);

        let mut posterior: Vec<f64> = likelihood.iter().map(|&l| l * prior).collect();
        let sum: f64 = posterior.iter().sum();
        if sum > 1e-30 {
            for p in posterior.iter_mut() {
                *p /= sum;
            }
        }

        posterior
    }

    fn softmax(values: &[f64], temperature: f64) -> Vec<f64> {
        if values.is_empty() {
            return vec![];
        }
        let inv_temp = 1.0 / temperature.max(1e-10);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let shifted: Vec<f64> = values
            .iter()
            .map(|v| ((v - max_val) * inv_temp).exp())
            .collect();
        let sum: f64 = shifted.iter().sum();
        if sum > 1e-30 {
            shifted.iter().map(|v| v / sum).collect()
        } else {
            let uniform = 1.0 / values.len() as f64;
            vec![uniform; values.len()]
        }
    }

    fn kl_divergence(&self, p: &[f64], q: &[f64]) -> f64 {
        let mut kl = 0.0;
        for i in 0..p.len().min(q.len()) {
            if p[i] > 1e-10 && q[i] > 1e-10 {
                kl += p[i] * (p[i] / q[i]).ln();
            }
        }
        kl
    }

    pub fn with_exploration_weight(mut self, weight: f64) -> Self {
        self.exploration_weight = weight;
        self
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self.prediction_errors.reserve(max);
        self
    }

    pub fn with_transition_matrix(mut self, matrix: Vec<Vec<f64>>) -> Self {
        assert_eq!(matrix.len(), self.n_states);
        for row in &matrix {
            assert_eq!(row.len(), self.n_states);
        }
        self.transition_matrix = matrix;
        self
    }

    pub fn with_likelihood_matrix(mut self, matrix: Vec<Vec<f64>>) -> Self {
        assert_eq!(matrix.len(), self.n_states);
        for row in &matrix {
            assert_eq!(row.len(), self.n_observations);
        }
        self.likelihood_matrix = matrix;
        self
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn beliefs(&self) -> &[f64] {
        &self.beliefs
    }

    pub fn n_states(&self) -> usize {
        self.n_states
    }

    pub fn n_observations(&self) -> usize {
        self.n_observations
    }

    pub fn prediction_error_history(&self) -> &VecDeque<f64> {
        &self.prediction_errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_initializes_uniform_beliefs() {
        let eng = FreeEnergyCuriosityEngine::new(3, 4);
        assert_eq!(eng.n_states, 3);
        assert_eq!(eng.n_observations, 4);
        assert_eq!(eng.beliefs.len(), 3);
        assert!((eng.beliefs[0] - 1.0 / 3.0).abs() < 1e-10);
        assert_eq!(eng.transition_matrix.len(), 3);
        assert_eq!(eng.likelihood_matrix.len(), 3);
        assert_eq!(eng.cycle, 0);
    }

    #[test]
    fn test_step_returns_prediction_error() {
        let mut eng = FreeEnergyCuriosityEngine::new(2, 2);
        let pe = eng.step(0);
        assert!(pe >= 0.0 && pe <= 1.0);
        assert_eq!(eng.cycle, 1);
    }

    #[test]
    fn test_step_updates_beliefs() {
        let mut eng = FreeEnergyCuriosityEngine::new(2, 2);
        eng.likelihood_matrix = vec![vec![0.9, 0.1], vec![0.2, 0.8]];
        let before = eng.beliefs.clone();
        eng.step(0);
        let after = &eng.beliefs;
        let changed = before
            .iter()
            .zip(after.iter())
            .any(|(b, a)| (b - a).abs() > 1e-10);
        assert!(changed, "beliefs should update after observation");
    }

    #[test]
    fn test_curiosity_score_starts_high() {
        let eng = FreeEnergyCuriosityEngine::new(3, 5);
        let cs = eng.curiosity_score();
        assert!((cs - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_curiosity_score_after_steps() {
        let mut eng = FreeEnergyCuriosityEngine::new(2, 2);
        eng.likelihood_matrix = vec![vec![0.99, 0.01], vec![0.02, 0.98]];
        eng.step(0);
        eng.step(0);
        let cs = eng.curiosity_score();
        assert!(cs >= 0.0 && cs <= 1.0);
    }

    #[test]
    fn test_select_action_returns_valid_index() {
        let eng = FreeEnergyCuriosityEngine::new(3, 4);
        let action = eng.select_action(5);
        assert!(action < 5);
    }

    #[test]
    fn test_action_curiosity_bonus_non_negative() {
        let eng = FreeEnergyCuriosityEngine::new(3, 4);
        let bonus = eng.action_curiosity_bonus(0);
        assert!(bonus >= 0.0);
    }

    #[test]
    fn test_reset_reinitializes() {
        let mut eng = FreeEnergyCuriosityEngine::new(3, 4);
        eng.step(0);
        eng.step(1);
        assert!(eng.cycle > 0);
        eng.reset(5, 6);
        assert_eq!(eng.n_states, 5);
        assert_eq!(eng.n_observations, 6);
        assert_eq!(eng.beliefs.len(), 5);
        assert_eq!(eng.cycle, 0);
        assert!(eng.prediction_errors.is_empty());
    }

    #[test]
    fn test_with_exploration_weight() {
        let eng = FreeEnergyCuriosityEngine::new(2, 2).with_exploration_weight(2.5);
        assert!((eng.exploration_weight - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_with_max_history() {
        let eng = FreeEnergyCuriosityEngine::new(2, 2).with_max_history(50);
        assert_eq!(eng.max_history, 50);
    }

    #[test]
    fn test_posterior_normalized() {
        let mut eng = FreeEnergyCuriosityEngine::new(4, 3);
        eng.step(2);
        let sum: f64 = eng.beliefs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_prediction_error_bounds() {
        let mut eng = FreeEnergyCuriosityEngine::new(3, 3);
        for o in 0..3 {
            let pe = eng.step(o);
            assert!(pe >= 0.0 && pe <= 1.0);
        }
    }

    #[test]
    fn test_select_action_returns_valid_action_with_sampling() {
        let eng = FreeEnergyCuriosityEngine::new(2, 2);
        for _ in 0..20 {
            let a = eng.select_action(5);
            assert!(a < 5, "action {} must be < 5", a);
        }
    }

    #[test]
    fn test_variational_free_energy_finite() {
        let mut eng = FreeEnergyCuriosityEngine::new(3, 3);
        eng.step(1);
        let vfe = eng.variational_free_energy();
        assert!(vfe.is_finite(), "VFE must be finite, got {}", vfe);
    }

    #[test]
    fn test_variational_free_energy_no_observation() {
        let eng = FreeEnergyCuriosityEngine::new(3, 3);
        let vfe = eng.variational_free_energy();
        assert!(vfe.is_finite(), "VFE without observation must be finite");
    }

    #[test]
    fn test_expected_free_energy_finite() {
        let eng = FreeEnergyCuriosityEngine::new(3, 3);
        let efe = eng.expected_free_energy(&[0, 1]);
        assert!(efe.is_finite(), "EFE must be finite, got {}", efe);
    }

    #[test]
    fn test_expected_free_energy_empty_policy() {
        let eng = FreeEnergyCuriosityEngine::new(3, 3);
        let efe = eng.expected_free_energy(&[]);
        assert!(efe.is_finite(), "empty policy EFE must be finite");
    }

    #[test]
    fn test_policy_posterior_normalized() {
        let mut eng = FreeEnergyCuriosityEngine::new(3, 3);
        eng.step(0);
        let policies = vec![vec![0], vec![1], vec![2]];
        let post = eng.policy_posterior(&policies);
        assert_eq!(post.len(), 3);
        let sum: f64 = post.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "posterior sum = {}", sum);
    }

    #[test]
    fn test_policy_posterior_empty() {
        let eng = FreeEnergyCuriosityEngine::new(3, 3);
        let post = eng.policy_posterior(&[]);
        assert!(post.is_empty());
    }

    #[test]
    fn test_softmax_normalized() {
        let values = vec![1.0, 2.0, 3.0];
        let probs = FreeEnergyCuriosityEngine::softmax(&values, 1.0);
        assert_eq!(probs.len(), 3);
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        assert!(probs[2] > probs[1]);
        assert!(probs[1] > probs[0]);
    }

    #[test]
    fn test_softmax_empty() {
        let probs = FreeEnergyCuriosityEngine::softmax(&[], 1.0);
        assert!(probs.is_empty());
    }

    #[test]
    fn test_kl_divergence_same() {
        let eng = FreeEnergyCuriosityEngine::new(3, 3);
        let p = vec![0.5, 0.3, 0.2];
        let kl = eng.kl_divergence(&p, &p);
        assert!((kl - 0.0).abs() < 1e-10, "KL(p||p) = {}", kl);
    }

    #[test]
    fn test_kl_divergence_different() {
        let eng = FreeEnergyCuriosityEngine::new(3, 3);
        let p = vec![0.9, 0.05, 0.05];
        let q = vec![0.33, 0.33, 0.34];
        let kl = eng.kl_divergence(&p, &q);
        assert!(kl > 0.0, "KL(p||q) must be > 0, got {}", kl);
    }

    #[test]
    fn test_vfe_decreases_with_correct_observations() {
        let mut eng = FreeEnergyCuriosityEngine::new(2, 2);
        eng.likelihood_matrix = vec![vec![0.95, 0.05], vec![0.05, 0.95]];
        eng.transition_matrix = vec![vec![0.9, 0.1], vec![0.1, 0.9]];

        eng.step(0);
        let vfe_after_wrong = eng.variational_free_energy();

        eng.step(0);
        eng.step(0);
        let vfe_after_correct = eng.variational_free_energy();

        assert!(
            vfe_after_correct <= vfe_after_wrong + 1e-6,
            "VFE should decrease with consistent observations: {} vs {}",
            vfe_after_correct,
            vfe_after_wrong
        );
    }
}
