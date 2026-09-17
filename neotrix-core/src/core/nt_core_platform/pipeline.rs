#![forbid(unsafe_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub name: String,
    pub stage_type: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub duration_ms: u64,
    pub stages_completed: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub name: String,
    pub stages: Vec<PipelineStage>,
    pub max_retries: u32,
    pub timeout_secs: u64,
}

#[async_trait]
pub trait Pipeline: Send + Sync {
    fn name(&self) -> &str;

    fn stages(&self) -> Vec<PipelineStage>;

    async fn run(&self, input: serde_json::Value) -> Result<PipelineResult, String>;

    async fn checkpoint(&self, stage: usize, state: serde_json::Value) -> Result<(), String>;

    async fn restore(&self, checkpoint_id: &str) -> Result<serde_json::Value, String>;
}
