//! Intra-reflection — reasoning quality self-assessment.
//!
//! Analyzes reasoning traces, E8 mode histories, and execution outcomes
//! to produce coherence, efficiency, error density, and mode stability scores.
//! Generates rule-based improvement suggestions and bottleneck detection.

pub mod types;
pub mod analyzer;

pub use types::{ReflectionInput, ReflectionReport};
pub use analyzer::analyze;
