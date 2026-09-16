#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub checks: Vec<VerifyCheckResult>,
    pub all_passed: bool,
}

#[derive(Debug, Clone)]
pub struct VerifyCheckResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

pub struct AutoVerifier {
    checks: Vec<Box<dyn Fn() -> VerifyCheckResult + Send + Sync>>,
}

impl AutoVerifier {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn add_check<F: Fn() -> VerifyCheckResult + Send + Sync + 'static>(&mut self, check: F) {
        self.checks.push(Box::new(check));
    }

    pub fn run(&self) -> VerificationReport {
        let checks: Vec<VerifyCheckResult> = self.checks.iter().map(|c| c()).collect();
        let all_passed = checks.iter().all(|c| c.passed);
        VerificationReport { checks, all_passed }
    }

    pub fn check_count(&self) -> usize {
        self.checks.len()
    }
}

impl Default for AutoVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        let mut v = AutoVerifier::new();
        v.add_check(|| VerifyCheckResult {
            name: "test".into(),
            passed: true,
            message: "ok".into(),
        });
        let r = v.run();
        assert!(r.all_passed);
        assert_eq!(r.checks.len(), 1);
    }

    #[test]
    fn test_failure() {
        let mut v = AutoVerifier::new();
        v.add_check(|| VerifyCheckResult {
            name: "f".into(),
            passed: false,
            message: "fail".into(),
        });
        let r = v.run();
        assert!(!r.all_passed);
    }
}
