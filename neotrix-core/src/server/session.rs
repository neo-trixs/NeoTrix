use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::nt_core_traits::SessionProvider;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: i64,
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    active: Option<String>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active: None,
        }
    }
    pub fn create(&mut self, id: &str, name: &str) -> Session {
        let s = Session {
            id: id.to_string(),
            name: name.to_string(),
            created_at: chrono::Utc::now().timestamp(),
        };
        self.sessions.insert(id.to_string(), s.clone());
        self.active = Some(id.to_string());
        s
    }
    pub fn list(&self) -> Vec<&Session> {
        self.sessions.values().collect()
    }
    pub fn active(&self) -> Option<&Session> {
        self.active.as_ref().and_then(|id| self.sessions.get(id))
    }
    pub fn switch(&mut self, id: &str) -> bool {
        if self.sessions.contains_key(id) {
            self.active = Some(id.to_string());
            true
        } else {
            false
        }
    }
    pub fn remove(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_traits::SessionProvider;

    #[test]
    fn test_session_provider_create_and_active() {
        let mut mgr = SessionManager::new();
        let session = SessionProvider::create_session(&mut mgr, "s1", "test-session");
        assert_eq!(session.id, "s1");
        assert_eq!(session.name, "test-session");
        let active = SessionProvider::active_session(&mgr);
        assert!(active.is_some());
        assert_eq!(active.expect("active should be ok in test").id, "s1");
    }

    #[test]
    fn test_session_provider_switch_and_list() {
        let mut mgr = SessionManager::new();
        SessionProvider::create_session(&mut mgr, "s1", "first");
        SessionProvider::create_session(&mut mgr, "s2", "second");
        // After create_session, active is the last created
        let active = SessionProvider::active_session(&mgr);
        assert_eq!(active.expect("active should be ok in test").id, "s2");
        // Switch back to s1
        assert!(SessionProvider::switch_session(&mut mgr, "s1"));
        let active = SessionProvider::active_session(&mgr);
        assert_eq!(active.expect("active should be ok in test").id, "s1");
        // List
        let sessions = SessionProvider::list_sessions(&mgr);
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_session_provider_switch_invalid() {
        let mut mgr = SessionManager::new();
        assert!(!SessionProvider::switch_session(&mut mgr, "nonexistent"));
        assert!(SessionProvider::active_session(&mgr).is_none());
    }
}

impl SessionProvider for SessionManager {
    type Session = Session;

    fn create_session(&mut self, id: &str, name: &str) -> Session {
        self.create(id, name)
    }

    fn switch_session(&mut self, id: &str) -> bool {
        self.switch(id)
    }

    fn active_session(&self) -> Option<&Session> {
        self.active()
    }

    fn list_sessions(&self) -> Vec<&Session> {
        self.list()
    }
}

// ====== Session Share ======

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionShare {
    pub token: String,
    pub session_name: String,
    pub session_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct SessionShareManager {
    shares_dir: PathBuf,
}

impl Default for SessionShareManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionShareManager {
    /// Creates directory ~/.neotrix/shares/ if not exists.
    pub fn new() -> Self {
        let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let shares_dir = base.join(".neotrix").join("shares");
        let _ = fs::create_dir_all(&shares_dir);
        Self { shares_dir }
    }

    /// Create a new share from a session JSON payload.
    pub fn create(
        &self,
        session_name: &str,
        session_json: serde_json::Value,
        ttl_hours: Option<u64>,
    ) -> Result<SessionShare, String> {
        let token = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let expires_at = ttl_hours.map(|h| created_at + chrono::Duration::hours(h as i64));
        let share = SessionShare {
            token: token.clone(),
            session_name: session_name.to_string(),
            session_json,
            created_at,
            expires_at,
        };
        let path = self.shares_dir.join(format!("{token}.json"));
        let json = serde_json::to_string_pretty(&share).map_err(|e| format!("序列化失败: {e}"))?;
        fs::write(&path, json).map_err(|e| format!("写入失败: {e}"))?;
        Ok(share)
    }

    /// Retrieve a share by token (checks expiration).
    pub fn get(&self, token: &str) -> Result<SessionShare, String> {
        let path = self.shares_dir.join(format!("{token}.json"));
        if !path.exists() {
            return Err("分享不存在或已过期".to_string());
        }
        let json = fs::read_to_string(&path).map_err(|e| format!("读取失败: {e}"))?;
        let share: SessionShare =
            serde_json::from_str(&json).map_err(|e| format!("反序列化失败: {e}"))?;
        if let Some(exp) = share.expires_at {
            if Utc::now() > exp {
                let _ = fs::remove_file(&path);
                return Err("分享已过期".to_string());
            }
        }
        Ok(share)
    }

    /// List all non-expired shares.
    pub fn list(&self) -> Result<Vec<SessionShare>, String> {
        let mut shares = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.shares_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let json = fs::read_to_string(&path).map_err(|e| format!("读取失败: {e}"))?;
                    if let Ok(share) = serde_json::from_str::<SessionShare>(&json) {
                        let expired = share.expires_at.map(|e| Utc::now() > e).unwrap_or(false);
                        if expired {
                            let _ = fs::remove_file(&path);
                            continue;
                        }
                        shares.push(share);
                    }
                }
            }
        }
        Ok(shares)
    }

    /// Delete a share by token.
    pub fn delete(&self, token: &str) -> Result<(), String> {
        let path = self.shares_dir.join(format!("{token}.json"));
        fs::remove_file(&path).map_err(|e| format!("删除失败: {e}"))
    }
}

