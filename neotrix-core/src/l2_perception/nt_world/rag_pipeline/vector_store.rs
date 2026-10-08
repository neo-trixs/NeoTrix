use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::document::SearchResult;

// ---------------------------------------------------------------------------
// Serde-compatible store entry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoreEntry {
    doc_id: String,
    chunk_text: String,
    embedding: Vec<f32>,
    #[serde(default)]
    metadata: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// In-memory vector store (unchanged public API)
// ---------------------------------------------------------------------------

/// In-memory vector store with brute-force cosine similarity search.
///
/// Stores chunk embeddings keyed by `(doc_id, chunk_index)` and provides
/// a `search` method that returns the `top_k` most similar chunks to a
/// query embedding.
#[derive(Debug, Default)]
pub struct VectorStore {
    entries: Vec<StoreEntry>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add chunks for a document into the store.
    /// Only chunks that **have** an embedding are stored.
    pub fn add(&mut self, doc_id: &str, chunks: &[super::document::Chunk]) {
        for chunk in chunks {
            if let Some(ref emb) = chunk.embedding {
                self.entries.push(StoreEntry {
                    doc_id: doc_id.to_string(),
                    chunk_text: chunk.text.clone(),
                    embedding: emb.clone(),
                    metadata: HashMap::new(),
                });
            }
        }
    }

    /// Return the number of stored embeddings.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Search for the `top_k` most similar chunks to `query_embedding`.
    ///
    /// Uses cosine similarity. Results are returned in descending score order.
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<SearchResult> {
        if self.entries.is_empty() || top_k == 0 {
            return Vec::new();
        }

        let mut scored: Vec<(usize, f32)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| (i, cosine_similarity(query_embedding, &e.embedding)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        scored
            .into_iter()
            .map(|(i, score)| {
                let e = &self.entries[i];
                SearchResult {
                    doc_id: e.doc_id.clone(),
                    chunk_text: e.chunk_text.clone(),
                    score,
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Pre-computed index entry (normalized embeddings for fast dot-product)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct IndexEntry {
    doc_id: String,
    chunk_text: String,
    norm_embedding: Vec<f32>,
    metadata: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// DiskVectorStore — persistent vector store
// ---------------------------------------------------------------------------

/// Persistent vector store backed by a JSON file on disk.
///
/// Uses `serde_json` for serialization (no external vector-DB dependency).
/// Supports batch operations, pre-computed index for fast search, and
/// metadata filtering.
///
/// # R-P118 Memory Lifecycle
/// Entries are flushed to disk on every `save()` call. Load is lazy — only
/// entries already persisted are restored; the caller decides when to persist.
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct DiskVectorStore {
    entries: Vec<StoreEntry>,
    /// Pre-computed index for O(1) dot-product search (built via `build_index`).
    index: Option<Vec<IndexEntry>>,
    /// Path to the backing JSON file (set on load or first save).
    path: Option<std::path::PathBuf>,
}


impl DiskVectorStore {
    pub fn new() -> Self {
        Self::default()
    }

    // -- Persistence --------------------------------------------------------

    /// Save the current store state to a JSON file at `path`.
    ///
    /// Creates parent directories if they don't exist. The file is written
    /// atomically by writing to a temp file then renaming.
    pub fn save(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("failed to create parent dirs: {e}"))?;
        }

        let tmp = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| format!("serialization failed: {e}"))?;
        fs::write(&tmp, json).map_err(|e| format!("write failed: {e}"))?;
        fs::rename(&tmp, path).map_err(|e| format!("rename failed: {e}"))?;

        self.path = Some(path.to_path_buf());
        Ok(())
    }

    /// Load entries from a JSON file at `path` into a new `DiskVectorStore`.
    ///
    /// Returns an empty store (without error) if the file does not exist,
    /// so callers can start fresh and `save` later.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self {
                entries: Vec::new(),
                index: None,
                path: Some(path.to_path_buf()),
            });
        }

        let data = fs::read_to_string(path).map_err(|e| format!("read failed: {e}"))?;
        let entries: Vec<StoreEntry> =
            serde_json::from_str(&data).map_err(|e| format!("deserialization failed: {e}"))?;

        Ok(Self {
            entries,
            index: None,
            path: Some(path.to_path_buf()),
        })
    }

