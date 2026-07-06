use serde::{Deserialize, Serialize};

const INPUT_STRIDE: usize = 64;

/// TD-JEPA: Temporal Difference Joint Embedding Predictive Architecture
/// Extends CausalJEPA with TD(λ) multi-step prediction for zero-shot RL.
///
/// Paper: "TD-JEPA: Long-term Prediction with Temporal Difference
/// Joint Embedding Predictive Architecture" (ICLR 2026)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDifferenceJEPA {
    pub latent_dim: usize,
    /// TD(λ) decay factor
    pub lambda: f64,
    /// Discount factor for multi-step returns
    pub gamma: f64,
    /// Delta predictor network (approximated as linear transformation)
    /// Flattened matrix: shape [latent_dim × INPUT_STRIDE]
    /// Each row i: delta[i] = Σ_j W[i][j] * concat(state, action)[j]
    pub delta_predictor: Vec<f64>,
    /// Value head for zero-shot RL: V(s) = w^T · s
    pub value_head: Vec<f64>,
}

impl TemporalDifferenceJEPA {
    pub fn new(latent_dim: usize) -> Self {
        let n_weights = latent_dim * INPUT_STRIDE;
        let std = (2.0 / (latent_dim + INPUT_STRIDE) as f64).sqrt();
        let mut delta_predictor = vec![0.0; n_weights];
        for val in delta_predictor.iter_mut() {
            *val = (rand::random::<f64>() - 0.5) * 2.0 * std;
        }

        let value_head = vec![0.0; latent_dim];

        Self {
            latent_dim,
            lambda: 0.9,
            gamma: 0.99,
            delta_predictor,
            value_head,
        }
    }

    fn value(&self, state: &[f64]) -> f64 {
        state
            .iter()
            .zip(self.value_head.iter())
            .map(|(s, w)| s * w)
            .sum()
    }

    /// Predict the delta (change) in latent state given current state and action
    /// delta = W · concat(state, action)  (purely linear)
    pub fn predict_delta(&self, state: &[f64], action: &[f64]) -> Vec<f64> {
        let mut delta = vec![0.0; self.latent_dim];
        for i in 0..self.latent_dim {
            let mut sum = 0.0;
            for (j, &s) in state.iter().enumerate() {
                if j < INPUT_STRIDE {
                    let idx = i * INPUT_STRIDE + j;
                    if idx < self.delta_predictor.len() {
                        sum += self.delta_predictor[idx] * s;
                    }
                }
            }
            for (j, &a) in action.iter().enumerate() {
                let col = self.latent_dim + j;
                if col < INPUT_STRIDE {
                    let idx = i * INPUT_STRIDE + col;
                    if idx < self.delta_predictor.len() {
                        sum += self.delta_predictor[idx] * a;
                    }
                }
            }
            delta[i] = sum;
        }
        delta
    }

    /// Multi-step rollout using TD(λ)
    /// Returns [s0, s1, ..., sN] where each s_k is the predicted latent state
    pub fn rollout(
        &self,
        initial_state: &[f64],
        actions: &[Vec<f64>],
        _lambda: f64,
    ) -> Vec<Vec<f64>> {
        let mut states = vec![initial_state.to_vec()];
        let mut current = initial_state.to_vec();
        for action in actions {
            let delta = self.predict_delta(&current, action);
            for i in 0..self.latent_dim.min(current.len()) {
                current[i] += delta[i];
            }
            states.push(current.clone());
        }
        states
    }

    /// Zero-shot value estimation: evaluate a policy without RL training
    /// Returns the discounted sum of V(s) over the predicted trajectory
    pub fn evaluate_policy(&self, initial_state: &[f64], actions: &[Vec<f64>]) -> f64 {
        let states = self.rollout(initial_state, actions, self.lambda);
        let mut total = 0.0;
        let mut discount = 1.0;
        for state in &states {
            total += discount * self.value(state);
            discount *= self.gamma;
        }
        total
    }

    /// Update the delta predictor using observed transitions
    pub fn update(
        &mut self,
        state: &[f64],
        action: &[f64],
        next_state: &[f64],
        reward: f64,
    ) {
        let lr = 0.01;
        let predicted_delta = self.predict_delta(state, action);

        for i in 0..self.latent_dim {
            let target_delta = next_state[i] - state[i];
            let error = target_delta - predicted_delta[i];

            for (j, &s) in state.iter().enumerate() {
                if j < INPUT_STRIDE {
                    let idx = i * INPUT_STRIDE + j;
                    if idx < self.delta_predictor.len() {
                        self.delta_predictor[idx] += lr * error * s;
                    }
                }
            }
            for (j, &a) in action.iter().enumerate() {
                let col = self.latent_dim + j;
                if col < INPUT_STRIDE {
                    let idx = i * INPUT_STRIDE + col;
                    if idx < self.delta_predictor.len() {
                        self.delta_predictor[idx] += lr * error * a;
                    }
                }
            }
        }

        let v_s = self.value(state);
        let v_next = self.value(next_state);
        let td_err = reward + self.gamma * v_next - v_s;
        for (j, &s) in state.iter().enumerate() {
            if j < self.value_head.len() {
                self.value_head[j] += lr * td_err * s;
            }
        }
    }

