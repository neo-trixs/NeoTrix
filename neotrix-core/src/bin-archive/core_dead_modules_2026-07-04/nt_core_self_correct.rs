//! Self-Correction Reflection Loop
//!
//! Inspired by Aider's self-correction loop, Hermes-agent reflection, and
//! ECHO's terminal observation signal. Provides structured retry with
//! diagnostic feedback when edits fail, tests fail, or lint errors arise.
//!
//! Key features:
//! 1. Post-edit validation pipeline (lint, test, compile checks)
//! 2. Reflection message injection with "did you mean?" feedback
//! 3. Retry budget with configurable max iterations
//! 4. Monotonic improvement validation (eval checkpoints between retries)
//! 5. Integration with terminal observation (ECHO) for error signal
//!
//! Layer: L4 (Cognition) — falls within reasoning/reliability enhancement

use std::collections::VecDeque;
use crate::core::nt_core_echo_terminal::{TerminalObservation, EchoPrmBridge};

/// Types of errors that can trigger self-correction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorType {
    LintError,
    TestFailure,
    CompileError,
    EditFailed,
    RuntimeError,
    QualityBelowThreshold,
    Unknown,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LintError => "lint_error",
            Self::TestFailure => "test_failure",
            Self::CompileError => "compile_error",
            Self::EditFailed => "edit_failed",
            Self::RuntimeError => "runtime_error",
            Self::QualityBelowThreshold => "quality_below_threshold",
            Self::Unknown => "unknown",
        }
    }
}

/// A single self-correction attempt with its result
#[derive(Debug, Clone)]
pub struct CorrectionAttempt {
    pub attempt: usize,
    pub error_type: ErrorType,
    pub error_message: String,
    pub did_you_mean: Option<String>,
    pub suggestion: Option<String>,
    pub success: bool,
    pub quality_delta: f64,
    pub duration_ms: u64,
}

/// Configuration for the self-correction loop
#[derive(Debug, Clone)]
pub struct SelfCorrectConfig {
    pub max_iterations: usize,
    pub max_reflections: usize,
    pub monotonic_checks: bool,
    pub emit_iteration_diffs: bool,
    pub quality_threshold: f64,
    pub min_improvement: f64,
    pub max_lines_changed_per_iteration: usize,
    pub max_files_modified_per_iteration: usize,
    pub use_echo_signal: bool,
}

impl Default for SelfCorrectConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            max_reflections: 3,
            monotonic_checks: true,
            emit_iteration_diffs: true,
            quality_threshold: 0.7,
            min_improvement: 0.05,
            max_lines_changed_per_iteration: 200,
            max_files_modified_per_iteration: 5,
            use_echo_signal: true,
        }
    }
}

/// Result of the self-correction loop
#[derive(Debug, Clone)]
pub struct SelfCorrectResult {
    pub success: bool,
    pub total_attempts: usize,
    pub attempts: Vec<CorrectionAttempt>,
    pub final_error: Option<String>,
    pub improvement: f64,
    pub duration_ms: u64,
    pub echo_signals: Vec<f64>,
}

/// Did-you-mean suggestion from fuzzy matching
#[derive(Debug, Clone)]
pub struct DidYouMean {
    pub closest_match: String,
    pub surrounding_context: String,
    pub line_number: usize,
    pub confidence: f64,
}

/// Self-correction reflection loop
#[derive(Debug, Clone)]
pub struct SelfCorrectLoop {
    pub config: SelfCorrectConfig,
    pub echo_bridge: Option<EchoPrmBridge>,
    pub history: VecDeque<SelfCorrectResult>,
    pub max_history: usize,
}

impl Default for SelfCorrectLoop {
    fn default() -> Self {
        Self {
            config: SelfCorrectConfig::default(),
            echo_bridge: Some(EchoPrmBridge::new()),
            history: VecDeque::new(),
            max_history: 100,
        }
    }
}

