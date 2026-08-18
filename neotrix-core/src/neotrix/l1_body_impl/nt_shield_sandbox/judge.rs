//! C2 编程比赛代理 judge 层 — 编译 → 测试用例循环 → verdict。
//!
//! 强化 `nt_shield_sandbox` (R-P42): 代码评判的执行底座就是沙箱, judge
//! 是对沙箱能力的叶节点扩展, 不建平行适配器。设计源自已获批 C2 路线图:
//!   - compile → testcase loop → verdict
//!   - Refine@K 2 轮封顶 (编译-修复闭环由调用方经 GauntletMachine 诊断驱动)
//!   - verdict 落 KB (NT-MEMORY)
//!
//! `normalize_output` / `classify` 为纯函数, 可离线单测; 容器执行经
//! `CloudSandbox::run_code` 生产接线。

use serde::{Deserialize, Serialize};

use super::{CloudResult, CloudRuntime, CloudSandbox};

/// 单个测试用例 (输入 → 期望输出)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub input: String,
    pub expected: String,
    /// 该用例独立超时 (ms); 0 = 用 JudgeConfig.timeout_ms 默认。
    pub timeout_ms: u64,
}

impl TestCase {
    pub fn new(id: &str, input: &str, expected: &str) -> Self {
        Self {
            id: id.to_string(),
            input: input.to_string(),
            expected: expected.to_string(),
            timeout_ms: 0,
        }
    }

    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }
}

/// 单用例评判结论。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JudgeVerdict {
    Passed,
    WrongAnswer,
    Timeout,
    RuntimeError,
    CompileError,
}

impl JudgeVerdict {
    pub fn is_pass(&self) -> bool {
        matches!(self, JudgeVerdict::Passed)
    }

    pub fn label(&self) -> &'static str {
        match self {
            JudgeVerdict::Passed => "passed",
            JudgeVerdict::WrongAnswer => "wrong_answer",
            JudgeVerdict::Timeout => "timeout",
            JudgeVerdict::RuntimeError => "runtime_error",
            JudgeVerdict::CompileError => "compile_error",
        }
    }
}

/// 单用例评判结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeResult {
    pub case_id: String,
    pub verdict: JudgeVerdict,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

/// judge 配置 (Refine@K 封顶 + 记账式超时)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeConfig {
    /// 每用例默认超时 (ms)。容器侧 Docker `--cpus 1` 已有物理墙钟限制;
    /// judge 层记账式追加逻辑超时 (execution_time 检查)。
    pub timeout_ms: u64,
    /// Refine@K — 编译-修复重试轮数上限 (2 轮封顶)。
    pub refine_rounds: usize,
    /// 内存上限 (MB), 透传给沙箱执行 (docker --memory)。
    pub memory_mb: u64,
    /// Phase 0a: 将 verdict 作为 loss 信号送入 AnomalyDetector (六层能力实现损失)。
    /// false 时 judge 纯执行, 不产生遥测副作用。
    pub loss_signal: bool,
}

impl Default for JudgeConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 2000,
            refine_rounds: 2,
            memory_mb: 512,
            loss_signal: true,
        }
    }
}

/// 全轮评判汇总。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeSummary {
    pub total: usize,
    pub passed: usize,
    pub verdicts: Vec<JudgeResult>,
}

impl JudgeSummary {
    pub fn all_pass(&self) -> bool {
        self.passed == self.total
    }

    pub fn score(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.passed as f64 / self.total as f64
        }
    }
}

/// 规范化程序输出用于比对:
///   - 去除每行首尾空白
///   - 去除首尾空行 (末尾换行容忍)
///   - 保留行间顺序
/// 经典 OJ 判空惯例 (输出尾随换行/空格不计分差)。
pub fn normalize_output(s: &str) -> String {
    let mut lines: Vec<&str> = s
        .lines()
        .map(str::trim_end)
        .collect();
    while lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        lines.pop();
    }
    lines.join("\n")
}

/// 依据 CloudResult 分类单用例 verdict。
/// CompileError 判定: stderr 含编译期特征 (rustc/error[E + 非零 exit 且无 stdout)。
pub fn classify(result: &CloudResult, expected: &str, timeout_ms: u64) -> JudgeVerdict {
    if result.execution_time.as_millis() as u64 > timeout_ms {
        return JudgeVerdict::Timeout;
    }
    if result.exit_code != 0 {
        if looks_like_compile_error(&result.stderr) {
            return JudgeVerdict::CompileError;
        }
        return JudgeVerdict::RuntimeError;
    }
    if normalize_output(&result.stdout) == normalize_output(expected) {
        JudgeVerdict::Passed
    } else {
        JudgeVerdict::WrongAnswer
    }
}