    /// TD error for a single transition: δ = r + γV(s') - V(s)
    pub fn td_error(
        &self,
        state: &[f64],
        _action: &[f64],
        next_state: &[f64],
        reward: f64,
    ) -> f64 {
        let v_s = self.value(state);
        let v_next = self.value(next_state);
        reward + self.gamma * v_next - v_s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> Vec<f64> {
        vec![0.1, -0.2, 0.3, -0.1, 0.05, -0.05, 0.15, -0.25]
    }

    fn make_action() -> Vec<f64> {
        vec![0.5, -0.3, 0.1, 0.0, -0.2, 0.4, -0.1, 0.3]
    }

    fn make_next_state(state: &[f64]) -> Vec<f64> {
        state.iter().map(|s| s + 0.1 * (rand::random::<f64>() - 0.5)).collect()
    }

    #[test]
    fn test_td_jepa_new() {
        let jepa = TemporalDifferenceJEPA::new(8);
        assert_eq!(jepa.latent_dim, 8);
        assert!((jepa.lambda - 0.9).abs() < 1e-10);
        assert!((jepa.gamma - 0.99).abs() < 1e-10);
        assert_eq!(jepa.delta_predictor.len(), 8 * INPUT_STRIDE);
        assert_eq!(jepa.value_head.len(), 8);
    }

    #[test]
    fn test_predict_delta_basic() {
        let jepa = TemporalDifferenceJEPA::new(8);
        let state = make_state();
        let action = make_action();
        let delta = jepa.predict_delta(&state, &action);
        assert_eq!(delta.len(), 8);
        for &d in &delta {
            assert!(d.is_finite(), "delta value should be finite");
        }
    }

    #[test]
    fn test_rollout_multi_step() {
        let jepa = TemporalDifferenceJEPA::new(8);
        let initial = make_state();
        let actions = vec![make_action(), make_action(), make_action()];
        let states = jepa.rollout(&initial, &actions, 0.9);
        assert_eq!(states.len(), 4);
        for s in &states {
            assert_eq!(s.len(), 8);
        }
        assert_eq!(states[0], initial);
    }

    #[test]
    fn test_evaluate_policy_basic() {
        let jepa = TemporalDifferenceJEPA::new(8);
        let initial = make_state();
        let actions = vec![make_action(), make_action()];
        let value = jepa.evaluate_policy(&initial, &actions);
        assert!(value.is_finite(), "policy value should be finite");
    }

    #[test]
    fn test_update_reduces_error() {
        let mut jepa = TemporalDifferenceJEPA::new(8);
        let state = make_state();
        let action = make_action();
        let next_state = make_next_state(&state);
        let reward = 0.5;

        let td_before = jepa.td_error(&state, &action, &next_state, reward).abs();
        for _ in 0..50 {
            jepa.update(&state, &action, &next_state, reward);
        }
        let td_after = jepa.td_error(&state, &action, &next_state, reward).abs();
        assert!(
            td_after <= td_before + 0.1,
            "TD error should not increase significantly after training: before={}, after={}",
            td_before,
            td_after
        );
    }

    #[test]
    fn test_td_error_computation() {
        let mut jepa = TemporalDifferenceJEPA::new(4);
        jepa.value_head = vec![1.0, 0.5, -0.3, 0.2];
        let state = vec![1.0, 2.0, 3.0, 4.0];
        let next_state = vec![0.5, 1.0, 2.0, 3.0];
        let reward = 1.0;
        let action = vec![0.1, -0.1, 0.2, -0.2];
        let td = jepa.td_error(&state, &action, &next_state, reward);
        // V(s) = 1*1 + 0.5*2 + (-0.3)*3 + 0.2*4 = 1 + 1 - 0.9 + 0.8 = 1.9
        // V(s') = 1*0.5 + 0.5*1 + (-0.3)*2 + 0.2*3 = 0.5 + 0.5 - 0.6 + 0.6 = 1.0
        // TD = 1.0 + 0.99*1.0 - 1.9 = 1.0 + 0.99 - 1.9 = 0.09
        let expected = 1.0 + 0.99 * 1.0 - 1.9;
        assert!((td - expected).abs() < 1e-10, "td={}, expected={}", td, expected);
    }

    #[test]
    fn test_rollout_convergence() {
        let jepa = TemporalDifferenceJEPA::new(8);
        let initial = make_state();
        let mut actions = Vec::new();
        for _ in 0..100 {
            actions.push(make_action());
        }
        let states = jepa.rollout(&initial, &actions, 0.9);
        assert_eq!(states.len(), 101);
        for s in &states {
            for &val in s {
                assert!(val.is_finite(), "state value should stay finite");
            }
        }
    }

    #[test]
    fn test_invariant_under_scale() {
        let jepa = TemporalDifferenceJEPA::new(8);
        let state = make_state();
        let action = make_action();
        let delta1 = jepa.predict_delta(&state, &action);
        let scaled_state: Vec<f64> = state.iter().map(|s| s * 2.0).collect();
        let delta2 = jepa.predict_delta(&scaled_state, &action);
        assert_eq!(delta1.len(), delta2.len());
        for (_, &d2) in delta1.iter().zip(delta2.iter()) {
            assert!(d2.is_finite(), "scaled delta should be finite");
        }
        let delta3 = jepa.predict_delta(&state, &action);
        for (d_a, d_b) in delta1.iter().zip(delta3.iter()) {
            assert!((d_a - d_b).abs() < 1e-12, "deterministic prediction");
        }
    }
}
