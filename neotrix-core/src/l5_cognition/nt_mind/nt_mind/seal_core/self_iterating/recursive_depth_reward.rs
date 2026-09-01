//! Phase 6.2 — Recursive Depth Reward (Thinking Pixel, arXiv:2604.25299 §3.3)
//!
//! Rewards deep-reasoning trajectories: the deeper the recursion (relative to
//! total steps) the larger the multiplicative bonus applied on top of the base
//! reward. Also exposes a depth/branching progress metric that saturates at
//! `max_depth`.

/// Recursive depth reward for the SEAL self-iteration loop.
pub struct RecursiveDepthReward {
    /// Depth at which the progress metric saturates (depth/max_depth capped at 1).
    pub max_depth: usize,
    /// Strength of the multiplicative depth bonus factor.
    pub depth_bonus: f64,
}

impl Default for RecursiveDepthReward {
    fn default() -> Self {
        Self {
            max_depth: 8,
            depth_bonus: 0.05,
        }
    }
}

/// Detailed result of evaluating the depth reward for one reasoning cycle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DepthRewardReport {
    /// Reward before the depth bonus was applied.
    pub base_reward: f64,
    /// Recursive depth of the trajectory (d_rec).
    pub depth: usize,
    /// Total reasoning steps in the cycle.
    pub total_steps: usize,
    /// Absolute bonus added by the depth factor (final - base).
    pub bonus: f64,
    /// Reward after the depth bonus was applied.
    pub final_reward: f64,
    /// Depth/branching progress in 0..=1, saturating at `max_depth`.
    pub progress: f64,
}

impl RecursiveDepthReward {
    /// Multiplicative depth reward: `reward * (1 + depth_bonus * depth/total_steps)`.
    /// A depth of 0 returns the reward unchanged.
    pub fn score(&self, depth: usize, total_steps: usize, reward: f64) -> f64 {
        if depth == 0 {
            return reward;
        }
        let ratio = depth as f64 / total_steps.max(1) as f64;
        reward * (1.0 + self.depth_bonus * ratio)
    }

    /// Progress metric in 0..=1: deeper depth with less branching approaches 1,
    /// saturating at `max_depth`.
    pub fn depth_progress(&self, current_depth: usize, branching: usize) -> f64 {
        let depth_term = (current_depth as f64 / self.max_depth.max(1) as f64).min(1.0);
        let branch_term = 1.0 / branching.max(1) as f64;
        (depth_term * branch_term).clamp(0.0, 1.0)
    }

    /// Evaluate a full depth-reward report for one reasoning cycle.
    pub fn evaluate(&self, depth: usize, total_steps: usize, branching: usize, reward: f64) -> DepthRewardReport {
        let final_reward = self.score(depth, total_steps, reward);
        DepthRewardReport {
            base_reward: reward,
            depth,
            total_steps,
            bonus: final_reward - reward,
            final_reward,
            progress: self.depth_progress(depth, branching),
        }
    }

    /// Blend a PRM (process reward) share with the base (pipeline) reward.
    /// `prm_share` is clamped to [0, 1].
    pub fn blend(&self, prm_share: f64, prm_reward: f64, base_reward: f64) -> f64 {
        let share = prm_share.clamp(0.0, 1.0);
        share * prm_reward + (1.0 - share) * base_reward
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_depth_bonus() {
        let rdr = RecursiveDepthReward { max_depth: 8, depth_bonus: 0.5 };
        // depth=2, steps=4 → ratio 0.5 → reward * (1 + 0.5*0.5) = 10 * 1.25
        assert!((rdr.score(2, 4, 10.0) - 12.5).abs() < 1e-9);
        // deeper depth relative to steps → larger bonus
        assert!(rdr.score(3, 4, 10.0) > rdr.score(2, 4, 10.0));
    }

    #[test]
    fn test_score_zero_depth_returns_reward() {
        let rdr = RecursiveDepthReward::default();
        assert_eq!(rdr.score(0, 10, 7.5), 7.5);
        assert_eq!(rdr.score(0, 0, 3.0), 3.0);
    }

    #[test]
    fn test_max_depth_saturation() {
        let rdr = RecursiveDepthReward { max_depth: 4, depth_bonus: 0.1 };
        assert_eq!(rdr.depth_progress(4, 1), 1.0);
        // beyond max_depth → still saturated at 1.0
        assert_eq!(rdr.depth_progress(400, 1), 1.0);
        // branching > 1 reduces progress
        assert!(rdr.depth_progress(4, 2) < 1.0);
    }

    #[test]
    fn test_evaluate_fields() {
        let rdr = RecursiveDepthReward { max_depth: 8, depth_bonus: 0.1 };
        let report = rdr.evaluate(2, 4, 2, 100.0);
        assert_eq!(report.base_reward, 100.0);
        assert_eq!(report.depth, 2);
        assert_eq!(report.total_steps, 4);
        // 100 * (1 + 0.1 * 0.5) = 105
        assert!((report.final_reward - 105.0).abs() < 1e-9);
        assert!((report.bonus - 5.0).abs() < 1e-9);
        assert!(report.progress > 0.0 && report.progress <= 1.0);
    }

    #[test]
    fn test_blend_boundaries() {
        let rdr = RecursiveDepthReward::default();
        // pure base reward
        assert_eq!(rdr.blend(0.0, 10.0, 2.0), 2.0);
        // pure PRM reward
        assert_eq!(rdr.blend(1.0, 10.0, 2.0), 10.0);
        // out-of-range shares are clamped
        assert_eq!(rdr.blend(2.0, 10.0, 2.0), 10.0);
        assert_eq!(rdr.blend(-1.0, 10.0, 2.0), 2.0);
        // 30/70 blend
        assert!((rdr.blend(0.3, 10.0, 2.0) - (0.3 * 10.0 + 0.7 * 2.0)).abs() < 1e-9);
    }

    #[test]
    fn test_progress_monotonic_in_depth() {
        let rdr = RecursiveDepthReward { max_depth: 8, depth_bonus: 0.1 };
        let mut prev = 0.0;
        for d in 0..=8 {
            let p = rdr.depth_progress(d, 1);
            assert!(p >= prev, "progress must be monotonic, d={} p={}", d, p);
            prev = p;
        }
    }
}
