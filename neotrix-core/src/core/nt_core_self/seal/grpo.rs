#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use serde::{Deserialize, Serialize};

use super::self_edit_gen::SelfEdit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpoConfig {
    pub learning_rate: f64,
    pub epsilon_clip: f64,
    pub kl_beta: f64,
    pub mini_batch_size: usize,
    pub epochs: usize,
}

impl Default for GrpoConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            epsilon_clip: 0.2,
            kl_beta: 0.01,
            mini_batch_size: 32,
            epochs: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpoReport {
    pub policy_loss: f64,
    pub kl_divergence: f64,
    pub approx_reward: f64,
    pub entropy: f64,
    pub grad_norm: f64,
}

pub struct GRPOLoop {
    pub policy: Vec<f64>,
    pub old_policy: Vec<f64>,
    pub config: GrpoConfig,
    step_count: u64,
}

fn simple_lcg(seed: u64) -> impl Iterator<Item = u64> {
    let mut state = seed;
    std::iter::from_fn(move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        Some(state)
    })
}

impl GRPOLoop {
    pub fn new(config: GrpoConfig, param_size: usize) -> Self {
        let mut rng = simple_lcg(42);
        let policy: Vec<f64> = (0..param_size)
            .map(|_| (rng.next().unwrap_or(0) % 1000) as f64 / 2000.0)
            .collect();
        let old_policy = policy.clone();
        Self {
            policy,
            old_policy,
            config,
            step_count: 0,
        }
    }

    pub fn update(
        &mut self,
        rewards: &[f64],
        old_log_probs: &[f64],
        new_log_probs: &[f64],
        advantages: &[f64],
    ) -> GrpoReport {
        let n = rewards
            .len()
            .min(old_log_probs.len())
            .min(new_log_probs.len())
            .min(advantages.len());
        if n == 0 {
            return GrpoReport {
                policy_loss: 0.0,
                kl_divergence: 0.0,
                approx_reward: rewards.iter().sum::<f64>().max(0.0),
                entropy: 0.0,
                grad_norm: 0.0,
            };
        }
        let eps = self.config.epsilon_clip;
        let beta = self.config.kl_beta;
        let mut total_policy_loss = 0.0;
        let mut total_kl = 0.0;
        let mut total_entropy = 0.0;
        let mut grad_norm = 0.0;
        for i in 0..n {
            let ratio = (new_log_probs[i] - old_log_probs[i]).exp();
            let clipped_ratio = ratio.max(1.0 - eps).min(1.0 + eps);
            let surr = ratio * advantages[i];
            let surr_clipped = clipped_ratio * advantages[i];
            let loss = -surr.min(surr_clipped)
                + beta * Self::kl_divergence(&[old_log_probs[i].exp()], &[new_log_probs[i].exp()]);
            total_policy_loss += loss;
            total_kl += (new_log_probs[i] - old_log_probs[i]).abs();
            let p = new_log_probs[i].exp().max(1e-10);
            total_entropy -= p * p.ln();
            grad_norm += loss * loss;
        }
        let count = n as f64;
        self.old_policy = self.policy.clone();
        for i in 0..self.policy.len().min(n) {
            let delta =
                advantages[i.min(advantages.len().saturating_sub(1))] * self.config.learning_rate;
            self.policy[i] = (self.policy[i] + delta).max(0.0).min(1.0);
        }
        self.step_count += 1;
        GrpoReport {
            policy_loss: total_policy_loss / count,
            kl_divergence: total_kl / count,
            approx_reward: rewards.iter().sum::<f64>() / count,
            entropy: total_entropy / count,
            grad_norm: (grad_norm / count).sqrt(),
        }
    }

    pub fn evaluate(&self, edit: &SelfEdit) -> f64 {
        let base = edit.confidence * 0.4 + edit.expected_improvement * 0.3;
        let policy_factor =
            self.policy.iter().take(5).sum::<f64>() / self.policy.len().max(1) as f64;
        (base + policy_factor * 0.3).max(0.0).min(1.0)
    }

    pub fn compute_advantage(rewards: &[f64], values: &[f64]) -> Vec<f64> {
        let n = rewards.len().min(values.len());
        if n == 0 {
            return Vec::new();
        }
        rewards
            .iter()
            .zip(values.iter())
            .map(|(r, v)| r - v)
            .collect()
    }

