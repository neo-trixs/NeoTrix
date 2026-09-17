//! Catalog Unified Adapters — Groq, Together, and other OpenAI-compatible providers

use super::{LlmError, LlmProviderType, LlmRequest, LlmResponse, UnifiedLlm};
use crate::l1_action::nt_io::nt_io_provider::openai::OpenAiProvider;
use crate::core::nt_core_llm::LlmProvider;

// ════════════════════════════════════════════════════════════════
// Groq
// ════════════════════════════════════════════════════════════════

/// Adapter for Groq (OpenAI-compatible API)
pub struct GroqUnifiedAdapter {
    inner: OpenAiProvider,
}

impl GroqUnifiedAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            inner: OpenAiProvider::new(api_key).with_base_url("https://api.groq.com/openai/v1"),
        }
    }
}

impl UnifiedLlm for GroqUnifiedAdapter {
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        tokio::runtime::Handle::current()
            .block_on(self.inner.complete_raw(request))
    }

    fn provider_type(&self) -> LlmProviderType {
        LlmProviderType::Groq
    }

    fn health_check(&self) -> Result<bool, LlmError> {
        Ok(true)
    }
}

// ════════════════════════════════════════════════════════════════
// Together
// ════════════════════════════════════════════════════════════════

/// Adapter for Together AI (OpenAI-compatible API)
pub struct TogetherUnifiedAdapter {
    inner: OpenAiProvider,
}

impl TogetherUnifiedAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            inner: OpenAiProvider::new(api_key).with_base_url("https://api.together.xyz/v1"),
        }
    }
}

impl UnifiedLlm for TogetherUnifiedAdapter {
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        tokio::runtime::Handle::current()
            .block_on(self.inner.complete_raw(request))
    }

    fn provider_type(&self) -> LlmProviderType {
        LlmProviderType::TogetherFree
    }

    fn health_check(&self) -> Result<bool, LlmError> {
        Ok(true)
    }
}
