// SEAL loop core: brain loop, self-edit, model routing, retrieval primitives
//
// Fused real module directory (Cycle 161b): flat nt_mind modules moved in,
// replacing the earlier empty re-export shell.
//
#![deny(clippy::unwrap_used)]

pub mod core;
pub mod embedding;
pub mod lora;
pub mod model_router;
pub mod multi_brain;
pub mod pipeline;
pub mod backlog;
pub mod self_edit;
pub mod self_iterating;
pub mod stats;
pub mod tier_prompts;

pub use crate::neotrix::l3_memory_impl::nt_memory_kb::bm25;

// Cross-domain surface: re-export nt_mind scope so internal super:: refs resolve at domain level
pub use super::*;
