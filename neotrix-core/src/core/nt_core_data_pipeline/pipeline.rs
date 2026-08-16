//! Data pipeline stage trait and orchestrator.
//!
//! Each stage is a unit of work in the data lifecycle:
//! 1. Acquire — fetch raw data from external sources
//! 2. Normalize — parse, validate, deduplicate
//! 3. Store — persist to KB, register in pool, encrypt secrets
//! 4. Distill — extract patterns, generate metadata
//! 5. Report — update panorama, log lineage
//!
//! The orchestrator runs stages in order for each data source type,
//! handling errors gracefully with per-stage isolation.

use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

use super::lineage::{DataLineage, LineageEntry};

/// Returned by each stage after execution.
#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage_name: String,
    pub items_processed: usize,
    pub items_succeeded: usize,
    pub items_failed: usize,
    pub duration_ms: f64,
    pub errors: Vec<String>,
}

impl StageResult {
    pub fn new(name: &str) -> Self {
        Self {
            stage_name: name.to_string(),
            items_processed: 0,
            items_succeeded: 0,
            items_failed: 0,
            duration_ms: 0.0,
            errors: Vec::new(),
        }
    }
}

/// A single stage in the data pipeline.
#[async_trait::async_trait]
pub trait PipelineStage: Send + Sync {
    fn name(&self) -> &str;

    /// Execute this stage. Receives a context string (e.g. resource type)
    /// and returns results.
    async fn execute(&self, context: &str) -> StageResult;
}

/// Combined result of a full pipeline run.
#[derive(Debug, Clone, Default)]
pub struct PipelineRunReport {
    pub stages: Vec<StageResult>,
    pub total_duration_ms: f64,
    pub total_processed: usize,
    pub total_failed: usize,
    pub lineage: Vec<LineageEntry>,
}

impl PipelineRunReport {
    pub fn success_rate(&self) -> f64 {
        let total = self.total_processed + self.total_failed;
        if total == 0 {
            1.0
        } else {
            self.total_processed as f64 / total as f64
        }
    }
}

/// Orchestrator that runs registered stages in sequence.
pub struct PipelineOrchestrator {
    name: String,
    stages: Vec<Box<dyn PipelineStage>>,
    lineage: Arc<RwLock<DataLineage>>,
}

impl PipelineOrchestrator {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            stages: Vec::new(),
            lineage: Arc::new(RwLock::new(DataLineage::new())),
        }
    }

    /// Register a new stage (appended to execution order).
    pub fn register(&mut self, stage: Box<dyn PipelineStage>) {
        log::info!(
            "[pipeline:{}] registered stage: {}",
            self.name,
            stage.name()
        );
        self.stages.push(stage);
    }

    /// Run all stages in sequence for a given context.
    /// Each stage is isolated: failure in one does not abort subsequent stages.
    pub async fn run(&self, context: &str) -> PipelineRunReport {
        let start = Instant::now();
        let mut report = PipelineRunReport::default();

        for stage in &self.stages {
            let stage_start = Instant::now();
            let result = stage.execute(context).await;
            let duration = stage_start.elapsed().as_millis() as f64;

            let mut result = result;
            result.duration_ms = duration;

            report.total_processed += result.items_succeeded;
            report.total_failed += result.items_failed;

            report.lineage.push(LineageEntry {
                source: context.to_string(),
                stage: stage.name().to_string(),
                items_processed: result.items_processed,
                items_succeeded: result.items_succeeded,
                items_failed: result.items_failed,
                duration_ms: duration,
            });

            log::info!(
                "[pipeline:{}] stage {}: {}/{} ok, {} failed in {:.0}ms",
                self.name,
                result.stage_name,
                result.items_succeeded,
                result.items_processed,
                result.items_failed,
                duration,
            );

            report.stages.push(result);
        }

        report.total_duration_ms = start.elapsed().as_millis() as f64;
        {
            let mut lineage = self.lineage.write().await;
            for entry in &report.lineage {
                lineage.record(entry.clone());
            }
        }

        log::info!(
            "[pipeline:{}] complete: {} processed, {} failed in {:.0}ms",
            self.name,
            report.total_processed,
            report.total_failed,
            report.total_duration_ms,
        );

        report
    }

    /// Access the lineage log.
    pub async fn lineage(&self) -> Vec<LineageEntry> {
        self.lineage.read().await.recent(100).await
    }

    /// Number of registered stages.
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
