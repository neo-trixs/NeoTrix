//! Shared types extracted to break circular dependencies between nt_core_e8 and nt_core_gwt.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

// ─── Modality (moved from nt_core_gwt::modality_router) ──────────────────

/// Modalities that can carry workspace content for attention routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Modality {
    /// Text / natural language
    Text,
    /// Image / visual
    Image,
    /// Audio / speech
    Audio,
    /// Structured code / data
    Code,
    /// Vector/latent knowledge
    Latent,
}

impl Modality {
    pub const ALL: [Modality; 5] = [
        Modality::Text,
        Modality::Image,
        Modality::Audio,
        Modality::Code,
        Modality::Latent,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Modality::Text => "text",
            Modality::Image => "image",
            Modality::Audio => "audio",
            Modality::Code => "code",
            Modality::Latent => "vector",
        }
    }
}

// ─── E8VsaEmbedding (moved from nt_core_e8_vsa) ──────────────────────────

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
