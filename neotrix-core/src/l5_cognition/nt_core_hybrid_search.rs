//! # Hybrid Code Search Retriever
//!
//! Reverse-engineered from CodeGraph + Contextro + fog-context.
//! Combines BM25 lexical search + Vector semantic search with RRF fusion.
//!
//! # Architecture
//! ```text
//! Query → [BM25 Lexical] + [Vector Semantic]
//!      → RRF (Reciprocal Rank Fusion)
//!      → [Optional: Spreading Activation]
//!      → Token-aware Context Packing
//!      → Results
//! ```
//!
//! # Key Libraries
//! - `bm25` crate for BM25 scoring
//! - `blake3` for content hashing (incremental index)
//! - `tree-sitter` for AST parsing
//! - SQLite FTS5 for full-text search
//! - hnsw-rs or LanceDB for vector search

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::sync::{Arc, Mutex};

/// Search result with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Node/symbol ID
    pub node_id: u64,
    /// File path
    pub file_path: String,
    /// Symbol name
    pub name: String,
    /// Symbol kind (function, class, method, etc.)
    pub kind: String,
    /// Start line number
    pub start_line: u32,
    /// End line number
    pub end_line: u32,
    /// Combined relevance score (after RRF fusion)
    pub score: f32,
    /// BM25 score (if available)
    pub bm25_score: Option<f32>,
    /// Vector similarity score (if available)
    pub vector_score: Option<f32>,
    /// Code snippet
    pub snippet: Option<String>,
    /// Language
    pub language: String,
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// RRF constant (higher = less weight to rank, default 60)
    pub rrf_k: f32,
    /// Maximum results to return
    pub max_results: usize,
    /// BM25 weight in fusion (default 0.5)
    pub bm25_weight: f32,
    /// Vector weight in fusion (default 0.5)
    pub vector_weight: f32,
    /// Minimum score threshold
    pub min_score: f32,
    /// Token budget for context packing
    pub token_budget: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            rrf_k: 60.0,
            max_results: 20,
            bm25_weight: 0.5,
            vector_weight: 0.5,
            min_score: 0.01,
            token_budget: 8000,
        }
    }
}

/// BM25 document for indexing
#[derive(Debug, Clone)]
pub struct Bm25Document {
    pub node_id: u64,
    pub text: String,  // name + signature + body
}

/// Hybrid search retriever
pub struct HybridRetriever {
    config: SearchConfig,
    /// BM25 index (in-memory or SQLite FTS5)
    bm25_index: Arc<Mutex<Bm25Index>>,
    /// Vector index (hnsw-rs or LanceDB)
    vector_index: Arc<Mutex<VectorIndex>>,
    /// Node metadata cache
    nodes: Arc<Mutex<HashMap<u64, NodeMeta>>>,
}

struct Bm25Index {
    documents: Vec<Bm25Document>,
    avgdl: f32,
    idf: HashMap<String, f32>,
}

struct VectorIndex {
    /// Placeholder — in production use hnsw-rs or LanceDB
    embeddings: Vec<(u64, Vec<f32>)>,
}

#[derive(Debug, Clone)]
pub struct NodeMeta {
    pub node_id: u64,
    pub file_path: String,
    pub name: String,
    pub kind: String,
    pub start_line: u32,
    pub end_line: u32,
    pub language: String,
    pub body: String,
}

