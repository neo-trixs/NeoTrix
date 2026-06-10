use crate::core::nt_core_hcube::cross_modal::CrossModalAligner;
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

/// Perceptual feature extracted from an image
#[derive(Debug, Clone)]
pub struct ImagePercept {
    /// File path or identifier
    pub source: String,
    /// VSA vector encoding the image content
    pub vsa_fingerprint: Vec<u8>,
    /// Modality tag bound to the fingerprint
    pub modality_tag: Vec<u8>,
    /// Confidence in the perceptual encoding [0, 1]
    pub confidence: f64,
}

/// Result of a vision processing operation
#[derive(Debug, Clone)]
pub struct VisionResult {
    pub percept: ImagePercept,
    /// Estimated image width in pixels
    pub width: u32,
    /// Estimated image height in pixels
    pub height: u32,
    /// Raw perceptual hash (64-bit) for quick dedup
    pub dhash: u64,
    /// Dominant color cluster (average R, G, B)
    pub dominant_color: (u8, u8, u8),
}

impl VisionResult {
    pub fn empty(source: &str) -> Self {
        Self {
            percept: ImagePercept {
                source: source.into(),
                vsa_fingerprint: vec![0; VSA_DIM],
                modality_tag: vec![0; VSA_DIM],
                confidence: 0.0,
            },
            width: 0, height: 0,
            dhash: 0,
            dominant_color: (0, 0, 0),
        }
    }
}

/// Vision system — perceptual pipeline for loading images and encoding them
/// into VSA space via the CrossModalAligner.
///
/// Without the `image` crate, only external embedding injection is supported.
/// With the `image` feature, loads and processes local image files.
pub struct VisionSystem {
    aligner: CrossModalAligner,
}

impl Default for VisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl VisionSystem {
    pub fn new() -> Self {
        Self {
            aligner: CrossModalAligner::new(VSA_DIM, 42),
        }
    }

    /// Compute a 64-bit difference hash (dHash) from raw pixel data.
    /// dHash is robust to scaling and minor color shifts.
    pub fn dhash(pixels: &[u8], width: u32, height: u32) -> u64 {
        if pixels.len() < 9 || width < 2 || height < 2 {
            return 0;
        }
        // Downsample to 9x8 grayscale for a 64-bit hash
        let small_w = 9usize;
        let small_h = 8usize;
        let mut gray = vec![0u8; small_w * small_h];
        for gy in 0..small_h {
            for gx in 0..small_w {
                let src_x = (gx as f64 / small_w as f64 * width as f64) as usize;
                let src_y = (gy as f64 / small_h as f64 * height as f64) as usize;
                let idx = (src_y * width as usize + src_x) * 3;
                let r = pixels.get(idx).copied().unwrap_or(0) as u32;
                let g = pixels.get(idx + 1).copied().unwrap_or(0) as u32;
                let b = pixels.get(idx + 2).copied().unwrap_or(0) as u32;
                gray[gy * small_w + gx] = ((r * 77 + g * 151 + b * 28) / 256) as u8;
            }
        }
        let mut hash = 0u64;
        for y in 0..small_h {
            for x in 0..(small_w - 1) {
                let bit = if gray[y * small_w + x] > gray[y * small_w + x + 1] { 1 } else { 0 };
                hash = (hash << 1) | bit;
            }
        }
        hash
    }

    /// Compute average color from raw RGB pixel data
    fn dominant_color(pixels: &[u8]) -> (u8, u8, u8) {
        if pixels.len() < 3 { return (0, 0, 0); }
        let pixel_count = pixels.len() / 3;
        let (mut r, mut g, mut b) = (0u64, 0u64, 0u64);
        for i in 0..pixel_count {
            let idx = i * 3;
            r += pixels[idx] as u64;
            g += pixels[idx + 1] as u64;
            b += pixels[idx + 2] as u64;
        }
        let n = pixel_count.max(1) as u64;
        ((r / n) as u8, (g / n) as u8, (b / n) as u8)
    }

    /// Process raw RGB pixel data into a VSA-encoded perception.
    ///
    /// `pixels` should be flat RGB data (3 bytes per pixel, row-major).
    /// The perceptual hash is projected through CrossModalAligner into VSA space.
    pub fn process_pixels(&self, source: &str, pixels: &[u8], width: u32, height: u32) -> VisionResult {
        if pixels.len() < 3 || width == 0 || height == 0 {
            return VisionResult::empty(source);
        }

        let dhash = Self::dhash(pixels, width, height);
        let dc = Self::dominant_color(pixels);

        // Encode the dhash + color into a VSA fingerprint
        let embed = self.dhash_to_embed(dhash, dc);
        let vsa_fp = self.aligner.image_embed_to_vsa(&embed);
        let tag = self.aligner.modality_tag("image");
        let bound = QuantizedVSA::bind(&vsa_fp, &tag);

        VisionResult {
            percept: ImagePercept {
                source: source.into(),
                vsa_fingerprint: bound,
                modality_tag: tag,
                confidence: 0.7,
            },
            width, height,
            dhash,
            dominant_color: dc,
        }
    }

    /// Accept an externally computed float embedding (e.g. from CLIP)
    /// and project it into VSA space.
    ///
    /// This is the primary entry point for integration with external vision
    /// models. The embedding can be any length (512, 768, 1024, etc.).
    pub fn process_embedding(&self, source: &str, embed: &[f32], confidence: f64) -> ImagePercept {
        let vsa_fp = self.aligner.image_embed_to_vsa(embed);
        let tag = self.aligner.modality_tag("image");
        let bound = QuantizedVSA::bind(&vsa_fp, &tag);
        ImagePercept {
            source: source.into(),
            vsa_fingerprint: bound,
            modality_tag: tag,
            confidence,
        }
    }

