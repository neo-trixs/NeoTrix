use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub category: MemoryCategory,
    pub confidence: f64,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryCategory {
    Principle,
    Pattern,
    CapabilityState,
    TaskOutcome,
    UserPreference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSessionMemory {
    store: HashMap<String, MemoryEntry>,
    storage_path: PathBuf,
    max_entries: usize,
}

impl CrossSessionMemory {
    pub fn new(storage_path: PathBuf) -> Self {
        CrossSessionMemory {
            store: HashMap::new(),
            storage_path,
            max_entries: 1000,
        }
    }

    pub fn remember(&mut self, key: &str, value: &str, category: MemoryCategory) {
        let now = current_timestamp();
        if let Some(entry) = self.store.get_mut(key) {
            entry.value = value.to_string();
            entry.category = category;
            entry.access_count += 1;
            entry.last_accessed = now;
        } else {
            let entry = MemoryEntry {
                key: key.to_string(),
                value: value.to_string(),
                category,
                confidence: 1.0,
                created_at: now,
                last_accessed: now,
                access_count: 1,
            };
            self.store.insert(key.to_string(), entry);
        }
        self.evict_if_needed();
    }

    pub fn recall(&mut self, key: &str) -> Option<&MemoryEntry> {
        let now = current_timestamp();
        let entry = self.store.get_mut(key)?;
        entry.access_count += 1;
        entry.last_accessed = now;
        Some(&*entry)
    }

    pub fn recall_by_category(&mut self, category: MemoryCategory) -> Vec<&MemoryEntry> {
        let now = current_timestamp();
        let keys: Vec<String> = self
            .store
            .iter()
            .filter(|(_, e)| e.category == category)
            .map(|(k, _)| k.clone())
            .collect();
        for key in &keys {
            if let Some(entry) = self.store.get_mut(key) {
                entry.access_count += 1;
                entry.last_accessed = now;
            }
        }
        self.store.values().filter(|e| e.category == category).collect()
    }

    pub fn forget(&mut self, key: &str) -> bool {
        self.store.remove(key).is_some()
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("serialize error: {}", e))?;
        std::fs::write(&self.storage_path, json).map_err(|e| format!("write error: {}", e))
    }

    pub fn load(&mut self) -> Result<(), String> {
        if !self.storage_path.exists() {
            return Ok(());
        }
        let data = std::fs::read_to_string(&self.storage_path)
            .map_err(|e| format!("read error: {}", e))?;
        let loaded: Self =
            serde_json::from_str(&data).map_err(|e| format!("deserialize error: {}", e))?;
        self.store.clone_from(&loaded.store);
        self.max_entries = loaded.max_entries;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn summary(&self) -> String {
        let categories: std::collections::BTreeSet<&MemoryCategory> =
            self.store.values().map(|e| &e.category).collect();
        let cat_count = categories.len();
        format!(
            "CrossSessionMemory: {} entries, {} categories",
            self.store.len(),
            cat_count
        )
    }

    pub fn auto_save(&self) {
        let _ = self.save();
    }

    fn evict_if_needed(&mut self) {
        if self.store.len() <= self.max_entries {
            return;
        }
        let mut entries: Vec<String> = self.store.keys().cloned().collect();
        entries.sort_by(|a, b| {
            let ea = self.store.get(a).unwrap();
            let eb = self.store.get(b).unwrap();
            match ea.last_accessed.cmp(&eb.last_accessed) {
                std::cmp::Ordering::Equal => ea.created_at.cmp(&eb.created_at),
                other => other,
            }
        });
        let to_remove = self.store.len() - self.max_entries;
        for key in entries.iter().take(to_remove) {
            self.store.remove(key);
        }
    }

    pub fn generate_key(prefix: &str) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("{}_{}", prefix, ts)
    }
}

