//! Selective Memory — importance-based selective forgetting
//!
//! Implements the "Context as Scarce Resource" axiom (KVMem): context window /
//! KV capacity is the fundamental bottleneck. SelectiveMemory enforces a
//! capacity budget by retaining only items above an importance threshold,
//! evicting the least important first.

use serde::{Deserialize, Serialize};

/// A single memory item with importance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    /// Importance score in [0.0, 1.0] — higher = more important
    pub importance: f64,
    /// Access recency: steps since last access (0 = just accessed)
    pub recency: u64,
    /// Number of times this item was accessed
    pub access_count: u64,
}

impl MemoryItem {
    pub fn new(id: impl Into<String>, content: impl Into<String>, importance: f64) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            importance: importance.clamp(0.0, 1.0),
            recency: 0,
            access_count: 0,
        }
    }

    /// Composite score: importance weighted by recency decay and access frequency
    pub fn retention_score(&self) -> f64 {
        let recency_factor = 1.0 / (1.0 + self.recency as f64);
        let access_factor = (self.access_count as f64).ln_1p() / 10.0;
        self.importance * 0.7 + recency_factor * 0.2 + access_factor.min(0.1)
    }
}

/// Selective memory manager — enforces capacity via importance-based eviction
#[derive(Debug, Clone)]
pub struct SelectiveMemory {
    /// Maximum number of items to retain
    capacity: usize,
    /// Minimum importance threshold — items below this are always evicted
    importance_threshold: f64,
}

impl SelectiveMemory {
    pub fn new(capacity: usize, importance_threshold: f64) -> Self {
        Self {
            capacity,
            importance_threshold: importance_threshold.clamp(0.0, 1.0),
        }
    }

    /// Check if an item should be remembered (above threshold)
    pub fn should_remember(&self, item: &MemoryItem) -> bool {
        item.importance > self.importance_threshold
    }

    /// Evict items that fall below threshold, then trim to capacity
    /// by removing lowest retention_score items.
    pub fn forget(&self, items: &mut Vec<MemoryItem>) {
        // Phase 1: remove below-threshold items
        items.retain(|i| self.should_remember(i));

        // Phase 2: if still over capacity, sort by retention score and truncate
        if items.len() > self.capacity {
            items.sort_by(|a, b| {
                b.retention_score()
                    .partial_cmp(&a.retention_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            items.truncate(self.capacity);
        }
    }

    /// Batch insert with automatic eviction
    pub fn insert_many(&self, items: &mut Vec<MemoryItem>, new_items: Vec<MemoryItem>) {
        items.extend(new_items);
        self.forget(items);
    }

    /// Touch an item to update its recency and access count
    pub fn touch(item: &mut MemoryItem) {
        item.recency = 0;
        item.access_count += 1;
    }

    /// Age all items by incrementing recency
    pub fn age_all(items: &mut [MemoryItem]) {
        for item in items {
            item.recency = item.recency.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_remember_above_threshold() {
        let mem = SelectiveMemory::new(100, 0.3);
        let item = MemoryItem::new("1", "content", 0.5);
        assert!(mem.should_remember(&item));
    }

    #[test]
    fn test_should_forget_below_threshold() {
        let mem = SelectiveMemory::new(100, 0.5);
        let item = MemoryItem::new("1", "content", 0.1);
        assert!(!mem.should_remember(&item));
    }

    #[test]
    fn test_forget_removes_low_importance() {
        let mem = SelectiveMemory::new(100, 0.3);
        let mut items = vec![
            MemoryItem::new("1", "low", 0.1),
            MemoryItem::new("2", "mid", 0.5),
            MemoryItem::new("3", "high", 0.9),
        ];
        mem.forget(&mut items);
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.importance > 0.3));
    }

    #[test]
    fn test_forget_truncates_to_capacity() {
        let mem = SelectiveMemory::new(2, 0.0);
        let mut items = vec![
            MemoryItem::new("1", "a", 0.9),
            MemoryItem::new("2", "b", 0.5),
            MemoryItem::new("3", "c", 0.3),
        ];
        mem.forget(&mut items);
        assert_eq!(items.len(), 2);
        // Highest importance retained
        assert!(items.iter().any(|i| i.id == "1"));
    }

    #[test]
    fn test_retention_score_favors_high_importance() {
        let high = MemoryItem::new("1", "content", 0.9);
        let low = MemoryItem::new("2", "content", 0.1);
        assert!(high.retention_score() > low.retention_score());
    }

    #[test]
    fn test_retention_score_favors_recent() {
        let mut recent = MemoryItem::new("1", "content", 0.5);
        recent.recency = 0;
        let mut old = MemoryItem::new("2", "content", 0.5);
        old.recency = 100;
        assert!(recent.retention_score() > old.retention_score());
    }

    #[test]
    fn test_insert_many_evicts() {
        let mem = SelectiveMemory::new(3, 0.0);
        let mut items = vec![
            MemoryItem::new("1", "a", 0.9),
            MemoryItem::new("2", "b", 0.8),
        ];
        let new = vec![
            MemoryItem::new("3", "c", 0.7),
            MemoryItem::new("4", "d", 0.6),
        ];
        mem.insert_many(&mut items, new);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_touch_updates_access() {
        let mut item = MemoryItem::new("1", "content", 0.5);
        assert_eq!(item.access_count, 0);
        SelectiveMemory::touch(&mut item);
        assert_eq!(item.access_count, 1);
        assert_eq!(item.recency, 0);
    }

    #[test]
    fn test_age_all_increments_recency() {
        let mut items = vec![
            MemoryItem::new("1", "a", 0.5),
            MemoryItem::new("2", "b", 0.5),
        ];
        items[0].recency = 5;
        SelectiveMemory::age_all(&mut items);
        assert_eq!(items[0].recency, 6);
        assert_eq!(items[1].recency, 1);
    }
}
