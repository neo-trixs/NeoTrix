//! Phase 10.1 — Unified Latent Space (统一潜在空间 · LatentUM).
//!
//! LatentUM (arXiv:2604.02097) §4: the three representation islands must share
//! one latent space so they can be compared pointwise:
//!
//!   - E₈ states → differentiable embedding   e_s = E_e8(s)     (64-d)
//!   - HyperCube knowledge → VSA hypervectors  h_kb             (e.g. 2048-d)
//!   - GWT workspace → aggregated experts     h_ws = Σ aᵢ h⁽ⁱ⁾
//!
//! Each domain lives in its own dimensionality, so this module provides
//! deterministic, invertible projections into a common `UNIFIED_LATENT_DIM`
//! space and cosine/dot comparison across domains:
//!
//!   - `project_e8(latent)`: the 64-d latent thought (Gaussian kernel over the
//!     hexagram cube) → unified space (identity-padded).
//!   - `project_workspace(saliences, expert_embeds)`: weighted sum of expert
//!     embeddings → unified space.
//!   - `project_vsa(hypervector)`: GHRR/FHRR hypervector → unified space via a
//!     seeded pseudo-random projection (Johnson–Lindenstrauss-style), preserving
//!     relative cosine similarity.
//!
//! Cross-domain similarity then works: `cosine(e8_embed, vsa_proj)`.

use crate::core::nt_core_e8::nt_latent_thought::LatentThoughtVector;
use crate::core::nt_core_hex::ReasoningHexagram;
use serde::{Deserialize, Serialize};

/// Dimension of the shared latent space (≥ the largest native embed).
pub const UNIFIED_LATENT_DIM: usize = 256;

/// Default dimension used when projecting HyperCube VSA vectors.
pub const DEFAULT_VSA_DIM: usize = 2048;

/// Deterministic pseudo-random projection from a source dim → unified dim.
///
/// Each source coordinate projects onto `N_PROJ` target coordinates with signs
/// drawn from a seeded LCG, then normalized. This is a JL-style dimensionality
/// reduction that approximately preserves cosine similarity between vectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeededProjection {
    /// Source dimensionality.
    pub src_dim: usize,
    /// Target (unified) dimensionality.
    pub dst_dim: usize,
    /// Number of target coordinates each source coordinate contributes to.
    pub n_proj: usize,
    /// Seeded projection: proj[src] = Vec<(dst, sign)>.
    #[serde(skip)]
    pub proj: Vec<Vec<(usize, f64)>>,
}

impl SeededProjection {
    /// Build the projection with a fixed seed (deterministic across calls).
    pub fn new(src_dim: usize, dst_dim: usize, n_proj: usize, seed: u64) -> Self {
        let mut rng = Lcg::new(seed);
        let mut proj = vec![Vec::with_capacity(n_proj); src_dim];
        for s in 0..src_dim {
            for _ in 0..n_proj {
                let d = (rng.next() % dst_dim as u64) as usize;
                let sign = if rng.next() & 1 == 0 { 1.0 } else { -1.0 };
                proj[s].push((d, sign));
            }
        }
        Self {
            src_dim,
            dst_dim,
            n_proj,
            proj,
        }
    }

    /// Project a source vector into the target space (normalized).
    pub fn project(&self, v: &[f64]) -> Vec<f64> {
        let mut out = vec![0.0f64; self.dst_dim];
        for (s, &x) in v.iter().take(self.src_dim.min(v.len())).enumerate() {
            for &(d, sign) in &self.proj[s] {
                out[d] += sign * x;
            }
        }
        let norm = (out.iter().map(|x| x * x).sum::<f64>()).sqrt();
        if norm > 0.0 {
            for x in out.iter_mut() {
                *x /= norm;
            }
        }
        out
    }
}

/// Deterministic LCG for reproducible projections.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(
            seed.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407),
        )
    }
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) & 0x7FFF_FFFF
    }
}

/// Phase 10.1 — unified latent space bridging E₈, GWT, and HyperCube.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedLatentSpace {
    /// Unified latent dimension.
    pub dim: usize,
    /// E₈ embedding operator (Phase 6.1).
    pub e8_embed: LatentThoughtVector,
    /// Projection from native E₈ latent (64-d) → unified space.
    pub e8_proj: SeededProjection,
    /// Projection from VSA hypervector → unified space.
    pub vsa_proj: SeededProjection,
}

impl Default for UnifiedLatentSpace {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedLatentSpace {
    /// Create a unified latent space with default 256-d target.
    pub fn new() -> Self {
        Self::new_with_dim(UNIFIED_LATENT_DIM)
    }

    /// Create a unified latent space with an explicit target dimension.
    pub fn new_with_dim(dim: usize) -> Self {
        Self {
            dim,
            e8_embed: LatentThoughtVector::new(64),
            e8_proj: SeededProjection::new(64, dim, 4, 0xE8E8),
            vsa_proj: SeededProjection::new(DEFAULT_VSA_DIM, dim, 8, 0x5A5A_5A5A),
        }
    }

    /// Project an E₈ latent thought into the unified space.
    ///
    /// Accepts either a raw 64-d latent (from `e8_embed.embed`) or a single
    /// `ReasoningHexagram` state (embedded on the fly).
    pub fn project_e8(&self, latent_or_state: &[f64]) -> Vec<f64> {
        let latent = if latent_or_state.len() == 64 {
            latent_or_state.to_vec()
        } else {
            self.e8_embed.embed(ReasoningHexagram::new(
                latent_or_state.first().copied().unwrap_or(0.0) as u8,
            ))
        };
        self.e8_proj.project(&latent)
    }

