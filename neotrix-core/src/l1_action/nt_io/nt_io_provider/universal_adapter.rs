//! 通用模型适配器 — 适用于所有外部模型
//!
//! 统一接口，屏蔽不同模型提供商（Claude/GPT/Gemini/Llama/Qwen/DeepSeek）的差异。
//! 提供模型注册、能力探测、最优选择、请求格式转换等核心能力。
//!
//! 设计原则:
//! - 零外部依赖
//! - 所有类型标注 `#[allow(dead_code)]` 以保留未来扩展空间
//! - 中文文档注释

use std::collections::HashMap;

/// 模型能力描述
///
/// 每个外部模型在注册时声明其能力集，适配器据此进行路由决策。
#[allow(dead_code)]
pub struct ModelCapabilities {
    /// 是否支持函数调用 / 工具调用
    pub supports_tools: bool,
    /// 是否支持流式输出 (SSE)
    pub supports_streaming: bool,
    /// 是否支持结构化输出 (JSON Schema / function calling)
    pub supports_structured_output: bool,
    /// 最大上下文窗口 (tokens)
    pub max_context_tokens: u32,
    /// 是否支持图像输入 (多模态)
    pub supports_images: bool,
    /// 是否支持音频输入 (多模态)
    pub supports_audio: bool,
    /// 每千 token 成本 (美元，用于路由权重)
    pub cost_per_1k_tokens: f64,
}

/// 模型配置
///
/// 包含模型标识、提供商信息、端点地址及 API 密钥环境变量名。
#[allow(dead_code)]
pub struct ModelConfig {
    /// 模型 ID，如 "claude-sonnet-4-20250514" / "gpt-4o" / "qwen-max"
    pub model_id: String,
    /// 提供商名称，如 "anthropic" / "openai" / "alibaba"
    pub provider: String,
    /// 模型能力集
    pub capabilities: ModelCapabilities,
    /// API 端点 URL
    pub endpoint: String,
    /// 存放 API Key 的环境变量名
    pub api_key_env: String,
}

/// 统一请求格式
///
/// 所有外部调用统一转换为此结构，再由适配器序列化为各提供商特定格式。
#[allow(dead_code)]
pub struct UnifiedRequest {
    /// 对话消息列表
    pub messages: Vec<Message>,
    /// 可用工具定义列表
    pub tools: Vec<Tool>,
    /// 采样温度
    pub temperature: f32,
    /// 最大生成 token 数
    pub max_tokens: u32,
    /// 是否启用流式输出
    pub stream: bool,
}

/// 对话消息
#[allow(dead_code)]
pub struct Message {
    /// 角色: "system" / "user" / "assistant" / "tool"
    pub role: String,
    /// 消息内容
    pub content: String,
}

/// 工具定义
#[allow(dead_code)]
pub struct Tool {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 参数 JSON Schema 字符串
    pub parameters: String,
}

/// 统一响应格式
///
/// 从各提供商响应解析后的统一结构。
#[allow(dead_code)]
pub struct UnifiedResponse {
    /// 模型生成的文本内容
    pub content: String,
    /// 工具调用列表 (可能为空)
    pub tool_calls: Vec<ToolCall>,
    /// 消耗的 token 总数
    pub tokens_used: u32,
    /// 实际使用的模型 ID
    pub model: String,
}

/// 工具调用
#[allow(dead_code)]
pub struct ToolCall {
    /// 调用 ID (用于后续结果回传)
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 参数 JSON 字符串
    pub arguments: String,
}

/// 格式转换器 trait
///
/// 各提供商实现此 trait，将 `UnifiedRequest` 转换为其原生 API 格式。
#[allow(dead_code)]
pub trait FormatConverter {
    /// 将统一请求序列化为提供商特定的 JSON payload
    fn to_provider_format(&self, request: &UnifiedRequest, model: &ModelConfig) -> String;

    /// 将提供商原始 JSON 响应反序列化为统一响应
    fn from_provider_response(&self, raw: &str, model: &ModelConfig) -> Option<UnifiedResponse>;
}

/// OpenAI 格式转换器
///
/// 适用于 OpenAI / Azure OpenAI / OpenRouter / 兼容 OpenAI 接口的提供商。
#[allow(dead_code)]
pub struct OpenAiConverter;