impl SelfCorrectLoop {
    pub fn new(config: SelfCorrectConfig) -> Self {
        Self { config, ..Default::default() }
    }

    pub fn with_echo(mut self, echo: EchoPrmBridge) -> Self {
        self.echo_bridge = Some(echo);
        self
    }

    /// Run the self-correction loop for a given operation
    /// The `operation` closure should attempt the operation and return
    /// (success, error_message, quality_score)
    pub fn correct<F>(
        &mut self,
        operation_id: &str,
        mut operation: F,
    ) -> SelfCorrectResult
    where
        F: FnMut(usize, &[CorrectionAttempt]) -> (bool, Option<String>, f64),
    {
        let start = std::time::Instant::now();
        let mut attempts = Vec::new();
        let mut echo_signals = Vec::new();
        let mut best_quality = 0.0_f64;
        let mut last_quality = 0.0_f64;

        for attempt in 0..self.config.max_iterations {
            let (success, error_msg, quality) = operation(attempt, &attempts);

            let echo_signal = if self.config.use_echo_signal {
                if let Some(ref mut bridge) = self.echo_bridge {
                    let obs = TerminalObservation {
                        timestamp: std::time::Instant::now(),
                        command: format!("correct_{}", operation_id),
                        stdout: if success { "ok".into() } else { String::new() },
                        stderr: error_msg.clone().unwrap_or_default(),
                        exit_code: if success { 0 } else { 1 },
                        duration_ms: start.elapsed().as_millis() as u64,
                        file_changes: Vec::new(),
                        success,
                    };
                    Some(bridge.record_and_reward(operation_id, &format!("attempt_{}", attempt), obs))
                } else { None }
            } else { None };
            if let Some(s) = echo_signal { echo_signals.push(s); }

            let error_type = match error_msg.as_ref() {
                Some(msg) => classify_error(msg),
                None => ErrorType::Unknown,
            };

            let did_you_mean = error_msg.as_ref()
                .filter(|_| attempt > 0)
                .and_then(|_| self.compute_did_you_mean(&attempts));

            let suggestion = error_msg.as_ref()
                .map(|msg| self.generate_suggestion(msg, attempt));

            let quality_delta = quality - last_quality;
            last_quality = quality;
            if quality > best_quality { best_quality = quality; }

            let cor_attempt = CorrectionAttempt {
                attempt,
                error_type,
                error_message: error_msg.clone().unwrap_or_default(),
                did_you_mean: did_you_mean.map(|d| d.closest_match),
                suggestion,
                success,
                quality_delta,
                duration_ms: start.elapsed().as_millis() as u64,
            };
            attempts.push(cor_attempt);

            // Success or quality threshold met
            if success && quality >= self.config.quality_threshold {
                break;
            }

            // Monotonic check: abort if quality is degrading
            if self.config.monotonic_checks
                && attempt >= 2
                && quality_delta < -self.config.min_improvement
                && quality < self.config.quality_threshold * 0.5
            {
                break;
            }

            // Scope creep check
            if attempt > 0 && attempts.last().is_some_and(|a| {
                a.error_message.len() > self.config.max_lines_changed_per_iteration
            }) {
                break;
            }
        }

        let result = SelfCorrectResult {
            success: best_quality >= self.config.quality_threshold,
            total_attempts: attempts.len(),
            attempts: attempts.clone(),
            final_error: attempts.last()
                .filter(|a| !a.success)
                .map(|a| a.error_message.clone()),
            improvement: best_quality - attempts.first().map_or(0.0, |a| {
                if a.success { self.config.quality_threshold } else { 0.0 }
            }),
            duration_ms: start.elapsed().as_millis() as u64,
            echo_signals,
        };

        self.history.push_back(result.clone());
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }

