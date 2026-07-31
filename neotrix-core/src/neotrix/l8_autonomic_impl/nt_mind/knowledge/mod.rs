// Memory & Knowledge: bank, mining, engine, chain, exploration, archiving
//
// Fused real module directory (Cycle 161b): flat nt_mind modules moved in,
// replacing the earlier empty re-export shell.
//
#![deny(clippy::unwrap_used)]

pub mod change_archive;
pub mod context_artifacts;
pub mod cortex_memory;
pub mod exploration_pipeline;
pub mod exploration_seeds;
pub mod export_import;
pub mod impact_matrix;
pub mod knowledge_chain;
pub mod knowledge_engine;
pub mod knowledge_maturity;
pub mod knowledge_miner;
pub mod memory;
pub mod seal_algebra;
pub mod web_miner;

// Cross-domain surface: re-export nt_mind scope so internal super:: refs resolve at domain level
pub use super::*;
