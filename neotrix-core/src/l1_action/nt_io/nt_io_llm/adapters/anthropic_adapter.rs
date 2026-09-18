//! Anthropic Unified Adapter

use super::{LlmError, LlmProviderType, LlmRequest, LlmResponse, UnifiedLlm};
use crate::l1_action::nt_io::nt_io_provider::anthropic::AnthropicProvider;
use crate::l1_action::nt_core_llm::LlmProvider;

/// Adapter wrapping `AnthropicProvider` to implement `UnifiedLlm`
pub struct AnthropicUnifiedAdapter {
    inner: AnthropicProvider,
}

impl AnthropicUnifiedAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            inner: AnthropicProvider::new(api_key),
        }
    }
}

impl UnifiedLlm for AnthropicUnifiedAdapter {
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        tokio::runtime::Handle::current()
            .block_on(self.inner.complete_raw(request))
    }

    fn provider_type(&self) -> LlmProviderType {
        LlmProviderType::Anthropic
    }

    fn health_check(&self) -> Result<bool, LlmError> {
        Ok(true)
    }
}
