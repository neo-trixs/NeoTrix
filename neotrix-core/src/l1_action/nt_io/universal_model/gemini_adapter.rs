//! Gemini 适配器 — 将 GeminiProvider 包装为 UniversalModel

use async_trait::async_trait;

use crate::core::l2_perception::nt_core_llm::{DataTrust, LlmError, LlmProvider, LlmRequest, LlmResponse};
use crate::l1_action::nt_io::nt_io_provider::gemini::GeminiProvider;

use super::traits::{ModelCapabilities, ModelHealth, ModelIdentifier, TaskType, UniversalModel};

/// Gemini UniversalModel 适配器
pub struct GeminiUniversal {
    inner: GeminiProvider,
    model_id: String,
    capabilities: ModelCapabilities,
}

impl GeminiUniversal {
    /// Create a new Gemini universal model adapter.
    ///
    /// Detects model capabilities from the `model_id` string and wraps
    /// the underlying `GeminiProvider` for unified `UniversalModel` access.
    pub fn new(api_key: String, model_id: &str) -> Self {
        let capabilities = Self::detect_capabilities(model_id);
        Self {
            inner: GeminiProvider::new(api_key),
            model_id: model_id.to_string(),
            capabilities,
        }
    }

    /// Override the base URL for API requests (e.g., for Vertex AI or proxied endpoints).
    pub fn with_base_url(self, url: &str) -> Self {
        Self {
            inner: self.inner.with_base_url(url),
            ..self
        }
    }

    /// Detect model capabilities from the model ID string.
    ///
    /// Note: Real implementation should query Gemini's model metadata endpoint
    /// or maintain a remote capability registry. The hardcoded map here is a
    /// subset; newer Gemini models (2.5, etc.) will need manual addition.
    fn detect_capabilities(model_id: &str) -> ModelCapabilities {
        match model_id {
            "gemini-1.5-pro" | "gemini-2.0-flash" => ModelCapabilities {
                task_types: vec![TaskType::Chat, TaskType::Completion],
                supports_streaming: true,
                supports_tools: true,
                supports_structured_output: true,
                supports_vision: true,
                supports_thinking: false,
                max_context_tokens: 1_000_000,
                max_output_tokens: 8_192,
                languages: vec!["en".to_string(), "zh".to_string(), "ja".to_string()],
                ..Default::default()
            },
            "gemini-1.5-flash" => ModelCapabilities {
                task_types: vec![TaskType::Chat],
                supports_streaming: true,
                supports_tools: true,
                supports_vision: true,
                max_context_tokens: 1_000_000,
                max_output_tokens: 8_192,
                ..Default::default()
            },
            "text-embedding-004" => ModelCapabilities {
                task_types: vec![TaskType::Embedding],
                supports_embedding: true,
                max_context_tokens: 2_048,
                max_output_tokens: 0,
                embedding_dim: Some(768),
                ..Default::default()
            },
            _ => ModelCapabilities {
                task_types: vec![TaskType::Chat],
                supports_streaming: true,
                supports_tools: true,
                max_context_tokens: 32_768,
                max_output_tokens: 8_192,
                ..Default::default()
            },
        }
    }
}

impl ModelIdentifier for GeminiUniversal {
    fn provider_name(&self) -> &str {
        "gemini"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[async_trait]
impl UniversalModel for GeminiUniversal {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.inner.complete(request).await
    }

    /// STUB: Returns default health. Real implementation should probe the Gemini
    /// /v1/models endpoint and track latency/error rate for circuit breaker.
    fn health(&self) -> ModelHealth {
        ModelHealth::default()
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
