//! Main pipeline — five-stage synchronous task processing pipeline.
//!
//! Flow: `Intake → Route → Execute → Validate → Output`
//!
//! Each stage is independently testable and returns typed results.
//! The pipeline measures per-stage timing and collects artifacts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::l0_substrate::nt_core_error::NeoTrixError;

use super::intake::IntakeStage;
use super::route::RouteStage;
use super::execute::ExecuteStage;
use super::validate::ValidateStage;
use super::output::OutputStage;

// ════════════════════════════════════════════════════════════════
// Pipeline Input
// ════════════════════════════════════════════════════════════════

/// Pipeline input — task description plus context and options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineInput {
    /// The task to process (natural language).
    pub task: String,
    /// Key-value context passed through all stages.
    #[serde(default)]
    pub context: HashMap<String, String>,
    /// Pipeline execution options.
    #[serde(default)]
    pub options: PipelineOptions,
}

impl PipelineInput {
    pub fn new(task: impl Into<String>) -> Self {
        Self {
            task: task.into(),
            context: HashMap::new(),
            options: PipelineOptions::default(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn with_options(mut self, options: PipelineOptions) -> Self {
        self.options = options;
        self
    }
}

/// Pipeline execution options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOptions {
    /// Maximum retry count per stage.
    pub max_retries: usize,
    /// Overall pipeline timeout in milliseconds (0 = no limit).
    pub timeout_ms: u64,
    /// If true, validation failures abort the pipeline.
    pub strict_validation: bool,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            max_retries: 0,
            timeout_ms: 0,
            strict_validation: true,
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Pipeline Output
// ════════════════════════════════════════════════════════════════

/// Pipeline result — output text, artifacts, metrics, and duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// Final output text.
    pub output: String,
    /// Artifacts produced during pipeline execution.
    pub artifacts: Vec<Artifact>,
    /// Per-stage timing and token metrics.
    pub metrics: PipelineMetrics,
    /// Total pipeline duration in milliseconds.
    pub duration_ms: u64,
}

/// An artifact produced by a pipeline stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique artifact identifier.
    pub id: String,
    /// Kind of artifact.
    pub kind: ArtifactKind,
    /// Artifact content.
    pub content: String,
    /// Unix timestamp when the artifact was created.
    pub created_at: u64,
    /// Which stage produced this artifact.
    pub source_stage: String,
}

/// Artifact kind classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactKind {
    /// Plain text output.
    Text,
    /// Structured data (JSON).
    Data,
    /// Error or warning record.
    Error,
}

/// Per-stage timing and cumulative metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineMetrics {
    /// Intake stage duration in ms.
    pub intake_ms: u64,
    /// Route stage duration in ms.
    pub route_ms: u64,
    /// Execute stage duration in ms.
    pub execute_ms: u64,
    /// Validate stage duration in ms.
    pub validate_ms: u64,
    /// Output stage duration in ms.
    pub output_ms: u64,
    /// Total tokens used (cumulative across stages).
    pub total_tokens: u32,
}

// ════════════════════════════════════════════════════════════════
// Pipeline Error
// ════════════════════════════════════════════════════════════════

/// Pipeline-specific error with stage context.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("pipeline error at stage '{stage}': {source}")]
    StageFailure {
        stage: &'static str,
        source: NeoTrixError,
    },
    #[error("pipeline timeout after {0}ms")]
    Timeout(u64),
}

impl PipelineError {
    pub fn stage(stage: &'static str, source: NeoTrixError) -> Self {
        Self::StageFailure { stage, source }
    }
}

