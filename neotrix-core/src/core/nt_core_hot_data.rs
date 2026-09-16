//! Hot data paths — safe mmap-inspired patterns for frequently accessed data.
//!
//! Inspired by Qingjian's .qj format (#[repr(C)] + zerocopy), but implemented
//! entirely in safe Rust. Provides O(1) lookups, batch operations, and
//! cosine-similarity vector search for VSA embeddings and salience weights.

use std::collections::HashMap;
use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// A fixed-layout record stored contiguously in memory.
/// Mirrors Qingjian's zerocopy pattern without requiring unsafe.
#[derive(Debug, Clone)]
pub struct HotRecord {
    pub key: String,
    pub value: Vec<f32>,
    pub score: f64,
    pub timestamp: i64,
}

/// In-memory hot data store with O(1) lookup.
/// Designed for frequently accessed data (VSA vectors, salience weights).
pub struct HotDataStore {
    index: HashMap<String, usize>,
    records: Vec<HotRecord>,
    max_capacity: usize,
    lru_order: VecDeque<usize>,
}

/// Snapshot of store utilization.
#[derive(Debug)]
pub struct StoreStats {
    pub total: usize,
    pub capacity: usize,
    pub utilization: f64,
}

/// Errors produced by store operations.
#[derive(Debug)]
pub enum HotDataError {
    CapacityExceeded,
    KeyNotFound,
}

// ---------------------------------------------------------------------------
// BatchLookup trait
// ---------------------------------------------------------------------------

/// Efficient batch operations over hot data.
pub trait BatchLookup {
    fn get_batch(&self, keys: &[&str]) -> Vec<Option<&HotRecord>>;
    fn insert_batch(&mut self, records: Vec<HotRecord>) -> Result<usize, HotDataError>;
    fn remove_batch(&mut self, keys: &[&str]) -> Vec<Option<HotRecord>>;
}

// ---------------------------------------------------------------------------
// HotDataStore implementation
// ---------------------------------------------------------------------------

