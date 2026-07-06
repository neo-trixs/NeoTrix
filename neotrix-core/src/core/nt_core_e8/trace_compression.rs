//! E₈ trajectory trace compression.
//!
//! Inspired by SWE-TRACE (Trajectory Reduction and Agentic Criteria Evaluation):
//! compresses E8 state trajectories by removing redundant states, pruning
//! oscillations, and keeping only "decision points" — transitions where the
//! reasoning actually changes direction or depth.
//!
//! Also supports CoRD-style (Collaborative Reasoning Decoding) step selection
//! via PRM-scored trajectory pruning.

use super::E8TransitionMatrix;
use serde::{Deserialize, Serialize};

/// Compression strategy for E8 trajectories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// Keep only states that change the block (top 3 bits)
    BlockLevel,
    /// Keep states where oscillation is not detected
    Deoscillate,
    /// Keep only "decision points" — states where the 6-bit pattern
    /// changes more than 1 axis from the previous
    DecisionPoints,
    /// Full SWE-TRACE: block-level + deoscillate + decision points
    SweTrace,
    /// No compression
    None,
}

/// Result of compressing an E8 trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedTrajectory {
    /// The compressed state sequence
    pub states: Vec<u8>,
    /// Original length before compression
    pub original_len: usize,
    /// Compression ratio (1.0 = no compression)
    pub ratio: f64,
    /// Number of oscillations removed
    pub oscillations_removed: usize,
    /// Number of redundant states removed
    pub redundant_removed: usize,
    /// Keep indices mapping from compressed → original positions
    pub keep_indices: Vec<usize>,
}

impl CompressedTrajectory {
    /// True if compression was meaningful (>10% reduction).
    pub fn is_significant(&self) -> bool {
        self.ratio > 1.1
    }

    /// Get the original positions that are "decision points".
    pub fn decision_points(&self) -> &[usize] {
        &self.keep_indices
    }

    /// Expand compressed trajectory back to original indices with interpolation.
    pub fn expand(&self) -> Vec<u8> {
        if self.states.is_empty() {
            return Vec::new();
        }
        if self.states.len() == 1 || self.original_len <= self.states.len() {
            return self.states.clone();
        }
        let mut expanded = Vec::with_capacity(self.original_len);
        for i in 0..self.original_len {
            // Find the nearest kept state
            let nearest = self.keep_indices.iter()
                .enumerate()
                .min_by_key(|(_, &k)| k.abs_diff(i))
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            expanded.push(self.states[nearest.min(self.states.len() - 1)]);
        }
        expanded
    }
}

/// E8 trajectory compression engine.
///
/// Compresses reasoning trajectories by detecting and removing:
/// 1. Oscillations (ABAB patterns wasting compute)
/// 2. Redundant states (consecutive same-block states)
/// 3. Low-information transitions (adjacent states differing by 1 axis)
///
/// Alignment with SWE-TRACE: the compression preserves "decision points"
/// where the reasoning trajectory makes meaningful changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8TraceCompressor {
    /// Whether to log compression statistics
    pub verbose: bool,
    /// Minimum block transition distance to keep (block change = keep)
    pub block_threshold: usize,
    /// Minimum per-axis Hamming distance to keep (for decision points)
    pub axis_threshold: usize,
    /// Whether to also compress via the transition matrix
    pub matrix_aware: bool,
}

impl Default for E8TraceCompressor {
    fn default() -> Self {
        Self {
            verbose: false,
            block_threshold: 1,
            axis_threshold: 2,
            matrix_aware: true,
        }
    }
}