/// 启发式: stderr 是否像编译器错误输出 (rustc panic 之外)。
pub fn looks_like_compile_error(stderr: &str) -> bool {
    let s = stderr.trim();
    s.contains("error[") || s.contains("error:") || s.contains("cannot find") || s.contains("mismatched types")
}

/// Phase 0a: verdict → capability-realization loss 层映射 (J-Space 六层)。
/// 外部信号 (比赛 judge 的失败结论) 注入遥测, 使自愈能区分失败根源
/// (推理模式 / 工具 schema / 表征 / 长程状态 / 验证机制)。
pub fn loss_layer_for_verdict(verdict: JudgeVerdict) -> crate::core::nt_core_telemetry::LossLayer {
    use crate::core::nt_core_telemetry::LossLayer;
    match verdict {
        JudgeVerdict::CompileError => LossLayer::ToolSchema,
        JudgeVerdict::WrongAnswer => LossLayer::ActiveRepresentation,
        JudgeVerdict::Timeout => LossLayer::LongHorizonState,
        JudgeVerdict::RuntimeError => LossLayer::Verification,
        JudgeVerdict::Passed => LossLayer::Verification,
    }
}

/// Phase 0a 接线: 将 judge 失败作为 loss 信号送入 AnomalyDetector。
/// `metric` 形如 `judge::{runtime}::{case_id}`; 失败时 observe_loss
/// (z-score 突刺才会告警), 通过时 observe 基线 0.0。
pub fn emit_loss_signal(
    detector: &crate::core::nt_core_telemetry::AnomalyDetector,
    runtime: CloudRuntime,
    case_id: &str,
    verdict: JudgeVerdict,
) {
    let metric = format!("judge::{}::{}", runtime.as_str(), case_id);
    let layer = loss_layer_for_verdict(verdict);
    match verdict {
        JudgeVerdict::Passed => {
            let _ = detector.observe(&metric, 0.0);
        }
        _ => {
            let _ = detector.observe_loss(&metric, 1.0, layer);
        }
    }
}

/// 执行整个测试集 (每用例独立沙箱 run_code)。
/// 编译-修复闭环不在此层: 调用方 (GauntletMachine 诊断) 据 CompileError
/// 驱动 retry_with_diagnosis, 由 refine_rounds 控制重试上限。
pub async fn judge_code(
    cloud: &mut CloudSandbox,
    code: &str,
    runtime: CloudRuntime,
    cases: &[TestCase],
    config: &JudgeConfig,
    detector: Option<&crate::core::nt_core_telemetry::AnomalyDetector>,
) -> JudgeSummary {
    let mut verdicts = Vec::with_capacity(cases.len());
    let mut passed = 0;
    for case in cases {
        let timeout = if case.timeout_ms > 0 {
            case.timeout_ms
        } else {
            config.timeout_ms
        };
        let result = match cloud.run_code(&program_with_input(code, &case.input), runtime).await {
            Ok(r) => r,
            Err(e) => {
                if let Some(d) = detector {
                    if config.loss_signal {
                        emit_loss_signal(d, runtime, &case.id, JudgeVerdict::RuntimeError);
                    }
                }
                verdicts.push(JudgeResult {
                    case_id: case.id.clone(),
                    verdict: JudgeVerdict::RuntimeError,
                    stdout: String::new(),
                    stderr: e,
                    execution_time_ms: 0,
                });
                continue;
            }
        };
        let verdict = classify(&result, &case.expected, timeout);
        if let Some(d) = detector {
            if config.loss_signal {
                emit_loss_signal(d, runtime, &case.id, verdict);
            }
        }
        if verdict.is_pass() {
            passed += 1;
        }
        verdicts.push(JudgeResult {
            case_id: case.id.clone(),
            verdict,
            stdout: result.stdout.clone(),
            stderr: result.stderr.clone(),
            execution_time_ms: result.execution_time.as_millis() as u64,
        });
    }
    JudgeSummary {
        total: cases.len(),
        passed,
        verdicts,
    }
}

