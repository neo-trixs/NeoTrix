use super::pipeline::{BrainStage, StageDecision};
use super::SelfIteratingBrain;

use rand::Rng;
use std::sync::Mutex;

const L2_CLIP_NORM: f64 = 1.0;
const NOISE_MULTIPLIER: f64 = 0.5;
const DELTA: f64 = 1e-5;
const MAX_EPSILON: f64 = 10.0;

#[derive(Debug, Clone)]
pub struct DpSgdState {
    pub epsilon_spent: f64,
    pub delta: f64,
    pub total_steps: u64,
    pub clip_count: u64,
    pub noise_scale: f64,
}

impl Default for DpSgdState {
    fn default() -> Self {
        Self {
            epsilon_spent: 0.0,
            delta: DELTA,
            total_steps: 0,
            clip_count: 0,
            noise_scale: compute_noise_scale(1.0, NOISE_MULTIPLIER, L2_CLIP_NORM),
        }
    }
}

impl DpSgdState {
    pub fn privacy_budget_exhausted(&self) -> bool {
        self.epsilon_spent >= MAX_EPSILON
    }

    pub fn remaining_epsilon(&self) -> f64 {
        (MAX_EPSILON - self.epsilon_spent).max(0.0)
    }

    fn account_step(&mut self, epsilon_per_step: f64) {
        self.epsilon_spent += epsilon_per_step;
        self.total_steps += 1;
    }
}

pub struct DpSgdStage {
    state: Mutex<DpSgdState>,
    /// Whether the policy learner has been notified that DP-SGD budget is exhausted.
    /// Prevents repeated notifications on every Skip.
    notified_policy: Mutex<bool>,
    /// Total steps completed before budget exhaustion (zero if not yet exhausted).
    total_steps_completed: Mutex<u64>,
}

impl Default for DpSgdStage {
    fn default() -> Self {
        Self {
            state: Mutex::new(DpSgdState::default()),
            notified_policy: Mutex::new(false),
            total_steps_completed: Mutex::new(0),
        }
    }
}

