#![allow(clippy::approx_constant)]

//! # Quantized FHRR (qFHRR) VSA — 3-Bit Integer Phase Indices
//!
//! Implements qFHRR (arXiv 2026-04): 3-bit quantization of FHRR phase angles
//! to discrete indices, reducing memory 8x vs 64-bit f64 FHRR.
//!
//! ## Theory
//!
//! Each element is a 3-bit quantized phase index `idx ∈ {0, 1, ..., 7}`
//! representing a phase angle `θ = (idx / 8) · 2π`. The complex value
//! is `e^{iθ} = cos θ + i·sin θ`.
//!
//! | Operation | Discrete-qFHRR | Continuous-FHRR |
//! |-----------|---------------|-----------------|
//! | Bind      | `(a + b) mod 8` | `(θ_a + θ_b) mod 2π` |
//! | Unbind    | `(c - a) rem_euclid 8` | `(θ_c - θ_a) rem_euclid 2π` |
//! | Bundle    | Complex sum → arg → nearest index | Complex sum → arg |
//! | Similarity | Circular distance average | Cosine of phase difference |
//!
//! ## Memory Reduction
//!
//! | Representation | Per-element | Dim=2048 | Ratio |
//! |---------------|-------------|----------|-------|
//! | f64 FHRR      | 8 bytes     | 16 KB    | 1x    |
//! | qFHRR (i8)    | 1 byte      | 2 KB     | 8x    |
//!
//! ## Layer
//!
//! ```text
//! L3: Memory Layer — VSA representation for HyperCube storage
//! ```
//!
//! # Safety
//!
//! `#![forbid(unsafe_code)]` — zero unsafe code in this module.

#![forbid(unsafe_code)]

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default dimension for qFHRR vectors (same as FHRR_DIM for compatibility).
pub const QFHRR_DIM: usize = 2048;

/// Number of quantization levels (3-bit = 8 levels).
pub const QFHRR_LEVELS: i8 = 8;

/// Half of QFHRR_LEVELS as f64, used in similarity computation.
const QFHRR_HALF: f64 = 4.0;

/// 2π constant (TAU).
const TWO_PI: f64 = std::f64::consts::TAU;

// ---------------------------------------------------------------------------
// Lookup tables — precomputed cos/sin for each of the 8 phase levels
// ---------------------------------------------------------------------------

/// Cosine of each quantized phase angle `cos(idx / 8 * 2π)`.
pub const COS_TABLE: [f64; QFHRR_LEVELS as usize] = [
    1.0,                 // idx=0:  cos(0)
    0.7071067811865476,  // idx=1:  cos(π/4)
    0.0,                 // idx=2:  cos(π/2)
    -0.7071067811865476, // idx=3:  cos(3π/4)
    -1.0,                // idx=4:  cos(π)
    -0.7071067811865476, // idx=5:  cos(5π/4)
    -0.0,                // idx=6:  cos(3π/2)
    0.7071067811865476,  // idx=7:  cos(7π/4)
];

/// Sine of each quantized phase angle `sin(idx / 8 * 2π)`.
pub const SIN_TABLE: [f64; QFHRR_LEVELS as usize] = [
    0.0,                 // idx=0:  sin(0)
    0.7071067811865476,  // idx=1:  sin(π/4)
    1.0,                 // idx=2:  sin(π/2)
    0.7071067811865476,  // idx=3:  sin(3π/4)
    0.0,                 // idx=4:  sin(π)
    -0.7071067811865476, // idx=5:  sin(5π/4)
    -1.0,                // idx=6:  sin(3π/2)
    -0.7071067811865476, // idx=7:  sin(7π/4)
];

/// 3-bit quantized phase index (0-7, stored in i8).
pub type QIndex = i8;

// ---------------------------------------------------------------------------
// Core operations on &[i8] slices
// ---------------------------------------------------------------------------

/// Bind two qFHRR vectors: element-wise phase index addition modulo QFHRR_LEVELS.
///
/// ```
/// let a = vec![3, 1, 5, 7];
/// let b = vec![2, 6, 3, 1];
/// let c = qbind(&a, &b);
/// assert_eq!(c, vec![5, 7, 0, 0]);
/// ```
pub fn qbind(a: &[i8], b: &[i8]) -> Vec<i8> {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x + y).rem_euclid(QFHRR_LEVELS))
        .collect()
}

/// Unbind: reverse of bind via modular subtraction.
///
/// `qunbind(qbind(a, b), a) == b` (exact for qFHRR modular arithmetic).
pub fn qunbind(c: &[i8], a: &[i8]) -> Vec<i8> {
    c.iter()
        .zip(a.iter())
        .map(|(x, y)| (x - y).rem_euclid(QFHRR_LEVELS))
        .collect()
}

