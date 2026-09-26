#![forbid(unsafe_code)]

use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::circuit_breaker::{CircuitBreaker, CircuitState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResilienceResult<T: Clone> {
    pub value: Option<T>,
    pub attempts: usize,
    pub circuit_state: CircuitState,
    pub success: bool,
}

impl<T: Clone> ResilienceResult<T> {
    pub fn success(value: T, attempts: usize, circuit_state: CircuitState) -> Self {
        Self {
            value: Some(value),
            attempts,
            circuit_state,
            success: true,
        }
    }

    pub fn failure(attempts: usize, circuit_state: CircuitState) -> Self {
        Self {
            value: None,
            attempts,
            circuit_state,
            success: false,
        }
    }
}

pub struct ResilienceManager {
    max_attempts: usize,
    backoff: Duration,
    timeout: Option<Duration>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl Default for ResilienceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResilienceManager {
    pub fn new() -> Self {
        Self {
            max_attempts: 3,
            backoff: Duration::from_millis(100),
            timeout: None,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        }
    }

    pub fn with_retry(mut self, max_attempts: usize, backoff: Duration) -> Self {
        self.max_attempts = max_attempts;
        self.backoff = backoff;
        self
    }

    pub fn with_circuit_breaker(mut self, failure_threshold: usize) -> Self {
        self.circuit_breaker = Arc::new(CircuitBreaker::new(
            failure_threshold,
            Duration::from_secs(30),
        ));
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn execute<T, F, E>(&self, op: F) -> ResilienceResult<T>
    where
        T: Clone,
        F: Fn() -> Result<T, E>,
        E: fmt::Display,
    {
        let mut attempts = 0;
        let start = Instant::now();

        // Fast-path: an already-open circuit blocks without consuming an attempt,
        // so callers can distinguish "rejected" (0) from "tried and failed" (N).
        if self.circuit_breaker.state() == CircuitState::Open {
            return ResilienceResult::failure(0, CircuitState::Open);
        }

        loop {
            attempts += 1;

            if let Some(timeout) = self.timeout {
                if start.elapsed() >= timeout {
                    return ResilienceResult::failure(attempts, self.circuit_breaker.state());
                }
            }

            let result = self.circuit_breaker.call(|| op());

            match result {
                Ok(val) => {
                    // Deadline applies to slow successes too: an op that
                    // completes after the timeout still reports failure.
                    if let Some(timeout) = self.timeout {
                        if start.elapsed() >= timeout {
                            return ResilienceResult::failure(
                                attempts,
                                self.circuit_breaker.state(),
                            );
                        }
                    }
                    return ResilienceResult::success(val, attempts, self.circuit_breaker.state());
                }
                Err(_) => {
                    if attempts >= self.max_attempts {
                        return ResilienceResult::failure(attempts, self.circuit_breaker.state());
                    }
                    let delay = self.backoff * attempts as u32;
                    std::thread::sleep(delay);
                }
            }
        }
    }

    pub fn circuit_state(&self) -> CircuitState {
        self.circuit_breaker.state()
    }

    pub fn reset_circuit(&self) {
        self.circuit_breaker.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resilience_manager_default() {
        let rm = ResilienceManager::new();
        assert_eq!(rm.circuit_state(), CircuitState::Closed);
    }

    #[test]
    fn test_execute_success_first_try() {
        let rm = ResilienceManager::new();
        let result = rm.execute(|| Ok::<_, String>("hello"));
        assert!(result.success);
        assert_eq!(result.attempts, 1);
        assert_eq!(result.value, Some("hello"));
    }

    #[test]
    fn test_execute_retry_on_failure() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let call_count = AtomicU32::new(0);
        let rm = ResilienceManager::new().with_retry(3, Duration::from_millis(10));
        let result = rm.execute(|| {
            let c = call_count.fetch_add(1, Ordering::SeqCst) + 1;
            if c < 3 {
                Err("transient error")
            } else {
                Ok("recovered")
            }
        });
        assert!(result.success);
        assert_eq!(result.attempts, 3);
        assert_eq!(result.value, Some("recovered"));
    }

    #[test]
    fn test_execute_exhausts_retries() {
        let rm = ResilienceManager::new().with_retry(2, Duration::from_millis(10));
        let result = rm.execute(|| Err::<String, _>("always fail"));
        assert!(!result.success);
        assert_eq!(result.attempts, 2);
        assert!(result.value.is_none());
    }

    #[test]
    fn test_execute_timeout() {
        let rm = ResilienceManager::new()
            .with_retry(100, Duration::from_millis(1))
            .with_timeout(Duration::from_millis(50));
        let result = rm.execute(|| {
            // Fixture must exceed the deadline: a 5ms op under a 50ms timeout
            // always succeeds, so the timeout path was never exercised.
            std::thread::sleep(Duration::from_millis(60));
            Ok::<_, String>("never".to_string())
        });
        assert!(!result.success);
    }

    #[test]
    fn test_circuit_breaker_stops_after_retries() {
        let rm = ResilienceManager::new()
            .with_retry(2, Duration::from_millis(10))
            .with_circuit_breaker(2);
        let _ = rm.execute(|| Err::<String, _>("1"));
        let _ = rm.execute(|| Err::<String, _>("2"));
        assert_eq!(rm.circuit_state(), CircuitState::Open);
        let result = rm.execute(|| Ok::<_, String>("blocked"));
        assert!(!result.success);
        assert_eq!(result.attempts, 0);
    }

    #[test]
    fn test_circuit_reset_allows_retry() {
        let rm = ResilienceManager::new()
            .with_retry(2, Duration::from_millis(10))
            .with_circuit_breaker(2);
        let _ = rm.execute(|| Err::<String, _>("1"));
        let _ = rm.execute(|| Err::<String, _>("2"));
        assert_eq!(rm.circuit_state(), CircuitState::Open);
        rm.reset_circuit();
        assert_eq!(rm.circuit_state(), CircuitState::Closed);
        let result = rm.execute(|| Ok::<_, String>("works now"));
        assert!(result.success);
    }

    #[test]
    fn test_builder_chain() {
        let rm = ResilienceManager::new()
            .with_retry(5, Duration::from_millis(200))
            .with_circuit_breaker(10)
            .with_timeout(Duration::from_secs(30));
        assert_eq!(rm.max_attempts, 5);
        assert_eq!(rm.backoff, Duration::from_millis(200));
        assert_eq!(rm.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_resilience_result_success_helpers() {
        let r = ResilienceResult::success("val".to_string(), 1, CircuitState::Closed);
        assert!(r.success);
        assert_eq!(r.value, Some("val".to_string()));
        assert_eq!(r.attempts, 1);
        assert_eq!(r.circuit_state, CircuitState::Closed);

        let r: ResilienceResult<String> = ResilienceResult::failure(3, CircuitState::Open);
        assert!(!r.success);
        assert!(r.value.is_none());
        assert_eq!(r.attempts, 3);
        assert_eq!(r.circuit_state, CircuitState::Open);
    }

    #[test]
    fn test_execute_with_immediate_success_no_retries() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let rm = ResilienceManager::new().with_retry(5, Duration::from_millis(10));
        let calls = AtomicU32::new(0);
        let result = rm.execute(|| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok::<_, String>("done")
        });
        assert!(result.success);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(result.attempts, 1);
    }

    #[test]
    fn test_execute_all_failures_returns_last_failure() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let rm = ResilienceManager::new().with_retry(3, Duration::from_millis(1));
        let calls = AtomicU32::new(0);
        let result = rm.execute(|| {
            let c = calls.fetch_add(1, Ordering::SeqCst) + 1;
            Err::<String, _>(format!("err_{}", c))
        });
        assert!(!result.success);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_execute_success_on_second_attempt() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let rm = ResilienceManager::new().with_retry(3, Duration::from_millis(1));
        let calls = AtomicU32::new(0);
        let result = rm.execute(|| {
            let c = calls.fetch_add(1, Ordering::SeqCst) + 1;
            if c == 1 {
                Err("transient")
            } else {
                Ok("recovered")
            }
        });
        assert!(result.success);
        assert_eq!(result.attempts, 2);
        assert_eq!(result.value, Some("recovered"));
    }

    #[test]
    fn test_circuit_breaker_opens_after_exhausted_retries() {
        let rm = ResilienceManager::new()
            .with_retry(2, Duration::from_millis(1))
            .with_circuit_breaker(2);
        let _ = rm.execute(|| Err::<String, _>("fail 1"));
        let _ = rm.execute(|| Err::<String, _>("fail 2"));
        assert_eq!(rm.circuit_state(), CircuitState::Open);

        // Next call should be blocked by circuit breaker (0 attempts used)
        let result = rm.execute(|| Ok::<_, String>("should not run"));
        assert!(!result.success);
        assert_eq!(result.attempts, 0);
    }

    #[test]
    fn test_timeout_stops_execution() {
        let rm = ResilienceManager::new()
            .with_retry(100, Duration::from_millis(50))
            .with_timeout(Duration::from_millis(100));
        let start = std::time::Instant::now();
        let result = rm.execute(|| {
            // Fixture must exceed the deadline (was 10ms op vs 100ms timeout,
            // which always succeeded before reaching the timeout path).
            std::thread::sleep(Duration::from_millis(150));
            Ok::<_, String>("late")
        });
        let elapsed = start.elapsed();
        assert!(!result.success);
        // Should have stopped well before 100 retries * 50ms
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn test_circuit_reset_allows_new_cycle() {
        let rm = ResilienceManager::new()
            .with_retry(1, Duration::from_millis(1))
            .with_circuit_breaker(1);
        let _ = rm.execute(|| Err::<String, _>("fail"));
        assert_eq!(rm.circuit_state(), CircuitState::Open);

        rm.reset_circuit();
        assert_eq!(rm.circuit_state(), CircuitState::Closed);

        let result = rm.execute(|| Ok::<_, String>("works"));
        assert!(result.success);
    }

    #[test]
    fn test_default_impl_matches_new() {
        let rm1 = ResilienceManager::new();
        let rm2 = ResilienceManager::default();
        assert_eq!(rm1.max_attempts, rm2.max_attempts);
        assert_eq!(rm1.backoff, rm2.backoff);
        assert_eq!(rm1.timeout, rm2.timeout);
    }

    #[test]
    fn test_execute_result_eq() {
        let r1 = ResilienceResult::success(42, 1, CircuitState::Closed);
        let r2 = ResilienceResult::success(42, 1, CircuitState::Closed);
        assert_eq!(r1, r2);
    }
}
