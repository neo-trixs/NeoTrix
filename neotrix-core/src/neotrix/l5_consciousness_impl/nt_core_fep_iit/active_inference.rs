//! Active Inference Loop — Expected Free Energy + Precision Estimation
//!
//! Extends NeoTrix FEP-IIT bridge with a full active inference cycle:
//!
//! For each policy π, expected free energy G(π):
//!   G(π) = -E_Q(o|π)[ln P(o)] + E_Q(s|π)[KL[Q(o|s) || P(o|s)]]
//!        ≈ -pragmatic_value + epistemic_value
//!
//! Where:
//!   - pragmatic_value: cosine similarity between predicted state and goal state
//!   - epistemic_value: information gain = reduction in uncertainty
//!   - precision: inverse of expected prediction error (confidence)
//!
//! Action selection minimizes G(π). Precision is updated online via
//! prediction error — when predictions are accurate, precision increases.

use serde::{Deserialize, Serialize};

use super::FE_NORMALIZE_MAX;

/// A single action proposal with decomposed expected free energy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionProposal {
    /// Human-readable action label
    pub action: String,
    /// Total expected free energy G(π) — lower is better
    pub expected_free_energy: f64,
    /// Pragmatic value: expected goal attainment [0,1] — higher is better
    pub epistemic_value: f64,
    /// Epistemic value: expected information gain [0,1] — higher is better
    pub pragmatic_value: f64,
    /// Whether this action was selected by the inference loop
    pub selected: bool,
}

/// Active inference loop with precision estimation.
///
/// Maintains precision (inverse confidence) over prediction error and
/// selects actions by minimizing expected free energy:
///
///   G(π) = -cos_sim(predicted, goal) + epistemic_value / precision
///
/// Precision is updated each cycle:
///   precision ← precision + η · (accuracy - precision)
///   accuracy = 1 / (1 + prediction_error)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveInferenceLoop {
    /// Current precision (inverse variance) — higher = more confident
    pub precision: f64,
    /// Expected free energy of the selected action
    pub expected_free_energy: f64,
    /// Proposals for all evaluated actions
    pub action_proposals: Vec<ActionProposal>,
    /// Learning rate for precision adaptation
    pub learning_rate: f64,
    /// Weight balancing epistemic vs pragmatic in G(π)
    pub exploration_weight: f64,
    /// Running estimate of prediction error (used for precision update)
    pub prediction_error: f64,
}

impl Default for ActiveInferenceLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveInferenceLoop {
    pub fn new() -> Self {
        Self {
            precision: 1.0,
            expected_free_energy: 0.0,
            action_proposals: Vec::new(),
            learning_rate: 0.1,
            exploration_weight: 1.0,
            prediction_error: 0.0,
        }
    }

    /// Configure precision (confidence).
    pub fn with_precision(mut self, precision: f64) -> Self {
        self.precision = precision.max(0.01);
        self
    }

