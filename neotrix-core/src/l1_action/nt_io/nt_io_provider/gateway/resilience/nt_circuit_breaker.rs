//! 熔断 — CircuitBreaker half-open 状态机, 防止级联故障。
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::nt_resilience_types::CircuitState;

// ═══════════════════════════════════════════════════════════════════
// Circuit Breaker — 防止级联故障
// ═══════════════════════════════════════════════════════════════════
/// 熔断器 — 防止级联故障，含完整 half-open 状态机
pub struct CircuitBreaker {
    state: Arc<AtomicBool>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    threshold: u32,
    recovery_timeout: Duration,
    last_failure: Mutex<Option<Instant>>,
    half_open_max_calls: u32,
    half_open_calls: AtomicU32,
}
impl CircuitBreaker {
    /// Create a CircuitBreaker with the given failure threshold and recovery timeout.
    ///
    /// Note: Real implementation needs — default half-open max calls is 1. Consider:
    /// making threshold configurable per error type (rate limit vs server error),
    /// and adding a jitter to recovery timeout to prevent thundering herd.
    pub fn new(threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: Arc::new(AtomicBool::new(false)),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            threshold,
            recovery_timeout,
            last_failure: std::sync::Mutex::new(None),
            half_open_max_calls: 1,
            half_open_calls: AtomicU32::new(0),
        }
    }

    /// Set the maximum number of concurrent half-open probe requests.
    ///
    /// Note: Real implementation needs — higher values speed up recovery detection
    /// but increase load on a potentially unhealthy provider. Consider: adaptive
    /// probe count based on provider historical reliability.
    pub fn with_half_open_max(mut self, max: u32) -> Self {
        self.half_open_max_calls = max;
        self
    }

    /// Check if the circuit breaker should allow a request through.
    ///
    /// Note: Real implementation needs — the half-open state transition is triggered
    /// by checking `last_failure.elapsed() > recovery_timeout`. Consider adding:
    /// - Probe count tracking to limit concurrent half-open requests
    /// - Success rate threshold for half-open → closed transition
    /// - Gradual traffic increase during half-open (not just probe limits)
    pub fn should_allow(&self) -> bool {
        if self.state.load(Ordering::Relaxed) {
            // Open 状态：检查冷却期是否已过，尝试进入 half-open
            if let Some(last) = self.last_failure.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                if last.elapsed() > self.recovery_timeout {
                    // 冷却期已过 → 进入 half-open，允许探测
                    self.half_open_calls.store(0, Ordering::Relaxed);
                    return true;
                }
            }
            false
        } else if self.half_open_calls.load(Ordering::Relaxed) > 0 {
            // Half-open 状态：限制探测次数
            self.half_open_calls.load(Ordering::Relaxed) < self.half_open_max_calls
        } else {
            // Closed 状态：正常放行
            true
        }
    }

    /// Check if the circuit breaker is currently in the Open state.
    ///
    /// Note: Real implementation needs — auto-transitions to Closed if recovery
    /// timeout has elapsed. Consider: adding a callback for state transitions
    /// to notify monitoring systems.
    pub fn is_open(&self) -> bool {
        if self.state.load(Ordering::Relaxed) {
            if let Some(last) = self.last_failure.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                if last.elapsed() > self.recovery_timeout {
                    self.state.store(false, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.half_open_calls.store(0, Ordering::Relaxed);
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Record a failure and potentially trip the circuit breaker.
    ///
    /// Note: Real implementation needs — the threshold check is simple count-based.
    /// Consider: time-windowed failure counting (failures in last N seconds),
    /// error-type-aware tripping (transient errors count less than permanent ones),
    /// and per-model circuit breaking in addition to per-provider.
    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
        if count >= self.threshold {
            self.state.store(true, Ordering::Relaxed);
        }
    }

    /// Record a failure while allowing state transitions (used during half-open probes).
    ///
    /// Note: Real implementation needs — in Open state, resets cooldown without
    /// incrementing failure count. Consider: tracking probe-specific failure reasons
    /// to differentiate between transient and permanent failures.
    pub fn record_failure_allow_transition(&self) {
        if self.state.load(Ordering::Relaxed) {
            // Open 状态下的 failure — 保持 open 并重置冷却
            *self.last_failure.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
            return;
        }
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
        if count >= self.threshold {
            self.state.store(true, Ordering::Relaxed);
        }
    }

    /// Record a success, resetting failure count and potentially closing the circuit.
    ///
    /// Note: Real implementation needs — success during half-open immediately closes
    /// the circuit. Consider: requiring N consecutive successes before closing,
    /// and tracking success rate during half-open to detect flapping.
    pub fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        if self.state.load(Ordering::Relaxed) {
            // half-open probe succeeded → 恢复 closed
            self.state.store(false, Ordering::Relaxed);
            self.failure_count.store(0, Ordering::Relaxed);
            self.half_open_calls.store(0, Ordering::Relaxed);
        } else {
            self.failure_count.store(0, Ordering::Relaxed);
        }
    }

    /// Return the current circuit breaker state as a CircuitState enum.
    ///
    /// Note: Real implementation needs — the HalfOpen state is determined by
    /// `failure_count > 0` while not Open, which may be inaccurate after a
    /// successful half-open probe. Consider: tracking explicit half-open state.
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
impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(60))
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
