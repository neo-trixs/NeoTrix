//! E₈ experimentation framework — systematic state-space exploration.
//!
//! Records outcomes of E8 reasoning mode selections and learns which
//! state transitions are effective for which task types over time.
//! Uses the 64-hexagram state space as an experiment design space.

use super::nt_core_hex::{
    ReasoningHexagram, evolve_strategy_entry,
};
use rand::Rng;
#[cfg(test)]
use super::nt_core_hex::strategy_matrix;
use crate::neotrix::nt_world_model::TaskType;

/// Number of factorized energy dimensions per mode.
/// Corresponds to E8's 6 binary reasoning axes.
pub const NUM_E8_FACTORS: usize = 6;

/// Compute task demand D_task from E8 factor context.
///
/// Maps E8 reasoning axes to D_task components (arXiv 2605.29682):
///   - ABST (bit5) → S_state  (state pressure / abstraction demand)
///   - SCOPE (bit4) → 1-V_oracle  (lack of verifier signal visibility)
///   - METH (bit3) → H_tool  (tool selection entropy)
///   - DEPTH (bit2) → L  (reasoning depth / steps required)
///   - MODE (bit1) → unused in base formula (extends observation noise scale)
///   - STANCE (bit0) → N_obs  (observation noise / ambiguity)
///
/// Formula: D_task = L · H_tool · S_state · (1 + N_obs) · (1 - V_oracle)
/// All values are normalized to [0.1, 1.0] before multiplication.
pub fn compute_task_demand(factor_context: &[f64; NUM_E8_FACTORS]) -> f64 {
    // Map E8 factor indices to D_task components
    // WARNING: This mapping assumes factor index convention:
    //   idx 0 = STANCE (N_obs), 1 = MODE, 2 = DEPTH (L),
    //   3 = METH (H_tool), 4 = SCOPE (1-V_oracle), 5 = ABST (S_state)
    let l = factor_context[2].abs().clamp(0.1, 1.0);      // DEPTH → L
    let h_tool = factor_context[3].abs().clamp(0.1, 1.0); // METH → H_tool
    let s_state = factor_context[5].abs().clamp(0.1, 1.0); // ABST → S_state
    let n_obs = factor_context[0].abs().clamp(0.0, 1.0);   // STANCE → N_obs (can be 0)
    let v_oracle = factor_context[4].abs().clamp(0.0, 0.9); // SCOPE → V_oracle (cap at 0.9)

    l * h_tool * s_state * (1.0 + n_obs) * (1.0 - v_oracle)
}

/// Epsilon-greedy RL policy over the 64-mode E8 state space with
/// factorized energy representations for each mode.
///
/// Each mode maintains a K-dim energy vector (K = NUM_E8_FACTORS)
/// where each dimension represents the mode's effectiveness on a
/// specific reasoning factor. Selection can be driven by factor context.
#[derive(Debug, Clone)]
pub struct E8Policy {
    epsilon: f64,
    epsilon_decay: f64,
    min_epsilon: f64,
    learning_rate: f64,
    discount: f64,
    pub mode_values: [f64; 64],
    pub mode_counts: [u64; 64],
    /// Per-mode factor energy vectors: [mode_idx][factor_idx]
    pub factor_energies: [[f64; NUM_E8_FACTORS]; 64],
    /// Overall factor control strength: how much the policy can influence each factor
    pub factor_control: [f64; NUM_E8_FACTORS],
    previous_mode: Option<ReasoningHexagram>,
}

impl Default for E8Policy {
    fn default() -> Self {
        Self::new(0.3, 0.995, 0.01, 0.1, 0.9)
    }
}

impl E8Policy {
    pub fn new(
        epsilon: f64, epsilon_decay: f64, min_epsilon: f64,
        learning_rate: f64, discount: f64,
    ) -> Self {
        Self {
            epsilon, epsilon_decay, min_epsilon, learning_rate, discount,
            mode_values: [0.0; 64],
            mode_counts: [0; 64],
            factor_energies: [[0.0; NUM_E8_FACTORS]; 64],
            factor_control: [0.0; NUM_E8_FACTORS],
            previous_mode: None,
        }
    }

