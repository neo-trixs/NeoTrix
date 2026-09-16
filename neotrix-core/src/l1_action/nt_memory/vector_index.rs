//! Vector Index — Approximate Nearest Neighbor Search
//!
//! Uses LRU caching with cosine similarity for vector matching.
//! Provides insert, search, delete, and update operations.

use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

// ============================================================================
// Types
// ============================================================================

/// A vector entry stored in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub vector: Vec<f64>,
    pub metadata: HashMap<String, String>,
    pub score: f64,
}

/// Search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub similarity: f64,
    pub metadata: HashMap<String, String>,
}

/// Configuration for the vector index
#[derive(Debug, Clone)]
pub struct VectorIndexConfig {
    pub capacity: usize,
    pub dimensions: usize,
}

impl Default for VectorIndexConfig {
    fn default() -> Self {
        Self {
            capacity: 10_000,
            dimensions: 128,
        }
    }
}

// ============================================================================
// Vector Index
// ============================================================================

/// Vector Index with LRU caching and cosine similarity-based ANN search.
///
/// Stores vector embeddings and supports efficient approximate nearest
/// neighbor queries using cosine similarity as the distance metric.
pub struct VectorIndex {
    /// LRU cache storing vectors by ID
    cache: Arc<Mutex<LruCache<String, VectorEntry>>>,
    /// Dimensionality of vectors
    dimensions: usize,
    /// Total insert count
    insert_count: Arc<Mutex<u64>>,
    /// Deleted IDs tracking
    deleted_ids: Arc<Mutex<HashSet<String>>>,
}

impl VectorIndex {
    /// Create a new vector index with default configuration
    pub fn new() -> Self {
        Self::with_config(VectorIndexConfig::default())
    }

    /// Create a new vector index with custom configuration
    pub fn with_config(config: VectorIndexConfig) -> Self {
        let capacity = NonZeroUsize::new(config.capacity.max(1)).unwrap();
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            dimensions: config.dimensions.max(1),
            insert_count: Arc::new(Mutex::new(0)),
            deleted_ids: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Insert a vector entry into the index
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the vector
    /// * `vector` - The embedding vector
    /// * `metadata` - Optional metadata associated with the vector
    ///
    /// # Returns
    /// `true` if the insert succeeded, `false` if vector dimension mismatch
    pub fn insert(&mut self, id: &str, vector: Vec<f64>, metadata: HashMap<String, String>) -> bool {
        if vector.len() != self.dimensions {
            return false;
        }

        let score = Self::l2_norm(&vector);
        let entry = VectorEntry {
            id: id.to_string(),
            vector,
            metadata,
            score,
        };

        let mut cache = self.cache.lock().unwrap();
        cache.put(id.to_string(), entry);

        let mut count = self.insert_count.lock().unwrap();
        *count += 1;

        let mut deleted = self.deleted_ids.lock().unwrap();
        deleted.remove(id);

        true
    }

    /// Search for the nearest neighbors to the query vector
    ///
    /// # Arguments
    /// * `query` - The query vector
    /// * `k` - Number of results to return
    ///
    /// # Returns
    /// Vec of `SearchResult` sorted by cosine similarity descending
    pub fn search(&self, query: &[f64], k: usize) -> Vec<SearchResult> {
        if query.len() != self.dimensions {
            return vec![];
        }

        let entries: Vec<VectorEntry> = {
        let cache = self.cache.lock().unwrap();
        cache.iter().map(|(_, v)| v.clone()).collect()
    };

        let mut results: Vec<SearchResult> = entries
            .iter()
            .filter_map(|entry| {
                let similarity = Self::cosine_similarity(query, &entry.vector);
                if similarity > -1.0 {
                    Some(SearchResult {
                        id: entry.id.clone(),
                        similarity,
                        metadata: entry.metadata.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);

        results
    }

    /// Delete a vector entry by ID
    ///
    /// # Returns
    /// `true` if the entry was found and deleted, `false` otherwise
    pub fn delete(&mut self, id: &str) -> bool {
        let mut cache = self.cache.lock().unwrap();
        let removed = cache.pop(id);

        if removed.is_some() {
            let mut deleted = self.deleted_ids.lock().unwrap();
            deleted.insert(id.to_string());
            true
        } else {
            false
        }
    }

    /// Update an existing vector entry
    ///
    /// # Returns
    /// `true` if the entry was found and updated, `false` otherwise
    pub fn update(&mut self, id: &str, vector: Vec<f64>, metadata: HashMap<String, String>) -> bool {
        if vector.len() != self.dimensions {
            return false;
        }

        let mut cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.get_mut(id) {
            entry.vector = vector;
            entry.metadata = metadata;
            entry.score = Self::l2_norm(&entry.vector);
            true
        } else {
            false
        }
    }

    /// Get a vector entry by ID
    pub fn get(&self, id: &str) -> Option<VectorEntry> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(id).cloned()
    }

    /// Get the number of entries in the index
    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the total insert count
    pub fn total_inserts(&self) -> u64 {
        let count = self.insert_count.lock().unwrap();
        *count
    }

    /// Compute cosine similarity between two vectors
    fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Compute L2 norm of a vector
    fn l2_norm(vector: &[f64]) -> f64 {
        vector.iter().map(|x| x * x).sum::<f64>().sqrt()
    }
}

impl Default for VectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vector(dim: usize) -> Vec<f64> {
        (0..dim).map(|i| (i as f64) / dim as f64).collect()
    }

    #[test]
    fn test_insert_and_get() {
        let mut index = VectorIndex::new();
        let v = make_vector(128);
        assert!(index.insert("v1", v.clone(), HashMap::new()));
        assert_eq!(index.len(), 1);

        let entry = index.get("v1");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().id, "v1");
    }

    #[test]
    fn test_dimension_mismatch() {
        let mut index = VectorIndex::with_config(VectorIndexConfig {
            capacity: 10,
            dimensions: 4,
        });
        let v = make_vector(128);
        assert!(!index.insert("v1", v, HashMap::new()));
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_search() {
        let mut index = VectorIndex::new();
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];
        let v3 = vec![1.0, 0.1, 0.0];

        index.insert("a", v1, HashMap::new());
        index.insert("b", v2, HashMap::new());
        index.insert("c", v3, HashMap::new());

        let query = vec![1.0, 0.0, 0.0];
        let results = index.search(&query, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "a");
        assert!(results[0].similarity > 0.9);
    }

    #[test]
    fn test_delete() {
        let mut index = VectorIndex::new();
        index.insert("v1", make_vector(128), HashMap::new());
        assert_eq!(index.len(), 1);

        assert!(index.delete("v1"));
        assert_eq!(index.len(), 0);
        assert!(index.get("v1").is_none());
    }

    #[test]
    fn test_update() {
        let mut index = VectorIndex::new();
        let v1 = vec![1.0, 0.0];
        let v2 = vec![0.0, 1.0];

        index.insert("v1", v1, HashMap::new());
        assert!(index.update("v1", v2.clone(), HashMap::new()));

        let entry = index.get("v1").unwrap();
        assert_eq!(entry.vector, v2);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = VectorIndex::cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-10);

        let c = vec![0.0, 1.0, 0.0];
        let sim_orth = VectorIndex::cosine_similarity(&a, &c);
        assert!(sim_orth.abs() < 1e-10);
    }

    #[test]
    fn test_empty_index_search() {
        let index = VectorIndex::new();
        let results = index.search(&[1.0, 0.0], 5);
        assert!(results.is_empty());
    }
}
