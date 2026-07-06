use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub created_at: i64,
    pub messages: Vec<SessionMessage>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStore {
    sessions: Vec<SessionRecord>,
    max_sessions: usize,
}

impl SessionStore {
    pub fn new(max_sessions: usize) -> Self {
        Self {
            sessions: Vec::new(),
            max_sessions,
        }
    }

    pub fn create(&mut self) -> SessionRecord {
        let now = chrono::Utc::now().timestamp();
        let id = format!("session-{}", uuid::Uuid::new_v4());
        let record = SessionRecord {
            id,
            created_at: now,
            messages: Vec::new(),
            metadata: HashMap::new(),
        };
        self.sessions.push(record.clone());
        if self.sessions.len() > self.max_sessions {
            self.sessions.remove(0);
        }
        record
    }

    pub fn get(&self, id: &str) -> Option<&SessionRecord> {
        self.sessions.iter().find(|s| s.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut SessionRecord> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    pub fn add_message(&mut self, session_id: &str, msg: SessionMessage) -> Result<(), String> {
        let session = self.get_mut(session_id).ok_or_else(|| format!("session not found: {}", session_id))?;
        session.messages.push(msg);
        Ok(())
    }

    pub fn all(&self) -> &[SessionRecord] {
        &self.sessions
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn max_sessions(&self) -> usize {
        self.max_sessions
    }

    pub fn clear(&mut self) {
        self.sessions.clear();
    }

    pub fn remove(&mut self, id: &str) -> Option<SessionRecord> {
        let idx = self.sessions.iter().position(|s| s.id == id)?;
        Some(self.sessions.remove(idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(role: &str, content: &str) -> SessionMessage {
        SessionMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    #[test]
    fn test_session_store_create() {
        let mut store = SessionStore::new(10);
        let session = store.create();
        assert!(!session.id.is_empty());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_session_store_get() {
        let mut store = SessionStore::new(10);
        let session = store.create();
        let retrieved = store.get(&session.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("session should exist").id, session.id);
    }

    #[test]
    fn test_session_store_get_mut() {
        let mut store = SessionStore::new(10);
        let session = store.create();
        let retrieved = store.get_mut(&session.id);
        assert!(retrieved.is_some());
        retrieved.expect("session should exist").metadata.insert("key".into(), "val".into());
        assert_eq!(store.get(&session.id).expect("session exists").metadata.get("key").expect("key exists"), "val");
    }

    #[test]
    fn test_add_message() {
        let mut store = SessionStore::new(10);
        let session = store.create();
        let msg = make_msg("user", "hello");
        assert!(store.add_message(&session.id, msg).is_ok());
        assert_eq!(store.get(&session.id).expect("session exists").messages.len(), 1);
    }

    #[test]
    fn test_add_message_nonexistent_session() {
        let mut store = SessionStore::new(10);
        let msg = make_msg("user", "hello");
        let result = store.add_message("nonexistent", msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_sessions_eviction() {
        let mut store = SessionStore::new(2);
        let s1 = store.create();
        let s2 = store.create();
        let s3 = store.create();
        assert_eq!(store.len(), 2);
        assert!(store.get(&s1.id).is_none());
        assert!(store.get(&s2.id).is_some());
        assert!(store.get(&s3.id).is_some());
    }

    #[test]
    fn test_all_returns_all_sessions() {
        let mut store = SessionStore::new(10);
        store.create();
        store.create();
        assert_eq!(store.all().len(), 2);
    }

    #[test]
    fn test_remove() {
        let mut store = SessionStore::new(10);
        let session = store.create();
        assert_eq!(store.len(), 1);
        let removed = store.remove(&session.id);
        assert!(removed.is_some());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_clear() {
        let mut store = SessionStore::new(10);
        store.create();
        store.create();
        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn test_empty_session_store() {
        let store: SessionStore = SessionStore::new(5);
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_session_message_fields() {
        let msg = make_msg("assistant", "hello world");
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, "hello world");
        assert!(msg.timestamp > 0);
    }
}