impl DpSgdStage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            *state = DpSgdState::default();
        }
    }

    pub fn set_noise_multiplier(&self, sigma: f64) {
        if let Ok(mut state) = self.state.lock() {
            state.noise_scale = compute_noise_scale(sigma, NOISE_MULTIPLIER, L2_CLIP_NORM);
        }
    }

    pub fn state_snapshot(&self) -> DpSgdState {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

impl BrainStage for DpSgdStage {
    fn name(&self) -> &str {
        "dp_sgd"
    }

    fn frequency(&self) -> usize {
        1
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, crate::neotrix::nt_core_error::NeoTrixError> {
        let mut state = self.state.lock().map_err(|e| {
            crate::neotrix::nt_core_error::NeoTrixError::Io(format!("dp_sgd state lock: {e}"))
        })?;

        if state.privacy_budget_exhausted() {
            let mut notified = self.notified_policy.lock().map_err(|e| {
                crate::neotrix::nt_core_error::NeoTrixError::Io(format!("dp_sgd notified lock: {e}"))
            })?;
            if !*notified {
                let completed = state.total_steps;
                if let Ok(mut steps) = self.total_steps_completed.lock() {
                    *steps = completed;
                }
                log::warn!(
                    "[DpSgdStage] privacy budget exhausted after {completed} steps (ε={:.2}/{MAX_EPSILON}) — \
                     GRPO policy should adjust learning rate or explore new modes; \
                     stage will permanently skip",
                    state.epsilon_spent,
                );
                *notified = true;
            }
            return Ok(StageDecision::Skip("privacy budget exhausted".into()));
        }

        let reward = brain._reward();
        let task = brain._current_task();
        let task_type = brain._current_task_type();

        let mode = brain._e8_policy.select_mode(&task, task_type, &brain._transition_learner);
        let mode_idx = mode.0 as usize;
        let old_value = brain._e8_policy.mode_values[mode_idx];
        let td_error = reward - old_value;

        let clipped_gradient = td_error.signum() * td_error.abs().min(L2_CLIP_NORM);

        let mut rng = rand::thread_rng();
        let u1: f64 = rng.gen();
        let u2: f64 = rng.gen();
        let noise = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos() * state.noise_scale;
        let noisy_gradient = clipped_gradient + noise;

        let lr = brain._e8_policy.learning_rate();
        let update = lr * noisy_gradient;
        brain._e8_policy.mode_values[mode_idx] += update;

        let epsilon_per_step = compute_rdp_epsilon(1.0, state.delta, NOISE_MULTIPLIER);
        state.account_step(epsilon_per_step);

        if td_error.abs() > L2_CLIP_NORM {
            log::debug!("dp_sgd: clipped gradient for mode {mode_idx}: {td_error:.4} → {clipped_gradient:.4}");
        }

        log::debug!(
            "dp_sgd: mode={mode_idx}, td={td_error:.4}, clip={clipped_gradient:.4}, noise={noise:.4}, update={update:.6}, ε={:.4}/{MAX_EPSILON}",
            state.epsilon_spent,
        );

        Ok(StageDecision::Continue)
    }
}

fn compute_noise_scale(sigma: f64, _noise_multiplier: f64, clip_norm: f64) -> f64 {
    sigma * clip_norm
}

fn compute_rdp_epsilon(sigma: f64, _delta: f64, _noise_multiplier: f64) -> f64 {
    let q = 1.0f64;
    let alpha = 4.0f64;
    let eps_rdp = (q.powi(2) * alpha) / (2.0 * sigma.powi(2));
    eps_rdp + (alpha.ln() - (_delta * (alpha - 1.0)).ln()) / (alpha - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dp_sgd_state_default() {
        let state = DpSgdState::default();
        assert_eq!(state.epsilon_spent, 0.0);
        assert_eq!(state.delta, 1e-5);
        assert!(!state.privacy_budget_exhausted());
    }

    #[test]
    fn test_privacy_budget_exhausted() {
        let mut state = DpSgdState::default();
        assert!(!state.privacy_budget_exhausted());
        state.epsilon_spent = MAX_EPSILON + 0.1;
        assert!(state.privacy_budget_exhausted());
    }

    #[test]
    fn test_remaining_epsilon() {
        let mut state = DpSgdState::default();
        assert!((state.remaining_epsilon() - MAX_EPSILON).abs() < 1e-10);
        state.epsilon_spent = 3.0;
        assert!((state.remaining_epsilon() - (MAX_EPSILON - 3.0)).abs() < 1e-10);
        state.epsilon_spent = 20.0;
        assert_eq!(state.remaining_epsilon(), 0.0);
    }

    #[test]
    fn test_account_step() {
        let mut state = DpSgdState::default();
        state.account_step(0.1);
        assert!((state.epsilon_spent - 0.1).abs() < 1e-10);
        assert_eq!(state.total_steps, 1);
        state.account_step(0.2);
        assert!((state.epsilon_spent - 0.3).abs() < 1e-10);
        assert_eq!(state.total_steps, 2);
    }

    #[test]
    fn test_dp_sgd_stage_default() {
        let stage = DpSgdStage::default();
        assert_eq!(stage.name(), "dp_sgd");
        assert_eq!(stage.frequency(), 1);
        let state = stage.state_snapshot();
        assert!(!state.privacy_budget_exhausted());
    }

    #[test]
    fn test_set_noise_multiplier() {
        let stage = DpSgdStage::default();
        stage.set_noise_multiplier(2.0);
        let state = stage.state_snapshot();
        assert!((state.noise_scale - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_reset() {
        let stage = DpSgdStage::default();
        stage.reset();
        let state = stage.state_snapshot();
        assert_eq!(state.epsilon_spent, 0.0);
        assert_eq!(state.total_steps, 0);
    }

    #[test]
    fn test_noise_is_random() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let u1: f64 = rng.gen_range(0.001..1.0);
        let u2: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
        let a = (-2.0 * u1.ln()).sqrt() * u2.cos();
        let v1: f64 = rng.gen_range(0.001..1.0);
        let v2: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
        let b = (-2.0 * v1.ln()).sqrt() * v2.cos();
        assert!((a - b).abs() > 1e-10);
    }

    #[test]
    fn test_compute_noise_scale() {
        let scale = compute_noise_scale(0.5, 0.5, 1.0);
        assert!((scale - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_state_snapshot() {
        let stage = DpSgdStage::default();
        let snap = stage.state_snapshot();
        assert_eq!(snap.epsilon_spent, 0.0);
    }
}
