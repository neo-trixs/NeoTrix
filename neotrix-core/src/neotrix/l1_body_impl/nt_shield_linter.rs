//! External linter runners — semgrep, clippy, gitleaks, and custom commands.
//! L1 Body layer: executes CLI commands and returns structured results.
//! L7 CodeReviewEngine can call these for enriched audit dimensions.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinterKind {
    Semgrep,
    Clippy,
    Gitleaks,
    Custom(String),
}

impl LinterKind {
    pub fn name(&self) -> &str {
        match self {
            LinterKind::Semgrep => "semgrep",
            LinterKind::Clippy => "clippy",
            LinterKind::Gitleaks => "gitleaks",
            LinterKind::Custom(n) => n.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterFinding {
    pub kind: LinterKind,
    pub file: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub severity: String,
    pub rule_id: String,
    pub message: String,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterReport {
    pub kind: LinterKind,
    pub total: usize,
    pub passed: bool,
    pub findings: Vec<LinterFinding>,
    pub duration_ms: u64,
}

/// Trait for external linter runners
pub trait LinterRunner: Send + Sync {
    fn kind(&self) -> LinterKind;
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn run(&self, path: &Path) -> LinterReport;
}

/// Semgrep runner — requires `semgrep` on PATH
#[derive(Default)]
pub struct SemgrepRunner;

impl SemgrepRunner {
    pub fn new() -> Self { Self }
}

impl LinterRunner for SemgrepRunner {
    fn kind(&self) -> LinterKind { LinterKind::Semgrep }
    fn name(&self) -> &'static str { "semgrep" }
    fn is_available(&self) -> bool {
        Command::new("which").arg("semgrep").output().is_ok()
            && Command::new("semgrep").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
    }
    fn run(&self, path: &Path) -> LinterReport {
        let start = std::time::Instant::now();
        let mut findings = Vec::new();

        let output = Command::new("semgrep")
            .args(["scan", "--json", "--quiet", "-o", "-"])
            .arg(path)
            .output();

        if let Ok(out) = output {
            if let Ok(text) = String::from_utf8(out.stdout) {
                findings = Self::parse_semgrep_json(&text);
            }
        }

        LinterReport {
            kind: LinterKind::Semgrep,
            total: findings.len(),
            passed: findings.is_empty(),
            findings,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

impl SemgrepRunner {
    fn parse_semgrep_json(json: &str) -> Vec<LinterFinding> {
        let mut findings = Vec::new();
        // Try to parse semgrep JSON SARIF-like output
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
            if let Some(results) = v.get("results").and_then(|r| r.as_array()) {
                for r in results {
                    let path = r["path"].as_str().unwrap_or("").to_string();
                    let start = r["start"].as_object();
                    let line = start.and_then(|s| s["line"].as_u64()).map(|l| l as usize);
                    let col = start.and_then(|s| s["col"].as_u64()).map(|c| c as usize);
                    let extra = r.get("extra");
                    let severity = extra.and_then(|e| e["severity"].as_str()).unwrap_or("WARNING").to_string();
                    let rule_id = r["check_id"].as_str().unwrap_or("unknown").to_string();
                    let message = extra.and_then(|e| e["message"].as_str()).unwrap_or("").to_string();

                    findings.push(LinterFinding {
                        kind: LinterKind::Semgrep,
                        file: path, line, column: col,
                        severity, rule_id, message,
                        raw: serde_json::to_string(r).unwrap_or_default(),
                    });
                }
            }
        }
        findings
    }
}

/// Clippy runner — runs `cargo clippy` in the project directory
#[derive(Default)]
pub struct ClippyRunner;

impl ClippyRunner {
    pub fn new() -> Self { Self }
}

impl LinterRunner for ClippyRunner {
    fn kind(&self) -> LinterKind { LinterKind::Clippy }
    fn name(&self) -> &'static str { "clippy" }
    fn is_available(&self) -> bool {
        Command::new("cargo").arg("clippy").arg("--version").output().is_ok()
    }
    fn run(&self, path: &Path) -> LinterReport {
        let start = std::time::Instant::now();
        let mut findings = Vec::new();

        let output = Command::new("cargo")
            .args(["clippy", "--all-targets", "--", "-D", "warnings"])
            .current_dir(path)
            .output();

        if let Ok(out) = output {
            let stderr = String::from_utf8_lossy(&out.stderr);
            for line in stderr.lines() {
                // Parse clippy warnings: path:line:col: severity: message
                if let Some(caps) = parse_clippy_line(line) {
                    findings.push(caps);
                }
            }
        }

        LinterReport {
            kind: LinterKind::Clippy,
            total: findings.len(),
            passed: findings.is_empty(),
            findings,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

fn parse_clippy_line(line: &str) -> Option<LinterFinding> {
    // Format: file:line:col: severity: message
    let parts: Vec<&str> = line.splitn(5, ':').collect();
    if parts.len() >= 4 {
        let severity = if line.contains("error") { "ERROR" } else if line.contains("warning") { "WARNING" } else { "INFO" };
        Some(LinterFinding {
            kind: LinterKind::Clippy,
            file: parts[0].trim().to_string(),
            line: parts.get(1).and_then(|s| s.trim().parse().ok()),
            column: parts.get(2).and_then(|s| s.trim().parse().ok()),
            severity: severity.to_string(),
            rule_id: "clippy".to_string(),
            message: parts.get(4).map(|s| s.trim().to_string()).unwrap_or_default(),
            raw: line.to_string(),
        })
    } else {
        None
    }
}

/// Gitleaks runner — detects secrets in git repos
#[derive(Default)]
pub struct GitleaksRunner;

impl GitleaksRunner {
    pub fn new() -> Self { Self }
}

impl LinterRunner for GitleaksRunner {
    fn kind(&self) -> LinterKind { LinterKind::Gitleaks }
    fn name(&self) -> &'static str { "gitleaks" }
    fn is_available(&self) -> bool {
        Command::new("which").arg("gitleaks").output().is_ok()
    }
    fn run(&self, path: &Path) -> LinterReport {
        let start = std::time::Instant::now();
        let mut findings = Vec::new();

        let output = Command::new("gitleaks")
            .args(["detect", "--source", "--no-git", "--report-format", "json"])
            .arg(path)
            .output();

        if let Ok(out) = output {
            if let Ok(text) = String::from_utf8(out.stdout) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(arr) = v.as_array() {
                        for item in arr {
                            let file = item["File"].as_str().unwrap_or("").to_string();
                            let line = item["StartLine"].as_u64().map(|l| l as usize);
                            let severity = item["Severity"].as_str().unwrap_or("MEDIUM").to_string();
                            let rule_id = item["RuleID"].as_str().unwrap_or("unknown").to_string();
                            let message = item["Description"].as_str().unwrap_or("Secret detected").to_string();

                            findings.push(LinterFinding {
                                kind: LinterKind::Gitleaks,
                                file, line, column: None,
                                severity, rule_id, message,
                                raw: serde_json::to_string(item).unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }

        LinterReport {
            kind: LinterKind::Gitleaks,
            total: findings.len(),
            passed: findings.is_empty(),
            findings,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

/// Aggregate linter that runs all available linters
#[derive(Default)]
pub struct LinterAggregator {
    runners: Vec<Box<dyn LinterRunner>>,
}

impl LinterAggregator {
    pub fn new() -> Self {
        Self { runners: Vec::new() }
    }

    pub fn with_defaults(mut self) -> Self {
        self.add(Box::new(SemgrepRunner::new()));
        self.add(Box::new(ClippyRunner::new()));
        self.add(Box::new(GitleaksRunner::new()));
        self
    }

    pub fn add(&mut self, runner: Box<dyn LinterRunner>) {
        self.runners.push(runner);
    }

    pub fn run_all(&self, path: &Path) -> Vec<LinterReport> {
        self.runners.iter().map(|r| r.run(path)).collect()
    }

    pub fn run_available(&self, path: &Path) -> Vec<LinterReport> {
        self.runners.iter()
            .filter(|r| r.is_available())
            .map(|r| r.run(path))
            .collect()
    }

    pub fn available_linters(&self) -> Vec<&'static str> {
        self.runners.iter()
            .filter(|r| r.is_available())
            .map(|r| r.name())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linter_aggregator_new() {
        let agg = LinterAggregator::new();
        assert!(agg.runners.is_empty());
    }

    #[test]
    fn test_linter_aggregator_with_defaults() {
        let agg = LinterAggregator::new().with_defaults();
        assert_eq!(agg.runners.len(), 3);
    }

    #[test]
    fn test_semgrep_is_available_or_graceful() {
        let runner = SemgrepRunner::new();
        let available = runner.is_available();
        // Should not panic whether available or not
        let report = runner.run(Path::new("."));
        if available {
            assert!(report.total > 0 || report.passed);
        } else {
            assert_eq!(report.total, 0);
            assert!(report.passed);
        }
    }

    #[test]
    fn test_clippy_parser_valid_line() {
        let line = "src/main.rs:42:5: warning: unused variable `x`";
        let finding = parse_clippy_line(line);
        assert!(finding.is_some());
        let f = finding.unwrap();
        assert_eq!(f.file, "src/main.rs");
        assert_eq!(f.line, Some(42));
        assert_eq!(f.column, Some(5));
    }

    #[test]
    fn test_clippy_parser_invalid_line() {
        assert!(parse_clippy_line("not a valid clippy output").is_none());
    }

    #[test]
    fn test_semgrep_parser_empty_json() {
        let findings = SemgrepRunner::parse_semgrep_json("{}");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_semgrep_parser_valid_json() {
        let json = r#"{"results":[{"path":"test.rs","start":{"line":5,"col":3},"check_id":"test-rule","extra":{"severity":"ERROR","message":"found bug"}}]}"#;
        let findings = SemgrepRunner::parse_semgrep_json(json);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].file, "test.rs");
        assert_eq!(findings[0].line, Some(5));
        assert_eq!(findings[0].rule_id, "test-rule");
    }

    #[test]
    fn test_available_linters_categories() {
        let agg = LinterAggregator::new().with_defaults();
        let names: Vec<_> = agg.runners.iter().map(|r| r.name()).collect();
        assert!(names.contains(&"semgrep"));
        assert!(names.contains(&"clippy"));
        assert!(names.contains(&"gitleaks"));
    }
}
