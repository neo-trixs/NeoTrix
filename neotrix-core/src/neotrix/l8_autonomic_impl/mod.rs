//! L8 Autonomic Implementation Layer
//!
//! Self-evolution: SEAL pipeline, sleep, aging, autofix, background maintenance.
//! Runs below consciousness — no GWT (L5) required.

pub mod nt_mind;
pub mod nt_mind_awakening;
pub mod nt_mind_background_loop;
pub mod nt_mind_ingestion;

pub mod nt_mind_autofixer;
pub mod nt_mind_background_config;
pub mod nt_mind_benchmark;
pub mod nt_mind_cleanup;
// nt_mind_consciousness_gold_standard and nt_mind_consciousness_monitor
// have been migrated to L9 (l9_transcendent_impl/).
// Re-exported from neotrix/mod.rs via l9_transcendent_impl.
pub mod nt_mind_distiller;
pub mod nt_mind_evolution_daemon;
pub mod nt_mind_evolution_loop;
pub mod nt_mind_scheduler;
pub mod nt_mind_self_diagnose;
#[cfg(feature = "research")]
pub mod nt_mind_topic_aggregator;
pub mod nt_mind_knowledge_pipeline;
pub mod nt_mind_hook;
pub mod nt_mind_memory;
pub mod nt_mind_skill_engine;
#[cfg(feature = "research")]
pub mod nt_mind_absorption_registry;
