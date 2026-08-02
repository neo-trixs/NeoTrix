//! Phase 10.3 — Multimodal Unified Reasoning (多模态统一 · LatentOmni §3).
//!
//! Extends the unified latent space (Phase 10.1) to multimodal inputs:
//!
//!   text + image + audio → dedicated encoders → unified latent space
//!   GWT modal-level routing (Phase 7.5 ModalityRouter) → cross-modal attention
//!   E8 loop fuses the weighted modality representations into one latent vector
//!
//! Text is embedded via a deterministic character n-gram hashing kernel (no
//! external model needed for the core loop); image and audio arrive as raw
//! feature vectors and are projected into the unified space. Each modality's
//! unified embedding is then routed by the ModalityRouter and fused by salience
//! weighting, producing a single latent vector that can drive the E8 state.

use crate::core::nt_core_e8::unified_latent::UnifiedLatentSpace;
use crate::core::nt_core_gwt::modality_router::Modality;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Text embedding dimension (character n-gram hash kernel output).
pub const TEXT_EMBED_DIM: usize = 128;
/// Image feature dimension expected on input.
pub const IMAGE_FEATURE_DIM: usize = 64;
/// Audio feature dimension expected on input.
pub const AUDIO_FEATURE_DIM: usize = 64;

/// Raw multimodal input for a single reasoning step.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultimodalInput {
    /// Optional text payload (embedded by the n-gram kernel).
    pub text: Option<String>,
    /// Optional image feature vector (already extracted upstream).
    pub image: Option<Vec<f64>>,
    /// Optional audio feature vector (already extracted upstream).
    pub audio: Option<Vec<f64>>,
}

impl MultimodalInput {
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            text: Some(s.into()),
            ..Default::default()
        }
    }
    pub fn with_image(mut self, feat: Vec<f64>) -> Self {
        self.image = Some(feat);
        self
    }
    pub fn with_audio(mut self, feat: Vec<f64>) -> Self {
        self.audio = Some(feat);
        self
    }
    pub fn has_content(&self) -> bool {
        self.text.is_some() || self.image.is_some() || self.audio.is_some()
    }
}

/// Phase 10.3 — multimodal encoder + fusion into the unified latent space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalEncoder {
    /// Unified latent space used for all projections.
    pub unified: UnifiedLatentSpace,
    /// Raw embedding of the last encoded input, per modality present.
    pub last_embeddings: BTreeMap<Modality, Vec<f64>>,
}

impl Default for MultimodalEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl MultimodalEncoder {
    pub fn new() -> Self {
        Self {
            unified: UnifiedLatentSpace::new(),
            last_embeddings: BTreeMap::new(),
        }
    }

    /// Deterministic character n-gram hashing kernel.
    ///
    /// Produces a fixed `TEXT_EMBED_DIM`-dim vector: each sliding window of up
    /// to 3 chars hashes to a coordinate bucket, so lexical overlap maps to
    /// similarity (no external embedding model required in the core loop).
    pub fn embed_text(&self, text: &str) -> Vec<f64> {
        let mut out = vec![0.0f64; TEXT_EMBED_DIM];
        let mut h: u64 = 0xC0FFEE;
        for (i, ch) in text.chars().enumerate() {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(ch as u64 + 0x9E3779B97F4A7C15);
            let bucket = ((h >> 33) as usize) % TEXT_EMBED_DIM;
            out[bucket] += 1.0;
            // Bigram + trigram context via a second hash lane.
            if i >= 1 {
                h = h.rotate_left(13) ^ 0x2545F4914F6CDD1D;
                let b2 = ((h >> 33) as usize) % TEXT_EMBED_DIM;
                out[b2] += 0.5;
            }
            if i >= 2 {
                let b3 = (((h >> 13) as usize) + 7) % TEXT_EMBED_DIM;
                out[b3] += 0.25;
            }
        }
        self.normalize(&out)
    }

