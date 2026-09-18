//! Ollama Unified Adapter

use super::{LlmError, LlmProviderType, LlmRequest, LlmResponse, UnifiedLlm};
use crate::l1_action::nt_io::nt_io_provider::ollama::OllamaProvider;
use crate::l1_action::nt_core_llm::LlmProvider;

/// Adapter wrapping `OllamaProvider` to implement `UnifiedLlm`
pub struct OllamaUnifiedAdapter {
    inner: OllamaProvider,
}

impl OllamaUnifiedAdapter {
    pub fn new() -> Self {
        Self {
            inner: OllamaProvider::new(),
        }
    }

    pub fn with_base_url(self, url: &str) -> Self {
        Self {
            inner: self.inner.with_base_url(url),
        }
    }
}

impl Default for OllamaUnifiedAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedLlm for OllamaUnifiedAdapter {
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        tokio::runtime::Handle::current()
            .block_on(self.inner.complete_raw(request))
    }

    fn provider_type(&self) -> LlmProviderType {
        LlmProviderType::Ollama
    }

    fn health_check(&self) -> Result<bool, LlmError> {
        Ok(true)
    }
}
