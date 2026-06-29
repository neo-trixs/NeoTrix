use std::time::Instant;

use crate::neotrix::nt_mind::code_review::{IssueCategory, IssueSeverity, ReviewIssue};

use super::review_aggregator::AgentReviewResult;

pub struct CompatibilityAgent;

impl CompatibilityAgent {
    pub fn review(code: &str, _file: &str) -> AgentReviewResult {
        let start = Instant::now();
        let mut issues = Vec::new();

        Self::check_breaking_api_changes(code, &mut issues);
        Self::check_serde_compatibility(code, &mut issues);
        Self::check_hidden_side_effects(code, &mut issues);
        Self::check_trait_contract_changes(code, &mut issues);
        Self::check_public_struct_field_changes(code, &mut issues);
        Self::check_import_remapping(code, &mut issues);
        Self::check_default_behavior_change(code, &mut issues);
        Self::check_error_type_change(code, &mut issues);

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
            agent_name: "agent-compatibility".into(),
            dimension: "Compatibility & Side Effects".into(),
            issues,
            score,
            duration_ms: elapsed,
        }
    }

    fn check_breaking_api_changes(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.contains("pub fn ") && (trimmed.contains("_v2") || trimmed.contains("_v3")) {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::Maintainability,
                    message: "Versioned function name suggests API evolution — check all callers for migration".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Use deprecation attribute (#[deprecated]) on old version, ensure semver bump".into()),
                });
            }

            if trimmed.contains("pub fn ") && trimmed.contains("&mut self") {
                let fn_name = trimmed
                    .split_whitespace()
                    .skip_while(|w| *w != "fn")
                    .skip(1)
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('(');
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Info,
                    category: IssueCategory::Correctness,
                    message: format!(
                        "'{}' takes &mut self — verify no caller assumes immutability",
                        fn_name
                    ),
                    line: Some((i + 1) as u32),
                    suggestion: Some(
                        "Check if this was previously &self; if so, it's a breaking change".into(),
                    ),
                });
            }
        }
    }

    fn check_serde_compatibility(code: &str, issues: &mut Vec<ReviewIssue>) {
        let mut has_serde_derive = false;
        let mut has_rename_attr = false;
        let mut has_skip = false;
        let mut has_flatten = false;

        for line in code.lines() {
            let l = line.trim();
            if l.contains("#[derive") && l.contains("Serialize") || l.contains("Deserialize") {
                has_serde_derive = true;
            }
            if l.contains("#[serde(rename") {
                has_rename_attr = true;
            }
            if l.contains("#[serde(skip") {
                has_skip = true;
            }
            if l.contains("#[serde(flatten") {
                has_flatten = true;
            }
        }

        if has_serde_derive && has_rename_attr {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Correctness,
                message: "#[serde(rename)] changes serialized field names — breaks existing caches/storage".into(),
                line: None,
                suggestion: Some("Add #[serde(alias = \"old_name\")] for backward-compatible deserialization".into()),
            });
        }

        if has_serde_derive && has_skip {
            issues.push(ReviewIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::Correctness,
                message: "#[serde(skip)] drops field during serialization — data loss risk".into(),
                line: None,
                suggestion: Some(
                    "Use #[serde(default)] instead of skip if backward compat needed".into(),
                ),
            });
        }

        if has_serde_derive && has_flatten {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Correctness,
                message:
                    "#[serde(flatten)] changes serialization shape — verify consumers expect this"
                        .into(),
                line: None,
                suggestion: Some("Test deserialization of old format with new struct".into()),
            });
        }
    }

    fn check_hidden_side_effects(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.contains("fn ") && !trimmed.contains("->") && !trimmed.contains("mut") {
                let fn_name = trimmed
                    .split_whitespace()
                    .skip_while(|w| *w != "fn")
                    .skip(1)
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('(');
                let body_lines: Vec<&str> = code.lines().skip(i + 1).take(30).collect();
                let body = body_lines.join("\n");

                let has_side_effect = body.contains("write")
                    || body.contains("fs::")
                    || body.contains("println")
                    || body.contains("lock(")
                    || body.contains("send(")
                    || body.contains("store(")
                    || body.contains("Cell")
                    || body.contains("RefCell")
                    || body.contains("Mutex")
                    || body.contains("Atomic");

                if has_side_effect {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::Medium,
                        category: IssueCategory::Correctness,
                        message: format!("'{}' has hidden side effect (I/O/mutation) despite no &mut return", fn_name),
                        line: Some((i + 1) as u32),
                        suggestion: Some("Document side effects, or refactor to return value instead of mutating".into()),
                    });
                }
            }
        }
    }

    fn check_trait_contract_changes(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.contains("pub trait ") || trimmed.contains("trait ") {
                let trait_name = trimmed
                    .split_whitespace()
                    .skip_while(|w| *w != "trait")
                    .skip(1)
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('{')
                    .trim();
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Info,
                    category: IssueCategory::Architecture,
                    message: format!("Trait '{}' defined — verify no existing impls break with new required methods", trait_name),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Provide default impl for new methods to avoid breaking existing implementors".into()),
                });
            }

            if trimmed.contains("impl ") && (trimmed.contains("for ") || trimmed.contains("for ")) {
                if trimmed.contains("unsafe") {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::High,
                        category: IssueCategory::Correctness,
                        message: "Unsafe trait implementation — verify safety invariants across all callers".into(),
                        line: Some((i + 1) as u32),
                        suggestion: Some("Add SAFETY doc to impl; verify correctness under all input conditions".into()),
                    });
                }
            }
        }
    }

    fn check_public_struct_field_changes(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.contains("pub struct ") || trimmed.contains("struct ") {
                let struct_name = trimmed
                    .split_whitespace()
                    .skip_while(|w| *w != "struct")
                    .skip(1)
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('{')
                    .trim();
                if trimmed.contains("#[non_exhaustive]")
                    || code
                        .lines()
                        .skip(i)
                        .any(|l| l.contains("#[non_exhaustive]"))
                {
                    continue;
                }
                let fields: Vec<&str> = code
                    .lines()
                    .skip(i + 1)
                    .take(50)
                    .take_while(|l| !l.trim().starts_with('}'))
                    .filter(|l| l.contains("pub "))
                    .collect();
                if fields.len() > 3 {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::Low,
                        category: IssueCategory::Architecture,
                        message: format!("'{}' has {} pub fields — adding/removing fields is breaking without #[non_exhaustive]", struct_name, fields.len()),
                        line: Some((i + 1) as u32),
                        suggestion: Some("Mark struct as #[non_exhaustive] or use builder pattern for extensibility".into()),
                    });
                }
            }
        }
    }

    fn check_import_remapping(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.contains("pub use ") || trimmed.contains("pub(crate) use ") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Info,
                    category: IssueCategory::Maintainability,
                    message: "Re-export detected — renaming or removing breaks downstream consumers".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Add #[doc(inline)] to re-exports for discoverability; use deprecation before removal".into()),
                });
            }

            if trimmed.starts_with("use ") && trimmed.contains(" as ") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Low,
                    category: IssueCategory::Style,
                    message: "Import alias (as) — rename makes grep and refactoring harder".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some(
                        "Use original name or add comment explaining why alias is needed".into(),
                    ),
                });
            }
        }
    }

    fn check_default_behavior_change(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.contains("impl Default for ") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::Correctness,
                    message: "Default impl changed — existing users will see different initial state".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Verify new default is semantically equivalent; document behavior change in changelog".into()),
                });
            }

            if trimmed.contains("fn ")
                && (trimmed.contains("default") || trimmed.contains("DEFAULT"))
            {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Low,
                    category: IssueCategory::Maintainability,
                    message: "Default value function — verify changing this does not affect existing callers".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Consider making default configurable (builder pattern) instead of hardcoded".into()),
                });
            }
        }
    }

    fn check_error_type_change(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.contains("type Error = ") || trimmed.contains("Error = ") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::Correctness,
                    message:
                        "Error type changed — all match arms and ? propagation sites may break"
                            .into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some(
                        "Add From impl for old error type, or use .map_err() in all call sites"
                            .into(),
                    ),
                });
            }

            if trimmed.contains("thiserror") || trimmed.contains("derive(Error") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Info,
                    category: IssueCategory::Maintainability,
                    message: "Error enum modified — verify #[source] and #[from] attributes for backward compat".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Adding new variant is safe; removing or renaming variant is breaking".into()),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_code_produces_report() {
        let result = CompatibilityAgent::review("fn foo() {}", "test.rs");
        assert_eq!(result.agent_name, "agent-compatibility");
        assert!(result.score >= 0.0 && result.score <= 1.0);
    }

    #[test]
    fn test_serde_skip_detected() {
        let code = r#"
#[derive(Serialize, Deserialize)]
struct Config {
    name: String,
    #[serde(skip)]
    temp: u32,
}"#;
        let result = CompatibilityAgent::review(code, "config.rs");
        let has_skip = result.issues.iter().any(|i| i.message.contains("skip"));
        assert!(has_skip, "Should detect serde skip: {:?}", result.issues);
    }

    #[test]
    fn test_hidden_side_effect_detected() {
        let code = r#"
fn log_and_return(x: i32) -> i32 {
    println!("value: {}", x);
    x
}"#;
        let result = CompatibilityAgent::review(code, "test.rs");
        let has_side_effect = result
            .issues
            .iter()
            .any(|i| i.message.contains("hidden side effect"));
        assert!(has_side_effect);
    }

    #[test]
    fn test_breaking_api_change_detected() {
        let code = r#"
pub fn compute_v2(data: &[u8]) -> u32 {
    42
}"#;
        let result = CompatibilityAgent::review(code, "test.rs");
        let has_versioned = result
            .issues
            .iter()
            .any(|i| i.message.contains("Versioned function"));
        assert!(has_versioned);
    }

    #[test]
    fn test_trait_contract_detected() {
        let code =
            "pub trait Storage { fn read(&self) -> Vec<u8>; fn write(&mut self, d: &[u8]); }";
        let result = CompatibilityAgent::review(code, "test.rs");
        let has_trait = result.issues.iter().any(|i| i.message.contains("Trait"));
        assert!(has_trait);
    }

    #[test]
    fn test_default_impl_detected() {
        let code = "impl Default for Config { fn default() -> Self { Self { port: 8080 } } }";
        let result = CompatibilityAgent::review(code, "test.rs");
        let has_default = result.issues.iter().any(|i| i.message.contains("Default"));
        assert!(has_default);
    }

    #[test]
    fn test_error_type_change_detected() {
        let code = "type Error = Box<dyn std::error::Error>;";
        let result = CompatibilityAgent::review(code, "test.rs");
        let has_error = result
            .issues
            .iter()
            .any(|i| i.message.contains("Error type"));
        assert!(has_error);
    }

    #[test]
    fn test_pub_struct_fields_detected() {
        let code = r#"
pub struct Config {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub retries: u32,
}"#;
        let result = CompatibilityAgent::review(code, "config.rs");
        let has_fields = result
            .issues
            .iter()
            .any(|i| i.message.contains("pub fields"));
        assert!(has_fields);
    }

    #[test]
    fn test_serde_rename_detected() {
        let code = r#"
#[derive(Serialize)]
struct Request {
    #[serde(rename = "user_id")]
    uid: String,
}"#;
        let result = CompatibilityAgent::review(code, "test.rs");
        let has_rename = result.issues.iter().any(|i| i.message.contains("rename"));
        assert!(has_rename);
    }
}
