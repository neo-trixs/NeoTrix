//! E8 → VSA 超向量嵌入层
//!
//! Bridges the discrete E8 hexagram state space (6-bit → 64 states)
//! with continuous VSA hypervectors for gradient-friendly GWT integration.
//!
//! Architecture:
//!   E8 state (u8) → base hypervector (R^D) ⊕ meta-state → VSA bound with task context
//!
//! Uses MAP-BSC (Multiply-Add-Permute) VSA operations via VSAEngine.

use serde::{Serialize, Deserialize};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Default hypervector dimension.
pub const E8_VSA_DIM: usize = 1024;

/// Seed for deterministic E8→VSA mapping.
pub const E8_VSA_SEED: u64 = 0xE8_E8_E8_E8_E8_E8_E8_E8;

/// E8 state → VSA hypervector embedding.
///
/// Maps each of the 64 E8 hexagram states to a unique random hypervector
/// in ℝ^D. The mapping is deterministic (seeded ChaCha12) so the same
/// E8 state always maps to the same hypervector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8VsaEmbedding {
    /// Dimension of the hypervector space
    pub dim: usize,
    /// Deterministic lookup table: [64] × [D] — pre-generated E8→VSA maps
    pub e8_table: Vec<Vec<f64>>,
}

impl Default for E8VsaEmbedding {
    fn default() -> Self {
        Self::new(E8_VSA_DIM)
    }
}

impl E8VsaEmbedding {
    pub fn new(dim: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(E8_VSA_SEED);
        let mut e8_table = Vec::with_capacity(64);
        for _ in 0..64 {
            let vec: Vec<f64> = (0..dim)
                .map(|_| if rng.gen::<f64>() > 0.5 { 1.0 } else { -1.0 })
                .collect();
            e8_table.push(vec);
        }
        Self { dim, e8_table }
    }

    /// Embed an E8 state (0..63) into a normalized hypervector.
    ///
    /// Returns a bipolar (±1) hypervector of dimension `dim`.
    pub fn embed(&self, e8_state: u8) -> &[f64] {
        &self.e8_table[(e8_state & 0b00111111) as usize]
    }

    /// Embed with meta-state binding: hv = bundle(E8(state), meta_hv(meta_bits))
    ///
    /// Bundles the E8 base vector with a meta-state hypervector.
    pub fn embed_with_meta(&self, e8_state: u8, meta_bits: u8) -> Vec<f64> {
        let base = self.embed(e8_state);
        let meta_hv = self.meta_hypervector(meta_bits);
        self.bundle(base, &meta_hv)
    }

    /// Generate a meta-state hypervector deterministically.
    fn meta_hypervector(&self, meta_bits: u8) -> Vec<f64> {
        let mut rng = StdRng::seed_from_u64(E8_VSA_SEED ^ (meta_bits as u64 + 1));
        (0..self.dim)
            .map(|_| if rng.gen::<f64>() > 0.5 { 1.0 } else { -1.0 })
            .collect()
    }

    /// Cosine similarity between two E8-embedded vectors.
    pub fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a = (a.iter().map(|x| x * x).sum::<f64>()).sqrt().max(1e-10);
        let norm_b = (b.iter().map(|x| x * x).sum::<f64>()).sqrt().max(1e-10);
        (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
    }

