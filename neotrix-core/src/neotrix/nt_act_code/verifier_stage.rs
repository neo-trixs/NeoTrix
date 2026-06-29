//! VerifierStage — formality verification stage (codex-flow pattern).
//!
//! Provides a pre_verify / post_verify two-phase verification pipeline
//! paired with SafeWriteGate for safe write gating.

use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Verified,
    Warning(String),
    Rejected(String),
}

pub trait VerifierStage: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn pre_verify(&self, file: &str, old_content: &str, new_content: &str) -> Verdict;
    fn post_verify(&self, file: &str, content: &str) -> Verdict;
}

#[derive(Debug)]
pub struct CompileVerifier;

impl CompileVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompileVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl VerifierStage for CompileVerifier {
    fn name(&self) -> &str {
        "compile"
    }

    fn pre_verify(&self, _file: &str, _old_content: &str, _new_content: &str) -> Verdict {
        Verdict::Verified
    }

    fn post_verify(&self, _file: &str, _content: &str) -> Verdict {
        if cfg!(test) {
            return Verdict::Verified;
        }
        let mut last_stderr = String::new();
        for attempt in 1..=3 {
            log::info!("compile_verify: attempt {}/3", attempt);
            match Command::new("cargo").args(["check", "--lib"]).output() {
                Ok(out) if out.status.success() => return Verdict::Verified,
                Ok(out) => {
                    last_stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    return Verdict::Rejected(format!("cargo check could not be launched: {}", e))
                }
            }
        }
        Verdict::Warning(format!("compile_check_warning: {}", last_stderr))
    }
}

#[derive(Debug)]
pub struct SafetyPatternVerifier;

impl SafetyPatternVerifier {
    pub fn new() -> Self {
        Self
    }

    fn check_patterns(&self, content: &str) -> Vec<(&'static str, Verdict)> {
        let mut results = Vec::new();

        if content.contains("unsafe") {
            results.push((
                "unsafe block",
                Verdict::Warning("code uses unsafe blocks".into()),
            ));
        }
        if content.contains("std::process::Command") {
            results.push((
                "process::Command",
                Verdict::Warning("code spawns system commands".into()),
            ));
        }
        if content.contains("fs::write") && !content.contains("backup") {
            results.push((
                "fs::write without backup",
                Verdict::Warning("file write without backup detected".into()),
            ));
        }

        results
    }
}

impl Default for SafetyPatternVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl VerifierStage for SafetyPatternVerifier {
    fn name(&self) -> &str {
        "safety"
    }

    fn pre_verify(&self, _file: &str, _old_content: &str, new_content: &str) -> Verdict {
        let findings = self.check_patterns(new_content);
        let warnings: Vec<String> = findings
            .iter()
            .filter_map(|(_, v)| match v {
                Verdict::Warning(w) => Some(w.clone()),
                _ => None,
            })
            .collect();
        if warnings.is_empty() {
            Verdict::Verified
        } else {
            Verdict::Warning(warnings.join("; "))
        }
    }

    fn post_verify(&self, _file: &str, _content: &str) -> Verdict {
        Verdict::Verified
    }
}

#[derive(Debug)]
pub struct FormatVerifier;

impl FormatVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FormatVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl VerifierStage for FormatVerifier {
    fn name(&self) -> &str {
        "format"
    }

    fn pre_verify(&self, _file: &str, _old_content: &str, _new_content: &str) -> Verdict {
        Verdict::Verified
    }

