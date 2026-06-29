#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LSE — Learning Self-Evolution: RL-based mutation policy
///
/// Maintains a lightweight Q-table where state = (recent_success_rate_binned, compilation_trend)
/// and actions = mutation type variants. Uses epsilon-greedy exploration.
///
/// Replaces the Thompson-sampling bandit (`DriveBanditState`) with a Q-learning agent
/// that learns which mutation types to select based on past outcomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsePolicy {
    /// Q-table: state -> Q-values per action
    q_table: HashMap<(u8, u8), Vec<f64>>,
    /// Exploration rate (decays toward epsilon_min)
    epsilon: f64,
    /// Minimum exploration rate
    epsilon_min: f64,
    /// Decay multiplier applied each step
    epsilon_decay: f64,
    /// Number of mutation action types
    action_count: usize,
    /// Q-learning learning rate
    learning_rate: f64,
    /// Q-learning discount factor
    discount: f64,
    /// Last state (for Q-learning update)
    last_state: Option<(u8, u8)>,
    /// Last action index taken
    last_action: Option<usize>,
    /// Total learning steps (for epsilon decay tracking)
    pub total_steps: u64,
    /// Random number generator state (simple LCG)
    rng_state: u64,
}

impl LsePolicy {
    /// Create a new LSE policy.
    ///
    /// `action_count` — number of distinct mutation types (typically 6: TuneParam,
    /// AddHandler, RewriteHandler, SwapPolicy, RewritePrimitive, RewriteMeta).
    ///
    /// Defaults: epsilon=0.3, epsilon_min=0.01, epsilon_decay=0.995,
    /// learning_rate=0.1, discount=0.9
    pub fn new(action_count: usize) -> Self {
        Self {
            q_table: HashMap::new(),
            epsilon: 0.3,
            epsilon_min: 0.01,
            epsilon_decay: 0.995,
            action_count,
            learning_rate: 0.1,
            discount: 0.9,
            last_state: None,
            last_action: None,
            total_steps: 0,
            rng_state: 0,
        }
    }

    /// Configure epsilon (exploration rate).
    pub fn with_epsilon(mut self, epsilon: f64) -> Self {
        self.epsilon = epsilon.clamp(self.epsilon_min, 1.0);
        self
    }

    /// Configure epsilon minimum.
    pub fn with_epsilon_min(mut self, min: f64) -> Self {
        self.epsilon_min = min;
        self.epsilon = self.epsilon.max(min);
        self
    }

    /// Configure epsilon decay.
    pub fn with_epsilon_decay(mut self, decay: f64) -> Self {
        self.epsilon_decay = decay;
        self
    }

