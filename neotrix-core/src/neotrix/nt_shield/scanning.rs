use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityLevel {
    Warning,
    Error,
    Critical,
}

impl SeverityLevel {
    pub fn label(&self) -> &'static str {
        match self {
            SeverityLevel::Warning => "Warning",
            SeverityLevel::Error => "Error",
            SeverityLevel::Critical => "Critical",
        }
    }

    pub fn threshold_score(&self) -> u8 {
        match self {
            SeverityLevel::Warning => 1,
            SeverityLevel::Error => 2,
            SeverityLevel::Critical => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanFinding {
    pub id: String,
    pub severity: SeverityLevel,
    pub title: String,
    pub description: String,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
    pub rule_id: String,
    pub remediation: String,
}

#[derive(Debug, Clone)]
pub struct ScanRule {
    pub id: String,
    pub name: String,
    pub severity: SeverityLevel,
    pub pattern: String,
    pub check_fn: fn(&str) -> bool,
    pub remediation: String,
}

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub rules: Vec<ScanRule>,
    pub scan_paths: Vec<PathBuf>,
    pub exclude_patterns: Vec<String>,
    pub fail_on: SeverityLevel,
}

#[derive(Debug, Clone)]
pub struct SecurityReport {
    pub total_findings: usize,
    pub critical_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub findings: Vec<ScanFinding>,
    pub passed: bool,
    pub scan_duration: Duration,
}

fn check_hardcoded_secrets(content: &str) -> bool {
    let patterns = [
        r#"(?i)(?:api[_-]?key|apikey)\s*[:=]\s*['"][a-zA-Z0-9_\-]{16,}['"]"#,
        r#"(?i)(?:secret|token|password|passwd)\s*[:=]\s*['"][a-zA-Z0-9_\-!@#$%^&*()]{8,}['"]"#,
        r#"(?i)(?:aws_access_key_id|aws_secret_access_key)\s*[:=]\s*['"][a-zA-Z0-9/+=]{16,}['"]"#,
        r#"(?i)(?:ghp_|gho_|ghu_|ghs_|ghr_)[a-zA-Z0-9_]{36,}"#,
        r#"(?i)sk-[a-zA-Z0-9]{32,}"#,
        r#"(?i)(?:-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----)"#,
    ];
    let re = patterns
        .iter()
        .map(|p| regex::Regex::new(p).expect("hardcoded secret pattern"))
        .collect::<Vec<_>>();
    re.iter().any(|r| r.is_match(content))
}

fn check_unwrap(content: &str) -> bool {
    let re = regex::Regex::new(r"\b\.unwrap\(\)").expect("hardcoded unwrap pattern");
    let count = re.find_iter(content).count();
    count > 0
}

fn check_banned_functions(content: &str) -> bool {
    let re = regex::Regex::new(r"\bCommand\s*::\s*new\b").expect("hardcoded command pattern");
    re.is_match(content)
}

fn check_deny_unsafe(content: &str) -> bool {
    let re = regex::Regex::new(r"#!\[forbid\(unsafe_code\)\]").expect("hardcoded forbid pattern");
    !re.is_match(content)
}

fn check_large_unwrap_count(content: &str) -> bool {
    let re = regex::Regex::new(r"\b\.unwrap\(\)").expect("hardcoded unwrap pattern");
    let count = re.find_iter(content).count();
    count > 1000
}

fn check_todo_comments(content: &str) -> bool {
    let re = regex::Regex::new(r#"(?i)(?://|#|/\*|<!--)\s*(TODO|FIXME|HACK|XXX|BUG)"#).expect("hardcoded todo pattern");
    re.is_match(content)
}

fn check_missing_error_handling(content: &str) -> bool {
    let ok_re = regex::Regex::new(r"\b\.ok\(\)").expect("hardcoded .ok() pattern");
    let ok_count = ok_re.find_iter(content).count();
    let unwrap_re = regex::Regex::new(r"\b\.unwrap\(\)").expect("hardcoded unwrap pattern");
    let unwrap_count = unwrap_re.find_iter(content).count();
    ok_count > 10 && (unwrap_count as f64 / ok_count as f64) > 0.5
}

fn check_print_statements(content: &str) -> bool {
    let re = regex::Regex::new(r#"(?i)\b(println!|dbg!|eprintln!|print!)"#).expect("hardcoded print pattern");
    let count = re.find_iter(content).count();
    count > 2
}

impl ScanConfig {
    pub fn default_ci() -> Self {
        Self {
            rules: vec![
                ScanRule {
                    id: "SEC-001".to_string(),
                    name: "Hardcoded Secrets".to_string(),
                    severity: SeverityLevel::Critical,
                    pattern: "*.rs".to_string(),
                    check_fn: check_hardcoded_secrets,
                    remediation: "Move secrets to environment variables or a vault service. Use std::env::var() or a key management system.".to_string(),
                },
                ScanRule {
                    id: "SEC-002".to_string(),
                    name: "Unsafe .unwrap() Calls".to_string(),
                    severity: SeverityLevel::Warning,
                    pattern: "*.rs".to_string(),
                    check_fn: check_unwrap,
                    remediation: "Replace .unwrap() with proper error handling: match, ?, map_err, or context().".to_string(),
                },
                ScanRule {
                    id: "SEC-003".to_string(),
                    name: "Banned Functions".to_string(),
                    severity: SeverityLevel::Critical,
                    pattern: "*.rs".to_string(),
                    check_fn: check_banned_functions,
                    remediation: "Avoid std::process::Command with user input. Use a sandboxed execution environment or validate all inputs.".to_string(),
                },
                ScanRule {
                    id: "SEC-004".to_string(),
                    name: "Missing #[deny(unsafe_code)]".to_string(),
                    severity: SeverityLevel::Error,
                    pattern: "lib.rs".to_string(),
                    check_fn: check_deny_unsafe,
                    remediation: "Add #![forbid(unsafe_code)] to lib.rs to prevent unsafe code usage.".to_string(),
                },
                ScanRule {
                    id: "SEC-005".to_string(),
                    name: "Excessive Unwrap Count".to_string(),
                    severity: SeverityLevel::Error,
                    pattern: "*.rs".to_string(),
                    check_fn: check_large_unwrap_count,
                    remediation: "Reduce .unwrap() usage below 1000. Each .unwrap() is a potential panic point.".to_string(),
                },
                ScanRule {
                    id: "SEC-006".to_string(),
                    name: "Technical Debt Markers".to_string(),
                    severity: SeverityLevel::Warning,
                    pattern: "*.rs".to_string(),
                    check_fn: check_todo_comments,
                    remediation: "Resolve TODO/FIXME/HACK items before merge or file a tracking issue.".to_string(),
                },
                ScanRule {
                    id: "SEC-007".to_string(),
                    name: "Missing Error Handling".to_string(),
                    severity: SeverityLevel::Warning,
                    pattern: "*.rs".to_string(),
                    check_fn: check_missing_error_handling,
                    remediation: "Use ? operator or proper match on Result types instead of .ok() + .unwrap() chains.".to_string(),
                },
                ScanRule {
                    id: "SEC-008".to_string(),
                    name: "Debug Print Statements".to_string(),
                    severity: SeverityLevel::Warning,
                    pattern: "*.rs".to_string(),
                    check_fn: check_print_statements,
                    remediation: "Remove println!/dbg! statements from production code. Use the log crate instead.".to_string(),
                },
            ],
            scan_paths: vec![PathBuf::from("src")],
            exclude_patterns: vec![
                "target/".to_string(),
                ".git/".to_string(),
                "**/tests/**".to_string(),
            ],
            fail_on: SeverityLevel::Error,
        }
    }
}

pub struct SecurityScanner {
    pub config: ScanConfig,
    pub findings: Vec<ScanFinding>,
    finding_ids: HashSet<String>,
}

impl SecurityScanner {
    pub fn new(config: ScanConfig) -> Self {
        Self {
            config,
            findings: Vec::new(),
            finding_ids: HashSet::new(),
        }
    }

    pub fn default_ci_config() -> Self {
        Self::new(ScanConfig::default_ci())
    }

    pub fn run_scan(&mut self) -> Vec<ScanFinding> {
        self.findings.clear();
        self.finding_ids.clear();

        let paths = self.config.scan_paths.clone();
        for path in &paths {
            if !path.exists() {
                continue;
            }
            if path.is_dir() {
                self.scan_directory(path);
            } else if path.is_file() {
                self.scan_file(path);
            }
        }

        self.findings.clone()
    }

    fn scan_directory(&mut self, dir: &PathBuf) {
        let exclude = self.config.exclude_patterns.clone();
        let walker = walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_entry(move |e| {
                let p = e.path().to_string_lossy().to_string();
                !exclude.iter().any(|ex| p.contains(ex.as_str()))
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                self.scan_file(&entry.path().to_path_buf());
            }
        }
    }

    fn scan_file(&mut self, path: &PathBuf) {
        let path_str = path.to_string_lossy().to_string();
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let matched_rules: Vec<&ScanRule> = self
            .config
            .rules
            .iter()
            .filter(|rule| {
                let pattern_matched = if rule.pattern == "*" || rule.pattern == "*.rs" {
                    path_str.ends_with(".rs")
                } else if rule.pattern == "lib.rs" {
                    path_str.ends_with("lib.rs")
                } else if rule.pattern.starts_with('*') {
                    path_str.ends_with(&rule.pattern[1..])
                } else {
                    path_str.contains(&rule.pattern)
                };
                pattern_matched && (rule.check_fn)(&content)
            })
            .collect();

        for rule in matched_rules {
            let finding_id = format!("{}-{}", rule.id, path_str);
            if !self.finding_ids.insert(finding_id.clone()) {
                continue;
            }

            self.findings.push(ScanFinding {
                id: finding_id,
                severity: rule.severity.clone(),
                title: rule.name.clone(),
                description: format!("{} detected pattern in {}", rule.name, path_str),
                file_path: Some(path_str.clone()),
                line_number: None,
                rule_id: rule.id.clone(),
                remediation: rule.remediation.clone(),
            });
        }
    }

    pub fn report(&self) -> SecurityReport {
        let total_findings = self.findings.len();
        let critical_count = self
            .findings
            .iter()
            .filter(|f| f.severity == SeverityLevel::Critical)
            .count();
        let error_count = self
            .findings
            .iter()
            .filter(|f| f.severity == SeverityLevel::Error)
            .count();
        let warning_count = self
            .findings
            .iter()
            .filter(|f| f.severity == SeverityLevel::Warning)
            .count();

        SecurityReport {
            total_findings,
            critical_count,
            error_count,
            warning_count,
            findings: self.findings.clone(),
            passed: self.passes(),
            scan_duration: Duration::ZERO,
        }
    }

    pub fn passes(&self) -> bool {
        let threshold = self.config.fail_on.threshold_score();
        !self
            .findings
            .iter()
            .any(|f| f.severity.threshold_score() >= threshold)
    }

    pub fn add_rule(&mut self, rule: ScanRule) {
        self.config.rules.push(rule);
    }

    pub fn add_exclude_pattern(&mut self, pattern: String) {
        self.config.exclude_patterns.push(pattern);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temp_rs_file(content: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("test_file.rs");
        std::fs::write(&path, content).expect("failed to write test file");
        (dir, path)
    }

    #[test]
    fn test_default_ci_config() {
        let scanner = SecurityScanner::default_ci_config();
        assert_eq!(scanner.config.rules.len(), 8);
        assert!(scanner.config.rules.iter().any(|r| r.id == "SEC-001"));
        assert!(scanner.config.rules.iter().any(|r| r.id == "SEC-008"));
    }

    #[test]
    fn test_scan_safe_file_zero_findings() {
        let config = ScanConfig {
            rules: vec![
                ScanRule {
                    id: "TEST-001".to_string(),
                    name: "No Secrets".to_string(),
                    severity: SeverityLevel::Critical,
                    pattern: "*.rs".to_string(),
                    check_fn: |_| false,
                    remediation: "".to_string(),
                },
            ],
            scan_paths: vec![],
            exclude_patterns: vec![],
            fail_on: SeverityLevel::Error,
        };
        let mut scanner = SecurityScanner::new(config);
        let findings = scanner.run_scan();
        assert!(findings.is_empty());
        assert!(scanner.passes());
    }

    #[test]
    fn test_detect_hardcoded_secret() {
        let content = r#"
            fn connect() {
                let api_key = "sk-1234567890abcdef1234567890abcdef";
            }
        "#;
        assert!(check_hardcoded_secrets(content));
    }

    #[test]
    fn test_detect_unwrap() {
        let content = r#"
            fn parse() {
                let val = some_result.unwrap();
            }
        "#;
        assert!(check_unwrap(content));
    }

    #[test]
    fn test_report_format() {
        let config = ScanConfig {
            rules: vec![],
            scan_paths: vec![],
            exclude_patterns: vec![],
            fail_on: SeverityLevel::Error,
        };
        let scanner = SecurityScanner::new(config);
        let report = scanner.report();
        assert_eq!(report.total_findings, 0);
        assert_eq!(report.critical_count, 0);
        assert_eq!(report.error_count, 0);
        assert_eq!(report.warning_count, 0);
        assert!(report.passed);
        assert_eq!(report.findings.len(), 0);
    }

    #[test]
    fn test_passes_when_no_critical() {
        let findings = vec![
            ScanFinding {
                id: "1".to_string(),
                severity: SeverityLevel::Warning,
                title: "test".to_string(),
                description: "".to_string(),
                file_path: None,
                line_number: None,
                rule_id: "R1".to_string(),
                remediation: "".to_string(),
            },
        ];
        let config = ScanConfig {
            rules: vec![],
            scan_paths: vec![],
            exclude_patterns: vec![],
            fail_on: SeverityLevel::Error,
        };
        let scanner = SecurityScanner {
            config,
            findings,
            finding_ids: HashSet::new(),
        };
        assert!(scanner.passes());
    }

    #[test]
    fn test_fails_when_critical_exists() {
        let findings = vec![
            ScanFinding {
                id: "1".to_string(),
                severity: SeverityLevel::Critical,
                title: "secret".to_string(),
                description: "".to_string(),
                file_path: None,
                line_number: None,
                rule_id: "R1".to_string(),
                remediation: "".to_string(),
            },
        ];
        let config = ScanConfig {
            rules: vec![],
            scan_paths: vec![],
            exclude_patterns: vec![],
            fail_on: SeverityLevel::Error,
        };
        let scanner = SecurityScanner {
            config,
            findings,
            finding_ids: HashSet::new(),
        };
        assert!(!scanner.passes());
    }

    #[test]
    fn test_add_custom_rule() {
        let mut scanner = SecurityScanner::default_ci_config();
        assert_eq!(scanner.config.rules.len(), 8);
        scanner.add_rule(ScanRule {
            id: "CUSTOM-001".to_string(),
            name: "Custom Check".to_string(),
            severity: SeverityLevel::Error,
            pattern: "*.rs".to_string(),
            check_fn: |_| true,
            remediation: "Fix it".to_string(),
        });
        assert_eq!(scanner.config.rules.len(), 9);
    }

    #[test]
    fn test_exclude_pattern_works() {
        let mut scanner = SecurityScanner::default_ci_config();
        scanner.add_exclude_pattern("safe_dir/".to_string());
        assert!(scanner.config.exclude_patterns.iter().any(|p| p == "safe_dir/"));
    }

    #[test]
    fn test_multiple_rule_matching() {
        let content = r#"
            fn bad() {
                let key = "sk-abcdef1234567890abcdef1234567890";
                let val = opt.unwrap();
                println!("key = {}", key);
            }
        "#;
        let (_dir, path) = create_temp_rs_file(content);
        let config = ScanConfig {
            rules: vec![
                ScanRule {
                    id: "S1".to_string(),
                    name: "Secret Check".to_string(),
                    severity: SeverityLevel::Critical,
                    pattern: "*.rs".to_string(),
                    check_fn: check_hardcoded_secrets,
                    remediation: "".to_string(),
                },
                ScanRule {
                    id: "S2".to_string(),
                    name: "Unwrap Check".to_string(),
                    severity: SeverityLevel::Warning,
                    pattern: "*.rs".to_string(),
                    check_fn: check_unwrap,
                    remediation: "".to_string(),
                },
            ],
            scan_paths: vec![path],
            exclude_patterns: vec![],
            fail_on: SeverityLevel::Error,
        };
        let mut scanner = SecurityScanner::new(config);
        let findings = scanner.run_scan();
        assert!(findings.len() >= 2);
    }

    #[test]
    fn test_deduplication() {
        let content = r#"let api_key = "sk-abcdef1234567890abcdef1234567890";"#;
        let (_dir, path) = create_temp_rs_file(content);
        let config = ScanConfig {
            rules: vec![
                ScanRule {
                    id: "S1".to_string(),
                    name: "Secret".to_string(),
                    severity: SeverityLevel::Critical,
                    pattern: "*.rs".to_string(),
                    check_fn: check_hardcoded_secrets,
                    remediation: "".to_string(),
                },
            ],
            scan_paths: vec![path.clone(), path],
            exclude_patterns: vec![],
            fail_on: SeverityLevel::Error,
        };
        let mut scanner = SecurityScanner::new(config);
        let findings = scanner.run_scan();
        let secret_findings: Vec<_> = findings.iter().filter(|f| f.rule_id == "S1").collect();
        assert_eq!(secret_findings.len(), 1);
    }

    #[test]
    fn test_severity_ordering() {
        let mut levels = vec![
            SeverityLevel::Warning,
            SeverityLevel::Critical,
            SeverityLevel::Error,
        ];
        levels.sort();
        assert_eq!(
            levels,
            vec![
                SeverityLevel::Warning,
                SeverityLevel::Error,
                SeverityLevel::Critical,
            ]
        );
    }

    #[test]
    fn test_severity_label() {
        assert_eq!(SeverityLevel::Warning.label(), "Warning");
        assert_eq!(SeverityLevel::Error.label(), "Error");
        assert_eq!(SeverityLevel::Critical.label(), "Critical");
    }

    #[test]
    fn test_severity_threshold_score() {
        assert_eq!(SeverityLevel::Warning.threshold_score(), 1);
        assert_eq!(SeverityLevel::Error.threshold_score(), 2);
        assert_eq!(SeverityLevel::Critical.threshold_score(), 3);
    }

    #[test]
    fn test_detect_todo_comment() {
        let content = "// TODO: fix this later";
        assert!(check_todo_comments(content));

        let content = "/* FIXME: critical bug */";
        assert!(check_todo_comments(content));

        let content = "let x = 1; // this is fine";
        assert!(!check_todo_comments(content));
    }

    #[test]
    fn test_detect_print_statements() {
        let content = r#"
            fn main() {
                println!("hello");
                dbg!(x);
                eprintln!("error");
                print!("no newline");
            }
        "#;
        assert!(check_print_statements(content));
    }

    #[test]
    fn test_no_false_positive_print() {
        let content = r#"
            fn main() {
                let x = 1;
                log::info!("normal log");
            }
        "#;
        assert!(!check_print_statements(content));
    }

    #[test]
    fn test_detect_banned_function() {
        let content = r#"let cmd = std::process::Command::new("ls");"#;
        assert!(check_banned_functions(content));
    }

    #[test]
    fn test_scan_with_actual_file_triggers_rules() {
        let content = r#"
            fn main() {
                // TODO: refactor this
                let secret = "sk-test1234567890abcdef1234567890abcdef";
                let x = opt.unwrap();
                println!("got: {}", x);
            }
        "#;
        let (_dir, path) = create_temp_rs_file(content);
        let config = ScanConfig {
            rules: vec![
                ScanRule {
                    id: "S1".to_string(),
                    name: "Secrets".to_string(),
                    severity: SeverityLevel::Critical,
                    pattern: "*.rs".to_string(),
                    check_fn: check_hardcoded_secrets,
                    remediation: "".to_string(),
                },
                ScanRule {
                    id: "S2".to_string(),
                    name: "TODO".to_string(),
                    severity: SeverityLevel::Warning,
                    pattern: "*.rs".to_string(),
                    check_fn: check_todo_comments,
                    remediation: "".to_string(),
                },
            ],
            scan_paths: vec![path],
            exclude_patterns: vec![],
            fail_on: SeverityLevel::Error,
        };
        let mut scanner = SecurityScanner::new(config);
        let findings = scanner.run_scan();
        assert_eq!(findings.len(), 2);
        assert!(!scanner.passes());
    }
}
