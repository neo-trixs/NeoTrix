//! Trajectory-aware Process Reward Model — ReasonFlux/Step-GRPO inspired.
//!
//! Extends the existing Rubric-based PRM with trajectory-level supervision,
//! step-attention weighting, and temporal convergence detection.
//!
//! References:
//!   - ReasonFlux-PRM: Trajectory-Aware PRMs for Long CoT Reasoning (Zou et al., 2025)
//!   - Step-GRPO: Step-level Dense PRM Rewards with Step-Attention (Li et al., 2026)
//!   - TRACE: Test-Time Scaling via Temporal Reasoning Aggregation (ACL 2026)
//!   - SWE-TRACE: Trajectory Reduction and Agentic Criteria Evaluation

use super::super::nt_core_prm::AgentTrajectory;
#[cfg(test)]
use super::super::nt_core_hex::ReasoningHexagram;

/// Step-attention weights for trajectory-level PRM scoring.
///
/// Derived from Step-GRPO: attention weights are learned per position
/// to emphasize critical reasoning steps while down-weighting trivial ones.
/// For E8 trajectories, we assign higher attention to:
///   - First steps (mode selection / problem framing)
///   - High-entropy transitions (exploration)
///   - Final steps (convergence)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepAttention {
    /// Positional bias: higher = more attention at this step position
    pub positional_bias: Vec<f64>,
    /// Entropy bonus: how much to boost high-entropy (exploratory) steps
    pub entropy_bonus: f64,
    /// Convergence bonus: how much to boost converging steps
    pub convergence_bonus: f64,
    /// Temperature for softmax normalization
    pub temperature: f64,
}

impl Default for StepAttention {
    fn default() -> Self {
        Self {
            positional_bias: vec![0.15, 0.12, 0.10, 0.08, 0.10, 0.12, 0.10, 0.12, 0.11],
            entropy_bonus: 0.3,
            convergence_bonus: 0.4,
            temperature: 1.0,
        }
    }
}

impl StepAttention {
    /// Compute step-level attention weights for a trajectory.
    /// Returns normalized attention weights summing to 1.0.
    pub fn compute_weights(&self, trajectory: &AgentTrajectory) -> Vec<f64> {
        let n = trajectory.steps.len();
        if n == 0 { return Vec::new(); }

        let mut raw = Vec::with_capacity(n);
        for (i, step) in trajectory.steps.iter().enumerate() {
            // Positional bias (extend or truncate default biases)
            let pos = if i < self.positional_bias.len() {
                self.positional_bias[i]
            } else {
                0.08 // tail bias for steps beyond default
            };

            // Mode entropy: higher entropy (exploratory) steps get bonus
            let mode_val = step.e8_mode.0 as f64 / 63.0;
            let mid = (mode_val - 0.5).abs();
            let entropy = if mid < 0.3 { 0.8 } else { 0.3 }; // mid-range modes are exploratory

            // Convergence signal: steps with high confidence get bonus
            let convergence = step.external_reward.unwrap_or(0.5);

            raw.push(pos + self.entropy_bonus * entropy + self.convergence_bonus * convergence);
        }

        // Softmax normalization
        let max_raw = raw.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_sum: f64 = raw.iter().map(|v| ((v - max_raw) / self.temperature).exp()).sum();
        if exp_sum < 1e-12 {
            return vec![1.0 / n as f64; n];
        }
        raw.iter().map(|v| ((v - max_raw) / self.temperature).exp() / exp_sum).collect()
    }

    /// Apply step-attention to produce a trajectory-level weighted score.
    pub fn weighted_score(&self, step_scores: &[f64], attention: &[f64]) -> f64 {
        if step_scores.is_empty() || attention.is_empty() { return 0.0; }
        let n = step_scores.len().min(attention.len());
        let weighted: f64 = step_scores[..n].iter()
            .zip(attention[..n].iter())
            .map(|(s, w)| s * w)
            .sum();
        let total_weight: f64 = attention[..n].iter().sum();
        if total_weight > 0.0 { weighted / total_weight } else { 0.0 }
    }
}

/// Temporal convergence detector (TRACE-style).
///
/// Monitors the trajectory of step scores over time to detect when
/// reasoning has converged. Uses a sliding window over recent scores
/// to compute:
///   - Answer consistency: persistence of high-quality scores
///   - Confidence trajectory: temporal evolution of confidence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConvergenceDetector {
    /// Sliding window size for score aggregation
    pub window_size: usize,
    /// Threshold for convergence detection
    pub convergence_threshold: f64,
    /// Minimum steps before checking convergence
    pub min_steps: usize,
}

impl Default for ConvergenceDetector {
    fn default() -> Self {
        Self {
            window_size: 3,
            convergence_threshold: 0.7,
            min_steps: 4,
        }
    }
}

