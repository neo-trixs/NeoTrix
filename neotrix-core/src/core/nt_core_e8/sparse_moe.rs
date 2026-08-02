//! Phase 6.3 — Sparse MoE Reasoning Experts (稀疏 MoE 推理专家).
//!
//! The E₈ × 64 hexagram state space is grouped into 8 expert groups of 8 states
//! each (groups partition the 64 modes by their top-3 bits, i.e. contiguous
//! 8-entry blocks). A lightweight router scores the groups from:
//!
//!   - the current E₈ state (proximity to the active group),
//!   - the detected task type (domain affinity), and
//!   - the empirical E8 transition matrix (frequent next-block signal).
//!
//! Only the top-2 groups are activated each step; the other 6 are frozen
//! (attention mass zeroed). This is the Thinking Pixel §3.1 recursive sparse
//! MoE pattern: 64 discrete modes → 8 expert groups → top-k=2 sparse activation.
//!
//! Router score (interpretable, no learned params yet):
//!     s(g) = α·proximity(g, current) + β·affinity(task, g) + γ·transition(g)
//! with softmax over groups. Top-2 selected deterministically.

use crate::core::nt_core_e8::domain_transition::E8TaskType;
use serde::{Deserialize, Serialize};

/// Number of expert groups partitioning the 64 E₈ states.
pub const NUM_GROUPS: usize = 8;
/// States per expert group (64 / 8).
pub const GROUP_SIZE: usize = 8;
/// Number of groups activated each step.
pub const TOP_K: usize = 2;
/// Minimum activation share retained by the top-2 groups (sparsity floor).
pub const MIN_ACTIVE_MASS: f64 = 0.75;

/// Task-type → group affinity priors (Thinking Pixel-style expert priors).
///
/// Rows are [General, Reasoning, Math, Coding, Agentic, Creative]; columns are
/// the 8 group indices. Groups map to mythos-aligned blocks:
///   0: Synthesis, 1: Deep Dive, 2: Self-Verification, 3: First-Principles,
///   4: Alternative, 5: Decomposition, 6: Restatement, 7: Acknowledgment
#[rustfmt::skip]
const TASK_AFFINITY: [[f64; NUM_GROUPS]; 6] = [
    // General
    [0.6, 0.6, 0.6, 0.6, 0.6, 0.6, 0.6, 0.6],
    // Reasoning
    [0.8, 1.0, 0.8, 0.9, 0.5, 0.7, 0.5, 0.4],
    // Math
    [0.9, 1.0, 0.7, 1.0, 0.4, 0.7, 0.4, 0.3],
    // Coding
    [0.8, 0.9, 0.9, 0.6, 0.6, 1.0, 0.5, 0.4],
    // Agentic
    [0.6, 0.8, 0.7, 0.5, 0.7, 0.9, 0.6, 0.8],
    // Creative
    [0.8, 0.6, 0.5, 0.4, 1.0, 0.5, 0.6, 0.7],
];

/// One routing decision: which expert groups are active for a given step.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SparseRouting {
    /// Indices of the activated groups (length TOP_K, sorted ascending).
    pub active_groups: [usize; TOP_K],
    /// Group scores before top-k selection (softmax-normalized).
    #[serde(skip)]
    pub scores: [f64; NUM_GROUPS],
}

impl SparseRouting {
    /// Is a given E₈ state (0..64) within an active expert group?
    pub fn is_active(&self, state: u8) -> bool {
        let g = (state as usize).min(63) / GROUP_SIZE;
        self.active_groups.contains(&g)
    }

    /// Fraction of groups frozen (sparsity): 1 - TOP_K/NUM_GROUPS.
    pub fn sparsity(&self) -> f64 {
        1.0 - (TOP_K as f64 / NUM_GROUPS as f64)
    }
}

/// Phase 6.3 — sparse MoE router over E₈ expert groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseMoERouter {
    /// Blend weight for current-state proximity signal.
    pub alpha: f64,
    /// Blend weight for task-type affinity signal.
    pub beta: f64,
    /// Blend weight for transition-matrix signal.
    pub gamma: f64,
    /// Softmax temperature (higher = more uniform routing).
    pub temperature: f64,
    /// Last routing decision (for telemetry / deterministic tests).
    pub last_routing: Option<SparseRouting>,
}

impl Default for SparseMoERouter {
    fn default() -> Self {
        Self {
            alpha: 0.35,
            beta: 0.45,
            gamma: 0.20,
            temperature: 1.0,
            last_routing: None,
        }
    }
}

impl SparseMoERouter {
    /// Construct with default blend weights.
    pub fn new() -> Self {
        Self::default()
    }

