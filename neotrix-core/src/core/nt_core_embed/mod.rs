//! Backward-compatible stub: nt_core_embed types

pub const EMBEDDING_DIM: usize = 4096;

pub struct TextEmbedder;

impl TextEmbedder {
    pub fn new() -> Self {
        Self
    }

    pub fn embed(&self, text: &str) -> Vec<f64> {
        // Deterministic projection: hash-based embedding
        let mut vec = vec![0.0f64; EMBEDDING_DIM];
        let bytes = text.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            vec[i % EMBEDDING_DIM] += (b as f64) / 255.0;
        }
        vec
    }

    pub fn similarity(&self, a: &str, b: &str) -> f64 {
        let va = self.embed(a);
        let vb = self.embed(b);
        let dot: f64 = va.iter().zip(vb.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = va.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
        let nb: f64 = vb.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
        dot / (na * nb)
    }

    pub fn find_most_similar<'a>(&self, query: &str, candidates: &[&'a str]) -> Option<(usize, f64, &'a str)> {
        candidates
            .iter()
            .enumerate()
            .map(|(i, &c)| (i, self.similarity(query, c), c))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn vocab_size(&self) -> usize {
        EMBEDDING_DIM
    }
}

impl Default for TextEmbedder {
    fn default() -> Self {
        Self
    }
}
