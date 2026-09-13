//! LLM Provider 模块
//!
//! 统一架构: Provider Pool + IP Proxy Pool + Gateway
//!
//! 功能分组:
//!   - providers/  : LLM Provider 实现 (Anthropic / Gemini / Ollama / OpenAI / LLaMA)
//!   - gateway/    : 统一网关 (路由 / 执行 / 韧性 / 可观测性)
//!   - catalog/    : Provider 目录 / 发现 / 注册
//!   - common/     : 公共类型 / 工厂 / 隐私守卫 / 生成分类
//!   - health/     : 健康检查 / 限流 / 熔断 / 上下文预算
//!   - pool/       : 连接池管理 (Provider Pool / Account Pool / Free Pool)
//!   - routing/    : 故障转移 / Provider 切换
#![deny(clippy::unwrap_used)]

// ── 子模块声明 ──────────────────────────────────────────────
// Provider 实现
pub mod anthropic;
pub mod gemini;
pub mod ollama;
pub mod openai;
pub mod llama;

// 网关
pub mod gateway;

// 基础设施
pub mod catalog;
pub mod common;
pub mod health;
pub mod pool;
pub mod routing;

// ── Provider 实现 ───────────────────────────────────────────
pub use anthropic::AnthropicProvider;
pub use gemini::GeminiProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use llama::{
    global_manager, find_executable, scan_models, select_best_model, compute_optimal_config,
    GgufModel, HardwareProfile, LlamaProcessManager, LlamaServerConfig,
};

// ── 网关核心 ────────────────────────────────────────────────
pub use gateway::{
    AttemptPhase, CallEvent, CallObserver, CapabilityCoordinator, CapabilityIntent,
    CoordinationOutcome, CoordinationRequest, GatewayV2, SubGrid, SubGridHealth,
    AgentRoutingTable, ProviderProfile, ProviderProfileManager,
    CapabilityRouter,
    InferenceRouter, RouterConfig,
};
pub use gateway::execution::universal_adapter::*;
pub use gateway::execution::unified_inference::*;
pub use gateway::routing::search_router::*;

// ── 目录 / 发现 / 注册 ─────────────────────────────────────
pub use catalog::{
    find_by_capabilities, find_by_capabilities_in_category,
    keyless_providers, lookup_provider, providers_by_category, providers_with_key,
    CommunicationProfile, ProviderCapabilities, ProviderCategory, ProviderInfo, PROVIDER_CATALOG,
};
pub use catalog::registry::ProviderRegistry;
pub use catalog::{discovery, free_catalog, provider_catalog, registry};
pub use catalog::gateway_adapter::GatewayV2Adapter;

// ── 公共类型 / 工厂 ────────────────────────────────────────
pub use common::{
    create_gateway, create_provider, create_provider_from_type, LlmProviderType, ProviderConfig,
    Classification, Complexity, Domain, GenerationAnalytics, GenerationClassifier,
    GenerationRecord, TaskType,
};
pub use common::types::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Message, Role, Tool,
    ToolCallFunction, ToolCallInfo, Usage,
};
pub use health::context_budget::{apply_context_budget, estimate_messages_tokens, estimate_tokens};
pub use common::{factory, generation_classifier, privacy_guard, types};

// ── 健康 / 限流 / 熔断 ─────────────────────────────────────
pub use health::circuit_breaker::CircuitBreaker;
pub use health::rate_limiter::{RateLimiter, TokenBucket, AdaptivePacer, BrainTier, TieredSemaphore};
pub use health::{circuit_breaker, compaction, context_budget, rate_limiter, rate_profiles};

// ── 连接池 ──────────────────────────────────────────────────
pub use pool::account_pool::{
    AccountHealth, AccountLease, AccountPool, AccountPoolConfig, AccountPoolError,
};
pub use pool::free_pool::FreePool;
pub use pool::provider_pool::{global_provider_pool, PoolEntry, ProviderPool};
pub use pool::{account_pool, free_pool, provider_pool};

// ── 故障转移 / Provider 切换 ────────────────────────────────
pub use routing::failover_history::{
    clear_history, failover_history, record_failover, report as failover_report, total_failovers,
    FailoverEvent, FailoverHistory,
};
pub use routing::provider_swap::{
    ProviderHealth, ProviderHealthSummary, ProviderSwapManager, SwapRule, GLOBAL_SWAP_MANAGER,
};
pub use routing::provider_swap;
