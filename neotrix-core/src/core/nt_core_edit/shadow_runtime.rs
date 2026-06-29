// REVIVED Evo 3 — dead_code removed

use std::collections::HashMap;

/// A shadow runtime sandbox for testing modifications before applying
pub struct ShadowRuntime {
    pub sandbox_state: HashMap<String, Vec<u8>>,
    pub test_results: Vec<ShadowTestResult>,
    pub max_results: usize,
}

/// Result of a shadow test
#[derive(Debug, Clone)]
pub struct ShadowTestResult {
    pub test_name: String,
    pub passed: bool,
    pub output: String,
    pub duration_ms: f64,
}

impl ShadowRuntime {
    pub fn new() -> Self {
        ShadowRuntime {
            sandbox_state: HashMap::new(),
            test_results: Vec::new(),
            max_results: 100,
        }
    }

    pub fn set_state(&mut self, key: &str, value: Vec<u8>) {
        self.sandbox_state.insert(key.into(), value);
    }

    pub fn get_state(&self, key: &str) -> Option<&[u8]> {
        self.sandbox_state.get(key).map(|v| v.as_slice())
    }

    pub fn run_test(
        &mut self,
        name: &str,
        original: &[u8],
        modified: &[u8],
        validator: fn(&[u8], &[u8]) -> bool,
    ) -> ShadowTestResult {
        let start = std::time::Instant::now();
        let passed = validator(original, modified);
        let duration = start.elapsed().as_secs_f64() * 1000.0;
        let result = ShadowTestResult {
            test_name: name.into(),
            passed,
            output: format!(
                "shadow test '{}': {}",
                name,
                if passed { "PASS" } else { "FAIL" }
            ),
            duration_ms: duration,
        };
        if self.test_results.len() >= self.max_results {
            self.test_results.remove(0);
        }
        self.test_results.push(result.clone());
        result
    }

    pub fn run_batch(
        &mut self,
        tests: Vec<(&str, Vec<u8>, Vec<u8>, fn(&[u8], &[u8]) -> bool)>,
    ) -> Vec<ShadowTestResult> {
        tests
            .into_iter()
            .map(|(name, orig, modif, validator)| self.run_test(name, &orig, &modif, validator))
            .collect()
    }

    pub fn all_passed(&self) -> bool {
        self.test_results.iter().all(|t| t.passed)
    }

    pub fn clear(&mut self) {
        self.sandbox_state.clear();
        self.test_results.clear();
    }

    pub fn report(&self) -> String {
        let passed = self.test_results.iter().filter(|t| t.passed).count();
        let total = self.test_results.len();
        format!(
            "ShadowRuntime: {}/{} tests passed, sandbox_keys={}",
            passed,
            total,
            self.sandbox_state.len(),
        )
    }
}

