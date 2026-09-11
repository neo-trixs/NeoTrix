use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// 自动恢复配置
#[derive(Debug, Clone)]
pub struct AutoRecoveryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
    pub circuit_breaker_threshold: u32,
    pub recovery_check_interval: Duration,
}

impl Default for AutoRecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
            circuit_breaker_threshold: 5,
            recovery_check_interval: Duration::from_secs(60),
        }
    }
}

/// Provider 健康状态
#[derive(Debug, Clone, PartialEq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Recovering,
    CircuitOpen,
}

/// Provider 恢复跟踪器
#[derive(Debug, Clone)]
struct RecoveryTracker {
    consecutive_failures: u32,
    last_failure: Option<Instant>,
    state: HealthState,
    recovery_attempts: u32,
}

/// 自动恢复管理器 — 监控 provider 健康并自动恢复
pub struct AutoRecovery {
    config: AutoRecoveryConfig,
    trackers: RwLock<HashMap<String, RecoveryTracker>>,
    #[allow(dead_code)]
    last_check: RwLock<Instant>,
}

impl AutoRecovery {
    pub fn new(config: AutoRecoveryConfig) -> Self {
        Self {
            config,
            trackers: RwLock::new(HashMap::new()),
            last_check: RwLock::new(Instant::now()),
        }
    }

    pub fn record_failure(&self, provider: &str) {
        let mut trackers = self.trackers.write().unwrap();
        let tracker = trackers.entry(provider.to_string()).or_insert_with(|| RecoveryTracker {
            consecutive_failures: 0,
            last_failure: None,
            state: HealthState::Healthy,
            recovery_attempts: 0,
        });

        tracker.consecutive_failures += 1;
        tracker.last_failure = Some(Instant::now());

        if tracker.consecutive_failures >= self.config.circuit_breaker_threshold {
            tracker.state = HealthState::CircuitOpen;
        } else if tracker.consecutive_failures >= 2 {
            tracker.state = HealthState::Degraded;
        }
    }

    pub fn record_success(&self, provider: &str) {
        let mut trackers = self.trackers.write().unwrap();
        if let Some(tracker) = trackers.get_mut(provider) {
            tracker.consecutive_failures = 0;
            tracker.recovery_attempts = 0;
            tracker.state = HealthState::Healthy;
        }
    }

    pub fn should_skip(&self, provider: &str) -> bool {
        let trackers = self.trackers.read().unwrap();
        match trackers.get(provider) {
            Some(t) => t.state == HealthState::CircuitOpen,
            None => false,
        }
    }

    pub fn get_state(&self, provider: &str) -> HealthState {
        let trackers = self.trackers.read().unwrap();
        trackers
            .get(provider)
            .map(|t| t.state.clone())
            .unwrap_or(HealthState::Healthy)
    }

    pub fn calculate_backoff(&self, provider: &str) -> Duration {
        let trackers = self.trackers.read().unwrap();
        match trackers.get(provider) {
            Some(t) => {
                let delay = self.config.base_delay.as_millis() as f64
                    * self.config.backoff_factor.powi(t.recovery_attempts as i32);
                let capped = delay.min(self.config.max_delay.as_millis() as f64);
                Duration::from_millis(capped as u64)
            }
            None => self.config.base_delay,
        }
    }

    pub fn try_recover(&self, provider: &str) -> bool {
        let mut trackers = self.trackers.write().unwrap();
        if let Some(tracker) = trackers.get_mut(provider) {
            if tracker.state == HealthState::CircuitOpen {
                tracker.state = HealthState::Recovering;
                tracker.recovery_attempts += 1;
                return true;
            }
        }
        false
    }

    pub fn get_all_states(&self) -> HashMap<String, HealthState> {
        let trackers = self.trackers.read().unwrap();
        trackers
            .iter()
            .map(|(k, v)| (k.clone(), v.state.clone()))
            .collect()
    }
}

impl Default for AutoRecovery {
    fn default() -> Self {
        Self::new(AutoRecoveryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker() {
        let recovery = AutoRecovery::new(AutoRecoveryConfig {
            circuit_breaker_threshold: 3,
            ..Default::default()
        });

        assert_eq!(recovery.get_state("p1"), HealthState::Healthy);
        recovery.record_failure("p1");
        assert_eq!(recovery.get_state("p1"), HealthState::Degraded);
        recovery.record_failure("p1");
        assert_eq!(recovery.get_state("p1"), HealthState::Degraded);
        recovery.record_failure("p1");
        assert_eq!(recovery.get_state("p1"), HealthState::CircuitOpen);
        assert!(recovery.should_skip("p1"));
    }

    #[test]
    fn test_recovery_success() {
        let recovery = AutoRecovery::default();
        recovery.record_failure("p1");
        recovery.record_failure("p1");
        assert!(recovery.try_recover("p1"));
        recovery.record_success("p1");
        assert_eq!(recovery.get_state("p1"), HealthState::Healthy);
    }

    #[test]
    fn test_backoff() {
        let recovery = AutoRecovery::default();
        recovery.record_failure("p1");
        let b1 = recovery.calculate_backoff("p1");
        recovery.record_failure("p1");
        recovery.try_recover("p1");
        let b2 = recovery.calculate_backoff("p1");
        assert!(b2 > b1);
    }
}
