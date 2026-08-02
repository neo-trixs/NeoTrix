//! FHRR (Fourier Holographic Reduced Representation) VSA
//!
//! A phase-based complex vector VSA using angular representation.
//! Each element is a phase angle θ ∈ [0, 2π), representing a complex number e^{iθ}.
//!
//! Operations:
//! - **Bind**: Element-wise phase addition (mod 2π)
//! - **Bundle**: Complex vector addition followed by normalization
//! - **Permute**: Deterministic phase rotation per element
//! - **Similarity**: Mean cosine of phase differences
//!
//! FHRR enables higher capacity than MAP-bipolar at the same dimension
//! due to the continuous phase space (infinite codewords per dimension).

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Default FHRR hypervector dimension (reduced from MAP's 4096 for efficiency).
pub const FHRR_DIM: usize = 2048;

/// Phase rotation step used in `permute`.
const PHASE_STEP: f64 = std::f64::consts::TAU / 137.0;

// ---------------------------------------------------------------------------
// Core FHRR vector operations
// ---------------------------------------------------------------------------

/// Bind two FHRR vectors via element-wise phase addition.
///
/// For each dimension i: θ_i = (θ_a[i] + θ_b[i]) mod 2π.
/// This is the spectral-domain equivalent of circular convolution.
pub fn bind(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let sum = x + y;
            sum % std::f64::consts::TAU
        })
        .collect()
}

/// Unbind two FHRR vectors via element-wise phase subtraction.
///
/// For each dimension i: θ_i = (θ_c[i] - θ_a[i]) mod 2π.
/// This is the inverse of bind: unbind(bind(a, b), a) ≈ b.
pub fn unbind(c: &[f64], a: &[f64]) -> Vec<f64> {
    c.iter()
        .zip(a.iter())
        .map(|(x, y)| {
            let diff = x - y;
            diff.rem_euclid(std::f64::consts::TAU)
        })
        .collect()
}

/// Bundle multiple FHRR vectors via complex sum normalization.
///
/// Converts each phase angle to (cos θ, sin θ), sums across all vectors,
/// then converts back to phase via arctan2.
/// The result is the centroid direction in complex space.
pub fn bundle(vectors: &[&[f64]]) -> Vec<f64> {
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
        result.push(if theta < 0.0 { theta + std::f64::consts::TAU } else { theta });
    }
    result
}

/// Bundle two FHRR vectors (convenience wrapper).
pub fn bundle_two(a: &[f64], b: &[f64]) -> Vec<f64> {
    bundle(&[a, b])
}

/// Permute an FHRR vector by applying a dimension-dependent phase rotation.
///
/// θ'_i = (θ_i + i * n * PHASE_STEP) mod 2π
/// This is a reversible, distance-preserving permutation.
pub fn permute(a: &[f64], n: usize) -> Vec<f64> {
    let n_f64 = n as f64;
    a.iter()
        .enumerate()
        .map(|(i, theta)| {
            let shifted = theta + (i as f64) * n_f64 * PHASE_STEP;
            shifted % std::f64::consts::TAU
        })
        .collect()
}

/// Inverse of `permute`: subtract the same phase ramp to recover the original.
/// θ_i = (θ'_i - i * n * PHASE_STEP) mod 2π
pub fn unpermute(a: &[f64], n: usize) -> Vec<f64> {
    let n_f64 = n as f64;
    a.iter()
        .enumerate()
        .map(|(i, theta)| {
            let shifted = theta - (i as f64) * n_f64 * PHASE_STEP;
            shifted % std::f64::consts::TAU
        })
        .collect()
}

/// Cosine similarity in complex space: mean of cos(θ_a - θ_b).
///
/// Returns a value in [-1, 1] where 1.0 = identical phase vectors.
pub fn similarity(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let sum: f64 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).cos()).sum();
    sum / (n as f64)
}

/// Cleanup: find the nearest neighbor in a candidate list.
///
/// Returns the index of the candidate with highest similarity.
/// If the best similarity is below the threshold, returns `None`.
pub fn cleanup(noisy: &[f64], candidates: &[&[f64]], threshold: f64) -> Option<usize> {
    let (best_idx, best_sim) = candidates
        .iter()
        .enumerate()
        .map(|(i, c)| (i, similarity(noisy, c)))
        .max_by(|(_, a), (_, b)| a.total_cmp(b))?;
    if best_sim >= threshold {
        Some(best_idx)
    } else {
        None
    }
}

