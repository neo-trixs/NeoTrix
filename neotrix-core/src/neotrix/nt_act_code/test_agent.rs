use std::time::Instant;

use crate::neotrix::nt_mind::code_review::{IssueCategory, IssueSeverity, ReviewIssue};

use super::review_aggregator::AgentReviewResult;

pub struct TestEdgeCaseAgent;

impl TestEdgeCaseAgent {
    pub fn review(code: &str, _file: &str) -> AgentReviewResult {
        let start = Instant::now();
        let mut issues = Vec::new();

        Self::check_missing_test_module(code, &mut issues);
        Self::check_uncovered_functions(code, &mut issues);
        Self::check_option_result_unhandled(code, &mut issues);
        Self::check_proptest_opportunities(code, &mut issues);
        Self::check_edge_case_triggers(code, &mut issues);
        Self::check_empty_input_vulnerability(code, &mut issues);
        Self::check_boundary_values(code, &mut issues);
        Self::check_error_path_missing(code, &mut issues);

        let penalty: f64 = issues
            .iter()
            .map(|i| match i.severity {
                IssueSeverity::Critical => 0.15,
                IssueSeverity::High => 0.08,
                IssueSeverity::Medium => 0.04,
                IssueSeverity::Low => 0.02,
                IssueSeverity::Info => 0.01,
            })
            .sum();
        let score = (1.0 - penalty).max(0.0);
        let elapsed = start.elapsed().as_millis() as u64;

        AgentReviewResult {
            agent_name: "agent-test-edge".into(),
            dimension: "Testing & Edge Cases".into(),
            issues,
            score,
            duration_ms: elapsed,
        }
    }

    fn check_missing_test_module(code: &str, issues: &mut Vec<ReviewIssue>) {
        let has_test_attr = code.contains("#[test]");
        let has_test_cfg = code.contains("#[cfg(test)]");
        let _has_test_mod = code.contains("#[cfg(test)]");
        let fn_count = code.matches("fn ").count();

        if !has_test_attr && fn_count > 2 {
            let sev = if fn_count > 10 {
                IssueSeverity::High
            } else {
                IssueSeverity::Medium
            };
            let suggestion = if has_test_cfg {
                Some("Add #[test] functions inside the existing #[cfg(test)] module".into())
            } else {
                Some(
                    "Add a #[cfg(test)] module block with #[test] functions covering public API"
                        .into(),
                )
            };
            issues.push(ReviewIssue {
                severity: sev,
                category: IssueCategory::Testing,
                message: format!("{} functions found but no #[test] attribute", fn_count),
                line: None,
                suggestion,
            });
        }
    }

