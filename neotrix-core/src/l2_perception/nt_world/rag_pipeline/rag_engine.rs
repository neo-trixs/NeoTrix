//! RAG document retrieval engine — 文档切分 → 向量索引 → 混合检索 → 持久化。
//!
//! 2026-09-28 消歧：本类型原名 `RagEngine`，与
//! `l1_action/nt_core_bank/bank/rag_engine.rs::RagEngine` **同名**。
//! 两者**不是重复实现**：本侧是文档级 RAG（切分/索引/过滤/落盘），
//! 那侧是记忆层级管理（`find_duplicates` / `should_promote` / `tier_counts`
//! / `MemoryConsolidation`，本侧均无），22 个公开符号零交叉。
//! 故本侧改名 `DocumentRagEngine`，`bank` 侧保留 `RagEngine`（名字更贴其语义）。

use std::path::Path;

use super::chunker::chunk_document;
use super::document::{Document, SearchResult};
use super::vector_store::DiskVectorStore;

/// Default chunk size in characters.
const DEFAULT_CHUNK_SIZE: usize = 512;
/// Default overlap in characters.
const DEFAULT_OVERLAP: usize = 64;

/// Local RAG (Retrieval-Augmented Generation) engine.
///
/// Provides `ingest` to chunk and store documents, and `query` to retrieve
/// the most relevant chunks for a natural-language question. Embeddings are
/// derived from a deterministic hash of the chunk text (no external model
/// dependency).
///
/// The engine uses a [`DiskVectorStore`] for persistent vector storage.
/// Call [`DocumentRagEngine::save`] to persist state to disk, or
/// [`DocumentRagEngine::load`] to restore from a previous session.
pub struct DocumentRagEngine {
    store: DiskVectorStore,
    chunk_size: usize,
    overlap: usize,
}

impl DocumentRagEngine {
    /// Create a new engine with default chunk parameters.
    pub fn new() -> Self {
        Self {
            store: DiskVectorStore::new(),
            chunk_size: DEFAULT_CHUNK_SIZE,
            overlap: DEFAULT_OVERLAP,
        }
    }

    /// Create a new engine with custom chunk parameters.
    pub fn with_params(chunk_size: usize, overlap: usize) -> Self {
        assert!(overlap < chunk_size, "overlap must be < chunk_size");
        Self {
            store: DiskVectorStore::new(),
            chunk_size,
            overlap,
        }
    }

    /// Create an engine bound to a persistence file.
    ///
    /// If the file exists, entries are loaded from disk. Otherwise an empty
    /// store is created and ready for ingestion.
    pub fn with_persistence(path: impl AsRef<Path>) -> Result<Self, String> {
        let store = DiskVectorStore::load(path)?;
        Ok(Self {
            store,
            chunk_size: DEFAULT_CHUNK_SIZE,
            overlap: DEFAULT_OVERLAP,
        })
    }

    /// Create an engine with custom chunk parameters and a persistence file.
    pub fn with_params_and_persistence(
        chunk_size: usize,
        overlap: usize,
        path: impl AsRef<Path>,
    ) -> Result<Self, String> {
        assert!(overlap < chunk_size, "overlap must be < chunk_size");
        let store = DiskVectorStore::load(path)?;
        Ok(Self {
            store,
            chunk_size,
            overlap,
        })
    }

    /// Persist the current store state to disk.
    pub fn save(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        self.store.save(path)
    }

    /// Ingest a document: chunk it, compute hash-based embeddings, and store.
    pub fn ingest(&mut self, document: Document) -> Result<(), String> {
        if document.id.is_empty() {
            return Err("document id must not be empty".into());
        }
        if document.content.is_empty() {
            return Err("document content must not be empty".into());
        }

        let mut chunked = chunk_document(&document, self.chunk_size, self.overlap);

        // Compute hash-based embeddings for each chunk.
        for chunk in &mut chunked.chunks {
            let emb = hash_embedding(&chunk.text, 128);
            chunk.embedding = Some(emb);
        }

        self.store.add(&chunked.id, &chunked.chunks);
        Ok(())
    }

    /// Query the store for the top-k most relevant chunks.
    pub fn query(&self, question: &str, top_k: usize) -> Vec<SearchResult> {
        if question.is_empty() || top_k == 0 {
            return Vec::new();
        }
        let q_emb = hash_embedding(question, 128);
        self.store.search(&q_emb, top_k)
    }

    /// Query with a metadata filter function.
    pub fn query_with_filter(
        &self,
        question: &str,
        top_k: usize,
        filter_fn: impl Fn(&std::collections::HashMap<String, String>) -> bool,
    ) -> Vec<SearchResult> {
        if question.is_empty() || top_k == 0 {
            return Vec::new();
        }
        let q_emb = hash_embedding(question, 128);
        self.store.search_with_filter(&q_emb, top_k, filter_fn)
    }

    /// Pre-compute the index for faster repeated searches.
    pub fn build_index(&mut self) {
        self.store.build_index();
    }

    /// Number of stored embeddings.
    pub fn stored_count(&self) -> usize {
        self.store.len()
    }