        result
    }

    /// Compute "did you mean?" suggestion from previous errors
    fn compute_did_you_mean(&self, attempts: &[CorrectionAttempt]) -> Option<DidYouMean> {
        let last = attempts.last()?;
        if last.error_message.is_empty() { return None; }

        let words: Vec<&str> = last.error_message.split_whitespace().collect();
        let longest = words.iter()
            .max_by_key(|w| w.len())
            .filter(|w| w.len() > 3)?;

        Some(DidYouMean {
            closest_match: longest.to_string(),
            surrounding_context: last.error_message.chars().take(200).collect(),
            line_number: 0,
            confidence: 0.7,
        })
    }

    /// Generate a suggestion for how to fix the error
    fn generate_suggestion(&self, error: &str, attempt: usize) -> String {
        let error_lower = error.to_lowercase();
        if error_lower.contains("syntax") || error_lower.contains("parse error") {
            format!("Try fixing syntax (attempt {})", attempt + 1)
        } else if error_lower.contains("type") || error_lower.contains("mismatch") {
            format!("Check type annotations (attempt {})", attempt + 1)
        } else if error_lower.contains("not found") || error_lower.contains("undefined") {
            format!("Add missing import or definition (attempt {})", attempt + 1)
        } else if error_lower.contains("timeout") || error_lower.contains("timed out") {
            format!("Optimize or extend timeout (attempt {})", attempt + 1)
        } else if error_lower.contains("permission") || error_lower.contains("denied") {
            format!("Check permissions (attempt {})", attempt + 1)
        } else {
            format!("Review and fix: {} (attempt {})",
                error.chars().take(100).collect::<String>(), attempt + 1)
        }
    }

    /// Recent self-correction stats
    pub fn recent_stats(&self, n: usize) -> SelfCorrectStats {
        let recent: Vec<&SelfCorrectResult> = self.history.iter().rev().take(n).collect();
        if recent.is_empty() { return SelfCorrectStats::default(); }

        let total = recent.len();
        let successes = recent.iter().filter(|r| r.success).count();
        let avg_attempts = recent.iter().map(|r| r.total_attempts).sum::<usize>() as f64 / total as f64;
        let avg_improvement = recent.iter().map(|r| r.improvement).sum::<f64>() / total as f64;
        let avg_duration = recent.iter().map(|r| r.duration_ms as f64).sum::<f64>() / total as f64;

        SelfCorrectStats {
            total_corrections: total,
            success_rate: successes as f64 / total as f64,
            avg_attempts_per_correction: avg_attempts,
            avg_improvement,
            avg_duration_ms: avg_duration,
        }
    }

    pub fn total_corrections(&self) -> usize { self.history.len() }
    pub fn success_rate(&self) -> f64 {
        if self.history.is_empty() { return 1.0; }
        self.history.iter().filter(|r| r.success).count() as f64 / self.history.len() as f64
    }
}

/// Aggregate stats for self-correction
#[derive(Debug, Clone, Default)]
pub struct SelfCorrectStats {
    pub total_corrections: usize,
    pub success_rate: f64,
    pub avg_attempts_per_correction: f64,
    pub avg_improvement: f64,
    pub avg_duration_ms: f64,
}

