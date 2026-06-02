//! Temporal Difference Flows (Farebrother, Pirotta, Tirinzoni et al., ICLR 2026 Oral)
//!
//! Extends TD learning to continuous-time flow matching.
//! The flow formulation enables prediction at ANY continuous time horizon.

use serde::{Deserialize, Serialize};

/// Configuration for Temporal Difference Flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDFlowsConfig {
    pub time_horizon: f64,
    pub n_time_steps: usize,
    pub gamma: f64,
    pub flow_smoothness: f64,
}

impl Default for TDFlowsConfig {
    fn default() -> Self {
        Self {
            time_horizon: 10.0,
            n_time_steps: 50,
            gamma: 0.95,
            flow_smoothness: 0.01,
        }
    }
}

/// A state in the continuous-time flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowState {
    pub value: Vec<f32>,
    pub time: f64,
    pub flow: Vec<f32>,
}

/// Temporal Difference Flows — continuous-time TD learning via flow matching
///
/// flow_field: Vec<Vec<Vec<f32>>> indexed by [time_idx][state_dim][1]
///   flow_field[t][d][0] = learned flow for dimension d at time step t.
///   Third dimension is reserved for future state-dependent expansion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDifferenceFlows {
    pub config: TDFlowsConfig,
    pub flow_field: Vec<Vec<Vec<f32>>>,
    pub value_function: Vec<f32>,
    state_dim: usize,
}

impl TemporalDifferenceFlows {
    pub fn new(config: TDFlowsConfig, state_dim: usize) -> Self {
        let n = config.n_time_steps;
        let mut flow_field = Vec::with_capacity(n);
        for _ in 0..n {
            let mut time_slice = Vec::with_capacity(state_dim);
            for _ in 0..state_dim {
                let w: f32 = (rand::random::<f32>() - 0.5) * 0.1;
                time_slice.push(vec![w]);
            }
            flow_field.push(time_slice);
        }
        let value_function: Vec<f32> = (0..state_dim)
            .map(|_| (rand::random::<f32>() - 0.5) * 0.1)
            .collect();

        Self { config, flow_field, value_function, state_dim }
    }

    pub fn flow(&self, _state: &[f32], time: f64) -> Vec<f32> {
        let n = self.config.n_time_steps;
        if n == 0 || self.state_dim == 0 {
            return vec![0.0; self.state_dim];
        }
        let dt = self.config.time_horizon / n as f64;
        if dt == 0.0 {
            return vec![0.0; self.state_dim];
        }
        let t = time.clamp(0.0, self.config.time_horizon);
        let t_idx = t / dt;
        let idx = (t_idx.floor() as usize).min(n.saturating_sub(1));
        let next_idx = (idx + 1).min(n.saturating_sub(1));
        let frac = (t_idx - idx as f64) as f32;
        let mut result = Vec::with_capacity(self.state_dim);
        for d in 0..self.state_dim {
            let v0 = self.flow_field[idx][d][0];
            let v1 = self.flow_field[next_idx][d][0];
            result.push(v0 + frac * (v1 - v0));
        }
        result
    }

    pub fn integrate(&self, initial_state: &[f32], from_time: f64, to_time: f64) -> Vec<f32> {
        let config = &self.config;
        if config.time_horizon <= 0.0 || config.n_time_steps == 0 {
            return initial_state.to_vec();
        }
        let dt = config.time_horizon / config.n_time_steps as f64;
        if dt == 0.0 || (from_time - to_time).abs() < 1e-12 {
            return initial_state.to_vec();
        }
        let mut state = initial_state.to_vec();
        let direction = if to_time > from_time { 1.0 } else { -1.0 };
        let dist = (to_time - from_time).abs();
        let n_steps = (dist / dt).ceil() as usize;
        let step_size = direction * dist / n_steps.max(1) as f64;
        let mut t = from_time;
        for _ in 0..n_steps {
            let flow_val = self.flow(&state, t);
            for d in 0..self.state_dim {
                state[d] += flow_val[d] * step_size as f32;
            }
            t += step_size;
        }
        state
    }

    pub fn learn_transition(&mut self, state_before: &[f32], state_after: &[f32], dt: f64) {
        let config = &self.config;
        if dt == 0.0 || config.n_time_steps == 0 || config.time_horizon <= 0.0 {
            return;
        }
        let target_flow: Vec<f32> = state_before
            .iter()
            .zip(state_after.iter())
            .map(|(b, a)| (a - b) / dt as f32)
            .collect();
        // Distribute across ALL time indices up to the learned time
        let t_idx = (dt / config.time_horizon * config.n_time_steps as f64) as usize;
        let idx = t_idx.min(config.n_time_steps.saturating_sub(1));
        let lr = 0.01_f32;
        for i in 0..=idx {
            for d in 0..self.state_dim {
                let error = target_flow[d] - self.flow_field[i][d][0];
                self.flow_field[i][d][0] += lr * error;
            }
        }
    }

