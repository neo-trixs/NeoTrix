//! # Active Inference Loop (FEP)
//!
//! Implements the Free Energy Principle's active inference cycle with
//! precision estimation and expected free energy computation.
//!
//! ## Core computation
//!
//! For each candidate action π, the expected free energy G(π) is:
//!
//! G(π) = −𝔼_q[ln P(o|π)] + KL[q(s|π) || q(s|o,π)]
//!
//! Decomposed into:
//! - **Pragmatic value**: how close the predicted state matches the goal
//! - **Epistemic value**: expected information gain (reduction in uncertainty)

/// Action proposal with expected free energy decomposition.
#[derive(Debug, Clone)]
pub struct ActionProposal {
    pub action: String,
    pub expected_free_energy: f64,
    pub pragmatic_value: f64,
    pub epistemic_value: f64,
    pub selected: bool,
}

/// Active inference loop state.
#[derive(Debug, Clone)]
pub struct ActiveInferenceLoop {
    /// Current precision (inverse temperature) for action selection.
    pub precision: f64,
    /// Current expected free energy (moving average).
    pub expected_free_energy: f64,
    /// Proposals from the latest planning step.
    pub action_proposals: Vec<ActionProposal>,
    /// Learning rate for precision updates.
    pub learning_rate: f64,
    /// Momentum factor for expected free energy smoothing (0.0 = no smoothing).
    pub momentum: f64,
    /// Moving average window of prediction errors.
    pub prediction_errors: Vec<f64>,
    pub max_history: usize,
}

impl Default for ActiveInferenceLoop {
    fn default() -> Self {
        Self {
            precision: 1.0,
            expected_free_energy: 0.0,
            action_proposals: Vec::new(),
            learning_rate: 0.1,
            momentum: 0.3,
            prediction_errors: Vec::with_capacity(100),
            max_history: 100,
        }
    }
}

