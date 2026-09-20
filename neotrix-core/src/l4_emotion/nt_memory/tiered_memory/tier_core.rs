#![forbid(unsafe_code)]

use std::collections::HashMap;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

use super::traits::{MemoryItem, MemoryTier};

/// Tier 1 Core — always-in-context memory.
///
/// Design constraints (mem0-inspired):
/// - Capacity budget: 2-5K characters total (configurable).
/// - Items are ranked by importance; lowest-importance evicted when budget exceeded.
/// - Every write triggers budget enforcement — no deferred eviction.
/// - `snapshot_for_context()` returns a formatted string ready for LLM injection.
#[derive(Debug)]
pub struct CoreStore {
    /// Ordered by insertion (Vec maintains order for deterministic snapshots).
    items: Vec<MemoryItem>,
    /// Index: id → position in `items`.
    index: HashMap<String, usize>,
    /// Maximum character budget for all items combined.
    char_budget: usize,
    /// Maximum number of items (hard cap to prevent O(n) scans).
    max_items: usize,
    /// LRU cache for recently accessed items (hot path optimization).
    access_cache: LruCache<String, ()>,
    /// Total writes for stats.
    writes: u64,
}

/// Configuration for CoreStore (R-P11 config struct).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreStoreConfig {
    /// Maximum character budget (default: 4096 = ~4K chars).
    pub char_budget: usize,
    /// Maximum number of items (default: 50).
    pub max_items: usize,
    /// LRU cache capacity (default: 32).
    pub cache_capacity: usize,
}

impl Default for CoreStoreConfig {
    fn default() -> Self {
        Self {
            char_budget: 4096,
            max_items: 50,
            cache_capacity: 32,
        }
    }
}

impl CoreStore {
    pub fn new(config: CoreStoreConfig) -> Self {
        Self {
            items: Vec::with_capacity(config.max_items),
            index: HashMap::with_capacity(config.max_items),
            char_budget: config.char_budget,
            max_items: config.max_items,
            access_cache: LruCache::new(
                NonZeroUsize::new(config.cache_capacity).unwrap_or(NonZeroUsize::new(1).unwrap()),
            ),
            writes: 0,
        }
    }

    /// Insert or update a core memory item.
    ///
    /// If the item already exists, it's updated in-place. After insertion,
    /// the budget is enforced: lowest-importance items are evicted until
    /// total chars ≤ char_budget.
    pub fn insert(&mut self, mut item: MemoryItem) -> Result<(), String> {
        item.tier = MemoryTier::Tier1Core;

        // Update existing item
        if let Some(&pos) = self.index.get(&item.id) {
            self.items[pos] = item;
            self.writes += 1;
            self.enforce_budget();
            return Ok(());
        }

        // Check hard cap
        if self.items.len() >= self.max_items {
            self.evict_lowest_importance(1);
        }

        // Insert new
        let pos = self.items.len();
        self.index.insert(item.id.clone(), pos);
        self.items.push(item);
        self.writes += 1;

        self.enforce_budget();
        Ok(())
    }

    /// Get a core memory item by id. Updates access count and LRU position.
    pub fn get(&mut self, id: &str) -> Option<&MemoryItem> {
        self.access_cache.put(id.to_string(), ());
        let &pos = self.index.get(id)?;
        let item = &self.items[pos];
        Some(item)
    }