/// Bundle multiple qFHRR vectors via complex sum and argmax quantization.
///
/// Converts each index to (cos, sin) via lookup table, sums across all
/// vectors, computes the argument angle, then quantizes back to the
/// nearest discrete index.
///
/// For a single vector, returns a copy.
/// For empty input, returns a zero vector of length `QFHRR_DIM`.
pub fn qbundle(vectors: &[&[i8]]) -> Vec<i8> {
    if vectors.is_empty() {
        return vec![0; QFHRR_DIM];
    }
    if vectors.len() == 1 {
        return vectors[0].to_vec();
    }

    let dim = vectors[0].len();
    let n = vectors.len() as f64;
    let mut result = Vec::with_capacity(dim);

    for i in 0..dim {
        let mut cos_sum = 0.0;
        let mut sin_sum = 0.0;
        for v in vectors {
            let idx = v[i].max(0).min(QFHRR_LEVELS - 1) as usize;
            cos_sum += COS_TABLE[idx];
            sin_sum += SIN_TABLE[idx];
        }
        // Normalize by count to avoid precision loss for large n
        cos_sum /= n;
        sin_sum /= n;

        let angle = sin_sum.atan2(cos_sum);
        // Map from [-π, π] to [0, 2π)
        let angle_pos = if angle < 0.0 { angle + TWO_PI } else { angle };
        // Quantize to nearest index
        let raw = (angle_pos / TWO_PI * QFHRR_LEVELS as f64).round() as i8;
        let idx = raw.rem_euclid(QFHRR_LEVELS);
        result.push(idx);
    }
    result
}

/// Circular similarity between two qFHRR vectors.
///
/// For each dimension: similarity = 1 - circular_distance / 4.0
/// where circular_distance = min(|a-b|, QFHRR_LEVELS - |a-b|).
///
/// Returns `[0, 1]` where 1.0 = identical, 0.0 = maximally different.
pub fn qsimilarity(a: &[i8], b: &[i8]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }

    let levels_f = QFHRR_LEVELS as f64;

    let sum: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff_u = (*x).abs_diff(*y);
            let diff_f = diff_u as f64;
            let circular = diff_f.min(levels_f - diff_f);
            1.0 - circular / QFHRR_HALF
        })
        .sum();

    sum / n as f64
}

// ---------------------------------------------------------------------------
// Random and encoding helpers
// ---------------------------------------------------------------------------

/// Generate a random qFHRR vector with a seeded RNG.
///
/// Each element is uniformly sampled from {0, 1, ..., QFHRR_LEVELS-1}.
pub fn random_qfhrr(seed: u64) -> Vec<i8> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..QFHRR_DIM)
        .map(|_| rng.gen_range(0..QFHRR_LEVELS))
        .collect()
}

/// Generate a random qFHRR vector with custom dimension.
pub fn random_qfhrr_dim(dim: usize, seed: u64) -> Vec<i8> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..dim).map(|_| rng.gen_range(0..QFHRR_LEVELS)).collect()
}

/// Deterministically encode a scalar f64 value into a qFHRR vector.
///
/// Uses a splitmix64-style hash seeded by the value, producing a
/// reproducible vector. Nearby scalar values produce decorrelated
/// vectors (standard VSA encoding property).
pub fn encode_scalar_qfhrr(value: f64) -> Vec<i8> {
    let seed = value.to_bits();
    let mut result = Vec::with_capacity(QFHRR_DIM);
    let mut state: u64 = seed ^ 0x9E37_79B9_7F4A_7C15;

    for _ in 0..QFHRR_DIM {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        let idx = (z as usize % QFHRR_LEVELS as usize) as i8;
        result.push(idx);
    }
    result
}

/// Convert an f64 FHRR phase vector to qFHRR indices.
///
/// Each phase angle `θ` is mapped to the nearest quantized index:
/// `idx = round(θ / (2π) * QFHRR_LEVELS) mod QFHRR_LEVELS`.
pub fn fhrr_to_qfhrr(phases: &[f64]) -> Vec<i8> {
    phases
        .iter()
        .map(|&phase| {
            let mut p = phase % TWO_PI;
            if p < 0.0 {
                p += TWO_PI;
            }
            let normalized = p / TWO_PI;
            let raw = (normalized * QFHRR_LEVELS as f64).round() as i8;
            raw.rem_euclid(QFHRR_LEVELS)
        })
        .collect()
}

/// Convert qFHRR indices back to f64 FHRR phase angles.
///
/// Each index `idx` maps to `θ = (idx / QFHRR_LEVELS) * 2π`.
pub fn qfhrr_to_fhrr(indices: &[i8]) -> Vec<f64> {
    indices
        .iter()
        .map(|&idx| {
            let clamped = idx.max(0).min(QFHRR_LEVELS - 1);
            (clamped as f64 / QFHRR_LEVELS as f64) * TWO_PI
        })
        .collect()
}

// ---------------------------------------------------------------------------
// QuantizedFhrrVector — convenience wrapper
// ---------------------------------------------------------------------------

/// A convenience wrapper around a qFHRR index vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizedFhrrVector {
    indices: Vec<i8>,
}

impl QuantizedFhrrVector {
    /// Create a new qFHRR vector from index data.
    pub fn new(indices: Vec<i8>) -> Self {
        Self { indices }
    }

    /// Create a random qFHRR vector with the default dimension.
    pub fn random(seed: u64) -> Self {
        Self::new(random_qfhrr(seed))
    }

    /// Create a random qFHRR vector with a custom dimension.
    pub fn random_dim(dim: usize, seed: u64) -> Self {
        Self::new(random_qfhrr_dim(dim, seed))
    }