    fn check_uncovered_functions(code: &str, issues: &mut Vec<ReviewIssue>) {
        let test_count = code.matches("#[test]").count();
        let pub_fn_count = code.matches("pub fn ").count();

        if pub_fn_count > 0 && test_count == 0 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::Testing,
                message: format!("{} public functions with zero tests", pub_fn_count),
                line: None,
                suggestion: Some("Add unit tests for each public function covering: normal path, error path, edge case".into()),
            });
        } else if pub_fn_count > test_count * 3 && test_count > 0 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Testing,
                message: format!(
                    "Test ratio low: {} tests for {} public functions",
                    test_count, pub_fn_count
                ),
                line: None,
                suggestion: Some("Aim for at least one test per public function".into()),
            });
        }
    }

    fn check_option_result_unhandled(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.contains("-> Option<") {
                let mut has_none_handling = false;
                let mut _has_some_pattern = false;
                let mut brace_depth = 0;
                let mut started = false;

                for fl in code.lines().skip(i + 1) {
                    let f = fl.trim();
                    if f.starts_with("fn ") {
                        break;
                    }
                    if f == "{" {
                        started = true;
                        brace_depth += 1;
                        continue;
                    }
                    if !started {
                        continue;
                    }
                    if f.contains('{') {
                        brace_depth += 1;
                    }
                    if f.contains('}') {
                        brace_depth -= 1;
                        if brace_depth <= 0 {
                            break;
                        }
                    }
                    if f.contains("None") {
                        has_none_handling = true;
                    }
                    if f.contains("Some(") {
                        _has_some_pattern = true;
                    }
                }

                if started && !has_none_handling {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::Medium,
                        category: IssueCategory::Testing,
                        message: "Function returns Option but never handles None case in body"
                            .into(),
                        line: Some((i + 1) as u32),
                        suggestion: Some(
                            "Add None test case: verify function returns None for invalid input"
                                .into(),
                        ),
                    });
                }
            }
        }
    }

    fn check_proptest_opportunities(code: &str, issues: &mut Vec<ReviewIssue>) {
        let mut symmetry_candidates = 0;
        let mut idempotent_candidates = 0;
        let mut roundtrip_candidates = 0;

        for line in code.lines() {
            let l = line.trim();
            if l.contains("fn ")
                && (l.contains("sort") || l.contains("reverse") || l.contains("normalize"))
            {
                symmetry_candidates += 1;
            }
            if l.contains("fn ")
                && (l.contains("validate") || l.contains("sanitize") || l.contains("clean"))
            {
                idempotent_candidates += 1;
            }
            if l.contains("fn ")
                && (l.contains("serialize") || l.contains("encode") || l.contains("compress"))
            {
                roundtrip_candidates += 1;
            }
        }

        if symmetry_candidates > 0 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Low,
                category: IssueCategory::Testing,
                message: format!(
                    "{} symmetry candidates: property-based test (∀x: f(f(x)) == x)",
                    symmetry_candidates
                ),
                line: None,
                suggestion: Some(
                    "Add proptest: verify idempotence or self-inverse property".into(),
                ),
            });
        }
        if idempotent_candidates > 0 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Low,
                category: IssueCategory::Testing,
                message: format!(
                    "{} idempotency candidates: property-based test (∀x: f(f(x)) == f(x))",
                    idempotent_candidates
                ),
                line: None,
                suggestion: Some("Add proptest: verify idempotence".into()),
            });
        }
        if roundtrip_candidates > 0 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Low,
                category: IssueCategory::Testing,
                message: format!(
                    "{} round-trip candidates: property-based test (∀x: decode(encode(x)) == x)",
                    roundtrip_candidates
                ),
                line: None,
                suggestion: Some("Add proptest: verify round-trip serialization".into()),
            });
        }
    }

    fn check_edge_case_triggers(code: &str, issues: &mut Vec<ReviewIssue>) {
        let mut has_zero_input = false;
        let mut _has_empty_input = false;
        let mut _has_max_input = false;
        let mut has_overflow = false;

        for line in code.lines() {
            let l = line.trim();
            if l.starts_with("//") || l.starts_with('#') {
                continue;
            }
            if l.contains("== 0") || l.contains(".is_empty()") || l.contains("len() == 0") {
                has_zero_input = true;
            }
            if l.contains("> 0") || l.contains("!= 0") || l.contains("len() >") {
                _has_empty_input = true;
            }
            if l.contains("MAX") || l.contains("max") || l.contains("overflow") {
                _has_max_input = true;
            }
            if l.contains("checked_") || l.contains("saturating_") || l.contains("wrapping_") {
                has_overflow = true;
            }
        }

        if !has_zero_input {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Testing,
                message: "No zero/empty input guard detected — edge case test recommended".into(),
                line: None,
                suggestion: Some(
                    "Add test: call function with 0, empty string, or empty collection".into(),
                ),
            });
        }

        if !has_overflow {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Low,
                category: IssueCategory::Testing,
                message: "No overflow-safe arithmetic (checked/saturating/wrapping) — fuzz test recommended".into(),
                line: None,
                suggestion: Some("Add fuzz test: verify no panics on extreme numeric inputs".into()),
            });
        }
    }

    fn check_empty_input_vulnerability(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.contains("fn ") && !trimmed.contains("->") {
                continue;
            }

            if trimmed.contains("[u8]")
                || trimmed.contains("&str")
                || trimmed.contains("String")
                || trimmed.contains("Vec<")
                || trimmed.contains("&[")
            {
                let has_empty_check = code.lines().skip(i).take(10).any(|l| {
                    let t = l.trim();
                    t.contains("is_empty()") || t.contains("len() == 0") || t.contains("is_empty")
                });
                if !has_empty_check {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::Low,
                        category: IssueCategory::Testing,
                        message: "Function accepts slice/string but no empty-input guard".into(),
                        line: Some((i + 1) as u32),
                        suggestion: Some(
                            "Add empty input test and guard clause at function entry".into(),
                        ),
                    });
                }
            }
        }
    }

    fn check_boundary_values(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.contains("== ") && (trimmed.contains("0") || trimmed.contains("1")) {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Info,
                    category: IssueCategory::Testing,
                    message: "Equality comparison — verify boundary test covers values just above/below threshold".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Add boundary tests: N-1, N, N+1 where N is the comparison value".into()),
                });
            }
        }
    }

    fn check_error_path_missing(code: &str, issues: &mut Vec<ReviewIssue>) {
        let mut result_fns = 0;
        let mut err_tests = 0;

        for line in code.lines() {
            let l = line.trim();
            if l.contains("fn ") && l.contains("Result<") {
                result_fns += 1;
            }
        }

        let test_blocks: Vec<&str> = code.split("#[test]").collect();
        for block in &test_blocks {
            if block.contains("Err") || block.contains("error") || block.contains("fail") {
                err_tests += 1;
            }
        }

        if result_fns > 0 && err_tests == 0 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Testing,
                message: format!("{} functions return Result but zero error-path tests", result_fns),
                line: None,
                suggestion: Some("Add #[test] that asserts Err variant: verify error type, message, and recovery".into()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_code_produces_reasonable_report() {
        let result = TestEdgeCaseAgent::review("fn foo() {}", "test.rs");
        assert_eq!(result.agent_name, "agent-test-edge");
        assert!(result.score >= 0.0 && result.score <= 1.0);
    }

    #[test]
    fn test_code_with_tests_generates_fewer_issues() {
        let code = r#"
pub fn add(a: i32, b: i32) -> i32 { a + b }
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add() { assert_eq!(add(1, 2), 3); }
}"#;
        let result = TestEdgeCaseAgent::review(code, "lib.rs");
        let no_test_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.message.contains("test") || i.message.contains("Test"))
            .collect();
        assert!(
            no_test_issues.is_empty(),
            "Should not flag missing tests: {:?}",
            no_test_issues
        );
    }

    #[test]
    fn test_option_function_detected() {
        let code = r#"
fn find(key: &str) -> Option<i32> {
    if key == "a" { Some(1) } else { None }
}"#;
        let result = TestEdgeCaseAgent::review(code, "test.rs");
        let has_option_check = result.issues.iter().any(|i| i.message.contains("Option"));
        assert!(has_option_check);
    }

    #[test]
    fn test_boundary_check_triggers() {
        let code = r#"
fn is_adult(age: i32) -> bool {
    age == 18
}"#;
        let result = TestEdgeCaseAgent::review(code, "test.rs");
        let has_boundary = result.issues.iter().any(|i| i.message.contains("boundary"));
        assert!(has_boundary);
    }

    #[test]
    fn test_proptest_opportunities_detected() {
        let code = r#"
fn sort_items(items: &mut [i32]) { items.sort(); }
fn sanitize(input: &str) -> String { input.trim().to_string() }
fn encode(data: &[u8]) -> Vec<u8> { data.to_vec() }"#;
        let result = TestEdgeCaseAgent::review(code, "test.rs");
        let proptest_count = result
            .issues
            .iter()
            .filter(|i| i.message.contains("proptest") || i.message.contains("property"))
            .count();
        assert_eq!(
            proptest_count, 3,
            "Should detect 3 property-based test opportunities"
        );
    }

    #[test]
    fn test_error_path_detected() {
        let code = r#"
pub fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 { Err("div by zero".into()) } else { Ok(a / b) }
}"#;
        let result = TestEdgeCaseAgent::review(code, "test.rs");
        let has_err_test = result
            .issues
            .iter()
            .any(|i| i.message.contains("error-path"));
        assert!(has_err_test);
    }

    #[test]
    fn test_empty_input_guard_detected() {
        let code = r#"
fn process(data: &[u8]) -> u32 {
    data.iter().sum()
}"#;
        let result = TestEdgeCaseAgent::review(code, "test.rs");
        let has_empty_guard = result
            .issues
            .iter()
            .any(|i| i.message.contains("empty-input"));
        assert!(has_empty_guard);
    }
}
