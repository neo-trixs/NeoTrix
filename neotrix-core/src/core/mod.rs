// core/ — Shared foundational modules used by multiple layers
// Top-level nt_core_* modules are the canonical path for new code.

#![forbid(unsafe_code)]

// Shared types — breaks circular deps between e8 ↔ gwt
pub mod nt_core_shared_types;

// Old layer directories (backward-compatible import paths)
// l5_consciousness → l5_cognition (migrated)
// l6_self → l6_meta (migrated)
pub use crate::l5_cognition::nt_core::capability as l7_capability;
// l8_autonomic → l6_meta/healing (backward-compat re-export)
pub mod l8_autonomic;
// Consciousness modules (lived in l5_cognition, referenced via crate::core::*)
pub use crate::l5_cognition::nt_core_consciousness_core;
pub use crate::l5_cognition::nt_core_consciousness_tree;

// L0 — Substrate
pub mod nt_core_error;
pub mod nt_core_hot_data;

// L3 — Memory
pub mod nt_core_bank;

// L4 — Cognition
pub mod nt_core_rule_memory;
pub mod nt_core_meaning;
pub mod nt_core_paradigm;
pub mod nt_core_aura;
pub mod nt_core_cot_generator;
pub mod nt_core_credit;
// nt_core_crt migrated → l5_cognition/nt_core/nt_crt
pub use crate::l5_cognition::nt_core::nt_crt as nt_core_crt;
pub mod nt_core_math;
pub mod nt_core_graph;
pub mod nt_core_guard_chain;
pub mod nt_core_kron;
pub mod nt_core_walsh;
pub mod nt_core_memory_budget;
pub mod nt_core_e8;
pub mod nt_core_e8_predictor;
pub mod nt_core_e8_vsa;
// nt_core_forecast migrated → l5_cognition/nt_core/nt_forecast
pub use crate::l5_cognition::nt_core::nt_forecast as nt_core_forecast;
pub mod nt_core_gate;
pub mod nt_core_code_search;
pub mod nt_core_hex;
// nt_core_iit_phi migrated → l5_cognition/nt_core/nt_iit_phi
pub use crate::l5_cognition::nt_core::nt_iit_phi as nt_core_iit_phi;
pub mod nt_core_kernel_types;
pub mod nt_core_plan;
pub mod nt_core_policy;
pub mod nt_core_prm;
pub mod nt_core_reasoning;
pub mod nt_core_sae;
pub mod nt_core_sae_bridge;
pub mod nt_core_task_dispatcher;
pub mod nt_core_td;
pub mod nt_core_trajectory_compress;
pub mod nt_core_ttc;
pub mod nt_core_tlc_correction;
pub mod nt_core_narrative_types;

// L5 — Consciousness
pub mod nt_core_context;
pub mod nt_core_dispatch;
pub mod nt_core_gwt;
pub mod nt_core_heartbeat;
pub mod nt_core_consciousness;
pub mod nt_core_echo_terminal;

// L6 — Self
pub mod nt_core_aware;
pub mod nt_core_self;
pub mod nt_core_self_constitution;
pub mod nt_core_kb_primitives;
pub mod nt_core_kb_types;
pub mod nt_core_memory_asset;
pub mod nt_core_arch_diagram;
pub mod nt_core_state;

// L7 — Capability
pub mod nt_core_model_skills;
pub mod nt_core_capability;

// L8 — Autonomic
pub mod nt_core_absorb;
pub mod nt_core_iter;
pub mod nt_core_scheduler;

// L9 — Transcendent
// nt_core_meta migrated → l5_cognition/nt_core/nt_meta
pub use crate::l5_cognition::nt_core::nt_meta as nt_core_meta;
pub mod nt_core_observer;
pub mod nt_core_observer_error;

