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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orchestrator_config_default() {
        let cfg = OrchestratorConfig::default();
        assert_eq!(cfg.max_concurrent_tasks, 8);
        assert_eq!(cfg.task_timeout_secs, 300);
        assert_eq!(cfg.retry_attempts, 3);
        assert!(cfg.enable_monitoring);
    }

    #[test]
    fn orchestrator_config_serde_roundtrip() {
        let cfg = OrchestratorConfig {
            max_concurrent_tasks: 16,
            task_timeout_secs: 600,
            retry_attempts: 5,
            enable_monitoring: false,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: OrchestratorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_concurrent_tasks, 16);
        assert!(!back.enable_monitoring);
    }

    #[test]
    fn orchestrator_status_serde_roundtrip() {
        let statuses = vec![
            OrchestratorStatus::Idle,
            OrchestratorStatus::Running,
            OrchestratorStatus::Paused,
            OrchestratorStatus::Stopping,
            OrchestratorStatus::Stopped,
            OrchestratorStatus::Error("fail".into()),
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let back: OrchestratorStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", s), format!("{:?}", back));
        }
    }

    #[test]
    fn orchestrator_stats_default() {
        let s = OrchestratorStats::default();
        assert_eq!(s.total_tasks, 0);
        assert_eq!(s.completed_tasks, 0);
        assert!(s.by_status.is_empty());
    }

    #[test]
    fn orchestrator_stats_serde_roundtrip() {
        let mut s = OrchestratorStats::default();
        s.total_tasks = 10;
        s.by_status.insert("done".into(), 8);
        let json = serde_json::to_string(&s).unwrap();
        let back: OrchestratorStats = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_tasks, 10);
    }
}
