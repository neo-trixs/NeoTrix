// Specialized Tools & Infrastructure: reviews, code graph, benchmarks, tests
//
// Fused real module directory (Cycle 161b): flat nt_mind modules moved in,
// replacing the earlier empty re-export shell.
//
#![deny(clippy::unwrap_used)]

pub mod build_context;
pub mod case_study;
pub mod code_graph;
pub mod code_graph_executor;
pub mod code_review;
#[cfg(test)]
pub mod context_integration_test;
pub mod group_contracts;
pub mod kronecker_cleanup;
pub mod open_source_benchmark;
pub mod react_doctor;
pub mod stakeholder_comm;
pub mod tests;
pub mod ux_review;

// Cross-domain surface: re-export nt_mind scope so internal super:: refs resolve at domain level
pub use super::*;
