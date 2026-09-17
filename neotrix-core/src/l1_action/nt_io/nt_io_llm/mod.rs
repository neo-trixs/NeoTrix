//! Unified LLM Module — Migration Interface Layer
//!
//! This module provides a unified `UnifiedLlm` trait as a simplified alternative
//! to the production `LlmProvider` trait in `nt_io_provider`. It also provides
//! `LlmRegistry` for dynamic adapter registration.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                 UnifiedLlm trait (simplified)            │
//! │  complete() / embedding() / health_check() / provider   │
//! ├─────────────────────────────────────────────────────────┤
//! │  LlmRegistry (adapter registry + default routing)       │
//! ├─────────────────────────────────────────────────────────┤
//! │  OpenAI │ Anthropic │ Ollama │ Groq │ Together │ ...   │
//! │  adapter│ adapter   │ adapter│ impl │ adapter  │       │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Design Principles
//!
//! 1. **Trait minimalism**: Only `complete`, `embedding`, `health_check`, `provider_type`
//! 2. **Registry pattern**: Dynamic adapter registration and lookup
//! 3. **Backward compatible**: Wraps existing `LlmProvider` implementations
//! 4. **Sync-first**: Methods are sync for simple use cases; async variants via `tokio::spawn`
//!
//! # Migration Status
//!
//! - **Production code** uses `nt_io_provider::LlmProvider` + `GatewayV2`
//! - **New code** can use `UnifiedLlm` + `LlmRegistry` for simpler use cases
//! - **Types** are unified via `neotrix-types::llm_types` (single source of truth)

#![deny(clippy::unwrap_used)]

pub mod registry;
pub mod adapters;

// ════════════════════════════════════════════════════════════════
// Re-export from neotrix-types (single source of truth)
// ════════════════════════════════════════════════════════════════

pub use neotrix_types::llm_types::{
    LlmError, LlmRequest, LlmResponse, Message, Role, Usage, FinishReason,
    Tool, ToolCallInfo, StructuredOutputConfig, DataTrust,
};

pub use registry::LlmRegistry;

// ════════════════════════════════════════════════════════════════
// Re-export canonical LlmProviderType from nt_io_provider
// ════════════════════════════════════════════════════════════════

pub use crate::l1_action::nt_io::nt_io_provider::LlmProviderType;

// ════════════════════════════════════════════════════════════════
// UnifiedLlm Trait
// ════════════════════════════════════════════════════════════════

/// Unified LLM interface — the single abstraction for all LLM providers.
///
/// This trait is intentionally minimal: only the four core operations.
/// Providers implement this trait and register with `LlmRegistry`.
pub trait UnifiedLlm: Send + Sync {
    /// Generate a completion for the given request.
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;

    /// Generate embeddings for the given texts.
    ///
    /// Returns a vector of embedding vectors, one per input text.
    /// Default implementation returns `UnsupportedOperation`.
    fn embedding(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>, LlmError> {
        Err(LlmError::UnsupportedOperation(
            "embedding not supported by this provider".to_string(),
        ))
    }

    /// Check if the provider is healthy and reachable.
    ///
    /// Default implementation returns `Ok(true)` — providers should
    /// override with actual health check logic.
    fn health_check(&self) -> Result<bool, LlmError> {
        Ok(true)
    }

    /// Return the provider type for this adapter.
    fn provider_type(&self) -> LlmProviderType;
}
