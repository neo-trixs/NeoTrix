use crate::neotrix::agent_orchestrator::session_lifecycle::{AgentSession, SessionState};
use crate::neotrix::agent_orchestrator::worktree_manager::{WorktreeManager, Worktree};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SpawnResult {
    pub session: AgentSession,
    pub worktree: Option<Worktree>,
    pub workspace_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SpawnConfig {
    pub agent_name: String,
    pub task_description: String,
    pub repo_path: Option<PathBuf>,
    pub branch: Option<String>,
    pub create_worktree: bool,
    pub max_iterations: u32,
    pub worktree_base: Option<PathBuf>,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            agent_name: "default-agent".to_string(),
            task_description: String::new(),
            repo_path: None,
            branch: None,
            create_worktree: true,
            max_iterations: 10,
            worktree_base: None,
        }
    }
}

#[derive(Debug)]
pub struct SpawnManager {
    sessions: Vec<AgentSession>,
    spawn_count: u64,
}

impl SpawnManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            spawn_count: 0,
        }
    }

    pub fn spawn(&mut self, config: SpawnConfig) -> Result<SpawnResult, String> {
        if config.task_description.is_empty() {
            return Err("Task description cannot be empty".to_string());
        }

        let branch = config.branch.unwrap_or_else(|| format!("agent-{}", self.spawn_count));
        let id = uuid::Uuid::new_v4().to_string();
        let mut session = AgentSession::new(id, branch.clone());

        let worktree = if config.create_worktree {
            if let Some(ref repo_path) = config.repo_path {
                let base_dir = config.worktree_base.clone()
                    .unwrap_or_else(|| PathBuf::from("/tmp/neotrix-worktrees"));
                let git_dir = repo_path.join(".git");
                let mgr = WorktreeManager::new(base_dir, git_dir);
                match mgr.create(&branch, "main") {
                    Ok(wt) => {
                        session.worktree_path = Some(wt.path.to_string_lossy().to_string());
                        Some(wt)
                    }
                    Err(e) => {
                        session.transition(SessionState::Failed).ok();
                        return Err(format!("Worktree creation failed: {}", e));
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let workspace_path = worktree.as_ref()
            .map(|wt| wt.path.clone())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        session.metadata.insert("task".to_string(), config.task_description.clone());
        session.metadata.insert("workspace".to_string(), workspace_path.to_string_lossy().to_string());
        session.metadata.insert("name".to_string(), config.agent_name.clone());

        session.transition(SessionState::Working).ok();

        let session_clone = session.clone();
        self.sessions.push(session);
        self.spawn_count += 1;

        Ok(SpawnResult {
            session: session_clone,
            worktree,
            workspace_path,
        })
    }

    pub fn active_sessions(&self) -> Vec<&AgentSession> {
        self.sessions.iter().filter(|s| s.state.is_active()).collect()
    }

    pub fn get_session(&self, id: &str) -> Option<&AgentSession> {
        self.sessions.iter().find(|s| s.id == id)
    }

    pub fn spawn_count(&self) -> u64 {
        self.spawn_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_empty_task() {
        let mut mgr = SpawnManager::new();
        let config = SpawnConfig {
            agent_name: "test".to_string(),
            task_description: "".to_string(),
            ..Default::default()
        };
        assert!(mgr.spawn(config).is_err());
    }

    #[test]
    fn test_spawn_basic() {
        let mut mgr = SpawnManager::new();
        let config = SpawnConfig {
            agent_name: "test".to_string(),
            task_description: "Implement feature X".to_string(),
            create_worktree: false,
            ..Default::default()
        };
        let result = mgr.spawn(config).expect("spawn should succeed for basic config");
        assert_eq!(result.session.state, SessionState::Working);
        assert!(result.worktree.is_none());
    }

    #[test]
    fn test_active_sessions() {
        let mut mgr = SpawnManager::new();
        let config = SpawnConfig {
            agent_name: "a".to_string(),
            task_description: "Task A".to_string(),
            create_worktree: false,
            ..Default::default()
        };
        mgr.spawn(config).expect("spawn should succeed for active session test");
        assert_eq!(mgr.active_sessions().len(), 1);
        assert_eq!(mgr.spawn_count(), 1);
    }

    #[test]
    fn test_get_session() {
        let mut mgr = SpawnManager::new();
        let config = SpawnConfig {
            agent_name: "s1".to_string(),
            task_description: "Task".to_string(),
            create_worktree: false,
            ..Default::default()
        };
        let result = mgr.spawn(config).expect("spawn should succeed for get_session test");
        let found = mgr.get_session(&result.session.id);
        assert!(found.is_some());
        assert_eq!(found.expect("session should exist").metadata.get("name").expect("session name should exist"), "s1");
    }

    #[test]
    fn test_multiple_spawns() {
        let mut mgr = SpawnManager::new();
        for i in 0..3 {
            let config = SpawnConfig {
                agent_name: format!("agent-{}", i),
                task_description: format!("Task {}", i),
                create_worktree: false,
                ..Default::default()
            };
            mgr.spawn(config).expect("spawn should succeed for multiple spawns");
        }
        assert_eq!(mgr.spawn_count(), 3);
        assert_eq!(mgr.active_sessions().len(), 3);
    }
}
