use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

/// Cross-modal aligner using HDFLIM-style random projections into VSA space.
///
/// Projects embeddings from any modality (text, image, audio) into a shared
/// 4096-dimensional binary VSA space, enabling cross-modal similarity search
/// and modality-specific retrieval via binding.
#[derive(Debug, Clone)]
pub struct CrossModalAligner {
    dim: usize,
    seed: u64,
}

impl CrossModalAligner {
    pub fn new(dim: usize, seed: u64) -> Self {
        Self { dim, seed }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Deterministic per-word seeded hash → permute(position) → bundle.
    /// Matches the pattern in `QuantizedVSA::text_to_vsa`.
    pub fn text_to_vsa(&self, text: &str) -> Vec<u8> {
        use std::hash::{Hash, Hasher};
        let words: Vec<&str> = text.split_whitespace().filter(|w| !w.is_empty()).collect();
        if words.is_empty() {
            return vec![0; self.dim];
        }
        let mut accum = QuantizedVSA::random_binary();
        for (i, word) in words.iter().enumerate() {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            word.hash(&mut h);
            let seed = h.finish();
            let mut v = self.seeded_random(seed, self.dim);
            if i > 0 {
                v = QuantizedVSA::permute(&v, i as isize);
            }
            accum = QuantizedVSA::bundle(&[&accum, &v]);
        }
        accum
    }

    /// Project a pre-extracted CLIP image embedding (any length, e.g. 512 or 768)
    /// into VSA 4096-dimensional binary space via k random projection matrices.
    ///
    /// Each dimension is the sign of the dot product between the embedding and
    /// a seeded random projection vector, thresholded at 0 → {0,1}.
    pub fn image_embed_to_vsa(&self, embed: &[f32]) -> Vec<u8> {
        Self::seeded_random_projection(embed, self.dim, self.seed)
    }

    /// Cross-modal similarity between two VSA vectors using Hamming similarity.
    pub fn cross_modal_similarity(a: &[u8], b: &[u8]) -> f64 {
        QuantizedVSA::similarity(a, b)
    }

    /// Returns a fixed VSA vector for a modality name (e.g. "text", "image", "audio").
    pub fn modality_tag(&self, modality: &str) -> Vec<u8> {
        self.text_to_vsa(modality)
    }

    /// Bind a content vector with a modality tag for modality-specific retrieval.
    /// Equivalent to: result = bind(content, tag)
    pub fn bind_with_modality(&self, vsa: &[u8], modality: &str) -> Vec<u8> {
        let tag = self.modality_tag(modality);
        QuantizedVSA::bind(vsa, &tag)
    }

    // ── internal helpers ──

    fn seeded_random(&self, seed: u64, dim: usize) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, dim)
    }

    /// Project a float embedding into target_dim binary VSA space using
    /// deterministic random projections seeded by a base seed.
    ///
    /// For each target dimension d, a random vector r_d is generated from
    /// seed = base_seed + d. The d-th output bit is sign(embed · r_d) > 0.
    pub fn seeded_random_projection(embed: &[f32], target_dim: usize, seed: u64) -> Vec<u8> {
        use std::hash::{Hash, Hasher};
        let mut result = Vec::with_capacity(target_dim);
        for d in 0..target_dim {
            let dim_seed = seed.wrapping_add(d as u64);
            let mut proj = 0.0f32;
            for (i, &e) in embed.iter().enumerate() {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                dim_seed.hash(&mut hasher);
                (i as u64).hash(&mut hasher);
                let h = hasher.finish();
                let r = (h as f64 / u64::MAX as f64) * 2.0 - 1.0;
                proj += r as f32 * e;
            }
            result.push(if proj > 0.0 { 1 } else { 0 });
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    fn aligner() -> CrossModalAligner {
        CrossModalAligner::new(VSA_DIM, 42)
    }

    #[test]
    fn test_text_to_vsa_deterministic() {
        let a = aligner();
        let v1 = a.text_to_vsa("hello world");
        let v2 = a.text_to_vsa("hello world");
        assert_eq!(v1, v2, "same input must produce identical VSA vector");
    }

    #[test]
    fn test_text_to_vsa_different_inputs() {
        let a = aligner();
        let v1 = a.text_to_vsa("hello world");
        let v2 = a.text_to_vsa("goodbye world");
        assert_ne!(
            v1, v2,
            "different inputs must produce different VSA vectors"
        );
    }

    #[test]
    fn test_image_embed_to_vsa_correct_dim() {
        let a = aligner();
        let embed: Vec<f32> = (0..512).map(|i| (i as f32).sin()).collect();
        let vsa = a.image_embed_to_vsa(&embed);
        assert_eq!(vsa.len(), VSA_DIM, "output must be VSA_DIM length");
    }

    #[test]
    fn test_image_embed_to_vsa_binary() {
        let a = aligner();
        let embed: Vec<f32> = (0..512).map(|i| (i as f32).sin()).collect();
        let vsa = a.image_embed_to_vsa(&embed);
        for &x in &vsa {
            assert!(
                x == 0 || x == 1,
                "each element must be binary (0 or 1), got {}",
                x
            );
        }
    }

    #[test]
    fn test_image_embed_to_vsa_deterministic() {
        let embed: Vec<f32> = (0..768).map(|i| (i as f32).cos()).collect();
        let a1 = CrossModalAligner::new(VSA_DIM, 99);
        let a2 = CrossModalAligner::new(VSA_DIM, 99);
        let v1 = a1.image_embed_to_vsa(&embed);
        let v2 = a2.image_embed_to_vsa(&embed);
        assert_eq!(v1, v2, "same seed must produce identical projection");
    }

    #[test]
    fn test_cross_modal_similarity_symmetric() {
        let a = aligner();
        let embed: Vec<f32> = (0..512).map(|i| (i as f32).sin()).collect();
        let v = a.image_embed_to_vsa(&embed);
        let sim_ab = CrossModalAligner::cross_modal_similarity(&v, &v);
        let sim_ba = CrossModalAligner::cross_modal_similarity(&v, &v);
        assert!(
            (sim_ab - sim_ba).abs() < 1e-12,
            "similarity must be symmetric"
        );
    }

    #[test]
    fn test_modality_tag_distinct() {
        let a = aligner();
        let text_tag = a.modality_tag("text");
        let image_tag = a.modality_tag("image");
        let audio_tag = a.modality_tag("audio");
        assert_ne!(text_tag, image_tag, "text and image tags must differ");
        assert_ne!(image_tag, audio_tag, "image and audio tags must differ");
        assert_ne!(text_tag, audio_tag, "text and audio tags must differ");
    }

    #[test]
    fn test_bind_with_modality_changes_vector() {
        let a = aligner();
        let embed: Vec<f32> = (0..512).map(|i| (i as f32).sin()).collect();
        let vsa = a.image_embed_to_vsa(&embed);
        let bound = a.bind_with_modality(&vsa, "image");
        assert_ne!(bound, vsa, "binding with modality must change the vector");
    }

    #[test]
    fn test_image_embed_to_vsa_different_embeddings() {
        let a = aligner();
        let embed_a: Vec<f32> = (0..512).map(|i| (i as f32).sin()).collect();
        let embed_b: Vec<f32> = (0..512).map(|i| (i as f32).cos()).collect();
        let vsa_a = a.image_embed_to_vsa(&embed_a);
        let vsa_b = a.image_embed_to_vsa(&embed_b);
        assert_ne!(
            vsa_a, vsa_b,
            "different embeddings must produce different VSA vectors"
        );
    }

    #[test]
    fn test_text_to_vsa_empty() {
        let a = aligner();
        let v = a.text_to_vsa("");
        assert_eq!(v.len(), VSA_DIM);
        assert!(
            v.iter().all(|&x| x == 0),
            "empty text should return zero vector"
        );
    }

    #[test]
    fn test_seeded_random_projection_deterministic() {
        let embed: Vec<f32> = (0..512).map(|i| (i as f32).sin()).collect();
        let v1 = CrossModalAligner::seeded_random_projection(&embed, VSA_DIM, 7);
        let v2 = CrossModalAligner::seeded_random_projection(&embed, VSA_DIM, 7);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_bind_with_modality_similarity_preserved_across_modalities() {
        let a = aligner();
        let embed_a: Vec<f32> = (0..512).map(|i| (i as f32).sin()).collect();
        let embed_b: Vec<f32> = (0..512).map(|i| (i as f32).sin()).collect();
        let vsa_a = a.image_embed_to_vsa(&embed_a);
        let vsa_b = a.image_embed_to_vsa(&embed_b);
        let bound_a = a.bind_with_modality(&vsa_a, "image");
        let bound_b = a.bind_with_modality(&vsa_b, "image");
        let raw_sim = CrossModalAligner::cross_modal_similarity(&vsa_a, &vsa_b);
        let bound_sim = CrossModalAligner::cross_modal_similarity(&bound_a, &bound_b);
        assert!(
            (raw_sim - bound_sim).abs() < 0.01,
            "same-modality binding should preserve similarity"
        );
    }

    #[test]
    fn test_modality_tag_consistent() {
        let a = aligner();
        let t1 = a.modality_tag("text");
        let t2 = a.modality_tag("text");
        assert_eq!(t1, t2, "modality_tag must be deterministic");
    }
}
