//! # Evolution — SEAL Self-Modification Pipeline
//!
//! The meta-cognitive layer that evolves mind/ and body/.
//! CANNOT modify self/ — that's constitutionally protected.
//!
//! ## Architecture
//!
//! - `evolution_task`: Task types, lifecycle, and scheduler
//! - `meta_controller`: Cross-crate evolution orchestration

pub mod evolution_task;
pub mod meta_controller;
pub mod critique_distiller;
pub mod self_harness;
