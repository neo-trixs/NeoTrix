use crate::core::nt_core_consciousness::pixel_perception::EmbeddingBackend;
use crate::core::nt_core_hcube::cross_modal::CrossModalAligner;

const DEFAULT_EMBED_DIM: usize = 768;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisualEmbeddingModel {
    Qwen3VL2B,
    Qwen3VL8B,
    ColPali,
    Custom(String),
}

impl VisualEmbeddingModel {
    pub fn name(&self) -> &str {
        match self {
            VisualEmbeddingModel::Qwen3VL2B => "Qwen/Qwen3-VL-Embedding-2B",
            VisualEmbeddingModel::Qwen3VL8B => "Qwen/Qwen3-VL-Embedding-8B",
            VisualEmbeddingModel::ColPali => "vidore/colpali-v1.2",
            VisualEmbeddingModel::Custom(name) => name,
        }
    }

    pub fn embed_dim(&self) -> usize {
        match self {
            VisualEmbeddingModel::Qwen3VL2B => 1536,
            VisualEmbeddingModel::Qwen3VL8B => 2048,
            VisualEmbeddingModel::ColPali => 128,
            VisualEmbeddingModel::Custom(_) => DEFAULT_EMBED_DIM,
        }
    }
}

impl Default for VisualEmbeddingModel {
    fn default() -> Self {
        VisualEmbeddingModel::Qwen3VL2B
    }
}

#[derive(Debug, Clone)]
pub struct VisualEmbeddingFrontend {
    pub model: VisualEmbeddingModel,
    pub backend: EmbeddingBackend,
    pub embed_dim: usize,
    pub batch_size: usize,
}

