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
}

impl Default for JudgeConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 2000,
            refine_rounds: 2,
            memory_mb: 512,
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

/// 执行整个测试集 (每用例独立沙箱 run_code)。
/// 编译-修复闭环不在此层: 调用方 (GauntletMachine 诊断) 据 CompileError
/// 驱动 retry_with_diagnosis, 由 refine_rounds 控制重试上限。
pub async fn judge_code(
    cloud: &mut CloudSandbox,
    code: &str,
    runtime: CloudRuntime,
    cases: &[TestCase],
    config: &JudgeConfig,
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
}
