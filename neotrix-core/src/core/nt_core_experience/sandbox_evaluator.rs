use std::process::Command;
use std::time::Instant;

const TRUNCATE_LIMIT: usize = 2000;
const MAX_RESULTS: usize = 100;

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub compile_timeout_secs: u64,
    pub test_timeout_secs: u64,
    pub clippy_enabled: bool,
    pub max_sandbox_steps: usize,
    pub temp_dir: Option<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            compile_timeout_secs: 60,
            test_timeout_secs: 120,
            clippy_enabled: true,
            max_sandbox_steps: 5,
            temp_dir: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub compile_passed: bool,
    pub compile_output: String,
    pub tests_passed: bool,
    pub test_output: String,
    pub clippy_passed: bool,
    pub clippy_output: String,
    pub score: f64,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SandboxEvaluator {
    pub config: SandboxConfig,
    pub results: Vec<SandboxResult>,
    pub total_evals: u64,
    pub passed_evals: u64,
}

impl SandboxEvaluator {
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
            results: Vec::new(),
            total_evals: 0,
            passed_evals: 0,
        }
    }

    pub fn new_with_config(config: SandboxConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            total_evals: 0,
            passed_evals: 0,
        }
    }

    pub fn evaluate(&mut self, workspace_path: &str, _cargo_args: &[&str]) -> SandboxResult {
        let start = Instant::now();

        let (compile_passed, compile_output) =
            run_command("cargo", &["check", "--lib"], workspace_path, self.config.compile_timeout_secs);

        let (tests_passed, test_output) =
            run_command("cargo", &["test", "--lib"], workspace_path, self.config.test_timeout_secs);

        let (clippy_passed, clippy_output) = if self.config.clippy_enabled {
            run_command(
                "cargo",
                &["clippy", "--lib", "-D", "warnings"],
                workspace_path,
                self.config.compile_timeout_secs,
            )
        } else {
            (true, String::new())
        };

        let mut score = 0.0_f64;
        if compile_passed {
            score += 1.0 / 3.0;
        }
        if tests_passed {
            score += 1.0 / 3.0;
        }
        if clippy_passed {
            score += 1.0 / 3.0;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        let result = SandboxResult {
            compile_passed,
            compile_output: truncate(&compile_output),
            tests_passed,
            test_output: truncate(&test_output),
            clippy_passed,
            clippy_output: truncate(&clippy_output),
            score,
            duration_ms,
            error: None,
        };

        self.total_evals += 1;
        if score >= 1.0 {
            self.passed_evals += 1;
        }

        self.results.push(result.clone());
        if self.results.len() > MAX_RESULTS {
            self.results.remove(0);
        }

        result
    }

    pub fn evaluate_mutation(
        &mut self,
        workspace_path: &str,
        _original_hash: &str,
        _mutated_file: &str,
        _mutation_description: &str,
    ) -> SandboxResult {
        let mut result = self.evaluate(workspace_path, &[]);
        result.error = Some(format!(
            "mutation: {} on {}",
            _mutation_description, _mutated_file
        ));
        if let Some(last) = self.results.last_mut() {
            last.error = result.error.clone();
        }
        result
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_evals == 0 {
            return 0.0;
        }
        self.passed_evals as f64 / self.total_evals as f64
    }

    pub fn last_results(&self, n: usize) -> &[SandboxResult] {
        let len = self.results.len();
        if len == 0 || n == 0 {
            return &[];
        }
        let start = if n >= len { 0 } else { len - n };
        &self.results[start..]
    }

    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    pub fn summary(&self) -> String {
        let last_score = self.results.last().map(|r| r.score).unwrap_or(0.0);
        format!(
            "SandboxEvaluator: evals={} passed={} rate={:.2}% last_score={:.2}",
            self.total_evals,
            self.passed_evals,
            self.success_rate() * 100.0,
            last_score
        )
    }
}

impl Default for SandboxEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

fn truncate(s: &str) -> String {
    if s.len() > TRUNCATE_LIMIT {
        format!("{}... [truncated {} chars]", &s[..TRUNCATE_LIMIT], s.len())
    } else {
        s.to_string()
    }
}

