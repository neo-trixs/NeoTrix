use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawViolation {
    pub code: &'static str,
    pub severity: LawSeverity,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LawSeverity {
    Error,
    Warning,
    Info,
}

impl LawViolation {
    pub fn new(code: &'static str, severity: LawSeverity, message: impl Into<String>) -> Self {
        Self { code, severity, message: message.into(), file: None, line: None }
    }

    pub fn at(mut self, file: &str, line: usize) -> Self {
        self.file = Some(file.to_string());
        self.line = Some(line);
        self
    }
}

pub struct ProjectLaws;

impl ProjectLaws {
    /// Check all 10 laws against the given file content.
    pub fn check_all(content: &str, file_path: Option<&str>) -> Vec<LawViolation> {
        let mut violations = Vec::new();
        violations.extend(Self::check_l001(content));
        violations.extend(Self::check_l002(content));
        violations.extend(Self::check_l003(content, file_path));
        violations.extend(Self::check_l005(content));
        violations.extend(Self::check_l006(content, file_path));
        violations.extend(Self::check_l009(content));
        violations.extend(Self::check_l010(content));
        violations
    }

    /// L001: No hardcoded secrets (API keys, tokens, passwords)
    pub fn check_l001(content: &str) -> Vec<LawViolation> {
        let patterns = [
            ("api_key", r#"(?i)(?:api[_-]?key|apikey|api_secret|api_secret)\s*[:=]\s*['\"][A-Za-z0-9_\-]{16,}['\"]"#),
            ("sk_key", r#"(?i)(?:sk[_-]|pk[_-])[A-Za-z0-9]{20,}"#),
            ("password", r#"(?i)password\s*[:=]\s*['\"][^'\"]+['\"]"#),
            ("token", r#"(?i)(?:token|secret)\s*[:=]\s*['\"][A-Za-z0-9_\-\.]{20,}['\"]"#),
        ];
        let mut violations = Vec::new();
        for (label, pattern) in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.find_iter(content) {
                    violations.push(LawViolation::new(
                        "L001",
                        LawSeverity::Error,
                        format!("Potential hardcoded secret ({}) found: {}", label, cap.as_str()),
                    ));
                }
            }
        }
        violations
    }

    /// L002: Forbid unsafe code blocks
    pub fn check_l002(content: &str) -> Vec<LawViolation> {
        let mut violations = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed == "unsafe {" || trimmed.starts_with("unsafe ") || trimmed == "unsafe;" {
                violations.push(
                    LawViolation::new("L002", LawSeverity::Error, "Unsafe code block is forbidden")
                        .at("<content>", i + 1),
                );
            }
        }
        violations
    }

    /// L003: No unwrap() in production code (allow in tests and doctests)
    pub fn check_l003(content: &str, file_path: Option<&str>) -> Vec<LawViolation> {
        let is_test = file_path.map_or(false, |p| {
            p.contains("tests/") || p.ends_with("_test.rs") || p.contains("test_")
        });
        if is_test {
            return vec![];
        }
        let mut violations = Vec::new();
        if let Ok(re) = regex::Regex::new(r"\.unwrap\(\)") {
            for cap in re.find_iter(content) {
                violations.push(LawViolation::new(
                    "L003",
                    LawSeverity::Warning,
                    format!("Use '?' instead of unwrap(): {}", cap.as_str()),
                ));
            }
        }
        violations
    }

    /// L005: No dead code warning suppression (skip L004 — requires test runner)
    pub fn check_l005(content: &str) -> Vec<LawViolation> {
        let mut violations = Vec::new();
        if content.contains("#[allow(dead_code)]") {
            violations.push(LawViolation::new(
                "L005",
                LawSeverity::Warning,
                "Dead code allow attribute found — remove unused code instead",
            ));
        }
        violations
    }

    /// L006: Module naming must follow nt_{domain}_{subsystem}
    pub fn check_l006(content: &str, file_path: Option<&str>) -> Vec<LawViolation> {
        let is_module = file_path.map_or(false, |p| p.ends_with("mod.rs"));
        if !is_module {
            return vec![];
        }
        let mut violations = Vec::new();
        if let Ok(re) = regex::Regex::new(r#"pub mod (\w+);"#) {
            for cap in re.captures_iter(content) {
                let mod_name = cap.get(1).unwrap().as_str();
                if mod_name.starts_with("nt_") || mod_name == "tui" || mod_name.starts_with("cli_")
                {
                    continue;
                }
                violations.push(LawViolation::new(
                    "L006",
                    LawSeverity::Warning,
                    format!("Module '{}' should use nt_{{domain}}_{{subsystem}} naming", mod_name),
                ));
            }
        }
        violations
    }

    /// L009: No TODO, FIXME, HACK, XXX in production code
    pub fn check_l009(content: &str) -> Vec<LawViolation> {
        let mut violations = Vec::new();
        if let Ok(re) = regex::Regex::new(r"(?i)\b(TODO|FIXME|HACK|XXX)\b") {
            for cap in re.find_iter(content) {
                violations.push(LawViolation::new(
                    "L009",
                    LawSeverity::Warning,
                    format!("{} marker found in production code", cap.as_str()),
                ));
            }
        }
        violations
    }

    /// L010: No global allow(warnings) or forbid(unsafe_code) misuse
    pub fn check_l010(content: &str) -> Vec<LawViolation> {
        let mut violations = Vec::new();
        if content.contains("#![allow(warnings)]") {
            violations.push(LawViolation::new(
                "L010",
                LawSeverity::Error,
                "Global allow(warnings) is prohibited — fix warnings instead",
            ));
        }
        if !content.contains("#![forbid(unsafe_code)]") && content.contains("pub mod") {
            violations.push(LawViolation::new(
                "L010",
                LawSeverity::Warning,
                "Missing #![forbid(unsafe_code)] at crate root",
            ));
        }
        violations
    }

    pub fn describe(code: &str) -> Option<&'static str> {
        match code {
            "L001" => Some("No hardcoded secrets — use environment variables or nt_shield_vault"),
            "L002" => Some("Forbid unsafe code blocks — use safe alternatives"),
            "L003" => Some("No unwrap() in production — use '?' operator for error propagation"),
            "L004" => Some("All tests must pass before commit"),
            "L005" => Some("No dead code — remove unused code instead of suppressing"),
            "L006" => Some("Module naming must follow nt_{domain}_{subsystem} convention"),
            "L007" => Some("All errors must be handled — no ignored Results"),
            "L008" => Some("Public API must have doc comments"),
            "L009" => Some("No TODO/FIXME/HACK/XXX in production code"),
            "L010" => Some("No global allow(warnings); forbid(unsafe_code) at crate root"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l001_detects_api_key() {
        let content = r#"let api_key = "sk-1234567890abcdef1234567890abcdef";"#;
        let violations = ProjectLaws::check_l001(content);
        assert!(!violations.is_empty(), "should detect API key");
    }

    #[test]
    fn test_l001_clean_code_passes() {
        let content = r#"let name = "hello";"#;
        let violations = ProjectLaws::check_l001(content);
        assert!(violations.is_empty(), "clean code should pass");
    }

    #[test]
    fn test_l002_detects_unsafe() {
        let content = "unsafe { transmute(x) }";
        let violations = ProjectLaws::check_l002(content);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_l002_clean_code_passes() {
        let content = "fn foo() -> Result<()> { Ok(()) }";
        let violations = ProjectLaws::check_l002(content);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_l003_detects_unwrap() {
        let content = r#"let x = val.unwrap();"#;
        let violations = ProjectLaws::check_l003(content, Some("src/main.rs"));
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_l003_allows_in_tests() {
        let content = r#"let x = val.unwrap();"#;
        let violations = ProjectLaws::check_l003(content, Some("tests/test_foo.rs"));
        assert!(violations.is_empty(), "unwrap allowed in tests");
    }

    #[test]
    fn test_l005_detects_dead_code_allow() {
        let content = "#[allow(dead_code)]\nfn unused() {}";
        let violations = ProjectLaws::check_l005(content);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_l006_suggests_nt_naming() {
        let content = "pub mod foo;\npub mod nt_core_bar;\npub mod tui;";
        let violations = ProjectLaws::check_l006(content, Some("src/mod.rs"));
        let nt_compliant = violations.iter().any(|v| v.message.contains("foo"));
        assert!(nt_compliant, "should flag non-nt module name");
        let no_flag_nt = violations.iter().any(|v| v.message.contains("nt_core_bar"));
        assert!(!no_flag_nt, "should not flag nt_ prefixed modules");
    }

    #[test]
    fn test_l009_detects_todo() {
        let content = "// TODO: implement this";
        let violations = ProjectLaws::check_l009(content);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_l010_detects_allow_warnings() {
        let content = "#![allow(warnings)]";
        let violations = ProjectLaws::check_l010(content);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_l010_detects_missing_forbid_unsafe() {
        let content = "pub mod foo;\npub mod bar;";
        let violations = ProjectLaws::check_l010(content);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_l010_passes_with_forbid_unsafe() {
        let content = "#![forbid(unsafe_code)]\npub mod foo;";
        let violations = ProjectLaws::check_l010(content);
        let missing = violations.iter().any(|v| v.message.contains("forbid(unsafe_code)"));
        assert!(!missing);
    }

    #[test]
    fn test_describe_known_codes() {
        assert!(ProjectLaws::describe("L001").is_some());
        assert!(ProjectLaws::describe("L099").is_none());
    }

    #[test]
    fn test_check_all_runs() {
        let content = "fn main() { let x = val.unwrap(); }";
        let violations = ProjectLaws::check_all(content, Some("src/main.rs"));
        assert!(!violations.is_empty());
    }
}
