// ============================================================================
// Facade & Core — 层门面与核心
// ============================================================================
pub mod traits;
pub mod nt_core;
pub mod nt_mind;
/// L5 Cognition Facade — 唯一对外门面 (Qingjian 模式)
pub mod nt_cognition_facade;
/// Multi-perspective deliberation council (council-of-high-intelligence inspired)
pub mod nt_council;
/// L1 Facade — consolidated re-export facade
pub mod l1_facade;
/// L5 → L0 error conversions (moved from l0_substrate to respect L0 ← L5 direction)
pub mod error_conversions;

// ============================================================================
// Consciousness — 意识核心与 GWT
// ============================================================================
pub mod nt_core_consciousness_tree;
pub mod nt_core_consciousness;
pub mod nt_core_cad_consciousness;
pub mod nt_core_gwt;
pub mod nt_core_context;

// ============================================================================
// Reasoning — 推理与思维链
// ============================================================================
pub mod nt_core_cot_generator;
pub mod nt_core_gate;
pub mod nt_core_prm;
pub mod nt_core_dispatch;
/// EVO-03 DSPy 声明式自优化层 (Signature/Metric/BootstrapCompiler/CompiledPrompt 纯逻辑)
pub mod nt_dspy;

// ============================================================================
// Strategy — 目标/计划/策略/规则
// ============================================================================
/// Goal management
pub mod nt_goal;
pub mod nt_core_plan;
pub mod nt_core_policy;
pub mod nt_core_credit;
pub mod nt_core_rule_memory;
/// Context Engine
pub mod nt_core_context_engine;
/// AGENTS.md Discovery System
pub mod nt_core_agents_md;

// ============================================================================
// Evolution — 进化/评分/架构自省
// ============================================================================
pub mod nt_core_quantum_fusion;
pub mod nt_core_scoring_substrate;
pub mod nt_core_sae;
pub mod nt_core_sae_bridge;
pub mod nt_core_arch_diagram;
pub mod nt_core_arch_fitness;
pub mod nt_core_aura;

// ============================================================================
// Gateway — 模型网关/路由/多Agent/技能注册
// ============================================================================
/// Model Gateway — 统一模型网关 (cost-aware routing + fallback chain)
pub mod nt_core_model_gateway;
/// Semantic Router — 语义路由 (confidence-based dispatch)
pub mod nt_core_semantic_router;
/// BYOA — Bring Your Own Agent (absorbed from cumora)
pub mod nt_core_byoa;
/// Agent Circuit Breaker — steer→constrain→stop 三级行为控制 (moved from L3 shield)
pub mod nt_core_agent_circuit_breaker;
/// Real-time Model Router
pub mod nt_core_model_router;
/// Skill Registry
pub mod nt_core_model_skills;
/// EVO-06 技能路由单源真理＋三件套评审（RouteTable/Manifest 纯逻辑）
pub mod nt_skill_route;
/// Hybrid Code Search Retriever
pub mod nt_core_hybrid_search;
pub mod nt_core_second_brain;
pub mod nt_core_orchestration_failure_taxonomy;
pub mod nt_core_panic_recovery;

// ============================================================================
// Math — 数学基础 (HyperCube/E8/信号处理)
// ============================================================================
pub use crate::l0_substrate::nt_core_math;
pub use crate::l0_substrate::nt_core_hex;
pub use crate::l0_substrate::nt_core_shared_types;
pub mod nt_core_walsh;
pub mod nt_core_td;
pub mod nt_core_ttc;
pub mod nt_core_trajectory_compress;

// ============================================================================
// Absorbed — 从 guard_core / neotrix-sim / nt-world-sim 吸收
// ============================================================================
/// VSA Resonator Network — absorbed from guard_core (Frady et al., 2020)
pub mod nt_resonator_network;
/// Multi-Layer Decision Engine — absorbed from neotrix-sim (Stimulus→GOAP→BT→Utility)
pub mod nt_decision_engine;
/// Multi-Engine Code Generation — absorbed from nt-world-sim
pub mod nt_codegen;

// ============================================================================
// Types — 共享类型与状态模型
// ============================================================================
pub mod nt_core_narrative_types;
pub use crate::l0_substrate::nt_core_state;
pub mod nt_core_meaning;
pub mod nt_core_paradigm;

// ============================================================================
// Agent — 执行模式 / ODE loop
// ============================================================================
pub mod nt_agent;

// ============================================================================
// Re-exports from new L5 crates
// ============================================================================
// Modules below have been migrated to standalone crates. The re-exports
// preserve backwards-compatible paths (e.g. `crate::l5_cognition::consciousness_core`).

// ── neotrix-consciousness ───────────────────────────────────────────────────
// T39-A4：空占位 `neotrix_consciousness::consciousness_core`（0 行）腾名——
// 本地 E2 目录正式接管 `consciousness_core` 之名，消费者直指 E2 实现。
pub mod consciousness_core;
pub use neotrix_consciousness::echo_terminal;
pub use neotrix_consciousness::source_hierarchy;
pub use neotrix_consciousness::cognitive_load;
pub use neotrix_consciousness::bubble_wall;
pub use neotrix_consciousness::vsa_tag;

// ── neotrix-reasoning ───────────────────────────────────────────────────────
pub use neotrix_reasoning::reasoning_core;
pub use neotrix_reasoning::kron;
pub use neotrix_reasoning::kernel_types as reasoning_kernel_types;

// ── neotrix-gateway ─────────────────────────────────────────────────────────
pub use neotrix_gateway::model_gateway;
pub use neotrix_gateway::model_router;
pub use neotrix_gateway::skill_registry;  // ← SKILL.md 正典 (2026-09-27: multi-agent 的 600 行副本零消费者已删)
pub use neotrix_gateway::hybrid_search;
pub use neotrix_gateway::hive;
pub use neotrix_gateway::mind_modules;
pub use neotrix_gateway::context_mgmt;
pub use neotrix_gateway::gate as gateway_gate;

// ── neotrix-multi-agent ─────────────────────────────────────────────────────
pub use neotrix_multi_agent::multi_agent;
// Migrated from L5 cognition
pub use neotrix_multi_agent::coordinator;
pub use neotrix_multi_agent::coordination as multi_agent_coordination;
pub use neotrix_multi_agent::hive as multi_agent_hive;
pub use neotrix_multi_agent::god_agent;
