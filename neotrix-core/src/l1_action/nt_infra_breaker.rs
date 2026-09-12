//! L1 基础设施 — 断路器 (Circuit Breaker)
//!
//! 三态: Closed(正常) → Open(熔断) → HalfOpen(试探)
//! 触发: 错误率 > threshold → 熔断 duration_ms → 半开 → 成功则恢复

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use neotrix_types::shared::{BreakerState, CircuitBreaker as CanonicalCircuitBreaker};

/// 断路器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BreakerConfig {
    /// 错误率阈值 (0.0-1.0), 超过则熔断
    pub error_threshold: f64,
    /// 熔断持续时间 (ms)
    pub open_duration_ms: u64,
    /// 半开状态允许的试探次数
    pub half_open_max_calls: u32,
    /// 窗口大小 (最近 N 次调用)
    pub window_size: u32,
}

impl Default for BreakerConfig {
    fn default() -> Self {
        Self {
            error_threshold: 0.5,
            open_duration_ms: 30000,
            half_open_max_calls: 3,
            window_size: 10,
        }
    }
}

/// Infrastructure circuit breaker — wraps canonical `CircuitBreaker` with
/// error-rate sliding-window tracking and timestamp-based cooldown.
#[derive(Debug, Clone)]
pub(crate) struct InfraBreaker {
    inner: CanonicalCircuitBreaker,
    config: BreakerConfig,
    recent_results: Vec<bool>,
    open_since: Option<u64>,
    half_open_calls: u32,
    half_open_successes: u32,
}

/// Backward-compatible alias.
pub type CircuitBreaker = InfraBreaker;

impl InfraBreaker {
    pub fn new(config: BreakerConfig) -> Self {
        let cooldown_secs = config.open_duration_ms / 1000;
        Self {
            inner: CanonicalCircuitBreaker::new(config.half_open_max_calls as u64, cooldown_secs),
            config,
            recent_results: Vec::new(),
            open_since: None,
            half_open_calls: 0,
            half_open_successes: 0,
        }
    }

    /// 记录调用结果
    pub fn record_result(&mut self, success: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        match self.inner.state {
            BreakerState::Closed => {
                self.recent_results.push(success);
                if self.recent_results.len() > self.config.window_size as usize {
                    self.recent_results.remove(0);
                }
                if self.error_rate() >= self.config.error_threshold {
                    self.inner.state = BreakerState::Open;
                    self.inner.last_state_change = Some(std::time::Instant::now());
                    self.open_since = Some(now);
                }
            }
            BreakerState::Open => {
                if let Some(since) = self.open_since {
                    if now - since >= self.config.open_duration_ms {
                        self.inner.state = BreakerState::HalfOpen;
                        self.half_open_calls = 0;
                        self.half_open_successes = 0;
                    }
                }
            }
            BreakerState::HalfOpen => {
                self.half_open_calls += 1;
                if success { self.half_open_successes += 1; }
                if self.half_open_calls >= self.config.half_open_max_calls {
                    if self.half_open_successes as f64 / self.half_open_calls as f64 >= 0.5 {
                        self.inner.state = BreakerState::Closed;
                        self.recent_results.clear();
                    } else {
                        self.inner.state = BreakerState::Open;
                        self.inner.last_state_change = Some(std::time::Instant::now());
                        self.open_since = Some(now);
                    }
                }
            }
        }
    }

    /// 检查是否允许调用
    pub fn allow(&self) -> bool {
        match self.inner.state {
            BreakerState::Closed => true,
            BreakerState::Open { .. } => false,
            BreakerState::HalfOpen => self.half_open_calls < self.config.half_open_max_calls,
        }
    }

    pub fn state(&self) -> BreakerState { self.inner.state }
    pub fn error_rate(&self) -> f64 {
        if self.recent_results.is_empty() { return 0.0; }
        let failures = self.recent_results.iter().filter(|&&r| !r).count();
        failures as f64 / self.recent_results.len() as f64
    }
}

/// 断路器注册表 — 每个 capability 一个
pub struct BreakerRegistry {
    breakers: HashMap<String, InfraBreaker>,
    default_config: BreakerConfig,
}

impl Default for BreakerRegistry {
    fn default() -> Self { Self::new() }
}

impl BreakerRegistry {
    pub fn new() -> Self {
        Self {
            breakers: HashMap::new(),
            default_config: BreakerConfig::default(),
        }
    }

    pub fn with_config(config: BreakerConfig) -> Self {
        Self {
            breakers: HashMap::new(),
            default_config: config,
        }
    }

    pub fn get_or_create(&mut self, capability_id: &str) -> &mut InfraBreaker {
        self.breakers.entry(capability_id.to_string())
            .or_insert_with(|| InfraBreaker::new(self.default_config.clone()))
    }

    pub fn allow(&self, capability_id: &str) -> bool {
        self.breakers.get(capability_id)
            .map(|b| b.allow())
            .unwrap_or(true)
    }

    pub fn record_result(&mut self, capability_id: &str, success: bool) {
        if let Some(b) = self.breakers.get_mut(capability_id) {
            b.record_result(success);
        }
    }

    pub fn states(&self) -> HashMap<String, BreakerState> {
        self.breakers.iter().map(|(k, b)| (k.clone(), b.state())).collect()
    }
}

// 全局断路器注册表
lazy_static::lazy_static! {
    static ref GLOBAL_BREAKERS: Mutex<BreakerRegistry> = Mutex::new(BreakerRegistry::new());
}

pub(crate) fn breaker_allow(capability_id: &str) -> bool {
    GLOBAL_BREAKERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .allow(capability_id)
}

pub(crate) fn breaker_record(capability_id: &str, success: bool) {
    GLOBAL_BREAKERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .record_result(capability_id, success);
}

pub(crate) fn breaker_states() -> HashMap<String, BreakerState> {
    GLOBAL_BREAKERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .states()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breaker_closed_allows() {
        let mut b = InfraBreaker::new(BreakerConfig::default());
        assert!(b.allow());
        assert_eq!(b.state(), BreakerState::Closed);
    }

    #[test]
    fn test_breaker_opens_on_high_error_rate() {
        let config = BreakerConfig {
            error_threshold: 0.5,
            window_size: 4,
            ..Default::default()
        };
        let mut b = InfraBreaker::new(config);
        b.record_result(false);
        b.record_result(false);
        b.record_result(true);
        b.record_result(false); // 75% error → open
        assert_eq!(b.state(), BreakerState::Open);
        assert!(!b.allow());
    }

    #[test]
    fn test_breaker_half_open_recovery() {
        let config = BreakerConfig {
            error_threshold: 0.5,
            window_size: 2,
            open_duration_ms: 0, // instant for test
            half_open_max_calls: 2,
        };
        let mut b = InfraBreaker::new(config);
        b.record_result(false);
        b.record_result(false); // open
        assert_eq!(b.state(), BreakerState::Open);
        // After duration, allow → half-open
        b.record_result(true); // triggers half-open check
        assert!(b.allow() || b.state() == BreakerState::HalfOpen);
    }
}