/// 注入 stdin — 将测试输入写入 /tmp/neotrix-test-input 并让程序读取。
/// 兼容 rustc/go/python/node 的 stdin 注入: 使用 shell 重定向 < input。
fn program_with_input(code: &str, input: &str) -> String {
    let escaped = input.replace('\'', "'\"'\"'");
    format!("cat > /tmp/neotrix-test-input << 'NTINPUT'\n{}\nNTINPUT\n{} < /tmp/neotrix-test-input", escaped, code)
}

/// Refine@K 单轮结果 (含诊断链, J-Space 诊断携带重试语义)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineRound {
    pub round: usize,
    pub summary: JudgeSummary,
    /// 本轮到点为止的诊断链 — 每次失败追加一条 verdict 诊断,
    /// 供上层 retry-with-diagnosis 决策 (blank retry 禁止)。
    pub diagnosis_chain: Vec<String>,
}

/// Refine@K 闭环汇总。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineOutcome {
    pub rounds_used: usize,
    pub rounds_cap: usize,
    /// 最终轮 (all_pass 或 cap 用尽)。
    pub final_round: RefineRound,
    /// 历史轮次 (不含最终轮)。
    pub history: Vec<RefineRound>,
}

impl RefineOutcome {
    pub fn solved(&self) -> bool {
        self.final_round.summary.all_pass()
    }
}

/// 从单轮 judge 结果生成 J-Space 风格紧凑诊断。
/// 每个失败用例一条: `<case_id>: <verdict>: <首个错误线索>`。
pub fn diagnosis_from_summary(summary: &JudgeSummary) -> Vec<String> {
    summary
        .verdicts
        .iter()
        .filter(|r| !r.verdict.is_pass())
        .map(|r| {
            let hint = if !r.stderr.trim().is_empty() {
                let line = r.stderr.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
                line.trim().chars().take(120).collect::<String>()
            } else if !r.stdout.trim().is_empty() {
                "wrong output".to_string()
            } else {
                "no output".to_string()
            };
            format!("{}: {}: {}", r.case_id, r.verdict.label(), hint)
        })
        .collect()
}

/// Refine@K 编译-修复闭环。
///
/// 职责分离:
/// - judge 层只负责"跑 → 出诊断"; 修复动作由 `repair` 回调产生 (上层
///   可挂 AutoFixer/编译器建议/重采样)。
/// - `refine_rounds` 封顶 (Refine@K), 不无限重试。
/// - 诊断链单调追加 (J-Space retry-with-diagnosis: 重试必须携带诊断,
///   空白重试被上层 retry_with_diagnosis 拒绝)。
///
/// 回调返回 None = 本轮不再修复 (提前终止)。
pub async fn refine_loop<F>(
    cloud: &mut CloudSandbox,
    mut code: String,
    runtime: CloudRuntime,
    cases: &[TestCase],
    config: &JudgeConfig,
    detector: Option<&crate::core::nt_core_telemetry::AnomalyDetector>,
    repair: &mut F,
) -> RefineOutcome
where
    F: FnMut(&JudgeSummary, &[String]) -> Option<String>,
{
    let cap = config.refine_rounds.max(1);
    let mut history: Vec<RefineRound> = Vec::new();
    let mut diagnosis_chain: Vec<String> = Vec::new();

    for round in 0..cap {
        let rounds_used = round + 1;
        let summary = judge_code(cloud, &code, runtime, cases, config, detector).await;
        // 追加诊断 (仅失败时)。
        let diags = diagnosis_from_summary(&summary);
        diagnosis_chain.extend(diags.iter().cloned());

        if summary.all_pass() || round + 1 >= cap {
            return RefineOutcome {
                rounds_used,
                rounds_cap: cap,
                final_round: RefineRound {
                    round,
                    summary,
                    diagnosis_chain,
                },
                history,
            };
        }
        // 修复回调: 基于失败摘要 + 诊断链产生新 code; None = 提前终止。
        match repair(&summary, &diagnosis_chain) {
            Some(new_code) => {
                history.push(RefineRound {
                    round,
                    summary,
                    diagnosis_chain: diagnosis_chain.clone(),
                });
                code = new_code;
            }
            None => {
                // 提前终止: 将当前轮作为 final (未 solved)。
                return RefineOutcome {
                    rounds_used,
                    rounds_cap: cap,
                    final_round: RefineRound {
                        round,
                        summary,
                        diagnosis_chain,
                    },
                    history,
                };
            }
        }
    }
    unreachable!("cap >= 1 guarantees at least one iteration")
}