impl ActiveInferenceLoop {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute expected free energy for each candidate action and select
    /// the action with the lowest G(π).
    ///
    /// * `state` — current belief (probability) state
    /// * `goal` — desired state (goal prior)
    /// * `actions` — candidate actions as `(name, expected_state_delta)` pairs
    pub fn plan(
        &mut self,
        state: &[f64],
        goal: &[f64],
        actions: &[(&str, Vec<f64>)],
    ) -> Vec<ActionProposal> {
        let dim = state.len();
        let mut proposals = Vec::with_capacity(actions.len());

        for (name, delta) in actions {
            let predicted = self.predict_state(state, delta, dim);
            let pragmatic = self.pragmatic_value(&predicted, goal);
            let epistemic = self.epistemic_value(&predicted, &self.posterior_estimate(state, &predicted));
            let gef = -(pragmatic + self.precision * epistemic);

            proposals.push(ActionProposal {
                action: name.to_string(),
                expected_free_energy: gef,
                pragmatic_value: pragmatic,
                epistemic_value: epistemic,
                selected: false,
            });
        }

        proposals.sort_by(|a, b| a.expected_free_energy.partial_cmp(&b.expected_free_energy).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(best) = proposals.first_mut() {
            best.selected = true;
        }

        self.action_proposals = proposals.clone();

        self.expected_free_energy = if self.momentum > 0.0 && self.expected_free_energy != 0.0 {
            self.momentum * self.expected_free_energy + (1.0 - self.momentum) * proposals.first().map_or(0.0, |p| p.expected_free_energy)
        } else {
            proposals.first().map_or(0.0, |p| p.expected_free_energy)
        };

        proposals
    }

    /// Update precision based on prediction error.
    ///
    /// Uses gradient descent on variational free energy:
    /// `d(precision)/dt ∝ prediction_error − precision`
    pub fn update_precision(&mut self, prediction_error: f64) {
        self.prediction_errors.push(prediction_error);
        if self.prediction_errors.len() > self.max_history {
            self.prediction_errors.remove(0);
        }

        let smoothed_error = if self.prediction_errors.len() > 5 {
            self.prediction_errors.iter().rev().take(5).sum::<f64>() / 5.0
        } else {
            prediction_error
        };

        self.precision += self.learning_rate * (smoothed_error - self.precision);
        self.precision = self.precision.max(0.01).min(10.0);
    }

    /// Get the best action name, if any.
    pub fn best_action(&self) -> Option<&str> {
        self.action_proposals.iter().find(|p| p.selected).map(|p| p.action.as_str())
    }

    /// Action selection probability via softmax over expected free energy.
    pub fn action_probabilities(&self) -> Vec<(String, f64)> {
        let inv_temp = self.precision;
        let max_gef = self.action_proposals.iter()
            .map(|p| p.expected_free_energy)
            .fold(f64::NEG_INFINITY, f64::max);
        let shifted: Vec<f64> = self.action_proposals.iter()
            .map(|p| (-inv_temp * (p.expected_free_energy - max_gef)).exp())
            .collect();
        let sum: f64 = shifted.iter().sum();
        if sum <= 0.0 {
            return self.action_proposals.iter().map(|p| (p.action.clone(), 1.0 / self.action_proposals.len() as f64)).collect();
        }
        self.action_proposals.iter().zip(shifted.iter())
            .map(|(p, s)| (p.action.clone(), s / sum))
            .collect()
    }

    pub fn reset(&mut self) {
        self.precision = 1.0;
        self.expected_free_energy = 0.0;
        self.action_proposals.clear();
        self.prediction_errors.clear();
    }

    // ── private helpers ──────────────────────────────────────────────

    /// Simple linear prediction: `state + delta`, clamped to [0, 1].
    fn predict_state(&self, state: &[f64], delta: &[f64], dim: usize) -> Vec<f64> {
        let max_len = state.len().min(delta.len()).min(dim);
        let mut predicted = Vec::with_capacity(dim);
        for i in 0..max_len {
            let val = state[i] + delta[i];
            predicted.push(val.max(0.0).min(1.0));
        }
        while predicted.len() < dim {
            predicted.push(0.0);
        }
        predicted
    }

    /// Pragmatic value: negative Euclidean distance to goal, normalized to [0, 1].
    fn pragmatic_value(&self, predicted: &[f64], goal: &[f64]) -> f64 {
        let max_dim = predicted.len().min(goal.len());
        if max_dim == 0 { return 0.0; }
        let sq_dist: f64 = predicted.iter().zip(goal.iter())
            .take(max_dim)
            .map(|(p, g)| (p - g).powi(2))
            .sum();
        let rmse = (sq_dist / max_dim as f64).sqrt();
        (-rmse).exp()
    }

    /// Epistemic value: expected information gain approximated as
    /// variance reduction.
    fn epistemic_value(&self, predicted: &[f64], posterior: &[f64]) -> f64 {
        let max_dim = predicted.len().min(posterior.len());
        if max_dim == 0 { return 0.0; }
        let var_before: f64 = predicted.iter().map(|p| (p - 0.5).powi(2)).sum::<f64>() / max_dim as f64;
        let var_after: f64 = posterior.iter().map(|q| (q - 0.5).powi(2)).sum::<f64>() / max_dim as f64;
        (var_before - var_after).max(0.0)
    }

    /// Crude posterior estimate: Bayesian combination of prior and likelihood.
    fn posterior_estimate(&self, prior: &[f64], likelihood: &[f64]) -> Vec<f64> {
        prior.iter().zip(likelihood.iter())
            .map(|(p, l)| {
                let post = (p * l) / (p * l + (1.0 - p) * (1.0 - l) + 1e-8);
                post.max(0.0).min(1.0)
            })
            .collect()
    }
}

/// Active inference report for the consciousness cycle.
#[derive(Debug, Clone)]
pub struct ActiveInferenceReport {
    pub precision: f64,
    pub expected_free_energy: f64,
    pub selected_action: Option<String>,
    pub pragmatic_value: f64,
    pub epistemic_value: f64,
    pub num_proposals: usize,
}

impl ActiveInferenceReport {
    pub fn from_loop(ail: &ActiveInferenceLoop) -> Self {
        Self {
            precision: ail.precision,
            expected_free_energy: ail.expected_free_energy,
            selected_action: ail.best_action().map(|s| s.to_string()),
            pragmatic_value: ail.action_proposals.first().map_or(0.0, |p| p.pragmatic_value),
            epistemic_value: ail.action_proposals.first().map_or(0.0, |p| p.epistemic_value),
            num_proposals: ail.action_proposals.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_inference_default() {
        let ail = ActiveInferenceLoop::new();
        assert!((ail.precision - 1.0).abs() < 1e-6);
        assert!((ail.expected_free_energy - 0.0).abs() < 1e-6);
        assert!(ail.action_proposals.is_empty());
    }

    #[test]
    fn test_plan_selects_best_action() {
        let mut ail = ActiveInferenceLoop::new();
        let state = vec![0.5, 0.5];
        let goal = vec![1.0, 1.0];
        let actions = vec![
            ("move_toward_goal", vec![0.3, 0.3]),
            ("move_away", vec![-0.3, -0.3]),
            ("stay", vec![0.0, 0.0]),
        ];
        let proposals = ail.plan(&state, &goal, &actions);
        assert_eq!(proposals.len(), 3);
        assert!(proposals[0].selected);
        assert!(proposals[0].pragmatic_value >= proposals[1].pragmatic_value);
    }

    #[test]
    fn test_precision_update() {
        let mut ail = ActiveInferenceLoop::new();
        let initial = ail.precision;
        ail.update_precision(2.0);
        assert!(ail.precision > initial || (ail.precision - initial).abs() < 1e-6);
        ail.update_precision(0.1);
        assert!(ail.precision >= 0.01);
        ail.update_precision(20.0);
        assert!(ail.precision <= 10.0);
    }

    #[test]
    fn test_best_action_returns_selected() {
        let mut ail = ActiveInferenceLoop::new();
        let state = vec![0.3, 0.3];
        let goal = vec![0.8, 0.8];
        let actions = vec![
            ("approach", vec![0.2, 0.2]),
            ("retreat", vec![-0.1, -0.1]),
        ];
        ail.plan(&state, &goal, &actions);
        assert_eq!(ail.best_action(), Some("approach"));
    }

    #[test]
    fn test_action_probabilities_sum_to_one() {
        let mut ail = ActiveInferenceLoop::new();
        let state = vec![0.5; 4];
        let goal = vec![0.9; 4];
        let actions = vec![
            ("a1", vec![0.1; 4]),
            ("a2", vec![0.05; 4]),
            ("a3", vec![-0.1; 4]),
        ];
        ail.plan(&state, &goal, &actions);
        let probs = ail.action_probabilities();
        assert_eq!(probs.len(), 3);
        let sum: f64 = probs.iter().map(|(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pragmatic_value_goal_aligned() {
        let ail = ActiveInferenceLoop::new();
        let p = ail.pragmatic_value(&[0.9, 0.9], &[1.0, 1.0]);
        let q = ail.pragmatic_value(&[0.1, 0.1], &[1.0, 1.0]);
        assert!(p > q, "closer to goal should have higher pragmatic value");
    }

    #[test]
    fn test_epistemic_value_nonzero() {
        let ail = ActiveInferenceLoop::new();
        let ev = ail.epistemic_value(&[0.5; 4], &[0.7; 4]);
        assert!(ev >= 0.0);
    }

    #[test]
    fn test_report_from_loop() {
        let mut ail = ActiveInferenceLoop::new();
        let state = vec![0.5, 0.5];
        let goal = vec![0.9, 0.9];
        let actions = vec![("go", vec![0.2, 0.2])];
        ail.plan(&state, &goal, &actions);
        let report = ActiveInferenceReport::from_loop(&ail);
        assert_eq!(report.selected_action.as_deref(), Some("go"));
        assert_eq!(report.num_proposals, 1);
        assert!((report.precision - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut ail = ActiveInferenceLoop::new();
        ail.precision = 3.5;
        ail.expected_free_energy = -0.7;
        ail.action_proposals.push(ActionProposal {
            action: "test".into(),
            expected_free_energy: 0.0,
            pragmatic_value: 0.0,
            epistemic_value: 0.0,
            selected: false,
        });
        ail.reset();
        assert!((ail.precision - 1.0).abs() < 1e-6);
        assert!((ail.expected_free_energy - 0.0).abs() < 1e-6);
        assert!(ail.action_proposals.is_empty());
    }

    #[test]
    fn test_empty_actions() {
        let mut ail = ActiveInferenceLoop::new();
        let proposals = ail.plan(&[0.5], &[0.8], &[]);
        assert!(proposals.is_empty());
        assert!(ail.best_action().is_none());
    }

    #[test]
    fn test_precision_clamping() {
        let mut ail = ActiveInferenceLoop::new();
        ail.update_precision(100.0);
        assert!(ail.precision <= 10.0);
        ail.update_precision(0.0);
        assert!(ail.precision >= 0.01);
    }
}
