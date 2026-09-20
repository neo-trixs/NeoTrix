#![forbid(unsafe_code)]

//! Short-Term Store — TTL 7 days, max 1000, LRU eviction.
//!
//! Tier 3 of the five-tier cascade. Holds memories that survived working
//! memory but haven't yet been compressed into episodic summaries.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A short-term memory entry with LRU tracking.
#[derive(Debug, Clone)]
pub struct ShortTermEntry {
    pub id: u64,
    pub content: String,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u32,
    pub attention_weight: f64,
}

/// ShortTermStore: TTL 7 days, max 1000, LRU eviction.
#[derive(Debug)]
pub struct ShortTermStore {
    entries: HashMap<u64, ShortTermEntry>,
    ttl: Duration,
    max_capacity: usize,
    next_id: u64,
}

impl Default for ShortTermStore {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            ttl: Duration::from_secs(7 * 24 * 3600),
            max_capacity: 1000,
            next_id: 1,
        }
    }
}

impl ShortTermStore {
    pub fn new(ttl: Duration, max_capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
            max_capacity,
            next_id: 1,
        }
    }

    /// Store a new entry. Evicts LRU if at capacity.
    pub fn store(&mut self, content: String, attention_weight: f64) -> u64 {
        self.evict_expired();
        if self.entries.len() >= self.max_capacity {
            self.evict_lru();
        }

        let id = self.next_id;
        self.next_id += 1;
        let now = Instant::now();

        self.entries.insert(
            id,
            ShortTermEntry {
                id,
                content,
                created_at: now,
                last_accessed: now,
                access_count: 0,
                attention_weight,
            },
        );
        id
    }

    /// Recall an entry by id, updating LRU timestamp.
    pub fn recall(&mut self, id: u64) -> Option<&ShortTermEntry> {
        self.evict_expired();
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            Some(entry)
        } else {
            None
        }
    }

    /// Recall an entry by id (immutable view).
    pub fn peek(&self, id: u64) -> Option<&ShortTermEntry> {
        self.entries.get(&id)
    }

    /// Return all entries sorted by last_accessed descending.
    pub fn all_by_recency(&self) -> Vec<&ShortTermEntry> {
        let mut items: Vec<&ShortTermEntry> = self.entries.values().collect();
        items.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        items
    }

    /// Promote entries above attention threshold out of short-term.
    pub fn promote_candidates(&self, threshold: f64) -> Vec<&ShortTermEntry> {
        self.entries
            .values()
            .filter(|e| e.attention_weight >= threshold)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn evict_expired(&mut self) {
        let now = Instant::now();
        self.entries
            .retain(|_, e| now.duration_since(e.created_at) <= self.ttl);
    }

    fn evict_lru(&mut self) {
        if let Some((min_id, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.last_accessed)
        {
            let min_id = *min_id;
            self.entries.remove(&min_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_recall() {
        let mut st = ShortTermStore::default();
        let id = st.store("hello".into(), 0.5);
        let entry = st.recall(id).unwrap();
        assert_eq!(entry.content, "hello");
        assert_eq!(entry.access_count, 1);
    }

    #[test]
    fn lru_eviction() {
        let mut st = ShortTermStore::new(Duration::from_secs(3600), 2);
        let id1 = st.store("a".into(), 0.5);
        st.store("b".into(), 0.5);
        st.recall(id1);
        st.store("c".into(), 0.5);
        assert!(st.peek(id1).is_some());
        assert!(st.peek(2).is_none());
    }

    #[test]
    fn promote_candidates() {
        let mut st = ShortTermStore::default();
        st.store("low".into(), 0.2);
        st.store("high".into(), 0.9);
        let candidates = st.promote_candidates(0.5);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].content, "high");
    }
}
