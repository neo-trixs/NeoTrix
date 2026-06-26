//! # Self-Organizing Agent Teams
//!
//! Self-organization subsystem for agent teams.
//! - `evolution` — Template evolution system with KEEP/MODIFY/DEPRECATE protocol.

pub mod evolution;

pub use evolution::{AgentTemplate, KeepAction, TemplateEvolution};
