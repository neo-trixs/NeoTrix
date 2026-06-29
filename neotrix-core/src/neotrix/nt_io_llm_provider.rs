//! LLM Provider core types extracted from neotrix layer to break circular import.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub tools: Vec<Tool>,
    pub image_data: Option<Vec<u8>>,
}

impl LlmRequest {
    pub fn new(model: &str, prompt: &str) -> Self {
        Self {
            model: model.to_string(),
            messages: vec![Message {
                role: Role::User,
                content: prompt.to_string(),
                tool_calls: None,
                tool_call_id: None,
            }],
            temperature: 0.7,
            max_tokens: 4096,
            tools: vec![],
            image_data: None,
        }
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    Tool,
    ContentFilter,
    Unknown,
}

#[derive(Debug)]
pub enum LlmError {
    Network(String),
    Authentication(String),
    RateLimit(String),
    InvalidRequest(String),
    Server(String),
    Unknown(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Network(e) => write!(f, "Network: {}", e),
            LlmError::Authentication(e) => write!(f, "Auth: {}", e),
            LlmError::RateLimit(e) => write!(f, "RateLimit: {}", e),
            LlmError::InvalidRequest(e) => write!(f, "Invalid: {}", e),
            LlmError::Server(e) => write!(f, "Server: {}", e),
            LlmError::Unknown(e) => write!(f, "Unknown: {}", e),
        }
    }
}

impl std::error::Error for LlmError {}

impl From<String> for LlmError {
    fn from(s: String) -> Self {
        LlmError::Unknown(s)
    }
}
impl From<&str> for LlmError {
    fn from(s: &str) -> Self {
        LlmError::Unknown(s.to_string())
    }
}

/// LLM Provider trait — the minimal interface needed by ImagePipeline.
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;
    async fn stream_complete(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message {
            role: Role::User,
            content: "hello".into(),
            tool_calls: None,
            tool_call_id: None,
        };
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "hello");
    }

    #[test]
    fn test_llm_request_new() {
        let req = LlmRequest::new("gpt-4", "test prompt");
        assert_eq!(req.model, "gpt-4");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, Role::User);
        assert_eq!(req.messages[0].content, "test prompt");
        assert!((req.temperature - 0.7).abs() < 1e-9);
        assert_eq!(req.max_tokens, 4096);
    }

    #[test]
    fn test_llm_request_builder() {
        let tool = Tool {
            name: "search".into(),
            description: "web search".into(),
            input_schema: serde_json::json!({}),
        };
        let req = LlmRequest::new("claude", "q")
            .with_tools(vec![tool])
            .with_max_tokens(1024);
        assert_eq!(req.tools.len(), 1);
        assert_eq!(req.max_tokens, 1024);
    }
}
