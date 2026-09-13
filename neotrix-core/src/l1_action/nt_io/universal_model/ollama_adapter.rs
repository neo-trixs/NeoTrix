//! Ollama 适配器 — 将 OllamaProvider 包装为 UniversalModel

use async_trait::async_trait;

use crate::core::l2_perception::nt_core_llm::{DataTrust, LlmError, LlmProvider, LlmRequest, LlmResponse};
use crate::l1_action::nt_io::nt_io_provider::ollama::OllamaProvider;

use super::traits::{ModelCapabilities, ModelHealth, ModelIdentifier, TaskType, UniversalModel};

/// Ollama UniversalModel 适配器
pub struct OllamaUniversal {
    inner: OllamaProvider,
    model_id: String,
    capabilities: ModelCapabilities,
}

impl OllamaUniversal {
    /// Create a new Ollama universal model adapter.
    ///
    /// Ollama is local-only (no API key needed). Detects model capabilities
    /// from the `model_id` string and wraps `OllamaProvider`.
    pub fn new(model_id: &str) -> Self {
        let capabilities = Self::detect_capabilities(model_id);
        Self {
            inner: OllamaProvider::new(),
            model_id: model_id.to_string(),
            capabilities,
        }
    }

    /// Override the Ollama server URL (default: http://localhost:11434).
    pub fn with_base_url(self, url: &str) -> Self {
        Self {
            inner: self.inner.with_base_url(url),
            ..self
        }
    }

    /// Detect model capabilities from the model ID string.
    ///
    /// Note: Real implementation should call `ollama list` or `/api/tags` to
    /// discover available models and their capabilities dynamically, rather
    /// than hardcoding. This enables support for user-pulled models.
    fn detect_capabilities(model_id: &str) -> ModelCapabilities {
        match model_id {
            "llama3" | "llama3:70b" | "llama3.1" | "llama3.1:70b" => ModelCapabilities {
                task_types: vec![TaskType::Chat, TaskType::Completion],
                supports_streaming: true,
                supports_tools: false,
                supports_vision: false,
                max_context_tokens: 8_192,
                max_output_tokens: 4_096,
                ..Default::default()
            },
            "llava" | "llava:13b" | "bakllava" => ModelCapabilities {
                task_types: vec![TaskType::Chat],
                supports_streaming: true,
                supports_vision: true,
                max_context_tokens: 4_096,
                max_output_tokens: 2_048,
                ..Default::default()
            },
            "nomic-embed-text" | "mxbai-embed-large" => ModelCapabilities {
                task_types: vec![TaskType::Embedding],
                supports_embedding: true,
                max_context_tokens: 8_192,
                max_output_tokens: 0,
                embedding_dim: Some(768),
                ..Default::default()
            },
            _ => ModelCapabilities {
                task_types: vec![TaskType::Chat],
                supports_streaming: true,
                max_context_tokens: 4_096,
                max_output_tokens: 2_048,
                ..Default::default()
            },
        }
    }
}

impl ModelIdentifier for OllamaUniversal {
    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[async_trait]
impl UniversalModel for OllamaUniversal {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.inner.complete(request).await
    }

    /// STUB: Returns default health. Real implementation should call `ollama ps`
    /// to check running models and track local inference latency.
    fn health(&self) -> ModelHealth {
        ModelHealth::default()
    }

    fn capabilities(&self) -> ModelCapabilities {
        self.capabilities.clone()
    }

    /// Local Ollama is Trusted — no secrets or user data leave the machine.
    fn data_trust(&self) -> DataTrust {
        DataTrust::Trusted
    }

    fn set_proxy(&mut self, proxy_url: &str) {
        self.inner.set_proxy(proxy_url);
    }
}
