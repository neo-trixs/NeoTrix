use crate::core::nt_core_consciousness::vsa_tag::SenseModality;
use crate::core::nt_core_hcube::cross_modal::CrossModalAligner;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

const VSA_DIM: usize = 4096;
const MAX_TILES: usize = 64;
const CHUNK_HEIGHT: u32 = 1024;
const TILE_HEIGHT: u32 = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingBackend {
    VLLM,
    SGLang,
    DirectGPU,
    CPU,
    MPS,
}

impl EmbeddingBackend {
    pub fn name(&self) -> &'static str {
        match self {
            EmbeddingBackend::VLLM => "vllm",
            EmbeddingBackend::SGLang => "sglang",
            EmbeddingBackend::DirectGPU => "direct_gpu",
            EmbeddingBackend::CPU => "cpu",
            EmbeddingBackend::MPS => "mps",
        }
    }

    pub fn auto() -> Self {
        #[cfg(target_os = "macos")]
        {
            EmbeddingBackend::MPS
        }
        #[cfg(all(target_os = "linux", feature = "cuda"))]
        {
            EmbeddingBackend::DirectGPU
        }
        #[cfg(not(any(target_os = "macos", feature = "cuda")))]
        {
            EmbeddingBackend::CPU
        }
    }
}

#[derive(Debug, Clone)]
pub struct PixelRenderingConfig {
    pub viewport_width: u32,
    pub tile_height: u32,
    pub chunk_height: u32,
    pub wait_network_idle: bool,
    pub max_tiles: usize,
    pub dpi: u32,
}