impl VisualEmbeddingFrontend {
    pub fn new(model: VisualEmbeddingModel, backend: EmbeddingBackend) -> Self {
        let embed_dim = model.embed_dim();
        Self {
            model,
            backend,
            embed_dim,
            batch_size: 8,
        }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub fn embed_image(&self, pixels: &[u8], width: u32, height: u32) -> Vec<f32> {
        let pixel_count = (width * height) as usize;
        let channels = if pixels.len() > pixel_count { 4 } else { 3 };
        let expected = pixel_count * channels;
        if pixels.len() < expected || width == 0 || height == 0 {
            return vec![0.0; self.embed_dim];
        }
        let downsampled = downsample(pixels, width, height, self.embed_dim);
        embed_stub(downsampled, self.embed_dim)
    }

    pub fn embed_batch(&self, images: &[Vec<u8>]) -> Vec<Vec<f32>> {
        images
            .iter()
            .map(|img| {
                if img.is_empty() {
                    return vec![0.0; self.embed_dim];
                }
                let w = 224;
                let h = 224;
                let downsampled = downsample(img, w, h, self.embed_dim);
                embed_stub(downsampled, self.embed_dim)
            })
            .collect()
    }

    pub fn embed_with_backend(
        &self,
        backend: EmbeddingBackend,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Vec<f32> {
        let _ = backend;
        self.embed_image(pixels, width, height)
    }

    pub fn project_to_vsa(embed: &[f32], aligner: &CrossModalAligner) -> Vec<u8> {
        aligner.image_embed_to_vsa(embed)
    }

    pub fn pipeline_from_url(url: &str, aligner: &CrossModalAligner) -> Vec<u8> {
        let dummy_pixels = simulate_capture(url);
        let width = 1280u32;
        let height = 1024u32;
        let dim = 768usize;
        let downsampled = downsample(&dummy_pixels, width, height, dim);
        let embed = embed_stub(downsampled, dim);
        aligner.image_embed_to_vsa(&embed)
    }

    pub fn model(&self) -> &VisualEmbeddingModel {
        &self.model
    }

    pub fn backend(&self) -> &EmbeddingBackend {
        &self.backend
    }

    pub fn embed_dim(&self) -> usize {
        self.embed_dim
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}

impl Default for VisualEmbeddingFrontend {
    fn default() -> Self {
        Self::new(VisualEmbeddingModel::default(), EmbeddingBackend::auto())
    }
}

fn downsample(pixels: &[u8], width: u32, height: u32, target_dim: usize) -> Vec<f32> {
    let total_pixels = (width * height) as usize;
    let channels = if pixels.len() > total_pixels { 4 } else { 3 };
    let total = total_pixels * channels;
    if total == 0 || target_dim == 0 {
        return vec![0.0; target_dim];
    }
    let step = (total / target_dim).max(1);
    let mut result = Vec::with_capacity(target_dim);
    let mut idx = 0usize;
    for _ in 0..target_dim {
        let raw = pixels.get(idx).copied().unwrap_or(128) as f32;
        result.push(raw / 255.0);
        idx = idx.wrapping_add(step);
    }
    result
}

fn embed_stub(input: Vec<f32>, dim: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(dim);
    for i in 0..dim {
        let val = input.get(i).copied().unwrap_or(0.0);
        out.push(val.sin() * 0.5 + 0.5);
    }
    out
}

fn simulate_capture(url: &str) -> Vec<u8> {
    let _ = url;
    vec![128u8; 1280 * 1024 * 3]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    fn aligner() -> CrossModalAligner {
        CrossModalAligner::new(VSA_DIM, 42)
    }

    fn dummy_image() -> (Vec<u8>, u32, u32) {
        let w = 224u32;
        let h = 224u32;
        let pixels = vec![128u8; (w * h * 3) as usize];
        (pixels, w, h)
    }

    #[test]
    fn test_embed_image_default_dim() {
        let frontend = VisualEmbeddingFrontend::default();
        let (pixels, w, h) = dummy_image();
        let embed = frontend.embed_image(&pixels, w, h);
        assert_eq!(embed.len(), DEFAULT_EMBED_DIM);
    }

    #[test]
    fn test_embed_image_non_empty() {
        let frontend = VisualEmbeddingFrontend::default();
        let (pixels, w, h) = dummy_image();
        let embed = frontend.embed_image(&pixels, w, h);
        assert!(!embed.is_empty());
        assert!(embed.iter().any(|&v| v != 0.0));
    }

    #[test]
    fn test_embed_image_zero_dim_input() {
        let frontend = VisualEmbeddingFrontend::default();
        let embed = frontend.embed_image(&[], 0, 0);
        assert_eq!(embed.len(), DEFAULT_EMBED_DIM);
        assert!(embed.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_embed_image_zero_width() {
        let frontend = VisualEmbeddingFrontend::default();
        let pixels = vec![128u8; 100];
        let embed = frontend.embed_image(&pixels, 0, 100);
        assert_eq!(embed.len(), DEFAULT_EMBED_DIM);
        assert!(embed.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_embed_batch_multi() {
        let frontend = VisualEmbeddingFrontend::default();
        let (p, w, h) = dummy_image();
        let images = vec![p.clone(), p, vec![]];
        let embeddings = frontend.embed_batch(&images);
        assert_eq!(embeddings.len(), 3);
        for e in &embeddings {
            assert_eq!(e.len(), DEFAULT_EMBED_DIM);
        }
        assert!(embeddings[2].iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_embed_batch_empty() {
        let frontend = VisualEmbeddingFrontend::default();
        let embeddings = frontend.embed_batch(&[]);
        assert!(embeddings.is_empty());
    }

    #[test]
    fn test_embed_with_backend_routes() {
        let frontend = VisualEmbeddingFrontend::default();
        let (pixels, w, h) = dummy_image();
        let cpu_embed = frontend.embed_with_backend(EmbeddingBackend::CPU, &pixels, w, h);
        let mps_embed = frontend.embed_with_backend(EmbeddingBackend::MPS, &pixels, w, h);
        assert_eq!(cpu_embed.len(), DEFAULT_EMBED_DIM);
        assert_eq!(mps_embed.len(), DEFAULT_EMBED_DIM);
    }

    #[test]
    fn test_project_to_vsa_deterministic() {
        let a = aligner();
        let embed: Vec<f32> = (0..768).map(|i| (i as f32).sin()).collect();
        let vsa1 = VisualEmbeddingFrontend::project_to_vsa(&embed, &a);
        let vsa2 = VisualEmbeddingFrontend::project_to_vsa(&embed, &a);
        assert_eq!(vsa1.len(), VSA_DIM);
        assert_eq!(vsa1, vsa2);
        for &x in &vsa1 {
            assert!(x == 0 || x == 1);
        }
    }

    #[test]
    fn test_project_to_vsa_different_embeddings_differ() {
        let a = aligner();
        let embed_a: Vec<f32> = (0..768).map(|i| (i as f32).sin()).collect();
        let embed_b: Vec<f32> = (0..768).map(|i| (i as f32).cos()).collect();
        let vsa_a = VisualEmbeddingFrontend::project_to_vsa(&embed_a, &a);
        let vsa_b = VisualEmbeddingFrontend::project_to_vsa(&embed_b, &a);
        assert_ne!(vsa_a, vsa_b);
    }

    #[test]
    fn test_pipeline_from_url_returns_correct_len() {
        let a = aligner();
        let vsa = VisualEmbeddingFrontend::pipeline_from_url("https://example.com", &a);
        assert_eq!(vsa.len(), VSA_DIM);
        for &x in &vsa {
            assert!(x == 0 || x == 1);
        }
    }

    #[test]
    fn test_visual_embedding_model_name() {
        assert_eq!(
            VisualEmbeddingModel::Qwen3VL2B.name(),
            "Qwen/Qwen3-VL-Embedding-2B"
        );
        assert_eq!(
            VisualEmbeddingModel::Qwen3VL8B.name(),
            "Qwen/Qwen3-VL-Embedding-8B"
        );
        assert_eq!(VisualEmbeddingModel::ColPali.name(), "vidore/colpali-v1.2");
        assert_eq!(
            VisualEmbeddingModel::Custom("my-model".into()).name(),
            "my-model"
        );
    }

    #[test]
    fn test_visual_embedding_model_embed_dim() {
        assert_eq!(VisualEmbeddingModel::Qwen3VL2B.embed_dim(), 1536);
        assert_eq!(VisualEmbeddingModel::Qwen3VL8B.embed_dim(), 2048);
        assert_eq!(VisualEmbeddingModel::ColPali.embed_dim(), 128);
        assert_eq!(
            VisualEmbeddingModel::Custom("x".into()).embed_dim(),
            DEFAULT_EMBED_DIM
        );
    }

    #[test]
    fn test_frontend_default_construction() {
        let frontend = VisualEmbeddingFrontend::default();
        assert_eq!(frontend.model, VisualEmbeddingModel::Qwen3VL2B);
        assert_eq!(frontend.embed_dim, 1536);
        assert_eq!(frontend.batch_size, 8);
    }

    #[test]
    fn test_frontend_with_batch_size() {
        let frontend = VisualEmbeddingFrontend::default().with_batch_size(16);
        assert_eq!(frontend.batch_size, 16);
    }

    #[test]
    fn test_frontend_custom_model() {
        let model = VisualEmbeddingModel::Custom("my-vlm".into());
        let frontend = VisualEmbeddingFrontend::new(model, EmbeddingBackend::CPU);
        assert_eq!(frontend.model.name(), "my-vlm");
        assert_eq!(frontend.embed_dim, DEFAULT_EMBED_DIM);
    }

    #[test]
    fn test_model_default_is_qwen_2b() {
        let m: VisualEmbeddingModel = Default::default();
        assert_eq!(m, VisualEmbeddingModel::Qwen3VL2B);
    }
}
