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
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Mutex;

/// Vision capability detection for model routing.
///
/// Text-only reasoning models (e.g. deepseek chat, most local Ollama qwen/llama
/// variants without mm suffix) cannot consume `image_data`; vision-native models
/// (gpt-4o, claude-3.5-sonnet, gemini-1.5/2.x) pass images through the provider.
/// This mirrors the VisionBridge design: the *gateway* decides — if the active
/// model is text-only, image bytes are converted to structured evidence text
/// upstream; otherwise they are forwarded natively.
pub fn model_supports_vision(model: &str) -> bool {
    let m = model.to_ascii_lowercase();
    if m.is_empty() {
        return false;
    }
    if m.contains("-mm") || m.ends_with("-mm") {
        return true;
    }
    if m.contains("vision") {
        return true;
    }
    for vision_family in [
        "gpt-4o",
        "gpt-4.1",
        "gpt-5",
        "claude-3",
        "claude-4",
        "gemini-1.5",
        "gemini-2",
        "gemini-3",
        "qwen-vl",
        "qwen2-vl",
        "qwen2.5-vl",
        "llava",
        "llama3.2-vision",
        "molmo",
        "pixtral",
        "glm-4v",
        "minicpm-v",
    ] {
        if m.contains(vision_family) {
            return true;
        }
    }
    false
}

/// Deterministic image-type classification (classification-first pattern,
/// research §1/§4). Computed from pixel statistics, never a model guess —
/// the caller routes to OCR vs semantic processing based on this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ImageClass {
    /// Clean white/light background, few saturated colors, text-heavy (docs, scans).
    Document,
    /// UI screenshot: rects, saturated accents, dark/light chrome, near-flat regions.
    Screenshot,
    /// Natural scene/photo: high color variance, low text density, smooth gradients.
    Photo,
    /// Blank / near-solid (single color, negligible variance).
    Blank,
    /// Low-signal: small, underexposed, or indeterminate. Route with caution.
    Unknown,
}

impl ImageClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Screenshot => "screenshot",
            Self::Photo => "photo",
            Self::Blank => "blank",
            Self::Unknown => "unknown",
        }
    }
}

/// Structured image evidence — the deterministic, fabricate-free text contract
/// a text-only model can reason over (VisionBridge / modlens pattern).
///
/// Every field is computed from the actual decoded pixels; `uncertainty` is
/// first-class (higher = the bridge had less signal, model should be cautious).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEvidence {
    /// Decoded dimensions (width, height).
    pub width: u32,
    pub height: u32,
    /// Aspect ratio (w/h), NaN-safe as f64.
    pub aspect_ratio: f64,
    /// Fraction of pixels within the active luminance range (0..1).
    pub contrast: f64,
    /// Mean luminance (0..1).
    pub mean_luminance: f64,
    /// Dominant color buckets (RGB triples) by pixel share, most frequent first.
    pub dominant_colors: Vec<[u8; 3]>,
    /// Per-tile luminance centroid row vector; feeds the image feature embedder.
    pub tiles: Vec<f64>,
    /// 0..1 — 1.0 when decode produced full signal, decaying with downscale loss.
    pub confidence: f64,
    /// Free-form: source byte length, codec guessed from magic bytes.
    pub source: String,
    /// Deterministic image-type classification (routes OCR vs semantic).
    pub classification: ImageClass,
    /// Perceptual hash (64-bit) for cross-turn dedup — identical/similar images
    /// share the hash so cached evidence can be reused.
    pub phash: u64,
    /// SHA-256 of the raw bytes (content address). Full dedup key.
    pub sha256: String,
    /// Text-ish pixel density: fraction of tiles with high luminance variance
    /// (doc/screenshot text regions). 0..1.
    pub text_density: f64,
    /// Edge density (mean absolute luminance delta between adjacent tiles).
    pub edge_density: f64,
}

