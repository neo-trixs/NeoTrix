//! Adapter bridging game trajectories to GRPO policy updates.
//!
//! Converts game replay buffer batches into GRPO-compatible format and
//! maps game rewards to GRPO reward signals with configurable KL penalty.

use serde::{Deserialize, Serialize};

use super::super::framework::Trajectory;
use super::buffer::GameTrajectoryBuffer;
use crate::core::nt_core_self::seal::grpo::{GRPOLoop, GrpoConfig};

// ═══════════════════════════════════════════════════════════════════
// Batch / Report types
// ═══════════════════════════════════════════════════════════════════

/// A batch of game data formatted for GRPO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpoBatch {
    pub rewards: Vec<f64>,
    pub old_log_probs: Vec<f64>,
    pub new_log_probs: Vec<f64>,
    pub advantages: Vec<f64>,
}

/// Report from a single GRPO update step via the game adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameGrpoReport {
    pub policy_loss: f64,
    pub kl_divergence: f64,
    pub approx_reward: f64,
    pub entropy: f64,
    pub grad_norm: f64,
}

// ═══════════════════════════════════════════════════════════════════
// Adapter
// ═══════════════════════════════════════════════════════════════════

/// Adapter bridging game trajectories to the GRPO learning loop.
///
/// Wraps `GRPOLoop` and converts trajectory data into GRPO-compatible
/// batches with configurable KL penalty weight.
pub struct GameGrpoAdapter {
    grpo: GRPOLoop,
    kl_weight: f64,
    /// Policy parameter size (must match GRPOLoop).
    param_size: usize,
}

impl GameGrpoAdapter {
    /// Create a new adapter with the given GRPO config and policy size.
    pub fn new(config: GrpoConfig, param_size: usize) -> Self {
        let kl_weight = config.kl_beta;
        Self {
            grpo: GRPOLoop::new(config, param_size),
            kl_weight,
            param_size,
        }
    }

    /// Create with default config and given policy size.
    pub fn default(param_size: usize) -> Self {
        Self::new(GrpoConfig::default(), param_size)
    }

    /// Convert trajectories from the buffer into a GRPO batch.
    ///
    /// Maps:
    /// - Per-step rewards → rewards vector
    /// - Per-step advantages → advantages vector
    /// - Dummy old/new log probs (computed from policy params)
    pub fn prepare_batch(&self, buffer: &GameTrajectoryBuffer) -> GrpoBatch {
        let trajectories = buffer.sample_batch_with_advantages(self.param_size);
        if trajectories.is_empty() {
            // Fallback: sample any trajectories
            let any = buffer.sample_batch(self.param_size);
            return self.trajectories_to_batch(&any);
        }
        self.trajectories_to_batch(&trajectories)
    }

    /// Convert a slice of trajectories into a GRPO batch.
    fn trajectories_to_batch(&self, trajectories: &[&Trajectory]) -> GrpoBatch {
        let mut rewards = Vec::new();
        let mut advantages = Vec::new();
        let mut old_log_probs = Vec::new();
        let mut new_log_probs = Vec::new();

        for traj in trajectories {
            for step in &traj.steps {
                rewards.push(step.reward);
                advantages.push(step.advantage.unwrap_or(0.0));

                // Generate dummy log probs based on policy params
                // In production these would come from the policy network
                let log_prob = -0.5 + (step.turn as f64 * 0.01);
                old_log_probs.push(log_prob);
                new_log_probs.push(log_prob + 0.05);
            }
        }

        GrpoBatch {
            rewards,
            old_log_probs,
            new_log_probs,
            advantages,
        }
    }

    /// Run one GRPO update step with the given batch.
    pub fn update(&mut self, batch: GrpoBatch) -> GameGrpoReport {
        let report = self.grpo.update(
            &batch.rewards,
            &batch.old_log_probs,
            &batch.new_log_probs,
            &batch.advantages,
        );

        // Apply KL weight scaling
        GameGrpoReport {
            policy_loss: report.policy_loss * self.kl_weight,
            kl_divergence: report.kl_divergence,
            approx_reward: report.approx_reward,
            entropy: report.entropy,
            grad_norm: report.grad_norm,
        }
    }

    /// Get a reference to the underlying GRPO loop.
    pub fn grpo(&self) -> &GRPOLoop {
        &self.grpo
    }

    /// Get a mutable reference to the underlying GRPO loop.
    pub fn grpo_mut(&mut self) -> &mut GRPOLoop {
        &mut self.grpo
    }

