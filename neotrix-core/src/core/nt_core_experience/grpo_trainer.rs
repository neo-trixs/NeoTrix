use std::collections::HashMap;

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

// ── Config ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GRPOConfig {
    pub clip_epsilon: f64,
    pub clip_higher: bool,
    pub dynamic_sampling: bool,
    pub group_size: usize,
    pub learning_rate: f64,
    pub beta_kl: f64,
}

impl Default for GRPOConfig {
    fn default() -> Self {
        Self {
            clip_epsilon: 0.2,
            clip_higher: true,
            dynamic_sampling: true,
            group_size: 8,
            learning_rate: 0.01,
            beta_kl: 0.001,
        }
    }
}

// ── GRPOTrainer ────────────────────────────────────────────────────────────

/// Group Relative Policy Optimization trainer.
///
/// Policy is stored as HashMap<state_hash ⊕ action_hash, logit>.
/// Each call to `train_step` computes group-relative advantages,
/// simulates the post-update probability ratio, checks clipping,
/// and applies an SGD step to the policy logit.
#[derive(Debug, Clone)]
pub struct GRPOTrainer {
    config: GRPOConfig,
    policy: HashMap<u64, f64>,
    state_action_counts: HashMap<u64, u64>,
    total_updates: u64,
    total_samples: u64,
}

impl GRPOTrainer {
    pub fn new(config: GRPOConfig) -> Self {
        Self {
            config,
            policy: HashMap::new(),
            state_action_counts: HashMap::new(),
            total_updates: 0,
            total_samples: 0,
        }
    }

    // ── Hashing helpers ────────────────────────────────────────────────────