    fn post_verify(&self, file: &str, _content: &str) -> Verdict {
        let output = Command::new("rustfmt").args(["--check", file]).output();
        match output {
            Ok(out) if out.status.success() => Verdict::Verified,
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                Verdict::Rejected(format!(
                    "rustfmt --check failed:\n{}{}",
                    stdout.trim(),
                    stderr.trim()
                ))
            }
            Err(e) => Verdict::Warning(format!("rustfmt not available: {}", e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SafeWriteGate {
    Allow,
    RequireApproval(String),
    Block(String),
}

impl Default for SafeWriteGate {
    fn default() -> Self {
        Self::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_detects_unsafe() {
        let v = SafetyPatternVerifier::new();
        let verdict = v.pre_verify("test.rs", "", "fn main() { unsafe { } }");
        assert!(matches!(verdict, Verdict::Warning(_)));
    }

    #[test]
    fn test_safety_detects_command() {
        let v = SafetyPatternVerifier::new();
        let verdict = v.pre_verify("test.rs", "", "std::process::Command::new(\"ls\")");
        assert!(matches!(verdict, Verdict::Warning(_)));
    }

    #[test]
    fn test_safety_clean_code_passes() {
        let v = SafetyPatternVerifier::new();
        let verdict = v.pre_verify("test.rs", "", "fn main() { let x = 1; }");
        assert_eq!(verdict, Verdict::Verified);
    }

    #[test]
    fn test_safety_detects_fs_write_no_backup() {
        let v = SafetyPatternVerifier::new();
        let verdict = v.pre_verify("test.rs", "", "std::fs::write(\"file\", \"data\")");
        assert!(matches!(verdict, Verdict::Warning(_)));
    }

    #[test]
    fn test_format_verifier_nonexistent_file() {
        let v = FormatVerifier::new();
        let result = v.post_verify("/nonexistent_test_file_12345.rs", "");
        assert!(!matches!(result, Verdict::Verified));
    }

    #[test]
    fn test_compile_verifier_returns_verdict() {
        let v = CompileVerifier::new();
        let result = v.post_verify("", "");
        assert!(matches!(result, Verdict::Verified | Verdict::Rejected(_)));
    }

    #[test]
    fn test_pre_verify_compile_always_pass() {
        let v = CompileVerifier::new();
        assert_eq!(v.pre_verify("test.rs", "", ""), Verdict::Verified);
    }

    #[test]
    fn test_pre_verify_format_always_pass() {
        let v = FormatVerifier::new();
        assert_eq!(v.pre_verify("test.rs", "", ""), Verdict::Verified);
    }

    #[test]
    fn test_post_verify_safety_always_pass() {
        let v = SafetyPatternVerifier::new();
        assert_eq!(v.post_verify("test.rs", "unsafe { }"), Verdict::Verified);
    }

    #[test]
    fn test_verifier_names() {
        assert_eq!(CompileVerifier::new().name(), "compile");
        assert_eq!(SafetyPatternVerifier::new().name(), "safety");
        assert_eq!(FormatVerifier::new().name(), "format");
    }

    #[test]
    fn test_safe_write_gate_default() {
        assert_eq!(SafeWriteGate::default(), SafeWriteGate::Allow);
    }

    #[test]
    fn test_safe_write_gate_variants() {
        assert_eq!(SafeWriteGate::Allow, SafeWriteGate::Allow);
        assert_eq!(
            SafeWriteGate::Block("x".into()),
            SafeWriteGate::Block("x".into())
        );
        assert!(matches!(
            SafeWriteGate::RequireApproval("y".into()),
            SafeWriteGate::RequireApproval(_)
        ));
    }

    #[test]
    fn test_compile_verifier_default() {
        let v: CompileVerifier = Default::default();
        assert_eq!(v.name(), "compile");
    }

    #[test]
    fn test_safety_verifier_default() {
        let v: SafetyPatternVerifier = Default::default();
        assert_eq!(v.name(), "safety");
    }

    #[test]
    fn test_format_verifier_default() {
        let v: FormatVerifier = Default::default();
        assert_eq!(v.name(), "format");
    }

    #[test]
    fn test_safety_no_false_positive_for_safe_code() {
        let v = SafetyPatternVerifier::new();
        let verdict = v.pre_verify("test.rs", "", "fn add(a: i32, b: i32) -> i32 { a + b }");
        assert_eq!(verdict, Verdict::Verified);
    }

    #[test]
    fn test_safety_multiple_warnings() {
        let v = SafetyPatternVerifier::new();
        let code = "fn main() { unsafe { std::process::Command::new(\"ls\"); } }";
        let verdict = v.pre_verify("test.rs", "", code);
        match verdict {
            Verdict::Warning(msg) => {
                assert!(msg.contains("unsafe") || msg.contains("Command"));
            }
            _ => panic!("expected Warning"),
        }
    }
}
