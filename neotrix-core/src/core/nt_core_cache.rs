//! # nt_core_cache — Semantic Response Cache
//!
//! Embedding-similarity cache for LLM responses with:
//! - Online-learned adaptive threshold (vCache-style)
//! - LRU eviction (1000 entries)
//! - TTL per entry (default 60s, configurable)
//! - Hit/miss statistics for observability

use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::{Duration, Instant};

/// A single cached entry mapping a prompt embedding to its LLM response.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub prompt_embedding: Vec<f64>,
    pub response: String,
    pub created_at: Instant,
    pub ttl: Duration,
    pub hit_count: u64,
}

/// Semantic response cache with adaptive similarity threshold.
///
/// ## Adaptive Threshold (vCache-style)
/// - Starts at 0.92
/// - On false positive: raise by 0.01 (cache returned stale but semantically different result)
/// - On false negative: lower by 0.005 (cache missed a valid semantic match)
/// - Clamped to [0.80, 0.99]
///
/// ## LRU Eviction
/// When the cache exceeds `max_entries`, the least recently used entry (back of the
/// deque) is evicted. The deque front always holds the most recently accessed entry.
pub struct SemanticCache {
    entries: HashMap<u64, CacheEntry>,
    lru: VecDeque<u64>,
    max_entries: usize,
    default_ttl: Duration,
    threshold: f64,
    pub stats: CacheStats,
}

/// Hit/miss/eviction statistics for the semantic cache.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub false_positives: u64,
    pub false_negatives: u64,
    pub evictions: u64,
}

