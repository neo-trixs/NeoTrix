//! LLM Provider 模块
//!
//! 统一架构: Provider Pool + IP Proxy Pool + Gateway
#![deny(clippy::unwrap_used)]

pub mod anthropic;
pub mod catalog;
pub mod common;
pub mod gateway;
pub mod gemini;
pub mod health;
pub mod llama;
pub mod ollama;
pub mod openai;
pub mod pool;
pub mod routing;

// Backward-compat re-exports at flat paths
pub use catalog::{discovery, free_catalog, provider_catalog, registry};
pub use common::{factory, generation_classifier, privacy_guard, types};
pub use health::{circuit_breaker, compaction, context_budget, rate_limiter, rate_profiles};
pub use pool::{account_pool, free_pool, provider_pool};
pub use routing::{failover_history, provider_swap};

// Re-export key types at module level
pub use gateway::{
    AttemptPhase, CallEvent, CallObserver, CapabilityCoordinator, CapabilityIntent,
    CoordinationOutcome, CoordinationRequest, GatewayV2, SubGrid, SubGridHealth,
    AgentRoutingTable, ProviderProfile, ProviderProfileManager,
    CapabilityRouter,
    InferenceRouter, RouterConfig,
};
pub use gateway::universal_adapter::*;
pub use gateway::unified_inference::*;
pub use gateway::search_router::*;
pub use gateway::free_providers::*;
pub use catalog::{
    find_by_capabilities, find_by_capabilities_in_category,
    keyless_providers, lookup_provider, providers_by_category, providers_with_key,
    CommunicationProfile, ProviderCapabilities, ProviderCategory, ProviderInfo, PROVIDER_CATALOG,
};
pub use catalog::registry::ProviderRegistry;
pub use common::{
    create_gateway, create_provider, create_provider_from_type, LlmProviderType, ProviderConfig,
    Classification, Complexity, Domain, GenerationAnalytics, GenerationClassifier,
    GenerationRecord, TaskType,
};
pub use common::types::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Message, Role, Tool,
    ToolCallFunction, ToolCallInfo, Usage,
};
pub use common::context_budget::{apply_context_budget, estimate_messages_tokens, estimate_tokens};
pub use anthropic::AnthropicProvider;
pub use gemini::GeminiProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use health::circuit_breaker::CircuitBreaker;
pub use health::rate_limiter::{RateLimiter, TokenBucket, AdaptivePacer, BrainTier, TieredSemaphore};
pub use pool::account_pool::{
    AccountHealth, AccountLease, AccountPool, AccountPoolConfig, AccountPoolError,
};
pub use pool::free_pool::FreePool;
pub use pool::provider_pool::{global_provider_pool, PoolEntry, ProviderPool};
pub use routing::failover_history::{
    clear_history, failover_history, record_failover, report as failover_report, total_failovers,
    FailoverEvent, FailoverHistory,
};
pub use routing::provider_swap::{
    ProviderHealth, ProviderHealthSummary, ProviderSwapManager, SwapRule, GLOBAL_SWAP_MANAGER,
};
