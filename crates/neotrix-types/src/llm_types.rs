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

impl LlmRequest {
    /// Backward-compatible constructor: accepts model name and prompt string
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: vec![Message::user(prompt)],
            temperature: None,
            max_tokens: 4096,
            tools: Vec::new(),
            image_data: None,
            thinking_budget: None,
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
            cacheable_prefix_tokens: None,
        }
    }

    /// Backward-compatible: get cleaned temperature (None if not set)
    pub fn temperature_clean(&self) -> Option<f32> {
        self.temperature
    }

    /// Backward-compatible: set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Backward-compatible: set temperature
    pub fn with_temperature(mut self, temperature: Option<f32>) -> Self {
        self.temperature = temperature;
        self
    }

    /// Backward-compatible: set image data
    pub fn with_image_b64(mut self, b64: String) -> Self {
        self.image_data = Some(b64);
        self
    }

    /// Builder: set structured output
    pub fn with_structured_output(mut self, config: StructuredOutputConfig) -> Self {
        self.structured_output = Some(config);
        self
    }
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

impl Message {
    /// Backward-compatible constructor
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// System message shorthand
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }

    /// User message shorthand
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }

    /// Tool message shorthand
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }

    /// Assistant message with tool calls
    pub fn assistant_with_calls(content: impl Into<String>, calls: Vec<ToolCallInfo>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            tool_calls: Some(calls),
            tool_call_id: None,
        }
    }
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
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub arguments: String,
    /// Backward-compatible: nested function info
    #[serde(default)]
    pub function: Option<ToolCallFunction>,
    /// Backward-compatible: call type identifier
    #[serde(default)]
    pub call_type: Option<String>,
}

/// Backward-compatible: ToolCallFunction (nested in ToolCallInfo)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

/// LLM 错误
#[derive(Debug, Clone)]
pub enum LlmError {
    Network(String),
    Authentication(String),
    RateLimit(String),
    InvalidRequest(String),
    Server(String),
    Unknown(String),
    UnsupportedOperation(String),
    ProviderNotFound(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Network(msg) => write!(f, "Network error: {}", msg),
            LlmError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            LlmError::RateLimit(msg) => write!(f, "Rate limited: {}", msg),
            LlmError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            LlmError::Server(msg) => write!(f, "Server error: {}", msg),
            LlmError::Unknown(msg) => write!(f, "Unknown: {}", msg),
            LlmError::UnsupportedOperation(msg) => write!(f, "Unsupported: {}", msg),
            LlmError::ProviderNotFound(msg) => write!(f, "Provider not found: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

impl LlmError {
    /// Whether this error is retryable (transient failures)
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::RateLimit(_) | Self::Server(_)
        )
    }

    /// Whether this error should trigger provider fallback
    pub fn should_fallback(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::RateLimit(_) | Self::Server(_) | Self::Unknown(_)
        )
    }

    /// Whether this error indicates quota exhaustion
    pub fn is_quota_exhaustion(&self) -> bool {
        matches!(self, Self::RateLimit(_))
    }
}

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
