//! # Self-Organization Protocol — Agent Heartbeat & Self-Organization
//!
//! Types for agent heartbeat monitoring and self-organizing agent teams.

pub mod state;

pub use state::{DeadEndRecord, DeadEndRegistry, SharedState};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent operational status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Idle,
    Busy,
    Overloaded,
    Failed,
    Offline,
}

/// Heartbeat signal from an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub agent_id: String,
    pub timestamp: i64,
    pub status: AgentStatus,
    pub load: f64,
    pub capacity: f64,
    pub capabilities: Vec<String>,
}

/// Metadata about a registered agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub id: String,
    pub last_heartbeat: i64,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub specialties: Vec<String>,
}

/// Self-organization protocol for managing agent teams
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
