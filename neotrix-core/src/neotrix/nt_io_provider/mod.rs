//! LLM Provider 模块
//!
//! 支持多种 LLM: OpenAI, Anthropic, Gemini, Ollama 等

pub mod types;
pub mod openai;
pub mod ollama;
pub mod anthropic;
pub mod gemini;
pub mod factory;
pub mod search_router;
pub mod agent_routing;

// Re-export 核心类型
pub use types::{
    LlmProvider, LlmRequest, Message, Role, Tool,
    LlmResponse, Usage, FinishReason, LlmError,
};

// Re-export Provider 实现
pub use openai::OpenAiProvider;
pub use ollama::OllamaProvider;
pub use anthropic::AnthropicProvider;
pub use gemini::GeminiProvider;

// Re-export 工厂和配置
pub use factory::{LlmProviderType, ProviderConfig, create_provider, create_provider_from_type};

// Re-export 路由和配置管理
pub use agent_routing::{AgentRoutingTable, ProviderProfile, ProviderProfileManager};
