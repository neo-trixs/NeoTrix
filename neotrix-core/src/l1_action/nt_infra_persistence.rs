//! L1 基础设施 — Registry 持久化
//!
//! Registry 状态存入 KB，重启后自动恢复
//! 支持: JSON 序列化 + 增量同步

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// 持久化条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedEntry {
    pub id: String,
    pub category: String,
    pub constellation: String,
    pub description: String,
    pub health_healthy: bool,
    pub health_error_rate: f64,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Registry 持久化存储
pub struct RegistryPersistence {
    path: PathBuf,
    entries: Vec<PersistedEntry>,
}

impl RegistryPersistence {
    pub fn new(path: &str) -> Self {
        let path = PathBuf::from(path);
        let entries = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        Self { path, entries }
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| e.to_string())?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&self.path, json).map_err(|e| e.to_string())
    }

    pub fn load(&mut self) -> Result<(), String> {
        if self.path.exists() {
            let content = std::fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
            self.entries = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn upsert(&mut self, entry: PersistedEntry) {
        self.entries.retain(|e| e.id != entry.id);
        self.entries.push(entry);
    }

    pub fn remove(&mut self, id: &str) {
        self.entries.retain(|e| e.id != id);
    }

    pub fn entries(&self) -> &[PersistedEntry] { &self.entries }
    pub fn count(&self) -> usize { self.entries.len() }
}

/// 全局持久化存储
lazy_static::lazy_static! {
    static ref GLOBAL_PERSISTENCE: std::sync::Mutex<RegistryPersistence> =
        std::sync::Mutex::new(RegistryPersistence::new(
            &format!("{}/.neotrix/registry.json",
                std::env::var("HOME").unwrap_or_default())
        ));
}

pub fn persistence_save() -> Result<(), String> {
    GLOBAL_PERSISTENCE.lock().unwrap().save()
}

pub fn persistence_load() -> Result<(), String> {
    GLOBAL_PERSISTENCE.lock().unwrap().load()
}

pub fn persistence_upsert(entry: PersistedEntry) {
    GLOBAL_PERSISTENCE.lock().unwrap().upsert(entry);
}

pub fn persistence_count() -> usize {
    GLOBAL_PERSISTENCE.lock().unwrap().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistence_crud() {
        let mut p = RegistryPersistence::new("/tmp/test_registry.json");
        p.upsert(PersistedEntry {
            id: "test1".into(),
            category: "search".into(),
            constellation: "C2".into(),
            description: "Test".into(),
            health_healthy: true,
            health_error_rate: 0.0,
            tags: vec![],
            metadata: HashMap::new(),
        });
        assert_eq!(p.count(), 1);
        p.remove("test1");
        assert_eq!(p.count(), 0);
    }
}
