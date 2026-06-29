use nt_domain::*;

pub trait AgentRepo: Send + Sync {
    fn create_agent(&self, profile: &AgentProfile) -> Result<AgentProfile, String>;
    fn get_agent(&self, id: uuid::Uuid) -> Option<AgentProfile>;
    fn list_agents(&self, workspace_id: uuid::Uuid) -> Vec<AgentProfile>;
    fn update_agent_status(&self, id: uuid::Uuid, status: AgentStatus) -> bool;
    fn delete_agent(&self, id: uuid::Uuid) -> bool;
}
