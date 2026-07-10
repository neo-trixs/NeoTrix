#![forbid(unsafe_code)]

pub mod reflection_head;
pub mod abductive_solver;
pub mod pipeline;

pub use reflection_head::{ReflectionHead, ReflectionOutput};
pub use abductive_solver::{AbductiveSolver, AbductiveExplanation};
pub use pipeline::{ReflectionPipeline, ReflectionResult, ConsensusReport, ConsensusConfig};
