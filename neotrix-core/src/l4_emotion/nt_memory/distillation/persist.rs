#![forbid(unsafe_code)]

//! File-based JSON session persistence for memory entries.
//!
//! Stores each session as a separate JSON file under a configurable root directory.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use super::distiller::MemoryEntry;

/// Metadata about a persisted session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub memory_count: usize,
    pub saved_at: i64,
}

/// File-based JSON session persistence.
#[derive(Debug)]
pub struct SessionPersistence {
    root_dir: PathBuf,
}

impl SessionPersistence {
    /// Create a new persistence store rooted at `dir`.
    /// Creates the directory if it does not exist.
    pub fn new(dir: impl Into<PathBuf>) -> Result<Self, String> {
        let root = dir.into();
        fs::create_dir_all(&root).map_err(|e| format!("create dir: {e}"))?;
        Ok(Self { root_dir: root })
    }

    /// Default persistence directory: `~/.neotrix/sessions/`
    pub fn default_dir() -> Result<Self, String> {
        let home = dirs::home_dir().ok_or("cannot determine home dir")?;
        Self::new(home.join(".neotrix").join("sessions"))
    }

    /// Save a session's memories to `{root_dir}/{session_id}.json`.
    pub fn save_session(&self, session_id: &str, memories: &[MemoryEntry]) -> Result<(), String> {
        let path = self.session_path(session_id);
        let data = serde_json::to_string_pretty(memories).map_err(|e| format!("serialize: {e}"))?;
        fs::write(&path, data).map_err(|e| format!("write {}: {e}", path.display()))?;
        Ok(())
    }

    /// Load a session's memories from disk. Returns `None` if not found.
    pub fn load_session(&self, session_id: &str) -> Option<Vec<MemoryEntry>> {
        let path = self.session_path(session_id);
        let data = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// List all persisted sessions with metadata.
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        let mut infos = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.root_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let session_id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    if let Ok(data) = fs::read_to_string(&path) {
                        if let Ok(memories) = serde_json::from_str::<Vec<MemoryEntry>>(&data) {
                            let saved_at = fs::metadata(&path)
                                .and_then(|m| m.modified())
                                .ok()
                                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                                .map(|d| d.as_secs() as i64)
                                .unwrap_or(0);

                            infos.push(SessionInfo {
                                session_id,
                                memory_count: memories.len(),
                                saved_at,
                            });
                        }
                    }
                }
            }
        }
        infos.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
        infos
    }

    /// Delete a session from disk.
    pub fn delete_session(&self, session_id: &str) -> Result<(), String> {
        let path = self.session_path(session_id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("delete {}: {e}", path.display()))?;
        }
        Ok(())
    }

    fn session_path(&self, session_id: &str) -> PathBuf {
        self.root_dir.join(format!("{session_id}.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: &str) -> MemoryEntry {
        MemoryEntry {
            id: id.to_string(),
            content: format!("content-{id}"),
            embedding: vec![1.0, 0.0],
            access_count: 1,
            created_at: 1000,
            tags: Vec::new(),
        }
    }

    #[test]
    fn test_save_and_load() {
        let tmp = tempfile::tempdir().unwrap();
        let store = SessionPersistence::new(tmp.path()).unwrap();
        let memories = vec![make_entry("e1"), make_entry("e2")];
        store.save_session("sess_1", &memories).unwrap();

        let loaded = store.load_session("sess_1").unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "e1");
    }

    #[test]
    fn test_load_nonexistent_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let store = SessionPersistence::new(tmp.path()).unwrap();
        assert!(store.load_session("nope").is_none());
    }

    #[test]
    fn test_list_sessions() {
        let tmp = tempfile::tempdir().unwrap();
        let store = SessionPersistence::new(tmp.path()).unwrap();
        store.save_session("s1", &[make_entry("a")]).unwrap();
        store
            .save_session("s2", &[make_entry("b"), make_entry("c")])
            .unwrap();

        let mut infos = store.list_sessions();
        infos.sort_by(|a, b| a.session_id.cmp(&b.session_id));
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].memory_count, 1);
        assert_eq!(infos[1].memory_count, 2);
    }

    #[test]
    fn test_delete_session() {
        let tmp = tempfile::tempdir().unwrap();
        let store = SessionPersistence::new(tmp.path()).unwrap();
        store.save_session("del", &[make_entry("x")]).unwrap();
        assert!(store.load_session("del").is_some());
        store.delete_session("del").unwrap();
        assert!(store.load_session("del").is_none());
    }

    #[test]
    fn test_delete_nonexistent_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let store = SessionPersistence::new(tmp.path()).unwrap();
        store.delete_session("ghost").unwrap();
    }

    #[test]
    fn test_overwrite_session() {
        let tmp = tempfile::tempdir().unwrap();
        let store = SessionPersistence::new(tmp.path()).unwrap();
        store.save_session("s", &[make_entry("v1")]).unwrap();
        store
            .save_session("s", &[make_entry("v2"), make_entry("v3")])
            .unwrap();
        let loaded = store.load_session("s").unwrap();
        assert_eq!(loaded.len(), 2);
    }
}