impl FormatConverter for OpenAiConverter {
    fn to_provider_format(&self, request: &UnifiedRequest, model: &ModelConfig) -> String {
        let messages: Vec<String> = request
            .messages
            .iter()
            .map(|m| {
                format!(
                    "{{\"role\":\"{}\",\"content\":\"{}\"}}",
                    escape_json(&m.role),
                    escape_json(&m.content)
                )
            })
            .collect();

        let tools: Vec<String> = request
            .tools
            .iter()
            .map(|t| {
                format!(
                    "{{\"type\":\"function\",\"function\":{{\"name\":\"{}\",\"description\":\"{}\",\"parameters\":{}}}}}",
                    escape_json(&t.name),
                    escape_json(&t.description),
                    t.parameters
                )
            })
            .collect();

        format!(
            "{{\"model\":\"{}\",\"messages\":[{}],\"tools\":[{}],\"temperature\":{},\"max_tokens\":{},\"stream\":{}}}",
            escape_json(&model.model_id),
            messages.join(","),
            tools.join(","),
            request.temperature,
            request.max_tokens,
            request.stream,
        )
    }

    fn from_provider_response(&self, raw: &str, model: &ModelConfig) -> Option<UnifiedResponse> {
        let content = extract_json_string(raw, "content").unwrap_or_default();
        let tokens_used = extract_json_number(raw, "total_tokens").unwrap_or(0.0) as u32;

        Some(UnifiedResponse {
            content,
            tool_calls: Vec::new(),
            tokens_used,
            model: model.model_id.clone(),
        })
    }
}

/// Anthropic 格式转换器
///
/// 适用于 Claude 系列模型，使用 Anthropic Messages API 格式。
#[allow(dead_code)]
pub struct AnthropicConverter;

impl FormatConverter for AnthropicConverter {
    fn to_provider_format(&self, request: &UnifiedRequest, model: &ModelConfig) -> String {
        let messages: Vec<String> = request
            .messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| {
                format!(
                    "{{\"role\":\"{}\",\"content\":\"{}\"}}",
                    escape_json(&m.role),
                    escape_json(&m.content)
                )
            })
            .collect();

        let system_msg = request
            .messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| format!(",\"system\":\"{}\"", escape_json(&m.content)))
            .unwrap_or_default();

        format!(
            "{{\"model\":\"{}\",\"messages\":[{}]{},\"max_tokens\":{}}}",
            escape_json(&model.model_id),
            messages.join(","),
            system_msg,
            request.max_tokens,
        )
    }

    fn from_provider_response(&self, raw: &str, model: &ModelConfig) -> Option<UnifiedResponse> {
        let content = extract_json_string(raw, "text").unwrap_or_default();
        let tokens_used = extract_json_number(raw, "input_tokens")
            .zip(extract_json_number(raw, "output_tokens"))
            .map(|(i, o)| (i + o) as u32)
            .unwrap_or(0);

        Some(UnifiedResponse {
            content,
            tool_calls: Vec::new(),
            tokens_used,
            model: model.model_id.clone(),
        })
    }
}

/// Gemini 格式转换器
///
/// 适用于 Google Gemini / Vertex AI，使用 Google Generative Language API 格式。
#[allow(dead_code)]
pub struct GeminiConverter;

impl FormatConverter for GeminiConverter {
    fn to_provider_format(&self, request: &UnifiedRequest, _model: &ModelConfig) -> String {
        let contents: Vec<String> = request
            .messages
            .iter()
            .map(|m| {
                format!(
                    "{{\"role\":\"{}\",\"parts\":[{{\"text\":\"{}\"}}]}}",
                    if m.role == "assistant" {
                        "model"
                    } else {
                        "user"
                    },
                    escape_json(&m.content)
                )
            })
            .collect();

        format!(
            "{{\"contents\":[{}],\"generationConfig\":{{\"temperature\":{},\"maxOutputTokens\":{}}}}}",
            contents.join(","),
            request.temperature,
            request.max_tokens,
        )
    }

    fn from_provider_response(&self, raw: &str, model: &ModelConfig) -> Option<UnifiedResponse> {
        let content = extract_json_string(raw, "text").unwrap_or_default();
        let tokens_used = extract_json_number(raw, "totalTokenCount").unwrap_or(0.0) as u32;

        Some(UnifiedResponse {
            content,
            tool_calls: Vec::new(),
            tokens_used,
            model: model.model_id.clone(),
        })
    }
}

