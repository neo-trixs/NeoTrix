#![forbid(unsafe_code)]

//! Working Memory — GWT-gated, max 7 items, evict lowest attention_weight.
//!
//! Tier 2 of the five-tier cascade. Only observations that pass the
//! Global Workspace Theory attention gate are admitted here.

use std::collections::BTreeMap;

/// A working memory item with an attention weight for priority eviction.
#[derive(Debug, Clone)]
pub struct WorkingItem {
    pub id: u64,
    pub content: String,
    pub attention_weight: f64,
    pub source_tier: String,
    pub turn_count: u32,
}

/// WorkingMemory: GWT-gated, max 7 items, evict lowest attention_weight.
#[derive(Debug)]
pub struct WorkingMemory {
    items: BTreeMap<u64, WorkingItem>,
    max_capacity: usize,
    next_id: u64,
    attention_threshold: f64,
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self {
            items: BTreeMap::new(),
            max_capacity: 7,
            next_id: 1,
            attention_threshold: 0.3,
        }
    }
}

impl WorkingMemory {
    pub fn new(max_capacity: usize, attention_threshold: f64) -> Self {
        Self {
            items: BTreeMap::new(),
            max_capacity,
            next_id: 1,
            attention_threshold,
        }
    }

    /// Attempt to admit an item. Returns false if attention weight is below threshold.
    pub fn attend(&mut self, content: String, attention_weight: f64, source_tier: String) -> bool {
        if attention_weight < self.attention_threshold {
            return false;
        }

        if self.items.len() >= self.max_capacity {
            self.evict_lowest();
        }

        let id = self.next_id;
        self.next_id += 1;

        self.items.insert(
            id,
            WorkingItem {
                id,
                content,
                attention_weight,
                source_tier,
                turn_count: 0,
            },
        );
        true
    }

    /// Recall all items, sorted by attention weight descending.
    pub fn recall_all(&self) -> Vec<&WorkingItem> {
        let mut items: Vec<&WorkingItem> = self.items.values().collect();
        items.sort_by(|a, b| b.attention_weight.partial_cmp(&a.attention_weight).unwrap());
        items
    }

    /// Retrieve a single item by id.
    pub fn get(&self, id: u64) -> Option<&WorkingItem> {
        self.items.get(&id)
    }

    /// Boost attention weight for a specific item.
    pub fn reinforce(&mut self, id: u64, delta: f64) -> bool {
        if let Some(item) = self.items.get_mut(&id) {
            item.attention_weight = (item.attention_weight + delta).min(1.0);
            true
        } else {
            false
        }
    }

    /// Remove an item by id, returning it if present.
    pub fn pop(&mut self, id: u64) -> Option<WorkingItem> {
        self.items.remove(&id)
    }

    /// Drain all items, returning them sorted by attention weight.
    pub fn drain_all(&mut self) -> Vec<WorkingItem> {
        let mut items: Vec<WorkingItem> = self.items.values().cloned().collect();
        self.items.clear();
        items.sort_by(|a, b| b.attention_weight.partial_cmp(&a.attention_weight).unwrap());
        items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn evict_lowest(&mut self) {
        if let Some((min_id, _)) = self
            .items
            .iter()
            .min_by(|a, b| a.1.attention_weight.partial_cmp(&b.1.attention_weight).unwrap())
        {
            let min_id = *min_id;
            self.items.remove(&min_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attend_below_threshold_rejected() {
        let mut wm = WorkingMemory::default();
        assert!(!wm.attend("low".into(), 0.1, "sensory".into()));
        assert!(wm.is_empty());
    }

    #[test]
    fn attend_above_threshold_admitted() {
        let mut wm = WorkingMemory::default();
        assert!(wm.attend("hi".into(), 0.8, "sensory".into()));
        assert_eq!(wm.len(), 1);
    }

    #[test]
    fn eviction_of_lowest_weight() {
        let mut wm = WorkingMemory::new(2, 0.0);
        wm.attend("a".into(), 0.5, "s".into());
        wm.attend("b".into(), 0.9, "s".into());
        wm.attend("c".into(), 0.3, "s".into());
        assert_eq!(wm.len(), 2);
        let ids: Vec<u64> = wm.recall_all().iter().map(|i| i.id).collect();
        assert_eq!(ids.len(), 2);
    }
}
