//! Visual KB — semantic image search over KnowledgeBase screenshots
//! Absorption from PixelRAG (6.8K stars): visual content indexing + retrieval

#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct VisualEmbedding {
    pub dimensions: usize,
    pub vector: Vec<f32>,
    pub source_node_id: String,
    pub timestamp: i64,
}

#[derive(Clone, Debug)]
pub struct VisualSearchResult {
    pub node_id: String,
    pub similarity: f64,
    pub thumbnail_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct VisualRagConfig {
    pub embedding_dim: usize,
    pub top_k: usize,
    pub similarity_threshold: f64,
}

impl Default for VisualRagConfig {
    fn default() -> Self {
        Self {
            embedding_dim: 512,
            top_k: 10,
            similarity_threshold: 0.6,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VisualRagIndex {
    pub config: VisualRagConfig,
    pub embeddings: Vec<VisualEmbedding>,
    pub label_map: HashMap<String, String>,
}

impl VisualRagIndex {
    pub fn new(config: VisualRagConfig) -> Self {
        Self {
            config,
            embeddings: Vec::new(),
            label_map: HashMap::new(),
        }
    }

    pub fn index_screenshot(
        &mut self,
        node_id: &str,
        embedding: Vec<f32>,
        timestamp: i64,
    ) {
        let dimensions = embedding.len();
        self.embeddings.push(VisualEmbedding {
            dimensions,
            vector: embedding,
            source_node_id: node_id.to_string(),
            timestamp,
        });
        self.label_map
            .entry(node_id.to_string())
            .or_insert_with(|| node_id.to_string());
    }

    pub fn search(&self, query_embedding: &[f32]) -> Vec<VisualSearchResult> {
        let mut results: Vec<VisualSearchResult> = self
            .embeddings
            .iter()
            .map(|emb| {
                let sim = cosine_similarity(query_embedding, &emb.vector);
                VisualSearchResult {
                    node_id: emb.source_node_id.clone(),
                    similarity: sim.max(0.0).min(1.0),
                    thumbnail_path: None,
                }
            })
            .filter(|r| r.similarity >= self.config.similarity_threshold)
            .collect();

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(self.config.top_k);
        results
    }

    pub fn remove(&mut self, node_id: &str) {
        self.embeddings.retain(|e| e.source_node_id != node_id);
        self.label_map.remove(node_id);
    }

    pub fn size(&self) -> usize {
        self.embeddings.len()
    }

    pub fn nearest_centroid(&self) -> Option<Vec<f32>> {
        if self.embeddings.is_empty() {
            return None;
        }
        let dim = self.embeddings[0].dimensions;
        let mut centroid = vec![0.0_f32; dim];
        for emb in &self.embeddings {
            for (i, &val) in emb.vector.iter().enumerate() {
                centroid[i] += val;
            }
        }
        let count = self.embeddings.len() as f32;
        for val in centroid.iter_mut() {
            *val /= count;
        }
        l2_normalize(&mut centroid);
        Some(centroid)
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)) as f64
}

pub fn l2_normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in v.iter_mut() {
            *val /= norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_embedding(v: Vec<f32>) -> Vec<f32> {
        let mut e = v;
        l2_normalize(&mut e);
        e
    }

    #[test]
    fn test_new_index_is_empty() {
        let idx = VisualRagIndex::new(VisualRagConfig::default());
        assert_eq!(idx.size(), 0);
    }

    #[test]
    fn test_index_screenshot_adds_embedding() {
        let mut idx = VisualRagIndex::new(VisualRagConfig::default());
        idx.index_screenshot("node_1", make_embedding(vec![1.0, 0.0, 0.0]), 100);
        assert_eq!(idx.size(), 1);
    }

    #[test]
    fn test_index_screenshot_creates_label() {
        let mut idx = VisualRagIndex::new(VisualRagConfig::default());
        idx.index_screenshot("node_a", make_embedding(vec![0.0, 1.0, 0.0]), 200);
        assert_eq!(idx.label_map.get("node_a").unwrap(), "node_a");
    }

    #[test]
    fn test_search_returns_most_similar() {
        let mut idx = VisualRagIndex::new(VisualRagConfig::default());
        idx.index_screenshot("cat", make_embedding(vec![1.0, 0.0, 0.0]), 0);
        idx.index_screenshot("dog", make_embedding(vec![0.0, 1.0, 0.0]), 0);
        idx.index_screenshot("bird", make_embedding(vec![0.0, 0.0, 1.0]), 0);

        let results = idx.search(&make_embedding(vec![0.9, 0.1, 0.0]));
        assert!(!results.is_empty());
        assert_eq!(results[0].node_id, "cat");
    }

    #[test]
    fn test_search_respects_top_k() {
        let mut idx = VisualRagIndex::new(VisualRagConfig {
            top_k: 2,
            ..VisualRagConfig::default()
        });
        for i in 0..5 {
            let v = vec![i as f32; 3];
            idx.index_screenshot(&format!("n{}", i), make_embedding(v), 0);
        }
        let results = idx.search(&make_embedding(vec![1.0, 1.0, 1.0]));
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_search_filters_by_threshold() {
        let mut idx = VisualRagIndex::new(VisualRagConfig {
            similarity_threshold: 0.99,
            ..VisualRagConfig::default()
        });
        idx.index_screenshot("close", make_embedding(vec![1.0, 0.0, 0.0]), 0);
        idx.index_screenshot("far", make_embedding(vec![0.0, 1.0, 0.0]), 0);

        let results = idx.search(&make_embedding(vec![0.0, 1.0, 0.0]));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].node_id, "far");
    }

    #[test]
    fn test_remove_deletes_embedding_and_label() {
        let mut idx = VisualRagIndex::new(VisualRagConfig::default());
        idx.index_screenshot("del", make_embedding(vec![1.0, 0.0, 0.0]), 0);
        idx.remove("del");
        assert_eq!(idx.size(), 0);
        assert!(idx.label_map.get("del").is_none());
    }

    #[test]
    fn test_remove_nonexistent_is_noop() {
        let mut idx = VisualRagIndex::new(VisualRagConfig::default());
        idx.index_screenshot("a", make_embedding(vec![1.0, 0.0, 0.0]), 0);
        idx.remove("nonexistent");
        assert_eq!(idx.size(), 1);
    }

    #[test]
    fn test_centroid_with_empty_index() {
        let idx = VisualRagIndex::new(VisualRagConfig::default());
        assert!(idx.nearest_centroid().is_none());
    }

    #[test]
    fn test_centroid_with_single_embedding() {
        let mut idx = VisualRagIndex::new(VisualRagConfig::default());
        let v = make_embedding(vec![3.0, 4.0, 0.0]);
        idx.index_screenshot("x", v.clone(), 0);
        let centroid = idx.nearest_centroid().unwrap();
        assert_eq!(centroid.len(), 3);
        let norm: f32 = centroid.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_centroid_with_multiple_embeddings() {
        let mut idx = VisualRagIndex::new(VisualRagConfig::default());
        idx.index_screenshot("a", make_embedding(vec![1.0, 0.0, 0.0]), 0);
        idx.index_screenshot("b", make_embedding(vec![0.0, 1.0, 0.0]), 0);
        idx.index_screenshot("c", make_embedding(vec![0.0, 0.0, 1.0]), 0);
        let centroid = idx.nearest_centroid().unwrap();
        assert_eq!(centroid.len(), 3);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let sim = cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]);
        assert!((sim - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_cosine_similarity_dim_mismatch() {
        let sim = cosine_similarity(&[1.0, 0.0], &[1.0]);
        assert!((sim - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let sim = cosine_similarity(&[], &[]);
        assert!((sim - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_l2_normalize_zero_vector() {
        let mut v = vec![0.0, 0.0, 0.0];
        l2_normalize(&mut v);
        assert_eq!(v, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_l2_normalize_nonzero() {
        let mut v = vec![3.0, 4.0, 0.0];
        l2_normalize(&mut v);
        assert!((v[0] - 0.6).abs() < 1e-5);
        assert!((v[1] - 0.8).abs() < 1e-5);
    }
}
