//! Validate stage - validates outputs against policies.

use crate::l0_substrate::nt_core_error::{NeoTrixError, NeoTrixResult};

use super::execute::ExecuteResult;
use super::main_pipeline::{Artifact, ArtifactKind, PipelineOptions};

/// Result of the validate stage.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed.
    pub passed: bool,
    /// Validation issues found.
    pub issues: Vec<String>,
    /// Artifacts produced during validation.
    pub artifacts: Vec<Artifact>,
}

/// Validate stage - checks execution results against policies.
#[derive(Default)]
pub struct ValidateStage;

impl ValidateStage {
    pub fn new() -> Self {
        Self
    }

    /// Validate the execution result.
    pub fn process(
        &self,
        execute: &ExecuteResult,
        options: &PipelineOptions,
    ) -> NeoTrixResult<ValidationResult> {
        let mut issues = Vec::new();

        // Check: execution must have succeeded
        if !execute.success {
            issues.push("execution reported failure".into());
        }

        // Check: output must not be empty
        if execute.raw_output.is_empty() {
            issues.push("execution output is empty".into());
        }

        // Check: error field should be None on success
        if execute.error.is_some() {
            issues.push(format!(
                "execution error: {}",
                execute.error.as_deref().unwrap_or("unknown")
            ));
        }

        // Check: output length reasonableness
        if execute.raw_output.len() > 1_000_000 {
            issues.push("output exceeds 1MB size limit".into());
        }

        // Check: no obviously malformed content
        if execute.raw_output.contains("\0") {
            issues.push("output contains null bytes".into());
        }

        let passed = issues.is_empty();
        let artifact = Artifact {
            id: format!("validate_{}", now_nanos()),
            kind: ArtifactKind::Data,
            content: serde_json::json!({
                "passed": passed,
                "issue_count": issues.len(),
            })
            .to_string(),
            created_at: now_secs(),
            source_stage: "validate".into(),
        };

        if !passed && options.strict_validation {
            return Err(NeoTrixError::InvalidState(format!(
                "validation failed with {} issues: {}",
                issues.len(),
                issues.join("; ")
            )));
        }

        Ok(ValidationResult {
            passed,
            issues,
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
    use crate::pipeline::main_pipeline::PipelineOptions;

    fn make_execute_result(success: bool, output: &str, error: Option<&str>) -> ExecuteResult {
        ExecuteResult {
            raw_output: output.to_string(),
            agent_id: "test".into(),
            success,
            error: error.map(|s| s.to_string()),
            artifacts: vec![],
        }
    }

    #[test]
    fn test_validate_passes_on_good_result() {
        let stage = ValidateStage::new();
        let exec = make_execute_result(true, "good output", None);
        let result = stage.process(&exec, &PipelineOptions::default()).unwrap();
        assert!(result.passed);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validate_fails_on_empty_output() {
        let stage = ValidateStage::new();
        let exec = make_execute_result(true, "", None);
        let result = stage.process(&exec, &PipelineOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_fails_on_execution_failure() {
        let stage = ValidateStage::new();
        let exec = make_execute_result(false, "some output", Some("crashed"));
        let result = stage.process(&exec, &PipelineOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_lenient_mode_collects_issues() {
        let stage = ValidateStage::new();
        let exec = make_execute_result(true, "", None);
        let opts = PipelineOptions {
            strict_validation: false,
            ..Default::default()
        };
        let result = stage.process(&exec, &opts).unwrap();
        assert!(!result.passed);
        assert!(!result.issues.is_empty());
    }
}