impl From<PipelineError> for NeoTrixError {
    fn from(e: PipelineError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

// ════════════════════════════════════════════════════════════════
// MainPipeline
// ════════════════════════════════════════════════════════════════

/// Main pipeline — orchestrates five stages synchronously.
///
/// ```ignore
/// use neotrix::pipeline::{MainPipeline, PipelineInput};
///
/// let pipeline = MainPipeline::new();
/// let input = PipelineInput::new("analyze this data");
/// let result = pipeline.process(input).unwrap();
/// assert!(!result.output.is_empty());
/// ```
pub struct MainPipeline {
    intake: IntakeStage,
    route: RouteStage,
    execute: ExecuteStage,
    validate: ValidateStage,
    output: OutputStage,
}

impl Default for MainPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl MainPipeline {
    pub fn new() -> Self {
        Self {
            intake: IntakeStage::new(),
            route: RouteStage::new(),
            execute: ExecuteStage::new(),
            validate: ValidateStage::new(),
            output: OutputStage::new(),
        }
    }

    /// Process input through all five pipeline stages.
    pub fn process(&self, input: PipelineInput) -> Result<PipelineResult, PipelineError> {
        let pipeline_start = now_millis();
        let mut metrics = PipelineMetrics::default();
        let mut artifacts = Vec::new();

        // Stage 1: Intake
        let t0 = now_millis();
        let mut intake_result = self
            .intake
            .process(&input)
            .map_err(|e| PipelineError::stage("intake", e))?;
        metrics.intake_ms = elapsed_ms(t0);
        artifacts.extend(std::mem::take(&mut intake_result.artifacts));

        // Stage 2: Route
        let t0 = now_millis();
        let mut route_result = self
            .route
            .process(&intake_result, &input.context)
            .map_err(|e| PipelineError::stage("route", e))?;
        metrics.route_ms = elapsed_ms(t0);
        artifacts.extend(std::mem::take(&mut route_result.artifacts));

        // Stage 3: Execute
        let t0 = now_millis();
        let mut execute_result = self
            .execute
            .process(&route_result, &input.context)
            .map_err(|e| PipelineError::stage("execute", e))?;
        metrics.execute_ms = elapsed_ms(t0);
        artifacts.extend(std::mem::take(&mut execute_result.artifacts));

        // Stage 4: Validate
        let t0 = now_millis();
        let mut validate_result = self
            .validate
            .process(&execute_result, &input.options)
            .map_err(|e| PipelineError::stage("validate", e))?;
        metrics.validate_ms = elapsed_ms(t0);
        artifacts.extend(std::mem::take(&mut validate_result.artifacts));

        // Stage 5: Output
        let t0 = now_millis();
        let output_result = self
            .output
            .process(&execute_result, &validate_result, &artifacts)
            .map_err(|e| PipelineError::stage("output", e))?;
        metrics.output_ms = elapsed_ms(t0);

        let duration_ms = elapsed_ms(pipeline_start);

        Ok(PipelineResult {
            output: output_result,
            artifacts,
            metrics,
            duration_ms,
        })
    }
}

// ════════════════════════════════════════════════════════════════
// Utilities
// ════════════════════════════════════════════════════════════════

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn elapsed_ms(start: u64) -> u64 {
    now_millis().saturating_sub(start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_input_default_options() {
        let input = PipelineInput::new("hello");
        assert_eq!(input.task, "hello");
        assert!(input.context.is_empty());
        assert_eq!(input.options.max_retries, 0);
        assert!(input.options.strict_validation);
    }

    #[test]
    fn test_pipeline_input_builder() {
        let input = PipelineInput::new("task")
            .with_context("key", "value")
            .with_options(PipelineOptions {
                max_retries: 3,
                timeout_ms: 5000,
                strict_validation: false,
            });
        assert_eq!(input.context.get("key").unwrap(), "value");
        assert_eq!(input.options.max_retries, 3);
    }

    #[test]
    fn test_pipeline_result_serialization() {
        let result = PipelineResult {
            output: "done".into(),
            artifacts: vec![],
            metrics: PipelineMetrics::default(),
            duration_ms: 42,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("done"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_pipeline_process_basic() {
        let pipeline = MainPipeline::new();
        let input = PipelineInput::new("analyze this data");
        let result = pipeline.process(input).unwrap();
        assert!(!result.output.is_empty());
        assert!(result.duration_ms < 1000);
    }

    #[test]
    fn test_artifact_kind_equality() {
        assert_eq!(ArtifactKind::Text, ArtifactKind::Text);
        assert_ne!(ArtifactKind::Text, ArtifactKind::Error);
    }
}