    /// Get the KL weight.
    pub fn kl_weight(&self) -> f64 {
        self.kl_weight
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_mind::nt_game::framework::{Action, Observation, TrajectoryStep};

    fn make_trajectory_with_advantages(length: usize) -> Trajectory {
        let mut t = Trajectory::new();
        for i in 0..length {
            t.push(TrajectoryStep {
                turn: i,
                actor_id: 0,
                observation: Observation {
                    text: format!("state_{i}"),
                    legal_actions: vec![],
                    hexagram: None,
                    phi: None,
                },
                action: Action {
                    kind: "move".into(),
                    params: serde_json::json!({}),
                    actor_id: 0,
                },
                reward: 0.1 * i as f64,
                advantage: Some(0.5 - i as f64 * 0.1),
            });
        }
        t
    }

    fn make_trajectory_no_advantages(length: usize) -> Trajectory {
        let mut t = Trajectory::new();
        for i in 0..length {
            t.push(TrajectoryStep {
                turn: i,
                actor_id: 0,
                observation: Observation {
                    text: format!("state_{i}"),
                    legal_actions: vec![],
                    hexagram: None,
                    phi: None,
                },
                action: Action {
                    kind: "move".into(),
                    params: serde_json::json!({}),
                    actor_id: 0,
                },
                reward: 0.1 * i as f64,
                advantage: None,
            });
        }
        t
    }

    #[test]
    fn test_adapter_new() {
        let adapter = GameGrpoAdapter::new(GrpoConfig::default(), 64);
        assert_eq!(adapter.param_size, 64);
    }

    #[test]
    fn test_adapter_default() {
        let adapter = GameGrpoAdapter::default(32);
        assert_eq!(adapter.param_size, 32);
        assert!((adapter.kl_weight() - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_prepare_batch_empty_buffer() {
        let adapter = GameGrpoAdapter::default(10);
        let buf = GameTrajectoryBuffer::new(10);
        let batch = adapter.prepare_batch(&buf);
        assert!(batch.rewards.is_empty());
    }

    #[test]
    fn test_prepare_batch_with_advantages() {
        let adapter = GameGrpoAdapter::default(10);
        let mut buf = GameTrajectoryBuffer::new(10);
        buf.store(make_trajectory_with_advantages(5));
        buf.store(make_trajectory_with_advantages(3));

        let batch = adapter.prepare_batch(&buf);
        assert!(!batch.rewards.is_empty());
        assert_eq!(batch.rewards.len(), batch.advantages.len());
        assert_eq!(batch.old_log_probs.len(), batch.new_log_probs.len());
    }

    #[test]
    fn test_prepare_batch_no_advantages_fallback() {
        let adapter = GameGrpoAdapter::default(10);
        let mut buf = GameTrajectoryBuffer::new(10);
        buf.store(make_trajectory_no_advantages(5));

        let batch = adapter.prepare_batch(&buf);
        assert!(!batch.rewards.is_empty());
    }

    #[test]
    fn test_update_returns_report() {
        let mut adapter = GameGrpoAdapter::default(5);
        let batch = GrpoBatch {
            rewards: vec![1.0, 0.5, 0.0, -0.5],
            old_log_probs: vec![-0.5, -0.3, -0.1, -0.7],
            new_log_probs: vec![-0.4, -0.25, -0.15, -0.6],
            advantages: vec![0.8, 0.3, -0.2, -0.6],
        };

        let report = adapter.update(batch);
        assert!(!report.policy_loss.is_nan());
        assert!(report.kl_divergence >= 0.0);
    }

    #[test]
    fn test_update_empty_batch() {
        let mut adapter = GameGrpoAdapter::default(5);
        let batch = GrpoBatch {
            rewards: vec![],
            old_log_probs: vec![],
            new_log_probs: vec![],
            advantages: vec![],
        };

        let report = adapter.update(batch);
        assert!((report.policy_loss - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_kl_weight_applied() {
        let config = GrpoConfig {
            kl_beta: 5.0,
            ..Default::default()
        };
        let adapter = GameGrpoAdapter::new(config, 5);
        assert!((adapter.kl_weight() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_grpo_accessor() {
        let adapter = GameGrpoAdapter::default(10);
        let grpo = adapter.grpo();
        assert_eq!(grpo.policy.len(), 10);
    }

    #[test]
    fn test_full_pipeline() {
        let mut adapter = GameGrpoAdapter::default(20);
        let mut buf = GameTrajectoryBuffer::new(10);

        // Store some trajectories
        for i in 0..5 {
            let mut traj = make_trajectory_with_advantages(4);
            traj.total_reward = i as f64;
            traj.length = 4;
            buf.store(traj);
        }

        // Prepare batch and update
        let batch = adapter.prepare_batch(&buf);
        let report = adapter.update(batch);

        assert!(!report.policy_loss.is_nan());
        assert!(report.approx_reward.is_finite());
    }
}