    /// Get a mutable reference to an item.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut MemoryItem> {
        self.access_cache.put(id.to_string(), ());
        let &pos = self.index.get(id)?;
        Some(&mut self.items[pos])
    }

    /// Remove a specific item by id.
    pub fn remove(&mut self, id: &str) -> Option<MemoryItem> {
        let pos = self.index.remove(id)?;
        self.access_cache.pop(id);

        // Swap-remove for O(1) removal
        let item = self.items.swap_remove(pos);

        // Update index for the item that was swapped into `pos`
        if pos < self.items.len() {
            let moved_id = self.items[pos].id.clone();
            self.index.insert(moved_id, pos);
        }

        Some(item)
    }

    /// Return all items sorted by importance (descending) for snapshot.
    pub fn items_by_importance(&self) -> Vec<&MemoryItem> {
        let mut sorted: Vec<&MemoryItem> = self.items.iter().collect();
        sorted.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// Format all core memories as a context string for LLM injection.
    ///
    /// Format:
    /// ```text
    /// [Core Memory]
    /// - User identity: Alice, ML engineer...
    /// - Active goals: Implement 3-tier memory...
    /// - Project state: NeoTrix v0.8...
    /// ```
    pub fn snapshot(&self) -> String {
        let mut lines = vec!["[Core Memory]".to_string()];
        let sorted = self.items_by_importance();
        for item in sorted {
            // Truncate individual items to prevent single-item bloat
            let truncated = if item.content.len() > 500 {
                format!("{}...", &item.content[..497])
            } else {
                item.content.clone()
            };
            lines.push(format!("- {}: {}", item.id, truncated));
        }
        lines.join("\n")
    }

    /// Total character count of all items.
    pub fn total_chars(&self) -> usize {
        self.items.iter().map(|i| i.content.len()).sum()
    }

    /// Number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Enforce the character budget by evicting lowest-importance items.
    fn enforce_budget(&mut self) {
        while self.total_chars() > self.char_budget && !self.items.is_empty() {
            self.evict_lowest_importance(1);
        }
    }

    /// Remove the N lowest-importance items.
    fn evict_lowest_importance(&mut self, n: usize) {
        // Find indices of lowest-importance items
        let mut indices_with_importance: Vec<(usize, f64)> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| (i, item.importance))
            .collect();
        indices_with_importance
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let to_remove: Vec<String> = indices_with_importance
            .iter()
            .take(n)
            .map(|(i, _)| self.items[*i].id.clone())
            .collect();

        for id in to_remove {
            self.remove(&id);
        }
    }

    pub fn stats(&self) -> CoreStats {
        CoreStats {
            item_count: self.items.len(),
            total_chars: self.total_chars(),
            char_budget: self.char_budget,
            writes: self.writes,
            cache_hit_rate: if self.access_cache.len() > 0 {
                self.access_cache.len() as f64 / (self.access_cache.len() + 1) as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoreStats {
    pub item_count: usize,
    pub total_chars: usize,
    pub char_budget: usize,
    pub writes: u64,
    pub cache_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, content: &str, importance: f64) -> MemoryItem {
        MemoryItem::new(
            id.to_string(),
            content.to_string(),
            MemoryTier::Tier1Core,
            importance,
            "test".to_string(),
        )
    }

    #[test]
    fn test_insert_and_get() {
        let mut store = CoreStore::new(CoreStoreConfig::default());
        let item = make_item("user-1", "Alice is an ML engineer", 0.9);
        store.insert(item).unwrap();
        assert_eq!(store.len(), 1);
        assert!(store.get("user-1").is_some());
    }

    #[test]
    fn test_budget_enforcement() {
        let config = CoreStoreConfig {
            char_budget: 100,
            max_items: 50,
            cache_capacity: 32,
        };
        let mut store = CoreStore::new(config);

        store.insert(make_item("a", &"x".repeat(60), 0.3)).unwrap();
        store.insert(make_item("b", &"y".repeat(60), 0.7)).unwrap();
        // Total = 120 > budget of 100, so lowest importance ("a") should be evicted
        assert_eq!(store.len(), 1);
        assert!(store.get("a").is_none());
        assert!(store.get("b").is_some());
    }

    #[test]
    fn test_remove() {
        let mut store = CoreStore::new(CoreStoreConfig::default());
        store.insert(make_item("a", "hello", 0.5)).unwrap();
        let removed = store.remove("a");
        assert!(removed.is_some());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_update_in_place() {
        let mut store = CoreStore::new(CoreStoreConfig::default());
        store.insert(make_item("a", "old", 0.5)).unwrap();
        store.insert(make_item("a", "new", 0.9)).unwrap();
        assert_eq!(store.len(), 1);
        assert_eq!(store.get("a").unwrap().content, "new");
    }

    #[test]
    fn test_snapshot_format() {
        let mut store = CoreStore::new(CoreStoreConfig::default());
        store.insert(make_item("user", "Alice", 0.9)).unwrap();
        store.insert(make_item("goal", "Build memory system", 0.8)).unwrap();
        let snap = store.snapshot();
        assert!(snap.starts_with("[Core Memory]"));
        assert!(snap.contains("Alice"));
    }

    #[test]
    fn test_items_by_importance_order() {
        let mut store = CoreStore::new(CoreStoreConfig::default());
        store.insert(make_item("low", "low", 0.2)).unwrap();
        store.insert(make_item("high", "high", 0.9)).unwrap();
        store.insert(make_item("mid", "mid", 0.5)).unwrap();
        let sorted = store.items_by_importance();
        assert_eq!(sorted[0].id, "high");
        assert_eq!(sorted[1].id, "mid");
        assert_eq!(sorted[2].id, "low");
    }

    #[test]
    fn test_max_items_cap() {
        let config = CoreStoreConfig {
            char_budget: 100_000,
            max_items: 3,
            cache_capacity: 32,
        };
        let mut store = CoreStore::new(config);
        store.insert(make_item("a", "a", 0.1)).unwrap();
        store.insert(make_item("b", "b", 0.2)).unwrap();
        store.insert(make_item("c", "c", 0.3)).unwrap();
        store.insert(make_item("d", "d", 0.4)).unwrap();
        assert!(store.len() <= 3);
    }

    #[test]
    fn test_total_chars() {
        let mut store = CoreStore::new(CoreStoreConfig::default());
        store.insert(make_item("a", "hello", 0.5)).unwrap();
        store.insert(make_item("b", "world", 0.5)).unwrap();
        assert_eq!(store.total_chars(), 10);
    }
}
