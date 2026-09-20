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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_stage_serde_roundtrip() {
        let stage = PipelineStage {
            name: "parse".into(),
            stage_type: "transform".into(),
            config: serde_json::json!({"mode": "fast"}),
        };
        let json = serde_json::to_string(&stage).unwrap();
        let back: PipelineStage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "parse");
        assert_eq!(back.config["mode"], "fast");
    }

    #[test]
    fn pipeline_result_serde_roundtrip() {
        let r = PipelineResult {
            success: true,
            output: serde_json::json!("done"),
            duration_ms: 150,
            stages_completed: 3,
            error: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: PipelineResult = serde_json::from_str(&json).unwrap();
        assert!(back.success);
        assert_eq!(back.stages_completed, 3);
    }

    #[test]
    fn pipeline_result_with_error() {
        let r = PipelineResult {
            success: false,
            output: serde_json::json!(null),
            duration_ms: 0,
            stages_completed: 1,
            error: Some("timeout".into()),
        };
        assert!(!r.success);
        assert!(r.error.is_some());
    }

    #[test]
    fn pipeline_config_serde_roundtrip() {
        let cfg = PipelineConfig {
            name: "test-pipe".into(),
            stages: vec![PipelineStage {
                name: "s1".into(),
                stage_type: "a".into(),
                config: serde_json::json!({}),
            }],
            max_retries: 3,
            timeout_secs: 60,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: PipelineConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test-pipe");
        assert_eq!(back.stages.len(), 1);
    }
}
