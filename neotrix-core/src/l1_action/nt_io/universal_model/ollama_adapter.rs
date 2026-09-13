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
    pub fn new(model_id: &str) -> Self {
        let capabilities = Self::detect_capabilities(model_id);
        Self {
            inner: OllamaProvider::new(),
            model_id: model_id.to_string(),
            capabilities,
        }
    }

    pub fn with_base_url(self, url: &str) -> Self {
        Self {
            inner: self.inner.with_base_url(url),
            ..self
        }
    }

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

    fn health(&self) -> ModelHealth {
        ModelHealth::default()
    }

    fn capabilities(&self) -> ModelCapabilities {
        self.capabilities.clone()
    }

    fn data_trust(&self) -> DataTrust {
        DataTrust::Trusted
    }

    fn set_proxy(&mut self, proxy_url: &str) {
        self.inner.set_proxy(proxy_url);
    }
}