/// 通用模型适配器
///
/// 管理所有已注册的外部模型，提供统一的选择、转换、调用接口。
#[allow(dead_code)]
pub struct UniversalAdapter {
    /// 已注册的模型配置列表
    models: Vec<ModelConfig>,
    /// 当前激活的模型 ID
    active_model: Option<String>,
    /// 格式转换器注册表 (provider -> converter)
    converters: HashMap<String, Box<dyn FormatConverter>>,
}

impl UniversalAdapter {
    /// 创建空适配器
    pub fn new() -> Self {
        let mut converters: HashMap<String, Box<dyn FormatConverter>> = HashMap::new();
        converters.insert("openai".to_string(), Box::new(OpenAiConverter));
        converters.insert("anthropic".to_string(), Box::new(AnthropicConverter));
        converters.insert("gemini".to_string(), Box::new(GeminiConverter));

        Self {
            models: Vec::new(),
            active_model: None,
            converters,
        }
    }

    /// 注册一个新模型
    pub fn register_model(&mut self, config: ModelConfig) {
        self.models.push(config);
    }

    /// 设置当前激活模型
    pub fn set_active(&mut self, model_id: &str) -> bool {
        if self.models.iter().any(|m| m.model_id == model_id) {
            self.active_model = Some(model_id.to_string());
            true
        } else {
            false
        }
    }

    /// 获取当前激活模型配置
    pub fn active_model(&self) -> Option<&ModelConfig> {
        self.active_model
            .as_ref()
            .and_then(|id| self.models.iter().find(|m| m.model_id == *id))
    }

    /// 按模型 ID 查找配置
    pub fn find_model(&self, model_id: &str) -> Option<&ModelConfig> {
        self.models.iter().find(|m| m.model_id == model_id)
    }

    /// 基于任务需求选择最佳模型
    ///
    /// 按成本升序排列，返回第一个满足所有硬性约束的模型。
    /// 优先选择成本最低的可用模型 (Cost-Aware Routing)。
    pub fn select_best(
        &self,
        needs_tools: bool,
        needs_streaming: bool,
        needs_images: bool,
        min_context_tokens: u32,
    ) -> Option<&ModelConfig> {
        let mut candidates: Vec<&ModelConfig> = self
            .models
            .iter()
            .filter(|m| {
                (!needs_tools || m.capabilities.supports_tools)
                    && (!needs_streaming || m.capabilities.supports_streaming)
                    && (!needs_images || m.capabilities.supports_images)
                    && m.capabilities.max_context_tokens >= min_context_tokens
            })
            .collect();

        candidates.sort_by(|a, b| {
            a.capabilities
                .cost_per_1k_tokens
                .partial_cmp(&b.capabilities.cost_per_1k_tokens)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates.into_iter().next()
    }

    /// 按提供商过滤模型
    pub fn models_by_provider(&self, provider: &str) -> Vec<&ModelConfig> {
        self.models
            .iter()
            .filter(|m| m.provider == provider)
            .collect()
    }

    /// 获取指定提供商的格式转换器
    pub fn converter(&self, provider: &str) -> Option<&dyn FormatConverter> {
        self.converters.get(provider).map(|c| c.as_ref())
    }

    /// 将统一请求转换为指定提供商的格式
    pub fn to_provider_format(
        &self,
        request: &UnifiedRequest,
        model: &ModelConfig,
    ) -> Option<String> {
        self.converter(&model.provider)
            .map(|c| c.to_provider_format(request, model))
    }

    /// 将提供商原始响应转换为统一响应
    pub fn from_provider_response(
        &self,
        raw: &str,
        model: &ModelConfig,
    ) -> Option<UnifiedResponse> {
        self.converter(&model.provider)
            .and_then(|c| c.from_provider_response(raw, model))
    }

    /// 获取所有已注册模型
    pub fn models(&self) -> &[ModelConfig] {
        &self.models
    }

    /// 获取模型统计: (总数, 支持工具数, 支持流式数)
    pub fn stats(&self) -> (usize, usize, usize) {
        let with_tools = self
            .models
            .iter()
            .filter(|m| m.capabilities.supports_tools)
            .count();
        let with_streaming = self
            .models
            .iter()
            .filter(|m| m.capabilities.supports_streaming)
            .count();
        (self.models.len(), with_tools, with_streaming)
    }
}

// ── 内部辅助函数 ──

/// JSON 字符串转义
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// 从简单 JSON 字符串中提取字符串字段值
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();

    if rest.starts_with('"') {
        let end = rest[1..].find('"')? + 1;
        Some(rest[1..end].to_string())
    } else {
        None
    }
}

/// 从简单 JSON 字符串中提取数值字段值
fn extract_json_number(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();

    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    rest[..end].parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_model(id: &str, provider: &str) -> ModelConfig {
        ModelConfig {
            model_id: id.to_string(),
            provider: provider.to_string(),
            capabilities: ModelCapabilities {
                supports_tools: true,
                supports_streaming: true,
                supports_structured_output: true,
                max_context_tokens: 128_000,
                supports_images: true,
                supports_audio: false,
                cost_per_1k_tokens: 0.01,
            },
            endpoint: format!("https://api.{}.com/v1", provider),
            api_key_env: format!("{}_API_KEY", provider.to_uppercase()),
        }
    }

    #[test]
    fn test_register_and_find() {
        let mut adapter = UniversalAdapter::new();
        adapter.register_model(mock_model("gpt-4o", "openai"));
        adapter.register_model(mock_model("claude-sonnet-4-20250514", "anthropic"));

        assert!(adapter.find_model("gpt-4o").is_some());
        assert!(adapter.find_model("nonexistent").is_none());
        assert_eq!(adapter.models().len(), 2);
    }

    #[test]
    fn test_select_best() {
        let mut adapter = UniversalAdapter::new();
        adapter.register_model(mock_model("gpt-4o", "openai"));

        let selected = adapter.select_best(true, true, false, 1000);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().model_id, "gpt-4o");
    }

