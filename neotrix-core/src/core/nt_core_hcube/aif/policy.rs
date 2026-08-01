#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::free_energy::FreeEnergyCalculator;
use super::generative_model::GenerativeModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluator {
    pub num_states: usize,
    pub horizon: usize,
}

impl PolicyEvaluator {
    pub fn new(num_states: usize, horizon: usize) -> Self {
        PolicyEvaluator { num_states, horizon }
    }

    pub fn evaluate_policy(
        &self,
        policy: &[usize],
        _belief: &[f64],
        model: &GenerativeModel,
    ) -> f64 {
        let calc = FreeEnergyCalculator::new();
        let mut total_efe = 0.0;

        let steps = policy.len().min(self.horizon);
        for &action in policy.iter().take(steps) {
            let from = action;
            let mut next_belief = vec![0.0; self.num_states];
            for j in 0..self.num_states {
                if from < self.num_states {
                    next_belief[j] = model.transition_matrix[from][j];
                }
            }

            let complexity = calc.compute_complexity(&next_belief, &model.prior_over_states);

            let mut inaccuracy = 0.0;
            for s in 0..self.num_states {
                if next_belief[s] > 0.0 {
                    let log_val = (next_belief[s] / (model.prior_over_states[s] + 1e-10) + 1e-10)
                        .ln();
                    inaccuracy += next_belief[s] * log_val;
                }
            }

            total_efe += complexity + inaccuracy;
        }

        total_efe.max(-1e10).min(1e10)
    }

    pub fn evaluate_policies(
        &self,
        policies: &[Vec<usize>],
        belief: &[f64],
        model: &GenerativeModel,
    ) -> Vec<(usize, f64)> {
        let mut results = Vec::with_capacity(policies.len());
        for (i, policy) in policies.iter().enumerate() {
            let efe = self.evaluate_policy(policy, belief, model);
            results.push((i, efe));
        }
        results
    }

    pub fn select_best_policy(
        &self,
        policies: &[Vec<usize>],
        belief: &[f64],
        model: &GenerativeModel,
    ) -> (usize, f64) {
        let results = self.evaluate_policies(policies, belief, model);
        results
            .into_iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((0, f64::MAX))
    }

    pub fn compute_expected_information_gain(
        policy: &[usize],
        belief: &[f64],
        model: &GenerativeModel,
    ) -> f64 {
        let calc = FreeEnergyCalculator::new();
        let mut current_belief = belief.to_vec();
        let mut total_gain = 0.0;

        for &action in policy {
            let mut next_belief = vec![0.0; current_belief.len()];
            if action < model.num_states && current_belief.len() >= model.num_states {
                next_belief[..model.num_states].copy_from_slice(&model.transition_matrix[action][..model.num_states]);
            }

            let predicted_obs = model.predict_observation(&next_belief);
            let prior_entropy = calc.compute_entropy(&current_belief);

            let mut expected_posterior_entropy = 0.0;
            for o in 0..model.num_observations {
                let mut posterior = vec![0.0; model.num_states];
                let mut evidence = 0.0;
                for s in 0..model.num_states {
                    posterior[s] = model.likelihood_matrix[s][o] * next_belief[s];
                    evidence += posterior[s];
                }
                if evidence > 0.0 {
                    for p in &mut posterior {
                        *p /= evidence;
                    }
                } else {
                    let uniform = 1.0 / model.num_states as f64;
                    posterior.fill(uniform);
                }

                let posterior_entropy = calc.compute_entropy(&posterior);
                expected_posterior_entropy += predicted_obs[o] * posterior_entropy;
            }

            total_gain += prior_entropy - expected_posterior_entropy;
            current_belief = next_belief;
        }

        total_gain.max(0.0)
    }