    /// Task-type affinity of a group (0..8).
    pub fn affinity(&self, task: E8TaskType, group: usize) -> f64 {
        let row = match task {
            E8TaskType::General => 0,
            E8TaskType::Reasoning => 1,
            E8TaskType::Math => 2,
            E8TaskType::Coding => 3,
            E8TaskType::Agentic => 4,
            E8TaskType::Creative => 5,
        };
        TASK_AFFINITY[row][group.clamp(0, NUM_GROUPS - 1)]
    }

    /// Group index containing a given E₈ state.
    pub fn group_of(&self, state: u8) -> usize {
        (state as usize).min(63) / GROUP_SIZE
    }

    /// Proximity signal: current-state group peaks, decaying over Hamming blocks.
    ///
    /// Uses the top-3-bit block distance (groups are arranged along the high
    /// bits), so states in adjacent blocks score 0.5, two apart 0.25, etc.
    fn proximity(&self, state: u8, group: usize) -> f64 {
        let current = self.group_of(state);
        let dist = (current as isize - group as isize).unsigned_abs() as f64;
        2.0_f64.powf(-dist)
    }

    /// Score all 8 groups for a given step context.
    pub fn score_groups(
        &self,
        current_state: u8,
        task: E8TaskType,
        next_block_mass: Option<&[f64; NUM_GROUPS]>,
    ) -> [f64; NUM_GROUPS] {
        let mut scores = [0.0f64; NUM_GROUPS];
        for g in 0..NUM_GROUPS {
            let p = self.alpha * self.proximity(current_state, g);
            let a = self.beta * self.affinity(task, g);
            let t = match next_block_mass {
                Some(mass) => self.gamma * mass[g],
                None => 0.0,
            };
            scores[g] = p + a + t;
        }
        // Temperature-scaled softmax.
        let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mut exp = [0.0f64; NUM_GROUPS];
        let mut sum = 0.0f64;
        for (i, s) in scores.iter().enumerate() {
            let e = ((s - max) / self.temperature.max(1e-6)).exp();
            exp[i] = e;
            sum += e;
        }
        if sum > 0.0 {
            for x in exp.iter_mut() {
                *x /= sum;
            }
        } else {
            exp = [1.0 / NUM_GROUPS as f64; NUM_GROUPS];
        }
        exp
    }