    #[test]
    fn test_select_best_no_match() {
        let mut adapter = UniversalAdapter::new();
        let mut m = mock_model("gpt-4o", "openai");
        m.capabilities.max_context_tokens = 1000;
        adapter.register_model(m);

        let selected = adapter.select_best(false, false, false, 100_000);
        assert!(selected.is_none());
    }

    #[test]
    fn test_set_active() {
        let mut adapter = UniversalAdapter::new();
        adapter.register_model(mock_model("gpt-4o", "openai"));

        assert!(adapter.set_active("gpt-4o"));
        assert!(!adapter.set_active("nonexistent"));
        assert!(adapter.active_model().is_some());
    }

    #[test]
    fn test_stats() {
        let mut adapter = UniversalAdapter::new();
        adapter.register_model(mock_model("gpt-4o", "openai"));
        adapter.register_model(mock_model("claude-sonnet-4-20250514", "anthropic"));

        let (total, tools, streaming) = adapter.stats();
        assert_eq!(total, 2);
        assert_eq!(tools, 2);
        assert_eq!(streaming, 2);
    }

    #[test]
    fn test_openai_format_conversion() {
        let adapter = UniversalAdapter::new();
        let model = mock_model("gpt-4o", "openai");

        let request = UnifiedRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello, world!".to_string(),
            }],
            tools: Vec::new(),
            temperature: 0.7,
            max_tokens: 1024,
            stream: false,
        };

        let json = adapter.to_provider_format(&request, &model).unwrap();
        assert!(json.contains("gpt-4o"));
        assert!(json.contains("Hello, world!"));
        assert!(json.contains("0.7"));
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("he\"llo"), "he\\\"llo");
        assert_eq!(escape_json("he\nllo"), "he\\nllo");
    }

    #[test]
    fn test_extract_json_string() {
        let json = r#"{"name":"test","value":42}"#;
        assert_eq!(extract_json_string(json, "name"), Some("test".to_string()));
        assert_eq!(extract_json_string(json, "missing"), None);
    }

    #[test]
    fn test_extract_json_number() {
        let json = r#"{"count":42,"ratio":3.14}"#;
        assert_eq!(extract_json_number(json, "count"), Some(42.0));
        assert_eq!(extract_json_number(json, "ratio"), Some(3.14));
        assert_eq!(extract_json_number(json, "missing"), None);
    }
}