impl HybridRetriever {
    pub fn new(config: SearchConfig) -> Self {
        Self {
            config,
            bm25_index: Arc::new(Mutex::new(Bm25Index {
                documents: Vec::new(),
                avgdl: 0.0,
                idf: HashMap::new(),
            })),
            vector_index: Arc::new(Mutex::new(VectorIndex {
                embeddings: Vec::new(),
            })),
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Index a document for BM25 search
    pub fn index_document(&self, node_id: u64, text: &str, meta: NodeMeta) {
        let mut bm25 = self.bm25_index.lock().unwrap();
        bm25.documents.push(Bm25Document {
            node_id,
            text: text.to_string(),
        });

        // Update avgdl
        let total_chars: usize = bm25.documents.iter().map(|d| d.text.len()).sum();
        bm25.avgdl = total_chars as f32 / bm25.documents.len() as f32;

        // Update IDF (simplified)
        let words: Vec<String> = text.split_whitespace().map(|w| w.to_lowercase()).collect();
        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        for word in &words {
            *doc_freq.entry(word.clone()).or_insert(0) += 1;
        }
        let n = bm25.documents.len() as f32;
        for (word, freq) in doc_freq {
            bm25.idf
                .entry(word)
                .and_modify(|v| *v = (*v + ((n - freq as f32 + 0.5) / (freq as f32 + 0.5)).ln()) / 2.0)
                .or_insert(((n - freq as f32 + 0.5) / (freq as f32 + 0.5)).ln());
        }

        // Store metadata
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert(node_id, meta);

        // Store vector embedding (placeholder)
        let mut vi = self.vector_index.lock().unwrap();
        vi.embeddings.push((node_id, vec![0.0; 384])); // placeholder
    }

    /// Hybrid search with RRF fusion
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let bm25_results = self.bm25_search(query);
        let vector_results = self.vector_search(query);

        // RRF fusion
        self.rrf_fuse(&bm25_results, &vector_results)
    }

    /// BM25 lexical search
    fn bm25_search(&self, query: &str) -> Vec<(u64, f32)> {
        let bm25 = self.bm25_index.lock().unwrap();
        let query_words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();

        let mut scores: Vec<(u64, f32)> = Vec::new();

        for doc in &bm25.documents {
            let mut score = 0.0;
            let doc_words: Vec<String> = doc.text.split_whitespace().map(|w| w.to_lowercase()).collect();
            let doc_len = doc_words.len() as f32;

            for q_word in &query_words {
                let tf = doc_words.iter().filter(|w| *w == q_word).count() as f32;
                let idf = bm25.idf.get(q_word).copied().unwrap_or(0.0);

                // BM25 formula
                let k1 = 1.2_f32;
                let b = 0.75_f32;
                let tf_norm = (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * doc_len / bm25.avgdl));
                score += idf * tf_norm;
            }

            if score > 0.0 {
                scores.push((doc.node_id, score));
            }
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }

    /// Vector semantic search (placeholder — needs real embedding model)
    fn vector_search(&self, _query: &str) -> Vec<(u64, f32)> {
        // In production: embed query → HNSW/LanceDB search → return results
        // Placeholder: return empty
        Vec::new()
    }

    /// Reciprocal Rank Fusion
    fn rrf_fuse(&self, bm25: &[(u64, f32)], vector: &[(u64, f32)]) -> Vec<SearchResult> {
        let mut scores: HashMap<u64, f32> = HashMap::new();
        let k = self.config.rrf_k;

        // BM25 contributions
        for (rank, (node_id, _score)) in bm25.iter().enumerate() {
            let rrf_score = 1.0 / (k + rank as f32 + 1.0); // rank is 0-indexed
            *scores.entry(*node_id).or_insert(0.0) += rrf_score * self.config.bm25_weight;
        }

        // Vector contributions
        for (rank, (node_id, _score)) in vector.iter().enumerate() {
            let rrf_score = 1.0 / (k + rank as f32 + 1.0);
            *scores.entry(*node_id).or_insert(0.0) += rrf_score * self.config.vector_weight;
        }

        // Sort and build results
        let mut results: Vec<SearchResult> = scores
            .into_iter()
            .filter(|(_, score)| *score >= self.config.min_score)
            .map(|(node_id, score)| {
                let nodes = self.nodes.lock().unwrap();
                let meta = nodes.get(&node_id).cloned().unwrap_or(NodeMeta {
                    node_id,
                    file_path: String::new(),
                    name: String::new(),
                    kind: String::new(),
                    start_line: 0,
                    end_line: 0,
                    language: String::new(),
                    body: String::new(),
                });

                SearchResult {
                    node_id,
                    file_path: meta.file_path,
                    name: meta.name,
                    kind: meta.kind,
                    start_line: meta.start_line,
                    end_line: meta.end_line,
                    score,
                    bm25_score: None,
                    vector_score: None,
                    snippet: Some(meta.body.chars().take(200).collect()),
                    language: meta.language,
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(self.config.max_results);
        results
    }

    /// Token-aware context packing
    pub fn pack_context(&self, results: &[SearchResult], token_budget: usize) -> Vec<SearchResult> {
        let mut packed = Vec::new();
        let mut tokens_used = 0;

        for result in results {
            // Estimate tokens (rough: 1 token ≈ 4 chars)
            let estimated_tokens = result.snippet.as_ref().map(|s| s.len() / 4).unwrap_or(0) + 20; // metadata overhead

            if tokens_used + estimated_tokens <= token_budget {
                tokens_used += estimated_tokens;
                packed.push(result.clone());
            } else {
                break;
            }
        }

        packed
    }

    /// Get index statistics
    pub fn stats(&self) -> SearchStats {
        let bm25 = self.bm25_index.lock().unwrap();
        let nodes = self.nodes.lock().unwrap();
        let vi = self.vector_index.lock().unwrap();

        SearchStats {
            total_documents: bm25.documents.len(),
            total_nodes: nodes.len(),
            total_embeddings: vi.embeddings.len(),
            avg_doc_length: bm25.avgdl,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub total_documents: usize,
    pub total_nodes: usize,
    pub total_embeddings: usize,
    pub avg_doc_length: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion() {
        let retriever = HybridRetriever::new(SearchConfig::default());

        // Add some documents
        for i in 0..5u32 {
            let body = format!("fn func_{}() {{ }}", i);
            let meta = NodeMeta {
                node_id: i as u64,
                file_path: format!("file{}.rs", i),
                name: format!("func_{}", i),
                kind: "function".to_string(),
                start_line: i * 10,
                end_line: i * 10 + 5,
                language: "rust".to_string(),
                body: body.clone(),
            };
            retriever.index_document(i as u64, &body, meta);
        }

        let results = retriever.search("func");
        assert!(!results.is_empty());
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_context_packing() {
        let retriever = HybridRetriever::new(SearchConfig {
            token_budget: 100,
            ..Default::default()
        });

        let results: Vec<SearchResult> = (0..10)
            .map(|i| SearchResult {
                node_id: i,
                file_path: format!("file{}.rs", i),
                name: format!("func_{}", i),
                kind: "function".to_string(),
                start_line: 0,
                end_line: 5,
                score: 1.0 - i as f32 * 0.1,
                bm25_score: None,
                vector_score: None,
                snippet: Some("x".repeat(200)), // ~50 tokens
                language: "rust".to_string(),
            })
            .collect();

        let packed = retriever.pack_context(&results, 100);
        assert!(packed.len() < results.len()); // Some should be dropped
    }

    #[test]
    fn test_search_stats() {
        let retriever = HybridRetriever::new(SearchConfig::default());
        let stats = retriever.stats();
        assert_eq!(stats.total_documents, 0);
        assert_eq!(stats.total_nodes, 0);
    }
}
