//! L6 Meta-Cognition Layer - Meta Modules
//! 
//! Meta-cognitive functionality including cross-domain awareness, goal analysis, routing.

// Local modules
pub mod nt_governance;
pub mod nt_core_intra_reflection;
pub mod nt_shield_approval;
pub mod nt_shield_audit;
pub mod nt_mind_repair;
pub mod nt_meta_sentrux;

// Re-export all
pub use nt_governance::*;
pub use nt_core_intra_reflection::*;
pub use nt_shield_approval::*;
pub use nt_shield_audit::*;
pub use nt_mind_repair::*;
pub use nt_meta_sentrux::{SentruxSensor, QualitySnapshot, SessionComparison};