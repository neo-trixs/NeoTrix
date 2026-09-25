//! 重试与退避 — AutoRecovery 健康状态机 + 指数退避。
//! 注: 源文件无独立 retry/backoff 类型, 重试语义 (record/try_recover/should_skip)
//! 与退避语义 (calculate_backoff) 均内聚于 AutoRecovery, 故合置一模块, 不强拆。
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use super::nt_resilience_types::{AutoRecoveryConfig, HealthState};

// ═══════════════════════════════════════════════════════════════════
// Auto Recovery — 监控 provider 健康并自动恢复
// ═══════════════════════════════════════════════════════════════════
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
    /// Create an AutoRecovery manager with the given configuration.
    ///
    /// Note: Real implementation needs — recovery state is in-memory only. Consider:
    /// persisting recovery state to survive process restarts and enabling cross-instance
    /// recovery coordination.
    pub fn new(config: AutoRecoveryConfig) -> Self {
        Self {
            config,
            trackers: RwLock::new(HashMap::new()),
            last_check: RwLock::new(Instant::now()),
        }
    }

    /// Record a failure for a provider, updating its health state.
    ///
    /// Note: Real implementation needs — state transitions are threshold-based only.
    /// Consider: time-windowed failure counting and error-type-aware state transitions.
    pub fn record_failure(&self, provider: &str) {
        let mut trackers = self.trackers.write().unwrap_or_else(|e| e.into_inner());
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

    /// Record a success for a provider, resetting failure counters.
    ///
    /// Note: Real implementation needs — immediate reset to Healthy state. Consider:
    /// requiring N consecutive successes before transitioning from Recovering to Healthy.
    pub fn record_success(&self, provider: &str) {
        let mut trackers = self.trackers.write().unwrap_or_else(|e| e.into_inner());
        if let Some(tracker) = trackers.get_mut(provider) {
            tracker.consecutive_failures = 0;
            tracker.recovery_attempts = 0;
            tracker.state = HealthState::Healthy;
        }
    }

    /// Check if a provider should be skipped (circuit open).
    ///
    /// Note: Real implementation needs — returns false for unknown providers.
    /// Consider: returning a Result with the provider's health state for richer
    /// skip-reason information.
    pub fn should_skip(&self, provider: &str) -> bool {
        let trackers = self.trackers.read().unwrap_or_else(|e| e.into_inner());
        match trackers.get(provider) {
            Some(t) => t.state == HealthState::CircuitOpen,
            None => false,
        }
    }

    /// Get the current health state of a provider.
    ///
    /// Note: Real implementation needs — returns Healthy for unknown providers.
    /// Consider: adding a provider registration check and returning an explicit
    /// "Unknown" state for unregistered providers.
    pub fn get_state(&self, provider: &str) -> HealthState {
        let trackers = self.trackers.read().unwrap_or_else(|e| e.into_inner());
        trackers
            .get(provider)
            .map(|t| t.state.clone())
            .unwrap_or(HealthState::Healthy)
    }

    /// Calculate exponential backoff delay for a provider's next recovery attempt.
    ///
    /// Note: Real implementation needs — uses base_delay * backoff_factor^attempts.
    /// Consider: adding jitter to prevent synchronized recovery probes, and
    /// capping at provider-specific max delay based on error type.
    pub fn calculate_backoff(&self, provider: &str) -> Duration {
        let trackers = self.trackers.read().unwrap_or_else(|e| e.into_inner());
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

    /// Attempt to transition a provider from CircuitOpen to Recovering state.
    ///
    /// Note: Real implementation needs — returns true if transition occurred.
    /// Consider: adding a cooldown between recovery attempts and limiting
    /// concurrent recovery probes per provider.
    pub fn try_recover(&self, provider: &str) -> bool {
        let mut trackers = self.trackers.write().unwrap_or_else(|e| e.into_inner());
        if let Some(tracker) = trackers.get_mut(provider) {
            if tracker.state == HealthState::CircuitOpen {
                tracker.state = HealthState::Recovering;
                tracker.recovery_attempts += 1;
                return true;
            }
        }
        false
    }

    /// Get health states for all tracked providers.
    ///
    /// Note: Real implementation needed — returns only providers that have been
    /// recorded. Consider: including providers with default Healthy state for
    /// complete visibility.
    pub fn get_all_states(&self) -> HashMap<String, HealthState> {
        let trackers = self.trackers.read().unwrap_or_else(|e| e.into_inner());
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
    fn test_auto_recovery_circuit_breaker() {
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
    fn test_auto_recovery_success() {
        let recovery = AutoRecovery::default();
        recovery.record_failure("p1");
        recovery.record_failure("p1");
        assert!(recovery.try_recover("p1"));
        recovery.record_success("p1");
        assert_eq!(recovery.get_state("p1"), HealthState::Healthy);
    }

    #[test]
    fn test_auto_recovery_backoff() {
        let recovery = AutoRecovery::default();
        recovery.record_failure("p1");
        let b1 = recovery.calculate_backoff("p1");
        recovery.record_failure("p1");
        recovery.try_recover("p1");
        let b2 = recovery.calculate_backoff("p1");
        assert!(b2 > b1);
    }
}
