//! Semantic cache layer — embedding-similarity LLM response cache.
//!
//! Three-tier cache:
//!   1. Exact match (key → value, O(1))
//!   2. Semantic (embedding cosine similarity, O(n))
//!   3. LLM call (bypass - no cache)
//!
//! Inspired by GPTCache, vCache (error-rate-guaranteed), SphereLFU.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Cache entry with metadata.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub response: String,
    pub embedding: Vec<f64>,
    pub access_count: u64,
    pub created: Instant,
    pub last_access: Instant,
    pub ttl: Duration,
}

/// Semantic cache with exact + cosine-similarity retrieval.
#[derive(Debug, Clone)]
pub struct SemanticCache {
    /// Exact-match cache: query_hash → entry
    exact: HashMap<u64, CacheEntry>,
    /// Semantic cache entries (for linear scan)
    entries: Vec<(String, u64, CacheEntry)>, // (query, hash, entry)
    /// Maximum entries
    capacity: usize,
    /// Similarity threshold for semantic cache hit (0.0–1.0)
    similarity_threshold: f64,
    /// LRU order (entry indices for eviction)
    lru: VecDeque<u64>,
}

impl Default for SemanticCache {
    fn default() -> Self {
        Self::new(100, 0.92)
    }
}

impl SemanticCache {
    pub fn new(capacity: usize, similarity_threshold: f64) -> Self {
        Self {
            exact: HashMap::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
            capacity,
            similarity_threshold: similarity_threshold.max(0.0).min(1.0),
            lru: VecDeque::with_capacity(capacity),
        }
    }

    /// Hash a query string for exact matching.
    fn hash_query(query: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        query.hash(&mut hasher);
        hasher.finish()
    }

    /// Insert a query+response into the cache.
    pub fn insert(&mut self, query: &str, response: &str, embedding: Vec<f64>) {
        let hash = Self::hash_query(query);
        let entry = CacheEntry {
            response: response.to_string(),
            embedding,
            access_count: 1,
            created: Instant::now(),
            last_access: Instant::now(),
            ttl: Duration::from_secs(300),
        };

        // Evict if at capacity
        if self.entries.len() >= self.capacity {
            self.evict_one();
        }

        self.exact.insert(hash, entry.clone());

        // Check for duplicate semantic entry
        let existing = self.entries.iter().position(|(q, _, _)| *q == query);
        if let Some(idx) = existing {
            self.entries[idx] = (query.to_string(), hash, entry);
        } else {
            self.entries.push((query.to_string(), hash, entry));
        }

        self.lru.push_back(hash);
    }

    /// Retrieve via exact match.
    pub fn get_exact(&mut self, query: &str) -> Option<String> {
        let hash = Self::hash_query(query);
        let entry = self.exact.get_mut(&hash)?;
        if entry.last_access.elapsed() > entry.ttl {
            self.exact.remove(&hash);
            return None;
        }
        entry.access_count += 1;
        entry.last_access = Instant::now();
        Some(entry.response.clone())
    }

    /// Retrieve via semantic similarity against all stored embeddings.
    /// Returns (response, similarity_score) if best match exceeds threshold.
    pub fn get_semantic(&mut self, query_embedding: &[f64]) -> Option<(String, f64)> {
        let mut best_score = 0.0;
        let mut best_entry: Option<usize> = None;

        for (i, (_, _, entry)) in self.entries.iter().enumerate() {
            if entry.last_access.elapsed() > entry.ttl {
                continue; // Skip expired
            }
            let sim = cosine_similarity(query_embedding, &entry.embedding);
            if sim > best_score {
                best_score = sim;
                best_entry = Some(i);
            }
        }

        let idx = best_entry?;
        if best_score < self.similarity_threshold {
            return None;
        }

        let entry = &mut self.entries[idx].2;
        entry.access_count += 1;
        entry.last_access = Instant::now();
        Some((entry.response.clone(), best_score))
    }

    /// Evict one entry (LRU + lowest access count).
    fn evict_one(&mut self) {
        while let Some(hash) = self.lru.pop_front() {
            if self.exact.remove(&hash).is_some() {
                self.entries.retain(|(_, h, _)| *h != hash);
                return;
            }
        }
        // Fallback: remove oldest entry
        if let Some((query, _, _)) = self.entries.first().cloned() {
            let hash = Self::hash_query(&query);
            self.exact.remove(&hash);
            self.entries.remove(0);
        }
    }

