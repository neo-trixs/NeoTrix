//! # GHRR (Gabor Holographic Reduced Representation) VSA
//!
//! A non-commutative extension of FHRR using a phase bias parameter η.
//! Each element is a phase angle θ ∈ [0, 2π), representing a complex number e^{iθ}.
//!
//! ## Non-Commutative Binding
//!
//! GHRR introduces directional binding through a phase bias η:
//!
//! ```text
//! bind_dir(a, b, η)[i] = (η · θ_a[i] + θ_b[i]) mod 2π
//! ```
//!
//! When η ≠ 1, the operation is non-commutative:
//! `bind_dir(a, b, η) ≠ bind_dir(b, a, η)`.
//!
//! ## Special Cases
//!
//! | η value | Behavior | Description |
//! |---------|----------|-------------|
//! | η = 0   | `θ_i = θ_b[i]` | Identity-like: second operand unchanged |
//! | η = 1   | `θ_i = θ_a[i] + θ_b[i]` | Pure FHRR (fully commutative) |
//! | η = φ   | `θ_i = φ·θ_a[i] + θ_b[i]` | Non-commutative (default) |
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

/// Default GHRR hypervector dimension (same as FHRR_DIM for compatibility).
pub const GHRR_DIM: usize = 2048;

/// Default phase bias — the golden ratio φ = 1.618...
///
/// Using the golden ratio maximises the asymmetry between
/// bind_dir(a,b,η) and bind_dir(b,a,η) since φ is the
/// "most irrational" number, producing minimal accidental
/// alignment between η·θ_a + θ_b and η·θ_b + θ_a.
pub const GHRR_ETA: f64 = 1.618033988749895;

/// Phase rotation step used in permute (shared with FHRR).
const PHASE_STEP: f64 = std::f64::consts::TAU / 137.0;

// ---------------------------------------------------------------------------
// Core GHRR vector operations
// ---------------------------------------------------------------------------

/// Directional binding of two GHRR vectors with phase bias η.
///
/// For each dimension i: θ_i = (η · θ_a[i] + θ_b[i]) mod 2π.
///
/// When η = 1, this is equivalent to FHRR `bind` (commutative).
/// When η = φ (golden ratio, default), binding is maximally non-commutative.
/// When η = 0, the first operand is suppressed and the result is just `b`.
pub fn ghrr_bind_dir(a: &[f64], b: &[f64], eta: f64) -> Vec<f64> {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let theta = eta * x + y;
            theta.rem_euclid(std::f64::consts::TAU)
        })
        .collect()
}

/// Inverse of directional binding.
///
/// For each dimension i: θ_i = (θ_c[i] - η · θ_a[i]) mod 2π.
///
/// This satisfies: `unbind_dir(bind_dir(a, b, η), a, η) ≈ b`.
pub fn ghrr_unbind_dir(c: &[f64], a: &[f64], eta: f64) -> Vec<f64> {
    c.iter()
        .zip(a.iter())
        .map(|(theta_c, theta_a)| {
            let diff = theta_c - eta * theta_a;
            diff.rem_euclid(std::f64::consts::TAU)
        })
        .collect()
}

/// Bundle multiple GHRR vectors via complex sum normalization.
///
/// Same as FHRR `bundle`: converts each phase angle to (cos θ, sin θ),
/// sums across all vectors, then converts back to phase via arctan2.
/// The result is the centroid direction in complex space.
pub fn ghrr_bundle(vectors: &[&[f64]]) -> Vec<f64> {
    if vectors.is_empty() {
        return Vec::new();
    }
    let dim = vectors[0].len();
    let mut result = Vec::with_capacity(dim);
    for i in 0..dim {
        let mut sum_cos = 0.0;
        let mut sum_sin = 0.0;
        for v in vectors {
            let (s, c) = v[i].sin_cos();
            sum_cos += c;
            sum_sin += s;
        }
        let theta = sum_sin.atan2(sum_cos);
        let theta = if theta < 0.0 {
            theta + std::f64::consts::TAU
        } else {
            theta
        };
        result.push(theta);
    }
    result
}

/// Bundle two GHRR vectors (convenience wrapper).
pub fn ghrr_bundle_two(a: &[f64], b: &[f64]) -> Vec<f64> {
    ghrr_bundle(&[a, b])
}

/// Cosine similarity in complex space: mean of cos(θ_a - θ_b).
///
/// Returns a value in [-1, 1] where 1.0 = identical phase vectors.
/// Same as FHRR `similarity`.
pub fn ghrr_similarity(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let sum: f64 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).cos()).sum();
    sum / (n as f64)
}

/// Permute a GHRR vector by applying a dimension-dependent phase rotation.
///
/// θ'_i = (θ_i + i · n · PHASE_STEP) mod 2π
/// Same as FHRR `permute` — reversible and distance-preserving.
pub fn ghrr_permute(a: &[f64], n: usize) -> Vec<f64> {
    let n_f64 = n as f64;
    a.iter()
        .enumerate()
        .map(|(i, theta)| {
            let shifted = theta + (i as f64) * n_f64 * PHASE_STEP;
            shifted.rem_euclid(std::f64::consts::TAU)
        })
        .collect()
}