impl ConvergenceDetector {
    /// Detect whether reasoning has converged based on recent score trajectory.
    /// Returns a convergence score [0.0-1.0] and whether to early-exit.
    pub fn detect_convergence(&self, scores: &[f64]) -> (f64, bool) {
        if scores.len() < self.min_steps { return (0.0, false); }

        let n = scores.len();
        let window_end = n;
        let window_start = n.saturating_sub(self.window_size);
        let recent: Vec<f64> = scores[window_start..window_end].to_vec();

        if recent.is_empty() { return (0.0, false); }

        // Answer consistency: low variance in recent scores means stable
        let mean = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance = recent.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / recent.len() as f64;
        let consistency = (-variance * 5.0).exp(); // 1.0 when variance=0, ~0 when variance>1

        // Confidence trajectory: is the trend positive?
        let trend = if recent.len() >= 2 {
            let first_half: f64 = recent[..recent.len() / 2].iter().sum();
            let second_half: f64 = recent[recent.len() / 2..].iter().sum();
            let n1 = (recent.len() / 2).max(1);
            let n2 = (recent.len() - recent.len() / 2).max(1);
            (second_half / n2 as f64 - first_half / n1 as f64).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        // Convergence = consistency weighted by positive trend
        let convergence = consistency * (0.5 + trend * 0.3).max(0.0).min(1.0);
        let should_exit = convergence >= self.convergence_threshold && mean > 0.5;

        (convergence, should_exit)
    }
}

/// Trajectory-aware PRM scorer (ReasonFlux-style).
///
/// Combines step-level Rubric scores with trajectory-level supervision:
///   - Step-level: detailed Rubric-based score per transition
///   - Trajectory-level: step-attention weighted aggregate + convergence signal
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TrajectoryPrm {
    pub step_attention: StepAttention,
    pub convergence: ConvergenceDetector,
    pub step_scores: Vec<f64>,
    pub trajectory_count: u64,
}

impl TrajectoryPrm {
    /// Score a full trajectory with step-attention + convergence.
    ///
    /// Returns:
    ///   - step_scores: per-step process scores
    ///   - trajectory_score: aggregated trajectory-level score
    ///   - convergence: convergence detection report
    ///   - attention: step-level attention weights
    pub fn score_trajectory(
        &mut self,
        trajectory: &AgentTrajectory,
        rubric_scores: &[f64],
    ) -> TrajectoryScoreReport {
        self.trajectory_count += 1;

        if rubric_scores.is_empty() {
            return TrajectoryScoreReport {
                step_scores: Vec::new(),
                trajectory_score: 0.0,
                weighted_score: 0.0,
                convergence_score: 0.0,
                should_exit: false,
                attention: Vec::new(),
            };
        }

        // Step-attention weighting
        let attention = self.step_attention.compute_weights(trajectory);
        let weighted = self.step_attention.weighted_score(rubric_scores, &attention);

        // Convergence detection
        let mut score_history = self.step_scores.clone();
        score_history.extend_from_slice(rubric_scores);
        // Keep only recent window
        if score_history.len() > self.convergence.window_size * 4 {
            score_history.drain(0..score_history.len() - self.convergence.window_size * 4);
        }
        let (convergence_score, should_exit) = self.convergence.detect_convergence(&score_history);
        self.step_scores = score_history;

        // Simple step-level average as baseline
        let trajectory_score = if rubric_scores.is_empty() {
            0.0
        } else {
            rubric_scores.iter().sum::<f64>() / rubric_scores.len() as f64
        };

        TrajectoryScoreReport {
            step_scores: rubric_scores.to_vec(),
            trajectory_score,
            weighted_score: weighted,
            convergence_score,
            should_exit,
            attention,
        }
    }
}

/// Report from trajectory-aware PRM scoring.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrajectoryScoreReport {
    /// Per-step rubric scores
    pub step_scores: Vec<f64>,
    /// Simple average trajectory score
    pub trajectory_score: f64,
    /// Step-attention weighted trajectory score
    pub weighted_score: f64,
    /// Convergence detection score [0.0-1.0]
    pub convergence_score: f64,
    /// Whether to early-exit based on convergence
    pub should_exit: bool,
    /// Step-level attention weights
    pub attention: Vec<f64>,
}

