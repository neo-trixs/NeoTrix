//! Route stage — routes to appropriate agent based on task capabilities.

use std::collections::HashMap;

use crate::l0_substrate::nt_core_error::NeoTrixResult;

use super::intake::IntakeResult;
use super::main_pipeline::{Artifact, ArtifactKind};

/// Result of the route stage.
#[derive(Debug, Clone)]
pub struct RouteResult {
    /// Selected agent/capability identifier.
    pub agent_id: String,
    /// Capability tags that matched.
    pub matched_capabilities: Vec<String>,
    /// Routing confidence (0.0 - 1.0).
    pub confidence: f64,
    /// Artifacts produced during routing.
    pub artifacts: Vec<Artifact>,
}

/// Route stage — selects the best agent for a task.
#[derive(Default)]
pub struct RouteStage;

impl RouteStage {
    pub fn new() -> Self {
        Self
    }

    /// Route the intake result to an appropriate agent.
    pub fn process(
        &self,
        intake: &IntakeResult,
        _context: &HashMap<String, String>,
    ) -> NeoTrixResult<RouteResult> {
        let task = &intake.normalized_task;
        let lower = task.to_lowercase();

        // Capability detection — keyword-based routing
        let mut matched = Vec::new();
        let mut agent_id = "general".to_string();

        if lower.contains("code") || lower.contains("implement") || lower.contains("program") || lower.contains("rust") || lower.contains("python") {
            matched.push("code_generation".into());
            agent_id = "coder".into();
        }
        if lower.contains("analyze") || lower.contains("review") || lower.contains("audit") {
            matched.push("analysis".into());
            if agent_id == "general" {
                agent_id = "analyst".into();
            }
        }
        if lower.contains("search") || lower.contains("research") || lower.contains("find") {
            matched.push("research".into());
            if agent_id == "general" {
                agent_id = "researcher".into();
            }
        }
        if lower.contains("write") || lower.contains("draft") || lower.contains("document") {
            matched.push("writing".into());
            if agent_id == "general" {
                agent_id = "writer".into();
            }
        }
        if lower.contains("test") || lower.contains("verify") || lower.contains("validate") {
            matched.push("testing".into());
            if agent_id == "general" {
                agent_id = "tester".into();
            }
        }
        if lower.contains("design") || lower.contains("architect") || lower.contains("plan") {
            matched.push("design".into());
            if agent_id == "general" {
                agent_id = "architect".into();
            }
        }

        if matched.is_empty() {
            matched.push("general".into());
        }

        // Confidence: more capability matches = higher confidence
        let confidence = (0.5 + matched.len() as f64 * 0.1).min(1.0);

        let artifact = Artifact {
            id: format!("route_{}", now_nanos()),
            kind: ArtifactKind::Data,
            content: serde_json::json!({
                "agent_id": &agent_id,
                "capabilities": &matched,
                "confidence": confidence,
            })
            .to_string(),
            created_at: now_secs(),
            source_stage: "route".into(),
        };

        Ok(RouteResult {
            agent_id,
            matched_capabilities: matched,
            confidence,
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

    fn route_task(task: &str) -> RouteResult {
        let intake_stage = IntakeStage::new();
        let input = PipelineInput::new(task);
        let intake = intake_stage.process(&input).unwrap();
        let route_stage = RouteStage::new();
        route_stage.process(&intake, &HashMap::new()).unwrap()
    }

    #[test]
    fn test_route_code_task() {
        let result = route_task("implement a sorting algorithm in Rust");
        assert_eq!(result.agent_id, "coder");
        assert!(result.matched_capabilities.contains(&"code_generation".to_string()));
    }

    #[test]
    fn test_route_analysis_task() {
        let result = route_task("analyze this data set for anomalies");
        assert_eq!(result.agent_id, "analyst");
        assert!(result.matched_capabilities.contains(&"analysis".to_string()));
    }

    #[test]
    fn test_route_general_task() {
        let result = route_task("hello world");
        assert_eq!(result.agent_id, "general");
        assert!(result.matched_capabilities.contains(&"general".to_string()));
    }

    #[test]
    fn test_route_confidence_increases_with_matches() {
        let result = route_task("write code to test and analyze the implementation");
        assert!(result.confidence > 0.7);
        assert!(result.matched_capabilities.len() >= 3);
    }

    #[test]
    fn test_route_produces_artifact() {
        let result = route_task("design a system");
        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.artifacts[0].source_stage, "route");
    }
}
