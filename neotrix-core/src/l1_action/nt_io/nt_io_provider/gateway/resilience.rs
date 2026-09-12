use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════════
// Circuit Breaker — 防止级联故障
// ═══════════════════════════════════════════════════════════════════

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

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(60))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Anomaly Detector — 基于统计方法检测 provider 异常
// ═══════════════════════════════════════════════════════════════════

/// 异常检测器 — 基于统计方法检测 provider 异常
pub struct AnomalyDetector {
    windows: RwLock<HashMap<String, SlidingWindow>>,
    config: AnomalyConfig,
    alerts: RwLock<Vec<AnomalyAlert>>,
}

#[derive(Debug, Clone)]
pub(crate) struct AnomalyConfig {
    pub window_size: usize,
    pub z_score_threshold: f64,
    pub min_samples: usize,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            window_size: 100,
            z_score_threshold: 2.5,
            min_samples: 10,
        }
    }
}

#[derive(Debug, Clone)]
struct SlidingWindow {
    values: VecDeque<f64>,
    sum: f64,
    sum_sq: f64,
}

impl SlidingWindow {
    fn new(capacity: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(capacity),
            sum: 0.0,
            sum_sq: 0.0,
        }
    }

    fn push(&mut self, value: f64) {
        if self.values.len() == self.values.capacity() {
            if let Some(old) = self.values.pop_front() {
                self.sum -= old;
                self.sum_sq -= old * old;
            }
        }
        self.values.push_back(value);
        self.sum += value;
        self.sum_sq += value * value;
    }

    fn mean(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.sum / self.values.len() as f64
        }
    }

    fn std_dev(&self) -> f64 {
        if self.values.len() < 2 {
            return 0.0;
        }
        let n = self.values.len() as f64;
        let variance = (self.sum_sq - (self.sum * self.sum) / n) / (n - 1.0);
        variance.sqrt()
    }

    fn z_score(&self, value: f64) -> f64 {
        let mean = self.mean();
        let std = self.std_dev();
        if std == 0.0 {
            0.0
        } else {
            (value - mean) / std
        }
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AnomalyAlert {
    pub provider: String,
    pub metric: String,
    pub value: f64,
    pub z_score: f64,
    pub threshold: f64,
    pub timestamp: Instant,
}

impl AnomalyDetector {
    pub fn new(config: AnomalyConfig) -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
            config,
            alerts: RwLock::new(Vec::new()),
        }
    }

    pub fn record_latency(&self, provider: &str, latency_ms: f64) -> Option<AnomalyAlert> {
        self.record_metric(provider, "latency", latency_ms)
    }

    pub(crate) fn _record_error_rate(&self, provider: &str, rate: f64) -> Option<AnomalyAlert> {
        self.record_metric(provider, "error_rate", rate)
    }

    pub fn record_metric(
        &self,
        provider: &str,
        metric: &str,
        value: f64,
    ) -> Option<AnomalyAlert> {
        let key = format!("{}:{}", provider, metric);
        let mut windows = self.windows.write().unwrap();
        let window = windows
            .entry(key)
            .or_insert_with(|| SlidingWindow::new(self.config.window_size));

        let z = window.z_score(value);
        window.push(value);

        if window.len() >= self.config.min_samples && z.abs() > self.config.z_score_threshold {
            let alert = AnomalyAlert {
                provider: provider.to_string(),
                metric: metric.to_string(),
                value,
                z_score: z,
                threshold: self.config.z_score_threshold,
                timestamp: Instant::now(),
            };
            self.alerts.write().unwrap().push(alert.clone());
            return Some(alert);
        }
        None
    }

    pub fn is_anomalous(&self, provider: &str, metric: &str, value: f64) -> bool {
        let key = format!("{}:{}", provider, metric);
        let windows = self.windows.read().unwrap();
        match windows.get(&key) {
            Some(w) if w.len() >= self.config.min_samples => {
                w.z_score(value).abs() > self.config.z_score_threshold
            }
            _ => false,
        }
    }

    pub(crate) fn _get_alerts(&self, provider: Option<&str>) -> Vec<AnomalyAlert> {
        let alerts = self.alerts.read().unwrap();
        match provider {
            Some(p) => alerts.iter().filter(|a| a.provider == p).cloned().collect(),
            None => alerts.clone(),
        }
    }

    pub(crate) fn _clear_alerts(&self) {
        self.alerts.write().unwrap().clear();
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new(AnomalyConfig::default())
    }
}

// ═══════════════════════════════════════════════════════════════════
// Auto Recovery — 监控 provider 健康并自动恢复
// ═══════════════════════════════════════════════════════════════════

/// 自动恢复配置
#[derive(Debug, Clone)]
pub(crate) struct AutoRecoveryConfig {
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
pub(crate) enum HealthState {
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

    #[test]
    fn test_normal_values_no_alert() {
        let detector = AnomalyDetector::default();
        for i in 0..20 {
            assert!(detector.record_latency("p1", 100.0 + i as f64).is_none());
        }
    }

    #[test]
    fn test_anomaly_detected() {
        let detector = AnomalyDetector::default();
        for _ in 0..20 {
            detector.record_latency("p1", 100.0);
        }
        let alert = detector.record_latency("p1", 500.0);
        assert!(alert.is_some());
        let a = alert.unwrap();
        assert!(a.z_score > 2.0);
    }

    #[test]
    fn test_sliding_window() {
        let mut w = SlidingWindow::new(5);
        for i in 0..10 {
            w.push(i as f64);
        }
        assert_eq!(w.len(), 5);
        assert_eq!(w.mean(), 7.0);
    }

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