/// Generate a random GHRR phase vector (uniform in [0, 2π)).
pub fn ghrr_random_vector(seed: u64) -> Vec<f64> {
    ghrr_random_vector_dim(GHRR_DIM, seed)
}

/// Generate a random phase vector with custom dimension.
pub fn ghrr_random_vector_dim(dim: usize, seed: u64) -> Vec<f64> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..dim)
        .map(|_| rng.gen_range(0.0..std::f64::consts::TAU))
        .collect()
}

/// Cleanup: find the nearest neighbour in a candidate list.
///
/// Returns the index of the candidate with highest similarity.
/// If the best similarity is below the threshold, returns `None`.
pub fn ghrr_cleanup(noisy: &[f64], candidates: &[&[f64]], threshold: f64) -> Option<usize> {
    let (best_idx, best_sim) = candidates
        .iter()
        .enumerate()
        .map(|(i, c)| (i, ghrr_similarity(noisy, c)))
        .max_by(|(_, a), (_, b)| a.total_cmp(b))?;
    if best_sim >= threshold {
        Some(best_idx)
    } else {
        None
    }
}

/// Cleanup (no threshold): returns the index of the nearest neighbour.
pub fn ghrr_cleanup_always(noisy: &[f64], candidates: &[&[f64]]) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .map(|(i, c)| (i, ghrr_similarity(noisy, c)))
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(idx, _)| idx)
}

// ---------------------------------------------------------------------------
// GhrrVector — a convenience wrapper around phase data
// ---------------------------------------------------------------------------

/// A convenience wrapper for a GHRR phase vector with directional binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhrrVector {
    phases: Vec<f64>,
}

impl GhrrVector {
    /// Create a new GhrrVector from an existing phase vector.
    pub fn new(phases: Vec<f64>) -> Self {
        Self { phases }
    }

    /// Create a random GHRR vector with the default dimension.
    pub fn random(seed: u64) -> Self {
        Self::new(ghrr_random_vector_dim(GHRR_DIM, seed))
    }

    /// Create a random GHRR vector with a specific dimension.
    pub fn random_dim(dim: usize, seed: u64) -> Self {
        Self::new(ghrr_random_vector_dim(dim, seed))
    }

    /// Access the underlying phase slice.
    pub fn phases(&self) -> &[f64] {
        &self.phases
    }

    /// Consume the vector and return the phase data.
    pub fn into_phases(self) -> Vec<f64> {
        self.phases
    }

    /// Dimension of this vector.
    pub fn dim(&self) -> usize {
        self.phases.len()
    }

    /// Directional bind with another vector using the given phase bias.
    ///
    /// When `eta = 1.0`, this is equivalent to FHRR bind (commutative).
    /// When `eta = GHRR_ETA` (default), binding is maximally non-commutative.
    pub fn bind_dir(&self, other: &Self, eta: f64) -> Self {
        Self::new(ghrr_bind_dir(&self.phases, &other.phases, eta))
    }

    /// Directional unbind using the given phase bias.
    pub fn unbind_dir(&self, other: &Self, eta: f64) -> Self {
        Self::new(ghrr_unbind_dir(&self.phases, &other.phases, eta))
    }

    /// Bundle with another vector (same as FHRR bundle).
    pub fn bundle(&self, other: &Self) -> Self {
        Self::new(ghrr_bundle_two(&self.phases, &other.phases))
    }

    /// Permute the vector.
    pub fn permute(&self, n: usize) -> Self {
        Self::new(ghrr_permute(&self.phases, n))
    }

    /// Cosine similarity with another vector.
    pub fn similarity(&self, other: &Self) -> f64 {
        ghrr_similarity(&self.phases, &other.phases)
    }
}

// ---------------------------------------------------------------------------
// GhrrHyperCube — codebook-backed GHRR engine
// ---------------------------------------------------------------------------

/// A GHRR-based hypercube that maintains a codebook of atomic symbol vectors
/// and exposes direction-aware binding.
#[derive(Serialize, Deserialize)]
pub struct GhrrHyperCube {
    /// Dimension of stored vectors.
    dim: usize,
    /// Atomic symbol → GHRR phase vector codebook.
    codebook: HashMap<String, Vec<f64>>,
    /// Phase bias for directional binding.
    eta: f64,
    /// Next seed for deterministic symbol generation.
    next_seed: u64,
}

impl Default for GhrrHyperCube {
    fn default() -> Self {
        Self::new(GHRR_DIM, GHRR_ETA)
    }
}

impl GhrrHyperCube {
    /// Create a new empty hypercube with the given dimension and phase bias.
    ///
    /// When `eta = 1.0`, all binding operations match FHRR exactly.
    /// When `eta = GHRR_ETA` (default), binding is non-commutative.
    pub fn new(dim: usize, eta: f64) -> Self {
        Self {
            dim,
            codebook: HashMap::new(),
            eta,
            next_seed: 0xDEAD_BEEF_CAFE_0001,
        }
    }

    /// Create a new hypercube with the default GHRR eta (golden ratio).
    pub fn new_default(dim: usize) -> Self {
        Self::new(dim, GHRR_ETA)
    }

    /// Current vector dimension.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Current phase bias η.
    pub fn eta(&self) -> f64 {
        self.eta
    }

