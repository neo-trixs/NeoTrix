//! Circuit Breaker - 熔断器
//!
//! 快速失败 + 自动恢复

use std::time::{Duration, Instant};

use super::CircuitState;

/// 熔断器
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    last_failure: Option<Instant>,
    recovery_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold: 3,
            last_failure: None,
            recovery_timeout,
        }
    }

    /// 记录失败
    pub fn record_failure(&mut self) -> CircuitState {
        self.failure_count += 1;
        self.success_count = 0;
        self.last_failure = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }

        self.state
    }

    /// 记录成功
    pub fn record_success(&mut self) -> CircuitState {
        self.success_count += 1;

        if self.state == CircuitState::HalfOpen && self.success_count >= self.success_threshold {
            self.state = CircuitState::Closed;
            self.failure_count = 0;
        }

        self.state
    }

    /// 检查是否允许执行
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last) = self.last_failure {
                    if last.elapsed() > self.recovery_timeout {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> CircuitState {
        self.state
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_normal() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(10));
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_trips() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(10));
        for _ in 0..3 {
            cb.record_failure();
        }
        assert!(!cb.allow_request());
    }
}