impl ImageEvidence {
    /// Deterministic text serialization for prompt injection (no JSON escaping
    /// surprises in the LLM loop).
    pub fn to_evidence_text(&self) -> String {
        let mut out = String::from("<image_evidence>\n");
        out.push_str(&format!("  source: {}\n", self.source));
        out.push_str(&format!(
            "  classification: {}\n",
            self.classification.as_str()
        ));
        out.push_str(&format!("  dimensions: {}x{}\n", self.width, self.height));
        out.push_str(&format!("  aspect_ratio: {:.3}\n", self.aspect_ratio));
        out.push_str(&format!("  mean_luminance: {:.3}\n", self.mean_luminance));
        out.push_str(&format!("  contrast: {:.3}\n", self.contrast));
        out.push_str(&format!("  text_density: {:.3}\n", self.text_density));
        out.push_str(&format!("  edge_density: {:.3}\n", self.edge_density));
        out.push_str(&format!("  phash: {:016x}\n", self.phash));
        out.push_str(&format!(
            "  content_tag: {}\n",
            &self.sha256[..self.sha256.len().min(12)]
        ));
        out.push_str(&format!("  confidence: {:.3}\n", self.confidence));
        if !self.dominant_colors.is_empty() {
            let colors = self
                .dominant_colors
                .iter()
                .map(|c| format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("  dominant_colors: {}\n", colors));
        }
        out.push_str("</image_evidence>");
        out
    }
}

/// VisionBridge — deterministic image feature extraction for the multimodal
/// loop, filling the "image features already extracted upstream" contract that
/// `MultimodalEncoder` was written against.
///
/// Pure local decode via the `image` crate (png/jpeg). No external vision model
/// is required in the core loop; structured evidence + feature vector are
/// produced from the raw pixels. `confidence` degrades with downscale.
#[derive(Debug, Clone, Default)]
pub struct VisionBridge;

/// Number of tiles per side for the coarse luminance grid. 8×8 = 64 cells,
/// exactly matching `IMAGE_FEATURE_DIM`.
const TILE_SIDE: usize = 8;

