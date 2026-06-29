use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionRequest {
    pub agent_id: Uuid,
    pub task: String,
    pub context: Option<serde_json::Value>,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionResponse {
    pub request_id: Uuid,
    pub status: ExecutionStatus,
    pub output: String,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HarnessKind {
    ClaudeCode,
    Codex,
    OpenAI,
    Generic,
}
