use std::process::Command;

/// 变更验证结果
#[derive(Clone, Debug)]
pub struct VerificationResult {
    pub success: bool,
    pub compile_errors: Vec<String>,
    pub compile_warnings: Vec<String>,
    pub test_output: String,
    pub test_summary: String,
}

/// 变更验证器 — 通过真实编译和测试验证架构变更
pub struct ChangeVerifier;

impl Default for ChangeVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeVerifier {
    pub fn new() -> Self {
        Self
    }

    /// 运行 cargo check --lib 验证编译
    pub fn verify_compilation(&self, project_root: &str) -> VerificationResult {
        let output = self.run_cargo_check(project_root);
        let (errors, warnings) = self.parse_output(&output);

        VerificationResult {
            success: errors.is_empty(),
            compile_errors: errors,
            compile_warnings: warnings,
            test_output: String::new(),
            test_summary: String::new(),
        }
    }

    /// 运行 cargo test --lib 验证测试
    pub fn verify_tests(&self, project_root: &str, test_filter: &str) -> VerificationResult {
        let output = self.run_cargo_test(project_root, test_filter);
        let (errors, warnings) = self.parse_output(&output);
        let summary = self.extract_test_summary(&output);

        VerificationResult {
            success: errors.is_empty() && !summary.contains("FAILED"),
            compile_errors: errors,
            compile_warnings: warnings,
            test_output: output,
            test_summary: summary,
        }
    }

    /// 全流程验证：编译 + 测试
    pub fn verify_all(&self, project_root: &str) -> VerificationResult {
        let compile = self.verify_compilation(project_root);
        if !compile.success {
            return compile;
        }
        self.verify_tests(project_root, "")
    }

    fn run_cargo_check(&self, root: &str) -> String {
        let output = Command::new("cargo")
            .args(["check", "--lib", "-p", "neotrix"])
            .current_dir(root)
            .output();
        match output {
            Ok(o) => {
                let mut result = String::from_utf8_lossy(&o.stdout).to_string();
                result.push_str(&String::from_utf8_lossy(&o.stderr));
                result
            }
            Err(e) => format!("cargo check failed to execute: {}", e),
        }
    }

    fn run_cargo_test(&self, root: &str, filter: &str) -> String {
        let mut args = vec!["test", "--lib", "-p", "neotrix"];
        if !filter.is_empty() {
            args.push("--");
            args.push(filter);
        }
        let output = Command::new("cargo")
            .args(&args)
            .current_dir(root)
            .output();
        match output {
            Ok(o) => {
                let mut result = String::from_utf8_lossy(&o.stdout).to_string();
                result.push_str(&String::from_utf8_lossy(&o.stderr));
                result
            }
            Err(e) => format!("cargo test failed to execute: {}", e),
        }
    }

    fn parse_output(&self, output: &str) -> (Vec<String>, Vec<String>) {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        for line in output.lines() {
            if line.starts_with("error") || line.contains("error[E") {
                errors.push(line.to_string());
            } else if line.starts_with("warning") || line.contains("warning:") {
                warnings.push(line.to_string());
            }
        }
        (errors, warnings)
    }

    fn extract_test_summary(&self, output: &str) -> String {
        for line in output.lines().rev() {
            let trimmed = line.trim();
            if trimmed.starts_with("test result:") || trimmed.contains("FAILED") || trimmed.contains("test passed") {
                return trimmed.to_string();
            }
        }
        "no test summary found".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_output_errors() {
        let verifier = ChangeVerifier::new();
        let output = "error[E0308]: mismatched types\n   --> file.rs:10:5\nwarning: unused variable: `x`\nsome other text";
        let (errors, warnings) = verifier.parse_output(output);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("E0308"));
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("unused variable"));
    }

    #[test]
    fn test_empty_output() {
        let verifier = ChangeVerifier::new();
        let (errors, warnings) = verifier.parse_output("everything is fine");
        assert!(errors.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_extract_test_summary() {
        let verifier = ChangeVerifier::new();
        let output = "running 3 tests\ntest result: ok. 3 passed; 0 failed";
        let summary = verifier.extract_test_summary(output);
        assert!(summary.contains("test result"));
    }

    #[test]
    fn test_parse_output_with_mixed_content() {
        let verifier = ChangeVerifier::new();
        let output = "Compiling foo v0.1.0\nerror[E0308]: mismatched types\n   --> src/lib.rs:1:1\nwarning: unused import\n";
        let (errors, warnings) = verifier.parse_output(output);
        assert_eq!(errors.len(), 1);
        assert!(!warnings.is_empty());
    }
}
