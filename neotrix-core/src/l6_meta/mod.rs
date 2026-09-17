//! L6 Meta-Cognition Layer
//!
//! 4 目录架构:
//!   coordination/ — 协调类 (meta + governance)
//!   memory/       — 记忆类 (memory + nexus)
//!   healing/      — 修复类 (repair)
//!   evolution/    — 进化类 (mind + act)

pub mod coordination;
pub mod memory;
pub mod healing;
pub mod evolution;
/// L1 Facade
pub mod l1_facade;

pub mod nt_meta;
pub use coordination as nt_governance;
pub use healing as nt_repair;
pub mod nt_nexus;

pub mod runtime_monitor;
pub use runtime_monitor::RuntimeMonitor;

pub mod evolving_evaluator;
pub use evolving_evaluator::EvolvingEvaluator;

/// Self-model
pub mod nt_core_self_model;

// 从 core/ 迁移的 L6 模块
pub mod nt_core_self;
pub mod nt_core_self_constitution;
pub mod nt_core_aware;
pub mod nt_core_observer;
pub mod nt_core_observer_error;
pub mod nt_core_kb_primitives;
pub mod nt_core_kb_types;
pub mod nt_core_memory_asset;
pub mod nt_core_absorb;
pub mod nt_core_iter;
pub mod nt_core_scheduler;
pub mod nt_core_self_review;
pub mod nt_core_capability;
