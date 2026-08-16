//! # +1 Observer — Three-Layer Error Recovery
//!
//! Implements a three-layer error recovery system:
//! 1. **Retry** — automatic retry with exponential backoff (max 3 attempts)
//! 2. **Circuit Breaker** — trip after N consecutive failures, half-open after timeout
//! 3. **Fallback** — degrade gracefully when all retries + circuit breaker fail

use std::time::{Duration, Instant};

use crate::core::nt_core_observer::ObserverReport;

/// Three-layer error recovery for the +1 observer.
#[derive(Debug, Clone)]
pub struct ObserverErrorRecovery {
    pub retry_config: RetryConfig,
    pub circuit_breaker: CircuitBreaker,
    pub fallback: FallbackHandler,
}

impl ObserverErrorRecovery {
    pub fn new() -> Self {
        Self {
            retry_config: RetryConfig::default(),
            circuit_breaker: CircuitBreaker::default(),
            fallback: FallbackHandler::default(),
        }
    }

    /// Execute an operation through retry → circuit breaker → fallback.
    ///
    /// 1. Checks the circuit breaker — if open, goes directly to fallback.
    /// 2. Attempts the operation with exponential backoff retries.
    /// 3. On success, records success on the circuit breaker.
    /// 4. On all retries exhausted, records failure on the circuit breaker.
    /// 5. If the circuit breaker trips or fallback is configured, returns a degraded report.
    pub fn execute<F>(&mut self, mut operation: F) -> Result<ObserverReport, ErrorRecoveryError>
    where
        F: FnMut() -> Result<ObserverReport, ErrorRecoveryError>,
    {
        // Step 1: Circuit breaker check
        if !self.circuit_breaker.allow_request() {
            return Ok(self.fallback.fallback());
        }

        // Step 2: Retry loop with exponential backoff
        let max_attempts = self.retry_config.max_attempts.max(1);
        let mut _last_error = None;

        for attempt in 1..=max_attempts {
            match operation() {
                Ok(report) => {
                    self.circuit_breaker.record_success();
                    return Ok(report);
                }
                Err(e) => {
                    _last_error = Some(e);
                    if attempt < max_attempts {
                        let delay = self.retry_config.delay(attempt);
                        std::thread::sleep(Duration::from_millis(delay));
                    }
                }
            }
        }

        // Step 3: All retries exhausted — record failure on circuit breaker
        self.circuit_breaker.record_failure();

        // Step 4: Fallback
        Ok(self.fallback.fallback())
    }
}

impl Default for ObserverErrorRecovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for retry with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (default: 3)
    pub max_attempts: u32,
    /// Base delay in milliseconds (default: 100)
    pub base_delay_ms: u64,
    /// Maximum delay in milliseconds (default: 5000)
    pub max_delay_ms: u64,
}

impl RetryConfig {
    /// Compute the delay for the nth attempt using exponential backoff.
    ///
    /// Formula: `min(max_delay_ms, base_delay_ms * 2^(attempt - 1))`
    pub fn delay(&self, attempt: u32) -> u64 {
        let exp = self
            .base_delay_ms
            .saturating_mul(2u64.saturating_pow(attempt.saturating_sub(1)));
        exp.min(self.max_delay_ms)
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

/// State of the circuit breaker.
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed — requests proceed normally.
    Closed,
    /// Circuit is open — requests are blocked.
    /// Contains the tick count when the circuit tripped.
    Open { since: usize },
    /// Circuit is half-open — a single test request is allowed.
    HalfOpen,
}

/// Circuit breaker that trips after N consecutive failures
/// and transitions to half-open after a timeout.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub state: CircuitState,
    /// Number of consecutive failures before the circuit trips (default: 3)
    pub failure_threshold: u32,
    /// Time in milliseconds before transitioning from open to half-open (default: 30000)
    pub half_open_timeout_ms: u64,
    /// Current consecutive failure count
    pub consecutive_failures: u32,
    /// Timestamp of when the circuit was last tripped (Instant for relative time)
    last_tripped: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, half_open_timeout_ms: u64) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_threshold,
            half_open_timeout_ms,
            consecutive_failures: 0,
            last_tripped: None,
        }
    }

    /// Check whether a request may proceed.
    ///
    /// - **Closed**: always allowed.
    /// - **Open**: denied unless the half-open timeout has elapsed (→ transitions to HalfOpen).
    /// - **HalfOpen**: allowed (single test request).
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open { since: _ } => {
                if let Some(tripped) = self.last_tripped {
                    if tripped.elapsed() >= Duration::from_millis(self.half_open_timeout_ms) {
                        self.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful operation.
    ///
    /// - Resets consecutive failures to 0.
    /// - If half-open, transitions back to closed.
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        if self.state == CircuitState::HalfOpen {
            self.state = CircuitState::Closed;
            self.last_tripped = None;
        }
    }

    /// Record a failed operation.
    ///
    /// - Increments consecutive failures.
    /// - If threshold is reached, transitions to Open and records the trip time.
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= self.failure_threshold {
            self.state = CircuitState::Open {
                since: self.consecutive_failures as usize,
            };
            self.last_tripped = Some(Instant::now());
        }
    }

    /// Reset the circuit breaker to its initial closed state.
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.consecutive_failures = 0;
        self.last_tripped = None;
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(3, 30000)
    }
}