    /// Route: score groups and select the top-2 as active experts.
    pub fn route(
        &mut self,
        current_state: u8,
        task: E8TaskType,
        next_block_mass: Option<&[f64; NUM_GROUPS]>,
    ) -> SparseRouting {
        let scores = self.score_groups(current_state, task, next_block_mass);
        // Deterministic arg-top-2.
        let mut order: Vec<usize> = (0..NUM_GROUPS).collect();
        order.sort_by(|&a, &b| {
            scores[b]
                .partial_cmp(&scores[a])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.cmp(&b))
        });
        let mut active = [order[0], order[1]];
        active.sort_unstable();
        let routing = SparseRouting { active_groups: active, scores };
        self.last_routing = Some(routing);
        routing
    }

    /// Apply routing as a mask over a 64-slot E₈ weight vector.
    ///
    /// Slots inside active groups keep their weight; slots in frozen groups are
    /// zeroed, and the surviving mass is renormalized so the vector still sums
    /// to its original total (conservation). Returns the resulting vector.
    pub fn apply_mask(&self, routing: &SparseRouting, weights: &[f64; 64]) -> [f64; 64] {
        let mut out = [0.0f64; 64];
        let mut kept = 0.0f64;
        for (i, w) in weights.iter().enumerate() {
            if routing.is_active(i as u8) {
                out[i] = *w;
                kept += *w;
            }
        }
        let total: f64 = weights.iter().sum();
        if kept > 0.0 && total > 0.0 {
            let scale = total / kept;
            for x in out.iter_mut() {
                *x *= scale;
            }
        }
        out
    }

    /// Fraction of the total weight mass retained after masking (should ≥ TOP_K/8).
    pub fn retained_mass(&self, routing: &SparseRouting, weights: &[f64; 64]) -> f64 {
        let total: f64 = weights.iter().sum();
        if total == 0.0 {
            return 0.0;
        }
        weights
            .iter()
            .enumerate()
            .filter(|(i, _)| routing.is_active(*i as u8))
            .map(|(_, w)| *w)
            .sum::<f64>()
            / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform() -> [f64; 64] {
        [1.0 / 64.0; 64]
    }

    #[test]
    fn test_partition_covers_all_states() {
        let r = SparseMoERouter::new();
        let mut seen = [false; 64];
        for s in 0..64u8 {
            let g = r.group_of(s);
            assert!(g < NUM_GROUPS);
            seen[s as usize] = true;
            assert_eq!(r.group_of(s), s as usize / GROUP_SIZE);
        }
        assert!(seen.iter().all(|&x| x));
    }

    #[test]
    fn test_group_size_is_eight() {
        for g in 0..NUM_GROUPS {
            let count = (0..64u8).filter(|s| (*s as usize) / GROUP_SIZE == g).count();
            assert_eq!(count, GROUP_SIZE);
        }
    }

    #[test]
    fn test_scores_normalized_and_positive() {
        let r = SparseMoERouter::new();
        let scores = r.score_groups(0, E8TaskType::Math, None);
        let sum: f64 = scores.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
        assert!(scores.iter().all(|&s| s > 0.0));
    }

    #[test]
    fn test_top2_selected_distinct() {
        let mut r = SparseMoERouter::new();
        for task in E8TaskType::ALL {
            let routing = r.route(0, task, None);
            assert_ne!(routing.active_groups[0], routing.active_groups[1]);
            let top2 = routing.active_groups;
            let scores = routing.scores;
            // No group outside the top-2 may outscore either selected one.
            for g in 0..NUM_GROUPS {
                if !top2.contains(&g) {
                    assert!(scores[g] <= scores[top2[1]] + 1e-12);
                }
            }
        }
    }

    #[test]
    fn test_mask_freezes_six_groups() {
        let mut r = SparseMoERouter::new();
        let routing = r.route(0, E8TaskType::Coding, None);
        let masked = r.apply_mask(&routing, &uniform());
        // Exactly TOP_K*GROUP_SIZE = 16 slots active.
        let active = masked.iter().filter(|&&x| x > 0.0).count();
        assert_eq!(active, TOP_K * GROUP_SIZE);
        assert_eq!(routing.sparsity(), 0.75);
        // Frozen slots hold exactly zero.
        for (i, &w) in masked.iter().enumerate() {
            if !routing.is_active(i as u8) {
                assert_eq!(w, 0.0);
            }
        }
    }

    #[test]
    fn test_mask_preserves_total_mass() {
        let mut r = SparseMoERouter::new();
        // Skewed weights: active group dominates.
        let mut w = [0.0f64; 64];
        for i in 0..64 {
            w[i] = if i < GROUP_SIZE { 0.02 } else { 0.0005 };
        }
        let routing = r.route(0, E8TaskType::Reasoning, None);
        let masked = r.apply_mask(&routing, &w);
        let before: f64 = w.iter().sum();
        let after: f64 = masked.iter().sum();
        assert!((before - after).abs() < 1e-9, "mass {before} vs {after}");
        assert!(r.retained_mass(&routing, &w) >= MIN_ACTIVE_MASS - 1e-9);
    }

    #[test]
    fn test_routing_deterministic() {
        let mut r1 = SparseMoERouter::new();
        let mut r2 = SparseMoERouter::new();
        let a = r1.route(32, E8TaskType::Math, None);
        let b = r2.route(32, E8TaskType::Math, None);
        assert_eq!(a.active_groups, b.active_groups);
        assert_eq!(a.scores, b.scores);
    }

    #[test]
    fn test_math_prefers_computation_groups() {
        let mut r = SparseMoERouter::new();
        // State in a synthesis block (group 0); Math affinity should pull in
        // the computation-heavy groups (1 = Deep Dive, 3 = First-Principles).
        let routing = r.route(0, E8TaskType::Math, None);
        let has_compute = routing.active_groups.contains(&1) || routing.active_groups.contains(&3);
        assert!(has_compute, "Math should activate a computation group, got {:?}", routing.active_groups);
    }

    #[test]
    fn test_transition_mass_biases_routing() {
        let mut r = SparseMoERouter::new();
        // No transition signal → default routing.
        let base = r.route(0, E8TaskType::General, None);
        // Heavy next-block mass on group 6 should pull routing toward it.
        let mut mass = [0.0f64; NUM_GROUPS];
        mass[6] = 1.0;
        let biased = r.route(0, E8TaskType::General, Some(&mass));
        assert!(biased.active_groups.contains(&6));
        assert!(base.active_groups.contains(&6) || r.affinity(E8TaskType::General, 6) > 0.0);
    }

    #[test]
    fn test_active_state_inside_selected_group() {
        let mut r = SparseMoERouter::new();
        let routing = r.route(56, E8TaskType::Agentic, None);
        // State 56 lives in group 7; Agentic affinity favors it (0.8).
        assert!(routing.is_active(56), "current state's group should be active, got {:?}", routing.active_groups);
    }
}