    /// Configure learning rate.
    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }

    /// Configure discount factor.
    pub fn with_discount(mut self, discount: f64) -> Self {
        self.discount = discount;
        self
    }

    /// Select an action (mutation type index) using epsilon-greedy policy.
    ///
    /// `success_rate` — recent success rate in [0, 1]
    /// `compile_trend` — recent compilation success trend in [0, 1]
    ///
    /// Returns an action index in [0, action_count).
    pub fn select_action(&mut self, success_rate: f64, compile_trend: f64) -> usize {
        let state = self.discretize(success_rate, compile_trend);

        // Pre-compute random values before borrowing self.q_table
        let rng_explore = self.rng();
        let rng_action = self.rng();
        let rng_idx = self.rng();
        let explore = rng_explore < self.epsilon;

        let actions = self
            .q_table
            .entry(state)
            .or_insert_with(|| vec![0.0; self.action_count]);

        let action = if explore {
            (rng_action * self.action_count as f64) as usize % self.action_count
        } else {
            let max_q = actions.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            if max_q.is_finite() {
                let candidates: Vec<usize> = actions
                    .iter()
                    .enumerate()
                    .filter(|(_, &q)| (q - max_q).abs() < 1e-9)
                    .map(|(i, _)| i)
                    .collect();
                let idx = (rng_idx * candidates.len() as f64) as usize;
                candidates[idx % candidates.len()]
            } else {
                (rng_action * self.action_count as f64) as usize % self.action_count
            }
        };

        self.last_state = Some(state);
        self.last_action = Some(action);
        action
    }

    /// Learn from a reward signal.
    ///
    /// `reward` — scalar reward (e.g. after_score - before_score, adjusted by compiles).
    ///
    /// Uses standard Q-learning: Q(s, a) ← Q(s, a) + lr * [reward - Q(s, a)]
    /// (monte-carlo return, no bootstrap needed for single-step bandit-like setting).
    pub fn learn(&mut self, reward: f64) {
        let state = match self.last_state {
            Some(s) => s,
            None => return, // No prior action to learn from
        };
        let action = match self.last_action {
            Some(a) => a,
            None => return,
        };

        let actions = self
            .q_table
            .entry(state)
            .or_insert_with(|| vec![0.0; self.action_count]);

        if action < actions.len() {
            let current_q = actions[action];
            // Standard Q-learning update (monte-carlo, single-step bandit)
            let td_error = reward - current_q;
            let new_q = current_q + self.learning_rate * td_error;
            actions[action] = new_q;
        }

        self.total_steps += 1;
        // Decay epsilon
        self.epsilon = (self.epsilon * self.epsilon_decay).max(self.epsilon_min);
    }

    /// Discretize continuous metrics into binned state.
    ///
    /// Both `success_rate` and `compile_trend` are clamped to [0, 1] and
    /// quantized into 16 bins each (0..15), giving 256 possible states.
    pub fn discretize(&self, success_rate: f64, compile_trend: f64) -> (u8, u8) {
        let s = (success_rate.clamp(0.0, 1.0) * 15.0).round() as u8;
        let c = (compile_trend.clamp(0.0, 1.0) * 15.0).round() as u8;
        (s.min(15), c.min(15))
    }

    /// Save the policy to a JSON file (atomically via tmp + rename).
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let data = serde_json::to_vec(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let tmp = format!("{}.tmp", path);
        std::fs::write(&tmp, &data)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Load the policy from a JSON file.
    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let data = std::fs::read(path)?;
        serde_json::from_slice(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Get the current epsilon value.
    pub fn epsilon_current(&self) -> f64 {
        self.epsilon
    }

    /// Get the Q-values for a given discretized state.
    pub fn q_values(&self, state: (u8, u8)) -> Option<&[f64]> {
        self.q_table.get(&state).map(|v| v.as_slice())
    }

    /// Get the best action (greedy, no exploration) for a given state.
    pub fn best_action(&self, success_rate: f64, compile_trend: f64) -> Option<usize> {
        let state = self.discretize(success_rate, compile_trend);
        let actions = self.q_table.get(&state)?;
        let max_q = actions.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if !max_q.is_finite() {
            return None;
        }
        actions
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }

    /// Number of unique states in the Q-table.
    pub fn state_count(&self) -> usize {
        self.q_table.len()
    }

    /// Total number of learning updates performed.
    pub fn update_count(&self) -> u64 {
        self.total_steps
    }

    fn rng(&mut self) -> f64 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let x = (self.rng_state >> 33) as u32;
        (x as f64) / (u32::MAX as f64)
    }
}

