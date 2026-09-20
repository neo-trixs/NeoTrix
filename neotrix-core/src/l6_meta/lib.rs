//! L6 Meta-Cognition Layer
//!
//! Meta-cognitive systems — coordination, memory, healing, evolution,
//! self-modeling, and agent identity management.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │  Coordination (meta + governance)                   │
//! ├─────────────────────────────────────────────────────┤
//! │  Memory (nexus + KB primitives)                     │
//! ├─────────────────────────────────────────────────────┤
//! │  Healing (repair + safety monitor)                  │
//! ├─────────────────────────────────────────────────────┤
//! │  Evolution (self-model + identity + emergence)      │
//! └─────────────────────────────────────────────────────┘
//! ```

// ── Core Subsystems ────────────────────────────────────────────────────────
pub mod coordination;
pub mod memory;
pub mod healing;
pub mod evolution;
pub mod l1_facade;

pub mod nt_meta;
pub use coordination as nt_governance;
pub use healing as nt_repair;
pub mod nt_nexus;

// ── Runtime & Evaluation ───────────────────────────────────────────────────
pub mod runtime_monitor;
pub use runtime_monitor::RuntimeMonitor;

pub mod evolving_evaluator;
pub use evolving_evaluator::EvolvingEvaluator;

// ── Self-Model ─────────────────────────────────────────────────────────────
pub mod nt_core_self_model;
pub mod self_model_unified;

// ── Core Self & Awareness ──────────────────────────────────────────────────
pub mod nt_core_self;
pub mod nt_core_self_constitution;
pub mod nt_core_aware;
pub mod nt_core_observer;
pub mod nt_core_observer_error;

// ── Knowledge Base ─────────────────────────────────────────────────────────
pub mod nt_core_kb_primitives;
pub mod nt_core_kb_types;
pub mod nt_core_memory_asset;
pub mod nt_core_absorb;
pub mod nt_core_iter;

// ── Scheduling & Review ───────────────────────────────────────────────────
pub mod nt_core_scheduler;
pub mod nt_core_self_review;
pub mod nt_core_capability;

// ── Agent Identity ─────────────────────────────────────────────────────────
pub mod nt_agent_identity;
pub mod nt_agent_gallery;

// ── Safety & Emergence (absorbed from neotrix-sim) ────────────────────────
pub mod nt_safety_monitor;
pub mod nt_emergence_detector;
