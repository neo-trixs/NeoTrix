use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A2A Task — the core work unit exchanged between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ATask {
    pub id: String,
    pub session_id: String,
    pub status: TaskState,
    pub messages: Vec<A2AMessage>,
    pub artifacts: Vec<A2AArtifact>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskState {
    Submitted,
    Working,
    Completed,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub role: String,
    pub parts: Vec<A2APart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2APart {
    pub r#type: A2APartType,
    pub text: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum A2APartType {
    Text,
    Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AArtifact {
    pub id: String,
    pub name: String,
    pub mime_type: String,
}

// ── Request/Response types ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTaskRequest {
    pub id: String,
    pub session_id: String,
    pub messages: Vec<A2AMessage>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTaskResponse {
    pub task: A2ATask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationOffer {
    pub versions: Vec<String>,
    #[serde(rename = "agentName")]
    pub agent_name: String,
    #[serde(rename = "negotiationId")]
    pub negotiation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationResponse {
    #[serde(rename = "selectedVersion")]
    pub selected_version: String,
    #[serde(rename = "negotiationId")]
    pub negotiation_id: String,
    pub accepted: bool,
}