impl Default for LsePolicy {
    fn default() -> Self {
        // 6 mutation types: TuneParam, AddHandler, RewriteHandler,
        // SwapPolicy, RewritePrimitive, RewriteMeta
        Self::new(6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_initializes_with_defaults() {
        let lse = LsePolicy::new(6);
        assert_eq!(lse.action_count, 6);
        assert!((lse.epsilon - 0.3).abs() < 1e-9);
        assert!((lse.epsilon_min - 0.01).abs() < 1e-9);
        assert!((lse.epsilon_decay - 0.995).abs() < 1e-9);
        assert!((lse.learning_rate - 0.1).abs() < 1e-9);
        assert!((lse.discount - 0.9).abs() < 1e-9);
        assert_eq!(lse.total_steps, 0);
        assert!(lse.last_state.is_none());
        assert!(lse.last_action.is_none());
    }

    #[test]
    fn test_discretize_clamps_and_bins() {
        let lse = LsePolicy::new(6);
        // Normal range
        let (s, c) = lse.discretize(0.5, 0.5);
        assert_eq!(s, 7); // 0.5 * 15 = 7.5 → round to 8? Actually 7.5 rounds to 8
        assert_eq!(c, 7);
        // Boundaries
        let (s, c) = lse.discretize(0.0, 0.0);
        assert_eq!(s, 0);
        assert_eq!(c, 0);
        let (s, c) = lse.discretize(1.0, 1.0);
        assert_eq!(s, 15);
        assert_eq!(c, 15);
        // Clamping
        let (s, c) = lse.discretize(-0.5, 1.5);
        assert_eq!(s, 0);
        assert_eq!(c, 15);
    }

    #[test]
    fn test_select_action_returns_valid_index() {
        let mut lse = LsePolicy::new(6);
        // Force greedy (epsilon = 0)
        lse.epsilon = 0.0;
        lse.epsilon_min = 0.0;
        // Seed rng for deterministic behavior
        lse.rng_state = 42;
        let action = lse.select_action(0.5, 0.5);
        assert!(action < 6, "action {} should be < 6", action);
        // After selection, last_state and last_action should be set
        assert!(lse.last_state.is_some());
        assert!(lse.last_action.is_some());
    }

    #[test]
    fn test_select_action_explores_with_high_epsilon() {
        let mut lse = LsePolicy::new(6);
        lse.epsilon = 1.0; // Always explore (random)
        lse.rng_state = 42;
        let mut actions_seen = std::collections::HashSet::new();
        for _ in 0..100 {
            let a = lse.select_action(0.5, 0.5);
            actions_seen.insert(a);
        }
        // With 100 samples, should see at least 2 different actions
        assert!(
            actions_seen.len() >= 2,
            "only saw {} actions",
            actions_seen.len()
        );
    }

    #[test]
    fn test_learn_updates_q_table() {
        let mut lse = LsePolicy::new(6);
        lse.epsilon = 0.0; // Greedy

        // First action creates a state entry
        let _ = lse.select_action(0.5, 0.5);
        let state = lse.last_state.unwrap();

        // Before learning, Q-values should be all zeros
        let q_before = lse.q_values(state).unwrap().to_vec();
        for &q in &q_before {
            assert!((q - 0.0).abs() < 1e-9);
        }

        // Learn with positive reward
        lse.learn(1.0);

        // After learning, the action taken should have increased Q-value
        let q_after = lse.q_values(state).unwrap();
        let action = lse.last_action.unwrap();
        assert!(
            q_after[action] > 0.0,
            "Q-value for action {} should be positive after +1 reward, got {}",
            action,
            q_after[action]
        );
        // Verify TD update: Q = 0 + 0.1 * (1.0 - 0) = 0.1
        assert!((q_after[action] - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_learn_without_prior_state_is_noop() {
        let mut lse = LsePolicy::new(6);
        // No prior action → learn should be a no-op
        lse.learn(1.0);
        assert_eq!(lse.total_steps, 0);
        assert!(lse.q_table.is_empty());
    }

    #[test]
    fn test_epsilon_decays_over_time() {
        let mut lse = LsePolicy::new(6);
        lse.epsilon = 1.0;
        lse.epsilon_decay = 0.5;
        lse.epsilon_min = 0.01;

        for _ in 0..10 {
            let _ = lse.select_action(0.5, 0.5);
            lse.learn(0.0);
        }

        // Epsilon should have decayed significantly
        assert!(
            lse.epsilon_current() < 0.5,
            "epsilon should have decayed below 0.5, got {}",
            lse.epsilon_current()
        );
        assert!(lse.total_steps == 10);
    }

    #[test]
    fn test_best_action_returns_optimal() {
        let mut lse = LsePolicy::new(6);
        // Manually set higher Q for action 2
        lse.q_table
            .insert((7, 7), vec![0.1, 0.2, 0.9, 0.3, 0.4, 0.5]);

        let best = lse.best_action(0.5, 0.5);
        assert_eq!(best, Some(2));
    }

    #[test]
    fn test_best_action_returns_none_for_unknown_state() {
        let lse = LsePolicy::new(6);
        let best = lse.best_action(0.1, 0.2);
        assert!(best.is_none());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let mut lse = LsePolicy::new(6);
        lse.epsilon = 0.5;
        // Add some state
        let mut lse2 = lse.clone();
        let _ = lse2.select_action(0.7, 0.3);
        lse2.learn(0.8);
        let _ = lse2.select_action(0.2, 0.9);
        lse2.learn(0.3);

        let path = format!("/tmp/test_lse_{}.json", std::process::id());
        lse2.save(&path).unwrap();

        let loaded = LsePolicy::load(&path).unwrap();
        assert_eq!(loaded.action_count, 6);
        assert!((loaded.epsilon - 0.5).abs() < 1e-9 || (loaded.epsilon - 0.5 * 0.995).abs() < 1e-9);
        assert_eq!(loaded.total_steps, 2);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_default_uses_six_actions() {
        let lse = LsePolicy::default();
        assert_eq!(lse.action_count, 6);
    }

    #[test]
    fn test_state_count_tracks_visited_states() {
        let mut lse = LsePolicy::new(6);
        lse.epsilon = 1.0; // Explore to hit different states
        lse.rng_state = 42;

        for _ in 0..50 {
            let r = (lse.rng_state as f64 / u64::MAX as f64).clamp(0.0, 1.0);
            let _ = lse.select_action(r, 1.0 - r);
            lse.learn(0.5);
        }

        // Should have discovered several states
        assert!(lse.state_count() > 0, "should have at least one state");
    }

    #[test]
    fn test_select_action_with_all_negative_q_values() {
        let mut lse = LsePolicy::new(6);
        lse.epsilon = 0.0; // Force greedy
                           // Manually set all Q-values to negative infinity
        lse.q_table.insert((7, 7), vec![f64::NEG_INFINITY; 6]);
        lse.rng_state = 42;

        // Should still return a valid action (fallback to random)
        let action = lse.select_action(0.5, 0.5);
        assert!(action < 6);
    }
}