impl Default for PixelRenderingConfig {
    fn default() -> Self {
        Self {
            viewport_width: 1280,
            tile_height: TILE_HEIGHT,
            chunk_height: CHUNK_HEIGHT,
            wait_network_idle: true,
            max_tiles: MAX_TILES,
            dpi: 96,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VisualTile {
    pub article_id: String,
    pub tile_index: u32,
    pub chunk_index: u32,
    pub y_offset: u32,
    pub width: u32,
    pub height: u32,
    pub md5_hash: u64,
    pub source_url: String,
}

impl VisualTile {
    pub fn tile_key(&self) -> String {
        format!(
            "{}_t{:04}_c{:04}",
            self.article_id, self.tile_index, self.chunk_index
        )
    }
}

#[derive(Debug, Clone)]
pub struct VisualEmbedding {
    pub tile_key: String,
    pub embedding: Vec<f32>,
    pub vsa_vector: Vec<u8>,
    pub modality: SenseModality,
}

#[derive(Debug, Clone)]
pub struct VisualSceneBuffer {
    tiles: Vec<VisualTile>,
    embeddings: Vec<VisualEmbedding>,
    capacity: usize,
}

impl VisualSceneBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            tiles: Vec::with_capacity(capacity),
            embeddings: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, tile: VisualTile, embed: VisualEmbedding) {
        if self.tiles.len() >= self.capacity {
            self.tiles.remove(0);
            self.embeddings.remove(0);
        }
        self.tiles.push(tile);
        self.embeddings.push(embed);
    }

    pub fn tiles(&self) -> &[VisualTile] {
        &self.tiles
    }

    pub fn embeddings(&self) -> &[VisualEmbedding] {
        &self.embeddings
    }

    pub fn find_by_url(&self, url: &str) -> Option<&VisualTile> {
        self.tiles.iter().find(|t| t.source_url == url)
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
        self.embeddings.clear();
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct VisualEmbeddingConfig {
    pub model_name: String,
    pub backend: EmbeddingBackend,
    pub lora_adapter: Option<String>,
    pub device: String,
    pub batch_size: usize,
}

impl Default for VisualEmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "Qwen/Qwen3-VL-Embedding-2B".into(),
            backend: EmbeddingBackend::auto(),
            lora_adapter: None,
            device: "auto".into(),
            batch_size: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VisualToVSABridge {
    aligner: CrossModalAligner,
    dim: usize,
}

impl VisualToVSABridge {
    pub fn new(dim: usize, seed: u64) -> Self {
        Self {
            aligner: CrossModalAligner::new(dim, seed),
            dim,
        }
    }

    pub fn embed_to_vsa(&self, embed: &[f32]) -> Vec<u8> {
        self.aligner.image_embed_to_vsa(embed)
    }

    pub fn bind_visual_tag(&self, vsa: &[u8]) -> Vec<u8> {
        let tag = self.aligner.modality_tag("visual");
        QuantizedVSA::bind(vsa, &tag)
    }

    pub fn bind_position(&self, vsa: &[u8], x: u32, y: u32) -> Vec<u8> {
        let pos_vsa = self.position_to_vsa(x, y);
        QuantizedVSA::bind(vsa, &pos_vsa)
    }

    fn position_to_vsa(&self, x: u32, y: u32) -> Vec<u8> {
        let mut combined = Vec::with_capacity(self.dim);
        let x_vsa = QuantizedVSA::seeded_random(x as u64, self.dim);
        let y_vsa = QuantizedVSA::seeded_random(y as u64, self.dim);
        for i in 0..self.dim {
            combined.push(x_vsa[i] ^ y_vsa[i]);
        }
        combined
    }

    pub fn cross_modal_similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        QuantizedVSA::similarity(a, b)
    }

    pub fn aligner(&self) -> &CrossModalAligner {
        &self.aligner
    }
}

#[derive(Debug, Clone)]
pub struct PixelPerceptionPipeline {
    pub rendering_config: PixelRenderingConfig,
    pub embedding_config: VisualEmbeddingConfig,
    pub vsa_bridge: VisualToVSABridge,
    pub scene_buffer: VisualSceneBuffer,
}

impl PixelPerceptionPipeline {
    pub fn new() -> Self {
        Self {
            rendering_config: PixelRenderingConfig::default(),
            embedding_config: VisualEmbeddingConfig::default(),
            vsa_bridge: VisualToVSABridge::new(VSA_DIM, 42),
            scene_buffer: VisualSceneBuffer::new(MAX_TILES),
        }
    }

    pub fn with_config(
        rendering_config: PixelRenderingConfig,
        embedding_config: VisualEmbeddingConfig,
        seed: u64,
    ) -> Self {
        Self {
            rendering_config,
            embedding_config,
            vsa_bridge: VisualToVSABridge::new(VSA_DIM, seed),
            scene_buffer: VisualSceneBuffer::new(MAX_TILES),
        }
    }

    pub fn process_visual_input(&mut self, tile: VisualTile, raw_embedding: Vec<f32>) -> Vec<u8> {
        let vsa = self.vsa_bridge.embed_to_vsa(&raw_embedding);
        let tagged = self.vsa_bridge.bind_visual_tag(&vsa);
        let positioned = self
            .vsa_bridge
            .bind_position(&tagged, tile.width, tile.y_offset);

        let embed = VisualEmbedding {
            tile_key: tile.tile_key(),
            embedding: raw_embedding,
            vsa_vector: positioned.clone(),
            modality: SenseModality::Visual,
        };

        self.scene_buffer.push(tile, embed);
        positioned
    }

    pub fn query_visual_memory(&self, query_vsa: &[u8]) -> Vec<(usize, f64)> {
        let mut results: Vec<(usize, f64)> = self
            .scene_buffer
            .embeddings()
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let sim = QuantizedVSA::similarity(query_vsa, &e.vsa_vector);
                (i, sim)
            })
            .filter(|(_, s)| *s > 0.3)
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(10);
        results
    }

    pub fn rendering_config(&self) -> &PixelRenderingConfig {
        &self.rendering_config
    }

    pub fn embedding_config(&self) -> &VisualEmbeddingConfig {
        &self.embedding_config
    }

    pub fn process_tile(&mut self, tile: &VisualTile) -> Vec<u8> {
        let raw_embedding = self.make_default_embedding(tile);
        self.process_visual_input(tile.clone(), raw_embedding)
    }

    fn make_default_embedding(&self, tile: &VisualTile) -> Vec<f32> {
        (0..VSA_DIM)
            .map(|i| {
                let seed = tile.md5_hash ^ (tile.tile_index as u64) ^ (i as u64);
                (seed as f32).sin()
            })
            .collect()
    }

    pub fn scene_buffer(&self) -> &VisualSceneBuffer {
        &self.scene_buffer
    }

    pub fn scene_buffer_mut(&mut self) -> &mut VisualSceneBuffer {
        &mut self.scene_buffer
    }

    pub fn vsa_bridge(&self) -> &VisualToVSABridge {
        &self.vsa_bridge
    }
}