    /// Encode a scalar value into a qFHRR vector.
    pub fn from_scalar(value: f64) -> Self {
        Self::new(encode_scalar_qfhrr(value))
    }

    /// Convert from an f64 FHRR phase vector.
    pub fn from_fhrr(phases: &[f64]) -> Self {
        Self::new(fhrr_to_qfhrr(phases))
    }

    /// Convert back to f64 FHRR phase vector (dequantize).
    pub fn to_fhrr(&self) -> Vec<f64> {
        qfhrr_to_fhrr(&self.indices)
    }

    /// Bind this vector with another.
    pub fn bind(&self, other: &Self) -> Self {
        Self::new(qbind(&self.indices, &other.indices))
    }

    /// Unbind: remove the effect of another vector.
    pub fn unbind(&self, other: &Self) -> Self {
        Self::new(qunbind(&self.indices, &other.indices))
    }

    /// Bundle (superpose) this vector with another.
    pub fn bundle(&self, other: &Self) -> Self {
        let v1: &[i8] = &self.indices;
        let v2: &[i8] = &other.indices;
        let vectors = [v1, v2];
        Self::new(qbundle(&vectors))
    }

    /// Circular similarity with another qFHRR vector.
    pub fn similarity(&self, other: &Self) -> f64 {
        qsimilarity(&self.indices, &other.indices)
    }

    /// Dimension of this vector.
    pub fn dim(&self) -> usize {
        self.indices.len()
    }

    /// Access the raw indices slice.
    pub fn indices(&self) -> &[i8] {
        &self.indices
    }

    /// Consume and return the underlying index vector.
    pub fn into_indices(self) -> Vec<i8> {
        self.indices
    }
}

impl PartialEq for QuantizedFhrrVector {
    fn eq(&self, other: &Self) -> bool {
        self.indices == other.indices
    }
}

// ---------------------------------------------------------------------------
// QuantizedFhrrHyperCube — codebook-backed qFHRR engine
// ---------------------------------------------------------------------------

/// A qFHRR-based hypercube that maintains a codebook of named vectors.
///
/// Provides named access, binding, bundling, similarity search, and
/// optional diffusion retrieval.
#[derive(Serialize, Deserialize)]
pub struct QuantizedFhrrHyperCube {
    /// Dimension of stored vectors.
    dim: usize,
    /// Symbol name -> qFHRR index vector.
    codebook: HashMap<String, Vec<i8>>,
    /// Next seed for deterministic symbol generation.
    next_seed: u64,
}

impl Default for QuantizedFhrrHyperCube {
    fn default() -> Self {
        Self::new(QFHRR_DIM)
    }
}

