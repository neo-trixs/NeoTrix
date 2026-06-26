//! # Cross-Model Distillation
//!
//! Observes all LLM calls (any provider, any model) and extracts:
//! - **Behavioral patterns** — how models reason, structure code, explain concepts
//! - **Demonstrated capabilities** — what each model is good at (code gen, debugging, etc.)
//! - **Knowledge fragments** — factual/domain knowledge surfaced in responses
//!
//! ## Usage
//!
//! ```rust
//! use std::sync::{Arc, Mutex};
//! use neotrix_mind::distillation::*;
//!
//! // Create a shared capture buffer
//! let buffer = Arc::new(Mutex::new(CaptureBuffer::new(100)));
//!
//! // Push some sample interactions
//! {
//!     let mut buf = buffer.lock().unwrap();
//!     buf.push(CapturedInteraction::new(
//!         "openai", "gpt-4", "sys1", "write fibonacci in rust",
//!         "```rust\nfn fib(n: u64) -> u64 { match n { 0 | 1 => 1, _ => fib(n-1) + fib(n-2) } }\n```",
//!         50, 150, 400, true, "stop",
//!     ));
//!     buf.push(CapturedInteraction::new(
//!         "anthropic", "claude-3", "sys1", "explain monads",
//!         "A monad is a design pattern that wraps a value and provides bind/return operations.",
//!         80, 200, 300, true, "stop",
//!     ));
//! }
//!
//! // Distill
//! let mut distiller = CrossModelDistiller::new(buffer);
//! let report = distiller.distill();
//!
//! // Inspect results
//! assert!(report.total_interactions >= 2);
//! assert!(!report.model_performance.is_empty());
//! println!("Report: {} interactions across {} models",
//!     report.total_interactions,
//!     report.model_performance.len());
//! ```

pub mod capability_extractor;
pub mod capture;
pub mod cross_model_distiller;
pub mod knowledge_extractor;
pub mod pattern_extractor;

pub use capability_extractor::*;
pub use capture::*;
pub use cross_model_distiller::*;
pub use knowledge_extractor::*;
pub use pattern_extractor::*;
