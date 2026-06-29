use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::core::nt_core_agent::sub_agent::{RecoveryStrategy, SubAgentCapability, SubTaskSpec};
use crate::neotrix::nt_mind::code_review::{IssueCategory, IssueSeverity, ReviewIssue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ReviewDimension {
    Security,
    Performance,
    Architecture,
    Correctness,
    TestCoverage,
    Maintainability,
    Style,
    ErrorHandling,
}

impl ReviewDimension {
    pub fn label(&self) -> &'static str {
        match self {
            ReviewDimension::Security => "Security",
            ReviewDimension::Performance => "Performance",
            ReviewDimension::Architecture => "Architecture",
            ReviewDimension::Correctness => "Correctness",
            ReviewDimension::TestCoverage => "Test Coverage",
            ReviewDimension::Maintainability => "Maintainability",
            ReviewDimension::Style => "Style",
            ReviewDimension::ErrorHandling => "Error Handling",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionReviewResult {
    pub dimension: ReviewDimension,
    pub reviewer_id: String,
    pub issues: Vec<ReviewIssue>,
    pub score: f64,
    pub summary: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraReviewReport {
    pub file: String,
    pub dimension_results: Vec<DimensionReviewResult>,
    pub consolidated_score: f64,
    pub critical_issues: Vec<ReviewIssue>,
    pub high_issues: Vec<ReviewIssue>,
    pub total_issues: usize,
    pub duration_ms: u64,
}

impl UltraReviewReport {
    pub fn all_issues(&self) -> Vec<&ReviewIssue> {
        let mut all = Vec::new();
        for dr in &self.dimension_results {
            for issue in &dr.issues {
                all.push(issue);
            }
        }
        all
    }

    pub fn by_severity(&self, sev: IssueSeverity) -> Vec<&ReviewIssue> {
        self.all_issues()
            .into_iter()
            .filter(|i| i.severity == sev)
            .collect()
    }

    pub fn by_category(&self, cat: IssueCategory) -> Vec<&ReviewIssue> {
        self.all_issues()
            .into_iter()
            .filter(|i| i.category == cat)
            .collect()
    }

    pub fn blockers(&self) -> Vec<&ReviewIssue> {
        self.by_severity(IssueSeverity::Critical)
    }

    pub fn summary(&self) -> String {
        format!(
            "UltraReview: {} dimensions, score {:.1}%, {} issues ({} critical, {} high)",
            self.dimension_results.len(),
            self.consolidated_score * 100.0,
            self.total_issues,
            self.critical_issues.len(),
            self.high_issues.len(),
        )
    }

    /// Generate auto-fix subtasks from critical/high issues.
    /// Each issue becomes a SubTaskSpec with capability mapped from its category.
    pub fn generate_fix_tasks(&self, file_name: &str) -> Vec<SubTaskSpec> {
        let mut tasks = Vec::new();

        let all_priority = self
            .critical_issues
            .iter()
            .chain(self.high_issues.iter())
            .take(10);

        for (idx, issue) in all_priority.enumerate() {
            let capability = match issue.category {
                IssueCategory::Security => SubAgentCapability::SecurityAuditor,
                IssueCategory::Correctness => SubAgentCapability::Tester,
                IssueCategory::Performance => SubAgentCapability::Coder,
                IssueCategory::Maintainability => SubAgentCapability::Documenter,
                IssueCategory::Testing => SubAgentCapability::Tester,
                IssueCategory::Style => SubAgentCapability::Coder,
                _ => SubAgentCapability::Coder,
            };
            let suggestion = issue.suggestion.as_deref().unwrap_or("Review and fix");
            tasks.push(SubTaskSpec {
                id: idx as usize + 1,
                description: format!(
                    "[AutoFix] {} - {} (line {})",
                    issue.category.label(),
                    suggestion,
                    issue.line.map_or("N/A".into(), |l| l.to_string())
                ),
                capability,
                constraints: vec![format!("file:{}", file_name)],
                expected_artifacts: vec!["fix_diff".into()],
                recovery: RecoveryStrategy::default(),
            });
        }
        tasks
    }

    /// Whether the report needs human approval before auto-fix execution.
    pub fn needs_approval(&self) -> bool {
        !self.critical_issues.is_empty() || self.consolidated_score < 0.5
    }
}

pub struct UltraReviewEngine;

impl UltraReviewEngine {
    pub fn review(code: &str, file: &str, dimensions: &[ReviewDimension]) -> UltraReviewReport {
        let start = Instant::now();
        let mut dimension_results = Vec::new();
        let mut total_issues = 0;

        for (i, &dim) in dimensions.iter().enumerate() {
            let dim_start = Instant::now();
            let (issues, score) = Self::run_dimension_review(code, dim);
            let elapsed = dim_start.elapsed().as_millis() as u64;
            dimension_results.push(DimensionReviewResult {
                dimension: dim,
                reviewer_id: format!("reviewer-{}", i),
                issues,
                score,
                summary: format!(
                    "{} review complete: score {:.1}%",
                    dim.label(),
                    score * 100.0
                ),
                duration_ms: elapsed,
            });
            total_issues += dimension_results.last().map_or(0, |r| r.issues.len());
        }

        let consolidated_score = if dimension_results.is_empty() {
            1.0
        } else {
            dimension_results.iter().map(|r| r.score).sum::<f64>() / dimension_results.len() as f64
        };

        let critical_issues: Vec<ReviewIssue> = dimension_results
            .iter()
            .flat_map(|r| {
                r.issues
                    .iter()
                    .filter(|i| i.severity == IssueSeverity::Critical)
                    .cloned()
            })
            .collect();
        let high_issues: Vec<ReviewIssue> = dimension_results
            .iter()
            .flat_map(|r| {
                r.issues
                    .iter()
                    .filter(|i| i.severity == IssueSeverity::High)
                    .cloned()
            })
            .collect();

        let elapsed = start.elapsed().as_millis() as u64;

        UltraReviewReport {
            file: file.to_string(),
            dimension_results,
            consolidated_score,
            critical_issues,
            high_issues,
            total_issues,
            duration_ms: elapsed,
        }
    }

    pub fn run_dimension_review(code: &str, dim: ReviewDimension) -> (Vec<ReviewIssue>, f64) {
        let mut issues = Vec::new();
        match dim {
            ReviewDimension::Security => {
                Self::check_command_injection(code, &mut issues);
                Self::check_secrets(code, &mut issues);
                Self::check_unsafe_blocks(code, &mut issues);
                Self::check_sql_injection(code, &mut issues);
            }
            ReviewDimension::Performance => {
                Self::check_allocation_hot_paths(code, &mut issues);
                Self::check_nplus_one(code, &mut issues);
                Self::check_clone_on_copy(code, &mut issues);
            }
            ReviewDimension::Architecture => {
                Self::check_circular_deps(code, &mut issues);
                Self::check_god_class(code, &mut issues);
                Self::check_module_cohesion(code, &mut issues);
            }
            ReviewDimension::Correctness => {
                Self::check_unwrap(code, &mut issues);
                Self::check_panic(code, &mut issues);
                Self::check_index_out_of_bounds(code, &mut issues);
            }
            ReviewDimension::TestCoverage => {
                Self::check_missing_tests(code, &mut issues);
                Self::check_test_quality(code, &mut issues);
            }
            ReviewDimension::Maintainability => {
                Self::check_magic_numbers(code, &mut issues);
                Self::check_long_functions(code, &mut issues);
                Self::check_commented_code(code, &mut issues);
            }
            ReviewDimension::Style => {
                Self::check_naming_conventions(code, &mut issues);
                Self::check_unused_imports(code, &mut issues);
            }
            ReviewDimension::ErrorHandling => {
                Self::check_unwrap(code, &mut issues);
                Self::check_ignored_results(code, &mut issues);
                Self::check_missing_error_propagation(code, &mut issues);
            }
        }
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
        (issues, score)
    }

    fn check_unwrap(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains(".unwrap()") && !line.trim().starts_with("//") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::ErrorHandling,
                    message: "Use of .unwrap() may cause panic".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Replace with .expect(\"msg\") or ? operator".into()),
                });
            }
        }
    }

    fn check_panic(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains("panic!(") && !line.trim().starts_with("//") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::ErrorHandling,
                    message: "panic!() causes unrecoverable crash".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Replace with returning Result type".into()),
                });
            }
        }
    }

    fn check_unsafe_blocks(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed == "unsafe {" || trimmed.starts_with("unsafe {") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::Security,
                    message: "Unsafe block bypasses memory safety guarantees".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some(
                        "Minimize unsafe blocks; add SAFETY comment justifying each invariant"
                            .into(),
                    ),
                });
            }
        }
    }

    fn check_command_injection(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            if (line.contains("sh -c") || line.contains("bash -c") || line.contains("cmd /c"))
                && !line.trim().starts_with("//")
            {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::Security,
                    message: "[OWASP A05:2025] Command injection risk via shell invocation".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some(
                        "Use std::process::Command with array args instead of shell string".into(),
                    ),
                });
            }
        }
    }

    fn check_secrets(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let lower = line.to_lowercase();
            if (lower.contains("api_key")
                || lower.contains("password")
                || lower.contains("secret")
                || lower.contains("token"))
                && (line.contains('=') || line.contains(':'))
                && !line.trim().starts_with("//")
                && !line.contains("env!(")
                && !line.contains("std::env")
            {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::Security,
                    message: "[OWASP A04:2025] Possible hardcoded secret".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Use environment variables or a secrets manager".into()),
                });
            }
        }
    }

    fn check_sql_injection(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let lower = line.to_lowercase();
            if (lower.contains("format!(") || lower.contains("format!("))
                && (lower.contains("select")
                    || lower.contains("insert")
                    || lower.contains("delete")
                    || lower.contains("update"))
                && !line.trim().starts_with("//")
            {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::Security,
                    message: "[OWASP A01:2025] Possible SQL injection via string formatting".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Use parameterized queries or query builder".into()),
                });
            }
        }
    }

    fn check_allocation_hot_paths(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.contains("Vec::new")
                || trimmed.contains("String::new")
                || trimmed.contains("HashMap::new"))
                && !trimmed.starts_with("//")
            {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Low,
                    category: IssueCategory::Performance,
                    message: "Allocation on hot path may impact performance".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some(
                        "Consider pre-allocating with_capacity or using a pool".into(),
                    ),
                });
            }
        }
    }

    fn check_nplus_one(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains("for")
                && (line.contains(".iter()") || line.contains("in &"))
                && line.contains(".unwrap()")
            {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::Performance,
                    message: "N+1 query pattern detected".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Batch queries or use join".into()),
                });
            }
        }
    }

    fn check_clone_on_copy(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains(".clone()") && !line.trim().starts_with("//") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Info,
                    category: IssueCategory::Performance,
                    message: "Unnecessary clone may impact performance".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Consider borrowing instead of cloning".into()),
                });
            }
        }
    }

    fn check_circular_deps(code: &str, issues: &mut Vec<ReviewIssue>) {
        if code.contains("use crate::")
            && code.lines().filter(|l| l.contains("use crate::")).count() > 10
        {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Architecture,
                message: "High number of internal dependencies may indicate tight coupling".into(),
                line: None,
                suggestion: Some("Consider interface-based abstraction to reduce coupling".into()),
            });
        }
    }

    fn check_god_class(code: &str, issues: &mut Vec<ReviewIssue>) {
        let lines = code.lines().count();
        let fn_count = code.matches("fn ").count();
        if lines > 300 && fn_count > 10 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Architecture,
                message: format!(
                    "Large file ({} lines, {} functions) - possible god class/function",
                    lines, fn_count
                ),
                line: None,
                suggestion: Some("Split into smaller focused modules".into()),
            });
        }
    }

    fn check_module_cohesion(code: &str, issues: &mut Vec<ReviewIssue>) {
        if code.matches("pub fn").count() > 15 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Low,
                category: IssueCategory::Architecture,
                message: "Large public API surface - consider if module has single responsibility"
                    .into(),
                line: None,
                suggestion: Some("Review module boundaries".into()),
            });
        }
    }

    fn check_index_out_of_bounds(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains('[')
                && line.contains(']')
                && (line.contains('+') || line.contains('-'))
            {
                if line.contains('[') && (line.contains("i+") || line.contains("i -")) {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::High,
                        category: IssueCategory::Correctness,
                        message: "Manual index arithmetic may cause out-of-bounds access".into(),
                        line: Some((i + 1) as u32),
                        suggestion: Some("Use .get() for safe indexing".into()),
                    });
                }
            }
        }
    }

    fn check_missing_tests(code: &str, issues: &mut Vec<ReviewIssue>) {
        if !code.contains("#[cfg(test)]") && !code.contains("#[test]") {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Testing,
                message: "Module has no test module".into(),
                line: None,
                suggestion: Some("Add #[cfg(test)] module with unit tests".into()),
            });
        }
    }

    fn check_test_quality(code: &str, issues: &mut Vec<ReviewIssue>) {
        let test_count = code.matches("#[test]").count();
        let fn_count = code.matches("fn ").count();
        if fn_count > 5 && test_count == 0 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Testing,
                message: format!("{} functions but 0 tests", fn_count),
                line: None,
                suggestion: Some("Add tests covering edge cases and error paths".into()),
            });
        }
    }

    fn check_magic_numbers(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }
            if let Some(n) = Self::extract_magic_number(trimmed) {
                if n > 0.0 && n < 1.0 {
                    continue;
                }
                if n == 0.0 || n == 1.0 {
                    continue;
                }
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Low,
                    category: IssueCategory::Style,
                    message: format!("Magic number {} - extract to named constant", n),
                    line: Some((i + 1) as u32),
                    suggestion: Some("Define as const with descriptive name".into()),
                });
            }
        }
    }

    fn extract_magic_number(line: &str) -> Option<f64> {
        let line = line.split("//").next().unwrap_or(line);
        let line = line.split('#').next().unwrap_or(line);
        for word in line.split_whitespace() {
            let cleaned: String = word
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect();
            if cleaned.is_empty() {
                continue;
            }
            let rest: String = word
                .chars()
                .filter(|c| !c.is_ascii_digit() && *c != '.' && *c != '-')
                .collect();
            let has_only_punct_suffix = rest.chars().all(|c| c.is_ascii_punctuation());
            if !has_only_punct_suffix && cleaned != word {
                continue;
            }
            if let Ok(n) = cleaned.parse::<f64>() {
                if (n - n.round()).abs() < f64::EPSILON {
                    let ni = n as i64;
                    if ni > 1
                        && ni < 100
                        && !line.contains("let")
                        && !line.contains("const")
                        && !line.contains("static")
                    {
                        return Some(n);
                    }
                }
            }
        }
        None
    }

    fn check_long_functions(code: &str, issues: &mut Vec<ReviewIssue>) {
        let mut in_fn = false;
        let mut fn_start = 0;
        let mut fn_lines = 0;
        let mut fn_name = String::new();
        for (i, line) in code.lines().enumerate() {
            if line.trim().starts_with("fn ") && line.trim().ends_with('{') {
                in_fn = true;
                fn_start = i + 1;
                fn_lines = 1;
                fn_name = line
                    .trim()
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("unknown")
                    .to_string();
            } else if in_fn {
                fn_lines += 1;
                if line.trim() == "}" || fn_lines > 60 {
                    if fn_lines > 60 {
                        issues.push(ReviewIssue {
                            severity: IssueSeverity::Medium,
                            category: IssueCategory::Maintainability,
                            message: format!("Function '{}' is {} lines long", fn_name, fn_lines),
                            line: Some(fn_start as u32),
                            suggestion: Some("Break into smaller helper functions".into()),
                        });
                    }
                    in_fn = false;
                    fn_lines = 0;
                }
            }
        }
    }

    fn check_commented_code(code: &str, issues: &mut Vec<ReviewIssue>) {
        let mut comment_block_lines = 0;
        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("// ") && trimmed.len() > 10 {
                let content = &trimmed[3..];
                if content.contains("fn ")
                    || content.contains("if ")
                    || content.contains("for ")
                    || content.contains("let ")
                    || content.contains("return ")
                {
                    comment_block_lines += 1;
                } else {
                    comment_block_lines = 0;
                }
            } else {
                if comment_block_lines > 3 {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::Low,
                        category: IssueCategory::Maintainability,
                        message: "Commented-out code detected".into(),
                        line: None,
                        suggestion: Some(
                            "Remove dead commented code; use version control for history".into(),
                        ),
                    });
                }
                comment_block_lines = 0;
            }
        }
    }

    fn check_naming_conventions(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains("pub fn ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(fn_word) = parts.iter().position(|&w| w == "fn") {
                    if let Some(name) = parts.get(fn_word + 1) {
                        let name = name.trim_end_matches('(');
                        if name.contains('_') && name.chars().any(|c| c.is_uppercase()) {
                            issues.push(ReviewIssue {
                                severity: IssueSeverity::Info,
                                category: IssueCategory::Style,
                                message: format!(
                                    "Function '{}' uses mixed naming convention",
                                    name
                                ),
                                line: Some((i + 1) as u32),
                                suggestion: Some("Use snake_case for Rust functions".into()),
                            });
                        }
                    }
                }
            }
        }
    }

    fn check_unused_imports(code: &str, issues: &mut Vec<ReviewIssue>) {
        let import_count = code
            .lines()
            .filter(|l| l.trim().starts_with("use "))
            .count();
        let use_count = code.matches("use ").count();
        if import_count > 20 && use_count < import_count / 2 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Info,
                category: IssueCategory::Style,
                message: "High number of imports relative to usage - possible unused imports"
                    .into(),
                line: None,
                suggestion: Some("Run cargo fix or review imports".into()),
            });
        }
    }

    fn check_ignored_results(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("let _ = ") && !trimmed.starts_with("//") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Low,
                    category: IssueCategory::ErrorHandling,
                    message: "Result ignored with let _ = ".into(),
                    line: Some((i + 1) as u32),
                    suggestion: Some(
                        "Handle the error explicitly or add comment explaining why ignored".into(),
                    ),
                });
            }
        }
    }

    fn check_missing_error_propagation(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") && trimmed.contains("-> Result<") {
                let mut has_question_mark = false;
                for following_line in code.lines().skip(i + 1) {
                    let fl = following_line.trim();
                    if fl == "}" {
                        break;
                    }
                    if fl.contains('?') {
                        has_question_mark = true;
                    }
                }
                if !has_question_mark {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::Medium,
                        category: IssueCategory::ErrorHandling,
                        message: "Function returns Result but never uses ? operator".into(),
                        line: Some((i + 1) as u32),
                        suggestion: Some(
                            "Use ? to propagate errors or verify this is intentional".into(),
                        ),
                    });
                }
            }
        }
    }
}

pub fn default_review_dimensions() -> Vec<ReviewDimension> {
    vec![
        ReviewDimension::Security,
        ReviewDimension::Performance,
        ReviewDimension::Architecture,
        ReviewDimension::Correctness,
        ReviewDimension::TestCoverage,
        ReviewDimension::Maintainability,
        ReviewDimension::Style,
        ReviewDimension::ErrorHandling,
    ]
}

pub fn quick_review_dimensions() -> Vec<ReviewDimension> {
    vec![ReviewDimension::Security, ReviewDimension::Correctness]
}
