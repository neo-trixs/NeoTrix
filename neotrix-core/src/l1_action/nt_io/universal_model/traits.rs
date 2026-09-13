//! Universal Model Interface — 核心 Trait 定义
//!
//! 所有外部模型（LLM / Embedding / Vision）必须实现此 trait，
//! 为上层提供模型无关的统一调用入口。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::l2_perception::nt_core_llm::{
    DataTrust, LlmError, LlmRequest, LlmResponse, Usage,
};

// ════════════════════════════════════════════════════════════════
// 核心 Trait
// ════════════════════════════════════════════════════════════════

/// 模型提供商唯一标识
pub trait ModelIdentifier: Send + Sync + 'static {
    /// 提供商名称 (如 "openai", "anthropic", "ollama")
    fn provider_name(&self) -> &str;

    /// 模型 ID (如 "gpt-4o", "claude-sonnet-4-20250514", "llama3")
    fn model_id(&self) -> &str;

    /// 完整标识 (provider/model)
    fn full_id(&self) -> String {
        format!("{}/{}", self.provider_name(), self.model_id())
    }
}

/// 统一模型接口 — 所有外部模型必须实现
///
/// 设计原则:
/// 1. 接口最小化: 只暴露 `complete` / `embed` / `health` / `capabilities`
/// 2. 所有 provider 适配器在内部处理 API 差异
/// 3. 通过 `ModelCapabilities` 声明能力，上层按需调用
/// 4. 数据信任分级遵循 Egress Privacy Guard 协议
#[async_trait]
pub trait UniversalModel: ModelIdentifier + Send + Sync {
    /// 文本补全 / 对话
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;

    /// 文本嵌入 (仅支持 Embedding 能力的模型)
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ModelError> {
        let _ = text;
        Err(ModelError::UnsupportedOperation {
            model: self.full_id(),
            operation: "embed".to_string(),
        })
    }

    /// 批量文本嵌入
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, ModelError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// 健康检查
    fn health(&self) -> ModelHealth;

    /// 能力声明
    fn capabilities(&self) -> ModelCapabilities;

    /// 数据信任分级 — 决定出网守卫处置
    fn data_trust(&self) -> DataTrust {
        DataTrust::Contracted
    }

    /// 设置代理
    fn set_proxy(&mut self, _proxy_url: &str) {}
}

// ════════════════════════════════════════════════════════════════
// 能力声明
// ════════════════════════════════════════════════════════════════

/// 模型能力声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// 支持的任务类型
    pub task_types: Vec<TaskType>,
    /// 是否支持流式输出
    pub supports_streaming: bool,
    /// 是否支持工具调用
    pub supports_tools: bool,
    /// 是否支持结构化输出
    pub supports_structured_output: bool,
    /// 是否支持图像输入
    pub supports_vision: bool,
    /// 是否支持扩展思考 (extended thinking)
    pub supports_thinking: bool,
    /// 是否支持嵌入
    pub supports_embedding: bool,
    /// 最大上下文窗口 (tokens)
    pub max_context_tokens: u32,
    /// 最大输出 tokens
    pub max_output_tokens: u32,
    /// 嵌入维度 (仅 embedding 模型)
    pub embedding_dim: Option<usize>,
    /// 模型语言
    pub languages: Vec<String>,
    /// 自定义能力标记
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            task_types: vec![TaskType::Chat],
            supports_streaming: true,
            supports_tools: false,
            supports_structured_output: false,
            supports_vision: false,
            supports_thinking: false,
            supports_embedding: false,
            max_context_tokens: 4096,
            max_output_tokens: 4096,
            embedding_dim: None,
            languages: vec!["en".to_string()],
            custom: HashMap::new(),
        }
    }
}

/// 任务类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    Chat,
    Completion,
    Embedding,
    Reranking,
    ImageGeneration,
    AudioGeneration,
    VideoGeneration,
}

// ════════════════════════════════════════════════════════════════
// 健康状态
// ════════════════════════════════════════════════════════════════

