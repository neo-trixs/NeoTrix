//! Stealth Manager — JEPA-inspired undetectable operation patterns.
//!
//! Manages behavior profiles for stealthy operation: timing jitter,
//! request randomization, noise injection, and profile rotation.
//! Tracks per-profile success rates and provides risk assessment.

pub mod types;
pub mod manager;

pub use types::{StealthProfile, BehaviorPattern, PatternType, StealthReport};
pub use manager::StealthManager;
