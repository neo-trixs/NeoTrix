pub mod traits;
pub mod nt_core;
pub mod nt_mind;
/// L5 Cognition Facade — 唯一对外门面 (Qingjian 模式)
pub mod nt_cognition_facade;
/// Consolidated layer aliases (replaces 5 stub facades)
pub mod layer_aliases;
/// Goal management
pub mod nt_goal;
/// Consciousness modules
pub mod nt_core_consciousness_core;
pub mod nt_core_consciousness_tree;
pub mod consciousness_core;
/// Real-time Model Router
pub mod nt_core_model_router;
/// Skill Registry
pub mod nt_core_skill_registry;
/// Multi-Agent Coordinator
pub mod nt_core_multi_agent;
/// Context Engine
pub mod nt_core_context_engine;
/// AGENTS.md Discovery System
pub mod nt_core_agents_md;
/// Hybrid Code Search Retriever
pub mod nt_core_hybrid_search;
/// L1 Facade
pub mod l1_facade;
pub mod act_facade;
pub mod io_facade;
pub mod kb_facade;
pub mod l2_facade;
pub mod l3_facade;
pub mod io_skills_facade;

// 从 core/ 迁移的 L5 模块
pub mod nt_core_consciousness;
pub mod nt_core_context;
pub mod nt_core_dispatch;
pub mod nt_core_gwt;
pub mod nt_core_echo_terminal;
pub mod nt_core_reasoning;
pub mod nt_core_math;
pub mod nt_core_hex;
pub mod nt_core_gate;
pub mod nt_core_policy;
pub mod nt_core_prm;
pub mod nt_core_cot_generator;
pub mod nt_core_credit;
pub mod nt_core_rule_memory;
pub mod nt_core_meaning;
pub mod nt_core_paradigm;
pub mod nt_core_aura;
pub mod nt_core_walsh;
pub mod nt_core_kron;
pub mod nt_core_plan;
pub mod nt_core_kernel_types;
pub mod nt_core_sae;
pub mod nt_core_sae_bridge;
pub mod nt_core_state;
pub mod nt_core_narrative_types;
pub mod nt_core_td;
pub mod nt_core_trajectory_compress;
pub mod nt_core_ttc;
pub mod nt_core_quantum_fusion;
pub mod nt_core_panic_recovery;
pub mod nt_core_scoring_substrate;
pub mod nt_core_second_brain;
pub mod nt_core_orchestration_failure_taxonomy;
pub mod nt_core_cad_consciousness;
pub mod nt_core_arch_diagram;
pub mod nt_core_arch_fitness;
pub mod nt_core_model_skills;
pub mod nt_core_shared_types;
/// Hive Coordination Protocol — inbox/outbox/blackboard (absorbed from munder-difflin)
pub mod nt_core_hive;
/// BYOA — Bring Your Own Agent (absorbed from cumora)
pub mod nt_core_byoa;
/// Agent Circuit Breaker — steer→constrain→stop 三级行为控制 (moved from L3 shield)
pub mod nt_core_agent_circuit_breaker;
/// Model Gateway — 统一模型网关 (cost-aware routing + fallback chain)
pub mod nt_core_model_gateway;
/// Semantic Router — 语义路由 (confidence-based dispatch)
pub mod nt_core_semantic_router;