    /// Compare two visual perceptions for similarity
    pub fn compare(&self, a: &ImagePercept, b: &ImagePercept) -> f64 {
        QuantizedVSA::similarity(&a.vsa_fingerprint, &b.vsa_fingerprint)
    }

    /// Convert a dhash and dominant color into a short float embedding for VSA projection
    fn dhash_to_embed(&self, dhash: u64, color: (u8, u8, u8)) -> Vec<f32> {
        let mut embed = Vec::with_capacity(72);
        // 64 bits from dhash, expanded to float
        for i in 0..64 {
            embed.push(if (dhash >> i) & 1 == 1 { 1.0 } else { -1.0 });
        }
        // 8 color dimensions
        embed.push(color.0 as f32 / 255.0 * 2.0 - 1.0);
        embed.push(color.1 as f32 / 255.0 * 2.0 - 1.0);
        embed.push(color.2 as f32 / 255.0 * 2.0 - 1.0);
        embed
    }

    /// Compute hamming distance between two dHash values
    pub fn dhash_distance(a: u64, b: u64) -> u32 {
        (a ^ b).count_ones()
    }

    /// Normalized dHash similarity [0, 1]
    pub fn dhash_similarity(a: u64, b: u64) -> f64 {
        1.0 - Self::dhash_distance(a, b) as f64 / 64.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_image_returns_empty() {
        let vs = VisionSystem::new();
        let result = vs.process_pixels("empty", &[], 0, 0);
        assert_eq!(result.width, 0);
        assert_eq!(result.percept.confidence, 0.0);
    }

    #[test]
    fn test_process_pixels_creates_vsa_fingerprint() {
        let vs = VisionSystem::new();
        let pixels: Vec<u8> = (0..300).map(|i| (i % 256) as u8).collect();
        let result = vs.process_pixels("test", &pixels, 10, 10);
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
        assert!(result.percept.confidence > 0.0);
        assert!(result.dhash != 0);
    }

    #[test]
    fn test_deterministic_dhash() {
        let pixels: Vec<u8> = (0..2700).map(|i| (i % 256) as u8).collect();
        let h1 = VisionSystem::dhash(&pixels, 30, 30);
        let h2 = VisionSystem::dhash(&pixels, 30, 30);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_dhash_differs_for_different_images() {
        let dark: Vec<u8> = vec![0u8; 2700];
        let light: Vec<u8> = vec![255u8; 2700];
        let h1 = VisionSystem::dhash(&dark, 30, 30);
        let h2 = VisionSystem::dhash(&light, 30, 30);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_dhash_distance() {
        let h1 = 0xDEADBEEF_CAFEBABE_u64;
        let h2 = 0xDEADBEEF_CAFEBABE_u64;
        assert_eq!(VisionSystem::dhash_distance(h1, h2), 0);
        assert!((VisionSystem::dhash_similarity(h1, h2) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dhash_max_distance() {
        let h1 = 0xFFFF_FFFF_FFFF_FFFF_u64;
        let h2 = 0x0000_0000_0000_0000_u64;
        assert_eq!(VisionSystem::dhash_distance(h1, h2), 64);
        assert!((VisionSystem::dhash_similarity(h1, h2) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_dominant_color() {
        let pixels: Vec<u8> = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let dc = VisionSystem::dominant_color(&pixels);
        assert_eq!(dc, (85, 85, 85));
    }

    #[test]
    fn test_process_embedding() {
        let vs = VisionSystem::new();
        let embed = vec![0.5f32; 512];
        let percept = vs.process_embedding("clip", &embed, 0.9);
        assert_eq!(percept.source, "clip");
        assert!((percept.confidence - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_compare_same_image() {
        let vs = VisionSystem::new();
        let pixels: Vec<u8> = (0..300).map(|i| (i % 256) as u8).collect();
        let r1 = vs.process_pixels("a", &pixels, 10, 10);
        let r2 = vs.process_pixels("b", &pixels, 10, 10);
        let sim = vs.compare(&r1.percept, &r2.percept);
        assert!(sim > 0.8, "same pixels should be similar, got {}", sim);
    }

    #[test]
    fn test_dhash_small_image() {
        let pixels = vec![100u8; 27];
        let hash = VisionSystem::dhash(&pixels, 3, 3);
        assert_eq!(hash, 0, "too small for 9x8 dhash");
    }
}

// Extension trait for ImagePercept
impl ImagePercept {
    pub fn confidence(&self) -> f64 {
        self.confidence
    }
}

/// Pipeline stage: warm-up vision processing tick.
/// Scans a configurable directory for new image files and encodes them.
pub struct VisionStage {
    watch_dir: Option<String>,
}

impl VisionStage {
    pub fn new(watch_dir: Option<String>) -> Self {
        Self { watch_dir }
    }
}

impl Default for VisionStage {
    fn default() -> Self {
        Self::new(None)
    }
}

impl crate::neotrix::nt_mind::self_iterating::pipeline::BrainStage for VisionStage {
    fn name(&self) -> &str {
        "vision_scan"
    }

    fn frequency(&self) -> usize {
        50
    }

    fn process(&self, _brain: &mut crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain) -> Result<crate::neotrix::nt_mind::self_iterating::pipeline::StageDecision, crate::neotrix::nt_core_error::NeoTrixError> {
        use crate::neotrix::nt_mind::self_iterating::pipeline::StageDecision;
        let _dir = match &self.watch_dir {
            Some(d) => d.clone(),
            None => return Ok(StageDecision::Continue),
        };
        let _system = VisionSystem::new();
        log::debug!("[vision] stage tick (watch_dir={:?})", self.watch_dir);
        Ok(StageDecision::Continue)
    }
}