#[cfg(test)]
mod judge_core_tests {
    use super::*;
    use std::time::Duration;

    fn result(stdout: &str, stderr: &str, code: i32, ms: u64) -> CloudResult {
        CloudResult {
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            exit_code: code,
            execution_time: Duration::from_millis(ms),
            resource_usage: super::super::ResourceUsage::default(),
        }
    }

    #[test]
    fn test_normalize_trailing_whitespace() {
        assert_eq!(normalize_output("1\n2\n  \n"), "1\n2");
        assert_eq!(normalize_output("hello world  \n"), "hello world");
        assert_eq!(normalize_output("42"), "42");
    }

    #[test]
    fn test_classify_passed_ignores_trailing_newline() {
        let r = result("42\n", "", 0, 5);
        assert_eq!(classify(&r, "42", 2000), JudgeVerdict::Passed);
    }

    #[test]
    fn test_classify_wrong_answer() {
        let r = result("43\n", "", 0, 5);
        assert_eq!(classify(&r, "42", 2000), JudgeVerdict::WrongAnswer);
    }

    #[test]
    fn test_classify_timeout() {
        let r = result("", "", 0, 3000);
        assert_eq!(classify(&r, "42", 2000), JudgeVerdict::Timeout);
    }

    #[test]
    fn test_classify_runtime_error() {
        let r = result("", "thread 'main' panicked", 101, 5);
        assert_eq!(classify(&r, "", 2000), JudgeVerdict::RuntimeError);
    }

    #[test]
    fn test_classify_compile_error() {
        let r = result("", "error[E0425]: cannot find value `x` in this scope", 101, 5);
        assert_eq!(classify(&r, "", 2000), JudgeVerdict::CompileError);
    }

    #[test]
    fn test_verdict_label_roundtrip() {
        assert_eq!(JudgeVerdict::Passed.label(), "passed");
        assert_eq!(JudgeVerdict::WrongAnswer.label(), "wrong_answer");
        assert_eq!(JudgeVerdict::Timeout.label(), "timeout");
        assert_eq!(JudgeVerdict::RuntimeError.label(), "runtime_error");
        assert_eq!(JudgeVerdict::CompileError.label(), "compile_error");
        assert!(JudgeVerdict::Passed.is_pass());
        assert!(!JudgeVerdict::WrongAnswer.is_pass());
    }

