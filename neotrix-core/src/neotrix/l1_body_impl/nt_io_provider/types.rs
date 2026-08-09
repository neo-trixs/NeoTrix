//! LLM Provider 核心类型定义
//!
//! 更新说明 (2026-07-01):
//! - `temperature` 改为 `Option<f32>` — 部分新模型 (如 Claude Sonnet 5) 不再支持采样参数
//! - 新增 `thinking_budget` — 支持模型自适应思考 (extended thinking)
//! - 新增 `provider_params` — 额外 provider 专用参数映射
//!
//! 2026-07-04: 新增 ProviderCategory (自我/客体分离)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;
    async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError>;

    /// 将 provider 的 HTTP 客户端切换为代理路由 (子母阵 Proxied/Tor 画像注入)。
    /// 默认 no-op — 不支持代理注入的 provider 保持原客户端不变。
    fn set_proxy(&mut self, _proxy_url: &str) {}
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    /// Optional: None = use model default (Sonnet 5 removes temperature/top_p/top_k)
    pub temperature: Option<f32>,
    pub max_tokens: u32,
    pub tools: Vec<Tool>,
    pub image_data: Option<String>,
    /// Extended thinking budget (tokens). Some(0) = disable, Some(n) = budget, None = model default
    pub thinking_budget: Option<u32>,
    /// Provider-specific extra params (e.g. {"top_p": 0.9, "top_k": 40})
    pub provider_params: HashMap<String, serde_json::Value>,
    /// Optional constrained decoding: JSON-serialized Constraint from nt_io_constrained.
    /// Applied by ConstrainedGateway wrapper at the provider layer.
    pub constraint_json: Option<serde_json::Value>,
    /// Native structured output configuration for provider-native APIs.
    /// When set, providers use their native JSON mode (OpenAI response_format,
    /// Anthropic output_config, Gemini response_mime_type) instead of constrained decoding.
    pub structured_output: Option<StructuredOutputConfig>,
}

/// Provider-native structured output configuration.
///
/// Each major provider has its own native API for enforcing structured output:
/// - OpenAI: `response_format: { type: "json_schema", json_schema: { ... } }`
/// - Anthropic: `output_config: { format: { type: "json_object" } }`
/// - Gemini: `response_mime_type: "application/json"` + `response_schema`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuredOutputConfig {
    /// Simple JSON object mode (no schema enforcement, just pure JSON)
    JsonObject,
    /// JSON Schema enforcement (provider-native where supported)
    JsonSchema(Value),
}

impl StructuredOutputConfig {
    pub fn json_schema(schema: Value) -> Self {
        StructuredOutputConfig::JsonSchema(schema)
    }

    pub fn is_json_object(&self) -> bool {
        matches!(self, StructuredOutputConfig::JsonObject)
    }
}

impl LlmRequest {
    pub fn new(model: &str, prompt: &str) -> Self {
        Self {
            model: model.to_string(),
            messages: vec![Message::new(Role::User, prompt)],
            temperature: Some(0.7),
            max_tokens: 4096,
            tools: vec![],
            image_data: None,
            thinking_budget: None,
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
        }
    }