impl Default for PixelPerceptionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_embedding(dim: usize) -> Vec<f32> {
        (0..dim).map(|i| (i as f32).sin()).collect()
    }

    #[test]
    fn test_visual_tile_key() {
        let tile = VisualTile {
            article_id: "wiki_42".into(),
            tile_index: 1,
            chunk_index: 3,
            y_offset: 2048,
            width: 1280,
            height: 1024,
            md5_hash: 0xdeadbeef,
            source_url: "https://example.com".into(),
        };
        assert_eq!(tile.tile_key(), "wiki_42_t0001_c0003");
    }

    #[test]
    fn test_vsa_bridge_embed_to_vsa() {
        let bridge = VisualToVSABridge::new(256, 42);
        let embed = dummy_embedding(128);
        let vsa = bridge.embed_to_vsa(&embed);
        assert_eq!(vsa.len(), 256);
        assert!(vsa.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_vsa_bridge_tag_and_position() {
        let bridge = VisualToVSABridge::new(256, 42);
        let embed = dummy_embedding(128);
        let vsa = bridge.embed_to_vsa(&embed);
        let tagged = bridge.bind_visual_tag(&vsa);
        assert_eq!(tagged.len(), 256);

        let positioned = bridge.bind_position(&tagged, 1280, 2048);
        assert_eq!(positioned.len(), 256);
    }

    #[test]
    fn test_scene_buffer_push_and_query() {
        let mut buffer = VisualSceneBuffer::new(10);
        let tile = VisualTile {
            article_id: "test".into(),
            tile_index: 0,
            chunk_index: 0,
            y_offset: 0,
            width: 1280,
            height: 1024,
            md5_hash: 1,
            source_url: "https://test.com".into(),
        };
        let embed = VisualEmbedding {
            tile_key: tile.tile_key(),
            embedding: dummy_embedding(128),
            vsa_vector: QuantizedVSA::random_binary(),
            modality: SenseModality::Visual,
        };
        buffer.push(tile, embed);

        assert_eq!(buffer.len(), 1);
        assert!(buffer.find_by_url("https://test.com").is_some());
    }

    #[test]
    fn test_pipeline_process_visual_input() {
        let mut pipeline = PixelPerceptionPipeline::new();
        let tile = VisualTile {
            article_id: "test".into(),
            tile_index: 0,
            chunk_index: 0,
            y_offset: 0,
            width: 1280,
            height: 1024,
            md5_hash: 1,
            source_url: "https://test.com".into(),
        };
        let embed = dummy_embedding(128);
        let vsa = pipeline.process_visual_input(tile, embed);
        assert_eq!(vsa.len(), VSA_DIM);
        assert!(!pipeline.scene_buffer().is_empty());
    }

    #[test]
    fn test_scene_buffer_capacity() {
        let mut buffer = VisualSceneBuffer::new(3);
        for i in 0..5 {
            let tile = VisualTile {
                article_id: format!("t{}", i),
                tile_index: 0,
                chunk_index: 0,
                y_offset: 0,
                width: 1280,
                height: 1024,
                md5_hash: i as u64,
                source_url: format!("https://t{}.com", i),
            };
            let embed = VisualEmbedding {
                tile_key: tile.tile_key(),
                embedding: dummy_embedding(128),
                vsa_vector: QuantizedVSA::random_binary(),
                modality: SenseModality::Visual,
            };
            buffer.push(tile, embed);
        }
        assert_eq!(buffer.len(), 3);
        assert!(buffer.find_by_url("https://t0.com").is_none());
        assert!(buffer.find_by_url("https://t4.com").is_some());
    }

    #[test]
    fn test_embedding_backend_auto() {
        let backend = EmbeddingBackend::auto();
        assert!(matches!(
            backend,
            EmbeddingBackend::MPS | EmbeddingBackend::CPU | EmbeddingBackend::DirectGPU
        ));
    }

    #[test]
    fn test_vsa_bridge_cross_modal_similarity() {
        let bridge = VisualToVSABridge::new(256, 42);
        let a = bridge.embed_to_vsa(&dummy_embedding(128));
        let b = bridge.embed_to_vsa(&dummy_embedding(128));
        let sim = bridge.cross_modal_similarity(&a, &b);
        assert!((0.0..=1.0).contains(&sim));
    }

    #[test]
    fn test_pixel_rendering_config_default() {
        let config = PixelRenderingConfig::default();
        assert_eq!(config.viewport_width, 1280);
        assert_eq!(config.tile_height, 8192);
        assert_eq!(config.chunk_height, 1024);
    }

    #[test]
    fn test_pipeline_query_visual_memory() {
        let mut pipeline = PixelPerceptionPipeline::new();
        for i in 0..5 {
            let tile = VisualTile {
                article_id: format!("doc_{}", i),
                tile_index: i,
                chunk_index: 0,
                y_offset: i * 1024,
                width: 1280,
                height: 1024,
                md5_hash: i as u64,
                source_url: format!("https://doc{}.com", i),
            };
            pipeline.process_visual_input(tile, dummy_embedding(128));
        }
        let query = QuantizedVSA::random_binary();
        let results = pipeline.query_visual_memory(&query);
        assert!(results.len() <= 5);
        for (_, score) in &results {
            assert!((0.0..=1.0).contains(score));
        }
    }

    #[test]
    fn test_pipeline_with_config() {
        let render_config = PixelRenderingConfig {
            viewport_width: 1920,
            tile_height: 4096,
            chunk_height: 512,
            ..Default::default()
        };
        let embed_config = VisualEmbeddingConfig {
            model_name: "Qwen/Qwen3-VL-Embedding-8B".into(),
            backend: EmbeddingBackend::CPU,
            ..Default::default()
        };
        let pipeline = PixelPerceptionPipeline::with_config(render_config, embed_config, 99);
        assert_eq!(pipeline.rendering_config().viewport_width, 1920);
        assert_eq!(
            pipeline.embedding_config().model_name,
            "Qwen/Qwen3-VL-Embedding-8B"
        );
    }

    fn make_test_tile(article_id: &str, idx: u32, url: &str) -> VisualTile {
        VisualTile {
            article_id: article_id.into(),
            tile_index: idx,
            chunk_index: 0,
            y_offset: idx * 1024,
            width: 1280,
            height: 1024,
            md5_hash: idx.wrapping_mul(10007) as u64,
            source_url: url.into(),
        }
    }

    #[test]
    fn test_process_tile_deterministic() {
        let mut pipeline = PixelPerceptionPipeline::new();
        let tile = make_test_tile("det", 1, "https://det.example.com");
        let vsa_a = pipeline.process_tile(&tile);
        let vsa_b = pipeline.process_tile(&tile);
        assert_eq!(vsa_a.len(), VSA_DIM);
        assert_eq!(vsa_b.len(), VSA_DIM);
        assert_eq!(vsa_a, vsa_b, "same tile should produce identical VSA");
    }

    #[test]
    fn test_process_tile_appears_in_scene_buffer() {
        let mut pipeline = PixelPerceptionPipeline::new();
        let tile = make_test_tile("buf", 0, "https://buf.example.com");
        let _vsa = pipeline.process_tile(&tile);
        let buffer = pipeline.scene_buffer();
        assert_eq!(buffer.len(), 1);
        let found = buffer.find_by_url("https://buf.example.com");
        assert!(found.is_some());
        assert_eq!(found.unwrap().tile_index, 0);
    }

    #[test]
    fn test_multiple_tiles_produce_distinct_vsa_outputs() {
        let mut pipeline = PixelPerceptionPipeline::new();
        let tile_a = make_test_tile("multi", 0, "https://a.example.com");
        let tile_b = make_test_tile("multi", 1, "https://b.example.com");
        let vsa_a = pipeline.process_tile(&tile_a);
        let vsa_b = pipeline.process_tile(&tile_b);
        assert_eq!(vsa_a.len(), VSA_DIM);
        assert_eq!(vsa_b.len(), VSA_DIM);
        let sim = QuantizedVSA::similarity(&vsa_a, &vsa_b);
        assert!(
            sim < 0.95,
            "different tiles should produce different VSAs (sim={})",
            sim
        );
    }

    #[test]
    fn test_process_tile_buffer_state_until_capacity() {
        let mut pipeline = PixelPerceptionPipeline::new();
        for i in 0..3 {
            let tile = make_test_tile("cap", i, &format!("https://cap{}.example.com", i));
            pipeline.process_tile(&tile);
        }
        assert_eq!(pipeline.scene_buffer().len(), 3);
        let all_tiles = pipeline.scene_buffer().tiles();
        assert_eq!(all_tiles[0].tile_index, 0);
        assert_eq!(all_tiles[2].tile_index, 2);
    }

    #[test]
    fn test_embedding_backend_name_roundtrip() {
        assert_eq!(EmbeddingBackend::VLLM.name(), "vllm");
        assert_eq!(EmbeddingBackend::SGLang.name(), "sglang");
        assert_eq!(EmbeddingBackend::DirectGPU.name(), "direct_gpu");
        assert_eq!(EmbeddingBackend::CPU.name(), "cpu");
        assert_eq!(EmbeddingBackend::MPS.name(), "mps");
    }
}
