#![forbid(unsafe_code)]

//! # Five-Tier Memory Cascade
//!
//! A biologically-inspired memory architecture with five tiers, each optimised
//! for a specific access pattern and retention strategy:
//!
//! - **Tier 1 — SensoryBuffer**: raw observations, TTL 60s, max 100
//! - **Tier 2 — WorkingMemory**: GWT-gated, max 7 items, evict lowest attention_weight
//! - **Tier 3 — ShortTermStore**: TTL 7 days, max 1000, LRU eviction
//! - **Tier 4 — EpisodicStore**: compressed summaries, max 500
//! - **Tier 5 — LongTermStore**: distilled rules, confidence decay + reinforcement
//!
//! ## Pipeline
//!
//! ```text
//! observe → attend → promote → compress → distill
//! ```
//!
//! The `MemoryCascade` orchestrator drives this pipeline on each tick,
//! moving information through the tiers with appropriate eviction and
//! consolidation at each boundary.

pub mod sensory;
pub mod working;
pub mod short_term;
pub mod episodic;
pub mod long_term;
pub mod cascade;

pub use sensory::{Observation, SensoryBuffer};
pub use working::{WorkingItem, WorkingMemory};
pub use short_term::{ShortTermEntry, ShortTermStore};
pub use episodic::{Episode, EpisodicStore};
pub use long_term::{Rule, LongTermStore};
pub use cascade::{CascadeConfig, CascadeTickStats, CascadeRecallResult, MemoryCascade};
