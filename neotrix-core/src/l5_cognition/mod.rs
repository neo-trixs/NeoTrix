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
/// Unified Model Interface — 通用模型适配框架 (orphan wired T45+1: 含 ModelPreferences 三档位)
pub mod nt_core_model_unified;
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
pub use neotrix_consciousness::consciousness_tree;
pub use neotrix_consciousness::cad_consciousness;
pub use neotrix_consciousness::gwt;
pub use neotrix_consciousness::context;
pub use neotrix_consciousness::context_engine;
pub use neotrix_consciousness::echo_terminal;
pub use neotrix_consciousness::panic_recovery;
pub use neotrix_consciousness::second_brain;
pub use neotrix_consciousness::source_hierarchy;
pub use neotrix_consciousness::cognitive_load;
pub use neotrix_consciousness::bubble_wall;
pub use neotrix_consciousness::vsa_tag;
pub use neotrix_consciousness::consciousness_crystal;
pub use neotrix_consciousness::consciousness_subsystem;
pub use neotrix_consciousness::iit_phi;
pub use neotrix_consciousness::kernel_types as consciousness_kernel_types;
pub use neotrix_consciousness::state as consciousness_state;

// ── neotrix-reasoning ───────────────────────────────────────────────────────
pub use neotrix_reasoning::reasoning_core;
pub use neotrix_reasoning::kron;
pub use neotrix_reasoning::quantum_fusion;
pub use neotrix_reasoning::resonator;
pub use neotrix_reasoning::aura;
pub use neotrix_reasoning::arch_fitness;
pub use neotrix_reasoning::scoring;
pub use neotrix_reasoning::sae;
pub use neotrix_reasoning::credit;
pub use neotrix_reasoning::coordination;
pub use neotrix_reasoning::paradigm;
pub use neotrix_reasoning::meaning;
pub use neotrix_reasoning::narrative;
pub use neotrix_reasoning::kernel_types as reasoning_kernel_types;
pub use neotrix_reasoning::evolution;
pub use neotrix_reasoning::policy;
pub use neotrix_reasoning::rule_memory;
pub use neotrix_reasoning::cot;
pub use neotrix_reasoning::gate as reasoning_gate;
pub use neotrix_reasoning::prm;

// ── neotrix-gateway ─────────────────────────────────────────────────────────
pub use neotrix_gateway::model_gateway;
pub use neotrix_gateway::model_router;
pub use neotrix_gateway::skill_registry;
pub use neotrix_gateway::semantic_router;
pub use neotrix_gateway::hybrid_search;
pub use neotrix_gateway::byoa;
pub use neotrix_gateway::agents_md;
pub use neotrix_gateway::hive;
pub use neotrix_gateway::circuit_breaker;
pub use neotrix_gateway::capability;
pub use neotrix_gateway::knowledge_mgmt;
pub use neotrix_gateway::skill_engine;
pub use neotrix_gateway::mind_modules;
pub use neotrix_gateway::failure_taxonomy;
pub use neotrix_gateway::context_mgmt;
pub use neotrix_gateway::gate as gateway_gate;

// ── neotrix-multi-agent ─────────────────────────────────────────────────────
pub use neotrix_multi_agent::multi_agent;
pub use neotrix_multi_agent::parallel;
pub use neotrix_multi_agent::background_loop;
pub use neotrix_multi_agent::meta_panel;
pub use neotrix_multi_agent::skill_chain;
pub use neotrix_multi_agent::element_bus;
pub use neotrix_multi_agent::infrastructure;
pub use neotrix_multi_agent::experience_tree;
pub use neotrix_multi_agent::self_improvement;
pub use neotrix_multi_agent::dual_track;
// Migrated from L5 cognition
pub use neotrix_multi_agent::coordinator;
pub use neotrix_multi_agent::coordination as multi_agent_coordination;
pub use neotrix_multi_agent::hive as multi_agent_hive;
pub use neotrix_multi_agent::god_agent;
pub use neotrix_multi_agent::skill_registry as migrated_skill_registry;