    pub fn with_structured_output(mut self, config: StructuredOutputConfig) -> Self {
        self.structured_output = Some(config);
        self
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_thinking(mut self, budget: u32) -> Self {
        self.thinking_budget = Some(budget);
        self
    }

    pub fn with_temperature(mut self, temperature: Option<f32>) -> Self {
        self.temperature = temperature;
        self
    }

    /// 干净的 temperature 值（f64，四舍五入到 2 位小数）。
    ///
    /// f32 序列化会产生精度噪声（如 0.7 → 0.699999988079071），
    /// 部分 provider（Pollinations 实测 2026-08-09）会拒绝非标准采样值。
    /// 所有 provider 序列化 temperature 时必须用此方法而非 `json!(f32)`。
    pub fn temperature_clean(&self) -> Option<f64> {
        self.temperature.map(|t| {
            // 必须在 f64 域运算：f32 域 round 后仍是 f32 近似值
            let t64 = t as f64;
            (t64 * 100.0).round() / 100.0
        })
    }

    /// Attach a base64 image payload as `image_data` (data URI) for providers
    /// that support vision. `image_b64` is the raw base64 of the image bytes;
    /// the data-URI prefix is inferred as png/jpeg/jpeg by presence of the
    /// PNG magic bytes.
    pub fn with_image_b64(mut self, image_b64: &str) -> Self {
        let prefix = if image_b64.starts_with("data:image/") {
            String::new()
        } else if image_b64.starts_with("/9j/") {
            "data:image/jpeg;base64,".to_string()
        } else {
            "data:image/png;base64,".to_string()
        };
        self.image_data = Some(format!("{}{}", prefix, image_b64));
        self
    }

    pub fn with_provider_param(mut self, key: &str, value: serde_json::Value) -> Self {
        self.provider_params.insert(key.to_string(), value);
        self
    }

    /// Set a constrained decoding constraint (serialized JSON).
    /// The ConstrainedGateway will parse and apply this at inference time.
    pub fn with_constraint(mut self, constraint: serde_json::Value) -> Self {
        self.constraint_json = Some(constraint);
        self
    }

    /// Whether this model uses adaptive thinking (no temperature/top_p/top_k)
    pub fn has_no_sampling_params(&self) -> bool {
        self.temperature.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
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
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn new(role: Role, content: &str) -> Self {
        Self {
            role,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool(content: &str, call_id: &str) -> Self {
        Self {
            role: Role::Tool,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: Some(call_id.to_string()),
        }
    }

    pub fn assistant_with_calls(content: &str, calls: Vec<ToolCallInfo>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.to_string(),
            tool_calls: Some(calls),
            tool_call_id: None,
        }
    }
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
    /// Tool calls requested by the model (when finish_reason == Tool).
    /// Populated by providers that parse `tool_calls` from the raw response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

impl LlmResponse {
    /// Convenience constructor for providers that do not surface tool calls.
    pub fn plain(content: String, model: String, usage: Usage, finish_reason: FinishReason) -> Self {
        Self { content, model, usage, finish_reason, tool_calls: None }
    }
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

#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_image_b64_plain_png_gets_data_uri() {
        let req = LlmRequest::new("gpt-4o", "describe").with_image_b64("iVBORw0KGgo=");
        let uri = req.image_data.expect("image_data set");
        assert!(uri.starts_with("data:image/png;base64,iVBORw0KGgo="), "got {uri}");
    }

    #[test]
    fn test_with_image_b64_jpeg_magic_gets_jpeg_uri() {
        let req = LlmRequest::new("gpt-4o", "describe").with_image_b64("/9j/4AAQSkZJRg==");
        let uri = req.image_data.expect("image_data set");
        assert!(uri.starts_with("data:image/jpeg;base64,/9j/"), "got {uri}");
    }

    #[test]
    fn test_with_image_b64_already_data_uri_unchanged() {
        let req = LlmRequest::new("gpt-4o", "describe").with_image_b64("data:image/webp;base64,UklGRg==");
        let uri = req.image_data.expect("image_data set");
        assert_eq!(uri, "data:image/webp;base64,UklGRg==");
    }

    #[test]
    fn test_temperature_clean_removes_f32_noise() {
        // f32 0.7 序列化为 0.699999988079071，clean 后必须是干净的 0.7
        let req = LlmRequest::new("openai", "hi");
        let clean = req.temperature_clean().expect("temperature set");
        assert_eq!(clean, 0.7, "f32 noise must be removed, got {clean}");
        // 序列化验证：json!(f64) 输出干净值
        let json = serde_json::json!(clean);
        assert_eq!(json.to_string(), "0.7", "json serialization must be clean, got {json}");
    }

    #[test]
    fn test_temperature_clean_none_returns_none() {
        let req = LlmRequest::new("gpt-4o", "describe").with_temperature(None);
        assert_eq!(req.temperature_clean(), None);
    }

    #[test]
    fn test_temperature_clean_preserves_common_values() {
        // 常见采样值必须原样保留（f64 域四舍五入到 2 位小数）
        for (input, expected) in [
            (0.0f32, 0.0f64),
            (1.0f32, 1.0f64),
            (0.5f32, 0.5f64),
            (1.5f32, 1.5f64),
            (0.25f32, 0.25f64),
        ] {
            let req = LlmRequest::new("gpt-4o", "describe").with_temperature(Some(input));
            let clean = req.temperature_clean().expect("temperature set");
            assert_eq!(clean, expected, "input {input} -> {clean}, expected {expected}");
        }
    }

    #[test]
    fn test_temperature_clean_rounds_to_two_decimals() {
        // 超过 2 位小数的值应四舍五入到 2 位
        let req = LlmRequest::new("gpt-4o", "describe").with_temperature(Some(0.3333f32));
        let clean = req.temperature_clean().expect("temperature set");
        assert_eq!(clean, 0.33, "got {clean}");
    }
}