    /// Select a mode using Gaussian Thought Sampling (GTS).
    ///
    /// Replaces naive uniform random exploration with Gaussian sampling
    /// centered on the best-known mode. Standard deviation increases with
    /// epsilon (higher exploration → wider search).
    ///
    /// Reference: GTS (arXiv:2602.14077) — learnable latent exploration sampling
    pub fn select_mode(
        &mut self, task: &str, task_type: TaskType, learner: &E8TransitionLearner,
    ) -> ReasoningHexagram {
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < self.epsilon {
            // GTS: sample from Gaussian centered on best-known mode instead of uniform
            let center = if let Some(best) = learner.best_known_mode(task) {
                best.0 as f64
            } else {
                let (best_idx, _) = self.mode_values.iter().enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap_or((0, &0.0));
                best_idx as f64
            };
            // Box-Muller transform for Gaussian(center, sigma) where sigma ∝ epsilon
            let sigma = 8.0 * (self.epsilon * 4.0).min(1.0);
            let u1: f64 = rng.gen_range(0.001..1.0);
            let u2: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
            let z = (-2.0 * u1.ln()).sqrt() * u2.cos();
            let sample = (center + sigma * z).round().clamp(0.0, 63.0) as u8;
            self.previous_mode = Some(ReasoningHexagram(sample));
            return ReasoningHexagram(sample);
        }
        if let Some(best) = learner.best_known_mode(task) {
            self.previous_mode = Some(best);
            return best;
        }
        let (best_idx, _) = self.mode_values.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((task_type as usize % 64, &0.0));
        let mode = ReasoningHexagram(best_idx as u8);
        self.previous_mode = Some(mode);
        mode
    }