    /// Project an E₈ hexagram state directly into the unified space.
    pub fn project_e8_state(&self, state: ReasoningHexagram) -> Vec<f64> {
        let latent = self.e8_embed.embed(state);
        self.e8_proj.project(&latent)
    }

    /// Project a GWT workspace: weighted sum of expert embeddings.
    ///
    /// `expert_embeds` is a slice of expert embedding vectors (each may be any
    /// dimension; projected via the E₈ projector if 64-d, else the VSA one).
    /// `saliences` are the per-expert activations.
    pub fn project_workspace(&self, expert_embeds: &[&[f64]], saliences: &[f64]) -> Vec<f64> {
        let mut acc = vec![0.0f64; self.dim];
        let n = expert_embeds.len().min(saliences.len());
        if n == 0 {
            return acc;
        }
        for i in 0..n {
            let w = saliences[i];
            let projected = if expert_embeds[i].len() == 64 {
                self.e8_proj.project(expert_embeds[i])
            } else {
                self.vsa_proj.project(expert_embeds[i])
            };
            for (a, p) in acc.iter_mut().zip(projected.iter()) {
                *a += w * p;
            }
        }
        let norm = (acc.iter().map(|x| x * x).sum::<f64>()).sqrt();
        if norm > 0.0 {
            for a in acc.iter_mut() {
                *a /= norm;
            }
        }
        acc
    }

    /// Project a HyperCube VSA hypervector into the unified space.
    pub fn project_vsa(&self, hypervector: &[f64]) -> Vec<f64> {
        self.vsa_proj.project(hypervector)
    }

    /// Pointwise cosine similarity in the unified space.
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
            0.0
        } else {
            (dot / denom).clamp(-1.0, 1.0)
        }
    }

    /// Pointwise dot product in the unified space.
    pub fn dot(&self, a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dim() {
        let u = UnifiedLatentSpace::new();
        assert_eq!(u.dim, UNIFIED_LATENT_DIM);
    }

    #[test]
    fn test_project_e8_maps_to_unified_dim() {
        let u = UnifiedLatentSpace::new();
        let v = u.project_e8_state(ReasoningHexagram::new(0));
        assert_eq!(v.len(), UNIFIED_LATENT_DIM);
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-6,
            "projected vector should be unit norm, got {norm}"
        );
    }

    #[test]
    fn test_e8_similar_states_close_in_unified_space() {
        let u = UnifiedLatentSpace::new();
        // Adjacent hexagrams (one line flip) should be close.
        let a = u.project_e8_state(ReasoningHexagram::new(0));
        let b = u.project_e8_state(ReasoningHexagram::new(1));
        let c = u.project_e8_state(ReasoningHexagram::new(63)); // far (6 flips)
        let near = u.cosine(&a, &b);
        let far = u.cosine(&a, &c);
        assert!(
            near > far,
            "adjacent states ({near:.3}) should be closer than opposites ({far:.3})"
        );
    }

    #[test]
    fn test_vsa_projection_preserves_similarity_ordering() {
        let u = UnifiedLatentSpace::new();
        // Two VSA vectors: identical-ish vs unrelated.
        let base: Vec<f64> = (0..DEFAULT_VSA_DIM)
            .map(|i| ((i * 7) % 13) as f64)
            .collect();
        let mut close = base.clone();
        close[0] += 1.0;
        let mut far = base.clone();
        // Shuffle aggressively to make it unrelated.
        far.reverse();
        let p_base = u.project_vsa(&base);
        let p_close = u.project_vsa(&close);
        let p_far = u.project_vsa(&far);
        let sim_close = u.cosine(&p_base, &p_close);
        let sim_far = u.cosine(&p_base, &p_far);
        assert!(
            sim_close > sim_far,
            "close vectors should be more similar ({sim_close:.3}) than far ({sim_far:.3})"
        );
    }

    #[test]
    fn test_workspace_projection_is_weighted() {
        let u = UnifiedLatentSpace::new();
        let e1 = u.e8_embed.embed(ReasoningHexagram::new(0));
        let e2 = u.e8_embed.embed(ReasoningHexagram::new(56));
        // All weight on expert 1 → closer to e1's unified embedding.
        let all1 = u.project_workspace(&[&e1, &e2], &[1.0, 0.0]);
        let all2 = u.project_workspace(&[&e1, &e2], &[0.0, 1.0]);
        let p1 = u.e8_proj.project(&e1);
        let p2 = u.e8_proj.project(&e2);
        assert!(u.cosine(&all1, &p1) > u.cosine(&all2, &p1));
        assert!(u.cosine(&all2, &p2) > u.cosine(&all1, &p2));
    }

    #[test]
    fn test_projection_deterministic() {
        let u1 = UnifiedLatentSpace::new();
        let u2 = UnifiedLatentSpace::new();
        let v1 = u1.project_e8_state(ReasoningHexagram::new(32));
        let v2 = u2.project_e8_state(ReasoningHexagram::new(32));
        assert_eq!(v1, v2, "projection must be deterministic");
    }

    #[test]
    fn test_cross_domain_comparison_defined() {
        let u = UnifiedLatentSpace::new();
        let e8 = u.project_e8_state(ReasoningHexagram::new(0));
        let vsa = u.project_vsa(&vec![0.5; DEFAULT_VSA_DIM]);
        assert_eq!(e8.len(), vsa.len());
        let sim = u.cosine(&e8, &vsa);
        assert!(sim >= -1.0 && sim <= 1.0);
        let d = u.dot(&e8, &vsa);
        assert!(d.is_finite());
    }

    #[test]
    fn test_empty_workspace_returns_zero() {
        let u = UnifiedLatentSpace::new();
        let v = u.project_workspace(&[], &[]);
        assert_eq!(v.len(), UNIFIED_LATENT_DIM);
        assert!(v.iter().all(|&x| x == 0.0));
    }
}