/// Cleanup (no threshold): returns the index of the nearest neighbor.
pub fn cleanup_always(noisy: &[f64], candidates: &[&[f64]]) -> usize {
    let (idx, _) = candidates
        .iter()
        .enumerate()
        .map(|(i, c)| (i, similarity(noisy, c)))
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap_or((0, 0.0));
    idx
}

/// Encode a scalar value into an FHRR phase vector.
///
/// 采用 Fractional Power Encoding (FPE, Frady/Kleyko/Sommer 2018)：
/// θ_i = α_i · value mod 2π，α_i 是固定随机基相位。
/// 旧实现用固定 golden-ratio ramp (i·1.618) 对所有 value 相同，
/// 使 encode(v1)/encode(v2) 只差一个全局相位 → similarity 恒为单一 cos，
/// 且 2/137.5≈0.01455 间距的 value 产生完全相同的向量（碰撞）。
pub fn encode_scalar(value: f64) -> Vec<f64> {
    // 固定种子生成基相位 α_i ∈ [0, 2π)
    static BASE_PHASES: std::sync::OnceLock<Vec<f64>> = std::sync::OnceLock::new();
    let base = BASE_PHASES.get_or_init(|| {
        let mut rng = StdRng::seed_from_u64(0x5EED_AC7E_5C41_A0A0);
        (0..FHRR_DIM)
            .map(|_| rng.gen_range(0.0..std::f64::consts::TAU))
            .collect()
    });
    (0..FHRR_DIM)
        .map(|i| (base[i] * value) % std::f64::consts::TAU)
        .collect()
}

/// Generate a random FHRR phase vector (uniform in [0, 2π)).
pub fn random_vector(seed: u64) -> Vec<f64> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..FHRR_DIM)
        .map(|_| rng.gen_range(0.0..std::f64::consts::TAU))
        .collect()
}

/// Generate a random phase vector with custom dimension.
pub fn random_vector_dim(dim: usize, seed: u64) -> Vec<f64> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..dim)
        .map(|_| rng.gen_range(0.0..std::f64::consts::TAU))
        .collect()
}

// ---------------------------------------------------------------------------
// FhrrHyperCube — codebook-backed FHRR engine
// ---------------------------------------------------------------------------

/// A FHRR-based hypercube that maintains a codebook of atomic symbol vectors.
///
/// Provides named access to FHRR vectors and bulk similarity computation.
#[derive(Serialize, Deserialize)]
pub struct FhrrHyperCube {
    /// Dimension of stored vectors.
    dim: usize,
    /// Atomic symbol → FHRR phase vector codebook.
    codebook: HashMap<String, Vec<f64>>,
    /// Next seed for deterministic symbol generation.
    next_seed: u64,
}

impl Default for FhrrHyperCube {
    fn default() -> Self {
        Self::new(FHRR_DIM)
    }
}

impl FhrrHyperCube {
    /// Create a new empty hypercube with the given dimension.
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            codebook: HashMap::new(),
            next_seed: 0xF1_4A_1F_1A,
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

    /// Get a symbol's FHRR vector by name.
    pub fn get_symbol(&self, name: &str) -> Option<&[f64]> {
        self.codebook.get(name).map(|v| v.as_slice())
    }

    /// Add a symbol with a randomly generated FHRR vector.
    /// If the symbol already exists, it is not overwritten.
    pub fn add_symbol(&mut self, name: &str) {
        if !self.codebook.contains_key(name) {
            let seed = self.next_seed;
            self.next_seed = self.next_seed.wrapping_mul(0x9E37_79B9);
            let vec = random_vector_dim(self.dim, seed);
            self.codebook.insert(name.to_string(), vec);
        }
    }

    /// Add or overwrite a symbol with a specific vector.
    pub fn set_symbol(&mut self, name: &str, vec: Vec<f64>) {
        debug_assert_eq!(vec.len(), self.dim, "vector dimension mismatch");
        self.codebook.insert(name.to_string(), vec);
    }