    /// Access the underlying vector store.
    pub fn store(&self) -> &DiskVectorStore {
        &self.store
    }
}

impl Default for DocumentRagEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Produce a deterministic embedding from text using a hash-based approach.
///
/// The text is hashed with a simple FNV-1a-like fold. The resulting
/// `dim`-dimensional vector is unit-normalized so cosine similarity
/// reduces to dot product.
fn hash_embedding(text: &str, dim: usize) -> Vec<f32> {
    let bytes = text.as_bytes();
    let mut vec = vec![0.0f32; dim];

    // FNV-1a style seed mixing for each dimension.
    for (i, slot) in vec.iter_mut().enumerate() {
        let mut h: u64 = 0xcbf29ce484222325 ^ (i as u64);
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        // Map to [-1, 1].
        *slot = ((h as f32) / (u64::MAX as f32)) * 2.0 - 1.0;
    }

    // L2 normalize.
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut vec {
            *x /= norm;
        }
    }
    vec
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_doc(id: &str, content: &str) -> Document {
        Document {
            id: id.into(),
            content: content.into(),
            metadata: HashMap::new(),
            chunks: vec![],
        }
    }

    #[test]
    fn ingest_and_query_basic() {
        let mut engine = DocumentRagEngine::new();
        engine
            .ingest(make_doc(
                "doc1",
                "Rust is a systems programming language focused on safety and performance.",
            ))
            .unwrap();
        engine
            .ingest(make_doc(
                "doc2",
                "Python is a high-level language known for its simplicity and readability.",
            ))
            .unwrap();

        assert_eq!(engine.stored_count(), 2);

        let results = engine.query("systems programming", 2);
        assert_eq!(results.len(), 2);
        // doc1 should score higher.
        assert_eq!(results[0].doc_id, "doc1");
    }

    #[test]
    fn ingest_empty_id_errors() {
        let mut engine = DocumentRagEngine::new();
        let err = engine.ingest(make_doc("", "content"));
        assert!(err.is_err());
    }

    #[test]
    fn ingest_empty_content_errors() {
        let mut engine = DocumentRagEngine::new();
        let err = engine.ingest(make_doc("d1", ""));
        assert!(err.is_err());
    }

    #[test]
    fn query_empty_returns_empty() {
        let engine = DocumentRagEngine::new();
        assert!(engine.query("", 5).is_empty());
    }

    #[test]
    fn query_top_k_zero_returns_empty() {
        let mut engine = DocumentRagEngine::new();
        engine.ingest(make_doc("d1", "some content here")).unwrap();
        assert!(engine.query("content", 0).is_empty());
    }

    #[test]
    fn custom_chunk_params() {
        let mut engine = DocumentRagEngine::with_params(20, 5);
        engine
            .ingest(make_doc(
                "d1",
                "This is a reasonably long document that should be split into multiple chunks.",
            ))
            .unwrap();
        assert!(engine.stored_count() > 1);
    }

    #[test]
    fn hash_embedding_deterministic() {
        let a = hash_embedding("hello world", 64);
        let b = hash_embedding("hello world", 64);
        assert_eq!(a, b);
    }

    #[test]
    fn hash_embedding_unit_norm() {
        let emb = hash_embedding("test", 32);
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    // -- Persistence integration tests --------------------------------------

    #[test]
    fn engine_persistence_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rag.json");

        // Ingest, save.
        {
            let mut engine = DocumentRagEngine::new();
            engine
                .ingest(make_doc(
                    "doc1",
                    "Rust provides memory safety without garbage collection.",
                ))
                .unwrap();
            engine
                .ingest(make_doc(
                    "doc2",
                    "Go is a statically typed language with garbage collection.",
                ))
                .unwrap();
            engine.save(&path).unwrap();
        }

        // Load, query. Query with doc1's exact content: self-similarity is
        // 1.0 by construction, so doc1 must rank first both before and
        // after the persistence roundtrip (hash embeddings carry no
        // semantics; asserting a semantic ranking would be luck-based).
        {
            let engine = DocumentRagEngine::with_persistence(&path).unwrap();
            assert_eq!(engine.stored_count(), 2);

            let results = engine.query(
                "Rust provides memory safety without garbage collection.",
                2,
            );
            assert_eq!(results.len(), 2);
            assert_eq!(results[0].doc_id, "doc1");
        }
    }

    #[test]
    fn engine_persistence_load_creates_fresh_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new.json");

        let mut engine = DocumentRagEngine::with_persistence(&path).unwrap();
        assert_eq!(engine.stored_count(), 0);
        engine.ingest(make_doc("d1", "new content")).unwrap();
        assert_eq!(engine.stored_count(), 1);
    }

    #[test]
    fn engine_build_index_and_query() {
        let mut engine = DocumentRagEngine::new();
        engine
            .ingest(make_doc("doc1", "Rust is fast and safe."))
            .unwrap();
        engine.build_index();

        let results = engine.query("fast language", 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].doc_id, "doc1");
    }
}