/// Run a command, capture combined output, enforce a simple polling timeout.
/// Returns (success, output_string).
fn run_command(cmd: &str, args: &[&str], dir: &str, timeout_secs: u64) -> (bool, String) {
    let start = Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    let mut child = match Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return (false, format!("failed to spawn {}: {}", cmd, e)),
    };

    let poll_interval = std::time::Duration::from_millis(100);
    let output = loop {
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return (false, format!("timeout after {}s", timeout_secs));
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output().unwrap_or_else(|e| {
                    std::process::Output {
                        status,
                        stdout: Vec::new(),
                        stderr: format!("wait_with_output failed: {}", e).into_bytes(),
                    }
                });
                break output;
            }
            Ok(None) => {
                std::thread::sleep(poll_interval);
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return (false, format!("process error: {}", e));
            }
        }
    };

    let combined = if output.stderr.is_empty() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.trim().is_empty() {
            stdout.to_string()
        } else {
            format!("{}\n{}", stdout, stderr)
        }
    };

    (output.status.success(), combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ev = SandboxEvaluator::new();
        assert_eq!(ev.config.compile_timeout_secs, 60);
        assert_eq!(ev.config.test_timeout_secs, 120);
        assert!(ev.config.clippy_enabled);
        assert_eq!(ev.config.max_sandbox_steps, 5);
        assert!(ev.config.temp_dir.is_none());
        assert!(ev.results.is_empty());
        assert_eq!(ev.total_evals, 0);
        assert_eq!(ev.passed_evals, 0);
    }

    #[test]
    fn test_new_with_config() {
        let cfg = SandboxConfig {
            compile_timeout_secs: 30,
            test_timeout_secs: 60,
            clippy_enabled: false,
            max_sandbox_steps: 3,
            temp_dir: Some("/tmp/sbx".into()),
        };
        let ev = SandboxEvaluator::new_with_config(cfg.clone());
        assert_eq!(ev.config.compile_timeout_secs, 30);
        assert_eq!(ev.config.test_timeout_secs, 60);
        assert!(!ev.config.clippy_enabled);
        assert_eq!(ev.config.max_sandbox_steps, 3);
        assert_eq!(ev.config.temp_dir, Some("/tmp/sbx".into()));
    }

    #[test]
    fn test_run_command_success() {
        let (ok, out) = run_command("echo", &["hello"], ".", 5);
        assert!(ok);
        assert!(out.contains("hello"));
    }

    #[test]
    fn test_run_command_failure() {
        let (ok, _) = run_command("false", &[], ".", 5);
        assert!(!ok);
    }

    #[test]
    fn test_run_command_timeout() {
        let (ok, out) = run_command("sleep", &["5"], ".", 1);
        assert!(!ok);
        assert!(out.contains("timeout"));
    }

    #[test]
    fn test_evaluate_simulated() {
        let mut ev = SandboxEvaluator::new();
        let result = SandboxResult {
            compile_passed: true,
            compile_output: "compiled OK".into(),
            tests_passed: true,
            test_output: "all tests passed".into(),
            clippy_passed: true,
            clippy_output: "no warnings".into(),
            score: 1.0,
            duration_ms: 500,
            error: None,
        };
        ev.total_evals += 1;
        ev.passed_evals += 1;
        ev.results.push(result);

        assert_eq!(ev.total_evals, 1);
        assert_eq!(ev.passed_evals, 1);
        assert_eq!(ev.results.len(), 1);
        assert_eq!(ev.results[0].score, 1.0);
    }

    #[test]
    fn test_success_rate() {
        let mut ev = SandboxEvaluator::new();
        ev.total_evals = 4;
        ev.passed_evals = 3;
        assert!((ev.success_rate() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_success_rate_empty() {
        let ev = SandboxEvaluator::new();
        assert_eq!(ev.success_rate(), 0.0);
    }

    #[test]
    fn test_last_results() {
        let mut ev = SandboxEvaluator::new();
        for i in 0..5 {
            ev.results.push(SandboxResult {
                compile_passed: true,
                compile_output: format!("compile {}", i),
                tests_passed: true,
                test_output: format!("test {}", i),
                clippy_passed: true,
                clippy_output: format!("clippy {}", i),
                score: 1.0,
                duration_ms: 100,
                error: None,
            });
        }
        let last = ev.last_results(3);
        assert_eq!(last.len(), 3);
        assert!(last[0].compile_output.contains("compile 2"));
        assert!(last[2].compile_output.contains("compile 4"));
    }

    #[test]
    fn test_last_results_zero() {
        let ev = SandboxEvaluator::new();
        assert!(ev.last_results(0).is_empty());
        assert!(ev.last_results(5).is_empty());
    }

    #[test]
    fn test_clear_results() {
        let mut ev = SandboxEvaluator::new();
        ev.results.push(SandboxResult {
            compile_passed: true,
            compile_output: String::new(),
            tests_passed: true,
            test_output: String::new(),
            clippy_passed: true,
            clippy_output: String::new(),
            score: 1.0,
            duration_ms: 0,
            error: None,
        });
        assert_eq!(ev.results.len(), 1);
        ev.clear_results();
        assert!(ev.results.is_empty());
    }

    #[test]
    fn test_summary() {
        let mut ev = SandboxEvaluator::new();
        ev.total_evals = 10;
        ev.passed_evals = 7;
        ev.results.push(SandboxResult {
            compile_passed: true,
            compile_output: String::new(),
            tests_passed: true,
            test_output: String::new(),
            clippy_passed: true,
            clippy_output: String::new(),
            score: 1.0,
            duration_ms: 200,
            error: None,
        });
        let s = ev.summary();
        assert!(s.contains("evals=10"));
        assert!(s.contains("passed=7"));
        assert!(s.contains("rate=70.00%"));
        assert!(s.contains("last_score=1.00"));
    }

    #[test]
    fn test_evaluate_mutation_sets_error() {
        let mut ev = SandboxEvaluator::new();
        let result = SandboxResult {
            compile_passed: true,
            compile_output: "ok".into(),
            tests_passed: false,
            test_output: "fail".into(),
            clippy_passed: true,
            clippy_output: "clean".into(),
            score: 2.0 / 3.0,
            duration_ms: 300,
            error: Some("mutation: fix overflow on src/main.rs".into()),
        };
        ev.total_evals += 1;
        ev.results.push(result);
        assert_eq!(ev.results[0].error.as_deref(), Some("mutation: fix overflow on src/main.rs"));
    }

    #[test]
    fn test_truncate_long_output() {
        let long = "a".repeat(5000);
        let t = truncate(&long);
        assert!(t.len() < 3000);
        assert!(t.contains("truncated"));
    }

    #[test]
    fn test_truncate_short_output() {
        let short = "hello".to_string();
        let t = truncate(&short);
        assert_eq!(t, "hello");
    }
}
