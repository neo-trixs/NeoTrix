use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInvocation {
    pub id: Uuid,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub caller_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub invocation_id: Uuid,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub success: bool,
}
