use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 熔断器状态
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// 熔断器 — 防止级联故障
pub struct CircuitBreaker {
    state: Arc<AtomicBool>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    threshold: u32,
    recovery_timeout: Duration,
    last_failure: Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: Arc::new(AtomicBool::new(false)),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            threshold,
            recovery_timeout,
            last_failure: std::sync::Mutex::new(None),
        }
    }

    pub fn is_open(&self) -> bool {
        if self.state.load(Ordering::Relaxed) {
            if let Some(last) = self.last_failure.lock().unwrap().as_ref() {
                if last.elapsed() > self.recovery_timeout {
                    self.state.store(false, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    return false;
                }
            }
            return true;
        }
        false
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure.lock().unwrap() = Some(Instant::now());
        if count >= self.threshold {
            self.state.store(true, Ordering::Relaxed);
        }
    }

    pub fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
    }

    pub fn state(&self) -> CircuitState {
        if self.is_open() {
            CircuitState::Open
        } else if self.failure_count.load(Ordering::Relaxed) > 0 {
            CircuitState::HalfOpen
        } else {
            CircuitState::Closed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert!(cb.is_open());
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_success_resets() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count.load(Ordering::Relaxed), 0);
    }
}