impl HotDataStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            index: HashMap::new(),
            records: Vec::with_capacity(capacity),
            max_capacity: capacity,
            lru_order: VecDeque::new(),
        }
    }

    pub fn insert(&mut self, record: HotRecord) -> Result<(), HotDataError> {
        if self.records.len() >= self.max_capacity && !self.index.contains_key(&record.key) {
            return Err(HotDataError::CapacityExceeded);
        }

        if let Some(&pos) = self.index.get(&record.key) {
            self.records[pos] = record;
            self.touch_lru(pos);
            return Ok(());
        }

        let pos = self.records.len();
        self.index.insert(record.key.clone(), pos);
        self.records.push(record);
        self.lru_order.push_back(pos);
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Option<&HotRecord> {
        let pos = *self.index.get(key)?;
        self.touch_lru(pos);
        Some(&self.records[pos])
    }

    pub fn remove(&mut self, key: &str) -> Option<HotRecord> {
        let pos = self.index.remove(key)?;
        self.lru_order.retain(|&p| p != pos);

        let last = self.records.len() - 1;
        if pos == last {
            // Simple case: last element, just pop it.
            return self.records.pop();
        }

        // Swap with last, then pop — the record we want is what swap_remove returns.
        let removed = self.records.swap_remove(pos);

        // Re-index the element that moved from `last` into `pos`.
        let moved_key = self.records[pos].key.clone();
        self.index.insert(moved_key, pos);
        for p in self.lru_order.iter_mut() {
            if *p == last {
                *p = pos;
            }
        }

        Some(removed)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn evict_lru(&mut self) -> Option<HotRecord> {
        let pos = self.lru_order.pop_front()?;
        self.index.remove(&self.records[pos].key);

        let last = self.records.len() - 1;
        if pos != last {
            let removed = self.records.swap_remove(pos);
            // Update index and lru_order for the swapped element
            if pos < self.records.len() {
                let moved_key = self.records[pos].key.clone();
                self.index.insert(moved_key, pos);
                for p in self.lru_order.iter_mut() {
                    if *p == last {
                        *p = pos;
                    }
                }
            }
            Some(removed)
        } else {
            self.records.pop()
        }
    }

    pub fn stats(&self) -> StoreStats {
        let total = self.records.len();
        StoreStats {
            total,
            capacity: self.max_capacity,
            utilization: total as f64 / self.max_capacity as f64,
        }
    }

    fn touch_lru(&mut self, pos: usize) {
        if let Some(idx) = self.lru_order.iter().position(|&p| p == pos) {
            self.lru_order.remove(idx);
        }
        self.lru_order.push_back(pos);
    }
}

impl BatchLookup for HotDataStore {
    fn get_batch(&self, keys: &[&str]) -> Vec<Option<&HotRecord>> {
        // Direct index lookup without LRU touch (batch reads are typically cold scans)
        keys.iter()
            .map(|k| self.index.get(*k).map(|&pos| &self.records[pos]))
            .collect()
    }

    fn insert_batch(&mut self, records: Vec<HotRecord>) -> Result<usize, HotDataError> {
        let mut count = 0;
        for r in records {
            self.insert(r)?;
            count += 1;
        }
        Ok(count)
    }

    fn remove_batch(&mut self, keys: &[&str]) -> Vec<Option<HotRecord>> {
        keys.iter().map(|k| self.remove(k)).collect()
    }
}

// ---------------------------------------------------------------------------
// HotVectorStore — cosine similarity search
// ---------------------------------------------------------------------------

/// Stores embedding vectors and supports nearest-neighbor queries via cosine
/// similarity. Useful for VSA HyperCube lookups in hot paths.
pub struct HotVectorStore {
    records: Vec<HotRecord>,
    index: HashMap<String, usize>,
}

impl HotVectorStore {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn insert(&mut self, record: HotRecord) {
        if let Some(&pos) = self.index.get(&record.key) {
            self.records[pos] = record;
            return;
        }
        let pos = self.records.len();
        self.index.insert(record.key.clone(), pos);
        self.records.push(record);
    }

    pub fn get(&self, key: &str) -> Option<&HotRecord> {
        self.index.get(key).map(|&pos| &self.records[pos])
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Return up to `k` records most similar to `query_vec` by cosine similarity.
    pub fn query_similar(&self, query_vec: &[f32], k: usize) -> Vec<(&HotRecord, f64)> {
        if query_vec.is_empty() || self.records.is_empty() {
            return Vec::new();
        }

        let query_norm = norm(query_vec);
        if query_norm == 0.0 {
            return Vec::new();
        }

        let mut scored: Vec<(&HotRecord, f64)> = self
            .records
            .iter()
            .map(|r| {
                let sim = cosine_similarity(query_vec, &r.value, query_norm);
                (r, sim)
            })
            .collect();

        // Partial sort: only need top k
        if k < scored.len() {
            scored.select_nth_unstable_by(k - 1, |a, b| b.1.partial_cmp(&a.1).unwrap());
            scored.truncate(k);
        } else {
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        }
        scored
    }
}

fn norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

fn cosine_similarity(a: &[f32], b: &[f32], a_norm: f32) -> f64 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let b_norm = norm(b);
    if a_norm == 0.0 || b_norm == 0.0 {
        0.0
    } else {
        (dot / (a_norm * b_norm)) as f64
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(key: &str, dim: usize) -> HotRecord {
        HotRecord {
            key: key.to_string(),
            value: (0..dim).map(|i| i as f32).collect(),
            score: 1.0,
            timestamp: 1000,
        }
    }

    #[test]
    fn insert_and_get() {
        let mut store = HotDataStore::new(10);
        store.insert(make_record("a", 4)).unwrap();
        assert!(store.get("a").is_some());
        assert!(store.get("missing").is_none());
    }

    #[test]
    fn insert_duplicate_updates() {
        let mut store = HotDataStore::new(10);
        store.insert(make_record("a", 4)).unwrap();
        let mut updated = make_record("a", 4);
        updated.score = 99.0;
        store.insert(updated).unwrap();
        assert_eq!(store.get("a").unwrap().score, 99.0);
    }

    #[test]
    fn capacity_exceeded() {
        let mut store = HotDataStore::new(2);
        store.insert(make_record("a", 4)).unwrap();
        store.insert(make_record("b", 4)).unwrap();
        assert!(store.insert(make_record("c", 4)).is_err());
    }

    #[test]
    fn capacity_ok_after_eviction() {
        let mut store = HotDataStore::new(2);
        store.insert(make_record("a", 4)).unwrap();
        store.insert(make_record("b", 4)).unwrap();
        store.evict_lru();
        assert!(store.insert(make_record("c", 4)).is_ok());
    }

    #[test]
    fn remove_existing() {
        let mut store = HotDataStore::new(10);
        store.insert(make_record("a", 4)).unwrap();
        let removed = store.remove("a");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().key, "a");
        assert!(store.get("a").is_none());
    }

    #[test]
    fn remove_nonexistent() {
        let mut store = HotDataStore::new(10);
        assert!(store.remove("ghost").is_none());
    }

    #[test]
    fn lru_eviction_order() {
        let mut store = HotDataStore::new(3);
        store.insert(make_record("a", 1)).unwrap();
        store.insert(make_record("b", 1)).unwrap();
        store.insert(make_record("c", 1)).unwrap();

        // Touch b to make it most recent
        store.get("b");

        let evicted = store.evict_lru();
        assert_eq!(evicted.unwrap().key, "a"); // oldest untouched

        let evicted = store.evict_lru();
        assert_eq!(evicted.unwrap().key, "c");

        let evicted = store.evict_lru();
        assert_eq!(evicted.unwrap().key, "b");
    }

    #[test]
    fn stats_utilization() {
        let mut store = HotDataStore::new(100);
        assert_eq!(store.stats().total, 0);
        store.insert(make_record("x", 4)).unwrap();
        let s = store.stats();
        assert_eq!(s.total, 1);
        assert_eq!(s.capacity, 100);
        assert!((s.utilization - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn len_is_empty() {
        let mut store = HotDataStore::new(5);
        assert!(store.is_empty());
        store.insert(make_record("a", 4)).unwrap();
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }

    // BatchLookup tests

    #[test]
    fn batch_get() {
        let mut store = HotDataStore::new(10);
        store.insert(make_record("a", 4)).unwrap();
        store.insert(make_record("b", 4)).unwrap();

        let results = BatchLookup::get_batch(&store, &["a", "c", "b"]);
        assert!(results[0].is_some());
        assert!(results[1].is_none());
        assert!(results[2].is_some());
    }

    #[test]
    fn batch_insert() {
        let mut store = HotDataStore::new(10);
        let records = vec![make_record("a", 4), make_record("b", 4)];
        let count = BatchLookup::insert_batch(&mut store, records).unwrap();
        assert_eq!(count, 2);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn batch_remove() {
        let mut store = HotDataStore::new(10);
        store.insert(make_record("a", 4)).unwrap();
        store.insert(make_record("b", 4)).unwrap();

        let removed = BatchLookup::remove_batch(&mut store, &["a", "z"]);
        assert_eq!(removed.len(), 2);
        assert!(removed[0].is_some());
        assert!(removed[1].is_none());
        assert_eq!(store.len(), 1);
    }

    // HotVectorStore tests

    #[test]
    fn vector_store_insert_and_get() {
        let mut vs = HotVectorStore::new();
        vs.insert(make_record("v1", 3));
        assert!(vs.get("v1").is_some());
        assert!(vs.get("missing").is_none());
        assert_eq!(vs.len(), 1);
    }

    #[test]
    fn vector_store_query_similar() {
        let mut vs = HotVectorStore::new();
        vs.insert(HotRecord {
            key: "a".into(),
            value: vec![1.0, 0.0, 0.0],
            score: 0.0,
            timestamp: 0,
        });
        vs.insert(HotRecord {
            key: "b".into(),
            value: vec![0.0, 1.0, 0.0],
            score: 0.0,
            timestamp: 0,
        });
        vs.insert(HotRecord {
            key: "c".into(),
            value: vec![0.9, 0.1, 0.0],
            score: 0.0,
            timestamp: 0,
        });

        let results = vs.query_similar(&[1.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0.key, "a"); // exact match
        assert!(results[0].1 > results[1].1); // a > c
    }

    #[test]
    fn vector_store_empty_query() {
        let vs = HotVectorStore::new();
        let results = vs.query_similar(&[], 5);
        assert!(results.is_empty());
    }

    #[test]
    fn vector_store_zero_norm_query() {
        let mut vs = HotVectorStore::new();
        vs.insert(make_record("x", 3));
        let results = vs.query_similar(&[0.0, 0.0, 0.0], 5);
        assert!(results.is_empty());
    }

    #[test]
    fn vector_store_upsert() {
        let mut vs = HotVectorStore::new();
        vs.insert(make_record("k", 3));
        let mut updated = make_record("k", 3);
        updated.score = 42.0;
        vs.insert(updated);
        assert_eq!(vs.get("k").unwrap().score, 42.0);
        assert_eq!(vs.len(), 1);
    }
}