    /// Return the path this store is bound to (if any).
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    // -- Mutation -----------------------------------------------------------

    /// Add chunks for a document into the store.
    /// Only chunks that **have** an embedding are stored.
    pub fn add(&mut self, doc_id: &str, chunks: &[super::document::Chunk]) {
        for chunk in chunks {
            if let Some(ref emb) = chunk.embedding {
                self.entries.push(StoreEntry {
                    doc_id: doc_id.to_string(),
                    chunk_text: chunk.text.clone(),
                    embedding: emb.clone(),
                    metadata: HashMap::new(),
                });
            }
        }
        // Invalidate index when entries change.
        self.index = None;
    }

    /// Add chunks with per-entry metadata.
    pub fn add_with_metadata(
        &mut self,
        doc_id: &str,
        chunks: &[super::document::Chunk],
        metadata: HashMap<String, String>,
    ) {
        for chunk in chunks {
            if let Some(ref emb) = chunk.embedding {
                self.entries.push(StoreEntry {
                    doc_id: doc_id.to_string(),
                    chunk_text: chunk.text.clone(),
                    embedding: emb.clone(),
                    metadata: metadata.clone(),
                });
            }
        }
        self.index = None;
    }

    /// Batch-add multiple documents at once.
    pub fn add_batch(&mut self, items: &[(&str, &[super::document::Chunk])]) {
        for (doc_id, chunks) in items {
            self.add(doc_id, chunks);
        }
    }

    /// Batch-add multiple documents with shared metadata per document.
    pub fn add_batch_with_metadata(
        &mut self,
        items: &[(&str, &[super::document::Chunk], HashMap<String, String>)],
    ) {
        for (doc_id, chunks, meta) in items {
            self.add_with_metadata(doc_id, chunks, meta.clone());
        }
    }

    // -- Index ---------------------------------------------------------------

    /// Pre-compute normalized embeddings for faster search.
    ///
    /// After calling this, `search` and `search_with_filter` use dot-product
    /// on pre-normalized vectors instead of computing cosine similarity on
    /// the fly. Useful when the store is static and queried many times.
    pub fn build_index(&mut self) {
        self.index = Some(
            self.entries
                .iter()
                .map(|e| {
                    let norm_embedding = l2_normalize(&e.embedding);
                    IndexEntry {
                        doc_id: e.doc_id.clone(),
                        chunk_text: e.chunk_text.clone(),
                        norm_embedding,
                        metadata: e.metadata.clone(),
                    }
                })
                .collect(),
        );
    }

    // -- Query --------------------------------------------------------------

    /// Return the number of stored embeddings.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Search for the `top_k` most similar chunks to `query_embedding`.
    ///
    /// Uses cosine similarity. If an index has been built via `build_index`,
    /// uses pre-normalized dot-product for speed.
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<SearchResult> {
        if self.entries.is_empty() || top_k == 0 {
            return Vec::new();
        }

        if let Some(ref idx) = self.index {
            search_index(idx, query_embedding, top_k, None)
        } else {
            search_brute_force(&self.entries, query_embedding, top_k, None)
        }
    }

    /// Search with a metadata filter function.
    ///
    /// Only entries whose metadata satisfies `filter_fn` are considered.
    /// The filter receives a reference to the entry's metadata map.
    pub fn search_with_filter(
        &self,
        query_embedding: &[f32],
        top_k: usize,
        filter_fn: impl Fn(&HashMap<String, String>) -> bool,
    ) -> Vec<SearchResult> {
        if self.entries.is_empty() || top_k == 0 {
            return Vec::new();
        }

        if let Some(ref idx) = self.index {
            search_index(idx, query_embedding, top_k, Some(&filter_fn))
        } else {
            search_brute_force(&self.entries, query_embedding, top_k, Some(&filter_fn))
        }
    }

    /// Batch-search multiple query embeddings in one call.
    ///
    /// Returns a `Vec` of result sets, one per query, each containing
    /// `top_k` results.
    pub fn search_batch(&self, queries: &[&[f32]], top_k: usize) -> Vec<Vec<SearchResult>> {
        queries.iter().map(|q| self.search(q, top_k)).collect()
    }

