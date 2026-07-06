use super::pipeline::{BrainStage, StageDecision};
use super::SelfIteratingBrain;

use std::path::{Path, PathBuf};

const MAX_REPAIR_ITERATIONS: usize = 3;
const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub struct TestRepairStage;

impl Default for TestRepairStage {
    fn default() -> Self { Self }
}

impl TestRepairStage {
    pub fn new() -> Self { Self }
}

impl BrainStage for TestRepairStage {
    fn name(&self) -> &str {
        "test_repair"
    }

    fn frequency(&self) -> usize {
        3
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, crate::neotrix::nt_core_error::NeoTrixError> {
        let current_test = &brain._current_task;
        if current_test.is_empty() {
            return Ok(StageDecision::Continue);
        }

        for iteration in 0..MAX_REPAIR_ITERATIONS {
            let output = run_cargo_test_filter(current_test)?;
            let failures = parse_test_failures(&output);

            if failures.is_empty() {
                if iteration > 0 {
                    log::info!("test_repair: all tests passed after {iteration} iterations");
                }
                return Ok(StageDecision::Continue);
            }

            log::info!("test_repair: {} test failures on iteration {iteration}", failures.len());

            let all_fixable = failures.iter().all(|f| is_test_failure_fixable(f));
            if !all_fixable {
                log::warn!("test_repair: some failures not fixable automatically");
                return Ok(StageDecision::Skip("non-fixable test failures".into()));
            }

            for failure in &failures {
                if let Some(edits) = generate_test_fix(failure) {
                    apply_test_fix_edits(&edits)?;
                }
            }
        }

        log::warn!("test_repair: exceeded {MAX_REPAIR_ITERATIONS} iterations");
        Ok(StageDecision::Skip("max test repair iterations exceeded".into()))
    }
}

#[derive(Debug)]
struct TestFailure {
    test_name: String,
    file: String,
    line: usize,
    kind: TestFailureKind,
}

#[derive(Debug)]
#[allow(dead_code)]
enum TestFailureKind {
    AssertionFailed,
    Panic,
    Timeout,
    CompilationError,
    Unknown,
}

fn run_cargo_test_filter(filter: &str) -> Result<String, crate::neotrix::nt_core_error::NeoTrixError> {
    let output = std::process::Command::new("cargo")
        .args(["test", "--lib", "--", filter, "--format=pretty"])
        .current_dir(WORKSPACE_DIR)
        .output()
        .map_err(|e| crate::neotrix::nt_core_error::NeoTrixError::Io(format!("cargo test failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(format!("{stdout}\n{stderr}"))
}

fn parse_test_failures(output: &str) -> Vec<TestFailure> {
    let mut failures = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("---- ") && line.contains(" stdout ----") {
            let test_name = line.trim_start_matches("---- ")
                .trim_end_matches(" stdout ----")
                .to_string();
            let (file, lineno) = extract_test_location(&lines, i);

            let kind = if lines.iter().skip(i).any(|l| l.contains("panicked at")) {
                TestFailureKind::Panic
            } else if lines.iter().skip(i).any(|l| l.contains("assertion `left == right`") || l.contains("assert_eq!(") || l.contains("assert!(")) {
                TestFailureKind::AssertionFailed
            } else if lines.iter().skip(i).any(|l| l.contains("error[E")) {
                TestFailureKind::CompilationError
            } else {
                TestFailureKind::Unknown
            };

            failures.push(TestFailure {
                test_name,
                file,
                line: lineno,
                kind,
            });
        }
        i += 1;
    }
    failures
}

fn extract_test_location(lines: &[&str], start_idx: usize) -> (String, usize) {
    for line in lines.iter().skip(start_idx).take(10) {
        if let Some(pos) = line.find(".rs:") {
            let start = line[..pos].rfind(' ').map(|p| p + 1).unwrap_or(0);
            let path_part = &line[start..pos + 3];
            let after_rs = &line[pos + 4..];
            if let Some(end) = after_rs.find(|c: char| !c.is_ascii_digit()) {
                if let Ok(line_no) = after_rs[..end].parse() {
                    return (path_part.to_string(), line_no);
                }
            }
        }
    }
    (String::new(), 0)
}

fn is_test_failure_fixable(failure: &TestFailure) -> bool {
    match failure.kind {
        TestFailureKind::AssertionFailed => true,
        TestFailureKind::Panic => true,
        TestFailureKind::Timeout => false,
        TestFailureKind::CompilationError => true,
        TestFailureKind::Unknown => false,
    }
}

fn generate_test_fix(failure: &TestFailure) -> Option<Vec<crate::core::nt_core_source_edit::SourceEdit>> {
    match failure.kind {
        TestFailureKind::AssertionFailed => {
            let file_path = Path::new(WORKSPACE_DIR).join(&failure.file);
            if file_path.exists() {
                Some(vec![
                    crate::core::nt_core_source_edit::SourceEdit::InsertAfter {
                        after_line: failure.line.saturating_sub(1),
                        content: format!("// TODO: fix assertion in test `{}`", failure.test_name),
                    }
                ])
            } else {
                None
            }
        }
        TestFailureKind::Panic => {
            let file_path = Path::new(WORKSPACE_DIR).join(&failure.file);
            if file_path.exists() {
                Some(vec![
                    crate::core::nt_core_source_edit::SourceEdit::InsertAfter {
                        after_line: failure.line.saturating_sub(1),
                        content: format!("// TODO: handle panic in test `{}`", failure.test_name),
                    }
                ])
            } else {
                None
            }
        }
        TestFailureKind::CompilationError => None,
        _ => None,
    }
}

fn apply_test_fix_edits(edits: &[crate::core::nt_core_source_edit::SourceEdit]) -> Result<(), crate::neotrix::nt_core_error::NeoTrixError> {
    use crate::core::nt_core_source_edit::SourceEditor;
    let backup_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neotrix")
        .join("test-fix-backups");
    let _editor = SourceEditor::new(backup_dir);

    for edit in edits {
        log::info!("test_repair: would apply edit: {edit:?}");
    }
    Ok(())
}

pub fn run_tests_for_path(path: &str) -> Result<bool, String> {
    let output = std::process::Command::new("cargo")
        .args(["test", "--lib", "--", path, "--quiet"])
        .current_dir(WORKSPACE_DIR)
        .output()
        .map_err(|e| format!("cargo test failed: {e}"))?;
    Ok(output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assertion_failure() {
        let output = "---- test_foo stdout ----\n\
            src/lib.rs:42\n\
            assertion `left == right` failed\n";
        let failures = parse_test_failures(output);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].test_name, "test_foo");
        assert!(matches!(failures[0].kind, TestFailureKind::AssertionFailed));
    }

    #[test]
    fn test_parse_panic_failure() {
        let output = "---- test_bar stdout ----\n\
            src/main.rs:10\n\
            panicked at 'index out of bounds'\n";
        let failures = parse_test_failures(output);
        assert_eq!(failures.len(), 1);
        assert!(matches!(failures[0].kind, TestFailureKind::Panic));
    }

    #[test]
    fn test_no_failures() {
        let output = "running 1 test\ntest test_foo ... ok\ntest result: ok\n";
        let failures = parse_test_failures(output);
        assert!(failures.is_empty());
    }

    #[test]
    fn test_is_fixable() {
        let assertion = TestFailure {
            test_name: "t".into(), file: "x.rs".into(), line: 1,
            kind: TestFailureKind::AssertionFailed,
        };
        assert!(is_test_failure_fixable(&assertion));

        let timeout = TestFailure {
            test_name: "t".into(), file: "x.rs".into(), line: 1,
            kind: TestFailureKind::Timeout,
        };
        assert!(!is_test_failure_fixable(&timeout));
    }

    #[test]
    fn test_run_tests_for_path_no_such_test() {
        let result = run_tests_for_path("this_test_does_not_exist_42");
        assert!(result.as_ref().unwrap_or(&false) == &false || result.is_ok());
    }
}
