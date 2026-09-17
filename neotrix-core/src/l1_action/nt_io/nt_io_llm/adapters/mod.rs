//! Provider Adapters — wrappers that implement UnifiedLlm for existing providers
//!
//! Each adapter wraps an existing provider from `nt_io_provider` and implements
//! the `UnifiedLlm` trait, bridging the old and new interfaces.
//!
//! Note: Since types are now unified via `neotrix-types::llm_types`, no conversion
//! is needed. Adapters directly use the same types.

use super::{LlmError, LlmProviderType, LlmRequest, LlmResponse, UnifiedLlm};

pub mod openai_adapter;
pub mod anthropic_adapter;
pub mod ollama_adapter;
pub mod catalog_adapter;

pub use openai_adapter::OpenAiUnifiedAdapter;
pub use anthropic_adapter::AnthropicUnifiedAdapter;
pub use ollama_adapter::OllamaUnifiedAdapter;
pub use catalog_adapter::{GroqUnifiedAdapter, TogetherUnifiedAdapter};
