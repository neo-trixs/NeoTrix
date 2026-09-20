//! Main pipeline — five-stage synchronous task processing pipeline.
//!
//! Flow: `Intake → Route → Execute → Validate → Output`
//!
//! Each stage is independently testable and returns typed results.

pub mod main_pipeline;
pub mod intake;
pub mod route;
pub mod execute;
pub mod validate;
pub mod output;

pub use main_pipeline::{
    Artifact, ArtifactKind, MainPipeline, PipelineError, PipelineInput, PipelineMetrics,
    PipelineOptions, PipelineResult,
};
pub use intake::{IntakeResult, IntakeStage};
pub use route::{RouteResult, RouteStage};
pub use execute::{ExecuteResult, ExecuteStage};
pub use validate::{ValidateStage, ValidationResult};
pub use output::OutputStage;
