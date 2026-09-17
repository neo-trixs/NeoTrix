#![forbid(unsafe_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub max_concurrent_tasks: usize,
    pub task_timeout_secs: u64,
    pub retry_attempts: u32,
    pub enable_monitoring: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 8,
            task_timeout_secs: 300,
            retry_attempts: 3,
            enable_monitoring: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorStatus {
    Idle,
    Running,
    Paused,
    Stopping,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrchestratorStats {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub in_progress_tasks: u64,
    pub avg_task_duration_ms: f64,
    pub by_status: HashMap<String, u64>,
}

#[async_trait]
pub trait Orchestrator: Send + Sync {
    fn name(&self) -> &str;

    fn config(&self) -> &OrchestratorConfig;

    async fn start(&mut self) -> Result<(), String>;

    async fn stop(&mut self) -> Result<(), String>;

    async fn submit_task(&self, task: serde_json::Value) -> Result<String, String>;

    async fn cancel_task(&self, task_id: &str) -> Result<(), String>;

    fn status(&self) -> OrchestratorStatus;

    fn stats(&self) -> OrchestratorStats;
}
