use super::app::types::Session;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionExport {
    format_version: u32,
    sessions: Vec<SessionData>,
}

pub struct SessionStore {
    sessions_dir: PathBuf,
}

impl SessionStore {
    pub fn new() -> Self {
        let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let sessions_dir = base.join(".neotrix").join("sessions");
        let _ = fs::create_dir_all(&sessions_dir);
        Self { sessions_dir }
    }

    pub fn save_session(&self, name: &str, data: &SessionData) -> Result<(), String> {
        let path = self.sessions_dir.join(format!("{}.json", name));
        let json = serde_json::to_string_pretty(data).map_err(|e| format!("序列化失败: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("写入失败: {}", e))?;
        let last_path = self.sessions_dir.join(".last");
        let _ = fs::write(&last_path, name);
        Ok(())
    }

    pub fn load_session(&self, name: &str) -> Result<SessionData, String> {
        let path = self.sessions_dir.join(format!("{}.json", name));
        let json = fs::read_to_string(&path).map_err(|e| format!("读取失败: {}", e))?;
        serde_json::from_str(&json).map_err(|e| format!("反序列化失败: {}", e))
    }

    pub fn list_sessions(&self) -> Vec<String> {
        let mut sessions = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.sessions_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") && !name.starts_with('.') {
                        sessions.push(name.trim_end_matches(".json").to_string());
                    }
                }
            }
        }
        sessions.sort();
        sessions
    }

    pub fn delete_session(&self, name: &str) -> Result<(), String> {
        let path = self.sessions_dir.join(format!("{}.json", name));
        fs::remove_file(&path).map_err(|e| format!("删除失败: {}", e))
    }

    pub fn get_last_session(&self) -> Option<String> {
        let last_path = self.sessions_dir.join(".last");
        fs::read_to_string(&last_path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Fork a session: clone an existing session with a new name
    pub fn fork(&self, name: &str) -> Result<String, String> {
        let data = self.load_session(name)?;
        let new_name = format!("{} (副本)", data.name);
        let forked = SessionData {
            name: new_name.clone(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        self.save_session(&new_name, &forked)?;
        Ok(new_name)
    }

    /// Export a session to a pretty-printed JSON string
    pub fn export_to_json(&self, name: &str) -> Result<String, String> {
        let data = self.load_session(name)?;
        let export = SessionExport {
            format_version: 1,
            sessions: vec![data],
        };
        serde_json::to_string_pretty(&export).map_err(|e| format!("序列化失败: {}", e))
    }

    /// Import a session from a JSON string (SessionExport format)
    pub fn import_from_json(&self, json: &str) -> Result<String, String> {
        let export: SessionExport =
            serde_json::from_str(json).map_err(|e| format!("反序列化失败: {}", e))?;
        if export.format_version != 1 {
            return Err(format!("不支持的格式版本: {}", export.format_version));
        }
        let mut imported_names = Vec::new();
        for data in &export.sessions {
            let name = if self.list_sessions().contains(&data.name) {
                format!("{} (导入)", data.name)
            } else {
                data.name.clone()
            };
            let imported = SessionData {
                name: name.clone(),
                created_at: data.created_at.clone(),
                updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            self.save_session(&name, &imported)?;
            imported_names.push(name);
        }
        Ok(imported_names.join(", "))
    }

    /// Export a session to a file
    pub fn export_to_file(&self, name: &str, path: &str) -> Result<(), String> {
        let json = self.export_to_json(name)?;
        fs::write(path, &json).map_err(|e| format!("写入文件失败: {}", e))
    }

    /// Import a session from a file
    pub fn import_from_file(&self, path: &str) -> Result<String, String> {
        let json = fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;
        self.import_from_json(&json)
    }

    // ── Full Session Persistence (with messages) ──

    /// Save a full Session (with messages) to disk as JSON, keyed by session ID
    pub fn save_full_session(&self, session: &Session) -> Result<(), String> {
        let path = self.sessions_dir.join(format!("{}.json", session.id));
        let json =
            serde_json::to_string_pretty(session).map_err(|e| format!("序列化失败: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("写入失败: {}", e))?;
        let last_path = self.sessions_dir.join(".last");
        let _ = fs::write(&last_path, &session.id);
        Ok(())
    }

    /// Load a full Session by its ID
    pub fn load_full_session(&self, id: &str) -> Result<Session, String> {
        let path = self.sessions_dir.join(format!("{}.json", id));
        let json = fs::read_to_string(&path).map_err(|e| format!("读取失败: {}", e))?;
        serde_json::from_str(&json).map_err(|e| format!("反序列化失败: {}", e))
    }

    /// Load all full Sessions from disk, skipping files that don't deserialize as Session
    pub fn list_full_sessions(&self) -> Vec<Session> {
        let mut sessions = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                        if name.starts_with('.') {
                            continue;
                        }
                        if let Ok(json) = fs::read_to_string(&path) {
                            if let Ok(session) = serde_json::from_str::<Session>(&json) {
                                sessions.push(session);
                            }
                        }
                    }
                }
            }
        }
        sessions
    }

    /// Delete a full session file by ID
    pub fn delete_full_session(&self, id: &str) -> Result<(), String> {
        let path = self.sessions_dir.join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("删除失败: {}", e))
        } else {
            Ok(())
        }
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn temp_store() -> (SessionStore, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let store = SessionStore {
            sessions_dir: dir.path().to_path_buf(),
        };
        (store, dir)
    }

    fn seed_session(store: &SessionStore, name: &str) {
        let data = SessionData {
            name: name.to_string(),
            created_at: "2025-01-01 00:00:00".into(),
            updated_at: "2025-01-01 00:00:00".into(),
        };
        store.save_session(name, &data).unwrap();
    }

    #[test]
    fn test_fork_creates_copy() {
        let (store, _dir) = temp_store();
        seed_session(&store, "original");
        let new_name = store.fork("original").unwrap();
        assert!(new_name.contains("original"));
        let sessions = store.list_sessions();
        assert!(sessions.contains(&"original".to_string()));
        assert!(sessions.iter().any(|s| s.contains("original")));
    }

    #[test]
    fn test_fork_not_found() {
        let (store, _dir) = temp_store();
        let result = store.fork("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_to_json_roundtrip() {
        let (store, _dir) = temp_store();
        seed_session(&store, "test-session");
        let json = store.export_to_json("test-session").unwrap();
        assert!(json.contains("\"format_version\": 1"));
        assert!(json.contains("test-session"));

        let (store2, _dir2) = temp_store();
        let imported = store2.import_from_json(&json).unwrap();
        assert!(!imported.is_empty());
        let sessions = store2.list_sessions();
        assert!(sessions.contains(&"test-session".to_string()));
    }

    #[test]
    fn test_import_duplicate_renames() {
        let (store, _dir) = temp_store();
        seed_session(&store, "dup");
        let json = store.export_to_json("dup").unwrap();
        let imported = store.import_from_json(&json).unwrap();
        assert!(imported.contains("dup (导入)"));
    }

    #[test]
    fn test_export_to_file_and_import_from_file() {
        let (store, _dir) = temp_store();
        seed_session(&store, "file-test");
        let tmp = Path::new("/tmp/test_session_export.json");
        store
            .export_to_file("file-test", tmp.to_str().unwrap())
            .unwrap();
        assert!(tmp.exists());

        let (store2, _dir2) = temp_store();
        let imported = store2.import_from_file(tmp.to_str().unwrap()).unwrap();
        assert!(!imported.is_empty());
        let _ = fs::remove_file(tmp);
    }

    #[test]
    fn test_import_invalid_version() {
        let (store, _dir) = temp_store();
        let bad_json = r#"{"format_version": 99, "sessions": []}"#;
        let result = store.import_from_json(bad_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_malformed_json() {
        let (store, _dir) = temp_store();
        let result = store.import_from_json("not json");
        assert!(result.is_err());
    }
}
