//! Unified Inference Layer — 所有 LLM 调用的统一入口
//!
//! 设计原则:
//! - Facade 模式: InferenceRouter 包装 GatewayV2, 不替换
//! - 能力优先路由: capability → cost/latency 打破平局
//! - 错误分类: is_retryable() / should_fallback() 方法
//! - 成本感知: per-provider 定价 + 预算检查

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::*;

// ══════════════════════════════════════════════════════════════
// Request Types
// ══════════════════════════════════════════════════════════════

/// 统一推理请求 — 所有 LLM 调用使用此类型
#[derive(Debug, Clone, Default)]
pub struct InferenceRequest {
    /// 模型名 (None = 自动路由)
    pub model: Option<String>,
    /// 对话消息
    pub messages: Vec<Message>,
    /// 采样温度
    pub temperature: Option<f32>,
    /// 最大输出 token
    pub max_tokens: Option<u32>,
    /// 工具定义
    pub tools: Vec<Tool>,
    /// 图像数据 (base64)
    pub image_data: Option<String>,
    /// 思考预算
    pub thinking_budget: Option<u32>,
    /// 结构化输出配置
    pub structured_output: Option<StructuredOutputConfig>,
    /// 请求元数据 (路由决策用)
    pub metadata: RequestMetadata,
}

/// 请求元数据 — 路由和成本决策依据
#[derive(Debug, Clone, Default)]
pub struct RequestMetadata {
    /// 任务类型 ("code", "analysis", "creative", "chat")
    pub task_type: Option<String>,
    /// 优先级
    pub priority: Priority,
    /// 单次请求成本预算 (USD)
    pub cost_budget: Option<f64>,
    /// 延迟预算 (ms)
    pub latency_budget: Option<u64>,
    /// 自定义标签
    pub tags: HashMap<String, String>,
}

/// 请求优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

// ══════════════════════════════════════════════════════════════
// Response Types
// ══════════════════════════════════════════════════════════════

/// 统一推理响应
#[derive(Debug, Clone)]
pub struct InferenceResponse {
    /// 响应内容
    pub content: String,
    /// 使用的模型
    pub model: String,
    /// 使用的 provider
    pub provider: String,
    /// Token 使用量
    pub usage: Usage,
    /// 结束原因
    pub finish_reason: FinishReason,
    /// 工具调用
    pub tool_calls: Option<Vec<ToolCallInfo>>,
    /// 推理过程 (extended thinking)
    pub reasoning: Option<String>,
    /// 响应元数据
    pub metadata: ResponseMetadata,
}

/// 响应元数据 — 路由决策追踪
#[derive(Debug, Clone, Default)]
pub struct ResponseMetadata {
    /// 选定的 provider
    pub provider_selected: String,
    /// 故障转移次数
    pub fallback_count: u32,
    /// 总延迟 (ms)
    pub latency_ms: u64,
    /// 成本估算
    pub cost_estimate: CostEstimate,
    /// 是否命中缓存
    pub from_cache: bool,
}

// ══════════════════════════════════════════════════════════════
// Capability Types
// ══════════════════════════════════════════════════════════════

/// 路由器能力概览
#[derive(Debug, Clone, Default)]
pub struct InferenceCapabilities {
    /// 所有可用 provider
    pub providers: Vec<ProviderInfo>,
    /// 免费 provider 数
    pub total_free: usize,
    /// 付费 provider 数
    pub total_paid: usize,
    /// 本地 provider 数
    pub total_local: usize,
    /// 是否支持视觉
    pub supports_vision: bool,
    /// 是否支持工具调用
    pub supports_tools: bool,
    /// 是否支持流式
    pub supports_streaming: bool,
}

/// Provider 信息
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Provider 名称
    pub name: String,
    /// Provider 类别
    pub category: ProviderCategory,
    /// 是否免费
    pub is_free: bool,
    /// 健康状态
    pub health: HealthStatus,
    /// 能力描述
    pub capabilities: ProviderCapabilities,
}

