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
    #[serde(skip)]
    use_kb: bool,
}

impl CrossSessionMemory {
    pub fn new(storage_path: PathBuf) -> Self {
        CrossSessionMemory {
            store: HashMap::new(),
            storage_path,
            max_entries: 1000,
            use_kb: false,
        }
    }

    /// 生产接线: 记忆主存 KB kv_store `state.cross_session_memory`, 文件仅作 legacy dual-write。
    pub fn with_kb(mut self) -> Self {
        self.use_kb = true;
        self
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
        if self.use_kb {
            crate::core::nt_core_state::save("cross_session_memory", &json)
                .map_err(|e| format!("kb write error: {}", e))?;
        }
        if !self.use_kb {
            std::fs::write(&self.storage_path, json).map_err(|e| format!("write error: {}", e))?;
        }
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), String> {
        let data = if self.use_kb {
            match crate::core::nt_core_state::load("cross_session_memory") {
                Some(d) => Some(d),
                None => self.storage_path
                    .exists()
                    .then(|| std::fs::read_to_string(&self.storage_path).ok())
                    .flatten(),
            }
        } else {
            if !self.storage_path.exists() {
                return Ok(());
            }
            std::fs::read_to_string(&self.storage_path).ok()
        };
        let Some(data) = data else {
            return Ok(());
        };
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
        let entries: Vec<String> = self.store.keys().cloned().collect();
        let mut sorted: Vec<(String, u64, u64)> = entries.iter()
            .filter_map(|k| self.store.get(k).map(|e| (k.clone(), e.last_accessed, e.created_at)))
            .collect();
        sorted.sort_by(|(_, la, ca), (_, lb, cb)| {
            match la.cmp(lb) {
                std::cmp::Ordering::Equal => ca.cmp(cb),
                other => other,
            }
        });
        let to_remove = self.store.len().saturating_sub(self.max_entries);
        for (key, _, _) in sorted.iter().take(to_remove) {
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

/// NT-NEXUS 域轻量 SelfTest (T1) — 跨 session 记忆连接数检测。
/// 真实逻辑: 写入多条跨 session 记忆并校验连接数 (len) / 检索 / 遗忘语义。
/// 注册后结果以 `nt_nexus_` 前缀流入 Repair/Meta/Governance/Nexus 四分支迷雾治理。
#[derive(Debug, Clone, Copy, Default)]
pub struct CrossSessionMemorySelfTest;

impl crate::core::nt_core_self_test::SelfTest for CrossSessionMemorySelfTest {
    fn name(&self) -> &str {
        "nt_nexus_cross_session_memory"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // 纯内存实例, 不落盘: 空 storage_path 使 Drop 的 auto_save (写盘失败被
        // 忽略) 不产生任何磁盘副作用
        let mut memory = CrossSessionMemory::new(PathBuf::new());
        memory.remember("session_a", "pattern", MemoryCategory::Pattern);
        memory.remember("session_b", "principle", MemoryCategory::Principle);
        memory.remember("session_c", "task", MemoryCategory::TaskOutcome);
        if memory.len() != 3 {
            failures.push(format!("expected 3 cross-session connections, got {}", memory.len()));
        }
        let recall = memory.recall("session_a");
        if recall.map(|e| e.value.as_str()) != Some("pattern") {
            failures.push("cross-session recall returned wrong value".into());
        }
        let recalled = memory.recall_by_category(MemoryCategory::Principle);
        if recalled.len() != 1 {
            failures.push(format!(
                "expected 1 principle-category connection, got {}",
                recalled.len()
            ));
        }
        if !memory.forget("session_b") {
            failures.push("forget should remove existing entry".into());
        }
        if memory.len() != 2 {
            failures.push(format!("expected 2 after forget, got {}", memory.len()));
        }
        // 跨 session 计数语义 (generate_key 含时间戳, 必以前缀开头)
        if !CrossSessionMemory::generate_key("nt_nexus").starts_with("nt_nexus_") {
            failures.push("generate_key should preserve prefix".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
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
