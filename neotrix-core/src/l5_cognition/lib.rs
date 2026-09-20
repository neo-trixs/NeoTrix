//! L5 Cognition Layer
//!
//! Core cognitive systems — consciousness, reasoning, strategy, evolution,
//! gateway routing, and multi-agent coordination.
//!
//! # Architecture
//!
//! ```text
//! Facade (nt_cognition_facade)
//!   ├── Consciousness Core ←→ GWT ←→ Context
//!   ├── Reasoning (CoT, SEAL, Quantum Fusion)
//!   ├── Strategy (Goals, Plans, Policies)
//!   ├── Evolution (Scoring, SAE, Architecture Fitness)
//!   └── Gateway (Model Router, Skill Registry, BYOA)
//! ```

// ── Facade & Core ──────────────────────────────────────────────────────────
pub mod traits;
pub mod nt_core;
pub mod nt_mind;
pub mod nt_cognition_facade;
pub mod nt_council;
pub mod l1_facade;

// ── Consciousness ──────────────────────────────────────────────────────────
pub mod nt_core_consciousness_core;
pub mod nt_core_consciousness_tree;
pub mod nt_core_consciousness;
pub mod nt_core_cad_consciousness;
pub mod nt_core_gwt;
pub mod nt_core_context;

// ── Reasoning ──────────────────────────────────────────────────────────────
pub mod nt_core_cot_generator;
pub mod nt_core_gate;
pub mod nt_core_prm;
pub mod nt_core_dispatch;

// ── Strategy ───────────────────────────────────────────────────────────────
pub mod nt_goal;
pub mod nt_core_plan;
pub mod nt_core_policy;
pub mod nt_core_credit;
pub mod nt_core_rule_memory;
pub mod nt_core_context_engine;
pub mod nt_core_agents_md;

// ── Evolution ──────────────────────────────────────────────────────────────
pub mod nt_core_quantum_fusion;
pub mod nt_core_scoring_substrate;
pub mod nt_core_sae;
pub mod nt_core_sae_bridge;
pub mod nt_core_arch_diagram;
pub mod nt_core_arch_fitness;
pub mod nt_core_aura;

// ── Gateway ────────────────────────────────────────────────────────────────
pub mod nt_core_model_gateway;
pub mod nt_core_semantic_router;
pub mod nt_core_byoa;
pub mod nt_core_agent_circuit_breaker;
pub mod nt_core_model_router;
pub mod nt_core_model_skills;
pub mod nt_core_hybrid_search;
pub mod nt_core_second_brain;
pub mod nt_core_orchestration_failure_taxonomy;
pub mod nt_core_panic_recovery;

// ── Math (re-exports from L0) ─────────────────────────────────────────────
pub use crate::l0_substrate::nt_core_math;
pub use crate::l0_substrate::nt_core_hex;
pub use crate::l0_substrate::nt_core_shared_types;
pub mod nt_core_walsh;
pub mod nt_core_td;
pub mod nt_core_ttc;
pub mod nt_core_trajectory_compress;

// ── Absorbed ───────────────────────────────────────────────────────────────
pub mod nt_resonator_network;
pub mod nt_decision_engine;
pub mod nt_codegen;

// ── Types ──────────────────────────────────────────────────────────────────
pub mod nt_core_narrative_types;
pub mod nt_core_state;
pub mod nt_core_meaning;
pub mod nt_core_paradigm;

// ── Agent ──────────────────────────────────────────────────────────────────
pub mod nt_agent;

// ============================================================================
// Re-exports from migrated L5 crates
// ============================================================================

// neotrix-consciousness
pub use neotrix_consciousness::consciousness_core;
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

// neotrix-reasoning
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

// neotrix-gateway
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

// neotrix-multi-agent
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
pub use neotrix_multi_agent::coordinator;
pub use neotrix_multi_agent::coordination as multi_agent_coordination;
pub use neotrix_multi_agent::hive as multi_agent_hive;
pub use neotrix_multi_agent::god_agent;
pub use neotrix_multi_agent::skill_registry as migrated_skill_registry;