    /// Bind two hypervectors (element-wise multiply) for compositional VSA.
    pub fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).collect()
    }

    /// Bundle two hypervectors (element-wise add) for superposition.
    pub fn bundle(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter().zip(b.iter()).map(|(x, y)| (x + y) / 2.0).collect()
    }

    /// Permute (circular shift) a hypervector for role-filler binding.
    pub fn permute(&self, v: &[f64], shift: usize) -> Vec<f64> {
        let len = v.len();
        let shift = shift % len;
        let mut result = v.to_vec();
        result.rotate_left(shift);
        result
    }

    /// Find the closest E8 state to a given hypervector (inverse embedding).
    ///
    /// Returns (state_index, similarity).
    pub fn nearest_e8_state(&self, query: &[f64]) -> (u8, f64) {
        let mut best_idx = 0u8;
        let mut best_sim = f64::NEG_INFINITY;
        for (i, hv) in self.e8_table.iter().enumerate() {
            let sim = self.similarity(query, hv);
            if sim > best_sim {
                best_sim = sim;
                best_idx = i as u8;
            }
        }
        (best_idx, best_sim)
    }

    /// Embed the full E8→VSA pairwise similarity matrix (64×64).
    ///
    /// Returns a [64][64] matrix where entry [i][j] = cosine similarity
    /// between embed(state_i) and embed(state_j).
    /// Diagonal is always 1.0 (self-similarity).
    pub fn transition_similarity_matrix(&self) -> Vec<Vec<f64>> {
        let mut mat = vec![vec![0.0; 64]; 64];
        for i in 0..64 {
            let hv_i = self.embed(i as u8);
            for j in 0..64 {
                let hv_j = self.embed(j as u8);
                mat[i][j] = self.similarity(hv_i, hv_j);
            }
        }
        mat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_returns_bipolar_vector() {
        let emb = E8VsaEmbedding::new(128);
        let hv = emb.embed(0);
        assert_eq!(hv.len(), 128);
        for &v in hv {
            assert!((v.abs() - 1.0).abs() < 1e-9, "bipolar values should be ±1");
        }
    }

    #[test]
    fn test_deterministic_mapping() {
        let emb1 = E8VsaEmbedding::new(256);
        let emb2 = E8VsaEmbedding::new(256);
        let hv1 = emb1.embed(42).to_vec();
        let hv2 = emb2.embed(42);
        assert_eq!(hv1, hv2, "deterministic seed should produce identical vectors");
    }

    #[test]
    fn test_different_states_different_vectors() {
        let emb = E8VsaEmbedding::new(128);
        let hv0 = emb.embed(0);
        let hv1 = emb.embed(1);
        let sim = emb.similarity(hv0, hv1);
        assert!(sim.abs() < 0.5, "different states should have low similarity, got {}", sim);
    }

    #[test]
    fn test_self_similarity_is_one() {
        let emb = E8VsaEmbedding::new(64);
        let hv = emb.embed(31);
        let sim = emb.similarity(hv, hv);
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_bind_produces_dissimilar_vector() {
        let emb = E8VsaEmbedding::new(256);
        let a = emb.embed(5);
        let b = emb.embed(10);
        let bound = emb.bind(a, b);
        let sim_to_a = emb.similarity(&bound, a);
        let sim_to_b = emb.similarity(&bound, b);
        assert!(sim_to_a.abs() < 0.3, "bound vector should be dissimilar to inputs");
        assert!(sim_to_b.abs() < 0.3);
    }

    #[test]
    fn test_bundle_is_similar_to_components() {
        let emb = E8VsaEmbedding::new(256);
        let a = emb.embed(7);
        let b = emb.embed(21);
        let bundle = emb.bundle(a, b);
        let sim_a = emb.similarity(&bundle, a);
        let sim_b = emb.similarity(&bundle, b);
        assert!(sim_a > 0.3, "bundle should be similar to component a");
        assert!(sim_b > 0.3, "bundle should be similar to component b");
    }

    #[test]
    fn test_permute_is_reversible() {
        let emb = E8VsaEmbedding::new(128);
        let v = emb.embed(15);
        let shifted = emb.permute(v, 10);
        let unshifted = emb.permute(&shifted, 128 - 10);
        let sim = emb.similarity(v, &unshifted);
        assert!((sim - 1.0).abs() < 1e-9, "permute should be reversible");
    }

    #[test]
    fn test_nearest_e8_state_finds_self() {
        let emb = E8VsaEmbedding::new(128);
        let hv = emb.embed(33);
        let (idx, sim) = emb.nearest_e8_state(hv);
        assert_eq!(idx, 33);
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_embed_with_meta_modifies_vector() {
        let emb = E8VsaEmbedding::new(256);
        let with_meta_0 = emb.embed_with_meta(10, 0);
        let with_meta_1 = emb.embed_with_meta(10, 1);
        let sim = emb.similarity(&with_meta_0, &with_meta_1);
        // Different meta bits should produce different bundled vectors
        assert!(sim < 0.8, "different meta should give different vectors, sim={}", sim);
    }

    #[test]
    fn test_transition_similarity_matrix_shape() {
        let emb = E8VsaEmbedding::new(64);
        let mat = emb.transition_similarity_matrix();
        assert_eq!(mat.len(), 64);
        assert_eq!(mat[0].len(), 64);
        // Diagonal should have high similarity
        for i in 0..64 {
            assert!(mat[i][i] > 0.5, "diagonal similarity should be high");
        }
    }

    #[test]
    fn test_all_64_states_accessible() {
        let emb = E8VsaEmbedding::new(128);
        for state in 0u8..64 {
            let hv = emb.embed(state);
            assert_eq!(hv.len(), 128);
            let (decoded, _) = emb.nearest_e8_state(hv);
            assert_eq!(decoded, state, "roundtrip should recover state {}", state);
        }
    }
}
