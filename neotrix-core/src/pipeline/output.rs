//! Output stage - formats the final pipeline output.

use crate::l0_substrate::nt_core_error::NeoTrixResult;

use super::execute::ExecuteResult;
use super::main_pipeline::Artifact;
use super::validate::ValidationResult;

/// Output stage - produces the final formatted pipeline result string.
#[derive(Default)]
pub struct OutputStage;

impl OutputStage {
    pub fn new() -> Self {
        Self
    }

    /// Format the final pipeline output.
    pub fn process(
        &self,
        execute: &ExecuteResult,
        validate: &ValidationResult,
        artifacts: &[Artifact],
    ) -> NeoTrixResult<String> {
        let mut output = String::new();

        output.push_str("=== Pipeline Output ===\n");
        output.push_str(&format!("agent: {}\n", execute.agent_id));

        let status = if validate.passed { "PASS" } else { "WARN" };
        output.push_str(&format!("validation: {}\n", status));

        if !validate.issues.is_empty() {
            output.push_str(&format!("issues: {}\n", validate.issues.len()));
        }

        output.push_str("\n--- Result ---\n");
        output.push_str(&execute.raw_output);
        output.push('\n');

        output.push_str(&format!("\nartifacts: {}\n", artifacts.len()));
        output.push_str("========================\n");

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::execute::ExecuteResult;
    use crate::pipeline::validate::ValidationResult;

    fn make_execute(output: &str) -> ExecuteResult {
        ExecuteResult {
            raw_output: output.to_string(),
            agent_id: "tester".into(),
            success: true,
            error: None,
            artifacts: vec![],
        }
    }

    fn make_validation(passed: bool) -> ValidationResult {
        ValidationResult {
            passed,
            issues: if passed { vec![] } else { vec!["bad".into()] },
            artifacts: vec![],
        }
    }

    #[test]
    fn test_output_contains_result() {
        let stage = OutputStage::new();
        let exec = make_execute("hello world");
        let val = make_validation(true);
        let result = stage.process(&exec, &val, &[]).unwrap();
        assert!(result.contains("hello world"));
        assert!(result.contains("PASS"));
    }

    #[test]
    fn test_output_shows_warning_on_failure() {
        let stage = OutputStage::new();
        let exec = make_execute("output");
        let val = make_validation(false);
        let result = stage.process(&exec, &val, &[]).unwrap();
        assert!(result.contains("WARN"));
        assert!(result.contains("issues: 1"));
    }

    #[test]
    fn test_output_includes_agent_id() {
        let stage = OutputStage::new();
        let exec = make_execute("result");
        let val = make_validation(true);
        let result = stage.process(&exec, &val, &[]).unwrap();
        assert!(result.contains("agent: tester"));
    }

    #[test]
    fn test_output_artifact_count() {
        let stage = OutputStage::new();
        let exec = make_execute("done");
        let val = make_validation(true);
        let arts = vec![Artifact {
            id: "a1".into(),
            kind: super::super::main_pipeline::ArtifactKind::Text,
            content: "c".into(),
            created_at: 0,
            source_stage: "test".into(),
        }];
        let result = stage.process(&exec, &val, &arts).unwrap();
        assert!(result.contains("artifacts: 1"));
    }
}
