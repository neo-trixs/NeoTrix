//! LLM Provider 模块
//!
//! 统一架构: Provider Pool + IP Proxy Pool + Gateway
//! 参考: One API / New API / LiteLLM / freellmpool
//! 2026-06-30 架构升级: GatewayProvider 包装所有中间件层
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

// Re-export from anthropic
pub use anthropic::AnthropicProvider;

// Re-export from catalog
pub use catalog::{
    find_by_capabilities, find_by_capabilities_in_category,
    keyless_providers, lookup_provider, providers_by_category, providers_with_key,
    CommunicationProfile, ProviderCapabilities, ProviderCategory, ProviderInfo, PROVIDER_CATALOG,
};
pub use catalog::ProviderRegistry;

// Re-export from common
pub use common::{
    create_gateway, create_provider, create_provider_from_type, LlmProviderType, ProviderConfig,
    Classification, Complexity, Domain, GenerationAnalytics, GenerationClassifier,
    GenerationRecord, TaskType,
};
pub use common::types::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Message, Role, Tool,
    ToolCallFunction, ToolCallInfo, Usage,
};

// Re-export from gemini
pub use gemini::GeminiProvider;

// Re-export from health
pub use health::CircuitBreaker;
pub use health::RateLimiter;
pub use health::TokenBucket;
pub use health::{AdaptivePacer, BrainTier, TieredSemaphore};
pub use health::{apply_context_budget, estimate_messages_tokens, estimate_tokens};

// Re-export from llama
pub use llama::LlamaProcess;

// Re-export from ollama
pub use ollama::OllamaProvider;

// Re-export from openai
pub use openai::OpenAiProvider;

// Re-export from pool
pub use pool::{
    AccountHealth, AccountLease, AccountPool, AccountPoolConfig, AccountPoolError,
    FreePool,
    global_provider_pool, PoolEntry, ProviderPool,
};

// Re-export from routing
pub use routing::{
    clear_history, failover_history, record_failover, report as failover_report, total_failovers,
    FailoverEvent, FailoverHistory,
    ProviderHealth, ProviderHealthSummary, ProviderSwapManager, SwapRule, GLOBAL_SWAP_MANAGER,
};

// Re-export from gateway
pub use gateway::{
    AttemptPhase, CallEvent, CallObserver, CapabilityCoordinator, CapabilityIntent,
    CoordinationOutcome, CoordinationRequest, GatewayV2, SubGrid, SubGridHealth,
    AgentRoutingTable, ProviderProfile, ProviderProfileManager,
    CapabilityRouter,
    InferenceRouter, RouterConfig,
};
pub use gateway::universal_adapter::{
    UniversalAdapter, ModelConfig, ModelCapabilities, FormatConverter,
    OpenAiConverter, AnthropicConverter, GeminiConverter,
    UnifiedRequest, UnifiedResponse, ToolCall,
};
