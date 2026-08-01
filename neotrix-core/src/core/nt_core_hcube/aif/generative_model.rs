#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerativeModel {
    pub num_states: usize,
    pub num_observations: usize,
    pub transition_matrix: Vec<Vec<f64>>,
    pub likelihood_matrix: Vec<Vec<f64>>,
    pub prior_over_states: Vec<f64>,
    pub prior_over_policies: Vec<f64>,
}

impl GenerativeModel {
    pub fn new(num_states: usize, num_observations: usize) -> Self {
        // 防止 num_states/num_observations 为 0 时的除零与空矩阵
        let states = num_states.max(1);
        let observations = num_observations.max(1);
        let uniform_state = 1.0 / states as f64;
        let transition_matrix = vec![vec![uniform_state; states]; states];
        let uniform_obs = 1.0 / observations as f64;
        let likelihood_matrix = vec![vec![uniform_obs; observations]; states];
        let prior_over_states = vec![uniform_state; states];
        let prior_over_policies = vec![];

        GenerativeModel {
            num_states: states,
            num_observations: observations,
            transition_matrix,
            likelihood_matrix,
            prior_over_states,
            prior_over_policies,
        }
    }

    pub fn transition_prob(&self, from: usize, to: usize) -> f64 {
        if from < self.num_states && to < self.num_states {
            self.transition_matrix[from][to]
        } else {
            0.0
        }
    }

    pub fn observation_likelihood(&self, state: usize, obs: usize) -> f64 {
        if state < self.num_states && obs < self.num_observations {
            self.likelihood_matrix[state][obs]
        } else {
            0.0
        }
    }

    pub fn predict_next_state(&self, current: &[f64]) -> Vec<f64> {
        let mut next = vec![0.0; self.num_states];
        for i in 0..self.num_states {
            for j in 0..self.num_states {
                next[j] += current[i] * self.transition_matrix[i][j];
            }
        }
        next
    }

    pub fn predict_observation(&self, state: &[f64]) -> Vec<f64> {
        let mut obs = vec![0.0; self.num_observations];
        for s in 0..self.num_states {
            for o in 0..self.num_observations {
                obs[o] += state[s] * self.likelihood_matrix[s][o];
            }
        }
        obs
    }

    pub fn update_transition(&mut self, from: usize, to: usize, delta: f64) {
        if from >= self.num_states || to >= self.num_states {
            return;
        }
        self.transition_matrix[from][to] = (self.transition_matrix[from][to] + delta).max(0.0);
        let row_sum: f64 = self.transition_matrix[from].iter().sum();
        if row_sum > 0.0 {
            for j in 0..self.num_states {
                self.transition_matrix[from][j] /= row_sum;
            }
        }
    }

    pub fn normalize(&mut self) {
        for i in 0..self.num_states {
            let row_sum: f64 = self.transition_matrix[i].iter().sum();
            if row_sum > 0.0 {
                for j in 0..self.num_states {
                    self.transition_matrix[i][j] /= row_sum;
                }
            }
            let row_sum_l: f64 = self.likelihood_matrix[i].iter().sum();
            if row_sum_l > 0.0 {
                for j in 0..self.num_observations {
                    self.likelihood_matrix[i][j] /= row_sum_l;
                }
            }
        }
        let state_sum: f64 = self.prior_over_states.iter().sum();
        if state_sum > 0.0 {
            for s in &mut self.prior_over_states {
                *s /= state_sum;
            }
        }
    }

    pub fn validate(&self) -> bool {
        let eps = 1e-6;
        for i in 0..self.num_states {
            let row_sum: f64 = self.transition_matrix[i].iter().sum();
            if (row_sum - 1.0).abs() > eps {
                return false;
            }
            let row_sum_l: f64 = self.likelihood_matrix[i].iter().sum();
            if (row_sum_l - 1.0).abs() > eps {
                return false;
            }
        }
        let state_sum: f64 = self.prior_over_states.iter().sum();
        if (state_sum - 1.0).abs() > eps {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_model() {
        let model = GenerativeModel::new(3, 2);
        assert_eq!(model.num_states, 3);
        assert_eq!(model.num_observations, 2);
        assert!(model.validate());
    }

    #[test]
    fn test_transition_prob() {
        let model = GenerativeModel::new(3, 2);
        let p = model.transition_prob(1, 2);
        assert!((p - 1.0 / 3.0).abs() < 1e-10);
        let out_of_bounds = model.transition_prob(5, 0);
        assert!((out_of_bounds).abs() < 1e-10);
    }

    #[test]
    fn test_observation_likelihood() {
        let model = GenerativeModel::new(3, 2);
        let p = model.observation_likelihood(1, 1);
        assert!((p - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_predict_next_state() {
        let model = GenerativeModel::new(3, 2);
        let current = vec![1.0, 0.0, 0.0];
        let next = model.predict_next_state(&current);
        assert_eq!(next.len(), 3);
        let sum: f64 = next.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_predict_observation() {
        let model = GenerativeModel::new(3, 2);
        let state = vec![1.0, 0.0, 0.0];
        let obs = model.predict_observation(&state);
        assert_eq!(obs.len(), 2);
        let sum: f64 = obs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_update_transition() {
        let mut model = GenerativeModel::new(3, 2);
        model.update_transition(0, 1, 5.0);
        model.update_transition(0, 0, -10.0);
        let row_sum: f64 = model.transition_matrix[0].iter().sum();
        assert!((row_sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize() {
        let mut model = GenerativeModel::new(3, 2);
        model.transition_matrix[0][0] = 100.0;
        model.transition_matrix[0][1] = 0.0;
        model.transition_matrix[0][2] = 0.0;
        model.normalize();
        assert!((model.transition_matrix[0][0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_validate_passes() {
        let model = GenerativeModel::new(2, 2);
        assert!(model.validate());
    }

    #[test]
    fn test_validate_fails() {
        let mut model = GenerativeModel::new(2, 2);
        model.transition_matrix[0][0] = 999.0;
        model.transition_matrix[0][1] = 999.0;
        assert!(!model.validate());
    }

    #[test]
    fn test_predict_next_state_deterministic() {
        let mut model = GenerativeModel::new(2, 2);
        model.transition_matrix[0] = vec![1.0, 0.0];
        model.transition_matrix[1] = vec![0.0, 1.0];
        let current = vec![0.5, 0.5];
        let next = model.predict_next_state(&current);
        assert!((next[0] - 0.5).abs() < 1e-10);
        assert!((next[1] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_update_transition_renormalize() {
        let mut model = GenerativeModel::new(2, 2);
        model.update_transition(0, 1, 0.3);
        let row_sum: f64 = model.transition_matrix[0].iter().sum();
        assert!((row_sum - 1.0).abs() < 1e-10);
    }
}