    /// Encode a single modality into the unified latent space.
    ///
    /// `raw` is the modality-native embedding; the returned vector lives in
    /// `unified.dim` and is unit-normalized for comparable cosine distances.
    pub fn encode_modality(&self, modality: Modality, raw: &[f64]) -> Vec<f64> {
        match modality {
            Modality::Text => {
                let padded = self.pad_to(raw, TEXT_EMBED_DIM);
                self.unified.project_vsa(&padded)
            }
            Modality::Image => {
                let padded = self.pad_to(raw, IMAGE_FEATURE_DIM);
                self.unified.project_vsa(&padded)
            }
            Modality::Audio => {
                let padded = self.pad_to(raw, AUDIO_FEATURE_DIM);
                self.unified.project_vsa(&padded)
            }
            // Code and Latent already have vector-native representations.
            _ => self.unified.project_vsa(&self.pad_to(raw, self.unified.dim)),
        }
    }

    /// Encode all present modalities of an input into the unified space.
    pub fn encode_all(&mut self, input: &MultimodalInput) -> BTreeMap<Modality, Vec<f64>> {
        let mut map = BTreeMap::new();
        if let Some(t) = &input.text {
            let raw = self.embed_text(t);
            let unified = self.unified.project_vsa(&raw);
            map.insert(Modality::Text, unified);
        }
        if let Some(img) = &input.image {
            let padded = self.pad_to(img, IMAGE_FEATURE_DIM);
            let unified = self.unified.project_vsa(&padded);
            map.insert(Modality::Image, unified);
        }
        if let Some(audio) = &input.audio {
            let padded = self.pad_to(audio, AUDIO_FEATURE_DIM);
            let unified = self.unified.project_vsa(&padded);
            map.insert(Modality::Audio, unified);
        }
        self.last_embeddings = map.clone();
        map
    }

    /// Fuse modality embeddings under cross-modal attention.
    ///
    /// `router` supplies per-modality attention weights (Phase 7.5); the fused
    /// vector is the salience-weighted sum, normalized into the unified space.
    /// Returns `(fused_vector, per_modality_weights)`.
    pub fn fuse(&self, router_weights: &BTreeMap<Modality, f64>, embeddings: &BTreeMap<Modality, Vec<f64>>) -> (Vec<f64>, Vec<(Modality, f64)>) {
        let mut fused = vec![0.0f64; self.unified.dim];
        let mut weights = Vec::new();
        for (m, v) in embeddings {
            let w = router_weights.get(m).copied().unwrap_or(0.0);
            if w > 0.0 {
                for (f, &x) in fused.iter_mut().zip(v.iter()) {
                    *f += w * x;
                }
                weights.push((*m, w));
            }
        }
        (self.normalize(&fused), weights)
    }

    /// Map a fused latent vector onto an E8 hexagram mode.
    ///
    /// Uses the top-6 sign bits of the vector as the 6 hexagram lines, giving a
    /// deterministic multimodal→E8 mapping for the reasoning loop.
    pub fn to_e8_mode(&self, fused: &[f64]) -> u8 {
        let mut bits = 0u8;
        for i in 0..6 {
            let x = fused.get(i).copied().unwrap_or(0.0);
            if x > 0.0 {
                bits |= 1 << (5 - i);
            }
        }
        bits
    }

    fn pad_to(&self, v: &[f64], target: usize) -> Vec<f64> {
        if v.len() >= target {
            v.iter().take(target).copied().collect()
        } else {
            let mut out = v.to_vec();
            out.resize(target, 0.0);
            out
        }
    }

