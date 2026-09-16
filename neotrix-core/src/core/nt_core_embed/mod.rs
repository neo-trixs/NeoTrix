//! Backward-compatible stub: nt_core_embed types

pub const EMBEDDING_DIM: usize = 4096;

pub struct TextEmbedder;

impl TextEmbedder {
    pub fn new() -> Self {
        Self
    }

    pub fn embed(&self, _text: &str) -> Vec<f64> {
        vec![0.0; EMBEDDING_DIM]
    }
}

impl Default for TextEmbedder {
    fn default() -> Self {
        Self
    }
}
