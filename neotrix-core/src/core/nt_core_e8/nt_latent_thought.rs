//! Phase 6.1 — Latent Thought Vectors (潜在思想向量).
//!
//! Maps the discrete E₈ × 64 hexagram states onto a differentiable continuous
//! latent space. Each discrete `ReasoningHexagram` becomes a smooth, L₁-normalized
//! Gaussian kernel over the Hamming topology of the 6-bit hexagram cube, so:
//!
//!   - neighbouring states (one line flip apart) yield highly similar vectors
//!   - distant states (opposite corners) are nearly orthogonal
//!   - the map is deterministic and differentiable (continuity check point)
//!
//! This is the substrate for recursive latent reasoning (Phase 6.2+): a trajectory
//! of discrete modes becomes an aggregated continuous "thought" that can be
//! compared, interpolated, and decoded back to the nearest mode.

use crate::core::nt_core_hex::ReasoningHexagram;
use serde::{Deserialize, Serialize};

/// Default latent dimension = number of E₈ hexagram states (64).
pub const DEFAULT_LATENT_DIM: usize = 64;

/// Default Gaussian kernel width (σ) in Hamming-distance units.
pub const DEFAULT_KERNEL_SIGMA: f64 = 1.5;

/// Phase 6.1 — differentiable latent embedding of E₈ hexagram states.
///
/// `dim` defaults to 64 (one latent slot per E₈ state). The `embed` kernel is
/// defined over the 64-state hexagram cube, so a non-64 dimension is only
/// meaningful as an upstream projection target, not as the embedding itself.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LatentThoughtVector {
    /// Latent dimension. Defaults to 64 (one slot per E₈ state).
    pub dim: usize,
}

impl Default for LatentThoughtVector {
    fn default() -> Self {
        Self { dim: DEFAULT_LATENT_DIM }
    }
}

impl LatentThoughtVector {
    /// Construct with an explicit latent dimension.
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    /// Embed a discrete `ReasoningHexagram` into a continuous vector.
    ///
    /// Uses a Gaussian kernel over Hamming distance: each target state `j`
    /// receives weight `exp(-d²/2σ²)` where `d = hamming(state, j)`, followed
    /// by L₁ normalization. The result is a smooth probability-like distribution
    /// over the E₈ state space — differentiable w.r.t. continuous perturbations
    /// and deterministic across calls.
    pub fn embed(&self, state: ReasoningHexagram) -> Vec<f64> {
        debug_assert_eq!(
            self.dim,
            DEFAULT_LATENT_DIM,
            "kernel embedding is defined over the 64 E8 states; dim must be 64"
        );
        let mut v = vec![0.0f64; self.dim];
        for j in 0..64 {
            let d = state.hamming_dist(&ReasoningHexagram(j as u8)) as f64;
            v[j] = (-(d * d) / (2.0 * DEFAULT_KERNEL_SIGMA * DEFAULT_KERNEL_SIGMA)).exp();
        }
        let sum: f64 = v.iter().sum();
        if sum > 0.0 {
            for x in v.iter_mut() {
                *x /= sum;
            }
        }
        v
    }

    /// Embed a whole trajectory of states (oldest → newest).
    pub fn embed_trajectory(&self, states: &[ReasoningHexagram]) -> Vec<Vec<f64>> {
        states.iter().map(|s| self.embed(*s)).collect()
    }

    /// Aggregate a trajectory into a single workspace representation.
    ///
    /// Element-wise mean of the per-state embeddings — a convex mixture of
    /// Gaussian kernels, so the result stays a valid (unnormalized) distribution
    /// over the E₈ state space and remains comparable via `cosine`.
    pub fn average_thought(&self, states: &[ReasoningHexagram]) -> Vec<f64> {
        if states.is_empty() {
            return vec![0.0; self.dim];
        }
        let mut acc = vec![0.0f64; self.dim];
        for s in states {
            let e = self.embed(*s);
            for (a, b) in acc.iter_mut().zip(e.iter()) {
                *a += b;
            }
        }
        let n = states.len() as f64;
        for a in acc.iter_mut() {
            *a /= n;
        }
        acc
    }

    /// Cosine similarity in latent space. Returns 0.0 for degenerate inputs.
    pub fn cosine(&self, a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let mut dot = 0.0f64;
        let mut na = 0.0f64;
        let mut nb = 0.0f64;
        for (x, y) in a.iter().zip(b.iter()) {
            dot += x * y;
            na += x * x;
            nb += y * y;
        }
        let denom = na.sqrt() * nb.sqrt();
        if denom == 0.0 {
            return 0.0;
        }
        (dot / denom).clamp(-1.0, 1.0)
    }

