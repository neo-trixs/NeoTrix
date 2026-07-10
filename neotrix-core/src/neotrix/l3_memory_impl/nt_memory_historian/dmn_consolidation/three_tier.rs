#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryTier {
    ShortTerm,
    MediumTerm,
    LongTerm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: usize,
    pub content: String,
    pub importance: f64,
    pub timestamp: u64,
    pub access_count: u32,
    pub last_accessed: u64,
    pub consolidation_age: u64,
    pub tier: MemoryTier,
    pub embedding: Vec<f64>,
}

impl MemoryItem {
    pub fn new(id: usize, content: String, importance: f64, tier: MemoryTier) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            id,
            content,
            importance,
            timestamp: now,
            access_count: 0,
            last_accessed: now,
            consolidation_age: 0,
            tier,
            embedding: Vec::new(),
        }
    }
}

pub struct ThreeTierStore {
    pub items: VecDeque<MemoryItem>,
    pub capacity: usize,
    pub tier: MemoryTier,
    next_id: usize,
}

impl ThreeTierStore {
    pub fn new(capacity: usize, tier: MemoryTier) -> Self {
        Self {
            items: VecDeque::new(),
            capacity,
            tier,
            next_id: 1,
        }
    }

    pub fn push(&mut self, content: String, importance: f64) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let item = MemoryItem::new(id, content, importance, self.tier);
        if self.items.len() >= self.capacity {
            self.items.pop_front();
        }
        self.items.push_back(item);
        id
    }

    pub fn remove(&mut self, id: usize) -> Option<MemoryItem> {
        if let Some(pos) = self.items.iter().position(|item| item.id == id) {
            self.items.remove(pos)
        } else {
            None
        }
    }

    pub fn get(&self, id: usize) -> Option<&MemoryItem> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut MemoryItem> {
        self.items.iter_mut().find(|item| item.id == id)
    }

    pub fn consolidate_candidates(&self, threshold: f64) -> Vec<MemoryItem> {
        self.items
            .iter()
            .filter(|item| item.importance >= threshold)
            .cloned()
            .collect()
    }

    pub fn forget_candidates(&self, threshold: f64) -> Vec<usize> {
        self.items
            .iter()
            .filter(|item| item.importance < threshold)
            .map(|item| item.id)
            .collect()
    }

    pub fn rehearse(&mut self, id: usize) {
        if let Some(item) = self.get_mut(id) {
            item.access_count += 1;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs())
                .unwrap_or(0);
            item.last_accessed = now;
            item.importance = (item.importance + 0.05).max(0.0).min(1.0);
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    pub fn items_slice(&self) -> &VecDeque<MemoryItem> {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_item() {
        let mut store = ThreeTierStore::new(10, MemoryTier::ShortTerm);
        let id = store.push("hello world".to_string(), 0.8);
        assert_eq!(store.len(), 1);
        assert_eq!(id, 1);
        let item = store.get(id).unwrap();
        assert_eq!(item.content, "hello world");
        assert!((item.importance - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_capacity_eviction() {
        let mut store = ThreeTierStore::new(3, MemoryTier::ShortTerm);
        store.push("a".to_string(), 0.1);
        store.push("b".to_string(), 0.2);
        store.push("c".to_string(), 0.3);
        assert_eq!(store.len(), 3);
        store.push("d".to_string(), 0.4);
        assert_eq!(store.len(), 3);
        assert!(store.get(1).is_none());
        assert!(store.get(4).is_some());
    }

    #[test]
    fn test_remove() {
        let mut store = ThreeTierStore::new(10, MemoryTier::ShortTerm);
        let id = store.push("test".to_string(), 0.5);
        let removed = store.remove(id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, id);
        assert!(store.get(id).is_none());
    }

    #[test]
    fn test_get_and_get_mut() {
        let mut store = ThreeTierStore::new(10, MemoryTier::MediumTerm);
        let id = store.push("mut test".to_string(), 0.7);
        let item = store.get(id).unwrap();
        assert_eq!(item.content, "mut test");
        let item_mut = store.get_mut(id).unwrap();
        item_mut.content = "changed".to_string();
        assert_eq!(store.get(id).unwrap().content, "changed");
    }

    #[test]
    fn test_consolidate_candidates() {
        let mut store = ThreeTierStore::new(10, MemoryTier::ShortTerm);
        store.push("low".to_string(), 0.3);
        store.push("high".to_string(), 0.9);
        store.push("mid".to_string(), 0.6);
        let candidates = store.consolidate_candidates(0.6);
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().any(|c| c.content == "high"));
        assert!(candidates.iter().any(|c| c.content == "mid"));
    }

    #[test]
    fn test_forget_candidates() {
        let mut store = ThreeTierStore::new(10, MemoryTier::ShortTerm);
        store.push("keep".to_string(), 0.8);
        store.push("forget".to_string(), 0.2);
        store.push("border".to_string(), 0.5);
        let forget_ids = store.forget_candidates(0.5);
        assert_eq!(forget_ids.len(), 1);
        let item = store.get(forget_ids[0]).unwrap();
        assert_eq!(item.content, "forget");
    }

    #[test]
    fn test_rehearse_boosts_importance() {
        let mut store = ThreeTierStore::new(10, MemoryTier::ShortTerm);
        let id = store.push("rehearse me".to_string(), 0.5);
        let original = store.get(id).unwrap().importance;
        store.rehearse(id);
        let boosted = store.get(id).unwrap().importance;
        assert!((boosted - original - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_rehearse_updates_access_count() {
        let mut store = ThreeTierStore::new(10, MemoryTier::ShortTerm);
        let id = store.push("count me".to_string(), 0.5);
        store.rehearse(id);
        store.rehearse(id);
        assert_eq!(store.get(id).unwrap().access_count, 2);
    }

    #[test]
    fn test_is_full() {
        let mut store = ThreeTierStore::new(2, MemoryTier::ShortTerm);
        assert!(!store.is_full());
        store.push("a".to_string(), 0.1);
        assert!(!store.is_full());
        store.push("b".to_string(), 0.2);
        assert!(store.is_full());
    }

    #[test]
    fn test_items_slice() {
        let mut store = ThreeTierStore::new(5, MemoryTier::LongTerm);
        store.push("x".to_string(), 0.9);
        store.push("y".to_string(), 0.8);
        let slice = store.items_slice();
        assert_eq!(slice.len(), 2);
    }
}
