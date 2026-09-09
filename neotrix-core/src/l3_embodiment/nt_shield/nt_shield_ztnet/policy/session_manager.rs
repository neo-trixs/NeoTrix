//! C5: Session Manager — 会话管理
//!
//! 管理ZTA会话生命周期

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 会话
#[derive(Debug, Clone)]
pub struct ZtaSession {
    pub id: String,
    pub subject: String,
    pub resource: String,
    pub created_at: Instant,
    pub expires_at: Instant,
    pub active: bool,
}

/// 会话管理器
pub struct SessionManager {
    sessions: HashMap<String, ZtaSession>,
    default_lifetime: Duration,
}

impl SessionManager {
    pub fn new(default_lifetime: Duration) -> Self {
        Self {
            sessions: HashMap::new(),
            default_lifetime,
        }
    }

    pub fn create_session(&mut self, subject: String, resource: String) -> String {
        let id = format!("session-{}", self.sessions.len());
        let session = ZtaSession {
            id: id.clone(),
            subject,
            resource,
            created_at: Instant::now(),
            expires_at: Instant::now() + self.default_lifetime,
            active: true,
        };
        self.sessions.insert(id.clone(), session);
        id
    }

    pub fn validate_session(&self, session_id: &str) -> bool {
        if let Some(session) = self.sessions.get(session_id) {
            session.active && Instant::now() < session.expires_at
        } else {
            false
        }
    }

    pub fn destroy_session(&mut self, session_id: &str) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.active = false;
            true
        } else {
            false
        }
    }

    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        for session in self.sessions.values_mut() {
            if now >= session.expires_at {
                session.active = false;
            }
        }
    }

    pub fn sessions(&self) -> &HashMap<String, ZtaSession> {
        &self.sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_lifecycle() {
        let mut manager = SessionManager::new(Duration::from_secs(3600));

        let id = manager.create_session("user1".into(), "tunnel/test".into());
        assert!(manager.validate_session(&id));

        manager.destroy_session(&id);
        assert!(!manager.validate_session(&id));
    }
}
