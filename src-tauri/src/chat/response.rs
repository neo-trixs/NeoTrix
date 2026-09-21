use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Response from the chat system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Natural language reply to the user
    pub message: String,
    /// Actions that were performed (for UI feedback)
    #[serde(default)]
    pub actions: Vec<ChatAction>,
    /// Updated state to reflect in UI (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<Value>,
}

/// Record of an action performed by the chat system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatAction {
    /// Which domain was called (e.g., "proxy_pool", "im", "model_pool")
    pub domain: String,
    /// What action was performed (e.g., "add", "status", "toggle")
    pub action: String,
    /// Action result
    pub result: Value,
}
