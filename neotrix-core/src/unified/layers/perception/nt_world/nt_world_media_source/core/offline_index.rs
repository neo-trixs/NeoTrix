use super::types::*;
use std::collections::HashMap;

pub struct OfflineEntry {
    pub item: MediaItem,
    pub local_path: String,
    pub cached_at: i64,
}

pub struct OfflineIndex {
    index: HashMap<String, OfflineEntry>,
}

impl OfflineIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, item: MediaItem, path: String) {
        let id = item.id.clone();
        let cached_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.index.insert(
            id,
            OfflineEntry {
                item,
                local_path: path,
                cached_at,
            },
        );
    }

    pub fn get(&self, id: &str) -> Option<&OfflineEntry> {
        self.index.get(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<OfflineEntry> {
        self.index.remove(id)
    }

    pub fn list(&self) -> Vec<&OfflineEntry> {
        self.index.values().collect()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.index.contains_key(id)
    }

    pub fn update_path(&mut self, id: &str, new_path: String) -> bool {
        if let Some(entry) = self.index.get_mut(id) {
            entry.local_path = new_path;
            true
        } else {
            false
        }
    }
}

impl Default for OfflineIndex {
    fn default() -> Self {
        Self::new()
    }
}
