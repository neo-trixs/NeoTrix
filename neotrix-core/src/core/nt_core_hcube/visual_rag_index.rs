use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexBackend {
    FAISSIVFFlat,
    NTSSEGVSA,
    ColPali,
    Hybrid,
}

impl IndexBackend {
    pub fn name(&self) -> &'static str {
        match self {
            IndexBackend::FAISSIVFFlat => "faiss_ivf_flat",
            IndexBackend::NTSSEGVSA => "ntsseg_vsa",
            IndexBackend::ColPali => "colpali",
            IndexBackend::Hybrid => "hybrid",
        }
    }
}

#[derive(Debug, Clone)]
pub struct VisualIndexConfig {
    pub backend: IndexBackend,
    pub nlist: u32,
    pub nprobe: u32,
    pub embedding_dim: usize,
    pub normalize: bool,
    pub use_gpu: bool,
}

impl Default for VisualIndexConfig {
    fn default() -> Self {
        Self {
            backend: IndexBackend::FAISSIVFFlat,
            nlist: 4096,
            nprobe: 64,
            embedding_dim: 221,
            normalize: true,
            use_gpu: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexedDocument {
    pub article_id: String,
    pub tile_index: u32,
    pub chunk_index: u32,
    pub url: String,
    pub embedding: Vec<f32>,
    pub vsa_vector: Vec<u8>,
    pub metadata: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub article_id: String,
    pub tile_index: u32,
    pub chunk_index: u32,
    pub url: String,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct VisualRAGIndex {
    config: VisualIndexConfig,
    documents: Vec<IndexedDocument>,
}

impl VisualRAGIndex {
    pub fn new(config: VisualIndexConfig) -> Self {
        Self {
            config,
            documents: Vec::new(),
        }
    }

    pub fn add_document(&mut self, doc: IndexedDocument) {
        self.documents.push(doc);
    }

    pub fn add_documents(&mut self, docs: Vec<IndexedDocument>) {
        self.documents.extend(docs);
    }

    pub fn search(&self, query_embedding: &[f32], k: usize) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = self
            .documents
            .iter()
            .enumerate()
            .map(|(_, doc)| {
                let score = cosine_similarity(query_embedding, &doc.embedding);
                SearchResult {
                    article_id: doc.article_id.clone(),
                    tile_index: doc.tile_index,
                    chunk_index: doc.chunk_index,
                    url: doc.url.clone(),
                    score,
                }
            })
            .filter(|r| r.score > 0.3)
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(k);
        results
    }

    pub fn search_vsa(&self, query_vsa: &[u8], k: usize) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = self
            .documents
            .iter()
            .map(|doc| {
                let score = QuantizedVSA::similarity(query_vsa, &doc.vsa_vector);
                SearchResult {
                    article_id: doc.article_id.clone(),
                    tile_index: doc.tile_index,
                    chunk_index: doc.chunk_index,
                    url: doc.url.clone(),
                    score,
                }
            })
            .filter(|r| r.score > 0.3)
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(k);
        results
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    pub fn config(&self) -> &VisualIndexConfig {
        &self.config
    }

    pub fn deduplicate(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.documents.retain(|doc| {
            let key = format!("{}_{}_{}", doc.article_id, doc.tile_index, doc.chunk_index);
            seen.insert(key)
        });
    }

    pub fn clear(&mut self) {
        self.documents.clear();
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_embedding(dim: usize) -> Vec<f32> {
        (0..dim).map(|i| (i as f32).sin()).collect()
    }

    fn dummy_doc(id: &str, idx: u32) -> IndexedDocument {
        IndexedDocument {
            article_id: id.into(),
            tile_index: idx,
            chunk_index: 0,
            url: format!("https://example.com/{}", id),
            embedding: dummy_embedding(128),
            vsa_vector: QuantizedVSA::random_binary(),
            metadata: vec![("source".into(), "web".into())],
        }
    }

    #[test]
    fn test_add_and_search() {
        let mut index = VisualRAGIndex::new(VisualIndexConfig::default());
        index.add_document(dummy_doc("doc1", 0));
        index.add_document(dummy_doc("doc2", 1));
        assert_eq!(index.len(), 2);

        let results = index.search(&dummy_embedding(128), 5);
        assert!(!results.is_empty());
        assert!(results.len() <= 2);
        for r in &results {
            assert!((0.0..=1.0).contains(&r.score));
        }
    }

    #[test]
    fn test_search_vsa() {
        let mut index = VisualRAGIndex::new(VisualIndexConfig::default());
        index.add_document(dummy_doc("doc1", 0));
        index.add_document(dummy_doc("doc2", 1));

        let query = QuantizedVSA::random_binary();
        let results = index.search_vsa(&query, 5);
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_deduplicate() {
        let mut index = VisualRAGIndex::new(VisualIndexConfig::default());
        index.add_document(dummy_doc("doc1", 0));
        index.add_document(dummy_doc("doc1", 0));
        assert_eq!(index.len(), 2);
        index.deduplicate();
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_empty_index() {
        let index = VisualRAGIndex::new(VisualIndexConfig::default());
        assert!(index.is_empty());
        let results = index.search(&dummy_embedding(128), 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut index = VisualRAGIndex::new(VisualIndexConfig::default());
        index.add_document(dummy_doc("doc1", 0));
        index.clear();
        assert!(index.is_empty());
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_index_backend_names() {
        assert_eq!(IndexBackend::FAISSIVFFlat.name(), "faiss_ivf_flat");
        assert_eq!(IndexBackend::NTSSEGVSA.name(), "ntsseg_vsa");
        assert_eq!(IndexBackend::ColPali.name(), "colpali");
        assert_eq!(IndexBackend::Hybrid.name(), "hybrid");
    }
}