/// Handles graceful degradation when the observer cannot produce a real report.
#[derive(Debug, Clone)]
pub struct FallbackHandler {
    /// Score to use in the degraded report (default: 0.5 — neutral)
    pub fallback_score: f64,
    /// Whether to degrade gracefully (vs. panic/error)
    pub degrade_gracefully: bool,
}

impl FallbackHandler {
    /// Produce a degraded ObserverReport.
    ///
    /// Returns a report with the fallback quality score and no actionable insights,
    /// representing a graceful degradation in observer capability.
    pub fn fallback(&self) -> ObserverReport {
        ObserverReport {
            trajectory_len: 0,
            distinct_states: 0,
            patterns: Vec::new(),
            step_qualities: Vec::new(),
            quality_score: self.fallback_score,
            recommended_meta: None,
            capability_deltas: Vec::new(),
            has_actionable_insight: false,
            critical_patterns: Vec::new(),
            trajectory_weighted_score: None,
            convergence_score: None,
            should_exit_early: None,
            step_attention: None,
        }
    }
}

impl Default for FallbackHandler {
    fn default() -> Self {
        Self {
            fallback_score: 0.5,
            degrade_gracefully: true,
        }
    }
}

/// Error type for observer error recovery.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorRecoveryError {
    /// An internal error occurred during the operation.
    Internal(String),
    /// All retry attempts were exhausted.
    RetriesExhausted { attempts: u32, last_error: String },
    /// The circuit breaker is open and rejected the request.
    CircuitOpen { consecutive_failures: u32 },
    /// The operation timed out.
    Timeout { duration_ms: u64 },
}

impl std::fmt::Display for ErrorRecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorRecoveryError::Internal(msg) => write!(f, "observer internal error: {}", msg),
            ErrorRecoveryError::RetriesExhausted {
                attempts,
                last_error,
            } => {
                write!(
                    f,
                    "retries exhausted after {} attempts: {}",
                    attempts, last_error
                )
            }
            ErrorRecoveryError::CircuitOpen {
                consecutive_failures,
            } => {
                write!(
                    f,
                    "circuit breaker open ({} consecutive failures)",
                    consecutive_failures
                )
            }
            ErrorRecoveryError::Timeout { duration_ms } => {
                write!(f, "observer operation timed out after {}ms", duration_ms)
            }
        }
    }
}

