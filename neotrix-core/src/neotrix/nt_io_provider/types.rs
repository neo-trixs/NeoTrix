//! LLM Provider 核心类型定义

use serde::{Deserialize, Serialize};

#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;
    async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError>;
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub tools: Vec<Tool>,
    pub image_data: Option<String>,
}

impl LlmRequest {
    pub fn new(model: &str, prompt: &str) -> Self {
        Self {
            model: model.to_string(),
            messages: vec![Message {
                role: Role::User,
                content: prompt.to_string(),
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
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
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
