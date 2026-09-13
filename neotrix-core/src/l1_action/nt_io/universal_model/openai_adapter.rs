//! OpenAI 适配器 — 将 OpenAiProvider 包装为 UniversalModel

use async_trait::async_trait;

use crate::core::l2_perception::nt_core_llm::{DataTrust, LlmError, LlmProvider, LlmRequest, LlmResponse};
use crate::l1_action::nt_io::nt_io_provider::openai::OpenAiProvider;

use super::traits::{ModelCapabilities, ModelHealth, ModelIdentifier, TaskType, UniversalModel};

/// OpenAI UniversalModel 适配器
pub struct OpenAIUniversal {
    inner: OpenAiProvider,
    model_id: String,
    capabilities: ModelCapabilities,
}

impl OpenAIUniversal {
    /// Create a new OpenAI universal model adapter.
    ///
    /// Detects model capabilities from the `model_id` string and wraps
    /// the underlying `OpenAiProvider` for unified `UniversalModel` access.
    pub fn new(api_key: String, model_id: &str) -> Self {
        let capabilities = Self::detect_capabilities(model_id);
        Self {
            inner: OpenAiProvider::new(api_key),
            model_id: model_id.to_string(),
            capabilities,
        }
    }

    /// Override the base URL for API requests (e.g., for proxied endpoints).
    pub fn with_base_url(self, url: &str) -> Self {
        Self {
            inner: self.inner.with_base_url(url),
            ..self
        }
    }

    /// Enable/disable anonymous Zen mode (hides user-identifying headers).
    pub fn with_zen_anonymous(self, v: bool) -> Self {
        Self {
            inner: self.inner.with_zen_anonymous(v),
            ..self
        }
    }

    /// Detect model capabilities from the model ID string.
    ///
    /// Note: Real implementation should query the provider's model list endpoint
    /// dynamically rather than hardcoding known models. This ensures new models
    /// (e.g., gpt-5, o1-pro) are supported without code changes.
    fn detect_capabilities(model_id: &str) -> ModelCapabilities {
        match model_id {
            "gpt-4o" | "gpt-4o-mini" => ModelCapabilities {
                task_types: vec![TaskType::Chat, TaskType::Completion],
                supports_streaming: true,
                supports_tools: true,
                supports_structured_output: true,
                supports_vision: true,
                supports_thinking: false,
                supports_embedding: false,
                max_context_tokens: 128_000,
                max_output_tokens: 16_384,
                embedding_dim: None,
                languages: vec!["en".to_string(), "zh".to_string()],
                ..Default::default()
            },
            "gpt-4-turbo" => ModelCapabilities {
                task_types: vec![TaskType::Chat],
                supports_streaming: true,
                supports_tools: true,
                supports_structured_output: true,
                supports_vision: true,
                max_context_tokens: 128_000,
                max_output_tokens: 4_096,
                ..Default::default()
            },
            "gpt-3.5-turbo" => ModelCapabilities {
                task_types: vec![TaskType::Chat],
                supports_streaming: true,
                supports_tools: true,
                max_context_tokens: 16_385,
                max_output_tokens: 4_096,
                ..Default::default()
            },
            "text-embedding-3-large" => ModelCapabilities {
                task_types: vec![TaskType::Embedding],
                supports_embedding: true,
                max_context_tokens: 8_191,
                max_output_tokens: 0,
                embedding_dim: Some(3072),
                ..Default::default()
            },
            "text-embedding-3-small" | "text-embedding-ada-002" => ModelCapabilities {
                task_types: vec![TaskType::Embedding],
                supports_embedding: true,
                max_context_tokens: 8_191,
                max_output_tokens: 0,
                embedding_dim: Some(1536),
                ..Default::default()
            },
            _ => ModelCapabilities::default(),
        }
    }
}

impl ModelIdentifier for OpenAIUniversal {
    fn provider_name(&self) -> &str {
        "openai"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[async_trait]
impl UniversalModel for OpenAIUniversal {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.inner.complete(request).await
    }

    /// Returns health status for the OpenAI adapter.
    ///
    /// Currently returns an unavailable state since no live probe is wired.
    /// Real implementation should call `/v1/models` endpoint and track
    /// latency/error rate for the circuit breaker.
    fn health(&self) -> ModelHealth {
        ModelHealth {
            available: false,
            latency_ms: None,
            error_rate: 1.0,
            last_success: None,
            circuit_breaker_open: true,
            message: Some("Health probe not wired — requires /v1/models endpoint check".into()),
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
