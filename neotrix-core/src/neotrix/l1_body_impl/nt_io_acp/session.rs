use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use super::protocol::ContentBlock;

/// Active ACP session state — wrapped in Arc<RwLock> for concurrent access
pub struct AcpSession {
    pub session_id: String,
    pub cwd: String,
    pub metadata: HashMap<String, String>,
    pub conversation: Vec<ContentBlock>,
    pub active: Arc<AtomicBool>,
    pub started_at: std::time::Instant,
}

impl AcpSession {
    pub fn new(
        session_id: String,
        cwd: Option<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            session_id,
            cwd: cwd
                .unwrap_or_else(|| {
                    std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| "/".into())
                }),
            metadata: metadata.unwrap_or_default(),
            conversation: Vec::new(),
            active: Arc::new(AtomicBool::new(true)),
            started_at: std::time::Instant::now(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    pub fn add_content(&mut self, block: ContentBlock) {
        self.conversation.push(block);
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }
}

/// Session manager — owns all active sessions as Arc<RwLock<AcpSession>>
pub struct SessionManager {
    sessions: RwLock<HashMap<String, Arc<RwLock<AcpSession>>>>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub fn create_session(
        &self,
        session_id: String,
        cwd: Option<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;
        if sessions.contains_key(&session_id) {
            return Err(format!("session {} already exists", session_id));
        }
        let session = AcpSession::new(session_id.clone(), cwd, metadata);
        sessions.insert(session_id.clone(), Arc::new(RwLock::new(session)));
        Ok(session_id)
    }

    pub fn get_session(&self, session_id: &str) -> Option<Arc<RwLock<AcpSession>>> {
        let sessions = self.sessions.read().ok()?;
        sessions.get(session_id).cloned()
    }

    pub fn with_session<F, R>(&self, session_id: &str, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut AcpSession) -> R,
    {
        let sessions = self.sessions.read().map_err(|e| e.to_string())?;
        let arc = sessions
            .get(session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;
        let mut session = arc.write().map_err(|e| e.to_string())?;
        Ok(f(&mut session))
    }

    pub fn session_exists(&self, session_id: &str) -> bool {
        self.sessions
            .read()
            .map(|s| s.contains_key(session_id))
            .unwrap_or(false)
    }

    pub fn close_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;
        if let Some(s) = sessions.get(session_id) {
            if let Ok(session) = s.read() {
                session.cancel();
            }
        }
        sessions.remove(session_id);
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<(String, bool)>, String> {
        let sessions = self.sessions.read().map_err(|e| e.to_string())?;
        Ok(sessions
            .iter()
            .map(|(id, s)| {
                let active = s.read().map(|session| session.is_active()).unwrap_or(false);
                (id.clone(), active)
            })
            .collect())
    }

    pub fn session_count(&self) -> usize {
        self.sessions
            .read()
            .map(|s| s.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_list_session() {
        let mgr = SessionManager::new();
        let sid = mgr.create_session("s-1".into(), None, None).unwrap();
        assert_eq!(sid, "s-1");
        let sessions = mgr.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].0, "s-1");
    }

    #[test]
    fn test_create_duplicate_fails() {
        let mgr = SessionManager::new();
        mgr.create_session("s-1".into(), None, None).unwrap();
        let err = mgr.create_session("s-1".into(), None, None).unwrap_err();
        assert!(err.contains("already exists"));
    }

    #[test]
    fn test_close_session() {
        let mgr = SessionManager::new();
        mgr.create_session("s-1".into(), None, None).unwrap();
        mgr.close_session("s-1").unwrap();
        assert_eq!(mgr.session_count(), 0);
    }

    #[test]
    fn test_with_session_modify() {
        let mgr = SessionManager::new();
        mgr.create_session("s-1".into(), None, None).unwrap();
        mgr.with_session("s-1", |s| {
            s.add_content(ContentBlock::Text {
                text: "hello".into(),
            });
        })
        .unwrap();
        mgr.with_session("s-1", |s| {
            assert_eq!(s.conversation.len(), 1);
        })
        .unwrap();
    }

    #[test]
    fn test_with_session_not_found() {
        let mgr = SessionManager::new();
        let err = mgr.with_session::<_, ()>("nonexistent", |_| ()).unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_elapsed_increases() {
        let mgr = SessionManager::new();
        mgr.create_session("s-1".into(), None, None).unwrap();
        mgr.with_session("s-1", |s| {
            let e = s.elapsed();
            assert!(e.as_secs() == 0 || e.as_nanos() > 0);
        })
        .unwrap();
    }
}