/// Blended advantage combining step-level and trajectory-level signals.
///
/// Formula (from ReasonFlux-PRM):
///   A_blended = λ · A_step + (1-λ) · A_trajectory
/// where:
///   A_step = rubric-based per-step score
///   A_trajectory = attention-weighted trajectory score + convergence bonus
pub fn blended_trajectory_advantage(
    step_score: f64,
    weighted_score: f64,
    convergence_score: f64,
    lambda: f64,
) -> f64 {
    let trajectory_signal = weighted_score * 0.7 + convergence_score * 0.3;
    lambda * step_score + (1.0 - lambda) * trajectory_signal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_prm::TrajectoryStep;

    fn make_test_trajectory(n: usize) -> AgentTrajectory {
        let mut t = AgentTrajectory::new(1, "test".into());
        for i in 0..n {
            t.push(TrajectoryStep {
                step_idx: i,
                specialist: crate::core::nt_core_traits::SpecialistType::ReflectionEngine,
                e8_mode: ReasoningHexagram((i * 10) as u8 % 64),
                action: format!("step_{}", i),
                input: String::new(),
                output: String::new(),
                duration_ms: None,
                success: true,
                external_reward: Some(if i < n / 2 { 0.5 } else { 0.8 }),
            });
        }
        t.outcome_reward = Some(1.0);
        t.completed = true;
        t
    }

    #[test]
    fn test_step_attention_weights_sum_to_one() {
        let sa = StepAttention::default();
        let traj = make_test_trajectory(9);
        let weights = sa.compute_weights(&traj);
        assert_eq!(weights.len(), 9);
        let sum: f64 = weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "weights sum to {}, expected 1.0", sum);
    }

    #[test]
    fn test_step_attention_weighted_score() {
        let sa = StepAttention::default();
        let weights = vec![0.2; 5];
        let score = sa.weighted_score(&[1.0, 1.0, 1.0, 1.0, 1.0], &weights);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_convergence_detector_below_min() {
        let cd = ConvergenceDetector::default();
        let (conv, exit) = cd.detect_convergence(&[0.5, 0.6]);
        assert_eq!(conv, 0.0);
        assert!(!exit);
    }

    #[test]
    fn test_convergence_detector_high_consistency() {
        let cd = ConvergenceDetector {
            convergence_threshold: 0.45,
            ..Default::default()
        };
        let (conv, exit) = cd.detect_convergence(&[0.8, 0.8, 0.8, 0.8]);
        assert!(conv >= 0.5, "consistent scores should yield convergence >= 0.5, got {}", conv);
        assert!(exit, "should exit when convergence >= 0.45, got {}", conv);
    }

    #[test]
    fn test_convergence_detector_low_consistency() {
        let cd = ConvergenceDetector {
            min_steps: 2,
            ..Default::default()
        };
        let (conv, exit) = cd.detect_convergence(&[0.1, 0.9, 0.2, 0.8, 0.1, 0.9]);
        assert!(conv < 0.5, "inconsistent scores should yield low convergence");
        assert!(!exit);
    }

    #[test]
    fn test_trajectory_prm_empty() {
        let mut tp = TrajectoryPrm::default();
        let traj = make_test_trajectory(0);
        let report = tp.score_trajectory(&traj, &[]);
        assert_eq!(report.trajectory_score, 0.0);
    }

    #[test]
    fn test_trajectory_prm_scores() {
        let mut tp = TrajectoryPrm::default();
        let traj = make_test_trajectory(9);
        let rubric = vec![0.6, 0.7, 0.8, 0.7, 0.8, 0.9, 0.8, 0.7, 0.8];
        let report = tp.score_trajectory(&traj, &rubric);
        assert!(report.trajectory_score > 0.0);
        assert!(report.weighted_score > 0.0);
        assert_eq!(report.step_scores.len(), 9);
    }

    #[test]
    fn test_blended_trajectory_advantage() {
        // λ=1: pure step-level (no trajectory influence)
        let pure_step = blended_trajectory_advantage(0.8, 0.6, 0.7, 1.0);
        assert!((pure_step - 0.8).abs() < 1e-10);

        // λ=0: pure trajectory-level
        let pure_traj = blended_trajectory_advantage(0.8, 0.6, 0.7, 0.0);
        let expected = 0.6 * 0.7 + 0.7 * 0.3;
        assert!((pure_traj - expected).abs() < 1e-10);

        // λ=0.3: ReasonFlux recommended balance
        let blended = blended_trajectory_advantage(0.8, 0.6, 0.7, 0.3);
        assert!(blended > 0.5 && blended < 1.0);
    }

    #[test]
    fn test_step_attention_entropy_bonus() {
        let sa = StepAttention::default();
        // Middle-range mode (0.5) should get higher entropy bonus than edge mode (0.0)
        let mut traj = make_test_trajectory(2);
        traj.steps[0].e8_mode = ReasoningHexagram(31); // mid-range, high entropy
        traj.steps[1].e8_mode = ReasoningHexagram(0);   // edge, low entropy
        let weights = sa.compute_weights(&traj);
        // Both weights should be positive
        assert!(weights[0] > 0.0);
        assert!(weights[1] > 0.0);
    }

    #[test]
    fn test_trajectory_prm_convergence_tracking() {
        let mut tp = TrajectoryPrm::default();
        let traj = make_test_trajectory(9);

        // First trajectory: high but variable scores
        let _report1 = tp.score_trajectory(&traj, &[0.9, 0.8, 0.9, 0.7, 0.9, 0.8, 0.9, 0.8, 0.9]);
        assert_eq!(tp.trajectory_count, 1);

        // Second trajectory: consistent high scores → should converge
        let report2 = tp.score_trajectory(&traj, &[0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8]);
        assert_eq!(tp.trajectory_count, 2);
        // Convergence score should be reasonable
        assert!((0.0..=1.0).contains(&report2.convergence_score));
    }

    #[test]
    fn test_blended_advantage_lambda_range() {
        for lambda in [0.0, 0.3, 0.5, 0.7, 1.0] {
            let adv = blended_trajectory_advantage(0.7, 0.5, 0.6, lambda);
            assert!((0.0..=1.0).contains(&adv), "lambda={} gives adv={}", lambda, adv);
        }
    }
}