    /// FNV-1a hash over a VSA byte slice.
    pub fn hash_vsa(v: &[u8]) -> u64 {
        let mut h = 0xcbf29ce484222325u64;
        for chunk in v.chunks(8) {
            let mut word = 0u64;
            for (i, &b) in chunk.iter().enumerate() {
                word |= (b as u64) << (i * 8);
            }
            h ^= word;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    /// Combine state_hash and action_hash into a policy lookup key.
    pub fn policy_key(state_hash: u64, action_hash: u64) -> u64 {
        state_hash.wrapping_mul(1_000_003) ^ action_hash.wrapping_add(1)
    }

    // ── Math helpers ───────────────────────────────────────────────────────

    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    fn stable_std(data: &[f64]) -> f64 {
        let n = data.len() as f64;
        if n <= 1.0 {
            return 1.0;
        }
        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
        variance.sqrt().max(1e-8)
    }

    // ── Sampling ───────────────────────────────────────────────────────────

    /// Sample `n` action VSA vectors from the current policy for a given state.
    ///
    /// Each action is a deterministic function of (state_hash, action_index)
    /// via `QuantizedVSA::seeded_random`. Returns the action vector and its
    /// log-probability under the current policy.
    pub fn sample_actions(&self, state_vsa: &[u8], n: usize) -> Vec<(Vec<u8>, f64)> {
        let state_hash = Self::hash_vsa(state_vsa);
        let n = n.min(self.config.group_size * 2);
        let mut actions = Vec::with_capacity(n);

        for i in 0..n {
            let seed = state_hash
                .wrapping_add(i as u64)
                .wrapping_mul(6_364_136_223_846_793_005);
            let action_vsa = QuantizedVSA::seeded_random(seed, VSA_DIM);
            let action_hash = Self::hash_vsa(&action_vsa);
            let key = Self::policy_key(state_hash, action_hash);
            let logit = self.policy.get(&key).copied().unwrap_or(0.0);
            let prob = Self::sigmoid(logit);
            let log_prob = if prob < 1e-30 {
                -1e30_f64.ln()
            } else {
                prob.ln()
            };
            actions.push((action_vsa, log_prob));
        }
        actions
    }

    // ── Training ───────────────────────────────────────────────────────────

    /// Perform one GRPO update step.
    ///
    /// 1. Normalise rewards to group-relative advantages.
    /// 2. For each action, compute the policy gradient.
    /// 3. Simulate the post-update probability ratio.
    /// 4. Apply clipped surrogate (DAPO-style if `clip_higher`).
    /// 5. Skip near-zero-loss samples if `dynamic_sampling`.
    /// 6. Apply SGD: logit += lr * gradient.
    pub fn train_step(&mut self, state_vsa: &[u8], actions: &[Vec<u8>], rewards: &[f64]) {
        let n = actions.len().min(rewards.len());
        if n == 0 {
            return;
        }

        let state_hash = Self::hash_vsa(state_vsa);

        // Group-relative advantage
        let mean = rewards.iter().sum::<f64>() / n as f64;
        let std = Self::stable_std(rewards);
        let advantages: Vec<f64> = rewards.iter().map(|r| (r - mean) / std).collect();

        for i in 0..n {
            let action_hash = Self::hash_vsa(&actions[i]);
            let key = Self::policy_key(state_hash, action_hash);
            let old_logit = self.policy.get(&key).copied().unwrap_or(0.0);
            let prob = Self::sigmoid(old_logit);
            let advantage = advantages[i];

            // Dynamic sampling: skip samples with negligible advantage
            if self.config.dynamic_sampling && advantage.abs() < 1e-8 {
                continue;
            }

            // Policy gradient at current policy (ratio = 1):
            //   d/dθ [-A * π_new/π_old] = -A * (1 / π_old) * π_new * (1-π_new)
            //   At π_new = π_old:         = -A * (1 - π)
            let raw_grad = -advantage * prob * (1.0 - prob);

            // Simulate what the new logit and probability would be,
            // then compute the ratio π_new / π_old for clipping.
            let new_logit = old_logit - self.config.learning_rate * raw_grad;
            let new_prob = Self::sigmoid(new_logit);
            let ratio = if prob > 1e-30 {
                new_prob / prob
            } else {
                1.0 + self.config.clip_epsilon
            };

            // Clipping decision
            let clipped = if advantage >= 0.0 {
                ratio > 1.0 + self.config.clip_epsilon
            } else if !self.config.clip_higher {
                // Standard PPO: clip lower when A < 0 and ratio < 1-ε
                ratio < 1.0 - self.config.clip_epsilon
            } else {
                // DAPO: never clip lower, only upper
                false
            };

            if clipped {
                continue;
            }

            let updated_logit = old_logit - self.config.learning_rate * raw_grad;
            self.policy.insert(key, updated_logit);
            *self.state_action_counts.entry(key).or_insert(0) += 1;
            self.total_samples += 1;
        }

        self.total_updates += 1;
    }

    // ── Query ──────────────────────────────────────────────────────────────

    /// Probability that the policy takes a specific action in a given state.
    pub fn get_action_probability(&self, state_vsa: &[u8], action_vsa: &[u8]) -> f64 {
        let state_hash = Self::hash_vsa(state_vsa);
        let action_hash = Self::hash_vsa(action_vsa);
        let key = Self::policy_key(state_hash, action_hash);
        let logit = self.policy.get(&key).copied().unwrap_or(0.0);
        Self::sigmoid(logit)
    }

    // ── Accessors ──────────────────────────────────────────────────────────

    pub fn config(&self) -> &GRPOConfig {
        &self.config
    }

    pub fn total_updates(&self) -> u64 {
        self.total_updates
    }

    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }

    pub fn policy_size(&self) -> usize {
        self.policy.len()
    }

    pub fn policy(&self) -> &HashMap<u64, f64> {
        &self.policy
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> Vec<u8> {
        QuantizedVSA::seeded_random(42, VSA_DIM)
    }

    #[test]
    fn test_trainer_initializes() {
        let t = GRPOTrainer::new(GRPOConfig::default());
        assert_eq!(t.total_updates(), 0);
        assert_eq!(t.total_samples(), 0);
        assert_eq!(t.policy_size(), 0);
        assert_eq!(t.config().group_size, 8);
    }

    #[test]
    fn test_sample_actions_returns_n() {
        let t = GRPOTrainer::new(GRPOConfig::default());
        let state = make_state();
        let actions = t.sample_actions(&state, 5);
        assert_eq!(actions.len(), 5);
        for (vsa, lp) in &actions {
            assert_eq!(vsa.len(), VSA_DIM);
            assert!(lp.is_finite());
        }
    }

    #[test]
    fn test_train_step_updates_policy() {
        let mut t = GRPOTrainer::new(GRPOConfig::default());
        let state = make_state();
        let actions = t.sample_actions(&state, 3);

        let rewards = vec![0.2, 0.5, 0.8];
        let old_logits: Vec<f64> = actions
            .iter()
            .map(|(a, _)| {
                let h = GRPOTrainer::hash_vsa(a);
                let k = GRPOTrainer::policy_key(GRPOTrainer::hash_vsa(&state), h);
                t.policy().get(&k).copied().unwrap_or(0.0)
            })
            .collect();

        t.train_step(
            &state,
            &actions.iter().map(|(a, _)| a.clone()).collect::<Vec<_>>(),
            &rewards,
        );

        assert_eq!(t.total_updates(), 1);
        assert_eq!(t.total_samples(), 3);

        // Logits should have changed
        for (i, (a, _)) in actions.iter().enumerate() {
            let h = GRPOTrainer::hash_vsa(a);
            let k = GRPOTrainer::policy_key(GRPOTrainer::hash_vsa(&state), h);
            let new_logit = t.policy().get(&k).copied().unwrap_or(0.0);
            // Higher-reward actions (idx 2: 0.8) should increase logit,
            // lower-reward (idx 0: 0.2) should decrease
            if i == 2 {
                assert!(new_logit > old_logits[i] - 1e-10);
            }
        }
    }

    #[test]
    fn test_group_advantage_computation() {
        let mut t = GRPOTrainer::new(GRPOConfig::default());
        let state = make_state();
        let actions = t.sample_actions(&state, 3);

        // The advantage is computed internally, so we test indirectly:
        // rewards = [0.1, 0.5, 0.9] → advantages ≈ [-1.0, 0.0, 1.0]
        // Action 0 (lowest reward) should have its logit decrease
        // Action 2 (highest reward) should have its logit increase
        let rewards = vec![0.1, 0.5, 0.9];
        let old_logits: Vec<f64> = actions
            .iter()
            .map(|(a, _)| {
                let h = GRPOTrainer::hash_vsa(a);
                let k = GRPOTrainer::policy_key(GRPOTrainer::hash_vsa(&state), h);
                t.policy().get(&k).copied().unwrap_or(0.0)
            })
            .collect();

        t.train_step(
            &state,
            &actions.iter().map(|(a, _)| a.clone()).collect::<Vec<_>>(),
            &rewards,
        );

        for (i, (a, _)) in actions.iter().enumerate() {
            let h = GRPOTrainer::hash_vsa(a);
            let k = GRPOTrainer::policy_key(GRPOTrainer::hash_vsa(&state), h);
            let new_logit = t.policy().get(&k).copied().unwrap_or(0.0);

            if i == 0 {
                // Lowest reward → negative advantage → logit decreases
                assert!(new_logit <= old_logits[i] + 1e-10);
            } else if i == 2 {
                // Highest reward → positive advantage → logit increases
                assert!(new_logit >= old_logits[i] - 1e-10);
            }
        }
    }

    #[test]
    fn test_clip_higher_does_not_clip_low() {
        // DAPO: with clip_higher = true, a very low ratio (π_new << π_old)
        // combined with negative advantage should NOT be clipped.
        let mut config = GRPOConfig::default();
        config.clip_higher = true;
        config.learning_rate = 100.0; // Amplify to create a measurable ratio deviation
        let mut t = GRPOTrainer::new(config);

        let state = make_state();
        let action = QuantizedVSA::seeded_random(100, VSA_DIM);
        let action_hash = GRPOTrainer::hash_vsa(&action);
        let key = GRPOTrainer::policy_key(GRPOTrainer::hash_vsa(&state), action_hash);

        // Insert a moderate logit (π ≈ 0.5) so gradient is non-vanishing
        t.policy.insert(key, 0.0);

        // Train with a very negative reward → advantage < 0 → ratio will drop below 1-ε
        // With DAPO clip_higher=true, this should NOT be clipped
        t.train_step(&state, &[action.clone()], &[-5.0]);

        // Clipped = gradient zeroed = logit unchanged at 0.0
        // Unclipped = logit should decrease (advantage < 0 reduces prob)
        let final_logit = t.policy.get(&key).copied().unwrap_or(0.0);
        assert!(
            final_logit < -0.01,
            "clip_higher=true should allow logit to decrease; got {final_logit}"
        );
    }

    #[test]
    fn test_clip_lower_does_clip_when_enabled() {
        // Standard PPO: with clip_higher = false, a very low ratio (π_new << π_old)
        // combined with negative advantage SHOULD be clipped.
        let mut config = GRPOConfig::default();
        config.clip_higher = false;
        config.learning_rate = 100.0;
        let mut t = GRPOTrainer::new(config);

        let state = make_state();
        let action = QuantizedVSA::seeded_random(200, VSA_DIM);
        let action_hash = GRPOTrainer::hash_vsa(&action);
        let key = GRPOTrainer::policy_key(GRPOTrainer::hash_vsa(&state), action_hash);

        t.policy.insert(key, 0.0);

        t.train_step(&state, &[action.clone()], &[-5.0]);

        let final_logit = t.policy.get(&key).copied().unwrap_or(0.0);
        assert!(
            (final_logit - 0.0).abs() < 1e-10,
            "clip_higher=false should clip low ratio; logit should stay at 0.0, got {final_logit}"
        );
    }

    #[test]
    fn test_dynamic_sampling_skips_neutral() {
        // When all rewards are identical, advantages are all ~0,
        // so dynamic sampling should skip all samples.
        let mut config = GRPOConfig::default();
        config.dynamic_sampling = true;
        config.learning_rate = 1.0;
        let mut t = GRPOTrainer::new(config);

        let state = make_state();
        let actions = t.sample_actions(&state, 4);
        let rewards = vec![1.0, 1.0, 1.0, 1.0]; // All identical → advantages = 0

        let action_vecs: Vec<Vec<u8>> = actions.iter().map(|(a, _)| a.clone()).collect();
        t.train_step(&state, &action_vecs, &rewards);

        // All samples skipped → total_samples stays 0
        assert_eq!(t.total_samples(), 0);
        assert_eq!(t.total_updates(), 1);
    }

    #[test]
    fn test_get_action_probability() {
        let t = GRPOTrainer::new(GRPOConfig::default());
        let state = make_state();
        let action = QuantizedVSA::seeded_random(50, VSA_DIM);

        let prob = t.get_action_probability(&state, &action);

        // Default logit = 0 → sigmoid(0) = 0.5
        assert!((prob - 0.5).abs() < 1e-10);
        assert!(prob.is_finite());
        assert!(prob > 0.0 && prob < 1.0);
    }

    #[test]
    fn test_multi_step_convergence() {
        // Repeatedly train same state-action with positive reward
        // → logit should monotonically increase
        let mut config = GRPOConfig::default();
        config.dynamic_sampling = false;
        config.learning_rate = 0.1;
        let mut t = GRPOTrainer::new(config);

        let state = make_state();
        let action = QuantizedVSA::seeded_random(77, VSA_DIM);
        let action_hash = GRPOTrainer::hash_vsa(&action);
        let key = GRPOTrainer::policy_key(GRPOTrainer::hash_vsa(&state), action_hash);

        let mut prev = 0.0_f64;
        for _ in 0..10 {
            t.train_step(&state, &[action.clone()], &[1.0]);
            let cur = t.policy.get(&key).copied().unwrap_or(0.0);
            assert!(cur >= prev - 1e-12, "logit should increase: {cur} < {prev}");
            prev = cur;
        }
        assert!(
            prev > 0.5,
            "logit should have increased significantly: {prev}"
        );
    }

    #[test]
    fn test_policy_key_determinism() {
        let h1 = GRPOTrainer::policy_key(42, 100);
        let h2 = GRPOTrainer::policy_key(42, 100);
        assert_eq!(h1, h2);

        let h3 = GRPOTrainer::policy_key(42, 101);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_hash_vsa_determinism() {
        let v = QuantizedVSA::seeded_random(10, VSA_DIM);
        let h1 = GRPOTrainer::hash_vsa(&v);
        let h2 = GRPOTrainer::hash_vsa(&v);
        assert_eq!(h1, h2);
    }
}
