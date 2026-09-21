#![forbid(unsafe_code)]

//! # Circuit Breaker — Provider Failover 模式
//!
//! 基于 Azure APIM 和 oxllm 的 circuit breaker 模式。
//! 实现 CLOSED → OPEN → HALF-OPEN 状态机，支持指数退避和 per-provider 隔离。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Circuit Breaker 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitState {
    /// 正常运行，允许请求通过
    Closed,
    /// 断路状态，拒绝所有请求
    Open,
    /// 半开状态，允许探测请求通过
    HalfOpen,
}

/// Circuit Breaker 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 触发断路的连续失败次数
    pub failure_threshold: u32,
    /// 断路冷却时间（秒）
    pub cooldown_seconds: u64,
    /// 冷却时间倍增因子（重复断路时）
    pub cooldown_multiplier: f64,
    /// 最大冷却时间上限（秒）
    pub max_cooldown_seconds: u64,
    /// 半开状态下的探测超时（秒）
    pub probe_timeout_seconds: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            cooldown_seconds: 300, // 5 minutes
            cooldown_multiplier: 2.0,
            max_cooldown_seconds: 1800, // 30 minutes
            probe_timeout_seconds: 30,
        }
    }
}

/// 单个 Provider 的 Circuit Breaker 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCircuitBreaker {
    pub provider_id: String,
    pub state: CircuitState,
    pub consecutive_failures: u32,
    pub last_failure: Option<String>,
    pub cooldown_until: Option<String>,
    pub cooldown_minutes: u64,
}

/// Circuit Breaker 管理器
pub struct CircuitBreakerManager {
    config: CircuitBreakerConfig,
    breakers: HashMap<String, ProviderCircuitBreaker>,
}

impl CircuitBreakerManager {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            breakers: HashMap::new(),
        }
    }

    /// 获取或创建 provider 的 circuit breaker
    fn get_or_create(&mut self, provider_id: &str) -> &mut ProviderCircuitBreaker {
        self.breakers
            .entry(provider_id.to_string())
            .or_insert_with(|| ProviderCircuitBreaker {
                provider_id: provider_id.to_string(),
                state: CircuitState::Closed,
                consecutive_failures: 0,
                last_failure: None,
                cooldown_until: None,
                cooldown_minutes: self.config.cooldown_seconds / 60,
            })
    }

    /// 检查 provider 是否允许请求
    pub fn can_request(&self, provider_id: &str) -> bool {
        match self.breakers.get(provider_id) {
            None => true, // 无记录 = 允许
            Some(breaker) => match breaker.state {
                CircuitState::Closed => true,
                CircuitState::Open => {
                    // 检查冷却是否已过
                    if let Some(cooldown_str) = &breaker.cooldown_until {
                        if let Ok(cooldown_end) = cooldown_str.parse::<u64>() {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            return now >= cooldown_end;
                        }
                    }
                    false
                }
                CircuitState::HalfOpen => true, // 允许一个探测请求
            },
        }
    }

    /// 记录请求成功
    pub fn record_success(&mut self, provider_id: &str) {
        let breaker = self.get_or_create(provider_id);
        breaker.consecutive_failures = 0;
        breaker.state = CircuitState::Closed;
        breaker.last_failure = None;
        breaker.cooldown_until = None;
    }

    /// 记录请求失败
    pub fn record_failure(&mut self, provider_id: &str) {
        let failure_threshold = self.config.failure_threshold;
        let cooldown_multiplier = self.config.cooldown_multiplier;
        let max_cooldown_seconds = self.config.max_cooldown_seconds;

        let breaker = self.get_or_create(provider_id);
        breaker.consecutive_failures += 1;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        breaker.last_failure = Some(now.to_string());

        if breaker.consecutive_failures >= failure_threshold {
            // 触发断路
            let cooldown =
                (breaker.cooldown_minutes as f64 * cooldown_multiplier) as u64;
            let cooldown = cooldown.min(max_cooldown_seconds);

            breaker.state = CircuitState::Open;
            breaker.cooldown_minutes = cooldown / 60;
            breaker.cooldown_until = Some((now + cooldown).to_string());
        } else if breaker.state == CircuitState::HalfOpen {
            // 半开状态下失败，重新断路
            breaker.state = CircuitState::Open;
            let cooldown = breaker.cooldown_minutes * 2;
            let cooldown = cooldown.min(max_cooldown_seconds / 60);
            breaker.cooldown_minutes = cooldown;
            breaker.cooldown_until = Some((now + cooldown * 60).to_string());
        }
    }

    /// 尝试从 Open 转换到 HalfOpen（探测）
    pub fn try_half_open(&mut self, provider_id: &str) -> bool {
        let breaker = self.get_or_create(provider_id);
        if breaker.state == CircuitState::Open {
            if let Some(cooldown_str) = &breaker.cooldown_until {
                if let Ok(cooldown_end) = cooldown_str.parse::<u64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now >= cooldown_end {
                        breaker.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 获取所有 breaker 状态
    pub fn snapshot(&self) -> Vec<ProviderCircuitBreaker> {
        self.breakers.values().cloned().collect()
    }

    /// 获取指定 provider 状态
    pub fn status(&self, provider_id: &str) -> Option<&ProviderCircuitBreaker> {
        self.breakers.get(provider_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let manager = CircuitBreakerManager::new(CircuitBreakerConfig::default());
        assert!(manager.can_request("openai"));
    }

    #[test]
    fn test_record_success() {
        let mut manager = CircuitBreakerManager::new(CircuitBreakerConfig::default());
        manager.record_failure("openai");
        manager.record_success("openai");
        assert!(manager.can_request("openai"));
    }

    #[test]
    fn test_consecutive_failures() {
        let mut manager = CircuitBreakerManager::new(CircuitBreakerConfig {
            failure_threshold: 2,
            cooldown_seconds: 60,
            ..Default::default()
        });
        manager.record_failure("openai");
        assert!(manager.can_request("openai"));
        manager.record_failure("openai");
        assert!(!manager.can_request("openai"));
    }
}