/// Provider 能力
#[derive(Debug, Clone, Default)]
pub struct ProviderCapabilities {
    /// 支持文本生成
    pub text: bool,
    /// 支持视觉
    pub vision: bool,
    /// 支持工具调用
    pub function_calling: bool,
    /// 支持流式
    pub streaming: bool,
    /// 最大上下文窗口
    pub context_window: usize,
    /// 每 1K token 成本 (USD)
    pub cost_per_1k: f64,
}

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    CircuitOpen,
    Unknown,
}

/// 成本估算
#[derive(Debug, Clone, Default)]
pub struct CostEstimate {
    /// 提示 token 数
    pub prompt_tokens: usize,
    /// 输出 token 数
    pub completion_tokens: usize,
    /// 估算成本 (USD)
    pub estimated_cost_usd: f64,
    /// Provider 名称
    pub provider_name: String,
}

/// 路由器健康状态
#[derive(Debug, Clone)]
pub struct RouterHealth {
    /// 总 provider 数
    pub total_providers: usize,
    /// 健康数
    pub healthy: usize,
    /// 降级数
    pub degraded: usize,
    /// 熔断数
    pub circuit_open: usize,
    /// 池是否充足
    pub pool_sufficient: bool,
}

// ══════════════════════════════════════════════════════════════
// Error Types
// ══════════════════════════════════════════════════════════════

/// 统一推理错误
#[derive(Debug, Clone)]
pub enum InferenceError {
    /// Provider 错误
    ProviderError { provider: String, message: String },
    /// 速率限制
    RateLimitError { provider: String, retry_after: Option<u64> },
    /// 认证错误
    AuthenticationError { provider: String, message: String },
    /// 验证错误
    ValidationError(String),
    /// 网络错误
    NetworkError { provider: String, message: String },
    /// 超时
    TimeoutError { provider: String, elapsed_ms: u64 },
    /// 预算超限
    BudgetExceeded { estimated: f64, budget: f64 },
    /// 所有 provider 失败
    AllProvidersFailed { errors: Vec<InferenceError> },
    /// 无可用 provider
    NoProvidersAvailable,
}

impl InferenceError {
    /// 是否可重试 (速率限制/网络/超时)
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            InferenceError::RateLimitError { .. }
                | InferenceError::NetworkError { .. }
                | InferenceError::TimeoutError { .. }
        )
    }

    /// 是否应故障转移 (非认证/验证错误)
    pub fn should_fallback(&self) -> bool {
        !matches!(
            self,
            InferenceError::AuthenticationError { .. }
                | InferenceError::ValidationError(_)
                | InferenceError::BudgetExceeded { .. }
        )
    }

    /// 获取关联的 provider 名称
    pub fn provider_name(&self) -> Option<&str> {
        match self {
            InferenceError::ProviderError { provider, .. }
            | InferenceError::RateLimitError { provider, .. }
            | InferenceError::AuthenticationError { provider, .. }
            | InferenceError::NetworkError { provider, .. }
            | InferenceError::TimeoutError { provider, .. } => Some(provider),
            _ => None,
        }
    }

    /// 是否为配额耗尽 (应熔断而非重试)
    pub fn is_quota_exhaustion(&self) -> bool {
        match self {
            InferenceError::ProviderError { message, .. } => {
                let m = message.to_lowercase();
                m.contains("quota exceeded")
                    || m.contains("out of quota")
                    || m.contains("insufficient quota")
                    || m.contains("credit limit")
                    || m.contains("out of credits")
                    || m.contains("billing")
            }
            InferenceError::AuthenticationError { message, .. } => {
                let m = message.to_lowercase();
                m.contains("quota") || m.contains("billing")
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for InferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferenceError::ProviderError { provider, message } => {
                write!(f, "[{provider}] Provider error: {message}")
            }
            InferenceError::RateLimitError { provider, retry_after } => {
                write!(f, "[{provider}] Rate limited, retry after {retry_after:?}s")
            }
            InferenceError::AuthenticationError { provider, message } => {
                write!(f, "[{provider}] Auth error: {message}")
            }
            InferenceError::ValidationError(msg) => write!(f, "Validation: {msg}"),
            InferenceError::NetworkError { provider, message } => {
                write!(f, "[{provider}] Network: {message}")
            }
            InferenceError::TimeoutError { provider, elapsed_ms } => {
                write!(f, "[{provider}] Timeout after {elapsed_ms}ms")
            }
            InferenceError::BudgetExceeded { estimated, budget } => {
                write!(f, "Budget exceeded: ${estimated:.4} > ${budget:.4}")
            }
            InferenceError::AllProvidersFailed { errors } => {
                write!(f, "All {} providers failed: {}", errors.len(), errors.first()
                    .map(|e| e.to_string())
                    .unwrap_or_default())
            }
            InferenceError::NoProvidersAvailable => write!(f, "No providers available"),
        }
    }
}