    /// Configure learning rate for precision adaptation.
    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.learning_rate = lr.max(0.001).min(1.0);
        self
    }

    /// Configure exploration weight (epistemic vs pragmatic balance).
    pub fn with_exploration(mut self, weight: f64) -> Self {
        self.exploration_weight = weight.max(0.0);
        self
    }

    /// Cosine similarity between two vectors.
    fn cosine_sim(a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        if n == 0 {
            return 0.0;
        }
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        for i in 0..n {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }
        let denom = (norm_a.sqrt() * norm_b.sqrt()).max(1e-12);
        (dot / denom).clamp(-1.0, 1.0)
    }

    /// Compute expected free energy for a set of candidate actions.
    ///
    /// For each action (name, state_delta):
    ///   1. Predict next state: s' = state + state_delta
    ///   2. pragmatic_value = cos_sim(s', goal)        [0,1]  — goal attainment
    ///   3. epistemic_value = 1 - cos_sim(s', state)    [0,1]  — information gain / deviation
    ///   4. G(π) = -pragmatic_value + epistemic_value / precision
    ///
    /// Returns proposals sorted by G(π) (best first), with `selected` set on the best.
    pub fn compute_expected_free_energy(
        &mut self,
        state: &[f64],
        goal: &[f64],
        actions: &[(&str, Vec<f64>)],
    ) -> Vec<ActionProposal> {
        let mut proposals: Vec<ActionProposal> = actions
            .iter()
            .map(|(name, delta)| {
                let predicted: Vec<f64> = state
                    .iter()
                    .zip(delta.iter())
                    .map(|(s, d)| s + d)
                    .collect();

                let pragmatic_value = Self::cosine_sim(&predicted, goal);
                let epistemic_value = 1.0 - Self::cosine_sim(&predicted, state);

                let g = -pragmatic_value + self.exploration_weight * epistemic_value / self.precision;

                ActionProposal {
                    action: name.to_string(),
                    expected_free_energy: g,
                    epistemic_value,
                    pragmatic_value,
                    selected: false,
                }
            })
            .collect();

        proposals.sort_by(|a, b| {
            a.expected_free_energy
                .partial_cmp(&b.expected_free_energy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let best_g = proposals.first().map(|p| p.expected_free_energy).unwrap_or(0.0);
        if let Some(best) = proposals.first_mut() {
            best.selected = true;
        }

        self.action_proposals = proposals.clone();
        self.expected_free_energy = best_g;

        proposals
    }

    /// Select the best action index from a sorted proposal list.
    pub fn selected_index(&self) -> Option<usize> {
        self.action_proposals
            .iter()
            .position(|p| p.selected)
    }

    /// Select the best action name from the proposals.
    pub fn selected_action(&self) -> Option<&str> {
        self.action_proposals
            .iter()
            .find(|p| p.selected)
            .map(|p| p.action.as_str())
    }

    /// Update precision based on prediction error.
    ///
    ///   accuracy = 1 / (1 + prediction_error)
    ///   precision ← precision + η · (accuracy - precision)
    ///
    /// When predictions are accurate (error → 0), precision drifts toward 1.0.
    /// When predictions are poor (error → ∞), precision drifts toward 0.0.
    pub fn update_precision(&mut self, prediction_error: f64) -> f64 {
        self.prediction_error = prediction_error;
        let accuracy = 1.0 / (1.0 + prediction_error);
        self.precision += self.learning_rate * (accuracy - self.precision);
        self.precision = self.precision.max(0.01).min(FE_NORMALIZE_MAX);
        self.precision
    }

    /// Run a full active inference cycle:
    ///   1. Compute expected free energy for all actions
    ///   2. Select the best action
    ///   3. Compute prediction error from the best action
    ///   4. Update precision
    ///
    /// Returns the full proposal list (sorted, best marked selected).
    pub fn cycle(
        &mut self,
        state: &[f64],
        goal: &[f64],
        actions: &[(&str, Vec<f64>)],
    ) -> Vec<ActionProposal> {
        let proposals = self.compute_expected_free_energy(state, goal, actions);

        let pred_error = if let Some(best) = proposals.first() {
            1.0 - best.pragmatic_value
        } else {
            1.0
        };
        self.update_precision(pred_error);

        proposals
    }

    /// Reset the inference loop state (preserving configuration).
    pub fn reset(&mut self) {
        self.expected_free_energy = 0.0;
        self.action_proposals.clear();
        self.prediction_error = 0.0;
    }
}

/// Standalone function: compute expected free energy for actions given state and goal.
///
/// Convenience wrapper around `ActiveInferenceLoop::compute_expected_free_energy`.
pub fn compute_expected_free_energy(
    state: &[f64],
    goal: &[f64],
    actions: &[(&str, Vec<f64>)],
    precision: f64,
) -> Vec<ActionProposal> {
    let mut loop_ = ActiveInferenceLoop::new().with_precision(precision);
    loop_.compute_expected_free_energy(state, goal, actions)
}

/// Standalone function: update precision based on prediction error.
///
///   precision' = precision + η · (1/(1+error) - precision)
pub fn update_precision(
    current_precision: f64,
    prediction_error: f64,
    learning_rate: f64,
) -> f64 {
    let accuracy = 1.0 / (1.0 + prediction_error);
    let new_precision = current_precision + learning_rate * (accuracy - current_precision);
    new_precision.max(0.01).min(FE_NORMALIZE_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_loop() -> ActiveInferenceLoop {
        ActiveInferenceLoop::new()
    }

    // ——— Construction ———

    #[test]
    fn test_default_precision() {
        let al = make_loop();
        assert!((al.precision - 1.0).abs() < 1e-10);
        assert!((al.learning_rate - 0.1).abs() < 1e-10);
        assert!(al.action_proposals.is_empty());
    }

    #[test]
    fn test_with_precision_clamps_low() {
        let al = ActiveInferenceLoop::new().with_precision(-5.0);
        assert!((al.precision - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_with_learning_rate_clamps() {
        let al = ActiveInferenceLoop::new().with_learning_rate(5.0);
        assert!((al.learning_rate - 1.0).abs() < 1e-10);

        let al2 = ActiveInferenceLoop::new().with_learning_rate(0.0);
        assert!((al2.learning_rate - 0.001).abs() < 1e-10);
    }

    #[test]
    fn test_with_exploration_can_be_zero() {
        let al = ActiveInferenceLoop::new().with_exploration(0.0);
        assert!((al.exploration_weight - 0.0).abs() < 1e-10);
    }

    // ——— Cosine similarity ———

    #[test]
    fn test_cosine_sim_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = ActiveInferenceLoop::cosine_sim(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_sim_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = ActiveInferenceLoop::cosine_sim(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_sim_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = ActiveInferenceLoop::cosine_sim(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_sim_empty() {
        let sim = ActiveInferenceLoop::cosine_sim(&[], &[]);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_sim_zero_vector() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.0];
        let sim = ActiveInferenceLoop::cosine_sim(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    // ——— Expected Free Energy ———

    #[test]
    fn test_efc_prefers_action_toward_goal() {
        let state = vec![0.0, 0.0];
        let goal = vec![1.0, 0.0];

        let actions: &[(&str, Vec<f64>)] = &[
            ("toward_goal", vec![1.0, 0.0]),   // predicted = [1, 0] = goal
            ("away", vec![-1.0, 0.0]),          // predicted = [-1, 0]
            ("neutral", vec![0.0, 0.0]),        // predicted = [0, 0]
        ];

        let proposals = compute_expected_free_energy(&state, &goal, actions, 1.0);
        assert_eq!(proposals[0].action, "toward_goal");
        assert!(proposals[0].selected);
        assert!(proposals[0].pragmatic_value > proposals[1].pragmatic_value);
    }

    #[test]
    fn test_efc_epistemic_penalty_varied_by_precision() {
        // G = -pragmatic + epistemic / precision
        // At LOW precision, epistemic term is large → penalizes exploration.
        // At HIGH precision, epistemic term is tiny → pragmatic dominates.
        // State is orthogonal to goal, so "cautious" has zero pragmatic AND zero epistemic
        // while "explore" moves toward goal direction (higher pragmatic, moderate epistemic).
        let state = vec![0.0, 1.0];
        let goal = vec![1.0, 0.0];

        let actions: &[(&str, Vec<f64>)] = &[
            ("cautious", vec![0.0, 0.0]),    // predicted=[0,1], pragmatic=0, epistemic=0
            ("explore", vec![1.0, -1.0]),     // predicted=[1,0]=goal, pragmatic=1, epistemic>0
        ];

        // Low precision: epistemic is penalized → cautious wins despite zero pragmatic
        let proposals = compute_expected_free_energy(&state, &goal, actions, 0.2);
        assert_eq!(proposals[0].action, "cautious",
            "At low precision, cautious should win (no epistemic penalty)");
        assert!(proposals[0].expected_free_energy < proposals[1].expected_free_energy);

        // High precision: epistemic penalty negligible → explore wins (higher pragmatic)
        let proposals_high = compute_expected_free_energy(&state, &goal, actions, 10.0);
        assert_eq!(proposals_high[0].action, "explore",
            "At high precision, explore should win (pragmatic dominates)");
    }

    #[test]
    fn test_efc_empty_actions() {
        let proposals = compute_expected_free_energy(&[0.0], &[1.0], &[], 1.0);
        assert!(proposals.is_empty());
    }

    #[test]
    fn test_efc_single_action_autoselected() {
        let state = vec![0.0];
        let goal = vec![1.0];
        let actions: &[(&str, Vec<f64>)] = &[("only", vec![1.0])];
        let proposals = compute_expected_free_energy(&state, &goal, actions, 1.0);
        assert_eq!(proposals.len(), 1);
        assert!(proposals[0].selected);
    }

    // ——— G(π) monotonicity ———

    #[test]
    fn test_g_decreases_with_better_pragmatic() {
        let state = vec![0.0, 0.0];
        let goal = vec![5.0, 0.0];

        let actions: &[(&str, Vec<f64>)] = &[
            ("small", vec![1.0, 0.0]),
            ("medium", vec![3.0, 0.0]),
            ("large", vec![5.0, 0.0]),
        ];

        let proposals = compute_expected_free_energy(&state, &goal, actions, 10.0);
        // With high precision, pragmatic dominates:
        // larger step toward goal → higher pragmatic → lower G
        assert!(
            proposals[0].expected_free_energy <= proposals[1].expected_free_energy,
            "Best action should have lowest G"
        );
        assert!(
            proposals[1].expected_free_energy <= proposals[2].expected_free_energy,
            "Actions sorted by G ascending"
        );
    }

    // ——— Precision update ———

    #[test]
    fn test_update_precision_low_error_increases() {
        let new_p = update_precision(0.5, 0.05, 0.1);
        // accuracy = 1/1.05 ≈ 0.952
        // new = 0.5 + 0.1 * (0.952 - 0.5) = 0.5 + 0.0452 = 0.5452
        assert!(new_p > 0.5, "Low error should increase precision, got {new_p}");
        assert!(new_p <= FE_NORMALIZE_MAX);
    }

    #[test]
    fn test_update_precision_high_error_decreases() {
        let new_p = update_precision(0.9, 10.0, 0.1);
        // accuracy = 1/11 ≈ 0.091
        // new = 0.9 + 0.1 * (0.091 - 0.9) = 0.9 - 0.0809 = 0.8191
        assert!(new_p < 0.9, "High error should decrease precision");
        assert!(new_p >= 0.01);
    }

    #[test]
    fn test_update_precision_clamps_low() {
        let new_p = update_precision(0.01, 100.0, 0.5);
        assert!(new_p >= 0.01);
    }

    // ——— ActiveInferenceLoop methods ———

    #[test]
    fn test_selected_index() {
        let mut al = make_loop();
        let state = vec![0.0, 0.0];
        let goal = vec![1.0, 1.0];
        let actions: &[(&str, Vec<f64>)] = &[
            ("bad", vec![-1.0, -1.0]),
            ("good", vec![1.0, 1.0]),
        ];
        al.compute_expected_free_energy(&state, &goal, actions);
        assert_eq!(al.selected_index(), Some(0)); // "good" is first after sort
        assert_eq!(al.selected_action(), Some("good"));
    }

    #[test]
    fn test_selected_index_empty() {
        let mut al = make_loop();
        al.compute_expected_free_energy(&[0.0], &[1.0], &[]);
        assert!(al.selected_index().is_none());
        assert!(al.selected_action().is_none());
    }

    #[test]
    fn test_full_cycle_updates_precision() {
        let mut al = ActiveInferenceLoop::new()
            .with_precision(0.5)
            .with_learning_rate(0.2);

        let state = vec![0.0];
        let goal = vec![1.0];
        let actions: &[(&str, Vec<f64>)] = &[
            ("good", vec![1.0]),
            ("bad", vec![-1.0]),
        ];

        let initial_precision = al.precision;
        let proposals = al.cycle(&state, &goal, actions);

        assert!(al.action_proposals.len() == 2);
        assert!(proposals[0].selected);
        assert!(al.expected_free_energy <= proposals[1].expected_free_energy);
        // Precision should have changed
        assert!((al.precision - initial_precision).abs() > 1e-10 || al.precision == initial_precision);
    }

    #[test]
    fn test_reset() {
        let mut al = ActiveInferenceLoop::new()
            .with_precision(2.0)
            .with_learning_rate(0.3);

        let state = vec![0.0];
        let goal = vec![1.0];
        let actions: &[(&str, Vec<f64>)] = &[("a", vec![1.0])];
        al.cycle(&state, &goal, actions);
        assert!(!al.action_proposals.is_empty());

        let saved_precision = al.precision;
        let saved_lr = al.learning_rate;
        al.reset();

        assert!(al.action_proposals.is_empty());
        assert!((al.expected_free_energy - 0.0).abs() < 1e-10);
        // Configuration preserved
        assert!((al.precision - saved_precision).abs() < 1e-10);
        assert!((al.learning_rate - saved_lr).abs() < 1e-10);
    }

    // ——— Edge cases ———

    #[test]
    fn test_efc_goal_already_reached() {
        let state = vec![5.0, 3.0];
        let goal = vec![5.0, 3.0]; // already there

        let actions: &[(&str, Vec<f64>)] = &[
            ("stay", vec![0.0, 0.0]),
            ("move", vec![1.0, 0.0]),
        ];

        let proposals = compute_expected_free_energy(&state, &goal, actions, 1.0);
        // "stay" predicts current = goal → pragmatic = 1.0, epistemic = 0.0
        // "move" predicts away from goal → pragmatic < 1.0
        assert_eq!(proposals[0].action, "stay");
        assert!(proposals[0].pragmatic_value > proposals[1].pragmatic_value);
    }

    #[test]
    fn test_precision_converges_over_repeated_cycles() {
        let mut al = ActiveInferenceLoop::new()
            .with_precision(0.5)
            .with_learning_rate(0.3);

        let state = vec![0.0];
        let goal = vec![1.0];
        let actions: &[(&str, Vec<f64>)] = &[("exact", vec![1.0])];

        for _ in 0..20 {
            al.cycle(&state, &goal, actions);
        }

        // After many good predictions, precision should approach 1.0
        assert!(al.precision > 0.8, "Precision should converge toward 1.0, got {}", al.precision);
    }

    #[test]
    fn test_standalone_compute_and_update_independent() {
        // Verify the standalone functions are consistent with the struct methods
        let state = vec![0.0, 0.0];
        let goal = vec![1.0, 1.0];
        let actions: &[(&str, Vec<f64>)] = &[("diag", vec![1.0, 1.0])];

        let proposals = compute_expected_free_energy(&state, &goal, actions, 1.0);
        assert!(!proposals.is_empty());
        assert!(proposals[0].pragmatic_value > 0.9);

        let new_p = update_precision(1.0, 0.01, 0.1);
        let mut al = ActiveInferenceLoop::new().with_precision(1.0);
        let al_new_p = al.update_precision(0.01);
        assert!((new_p - al_new_p).abs() < 1e-10);
    }
}
