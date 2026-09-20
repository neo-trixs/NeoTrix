//! L1 Action Layer - Memory Modules

/// 共享工具函数 — 消除跨模块重复
pub mod shared_utils;

pub mod nt_trade_product_spec;
pub mod nt_memory_lead;
pub use nt_memory_lead::{LeadManager, Lead, LeadSource, LeadStage, LeadQuality, LeadScorer, Interaction, InteractionType};

pub mod nt_memory_babeldoc;
pub mod nt_memory_graphify;
pub mod nt_memory_leann_store;
pub mod nt_memory_pdf_math_translate;
pub mod nt_memory_yopedia;
pub mod nt_memory_historian;
pub mod nt_memory_kb;
pub mod nt_memory_openknowledge;
pub mod selective_memory;
pub mod addressable_store;

/// Memory Consolidation — short-term promotion, working memory, TTL cache (R-P117, R-P118, R-MEM09).
pub mod consolidation;

/// Memory Distillation — merge similar, extract key facts, cross-session persistence (R-P117, R-P118, R-MEM09).
pub mod distillation;

// Context-as-Filesystem — 基于 OpenViking 模式
pub mod context_fs;

/// Trinity Memory — 基于 TencentDB Agent Memory 模式
pub mod trinity;

/// 三类记忆 — 基于 Grok Build 模式
pub mod memory_types;

/// Evidence Ledger — 证据账本 (会话账本 + 反幻觉闸门)
pub mod evidence_ledger;

/// Typed Semantic Memory — typed memory estates with conflict resolution,
/// policy-driven forgetting, and multi-tier storage.
pub mod typed_memory;

/// Four-Layer Memory + Temporal Reasoning — 吸收自 hirn
pub mod nt_memory_typed;
pub mod vector_index;

/// Entity Linking — extraction, dedup, and indexing of named entities (R-P117 ADD-only).
pub mod entity_linking;

// Re-exports for cross-module integration
pub use addressable_store::AddressableStore;
pub use context_fs::{ContextFileSystem, ContextNode, ContextNodeType};
pub use trinity::TrinityMemory;
pub use typed_memory::{MemoryEstate, TypedMemoryEntry, ConflictResolver, PolicyDrivenForgetting, MemoryMultitier, TypedMemoryStore};
pub use vector_index::VectorIndex;
pub use entity_linking::{Entity, EntityType, EntityIndex, EntityLinker, extract_entities, link_text};

/// Five-Tier Memory Cascade — sensory → working → short-term → episodic → long-term pipeline.
pub mod cascade;

// Distillation re-exports
pub use distillation::{
    DistilledMemory, MemoryCompressor, MemoryDistiller, RelatedMemory,
    SessionInfo, SessionPersistence, CrossSessionBridge,
};

// Cascade re-exports
pub use cascade::{
    CascadeConfig, CascadeTickStats, CascadeRecallResult, MemoryCascade,
    Observation, SensoryBuffer, WorkingItem, WorkingMemory,
    ShortTermEntry, ShortTermStore, Episode, EpisodicStore,
    Rule, LongTermStore,
};