impl QuantizedFhrrHyperCube {
    /// Create an empty hypercube with the given dimension.
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            codebook: HashMap::new(),
            next_seed: 0xAB_BA_BE_EF,
        }
    }

    /// Current vector dimension.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Number of symbols in the codebook.
    pub fn symbol_count(&self) -> usize {
        self.codebook.len()
    }

    /// Check if a symbol exists.
    pub fn contains_symbol(&self, name: &str) -> bool {
        self.codebook.contains_key(name)
    }

    /// Get a symbol's qFHRR vector by name.
    pub fn get_symbol(&self, name: &str) -> Option<&[i8]> {
        self.codebook.get(name).map(|v| v.as_slice())
    }

    /// Add a symbol with a randomly generated qFHRR vector.
    /// If the symbol already exists, it is not overwritten.
    pub fn add_symbol(&mut self, name: &str) {
        if !self.codebook.contains_key(name) {
            let seed = self.next_seed;
            self.next_seed = self.next_seed.wrapping_mul(0x9E37_79B9);
            let vec = random_qfhrr_dim(self.dim, seed);
            self.codebook.insert(name.to_string(), vec);
        }
    }

    /// Add or overwrite a symbol with a specific vector.
    pub fn set_symbol(&mut self, name: &str, vec: Vec<i8>) {
        debug_assert_eq!(vec.len(), self.dim, "vector dimension mismatch");
        self.codebook.insert(name.to_string(), vec);
    }

    /// Remove a symbol from the codebook. Returns the vector if it existed.
    pub fn remove_symbol(&mut self, name: &str) -> Option<Vec<i8>> {
        self.codebook.remove(name)
    }

    /// Get all symbol names in the codebook.
    pub fn symbol_names(&self) -> Vec<String> {
        self.codebook.keys().cloned().collect()
    }

    /// Bind two symbols by name and return the resulting qFHRR vector.
    pub fn bind_symbols(&self, a: &str, b: &str) -> Result<Vec<i8>, String> {
        let va = self
            .get_symbol(a)
            .ok_or_else(|| format!("symbol not found: {a}"))?;
        let vb = self
            .get_symbol(b)
            .ok_or_else(|| format!("symbol not found: {b}"))?;
        Ok(qbind(va, vb))
    }

    /// Bundle two symbols by name and return the resulting qFHRR vector.
    pub fn bundle_symbols(&self, a: &str, b: &str) -> Result<Vec<i8>, String> {
        let va = self
            .get_symbol(a)
            .ok_or_else(|| format!("symbol not found: {a}"))?;
        let vb = self
            .get_symbol(b)
            .ok_or_else(|| format!("symbol not found: {b}"))?;
        let vectors = [va, vb];
        Ok(qbundle(&vectors))
    }

    /// Find the closest codebook symbol to a query vector.
    ///
    /// Returns `(name, similarity)` or `None` if the codebook is empty.
    pub fn nearest_symbol(&self, query: &[i8]) -> Option<(&str, f64)> {
        self.codebook
            .iter()
            .map(|(name, vec)| (name.as_str(), qsimilarity(query, vec)))
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
    }

    /// Find the closest codebook symbol above a threshold.
    pub fn nearest_symbol_threshold(&self, query: &[i8], threshold: f64) -> Option<(&str, f64)> {
        self.nearest_symbol(query)
            .filter(|(_, sim)| *sim >= threshold)
    }

    /// Compute the pairwise similarity matrix for a list of symbol names.
    ///
    /// Returns an NxN matrix where entry `[i][j] = qsimilarity(symbols[i], symbols[j])`.
    /// All symbols must exist in the codebook.
    pub fn compute_similarity_matrix(&self, symbols: &[&str]) -> Result<Vec<Vec<f64>>, String> {
        let n = symbols.len();
        let vecs: Vec<&[i8]> = symbols
            .iter()
            .map(|&name| {
                self.codebook
                    .get(name)
                    .ok_or_else(|| format!("symbol not found in codebook: {name}"))
                    .map(|v| v.as_slice())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut mat = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                mat[i][j] = qsimilarity(vecs[i], vecs[j]);
            }
        }
        Ok(mat)
    }

    // -- LSH-style bucket indexing for approximate nearest-neighbor --

    /// Build LSH bucket key from the first `key_len` indices.
    /// Vectors that share the same first `key_len` indices fall into
    /// the same bucket and are likely to be similar.
    fn bucket_key(vector: &[i8], key_len: usize) -> Vec<i8> {
        let kl = key_len.min(vector.len());
        vector[..kl].to_vec()
    }

    /// Search using LSH buckets for approximate nearest neighbors.
    ///
    /// Builds bucket keys from the first `key_len` indices of each
    /// stored vector, then only searches vectors in the matching
    /// bucket(s). Falls back to linear scan if the matching bucket
    /// is empty or smaller than `min_candidates`.
    pub fn search_lsh(
        &self,
        query: &[i8],
        top_k: usize,
        key_len: usize,
        min_candidates: usize,
    ) -> Vec<(String, f64)> {
        if self.codebook.is_empty() {
            return Vec::new();
        }

        let qkey = Self::bucket_key(query, key_len);

        // Collect candidates from the matching bucket
        let mut candidates: Vec<(&str, f64)> = Vec::new();

        for (name, vec) in &self.codebook {
            let key = Self::bucket_key(vec, key_len);
            if key == qkey {
                let sim = qsimilarity(query, vec);
                candidates.push((name.as_str(), sim));
            }
        }

        // Fall back to linear scan if not enough candidates
        if candidates.len() < min_candidates {
            candidates = self
                .codebook
                .iter()
                .map(|(name, vec)| (name.as_str(), qsimilarity(query, vec)))
                .collect();
        }

        candidates.sort_by(|(_, a), (_, b)| b.total_cmp(a));
        candidates.truncate(top_k);
        candidates
            .into_iter()
            .map(|(name, sim)| (name.to_string(), sim))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// QFhrrDiffusionConfig + diffusion retrieval for QuantizedFhrrHyperCube
// ---------------------------------------------------------------------------

/// Configuration for diffusion activation retrieval on qFHRR HyperCube.
#[derive(Serialize, Deserialize)]
pub struct QFhrrDiffusionConfig {
    /// Number of diffusion steps (default: 3).
    pub steps: usize,
    /// Activation decay per step (default: 0.7).
    pub decay: f64,
    /// Number of top results to return (default: 5).
    pub top_k: usize,
    /// Minimum circular similarity threshold for edge creation (default: 0.3).
    pub edge_threshold: f64,
    /// Minimum final activation to include (default: 0.01).
    pub activation_threshold: f64,
}

impl Default for QFhrrDiffusionConfig {
    fn default() -> Self {
        Self {
            steps: 3,
            decay: 0.7,
            top_k: 5,
            edge_threshold: 0.3,
            activation_threshold: 0.01,
        }
    }
}

/// A scored result from qFHRR diffusion retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QFhrrDiffusionResult {
    /// Symbol name.
    pub name: String,
    /// Accumulated activation after diffusion.
    pub activation: f64,
    /// Direct circular similarity to the query.
    pub seed_similarity: f64,
}