impl E8TraceCompressor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compress a trajectory using the specified strategy.
    pub fn compress(
        &self,
        trajectory: &[u8],
        strategy: CompressionStrategy,
    ) -> CompressedTrajectory {
        match strategy {
            CompressionStrategy::None => CompressedTrajectory {
                states: trajectory.to_vec(),
                original_len: trajectory.len(),
                ratio: 1.0,
                oscillations_removed: 0,
                redundant_removed: 0,
                keep_indices: (0..trajectory.len()).collect(),
            },
            CompressionStrategy::BlockLevel => self.compress_blocks(trajectory),
            CompressionStrategy::Deoscillate => self.compress_deoscillate(trajectory),
            CompressionStrategy::DecisionPoints => self.compress_decision_points(trajectory),
            CompressionStrategy::SweTrace => self.compress_swe_trace(trajectory),
        }
    }

    /// Compress by keeping only block-level changes (top 3 bits).
    fn compress_blocks(&self, trajectory: &[u8]) -> CompressedTrajectory {
        if trajectory.is_empty() {
            return self.empty_result();
        }
        let mut states = Vec::with_capacity(trajectory.len());
        let mut keep_indices = Vec::with_capacity(trajectory.len());
        let mut redundant = 0;

        for (i, &s) in trajectory.iter().enumerate() {
            let block = s & 0xF8;
            if i == 0 {
                states.push(s);
                keep_indices.push(i);
            } else {
                let last_block = states.last().unwrap() & 0xF8;
                if block != last_block {
                    states.push(s);
                    keep_indices.push(i);
                } else {
                    redundant += 1;
                }
            }
        }

        self.result(states, trajectory.len(), keep_indices, 0, redundant)
    }

    /// Remove oscillations (ABAB patterns).
    /// Iterates until no ABAB pattern remains (handles 4+ repetitions).
    fn compress_deoscillate(&self, trajectory: &[u8]) -> CompressedTrajectory {
        // Phase 1: iterative ABAB removal producing (state, original_idx) pairs
        let mut pairs: Vec<(u8, usize)> = trajectory.iter().enumerate().map(|(i, &s)| (s, i)).collect();
        let mut osc_removed = 0;
        let mut changed = true;
        while changed {
            changed = false;
            let mut new_pairs = Vec::with_capacity(pairs.len());
            let mut i = 0;
            while i < pairs.len() {
                if i + 3 < pairs.len()
                    && pairs[i].0 == pairs[i + 2].0
                    && pairs[i + 1].0 == pairs[i + 3].0
                {
                    new_pairs.push(pairs[i]);
                    i += 4;
                    osc_removed += 2;
                    changed = true;
                } else {
                    new_pairs.push(pairs[i]);
                    i += 1;
                }
            }
            pairs = new_pairs;
        }
        let states: Vec<u8> = pairs.iter().map(|(s, _)| *s).collect();
        let keep_indices: Vec<usize> = pairs.iter().map(|(_, idx)| *idx).collect();
        self.result(states, trajectory.len(), keep_indices, osc_removed, 0)
    }

    /// Keep only "decision points" — states where ≥2 hexagram axes change.
    fn compress_decision_points(&self, trajectory: &[u8]) -> CompressedTrajectory {
        if trajectory.is_empty() {
            return self.empty_result();
        }

        let mut states = Vec::with_capacity(trajectory.len());
        let mut keep_indices = Vec::with_capacity(trajectory.len());
        let mut redundant = 0;

        // Always keep first and last
        states.push(trajectory[0]);
        keep_indices.push(0);

        for i in 1..trajectory.len() - 1 {
            let prev = trajectory[i - 1];
            let curr = trajectory[i];
            let next = trajectory[i + 1];

            // Hamming distance between prev and curr
            let dist_prev = (prev ^ curr).count_ones() as usize;
            // Count unique states in local window
            let window = if i >= 2 { &trajectory[i - 2..=i] } else { &trajectory[0..=i] };
            let unique: std::collections::HashSet<&u8> = window.iter().collect();

            let is_decision = dist_prev >= self.axis_threshold
                || prev & 0xF8 != curr & 0xF8  // block change
                || (curr != next && dist_prev >= 1 && unique.len() >= window.len() - 1);

            if is_decision {
                states.push(curr);
                keep_indices.push(i);
            } else {
                redundant += 1;
            }
        }

        // Ensure last state is included
        let last = trajectory.len() - 1;
        if *states.last().unwrap_or(&0) != trajectory[last] {
            states.push(trajectory[last]);
            keep_indices.push(last);
        }

        self.result(states, trajectory.len(), keep_indices, 0, redundant)
    }

    /// Full SWE-TRACE-style compression: blocks + deoscillate + decision points.
    fn compress_swe_trace(&self, trajectory: &[u8]) -> CompressedTrajectory {
        if trajectory.is_empty() {
            return self.empty_result();
        }

        // Phase 1: Block-level compression
        let blocks = self.compress_blocks(trajectory);

        // Phase 2: Deoscillate the block-compressed result
        let deosc = self.compress_deoscillate(&blocks.states);

        // Phase 3: Decision points from deoscillated
        let dp = self.compress_decision_points(&deosc.states);

        // Map keep_indices back to original positions
        let keep_indices: Vec<usize> = dp.keep_indices.iter()
            .map(|&idx| {
                if idx < deosc.keep_indices.len() {
                    let mid = deosc.keep_indices[idx];
                    if mid < blocks.keep_indices.len() {
                        blocks.keep_indices[mid]
                    } else {
                        trajectory.len() - 1
                    }
                } else {
                    trajectory.len() - 1
                }
            })
            .collect();

        let states = dp.states;
        let redundant_removed = dp.redundant_removed + blocks.redundant_removed;
        let ratio = if states.is_empty() { 1.0 } else { trajectory.len() as f64 / states.len() as f64 };
        CompressedTrajectory {
            states,
            original_len: trajectory.len(),
            ratio,
            oscillations_removed: deosc.oscillations_removed,
            redundant_removed,
            keep_indices,
        }
    }

    /// Detect if a trajectory has meaningful reasoning progress.
    /// Returns false for oscillating or stuck trajectories.
    pub fn has_reasoning_progress(&self, trajectory: &[u8], matrix: Option<&E8TransitionMatrix>) -> bool {
        if trajectory.len() < 3 {
            return true;
        }

        // Check for pure oscillation (no new states in last 6)
        let recent: std::collections::HashSet<&u8> = trajectory.iter().rev().take(6).collect();
        if recent.len() <= 2 && trajectory.len() >= 6 {
            return false;
        }

        // Check for stuck: same state for 4+ consecutive steps
        let mut streak = 1;
        for i in 1..trajectory.len() {
            if trajectory[i] == trajectory[i - 1] {
                streak += 1;
                if streak >= 4 {
                    return false;
                }
            } else {
                streak = 1;
            }
        }

        // Check transition matrix likelihood
        if let Some(tm) = matrix {
            let mut avg_prob = 0.0;
            let mut count = 0;
            for i in 1..trajectory.len() {
                let p = tm.transition_prob(trajectory[i - 1], trajectory[i]);
                avg_prob += p;
                count += 1;
            }
            if count > 0 {
                avg_prob /= count as f64;
                // Very low average transition probability suggests random walk
                if avg_prob < 0.01 && trajectory.len() > 10 {
                    return false;
                }
            }
        }

        true
    }

    /// Suggest how many more E8 states this trajectory needs for convergence.
    /// Based on the trajectory compression ratio and remaining diversity.
    pub fn estimated_remaining_steps(&self, trajectory: &[u8]) -> usize {
        if trajectory.len() < 3 {
            return 5; // default minimum
        }

        // More compression → more was redundant → closer to convergence
        let compressed = self.compress(trajectory, CompressionStrategy::SweTrace);
        let ratio = compressed.ratio;

        if ratio > 3.0 {
            1 // highly redundant → almost done
        } else if ratio > 2.0 {
            3
        } else if ratio > 1.5 {
            5
        } else {
            8 // still exploring
        }
    }

    // ─── Private helpers ──────────────────────────────────────────

    fn empty_result(&self) -> CompressedTrajectory {
        CompressedTrajectory {
            states: Vec::new(),
            original_len: 0,
            ratio: 1.0,
            oscillations_removed: 0,
            redundant_removed: 0,
            keep_indices: Vec::new(),
        }
    }

    fn result(
        &self,
        states: Vec<u8>,
        original_len: usize,
        keep_indices: Vec<usize>,
        oscillations_removed: usize,
        redundant_removed: usize,
    ) -> CompressedTrajectory {
        let ratio = if states.is_empty() || original_len == 0 {
            1.0
        } else {
            original_len as f64 / states.len() as f64
        };
        CompressedTrajectory {
            states,
            original_len,
            ratio,
            oscillations_removed,
            redundant_removed,
            keep_indices,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trajectory() -> Vec<u8> {
        // A reasoning trajectory: Acknowledge → Restate → Decompose → FP → SV → DD → Synthesize
        vec![58, 58, 50, 48, 42, 40, 34, 32, 26, 26, 24, 18, 10, 10, 16, 8, 0, 4]
    }

    fn make_oscillating() -> Vec<u8> {
        // ABAB oscillation
        vec![42, 26, 42, 26, 42, 26, 50, 40, 40, 32, 24]
    }

    #[test]
    fn test_no_compression() {
        let compressor = E8TraceCompressor::new();
        let traj = make_trajectory();
        let result = compressor.compress(&traj, CompressionStrategy::None);
        assert_eq!(result.states.len(), traj.len());
        assert!((result.ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_block_level_compression() {
        let compressor = E8TraceCompressor::new();
        let traj = make_trajectory();
        let result = compressor.compress(&traj, CompressionStrategy::BlockLevel);
        assert!(result.states.len() < traj.len(), "block compression should reduce size");
        assert!(result.redundant_removed > 0);
    }

    #[test]
    fn test_deoscillate() {
        let compressor = E8TraceCompressor::new();
        let traj = make_oscillating();
        let result = compressor.compress(&traj, CompressionStrategy::Deoscillate);
        assert!(result.oscillations_removed > 0, "should detect ABAB pattern");
        // After deoscillation, no ABAB should remain
        for w in result.states.windows(4) {
            assert!(!(w[0] == w[2] && w[1] == w[3]),
                    "ABAB pattern still present after deoscillation");
        }
    }

    #[test]
    fn test_decision_points() {
        let compressor = E8TraceCompressor::new();
        let traj = make_trajectory();
        let result = compressor.compress(&traj, CompressionStrategy::DecisionPoints);
        assert!(result.states.len() <= traj.len());
        assert!(!result.states.is_empty());
        // First and last should be preserved
        assert_eq!(result.states[0], traj[0]);
        assert_eq!(*result.states.last().unwrap(), traj[traj.len() - 1]);
    }

    #[test]
    fn test_swe_trace_compression() {
        let compressor = E8TraceCompressor::new();
        let traj = make_trajectory();
        let result = compressor.compress(&traj, CompressionStrategy::SweTrace);
        assert!(result.states.len() <= traj.len(), "SWE-TRACE should reduce size");
        assert!(result.ratio >= 1.0);
        assert_eq!(result.original_len, traj.len());
        // Keep indices should map to original positions
        for &idx in &result.keep_indices {
            assert!(idx < traj.len(), "keep index {} out of bounds for len {}", idx, traj.len());
        }
    }

    #[test]
    fn test_empty_trajectory() {
        let compressor = E8TraceCompressor::new();
        let result = compressor.compress(&[], CompressionStrategy::SweTrace);
        assert!(result.states.is_empty());
        assert_eq!(result.original_len, 0);
    }

    #[test]
    fn test_single_state() {
        let compressor = E8TraceCompressor::new();
        let result = compressor.compress(&[42], CompressionStrategy::SweTrace);
        assert_eq!(result.states, vec![42]);
        assert_eq!(result.original_len, 1);
    }

    #[test]
    fn test_has_reasoning_progress() {
        let compressor = E8TraceCompressor::new();
        assert!(compressor.has_reasoning_progress(&[58, 50, 42, 34, 26, 18], None));
        assert!(!compressor.has_reasoning_progress(&[42, 42, 42, 42, 42], None));
        assert!(!compressor.has_reasoning_progress(&[40, 24, 40, 24, 40, 24], None));
    }

    #[test]
    fn test_estimated_remaining_steps() {
        let compressor = E8TraceCompressor::new();
        let short = compressor.estimated_remaining_steps(&[58, 50]);
        assert_eq!(short, 5);
        let compressed_traj = compressor.compress(&make_trajectory(), CompressionStrategy::SweTrace);
        let remaining = compressor.estimated_remaining_steps(&compressed_traj.states);
        assert!(remaining >= 1);
    }

    #[test]
    fn test_expand_roundtrip() {
        let compressor = E8TraceCompressor::new();
        let traj: Vec<u8> = (0..16).step_by(2).collect(); // [0, 2, 4, ..., 14]
        let compressed = compressor.compress(&traj, CompressionStrategy::DecisionPoints);
        let expanded = compressed.expand();
        assert_eq!(expanded.len(), traj.len(), "expand should return to original length");
    }

    #[test]
    fn test_is_significant() {
        let compressor = E8TraceCompressor::new();
        let traj = make_trajectory();
        let result = compressor.compress(&traj, CompressionStrategy::SweTrace);
        // Trajectory has 18 elements, compression should be meaningful
        assert!(result.is_significant() || result.states.len() == traj.len());
    }

    #[test]
    fn test_decision_points_mapping() {
        let compressor = E8TraceCompressor::new();
        let traj = vec![58, 58, 50, 42, 34, 34, 26, 26, 18, 10];
        let result = compressor.compress(&traj, CompressionStrategy::DecisionPoints);
        // Each keep index should reference a valid original position
        for &idx in result.decision_points() {
            assert!(idx < traj.len(), "decision point {} out of bounds", idx);
        }
    }

    #[test]
    fn test_stuck_trajectory_detection() {
        let compressor = E8TraceCompressor::new();
        // Stuck on same state for 4+ steps
        let stuck = vec![42, 42, 42, 42, 42, 50];
        assert!(!compressor.has_reasoning_progress(&stuck, None));

        // Not stuck with variety
        let varied = vec![58, 50, 42, 42, 34, 26];
        assert!(compressor.has_reasoning_progress(&varied, None));
    }

    #[test]
    fn test_swe_trace_three_phase() {
        // Ensure all 3 phases compose correctly
        let compressor = E8TraceCompressor::new();
        let mut traj = Vec::with_capacity(30);
        // Start with blocks
        for b in [56, 48, 40, 32, 24, 16, 8, 0] {
            traj.push(b);
            traj.push(b + 2); // within-block variation
            traj.push(b + 4);
        }
        // Add end
        traj.push(4);
        traj.push(4);

        let result = compressor.compress(&traj, CompressionStrategy::SweTrace);
        assert!(result.states.len() < traj.len());
        assert_eq!(result.original_len, traj.len());
    }
}
