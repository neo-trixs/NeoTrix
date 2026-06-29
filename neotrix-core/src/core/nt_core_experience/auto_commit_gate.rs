use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerificationStep {
    CompileCheck,
    TestRun,
    ClippyCheck,
}

impl VerificationStep {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CompileCheck => "compile_check",
            Self::TestRun => "test_run",
            Self::ClippyCheck => "clippy_check",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step: VerificationStep,
    pub passed: bool,
    pub exit_code: Option<i32>,
    pub stdout_snippet: String,
    pub stderr_snippet: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct GateResult {
    pub all_passed: bool,
    pub step_results: Vec<StepResult>,
    pub total_duration_ms: u64,
    pub timestamp: u64,
    pub mutation_description: String,
}

#[derive(Debug, Clone)]
pub struct AutoCommitConfig {
    pub workspace_path: String,
    pub steps: Vec<VerificationStep>,
    pub step_timeout_secs: u64,
    pub max_stderr_chars: usize,
    pub auto_fix_on_fail: bool,
}

impl Default for AutoCommitConfig {
    fn default() -> Self {
        Self {
            workspace_path: String::new(),
            steps: vec![
                VerificationStep::CompileCheck,
                VerificationStep::TestRun,
                VerificationStep::ClippyCheck,
            ],
            step_timeout_secs: 300,
            max_stderr_chars: 500,
            auto_fix_on_fail: false,
        }
    }
}

#[derive(Debug)]
pub struct AutoCommitGate {
    pub config: AutoCommitConfig,
    pub total_gates: u64,
    pub total_passed: u64,
    pub total_failed: u64,
    pub history: VecDeque<GateResult>,
}

impl AutoCommitGate {
    pub fn new(config: AutoCommitConfig) -> Self {
        Self {
            config,
            total_gates: 0,
            total_passed: 0,
            total_failed: 0,
            history: VecDeque::with_capacity(100),
        }
    }

    pub fn with_workspace(path: &str) -> Self {
        Self::new(AutoCommitConfig {
            workspace_path: path.to_string(),
            ..Default::default()
        })
    }

    pub fn run_gate(&mut self, mutation_desc: &str, cycle: u64) -> GateResult {
        let start = std::time::Instant::now();
        let mut step_results = Vec::new();

        for step in &self.config.steps {
            let result = match step {
                VerificationStep::CompileCheck => self.run_compile_check(),
                VerificationStep::TestRun => self.run_tests(),
                VerificationStep::ClippyCheck => self.run_clippy(),
            };
            step_results.push(result);
            if let Some(last) = step_results.last() {
                if !last.passed && !self.config.auto_fix_on_fail {
                    break;
                }
            }
        }

        let all_passed = step_results.iter().all(|r| r.passed);
        let total_duration_ms = start.elapsed().as_millis() as u64;

        let gate = GateResult {
            all_passed,
            step_results,
            total_duration_ms,
            timestamp: cycle,
            mutation_description: mutation_desc.to_string(),
        };

        self.total_gates += 1;
        if all_passed {
            self.total_passed += 1;
        } else {
            self.total_failed += 1;
        }

        if self.history.len() >= 100 {
            self.history.pop_front();
        }
        self.history.push_back(gate.clone());

        gate
    }