#[cfg(test)]
mod share_tests {
    use super::*;

    fn temp_share_manager() -> (SessionShareManager, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let mgr = SessionShareManager {
            shares_dir: dir.path().to_path_buf(),
        };
        (mgr, dir)
    }

    #[test]
    fn test_create_and_get_share() {
        let (mgr, _dir) = temp_share_manager();
        let share = mgr
            .create("test-session", serde_json::json!({"messages": []}), None)
            .unwrap();
        assert_eq!(share.session_name, "test-session");
        assert!(share.expires_at.is_none());

        let retrieved = mgr.get(&share.token).unwrap();
        assert_eq!(retrieved.token, share.token);
    }

    #[test]
    fn test_share_ttl_expired() {
        let (mgr, _dir) = temp_share_manager();
        let share = mgr
            .create(
                "expired-session",
                serde_json::json!({"done": true}),
                Some(0), // 0 hours → effectively expired immediately
            )
            .unwrap();
        // The share was created with ttl=0, which means expires_at == created_at.
        // Since created_at is Utc::now() and we check Utc::now() > expires_at,
        // this might not always be expired if the calls are fast.
        // So we manually set expires_at to the past via the file.
        let past = Utc::now() - chrono::Duration::hours(1);
        let expired_share = SessionShare {
            expires_at: Some(past),
            ..share
        };
        let path = mgr
            .shares_dir
            .join(format!("{}.json", &expired_share.token));
        let json = serde_json::to_string(&expired_share).unwrap();
        fs::write(&path, json).unwrap();

        let result = mgr.get(&expired_share.token);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("过期"));
    }

    #[test]
    fn test_list_shares_skips_expired() {
        let (mgr, _dir) = temp_share_manager();
        mgr.create("alive", serde_json::json!({"a": 1}), Some(24))
            .unwrap();
        let dead_token = {
            let s = mgr
                .create("dead", serde_json::json!({"b": 2}), Some(0))
                .unwrap();
            // Manually expire
            let past = Utc::now() - chrono::Duration::hours(2);
            let expired = SessionShare {
                expires_at: Some(past),
                ..s.clone()
            };
            let path = mgr.shares_dir.join(format!("{}.json", &s.token));
            let json = serde_json::to_string(&expired).unwrap();
            fs::write(&path, json).unwrap();
            s.token
        };

        let shares = mgr.list().unwrap();
        assert!(!shares.iter().any(|s| s.token == dead_token));
        assert!(shares.iter().any(|s| s.session_name == "alive"));
    }

    #[test]
    fn test_delete_share() {
        let (mgr, _dir) = temp_share_manager();
        let share = mgr
            .create("to-delete", serde_json::json!({"x": 1}), None)
            .unwrap();
        assert!(mgr.get(&share.token).is_ok());
        mgr.delete(&share.token).unwrap();
        assert!(mgr.get(&share.token).is_err());
    }

    #[test]
    fn test_get_nonexistent() {
        let (mgr, _dir) = temp_share_manager();
        let result = mgr.get("nonexistent-token");
        assert!(result.is_err());
    }
}
