//! Community dataset ingestion and fusion for E8 transition learning (facade).
//!
//! Original single-file `nt_core_community_ingester.rs` split into `nt_` submodules.
//! External path `nt_core_community_ingester::{...}` is unchanged via re-exports.

pub mod nt_community_data;
pub mod nt_community_fusion;
pub mod nt_community_persist;
pub mod nt_community_runtime;

pub use nt_community_data::{CommunityDataIngester, CommunityDataset};
pub use nt_community_runtime::seed_transition_matrix_with_community;