    /// Set a new phase bias for this hypercube.
    pub fn set_eta(&mut self, eta: f64) {
        self.eta = eta;
    }

    /// Number of symbols in the codebook.
    pub fn symbol_count(&self) -> usize {
        self.codebook.len()
    }

    /// Get a symbol's GHRR vector by name.
    pub fn get_symbol(&self, name: &str) -> Option<&[f64]> {
        self.codebook.get(name).map(|v| v.as_slice())
    }

    /// Add a symbol with a randomly generated GHRR vector.
    /// If the symbol already exists, it is not overwritten.
    pub fn add_symbol(&mut self, name: &str) {
        if !self.codebook.contains_key(name) {
            let seed = self.next_seed;
            self.next_seed = self.next_seed.wrapping_mul(0x9E37_79B9);
            let vec = ghrr_random_vector_dim(self.dim, seed);
            self.codebook.insert(name.to_string(), vec);
        }
    }

    /// Add or overwrite a symbol with a specific vector.
    ///
    /// # Panics
    /// If the vector dimension does not match the hypercube dimension.
    pub fn set_symbol(&mut self, name: &str, vec: Vec<f64>) {
        debug_assert_eq!(vec.len(), self.dim, "vector dimension mismatch");
        self.codebook.insert(name.to_string(), vec);
    }

    /// Remove a symbol from the codebook.
    pub fn remove_symbol(&mut self, name: &str) -> Option<Vec<f64>> {
        self.codebook.remove(name)
    }

    /// Get all symbol names in the codebook.
    pub fn symbol_names(&self) -> Vec<String> {
        self.codebook.keys().cloned().collect()
    }

    /// Directional bind of two symbols by name using the hypercube's η.
    ///
    /// Returns the resulting bound vector.
    pub fn bind_symbols_dir(&self, a: &str, b: &str) -> Result<Vec<f64>, String> {
        let va = self
            .get_symbol(a)
            .ok_or_else(|| format!("symbol not found: {a}"))?;
        let vb = self
            .get_symbol(b)
            .ok_or_else(|| format!("symbol not found: {b}"))?;
        Ok(ghrr_bind_dir(va, vb, self.eta))
    }

    /// Bind symbols with an explicit η (overriding the hypercube's default).
    pub fn bind_symbols_dir_with(&self, a: &str, b: &str, eta: f64) -> Result<Vec<f64>, String> {
        let va = self
            .get_symbol(a)
            .ok_or_else(|| format!("symbol not found: {a}"))?;
        let vb = self
            .get_symbol(b)
            .ok_or_else(|| format!("symbol not found: {b}"))?;
        Ok(ghrr_bind_dir(va, vb, eta))
    }

    /// Directional unbind: c ⊘ a using the hypercube's η.
    pub fn unbind_symbols_dir(&self, c: &str, a: &str) -> Result<Vec<f64>, String> {
        let vc = self
            .get_symbol(c)
            .ok_or_else(|| format!("symbol not found: {c}"))?;
        let va = self
            .get_symbol(a)
            .ok_or_else(|| format!("symbol not found: {a}"))?;
        Ok(ghrr_unbind_dir(vc, va, self.eta))
    }

    /// Bundle two symbols by name and return the resulting vector.
    pub fn bundle_symbols(&self, a: &str, b: &str) -> Result<Vec<f64>, String> {
        let va = self
            .get_symbol(a)
            .ok_or_else(|| format!("symbol not found: {a}"))?;
        let vb = self
            .get_symbol(b)
            .ok_or_else(|| format!("symbol not found: {b}"))?;
        Ok(ghrr_bundle_two(va, vb))
    }

    /// Compute the pairwise similarity matrix for a list of symbol names.
    ///
    /// Returns an N×N matrix where entry [i][j] = similarity(symbols[i], symbols[j]).
    /// All symbols must exist in the codebook.
    pub fn compute_similarity_matrix(&self, symbols: &[&str]) -> Result<Vec<Vec<f64>>, String> {
        let n = symbols.len();
        let vecs: Vec<&[f64]> = symbols
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
                mat[i][j] = ghrr_similarity(vecs[i], vecs[j]);
            }
        }
        Ok(mat)
    }

    /// Find the closest codebook symbol to a query vector.
    ///
    /// Returns `(name, similarity)` or `None` if the codebook is empty.
    pub fn nearest_symbol(&self, query: &[f64]) -> Option<(&str, f64)> {
        self.codebook
            .iter()
            .map(|(name, vec)| (name.as_str(), ghrr_similarity(query, vec)))
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
    }

    /// Find the closest codebook symbol above a threshold.
    pub fn nearest_symbol_threshold(&self, query: &[f64], threshold: f64) -> Option<(&str, f64)> {
        self.nearest_symbol(query)
            .filter(|(_, sim)| *sim >= threshold)
    }
}

// ---------------------------------------------------------------------------
// Diffusion Activation Retrieval (B130) — GHRR variant
// ---------------------------------------------------------------------------

