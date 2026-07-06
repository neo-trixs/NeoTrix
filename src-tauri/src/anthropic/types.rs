use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct MessageRequest {
    pub model: String,
    pub max_tokens: u32,
    pub stream: bool,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ContentBlock {
    Text { text: String, #[serde(skip_serializing_if = "Option::is_none")] r#type: Option<String> },
    Image { source: ImageSource, #[serde(skip_serializing_if = "Option::is_none")] r#type: Option<String> },
    ToolUse { name: String, input: serde_json::Value, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] r#type: Option<String> },
    ToolResult { tool_use_id: String, content: String, #[serde(skip_serializing_if = "Option::is_none")] r#type: Option<String> },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageSource {
    pub r#type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub delta: Option<Delta>,
    #[serde(default)]
    pub content_block: Option<ContentBlockHeader>,
    #[serde(default)]
    pub message: Option<MessageResponse>,
    #[serde(default)]
    pub index: Option<u32>,
    #[serde(default)]
    pub error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Delta {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub partial_json: Option<String>,
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub stop_sequence: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ContentBlockHeader {
    pub r#type: String,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MessageResponse {
    pub id: String,
    #[serde(default)]
    pub r#type: String,
    pub role: String,
    #[serde(default)]
    pub content: Vec<ContentBlock>,
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub stop_sequence: Option<String>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiError {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub model: String,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub attachments: Option<String>,
    pub status: String,
    pub created_at: i64,
}
