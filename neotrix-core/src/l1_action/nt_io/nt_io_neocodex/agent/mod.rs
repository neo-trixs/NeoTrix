// ── NeoCodex Agent Loop (from Claude Code: ReAct pattern + NeoTrix Consciousness) ──

pub mod nt_agent_types;
pub mod nt_agent_session;
pub mod nt_agent_exec;
pub mod nt_agent_subagents;
pub mod nt_agent_interaction;
#[cfg(test)]
mod nt_agent_tests;

pub use nt_agent_types::*;