    pub fn compute_expected_pragmatic_value(policy: &[usize], prior: &[f64]) -> f64 {
        if policy.is_empty() || prior.is_empty() {
            return 0.0;
        }

        let calc = FreeEnergyCalculator::new();
        let num_states = prior.len();

        let mut final_state = vec![0.0; num_states];
        if policy[0] < num_states {
            final_state[policy[0]] = 1.0;
        }

        for &action in policy.iter().skip(1) {
            let mut next = vec![0.0; num_states];
            if action < num_states {
                next[action] = 1.0;
            }
            final_state = next;
        }

        calc.compute_kl_divergence(&final_state, prior)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_model() -> GenerativeModel {
        let mut model = GenerativeModel::new(3, 2);
        model.transition_matrix = vec![
            vec![0.7, 0.2, 0.1],
            vec![0.3, 0.4, 0.3],
            vec![0.1, 0.2, 0.7],
        ];
        model.likelihood_matrix = vec![
            vec![0.9, 0.1],
            vec![0.5, 0.5],
            vec![0.1, 0.9],
        ];
        model
    }

    #[test]
    fn test_evaluate_single_policy() {
        let evaluator = PolicyEvaluator::new(3, 5);
        let model = make_test_model();
        let belief = vec![0.5, 0.3, 0.2];
        let policy = vec![0, 1, 2];
        let efe = evaluator.evaluate_policy(&policy, &belief, &model);
        assert!(efe.is_finite());
    }

    #[test]
    fn test_evaluate_multiple_policies() {
        let evaluator = PolicyEvaluator::new(3, 5);
        let model = make_test_model();
        let belief = vec![0.5, 0.3, 0.2];
        let policies = vec![
            vec![0, 0, 0],
            vec![1, 1, 1],
            vec![2, 2, 2],
        ];
        let results = evaluator.evaluate_policies(&policies, &belief, &model);
        assert_eq!(results.len(), 3);
        for (idx, efe) in &results {
            assert!(efe.is_finite());
            assert!(*idx < 3);
        }
    }

    #[test]
    fn test_select_best_policy() {
        let evaluator = PolicyEvaluator::new(3, 5);
        let model = make_test_model();
        let belief = vec![0.5, 0.3, 0.2];
        let policies = vec![
            vec![0, 0, 0],
            vec![1, 1, 1],
            vec![2, 2, 2],
        ];
        let (best_idx, _best_efe) = evaluator.select_best_policy(&policies, &belief, &model);
        assert!(best_idx < 3);
    }

    #[test]
    fn test_expected_information_gain() {
        let model = make_test_model();
        let belief = vec![0.5, 0.3, 0.2];
        let policy = vec![0, 1];
        let gain = PolicyEvaluator::compute_expected_information_gain(&policy, &belief, &model);
        assert!(gain.is_finite());
        assert!(gain >= 0.0);
    }

    #[test]
    fn test_expected_pragmatic_value() {
        let prior = vec![0.5, 0.5];
        let policy = vec![0, 0];
        let value = PolicyEvaluator::compute_expected_pragmatic_value(&policy, &prior);
        assert!(value.is_finite());
        assert!(value >= 0.0);
    }

    #[test]
    fn test_pragmatic_value_reaches_prior() {
        let prior = vec![0.3, 0.7];
        let policy = vec![1];
        let value = PolicyEvaluator::compute_expected_pragmatic_value(&policy, &prior);
        assert!(value.is_finite());
    }

    #[test]
    fn test_empty_policy_efe() {
        let evaluator = PolicyEvaluator::new(3, 1);
        let model = make_test_model();
        let belief = vec![0.5, 0.3, 0.2];
        let efe = evaluator.evaluate_policy(&[], &belief, &model);
        assert!((efe).abs() < 1e-10);
    }

    #[test]
    fn test_information_gain_deterministic() {
        let mut model = GenerativeModel::new(2, 2);
        model.transition_matrix = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        model.likelihood_matrix = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let belief = vec![1.0, 0.0];
        let policy = vec![0];
        let gain = PolicyEvaluator::compute_expected_information_gain(&policy, &belief, &model);
        assert!(gain.is_finite());
        assert!(gain >= 0.0);
    }
}
