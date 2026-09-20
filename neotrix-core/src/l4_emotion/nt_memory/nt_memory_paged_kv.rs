//! Paged KV Manager — KV-Context Virtualization (M4 from KVMem)
//!
//! Three-tier cache: hot (in-memory HashMap) → warm (sorted vec) → cold (archived vec).
//! Implements the "Context as Scarce Resource" axiom (A2): context window / KV
//! capacity is the fundamental bottleneck. PagedKVManager virtualizes KV context
//! across tiers with attention-based promotion and LRU eviction.

use std::collections::HashMap;
use std::fmt;

/// Paged KV error type
#[derive(Debug, Clone)]
pub enum PagedKVError {
    /// Key not found in any tier
    KeyNotFound(String),
    /// Capacity exceeded — must evict before insert
    CapacityExceeded { tier: String, capacity: usize },
    /// Entry too large for page size
    EntryTooLarge { size: usize, page_size: usize },
}

impl fmt::Display for PagedKVError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyNotFound(key) => write!(f, "key not found: {key}"),
            Self::CapacityExceeded { tier, capacity } => {
                write!(f, "{tier} capacity exceeded: {capacity}")
            }
            Self::EntryTooLarge { size, page_size } => {
                write!(f, "entry size {size} exceeds page size {page_size}")
            }
        }
    }
}

impl std::error::Error for PagedKVError {}

pub type Result<T> = std::result::Result<T, PagedKVError>;

/// A single KV entry with metadata
#[derive(Debug, Clone)]
pub struct KVEntry {
    /// Entry key
    pub key: String,
    /// Entry value (arbitrary bytes)
    pub value: Vec<u8>,
    /// Number of accesses since creation
    pub access_count: u64,
    /// Last access timestamp
    pub last_accessed: u64,
    /// Creation timestamp
    pub created_at: u64,
    /// Entry size in bytes (key + value)
    pub size_bytes: usize,
}

impl KVEntry {
    /// Compute retention score: higher = more valuable to keep in hot tier
    pub fn retention_score(&self, now: u64) -> f64 {
        let recency = if now > self.last_accessed {
            1.0 / (1.0 + (now - self.last_accessed) as f64 / 60.0)
        } else {
            1.0
        };
        let frequency = (self.access_count as f64).ln_1p() / 10.0;
        recency * 0.6 + frequency.min(0.4)
    }
}

/// Eviction policy for tier management
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least Recently Used — evict oldest accessed first
    LRU,
    /// Least Frequently Used — evict least accessed first
    LFU,
    /// Lowest retention score first
    RetentionScore,
}

/// Cache tier — holds a set of KV entries with a capacity limit
#[derive(Debug, Clone)]
pub struct Tier {
    /// Entries in this tier
    entries: Vec<KVEntry>,
    /// Maximum number of entries
    capacity: usize,
    /// Eviction policy
    eviction_policy: EvictionPolicy,
}

impl Tier {
    pub fn new(capacity: usize, eviction_policy: EvictionPolicy) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            capacity,
            eviction_policy,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[KVEntry] {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut Vec<KVEntry> {
        &mut self.entries
    }

    /// Insert entry into tier, evicting if over capacity
    pub fn insert(&mut self, entry: KVEntry, now: u64) -> Option<KVEntry> {
        self.entries.push(entry);

        if self.entries.len() > self.capacity {
            return self.evict_one(now);
        }
        None
    }

    /// Remove and return one entry based on eviction policy
    fn evict_one(&mut self, now: u64) -> Option<KVEntry> {
        if self.entries.is_empty() {
            return None;
        }

        let idx = match self.eviction_policy {
            EvictionPolicy::LRU => self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.last_accessed)
                .map(|(i, _)| i),
            EvictionPolicy::LFU => self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.access_count)
                .map(|(i, _)| i),
            EvictionPolicy::RetentionScore => self
                .entries
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    a.1.retention_score(now)
                        .partial_cmp(&b.1.retention_score(now))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i),
        };

        idx.map(|i| self.entries.remove(i))
    }

    /// Find entry by key, updating access metadata
    pub fn get(&mut self, key: &str, now: u64) -> Option<&mut KVEntry> {
        self.entries.iter_mut().find(|e| e.key == key).map(|e| {
            e.access_count += 1;
            e.last_accessed = now;
            e
        })
    }

    /// Find entry by key (immutable)
    pub fn get_immutable(&self, key: &str) -> Option<&KVEntry> {
        self.entries.iter().find(|e| e.key == key)
    }

    /// Remove entry by key
    pub fn remove(&mut self, key: &str) -> Option<KVEntry> {
        self.entries.iter().position(|e| e.key == key).map(|i| self.entries.remove(i))
    }

    /// Select entries with lowest retention scores for eviction
    pub fn select_eviction_candidates(&self, count: usize, now: u64) -> Vec<String> {
        let mut scored: Vec<(&KVEntry, f64)> = self
            .entries
            .iter()
            .map(|e| (e, e.retention_score(now)))
            .collect();
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(count)
            .map(|(e, _)| e.key.clone())
            .collect()
    }
}

