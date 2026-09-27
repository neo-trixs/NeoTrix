//! NeoTrix Multi-Agent Crate
//!
//! L5 Multi-Agent systems — coordination, crew, delegation, parallel execution,
//! background loop, meta-panel, skill chains, experience tree, self-improvement.
//!

#![forbid(unsafe_code)]
//! Migrated from L5 cognition:
//! - `coordinator` — Multi-Agent parallel execution coordinator
//! - `coordination` — Shape-level coordination principles
//! - `hive` — Hive Communication Protocol (inbox/outbox/blackboard)
//! - `god_agent` — GOD Agent central orchestrator
//! - `skill_registry` — SKILL.md discovery and activation

pub mod multi_agent;

// Migrated from L5 cognition
pub mod coordinator;
pub mod coordination;
pub mod hive;
pub mod god_agent;
pub mod skill_registry;
