use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 数据信任分级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataTrust {
    Trusted,
    Contracted,
    Untrusted,
}

/// LLM 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: u32,
    pub tools: Vec<Tool>,
    pub image_data: Option<String>,
    pub thinking_budget: Option<u32>,
    pub provider_params: HashMap<String, Value>,
    pub constraint_json: Option<Value>,
    pub structured_output: Option<StructuredOutputConfig>,
    pub cacheable_prefix_tokens: Option<usize>,
}

/// Provider-native structured output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuredOutputConfig {
    JsonObject,
    JsonSchema(Value),
}

/// LLM 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
    pub finish_reason: FinishReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallInfo>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

/// Token 用量
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 完成原因
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    Tool,
    ContentFilter,
    Unknown,
}

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCallInfo>>,
    pub tool_call_id: Option<String>,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// 工具调用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// LLM 错误
#[derive(Debug, Clone)]
pub enum LlmError {
    Network(String),
    Authentication(String),
    RateLimited(String),
    InvalidRequest(String),
    ServerError(String),
    Unknown(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Network(msg) => write!(f, "Network error: {}", msg),
            LlmError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            LlmError::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
            LlmError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            LlmError::ServerError(msg) => write!(f, "Server error: {}", msg),
            LlmError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

/// Gateway 门面 trait — L3 通过此 trait 抽象访问 L1 GatewayV2，
/// 避免 L3 直接依赖 L1 具体实现。
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait GatewayFacade: Send + Sync {
    /// 非流式补全
    async fn complete_with_selection(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;
    /// 流式补全
    async fn stream_complete_with_selection(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError>;
}
