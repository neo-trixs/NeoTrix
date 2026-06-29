use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::collections::{HashMap, HashSet};

// ── BM25 Configuration ──

#[derive(Debug, Clone)]
pub struct BM25Config {
    pub k1: f64,
    pub b: f64,
    pub epsilon: f64,
}

impl Default for BM25Config {
    fn default() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            epsilon: 0.25,
        }
    }
}

// ── BM25 Document ──

#[derive(Debug, Clone)]
pub struct Bm25Document {
    pub id: String,
    pub text: String,
    pub terms: Vec<String>,
}

// ── BM25 Index ──

#[derive(Clone)]
pub struct BM25Index {
    pub docs: Vec<Bm25Document>,
    pub avgdl: f64,
    pub idf: HashMap<String, f64>,
    pub term_freqs: Vec<HashMap<String, usize>>,
    pub config: BM25Config,
    total_terms: usize,
}

impl BM25Index {
    pub fn new(config: BM25Config) -> Self {
        Self {
            docs: Vec::new(),
            avgdl: 0.0,
            idf: HashMap::new(),
            term_freqs: Vec::new(),
            config,
            total_terms: 0,
        }
    }

    pub fn add_document(&mut self, id: String, text: String) {
        let terms = Self::tokenize(&text);
        let mut tf = HashMap::new();
        for t in &terms {
            *tf.entry(t.clone()).or_insert(0) += 1;
        }
        self.total_terms += terms.len();
        self.docs.push(Bm25Document { id, text, terms });
        self.term_freqs.push(tf);
    }

    pub fn rebuild(&mut self) {
        let n = self.docs.len() as f64;
        if n == 0.0 {
            self.avgdl = 0.0;
            self.idf.clear();
            return;
        }
        self.avgdl = self.total_terms as f64 / n;

        let mut df: HashMap<String, usize> = HashMap::new();
        for tf in &self.term_freqs {
            for term in tf.keys() {
                *df.entry(term.clone()).or_insert(0) += 1;
            }
        }

        self.idf.clear();
        for (term, doc_freq) in &df {
            let idf_val = Self::compute_idf_rs(*doc_freq as f64, n, self.config.epsilon);
            self.idf.insert(term.clone(), idf_val);
        }
    }

    fn compute_idf_rs(doc_freq: f64, n: f64, epsilon: f64) -> f64 {
        let idf_val = ((n - doc_freq + 0.5) / (doc_freq + 0.5) + epsilon).ln();
        idf_val.max(0.0)
    }

    pub fn search(&self, query: &str, top_k: usize) -> Vec<(String, f64)> {
        if self.docs.is_empty() {
            return Vec::new();
        }
        let query_terms = Self::tokenize(query);
        if query_terms.is_empty() {
            return Vec::new();
        }

        let n = self.docs.len() as f64;
        let mut scores: Vec<(usize, f64)> = Vec::new();
        for (i, doc) in self.docs.iter().enumerate() {
            let mut score = 0.0;
            for qt in &query_terms {
                let idf_val = self
                    .idf
                    .get(qt)
                    .copied()
                    .unwrap_or_else(|| Self::compute_idf_rs(0.0, n, self.config.epsilon));
                let tf = self.term_freqs[i].get(qt).copied().unwrap_or(0) as f64;
                let dl = doc.terms.len() as f64;
                let avgdl = self.avgdl.max(1.0);
                let denom =
                    tf + self.config.k1 * (1.0 - self.config.b + self.config.b * dl / avgdl);
                if denom > 0.0 {
                    score += idf_val * (tf * (self.config.k1 + 1.0)) / denom;
                }
            }
            if score > 0.0 {
                scores.push((i, score));
            }
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);
        scores
            .into_iter()
            .map(|(i, s)| (self.docs[i].id.clone(), s))
            .collect()
    }

    pub fn tokenize(text: &str) -> Vec<String> {
        let stop_words: HashSet<&str> = [
            "the", "a", "an", "this", "that", "to", "for", "of", "in", "on", "with", "and", "or",
            "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "may", "might", "i", "you", "he", "she",
            "it", "we", "they", "me", "my", "your", "his", "her", "its", "our", "their", "can",
            "need", "want", "help", "make", "do", "get", "use", "create", "build", "find", "tell",
            "show", "give", "take",
        ]
        .iter()
        .copied()
        .collect();

        let lower = text.to_lowercase();
        lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() >= 2 && !stop_words.contains(*s))
            .map(String::from)
            .collect()
    }
}

