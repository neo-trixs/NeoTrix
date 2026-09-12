//! ANE program cache —专用化编译缓存基础设施
//!
//! Provides LRU cache with TTL eviction for compiled ANE programs,
//! mapping (model_id, target) → compiled binary bytes.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A single entry in the ANE program cache
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Compiled program binary bytes
    pub program_bytes: Vec<u8>,
    /// The target backend this was compiled for
    pub compiled_target: String,
    /// When the entry was created
    pub created_at: Instant,
    /// How many times this entry has been looked up
    pub access_count: u64,
    /// Last access timestamp for LRU ordering
    pub last_access: Instant,
}

impl CacheEntry {
    fn new(program_bytes: Vec<u8>, target: String) -> Self {
        let now = Instant::now();
        Self {
            program_bytes,
            compiled_target: target,
            created_at: now,
            access_count: 0,
            last_access: now,
        }
    }

    fn touch(&mut self) {
        self.access_count += 1;
        self.last_access = Instant::now();
    }
}

/// Cache eviction policy
#[derive(Debug, Clone)]
pub struct CachePolicy {
    /// Maximum number of entries before eviction
    pub max_entries: usize,
    /// Time-to-live for each entry
    pub ttl: Duration,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            max_entries: 128,
            ttl: Duration::from_secs(3600),
        }
    }
}

/// LRU cache with TTL eviction for compiled ANE programs
#[derive(Debug, Clone)]
pub struct AneProgramCache {
    entries: HashMap<(String, String), CacheEntry>,
    policy: CachePolicy,
}

impl AneProgramCache {
    pub fn new(policy: CachePolicy) -> Self {
        Self {
            entries: HashMap::new(),
            policy,
        }
    }

    /// Cache a compiled program for a given model and target.
    /// Returns `true` if the entry was newly inserted.
    pub fn cache_compile(&mut self, model_id: &str, target: &str, program_bytes: Vec<u8>) -> bool {
        self.evict_expired();
        let key = (model_id.to_string(), target.to_string());
        let is_new = !self.entries.contains_key(&key);
        self.entries
            .insert(key, CacheEntry::new(program_bytes, target.to_string()));
        self.evict_lru();
        is_new
    }

    /// Look up a cached program by model_id and target.
    /// Touches the entry on hit (updates access_count + last_access).
    /// Returns `None` if the entry is missing or expired.
    pub fn lookup(&mut self, model_id: &str, target: &str) -> Option<&[u8]> {
        self.evict_expired();
        let key = (model_id.to_string(), target.to_string());
        let entry = self.entries.get_mut(&key)?;
        entry.touch();
        Some(&entry.program_bytes[..])
    }

    /// Check if a cached entry exists (without touching)
    pub fn contains(&self, model_id: &str, target: &str) -> bool {
        let key = (model_id.to_string(), target.to_string());
        self.entries
            .get(&key)
            .is_some_and(|e| e.created_at.elapsed() <= self.policy.ttl)
    }

    /// Remove a specific entry
    pub fn invalidate(&mut self, model_id: &str, target: &str) -> bool {
        let key = (model_id.to_string(), target.to_string());
        self.entries.remove(&key).is_some()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of valid (non-expired) entries
    pub fn len(&self) -> usize {
        let now = Instant::now();
        self.entries
            .values()
            .filter(|e| now.duration_since(e.created_at) <= self.policy.ttl)
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn evict_expired(&mut self) {
        let now = Instant::now();
        self.entries
            .retain(|_, e| now.duration_since(e.created_at) <= self.policy.ttl);
    }

    fn evict_lru(&mut self) {
        while self.entries.len() > self.policy.max_entries {
            let lru_key = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.last_access)
                .map(|(k, _)| k.clone());
            if let Some(key) = lru_key {
                self.entries.remove(&key);
            } else {
                break;
            }
        }
    }
}

impl Default for AneProgramCache {
    fn default() -> Self {
        Self::new(CachePolicy::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_compile_and_lookup() {
        let mut cache = AneProgramCache::default();
        let bytes = vec![0x01, 0x02, 0x03];
        assert!(cache.cache_compile("model_a", "ane", bytes.clone()));
        assert!(!cache.cache_compile("model_a", "ane", bytes)); // overwrite
        let result = cache.lookup("model_a", "ane");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = AneProgramCache::default();
        assert!(cache.lookup("nonexistent", "ane").is_none());
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = AneProgramCache::default();
        cache.cache_compile("m1", "ane", vec![0x00]);
        assert!(cache.contains("m1", "ane"));
        assert!(cache.invalidate("m1", "ane"));
        assert!(!cache.contains("m1", "ane"));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = AneProgramCache::default();
        cache.cache_compile("m1", "ane", vec![0x00]);
        cache.cache_compile("m2", "mlx", vec![0x01]);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_policy_evicts_lru() {
        let policy = CachePolicy {
            max_entries: 2,
            ttl: Duration::from_secs(3600),
        };
        let mut cache = AneProgramCache::new(policy);
        cache.cache_compile("a", "ane", vec![0x00]);
        cache.cache_compile("b", "ane", vec![0x01]);
        cache.cache_compile("c", "ane", vec![0x02]); // evicts "a" (LRU)
        assert!(!cache.contains("a", "ane"));
        assert!(cache.contains("b", "ane"));
        assert!(cache.contains("c", "ane"));
    }

    #[test]
    fn test_access_count_increments() {
        let mut cache = AneProgramCache::default();
        cache.cache_compile("m", "ane", vec![0x00]);
        let _ = cache.lookup("m", "ane");
        let _ = cache.lookup("m", "ane");
        let _ = cache.lookup("m", "ane");
        let key = ("m".to_string(), "ane".to_string());
        assert_eq!(cache.entries.get(&key).unwrap().access_count, 3);
    }
}