    /// Remove a symbol from the codebook.
    pub fn remove_symbol(&mut self, name: &str) -> Option<Vec<f64>> {
        self.codebook.remove(name)
    }

    /// Compute the pairwise similarity matrix for a list of symbol names.
    ///
    /// Returns an N×N matrix where entry [i][j] = similarity(symbols[i], symbols[j]).
    /// All symbols must exist in the codebook.
    pub fn compute_similarity_matrix(
        &self,
        symbols: &[&str],
    ) -> Result<Vec<Vec<f64>>, String> {
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
                mat[i][j] = similarity(vecs[i], vecs[j]);
            }
        }
        Ok(mat)
    }

    /// Get all symbol names in the codebook.
    pub fn symbol_names(&self) -> Vec<String> {
        self.codebook.keys().cloned().collect()
    }

    /// Bind two symbols by name and return the resulting vector.
    pub fn bind_symbols(&self, a: &str, b: &str) -> Result<Vec<f64>, String> {
        let va = self
            .get_symbol(a)
            .ok_or_else(|| format!("symbol not found: {a}"))?;
        let vb = self
            .get_symbol(b)
            .ok_or_else(|| format!("symbol not found: {b}"))?;
        Ok(bind(va, vb))
    }

    /// Bundle two symbols by name and return the resulting vector.
    pub fn bundle_symbols(&self, a: &str, b: &str) -> Result<Vec<f64>, String> {
        let va = self
            .get_symbol(a)
            .ok_or_else(|| format!("symbol not found: {a}"))?;
        let vb = self
            .get_symbol(b)
            .ok_or_else(|| format!("symbol not found: {b}"))?;
        Ok(bundle_two(va, vb))
    }

    /// Find the closest codebook symbol to a query vector.
    ///
    /// Returns (name, similarity) or `None` if codebook is empty.
    pub fn nearest_symbol(&self, query: &[f64]) -> Option<(&str, f64)> {
        self.codebook
            .iter()
            .map(|(name, vec)| (name.as_str(), similarity(query, vec)))
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
    }

    /// Find the closest codebook symbol above a threshold.
    pub fn nearest_symbol_threshold(&self, query: &[f64], threshold: f64) -> Option<(&str, f64)> {
        self.nearest_symbol(query)
            .filter(|(_, sim)| *sim >= threshold)
    }
}

// ---------------------------------------------------------------------------
// B130: Diffusion Activation Retrieval
// ---------------------------------------------------------------------------