    /// Number of stored embeddings (alias for `len`).
    pub fn count(&self) -> usize {
        self.len()
    }

    /// Clear all entries and invalidate the index.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.index = None;
    }
}

// ---------------------------------------------------------------------------
// Internal search helpers
// ---------------------------------------------------------------------------

fn search_brute_force(
    entries: &[StoreEntry],
    query: &[f32],
    top_k: usize,
    filter: Option<&dyn Fn(&HashMap<String, String>) -> bool>,
) -> Vec<SearchResult> {
    let mut scored: Vec<(usize, f32)> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| filter.is_none_or(|f| f(&e.metadata)))
        .map(|(i, e)| (i, cosine_similarity(query, &e.embedding)))
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    scored
        .into_iter()
        .map(|(i, score)| {
            let e = &entries[i];
            SearchResult {
                doc_id: e.doc_id.clone(),
                chunk_text: e.chunk_text.clone(),
                score,
            }
        })
        .collect()
}

fn search_index(
    index: &[IndexEntry],
    query: &[f32],
    top_k: usize,
    filter: Option<&dyn Fn(&HashMap<String, String>) -> bool>,
) -> Vec<SearchResult> {
    let q_norm = l2_normalize(query);

    let mut scored: Vec<(usize, f32)> = index
        .iter()
        .enumerate()
        .filter(|(_, e)| filter.is_none_or(|f| f(&e.metadata)))
        .map(|(i, e)| {
            // Dot product of normalized vectors == cosine similarity.
            let dot: f32 = q_norm
                .iter()
                .zip(e.norm_embedding.iter())
                .map(|(a, b)| a * b)
                .sum();
            (i, dot)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    scored
        .into_iter()
        .map(|(i, score)| {
            let e = &index[i];
            SearchResult {
                doc_id: e.doc_id.clone(),
                chunk_text: e.chunk_text.clone(),
                score,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Vector math
// ---------------------------------------------------------------------------

/// Compute cosine similarity between two vectors.
///
/// Returns 0.0 if either vector is zero-length or lengths mismatch.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (&ai, &bi) in a.iter().zip(b.iter()) {
        dot += ai * bi;
        norm_a += ai * ai;
        norm_b += bi * bi;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        return 0.0;
    }
    dot / denom
}

/// L2-normalize a vector in place. Returns a new normalized copy.
fn l2_normalize(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 {
        return v.to_vec();
    }
    v.iter().map(|x| x / norm).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_perception::nt_world::rag_pipeline::document::Chunk;

    fn emb(vals: &[f32]) -> Option<Vec<f32>> {
        Some(vals.to_vec())
    }

    fn chunks_with_emb(doc_id: &str, items: &[(&str, &[f32])]) -> Vec<Chunk> {
        items
            .iter()
            .map(|(text, e)| Chunk {
                text: (*text).into(),
                start_offset: 0,
                end_offset: text.len(),
                embedding: emb(e),
            })
            .collect()
    }

    // -- Cosine similarity tests -------------------------------------------

    #[test]
    fn cosine_identical() {
        assert!((cosine_similarity(&[1.0, 2.0], &[1.0, 2.0]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_orthogonal() {
        assert!((cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_mismatched_len() {
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn cosine_zero_vector() {
        assert_eq!(cosine_similarity(&[0.0, 0.0], &[1.0, 2.0]), 0.0);
    }

    // -- VectorStore (original) tests --------------------------------------

    #[test]
    fn vector_store_add_and_search() {
        let mut store = VectorStore::new();
        store.add(
            "doc1",
            &[
                Chunk {
                    text: "alpha".into(),
                    start_offset: 0,
                    end_offset: 5,
                    embedding: emb(&[1.0, 0.0, 0.0]),
                },
                Chunk {
                    text: "beta".into(),
                    start_offset: 6,
                    end_offset: 10,
                    embedding: emb(&[0.0, 1.0, 0.0]),
                },
            ],
        );
        assert_eq!(store.len(), 2);

        let results = store.search(&[1.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].doc_id, "doc1");
        assert_eq!(results[0].chunk_text, "alpha");
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn vector_store_search_empty() {
        let store = VectorStore::new();
        assert!(store.search(&[1.0], 5).is_empty());
    }

    #[test]
    fn vector_store_chunks_without_embedding_not_stored() {
        let mut store = VectorStore::new();
        store.add(
            "doc1",
            &[Chunk {
                text: "no emb".into(),
                start_offset: 0,
                end_offset: 6,
                embedding: None,
            }],
        );
        assert!(store.is_empty());
    }

    // -- DiskVectorStore: basic operations ----------------------------------

    #[test]
    fn disk_store_add_and_search() {
        let mut store = DiskVectorStore::new();
        store.add(
            "doc1",
            &chunks_with_emb(
                "doc1",
                &[("alpha", &[1.0, 0.0, 0.0]), ("beta", &[0.0, 1.0, 0.0])],
            ),
        );
        assert_eq!(store.len(), 2);

        let results = store.search(&[1.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].chunk_text, "alpha");
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn disk_store_add_with_metadata_and_filter() {
        let mut store = DiskVectorStore::new();
        let mut meta_a = HashMap::new();
        meta_a.insert("source".into(), "web".into());
        let mut meta_b = HashMap::new();
        meta_b.insert("source".into(), "file".into());

        store.add_with_metadata(
            "doc1",
            &[Chunk {
                text: "from web".into(),
                start_offset: 0,
                end_offset: 7,
                embedding: emb(&[1.0, 0.0]),
            }],
            meta_a,
        );
        store.add_with_metadata(
            "doc2",
            &[Chunk {
                text: "from file".into(),
                start_offset: 0,
                end_offset: 9,
                embedding: emb(&[0.9, 0.1]),
            }],
            meta_b,
        );

        // Filter: only "web" source
        let results = store.search_with_filter(&[1.0, 0.0], 10, |m| {
            m.get("source").map_or(false, |v| v == "web")
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].doc_id, "doc1");
    }

    #[test]
    fn disk_store_batch_add() {
        let mut store = DiskVectorStore::new();
        let c1 = chunks_with_emb("d1", &[("a", &[1.0])]);
        let c2 = chunks_with_emb("d2", &[("b", &[0.5])]);
        store.add_batch(&[("d1", &c1), ("d2", &c2)]);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn disk_store_search_batch() {
        let mut store = DiskVectorStore::new();
        store.add(
            "doc1",
            &chunks_with_emb("doc1", &[("x", &[1.0, 0.0]), ("y", &[0.0, 1.0])]),
        );

        let queries: Vec<&[f32]> = vec![&[1.0, 0.0], &[0.0, 1.0]];
        let results = store.search_batch(&queries, 1);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0][0].chunk_text, "x");
        assert_eq!(results[1][0].chunk_text, "y");
    }

    // -- Index optimization tests ------------------------------------------

    #[test]
    fn disk_store_index_search_matches_brute_force() {
        let mut store = DiskVectorStore::new();
        store.add(
            "doc1",
            &chunks_with_emb(
                "doc1",
                &[
                    ("alpha", &[1.0, 0.0, 0.0]),
                    ("beta", &[0.0, 1.0, 0.0]),
                    ("gamma", &[0.5, 0.5, 0.7]),
                ],
            ),
        );

        let query = &[0.8, 0.2, 0.1];
        let bf = store.search(query, 3);

        store.build_index();
        let idx = store.search(query, 3);

        assert_eq!(bf.len(), idx.len());
        for (a, b) in bf.iter().zip(idx.iter()) {
            assert_eq!(a.doc_id, b.doc_id);
            assert_eq!(a.chunk_text, b.chunk_text);
            assert!(
                (a.score - b.score).abs() < 1e-4,
                "scores differ: {} vs {}",
                a.score,
                b.score
            );
        }
    }

    #[test]
    fn disk_store_index_invalidation_on_add() {
        let mut store = DiskVectorStore::new();
        store.build_index();
        assert!(store.index.is_some());

        store.add("doc1", &chunks_with_emb("doc1", &[("x", &[1.0])]));
        assert!(store.index.is_none());
    }

    // -- Persistence tests --------------------------------------------------

    #[test]
    fn disk_store_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vectors.json");

        // Build and save.
        let mut store = DiskVectorStore::new();
        store.add(
            "doc1",
            &chunks_with_emb(
                "doc1",
                &[
                    ("rust is great", &[1.0, 0.0, 0.0]),
                    ("python is nice", &[0.0, 1.0, 0.0]),
                ],
            ),
        );
        store.add(
            "doc2",
            &chunks_with_emb("doc2", &[("go is fast", &[0.5, 0.5, 0.7])]),
        );
        store.save(&path).unwrap();
        assert_eq!(store.len(), 3);

        // Load into a fresh store.
        let loaded = DiskVectorStore::load(&path).unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded.path(), Some(path.as_path()));

        // Search results should be identical.
        let query = &[1.0, 0.0, 0.0];
        let original_results = store.search(query, 3);
        let loaded_results = loaded.search(query, 3);

        assert_eq!(original_results.len(), loaded_results.len());
        for (a, b) in original_results.iter().zip(loaded_results.iter()) {
            assert_eq!(a.doc_id, b.doc_id);
            assert_eq!(a.chunk_text, b.chunk_text);
            assert!((a.score - b.score).abs() < 1e-6);
        }
    }

    #[test]
    fn disk_store_load_nonexistent_file_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does_not_exist.json");
        let store = DiskVectorStore::load(&path).unwrap();
        assert!(store.is_empty());
        assert_eq!(store.path(), Some(path.as_path()));
    }

    #[test]
    fn disk_store_save_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("dir").join("vectors.json");

        let mut store = DiskVectorStore::new();
        store.add("d1", &chunks_with_emb("d1", &[("hello", &[1.0])]));
        store.save(&path).unwrap();

        let loaded = DiskVectorStore::load(&path).unwrap();
        assert_eq!(loaded.len(), 1);
    }

    #[test]
    fn disk_store_save_load_with_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("meta.json");

        let mut store = DiskVectorStore::new();
        let mut meta = HashMap::new();
        meta.insert("category".into(), "rust".into());
        store.add_with_metadata(
            "doc1",
            &[Chunk {
                text: "ownership".into(),
                start_offset: 0,
                end_offset: 10,
                embedding: emb(&[1.0, 0.0]),
            }],
            meta,
        );
        store.save(&path).unwrap();

        let loaded = DiskVectorStore::load(&path).unwrap();
        let results = loaded.search_with_filter(&[1.0, 0.0], 10, |m| {
            m.get("category").map_or(false, |v| v == "rust")
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk_text, "ownership");
    }

    #[test]
    fn disk_store_index_persistence_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("indexed.json");

        let mut store = DiskVectorStore::new();
        store.add(
            "doc1",
            &chunks_with_emb("doc1", &[("a", &[1.0, 0.0]), ("b", &[0.0, 1.0])]),
        );
        store.build_index();
        store.save(&path).unwrap();

        // Load — index should be None (not persisted), but search still works.
        let loaded = DiskVectorStore::load(&path).unwrap();
        assert!(loaded.index.is_none());
        let results = loaded.search(&[1.0, 0.0], 1);
        assert_eq!(results[0].chunk_text, "a");
    }

    #[test]
    fn disk_store_clear() {
        let mut store = DiskVectorStore::new();
        store.add("d1", &chunks_with_emb("d1", &[("x", &[1.0])]));
        store.build_index();
        assert_eq!(store.len(), 1);
        assert!(store.index.is_some());

        store.clear();
        assert!(store.is_empty());
        assert!(store.index.is_none());
    }

    #[test]
    fn disk_store_count_alias() {
        let mut store = DiskVectorStore::new();
        store.add(
            "d1",
            &chunks_with_emb("d1", &[("a", &[1.0]), ("b", &[0.5])]),
        );
        assert_eq!(store.count(), 2);
    }

    // -- L2 normalize tests -------------------------------------------------

    #[test]
    fn l2_normalize_unit_length() {
        let v = l2_normalize(&[3.0, 4.0]);
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn l2_normalize_zero_vector() {
        let v = l2_normalize(&[0.0, 0.0]);
        assert_eq!(v, vec![0.0, 0.0]);
    }
}
