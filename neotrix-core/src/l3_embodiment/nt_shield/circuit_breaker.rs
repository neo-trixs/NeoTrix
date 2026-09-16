use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Circuit breaker state machine for resilient external calls.
/// Prevents cascading failures by tripping when a service is unhealthy.
pub struct CircuitBreaker {
    state: Arc<Mutex<BreakerState>>,
    failure_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    failure_threshold: u64,
    recovery_timeout: Duration,
    half_open_max_calls: u64,
    last_failure_time: Arc<AtomicU64>,
    closed_at: Arc<AtomicU64>,
}

/// Current state of the circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    /// Normal operation. Calls pass through.
    Closed,
    /// Circuit is open. All calls fail fast without attempting.
    Open,
    /// Trial mode. Limited calls are allowed to test recovery.
    HalfOpen,
}

/// Result of a circuit breaker call attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerOutcome {
    /// Call succeeded and circuit is healthy.
    Success,
    /// Call failed but circuit is still operational.
    Failure,
    /// Call was rejected because circuit is open.
    OpenRejected,
    /// Call was allowed in half-open state for testing.
    HalfOpenProbe,
}

/// Error when a call is rejected by an open circuit breaker.
#[derive(Debug, thiserror::Error)]
#[error("Circuit breaker is open — call rejected after {failure_count} failures")]
pub struct CircuitBreakerOpenError {
    pub failure_count: u64,
    pub last_failure: Option<Duration>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    ///
    /// # Arguments
    /// * `failure_threshold` - Number of consecutive failures before opening
    /// * `recovery_timeout` - Time to wait before entering half-open state
    /// * `half_open_max_calls` - Max calls allowed in half-open state
    pub fn new(
        failure_threshold: u64,
        recovery_timeout: Duration,
        half_open_max_calls: u64,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(BreakerState::Closed)),
            failure_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            failure_threshold,
            recovery_timeout,
            half_open_max_calls,
            last_failure_time: Arc::new(AtomicU64::new(0)),
            closed_at: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Attempt to execute a call through the circuit breaker.
    /// Returns `Ok(())` if the call should proceed, or `Err` if rejected.
    pub fn call<F, R>(&self, operation: F) -> Result<R, CircuitBreakerOpenError>
    where
        F: FnOnce() -> Result<R, ()>,
    {
        let state = self.state.lock().map_err(|_| {
            CircuitBreakerOpenError {
                failure_count: self.failure_count.load(Ordering::SeqCst),
                last_failure: None,
            }
        })?;

        match *state {
            BreakerState::Closed => {
                drop(state);
                self.execute_call(operation)
            }
            BreakerState::Open => {
                let elapsed = self.elapsed_since_last_failure();
                if elapsed >= self.recovery_timeout.as_millis() as u64 {
                    drop(state);
                    self.transition_to_half_open();
                    self.execute_call(operation)
                } else {
                    drop(state);
                    let failure_count = self.failure_count.load(Ordering::SeqCst);
                    Err(CircuitBreakerOpenError {
                        failure_count,
                        last_failure: Some(Duration::from_millis(elapsed)),
                    })
                }
            }
            BreakerState::HalfOpen => {
                let current_calls = self.success_count.load(Ordering::SeqCst)
                    + self.failure_count.load(Ordering::SeqCst);
                if current_calls < self.half_open_max_calls {
                    drop(state);
                    self.execute_call(operation)
                } else {
                    drop(state);
                    let failure_count = self.failure_count.load(Ordering::SeqCst);
                    Err(CircuitBreakerOpenError {
                        failure_count,
                        last_failure: None,
                    })
                }
            }
        }
    }

    /// Execute the operation and handle the result.
    fn execute_call<F, R>(&self, operation: F) -> Result<R, CircuitBreakerOpenError>
    where
        F: FnOnce() -> Result<R, ()>,
    {
        match operation() {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(_) => {
                self.on_failure();
                let failure_count = self.failure_count.load(Ordering::SeqCst);
                Err(CircuitBreakerOpenError {
                    failure_count,
                    last_failure: Some(Duration::from_millis(0)),
                })
            }
        }
    }

    /// Record a successful call.
    fn on_success(&self) {
        self.success_count.fetch_add(1, Ordering::SeqCst);
        if self.success_count.load(Ordering::SeqCst) >= self.half_open_max_calls {
            self.transition_to_closed();
        }
    }

