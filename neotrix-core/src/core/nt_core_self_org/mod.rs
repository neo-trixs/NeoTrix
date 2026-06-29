pub mod state;

pub use state::{DeadEndRecord, DeadEndRegistry, SharedState};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum AgentStatus {
    Idle,
    Busy,
    Overloaded,
    Failed,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub agent_id: String,
    pub timestamp: i64,
    pub status: AgentStatus,
    pub load: f64,
    pub capacity: f64,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub id: String,
    pub last_heartbeat: i64,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub specialties: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelfOrgProtocol {
    pub agents: HashMap<String, AgentMetadata>,
    pub heartbeat_interval_secs: u64,
}

impl SelfOrgProtocol {
    pub fn new(heartbeat_interval_secs: u64) -> Self {
        Self {
            agents: HashMap::new(),
            heartbeat_interval_secs,
        }
    }

    pub fn register_agent(&mut self, id: String, specialties: Vec<String>) {
        let metadata = AgentMetadata {
            id: id.clone(),
            last_heartbeat: 0,
            status: AgentStatus::Idle,
            current_task: None,
            specialties,
        };
        self.agents.insert(id, metadata);
    }

    pub fn process_heartbeat(&mut self, heartbeat: Heartbeat) {
        if let Some(metadata) = self.agents.get_mut(&heartbeat.agent_id) {
            metadata.last_heartbeat = heartbeat.timestamp;
            metadata.status = heartbeat.status;
        }
    }

    pub fn get_agent(&self, id: &str) -> Option<&AgentMetadata> {
        self.agents.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_org_protocol_new() {
        let mut p = SelfOrgProtocol::new(30);
        assert_eq!(p.heartbeat_interval_secs, 30);
        assert!(p.agents.is_empty());
        p.register_agent("agent1".into(), vec!["search".into()]);
        assert!(p.get_agent("agent1").is_some());
    }

    #[test]
    fn test_heartbeat_processing() {
        let mut p = SelfOrgProtocol::new(10);
        p.register_agent("a".into(), vec![]);
        let hb = Heartbeat {
            agent_id: "a".into(),
            timestamp: 1000,
            status: AgentStatus::Busy,
            load: 0.5,
            capacity: 1.0,
            capabilities: vec![],
        };
        p.process_heartbeat(hb);
        let agent = p.get_agent("a").unwrap();
        assert_eq!(agent.last_heartbeat, 1000);
        assert_eq!(agent.status, AgentStatus::Busy);
    }
}
