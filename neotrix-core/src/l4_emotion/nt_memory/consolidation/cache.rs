#![forbid(unsafe_code)]

//! TTL-based memory cache with hit/miss/eviction statistics.
//!
//! Entries expire after `ttl_turns` turns. Manual eviction removes all expired
//! entries. Tracks operational metrics via `CacheStats`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics for cache performance monitoring.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl CacheStats {
    /// Compute hit rate (0.0–1.0). Returns 0.0 if no lookups occurred.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// A single cached entry with TTL tracking.
#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    inserted_turn: u64,
}

/// TTL-based memory cache.
///
/// Each `get` / `put` advances the internal turn counter. Entries older than
/// `ttl_turns` are considered expired and are removed by `evict_expired()`.
#[derive(Debug)]
pub struct MemoryCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
    ttl_turns: u64,
    current_turn: u64,
    stats: CacheStats,
}

impl MemoryCache {
    pub fn new(max_size: usize, ttl_turns: u64) -> Self {
        Self {
            entries: HashMap::with_capacity(max_size),
            max_size,
            ttl_turns,
            current_turn: 0,
            stats: CacheStats::default(),
        }
    }

    /// Retrieve a value by key. Returns `None` if missing or expired.
    /// Advances the turn counter.
    pub fn get(&mut self, key: &str) -> Option<String> {
        self.current_turn += 1;
        let turn = self.current_turn;
        if let Some(entry) = self.entries.get(key) {
            if turn.saturating_sub(entry.inserted_turn) <= self.ttl_turns {
                self.stats.hits += 1;
                Some(entry.value.clone())
            } else {
                self.entries.remove(key);
                self.stats.misses += 1;
                None
            }
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Insert a key-value pair. If at capacity, evicts the oldest entry.
    /// Advances the turn counter.
    pub fn put(&mut self, key: String, value: String) {
        self.current_turn += 1;
        if self.entries.len() >= self.max_size && !self.entries.contains_key(&key) {
            self.evict_oldest();
        }
        self.entries.insert(
            key,
            CacheEntry {
                value,
                inserted_turn: self.current_turn,
            },
        );
    }

    /// Remove all expired entries. Returns the number evicted.
    pub fn evict_expired(&mut self) -> usize {
        let turn = self.current_turn;
        let expired: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, e)| turn.saturating_sub(e.inserted_turn) > self.ttl_turns)
            .map(|(k, _)| k.clone())
            .collect();
        let count = expired.len();
        for key in &expired {
            self.entries.remove(key);
        }
        self.stats.evictions += count as u64;
        count
    }

    /// Return current cache statistics.
    pub fn cache_stats(&self) -> CacheStats {
        self.stats.clone()
    }

    /// Number of live entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Evict the entry with the lowest `inserted_turn` (FIFO).
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.inserted_turn)
            .map(|(k, _)| k.clone())
        {
            self.entries.remove(&oldest_key);
            self.stats.evictions += 1;
        }
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new(100, 50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut c = MemoryCache::new(10, 5);
        c.put("k1".into(), "v1".into());
        assert_eq!(c.get("k1"), Some("v1".to_string()));
    }

    #[test]
    fn test_miss_returns_none() {
        let mut c = MemoryCache::new(10, 5);
        assert_eq!(c.get("missing"), None);
    }

    #[test]
    fn test_ttl_expiration() {
        let mut c = MemoryCache::new(10, 2);
        c.put("k1".into(), "v1".into());
        // Advance past TTL
        let _ = c.get("k2"); // turn 1
        let _ = c.get("k3"); // turn 2
        let _ = c.get("k4"); // turn 3 — k1 inserted at turn 0, now expired
        assert_eq!(c.get("k1"), None);
    }

    #[test]
    fn test_capacity_eviction() {
        let mut c = MemoryCache::new(2, 100);
        c.put("a".into(), "va".into());
        c.put("b".into(), "vb".into());
        c.put("c".into(), "vc".into()); // evicts "a"
        assert_eq!(c.len(), 2);
        assert_eq!(c.get("a"), None);
    }

    #[test]
    fn test_evict_expired() {
        let mut c = MemoryCache::new(10, 1);
        c.put("x".into(), "vx".into());
        let _ = c.get("y"); // advance turn
        let _ = c.get("z"); // advance turn — x now expired
        let evicted = c.evict_expired();
        assert_eq!(evicted, 1);
        assert!(c.is_empty());
    }

    #[test]
    fn test_stats_tracking() {
        let mut c = MemoryCache::new(10, 10);
        c.put("a".into(), "va".into());
        let _ = c.get("a"); // hit
        let _ = c.get("b"); // miss
        let s = c.cache_stats();
        assert_eq!(s.hits, 1);
        assert_eq!(s.misses, 1);
        assert!((s.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hit_rate_zero_lookups() {
        let c = MemoryCache::new(10, 10);
        assert_eq!(c.cache_stats().hit_rate(), 0.0);
    }

    #[test]
    fn test_get_advances_turn() {
        let mut c = MemoryCache::new(10, 2);
        c.put("k".into(), "v".into());
        let _ = c.get("k"); // turn 1
        let _ = c.get("k"); // turn 2 — still within TTL
        assert_eq!(c.get("k"), Some("v".to_string()));
    }

    #[test]
    fn test_overwrite_same_key() {
        let mut c = MemoryCache::new(10, 100);
        c.put("k".into(), "v1".into());
        c.put("k".into(), "v2".into());
        assert_eq!(c.get("k"), Some("v2".to_string()));
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn test_default_cache() {
        let c = MemoryCache::default();
        assert_eq!(c.len(), 0);
        assert!(c.is_empty());
    }
}
