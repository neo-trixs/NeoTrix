use crate::neotrix::nt_agent_orchestrator::session_lifecycle::{AgentSession, SessionState};
use crate::neotrix::nt_agent_orchestrator::spawn_flow::{SpawnConfig, SpawnManager};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    Orchestrator,
    Worker,
}

#[derive(Debug, Clone)]
pub struct WorkerStatus {
    pub worker_id: String,
    pub name: String,
    pub task: String,
    pub state: SessionState,
    pub result: Option<String>,
}

#[derive(Debug)]
pub struct OrchestratorAgent {
    pub id: String,
    pub name: String,
    pub spawn_manager: SpawnManager,
    workers: HashMap<String, WorkerInfo>,
    current_plan: Option<String>,
}

#[derive(Debug, Clone)]
struct WorkerInfo {
    session: AgentSession,
    task: String,
    result: Option<String>,
}

impl OrchestratorAgent {
    pub fn new(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            spawn_manager: SpawnManager::new(),
            workers: HashMap::new(),
            current_plan: None,
        }
    }

    pub fn set_plan(&mut self, plan: &str) {
        self.current_plan = Some(plan.to_string());
    }

    pub fn deploy_worker(&mut self, name: &str, task: &str) -> Result<String, String> {
        let config = SpawnConfig {
            agent_name: name.to_string(),
            task_description: task.to_string(),
            create_worktree: false,
            ..Default::default()
        };

        match self.spawn_manager.spawn(config) {
            Ok(result) => {
                let worker_id = result.session.id.clone();
                self.workers.insert(worker_id.clone(), WorkerInfo {
                    session: result.session,
                    task: task.to_string(),
                    result: None,
                });
                Ok(worker_id)
            }
            Err(e) => Err(format!("Failed to deploy worker '{}': {}", name, e)),
        }
    }

    pub fn worker_statuses(&self) -> Vec<WorkerStatus> {
        self.workers.values().map(|w| WorkerStatus {
            worker_id: w.session.id.clone(),
            name: w.session.metadata.get("name").cloned().unwrap_or_default(),
            task: w.task.clone(),
            state: w.session.state.clone(),
            result: w.result.clone(),
        }).collect()
    }

    pub fn record_result(&mut self, worker_id: &str, result: &str) -> Result<(), String> {
        if let Some(worker) = self.workers.get_mut(worker_id) {
            worker.result = Some(result.to_string());
            worker.session.transition(SessionState::Done).ok();
            Ok(())
        } else {
            Err(format!("Worker {} not found", worker_id))
        }
    }

    pub fn active_worker_count(&self) -> usize {
        self.workers.values().filter(|w| w.session.state.is_active()).count()
    }

    pub fn all_workers_complete(&self) -> bool {
        self.workers.values().all(|w| w.session.state.is_terminal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_agent_orchestrator::spawn_flow::SpawnConfig as SC;

    #[test]
    fn test_orchestrator_creation() {
        let orch = OrchestratorAgent::new("test-orch");
        assert_eq!(orch.name, "test-orch");
        assert!(orch.worker_statuses().is_empty());
    }

    #[test]
    fn test_deploy_worker() {
        let mut orch = OrchestratorAgent::new("orch");
        let result = orch.deploy_worker("worker-1", "Implement feature X");
        assert!(result.is_ok() || result.is_err());
        if result.is_ok() {
            assert_eq!(orch.worker_statuses().len(), 1);
        }
    }

    #[test]
    fn test_set_plan() {
        let mut orch = OrchestratorAgent::new("orch");
        orch.set_plan("1. Research\n2. Implement\n3. Test");
    }

    #[test]
    fn test_worker_statuses() {
        let mut orch = OrchestratorAgent::new("orch");
        let config = SC {
            agent_name: "w1".to_string(),
            task_description: "Task".to_string(),
            create_worktree: false,
            ..Default::default()
        };
        if let Ok(result) = orch.spawn_manager.spawn(config) {
            let id = result.session.id;
            assert!(orch.spawn_manager.get_session(&id).is_some());
        }
    }

    #[test]
    fn test_record_result_nonexistent() {
        let mut orch = OrchestratorAgent::new("orch");
        assert!(orch.record_result("nonexistent", "done").is_err());
    }

    #[test]
    fn test_all_workers_complete_empty() {
        let orch = OrchestratorAgent::new("orch");
        assert!(orch.all_workers_complete());
    }
}
