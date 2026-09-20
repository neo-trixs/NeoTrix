use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agent_identity::{AgentIdentity, AgentStatus};

/// Multi-agent coordinator — Grok Bot + Cumora patterns combined.
///
/// Manages agent identities, their presence in channels, turn-taking,
/// and coordination via the GLANCE_YIELD_RULES standing prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub identity: AgentIdentity,
    pub engine_id: Option<String>,
    pub model_tier: String,
    pub status: AgentStatus,
    pub current_channel: Option<String>,
    pub last_active: Option<String>,
}

/// Turn — tracks whose turn it is in a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub channel_id: String,
    pub agent_id: String,
    pub started_at: String,
    pub kind: TurnKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TurnKind {
    Normal,
    Urgent,
    Background,
}

/// Coordination event — emitted when agent state changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationEvent {
    pub kind: CoordinationEventKind,
    pub agent_id: String,
    pub channel_id: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationEventKind {
    AgentJoined,
    AgentLeft,
    TurnStarted,
    TurnEnded,
    MessagePosted,
    WakeSuppressed,
}

/// The coordinator — manages multi-agent interactions.
pub struct AgentCoordinator {
    agents: HashMap<String, Agent>,
    turns: HashMap<String, Turn>,
    events: Vec<CoordinationEvent>,
}

impl AgentCoordinator {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            turns: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Register an agent.
    pub fn register_agent(&mut self, agent: Agent) {
        tracing::info!(
            "registered agent: {} ({})",
            agent.identity.name,
            agent.identity.id
        );
        self.agents.insert(agent.identity.id.clone(), agent);
    }

    /// Get an agent by ID.
    pub fn get_agent(&self, id: &str) -> Option<&Agent> {
        self.agents.get(id)
    }

    /// List all agents.
    pub fn list_agents(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    /// Update agent status.
    pub fn set_status(&mut self, agent_id: &str, status: AgentStatus) {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.status = status;
        }
    }

    /// Start a turn for an agent in a channel.
    pub fn start_turn(&mut self, agent_id: &str, channel_id: &str, kind: TurnKind) -> Option<Turn> {
        // Check if another agent has the turn
        if let Some(current) = self.turns.get(channel_id) {
            if current.agent_id != agent_id {
                tracing::warn!(
                    "agent {agent_id} tried to take turn in {channel_id}, but {} has it",
                    current.agent_id
                );
                return None;
            }
        }

        let turn = Turn {
            channel_id: channel_id.to_string(),
            agent_id: agent_id.to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            kind,
        };

        self.turns.insert(channel_id.to_string(), turn.clone());

        self.events.push(CoordinationEvent {
            kind: CoordinationEventKind::TurnStarted,
            agent_id: agent_id.to_string(),
            channel_id: Some(channel_id.to_string()),
            payload: serde_json::json!({"turn_kind": "normal"}),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        Some(turn)
    }

    /// End a turn.
    pub fn end_turn(&mut self, channel_id: &str) {
        if let Some(turn) = self.turns.remove(channel_id) {
            self.events.push(CoordinationEvent {
                kind: CoordinationEventKind::TurnEnded,
                agent_id: turn.agent_id,
                channel_id: Some(channel_id.to_string()),
                payload: serde_json::json!({}),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    /// Get recent coordination events.
    pub fn recent_events(&self, limit: usize) -> Vec<&CoordinationEvent> {
        self.events.iter().rev().take(limit).collect()
    }
}

impl Default for AgentCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
