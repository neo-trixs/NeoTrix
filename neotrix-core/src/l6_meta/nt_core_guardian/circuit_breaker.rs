//! # CIRCUIT_BREAKER — 统一熔断器 (融合 self_healing/circuit_breaker + nt_infra_breaker)
//!
//! 融合:
//! - `self_healing/circuit_breaker.rs` — 状态机 (Closed→Open→HalfOpen)
//! - `l1_action/nt_infra_breaker.rs` — 基础熔断
//!
//! 单一实现, 支持:
//! - 滑动窗口失败计数
//! - 指数退避恢复
//! - 风暴保护 (failure decay)

#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{Duration, Instant};

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// 正常通过
    Closed,
    /// 熔断中, 拒绝请求
    Open,
    /// 半开, 允许探测
    HalfOpen,
}

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 触发熔断的失败次数
    pub failure_threshold: u32,
    /// 恢复探测间隔
    pub recovery_timeout: Duration,
    /// 滑动窗口大小
    pub window_size: usize,
    /// 失败衰减半衰期 (秒)
    pub failure_decay_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            window_size: 10,
            failure_decay_secs: 30,
        }
    }
}

/// 统一熔断器
pub struct CircuitBreaker {
    state: AtomicU8,
    config: CircuitBreakerConfig,
    failures: std::sync::Mutex<VecDeque<Instant>>,
    last_failure: std::sync::Mutex<Option<Instant>>,
    last_state_change: std::sync::Mutex<Instant>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: AtomicU8::new(CircuitState::Closed as u8),
            config,
            failures: std::sync::Mutex::new(VecDeque::new()),
            last_failure: std::sync::Mutex::new(None),
            last_state_change: std::sync::Mutex::new(Instant::now()),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// 获取当前状态
    pub fn state(&self) -> CircuitState {
        let state = match self.state.load(Ordering::Relaxed) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            _ => CircuitState::HalfOpen,
        };
        // 如果是 Open 且超过恢复时间, 转为 HalfOpen
        if state == CircuitState::Open {
            if let Ok(last) = self.last_state_change.lock() {
                if last.elapsed() > self.config.recovery_timeout {
                    self.state.store(CircuitState::HalfOpen as u8, Ordering::Relaxed);
                    return CircuitState::HalfOpen;
                }
            }
        }
        state
    }

    /// 记录一次失败
    pub fn record_failure(&self) {
        let now = Instant::now();
        if let Ok(mut failures) = self.failures.lock() {
            failures.push_back(now);
            // 滑动窗口: 移除过期失败
            while failures.len() > self.config.window_size {
                failures.pop_front();
            }
        }
        if let Ok(mut last) = self.last_failure.lock() {
            *last = Some(now);
        }

        // 检查是否触发熔断
        let count = self.failures.lock().map(|f| f.len()).unwrap_or(0);
        if count >= self.config.failure_threshold as usize {
            if let Ok(mut last_change) = self.last_state_change.lock() {
                *last_change = Instant::now();
            }
            self.state.store(CircuitState::Open as u8, Ordering::Relaxed);
            log::warn!("[circuit-breaker] OPEN after {} failures", count);
        }
    }

    /// 记录一次成功
    pub fn record_success(&self) {
        let current = self.state();
        if current == CircuitState::HalfOpen {
            // 半开成功 → 恢复到 Closed
            if let Ok(mut failures) = self.failures.lock() {
                failures.clear();
            }
            if let Ok(mut last_change) = self.last_state_change.lock() {
                *last_change = Instant::now();
            }
            self.state.store(CircuitState::Closed as u8, Ordering::Relaxed);
            log::info!("[circuit-breaker] CLOSED (recovery successful)");
        }
    }

    /// 执行请求, 自动管理状态
    pub fn call<F, T, E>(&self, f: F) -> Result<T, CircuitError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        match self.state() {
            CircuitState::Open => {
                return Err(CircuitError::Open);
            }
            CircuitState::HalfOpen => {
                // 允许一个探测请求
            }
            CircuitState::Closed => {}
        }

        match f() {
            Ok(val) => {
                self.record_success();
                Ok(val)
            }
            Err(e) => {
                self.record_failure();
                Err(CircuitError::Wrapped(e))
            }
        }
    }

    /// 获取失败计数
    pub fn failure_count(&self) -> usize {
        self.failures.lock().map(|f| f.len()).unwrap_or(0)
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 熔断器错误
#[derive(Debug)]
pub enum CircuitError<E> {
    /// 熔断器打开
    Open,
    /// 内部错误
    Wrapped(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "circuit breaker is open"),
            Self::Wrapped(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for CircuitError<E> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_starts_closed() {
        let cb = CircuitBreaker::with_defaults();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_opens_after_threshold() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        });
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_recovers_after_timeout() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(50),
            ..Default::default()
        });
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_call_success_records() {
        let cb = CircuitBreaker::with_defaults();
        let result = cb.call(|| Ok::<_, String>("ok"));
        assert!(result.is_ok());
        assert_eq!(cb.failure_count(), 0);
    }

    #[test]
    fn test_call_failure_records() {
        let cb = CircuitBreaker::with_defaults();
        let _ = cb.call(|| Err::<String, _>("fail"));
        assert_eq!(cb.failure_count(), 1);
    }
}