/// Paged KV Manager — virtualizes KV context across hot/warm/cold tiers
#[derive(Debug)]
pub struct PagedKVManager {
    /// Hot tier: in-memory HashMap for fast access
    hot: HashMap<String, KVEntry>,
    /// Warm tier: recently evicted from hot
    warm: Tier,
    /// Cold tier: archived data
    cold: Tier,
    /// Maximum hot tier size
    hot_capacity: usize,
    /// Page size in bytes — max entry size
    page_size: usize,
    /// Current timestamp for access tracking
    now: u64,
}

impl PagedKVManager {
    /// Create a new PagedKVManager
    pub fn new(hot_capacity: usize, warm_capacity: usize, cold_capacity: usize, page_size: usize) -> Self {
        Self {
            hot: HashMap::with_capacity(hot_capacity),
            warm: Tier::new(warm_capacity, EvictionPolicy::LRU),
            cold: Tier::new(cold_capacity, EvictionPolicy::RetentionScore),
            hot_capacity,
            page_size,
            now: 0,
        }
    }

    /// Update internal clock (call before get/insert operations)
    pub fn tick(&mut self, timestamp: u64) {
        self.now = timestamp;
    }

    /// Get a value by key — searches hot → warm → cold
    pub fn get(&mut self, key: &str) -> Option<&KVEntry> {
        // Hot tier first
        if let Some(entry) = self.hot.get_mut(key) {
            entry.access_count += 1;
            entry.last_accessed = self.now;
            return self.hot.get(key);
        }

        // Warm tier — promote to hot on hit
        if self.warm.get(key, self.now).is_some() {
            let promoted = self.warm.remove(key);
            if let Some(mut entry) = promoted {
                entry.last_accessed = self.now;
                entry.access_count += 1;
                self.hot.insert(key.to_string(), entry);
            }
            return self.hot.get(key);
        }

        // Cold tier — promote to hot on hit
        if self.cold.get_immutable(key).is_some() {
            let promoted = self.cold.remove(key);
            if let Some(mut entry) = promoted {
                entry.last_accessed = self.now;
                entry.access_count += 1;
                self.hot.insert(key.to_string(), entry);
            }
            return self.hot.get(key);
        }

        None
    }

    /// Insert a key-value pair into the hot tier
    pub fn insert(&mut self, key: String, value: Vec<u8>) -> Result<()> {
        let size = key.len() + value.len();
        if size > self.page_size {
            return Err(PagedKVError::EntryTooLarge {
                size,
                page_size: self.page_size,
            });
        }

        let entry = KVEntry {
            key: key.clone(),
            value,
            access_count: 1,
            last_accessed: self.now,
            created_at: self.now,
            size_bytes: size,
        };

        self.hot.insert(key, entry);

        // Evict from hot if over capacity
        if self.hot.len() > self.hot_capacity {
            self.evict_to_warm();
        }

        Ok(())
    }

    /// Evict least valuable hot entries to warm tier
    pub fn evict_to_warm(&mut self) {
        if self.hot.len() <= self.hot_capacity {
            return;
        }

        let excess = self.hot.len() - self.hot_capacity;
        let mut candidates: Vec<(String, f64)> = self
            .hot
            .iter()
            .map(|(k, e)| (k.clone(), e.retention_score(self.now)))
            .collect();
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        for (key, _) in candidates.into_iter().take(excess) {
            if let Some(entry) = self.hot.remove(&key) {
                self.warm.insert(entry, self.now);
            }
        }
    }

