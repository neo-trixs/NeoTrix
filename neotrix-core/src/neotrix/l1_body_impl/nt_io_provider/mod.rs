//! LLM Provider 模块
//!
//! 统一架构: Provider Pool + IP Proxy Pool + Gateway
//! 参考: One API / New API / LiteLLM / freellmpool
//! 2026-06-30 架构升级: GatewayProvider 包装所有中间件层
#![deny(clippy::unwrap_used)]

pub mod agent_routing;
pub mod anthropic;
pub mod account_pool;
pub mod circuit_breaker;
pub mod compaction;
pub mod context_budget;
pub mod discovery;
pub mod factory;
pub mod failover_history;
pub mod free_catalog;
pub mod free_pool;
pub mod free_providers;
pub mod gateway;
pub mod generation_classifier;
pub mod gemini;
pub mod ollama;
pub mod openai;
pub mod provider_catalog;
pub mod provider_swap;
pub mod rate_limiter;
pub mod rate_profiles;
pub mod search_router;
pub mod types;

// Re-export 核心类型
pub use types::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Message, Role, Tool,
    ToolCallFunction, ToolCallInfo, Usage,
};

// Re-export Token 预算引擎 (上下文压缩, AgentLoop/neocodex 共享)
pub use context_budget::{apply_context_budget, estimate_messages_tokens, estimate_tokens};

// Re-export 故障转移历史
pub use failover_history::{
    clear_history, failover_history, record_failover, report as failover_report, total_failovers,
    FailoverEvent, FailoverHistory,
};

// Re-export Provider 实现
pub use anthropic::AnthropicProvider;
pub use gemini::GeminiProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;

// Re-export F6 生成分类器
pub use generation_classifier::{
    Classification, Complexity, Domain, GenerationAnalytics, GenerationClassifier,
    GenerationRecord, TaskType,
};

// Re-export 免费 Provider
pub use free_pool::FreePool;
pub use free_providers::{
    CerebrasProvider, GroqProvider, OpenRouterProvider, PollinationsProvider,
};

// Re-export 工厂和配置
pub use factory::{
    create_gateway, create_provider, create_provider_from_type, LlmProviderType, ProviderConfig,
};

// Re-export 路由和配置管理
pub use agent_routing::{AgentRoutingTable, ProviderProfile, ProviderProfileManager};

// Re-export 网关
pub use gateway::{
    AttemptPhase, CallEvent, CallObserver, CapabilityCoordinator, CapabilityIntent,
    CoordinationOutcome, CoordinationRequest, GatewayV2, SubGrid, SubGridHealth,
};

// Re-export 断路器 + 限流器
pub use circuit_breaker::CircuitBreaker;
pub use rate_limiter::RateLimiter;
pub use rate_limiter::TokenBucket;
pub use rate_limiter::{AdaptivePacer, BrainTier, TieredSemaphore};

// Re-export 账户池 (P7 吸收)
pub use account_pool::{
    AccountHealth, AccountLease, AccountPool, AccountPoolConfig, AccountPoolError,
};

// Re-export ProviderCatalog
pub use provider_catalog::{
    keyless_providers, lookup_provider, providers_by_category, providers_with_key,
    CommunicationProfile, ProviderCategory, ProviderInfo, PROVIDER_CATALOG,
};
pub use provider_swap::{
    ProviderHealth, ProviderHealthSummary, ProviderSwapManager, SwapRule, GLOBAL_SWAP_MANAGER,
};