// ── Search Result ──

#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub doc_id: String,
    pub content: String,
    pub score: f64,
    pub source: RetrievalSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RetrievalSource {
    BM25,
    VSA,
    Graph,
    EntityMatch,
    Hybrid,
}

// ── Retrieval Stats ──

#[derive(Debug, Clone)]
pub struct RetrievalStats {
    pub bm25_docs: usize,
    pub vsa_vectors: usize,
    pub graph_edges: usize,
    pub graph_nodes: usize,
}

// ── VSA Encoding ──

pub fn encode_query_to_vsa(query: &str) -> Vec<u8> {
    let seed: u64 = query
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    QuantizedVSA::seeded_random(seed, VSA_DIM)
}

// ── Hybrid Retrieval Engine ──

#[derive(Clone)]
pub struct HybridRetrievalEngine {
    pub bm25: BM25Index,
    pub vsa_vectors: HashMap<String, Vec<u8>>,
    pub vsa_texts: HashMap<String, String>,
    pub graph_edges: HashMap<String, Vec<(String, f64, String)>>,
    pub k_rrf: f64,
    pub vsa_similarity_threshold: f64,
    /// Entity matching signal weight (for multi-signal fusion)
    pub entity_weight: f64,
    /// Learned fusion weights: [semantic, keyword, entity]
    pub fusion_weights: [f64; 3],
}

impl Default for HybridRetrievalEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridRetrievalEngine {
    pub fn new() -> Self {
        Self {
            bm25: BM25Index::new(BM25Config::default()),
            vsa_vectors: HashMap::new(),
            vsa_texts: HashMap::new(),
            graph_edges: HashMap::new(),
            k_rrf: 60.0,
            vsa_similarity_threshold: 0.6,
            entity_weight: 1.0,
            fusion_weights: [1.0, 1.0, 1.0],
        }
    }

    pub fn index_text(&mut self, id: String, text: String) {
        let vsa = encode_query_to_vsa(&text);
        self.bm25.add_document(id.clone(), text.clone());
        self.vsa_vectors.insert(id.clone(), vsa);
        self.vsa_texts.insert(id.clone(), text);
    }

    pub fn index_vsa(&mut self, id: String, vector: Vec<u8>) {
        self.vsa_vectors.insert(id, vector);
    }

    pub fn index_text_with_vsa(&mut self, id: String, text: String, vector: Vec<u8>) {
        self.bm25.add_document(id.clone(), text.clone());
        self.vsa_vectors.insert(id.clone(), vector);
        self.vsa_texts.insert(id.clone(), text);
    }

    pub fn add_graph_edge(&mut self, from: String, to: String, weight: f64, relation: String) {
        self.graph_edges
            .entry(from)
            .or_default()
            .push((to, weight, relation));
    }

    pub fn rebuild_bm25(&mut self) {
        self.bm25.rebuild();
    }

    pub fn search_bm25(&self, query: &str, top_k: usize) -> Vec<RetrievalResult> {
        let results = self.bm25.search(query, top_k);
        results
            .into_iter()
            .map(|(id, score)| RetrievalResult {
                doc_id: id.clone(),
                content: self.vsa_texts.get(&id).cloned().unwrap_or_default(),
                score,
                source: RetrievalSource::BM25,
            })
            .collect()
    }

