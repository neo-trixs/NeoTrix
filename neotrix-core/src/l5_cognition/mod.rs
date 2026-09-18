// ============================================================================
// Facade & Core — 层门面与核心
// ============================================================================
pub mod traits;
pub mod nt_core;
pub mod nt_mind;
/// L5 Cognition Facade — 唯一对外门面 (Qingjian 模式)
pub mod nt_cognition_facade;
/// Consolidated layer aliases (replaces 5 stub facades)
pub mod layer_aliases;
/// L1 Facade
pub mod l1_facade;
pub mod act_facade;
pub mod io_facade;
pub mod kb_facade;
pub mod l2_facade;
pub mod l3_facade;
pub mod io_skills_facade;

// ============================================================================
// Consciousness — 意识核心与 GWT
// ============================================================================
pub mod nt_core_consciousness_core;
pub mod nt_core_consciousness_tree;
pub mod nt_core_consciousness;
pub mod nt_core_cad_consciousness;
pub mod nt_core_gwt;
pub mod nt_core_context;

// ============================================================================
// Reasoning — 推理与思维链
// ============================================================================
pub mod nt_core_reasoning;
pub mod nt_core_cot_generator;
pub mod nt_core_gate;
pub mod nt_core_prm;
pub mod nt_core_echo_terminal;
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
/// Hive Coordination Protocol — inbox/outbox/blackboard (absorbed from munder-difflin)
pub mod nt_core_hive;
/// Agent Circuit Breaker — steer→constrain→stop 三级行为控制 (moved from L3 shield)
pub mod nt_core_agent_circuit_breaker;
/// Real-time Model Router
pub mod nt_core_model_router;
/// Skill Registry
pub mod nt_core_model_skills;
pub mod nt_core_skill_registry;
/// Hybrid Code Search Retriever
pub mod nt_core_hybrid_search;
/// Multi-Agent Coordinator
pub mod nt_core_multi_agent;
pub mod nt_core_second_brain;
pub mod nt_core_orchestration_failure_taxonomy;
pub mod nt_core_panic_recovery;

// ============================================================================
// Math — 数学基础 (HyperCube/E8/信号处理)
// ============================================================================
pub use crate::l0_substrate::nt_core_math;
pub use crate::l0_substrate::nt_core_hex;
pub use crate::l0_substrate::nt_core_shared_types;
pub mod nt_core_kron;
pub mod nt_core_walsh;
pub mod nt_core_td;
pub mod nt_core_ttc;
pub mod nt_core_trajectory_compress;

// ============================================================================
// Types — 共享类型与状态模型
// ============================================================================
pub mod nt_core_kernel_types;
pub mod nt_core_narrative_types;
pub mod nt_core_state;
pub mod nt_core_meaning;
pub mod nt_core_paradigm;
pub mod nt_core_coordination_principles;