impl Drop for CrossSessionMemory {
    fn drop(&mut self) {
        self.auto_save();
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path() -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("cross_session_memory_test_{}", ts));
        p
    }

    #[test]
    fn test_remember_and_recall() {
        let path = temp_path();
        let mut mem = CrossSessionMemory::new(path.clone());
        mem.remember("k1", "v1", MemoryCategory::Principle);
        let entry = mem.recall("k1").expect("should find k1");
        assert_eq!(entry.value, "v1");
        assert_eq!(entry.category, MemoryCategory::Principle);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_recall_updates_access_count() {
        let path = temp_path();
        let mut mem = CrossSessionMemory::new(path.clone());
        mem.remember("k1", "v1", MemoryCategory::Pattern);
        let count_before = mem.recall("k1").unwrap().access_count;
        assert_eq!(count_before, 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_recall_by_category() {
        let path = temp_path();
        let mut mem = CrossSessionMemory::new(path.clone());
        mem.remember("a", "x", MemoryCategory::Principle);
        mem.remember("b", "y", MemoryCategory::Pattern);
        mem.remember("c", "z", MemoryCategory::Principle);
        let principles = mem.recall_by_category(MemoryCategory::Principle);
        assert_eq!(principles.len(), 2);
        let patterns = mem.recall_by_category(MemoryCategory::Pattern);
        assert_eq!(patterns.len(), 1);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_forget_removes_entry() {
        let path = temp_path();
        let mut mem = CrossSessionMemory::new(path.clone());
        mem.remember("k1", "v1", MemoryCategory::UserPreference);
        assert!(mem.forget("k1"));
        assert!(!mem.forget("k1"));
        assert!(mem.recall("k1").is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_save_and_reload_roundtrip() {
        let path = temp_path();
        {
            let mut mem = CrossSessionMemory::new(path.clone());
            mem.remember("k1", "v1", MemoryCategory::Principle);
            mem.remember("k2", "v2", MemoryCategory::Pattern);
            mem.save().unwrap();
        }
        {
            let mut mem = CrossSessionMemory::new(path.clone());
            mem.load().unwrap();
            assert_eq!(mem.len(), 2);
            assert_eq!(mem.recall("k1").unwrap().value, "v1");
            assert_eq!(mem.recall("k2").unwrap().value, "v2");
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_generate_key_format() {
        let key = CrossSessionMemory::generate_key("test");
        assert!(key.starts_with("test_"));
        let parts: Vec<&str> = key.split('_').collect();
        assert_eq!(parts.len(), 2);
        let _: u64 = parts[1].parse().expect("timestamp should be numeric");
    }

    #[test]
    fn test_eviction() {
        let path = temp_path();
        let mut mem = CrossSessionMemory::new(path.clone());
        mem.max_entries = 3;
        mem.remember("a", "", MemoryCategory::Principle);
        std::thread::sleep(std::time::Duration::from_millis(2));
        mem.remember("b", "", MemoryCategory::Principle);
        std::thread::sleep(std::time::Duration::from_millis(2));
        mem.remember("c", "", MemoryCategory::Principle);
        std::thread::sleep(std::time::Duration::from_millis(2));
        mem.remember("d", "", MemoryCategory::Principle);
        assert_eq!(mem.len(), 3);
        assert!(mem.recall("a").is_none());
        assert!(mem.recall("b").is_some());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_summary_format() {
        let path = temp_path();
        let mut mem = CrossSessionMemory::new(path.clone());
        assert!(mem.summary().contains("0 entries"));
        mem.remember("a", "x", MemoryCategory::Principle);
        mem.remember("b", "y", MemoryCategory::Pattern);
        let s = mem.summary();
        assert!(s.contains("CrossSessionMemory:"));
        assert!(s.contains("2 entries"));
        assert!(s.contains("2 categories"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_len() {
        let path = temp_path();
        let mut mem = CrossSessionMemory::new(path.clone());
        assert_eq!(mem.len(), 0);
        mem.remember("a", "", MemoryCategory::TaskOutcome);
        assert_eq!(mem.len(), 1);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_nonexistent_graceful() {
        let path = temp_path();
        let mut mem = CrossSessionMemory::new(path.clone());
        mem.load().unwrap();
        assert_eq!(mem.len(), 0);
        // no file should have been created
        assert!(!path.exists());
    }
}
