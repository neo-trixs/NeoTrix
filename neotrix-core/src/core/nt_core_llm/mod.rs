//! Backward-compatible re-export: types migrated to l1_action::nt_io::nt_io_llm
//!
//! Provides `LlmProvider` trait (old interface) alongside `UnifiedLlm` (new interface).
//! Note: `LlmProviderType` is defined in `nt_io_provider::common::factory` (comprehensive 30+ variants).

pub use crate::l1_action::nt_io::nt_io_llm::{
    LlmError, LlmRequest, LlmResponse, Message, Role, Usage, FinishReason,
    Tool, ToolCallInfo, StructuredOutputConfig, DataTrust,
    UnifiedLlm, LlmRegistry, LlmProviderType,
};

use async_trait::async_trait;
use tokio::sync::mpsc;

/// Backward-compatible LlmProvider trait (old interface).
///
/// The new interface is `UnifiedLlm`. This trait exists so that existing
/// implementations (`OpenAiProvider`, `AnthropicProvider`, etc.) continue to compile.
/// Providers should implement both traits for full compatibility.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Set proxy URL for HTTP requests.
    fn set_proxy(&mut self, proxy_url: &str);

    /// Raw completion: send request, get full response.
    async fn complete_raw(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;

    /// Streaming completion: returns a channel receiver of response chunks.
    async fn stream_complete_raw(
        &self,
        request: &LlmRequest,
    ) -> Result<mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError>;

    /// Data trust level for this provider.
    fn data_trust(&self) -> DataTrust;

    /// Complete (non-raw): delegates to complete_raw by default.
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.complete_raw(request).await
    }

    /// Stream complete: delegates to stream_complete_raw by default.
    async fn stream_complete(
        &self,
        request: &LlmRequest,
    ) -> Result<mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        self.stream_complete_raw(request).await
    }
}

/// Backward-compatible: estimate_tokens helper
pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Backward-compatible: truncate_preserving helper
pub fn truncate_preserving(text: &str, max_tokens: usize) -> &str {
    let max_chars = max_tokens * 4;
    if text.len() <= max_chars {
        text
    } else {
        &text[..max_chars]
    }
}

// ─── Additional backward-compatible stubs ──────────────────────────

/// Context budget calculation result
#[derive(Debug, Clone, Default)]
pub struct BudgetResult {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub within_budget: bool,
    pub is_cliff: bool,
    pub messages_evicted: usize,
    pub tool_outputs_truncated: usize,
}

impl BudgetResult {
    pub fn retention_ratio(&self) -> f64 {
        if self.input_tokens + self.output_tokens == 0 {
            1.0
        } else {
            (self.input_tokens + self.output_tokens - self.messages_evicted) as f64
                / (self.input_tokens + self.output_tokens) as f64
        }
    }
}

/// Apply context budget to messages (stub)
pub fn apply_context_budget(_messages: &[Message], _max_tokens: usize) -> BudgetResult {
    BudgetResult::default()
}

/// Estimate tokens in a list of messages (stub)
pub fn estimate_messages_tokens(_messages: &[Message]) -> usize {
    0
}

/// Unified provider metadata (stub)
#[derive(Debug, Clone, Default)]
pub struct ProviderMetadata {
    pub name: String,
    pub version: String,
}

/// Provider capability flags
#[derive(Debug, Clone, Default)]
pub struct ProviderCapabilities {
    pub text: bool,
    pub vision: bool,
    pub tools: bool,
    pub streaming: bool,
}

/// Unified provider trait (stub, distinct from LlmProvider)
pub trait UnifiedProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;

    fn estimate_cost(&self, _request: &LlmRequest) -> CostEstimate {
        CostEstimate::default()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }

    fn health(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async { HealthStatus::healthy() })
    }
}

/// Health status (stub)
#[derive(Debug, Clone, Default)]
pub struct HealthStatus {
    pub healthy: bool,
    pub message: String,
}

impl HealthStatus {
    pub fn healthy() -> Self {
        Self { healthy: true, message: "OK".into() }
    }
}

/// Cost estimate (stub)
#[derive(Debug, Clone, Default)]
pub struct CostEstimate {
    pub input_cost: f64,
    pub output_cost: f64,
    pub total_cost: f64,
    pub estimated_cost_usd: f64,
}

/// Register LLM self tests (stub)
pub fn register_llm_self_tests() -> Vec<String> {
    vec![]
}

/// Redact internal content (stub)
pub fn redact_internals(content: &str) -> String {
    content.to_string()
}

/// Egress privacy guard (stub)
pub fn egress_privacy_guard(content: &str) -> Result<String, LlmError> {
    Ok(content.to_string())
}