impl std::error::Error for ErrorRecoveryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_defaults() {
        let cfg = RetryConfig::default();
        assert_eq!(cfg.max_attempts, 3);
        assert_eq!(cfg.base_delay_ms, 100);
        assert_eq!(cfg.max_delay_ms, 5000);
    }

    #[test]
    fn test_retry_exponential_backoff() {
        let cfg = RetryConfig::default();
        assert_eq!(cfg.delay(1), 100);
        assert_eq!(cfg.delay(2), 200);
        assert_eq!(cfg.delay(3), 400);
        assert_eq!(cfg.delay(4), 800);
    }

    #[test]
    fn test_retry_delay_capped_at_max() {
        let cfg = RetryConfig {
            base_delay_ms: 2000,
            max_delay_ms: 5000,
            ..Default::default()
        };
        // 2000 * 2^2 = 8000 → capped to 5000
        assert_eq!(cfg.delay(3), 5000);
    }

    #[test]
    fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::default();
        assert_eq!(cb.state, CircuitState::Closed);
        assert_eq!(cb.consecutive_failures, 0);
    }

    #[test]
    fn test_circuit_breaker_trips_after_threshold() {
        let mut cb = CircuitBreaker::new(3, 30000);
        assert!(cb.allow_request());
        cb.record_failure();
        assert!(cb.allow_request());
        cb.record_failure();
        assert!(cb.allow_request());
        cb.record_failure();
        // After 3 failures, circuit should be open
        assert_eq!(cb.state, CircuitState::Open { since: 3 });
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_half_open_transition() {
        let mut cb = CircuitBreaker::new(1, 1); // trip after 1 failure, 1ms timeout
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open { since: 1 });
        // Immediately still open
        assert!(!cb.allow_request());
        // After 1ms, should transition to half-open
        std::thread::sleep(Duration::from_millis(2));
        assert!(cb.allow_request());
        assert_eq!(cb.state, CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_recovers_on_success() {
        let mut cb = CircuitBreaker::new(2, 1);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open { since: 2 });
        // Wait for half-open
        std::thread::sleep(Duration::from_millis(2));
        assert!(cb.allow_request()); // transitions to half-open
        cb.record_success();
        assert_eq!(cb.state, CircuitState::Closed);
        assert_eq!(cb.consecutive_failures, 0);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let mut cb = CircuitBreaker::new(2, 30000);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open { since: 2 });
        cb.reset();
        assert_eq!(cb.state, CircuitState::Closed);
        assert_eq!(cb.consecutive_failures, 0);
    }

    #[test]
    fn test_fallback_handler_returns_degraded_report() {
        let fb = FallbackHandler::default();
        let report = fb.fallback();
        assert_eq!(report.trajectory_len, 0);
        assert_eq!(report.quality_score, 0.5);
        assert!(!report.has_actionable_insight);
    }

    #[test]
    fn test_fallback_with_custom_score() {
        let fb = FallbackHandler {
            fallback_score: 0.3,
            degrade_gracefully: true,
        };
        let report = fb.fallback();
        assert_eq!(report.quality_score, 0.3);
    }

    #[test]
    fn test_execute_returns_ok_on_success() {
        let mut recovery = ObserverErrorRecovery::new();
        let result = recovery.execute(|| {
            Ok(ObserverReport {
                trajectory_len: 5,
                distinct_states: 3,
                patterns: Vec::new(),
                step_qualities: Vec::new(),
                quality_score: 0.8,
                recommended_meta: None,
                capability_deltas: Vec::new(),
                has_actionable_insight: false,
                critical_patterns: Vec::new(),
                trajectory_weighted_score: None,
                convergence_score: None,
                should_exit_early: None,
                step_attention: None,
            })
        });
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.trajectory_len, 5);
        assert_eq!(report.quality_score, 0.8);
    }

    #[test]
    fn test_execute_retries_on_failure_then_fallback() {
        let mut recovery = ObserverErrorRecovery::new();
        let attempt_counter = std::cell::Cell::new(0);
        let result = recovery.execute(|| {
            attempt_counter.set(attempt_counter.get() + 1);
            Err(ErrorRecoveryError::Internal("test error".into()))
        });
        // Should exhaust retries and return fallback
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.quality_score, 0.5);
        assert_eq!(report.trajectory_len, 0);
    }

    #[test]
    fn test_execute_succeeds_on_second_attempt() {
        let mut recovery = ObserverErrorRecovery::new();
        let attempt_counter = std::cell::Cell::new(0);
        let result = recovery.execute(|| {
            let attempt = attempt_counter.get();
            attempt_counter.set(attempt + 1);
            if attempt == 0 {
                Err(ErrorRecoveryError::Internal("transient error".into()))
            } else {
                Ok(ObserverReport {
                    trajectory_len: 3,
                    distinct_states: 2,
                    patterns: Vec::new(),
                    step_qualities: Vec::new(),
                    quality_score: 0.7,
                    recommended_meta: None,
                    capability_deltas: Vec::new(),
                    has_actionable_insight: false,
                    critical_patterns: Vec::new(),
                    trajectory_weighted_score: None,
                    convergence_score: None,
                    should_exit_early: None,
                    step_attention: None,
                })
            }
        });
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.trajectory_len, 3);
        assert_eq!(report.quality_score, 0.7);
    }

    #[test]
    fn test_execute_skips_when_circuit_open() {
        let mut recovery = ObserverErrorRecovery::new();
        // Trip the circuit breaker
        recovery.circuit_breaker.failure_threshold = 1;
        recovery.circuit_breaker.record_failure();
        assert!(!recovery.circuit_breaker.allow_request());

        // Execute should go directly to fallback without calling the operation
        let operation_called = std::cell::Cell::new(false);
        let result = recovery.execute(|| {
            operation_called.set(true);
            Ok(ObserverReport {
                trajectory_len: 99,
                distinct_states: 99,
                patterns: Vec::new(),
                step_qualities: Vec::new(),
                quality_score: 1.0,
                recommended_meta: None,
                capability_deltas: Vec::new(),
                has_actionable_insight: false,
                critical_patterns: Vec::new(),
                trajectory_weighted_score: None,
                convergence_score: None,
                should_exit_early: None,
                step_attention: None,
            })
        });
        assert!(result.is_ok());
        let report = result.unwrap();
        // Should be fallback, not the real result
        assert_eq!(report.quality_score, 0.5);
        assert_eq!(report.trajectory_len, 0);
        // Operation should NOT have been called
        assert!(!operation_called.get());
    }

    #[test]
    fn test_error_recovery_display() {
        let err = ErrorRecoveryError::Internal("oops".into());
        assert_eq!(format!("{}", err), "observer internal error: oops");

        let err = ErrorRecoveryError::RetriesExhausted {
            attempts: 3,
            last_error: "fail".into(),
        };
        assert_eq!(
            format!("{}", err),
            "retries exhausted after 3 attempts: fail"
        );

        let err = ErrorRecoveryError::CircuitOpen {
            consecutive_failures: 5,
        };
        assert_eq!(
            format!("{}", err),
            "circuit breaker open (5 consecutive failures)"
        );

        let err = ErrorRecoveryError::Timeout { duration_ms: 1000 };
        assert_eq!(
            format!("{}", err),
            "observer operation timed out after 1000ms"
        );
    }
}