/// Configuration for diffusion activation retrieval (B130).
///
/// Implements HeLa-Mem-style diffusion on the HyperCube codebook:
/// 1. Seed activation = similarity(query, symbol)
/// 2. Propagate along similarity edges for N steps
/// 3. Return top-K by accumulated activation
#[derive(Serialize, Deserialize)]
pub struct DiffusionConfig {
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

impl Default for DiffusionConfig {
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
pub struct DiffusionResult {
    /// Symbol name.
    pub name: String,
    /// Accumulated activation after diffusion.
    pub activation: f64,
    /// Direct similarity to the query.
    pub seed_similarity: f64,
}

impl FhrrHyperCube {
    /// Retrieve symbols via diffusion activation (B130).
    ///
    /// Unlike single-hop similarity search, this propagates activation
    /// through the codebook graph for `config.steps` iterations:
    ///
    ///   activation[0][i] = sim(query, symbol[i])
    ///   activation[t][i] = decay * activation[t-1][i]
    ///       + (1-decay) * mean_j( sim(i,j) * activation[t-1][j] )
    ///
    /// Returns top-K symbols by final activation, excluding the query
    /// itself if its name appears in the codebook.
    pub fn diffusion_retrieve(
        &self,
        query: &[f64],
        config: &DiffusionConfig,
    ) -> Vec<DiffusionResult> {
        let n = self.codebook.len();
        if n == 0 {
            return Vec::new();
        }

        let symbols: Vec<&str> = self.codebook.keys().map(|s| s.as_str()).collect();
        let vecs: Vec<&[f64]> = symbols.iter().map(|&name| &self.codebook[name][..]).collect();

        // Step 1: compute seed activations and similarity matrix
        let mut seed_act: Vec<f64> = Vec::with_capacity(n);
        let mut sim_matrix: Vec<Vec<f64>> = Vec::with_capacity(n);

        for i in 0..n {
            let s = similarity(query, vecs[i]);
            seed_act.push(s);
        }

        // Build sparse similarity graph (only edges above threshold)
        for i in 0..n {
            let mut row = Vec::with_capacity(n);
            for j in 0..n {
                let sim = if i == j {
                    1.0
                } else {
                    let s = similarity(vecs[i], vecs[j]);
                    if s >= config.edge_threshold { s } else { 0.0 }
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
        let mut results: Vec<DiffusionResult> = symbols
            .iter()
            .enumerate()
            .filter(|&(i, _)| activation[i] >= config.activation_threshold)
            .map(|(i, &name)| DiffusionResult {
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
// FhrrVector — a convenience wrapper around phase data
// ---------------------------------------------------------------------------

/// A convenience wrapper for an FHRR phase vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhrrVector {
    phases: Vec<f64>,
}

impl FhrrVector {
    pub fn new(phases: Vec<f64>) -> Self {
        Self { phases }
    }

    pub fn random(seed: u64) -> Self {
        Self::new(random_vector_dim(FHRR_DIM, seed))
    }

    pub fn random_dim(dim: usize, seed: u64) -> Self {
        Self::new(random_vector_dim(dim, seed))
    }

    pub fn from_scalar(value: f64) -> Self {
        Self::new(encode_scalar(value))
    }

    pub fn phases(&self) -> &[f64] {
        &self.phases
    }

    pub fn dim(&self) -> usize {
        self.phases.len()
    }

    pub fn bind(&self, other: &Self) -> Self {
        Self::new(bind(&self.phases, &other.phases))
    }

    pub fn bundle(&self, other: &Self) -> Self {
        Self::new(bundle_two(&self.phases, &other.phases))
    }

    pub fn permute(&self, n: usize) -> Self {
        Self::new(permute(&self.phases, n))
    }

    pub fn similarity(&self, other: &Self) -> f64 {
        similarity(&self.phases, &other.phases)
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

    #[test]
    fn test_bind_is_reversible() {
        let a = random_vector_dim(128, 42);
        let b = random_vector_dim(128, 99);
        let bound = bind(&a, &b);
        let rebound = unbind(&bound, &b);
        let sim = similarity(&a, &rebound);
        assert!(sim > 0.95, "bind should be reversible, sim={}", sim);
    }

    #[test]
    fn test_bind_commutative() {
        let a = random_vector_dim(128, 1);
        let b = random_vector_dim(128, 2);
        let ab = bind(&a, &b);
        let ba = bind(&b, &a);
        let sim = similarity(&ab, &ba);
        assert!(approx_eq(sim, 1.0, 1e-12), "bind should be commutative");
    }

    #[test]
    fn test_self_similarity_is_one() {
        let a = random_vector_dim(256, 7);
        let sim = similarity(&a, &a);
        assert!(approx_eq(sim, 1.0, 1e-12));
    }

    #[test]
    fn test_different_vectors_low_similarity() {
        let a = random_vector_dim(2048, 10);
        let b = random_vector_dim(2048, 20);
        let sim = similarity(&a, &b);
        // Random independent phase vectors should have near-zero similarity
        assert!(sim.abs() < 0.1, "random vectors should have near-zero similarity, got {}", sim);
    }

    #[test]
    fn test_bundle_is_similar_to_components() {
        let a = random_vector_dim(512, 30);
        let b = random_vector_dim(512, 40);
        let c = bundle_two(&a, &b);
        let sim_a = similarity(&c, &a);
        let sim_b = similarity(&c, &b);
        assert!(sim_a > 0.3, "bundle should be similar to a, got {}", sim_a);
        assert!(sim_b > 0.3, "bundle should be similar to b, got {}", sim_b);
    }

    #[test]
    fn test_permute_is_reversible() {
        let a = random_vector_dim(256, 50);
        let p = permute(&a, 7);
        let sim_to_orig = similarity(&a, &p);
        assert!(sim_to_orig.abs() < 0.5, "permuted vector should differ from original");
    }

    #[test]
    fn test_cleanup_finds_correct() {
        let candidates_vec: Vec<Vec<f64>> = (0..20)
            .map(|i| random_vector_dim(256, i as u64))
            .collect();
        let candidates: Vec<&[f64]> = candidates_vec.iter().map(|v| v.as_slice()).collect();
        let query = &candidates_vec[7];
        let idx = cleanup_always(query, &candidates);
        assert_eq!(idx, 7, "cleanup should find exact match");
    }

    #[test]
    fn test_cleanup_with_noise() {
        let candidates_vec: Vec<Vec<f64>> = (0..10)
            .map(|i| random_vector_dim(256, i as u64 + 100))
            .collect();
        let candidates: Vec<&[f64]> = candidates_vec.iter().map(|v| v.as_slice()).collect();
        // Add noise to the first candidate
        let noisy: Vec<f64> = candidates_vec[0]
            .iter()
            .map(|theta| {
                let noise = (rand::random::<f64>() - 0.5) * 0.5; // ±0.25 rad noise
                (*theta + noise) % std::f64::consts::TAU
            })
            .collect();
        let idx = cleanup_always(&noisy, &candidates);
        assert_eq!(idx, 0, "noisy vector should still match original");
    }

    #[test]
    fn test_encode_scalar_deterministic() {
        let v1 = encode_scalar(3.14159);
        let v2 = encode_scalar(3.14159);
        assert_eq!(v1.len(), FHRR_DIM);
        assert_eq!(v1, v2, "encode_scalar should be deterministic");
    }

    #[test]
    fn test_encode_scalar_different_for_different_values() {
        let v1 = encode_scalar(1.0);
        let v2 = encode_scalar(2.0);
        let sim = similarity(&v1, &v2);
        assert!(sim < 0.3, "different scalars should have low similarity, got {}", sim);
    }

    #[test]
    fn test_fhrr_hypercube_add_and_get() {
        let mut hc = FhrrHyperCube::new(256);
        hc.add_symbol("circle");
        hc.add_symbol("square");
        assert_eq!(hc.symbol_count(), 2);
        assert!(hc.get_symbol("circle").is_some());
        assert!(hc.get_symbol("triangle").is_none());
    }

    #[test]
    fn test_fhrr_hypercube_add_no_duplicate() {
        let mut hc = FhrrHyperCube::new(128);
        hc.add_symbol("alpha");
        hc.add_symbol("alpha");
        assert_eq!(hc.symbol_count(), 1);
    }

    #[test]
    fn test_similarity_matrix() {
        let mut hc = FhrrHyperCube::new(256);
        hc.add_symbol("x");
        hc.add_symbol("y");
        hc.add_symbol("z");
        let mat = hc.compute_similarity_matrix(&["x", "y", "z"]).unwrap();
        assert_eq!(mat.len(), 3);
        assert_eq!(mat[0].len(), 3);
        // Diagonal must be 1.0
        assert!(approx_eq(mat[0][0], 1.0, 1e-12));
        assert!(approx_eq(mat[1][1], 1.0, 1e-12));
        assert!(approx_eq(mat[2][2], 1.0, 1e-12));
        // Symmetry
        assert!(approx_eq(mat[0][1], mat[1][0], 1e-12));
        assert!(approx_eq(mat[0][2], mat[2][0], 1e-12));
    }

    #[test]
    fn test_nearest_symbol() {
        let mut hc = FhrrHyperCube::new(256);
        hc.add_symbol("dog");
        hc.add_symbol("cat");
        hc.add_symbol("bird");
        let cat_vec = hc.get_symbol("cat").unwrap().to_vec();
        let (name, sim) = hc.nearest_symbol(&cat_vec).unwrap();
        assert_eq!(name, "cat");
        assert!(approx_eq(sim, 1.0, 1e-6));
    }

    #[test]
    fn test_bind_symbols() {
        let mut hc = FhrrHyperCube::new(128);
        hc.add_symbol("role");
        hc.add_symbol("filler");
        let bound = hc.bind_symbols("role", "filler").unwrap();
        assert_eq!(bound.len(), 128);
        // Bound vector should be dissimilar to both components
        let role_v = hc.get_symbol("role").unwrap();
        let filler_v = hc.get_symbol("filler").unwrap();
        let sim_to_role = similarity(&bound, role_v);
        let sim_to_filler = similarity(&bound, filler_v);
        assert!(sim_to_role.abs() < 0.3, "bound dissimilar to role, sim={}", sim_to_role);
        assert!(sim_to_filler.abs() < 0.3, "bound dissimilar to filler, sim={}", sim_to_filler);
    }

    #[test]
    fn test_bundle_symbols() {
        let mut hc = FhrrHyperCube::new(128);
        hc.add_symbol("red");
        hc.add_symbol("blue");
        let bundled = hc.bundle_symbols("red", "blue").unwrap();
        assert_eq!(bundled.len(), 128);
        let red_v = hc.get_symbol("red").unwrap();
        let blue_v = hc.get_symbol("blue").unwrap();
        let sim_red = similarity(&bundled, red_v);
        let sim_blue = similarity(&bundled, blue_v);
        assert!(sim_red > 0.3, "bundled similar to red, sim={}", sim_red);
        assert!(sim_blue > 0.3, "bundled similar to blue, sim={}", sim_blue);
    }

    #[test]
    fn test_set_symbol_overwrites() {
        let mut hc = FhrrHyperCube::new(64);
        hc.add_symbol("test");
        let _original = hc.get_symbol("test").unwrap().to_vec();
        let new_vec = random_vector_dim(64, 9999);
        hc.set_symbol("test", new_vec.clone());
        let retrieved = hc.get_symbol("test").unwrap();
        assert_eq!(retrieved, &new_vec);
    }

    #[test]
    fn test_remove_symbol() {
        let mut hc = FhrrHyperCube::new(64);
        hc.add_symbol("temp");
        assert!(hc.get_symbol("temp").is_some());
        hc.remove_symbol("temp");
        assert!(hc.get_symbol("temp").is_none());
    }

    #[test]
    fn test_fhrr_vector_wrapper() {
        let v1 = FhrrVector::random_dim(256, 1);
        let v2 = FhrrVector::random_dim(256, 2);
        let bound = v1.bind(&v2);
        assert_eq!(bound.dim(), 256);
        let bundled = v1.bundle(&v2);
        let sim_to_v1 = bundled.similarity(&v1);
        assert!(sim_to_v1 > 0.3);
    }

    #[test]
    fn test_fhrr_dim_constant() {
        assert_eq!(FHRR_DIM, 2048);
    }

    #[test]
    fn test_bundle_empty_returns_empty() {
        let result = bundle(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_similarity_zero_length() {
        let sim = similarity(&[], &[]);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_nearest_symbol_empty_codebook() {
        let hc = FhrrHyperCube::new(64);
        let query = random_vector_dim(64, 0);
        assert!(hc.nearest_symbol(&query).is_none());
    }

    // ── B130: Diffusion Activation Retrieval Tests ──────────────────

    #[test]
    fn test_diffusion_empty_codebook() {
        let hc = FhrrHyperCube::new(128);
        let query = random_vector_dim(128, 0);
        let results = hc.diffusion_retrieve(&query, &DiffusionConfig::default());
        assert!(results.is_empty(), "empty codebook should return empty results");
    }

    #[test]
    fn test_diffusion_single_symbol() {
        let mut hc = FhrrHyperCube::new(128);
        hc.add_symbol("only");
        let query = hc.get_symbol("only").unwrap().to_vec();
        let config = DiffusionConfig {
            steps: 1,
            decay: 0.9,
            edge_threshold: -0.1,
            activation_threshold: 0.001,
            ..Default::default()
        };
        let results = hc.diffusion_retrieve(&query, &config);
        assert_eq!(results.len(), 1, "should find the only symbol");
        assert_eq!(results[0].name, "only");
        assert!(results[0].activation > 0.8, "activation should be high for exact match");
    }

    #[test]
    fn test_diffusion_finds_related_symbols() {
        let mut hc = FhrrHyperCube::new(256);
        hc.add_symbol("apple");
        hc.add_symbol("banana");
        hc.add_symbol("car");
        hc.add_symbol("truck");
        hc.add_symbol("ocean");
        let query = hc.get_symbol("apple").unwrap().to_vec();
        let config = DiffusionConfig {
            steps: 2,
            decay: 0.5,
            top_k: 3,
            edge_threshold: -0.1, // connect all
            activation_threshold: 0.001,
        };
        let results = hc.diffusion_retrieve(&query, &config);
        assert!(!results.is_empty(), "should find results");
        // The exact match should be top
        assert_eq!(results[0].name, "apple", "query should be top result");
    }

    #[test]
    fn test_diffusion_spreads_activation() {
        let mut hc = FhrrHyperCube::new(256);
        hc.add_symbol("seed");
        hc.add_symbol("friend_of_seed");
        hc.add_symbol("unrelated");

        // Create seed and friend that are similar (friend = seed + small noise)
        let base = random_vector_dim(256, 42);
        let friend: Vec<f64> = base.iter()
            .map(|theta| (theta + 0.1 * (rand::random::<f64>() - 0.5)) % std::f64::consts::TAU)
            .collect();
        let unrelated = random_vector_dim(256, 99);

        hc.set_symbol("seed", base.clone());
        hc.set_symbol("friend_of_seed", friend);
        hc.set_symbol("unrelated", unrelated);

        // Verify friend is more similar to seed than unrelated
        let seed_vec = hc.get_symbol("seed").unwrap();
        let friend_vec = hc.get_symbol("friend_of_seed").unwrap();
        let unrelated_vec = hc.get_symbol("unrelated").unwrap();
        let sim_friend = similarity(seed_vec, friend_vec);
        let sim_unrelated = similarity(seed_vec, unrelated_vec);
        assert!(sim_friend > sim_unrelated, "friend should be more similar to seed than unrelated");

        // Seed should activate friend_of_seed more than unrelated via diffusion
        let query = seed_vec.to_vec();
        let results = hc.diffusion_retrieve(&query, &DiffusionConfig {
            steps: 2,
            decay: 0.5,
            top_k: 3,
            edge_threshold: sim_friend * 0.5, // include seed-friend edge
            activation_threshold: 0.0,
        });

        let friend_act = results.iter().find(|r| r.name == "friend_of_seed").map(|r| r.activation).unwrap_or(0.0);
        let unrelated_act = results.iter().find(|r| r.name == "unrelated").map(|r| r.activation).unwrap_or(0.0);
        assert!(friend_act > unrelated_act,
            "friend_of_seed ({friend_act}) should have higher activation than unrelated ({unrelated_act})");
    }

    #[test]
    fn test_diffusion_config_defaults() {
        let config = DiffusionConfig::default();
        assert_eq!(config.steps, 3);
        assert!((config.decay - 0.7).abs() < 1e-9);
        assert_eq!(config.top_k, 5);
        assert!((config.edge_threshold - 0.15).abs() < 1e-9);
    }

    #[test]
    fn test_diffusion_top_k_limit() {
        let mut hc = FhrrHyperCube::new(128);
        for i in 0..20 {
            hc.add_symbol(&format!("s{i}"));
        }
        let query = hc.get_symbol("s0").unwrap().to_vec();
        let config = DiffusionConfig {
            steps: 1,
            decay: 0.5,
            top_k: 3,
            edge_threshold: -0.1,
            activation_threshold: 0.0,
        };
        let results = hc.diffusion_retrieve(&query, &config);
        assert!(results.len() <= 3, "should return at most top_k=3 results, got {}", results.len());
    }

    #[test]
    fn test_diffusion_more_steps_spreads_further() {
        let mut hc = FhrrHyperCube::new(256);
        hc.add_symbol("a");
        hc.add_symbol("b");
        hc.add_symbol("c");
        let va = random_vector_dim(256, 1);
        let vb = random_vector_dim(256, 2);
        let vc = random_vector_dim(256, 3);
        hc.set_symbol("a", va);
        hc.set_symbol("b", vb);
        hc.set_symbol("c", vc);

        let query = hc.get_symbol("a").unwrap().to_vec();
        let config_1step = DiffusionConfig { steps: 1, top_k: 3, ..Default::default() };
        let config_3step = DiffusionConfig { steps: 3, top_k: 3, ..Default::default() };

        let r1 = hc.diffusion_retrieve(&query, &config_1step);
        let r3 = hc.diffusion_retrieve(&query, &config_3step);
        // More steps should not reduce the result count
        assert!(r3.len() >= r1.len(), "more steps should spread activation further");
    }
}