/// Configuration for diffusion activation retrieval (B130).
///
/// Implements HeLa-Mem-style diffusion on the GHRR HyperCube codebook:
/// 1. Seed activation = similarity(query, symbol)
/// 2. Propagate along similarity edges for N steps
/// 3. Return top-K by accumulated activation
#[derive(Serialize, Deserialize)]
pub struct GhrrDiffusionConfig {
    /// Number of diffusion steps (default: 3).
    pub steps: usize,
    /// Activation decay per step (0=no memory, 1=full memory, default: 0.7).
    pub decay: f64,
    /// Number of top results to return (default: 5).
    pub top_k: usize,
    /// Minimum similarity threshold for edge creation (default: 0.15).
    pub edge_threshold: f64,
    /// Minimum final activation to include in results (default: 0.01).
    pub activation_threshold: f64,
}

impl Default for GhrrDiffusionConfig {
    fn default() -> Self {
        Self {
            steps: 3,
            decay: 0.7,
            top_k: 5,
            edge_threshold: 0.15,
            activation_threshold: 0.01,
        }
    }
}

/// A scored result from diffusion retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhrrDiffusionResult {
    /// Symbol name.
    pub name: String,
    /// Accumulated activation after diffusion.
    pub activation: f64,
    /// Direct similarity to the query.
    pub seed_similarity: f64,
}

