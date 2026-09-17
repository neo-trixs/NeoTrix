//! OpenAI Unified Adapter

use super::{LlmError, LlmProviderType, LlmRequest, LlmResponse, UnifiedLlm};
use crate::l1_action::nt_io::nt_io_provider::openai::OpenAiProvider;
use crate::core::nt_core_llm::LlmProvider;

/// Adapter wrapping `OpenAiProvider` to implement `UnifiedLlm`
pub struct OpenAiUnifiedAdapter {
    inner: OpenAiProvider,
}

impl OpenAiUnifiedAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            inner: OpenAiProvider::new(api_key),
        }
    }

    pub fn with_base_url(self, url: &str) -> Self {
        Self {
            inner: self.inner.with_base_url(url),
        }
    }
}

impl UnifiedLlm for OpenAiUnifiedAdapter {
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        tokio::runtime::Handle::current()
            .block_on(self.inner.complete_raw(request))
    }

    fn provider_type(&self) -> LlmProviderType {
        LlmProviderType::OpenAI
    }

    fn health_check(&self) -> Result<bool, LlmError> {
        // For now, just return true — real implementation would probe the API
        Ok(true)
    }
}