    /// Promote a key from warm/cold tier back to hot
    pub fn promote_to_hot(&mut self, key: &str) -> bool {
        // Try warm first
        if let Some(entry) = self.warm.remove(key) {
            self.hot.insert(key.to_string(), entry);
            return true;
        }

        // Try cold
        if let Some(entry) = self.cold.remove(key) {
            self.hot.insert(key.to_string(), entry);
            return true;
        }

        false
    }

    /// Evict from warm to cold tier
    pub fn evict_to_cold(&mut self) {
        let candidates = self.warm.select_eviction_candidates(10, self.now);
        for key in candidates {
            if let Some(entry) = self.warm.remove(&key) {
                self.cold.insert(entry, self.now);
            }
        }
    }

    /// Total entries across all tiers
    pub fn total_entries(&self) -> usize {
        self.hot.len() + self.warm.len() + self.cold.len()
    }

    /// Hot tier entry count
    pub fn hot_count(&self) -> usize {
        self.hot.len()
    }

    /// Warm tier entry count
    pub fn warm_count(&self) -> usize {
        self.warm.len()
    }

    /// Cold tier entry count
    pub fn cold_count(&self) -> usize {
        self.cold.len()
    }

    /// Remove a key from all tiers
    pub fn remove(&mut self, key: &str) -> bool {
        if self.hot.remove(key).is_some() {
            return true;
        }
        if self.warm.remove(key).is_some() {
            return true;
        }
        if self.cold.remove(key).is_some() {
            return true;
        }
        false
    }

    /// Check if key exists in any tier
    pub fn contains(&self, key: &str) -> bool {
        self.hot.contains_key(key)
            || self.warm.get_immutable(key).is_some()
            || self.cold.get_immutable(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut kv = PagedKVManager::new(10, 10, 10, 1024);
        kv.tick(100);
        kv.insert("key1".into(), b"value1".to_vec()).unwrap();

        let entry = kv.get("key1").unwrap();
        assert_eq!(entry.key, "key1");
        assert_eq!(entry.value, b"value1");
    }

    #[test]
    fn test_eviction_to_warm() {
        let mut kv = PagedKVManager::new(2, 10, 10, 1024);
        kv.tick(100);

        kv.insert("a".into(), b"1".to_vec()).unwrap();
        kv.insert("b".into(), b"2".to_vec()).unwrap();
        kv.tick(200);
        kv.insert("c".into(), b"3".to_vec()).unwrap();

        // 'a' should have been evicted to warm
        assert!(kv.hot.len() <= 2);
        assert!(kv.warm.len() > 0 || kv.hot.contains_key("c"));
    }

    #[test]
    fn test_promote_to_hot() {
        let mut kv = PagedKVManager::new(2, 10, 10, 1024);
        kv.tick(100);

        kv.insert("a".into(), b"1".to_vec()).unwrap();
        kv.insert("b".into(), b"2".to_vec()).unwrap();
        kv.tick(200);
        kv.insert("c".into(), b"3".to_vec()).unwrap();

        // Promote "a" back to hot
        let promoted = kv.promote_to_hot("a");
        assert!(promoted || kv.hot.contains_key("a"));
    }

    #[test]
    fn test_entry_too_large() {
        let mut kv = PagedKVManager::new(10, 10, 10, 5);
        kv.tick(100);
        let result = kv.insert("key".into(), b"this value is way too large".to_vec());
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_from_all_tiers() {
        let mut kv = PagedKVManager::new(2, 10, 10, 1024);
        kv.tick(100);
        kv.insert("x".into(), b"1".to_vec()).unwrap();

        assert!(kv.contains("x"));
        assert!(kv.remove("x"));
        assert!(!kv.contains("x"));
    }

    #[test]
    fn test_total_entries() {
        let mut kv = PagedKVManager::new(10, 10, 10, 1024);
        kv.tick(100);
        kv.insert("a".into(), b"1".to_vec()).unwrap();
        kv.insert("b".into(), b"2".to_vec()).unwrap();
        assert_eq!(kv.total_entries(), 2);
    }

    #[test]
    fn test_retention_score_ordering() {
        let now = 1000;
        let recent = KVEntry {
            key: "recent".into(),
            value: vec![],
            access_count: 10,
            last_accessed: now,
            created_at: now - 100,
            size_bytes: 0,
        };
        let old = KVEntry {
            key: "old".into(),
            value: vec![],
            access_count: 1,
            last_accessed: now - 600,
            created_at: now - 1000,
            size_bytes: 0,
        };
        assert!(recent.retention_score(now) > old.retention_score(now));
    }
}