impl GhrrHyperCube {
    /// Retrieve symbols via diffusion activation (B130).
    ///
    /// Unlike single-hop similarity search, this propagates activation
    /// through the codebook graph for `config.steps` iterations:
    ///
    ///   activation[0][i] = sim(query, symbol[i])
    ///   activation[t][i] = decay · activation[t-1][i]
    ///       + (1-decay) · mean_j( sim(i,j) · activation[t-1][j] )
    ///
    /// Returns top-K symbols by final activation, excluding the query
    /// itself if its name appears in the codebook.
    pub fn diffusion_retrieve(
        &self,
        query: &[f64],
        config: &GhrrDiffusionConfig,
    ) -> Vec<GhrrDiffusionResult> {
        let n = self.codebook.len();
        if n == 0 {
            return Vec::new();
        }

        let symbols: Vec<&str> = self.codebook.keys().map(|s| s.as_str()).collect();
        let vecs: Vec<&[f64]> = symbols
            .iter()
            .map(|&name| &self.codebook[name][..])
            .collect();

        // Step 1: compute seed activations
        let mut seed_act: Vec<f64> = Vec::with_capacity(n);
        for i in 0..n {
            let s = ghrr_similarity(query, vecs[i]);
            seed_act.push(s);
        }

        // Step 2: build sparse similarity graph
        let mut sim_matrix: Vec<Vec<f64>> = Vec::with_capacity(n);
        for i in 0..n {
            let mut row = Vec::with_capacity(n);
            for j in 0..n {
                let sim = if i == j {
                    1.0
                } else {
                    let s = ghrr_similarity(vecs[i], vecs[j]);
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

        // Step 3: diffuse activation
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

        // Step 4: collect and rank results
        let mut results: Vec<GhrrDiffusionResult> = symbols
            .iter()
            .enumerate()
            .filter(|&(i, _)| activation[i] >= config.activation_threshold)
            .map(|(i, &name)| GhrrDiffusionResult {
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

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    // ── Core operation tests ────────────────────────────────────────────

    #[test]
    fn test_ghrr_bind_non_commutative() {
        let a = ghrr_random_vector_dim(256, 42);
        let b = ghrr_random_vector_dim(256, 99);
        let eta = GHRR_ETA;

        let ab = ghrr_bind_dir(&a, &b, eta);
        let ba = ghrr_bind_dir(&b, &a, eta);

        let sim = ghrr_similarity(&ab, &ba);
        assert!(
            sim < 0.95,
            "non-commutative binding: bind_dir(a,b,φ) should differ from bind_dir(b,a,φ), sim={}",
            sim
        );
    }

    #[test]
    fn test_ghrr_bind_commutative_when_eta_one() {
        // When η = 1, GHRR is equivalent to FHRR (fully commutative).
        let a = ghrr_random_vector_dim(128, 1);
        let b = ghrr_random_vector_dim(128, 2);

        let ab = ghrr_bind_dir(&a, &b, 1.0);
        let ba = ghrr_bind_dir(&b, &a, 1.0);

        let sim = ghrr_similarity(&ab, &ba);
        assert!(
            approx_eq(sim, 1.0, 1e-12),
            "η=1 should give commutative FHRR bind, sim={}",
            sim
        );
    }

    #[test]
    fn test_ghrr_bind_reversible() {
        let a = ghrr_random_vector_dim(256, 42);
        let b = ghrr_random_vector_dim(256, 99);
        let eta = GHRR_ETA;

        let bound = ghrr_bind_dir(&a, &b, eta);
        let rebound = ghrr_unbind_dir(&bound, &a, eta);

        let sim = ghrr_similarity(&b, &rebound);
        assert!(
            sim > 0.95,
            "unbind_dir(bind_dir(a,b,η), a, η) ≈ b, sim={}",
            sim
        );
    }

    #[test]
    fn test_ghrr_eta_one_is_fhrr_equivalent() {
        // At η = 1, bind_dir(a,b,1) = 1·θ_a + θ_b = θ_a + θ_b = FHRR bind.
        let a = ghrr_random_vector_dim(128, 10);
        let b = ghrr_random_vector_dim(128, 20);

        let ghrr_result = ghrr_bind_dir(&a, &b, 1.0);
        let fhrr_result: Vec<f64> = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x + y) % std::f64::consts::TAU)
            .collect();

        let sim = ghrr_similarity(&ghrr_result, &fhrr_result);
        assert!(
            approx_eq(sim, 1.0, 1e-12),
            "η=1 GHRR should match FHRR bind, sim={}",
            sim
        );
    }

    #[test]
    fn test_ghrr_eta_zero_suppresses_first_operand() {
        // At η = 0, bind_dir(a,b,0) = 0·θ_a + θ_b = θ_b.
        let a = ghrr_random_vector_dim(256, 42);
        let b = ghrr_random_vector_dim(256, 99);

        let ab = ghrr_bind_dir(&a, &b, 0.0);
        let sim = ghrr_similarity(&ab, &b);

        assert!(
            approx_eq(sim, 1.0, 1e-12),
            "η=0 should preserve second operand unchanged, sim={}",
            sim
        );
    }

    #[test]
    fn test_ghrr_self_similarity_one() {
        let a = ghrr_random_vector_dim(256, 7);
        let sim = ghrr_similarity(&a, &a);
        assert!(approx_eq(sim, 1.0, 1e-12));
    }

    #[test]
    fn test_ghrr_random_vectors_low_similarity() {
        let a = ghrr_random_vector_dim(GHRR_DIM, 10);
        let b = ghrr_random_vector_dim(GHRR_DIM, 20);
        let sim = ghrr_similarity(&a, &b);
        assert!(
            sim.abs() < 0.1,
            "random vectors should have near-zero similarity, got {}",
            sim
        );
    }

    #[test]
    fn test_ghrr_bundle_similar_to_components() {
        let a = ghrr_random_vector_dim(512, 30);
        let b = ghrr_random_vector_dim(512, 40);
        let c = ghrr_bundle_two(&a, &b);
        let sim_a = ghrr_similarity(&c, &a);
        let sim_b = ghrr_similarity(&c, &b);
        assert!(sim_a > 0.3, "bundle should be similar to a, got {}", sim_a);
        assert!(sim_b > 0.3, "bundle should be similar to b, got {}", sim_b);
    }

    #[test]
    fn test_ghrr_eta_variation_affects_direction() {
        let a = ghrr_random_vector_dim(256, 42);
        let b = ghrr_random_vector_dim(256, 99);

        let r1 = ghrr_bind_dir(&a, &b, 0.5);
        let r2 = ghrr_bind_dir(&a, &b, 1.5);
        let r3 = ghrr_bind_dir(&a, &b, GHRR_ETA);

        // Different η values should produce different results
        let sim_12 = ghrr_similarity(&r1, &r2);
        let sim_13 = ghrr_similarity(&r1, &r3);

        assert!(
            sim_12 < 0.95,
            "different η values should produce different results, sim_12={}",
            sim_12
        );
        assert!(
            sim_13 < 0.95,
            "different η values should produce different results, sim_13={}",
            sim_13
        );
    }

    #[test]
    fn test_ghrr_default_eta() {
        assert!(
            approx_eq(GHRR_ETA, 1.618033988749895, 1e-12),
            "GHRR_ETA should be the golden ratio"
        );
    }

    #[test]
    fn test_ghrr_dim_constant() {
        assert_eq!(GHRR_DIM, 2048);
    }

    #[test]
    fn test_ghrr_bundle_empty_returns_empty() {
        let result = ghrr_bundle(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_ghrr_similarity_zero_length() {
        let sim = ghrr_similarity(&[], &[]);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_ghrr_permute_is_deterministic() {
        let a = ghrr_random_vector_dim(256, 50);
        let p1 = ghrr_permute(&a, 7);
        let p2 = ghrr_permute(&a, 7);
        let sim = ghrr_similarity(&p1, &p2);
        assert!(approx_eq(sim, 1.0, 1e-12));
    }

    #[test]
    fn test_ghrr_bind_dir_non_commutative_symmetry() {
        // Verify mathematical properties of the asymmetry.
        let a = ghrr_random_vector_dim(256, 42);
        let b = ghrr_random_vector_dim(256, 99);

        // At η = GHRR_ETA, ab and ba should be clearly different
        let ab = ghrr_bind_dir(&a, &b, GHRR_ETA);
        let ba = ghrr_bind_dir(&b, &a, GHRR_ETA);
        let sim_ab_ba = ghrr_similarity(&ab, &ba);

        // At η = 0: ab = b, ba = a; sim(ab, ba) = sim(b, a) = sim(a, b)
        let ab0 = ghrr_bind_dir(&a, &b, 0.0);
        let ba0 = ghrr_bind_dir(&b, &a, 0.0);
        let sim_ab_ba_0 = ghrr_similarity(&ab0, &ba0);
        let sim_a_b = ghrr_similarity(&a, &b);

        assert!(
            approx_eq(sim_ab_ba_0, sim_a_b, 1e-12),
            "at η=0, cross-sim should equal input similarity, got sim_ab_ba_0={} sim_a_b={}",
            sim_ab_ba_0,
            sim_a_b,
        );

        // η=1 should be perfectly commutative: ab = θ_a + θ_b = ba
        let ab1 = ghrr_bind_dir(&a, &b, 1.0);
        let ba1 = ghrr_bind_dir(&b, &a, 1.0);
        let sim_ab_ba_1 = ghrr_similarity(&ab1, &ba1);
        assert!(
            approx_eq(sim_ab_ba_1, 1.0, 1e-12),
            "at η=1, bind should be perfectly commutative, sim={}",
            sim_ab_ba_1
        );

        // At η = φ, ab != ba (non-commutative)
        assert!(
            sim_ab_ba < 1.0 - 1e-6,
            "at η=φ, ab and ba should differ, sim={}",
            sim_ab_ba
        );
    }

    #[test]
    fn test_ghrr_cleanup_finds_correct() {
        let candidates_vec: Vec<Vec<f64>> = (0..20)
            .map(|i| ghrr_random_vector_dim(256, i as u64))
            .collect();
        let candidates: Vec<&[f64]> = candidates_vec.iter().map(|v| v.as_slice()).collect();
        let query = &candidates_vec[7];
        let idx = ghrr_cleanup_always(query, &candidates).unwrap();
        assert_eq!(idx, 7, "cleanup should find exact match");
    }

    #[test]
    fn test_ghrr_cleanup_with_noise() {
        let candidates_vec: Vec<Vec<f64>> = (0..10)
            .map(|i| ghrr_random_vector_dim(256, i as u64 + 100))
            .collect();
        let candidates: Vec<&[f64]> = candidates_vec.iter().map(|v| v.as_slice()).collect();
        let noisy: Vec<f64> = candidates_vec[0]
            .iter()
            .map(|theta| {
                let noise = (rand::random::<f64>() - 0.5) * 0.5;
                (*theta + noise) % std::f64::consts::TAU
            })
            .collect();
        let idx = ghrr_cleanup_always(&noisy, &candidates).unwrap();
        assert_eq!(idx, 0, "noisy vector should still match original");
    }

    // ── GhrrVector wrapper tests ────────────────────────────────────────

    #[test]
    fn test_ghrr_vector_wrapper_bind_dir() {
        let v1 = GhrrVector::random_dim(256, 1);
        let v2 = GhrrVector::random_dim(256, 2);

        let bound = v1.bind_dir(&v2, GHRR_ETA);
        assert_eq!(bound.dim(), 256);

        let rebound = bound.unbind_dir(&v1, GHRR_ETA);
        let sim = rebound.similarity(&v2);
        assert!(
            sim > 0.95,
            "wrapper unbind should recover original, sim={}",
            sim
        );
    }

    #[test]
    fn test_ghrr_vector_wrapper_bundle() {
        let v1 = GhrrVector::random_dim(256, 1);
        let v2 = GhrrVector::random_dim(256, 2);

        let bundled = v1.bundle(&v2);
        let sim_to_v1 = bundled.similarity(&v1);
        assert!(sim_to_v1 > 0.3);
    }

    #[test]
    fn test_ghrr_vector_into_phases() {
        let v = GhrrVector::random_dim(128, 42);
        let phases = v.into_phases();
        assert_eq!(phases.len(), 128);
    }

    // ── GhrrHyperCube tests ─────────────────────────────────────────────

    #[test]
    fn test_ghrr_hypercube_add_and_get() {
        let mut hc = GhrrHyperCube::new(256, GHRR_ETA);
        hc.add_symbol("circle");
        hc.add_symbol("square");
        assert_eq!(hc.symbol_count(), 2);
        assert!(hc.get_symbol("circle").is_some());
        assert!(hc.get_symbol("triangle").is_none());
    }

    #[test]
    fn test_ghrr_hypercube_add_no_duplicate() {
        let mut hc = GhrrHyperCube::new(128, GHRR_ETA);
        hc.add_symbol("alpha");
        hc.add_symbol("alpha");
        assert_eq!(hc.symbol_count(), 1);
    }

    #[test]
    fn test_ghrr_hypercube_bind_symbols_dir() {
        let mut hc = GhrrHyperCube::new(256, GHRR_ETA);
        hc.add_symbol("role");
        hc.add_symbol("filler");

        let bound = hc.bind_symbols_dir("role", "filler").unwrap();
        assert_eq!(bound.len(), 256);

        // Bound should be dissimilar to either component
        let role_v = hc.get_symbol("role").unwrap();
        let filler_v = hc.get_symbol("filler").unwrap();
        let sim_role = ghrr_similarity(&bound, role_v);
        let sim_filler = ghrr_similarity(&bound, filler_v);
        assert!(
            sim_role.abs() < 0.3,
            "bound should be dissimilar to role, sim={}",
            sim_role
        );
        assert!(
            sim_filler.abs() < 0.3,
            "bound should be dissimilar to filler, sim={}",
            sim_filler
        );
    }

    #[test]
    fn test_ghrr_hypercube_bind_symbols_non_commutative() {
        let mut hc = GhrrHyperCube::new(256, GHRR_ETA);
        hc.add_symbol("x");
        hc.add_symbol("y");

        let xy = hc.bind_symbols_dir("x", "y").unwrap();
        let yx = hc.bind_symbols_dir("y", "x").unwrap();

        let sim = ghrr_similarity(&xy, &yx);
        assert!(
            sim < 0.95,
            "direction matters in GHRR: bind(x,y,φ) ≠ bind(y,x,φ), sim={}",
            sim
        );
    }

    #[test]
    fn test_ghrr_hypercube_bind_symbols_dir_commutative_when_eta_one() {
        let mut hc = GhrrHyperCube::new(256, 1.0);
        hc.add_symbol("a");
        hc.add_symbol("b");

        let ab = hc.bind_symbols_dir("a", "b").unwrap();
        let ba = hc.bind_symbols_dir("b", "a").unwrap();
        let sim = ghrr_similarity(&ab, &ba);
        assert!(
            approx_eq(sim, 1.0, 1e-12),
            "with η=1, bind_symbols_dir should be commutative, sim={}",
            sim
        );
    }

    #[test]
    fn test_ghrr_hypercube_bind_with_explicit_eta() {
        let mut hc = GhrrHyperCube::new(256, 1.0);
        hc.add_symbol("a");
        hc.add_symbol("b");

        // Override the hypercube's η explicitly
        let bound = hc.bind_symbols_dir_with("a", "b", GHRR_ETA).unwrap();
        assert_eq!(bound.len(), 256);

        // Verify non-commutativity with φ
        let ba = hc.bind_symbols_dir_with("b", "a", GHRR_ETA).unwrap();
        let sim = ghrr_similarity(&bound, &ba);
        assert!(
            sim < 0.95,
            "explicit η=φ should be non-commutative, sim={}",
            sim
        );
    }

    #[test]
    fn test_ghrr_hypercube_unbind_symbols_dir() {
        let mut hc = GhrrHyperCube::new(256, GHRR_ETA);
        hc.add_symbol("a");
        hc.add_symbol("b");

        // Manually set the vectors for precise testing
        let va = ghrr_random_vector_dim(256, 42);
        let vb = ghrr_random_vector_dim(256, 99);
        hc.set_symbol("a", va.clone());
        hc.set_symbol("b", vb.clone());

        let bound = hc.bind_symbols_dir("a", "b").unwrap();
        hc.set_symbol("bound", bound);

        let rebound = hc.unbind_symbols_dir("bound", "a").unwrap();
        let sim = ghrr_similarity(&rebound, &vb);
        assert!(sim > 0.95, "unbind should recover symbol b, sim={}", sim);
    }

    #[test]
    fn test_ghrr_hypercube_bundle_symbols() {
        let mut hc = GhrrHyperCube::new(128, GHRR_ETA);
        hc.add_symbol("red");
        hc.add_symbol("blue");

        let bundled = hc.bundle_symbols("red", "blue").unwrap();
        assert_eq!(bundled.len(), 128);

        let red_v = hc.get_symbol("red").unwrap();
        let blue_v = hc.get_symbol("blue").unwrap();
        assert!(ghrr_similarity(&bundled, red_v) > 0.3);
        assert!(ghrr_similarity(&bundled, blue_v) > 0.3);
    }

    #[test]
    fn test_ghrr_hypercube_set_eta() {
        let mut hc = GhrrHyperCube::new(256, 1.0);
        assert!(approx_eq(hc.eta(), 1.0, 1e-12));
        hc.set_eta(GHRR_ETA);
        assert!(approx_eq(hc.eta(), GHRR_ETA, 1e-12));
    }

    #[test]
    fn test_ghrr_hypercube_set_symbol_overwrites() {
        let mut hc = GhrrHyperCube::new(64, GHRR_ETA);
        hc.add_symbol("test");
        let new_vec = ghrr_random_vector_dim(64, 9999);
        hc.set_symbol("test", new_vec.clone());
        let retrieved = hc.get_symbol("test").unwrap();
        assert_eq!(retrieved, &new_vec);
    }

    #[test]
    fn test_ghrr_hypercube_remove_symbol() {
        let mut hc = GhrrHyperCube::new(64, GHRR_ETA);
        hc.add_symbol("temp");
        assert!(hc.get_symbol("temp").is_some());
        let removed = hc.remove_symbol("temp");
        assert!(removed.is_some());
        assert!(hc.get_symbol("temp").is_none());
    }

    #[test]
    fn test_ghrr_similarity_matrix() {
        let mut hc = GhrrHyperCube::new(256, GHRR_ETA);
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

    #[test]
    fn test_ghrr_nearest_symbol() {
        let mut hc = GhrrHyperCube::new(256, GHRR_ETA);
        hc.add_symbol("dog");
        hc.add_symbol("cat");
        hc.add_symbol("bird");
        let cat_vec = hc.get_symbol("cat").unwrap().to_vec();
        let (name, sim) = hc.nearest_symbol(&cat_vec).unwrap();
        assert_eq!(name, "cat");
        assert!(approx_eq(sim, 1.0, 1e-6));
    }

    #[test]
    fn test_ghrr_nearest_symbol_empty_codebook() {
        let hc = GhrrHyperCube::new(64, GHRR_ETA);
        let query = ghrr_random_vector_dim(64, 0);
        assert!(hc.nearest_symbol(&query).is_none());
    }

    #[test]
    fn test_ghrr_hypercube_default_constructor() {
        let hc = GhrrHyperCube::default();
        assert_eq!(hc.dim(), GHRR_DIM);
        assert!(approx_eq(hc.eta(), GHRR_ETA, 1e-12));
        assert_eq!(hc.symbol_count(), 0);
    }

    #[test]
    fn test_ghrr_hypercube_new_default() {
        let hc = GhrrHyperCube::new_default(128);
        assert_eq!(hc.dim(), 128);
        assert!(approx_eq(hc.eta(), GHRR_ETA, 1e-12));
    }

    #[test]
    fn test_ghrr_symbol_names() {
        let mut hc = GhrrHyperCube::new(64, GHRR_ETA);
        hc.add_symbol("a");
        hc.add_symbol("b");
        hc.add_symbol("c");
        let mut names = hc.symbol_names();
        names.sort();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    // ── GHRR Diffusion Activation Retrieval tests ───────────────────────

    #[test]
    fn test_ghrr_diffusion_empty_codebook() {
        let hc = GhrrHyperCube::new(128, GHRR_ETA);
        let query = ghrr_random_vector_dim(128, 0);
        let results = hc.diffusion_retrieve(&query, &GhrrDiffusionConfig::default());
        assert!(
            results.is_empty(),
            "empty codebook should return empty results"
        );
    }

    #[test]
    fn test_ghrr_diffusion_single_symbol() {
        let mut hc = GhrrHyperCube::new(128, GHRR_ETA);
        hc.add_symbol("only");
        let query = hc.get_symbol("only").unwrap().to_vec();
        let config = GhrrDiffusionConfig {
            steps: 1,
            decay: 0.9,
            edge_threshold: -0.1,
            activation_threshold: 0.001,
            ..Default::default()
        };
        let results = hc.diffusion_retrieve(&query, &config);
        assert_eq!(results.len(), 1, "should find the only symbol");
        assert_eq!(results[0].name, "only");
        assert!(
            results[0].activation > 0.8,
            "activation should be high for exact match"
        );
    }

    #[test]
    fn test_ghrr_diffusion_top_k_limit() {
        let mut hc = GhrrHyperCube::new(128, GHRR_ETA);
        for i in 0..20 {
            hc.add_symbol(&format!("s{i}"));
        }
        let query = hc.get_symbol("s0").unwrap().to_vec();
        let config = GhrrDiffusionConfig {
            steps: 1,
            decay: 0.5,
            top_k: 3,
            edge_threshold: -0.1,
            activation_threshold: 0.0,
        };
        let results = hc.diffusion_retrieve(&query, &config);
        assert!(
            results.len() <= 3,
            "should return at most top_k=3 results, got {}",
            results.len()
        );
    }

    #[test]
    fn test_ghrr_diffusion_config_defaults() {
        let config = GhrrDiffusionConfig::default();
        assert_eq!(config.steps, 3);
        assert!((config.decay - 0.7).abs() < 1e-9);
        assert_eq!(config.top_k, 5);
        assert!((config.edge_threshold - 0.15).abs() < 1e-9);
    }

    #[test]
    fn test_ghrr_diffusion_spreads_activation() {
        let mut hc = GhrrHyperCube::new(256, GHRR_ETA);
        hc.add_symbol("seed");

        // Create friend as seed with very small perturbation (guaranteed high similarity)
        let base = ghrr_random_vector_dim(256, 42);
        hc.set_symbol("seed", base.clone());

        let friend: Vec<f64> = base
            .iter()
            .map(|theta| (theta + 0.02 * (rand::random::<f64>() - 0.5)) % std::f64::consts::TAU)
            .collect();
        // Create unrelated with a completely different seed (guaranteed low similarity)
        let unrelated = ghrr_random_vector_dim(256, 9999);

        hc.set_symbol("friend", friend);
        hc.set_symbol("unrelated", unrelated);

        let seed_vec = hc.get_symbol("seed").unwrap();
        let friend_vec = hc.get_symbol("friend").unwrap();
        let unrelated_vec = hc.get_symbol("unrelated").unwrap();

        let sim_friend = ghrr_similarity(seed_vec, friend_vec);
        let sim_unrelated = ghrr_similarity(seed_vec, unrelated_vec);

        // Verify friend is much more similar than unrelated (by construction)
        let margin = sim_friend - sim_unrelated;
        assert!(
            margin > 0.2,
            "friend sim ({sim_friend}) should be much higher than unrelated sim ({sim_unrelated}), margin={margin}"
        );

        let query = seed_vec.to_vec();
        let results = hc.diffusion_retrieve(
            &query,
            &GhrrDiffusionConfig {
                steps: 2,
                decay: 0.5,
                top_k: 3,
                edge_threshold: sim_friend * 0.5,
                activation_threshold: 0.0,
                ..Default::default()
            },
        );

        let friend_act = results
            .iter()
            .find(|r| r.name == "friend")
            .map(|r| r.activation)
            .unwrap_or(0.0);
        let unrelated_act = results
            .iter()
            .find(|r| r.name == "unrelated")
            .map(|r| r.activation)
            .unwrap_or(0.0);
        assert!(
            friend_act > unrelated_act,
            "friend ({friend_act}) should have higher activation than unrelated ({unrelated_act})"
        );
    }
}