    pub fn search_vsa(&self, query: &str, top_k: usize) -> Vec<RetrievalResult> {
        if self.vsa_vectors.is_empty() {
            return Vec::new();
        }
        let query_vsa = encode_query_to_vsa(query);
        let mut scores: Vec<(&String, f64)> = Vec::new();
        for (id, vec) in &self.vsa_vectors {
            let sim = QuantizedVSA::cosine(&query_vsa, vec);
            if sim >= self.vsa_similarity_threshold {
                scores.push((id, sim));
            }
        }
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);
        scores
            .into_iter()
            .map(|(id, score)| {
                let content = self.vsa_texts.get(id).cloned().unwrap_or_default();
                RetrievalResult {
                    doc_id: id.clone(),
                    content,
                    score,
                    source: RetrievalSource::VSA,
                }
            })
            .collect()
    }

    /// Entity matching search signal.
    /// Extracts entities from the query, then finds entries mentioning those entities.
    pub fn search_entity(&self, query: &str, top_k: usize) -> Vec<RetrievalResult> {
        if top_k == 0 {
            return Vec::new();
        }
        // Simple entity extraction: capitalized word sequences
        let query_entities: Vec<String> = query
            .split_whitespace()
            .filter(|w| w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && w.len() > 1)
            .map(|w| w.to_lowercase())
            .collect();

        if query_entities.is_empty() {
            return Vec::new();
        }

        // Score documents by entity overlap
        let mut scored: Vec<(String, f64)> = Vec::new();
        for (id, doc) in &self.vsa_texts {
            let doc_lower = doc.to_lowercase();
            let match_count = query_entities
                .iter()
                .filter(|qe| doc_lower.contains(qe.as_str()))
                .count() as f64;
            if match_count > 0.0 {
                let score = match_count / query_entities.len() as f64;
                scored.push((id.clone(), score));
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
            .into_iter()
            .map(|(id, score)| {
                let doc_id = id.clone();
                RetrievalResult {
                    doc_id,
                    content: self.vsa_texts.get(&id).cloned().unwrap_or_default(),
                    score,
                    source: RetrievalSource::EntityMatch,
                }
            })
            .collect()
    }

    pub fn search_graph(&self, query: &str, top_k: usize) -> Vec<RetrievalResult> {
        if self.graph_edges.is_empty() {
            return Vec::new();
        }
        let query_terms: HashSet<String> = BM25Index::tokenize(query).into_iter().collect();
        if query_terms.is_empty() {
            return Vec::new();
        }

        let mut node_scores: HashMap<String, f64> = HashMap::new();

        for (node, edges) in &self.graph_edges {
            let node_terms: HashSet<String> = BM25Index::tokenize(node).into_iter().collect();
            let overlap = query_terms.intersection(&node_terms).count();
            if overlap > 0 {
                let direct_score = overlap as f64 / query_terms.len() as f64;
                node_scores.entry(node.clone()).or_insert(direct_score);

                for (neighbor, weight, _relation) in edges {
                    let neighbor_score = direct_score * weight * 0.5;
                    let entry = node_scores.entry(neighbor.clone()).or_insert(0.0);
                    if neighbor_score > *entry {
                        *entry = neighbor_score;
                    }
                }
            }
        }

        let mut results: Vec<RetrievalResult> = node_scores
            .into_iter()
            .filter(|(_, s)| *s > 0.0)
            .map(|(id, score)| RetrievalResult {
                doc_id: id.clone(),
                content: self.vsa_texts.get(&id).cloned().unwrap_or_default(),
                score,
                source: RetrievalSource::Graph,
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(top_k);
        results
    }

    pub fn search_hybrid(&self, query: &str, top_k: usize) -> Vec<RetrievalResult> {
        self.search_hybrid_with_weights(query, top_k, [1.0, 1.0, 1.0])
    }

    pub fn search_hybrid_with_weights(
        &self,
        query: &str,
        top_k: usize,
        weights: [f64; 3],
    ) -> Vec<RetrievalResult> {
        let entity_results = self.search_entity(query, top_k * 2);
        let rankings = vec![
            self.search_bm25(query, top_k * 2),
            self.search_vsa(query, top_k * 2),
            self.search_graph(query, top_k * 2),
            entity_results,
        ];
        let signal_weights: [f64; 4] = [weights[0], weights[1], weights[2], self.entity_weight];
        let mut fused = fuse_rrf(&rankings, self.k_rrf, &signal_weights);
        fused.truncate(top_k);
        for r in &mut fused {
            r.source = RetrievalSource::Hybrid;
        }
        fused
    }

    /// Update fusion weights based on feedback signal.
    /// Simple delta rule:  w_i += lr * error * result_i.count
    pub fn update_fusion_weights(&mut self, query: &str, feedback_score: f64, lr: f64) {
        let top_k = 5;
        let bm25 = self.search_bm25(query, top_k);
        let vsa = self.search_vsa(query, top_k);
        let graph = self.search_graph(query, top_k);
        let entity = self.search_entity(query, top_k);

        let counts = [
            bm25.len() as f64,
            vsa.len() as f64,
            graph.len() as f64,
            entity.len() as f64,
        ];
        let total: f64 = counts.iter().sum();
        if total == 0.0 {
            return;
        }

        let avg_count = total / 4.0;
        let mut new_weights = self.fusion_weights;
        for i in 0..3 {
            let error = (counts[i] - avg_count) / avg_count.max(1.0);
            new_weights[i] = (new_weights[i] + lr * error * feedback_score).clamp(0.1, 2.0);
        }
        self.entity_weight = (self.entity_weight
            + lr * (counts[3] - avg_count) / avg_count.max(1.0) * feedback_score)
            .clamp(0.1, 2.0);

        self.fusion_weights = new_weights;
    }

    pub fn stats(&self) -> RetrievalStats {
        RetrievalStats {
            bm25_docs: self.bm25.docs.len(),
            vsa_vectors: self.vsa_vectors.len(),
            graph_edges: self.graph_edges.values().map(|e| e.len()).sum(),
            graph_nodes: self.graph_edges.len(),
        }
    }

    pub fn search(
        &self,
        query: &str,
        top_k: usize,
        source: RetrievalSource,
    ) -> Vec<RetrievalResult> {
        match source {
            RetrievalSource::BM25 => self.search_bm25(query, top_k),
            RetrievalSource::VSA => self.search_vsa(query, top_k),
            RetrievalSource::Graph => self.search_graph(query, top_k),
            RetrievalSource::EntityMatch => self.search_entity(query, top_k),
            RetrievalSource::Hybrid => self.search_hybrid(query, top_k),
        }
    }
}

// ── RRF Fusion ──

pub fn fuse_rrf(
    rankings: &[Vec<RetrievalResult>],
    k: f64,
    weights: &[f64],
) -> Vec<RetrievalResult> {
    if rankings.is_empty() {
        return Vec::new();
    }
    let has_nonempty = rankings.iter().any(|r| !r.is_empty());
    if !has_nonempty {
        return Vec::new();
    }

    let mut rrf_scores: HashMap<String, (f64, String, f64, Vec<RetrievalSource>)> = HashMap::new();

    for (retriever_idx, ranking) in rankings.iter().enumerate() {
        let weight = weights.get(retriever_idx).copied().unwrap_or(1.0);
        for (rank, result) in ranking.iter().enumerate() {
            let rrf_contrib = weight / (k + (rank + 1) as f64);
            let entry = rrf_scores
                .entry(result.doc_id.clone())
                .or_insert_with(|| (0.0, result.content.clone(), 0.0, Vec::new()));
            entry.0 += rrf_contrib;
            entry.2 = entry.2.max(result.score);
            entry.3.push(result.source);
        }
    }

    let mut fused: Vec<RetrievalResult> = rrf_scores
        .into_iter()
        .map(|(doc_id, (rrf_score, content, _max_score, sources))| {
            let boost = if sources.len() > 1 {
                1.0 + 0.1 * (sources.len() as f64 - 1.0)
            } else {
                1.0
            };
            RetrievalResult {
                doc_id,
                content,
                score: rrf_score * boost,
                source: RetrievalSource::Hybrid,
            }
        })
        .collect();

    fused.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let max_score = fused.first().map(|r| r.score).unwrap_or(1.0);
    if max_score > 0.0 {
        for r in &mut fused {
            r.score /= max_score;
        }
    }

    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_engine() -> HybridRetrievalEngine {
        let mut engine = HybridRetrievalEngine::new();
        engine.index_text(
            "doc1".into(),
            "the quick brown fox jumps over the lazy dog".into(),
        );
        engine.index_text(
            "doc2".into(),
            "machine learning is transforming artificial intelligence research".into(),
        );
        engine.index_text(
            "doc3".into(),
            "the brown bear lives in the forest and eats berries".into(),
        );
        engine.index_text(
            "doc4".into(),
            "quantum computing uses qubits for parallel computation".into(),
        );
        engine.index_text(
            "doc5".into(),
            "deep neural networks learn hierarchical representations".into(),
        );
        engine.rebuild_bm25();
        engine.add_graph_edge("fox".into(), "dog".into(), 0.9, "related_to".into());
        engine.add_graph_edge("brown".into(), "bear".into(), 0.8, "describes".into());
        engine.add_graph_edge("machine".into(), "learning".into(), 1.0, "part_of".into());
        engine.add_graph_edge("quantum".into(), "qubits".into(), 1.0, "uses".into());
        engine
    }

    // ── BM25 Tests ──

    #[test]
    fn test_bm25_tokenize_removes_stop_words() {
        let tokens = BM25Index::tokenize("the quick brown fox and the lazy dog");
        assert!(!tokens.contains(&"the".to_string()));
        assert!(!tokens.contains(&"and".to_string()));
        assert!(tokens.contains(&"quick".to_string()));
        assert!(tokens.contains(&"brown".to_string()));
    }

    #[test]
    fn test_bm25_tokenize_lowercases() {
        let tokens = BM25Index::tokenize("Hello World TEST");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }

    #[test]
    fn test_bm25_add_document_and_rebuild() {
        let mut bm25 = BM25Index::new(BM25Config::default());
        bm25.add_document("d1".into(), "hello world".into());
        bm25.add_document("d2".into(), "hello there world".into());
        bm25.rebuild();
        assert_eq!(bm25.docs.len(), 2);
        assert!(bm25.avgdl > 0.0);
        assert!(bm25.idf.contains_key("hello"));
    }

    #[test]
    fn test_bm25_search_returns_correct_top_doc() {
        let mut bm25 = BM25Index::new(BM25Config::default());
        bm25.add_document("d1".into(), "the quick brown fox".into());
        bm25.add_document("d2".into(), "machine learning deep learning".into());
        bm25.add_document("d3".into(), "the lazy dog sleeps".into());
        bm25.rebuild();
        let results = bm25.search("brown fox", 3);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "d1");
    }

    #[test]
    fn test_bm25_no_matching_terms_returns_empty() {
        let mut bm25 = BM25Index::new(BM25Config::default());
        bm25.add_document("d1".into(), "aaa bbb ccc".into());
        bm25.rebuild();
        let results = bm25.search("zzz yyy", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_bm25_empty_index_returns_empty() {
        let bm25 = BM25Index::new(BM25Config::default());
        let results = bm25.search("anything", 5);
        assert!(results.is_empty());
    }

    // ── VSA Tests ──

    #[test]
    fn test_encode_query_to_vsa_deterministic() {
        let v1 = encode_query_to_vsa("hello world");
        let v2 = encode_query_to_vsa("hello world");
        assert_eq!(v1, v2);
        assert_eq!(v1.len(), VSA_DIM);
    }

    #[test]
    fn test_encode_query_to_vsa_different_queries_differ() {
        let v1 = encode_query_to_vsa("hello");
        let v2 = encode_query_to_vsa("world");
        let sim = QuantizedVSA::cosine(&v1, &v2);
        assert!(sim < 0.9);
    }

    #[test]
    fn test_vsa_search_returns_similar_docs() {
        let engine = setup_engine();
        let results = engine.search_vsa("brown fox", 3);
        assert!(!results.is_empty());
        for r in &results {
            assert!(r.score >= 0.0);
            assert_eq!(r.source, RetrievalSource::VSA);
        }
    }

    #[test]
    fn test_vsa_search_empty_index_returns_empty() {
        let engine = HybridRetrievalEngine::new();
        let results = engine.search_vsa("anything", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_vsa_threshold_filters_low_similarity() {
        let mut engine = HybridRetrievalEngine::new();
        engine.index_text("doc1".into(), "quantum physics entanglement".into());
        engine.vsa_similarity_threshold = 0.99;
        let results = engine.search_vsa("quantum", 5);
        assert!(results.is_empty() || results.iter().all(|r| r.score >= 0.99));
    }

    // ── Graph Tests ──

    #[test]
    fn test_graph_search_returns_one_hop_neighbors() {
        let engine = setup_engine();
        let results = engine.search_graph("brown", 5);
        assert!(!results.is_empty());
        let ids: Vec<&str> = results.iter().map(|r| r.doc_id.as_str()).collect();
        assert!(ids.contains(&"brown") || ids.contains(&"bear"));
    }

    #[test]
    fn test_graph_search_empty_edges_returns_empty() {
        let engine = HybridRetrievalEngine::new();
        let results = engine.search_graph("anything", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_graph_search_no_match_returns_empty() {
        let mut engine = HybridRetrievalEngine::new();
        engine.add_graph_edge("cat".into(), "dog".into(), 1.0, "related".into());
        let results = engine.search_graph("quantum", 5);
        assert!(results.is_empty());
    }

    // ── RRF Fusion Tests ──

    #[test]
    fn test_rrf_fusion_with_three_identical_rankings() {
        let results = vec![
            RetrievalResult {
                doc_id: "a".into(),
                content: "".into(),
                score: 10.0,
                source: RetrievalSource::BM25,
            },
            RetrievalResult {
                doc_id: "b".into(),
                content: "".into(),
                score: 5.0,
                source: RetrievalSource::BM25,
            },
            RetrievalResult {
                doc_id: "c".into(),
                content: "".into(),
                score: 1.0,
                source: RetrievalSource::BM25,
            },
        ];
        let rankings = vec![results.clone(), results.clone(), results.clone()];
        let fused = fuse_rrf(&rankings, 60.0, &[1.0, 1.0, 1.0]);
        assert!(!fused.is_empty());
        assert_eq!(fused[0].doc_id, "a");
        assert_eq!(fused[1].doc_id, "b");
        assert_eq!(fused[2].doc_id, "c");
    }

    #[test]
    fn test_rrf_fusion_with_different_rankings() {
        let bm25 = vec![
            RetrievalResult {
                doc_id: "a".into(),
                content: "".into(),
                score: 10.0,
                source: RetrievalSource::BM25,
            },
            RetrievalResult {
                doc_id: "b".into(),
                content: "".into(),
                score: 5.0,
                source: RetrievalSource::BM25,
            },
        ];
        let vsa = vec![
            RetrievalResult {
                doc_id: "c".into(),
                content: "".into(),
                score: 0.9,
                source: RetrievalSource::VSA,
            },
            RetrievalResult {
                doc_id: "a".into(),
                content: "".into(),
                score: 0.8,
                source: RetrievalSource::VSA,
            },
            RetrievalResult {
                doc_id: "b".into(),
                content: "".into(),
                score: 0.7,
                source: RetrievalSource::VSA,
            },
        ];
        let graph = vec![RetrievalResult {
            doc_id: "a".into(),
            content: "".into(),
            score: 1.0,
            source: RetrievalSource::Graph,
        }];
        let rankings: Vec<Vec<RetrievalResult>> = vec![bm25, vsa, graph];
        let fused = fuse_rrf(&rankings, 60.0, &[1.0, 1.0, 1.0]);
        assert!(!fused.is_empty());
        assert_eq!(fused[0].doc_id, "a");
    }

    #[test]
    fn test_weighted_rrf() {
        let r1 = vec![RetrievalResult {
            doc_id: "a".into(),
            content: "".into(),
            score: 1.0,
            source: RetrievalSource::BM25,
        }];
        let r2 = vec![RetrievalResult {
            doc_id: "b".into(),
            content: "".into(),
            score: 1.0,
            source: RetrievalSource::VSA,
        }];
        let rankings = vec![r1, r2];
        let fused = fuse_rrf(&rankings, 60.0, &[2.0, 0.5]);
        assert!(!fused.is_empty());
        let rrf_a = fused
            .iter()
            .find(|r| r.doc_id == "a")
            .map(|r| r.score)
            .unwrap_or(0.0);
        let rrf_b = fused
            .iter()
            .find(|r| r.doc_id == "b")
            .map(|r| r.score)
            .unwrap_or(0.0);
        assert!(rrf_a > rrf_b);
    }

    #[test]
    fn test_rrf_empty_rankings() {
        let fused = fuse_rrf(&[], 60.0, &[]);
        assert!(fused.is_empty());
    }

    // ── Hybrid Search Tests ──

    #[test]
    fn test_hybrid_search_combines_all_three_retrievers() {
        let engine = setup_engine();
        let results = engine.search_hybrid("brown fox machine", 5);
        assert!(!results.is_empty());
        assert!(results.len() <= 5);
        for r in &results {
            assert!(r.score >= 0.0 && r.score <= 1.0);
            assert_eq!(r.source, RetrievalSource::Hybrid);
        }
    }

    #[test]
    fn test_hybrid_search_empty_engine_returns_empty() {
        let engine = HybridRetrievalEngine::new();
        let results = engine.search_hybrid("anything", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_single_document_search() {
        let mut engine = HybridRetrievalEngine::new();
        engine.index_text("only".into(), "singular document for testing".into());
        engine.rebuild_bm25();
        let bm25 = engine.search_bm25("singular", 5);
        assert_eq!(bm25.len(), 1);
        assert_eq!(bm25[0].doc_id, "only");
        let hybrid = engine.search_hybrid("singular", 5);
        assert!(!hybrid.is_empty());
    }

    #[test]
    fn test_rebuild_after_bulk_add() {
        let mut engine = HybridRetrievalEngine::new();
        for i in 0..10 {
            engine.index_text(
                format!("d{}", i),
                format!("document number {} about testing", i),
            );
        }
        engine.rebuild_bm25();
        let results = engine.search_bm25("testing", 5);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_stats_report_correct_counts() {
        let engine = setup_engine();
        let stats = engine.stats();
        assert_eq!(stats.bm25_docs, 5);
        assert_eq!(stats.vsa_vectors, 5);
        assert!(stats.graph_edges > 0);
        assert!(stats.graph_nodes > 0);
    }

    #[test]
    fn test_empty_query_returns_empty() {
        let engine = setup_engine();
        let bm25 = engine.search_bm25("", 5);
        assert!(bm25.is_empty());
        let vsa = engine.search_vsa("", 5);
        assert!(vsa.is_empty());
        let graph = engine.search_graph("", 5);
        assert!(graph.is_empty());
    }

    #[test]
    fn test_single_term_query() {
        let engine = setup_engine();
        let results = engine.search_hybrid("brown", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_long_text_query_does_not_panic() {
        let engine = setup_engine();
        let long = "a ".repeat(500) + "brown fox";
        let results = engine.search_hybrid(&long, 3);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_graph_only_direct_match_with_no_edges() {
        let mut engine = HybridRetrievalEngine::new();
        engine.add_graph_edge("cat".into(), "dog".into(), 1.0, "related".into());
        let results = engine.search_graph("cat", 5);
        assert!(!results.is_empty());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_source_dispatches_correctly() {
        let engine = setup_engine();
        let bm25 = engine.search("brown", 3, RetrievalSource::BM25);
        assert!(bm25.iter().all(|r| r.source == RetrievalSource::BM25));
        let vsa = engine.search("brown", 3, RetrievalSource::VSA);
        assert!(vsa.iter().all(|r| r.source == RetrievalSource::VSA));
        let graph = engine.search("brown", 3, RetrievalSource::Graph);
        assert!(graph.iter().all(|r| r.source == RetrievalSource::Graph));
        let hybrid = engine.search("brown", 3, RetrievalSource::Hybrid);
        assert!(hybrid.iter().all(|r| r.source == RetrievalSource::Hybrid));
    }

    #[test]
    fn test_bm25_bulk_insert_many_docs() {
        let mut bm25 = BM25Index::new(BM25Config::default());
        for i in 0..100 {
            bm25.add_document(
                format!("d{}", i),
                format!(
                    "document number {} with searchable content about indexing",
                    i
                ),
            );
        }
        bm25.rebuild();
        let results = bm25.search("indexing", 10);
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_long_text_does_not_overflow() {
        let mut engine = HybridRetrievalEngine::new();
        let long_text = "word ".repeat(1000) + "unique_term_xyz";
        engine.index_text("long".into(), long_text);
        engine.rebuild_bm25();
        let results = engine.search_bm25("unique_term_xyz", 5);
        assert_eq!(results.len(), 1);
    }
}