    /// Record a failed call.
    fn on_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.last_failure_time.store(
            Instant::now().duration_since(Instant::now()).subsec_nanos() as u64,
            Ordering::SeqCst,
        );
        if count >= self.failure_threshold {
            self.transition_to_open();
        }
    }

    /// Transition to OPEN state.
    fn transition_to_open(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        *state = BreakerState::Open;
        self.closed_at.store(Instant::now().elapsed().as_millis() as u64, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        log::warn!("[circuit-breaker] transitioned to OPEN");
    }

    /// Transition to CLOSED state after successful recovery.
    fn transition_to_closed(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        *state = BreakerState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        log::info!("[circuit-breaker] transitioned to CLOSED");
    }

    /// Transition to HALF_OPEN state to test recovery.
    fn transition_to_half_open(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        *state = BreakerState::HalfOpen;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        log::info!("[circuit-breaker] transitioned to HALF_OPEN");
    }

    /// Get the current state.
    pub fn state(&self) -> BreakerState {
        self.state.lock().map(|s| *s).unwrap_or(BreakerState::Closed)
    }

    /// Get the current failure count.
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::SeqCst)
    }

    /// Get the current success count.
    pub fn success_count(&self) -> u64 {
        self.success_count.load(Ordering::SeqCst)
    }

    /// Check if the circuit is currently open.
    pub fn is_open(&self) -> bool {
        self.state() == BreakerState::Open
    }

    /// Check if the circuit is currently closed (operational).
    pub fn is_closed(&self) -> bool {
        self.state() == BreakerState::Closed
    }

    /// Time elapsed since last failure in milliseconds.
    fn elapsed_since_last_failure(&self) -> u64 {
        let last_failure = self.last_failure_time.load(Ordering::SeqCst);
        let now = Instant::now().duration_since(Instant::now()).subsec_nanos() as u64;
        now.saturating_sub(last_failure)
    }

    /// Reset the circuit breaker to closed state.
    pub fn reset(&self) {
        self.transition_to_closed();
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(30), 3)
    }
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            failure_count: Arc::clone(&self.failure_count),
            success_count: Arc::clone(&self.success_count),
            failure_threshold: self.failure_threshold,
            recovery_timeout: self.recovery_timeout,
            half_open_max_calls: self.half_open_max_calls,
            last_failure_time: Arc::clone(&self.last_failure_time),
            closed_at: Arc::clone(&self.closed_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_circuit_breaker_closed_by_default() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1), 2);
        assert!(cb.is_closed());
        assert!(!cb.is_open());
    }

    #[test]
    fn test_success_calls() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1), 2);
        for _ in 0..5 {
            let result = cb.call(|| Ok::<_, ()>(42));
            assert!(result.is_ok());
        }
        assert!(cb.is_closed());
    }

    #[test]
    fn test_failure_threshold_opens() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1), 2);
        for _ in 0..3 {
            let result = cb.call(|| Err::<i32, ()>(()));
            assert!(result.is_err());
        }
        assert!(cb.is_open());
    }

    #[test]
    fn test_open_rejects_calls() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(1), 2);
        let _ = cb.call(|| Err::<(), ()>(()));
        let _ = cb.call(|| Err::<(), ()>(()));
        assert!(cb.is_open());
        let result = cb.call(|| Ok::<(), ()>(()));
        assert!(result.is_err());
    }

    #[test]
    fn test_half_open_recovery() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100), 2);
        for _ in 0..2 {
            let _ = cb.call(|| Err::<(), ()>(()));
        }
        assert!(cb.is_open());
        thread::sleep(Duration::from_millis(150));
        let result = cb.call(|| Ok::<(), ()>(()));
        assert!(result.is_ok() || cb.state() == BreakerState::HalfOpen);
    }

    #[test]
    fn test_clone_shares_state() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(1), 2);
        let cb2 = cb.clone();
        let _ = cb.call(|| Err::<(), ()>(()));
        assert_eq!(cb.failure_count(), cb2.failure_count());
    }

    #[test]
    fn test_reset() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(1), 2);
        let _ = cb.call(|| Err::<(), ()>(()));
        let _ = cb.call(|| Err::<(), ()>(()));
        assert!(cb.is_open());
        cb.reset();
        assert!(cb.is_closed());
    }
}