    pub fn kl_divergence(p: &[f64], q: &[f64]) -> f64 {
        let n = p.len().min(q.len());
        if n == 0 {
            return 0.0;
        }
        let mut kl = 0.0;
        for i in 0..n {
            let pi = p[i].max(1e-10);
            let qi = q[i].max(1e-10);
            kl += pi * (pi / qi).ln();
        }
        kl / n as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EditType;

    #[test]
    fn test_config_default() {
        let cfg = GrpoConfig::default();
        assert!((cfg.learning_rate - 0.001).abs() < 1e-6);
        assert!((cfg.epsilon_clip - 0.2).abs() < 1e-6);
        assert!((cfg.kl_beta - 0.01).abs() < 1e-6);
        assert_eq!(cfg.mini_batch_size, 32);
        assert_eq!(cfg.epochs, 3);
    }

    #[test]
    fn test_new_creates_policy() {
        let cfg = GrpoConfig::default();
        let loop_ = GRPOLoop::new(cfg, 100);
        assert_eq!(loop_.policy.len(), 100);
        assert_eq!(loop_.old_policy.len(), 100);
    }

    #[test]
    fn test_new_policy_deterministic() {
        let cfg = GrpoConfig::default();
        let a = GRPOLoop::new(cfg.clone(), 50);
        let b = GRPOLoop::new(cfg, 50);
        assert_eq!(a.policy, b.policy);
    }

    #[test]
    fn test_update_returns_report() {
        let cfg = GrpoConfig {
            learning_rate: 0.01,
            epsilon_clip: 0.2,
            kl_beta: 0.01,
            mini_batch_size: 4,
            epochs: 1,
        };
        let mut loop_ = GRPOLoop::new(cfg, 5);
        let rewards = vec![1.0, 0.5, 0.0, -0.5];
        let old_log_probs = vec![-0.5, -0.3, -0.1, -0.7];
        let new_log_probs = vec![-0.4, -0.25, -0.15, -0.6];
        let advantages = vec![0.8, 0.3, -0.2, -0.6];
        let report = loop_.update(&rewards, &old_log_probs, &new_log_probs, &advantages);
        assert!(!report.policy_loss.is_nan());
        assert!(report.kl_divergence >= 0.0);
    }

    #[test]
    fn test_update_empty_returns_zero_loss() {
        let cfg = GrpoConfig::default();
        let mut loop_ = GRPOLoop::new(cfg, 5);
        let report = loop_.update(&[], &[], &[], &[]);
        assert!((report.policy_loss - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_update_modifies_policy() {
        let cfg = GrpoConfig {
            learning_rate: 0.1,
            ..Default::default()
        };
        let mut loop_ = GRPOLoop::new(cfg, 3);
        let old = loop_.policy.clone();
        loop_.update(&[1.0], &[-0.5], &[-0.4], &[1.0]);
        assert!(loop_
            .policy
            .iter()
            .zip(old.iter())
            .any(|(a, b)| (a - b).abs() > 1e-9));
    }

    #[test]
    fn test_update_increments_step() {
        let cfg = GrpoConfig::default();
        let mut loop_ = GRPOLoop::new(cfg, 5);
        assert_eq!(loop_.step_count, 0);
        loop_.update(&[1.0], &[-0.5], &[-0.4], &[0.5]);
        assert_eq!(loop_.step_count, 1);
    }

    #[test]
    fn test_evaluate_returns_bounded() {
        let cfg = GrpoConfig::default();
        let loop_ = GRPOLoop::new(cfg, 10);
        let edit = SelfEdit {
            id: 0,
            edit_type: EditType::Fix,
            target_location: "x".into(),
            original_text: "a".into(),
            proposed_text: "b".into(),
            confidence: 0.8,
            expected_improvement: 0.6,
        };
        let score = loop_.evaluate(&edit);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_evaluate_higher_confidence_higher_score() {
        let cfg = GrpoConfig::default();
        let loop_ = GRPOLoop::new(cfg, 10);
        let low = SelfEdit {
            id: 0,
            edit_type: EditType::Fix,
            target_location: "x".into(),
            original_text: "a".into(),
            proposed_text: "b".into(),
            confidence: 0.2,
            expected_improvement: 0.2,
        };
        let high = SelfEdit {
            id: 1,
            edit_type: EditType::Fix,
            target_location: "x".into(),
            original_text: "a".into(),
            proposed_text: "b".into(),
            confidence: 0.9,
            expected_improvement: 0.9,
        };
        assert!(loop_.evaluate(&high) > loop_.evaluate(&low));
    }

    #[test]
    fn test_compute_advantage_basic() {
        let rewards = vec![1.0, 0.5, 0.0];
        let values = vec![0.8, 0.4, 0.1];
        let adv = GRPOLoop::compute_advantage(&rewards, &values);
        assert_eq!(adv.len(), 3);
        assert!((adv[0] - 0.2).abs() < 1e-6);
        assert!((adv[1] - 0.1).abs() < 1e-6);
        assert!((adv[2] - (-0.1)).abs() < 1e-6);
    }

    #[test]
    fn test_compute_advantage_empty() {
        let adv = GRPOLoop::compute_advantage(&[], &[]);
        assert!(adv.is_empty());
    }

    #[test]
    fn test_kl_divergence_same() {
        let p = vec![0.5, 0.5];
        let q = vec![0.5, 0.5];
        let kl = GRPOLoop::kl_divergence(&p, &q);
        assert!(kl.abs() < 1e-6);
    }

    #[test]
    fn test_kl_divergence_different() {
        let p = vec![0.9, 0.1];
        let q = vec![0.5, 0.5];
        let kl = GRPOLoop::kl_divergence(&p, &q);
        assert!(kl > 0.0);
    }

    #[test]
    fn test_kl_divergence_empty() {
        let kl = GRPOLoop::kl_divergence(&[], &[]);
        assert!((kl - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_report_has_entropy() {
        let cfg = GrpoConfig::default();
        let mut loop_ = GRPOLoop::new(cfg, 5);
        let report = loop_.update(&[0.5; 4], &[-0.5; 4], &[-0.3; 4], &[0.2; 4]);
        assert!(report.entropy >= 0.0);
    }

    #[test]
    fn test_update_kl_penalty() {
        let mut loop_ = GRPOLoop::new(
            GrpoConfig {
                kl_beta: 10.0,
                ..Default::default()
            },
            5,
        );
        let report = loop_.update(&[1.0], &[-2.0], &[-0.1], &[0.5]);
        assert!(report.kl_divergence > 0.0);
    }
}
