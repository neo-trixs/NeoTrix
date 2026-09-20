//! NeoTrix Multi-Agent Crate
//!
//! L5 Multi-Agent systems — coordination, crew, delegation, parallel execution,
//! background loop, meta-panel, skill chains, experience tree, self-improvement.
//!
//! Migrated from L5 cognition:
//! - `coordinator` — Multi-Agent parallel execution coordinator
//! - `coordination` — Shape-level coordination principles
//! - `hive` — Hive Communication Protocol (inbox/outbox/blackboard)
//! - `god_agent` — GOD Agent central orchestrator
//! - `skill_registry` — SKILL.md discovery and activation

pub mod multi_agent;
pub mod parallel;
pub mod background_loop;
pub mod meta_panel;
pub mod skill_chain;
pub mod element_bus;
pub mod infrastructure;
pub mod experience_tree;
pub mod self_improvement;
pub mod dual_track;

// Migrated from L5 cognition
pub mod coordinator;
pub mod coordination;
pub mod hive;
pub mod god_agent;
pub mod skill_registry;