impl VisionBridge {
    /// Decode image bytes and produce structured evidence + a fixed-dim image
    /// feature vector (the missing upstream extractor for `MultimodalInput::image`).
    pub fn analyze(bytes: &[u8]) -> Result<(ImageEvidence, Vec<f64>), String> {
        let img =
            image::load_from_memory(bytes).map_err(|e| format!("image decode failed: {}", e))?;
        let rgb = img.to_rgb8();
        let (w, h) = (rgb.width(), rgb.height());
        if w == 0 || h == 0 {
            return Err("image has zero dimensions".into());
        }

        let aspect = w as f64 / h as f64;

        // Downscale to 16x16 luminance grid (cheap bilinear-ish block sampling).
        let gw = 16usize;
        let gh = 16usize;
        let mut lum_grid = vec![0.0f64; gw * gh];
        let mut color_hist: BTreeMap<[u8; 3], u32> = BTreeMap::new();
        let mut lum_sum = 0.0f64;
        let mut lum_sq = 0.0f64;
        let mut n: f64 = 0.0;

        for gy in 0..gh {
            for gx in 0..gw {
                let x0 = (gx as u32 * w) / gw as u32;
                let x1 = (((gx + 1) as u32 * w) / gw as u32).max(x0 + 1).min(w);
                let y0 = (gy as u32 * h) / gh as u32;
                let y1 = (((gy + 1) as u32 * h) / gh as u32).max(y0 + 1).min(h);
                let mut cell = 0.0f64;
                let mut cell_n = 0.0f64;
                let mut px = x0;
                while px < x1 {
                    let mut py = y0;
                    while py < y1 {
                        let p = rgb.get_pixel(px, py);
                        let lum = 0.2126 * p[0] as f64 / 255.0
                            + 0.7152 * p[1] as f64 / 255.0
                            + 0.0722 * p[2] as f64 / 255.0;
                        cell += lum;
                        cell_n += 1.0;
                        // Quantize color to 32-bucket histogram for dominance.
                        let key = [(p[0] >> 3) as u8, (p[1] >> 3) as u8, (p[2] >> 3) as u8];
                        *color_hist.entry(key).or_insert(0) += 1;
                        py += 1;
                    }
                    px += 1;
                }
                let mean_cell = if cell_n > 0.0 { cell / cell_n } else { 0.0 };
                lum_grid[gy * gw + gx] = mean_cell;
                lum_sum += cell;
                lum_sq += cell * cell / cell_n;
                n += cell_n;
            }
        }

        let mean_lum = if n > 0.0 { lum_sum / n } else { 0.0 };
        let var = if n > 0.0 {
            (lum_sq / n) - mean_lum * mean_lum
        } else {
            0.0
        };
        let contrast = var.clamp(0.0, 1.0).sqrt();

        // Dominant colors: take top-4 by frequency.
        let mut hist: Vec<([u8; 3], u32)> = color_hist.into_iter().collect();
        hist.sort_by(|a, b| b.1.cmp(&a.1));
        let dominant_colors: Vec<[u8; 3]> = hist
            .iter()
            .take(4)
            .map(|(k, _)| [k[0] << 3 | 0b111, k[1] << 3 | 0b111, k[2] << 3 | 0b111])
            .collect();

        // Feature vector: 64-dim from the 16x16 grid (take every other cell) —
        // deterministic, unit-normalized, luminance-centric.
        let mut feat = Vec::with_capacity(IMAGE_FEATURE_DIM);
        for gy in 0..TILE_SIDE {
            for gx in 0..TILE_SIDE {
                let v = lum_grid[gy * 2 * gw + gx * 2];
                feat.push(v);
            }
        }
        let norm: f64 = feat.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-12 {
            for x in feat.iter_mut() {
                *x /= norm;
            }
        }

        let confidence = if w >= 32 && h >= 32 {
            1.0
        } else if w >= 8 && h >= 8 {
            0.6
        } else {
            0.3
        };

        // Text-ish density: fraction of tiles whose local variance is high —
        // text regions flicker between ink and paper across adjacent pixels.
        let mut text_tiles = 0usize;
        for gy in 1..gh - 1 {
            for gx in 1..gw - 1 {
                let c = lum_grid[gy * gw + gx];
                let nbr = lum_grid[(gy - 1) * gw + gx]
                    + lum_grid[(gy + 1) * gw + gx]
                    + lum_grid[gy * gw + (gx - 1)]
                    + lum_grid[gy * gw + (gx + 1)];
                let local_var = (c - nbr / 4.0).abs();
                if local_var > 0.06 {
                    text_tiles += 1;
                }
            }
        }
        let text_density = text_tiles as f64 / ((gw - 2) * (gh - 2)) as f64;

        // Edge density: mean absolute delta between horizontally adjacent tiles.
        let mut edge_sum = 0.0f64;
        let mut edge_n = 0.0f64;
        for gy in 0..gh {
            for gx in 0..gw - 1 {
                edge_sum += (lum_grid[gy * gw + gx] - lum_grid[gy * gw + gx + 1]).abs();
                edge_n += 1.0;
            }
        }
        let edge_density = if edge_n > 0.0 { edge_sum / edge_n } else { 0.0 };

        // Deterministic classification (classification-first routing).
        let classification = classify_image(
            w,
            h,
            mean_lum,
            contrast,
            text_density,
            edge_density,
            &dominant_colors,
        );

        // Perceptual hash (dHash variant): compare adjacent luminance on an
        // 8×8 tile sample, pack exactly 64 bits (56 horizontal + 8 vertical
        // deltas). Robust to re-encode/rescale; near-dup images share the hash.
        let mut phash: u64 = 0;
        let mut bit = 0u64;
        let sample = |gy: usize, gx: usize| lum_grid[gy * 2 * gw + gx * 2];
        for gy in 0..8 {
            for gx in 0..7 {
                if sample(gy, gx) >= sample(gy, gx + 1) {
                    phash |= 1u64 << bit;
                }
                bit += 1;
            }
        }
        for gy in 0..7 {
            if sample(gy, 0) >= sample(gy + 1, 0) {
                phash |= 1u64 << bit;
            }
            bit += 1;
        }

        // Content address: full SHA-256 of raw bytes for exact dedup.
        let sha256 = hex::encode(Sha256::digest(bytes));

        // Codec guess from magic bytes for the human-readable source line.
        let codec = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
            "png"
        } else if bytes.len() > 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
            "jpeg"
        } else {
            "raw"
        };

        let evidence = ImageEvidence {
            width: w,
            height: h,
            aspect_ratio: aspect,
            contrast,
            mean_luminance: mean_lum,
            dominant_colors,
            tiles: lum_grid,
            confidence,
            source: format!("{} ({} bytes)", codec, bytes.len()),
            classification,
            phash,
            sha256,
            text_density,
            edge_density,
        };
        Ok((evidence, feat))
    }

    /// Convenience: extract just the feature vector (feeds `MultimodalInput::with_image`).
    pub fn image_features(bytes: &[u8]) -> Result<Vec<f64>, String> {
        Self::analyze(bytes).map(|(_, f)| f)
    }

    /// Content-addressed evidence cache (cross-turn dedup).
    ///
    /// `analyze` is O(decoded pixels); identical image bytes across turns (e.g.
    /// the same screenshot re-attached) re-decode every time. The bridge keeps a
    /// small bounded SHA-256 → (evidence, features) cache so repeat payloads are
    /// served from memory. Bounded to `CACHE_CAPACITY` entries, thread-safe.
    pub fn analyze_cached(bytes: &[u8]) -> Result<(ImageEvidence, Vec<f64>), String> {
        let digest = hex::encode(Sha256::digest(bytes));
        // 毒化锁 → 视为缓存不可用，直接重算（cache 只是性能优化，非正确性依赖）。
        if let Some(hit) = EVIDENCE_CACHE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&digest)
        {
            return Ok(hit.clone());
        }
        let result = Self::analyze(bytes)?;
        let mut guard = EVIDENCE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if guard.len() >= CACHE_CAPACITY {
            guard.clear();
        }
        guard.insert(digest, result.clone());
        Ok(result)
    }

    /// Hamming distance between two perceptual hashes (0..64). ≤10 typically
    /// means "effectively the same image" (re-encode/crop/resize variants).
    pub fn phash_distance(a: u64, b: u64) -> u32 {
        (a ^ b).count_ones()
    }
}