    /// Linear interpolation between two latent vectors (t ∈ [0,1]).
    ///
    /// The basis for thought transformation / evolution: a trajectory of
    /// discrete modes is turned into continuous movement in latent space.
    pub fn interpolate(&self, a: &[f64], b: &[f64], t: f64) -> Vec<f64> {
        let n = self.dim.max(a.len()).max(b.len());
        let mut out = vec![0.0f64; n];
        let t = t.clamp(0.0, 1.0);
        for i in 0..n {
            let ai = a.get(i).copied().unwrap_or(0.0);
            let bi = b.get(i).copied().unwrap_or(0.0);
            out[i] = (1.0 - t) * ai + t * bi;
        }
        out
    }

    /// Decode a continuous vector back to the nearest `ReasoningHexagram`.
    ///
    /// Argmax over the vector entries; index `i` maps to state `i % 64`.
    /// For a mixture of Gaussian kernels the argmax lands on the dominant mode.
    pub fn nearest_state(&self, v: &[f64]) -> ReasoningHexagram {
        let mut best = 0usize;
        let mut best_val = f64::NEG_INFINITY;
        for (i, &x) in v.iter().enumerate() {
            if x > best_val {
                best_val = x;
                best = i;
            }
        }
        ReasoningHexagram((best % 64) as u8)
    }

