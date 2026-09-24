//! NeoTrix Gateway Crate
//!
//! L5 Gateway systems — model routing, skill registry, search,
//! BYOA (Bring Your Own Agent), capability system, Hive coordination,
//! gate control, context management, and awareness core bridging.

#![forbid(unsafe_code)]

pub mod model_gateway;
pub mod model_router;
pub mod skill_registry;
pub mod semantic_router;
pub mod hybrid_search;
pub mod byoa;
pub mod agents_md;
pub mod hive;
pub mod circuit_breaker;
pub mod capability;
pub mod knowledge_mgmt;
pub mod skill_engine;
pub mod mind_modules;
pub mod failure_taxonomy;
pub mod context_mgmt;
pub mod gate;

/// Re-export decision engine for structured decision-making
#[cfg(feature = "decision-engine")]
pub use neotrix_decision_engine as decision_engine;
