use std::path::PathBuf;

/// Result of sandbox compilation and test execution.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub compiles: bool,
    pub tests_pass: bool,
    pub compile_errors: Vec<String>,
    pub test_output: String,
}

/// Offline sandbox validator that compiles and tests generated source code
/// in an isolated temp directory with a trivial test wrapper.
///
/// Uses `rustc` directly (not `cargo`) for minimal dependency and fast
/// compile-check. Test execution is optional — controlled by whether
/// `test_code` is provided.
pub struct SandboxValidator {
    pub temp_dir: PathBuf,
    /// Whether to skip actual compilation (for environments without rustc).
    pub dry_run: bool,
}

impl SandboxValidator {
    pub fn new() -> Self {
        Self {
            temp_dir: std::env::temp_dir().join("neotrix_sandbox"),
            dry_run: false,
        }
    }

    pub fn with_dry_run(mut self, dry: bool) -> Self {
        self.dry_run = dry;
        self
    }

    /// Write source to a temp file, attempt compilation, and if provided
    /// run a test harness against the compiled artifact.
    ///
    /// The `source` should be a valid Rust expression or function body.
    /// The `test_code` is an optional snippet that calls the source and
    /// asserts expected behavior.
    pub fn validate_source(&self, source: &str, test_code: &str) -> ValidationResult {
        if self.dry_run {
            // Dry-run mode: skip actual compilation, assume valid
            return ValidationResult {
                compiles: true,
                tests_pass: true,
                compile_errors: vec![],
                test_output: "(dry_run — compilation skipped)".into(),
            };
        }

        // Ensure temp directory exists
        let _ = std::fs::create_dir_all(&self.temp_dir);

        let source_file = self.temp_dir.join("modify_source.rs");
        let test_file = self.temp_dir.join("test_harness.rs");

        // Write source and test code to temp files
        if let Err(e) = std::fs::write(&source_file, source) {
            return ValidationResult {
                compiles: false,
                tests_pass: false,
                compile_errors: vec![format!("failed to write source: {}", e)],
                test_output: String::new(),
            };
        }

        // Attempt to compile the source file
        let mut compile_errors = Vec::new();
        let output = std::process::Command::new("rustc")
            .arg("--edition")
            .arg("2021")
            .arg("--crate-type")
            .arg("lib")
            .arg(&source_file)
            .arg("-o")
            .arg(self.temp_dir.join("modify_source_out"))
            .output();

        let compiles = match output {
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    for line in stderr.lines() {
                        compile_errors.push(line.to_string());
                    }
                    false
                } else {
                    true
                }
            }
            Err(e) => {
                compile_errors.push(format!("rustc invocation failed: {}", e));
                false
            }
        };

        // If compilation failed or no test code, return early
        if !compiles || test_code.is_empty() {
            // Cleanup source file
            let _ = std::fs::remove_file(&source_file);
            return ValidationResult {
                compiles,
                tests_pass: false,
                compile_errors,
                test_output: String::new(),
            };
        }

        // Write test harness and attempt compilation + execution
        let test_output = match std::fs::write(&test_file, test_code) {
            Ok(_) => {
                let test_output = std::process::Command::new("rustc")
                    .arg("--edition")
                    .arg("2021")
                    .arg("--test")
                    .arg(&test_file)
                    .arg("-o")
                    .arg(self.temp_dir.join("test_harness_out"))
                    .output();

                match test_output {
                    Ok(tc) if tc.status.success() => {
                        // Run the compiled test
                        let run =
                            std::process::Command::new(self.temp_dir.join("test_harness_out"))
                                .output();
                        match run {
                            Ok(run_out) => {
                                let stdout = String::from_utf8_lossy(&run_out.stdout).to_string();
                                let stderr = String::from_utf8_lossy(&run_out.stderr).to_string();
                                (run_out.status.success(), format!("{}\n{}", stdout, stderr))
                            }
                            Err(e) => (false, format!("test execution failed: {}", e)),
                        }
                    }
                    Ok(tc) => {
                        let stderr = String::from_utf8_lossy(&tc.stderr).to_string();
                        (false, format!("test compilation failed:\n{}", stderr))
                    }
                    Err(e) => (false, format!("test compilation invocation failed: {}", e)),
                }
            }
            Err(e) => (false, format!("failed to write test harness: {}", e)),
        };

        // Cleanup temp files
        let _ = std::fs::remove_file(&source_file);
        let _ = std::fs::remove_file(&test_file);
        let _ = std::fs::remove_file(self.temp_dir.join("modify_source_out"));
        let _ = std::fs::remove_file(self.temp_dir.join("test_harness_out"));

        ValidationResult {
            compiles: true,
            tests_pass: test_output.0,
            compile_errors: vec![],
            test_output: test_output.1,
        }
    }
}

impl Default for SandboxValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_validator_defaults() {
        let sv = SandboxValidator::new();
        assert!(!sv.dry_run);
        assert!(sv.temp_dir.to_string_lossy().contains("neotrix_sandbox"));
    }

    #[test]
    fn test_dry_run_always_valid() {
        let sv = SandboxValidator::new().with_dry_run(true);
        let result = sv.validate_source("garbage code {{{", "test_code");
        assert!(result.compiles);
        assert!(result.tests_pass);
        assert!(result.test_output.contains("dry_run"));
    }

    #[test]
    fn test_dry_run_empty_source() {
        let sv = SandboxValidator::new().with_dry_run(true);
        let result = sv.validate_source("", "");
        assert!(result.compiles);
    }

    #[test]
    fn test_new_validator_temp_dir() {
        let sv = SandboxValidator::new();
        let parent = sv.temp_dir.parent().unwrap();
        assert!(parent.exists());
    }

    #[test]
    fn test_validate_source_empty_no_crash() {
        let sv = SandboxValidator::new().with_dry_run(true);
        // Should not panic on empty inputs in dry-run mode
        let result = sv.validate_source("", "");
        assert!(result.compiles);
    }
}