    pub fn predict_at_time(&self, initial: &[f32], target_time: f64) -> Vec<f32> {
        if target_time <= 0.0 {
            return initial.to_vec();
        }
        self.integrate(initial, 0.0, target_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: &[f32], b: &[f32], eps: f32) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < eps)
    }

    #[test]
    fn test_zero_flow_state_unchanged() {
        let config = TDFlowsConfig {
            n_time_steps: 10,
            time_horizon: 10.0,
            ..Default::default()
        };
        let mut tdf = TemporalDifferenceFlows::new(config, 4);
        for d in 0..4 {
            for t in 0..10 {
                tdf.flow_field[t][d][0] = 0.0;
            }
        }
        let state = vec![1.0, 2.0, 3.0, 4.0];
        let result = tdf.integrate(&state, 0.0, 5.0);
        assert!(approx_eq(&result, &state, 1e-6), "zero flow should keep state unchanged");
    }

    #[test]
    fn test_constant_positive_flow_linear_increase() {
        let config = TDFlowsConfig {
            n_time_steps: 1,
            time_horizon: 1.0,
            ..Default::default()
        };
        let mut tdf = TemporalDifferenceFlows::new(config, 2);
        tdf.flow_field[0][0][0] = 1.0;
        tdf.flow_field[0][1][0] = 2.0;
        let state = vec![0.0, 0.0];
        let result = tdf.integrate(&state, 0.0, 1.0);
        assert!((result[0] - 1.0).abs() < 0.1, "dim0 should increase to ~1.0, got {}", result[0]);
        assert!((result[1] - 2.0).abs() < 0.2, "dim1 should increase to ~2.0, got {}", result[1]);
    }

    #[test]
    fn test_flow_integration_reverse_returns_to_start() {
        let config = TDFlowsConfig {
            n_time_steps: 10,
            time_horizon: 10.0,
            ..Default::default()
        };
        let mut tdf = TemporalDifferenceFlows::new(config, 2);
        for t in 0..10 {
            tdf.flow_field[t][0][0] = 0.5;
            tdf.flow_field[t][1][0] = -0.3;
        }
        let state = vec![10.0, 20.0];
        let forward = tdf.integrate(&state, 0.0, 5.0);
        let back = tdf.integrate(&forward, 5.0, 0.0);
        assert!(approx_eq(&state, &back, 0.5), "reverse integration should return to start");
    }

    #[test]
    fn test_learn_transition_predicts_direction() {
        let config = TDFlowsConfig {
            n_time_steps: 5,
            time_horizon: 1.0,
            ..Default::default()
        };
        let mut tdf = TemporalDifferenceFlows::new(config, 2);
        // Zero out flow field to remove random initialization noise
        for t in 0..tdf.config.n_time_steps {
            for d in 0..2 {
                tdf.flow_field[t][d][0] = 0.0;
            }
        }
        let before = vec![0.0, 0.0];
        let after = vec![1.0, 2.0];
        // Multiple updates to ensure positive flow
        for _ in 0..5 {
            tdf.learn_transition(&before, &after, 1.0);
        }
        let predicted = tdf.predict_at_time(&before, 1.0);
        assert!(predicted[0] > 0.0, "should learn positive flow for dim0, got {}", predicted[0]);
        assert!(predicted[1] > 0.0, "should learn positive flow for dim1, got {}", predicted[1]);
    }

    #[test]
    fn test_predict_at_time_varying_horizons() {
        let config = TDFlowsConfig {
            n_time_steps: 20,
            time_horizon: 10.0,
            ..Default::default()
        };
        let mut tdf = TemporalDifferenceFlows::new(config, 3);
        for t in 0..20 {
            tdf.flow_field[t][0][0] = 1.0;
            tdf.flow_field[t][1][0] = 0.5;
            tdf.flow_field[t][2][0] = -0.5;
        }
        let init = vec![0.0; 3];
        let t1 = tdf.predict_at_time(&init, 1.0);
        let t2 = tdf.predict_at_time(&init, 2.0);
        assert!(t1[0] > 0.0 && t2[0] > 0.0, "both horizons should move in flow direction");
        assert!(t2[0] > t1[0], "longer horizon should be further");
    }

    #[test]
    fn test_empty_zero_state() {
        let config = TDFlowsConfig::default();
        let tdf = TemporalDifferenceFlows::new(config, 0);
        let result = tdf.integrate(&[], 0.0, 5.0);
        assert!(result.is_empty(), "zero-dim state should produce empty result");
        let flow_val = tdf.flow(&[], 0.5);
        assert!(flow_val.is_empty(), "zero-dim flow should be empty");
    }

    #[test]
    fn test_zero_time_horizon() {
        let config = TDFlowsConfig {
            time_horizon: 0.0,
            n_time_steps: 10,
            ..Default::default()
        };
        let tdf = TemporalDifferenceFlows::new(config, 3);
        let state = vec![1.0, 2.0, 3.0];
        let result = tdf.predict_at_time(&state, 5.0);
        assert!(approx_eq(&result, &state, 1e-6), "zero horizon should keep state unchanged");
    }

    #[test]
    fn test_negative_time_returns_initial() {
        let config = TDFlowsConfig::default();
        let tdf = TemporalDifferenceFlows::new(config, 3);
        let state = vec![5.0, 6.0, 7.0];
        let result = tdf.predict_at_time(&state, -1.0);
        assert!(approx_eq(&result, &state, 1e-6), "negative time should return initial state");
    }

    #[test]
    fn test_reverse_integration() {
        let config = TDFlowsConfig {
            n_time_steps: 5,
            time_horizon: 5.0,
            ..Default::default()
        };
        let mut tdf = TemporalDifferenceFlows::new(config, 2);
        for t in 0..5 {
            tdf.flow_field[t][0][0] = 1.0;
            tdf.flow_field[t][1][0] = 2.0;
        }
        let state = vec![10.0, 20.0];
        let rev = tdf.integrate(&state, 3.0, 1.0);
        assert!(rev[0] < state[0], "reverse integration should go backward on dim0");
        assert!(rev[1] < state[1], "reverse integration should go backward on dim1");
    }

    #[test]
    fn test_from_time_equals_to_time() {
        let config = TDFlowsConfig::default();
        let tdf = TemporalDifferenceFlows::new(config, 3);
        let state = vec![1.0, 2.0, 3.0];
        let result = tdf.integrate(&state, 2.0, 2.0);
        assert!(approx_eq(&result, &state, 1e-6));
    }
}