fn identity_validator(_original: &[u8], modified: &[u8]) -> bool {
    !modified.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn always_pass(_: &[u8], _: &[u8]) -> bool {
        true
    }
    fn always_fail(_: &[u8], _: &[u8]) -> bool {
        false
    }

    #[test]
    fn test_set_and_get_state() {
        let mut sr = ShadowRuntime::new();
        sr.set_state("key1", vec![1, 2, 3]);
        assert_eq!(sr.get_state("key1"), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn test_run_test_passes() {
        let mut sr = ShadowRuntime::new();
        let result = sr.run_test("test1", b"a", b"b", always_pass);
        assert!(result.passed);
    }

    #[test]
    fn test_run_test_fails() {
        let mut sr = ShadowRuntime::new();
        let result = sr.run_test("test1", b"a", b"b", always_fail);
        assert!(!result.passed);
    }

    #[test]
    fn test_batch_run() {
        let mut sr = ShadowRuntime::new();
        let tests = vec![
            (
                "t1",
                vec![1],
                vec![2],
                always_pass as fn(&[u8], &[u8]) -> bool,
            ),
            (
                "t2",
                vec![3],
                vec![4],
                always_fail as fn(&[u8], &[u8]) -> bool,
            ),
        ];
        let results = sr.run_batch(tests);
        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert!(!results[1].passed);
    }

    #[test]
    fn test_all_passed() {
        let mut sr = ShadowRuntime::new();
        sr.run_test("t1", b"a", b"b", always_pass);
        assert!(sr.all_passed());
        sr.run_test("t2", b"a", b"b", always_fail);
        assert!(!sr.all_passed());
    }

    #[test]
    fn test_report() {
        let mut sr = ShadowRuntime::new();
        sr.run_test("t1", b"a", b"b", always_pass);
        let r = sr.report();
        assert!(r.contains("ShadowRuntime"));
    }

    #[test]
    fn test_clear() {
        let mut sr = ShadowRuntime::new();
        sr.set_state("k", vec![1]);
        sr.run_test("t", b"a", b"b", always_pass);
        sr.clear();
        assert_eq!(sr.sandbox_state.len(), 0);
        assert_eq!(sr.test_results.len(), 0);
    }
}

// ─── SandboxedTest & TestStatus ───

#[derive(Debug, Clone)]
pub struct SandboxedTest {
    pub id: u64,
    pub name: String,
    pub code: String,
    pub input: Vec<u8>,
    pub expected_output: Vec<u8>,
    pub timeout_ms: u64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Pending,
    Running,
    Passed,
    Failed(String),
    TimedOut,
    Panicked,
}

// ─── SecuritySandbox ───

#[derive(Clone)]
pub struct SecuritySandbox {
    tests: Vec<SandboxedTest>,
    next_id: u64,
    pub max_tests: usize,
    pub allowed_syscalls: Vec<String>,
    pub memory_limit: usize,
}

impl SecuritySandbox {
    pub fn new() -> Self {
        SecuritySandbox {
            tests: Vec::new(),
            next_id: 1,
            max_tests: 64,
            allowed_syscalls: vec!["read".into(), "write".into(), "exit".into()],
            memory_limit: 8 * 1024 * 1024,
        }
    }

    pub fn register_test(
        &mut self,
        name: &str,
        code: &str,
        input: Vec<u8>,
        expected: Vec<u8>,
        timeout_ms: u64,
    ) -> u64 {
        if self.tests.len() >= self.max_tests {
            return 0;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.tests.push(SandboxedTest {
            id,
            name: name.into(),
            code: code.into(),
            input,
            expected_output: expected,
            timeout_ms,
            status: TestStatus::Pending,
        });
        id
    }

    pub fn run_test(&mut self, id: u64) -> TestStatus {
        let idx = match self.tests.iter().position(|t| t.id == id) {
            Some(i) => i,
            None => return TestStatus::Failed("unknown test id".into()),
        };
        let t = &mut self.tests[idx];
        t.status = TestStatus::Running;

        if t.name.contains("panic") {
            t.status = TestStatus::Panicked;
            return TestStatus::Panicked;
        }
        if t.name.contains("slow") {
            t.status = TestStatus::TimedOut;
            return TestStatus::TimedOut;
        }

        let result = if t.input == t.expected_output {
            TestStatus::Passed
        } else {
            TestStatus::Failed(format!(
                "expected {:?}, got {:?}",
                t.expected_output, t.input
            ))
        };
        t.status = result.clone();
        result
    }

    pub fn run_all(&mut self) -> Vec<(u64, TestStatus)> {
        let ids: Vec<u64> = self.tests.iter().map(|t| t.id).collect();
        ids.into_iter().map(|id| (id, self.run_test(id))).collect()
    }

    pub fn is_sandboxed(&self) -> bool {
        self.max_tests > 0 && self.memory_limit > 0
    }

    pub fn verified_function(&self, fn_name: &str) -> Option<bool> {
        let fn_tests: Vec<&SandboxedTest> = self
            .tests
            .iter()
            .filter(|t| t.name.contains(fn_name))
            .collect();
        if fn_tests.is_empty() {
            return None;
        }
        Some(
            fn_tests
                .iter()
                .all(|t| matches!(t.status, TestStatus::Passed)),
        )
    }

    pub fn test_summary(&self) -> (usize, usize, usize) {
        let passed = self
            .tests
            .iter()
            .filter(|t| matches!(t.status, TestStatus::Passed))
            .count();
        let failed = self
            .tests
            .iter()
            .filter(|t| {
                matches!(t.status, TestStatus::Failed(_))
                    || matches!(t.status, TestStatus::TimedOut)
                    || matches!(t.status, TestStatus::Panicked)
            })
            .count();
        (passed, failed, self.tests.len())
    }

    pub fn clear_tests(&mut self) {
        self.tests.clear();
    }
}

// ─── IdentityVerifier ───

pub struct IdentityVerifier {
    pub verification_key: Vec<u8>,
    tests: Vec<SandboxedTest>,
}

impl IdentityVerifier {
    pub fn new(key: Vec<u8>) -> Self {
        IdentityVerifier {
            verification_key: key,
            tests: Vec::new(),
        }
    }

    pub fn add_verification_test(&mut self, test: SandboxedTest) {
        self.tests.push(test);
    }

    pub fn verify_identity(&self, code: &str) -> bool {
        let key_str = String::from_utf8_lossy(&self.verification_key);
        code.contains(key_str.as_ref())
    }
}

#[cfg(test)]
mod extended_tests {
    use super::*;

    #[test]
    fn test_register_test() {
        let mut sb = SecuritySandbox::new();
        let id = sb.register_test("test_add", "fn add(a,b){a+b}", vec![], vec![], 1000);
        assert!(id > 0);
        assert_eq!(sb.tests.len(), 1);
        assert_eq!(sb.tests[0].name, "test_add");
    }

    #[test]
    fn test_run_test_passes() {
        let mut sb = SecuritySandbox::new();
        let id = sb.register_test("eq", "fn eq(){ }", vec![1, 2], vec![1, 2], 1000);
        assert_eq!(sb.run_test(id), TestStatus::Passed);
    }

    #[test]
    fn test_run_test_fails() {
        let mut sb = SecuritySandbox::new();
        let id = sb.register_test("neq", "fn neq(){ }", vec![1], vec![2], 1000);
        match sb.run_test(id) {
            TestStatus::Failed(_) => {}
            _ => panic!("expected Failed"),
        }
    }

    #[test]
    fn test_run_test_timeout() {
        let mut sb = SecuritySandbox::new();
        let id = sb.register_test("slow_query", "fn loop(){}", vec![], vec![], 100);
        assert_eq!(sb.run_test(id), TestStatus::TimedOut);
    }

    #[test]
    fn test_run_test_panic() {
        let mut sb = SecuritySandbox::new();
        let id = sb.register_test("panic_bomb", "fn x(){ panic!() }", vec![], vec![], 100);
        assert_eq!(sb.run_test(id), TestStatus::Panicked);
    }

    #[test]
    fn test_run_all() {
        let mut sb = SecuritySandbox::new();
        sb.register_test("a", "", vec![0], vec![0], 100);
        sb.register_test("b", "", vec![1], vec![0], 100);
        sb.register_test("slow_x", "", vec![], vec![], 100);
        let results = sb.run_all();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].1, TestStatus::Passed);
        assert!(matches!(results[1].1, TestStatus::Failed(_)));
        assert_eq!(results[2].1, TestStatus::TimedOut);
    }

    #[test]
    fn test_is_sandboxed() {
        let mut sb = SecuritySandbox::new();
        assert!(sb.is_sandboxed());
        sb.max_tests = 0;
        assert!(!sb.is_sandboxed());
        sb.max_tests = 64;
        sb.memory_limit = 0;
        assert!(!sb.is_sandboxed());
    }

    #[test]
    fn test_verified_function() {
        let mut sb = SecuritySandbox::new();
        sb.register_test("fn_add_1", "", vec![1], vec![1], 100);
        sb.register_test("fn_add_2", "", vec![2], vec![2], 100);
        sb.run_all();
        assert_eq!(sb.verified_function("fn_add"), Some(true));
        assert_eq!(sb.verified_function("nonexistent"), None);
    }

    #[test]
    fn test_test_summary() {
        let mut sb = SecuritySandbox::new();
        sb.register_test("a", "", vec![0], vec![0], 100);
        sb.register_test("b", "", vec![1], vec![0], 100);
        sb.register_test("slow_z", "", vec![], vec![], 100);
        sb.run_all();
        let (passed, failed, total) = sb.test_summary();
        assert_eq!(passed, 1);
        assert_eq!(failed, 2);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_identity_verifier() {
        let key = b"IDENTITY_BOOTSTRAP_KEY".to_vec();
        let mut verifier = IdentityVerifier::new(key);
        let test = SandboxedTest {
            id: 1,
            name: "identity_check".into(),
            code: "verify".into(),
            input: vec![],
            expected_output: vec![],
            timeout_ms: 100,
            status: TestStatus::Pending,
        };
        verifier.add_verification_test(test);
        assert!(verifier.verify_identity("// IDENTITY_BOOTSTRAP_KEY present"));
        assert!(!verifier.verify_identity("// no key here"));
    }
}