    /// Factor-aware mode selection: choose mode whose factor energy profile
    /// best matches the given factor context vector.
    pub fn select_mode_by_factors(&mut self, factor_context: &[f64; NUM_E8_FACTORS]) -> ReasoningHexagram {
        if factor_context.iter().all(|&v| v.abs() < 1e-9) {
            return self.best_mode();
        }
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < self.epsilon {
            let sample = rng.gen_range(0..64) as u8;
            self.previous_mode = Some(ReasoningHexagram(sample));
            return ReasoningHexagram(sample);
        }
        let best_idx = self.factor_energies.iter().enumerate()
            .map(|(i, energies)| {
                let match_score: f64 = energies.iter().zip(factor_context.iter())
                    .map(|(e, c)| e * c)
                    .sum();
                (i, match_score)
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let mode = ReasoningHexagram(best_idx as u8);
        self.previous_mode = Some(mode);
        mode
    }

    /// TD-style update: Q(s,a) += lr * (reward + discount * max_a Q(s',a) - Q(s,a))
    pub fn update(&mut self, reward: f64) {
        if let Some(prev) = self.previous_mode {
            let idx = prev.0 as usize;
            self.mode_counts[idx] += 1;
            let n = self.mode_counts[idx] as f64;
            let lr = self.learning_rate / (1.0 + n.sqrt());
            let max_q = self.mode_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let td_error = reward + self.discount * max_q - self.mode_values[idx];
            self.mode_values[idx] += lr * td_error;
        }
    }

    /// Factorized update: updates per-factor energies given factor deltas.
    /// factor_deltas[i] = how much factor i changed after the action.
    /// Larger delta = more controllable.
    pub fn update_factorized(&mut self, reward: f64, factor_deltas: &[f64; NUM_E8_FACTORS]) {
        if let Some(prev) = self.previous_mode {
            let idx = prev.0 as usize;
            self.mode_counts[idx] += 1;
            let n = self.mode_counts[idx] as f64;
            let lr = self.learning_rate / (1.0 + n.sqrt());

            // Update overall mode value
            let max_q = self.mode_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let td_error = reward + self.discount * max_q - self.mode_values[idx];
            self.mode_values[idx] += lr * td_error;

            // Update per-factor energies
            for f in 0..NUM_E8_FACTORS {
                let delta = factor_deltas[f];
                let factor_lr = lr * (1.0 + delta.abs());
                let energy_td = reward * (1.0 + delta.abs()) - self.factor_energies[idx][f];
                self.factor_energies[idx][f] += factor_lr * energy_td;
            }

            // Update global factor control tracking
            for f in 0..NUM_E8_FACTORS {
                let control_delta = factor_deltas[f].abs() - self.factor_control[f];
                self.factor_control[f] += 0.1 * control_delta;
                self.factor_control[f] = self.factor_control[f].max(0.0).min(1.0);
            }
        }
    }

    /// Find the best mode for a specific factor index.
    pub fn best_mode_by_factor(&self, factor_idx: usize) -> ReasoningHexagram {
        let factor_idx = factor_idx.min(NUM_E8_FACTORS - 1);
        let (best_idx, _) = self.factor_energies.iter().enumerate()
            .map(|(i, energies)| (i, energies[factor_idx]))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((0, 0.0));
        ReasoningHexagram(best_idx as u8)
    }

    /// Select mode by task-demand-normalized EFC.
    /// Computes D_task from factor_context, then selects mode maximizing
    /// factor_energy[mode] · factor_context / D_task.
    /// Drops to best_mode() if D_task is zero.
    pub fn select_mode_by_task_demand(&mut self, factor_context: &[f64; NUM_E8_FACTORS]) -> ReasoningHexagram {
        // If context is near-zero, fall back to best_mode()
        if factor_context.iter().all(|&v| v.abs() < 1e-9) {
            return self.best_mode();
        }
        let d_task = compute_task_demand(factor_context);
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < self.epsilon {
            let sample = rng.gen_range(0..64) as u8;
            self.previous_mode = Some(ReasoningHexagram(sample));
            return ReasoningHexagram(sample);
        }
        let best_idx = self.factor_energies.iter().enumerate()
            .map(|(i, energies)| {
                let match_score: f64 = energies.iter().zip(factor_context.iter())
                    .map(|(e, c)| e * c)
                    .sum();
                (i, match_score / d_task)  // Normalize by task demand
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let mode = ReasoningHexagram(best_idx as u8);
        self.previous_mode = Some(mode);
        mode
    }

    /// Decay epsilon after each selection.
    pub fn decay_epsilon(&mut self) {
        self.epsilon = (self.epsilon * self.epsilon_decay).max(self.min_epsilon);
    }

    pub fn best_mode(&self) -> ReasoningHexagram {
        let (idx, _) = self.mode_values.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((0, &0.0));
        ReasoningHexagram(idx as u8)
    }

    pub fn epsilon(&self) -> f64 { self.epsilon }
    pub fn reset_previous(&mut self) { self.previous_mode = None; }
    pub fn set_previous(&mut self, mode: ReasoningHexagram) {
        self.previous_mode = Some(mode);
    }

    pub fn previous_mode(&self) -> Option<ReasoningHexagram> {
        self.previous_mode
    }
}

/// A single E8 experiment outcome.
#[derive(Debug, Clone)]
pub struct E8Outcome {
    /// Task description (or task type identifier)
    pub task: String,
    /// E8 mode that was used
    pub mode: ReasoningHexagram,
    /// The reward obtained (from the SEAL loop or evaluation)
    pub reward: f64,
    /// Iteration when this was recorded
    pub iteration: u64,
}

/// Accumulates experiment outcomes and learns optimal transitions.
#[derive(Debug, Clone)]
pub struct E8TransitionLearner {
    /// All recorded outcomes
    pub outcomes: Vec<E8Outcome>,
    /// Per-mode average rewards: mode index → (total_reward, count)
    pub mode_rewards: [(f64, u64); 64],
    /// Per-(approach, domain) average rewards for the strategy matrix
    pub matrix_rewards: [[(f64, u64); 8]; 8],
    /// Max outcomes to retain
    pub max_outcomes: usize,
    /// Minimum data points before making a transition decision
    pub min_samples: u64,
    /// Reward improvement threshold to trigger an evolution
    pub improvement_threshold: f64,
}

impl E8TransitionLearner {
    pub fn new(max_outcomes: usize, min_samples: u64, improvement_threshold: f64) -> Self {
        Self {
            outcomes: Vec::with_capacity(max_outcomes),
            mode_rewards: [(0.0, 0); 64],
            matrix_rewards: [[(0.0, 0); 8]; 8],
            max_outcomes,
            min_samples,
            improvement_threshold,
        }
    }

    /// Record an outcome from a SEAL loop iteration.
    pub fn record(&mut self, task: &str, mode: ReasoningHexagram, reward: f64, iteration: u64) {
        if self.outcomes.len() >= self.max_outcomes {
            self.outcomes.remove(0);
        }
        let outcome = E8Outcome {
            task: task.to_string(),
            mode,
            reward,
            iteration,
        };

        // Update per-mode average
        let (total, count) = &mut self.mode_rewards[mode.0 as usize];
        *total += reward;
        *count += 1;

        // Update per-matrix-cell average (approach × domain)
        let approach = (mode.0 >> 3) as usize;
        let domain = (mode.0 & 0x07) as usize;
        let (m_total, m_count) = &mut self.matrix_rewards[approach][domain];
        *m_total += reward;
        *m_count += 1;

        self.outcomes.push(outcome);
    }

    /// Get the average reward for a given mode.
    pub fn mode_avg_reward(&self, mode: ReasoningHexagram) -> Option<f64> {
        let (total, count) = self.mode_rewards[mode.0 as usize];
        if count >= self.min_samples {
            Some(total / count as f64)
        } else {
            None
        }
    }

    /// Get the average reward for a strategy matrix cell.
    pub fn matrix_cell_avg(&self, approach: usize, domain: usize) -> Option<f64> {
        let (total, count) = self.matrix_rewards[approach][domain];
        if count >= self.min_samples {
            Some(total / count as f64)
        } else {
            None
        }
    }

    /// Identify the best mode for a task by looking up keyword-matched outcomes.
    pub fn best_known_mode(&self, task: &str) -> Option<ReasoningHexagram> {
        let lower = task.to_lowercase();
        let mut best_score = f64::NEG_INFINITY;
        let mut best_mode = None;

        for outcome in &self.outcomes {
            if lower.contains(&outcome.task.to_lowercase()) || outcome.task.to_lowercase().contains(&lower) {
                let avg = self.mode_avg_reward(outcome.mode).unwrap_or(outcome.reward);
                if avg > best_score {
                    best_score = avg;
                    best_mode = Some(outcome.mode);
                }
            }
        }
        best_mode
    }

    /// Suggest a matrix evolution for an underperforming cell.
    /// Returns the pattern name to apply (e.g., "Oscillation", "Inefficient")
    /// and the cell coordinates if evolution would be beneficial.
    pub fn suggest_evolution(&self, approach: usize, domain: usize) -> Option<&'static str> {
        let current_avg = self.matrix_cell_avg(approach, domain);

        // Compare with the cell's 4 neighbors (flip axes 4, 3, 2, 1)
        let base_mode = ReasoningHexagram(((approach as u8) << 3) | (domain as u8));
        let candidates: [(u8, &str); 4] = [
            (1 << 4, "Oscillation"),    // flip scope
            (1 << 3, "Inefficient"),    // flip method
            (1 << 2, "Fast"),           // flip depth
            (1 << 1, "Collaborative"),  // flip mode
        ];

        for (flip, name) in candidates {
            let neighbor = base_mode.flip_axes(flip);
            let approach_n = (neighbor.0 >> 3) as usize;
            let domain_n = (neighbor.0 & 0x07) as usize;
            if let Some(neighbor_avg) = self.matrix_cell_avg(approach_n, domain_n) {
                if let Some(cur_avg) = current_avg {
                    if neighbor_avg > cur_avg + self.improvement_threshold {
                        return Some(name);
                    }
                }
            }
        }
        None
    }

    /// Run a full evolution pass over the strategy matrix.
    /// Returns the number of cells that were evolved.
    pub fn evolve_matrix(&self, matrix: &mut [[ReasoningHexagram; 8]; 8]) -> usize {
        let mut evolved = 0;
        for approach in 0..8 {
            for domain in 0..8 {
                if let Some(pattern) = self.suggest_evolution(approach, domain) {
                    let current = matrix[approach][domain];
                    if evolve_strategy_entry(matrix, current, pattern) {
                        evolved += 1;
                    }
                }
            }
        }
        evolved
    }

    pub fn clear(&mut self) {
        self.outcomes.clear();
        self.mode_rewards = [(0.0, 0); 64];
        self.matrix_rewards = [[(0.0, 0); 8]; 8];
    }
}

impl Default for E8TransitionLearner {
    fn default() -> Self {
        Self::new(500, 3, 0.05)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_query() {
        let mut learner = E8TransitionLearner::new(100, 2, 0.05);
        let mode = ReasoningHexagram(0); // Deep Debug

        learner.record("fix a crash bug", mode, 0.8, 1);
        learner.record("fix another crash", mode, 0.9, 2);

        let avg = learner.mode_avg_reward(mode);
        assert!(avg.is_some());
        assert!((avg.unwrap() - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_insufficient_data() {
        let learner = E8TransitionLearner::new(100, 5, 0.05);
        let mode = ReasoningHexagram(0);
        assert!(learner.mode_avg_reward(mode).is_none());
    }

    #[test]
    fn test_best_known_mode() {
        let mut learner = E8TransitionLearner::new(100, 1, 0.05);
        let mode_a = ReasoningHexagram(0); // Deep Debug
        let mode_b = ReasoningHexagram(4); // Code Review

        learner.record("review this code for bugs", mode_b, 0.9, 1);
        learner.record("fix crash", mode_a, 0.7, 2);

        let best = learner.best_known_mode("review this code");
        assert!(best.is_some());
        assert_eq!(best.unwrap(), mode_b);
    }

    #[test]
    fn test_suggest_evolution() {
        let mut learner = E8TransitionLearner::new(100, 1, 0.0);
        let cell_mode = ReasoningHexagram(0b000_000); // Debug × Bug (approach=0, domain=0)
        let neighbor = ReasoningHexagram(0b001_000); // Test × Bug (approach=1, domain=0)

        // Give neighbor a higher reward
        learner.record("test", neighbor, 0.9, 1);
        learner.record("debug", cell_mode, 0.3, 2);

        let suggestion = learner.suggest_evolution(0, 0);
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_evolve_matrix() {
        let mut learner = E8TransitionLearner::new(100, 1, 0.0);
        let mut matrix = strategy_matrix();

        // Seed rewards that suggest evolution for cell (0,0)
        for i in 0..8 {
            for j in 0..8 {
                let m = matrix[i][j];
                let reward = if i == 1 && j == 0 { 0.9 } else { 0.3 };
                learner.record("task", m, reward, (i * 8 + j) as u64);
            }
        }

        let count = learner.evolve_matrix(&mut matrix);
        assert!(count > 0);
    }

    #[test]
    fn test_e8_policy_default_has_epsilon() {
        let policy = E8Policy::default();
        assert!((policy.epsilon() - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_e8_policy_update_changes_values() {
        let mut policy = E8Policy::new(1.0, 1.0, 1.0, 0.5, 0.0);
        let learner = E8TransitionLearner::new(100, 1, 0.05);
        let mode = policy.select_mode("test", TaskType::General, &learner);
        let prev_val = policy.mode_values[mode.0 as usize];
        policy.update(1.0);
        assert!(policy.mode_values[mode.0 as usize] > prev_val, "positive reward should increase value");
    }

    #[test]
    fn test_e8_policy_epsilon_decays() {
        let mut policy = E8Policy::new(1.0, 0.5, 0.1, 0.1, 0.9);
        policy.decay_epsilon();
        assert!(policy.epsilon() < 1.0);
        assert!(policy.epsilon() >= 0.1);
    }

    #[test]
    fn test_e8_policy_best_mode_returns_some() {
        let policy = E8Policy::default();
        let mode = policy.best_mode();
        assert!(mode.0 < 64);
    }

    #[test]
    fn test_factor_energies_initialized_zero() {
        let policy = E8Policy::default();
        for mode in 0..64 {
            for f in 0..NUM_E8_FACTORS {
                assert_eq!(policy.factor_energies[mode][f], 0.0);
            }
        }
    }

    #[test]
    fn test_update_factorized_updates_energies() {
        let mut policy = E8Policy::new(1.0, 1.0, 1.0, 0.5, 0.0);
        policy.set_previous(ReasoningHexagram(0));
        let deltas = [0.8, 0.2, 0.0, 0.5, 0.1, 0.0];
        policy.update_factorized(1.0, &deltas);
        for f in 0..NUM_E8_FACTORS {
            if deltas[f].abs() > 0.01 {
                assert!(policy.factor_energies[0][f] != 0.0,
                    "factor {} should have non-zero energy after update", f);
            }
        }
    }

    #[test]
    fn test_factor_control_tracks_controllability() {
        let mut policy = E8Policy::new(1.0, 1.0, 1.0, 0.5, 0.0);
        policy.set_previous(ReasoningHexagram(0));
        // Repeatedly update with strong control on factor 0
        let deltas = [0.9, 0.1, 0.0, 0.0, 0.0, 0.0];
        for _ in 0..10 {
            policy.update_factorized(1.0, &deltas);
        }
        assert!(policy.factor_control[0] > policy.factor_control[1],
            "factor 0 should have higher control than factor 1");
    }

    #[test]
    fn test_best_mode_by_factor() {
        let mut policy = E8Policy::new(1.0, 1.0, 1.0, 0.5, 0.0);
        policy.set_previous(ReasoningHexagram(10));
        let deltas = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        policy.update_factorized(1.0, &deltas);

        policy.set_previous(ReasoningHexagram(20));
        let deltas2 = [2.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        policy.update_factorized(1.0, &deltas2);

        let best_for_0 = policy.best_mode_by_factor(0);
        assert_eq!(best_for_0.0, 20, "mode 20 should be best for factor 0");
    }

    #[test]
    fn test_select_mode_by_factors_matches_best_energy() {
        let mut policy = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0); // epsilon=0 → greedy
        policy.factor_energies[5] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        policy.factor_energies[10] = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0];

        let context = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let selected = policy.select_mode_by_factors(&context);
        assert_eq!(selected.0, 10, "should pick mode 10 for factor 1 context");
    }

    #[test]
    fn test_select_mode_by_factors_zero_context_falls_back() {
        let mut policy = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        policy.mode_values[3] = 1.0;
        let context = [0.0; NUM_E8_FACTORS];
        let selected = policy.select_mode_by_factors(&context);
        assert_eq!(selected.0, 3, "zero context should fall back to best_mode");
    }

    // ── D_task Tests ────────────────────────────────────────────────

    #[test]
    fn test_compute_task_demand_baseline() {
        // Moderate values across all axes
        let ctx = [0.5; NUM_E8_FACTORS];
        let d = compute_task_demand(&ctx);
        // L=0.5, H_tool=0.5, S_state=0.5, (1+N_obs)=1.5, (1-V_oracle)=0.5
        // Expected: 0.5 * 0.5 * 0.5 * 1.5 * 0.5 = 0.09375
        assert!((d - 0.09375).abs() < 0.001, "D_task should be ~0.09375, got {d}");
    }

    #[test]
    fn test_compute_task_demand_high_demand() {
        // High depth, high tool entropy, high abstraction, noisy, low verification
        let ctx = [0.9, 0.5, 0.9, 0.9, 0.1, 0.9];
        let d = compute_task_demand(&ctx);
        assert!(d > 0.2, "high-demand task should have D_task > 0.2, got {d}");
    }

    #[test]
    fn test_compute_task_demand_low_demand() {
        // Low depth, low entropy, concrete, clear, high verifiability
        let ctx = [0.1, 0.5, 0.1, 0.1, 0.9, 0.1];
        let d = compute_task_demand(&ctx);
        assert!(d < 0.05, "low-demand task should have D_task < 0.05, got {d}");
    }

    #[test]
    fn test_compute_task_demand_clamps_values() {
        // Extreme values should be clamped
        let ctx = [-5.0, 0.5, 10.0, 0.5, 2.0, 0.5];
        let d = compute_task_demand(&ctx);
        assert!(d.is_finite());
        assert!(d > 0.0);
    }

    #[test]
    fn test_select_mode_by_task_demand_greedy() {
        let mut policy = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0); // epsilon=0 → greedy
        policy.factor_energies[5] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        policy.factor_energies[10] = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0];

        // Context that strongly favors factor 1 (MODE)
        let ctx = [0.0, 1.0, 0.1, 0.1, 0.8, 0.1]; // Low D_task, high MODE demand
        let selected = policy.select_mode_by_task_demand(&ctx);
        assert_eq!(selected.0, 10, "should pick mode 10 for factor 1 context");
    }

    #[test]
    fn test_select_mode_by_task_demand_zero_demand_falls_back() {
        let mut policy = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        policy.mode_values[3] = 1.0;
        let ctx = [0.0; NUM_E8_FACTORS];
        let selected = policy.select_mode_by_task_demand(&ctx);
        assert_eq!(selected.0, 3, "zero D_task should fall back to best_mode");
    }
}