impl QuantizedFhrrHyperCube {
    /// Retrieve symbols via diffusion activation on the qFHRR codebook.
    ///
    /// Propagates activation through the codebook graph for multiple steps:
    ///
    /// ```text
    /// activation[0][i] = sim(query, symbol[i])
    /// activation[t][i] = decay * activation[t-1][i]
    ///     + (1-decay) * mean_j( sim(i,j) * activation[t-1][j] )
    /// ```
    ///
    /// Returns top-K symbols by final activation.
    pub fn diffusion_retrieve(
        &self,
        query: &[i8],
        config: &QFhrrDiffusionConfig,
    ) -> Vec<QFhrrDiffusionResult> {
        let n = self.codebook.len();
        if n == 0 {
            return Vec::new();
        }

        let symbols: Vec<&str> = self.codebook.keys().map(|s| s.as_str()).collect();
        let vecs: Vec<&[i8]> = symbols
            .iter()
            .map(|&name| &self.codebook[name][..])
            .collect();

        // Step 1: compute seed activations and similarity matrix
        let mut seed_act: Vec<f64> = Vec::with_capacity(n);
        let mut sim_matrix: Vec<Vec<f64>> = Vec::with_capacity(n);

        for i in 0..n {
            let s = qsimilarity(query, vecs[i]);
            seed_act.push(s);
        }

        for i in 0..n {
            let mut row = Vec::with_capacity(n);
            for j in 0..n {
                let sim = if i == j {
                    1.0
                } else {
                    let s = qsimilarity(vecs[i], vecs[j]);
                    if s >= config.edge_threshold {
                        s
                    } else {
                        0.0
                    }
                };
                row.push(sim);
            }
            sim_matrix.push(row);
        }

        // Step 2: diffuse activation
        let mut activation = seed_act.clone();
        for _step in 0..config.steps {
            let mut next = vec![0.0; n];
            for i in 0..n {
                let mut neighbor_sum = 0.0;
                let mut neighbor_weight = 0.0;
                for j in 0..n {
                    if i != j && sim_matrix[i][j] > 0.0 {
                        neighbor_sum += sim_matrix[i][j] * activation[j];
                        neighbor_weight += sim_matrix[i][j];
                    }
                }
                let spread = if neighbor_weight > 1e-12 {
                    neighbor_sum / neighbor_weight
                } else {
                    0.0
                };
                next[i] = config.decay * activation[i] + (1.0 - config.decay) * spread;
            }
            activation = next;
        }

        // Step 3: collect and rank results
        let mut results: Vec<QFhrrDiffusionResult> = symbols
            .iter()
            .enumerate()
            .filter(|&(i, _)| activation[i] >= config.activation_threshold)
            .map(|(i, &name)| QFhrrDiffusionResult {
                name: name.to_string(),
                activation: activation[i],
                seed_similarity: seed_act[i],
            })
            .collect();

        results.sort_by(|a, b| b.activation.total_cmp(&a.activation));
        results.truncate(config.top_k);
        results
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: approximate float equality.
    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    /// Helper: FHRR cosine similarity for f64 phase vectors.
    fn fhrr_similarity(a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        if n == 0 {
            return 0.0;
        }
        let sum: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x - *y).cos()).sum();
        sum / n as f64
    }

    // -- Core operation tests --

    #[test]
    fn test_qbind_is_reversible() {
        let a = random_qfhrr_dim(256, 42);
        let b = random_qfhrr_dim(256, 99);
        let bound = qbind(&a, &b);
        let rebound = qunbind(&bound, &b);
        let sim = qsimilarity(&a, &rebound);
        // qunbind(qbind(a,b), b) == a exactly (modular arithmetic)
        assert!(
            approx_eq(sim, 1.0, 1e-12),
            "qunbind(qbind(a,b), b) should recover a exactly, sim={}",
            sim
        );
    }

    #[test]
    fn test_qbind_is_commutative() {
        let a = random_qfhrr_dim(256, 1);
        let b = random_qfhrr_dim(256, 2);
        let ab = qbind(&a, &b);
        let ba = qbind(&b, &a);
        assert_eq!(ab, ba, "qbind should be commutative");
    }

    #[test]
    fn test_qself_similarity_is_one() {
        let a = random_qfhrr_dim(256, 7);
        let sim = qsimilarity(&a, &a);
        assert!(
            approx_eq(sim, 1.0, 1e-12),
            "self similarity should be 1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_qrandom_vectors_baseline_similarity() {
        // With 3-bit (8 levels), two random i.i.d. vectors have
        // expected circular similarity = 0.5 exactly (derived from
        // the discrete uniform distribution over 8 values).
        let a = random_qfhrr_dim(QFHRR_DIM, 10);
        let b = random_qfhrr_dim(QFHRR_DIM, 20);
        let sim = qsimilarity(&a, &b);
        // Should be close to theoretical baseline 0.5
        assert!(
            (sim - 0.5).abs() < 0.05,
            "random qFHRR vectors should have similarity ~0.5 (baseline), got {}",
            sim
        );
    }

    #[test]
    fn test_qbundle_similar_to_components() {
        let a = random_qfhrr_dim(512, 30);
        let b = random_qfhrr_dim(512, 40);
        let vectors = [a.as_slice(), b.as_slice()];
        let c = qbundle(&vectors);
        let sim_a = qsimilarity(&c, &a);
        let sim_b = qsimilarity(&c, &b);
        assert!(
            sim_a > 0.25,
            "bundled should be similar to component a, got {}",
            sim_a
        );
        assert!(
            sim_b > 0.25,
            "bundled should be similar to component b, got {}",
            sim_b
        );
    }

    #[test]
    fn test_qfhrr_quantize_dequantize_roundtrip() {
        let mut rng = StdRng::seed_from_u64(12345);
        let phases: Vec<f64> = (0..512).map(|_| rng.gen_range(0.0..TWO_PI)).collect();
        let original = phases.clone();

        let qvec = QuantizedFhrrVector::from_fhrr(&phases);
        let reconstructed = qvec.to_fhrr();

        let sim = fhrr_similarity(&original, &reconstructed);
        assert!(
            sim > 0.90,
            "quantize-dequantize should preserve FHRR similarity > 0.90, got {}",
            sim
        );
    }

    #[test]
    fn test_qfhrr_hypercube_add_and_get() {
        let mut hc = QuantizedFhrrHyperCube::new(256);
        hc.add_symbol("circle");
        hc.add_symbol("square");
        assert_eq!(hc.symbol_count(), 2);
        assert!(hc.get_symbol("circle").is_some());
        assert!(hc.get_symbol("triangle").is_none());
    }

    #[test]
    fn test_qencoding_binds_properly() {
        let v1 = encode_scalar_qfhrr(3.14159);
        let v2 = encode_scalar_qfhrr(2.71828);
        assert_eq!(v1.len(), QFHRR_DIM);
        assert_eq!(v2.len(), QFHRR_DIM);

        // Encoding is deterministic
        let v1b = encode_scalar_qfhrr(3.14159);
        assert_eq!(v1, v1b, "encode_scalar_qfhrr should be deterministic");

        // Different values produce different vectors.
        // With 3-bit qFHRR, decorrelated vectors have baseline ~0.5
        let sim = qsimilarity(&v1, &v2);
        assert!(
            sim < 0.55,
            "different scalars should have decorrelated (baseline ~0.5) similarity, got {}",
            sim
        );

        // Bind two encoded scalars and verify unbind recovers
        let bound = qbind(&v1, &v2);
        assert_eq!(bound.len(), QFHRR_DIM);
        let rebound = qunbind(&bound, &v2);
        let recover_sim = qsimilarity(&v1, &rebound);
        assert!(
            approx_eq(recover_sim, 1.0, 1e-12),
            "qFHRR bind-then-unbind should be exact, sim={}",
            recover_sim
        );
    }

    #[test]
    fn test_qmemory_reduction() {
        let dim = QFHRR_DIM;
        let qmem = dim * std::mem::size_of::<i8>();
        let fmem = dim * std::mem::size_of::<f64>();
        assert_eq!(std::mem::size_of::<i8>(), 1);
        assert_eq!(std::mem::size_of::<f64>(), 8);
        assert!(qmem < fmem / 4, "qFHRR should use < 1/4 of f64 FHRR memory");
        assert_eq!(qmem, 2048, "qFHRR Vec<i8> at dim=2048 should be 2048 bytes");
        assert_eq!(
            fmem, 16384,
            "f64 Vec<f64> at dim=2048 should be 16384 bytes"
        );
    }

    // -- Additional correctness tests --

    #[test]
    fn test_qbind_with_known_indices() {
        let a = vec![3i8, 1, 5, 7, 0, 2, 4, 6];
        let b = vec![2i8, 6, 3, 1, 7, 5, 4, 0];
        let c = qbind(&a, &b);
        let expected_bind = vec![5i8, 7, 0, 0, 7, 7, 0, 6];
        assert_eq!(c, expected_bind, "qbind with known indices");

        let rebound = qunbind(&c, &a);
        assert_eq!(rebound, b, "qunbind(qbind(a,b), a) should recover b");
    }

    #[test]
    fn test_qbundle_empty_returns_zero_vector() {
        let result = qbundle(&[]);
        assert_eq!(result.len(), QFHRR_DIM);
        assert!(result.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_qbundle_single_vector() {
        let a = random_qfhrr_dim(128, 55);
        let vectors = [a.as_slice()];
        let result = qbundle(&vectors);
        assert_eq!(result, a, "bundling a single vector should return a copy");
    }

    #[test]
    fn test_qbundle_three_vectors() {
        let a = random_qfhrr_dim(256, 10);
        let b = random_qfhrr_dim(256, 20);
        let c = random_qfhrr_dim(256, 30);
        let vectors = [a.as_slice(), b.as_slice(), c.as_slice()];
        let bundled = qbundle(&vectors);
        assert_eq!(bundled.len(), 256);

        let sim_a = qsimilarity(&bundled, &a);
        let sim_b = qsimilarity(&bundled, &b);
        let sim_c = qsimilarity(&bundled, &c);
        assert!(sim_a > 0.15, "bundled similar to a, got {}", sim_a);
        assert!(sim_b > 0.15, "bundled similar to b, got {}", sim_b);
        assert!(sim_c > 0.15, "bundled similar to c, got {}", sim_c);
    }

    #[test]
    fn test_qsimilarity_zero_length() {
        let sim = qsimilarity(&[], &[]);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_qsimilarity_identity() {
        let a = random_qfhrr_dim(128, 0);
        let sim = qsimilarity(&a, &a);
        assert!(approx_eq(sim, 1.0, 1e-12));
    }

    #[test]
    fn test_qsimilarity_opposite_max_distance() {
        let a = vec![0i8; 100];
        let b = vec![4i8; 100];
        let sim = qsimilarity(&a, &b);
        assert!(approx_eq(sim, 0.0, 1e-12));
    }

    #[test]
    fn test_qsimilarity_half_distance() {
        let a = vec![0i8; 100];
        let b = vec![2i8; 100];
        let sim = qsimilarity(&a, &b);
        assert!(approx_eq(sim, 0.5, 1e-12), "expected 0.5, got {}", sim);
    }

    #[test]
    fn test_quantized_fhrr_vector_wrapper() {
        let v1 = QuantizedFhrrVector::random_dim(256, 1);
        let v2 = QuantizedFhrrVector::random_dim(256, 2);
        assert_eq!(v1.dim(), 256);
        assert_eq!(v2.dim(), 256);

        let bound = v1.bind(&v2);
        assert_eq!(bound.dim(), 256);

        let rebound = bound.unbind(&v2);
        let sim = v1.similarity(&rebound);
        assert!(approx_eq(sim, 1.0, 1e-12));

        let bundled = v1.bundle(&v2);
        let sim_to_v1 = bundled.similarity(&v1);
        let sim_to_v2 = bundled.similarity(&v2);
        assert!(sim_to_v1 > 0.25, "bundled similar to v1, got {}", sim_to_v1);
        assert!(sim_to_v2 > 0.25, "bundled similar to v2, got {}", sim_to_v2);
    }

    #[test]
    fn test_from_scalar_different_values() {
        let v1 = QuantizedFhrrVector::from_scalar(0.0);
        let v2 = QuantizedFhrrVector::from_scalar(1.0);
        let v3 = QuantizedFhrrVector::from_scalar(100.0);
        assert_eq!(v1.dim(), QFHRR_DIM);
        assert_eq!(v2.dim(), QFHRR_DIM);
        assert_eq!(v3.dim(), QFHRR_DIM);

        // Same scalar -> same vector
        let v1b = QuantizedFhrrVector::from_scalar(0.0);
        assert_eq!(v1.indices(), v1b.indices());

        // Different scalars -> decorrelated (baseline ~0.5 for 3-bit)
        let sim_01 = v1.similarity(&v2);
        let sim_02 = v1.similarity(&v3);
        assert!(sim_01 < 0.55, "different scalars 0 vs 1: sim={}", sim_01);
        assert!(sim_02 < 0.55, "different scalars 0 vs 100: sim={}", sim_02);
    }

    // -- HyperCube tests --

    #[test]
    fn test_qhypercube_default_dim() {
        let hc = QuantizedFhrrHyperCube::default();
        assert_eq!(hc.dim(), QFHRR_DIM);
    }

    #[test]
    fn test_qhypercube_add_no_duplicate() {
        let mut hc = QuantizedFhrrHyperCube::new(128);
        hc.add_symbol("alpha");
        hc.add_symbol("alpha");
        assert_eq!(hc.symbol_count(), 1);
    }

    #[test]
    fn test_qhypercube_remove_symbol() {
        let mut hc = QuantizedFhrrHyperCube::new(64);
        hc.add_symbol("temp");
        assert!(hc.contains_symbol("temp"));
        let removed = hc.remove_symbol("temp");
        assert!(removed.is_some());
        assert!(!hc.contains_symbol("temp"));
    }

    #[test]
    fn test_qhypercube_set_symbol_overwrites() {
        let mut hc = QuantizedFhrrHyperCube::new(64);
        hc.add_symbol("test");
        let new_vec = random_qfhrr_dim(64, 9999);
        hc.set_symbol("test", new_vec.clone());
        let retrieved = hc.get_symbol("test").unwrap();
        assert_eq!(retrieved, &new_vec);
    }

    #[test]
    fn test_qhypercube_nearest_symbol() {
        let mut hc = QuantizedFhrrHyperCube::new(256);
        hc.add_symbol("dog");
        hc.add_symbol("cat");
        hc.add_symbol("bird");
        let cat_vec = hc.get_symbol("cat").unwrap().to_vec();
        let (name, sim) = hc.nearest_symbol(&cat_vec).unwrap();
        assert_eq!(name, "cat");
        assert!(approx_eq(sim, 1.0, 1e-6));
    }

    #[test]
    fn test_qhypercube_nearest_symbol_empty() {
        let hc = QuantizedFhrrHyperCube::new(64);
        let query = random_qfhrr_dim(64, 0);
        assert!(hc.nearest_symbol(&query).is_none());
    }

    #[test]
    fn test_qhypercube_nearest_symbol_threshold() {
        let mut hc = QuantizedFhrrHyperCube::new(128);
        hc.add_symbol("known");
        let query = random_qfhrr_dim(128, 42);
        assert!(hc.nearest_symbol_threshold(&query, 0.9).is_none());
    }

    #[test]
    fn test_qhypercube_bind_symbols() {
        let mut hc = QuantizedFhrrHyperCube::new(128);
        hc.add_symbol("role");
        hc.add_symbol("filler");
        let bound = hc.bind_symbols("role", "filler").unwrap();
        assert_eq!(bound.len(), 128);
        let role_v = hc.get_symbol("role").unwrap();
        let filler_v = hc.get_symbol("filler").unwrap();
        let sim_to_role = qsimilarity(&bound, role_v);
        let sim_to_filler = qsimilarity(&bound, filler_v);
        // With 3-bit qFHRR, bound vectors have baseline ~0.5 similarity to
        // any random vector (including components). Verify not identical.
        assert!(
            sim_to_role < 0.65,
            "bound should not be highly similar to role, sim={}",
            sim_to_role
        );
        assert!(
            sim_to_filler < 0.65,
            "bound should not be highly similar to filler, sim={}",
            sim_to_filler
        );
    }

    #[test]
    fn test_qhypercube_bundle_symbols() {
        let mut hc = QuantizedFhrrHyperCube::new(128);
        hc.add_symbol("red");
        hc.add_symbol("blue");
        let bundled = hc.bundle_symbols("red", "blue").unwrap();
        assert_eq!(bundled.len(), 128);
        let red_v = hc.get_symbol("red").unwrap();
        let blue_v = hc.get_symbol("blue").unwrap();
        let sim_red = qsimilarity(&bundled, red_v);
        let sim_blue = qsimilarity(&bundled, blue_v);
        assert!(sim_red > 0.25, "bundled similar to red, sim={}", sim_red);
        assert!(sim_blue > 0.25, "bundled similar to blue, sim={}", sim_blue);
    }

    #[test]
    fn test_qhypercube_similarity_matrix() {
        let mut hc = QuantizedFhrrHyperCube::new(256);
        hc.add_symbol("x");
        hc.add_symbol("y");
        hc.add_symbol("z");
        let mat = hc.compute_similarity_matrix(&["x", "y", "z"]).unwrap();
        assert_eq!(mat.len(), 3);
        assert_eq!(mat[0].len(), 3);
        assert!(approx_eq(mat[0][0], 1.0, 1e-12));
        assert!(approx_eq(mat[1][1], 1.0, 1e-12));
        assert!(approx_eq(mat[2][2], 1.0, 1e-12));
        assert!(approx_eq(mat[0][1], mat[1][0], 1e-12));
        assert!(approx_eq(mat[0][2], mat[2][0], 1e-12));
    }

    // -- LSH search tests --

    #[test]
    fn test_search_lsh_empty_codebook() {
        let hc = QuantizedFhrrHyperCube::new(64);
        let query = random_qfhrr_dim(64, 0);
        let results = hc.search_lsh(&query, 5, 3, 2);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_lsh_finds_exact_match() {
        let mut hc = QuantizedFhrrHyperCube::new(64);
        hc.add_symbol("target");
        let target_vec = hc.get_symbol("target").unwrap().to_vec();
        hc.add_symbol("other");
        let results = hc.search_lsh(&target_vec, 5, 3, 2);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "target");
        assert!(approx_eq(results[0].1, 1.0, 1e-6));
    }

    // -- FHRR interop tests --

    #[test]
    fn test_fhrr_conversion_roundtrip_preserves_dim() {
        let phases: Vec<f64> = (0..128).map(|i| (i as f64) * 0.1 % TWO_PI).collect();
        let indices = fhrr_to_qfhrr(&phases);
        assert_eq!(indices.len(), 128);
        let phases_back = qfhrr_to_fhrr(&indices);
        assert_eq!(phases_back.len(), 128);
    }

    #[test]
    fn test_fhrr_conversion_all_indices_covered() {
        // Use points slightly past the start of each bin (avoid exact rounding boundaries)
        let test_phases: Vec<f64> = (0..QFHRR_LEVELS)
            .map(|i| {
                // Phase just past the lower boundary of bin i
                let normalized = (i as f64 + 0.001) / QFHRR_LEVELS as f64;
                normalized * TWO_PI
            })
            .collect();
        let indices = fhrr_to_qfhrr(&test_phases);
        for (i, &idx) in indices.iter().enumerate() {
            assert_eq!(idx, i as i8, "bin {} start should map to index {}", i, i);
        }
    }

    #[test]
    fn test_qfhrr_dim_constant() {
        assert_eq!(QFHRR_DIM, 2048);
        assert_eq!(QFHRR_LEVELS, 8);
    }

    #[test]
    fn test_qfhrr_lookup_tables() {
        for i in 0..QFHRR_LEVELS as usize {
            let mag2 = COS_TABLE[i] * COS_TABLE[i] + SIN_TABLE[i] * SIN_TABLE[i];
            assert!(
                approx_eq(mag2, 1.0, 1e-14),
                "cos^2 + sin^2 should be 1 for index {}, got {}",
                i,
                mag2
            );
        }
    }

    // -- Diffusion tests --

    #[test]
    fn test_diffusion_empty_codebook() {
        let hc = QuantizedFhrrHyperCube::new(128);
        let query = random_qfhrr_dim(128, 0);
        let results = hc.diffusion_retrieve(&query, &QFhrrDiffusionConfig::default());
        assert!(results.is_empty());
    }

    #[test]
    fn test_diffusion_single_symbol() {
        let mut hc = QuantizedFhrrHyperCube::new(128);
        hc.add_symbol("only");
        let query = hc.get_symbol("only").unwrap().to_vec();
        let config = QFhrrDiffusionConfig {
            steps: 1,
            decay: 0.9,
            edge_threshold: -0.1,
            activation_threshold: 0.001,
            ..Default::default()
        };
        let results = hc.diffusion_retrieve(&query, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "only");
        assert!(results[0].activation > 0.8);
    }
}
