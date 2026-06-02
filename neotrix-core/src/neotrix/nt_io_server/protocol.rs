use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ACPMessage {
    Ping,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ACPResponse {
    Pong,
    Error { code: i32, message: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub output: String,
}
