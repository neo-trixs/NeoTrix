//! Execute stage - executes the task and collects outputs.

use std::collections::HashMap;

use crate::l0_substrate::nt_core_error::NeoTrixResult;

use super::main_pipeline::{Artifact, ArtifactKind};
use super::route::RouteResult;

/// Result of the execute stage.
#[derive(Debug, Clone)]
pub struct ExecuteResult {
    /// Raw output from execution.
    pub raw_output: String,
    /// Agent that performed execution.
    pub agent_id: String,
    /// Execution success flag.
    pub success: bool,
    /// Error message if execution failed.
    pub error: Option<String>,
    /// Artifacts produced during execution.
    pub artifacts: Vec<Artifact>,
}

/// Execute stage - runs the task (pass-through for initial implementation).
#[derive(Default)]
pub struct ExecuteStage;

impl ExecuteStage {
    pub fn new() -> Self {
        Self
    }

    /// Execute the routed task.
    ///
    /// Current implementation is a pass-through: returns the task text as output.
    /// Future versions will integrate with `LlmProvider` for real execution.
    pub fn process(
        &self,
        route: &RouteResult,
        _context: &HashMap<String, String>,
    ) -> NeoTrixResult<ExecuteResult> {
        let output = format!(
            "[agent:{}|caps:{}|conf:{:.2}] executed",
            route.agent_id,
            route.matched_capabilities.join(","),
            route.confidence,
        );

        let artifact = Artifact {
            id: format!("exec_{}", now_nanos()),
            kind: ArtifactKind::Text,
            content: output.clone(),
            created_at: now_secs(),
            source_stage: "execute".into(),
        };

        Ok(ExecuteResult {
            raw_output: output,
            agent_id: route.agent_id.clone(),
            success: true,
            error: None,
            artifacts: vec![artifact],
        })
    }
}

fn now_nanos() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::intake::IntakeStage;
    use crate::pipeline::main_pipeline::PipelineInput;
    use crate::pipeline::route::RouteStage;

    fn execute_task(task: &str) -> ExecuteResult {
        let intake_stage = IntakeStage::new();
        let input = PipelineInput::new(task);
        let intake = intake_stage.process(&input).unwrap();
        let route_stage = RouteStage::new();
        let route = route_stage.process(&intake, &HashMap::new()).unwrap();
        let exec_stage = ExecuteStage::new();
        exec_stage.process(&route, &HashMap::new()).unwrap()
    }

    #[test]
    fn test_execute_returns_output() {
        let result = execute_task("implement a function");
        assert!(result.success);
        assert!(!result.raw_output.is_empty());
    }

    #[test]
    fn test_execute_preserves_agent_id() {
        let result = execute_task("write documentation");
        assert_eq!(result.agent_id, "writer");
    }

    #[test]
    fn test_execute_produces_artifact() {
        let result = execute_task("test something");
        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.artifacts[0].source_stage, "execute");
    }
}
