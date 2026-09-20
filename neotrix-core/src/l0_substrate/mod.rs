//! L0 Substrate Layer - Foundation modules

pub mod nt_core_error;
pub mod nt_core_hot_data;
/// Shared time utilities (now_secs, now_millis)
pub mod nt_core_time;

/// Speculative Decoding Engine (GPT-5.6 pattern)
pub mod nt_core_speculative_decoding;

/// Prompt Caching System (GPT-5.6 pattern)
pub mod nt_core_prompt_cache;

/// SelfTest trait — 跨模块共享的自测试接口定义 (下沉自 L6)
pub mod nt_core_self_test;

/// 统一能力接口类型定义 (下沉自 L6 nt_core_capability)
pub mod nt_core_capability_types;

/// Math utilities (cosine similarity, URL normalization, Hamming distance)
pub mod nt_core_math;

/// E₈ × 64 state-space reasoning model (hexagram→reasoning mode mapping)
pub mod nt_core_hex;

/// Shared types (Modality, E8VsaEmbedding) — broken from circular deps
pub mod nt_core_shared_types;

/// KB 存储原语 — 消除低层→L6 反向依赖 (re-export from l6_meta)
pub mod nt_core_kb_primitives;

/// 内部状态统一落 KB 构造 (save/load/delete) — 从 l5_cognition 下沉, 消除 L3→L5 违规
pub mod nt_core_state;

/// NT-MEMORY 四态记忆资产模型 — 消除低层→L6 反向依赖 (re-export from l6_meta)
pub mod nt_core_memory_asset;

/// Cross-layer shared types — L5↔L6 abstraction via L0 re-exports.
pub mod nt_core_cross_layer;

/// Platform initialization, pipeline, agents — migrated from core/
pub mod nt_core_platform;

/// DI container — dependency injection
pub mod nt_core_di;

/// Architecture axiom tree
pub mod nt_core_axiom_tree;

/// Event system
pub mod nt_core_event;

/// Core traits
pub mod nt_core_traits;

/// L0 Substrate shared types — foundational types moved from L1/L2
pub mod nt_core_substrate_types;

/// Span tracking
pub mod nt_core_span;

/// Telemetry store
pub mod nt_core_telemetry;

/// Schema watchdog
pub mod nt_core_schema_watchdog;

/// Workspace management
pub mod nt_core_ws;

/// Semantic cache
pub mod nt_core_cache;

/// Test selection engine
pub mod nt_core_qtest;

/// Answer engine
pub mod nt_core_answer_engine;

// ============================================================================
// Absorbed — 从 nt-world-sim / neotrix-sim 吸收
// ============================================================================
/// ECS Foundation — absorbed from nt-world-sim (archetype-based, parallel scheduler)
pub mod nt_ecs;
/// Multi-Timescale Tick Schedule — absorbed from neotrix-sim (Reflex/Fast/Medium/Slow/Background)
pub mod nt_tick_schedule;
