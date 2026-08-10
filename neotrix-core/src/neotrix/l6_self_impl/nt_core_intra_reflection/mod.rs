//! Intra-reflection — reasoning quality self-assessment.
//!
//! Analyzes reasoning traces, E8 mode histories, and execution outcomes
//! to produce coherence, efficiency, error density, and mode stability scores.
//! Generates rule-based improvement suggestions and bottleneck detection.
//!
//! P1: 闭环执行器 (executor) — 检测 → 批判 → 修订 完整 Reflection 闭环。

pub mod types;
pub mod analyzer;
pub mod executor;

pub use types::{ReflectionInput, ReflectionReport};
pub use analyzer::analyze;
pub use executor::{ReflectionExecutor, Critique, Revision, Severity};