    #[test]
    fn test_summary_scoring() {
        let s = JudgeSummary {
            total: 4,
            passed: 3,
            verdicts: vec![],
        };
        assert!(!s.all_pass());
        assert!((s.score() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_looks_like_compile_error_heuristic() {
        assert!(looks_like_compile_error("error[E0308]: mismatched types"));
        assert!(looks_like_compile_error("error: expected `;`, found `}`"));
        assert!(!looks_like_compile_error("thread 'main' panicked at src/main.rs"));
        assert!(!looks_like_compile_error(""));
    }

    #[test]
    fn test_loss_layer_mapping_six_layers_cover_verdicts() {
        use crate::core::nt_core_telemetry::LossLayer;
        assert_eq!(
            loss_layer_for_verdict(JudgeVerdict::CompileError),
            LossLayer::ToolSchema,
            "compile error = tool/schema 失配 (API 形状不符)"
        );
        assert_eq!(
            loss_layer_for_verdict(JudgeVerdict::WrongAnswer),
            LossLayer::ActiveRepresentation,
            "wrong answer = 活动表征漂移 (输出与预期不符)"
        );
        assert_eq!(
            loss_layer_for_verdict(JudgeVerdict::Timeout),
            LossLayer::LongHorizonState,
            "timeout = 长程状态损失 (目标未达, 路径发散)"
        );
        assert_eq!(
            loss_layer_for_verdict(JudgeVerdict::RuntimeError),
            LossLayer::Verification,
            "runtime error = 验证机制缺口 (崩溃未在验证层拦截)"
        );
        assert!(LossLayer::ALL.contains(&loss_layer_for_verdict(JudgeVerdict::Passed)));
        assert_eq!(LossLayer::ALL.len(), 6);
    }

    #[test]
    fn test_emit_loss_signal_observes_without_panic() {
        use crate::core::nt_core_telemetry::AnomalyDetector;
        use std::time::Duration;
        let detector = AnomalyDetector::new(Duration::from_secs(600), 2.5, 64);
        emit_loss_signal(&detector, CloudRuntime::Python3, "t1", JudgeVerdict::WrongAnswer);
        emit_loss_signal(&detector, CloudRuntime::RustStable, "t1", JudgeVerdict::Passed);
    }

    #[test]
    fn test_diagnosis_from_summary_empty_when_all_pass() {
        let s = JudgeSummary {
            total: 1,
            passed: 1,
            verdicts: vec![JudgeResult {
                case_id: "t1".into(),
                verdict: JudgeVerdict::Passed,
                stdout: "42".into(),
                stderr: String::new(),
                execution_time_ms: 1,
            }],
        };
        assert!(diagnosis_from_summary(&s).is_empty());
    }

    #[test]
    fn test_diagnosis_from_summary_carries_verdict_and_hint() {
        let s = JudgeSummary {
            total: 1,
            passed: 0,
            verdicts: vec![JudgeResult {
                case_id: "t1".into(),
                verdict: JudgeVerdict::WrongAnswer,
                stdout: "43".into(),
                stderr: String::new(),
                execution_time_ms: 1,
            }],
        };
        let diags = diagnosis_from_summary(&s);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].contains("t1"), "diagnosis must name the case: {}", diags[0]);
        assert!(diags[0].contains("wrong_answer"));
        assert!(diags[0].contains("wrong output"), "fallback hint when no stderr");
    }

    #[test]
    fn test_diagnosis_prefers_stderr_first_line() {
        let s = JudgeSummary {
            total: 1,
            passed: 0,
            verdicts: vec![JudgeResult {
                case_id: "t1".into(),
                verdict: JudgeVerdict::CompileError,
                stdout: String::new(),
                stderr: "error[E0425]: cannot find value `x` in this scope\n  --> main.rs:3:5\n".into(),
                execution_time_ms: 1,
            }],
        };
        let diags = diagnosis_from_summary(&s);
        assert!(diags[0].contains("E0425"), "must surface compiler error line");
    }

    #[test]
    fn test_refine_loop_stops_when_solved() {
        let mut cloud = CloudSandbox::default_local();
        let cases = vec![TestCase::new("t1", "", "ok")];
        let config = JudgeConfig { refine_rounds: 3, ..Default::default() };
        let mut repair = |_s: &JudgeSummary, _d: &[String]| -> Option<String> { Some("fn main(){}".into()) };
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let outcome = rt.block_on(refine_loop(
            &mut cloud,
            "fn main(){}".into(),
            CloudRuntime::RustStable,
            &cases,
            &config,
            None,
            &mut repair,
        ));
        assert_eq!(outcome.rounds_used, 1, "solved on first try must not refine further");
        assert!(!outcome.solved());
    }

    #[test]
    fn test_refine_loop_respects_cap() {
        let mut cloud = CloudSandbox::default_local();
        let cases = vec![TestCase::new("t1", "", "ok")];
        let config = JudgeConfig { refine_rounds: 2, ..Default::default() };
        let mut repair = |_s: &JudgeSummary, _d: &[String]| -> Option<String> { Some("bad".into()) };
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let outcome = rt.block_on(refine_loop(
            &mut cloud,
            "bad".into(),
            CloudRuntime::Python3,
            &cases,
            &config,
            None,
            &mut repair,
        ));
        assert_eq!(outcome.rounds_used, 2, "cap must bound refine rounds");
        assert_eq!(outcome.history.len(), 1, "one intermediate round before final");
        assert_eq!(outcome.final_round.diagnosis_chain.len(), 2, "diagnosis appended per failed round");
    }

    #[test]
    fn test_refine_loop_early_termination_on_repair_none() {
        let mut cloud = CloudSandbox::default_local();
        let cases = vec![TestCase::new("t1", "", "ok")];
        let config = JudgeConfig { refine_rounds: 3, ..Default::default() };
        let mut repair = |_s: &JudgeSummary, _d: &[String]| -> Option<String> { None };
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let outcome = rt.block_on(refine_loop(
            &mut cloud,
            "bad".into(),
            CloudRuntime::Python3,
            &cases,
            &config,
            None,
            &mut repair,
        ));
        assert_eq!(outcome.rounds_used, 1, "None repair must stop immediately");
        assert_eq!(outcome.history.len(), 0);
        assert!(!outcome.solved());
    }
}