    /// Pairwise cosine similarity matrix over a trajectory.
    pub fn similarity_matrix(&self, states: &[ReasoningHexagram]) -> Vec<Vec<f64>> {
        let embeds = self.embed_trajectory(states);
        let n = embeds.len();
        let mut m = vec![vec![0.0f64; n]; n];
        for i in 0..n {
            for j in 0..n {
                m[i][j] = self.cosine(&embeds[i], &embeds[j]);
            }
        }
        m
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    #[test]
    fn test_default_dim_matches_e8_state_count() {
        let ltv = LatentThoughtVector::default();
        assert_eq!(ltv.dim, 64);
        assert_eq!(LatentThoughtVector::new(64).dim, 64);
    }

    #[test]
    fn test_embed_dimension() {
        let ltv = LatentThoughtVector::default();
        for s in 0..64u8 {
            let v = ltv.embed(ReasoningHexagram(s));
            assert_eq!(v.len(), ltv.dim);
            assert_eq!(v.len(), 64);
        }
    }

    #[test]
    fn test_embed_deterministic_and_non_degenerate() {
        let ltv = LatentThoughtVector::default();
        let a = ltv.embed(ReasoningHexagram(7));
        let b = ltv.embed(ReasoningHexagram(7));
        assert_eq!(a, b, "embedding must be deterministic");
        assert!(a.iter().any(|&x| x > 0.0), "embedding must be non-degenerate");
        assert!(a.iter().all(|&x| x >= 0.0), "Gaussian kernel is non-negative");
        let sum: f64 = a.iter().sum();
        assert!((sum - 1.0).abs() < EPS, "L1-normalized kernel sums to 1");
        // Peak on the source state itself.
        assert!(a[7] > a[8] && a[7] > a[6]);
    }

    #[test]
    fn test_adjacent_similarity_exceeds_distant() {
        let ltv = LatentThoughtVector::default();
        let base = ltv.embed(ReasoningHexagram(0));
        let near = ltv.embed(ReasoningHexagram(1));   // hamming distance 1
        let far = ltv.embed(ReasoningHexagram(63));   // hamming distance 6
        let c_near = ltv.cosine(&base, &near);
        let c_far = ltv.cosine(&base, &far);
        assert!(
            c_near > c_far,
            "adjacent state must be more similar than distant (near={c_near:.4}, far={c_far:.4})"
        );
        // Self-similarity is the maximum.
        assert_eq!(ltv.cosine(&base, &base), 1.0);
    }

    #[test]
    fn test_embed_to_nearest_roundtrip() {
        let ltv = LatentThoughtVector::default();
        for s in 0..64u8 {
            let v = ltv.embed(ReasoningHexagram(s));
            assert_eq!(
                ltv.nearest_state(&v),
                ReasoningHexagram(s),
                "embed→nearest_state must roundtrip for state {s}"
            );
        }
    }

    #[test]
    fn test_interpolate_endpoints() {
        let ltv = LatentThoughtVector::default();
        let a = ltv.embed(ReasoningHexagram(3));
        let b = ltv.embed(ReasoningHexagram(40));
        let at_zero = ltv.interpolate(&a, &b, 0.0);
        let at_one = ltv.interpolate(&a, &b, 1.0);
        let mid = ltv.interpolate(&a, &b, 0.5);
        assert_eq!(at_zero.len(), ltv.dim);
        assert_eq!(at_one.len(), ltv.dim);
        assert_eq!(mid.len(), ltv.dim);
        for (x, y) in at_zero.iter().zip(a.iter()) {
            assert!((x - y).abs() < EPS);
        }
        for (x, y) in at_one.iter().zip(b.iter()) {
            assert!((x - y).abs() < EPS);
        }
        for (i, m) in mid.iter().enumerate() {
            assert!((m - (a[i] + b[i]) / 2.0).abs() < EPS);
        }
        // Out-of-range t is clamped.
        let low = ltv.interpolate(&a, &b, -1.0);
        assert_eq!(low, at_zero);
    }

    #[test]
    fn test_cosine_normalized() {
        let ltv = LatentThoughtVector::default();
        let a = ltv.embed(ReasoningHexagram(10));
        let b = ltv.embed(ReasoningHexagram(20));
        assert_eq!(ltv.cosine(&a, &a), 1.0);
        assert_eq!(ltv.cosine(&b, &b), 1.0);
        let c = ltv.cosine(&a, &b);
        assert!(c >= -1.0 && c <= 1.0);
        // Symmetry.
        assert_eq!(c, ltv.cosine(&b, &a));
        // Degenerate inputs.
        assert_eq!(ltv.cosine(&[], &[]), 0.0);
        assert_eq!(ltv.cosine(&[1.0, 0.0], &[0.0, 0.0]), 0.0);
        // Length mismatch.
        assert_eq!(ltv.cosine(&[1.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn test_similarity_matrix_symmetric_diagonal_one() {
        let ltv = LatentThoughtVector::default();
        let states: Vec<ReasoningHexagram> = vec![
            ReasoningHexagram(0),
            ReasoningHexagram(1),
            ReasoningHexagram(8),
            ReasoningHexagram(42),
            ReasoningHexagram(63),
        ];
        let m = ltv.similarity_matrix(&states);
        assert_eq!(m.len(), states.len());
        for row in &m {
            assert_eq!(row.len(), states.len());
        }
        for i in 0..m.len() {
            assert_eq!(m[i][i], 1.0, "diagonal must be 1.0");
            for j in 0..m.len() {
                assert!((m[i][j] - m[j][i]).abs() < EPS, "matrix must be symmetric");
            }
        }
        // Closer states score higher than distant ones.
        assert!(m[0][1] > m[0][4]);
    }

    #[test]
    fn test_average_thought_stable() {
        let ltv = LatentThoughtVector::default();
        let s = ReasoningHexagram(5);
        // Averaging a repeated state reproduces that state's embedding.
        let avg_single = ltv.average_thought(&[s]);
        let embed = ltv.embed(s);
        assert_eq!(avg_single.len(), ltv.dim);
        for (x, y) in avg_single.iter().zip(embed.iter()) {
            assert!((x - y).abs() < EPS);
        }
        // A mixed trajectory is a valid distribution and stays comparable.
        let states: Vec<ReasoningHexagram> = vec![
            ReasoningHexagram(0),
            ReasoningHexagram(16),
            ReasoningHexagram(32),
            ReasoningHexagram(48),
        ];
        let avg = ltv.average_thought(&states);
        assert_eq!(avg.len(), ltv.dim);
        assert!(avg.iter().any(|&x| x > 0.0), "average must be non-degenerate");
        let re_embedded = ltv.average_thought(&states);
        assert_eq!(avg, re_embedded, "average must be stable / deterministic");
        // Empty trajectory degrades to the zero vector.
        let empty = ltv.average_thought(&[]);
        assert!(empty.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_embed_trajectory_shape() {
        let ltv = LatentThoughtVector::default();
        let states: Vec<ReasoningHexagram> = (0..64u8).step_by(7).map(ReasoningHexagram).collect();
        let embeds = ltv.embed_trajectory(&states);
        assert_eq!(embeds.len(), states.len());
        for e in &embeds {
            assert_eq!(e.len(), ltv.dim);
        }
    }
}