impl std::error::Error for InferenceError {}

impl From<LlmError> for InferenceError {
    fn from(e: LlmError) -> Self {
        match e {
            LlmError::Network(s) => InferenceError::NetworkError { provider: String::new(), message: s },
            LlmError::Authentication(s) => InferenceError::AuthenticationError { provider: String::new(), message: s },
            LlmError::RateLimit(s) => InferenceError::RateLimitError { provider: String::new(), retry_after: None },
            LlmError::InvalidRequest(s) => InferenceError::ValidationError(s),
            LlmError::Server(s) => InferenceError::ProviderError { provider: String::new(), message: s },
            LlmError::Unknown(s) => InferenceError::ProviderError { provider: String::new(), message: s },
        }
    }
}

// ══════════════════════════════════════════════════════════════
// Stream Types
// ══════════════════════════════════════════════════════════════

/// 流式推理句柄
pub struct StreamHandle {
    /// 响应流
    pub rx: tokio::sync::mpsc::Receiver<Result<InferenceResponse, InferenceError>>,
    /// 使用的 provider
    pub provider: String,
}

// ══════════════════════════════════════════════════════════════
// Core Trait
// ══════════════════════════════════════════════════════════════

/// 统一推理接口 — 所有 LLM 调用使用此 trait
#[async_trait]
pub trait UnifiedInference: Send + Sync {
    /// 完整推理
    async fn complete(&self, request: &InferenceRequest) -> Result<InferenceResponse, InferenceError>;

    /// 流式推理
    async fn stream(&self, request: &InferenceRequest) -> Result<StreamHandle, InferenceError>;

    /// 查询能力
    fn capabilities(&self) -> InferenceCapabilities;

    /// 健康检查
    async fn health(&self) -> RouterHealth;

    /// 成本估算
    fn estimate_cost(&self, request: &InferenceRequest) -> CostEstimate;
}

// ══════════════════════════════════════════════════════════════
// Conversion Helpers
// ══════════════════════════════════════════════════════════════

impl InferenceRequest {
    /// 转换为 LlmRequest
    pub fn to_llm_request(&self) -> LlmRequest {
        let model = self.model.clone().unwrap_or_default();
        let mut req = LlmRequest::new(&model, &self.messages.iter().map(|m| m.content.as_str()).collect::<Vec<_>>().join("\n"));
        req.messages = self.messages.clone();
        req.temperature = self.temperature;
        req.max_tokens = self.max_tokens.unwrap_or(4096);
        req.tools = self.tools.clone();
        req.image_data = self.image_data.clone();
        req.thinking_budget = self.thinking_budget;
        req.structured_output = self.structured_output.clone();
        req
    }
}

impl InferenceResponse {
    /// 从 LlmResponse 转换
    pub fn from_llm_response(
        resp: LlmResponse,
        provider: &str,
        latency_ms: u64,
        fallback_count: u32,
    ) -> Self {
        Self {
            content: resp.content,
            model: resp.model,
            provider: provider.to_string(),
            usage: resp.usage,
            finish_reason: resp.finish_reason,
            tool_calls: resp.tool_calls,
            reasoning: resp.reasoning,
            metadata: ResponseMetadata {
                provider_selected: provider.to_string(),
                fallback_count,
                latency_ms,
                cost_estimate: CostEstimate::default(),
                from_cache: false,
            },
        }
    }
}
