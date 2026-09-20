//! Typed judgment primitives — cheap probabilistic triage before expensive reasoning.
//! Inspired by Jev MCP (TypeSafe) pattern.

pub mod gate;
pub mod primitives;
pub mod verdict;

pub use gate::*;
pub use primitives::*;
pub use verdict::*;
