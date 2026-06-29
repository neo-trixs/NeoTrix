use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use nt_domain::*;
use uuid::Uuid;

use crate::workspace_svc::WorkspaceService;

pub struct AgentService {
    agents: Arc<Mutex<HashMap<Uuid, AgentProfile>>>,
    workspace_svc: WorkspaceService,
}

impl AgentService {
    pub fn new(workspace_svc: WorkspaceService) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            workspace_svc,
        }
    }

    pub fn register_agent(&self, ws_id: Uuid, name: &str, role: &str, owner_id: Uuid) -> Result<AgentProfile, String> {
        if name.is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }
        self.workspace_svc.get_workspace(ws_id)?;

        let agent = AgentProfile {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            name: name.to_string(),
            role: role.to_string(),
            owner_id,
            status: AgentStatus::Idle,
            created_at: Utc::now(),
        };
        self.agents.lock().unwrap().insert(agent.id, agent.clone());
        Ok(agent)
    }

    pub fn get_agent(&self, id: Uuid) -> Result<AgentProfile, String> {
        self.agents.lock().unwrap().get(&id).cloned().ok_or_else(|| "Agent not found".to_string())
    }

    pub fn list_agents(&self, ws_id: Uuid) -> Vec<AgentProfile> {
        self.agents.lock().unwrap().values().filter(|a| a.workspace_id == ws_id).cloned().collect()
    }

    pub fn update_status(&self, id: Uuid, status: AgentStatus) -> Result<AgentProfile, String> {
        let mut agents = self.agents.lock().unwrap();
        if let Some(agent) = agents.get_mut(&id) {
            agent.status = status;
            Ok(agent.clone())
        } else {
            Err("Agent not found".to_string())
        }
    }

    pub fn delete_agent(&self, id: Uuid) -> Result<(), String> {
        self.agents.lock().unwrap().remove(&id).ok_or_else(|| "Agent not found".to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace_svc::{MemWorkspaceRepo, WorkspaceService};

    fn test_setup() -> (AgentService, Uuid) {
        let ws_svc = WorkspaceService::new(Box::new(MemWorkspaceRepo::new()));
        let ws = ws_svc.create_workspace("test-ws", Uuid::new_v4()).unwrap();
        let svc = AgentService::new(ws_svc);
        (svc, ws.id)
    }

    #[test]
    fn test_register_agent() {
        let (svc, ws_id) = test_setup();
        let agent = svc.register_agent(ws_id, "agent-1", "coder", Uuid::new_v4()).unwrap();
        assert_eq!(agent.name, "agent-1");
        assert_eq!(agent.role, "coder");
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn test_register_empty_name() {
        let (svc, ws_id) = test_setup();
        assert!(svc.register_agent(ws_id, "", "coder", Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_register_invalid_workspace() {
        let svc = AgentService::new(WorkspaceService::new(Box::new(MemWorkspaceRepo::new())));
        assert!(svc.register_agent(Uuid::new_v4(), "agent", "coder", Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_get_agent() {
        let (svc, ws_id) = test_setup();
        let agent = svc.register_agent(ws_id, "agent-1", "coder", Uuid::new_v4()).unwrap();
        let found = svc.get_agent(agent.id).unwrap();
        assert_eq!(found.id, agent.id);
    }

    #[test]
    fn test_get_agent_not_found() {
        let (svc, _) = test_setup();
        assert!(svc.get_agent(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_list_agents() {
        let (svc, ws_id) = test_setup();
        svc.register_agent(ws_id, "a1", "coder", Uuid::new_v4()).unwrap();
        svc.register_agent(ws_id, "a2", "reviewer", Uuid::new_v4()).unwrap();
        assert_eq!(svc.list_agents(ws_id).len(), 2);
    }

    #[test]
    fn test_update_status() {
        let (svc, ws_id) = test_setup();
        let agent = svc.register_agent(ws_id, "agent", "coder", Uuid::new_v4()).unwrap();
        let updated = svc.update_status(agent.id, AgentStatus::Busy).unwrap();
        assert_eq!(updated.status, AgentStatus::Busy);
    }

    #[test]
    fn test_delete_agent() {
        let (svc, ws_id) = test_setup();
        let agent = svc.register_agent(ws_id, "agent", "coder", Uuid::new_v4()).unwrap();
        svc.delete_agent(agent.id).unwrap();
        assert!(svc.get_agent(agent.id).is_err());
    }
}