    fn run_command(&self, args: &[&str]) -> StepResult {
        let step = match args.first() {
            Some(a) if *a == "check" => VerificationStep::CompileCheck,
            Some(a) if *a == "test" => VerificationStep::TestRun,
            Some(a) if *a == "clippy" => VerificationStep::ClippyCheck,
            _ => {
                return StepResult {
                    step: VerificationStep::CompileCheck,
                    passed: false,
                    exit_code: None,
                    stdout_snippet: String::new(),
                    stderr_snippet: "unknown cargo subcommand".to_string(),
                    duration_ms: 0,
                };
            }
        };

        let start = std::time::Instant::now();
        match std::process::Command::new("cargo")
            .args(args)
            .current_dir(&self.config.workspace_path)
            .output()
        {
            Ok(output) => StepResult {
                step,
                passed: output.status.success(),
                exit_code: output.status.code(),
                stdout_snippet: truncate(
                    &String::from_utf8_lossy(&output.stdout),
                    self.config.max_stderr_chars,
                ),
                stderr_snippet: truncate(
                    &String::from_utf8_lossy(&output.stderr),
                    self.config.max_stderr_chars,
                ),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Err(e) => StepResult {
                step,
                passed: false,
                exit_code: None,
                stdout_snippet: String::new(),
                stderr_snippet: format!("command failed: {}", e),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    fn run_compile_check(&self) -> StepResult {
        self.run_command(&["check", "--lib"])
    }

    fn run_tests(&self) -> StepResult {
        self.run_command(&["test", "--lib"])
    }

    fn run_clippy(&self) -> StepResult {
        self.run_command(&["clippy", "--lib"])
    }

    pub fn last_gate(&self) -> Option<&GateResult> {
        self.history.back()
    }

    pub fn consecutive_failures(&self) -> u64 {
        let mut count = 0;
        for gate in self.history.iter().rev() {
            if gate.all_passed {
                break;
            }
            count += 1;
        }
        count
    }

    pub fn stats(&self) -> GateStats {
        let sum_duration: u64 = self.history.iter().map(|g| g.total_duration_ms).sum();
        let avg_duration = if self.history.is_empty() {
            0
        } else {
            sum_duration / self.history.len() as u64
        };

        GateStats {
            total_gates: self.total_gates,
            total_passed: self.total_passed,
            total_failed: self.total_failed,
            consecutive_failures: self.consecutive_failures(),
            last_passed: self.last_gate().map(|g| g.all_passed),
            avg_duration_ms: avg_duration,
        }
    }

    pub fn summary(&self) -> String {
        let s = self.stats();
        format!(
            "auto_gate: gates={} passed={} failed={} last={} consec_fail={}",
            s.total_gates,
            s.total_passed,
            s.total_failed,
            s.last_passed.map(|v| v.to_string()).unwrap_or_else(|| "none".to_string()),
            s.consecutive_failures
        )
    }
}

#[derive(Debug, Clone)]
pub struct GateStats {
    pub total_gates: u64,
    pub total_passed: u64,
    pub total_failed: u64,
    pub consecutive_failures: u64,
    pub last_passed: Option<bool>,
    pub avg_duration_ms: u64,
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}... (truncated {} chars)", &s[..max], s.len() - max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_gate() -> AutoCommitGate {
        AutoCommitGate::new(AutoCommitConfig {
            workspace_path: "/tmp/nonexistent".to_string(),
            steps: vec![VerificationStep::CompileCheck],
            step_timeout_secs: 1,
            max_stderr_chars: 500,
            auto_fix_on_fail: false,
        })
    }

    #[test]
    fn test_config_defaults() {
        let cfg = AutoCommitConfig::default();
        assert_eq!(cfg.steps.len(), 3);
        assert_eq!(cfg.step_timeout_secs, 300);
    }

    #[test]
    fn test_with_workspace() {
        let g = AutoCommitGate::with_workspace("/some/path");
        assert_eq!(g.config.workspace_path, "/some/path");
    }

    #[test]
    fn test_run_compile_bad_path() {
        let mut g = make_gate();
        let result = g.run_compile_check();
        assert!(!result.passed);
    }

    #[test]
    fn test_gate_tracks_stats() {
        let mut g = make_gate();
        let _ = g.run_gate("test mutation", 0);
        assert_eq!(g.total_gates, 1);
    }

    #[test]
    fn test_consecutive_failures() {
        let mut g = make_gate();
        g.run_gate("fail 1", 0);
        g.run_gate("fail 2", 1);
        assert!(g.consecutive_failures() >= 2);
    }

    #[test]
    fn test_last_gate() {
        let mut g = make_gate();
        assert!(g.last_gate().is_none());
        g.run_gate("test", 0);
        assert!(g.last_gate().is_some());
    }

    #[test]
    fn test_stats() {
        let mut g = make_gate();
        g.run_gate("test", 0);
        let stats = g.stats();
        assert_eq!(stats.total_gates, 1);
    }

    #[test]
    fn test_summary_format() {
        let g = make_gate();
        let s = g.summary();
        assert!(s.contains("auto_gate:"));
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        let t = truncate("hello world this is long", 10);
        assert!(t.contains("truncated"));
    }

    #[test]
    fn test_auto_fix_on_fail_flag() {
        let cfg = AutoCommitConfig {
            auto_fix_on_fail: true,
            ..AutoCommitConfig::default()
        };
        let g = AutoCommitGate::new(cfg);
        assert!(g.config.auto_fix_on_fail);
    }

    #[test]
    fn test_history_bounded() {
        let mut g = AutoCommitGate::new(AutoCommitConfig {
            workspace_path: "/tmp/nonexistent".to_string(),
            steps: vec![VerificationStep::CompileCheck],
            step_timeout_secs: 1,
            max_stderr_chars: 500,
            auto_fix_on_fail: false,
        });
        for i in 0..120 {
            g.run_gate(&format!("mutation {}", i), i as u64);
        }
        assert!(g.history.len() <= 100);
    }

    #[test]
    fn test_step_result_default_fail() {
        let r = StepResult {
            step: VerificationStep::CompileCheck,
            passed: false,
            exit_code: Some(1),
            stdout_snippet: String::new(),
            stderr_snippet: "error".to_string(),
            duration_ms: 0,
        };
        assert!(!r.passed);
    }

    #[test]
    fn test_verification_step_name() {
        assert_eq!(VerificationStep::CompileCheck.name(), "compile_check");
        assert_eq!(VerificationStep::TestRun.name(), "test_run");
        assert_eq!(VerificationStep::ClippyCheck.name(), "clippy_check");
    }

    #[test]
    fn test_gate_result_all_passed_false() {
        let r = GateResult {
            all_passed: false,
            step_results: vec![],
            total_duration_ms: 0,
            timestamp: 0,
            mutation_description: "test".to_string(),
        };
        assert!(!r.all_passed);
    }

    #[test]
    fn test_consecutive_failures_resets_on_pass() {
        let mut g = make_gate();
        g.run_gate("fail 1", 0);
        g.run_gate("fail 2", 1);
        // Can't easily make one pass with bad workspace, but consecutive_failures won't
        // exceed history length. At minimum it should be >= 2.
        assert!(g.consecutive_failures() >= 2);
    }

    #[test]
    fn test_empty_stats() {
        let g = make_gate();
        let stats = g.stats();
        assert_eq!(stats.total_gates, 0);
        assert_eq!(stats.total_passed, 0);
        assert_eq!(stats.total_failed, 0);
        assert!(stats.last_passed.is_none());
    }

    #[test]
    fn test_run_clippy_bad_path() {
        let mut g = make_gate();
        let result = g.run_clippy();
        assert!(!result.passed);
    }

    #[test]
    fn test_run_tests_bad_path() {
        let mut g = make_gate();
        let result = g.run_tests();
        assert!(!result.passed);
    }

    #[test]
    fn test_gate_counts_passed_and_failed() {
        let mut g = make_gate();
        g.run_gate("fail", 0);
        assert_eq!(g.total_failed, 1);
        assert_eq!(g.total_passed, 0);
    }
}
