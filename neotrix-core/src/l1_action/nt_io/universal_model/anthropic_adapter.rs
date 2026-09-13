//! Anthropic 适配器 — 将 AnthropicProvider 包装为 UniversalModel

use async_trait::async_trait;

use crate::core::l2_perception::nt_core_llm::{DataTrust, LlmError, LlmProvider, LlmRequest, LlmResponse};
use crate::l1_action::nt_io::nt_io_provider::anthropic::AnthropicProvider;

use super::traits::{ModelCapabilities, ModelHealth, ModelIdentifier, TaskType, UniversalModel};

/// Anthropic UniversalModel 适配器
pub struct AnthropicUniversal {
    inner: AnthropicProvider,
    model_id: String,
    capabilities: ModelCapabilities,
}

impl AnthropicUniversal {
    /// Create a new Anthropic universal model adapter.
    ///
    /// Detects model capabilities from the `model_id` string and wraps
    /// the underlying `AnthropicProvider` for unified `UniversalModel` access.
    pub fn new(api_key: String, model_id: &str) -> Self {
        let capabilities = Self::detect_capabilities(model_id);
        Self {
            inner: AnthropicProvider::new(api_key),
            model_id: model_id.to_string(),
            capabilities,
        }
    }

    /// Detect model capabilities from the model ID string.
    ///
    /// Note: Real implementation should query Anthropic's model endpoint
    /// or use a remote capability registry. The hardcoded map covers known
    /// Claude models; newer versions (Claude 4, etc.) will need manual addition.
    fn detect_capabilities(model_id: &str) -> ModelCapabilities {
        match model_id {
            "claude-sonnet-4-20250514" | "claude-3-5-sonnet-20241022" => ModelCapabilities {
                task_types: vec![TaskType::Chat, TaskType::Completion],
                supports_streaming: true,
                supports_tools: true,
                supports_structured_output: true,
                supports_vision: true,
                supports_thinking: true,
                max_context_tokens: 200_000,
                max_output_tokens: 16_384,
                languages: vec!["en".to_string(), "zh".to_string(), "ja".to_string()],
                ..Default::default()
            },
            "claude-3-opus-20240229" => ModelCapabilities {
                task_types: vec![TaskType::Chat],
                supports_streaming: true,
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
                max_context_tokens: 200_000,
                max_output_tokens: 4_096,
                ..Default::default()
            },
            "claude-3-haiku-20240307" => ModelCapabilities {
                task_types: vec![TaskType::Chat],
                supports_streaming: true,
                supports_tools: true,
                supports_vision: true,
                max_context_tokens: 200_000,
                max_output_tokens: 4_096,
                ..Default::default()
            },
            _ => ModelCapabilities {
                task_types: vec![TaskType::Chat],
                supports_streaming: true,
                supports_tools: true,
                max_context_tokens: 200_000,
                max_output_tokens: 4_096,
                ..Default::default()
            },
        }
    }
}

impl ModelIdentifier for AnthropicUniversal {
    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[async_trait]
impl UniversalModel for AnthropicUniversal {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.inner.complete(request).await
    }

    /// Returns health status for the Anthropic adapter.
    ///
    /// Currently returns an unavailable state since no live probe is wired.
    /// Real implementation should call `/v1/messages` endpoint and track
    /// latency/error rate for the circuit breaker.
    fn health(&self) -> ModelHealth {
        ModelHealth {
            available: false,
            latency_ms: None,
            error_rate: 1.0,
            last_success: None,
            circuit_breaker_open: true,
            message: Some("Health probe not wired — requires /v1/messages endpoint check".into()),
        }
    }

    fn capabilities(&self) -> ModelCapabilities {
        self.capabilities.clone()
    }

    fn data_trust(&self) -> DataTrust {
        DataTrust::Contracted
    }

    fn set_proxy(&mut self, proxy_url: &str) {
        self.inner.set_proxy(proxy_url);
    }
}