impl SemanticCache {
    /// Create a new semantic cache.
    ///
    /// * `max_entries` — Maximum number of entries before LRU eviction kicks in.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            lru: VecDeque::with_capacity(max_entries),
            max_entries,
            default_ttl: Duration::from_secs(60),
            threshold: 0.92,
            stats: CacheStats::default(),
        }
    }

    /// Compute cosine similarity between two embedding vectors.
    ///
    /// Returns a value in [0.0, 1.0]. Returns 0.0 if either vector is empty or
    /// has zero magnitude.
    fn cosine_sim(a: &[f64], b: &[f64]) -> f64 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
    }

    /// Hash a prompt embedding into a `u64` key for O(1) exact-match lookup.
    fn hash_embedding(embedding: &[f64]) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Iterate over raw bytes of each f64 for a deterministic hash.
        for v in embedding {
            v.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Check the cache for a semantically similar prompt.
    ///
    /// Returns the cached response string if a match is found above the adaptive
    /// similarity threshold. On a false positive (similarity > threshold but the
    /// cached response is semantically different), the caller should call
    /// [`feedback`] to penalise the threshold.
    pub fn get(&mut self, prompt_embedding: &[f64]) -> Option<&str> {
        if self.entries.is_empty() {
            self.stats.misses += 1;
            return None;
        }

        let mut best_key: Option<u64> = None;
        let mut best_sim: f64 = 0.0;

        // Linear scan over all entries — acceptable for up to 1000 entries at
        // embedding dimension ~768-1536. For larger caches, consider an ANN index.
        for (key, entry) in self.entries.iter() {
            let sim = Self::cosine_sim(prompt_embedding, &entry.prompt_embedding);
            if sim > best_sim {
                best_sim = sim;
                best_key = Some(*key);
            }
        }

        match best_key {
            Some(key) if best_sim >= self.threshold => {
                // Move to front of LRU
                if let Some(pos) = self.lru.iter().position(|k| *k == key) {
                    self.lru.remove(pos);
                }
                self.lru.push_front(key);

                // Phase 1: Check expiration with immutable borrow
                let is_expired = self
                    .entries
                    .get(&key)
                    .map(|e| e.created_at.elapsed() > e.ttl)
                    .unwrap_or(true);

                if is_expired {
                    self.entries.remove(&key);
                    if let Some(pos) = self.lru.iter().position(|k| *k == key) {
                        self.lru.remove(pos);
                    }
                    self.stats.misses += 1;
                    return None;
                }

                // Phase 2: Now mutable borrow for hit path
                if let Some(entry) = self.entries.get_mut(&key) {
                    entry.hit_count += 1;
                    self.stats.hits += 1;
                    return Some(entry.response.as_str());
                }

                self.stats.misses += 1;
                None
            }
            _ => {
                // Check for false negative: similarity was high but below threshold
                if best_sim >= self.threshold - 0.10 && best_sim < self.threshold {
                    self.stats.false_negatives += 1;
                }
                self.stats.misses += 1;
                None
            }
        }
    }

    /// Store a response in the cache.
    ///
    /// If an entry with an identical embedding hash already exists, it is updated
    /// in-place and moved to the front of the LRU. Otherwise a new entry is created.
    pub fn put(&mut self, prompt_embedding: Vec<f64>, response: String) {
        let key = Self::hash_embedding(&prompt_embedding);

        // Update existing entry
        if self.entries.contains_key(&key) {
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.response = response;
                entry.created_at = Instant::now();
                entry.hit_count = 0;
            }
            // Move to front of LRU
            if let Some(pos) = self.lru.iter().position(|k| *k == key) {
                self.lru.remove(pos);
            }
            self.lru.push_front(key);
            return;
        }

        // Evict if at capacity
        self.evict_if_needed();

        let entry = CacheEntry {
            prompt_embedding,
            response,
            created_at: Instant::now(),
            ttl: self.default_ttl,
            hit_count: 0,
        };

        self.entries.insert(key, entry);
        self.lru.push_front(key);
    }

    /// Evict the least recently used entry if the cache is at capacity.
    fn evict_if_needed(&mut self) {
        while self.entries.len() >= self.max_entries {
            if let Some(old_key) = self.lru.pop_back() {
                self.entries.remove(&old_key);
                self.stats.evictions += 1;
            } else {
                break;
            }
        }
    }

    /// Provide feedback to adapt the similarity threshold.
    ///
    /// vCache-style online learning:
    /// - False positive → raise threshold by 0.01 (be more strict)
    /// - False negative → lower threshold by 0.005 (be more permissive)
    ///
    /// The threshold is clamped to [0.80, 0.99].
    ///
    /// * `was_hit` — Whether `get()` returned `Some`.
    /// * `should_have_been_hit` — Whether, in hindsight, a cached result existed.
    /// * `similarity` — The cosine similarity observed between the query and best entry.
    pub fn feedback(&mut self, was_hit: bool, should_have_been_hit: bool, _similarity: f64) {
        if was_hit && !should_have_been_hit {
            // False positive: found a match but it was incorrect
            self.stats.false_positives += 1;
            self.threshold = (self.threshold + 0.01).clamp(0.80, 0.99);
        } else if !was_hit && should_have_been_hit {
            // False negative: missed a match that should have been found
            self.stats.false_negatives += 1;
            self.threshold = (self.threshold - 0.005).clamp(0.80, 0.99);
        }
        // True positive / true negative: no adjustment needed
    }

    /// Clear all entries and reset statistics.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru.clear();
        self.stats = CacheStats::default();
    }

    /// Return the number of entries currently in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the current adaptive threshold value.
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    /// Set a new default TTL for newly inserted entries.
    pub fn set_default_ttl(&mut self, ttl: Duration) {
        self.default_ttl = ttl;
    }

    /// Remove and return the entry with the given exact hash, if present.
    pub fn remove(&mut self, key: u64) -> Option<CacheEntry> {
        let entry = self.entries.remove(&key);
        if entry.is_some() {
            if let Some(pos) = self.lru.iter().position(|k| *k == key) {
                self.lru.remove(pos);
            }
        }
        entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a deterministic embedding of dimension `dim` from a seed value.
    fn make_embedding(seed: f64, dim: usize) -> Vec<f64> {
        (0..dim).map(|i| (seed + i as f64).sin()).collect()
    }

    #[test]
    fn test_cache_hit() {
        let mut cache = SemanticCache::new(100);
        let emb = make_embedding(1.0, 16);
        cache.put(emb.clone(), "hello world".to_string());

        let result = cache.get(&emb);
        assert_eq!(result, Some("hello world"));
        assert_eq!(cache.stats.hits, 1);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = SemanticCache::new(100);
        let emb_a = make_embedding(1.0, 16);
        let emb_b = make_embedding(999.0, 16);
        cache.put(emb_a, "hello".to_string());

        let result = cache.get(&emb_b);
        assert_eq!(result, None);
        assert_eq!(cache.stats.misses, 1);
    }

    #[test]
    fn test_eviction() {
        let mut cache = SemanticCache::new(3);
        // Insert 4 entries; the oldest should be evicted.
        let e1 = make_embedding(1.0, 8);
        let e2 = make_embedding(2.0, 8);
        let e3 = make_embedding(3.0, 8);
        let e4 = make_embedding(4.0, 8);

        cache.put(e1.clone(), "a".to_string());
        cache.put(e2.clone(), "b".to_string());
        cache.put(e3.clone(), "c".to_string());
        assert_eq!(cache.len(), 3);

        cache.put(e4.clone(), "d".to_string());
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.stats.evictions, 1);

        // e1 (oldest) should have been evicted
        assert_eq!(cache.get(&e1), None);
        // e2, e3, e4 should still be present
        assert!(cache.get(&e2).is_some());
        assert!(cache.get(&e3).is_some());
        assert!(cache.get(&e4).is_some());
    }

    #[test]
    fn test_threshold_adaptation() {
        let mut cache = SemanticCache::new(100);
        let initial = cache.threshold();

        // Feed a false positive
        cache.feedback(true, false, 0.95);
        assert!(
            (cache.threshold() - initial - 0.01).abs() < 1e-10,
            "expected threshold {:.4}, got {:.4}",
            initial + 0.01,
            cache.threshold()
        );
        assert_eq!(cache.stats.false_positives, 1);

        // Feed two false negatives
        let after_fp = cache.threshold();
        cache.feedback(false, true, 0.90);
        cache.feedback(false, true, 0.90);
        let expected = after_fp - 0.005 - 0.005;
        assert!(
            (cache.threshold() - expected).abs() < 1e-10,
            "expected threshold {:.4}, got {:.4}",
            expected,
            cache.threshold()
        );
        assert_eq!(cache.stats.false_negatives, 2);
    }

    #[test]
    fn test_ttl_expiry() {
        let mut cache = SemanticCache::new(100);
        cache.set_default_ttl(Duration::from_millis(1));
        let emb = make_embedding(42.0, 8);
        cache.put(emb.clone(), "ephemeral".to_string());

        // Should be available immediately
        assert!(cache.get(&emb).is_some());

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(5));

        // Should now be expired
        assert_eq!(cache.get(&emb), None);
    }

    #[test]
    fn test_clear() {
        let mut cache = SemanticCache::new(100);
        cache.put(make_embedding(1.0, 8), "x".to_string());
        cache.put(make_embedding(2.0, 8), "y".to_string());
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.stats.hits, 0);
        assert_eq!(cache.stats.misses, 0);
    }

    #[test]
    fn test_cosine_sim_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = SemanticCache::cosine_sim(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_sim_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = SemanticCache::cosine_sim(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_sim_mismatched_lengths() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        assert_eq!(SemanticCache::cosine_sim(&a, &b), 0.0);
    }

    #[test]
    fn test_update_existing_entry() {
        let mut cache = SemanticCache::new(100);
        let emb = make_embedding(1.0, 8);
        cache.put(emb.clone(), "old".to_string());
        assert_eq!(cache.get(&emb), Some("old"));
        assert_eq!(cache.len(), 1);

        // Update with new response
        cache.put(emb.clone(), "new".to_string());
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&emb), Some("new"));
    }

    #[test]
    fn test_threshold_clamping() {
        let mut cache = SemanticCache::new(100);
        // Push threshold up repeatedly
        for _ in 0..20 {
            cache.feedback(true, false, 0.95);
        }
        assert!(cache.threshold() <= 0.99);

        // Push threshold down repeatedly
        for _ in 0..50 {
            cache.feedback(false, true, 0.50);
        }
        assert!(cache.threshold() >= 0.80);
    }

    #[test]
    fn test_empty_cache_miss() {
        let mut cache = SemanticCache::new(100);
        let emb = make_embedding(1.0, 8);
        assert_eq!(cache.get(&emb), None);
        assert_eq!(cache.stats.misses, 1);
    }
}
