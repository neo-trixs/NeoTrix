//! Hive-related Tauri commands for OfficeFloor visualization

use serde::{Deserialize, Serialize};

/// Agent data for OfficeFloor visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFloorData {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub status: String,
    pub specialty: String,
    pub x: f64,
    pub y: f64,
    pub inbox_depth: usize,
    pub last_output: Option<String>,
}

/// Message data for OfficeFloor particle animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFloorMessage {
    pub from: String,
    pub to: String,
    pub msg_type: String,
    pub timestamp: u64,
}

/// Full floor state returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorState {
    pub agents: Vec<AgentFloorData>,
    pub messages: Vec<AgentFloorMessage>,
    pub stats: HiveStatsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveStatsData {
    pub total_messages: usize,
    pub total_agents: usize,
    pub blackboard_entries: usize,
    pub events_count: usize,
}

/// Get current floor state from HiveRouter
/// Returns agent positions, statuses, and recent messages
#[tauri::command]
pub fn hive_get_floor_state() -> FloorState {
    // Default floor state when no real agents are connected
    // This will be enhanced with real data once HiveRouter is wired
    let agents = vec![
        AgentFloorData {
            id: "god-agent".to_string(),
            name: "GOD Agent".to_string(),
            avatar: "🧠".to_string(),
            status: "running".to_string(),
            specialty: "orchestration".to_string(),
            x: 0.5,
            y: 0.3,
            inbox_depth: 0,
            last_output: None,
        },
        AgentFloorData {
            id: "coder-1".to_string(),
            name: "编码员".to_string(),
            avatar: "💻".to_string(),
            status: "idle".to_string(),
            specialty: "code_generation".to_string(),
            x: 0.2,
            y: 0.6,
            inbox_depth: 0,
            last_output: None,
        },
        AgentFloorData {
            id: "reviewer-1".to_string(),
            name: "审查员".to_string(),
            avatar: "🔍".to_string(),
            status: "idle".to_string(),
            specialty: "code_review".to_string(),
            x: 0.8,
            y: 0.6,
            inbox_depth: 0,
            last_output: None,
        },
    ];

    FloorState {
        agents,
        messages: vec![],
        stats: HiveStatsData {
            total_messages: 0,
            total_agents: 3,
            blackboard_entries: 0,
            events_count: 0,
        },
    }
}

/// Send a message between agents (for testing/demo)
#[tauri::command]
pub fn hive_send_message(from: String, to: String, msg_type: String, content: String) -> String {
    // Will be wired to HiveRouter in future iteration
    format!("Message sent: {} → {} ({})", from, to, msg_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floor_state_default() {
        let state = hive_get_floor_state();
        assert_eq!(state.agents.len(), 3);
        assert_eq!(state.agents[0].id, "god-agent");
        assert_eq!(state.stats.total_agents, 3);
    }

    #[test]
    fn test_send_message() {
        let result = hive_send_message(
            "agent-a".to_string(),
            "agent-b".to_string(),
            "task".to_string(),
            "do something".to_string(),
        );
        assert!(result.contains("agent-a"));
        assert!(result.contains("agent-b"));
    }
}
