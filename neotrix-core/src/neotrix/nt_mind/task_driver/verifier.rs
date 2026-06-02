use std::process::Command;

#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct CargoVerifier {
    pub project_dir: String,
}

impl CargoVerifier {
    pub fn new(project_dir: &str) -> Self {
        Self { project_dir: project_dir.to_string() }
    }

    pub fn check_lib(&self) -> VerifyResult {
        run_cargo(&self.project_dir, &["check", "--lib", "-p", "neotrix"])
    }

    pub fn check_all_features(&self) -> VerifyResult {
        run_cargo(&self.project_dir, &["check", "--all-features", "--lib", "-p", "neotrix"])
    }

    pub fn test_lib(&self) -> VerifyResult {
        run_cargo(&self.project_dir, &["test", "--lib", "-p", "neotrix"])
    }

    pub fn verify_all(&self) -> Vec<(&'static str, VerifyResult)> {
        let mut results = Vec::new();
        results.push(("cargo check --lib", self.check_lib()));
        if results.last().expect("result").1.passed {
            results.push(("cargo check --all-features", self.check_all_features()));
        }
        if results.last().expect("result").1.passed {
            results.push(("cargo test --lib", self.test_lib()));
        }
        results
    }

    pub fn summary(&self) -> String {
        let results = self.verify_all();
        let mut s = String::new();
        for (name, result) in &results {
            let icon = if result.passed { "✅" } else { "❌" };
            s.push_str(&format!("{} {}: {} error(s), {} warning(s)\n",
                icon, name, result.errors.len(), result.warnings.len()));
            if !result.passed {
                for err in &result.errors {
                    s.push_str(&format!("  {}\n", err));
                }
            }
        }
        s
    }

    pub fn all_passed(&self) -> bool {
        self.verify_all().iter().all(|(_, r)| r.passed)
    }
}

fn run_cargo(dir: &str, args: &[&str]) -> VerifyResult {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(dir)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let combined = format!("{}{}", stdout, stderr);

            let errors: Vec<String> = combined.lines()
                .filter(|l| l.starts_with("error"))
                .map(|l| l.to_string())
                .collect();

            let warnings: Vec<String> = combined.lines()
                .filter(|l| l.starts_with("warning"))
                .map(|l| l.to_string())
                .collect();

            let passed = out.status.success() && errors.is_empty();
            VerifyResult { passed, errors, warnings, details: combined }
        }
        Err(e) => VerifyResult {
            passed: false,
            errors: vec![format!("cargo 执行失败: {}", e)],
            warnings: vec![],
            details: e.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_check_exists() {
        let v = CargoVerifier::new(".");
        let result = v.check_lib();
        assert!(result.passed || !result.errors.is_empty());
    }

    #[test]
    fn test_verify_result_defaults() {
        let r = VerifyResult { passed: true, errors: vec![], warnings: vec![], details: "".into() };
        assert!(r.passed);
        assert!(r.errors.is_empty());
    }
}
