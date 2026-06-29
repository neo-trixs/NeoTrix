/// StaticCodeExecDetector — 静态代码执行检测器
///
/// 对标 OWASP ASI05 (Unexpected Code Execution)
/// 对标 CrewAI CVE-2026-2275 (CodeInterpreterTool sandbox escape)
/// 对标 AutoGen CVE-2026-7662 (exec() on LLM-generated code)
/// 对标 CWE-94 (Code Injection), CWE-95 (Eval Injection)
use std::sync::OnceLock;

static DETECTION_RULES: OnceLock<Vec<DetectionRule>> = OnceLock::new();

fn rules() -> &'static Vec<DetectionRule> {
    DETECTION_RULES.get_or_init(|| {
        vec![
            DetectionRule::new(
                "eval-call",
                r"\beval\s*\(",
                Severity::Critical,
                "eval() on untrusted input leads to RCE (CWE-95)",
            ),
            DetectionRule::new(
                "exec-call",
                r"\bexec\s*\(",
                Severity::Critical,
                "exec() executes arbitrary code (CWE-94)",
            ),
            DetectionRule::new(
                "compile-call",
                r"\bcompile\s*\(",
                Severity::Critical,
                "compile() on untrusted input is arbitrary code execution",
            ),
            DetectionRule::new(
                "importlib-import",
                r"importlib\.import_module\s*\(",
                Severity::High,
                "dynamic import without allowlist leads to RCE (CrewAI CVE-2026-5446)",
            ),
            DetectionRule::new(
                "importlib-load",
                r"importlib\.load_module\s*\(",
                Severity::High,
                "dynamic module loading without allowlist",
            ),
            DetectionRule::new(
                "pickle-loads",
                r"pickle\.loads?\s*\(",
                Severity::Critical,
                "pickle deserialization leads to RCE (CWE-502)",
            ),
            DetectionRule::new(
                "allow-dangerous-true",
                r"allow_dangerous_code\s*=\s*True",
                Severity::Critical,
                "hardcoded allow_dangerous_code=True (Langflow CVE-2026-27966)",
            ),
            DetectionRule::new(
                "allow-dangerous-true-rs",
                r"allow_dangerous_code\s*:\s*true",
                Severity::Critical,
                "hardcoded allow_dangerous_code: true",
            ),
            DetectionRule::new(
                "subprocess-shell",
                r"subprocess\.run\s*\(.*shell\s*=\s*True",
                Severity::Critical,
                "subprocess with shell=True allows shell injection",
            ),
            DetectionRule::new(
                "os-system",
                r"\bos\.system\s*\(",
                Severity::High,
                "os.system() executes shell commands",
            ),
            DetectionRule::new(
                "os-popen",
                r"\bos\.popen\s*\(",
                Severity::High,
                "os.popen() executes shell commands",
            ),
            DetectionRule::new(
                "classloader-load",
                r"ClassLoader\.loadClass\s*\(",
                Severity::Critical,
                "unvalidated ClassLoader.loadClass() is arbitrary class instantiation (CWE-470)",
            ),
            DetectionRule::new(
                "request-typosquat",
                r"\bimport request\b",
                Severity::Medium,
                "typosquat: 'import request' vs legitimate 'import requests'",
            ),
            DetectionRule::new(
                "jinja2-template",
                r"jinja2\.Template\s*\(",
                Severity::High,
                "Jinja2 Template() without sandbox leads to SSTI",
            ),
        ]
    })
}

#[derive(Debug, Clone)]
pub struct DetectionRule {
    pub id: &'static str,
    pub pattern: &'static str,
    pub severity: Severity,
    pub description: &'static str,
}

impl DetectionRule {
    pub const fn new(
        id: &'static str,
        pattern: &'static str,
        severity: Severity,
        description: &'static str,
    ) -> Self {
        Self {
            id,
            pattern,
            severity,
            description,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::High => write!(f, "HIGH"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::Low => write!(f, "LOW"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectionFinding {
    pub rule_id: String,
    pub severity: Severity,
    pub description: String,
    pub line: usize,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct DetectionReport {
    pub findings: Vec<DetectionFinding>,
    pub critical_count: usize,
    pub high_count: usize,
}

impl DetectionReport {
    pub fn is_pass(&self) -> bool {
        self.critical_count == 0 && self.high_count == 0
    }
}

/// Statically analyze Rust/Python code for dangerous execution patterns
pub fn analyze_static_code(code: &str) -> DetectionReport {
    let mut findings = Vec::new();

    for rule in rules() {
        let re = match regex::Regex::new(rule.pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for (line_idx, line) in code.lines().enumerate() {
            if re.is_match(line) {
                findings.push(DetectionFinding {
                    rule_id: rule.id.to_string(),
                    severity: rule.severity,
                    description: rule.description.to_string(),
                    line: line_idx + 1,
                    snippet: line.trim().to_string(),
                });
            }
        }
    }

    let critical_count = findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .count();
    let high_count = findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .count();

    DetectionReport {
        findings,
        critical_count,
        high_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_eval() {
        let code = r#"
def process(input):
    result = eval(input)
    return result
"#;
        let report = analyze_static_code(code);
        assert!(report.critical_count >= 1);
        assert!(report.findings.iter().any(|f| f.rule_id == "eval-call"));
    }

    #[test]
    fn test_detects_exec() {
        let code = r#"
exec(malicious_code)
"#;
        let report = analyze_static_code(code);
        assert!(report.critical_count >= 1);
    }

    #[test]
    fn test_detects_compile() {
        let code = r#"
fn run() {
    let c = compile(source, "", "exec");
}
"#;
        let report = analyze_static_code(code);
        assert!(report.critical_count >= 1);
    }

    #[test]
    fn test_detects_allow_dangerous() {
        let code = r#"
allow_dangerous_code = True
"#;
        let report = analyze_static_code(code);
        assert!(report.critical_count >= 1);
    }

    #[test]
    fn test_clean_code_passes() {
        let code = r#"
fn safe_add(a: i32, b: i32) -> i32 { a + b }
"#;
        let report = analyze_static_code(code);
        assert!(report.critical_count == 0);
        assert!(report.high_count == 0);
    }

    #[test]
    fn test_detects_request_typosquat() {
        let code = r#"import request"#;
        let report = analyze_static_code(code);
        assert!(report
            .findings
            .iter()
            .any(|f| f.rule_id == "request-typosquat"));
    }

    #[test]
    fn test_detects_importlib() {
        let code = r#"
module = importlib.import_module("os")
"#;
        let report = analyze_static_code(code);
        assert!(report.high_count >= 1);
    }
}
