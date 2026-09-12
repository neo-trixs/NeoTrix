//! Circuit Breaker — 故障熔断器
//!
//! 吸收 KB 经验:
//! - 三态熔断 (Closed → Open → Half-Open)
//! - 故障计数/超时恢复
//! - 降级策略
//! - 指标收集

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// 熔断器
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    config: CircuitBreakerConfig,
    stats: CircuitBreakerStats,
}

/// 熔断器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub half_open_max_calls: u32,
    pub fallback_strategy: FallbackStrategy,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
            fallback_strategy: FallbackStrategy::ReturnDefault,
        }
    }
}

/// 熔断器状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CircuitState {
    Closed,    // 正常
    Open,      // 熔断
    HalfOpen,  // 半开
}

/// 降级策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FallbackStrategy {
    ReturnDefault,
    ReturnCached,
    ReturnError,
    RetryWithBackoff,
    QueueForLater,
}

/// 熔断器统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CircuitBreakerStats {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub rejected_calls: u64,
    pub state_changes: u32,
    pub avg_response_time: Duration,
}

/// 熔断器结果
#[derive(Debug, Clone)]
pub(crate) enum CircuitBreakerResult<T> {
    Success(T),
    Fallback(T),
    Rejected(String),
    Timeout,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            config,
            stats: CircuitBreakerStats {
                total_calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                rejected_calls: 0,
                state_changes: 0,
                avg_response_time: Duration::from_millis(0),
            },
        }
    }

    /// 执行受保护的调用
    pub fn call<T, F, E>(&mut self, operation: F, fallback: Option<Box<dyn Fn() -> T>>) -> CircuitBreakerResult<T>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        self.stats.total_calls += 1;

        match self.state {
            CircuitState::Closed => {
                self.handle_closed(operation, fallback)
            }
            CircuitState::Open => {
                self.handle_open()
            }
            CircuitState::HalfOpen => {
                self.handle_half_open(operation, fallback)
            }
        }
    }

    /// 处理 Closed 状态
    fn handle_closed<T, F, E>(&mut self, operation: F, fallback: Option<Box<dyn Fn() -> T>>) -> CircuitBreakerResult<T>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        match operation() {
            Ok(result) => {
                self.on_success();
                CircuitBreakerResult::Success(result)
            }
            Err(e) => {
                self.on_failure();
                if let Some(fallback_fn) = fallback {
                    CircuitBreakerResult::Fallback(fallback_fn())
                } else {
                    match self.config.fallback_strategy {
                        FallbackStrategy::ReturnDefault => CircuitBreakerResult::Rejected(format!("Default fallback: {}", e)),
                        FallbackStrategy::ReturnError => CircuitBreakerResult::Rejected(e.to_string()),
                        _ => CircuitBreakerResult::Rejected(e.to_string()),
                    }
                }
            }
        }
    }

    /// 处理 Open 状态
    fn handle_open<T>(&mut self) -> CircuitBreakerResult<T> {
        // 检查是否应该转换到 Half-Open
        if let Some(last_failure) = self.last_failure_time {
            if last_failure.elapsed() > self.config.timeout {
                self.state = CircuitState::HalfOpen;
                self.stats.state_changes += 1;
                self.success_count = 0;
                return CircuitBreakerResult::Rejected("Circuit transitioning to Half-Open".into());
            }
        }

        self.stats.rejected_calls += 1;
        CircuitBreakerResult::Rejected("Circuit is OPEN".into())
    }

    /// 处理 Half-Open 状态
    fn handle_half_open<T, F, E>(&mut self, operation: F, fallback: Option<Box<dyn Fn() -> T>>) -> CircuitBreakerResult<T>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        // 限制半开状态的调用数
        if self.success_count >= self.config.half_open_max_calls {
            self.state = CircuitState::Closed;
            self.stats.state_changes += 1;
            self.failure_count = 0;
            return CircuitBreakerResult::Rejected("Circuit closing after successful recovery".into());
        }

        match operation() {
            Ok(result) => {
                self.on_success();
                CircuitBreakerResult::Success(result)
            }
            Err(e) => {
                self.on_failure();
                if let Some(fallback_fn) = fallback {
                    CircuitBreakerResult::Fallback(fallback_fn())
                } else {
                    CircuitBreakerResult::Rejected(e.to_string())
                }
            }
        }
    }

    /// 成功回调
    fn on_success(&mut self) {
        self.stats.successful_calls += 1;
        self.success_count += 1;
        self.failure_count = 0;
    }

    /// 失败回调
    fn on_failure(&mut self) {
        self.stats.failed_calls += 1;
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.config.failure_threshold {
            self.state = CircuitState::Open;
            self.stats.state_changes += 1;
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> &CircuitState {
        &self.state
    }

    /// 获取统计信息
    pub fn stats(&self) -> &CircuitBreakerStats {
        &self.stats
    }

    /// 手动重置
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure_time = None;
        self.stats.state_changes += 1;
    }
}

/// 熔断器管理器 — 多实例管理
pub(crate) struct CircuitBreakerManager {
    breakers: std::collections::HashMap<String, CircuitBreaker>,
}

impl CircuitBreakerManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            breakers: std::collections::HashMap::new(),
        }
    }

    /// 获取或创建熔断器
    pub fn get_or_create(&mut self, name: &str, config: CircuitBreakerConfig) -> &mut CircuitBreaker {
        self.breakers
            .entry(name.to_string())
            .or_insert_with(|| CircuitBreaker::new(config))
    }

    /// 获取熔断器
    pub fn get(&mut self, name: &str) -> Option<&mut CircuitBreaker> {
        self.breakers.get_mut(name)
    }

    /// 获取所有熔断器状态
    pub fn get_all_states(&self) -> std::collections::HashMap<String, CircuitState> {
        self.breakers.iter()
            .map(|(name, breaker)| (name.clone(), breaker.state().clone()))
            .collect()
    }
}
