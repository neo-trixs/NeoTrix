#![forbid(unsafe_code)]

//! Decay-Based Forgetting (R-P121)
//!
//! Memory entries lose retention over time via configurable decay curves.
//! The pruner removes entries that fall below a retention threshold.

pub mod config;
pub mod curves;
pub mod pruner;
pub mod salience;

pub use config::DecayConfig;
pub use curves::{ebbinghaus_retention, exponential_decay, DecayCurve, EbbinghausDecay, ExponentialDecay};
pub use pruner::{MemoryPruner, PruneResult, DecayPruneConfig, compute_retention, prune_by_retention};
pub use salience::SalienceCalculator;
