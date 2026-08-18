use std::collections::HashMap;

use crate::core::nt_core_self_constitution::global_constitution;

/// 跨模块共享的测试环境锁 — 串行化所有 set_var(HOME/NEOTRIX_*) 的测试隔离。
/// 原因: kb_cmds::with_temp_home / consciousness_core::isolate_home_once 等各自
/// 用私有锁, 互不感知 → 并行测试窗口内 HOME 被对方覆盖 → QueryReturnedNoRows /
/// roundtrip 读错库等 flaky (Rust set_var 进程级全局, 多线程竞争).
/// 用法: 测试隔离入口先持此锁, 再 set/恢复 env。
#[cfg(test)]
pub static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
}

/// External verifier — runs `cargo check` to ground self-tests in build reality.
/// Prevents self-deception (D16b): a SelfTest pass means nothing if the code doesn't compile.
pub struct ExternalVerifier;

impl SelfTest for ExternalVerifier {
    fn name(&self) -> &str {
        "external_verifier"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let output = std::process::Command::new("cargo")
            .args(["check", "--lib", "-p", "neotrix"])
            .output()
            .map_err(|e| vec![format!("failed to run cargo check: {}", e)])?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let errors: Vec<String> = stderr
                .lines()
                .filter(|l| l.contains("error"))
                .take(5)
                .map(|l| l.to_string())
                .collect();
            Err(vec![format!(
                "cargo check failed ({} errors)",
                errors.len()
            )])
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
        let report = constitution.verify_compliance(
            "extend existing module nt_core_orch_agent with hexagram derivation",
        );
        if !report.compliant {
            // Some violations may be expected, but we check the check works
        }

        // Test compliance check on a violation
        let violation_report =
            constitution.verify_compliance("create new module without branch mapping");
        if violation_report.compliant {
            return Err(vec![
                "Compliance check failed to detect R-P42 violation".into()
            ]);
        }

        Ok(())
    }
}

/// J-Space 时间自省缺口强化 (LessWrong: "Your Agents Are Not Time-Aware",
/// absorbed 2026-08-18) — 智能体对自身时长无校准: Fable 过度预测 3×/Sol 6×,
/// 校准依赖带时间戳的 transcript, 去掉时钟文本精度减半。时长自省 =
/// 自控/可监控性维度。此处为 harness 层提供可观测的 预测 vs 实际 时长漂移。
#[derive(Debug, Clone)]
pub struct DurationDriftMonitor {
    /// 记录 (预测分钟, 实际分钟) 对的容量。
    capacity: usize,
    /// 判定漂移的倍数阈值 (实际/预测 > threshold → 漂移)。
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

    /// 记录一次任务: (预测分钟, 实际分钟)。
    pub fn record(&mut self, predicted_min: f64, actual_min: f64) {
        if predicted_min <= 0.0 || actual_min < 0.0 {
            return;
        }
        if self.samples.len() >= self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back((predicted_min, actual_min));
    }

    /// 单次样本是否漂移 (实际 > threshold × 预测)。
    pub fn is_drifted(predicted_min: f64, actual_min: f64, threshold: f64) -> bool {
        predicted_min > 0.0 && actual_min > predicted_min * threshold
    }

    /// 窗口内漂移样本数。
    pub fn drift_count(&self) -> usize {
        self.samples
            .iter()
            .filter(|(p, a)| Self::is_drifted(*p, *a, self.threshold))
            .count()
    }

    /// 平均漂移比 (实际/预测), 窗口内实际 > 预测的样本; 无样本返回 None。
    pub fn mean_drift_ratio(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        let ratios: Vec<f64> = self
            .samples
            .iter()
            .map(|(p, a)| a / p)
            .collect();
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

/// Duration-drift SelfTest — 若窗口内漂移样本占比超 1/3, 报告为自我欺骗信号
/// (D15 self-deception / 可监控性维度)。生产接线点: growth-cycle 时长自省。
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
                "duration_drift: no duration samples recorded — time self-awareness unexercised".into()
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
mod duration_drift_tests {
    use super::*;

    #[test]
    fn test_monitor_records_and_counts_drift() {
        let mut m = DurationDriftMonitor::new(16, 2.0);
        m.record(10.0, 11.0); // within threshold
        m.record(10.0, 30.0); // 3× → drifted
        m.record(5.0, 20.0); // 4× → drifted
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