/// Bounded cross-turn evidence cache: SHA-256 → (ImageEvidence, features).
static EVIDENCE_CACHE: Mutex<BTreeMap<String, (ImageEvidence, Vec<f64>)>> =
    Mutex::new(BTreeMap::new());
const CACHE_CAPACITY: usize = 128;

/// Deterministic classification heuristics (research: classification-first).
///
/// Pure statistics, no model:
/// - Blank: near-zero variance (single color / tiny image).
/// - Document: light dominant background + high text density + low color variety.
/// - Screenshot: low-mid text density + saturated accent palette + rect-ish layout.
/// - Photo: high color variance, low text density, warm mean luminance spread.
/// - Otherwise: Unknown (route conservatively).
fn classify_image(
    w: u32,
    h: u32,
    mean_lum: f64,
    contrast: f64,
    text_density: f64,
    edge_density: f64,
    dominant_colors: &[[u8; 3]],
) -> ImageClass {
    if w < 8 || h < 8 {
        return ImageClass::Unknown;
    }
    // Blank: near-zero variance OR an essentially solid near-black/white frame
    // (the classic logo/black-screen intro frame that wastes video analysis).
    if contrast < 0.03 || !(0.01..=0.99).contains(&mean_lum) {
        return ImageClass::Blank;
    }
    // Saturated dominant palette = UI chrome (screenshots) more than documents.
    let saturated = dominant_colors
        .iter()
        .filter(|c| {
            let mx = c[0].max(c[1]).max(c[2]) as i32;
            let mn = c[0].min(c[1]).min(c[2]) as i32;
            mx - mn > 80
        })
        .count();
    let light_dominant = dominant_colors
        .first()
        .map(|c| c[0] > 200 && c[1] > 200 && c[2] > 200)
        .unwrap_or(false);

    if text_density > 0.35 && light_dominant {
        ImageClass::Document
    } else if text_density > 0.25 || (saturated >= 2 && text_density > 0.12) {
        ImageClass::Screenshot
    } else if saturated <= 1 && edge_density < 0.12 && contrast > 0.1 {
        ImageClass::Photo
    } else {
        ImageClass::Unknown
    }
}

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
            h = h
                .wrapping_mul(6364136223846793005)
                .wrapping_add(ch as u64 + 0x9E3779B97F4A7C15);
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
            _ => self
                .unified
                .project_vsa(&self.pad_to(raw, self.unified.dim)),
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
    pub fn fuse(
        &self,
        router_weights: &BTreeMap<Modality, f64>,
        embeddings: &BTreeMap<Modality, Vec<f64>>,
    ) -> (Vec<f64>, Vec<(Modality, f64)>) {
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
        assert!(
            sim > 0.99,
            "fusion should preserve dominant modality, sim={sim}"
        );
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
        let va: Vec<f64> = (0..IMAGE_FEATURE_DIM)
            .map(|i| (i as f64 * 0.01).sin())
            .collect();
        let vb: Vec<f64> = (0..IMAGE_FEATURE_DIM)
            .map(|i| (i as f64 * 0.01).sin())
            .collect();
        let vc: Vec<f64> = (0..IMAGE_FEATURE_DIM)
            .map(|i| (i as f64 * 0.01).cos())
            .collect();
        let a = enc.encode_modality(Modality::Image, &va);
        let b = enc.encode_modality(Modality::Image, &vb);
        let c = enc.encode_modality(Modality::Image, &vc);
        assert_eq!(a, b);
        assert_eq!(enc.unified.cosine(&a, &b), 1.0);
        assert!(enc.unified.cosine(&a, &c) < 1.0);
    }

    // ─── VisionBridge ────────────────────────────────────────────────────

    fn test_png(w: u32, h: u32, fill: [u8; 3]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        let img = image::RgbImage::from_pixel(w, h, image::Rgb(fill));
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, image::ImageFormat::Png)
            .expect("encode test png");
        buf.into_inner()
    }

    #[test]
    fn test_vision_bridge_analyze_black_square() {
        let png = test_png(64, 64, [0, 0, 0]);
        let (ev, feat) = VisionBridge::analyze(&png).expect("decode");
        assert_eq!((ev.width, ev.height), (64, 64));
        assert!((ev.aspect_ratio - 1.0).abs() < 1e-9);
        assert!(
            ev.mean_luminance < 0.05,
            "black image mean {}",
            ev.mean_luminance
        );
        assert_eq!(feat.len(), IMAGE_FEATURE_DIM);
        assert!(ev.confidence > 0.9);
        assert!(ev.source.starts_with("png"));
        // Deterministic feature extraction.
        let (_, feat2) = VisionBridge::analyze(&png).expect("decode");
        assert_eq!(feat, feat2);
    }

    #[test]
    fn test_vision_bridge_white_brighter_than_black() {
        let black = VisionBridge::analyze(&test_png(64, 64, [0, 0, 0])).expect("black");
        let white = VisionBridge::analyze(&test_png(64, 64, [255, 255, 255])).expect("white");
        assert!(white.0.mean_luminance > black.0.mean_luminance);
        // Distinct color dominance.
        assert!(white.0.dominant_colors[0] != black.0.dominant_colors[0]);
        // Feature vectors differ.
        assert!(black.1 != white.1);
    }

    #[test]
    fn test_vision_bridge_aspect_ratio() {
        let wide = VisionBridge::analyze(&test_png(128, 32, [10, 20, 30])).expect("wide");
        assert!((wide.0.aspect_ratio - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_vision_bridge_evidence_text_embeds_fields() {
        let png = test_png(32, 32, [200, 100, 50]);
        let (ev, _) = VisionBridge::analyze(&png).expect("decode");
        let txt = ev.to_evidence_text();
        assert!(txt.contains("dimensions: 32x32"));
        assert!(txt.contains("confidence:"));
        assert!(txt.contains("dominant_colors:"));
    }

    #[test]
    fn test_vision_bridge_rejects_garbage_bytes() {
        assert!(VisionBridge::analyze(b"not an image at all").is_err());
    }

    #[test]
    fn test_vision_features_feed_multimodal_encoder() {
        let mut enc = MultimodalEncoder::new();
        let feat = VisionBridge::image_features(&test_png(64, 64, [0, 0, 0])).expect("feat");
        let input = MultimodalInput::text("analyze image").with_image(feat);
        let map = enc.encode_all(&input);
        assert!(map.contains_key(&Modality::Image));
        assert!(map.contains_key(&Modality::Text));
    }

    #[test]
    fn test_model_supports_vision_detection() {
        assert!(model_supports_vision("gpt-4o"));
        assert!(model_supports_vision("claude-3.5-sonnet-20241022"));
        assert!(model_supports_vision("gemini-2.0-flash"));
        assert!(model_supports_vision("qwen2.5-vl-7b-instruct"));
        assert!(model_supports_vision("llava:7b"));
        // Text-only families must NOT be flagged.
        assert!(!model_supports_vision("deepseek-v4-flash"));
        assert!(!model_supports_vision("qwen2.5:7b"));
        assert!(!model_supports_vision("llama3.1:8b"));
        assert!(!model_supports_vision(""));
    }

    // ─── classification-first routing ────────────────────────────────────

    fn classify(
        w: u32,
        h: u32,
        lum: f64,
        contrast: f64,
        text: f64,
        edge: f64,
        colors: &[[u8; 3]],
    ) -> ImageClass {
        classify_image(w, h, lum, contrast, text, edge, colors)
    }

    #[test]
    fn test_classify_blank_via_low_contrast() {
        assert_eq!(
            classify(64, 64, 0.2, 0.01, 0.0, 0.0, &[[100, 100, 100]]),
            ImageClass::Blank
        );
    }

    #[test]
    fn test_classify_blank_via_near_black_luminance() {
        // Logo/black-screen intro frames: solid near-black even with faint noise.
        assert_eq!(
            classify(64, 64, 0.005, 0.1, 0.0, 0.0, &[[0, 0, 0]]),
            ImageClass::Blank
        );
        // Near-white flash frames.
        assert_eq!(
            classify(64, 64, 0.995, 0.1, 0.0, 0.0, &[[255, 255, 255]]),
            ImageClass::Blank
        );
    }

    #[test]
    fn test_classify_document_needs_light_bg_and_text() {
        // Light dominant + high text density -> Document.
        assert_eq!(
            classify(
                256,
                256,
                0.8,
                0.4,
                0.5,
                0.05,
                &[[248, 248, 248], [10, 10, 10]]
            ),
            ImageClass::Document
        );
        // Dark dominant must NOT be a document.
        assert_eq!(
            classify(
                256,
                256,
                0.2,
                0.4,
                0.5,
                0.05,
                &[[10, 10, 10], [248, 248, 248]]
            ),
            ImageClass::Screenshot
        );
    }

    #[test]
    fn test_classify_screenshot_via_saturated_palette() {
        // Two saturated accent colors + some text -> UI screenshot.
        assert_eq!(
            classify(
                256,
                256,
                0.5,
                0.4,
                0.2,
                0.1,
                &[[200, 60, 60], [60, 60, 220], [240, 240, 240]]
            ),
            ImageClass::Screenshot
        );
    }

    #[test]
    fn test_classify_photo_low_saturation_low_edge() {
        assert_eq!(
            classify(
                256,
                256,
                0.5,
                0.3,
                0.05,
                0.06,
                &[[120, 130, 140], [90, 100, 110]]
            ),
            ImageClass::Photo
        );
    }

    #[test]
    fn test_classify_tiny_or_ambiguous_is_unknown() {
        assert_eq!(
            classify(4, 4, 0.5, 0.4, 0.5, 0.1, &[[10, 10, 10]]),
            ImageClass::Unknown
        );
        assert_eq!(
            classify(64, 64, 0.5, 0.3, 0.15, 0.3, &[[120, 130, 140]]),
            ImageClass::Unknown
        );
    }

    #[test]
    fn test_image_class_as_str_roundtrip() {
        assert_eq!(ImageClass::Document.as_str(), "document");
        assert_eq!(ImageClass::Screenshot.as_str(), "screenshot");
        assert_eq!(ImageClass::Photo.as_str(), "photo");
        assert_eq!(ImageClass::Blank.as_str(), "blank");
        assert_eq!(ImageClass::Unknown.as_str(), "unknown");
    }

    /// Build a 256×256 pattern where tile (gx,gy) is black when
    /// `(gx + 2*gy) % 5 < 2`, otherwise white: ~40% black tiles, no adjacency
    /// chain large enough to flatten the luminance variance, white dominant.
    fn test_document_png() -> Vec<u8> {
        let mut img = image::RgbImage::new(256, 256);
        for gy in 0..16 {
            for gx in 0..16 {
                let black = (gx + 2 * gy) % 5 < 2;
                let c = if black { [0, 0, 0] } else { [255, 255, 255] };
                for py in (gy * 16)..((gy + 1) * 16) {
                    for px in (gx * 16)..((gx + 1) * 16) {
                        img.put_pixel(px, py, image::Rgb(c));
                    }
                }
            }
        }
        let mut buf = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, image::ImageFormat::Png)
            .expect("encode doc png");
        buf.into_inner()
    }

    #[test]
    fn test_analyze_classifies_document_png() {
        let (ev, _) = VisionBridge::analyze(&test_document_png()).expect("decode");
        assert_eq!(ev.classification, ImageClass::Document, "evidence: {ev:?}");
        assert!(ev.text_density > 0.35, "text_density {}", ev.text_density);
        assert!(ev.contrast > 0.1);
    }

    #[test]
    fn test_analyze_classifies_solid_colors_blank() {
        for c in [[0, 0, 0], [255, 255, 255], [100, 100, 100]] {
            let (ev, _) = VisionBridge::analyze(&test_png(64, 64, c)).expect("decode");
            assert_eq!(ev.classification, ImageClass::Blank, "color {c:?}");
        }
    }

    #[test]
    fn test_analyze_evidence_text_includes_classification_and_hash() {
        let (ev, _) = VisionBridge::analyze(&test_document_png()).expect("decode");
        let txt = ev.to_evidence_text();
        assert!(txt.contains("classification: document"), "got:\n{txt}");
        assert!(txt.contains("phash:"));
        assert!(txt.contains("content_tag:"));
    }

    // ─── content-addressed cache ─────────────────────────────────────────

    #[test]
    fn test_analyze_cached_serves_repeat_payload_from_cache() {
        let png = test_document_png();
        let a = VisionBridge::analyze_cached(&png).expect("first");
        let b = VisionBridge::analyze_cached(&png).expect("cached");
        assert_eq!(a.0.sha256, b.0.sha256);
        assert_eq!(a.1, b.1);
    }

    #[test]
    fn test_analyze_cached_stays_bounded() {
        // Insert CACHE_CAPACITY + 1 unique payloads; cache must never exceed cap.
        for i in 0..(CACHE_CAPACITY + 3) {
            let fill = [i as u8, (i / 2) as u8, (i / 3) as u8];
            let png = test_png(8, 8, fill);
            let _ = VisionBridge::analyze_cached(&png).expect("decode");
        }
        assert!(EVIDENCE_CACHE.lock().unwrap().len() <= CACHE_CAPACITY);
    }

    // ─── perceptual hash ─────────────────────────────────────────────────

    #[test]
    fn test_phash_stable_across_resizes() {
        // Left-black / right-white halves: block sampling is scale-invariant,
        // so a 64×64 and 63×63 rendering share an (almost) identical phash.
        fn half_png(w: u32, h: u32) -> Vec<u8> {
            let mut img = image::RgbImage::new(w, h);
            for (px, _py, p) in img.enumerate_pixels_mut() {
                *p = if px < w / 2 {
                    image::Rgb([0, 0, 0])
                } else {
                    image::Rgb([255, 255, 255])
                };
            }
            let mut buf = std::io::Cursor::new(Vec::new());
            image::DynamicImage::ImageRgb8(img)
                .write_to(&mut buf, image::ImageFormat::Png)
                .expect("encode half png");
            buf.into_inner()
        }
        let (a, _) = VisionBridge::analyze(&half_png(64, 64)).expect("a");
        let (b, _) = VisionBridge::analyze(&half_png(63, 63)).expect("b");
        let (c, _) = VisionBridge::analyze(&test_document_png()).expect("c");
        let d_ab = VisionBridge::phash_distance(a.phash, b.phash);
        let d_ac = VisionBridge::phash_distance(a.phash, c.phash);
        assert!(d_ab <= 4, "near-dup distance too large: {d_ab}");
        assert!(d_ac > 12, "dissimilar images too close: {d_ac}");
        assert!(d_ab < d_ac, "near-dup must be closer than dissimilar");
    }
}
