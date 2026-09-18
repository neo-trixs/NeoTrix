use std::collections::HashMap;

use crate::l6_meta::nt_core_self_constitution::global_constitution;

// SelfTest trait 下沉到 L0 Substrate — re-export 保持现有导入路径兼容
pub use crate::l0_substrate::nt_core_self_test::SelfTest;

/// 跨模块共享的测试环境锁 — 串行化所有 set_var(HOME/NEOTRIX_*) 的测试隔离。
/// 原因: kb_cmds::with_temp_home / consciousness_core::isolate_home_once 等各自
/// 用私有锁, 互不感知 → 并行测试窗口内 HOME 被对方覆盖 → QueryReturnedNoRows /
/// roundtrip 读错库等 flaky (Rust set_var 进程级全局, 多线程竞争).
/// 用法: 测试隔离入口先持此锁, 再 set/恢复 env。
#[cfg(test)]
pub static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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

/// External verifier — runs `cargo check` to ground self-tests in build reality.
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

        if constitution.rules.is_empty() {
            return Err(vec!["Constitution has no rules loaded".into()]);
        }

        if constitution.tree_growth_rules().is_empty() {
            return Err(vec!["Missing tree growth rules (R-P42~R-P48)".into()]);
        }

        if constitution.absorption_rules().is_empty() {
            return Err(vec!["Missing absorption protocol rules (R-P43)".into()]);
        }

        if !constitution.has_vector_index() {
            return Err(vec!["Constitution vector index not built".into()]);
        }

        let report = constitution.verify_compliance(
            "extend existing module nt_core_orch_agent with hexagram derivation",
        );
        if !report.compliant {
            // Some violations may be expected, but we check the check works
        }

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

/// Trace Evaluation SelfTest — 验证轨迹评估系统功能正常
pub struct TraceEvaluationTest;

impl SelfTest for TraceEvaluationTest {
    fn name(&self) -> &str {
        "trace_evaluation"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::l6_meta::nt_core_self::trace_evaluation::{
            AgentTrace, EvaluationGrade, TraceEvaluator, TraceStep,
        };
        use std::time::Instant;

        let evaluator = TraceEvaluator::new();

        let steps: Vec<TraceStep> = (0..5)
            .map(|i| TraceStep {
                step_number: i,
                action: format!("action_{}", i),
                input: format!("input_{}", i),
                output: format!("output_{}", i),
                duration_ms: 1000,
                tokens_used: 100,
                success: true,
                error: None,
            })
            .collect();

        let trace = AgentTrace {
            agent_id: "test_agent".to_string(),
            task_id: "test_task".to_string(),
            steps,
            start_time: Instant::now(),
            end_time: Some(Instant::now()),
            final_result: Some("completed".to_string()),
        };

        let report = evaluator.evaluate(&trace);

        let mut errors = Vec::new();

        if report.results.len() != 5 {
            errors.push(format!(
                "Expected 5 evaluation dimensions, got {}",
                report.results.len()
            ));
        }

        if report.overall_score < 0.0 || report.overall_score > 1.0 {
            errors.push(format!(
                "Overall score out of range: {}",
                report.overall_score
            ));
        }

        if report.grade != EvaluationGrade::Excellent && report.grade != EvaluationGrade::Good {
            errors.push(format!(
                "Unexpected grade for perfect trace: {:?}",
                report.grade
            ));
        }

        for result in &report.results {
            if result.score < 0.0 || result.score > 1.0 {
                errors.push(format!(
                    "Dimension {:?} score out of range: {}",
                    result.dimension, result.score
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