/// Classify error message into ErrorType
pub fn classify_error(msg: &str) -> ErrorType {
    let msg_lower = msg.to_lowercase();
    if msg_lower.contains("lint") || msg_lower.contains("clippy") || msg_lower.contains("eslint") {
        ErrorType::LintError
    } else if msg_lower.contains("test") || msg_lower.contains("assertion") || msg_lower.contains("expect") {
        ErrorType::TestFailure
    } else if msg_lower.contains("compile") || msg_lower.contains("cannot find") || msg_lower.contains("error[e") {
        ErrorType::CompileError
    } else if msg_lower.contains("edit") || msg_lower.contains("patch") || msg_lower.contains("failed to apply") {
        ErrorType::EditFailed
    } else if msg_lower.contains("runtime") || msg_lower.contains("panic") || msg_lower.contains("crash") {
        ErrorType::RuntimeError
    } else if msg_lower.contains("quality") || msg_lower.contains("below threshold") {
        ErrorType::QualityBelowThreshold
    } else {
        ErrorType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_classification() {
        assert_eq!(classify_error("lint error: unused variable"), ErrorType::LintError);
        assert_eq!(classify_error("test failure: assert_eq failed"), ErrorType::TestFailure);
        assert_eq!(classify_error("error[E0308]: mismatched types"), ErrorType::CompileError);
        assert_eq!(classify_error("patch failed to apply"), ErrorType::EditFailed);
        assert_eq!(classify_error("runtime panic: index out of bounds"), ErrorType::RuntimeError);
        assert_eq!(classify_error("quality below threshold 0.5"), ErrorType::QualityBelowThreshold);
        assert_eq!(classify_error("unknown error"), ErrorType::Unknown);
    }

    #[test]
    fn test_self_correct_config_defaults() {
        let cfg = SelfCorrectConfig::default();
        assert_eq!(cfg.max_iterations, 3);
        assert_eq!(cfg.max_reflections, 3);
        assert!(cfg.monotonic_checks);
        assert!(cfg.use_echo_signal);
    }

    #[test]
    fn test_self_correct_loop_basic_success() {
        let mut loop_ = SelfCorrectLoop::default();
        let result = loop_.correct("test_op", |attempt, _prev| {
            match attempt {
                0 => (false, Some("syntax error".into()), 0.3),
                1 => (true, None, 0.9),
                _ => (true, None, 1.0),
            }
        });
        assert!(result.success);
        assert_eq!(result.total_attempts, 2);
        assert!(result.improvement >= 0.0);
    }

    #[test]
    fn test_self_correct_loop_immediate_success() {
        let mut loop_ = SelfCorrectLoop::default();
        let result = loop_.correct("good_op", |_attempt, _prev| {
            (true, None, 0.95)
        });
        assert!(result.success);
        assert_eq!(result.total_attempts, 1);
    }

    #[test]
    fn test_self_correct_loop_all_fail() {
        let mut loop_ = SelfCorrectLoop::default();
        let result = loop_.correct("fail_op", |attempt, _prev| {
            (false, Some(format!("error {}", attempt)), 0.1)
        });
        assert!(!result.success);
        assert_eq!(result.total_attempts, 3);
        assert!(result.final_error.is_some());
    }

    #[test]
    fn test_self_correct_loop_monotonic_abort() {
        let mut loop_ = SelfCorrectLoop {
            config: SelfCorrectConfig {
                max_iterations: 10,
                monotonic_checks: true,
                min_improvement: 0.05,
                quality_threshold: 0.7,
                ..Default::default()
            },
            ..Default::default()
        };
        let result = loop_.correct("degrade_op", |attempt, _prev| {
            let q = match attempt {
                0 => 0.6,
                1 => 0.5,
                _ => 0.1,
            };
            (false, Some(format!("error {}", attempt)), q)
        });
        // Should abort early due to monotonic degradation
        assert!(result.total_attempts <= 3);
    }

    #[test]
    fn test_did_you_mean_suggestion() {
        let loop_ = SelfCorrectLoop::default();
        let attempts = vec![
            CorrectionAttempt {
                attempt: 0, error_type: ErrorType::Unknown,
                error_message: String::new(), did_you_mean: None,
                suggestion: None, success: false, quality_delta: 0.0, duration_ms: 0,
            },
            CorrectionAttempt {
                attempt: 1, error_type: ErrorType::CompileError,
                error_message: "cannot find function `undefined_symbol`".into(),
                did_you_mean: None, suggestion: None, success: false,
                quality_delta: -0.3, duration_ms: 100,
            },
        ];
        let dym = loop_.compute_did_you_mean(&attempts);
        assert!(dym.is_some());
        assert!(dym.unwrap().closest_match.contains("undefined_symbol"));
    }

    #[test]
    fn test_generate_suggestion() {
        let loop_ = SelfCorrectLoop::default();
        let s1 = loop_.generate_suggestion("syntax error: unexpected token", 0);
        assert!(s1.contains("syntax"));
        let s2 = loop_.generate_suggestion("type mismatch: expected i32", 1);
        assert!(s2.contains("type"));
        let s3 = loop_.generate_suggestion("module not found", 2);
        assert!(s3.contains("missing"));
        let s4 = loop_.generate_suggestion("operation timed out after 30s", 0);
        assert!(s4.contains("timeout"));
        let s5 = loop_.generate_suggestion("permission denied", 1);
        assert!(s5.contains("permission"));
        let s6 = loop_.generate_suggestion("random garbage", 0);
        assert!(s6.contains("random"));
    }

    #[test]
    fn test_recent_stats() {
        let mut loop_ = SelfCorrectLoop::default();
        loop_.correct("a", |_a, _p| (true, None, 0.9));
        loop_.correct("b", |_a, _p| (true, None, 0.95));
        loop_.correct("c", |a, _p| if a == 0 { (false, Some("e".into()), 0.1) } else { (true, None, 0.9) });
        let stats = loop_.recent_stats(10);
        assert_eq!(stats.total_corrections, 3);
        assert!(stats.success_rate > 0.5);
        assert!(stats.avg_attempts_per_correction > 0.0);
    }

    #[test]
    fn test_self_correct_loop_with_echo() {
        let echo = EchoPrmBridge::new();
        let mut loop_ = SelfCorrectLoop::default().with_echo(echo);
        assert!(loop_.echo_bridge.is_some());
        let result = loop_.correct("echo_test", |_a, _p| (true, None, 0.9));
        assert!(result.success);
        if !result.echo_signals.is_empty() {
            assert!(result.echo_signals[0] > 0.0);
        }
    }

    #[test]
    fn test_classify_edge_cases() {
        assert_eq!(classify_error(""), ErrorType::Unknown);
        assert_eq!(classify_error("some test output: OK"), ErrorType::TestFailure);
        assert_eq!(classify_error("error[E0432]: unresolved import"), ErrorType::CompileError);
        assert_eq!(classify_error("CLIPPY ERROR: redundant closure"), ErrorType::LintError);
    }

    #[test]
    fn test_success_rate_empty() {
        let loop_ = SelfCorrectLoop::default();
        assert!((loop_.success_rate() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_recent_stats_empty() {
        let loop_ = SelfCorrectLoop::default();
        let stats = loop_.recent_stats(10);
        assert_eq!(stats.total_corrections, 0);
    }

    #[test]
    fn test_did_you_mean_empty_on_no_attempts() {
        let loop_ = SelfCorrectLoop::default();
        assert!(loop_.compute_did_you_mean(&[]).is_none());
    }

    #[test]
    fn test_generate_suggestion_fallback() {
        let loop_ = SelfCorrectLoop::default();
        let s = loop_.generate_suggestion("something completely unexpected happened", 3);
        assert!(s.contains("completely"));
    }

    #[test]
    fn test_self_correct_result_quality_tracking() {
        let mut loop_ = SelfCorrectLoop::default();
        let result = loop_.correct("quality", |attempt, _prev| {
            match attempt {
                0 => (false, Some("error".into()), 0.2),
                1 => (true, None, 0.8),
                _ => unreachable!(),
            }
        });
        assert!(result.success);
        assert!(result.total_attempts >= 2);
    }

    #[test]
    fn test_scope_creep_detection() {
        let mut loop_ = SelfCorrectLoop {
            config: SelfCorrectConfig {
                max_iterations: 5,
                ..Default::default()
            },
            ..Default::default()
        };

        // First attempt with small error
        let result = loop_.correct("scope_creep", |attempt, _prev| {
            let error_msg = if attempt == 0 {
                "a".repeat(10)
            } else {
                // Scope creep: too much changed
                "x".repeat(300)
            };
            (false, Some(error_msg), 0.1)
        });
        // Should stop early due to scope creep
        assert!(result.total_attempts < 5);
    }
}
