#![forbid(unsafe_code)]

//! Working memory — fixed-capacity LRU-like buffer for recent interactions.
//!
//! When capacity is exceeded, the least recently recalled item is evicted.
//! Provides `recall(query)` for substring matching and `decay()` to age all items.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A lightweight reference into working memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRef {
    pub id: String,
    pub content: String,
    pub last_recalled_at: u64,
    pub recall_count: u64,
}

/// Configuration for working memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryConfig {
    pub max_size: usize,
}

impl Default for WorkingMemoryConfig {
    fn default() -> Self {
        Self { max_size: 50 }
    }
}

/// Fixed-capacity working memory with LRU-like eviction.
///
/// Items are stored in insertion order. Recall operations update `last_recalled_at`.
/// When full, the item with the oldest `last_recalled_at` is evicted on insert.
#[derive(Debug)]
pub struct WorkingMemory {
    items: VecDeque<MemoryRef>,
    max_size: usize,
    turn_counter: u64,
}

impl WorkingMemory {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(max_size),
            max_size,
            turn_counter: 0,
        }
    }

    pub fn with_config(config: WorkingMemoryConfig) -> Self {
        Self::new(config.max_size)
    }

    /// Add an item to working memory. Evicts LRU if at capacity.
    pub fn add(&mut self, item: MemoryRef) {
        if self.items.len() >= self.max_size {
            self.evict_lru();
        }
        self.items.push_back(item);
    }

    /// Recall items whose content matches the query (case-insensitive substring).
    /// Matching items have their `last_recalled_at` bumped to the current turn.
    pub fn recall(&mut self, query: &str) -> Vec<MemoryRef> {
        let q = query.to_lowercase();
        let matches: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.content.to_lowercase().contains(&q))
            .map(|(idx, _)| idx)
            .collect();

        self.turn_counter += 1;
        let turn = self.turn_counter;

        let mut results = Vec::new();
        for idx in &matches {
            if let Some(item) = self.items.get_mut(*idx) {
                item.last_recalled_at = turn;
                item.recall_count += 1;
                results.push(item.clone());
            }
        }
        results
    }

    /// Advance the turn counter and decay all items by bumping `last_recalled_at`
    /// backwards (simulates forgetting). Items that haven't been recalled in a while
    /// will naturally be evicted first.
    pub fn decay(&mut self) {
        self.turn_counter += 1;
    }

    /// Snapshot current items (read-only, ordered by insertion).
    pub fn get_items(&self) -> Vec<MemoryRef> {
        self.items.iter().cloned().collect()
    }

    /// Current number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the working memory is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Evict the item with the oldest `last_recalled_at` (LRU policy).
    fn evict_lru(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let lru_idx = self
            .items
            .iter()
            .enumerate()
            .min_by_key(|(_, item)| item.last_recalled_at)
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        self.items.remove(lru_idx);
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new(50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_ref(id: &str) -> MemoryRef {
        MemoryRef {
            id: id.to_string(),
            content: format!("content for {id}"),
            last_recalled_at: 0,
            recall_count: 0,
        }
    }

    #[test]
    fn test_add_and_get_items() {
        let mut wm = WorkingMemory::new(3);
        wm.add(mem_ref("a"));
        wm.add(mem_ref("b"));
        assert_eq!(wm.len(), 2);
        let items = wm.get_items();
        assert_eq!(items[0].id, "a");
        assert_eq!(items[1].id, "b");
    }

    #[test]
    fn test_lru_eviction() {
        let mut wm = WorkingMemory::new(2);
        wm.add(mem_ref("a"));
        wm.add(mem_ref("b"));
        wm.add(mem_ref("c")); // evicts "a" (oldest recalled)
        assert_eq!(wm.len(), 2);
        let ids: Vec<String> = wm.get_items().iter().map(|i| i.id.clone()).collect();
        assert!(ids.contains(&"b".to_string()));
        assert!(ids.contains(&"c".to_string()));
        assert!(!ids.contains(&"a".to_string()));
    }

    #[test]
    fn test_recall_updates_lru() {
        let mut wm = WorkingMemory::new(2);
        wm.add(mem_ref("a"));
        wm.add(mem_ref("b"));
        // Recall "a" so it becomes recently used
        let _ = wm.recall("a");
        wm.add(mem_ref("c")); // should evict "b" (now least recently recalled)
        let ids: Vec<String> = wm.get_items().iter().map(|i| i.id.clone()).collect();
        assert!(ids.contains(&"a".to_string()));
        assert!(ids.contains(&"c".to_string()));
        assert!(!ids.contains(&"b".to_string()));
    }

    #[test]
    fn test_recall_returns_matches() {
        let mut wm = WorkingMemory::new(5);
        wm.add(MemoryRef {
            id: "1".into(),
            content: "Rust memory safety".into(),
            last_recalled_at: 0,
            recall_count: 0,
        });
        wm.add(MemoryRef {
            id: "2".into(),
            content: "Python asyncio".into(),
            last_recalled_at: 0,
            recall_count: 0,
        });
        let results = wm.recall("rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "1");
    }

    #[test]
    fn test_decay_increments_turn() {
        let mut wm = WorkingMemory::new(3);
        wm.add(mem_ref("x"));
        wm.decay();
        let items = wm.get_items();
        assert_eq!(items[0].recall_count, 0);
    }

    #[test]
    fn test_empty_working_memory() {
        let mut wm = WorkingMemory::new(5);
        assert!(wm.is_empty());
        assert!(wm.recall("anything").is_empty());
    }

    #[test]
    fn test_default_config() {
        let config = WorkingMemoryConfig::default();
        assert_eq!(config.max_size, 50);
    }

    #[test]
    fn test_with_config() {
        let mut wm = WorkingMemory::with_config(WorkingMemoryConfig { max_size: 2 });
        wm.add(mem_ref("x"));
        wm.add(mem_ref("y"));
        wm.add(mem_ref("z"));
        assert_eq!(wm.len(), 2);
    }
}