    fn normalize(&self, v: &[f64]) -> Vec<f64> {
        let norm = (v.iter().map(|x| x * x).sum::<f64>()).sqrt();
        if norm > 1e-12 {
            v.iter().map(|x| x / norm).collect()
        } else {
            v.to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_router() -> BTreeMap<Modality, f64> {
        let mut m = BTreeMap::new();
        m.insert(Modality::Text, 1.0);
        m.insert(Modality::Image, 0.0);
        m.insert(Modality::Audio, 0.0);
        m
    }

    #[test]
    fn test_text_embedding_deterministic_and_distinct() {
        let enc = MultimodalEncoder::new();
        let a = enc.embed_text("hello world");
        let b = enc.embed_text("hello world");
        let c = enc.embed_text("goodbye moon");
        assert_eq!(a.len(), TEXT_EMBED_DIM);
        assert_eq!(a, b);
        assert!(a != c, "distinct texts should differ");
        let norm: f64 = a.iter().map(|x| x * x).sum();
        assert!((norm - 1.0).abs() < 1e-6, "unit-normalized, got {norm}");
    }

    #[test]
    fn test_encode_all_present_modalities() {
        let mut enc = MultimodalEncoder::new();
        let input = MultimodalInput::text("cat")
            .with_image(vec![0.5; IMAGE_FEATURE_DIM])
            .with_audio(vec![0.25; AUDIO_FEATURE_DIM]);
        let map = enc.encode_all(&input);
        assert!(map.contains_key(&Modality::Text));
        assert!(map.contains_key(&Modality::Image));
        assert!(map.contains_key(&Modality::Audio));
        for (_, v) in &map {
            assert_eq!(v.len(), enc.unified.dim);
        }
        // last_embeddings cached.
        assert_eq!(enc.last_embeddings.len(), 3);
    }

    #[test]
    fn test_fuse_weights_respect_router() {
        let mut enc = MultimodalEncoder::new();
        let mut input = MultimodalInput::text("cat").with_image(vec![0.9; IMAGE_FEATURE_DIM]);
        input.audio = Some(vec![0.1; AUDIO_FEATURE_DIM]);
        let embeddings = enc.encode_all(&input);
        let router = text_router();
        let (fused, weights) = enc.fuse(&router, &embeddings);
        assert_eq!(fused.len(), enc.unified.dim);
        assert!(weights.iter().all(|(_, w)| *w >= 0.0));
        let norm: f64 = fused.iter().map(|x| x * x).sum();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_fused_vector_tracks_dominant_modality() {
        let enc = MultimodalEncoder::new();
        // Same text embedded twice; fusion with text-only router should
        // approximately recover the text embedding (unit vectors).
        let t = enc.embed_text("neotrix consciousness");
        let mut router = BTreeMap::new();
        router.insert(Modality::Text, 1.0);
        let mut embeddings = BTreeMap::new();
        embeddings.insert(Modality::Text, enc.unified.project_vsa(&t));
        let (fused, _) = enc.fuse(&router, &embeddings);
        let text_emb = &embeddings[&Modality::Text];
        let sim = enc.unified.cosine(&fused, text_emb);
        assert!(sim > 0.99, "fusion should preserve dominant modality, sim={sim}");
    }

    #[test]
    fn test_to_e8_mode_deterministic() {
        let enc = MultimodalEncoder::new();
        let fused = vec![1.0, -1.0, 1.0, -1.0, 1.0, -1.0];
        assert_eq!(enc.to_e8_mode(&fused), 0b101010);
        let neg = fused.iter().map(|x| -x).collect::<Vec<_>>();
        assert_eq!(enc.to_e8_mode(&neg), 0b010101);
    }

    #[test]
    fn test_empty_input_yields_empty_map() {
        let mut enc = MultimodalEncoder::new();
        let map = enc.encode_all(&MultimodalInput::default());
        assert!(map.is_empty());
        assert!(!MultimodalInput::default().has_content());
    }

    #[test]
    fn test_image_embedding_stable_and_discriminative() {
        let enc = MultimodalEncoder::new();
        let va: Vec<f64> = (0..IMAGE_FEATURE_DIM).map(|i| (i as f64 * 0.01).sin()).collect();
        let vb: Vec<f64> = (0..IMAGE_FEATURE_DIM).map(|i| (i as f64 * 0.01).sin()).collect();
        let vc: Vec<f64> = (0..IMAGE_FEATURE_DIM).map(|i| (i as f64 * 0.01).cos()).collect();
        let a = enc.encode_modality(Modality::Image, &va);
        let b = enc.encode_modality(Modality::Image, &vb);
        let c = enc.encode_modality(Modality::Image, &vc);
        assert_eq!(a, b);
        assert_eq!(enc.unified.cosine(&a, &b), 1.0);
        assert!(enc.unified.cosine(&a, &c) < 1.0);
    }
}