/// 模型健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    /// 是否可用
    pub available: bool,
    /// 最近延迟 (ms)
    pub latency_ms: Option<f64>,
    /// 错误率 (0.0-1.0)
    pub error_rate: f32,
    /// 最近成功请求时间戳
    pub last_success: Option<u64>,
    /// 熔断状态
    pub circuit_breaker_open: bool,
    /// 健康消息
    pub message: Option<String>,
}

impl Default for ModelHealth {
    fn default() -> Self {
        Self {
            available: true,
            latency_ms: None,
            error_rate: 0.0,
            last_success: None,
            circuit_breaker_open: false,
            message: None,
        }
    }
}

impl ModelHealth {
    /// Create an unhealthy status (available=false, no circuit breaker).
    ///
    /// Note: Real implementation should also set `circuit_breaker_open` and
    /// record the failure timestamp for cooldown tracking.
    pub fn unhealthy(msg: &str) -> Self {
        Self {
            available: false,
            message: Some(msg.to_string()),
            ..Default::default()
        }
    }

    /// Create a degraded status (available but with elevated error rate).
    ///
    /// Note: Real implementation should track error rate trend over a sliding
    /// window (not just instantaneous) to avoid flapping between healthy/degraded.
    pub fn degraded(msg: &str, error_rate: f32) -> Self {
        Self {
            available: true,
            error_rate,
            message: Some(msg.to_string()),
            ..Default::default()
        }
    }
}

// ════════════════════════════════════════════════════════════════
// 错误类型
// ════════════════════════════════════════════════════════════════

/// 模型错误 (扩展 LlmError，增加不支持操作等)
#[derive(Debug, Clone)]
pub enum ModelError {
    /// LLM 调用错误
    Llm(LlmError),
    /// 不支持的操作
    UnsupportedOperation {
        model: String,
        operation: String,
    },
    /// 模型不可用
    Unavailable {
        model: String,
        reason: String,
    },
    /// 配置错误
    Config(String),
    /// 能力不足
    InsufficientCapability {
        model: String,
        required: String,
    },
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Llm(e) => write!(f, "{}", e),
            Self::UnsupportedOperation { model, operation } => {
                write!(f, "Model '{}' does not support operation '{}'", model, operation)
            }
            Self::Unavailable { model, reason } => {
                write!(f, "Model '{}' is unavailable: {}", model, reason)
            }
            Self::Config(msg) => write!(f, "Config error: {}", msg),
            Self::InsufficientCapability { model, required } => {
                write!(f, "Model '{}' lacks capability '{}'", model, required)
            }
        }
    }
}

impl std::error::Error for ModelError {}

impl From<LlmError> for ModelError {
    fn from(e: LlmError) -> Self {
        Self::Llm(e)
    }
}

impl ModelError {
    /// 是否可重试
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Llm(e) => e.is_retryable(),
            Self::Unavailable { .. } => true,
            _ => false,
        }
    }

    /// 是否应故障转移
    pub fn should_fallback(&self) -> bool {
        match self {
            Self::Llm(e) => e.should_fallback(),
            Self::UnsupportedOperation { .. } => true,
            Self::InsufficientCapability { .. } => true,
            Self::Unavailable { .. } => true,
            Self::Config(_) => false,
        }
    }
}

// ════════════════════════════════════════════════════════════════
// 请求 / 响应 扩展类型
// ════════════════════════════════════════════════════════════════

/// 统一嵌入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// 输入文本
    pub input: Vec<String>,
    /// 模型 (可选，使用模型默认)
    pub model: Option<String>,
}

/// 统一嵌入响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// 嵌入向量列表
    pub embeddings: Vec<Vec<f32>>,
    /// 使用的模型
    pub model: String,
    /// Token 使用量
    pub usage: Usage,
}

/// 模型信息 (用于注册/发现)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// 提供商
    pub provider: String,
    /// 模型 ID
    pub model_id: String,
    /// 显示名称
    pub display_name: String,
    /// 能力
    pub capabilities: ModelCapabilities,
    /// 数据信任分级
    pub data_trust: DataTrust,
    /// 价格 (每 1M tokens，USD)
    pub price_per_1m_tokens: Option<f64>,
    /// 是否默认模型
    pub is_default: bool,
}
