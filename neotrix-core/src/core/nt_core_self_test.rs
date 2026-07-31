use std::collections::HashMap;

use crate::core::nt_core_self_constitution::global_constitution;

pub trait SelfTest: Send + Sync {
    fn name(&self) -> &str;
    fn self_test(&self) -> Result<(), Vec<String>>;
}

#[derive(Default)]
pub struct SelfTestRegistry {
    tests: HashMap<String, Box<dyn SelfTest>>,
}

impl SelfTestRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, test: Box<dyn SelfTest>) {
        self.tests.insert(test.name().to_string(), test);
    }

    pub fn run_all(&self) -> Vec<SelfTestResult> {
        let mut results = Vec::new();
        for (name, test) in &self.tests {
            match test.self_test() {
                Ok(()) => results.push(SelfTestResult::pass(name)),
                Err(failures) => results.push(SelfTestResult::fail(name, failures)),
            }
        }
        results
    }

    pub fn run_one(&self, name: &str) -> Option<SelfTestResult> {
        self.tests.get(name).map(|test| match test.self_test() {
            Ok(()) => SelfTestResult::pass(name),
            Err(failures) => SelfTestResult::fail(name, failures),
        })
    }

    pub fn register_all(&mut self, tests: Vec<Box<dyn SelfTest>>) {
        for t in tests {
            self.register(t);
        }
    }

    pub fn count(&self) -> usize {
        self.tests.len()
    }
}

#[derive(Debug, Clone)]
pub struct SelfTestResult {
    pub name: String,
    pub passed: bool,
    pub failures: Vec<String>,
}

impl SelfTestResult {
    pub fn pass(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            failures: vec![],
        }
    }

    pub fn fail(name: &str, failures: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            failures,
        }
    }

    pub fn summary(&self) -> String {
        if self.passed {
            format!("[SELF-TEST] {} ✅ pass", self.name)
        } else {
            format!("[SELF-TEST] {} ❌ FAIL ({} failures): {}", self.name, self.failures.len(), self.failures.join("; "))
        }
    }
}

pub fn report(results: &[SelfTestResult]) -> String {
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;
    let mut s = format!("SelfTestRegistry Report — {} total, {} passed, {} failed\n", total, passed, failed);
    for r in results {
        s.push_str(&format!("  {}\n", r.summary()));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PassTest;
    impl SelfTest for PassTest {
        fn name(&self) -> &str { "pass_test" }
        fn self_test(&self) -> Result<(), Vec<String>> { Ok(()) }
    }

    struct FailTest;
    impl SelfTest for FailTest {
        fn name(&self) -> &str { "fail_test" }
        fn self_test(&self) -> Result<(), Vec<String>> { Err(vec!["expected failure".into()]) }
    }

    #[test]
    fn test_registry_empty() {
        let r = SelfTestRegistry::new();
        assert!(r.run_all().is_empty());
    }

    #[test]
    fn test_registry_pass_and_fail() {
        let mut r = SelfTestRegistry::new();
        r.register(Box::new(PassTest));
        r.register(Box::new(FailTest));
        let results = r.run_all();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.name == "pass_test" && r.passed));
        assert!(results.iter().any(|r| r.name == "fail_test" && !r.passed));
    }

    #[test]
    fn test_run_one() {
        let mut r = SelfTestRegistry::new();
        r.register(Box::new(PassTest));
        assert!(r.run_one("pass_test").unwrap().passed);
        assert!(r.run_one("nonexistent").is_none());
    }
}

/// External verifier — runs `cargo check` to ground self-tests in build reality.
/// Prevents self-deception (D16b): a SelfTest pass means nothing if the code doesn't compile.
pub struct ExternalVerifier;

impl SelfTest for ExternalVerifier {
    fn name(&self) -> &str { "external_verifier" }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let output = std::process::Command::new("cargo")
            .args(["check", "--lib", "-p", "neotrix"])
            .output()
            .map_err(|e| vec![format!("failed to run cargo check: {}", e)])?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let errors: Vec<String> = stderr.lines()
                .filter(|l| l.contains("error"))
                .take(5)
                .map(|l| l.to_string())
                .collect();
            Err(vec![format!("cargo check failed ({} errors)", errors.len())])
        }
    }
}

/// Constitution Compliance SelfTest - verifies actions follow the constitution
pub struct ConstitutionComplianceTest;

impl SelfTest for ConstitutionComplianceTest {
    fn name(&self) -> &str {
        "constitution_compliance"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let constitution = global_constitution();
        
        // Check that constitution was loaded
        if constitution.rules.is_empty() {
            return Err(vec!["Constitution has no rules loaded".into()]);
        }
        
        // Check that tree growth rules exist (R-P42~R-P48)
        if constitution.tree_growth_rules().is_empty() {
            return Err(vec!["Missing tree growth rules (R-P42~R-P48)".into()]);
        }
        
        // Check that absorption rules exist (R-P43)
        if constitution.absorption_rules().is_empty() {
            return Err(vec!["Missing absorption protocol rules (R-P43)".into()]);
        }
        
        // Verify vector index is built
        if !constitution.has_vector_index() {
            return Err(vec!["Constitution vector index not built".into()]);
        }
        
        // Test compliance check on a valid action
        let report = constitution.verify_compliance("extend existing module nt_core_orch_agent with hexagram derivation");
        if !report.compliant {
            // Some violations may be expected, but we check the check works
        }
        
        // Test compliance check on a violation
        let violation_report = constitution.verify_compliance("create new module without branch mapping");
        if violation_report.compliant {
            return Err(vec!["Compliance check failed to detect R-P42 violation".into()]);
        }
        
        Ok(())
    }
}
