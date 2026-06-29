//! LLM Provider 模块
//!
//! 支持多种 LLM: OpenAI, Anthropic, Gemini, Ollama 等

pub mod agent_routing;
pub mod anonymous_provider;
pub mod anthropic;
pub mod api_key_pool;
pub mod factory;
pub mod gemini;
pub mod identity_council;
pub mod ollama;
pub mod openai;
pub mod search_router;
pub mod types;

// Re-export 核心类型
pub use types::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Message, Role, Tool, ToolCall,
    ToolCallFunction, Usage,
};

pub use compaction::sanitize_history;

// Re-export Provider 实现
pub use anthropic::AnthropicProvider;
pub use gemini::GeminiProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;

// Re-export Identity Council & Anonymous Provider
pub use identity_council::{
    CallPlan, HealthDashboard, IdentityCouncil, KeyPriority, ProviderHeatSnapshot,
};
pub use anonymous_provider::AnonymousLlmProvider;

// Re-export 工厂和配置
pub use factory::{create_provider, create_provider_from_type, LlmProviderType, ProviderConfig};

// Re-export 路由和配置管理
pub use agent_routing::{AgentRoutingTable, ProviderProfile, ProviderProfileManager};

pub mod compaction;
pub mod discovery;
pub mod free_catalog;
pub mod internet_discovery;
pub mod okf_exporter;
pub mod token_economy;
pub use okf_exporter::OkfExporter;
