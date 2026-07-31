//! LLM Provider 模块
//!
//! 统一架构: Provider Pool + IP Proxy Pool + Gateway
//! 参考: One API / New API / LiteLLM / freellmpool
//! 2026-06-30 架构升级: GatewayProvider 包装所有中间件层

pub mod types;
pub mod openai;
pub mod ollama;
pub mod anthropic;
pub mod gemini;
pub mod factory;
pub mod search_router;
pub mod agent_routing;
pub mod discovery;
pub mod free_catalog;
pub mod compaction;
pub mod circuit_breaker;
pub mod rate_limiter;
pub mod rate_profiles;
pub mod free_providers;
pub mod free_pool;
pub mod gateway;
pub mod provider_catalog;
pub mod provider_swap;

// Re-export 核心类型
pub use types::{
    LlmProvider, LlmRequest, Message, Role, Tool, ToolCallInfo, ToolCallFunction,
    LlmResponse, Usage, FinishReason, LlmError,
};

// Re-export Provider 实现
pub use openai::OpenAiProvider;
pub use ollama::OllamaProvider;
pub use anthropic::AnthropicProvider;
pub use gemini::GeminiProvider;

// Re-export 免费 Provider
pub use free_providers::{GroqProvider, OpenRouterProvider, PollinationsProvider, CerebrasProvider};
pub use free_pool::FreePool;

// Re-export 工厂和配置
pub use factory::{LlmProviderType, ProviderConfig, create_provider, create_provider_from_type, create_gateway};

// Re-export 路由和配置管理
pub use agent_routing::{AgentRoutingTable, ProviderProfile, ProviderProfileManager};

// Re-export 网关
pub use gateway::{GatewayV2, CallEvent, CallObserver, AttemptPhase, SubGrid, CapabilityCoordinator, CapabilityIntent, CoordinationRequest, CoordinationOutcome, SubGridHealth};

// Re-export 断路器 + 限流器
pub use circuit_breaker::CircuitBreaker;
pub use rate_limiter::RateLimiter;
pub use rate_limiter::TokenBucket;

// Re-export ProviderCatalog
pub use provider_catalog::{ProviderCategory, ProviderInfo, CommunicationProfile, PROVIDER_CATALOG, lookup_provider, providers_by_category, providers_with_key, keyless_providers};
pub use provider_swap::{ProviderSwapManager, ProviderHealth, SwapRule, GLOBAL_SWAP_MANAGER, ProviderHealthSummary};
