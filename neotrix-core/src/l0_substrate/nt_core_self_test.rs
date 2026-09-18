//! SelfTest trait + SelfTestRegistry — 跨模块共享的自测试基础设施
//!
//! 下沉到 L0 Substrate 层，供 L1-L6 所有层使用。
//! 实现保留在各自层（如 L6 的 nt_core_self_test.rs 保留 ConstitutionComplianceTest）。
//! L6 通过 re-export 保持现有导入路径兼容。

use std::collections::HashMap;

/// SelfTest trait — 跨模块共享的自测试接口定义
pub trait SelfTest: Send + Sync {
    fn name(&self) -> &str;
    fn self_test(&self) -> Result<(), Vec<String>>;
}

/// 跨模块共享的测试环境锁 — 串行化所有 set_var(HOME/NEOTRIX_*) 的测试隔离。
#[cfg(test)]
pub static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// SelfTestRegistry — 管理和运行所有注册的 SelfTest 实现
#[derive(Default)]
pub struct SelfTestRegistry {
    tests: HashMap<String, Box<dyn SelfTest>>,
    t1_count: usize,
    t2_count: usize,
    t3_count: usize,
    production_influences: Vec<String>,
}

impl SelfTestRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, test: Box<dyn SelfTest>) {
        self.tests.insert(test.name().to_string(), test);
        self.t1_count += 1;
        self.t2_count += 1;
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

    pub fn check_t3_production_wiring(&self) -> Vec<String> {
        self.production_influences.clone()
    }

    pub fn record_t3_influence(&mut self, influence: &str) {
        self.t3_count += 1;
        self.production_influences.push(influence.to_string());
    }

    pub fn t3_influences(&self) -> &[String] {
        &self.production_influences
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
            format!(
                "[SELF-TEST] {} ❌ FAIL ({} failures): {}",
                self.name,
                self.failures.len(),
                self.failures.join("; ")
            )
        }
    }
}

pub fn report(results: &[SelfTestResult]) -> String {
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;
    let mut s = format!(
        "SelfTestRegistry Report — {} total, {} passed, {} failed\n",
        total, passed, failed
    );
    for r in results {
        s.push_str(&format!("  {}\n", r.summary()));
    }
    s
}

/// Duration-drift monitor — 时间自省缺口强化
#[derive(Debug, Clone)]
pub struct DurationDriftMonitor {
    capacity: usize,
    threshold: f64,
    samples: std::collections::VecDeque<(f64, f64)>,
}

impl Default for DurationDriftMonitor {
    fn default() -> Self {
        Self {
            capacity: 32,
            threshold: 2.0,
            samples: std::collections::VecDeque::new(),
        }
    }
}

impl DurationDriftMonitor {
    pub fn new(capacity: usize, threshold: f64) -> Self {
        Self {
            capacity: capacity.max(4),
            threshold,
            samples: std::collections::VecDeque::with_capacity(capacity.max(4)),
        }
    }

    pub fn record(&mut self, predicted_min: f64, actual_min: f64) {
        if predicted_min <= 0.0 || actual_min < 0.0 {
            return;
        }
        if self.samples.len() >= self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back((predicted_min, actual_min));
    }

    pub fn is_drifted(predicted_min: f64, actual_min: f64, threshold: f64) -> bool {
        predicted_min > 0.0 && actual_min > predicted_min * threshold
    }

    pub fn drift_count(&self) -> usize {
        self.samples
            .iter()
            .filter(|(p, a)| Self::is_drifted(*p, *a, self.threshold))
            .count()
    }

    pub fn mean_drift_ratio(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        let ratios: Vec<f64> = self.samples.iter().map(|(p, a)| a / p).collect();
        Some(ratios.iter().sum::<f64>() / ratios.len() as f64)
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn threshold(&self) -> f64 {
        self.threshold
    }
}

/// Duration-drift SelfTest
#[derive(Default)]
pub struct DurationDriftTest {
    pub monitor: DurationDriftMonitor,
}

impl DurationDriftTest {
    pub fn new(monitor: DurationDriftMonitor) -> Self {
        Self { monitor }
    }
}

impl SelfTest for DurationDriftTest {
    fn name(&self) -> &str {
        "duration_drift"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if self.monitor.is_empty() {
            return Err(vec![
                "duration_drift: no duration samples recorded — time self-awareness unexercised"
                    .into(),
            ]);
        }
        let total = self.monitor.len();
        let drifted = self.monitor.drift_count();
        if drifted > total / 3 {
            Err(vec![format!(
                "duration_drift: {drifted}/{total} samples drifted (actual > {}× predicted) — temporal self-deception",
                self.monitor.threshold()
            )])
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PassTest;
    impl SelfTest for PassTest {
        fn name(&self) -> &str {
            "pass_test"
        }
        fn self_test(&self) -> Result<(), Vec<String>> {
            Ok(())
        }
    }

    struct FailTest;
    impl SelfTest for FailTest {
        fn name(&self) -> &str {
            "fail_test"
        }
        fn self_test(&self) -> Result<(), Vec<String>> {
            Err(vec!["expected failure".into()])
        }
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

    #[test]
    fn test_monitor_records_and_counts_drift() {
        let mut m = DurationDriftMonitor::new(16, 2.0);
        m.record(10.0, 11.0);
        m.record(10.0, 30.0);
        m.record(5.0, 20.0);
        assert_eq!(m.drift_count(), 2);
        assert_eq!(m.len(), 3);
        let ratio = m.mean_drift_ratio().unwrap();
        assert!(ratio > 2.0);
    }

    #[test]
    fn test_monitor_rejects_invalid_samples() {
        let mut m = DurationDriftMonitor::default();
        m.record(0.0, 5.0);
        m.record(-1.0, 5.0);
        assert!(m.is_empty());
    }

    #[test]
    fn test_is_drifted_boundary() {
        assert!(!DurationDriftMonitor::is_drifted(10.0, 20.0, 2.0));
        assert!(DurationDriftMonitor::is_drifted(10.0, 20.1, 2.0));
        assert!(!DurationDriftMonitor::is_drifted(0.0, 5.0, 2.0));
    }

    #[test]
    fn test_selftest_passes_when_calibrated() {
        let mut m = DurationDriftMonitor::new(16, 2.0);
        for i in 1..=6 {
            m.record(i as f64, i as f64 * 1.2);
        }
        let t = DurationDriftTest::new(m);
        assert!(t.self_test().is_ok());
    }

    #[test]
    fn test_selftest_fails_when_many_drifted() {
        let mut m = DurationDriftMonitor::new(16, 2.0);
        m.record(10.0, 10.0);
        m.record(10.0, 40.0);
        m.record(10.0, 45.0);
        let t = DurationDriftTest::new(m);
        let result = t.self_test();
        assert!(result.is_err(), "majority drift must be flagged");
        let err = result.unwrap_err();
        assert!(err.iter().any(|f| f.contains("drifted")));
    }

    #[test]
    fn test_selftest_fails_when_no_samples() {
        let t = DurationDriftTest::default();
        assert!(t.self_test().is_err());
    }
}
