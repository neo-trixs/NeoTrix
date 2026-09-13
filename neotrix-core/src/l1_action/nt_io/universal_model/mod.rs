//! Universal Model Interface — 模型无关的统一调用层
//!
//! 为所有外部模型（LLM / Embedding / Vision）提供统一接口，
//! 消除不同 provider 之间的 API 差异。
//!
//! # 架构
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                 UniversalModel trait                     │
//! │  complete() / embed() / health() / capabilities()      │
//! ├─────────────────────────────────────────────────────────┤
//! │  OpenAI  │  Anthropic  │  Gemini  │  Ollama  │  ...   │
//! │  adapter │  adapter    │  adapter │  adapter │        │
//! ├─────────────────────────────────────────────────────────┤
//! │              FallbackRouter (fallback chain)            │
//! │              CapabilityDetector (auto-discovery)        │
//! │              HealthChecker (runtime monitoring)         │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use crate::l1_action::nt_io::universal_model::*;
//!
//! // 创建模型
//! let model = OpenAIUniversal::new("sk-xxx".to_string(), "gpt-4o");
//!
//! // 统一调用
//! let response = model.complete(&request).await?;
//!
//! // 检查能力
//! let caps = model.capabilities();
//! if caps.supports_tools {
//!     // 使用工具调用
//! }
//! ```

pub mod traits;
pub mod openai_adapter;
pub mod anthropic_adapter;
pub mod gemini_adapter;
pub mod ollama_adapter;
pub mod fallback;
pub mod capabilities;

// Re-exports
pub use traits::{
    EmbeddingRequest, EmbeddingResponse, ModelCapabilities, ModelError, ModelHealth, ModelIdentifier,
    ModelInfo, TaskType, UniversalModel,
};
pub use openai_adapter::OpenAIUniversal;
pub use anthropic_adapter::AnthropicUniversal;
pub use gemini_adapter::GeminiUniversal;
pub use ollama_adapter::OllamaUniversal;
pub use fallback::{FallbackConfig, FallbackRouter};
pub use capabilities::{CapabilityDetector, HealthChecker};