// Cross-layer infrastructure
pub mod nt_core_di;
pub mod nt_core_axiom_tree;
pub mod nt_core_cap;
pub use crate::cli::nt_conn as nt_core_conn;
pub mod nt_core_event;
pub use nt_core_event::CoreEvent;
pub use crate::neotrix::nt_core_event_bus::EventBus;
pub mod nt_core_event_bus;
pub mod nt_core_retrieval;
pub use crate::cli::nt_router as nt_core_router;
pub mod nt_core_self_review;
pub mod nt_core_traits;
pub mod nt_core_ws;
pub mod nt_core_cache;
pub mod nt_core_span;
pub mod nt_core_answer_engine;
pub mod nt_core_arch_fitness;
pub mod nt_core_qtest;
pub mod nt_core_quantum_fusion;
pub mod nt_core_resource_pool;
pub mod nt_core_panic_recovery;
pub mod nt_core_schema_watchdog;
pub mod nt_core_scoring_substrate;
pub mod nt_core_second_brain;
pub mod nt_core_orchestration_failure_taxonomy;
pub mod nt_core_cad_consciousness;
pub mod nt_core_platform;
pub mod nt_core_simulate_engine;
pub mod nt_core_knowledge;
pub mod nt_core_hcube;
pub mod nt_core_vector_store;
pub mod nt_core_sense;
pub mod nt_core_llm;
pub mod nt_core_edit;
pub mod nt_core_embed;
pub mod nt_core_harness;
// nt_core_state_substrate migrated → l5_cognition/nt_core/nt_state_substrate
pub use crate::l5_cognition::nt_core::nt_state_substrate as nt_core_state_substrate;
pub use crate::cli::nt_subagent as nt_core_subagent;
pub mod nt_core_telemetry;
#[cfg(test)]
pub mod kani_proofs;

// ════════════════════════════════════════════════════════════════
// 向后兼容重导出 — 旧路径迁移后保留 import 兼容性
// ════════════════════════════════════════════════════════════════

// crate::core::l2_perception → crate::l2_perception
pub use crate::l2_perception;
// crate::core::l1_body → crate::l3_embodiment (旧名)
pub use crate::l3_embodiment as l1_body;
// crate::core::l0_substrate → crate::core (自身，作为占位)
// (l0_substrate 旧代码直接引用 core 下的底层类型，无需额外重导出)

// 从 nt_core_vector_store 重导出
pub use nt_core_vector_store::{DistanceMetric, VectorSearchResult, VectorRecord, create_default_store, create_store, StoreBackend};

// 从 nt_core_sense 重导出
pub use nt_core_sense::{Sensor, SensorSample};

// 从 nt_core_knowledge 重导出共享类型
pub use nt_core_knowledge::types::{KnowledgeSource, RewardSource, TaskType};

// 从 nt_core_sae 重导出 SAE 类型
pub use nt_core_sae::{SaeFeature, SparseAutoencoder, SAE_INPUT_DIM};

// 从 nt_core_hex 重导出推理状态类型
pub use nt_core_hex::{FullReasoningState, MetaState};

// 从 l5_cognition capability 重导出 CapabilityVector
pub use crate::l5_cognition::nt_core::capability::types::CapabilityVector;

// 从 nt_core_shared_types 重导出
pub use nt_core_shared_types::*;

// 从 nt_core_error 重导出 NeoTrixError
pub use nt_core_error::NeoTrixError;

// 从 nt_core_bank 重导出 ReasoningBank
pub use nt_core_bank::bank::ReasoningBank;

// 从 nt_core_hex 重导出
pub use nt_core_hex::ReasoningHexagram;
pub use nt_core_hex::optimal_starting_mode;

// 从 nt_core_policy 重导出
pub use nt_core_policy::E8Policy;
pub use nt_core_policy::E8TransitionLearner;

// 从 nt_core_ws 重导出
pub use nt_core_ws::WORKSPACE_MANAGER;

// 从 l5_cognition nt_crt 重导出
pub use crate::l5_cognition::nt_core::nt_crt::{CrtTimeScale, CrtPlan};
