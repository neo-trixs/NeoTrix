#![forbid(unsafe_code)]

use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

struct CircuitBreakerInner {
    failure_threshold: usize,
    reset_timeout: Duration,
    consecutive_failures: usize,
    state: CircuitState,
    last_failure: Option<Instant>,
}

pub struct CircuitBreaker {
    inner: Mutex<CircuitBreakerInner>,
    total_calls: AtomicUsize,
    total_failures: AtomicUsize,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, reset_timeout: Duration) -> Self {
        Self {
            inner: Mutex::new(CircuitBreakerInner {
                failure_threshold,
                reset_timeout,
                consecutive_failures: 0,
                state: CircuitState::Closed,
                last_failure: None,
            }),
            total_calls: AtomicUsize::new(0),
            total_failures: AtomicUsize::new(0),
        }
    }

    pub fn state(&self) -> CircuitState {
        let inner = self.inner.lock().unwrap();
        self.effective_state(&inner)
    }

    fn effective_state(&self, inner: &CircuitBreakerInner) -> CircuitState {
        match inner.state {
            CircuitState::Open => {
                if let Some(last) = inner.last_failure {
                    if last.elapsed() >= inner.reset_timeout {
                        CircuitState::HalfOpen
                    } else {
                        CircuitState::Open
                    }
                } else {
                    CircuitState::Open
                }
            }
            other => other,
        }
    }

    pub fn consecutive_failures(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.consecutive_failures
    }

    pub fn total_calls(&self) -> usize {
        self.total_calls.load(Ordering::Relaxed)
    }

    pub fn total_failures(&self) -> usize {
        self.total_failures.load(Ordering::Relaxed)
    }

    pub fn call<T, F, E>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, E>,
        E: fmt::Display,
    {
        self.total_calls.fetch_add(1, Ordering::Relaxed);

        // 记住进入调用时的有效状态: 半开探针的失败要区别于常规失败(见下)。
        let was_half_open = {
            let inner = self.inner.lock().unwrap();
            let state = self.effective_state(&inner);
            if state == CircuitState::Open {
                return Err(format!(
                    "circuit breaker is OPEN, rejecting call (failures: {}, threshold: {})",
                    inner.consecutive_failures, inner.failure_threshold
                ));
            }
            state == CircuitState::HalfOpen
        };

        match f() {
            Ok(val) => {
                let mut inner = self.inner.lock().unwrap();
                inner.consecutive_failures = 0;
                inner.state = CircuitState::Closed;
                Ok(val)
            }
            Err(e) => {
                self.total_failures.fetch_add(1, Ordering::Relaxed);
                let mut inner = self.inner.lock().unwrap();
                inner.last_failure = Some(Instant::now());
                if was_half_open {
                    // 半开探针失败 = 依赖**仍然坏着** => 立即回到 Open, 且计数
                    // **重置**为 1 开启新周期。
                    // 2026-09-28 修复: 原实现是 `+= 1` 累加, 于是冷却后的第一
                    // 次探针失败会把「曾经熔断的旧计数」继续往上加, 熔断器只能靠
                    // 成功脱困; 计数永不清零意味着每个新周期都从超阈值起步。
                    inner.consecutive_failures = 1;
                    inner.state = CircuitState::Open;
                } else {
                    inner.consecutive_failures += 1;
                    if inner.consecutive_failures >= inner.failure_threshold {
                        inner.state = CircuitState::Open;
                    }
                }
                Err(format!("{}", e))
            }
        }
    }

    pub fn reset(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.consecutive_failures = 0;
        inner.state = CircuitState::Closed;
        inner.last_failure = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_starts_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(30));
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_successful_call_stays_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(30));
        let _ = cb.call(|| Ok::<_, String>("ok"));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 0);
    }

    #[test]
    fn test_single_failure_stays_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(30));
        let _ = cb.call(|| Err::<String, _>("fail"));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 1);
    }

    #[test]
    fn test_threshold_trips_to_open() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        let _ = cb.call(|| Err::<String, _>("3"));
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_open_rejects_calls() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(60));
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        let result = cb.call(|| Ok::<_, String>("should not run"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("OPEN"));
    }

    #[test]
    fn test_half_open_after_timeout() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        assert_eq!(cb.state(), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_half_open_success_resets_to_closed() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        std::thread::sleep(Duration::from_millis(60));
        let _ = cb.call(|| Ok::<_, String>("recovered"));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 0);
    }

    #[test]
    fn test_half_open_failure_trips_again() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        std::thread::sleep(Duration::from_millis(60));
        let _ = cb.call(|| Err::<String, _>("still broken"));
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_reset_manually() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(60));
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        assert_eq!(cb.state(), CircuitState::Open);
        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 0);
    }

    #[test]
    fn test_total_counts() {
        let cb = CircuitBreaker::new(5, Duration::from_secs(60));
        let _ = cb.call(|| Ok::<_, String>("ok"));
        let _ = cb.call(|| Err::<String, _>("fail"));
        assert_eq!(cb.total_calls(), 2);
        assert_eq!(cb.total_failures(), 1);
    }

    #[test]
    fn test_success_resets_failure_count() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        let _ = cb.call(|| Ok::<_, String>("recovery"));
        assert_eq!(cb.consecutive_failures(), 0);
        let _ = cb.call(|| Err::<String, _>("1 again"));
        assert_eq!(cb.consecutive_failures(), 1);
    }

    #[test]
    fn test_circuit_state_display() {
        assert_eq!(format!("{}", CircuitState::Closed), "Closed");
        assert_eq!(format!("{}", CircuitState::Open), "Open");
        assert_eq!(format!("{}", CircuitState::HalfOpen), "HalfOpen");
    }

    #[test]
    fn test_full_lifecycle_closed_open_halfopen_closed() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        // Closed
        assert_eq!(cb.state(), CircuitState::Closed);

        // Fail twice → Open
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for timeout → HalfOpen
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Succeed in HalfOpen → Closed
        let _ = cb.call(|| Ok::<_, String>("recovered"));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 0);
        assert_eq!(cb.total_failures(), 2);
    }

    #[test]
    fn test_total_calls_counted_even_when_rejected() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(60));
        let _ = cb.call(|| Err::<String, _>("fail"));
        assert_eq!(cb.state(), CircuitState::Open);
        let _ = cb.call(|| Ok::<_, String>("blocked"));
        // Both calls counted even though second was rejected
        assert_eq!(cb.total_calls(), 2);
        assert_eq!(cb.total_failures(), 1);
    }

    #[test]
    fn test_multiple_successes_keep_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        for i in 0..10 {
            let _ = cb.call(|| Ok::<_, String>(format!("ok_{}", i)));
        }
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 0);
        assert_eq!(cb.total_calls(), 10);
        assert_eq!(cb.total_failures(), 0);
    }

    #[test]
    fn test_threshold_boundary_one_below() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 2);
    }

    #[test]
    fn test_half_open_failure_reopens() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        std::thread::sleep(Duration::from_millis(60));
        let _ = cb.call(|| Err::<String, _>("still broken"));
        assert_eq!(cb.state(), CircuitState::Open);
        assert_eq!(cb.consecutive_failures(), 1);
    }

    #[test]
    fn test_reset_from_half_open() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        let _ = cb.call(|| Err::<String, _>("1"));
        let _ = cb.call(|| Err::<String, _>("2"));
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 0);
    }

    #[test]
    fn test_concurrent_safe_creation() {
        let cb = std::sync::Arc::new(CircuitBreaker::new(5, Duration::from_secs(30)));
        let mut handles = vec![];
        for _ in 0..4 {
            let cb_clone = cb.clone();
            handles.push(std::thread::spawn(move || {
                let _ = cb_clone.call(|| Ok::<_, String>("ok"));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(cb.total_calls(), 4);
        assert_eq!(cb.total_failures(), 0);
    }
}