    /// Clear the cache.
    pub fn clear(&mut self) {
        self.exact.clear();
        self.entries.clear();
        self.lru.clear();
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Overall hit rate estimate (access_count weighted).
    pub fn hit_rate(&self) -> f64 {
        let total: u64 = self.entries.iter().map(|(_, _, e)| e.access_count).sum();
        if total == 0 {
            return 0.0;
        }
        // Conservative: count entries with >1 access as "hits"
        let hits: u64 = self.entries.iter().map(|(_, _, e)| if e.access_count > 1 { 1 } else { 0 }).sum();
        hits as f64 / self.entries.len().max(1) as f64
    }

    /// Configurable similarity threshold.
    pub fn set_threshold(&mut self, threshold: f64) {
        self.similarity_threshold = threshold.max(0.0).min(1.0);
    }
}

/// Cosine similarity between two embedding vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a < 1e-12 || norm_b < 1e-12 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).max(-1.0).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_embedding(value: f64, dim: usize) -> Vec<f64> {
        vec![value; dim]
    }

    #[test]
    fn test_insert_and_get_exact() {
        let mut cache = SemanticCache::new(10, 0.9);
        cache.insert("hello", "world", make_embedding(0.5, 8));
        let result = cache.get_exact("hello");
        assert_eq!(result, Some("world".to_string()));
    }

    #[test]
    fn test_exact_miss_returns_none() {
        let mut cache = SemanticCache::new(10, 0.9);
        let result = cache.get_exact("nonexistent");
        assert_eq!(result, None);
    }

    #[test]
    fn test_semantic_match() {
        let mut cache = SemanticCache::new(10, 0.7);
        // [1,0,0,...] vs [0.9,0.1,0,...]
        let mut e1 = vec![0.0; 8];
        e1[0] = 1.0;
        let mut e2 = vec![0.0; 8];
        e2[0] = 0.0;
        e2[1] = 1.0;
        let mut query = vec![0.0; 8];
        query[0] = 0.9;
        query[1] = 0.1;
        cache.insert("q1", "a1", e1);
        cache.insert("q2", "a2", e2);
        let result = cache.get_semantic(&query);
        assert!(result.is_some(), "should find semantic match");
        if let Some((resp, sim)) = result {
            assert_eq!(resp, "a1");
            assert!(sim > 0.7, "similarity should exceed threshold");
        }
    }

    #[test]
    fn test_semantic_below_threshold() {
        let mut cache = SemanticCache::new(10, 0.95);
        // Use orthogonal vectors: [1,0,0,...] vs [0,1,0,...]
        let mut e1 = vec![0.0; 8];
        e1[0] = 1.0;
        let mut e2 = vec![0.0; 8];
        e2[1] = 1.0;
        cache.insert("q1", "a1", e1);
        let result = cache.get_semantic(&e2);
        assert!(result.is_none(), "orthogonal vectors should not match at threshold 0.95");
    }

    #[test]
    fn test_eviction() {
        let mut cache = SemanticCache::new(3, 0.9);
        for i in 0..5 {
            cache.insert(&format!("q{}", i), &format!("a{}", i), make_embedding(i as f64 * 0.1, 8));
        }
        assert!(cache.len() <= 3, "should evict to capacity, got {}", cache.len());
    }

    #[test]
    fn test_clear() {
        let mut cache = SemanticCache::new(10, 0.9);
        cache.insert("q1", "a1", make_embedding(0.5, 8));
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_hit_rate_empty() {
        let cache = SemanticCache::new(10, 0.9);
        assert!((cache.hit_rate() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_set_threshold() {
        let mut cache = SemanticCache::new(10, 0.9);
        let e1 = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let query = vec![0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        cache.insert("q1", "a1", e1);
        // With threshold 0.5, similarity ~0.9 should match
        cache.set_threshold(0.5);
        let result = cache.get_semantic(&query);
        assert!(result.is_some(), "should match at threshold 0.5");
    }
}
