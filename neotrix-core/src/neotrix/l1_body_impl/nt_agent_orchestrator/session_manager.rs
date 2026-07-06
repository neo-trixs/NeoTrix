use std::collections::HashMap;
use super::session_lifecycle::{AgentSession, SessionState};
use super::worktree_manager::{WorktreeManager, Worktree};

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub id: String,
    pub branch: String,
    pub base_branch: String,
    pub task_description: String,
    pub agent_command: Option<String>,
}

pub struct SessionManager {
    pub sessions: HashMap<String, AgentSession>,
    pub worktree_manager: WorktreeManager,
}

impl SessionManager {
    pub fn new(worktree_mgr: WorktreeManager) -> Self {
        Self {
            sessions: HashMap::new(),
            worktree_manager: worktree_mgr,
        }
    }

    pub fn spawn(&mut self, config: SessionConfig) -> Result<String, String> {
        if config.id.is_empty() || config.branch.is_empty() {
            return Err("Session config missing required fields".into());
        }

        let mut session = AgentSession::new(config.id.clone(), config.branch.clone());

        let worktree = self.worktree_manager.create(&config.branch, &config.base_branch)?;
        session.worktree_path = Some(worktree.path.to_string_lossy().to_string());

        let cmd = config.agent_command.unwrap_or_else(|| "opencode".to_string());
        session.metadata.insert("agent_command".into(), cmd);
        session.metadata.insert("task".into(), config.task_description);

        session.transition(SessionState::Working).ok();
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        Ok(id)
    }

    pub fn get(&self, id: &str) -> Option<&AgentSession> {
        self.sessions.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut AgentSession> {
        self.sessions.get_mut(id)
    }

    pub fn update_state(&mut self, id: &str, state: SessionState) -> Result<(), String> {
        if let Some(session) = self.sessions.get_mut(id) {
            session.transition(state)
        } else {
            Err(format!("Session {} not found", id))
        }
    }

    pub fn active_sessions(&self) -> Vec<&AgentSession> {
        self.sessions.values().filter(|s| !s.state.is_terminal()).collect()
    }

    pub fn cleanup(&mut self, id: &str) -> Result<(), String> {
        if let Some(session) = self.sessions.remove(id) {
            if let Some(path) = &session.worktree_path {
                let wt = Worktree {
                    path: std::path::PathBuf::from(path),
                    branch: session.branch.clone(),
                    created_at: session.created_at.clone(),
                };
                self.worktree_manager.remove(&wt).ok();
            }
            Ok(())
        } else {
            Err(format!("Session {} not found for cleanup", id))
        }
    }

    pub fn session_count(&self) -> (usize, usize) {
        let active = self.active_sessions().len();
        let total = self.sessions.len();
        (active, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_session_manager_empty() {
        let wt = WorktreeManager::new(PathBuf::from("/tmp"), PathBuf::from("/tmp/.git"));
        let mgr = SessionManager::new(wt);
        assert_eq!(mgr.session_count(), (0, 0));
    }

    #[test]
    fn test_session_manager_invalid_config() {
        let wt = WorktreeManager::new(PathBuf::from("/tmp"), PathBuf::from("/tmp/.git"));
        let mut mgr = SessionManager::new(wt);
        let result = mgr.spawn(SessionConfig {
            id: "".into(), branch: "".into(),
            base_branch: "main".into(),
            task_description: "test".into(),
            agent_command: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_active_sessions_filter() {
        let wt = WorktreeManager::new(PathBuf::from("/tmp"), PathBuf::from("/tmp/.git"));
        let mut mgr = SessionManager::new(wt);
        let mut session = AgentSession::new("s1".into(), "branch".into());
        session.worktree_path = Some("/tmp/wt".into());
        session.transition(SessionState::Working).ok();
        mgr.sessions.insert("s1".into(), session);

        let mut session2 = AgentSession::new("s2".into(), "branch2".into());
        session2.state = SessionState::Done;
        mgr.sessions.insert("s2".into(), session2);

        assert_eq!(mgr.active_sessions().len(), 1);
    }

    #[test]
    fn test_session_count() {
        let wt = WorktreeManager::new(PathBuf::from("/tmp"), PathBuf::from("/tmp/.git"));
        let mut mgr = SessionManager::new(wt);
        let mut s1 = AgentSession::new("s1".into(), "b1".into());
        s1.transition(SessionState::Working).ok();
        s1.worktree_path = Some("/tmp/wt1".into());
        mgr.sessions.insert("s1".into(), s1);

        let mut s2 = AgentSession::new("s2".into(), "b2".into());
        s2.state = SessionState::Done;
        mgr.sessions.insert("s2".into(), s2);

        assert_eq!(mgr.session_count(), (1, 2));
    }
}
