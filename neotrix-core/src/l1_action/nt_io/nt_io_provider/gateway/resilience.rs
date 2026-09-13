//! 韧性 — 漂移检测、健康检查、弹性恢复、响应缓存、响应修复

#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::core::nt_core_llm::{LlmRequest, LlmResponse};
use crate::l1_action::nt_io::nt_io_provider::common::generation_classifier::{GenerationRecord, LlmPurpose, TaskType};
use crate::core::nt_core_llm::Message;
use super::types::ProviderState;
use super::GatewayV2;

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
pub struct AnomalyConfig {
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
pub struct AnomalyAlert {
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

    /// Record a metric value and check for anomaly (z-score based).
    ///
    /// Note: Real implementation needs — the z-score threshold is static (2.5). Consider:
    /// - Adaptive thresholds based on provider volatility
    /// - Separate thresholds per metric type (latency vs error rate)
    /// - Exponential decay weighting for recent observations
    /// - Alert deduplication (don't re-alert for same持续 anomaly)
    pub fn record_metric(
        &self,
        provider: &str,
        metric: &str,
        value: f64,
    ) -> Option<AnomalyAlert> {
        let key = format!("{}:{}", provider, metric);
        let mut windows = self.windows.write().unwrap_or_else(|e| e.into_inner());
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
            self.alerts.write().unwrap_or_else(|e| e.into_inner()).push(alert.clone());
            return Some(alert);
        }
        None
    }

    pub fn is_anomalous(&self, provider: &str, metric: &str, value: f64) -> bool {
        let key = format!("{}:{}", provider, metric);
        let windows = self.windows.read().unwrap_or_else(|e| e.into_inner());
        match windows.get(&key) {
            Some(w) if w.len() >= self.config.min_samples => {
                w.z_score(value).abs() > self.config.z_score_threshold
            }
            _ => false,
        }
    }

    pub(crate) fn _get_alerts(&self, provider: Option<&str>) -> Vec<AnomalyAlert> {
        let alerts = self.alerts.read().unwrap_or_else(|e| e.into_inner());
        match provider {
            Some(p) => alerts.iter().filter(|a| a.provider == p).cloned().collect(),
            None => alerts.clone(),
        }
    }

    pub(crate) fn _clear_alerts(&self) {
        self.alerts.write().unwrap_or_else(|e| e.into_inner()).clear();
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

    pub fn record_success(&self, provider: &str) {
        let mut trackers = self.trackers.write().unwrap_or_else(|e| e.into_inner());
        if let Some(tracker) = trackers.get_mut(provider) {
            tracker.consecutive_failures = 0;
            tracker.recovery_attempts = 0;
            tracker.state = HealthState::Healthy;
        }
    }

    pub fn should_skip(&self, provider: &str) -> bool {
        let trackers = self.trackers.read().unwrap_or_else(|e| e.into_inner());
        match trackers.get(provider) {
            Some(t) => t.state == HealthState::CircuitOpen,
            None => false,
        }
    }

    pub fn get_state(&self, provider: &str) -> HealthState {
        let trackers = self.trackers.read().unwrap_or_else(|e| e.into_inner());
        trackers
            .get(provider)
            .map(|t| t.state.clone())
            .unwrap_or(HealthState::Healthy)
    }

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

// ═══════════════════════════════════════════════════════════════════
// Drift Detector — 基于滚动窗口监控 provider 质量漂移
// ═══════════════════════════════════════════════════════════════════

/// Provider 指标快照
pub struct ProviderMetric {
    pub provider_id: String,
    pub latency_ms: u64,
    pub cost_per_token: f64,
    pub success_rate: f64,
    pub quality_score: f64,
    pub timestamp: i64,
}

/// 漂移检测结果
#[derive(Debug, Clone, PartialEq)]
pub enum DriftStatus {
    Normal,
    MildDrift {
        metric: String,
        deviation: f64,
    },
    SevereDrift {
        metric: String,
        deviation: f64,
        suggested_alternative: String,
    },
}

pub struct DriftDetector {
    windows: VecDeque<ProviderMetric>,
    window_size: usize,
    drift_threshold: f64,
    severe_threshold: f64,
}

impl DriftDetector {
    pub fn new(window_size: usize, drift_threshold: f64, severe_threshold: f64) -> Self {
        Self {
            windows: VecDeque::with_capacity(window_size * 10),
            window_size,
            drift_threshold,
            severe_threshold,
        }
    }

    pub fn record(&mut self, metric: ProviderMetric) {
        self.windows.push_back(metric);
        while self.windows.len() > self.window_size * 10 {
            self.windows.pop_front();
        }
    }

    /// Detect quality drift for a provider using z-score analysis on recent metrics.
    ///
    /// Note: Real implementation needs — the drift detection uses simple z-score on
    /// the latest observation vs rolling mean. Consider:
    /// - CUSUM (cumulative sum) control charts for trend detection
    /// - Seasonal decomposition (latency patterns vary by time of day)
    /// - Multivariate drift detection (correlated latency + error rate changes)
    /// - Integration with circuit breaker for automatic failover on severe drift
    pub fn detect(&self, provider_id: &str) -> DriftStatus {
        let metrics: Vec<&ProviderMetric> = self
            .windows
            .iter()
            .filter(|m| m.provider_id == provider_id)
            .collect();

        if metrics.len() < 3 {
            return DriftStatus::Normal;
        }

        let start = metrics.len().saturating_sub(self.window_size);
        let recent = &metrics[start..];

        let avg_latency =
            recent.iter().map(|m| m.latency_ms as f64).sum::<f64>() / recent.len() as f64;
        let var_latency = recent
            .iter()
            .map(|m| (m.latency_ms as f64 - avg_latency).powi(2))
            .sum::<f64>()
            / recent.len() as f64;
        let std_latency = var_latency.sqrt();

        let avg_quality = recent.iter().map(|m| m.quality_score).sum::<f64>() / recent.len() as f64;
        let var_quality = recent
            .iter()
            .map(|m| (m.quality_score - avg_quality).powi(2))
            .sum::<f64>()
            / recent.len() as f64;
        let std_quality = var_quality.sqrt();

        let latest = match recent.last() {
            Some(m) => m,
            None => return DriftStatus::Normal,
        };

        let latency_z = if std_latency > 0.0 {
            (latest.latency_ms as f64 - avg_latency) / std_latency
        } else {
            0.0
        };

        let quality_z = if std_quality > 0.0 {
            (avg_quality - latest.quality_score) / std_quality
        } else {
            0.0
        };

        let (worst_metric, worst_z) = if latency_z >= quality_z {
            ("latency".to_string(), latency_z)
        } else {
            ("quality".to_string(), quality_z)
        };

        if worst_z > self.severe_threshold {
            DriftStatus::SevereDrift {
                metric: worst_metric,
                deviation: worst_z,
                suggested_alternative: "backup_provider".to_string(),
            }
        } else if worst_z > self.drift_threshold {
            DriftStatus::MildDrift {
                metric: worst_metric,
                deviation: worst_z,
            }
        } else {
            DriftStatus::Normal
        }
    }

    pub fn health_summary(&self) -> Vec<(String, f64, f64)> {
        let mut summaries: Vec<(String, f64, f64)> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for metric in &self.windows {
            if seen.insert(metric.provider_id.clone()) {
                let metrics: Vec<&ProviderMetric> = self
                    .windows
                    .iter()
                    .filter(|m| m.provider_id == metric.provider_id)
                    .collect();
                let avg_quality =
                    metrics.iter().map(|m| m.quality_score).sum::<f64>() / metrics.len() as f64;
                let avg_latency =
                    metrics.iter().map(|m| m.latency_ms as f64).sum::<f64>() / metrics.len() as f64;
                summaries.push((metric.provider_id.clone(), avg_quality, avg_latency));
            }
        }

        summaries
    }

    pub fn prune(&mut self, max_entries: usize) {
        while self.windows.len() > max_entries {
            self.windows.pop_front();
        }
    }

    pub fn provider_count(&self, provider_id: &str) -> usize {
        self.windows
            .iter()
            .filter(|m| m.provider_id == provider_id)
            .count()
    }

    pub fn total_records(&self) -> usize {
        self.windows.len()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Response Cache — LRU 响应缓存
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct ResponseCache {
    entries: HashMap<u64, (String, u64)>,
    capacity: usize,
    tick: u64,
    hit_count: u64,
    miss_count: u64,
    pinned: HashSet<u64>,
    prefetch_hits: u64,
}

impl ResponseCache {
    pub const DEFAULT_CAPACITY: usize = 256;
    pub const MAX_PINNED: usize = 32;

    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity: capacity.max(1),
            tick: 0,
            hit_count: 0,
            miss_count: 0,
            pinned: HashSet::new(),
            prefetch_hits: 0,
        }
    }

    pub fn key_for(model_id: &str, messages: &[Message]) -> String {
        let body = messages
            .iter()
            .map(|m| format!("{:?}:{}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}|{}", model_id, body)
    }

    pub fn key_for_request(model_id: &str, fingerprint: &str) -> String {
        format!("{}|fp={}", model_id, fingerprint)
    }

    fn hash_key(key: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    pub fn cache(&mut self, key: &str) -> Option<String> {
        let hash = Self::hash_key(key);
        if let Some((resp, last_used)) = self.entries.get_mut(&hash) {
            self.tick += 1;
            *last_used = self.tick;
            self.hit_count += 1;
            return Some(resp.clone());
        }
        self.miss_count += 1;
        None
    }

    pub fn insert(&mut self, key: &str, response: String) {
        let hash = Self::hash_key(key);
        self.tick += 1;
        if let Some(entry) = self.entries.get_mut(&hash) {
            *entry = (response, self.tick);
            return;
        }
        if self.entries.len() >= self.capacity {
            let lru_key = self
                .entries
                .iter()
                .filter(|(k, _)| !self.pinned.contains(k))
                .min_by_key(|(_, (_, t))| *t)
                .map(|(k, _)| *k);
            if let Some(k) = lru_key {
                self.entries.remove(&k);
            }
        }
        self.entries.insert(hash, (response, self.tick));
    }

    pub fn pin(&mut self, key: &str) -> bool {
        if self.pinned.len() >= Self::MAX_PINNED {
            return false;
        }
        self.pinned.insert(Self::hash_key(key))
    }

    pub fn unpin(&mut self, key: &str) {
        self.pinned.remove(&Self::hash_key(key));
    }

    pub fn pinned_count(&self) -> usize {
        self.pinned.len()
    }

    pub fn prefetch(&mut self, key: &str) -> Option<String> {
        let hash = Self::hash_key(key);
        let exists = self.entries.contains_key(&hash);
        if !exists {
            return None;
        }
        self.tick += 1;
        if let Some((_, t)) = self.entries.get_mut(&hash) {
            *t = self.tick;
        }
        self.prefetch_hits += 1;
        self.entries.get(&hash).map(|(resp, _)| resp.clone())
    }

    pub fn prefetch_hit_count(&self) -> u64 {
        self.prefetch_hits
    }

    pub fn prefetch_lookahead(&mut self, hints: &[String]) -> (usize, Vec<String>) {
        let mut hits = 0;
        let mut missing = Vec::new();
        for hint in hints {
            let hash = Self::hash_key(hint);
            if self.entries.contains_key(&hash) {
                self.tick += 1;
                if let Some((_, t)) = self.entries.get_mut(&hash) {
                    *t = self.tick;
                }
                self.prefetch_hits += 1;
                hits += 1;
            } else {
                missing.push(hint.clone());
            }
        }
        (hits, missing)
    }

    pub fn lookahead_hints(&self, key: &str) -> Vec<String> {
        let mut hints = Vec::new();
        if let Some((model, _)) = key.split_once('|') {
            hints.push(format!("{}|fp=lookahead:1", model));
            hints.push(format!("{}|fp=lookahead:2", model));
        }
        hints
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn hit_count(&self) -> u64 {
        self.hit_count
    }

    pub fn miss_count(&self) -> u64 {
        self.miss_count
    }
}

// ═══════════════════════════════════════════════════════════════════
// Response Quality — 响应质量评估
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ResponseQualityScore {
    pub coherence: f64,
    pub relevance: f64,
    pub completeness: f64,
}

impl ResponseQualityScore {
    pub fn new(coherence: f64, relevance: f64, completeness: f64) -> Self {
        Self { coherence, relevance, completeness }
    }

    pub fn composite(&self) -> f64 {
        (self.coherence + self.relevance + self.completeness) / 3.0
    }
}

pub fn evaluate_response_quality(content: &str) -> ResponseQualityScore {
    let coherence = if content.is_empty() {
        0.0
    } else {
        let len = content.len() as f64;
        (1.0 - (len / 10000.0).min(1.0)) * 0.8
    };
    let relevance = 0.7;
    let completeness = if content.contains('.') || content.contains('。') {
        0.9
    } else {
        0.5
    };
    ResponseQualityScore::new(coherence, relevance, completeness)
}

// ═══════════════════════════════════════════════════════════════════
// Response Healer — 畸形 JSON 修复
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Default)]
pub struct ResponseHealer {
    heal_count: u64,
    unrepairable_count: u64,
}

impl ResponseHealer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Heal malformed JSON response (extract, trim trailing commas, close unclosed brackets).
    ///
    /// Note: Real implementation needs — the healing pipeline is extract → trim → close.
    /// Consider adding: Unicode normalization, escape sequence repair, and schema
    /// validation after healing. Also track which providers commonly return malformed
    /// JSON for targeted improvement.
    pub fn heal(&mut self, raw: &str) -> String {
        if serde_json::from_str::<serde_json::Value>(raw).is_ok() {
            return raw.to_string();
        }
        let extracted = self.extract_json(raw);
        let trimmed = self.trim_trailing_commas(&extracted);
        let closed = self.close_unclosed(&trimmed);
        if serde_json::from_str::<serde_json::Value>(&closed).is_ok() {
            self.heal_count += 1;
            return closed;
        }
        self.unrepairable_count += 1;
        raw.to_string()
    }

    fn extract_json(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if let Some(fence) = trimmed.find("```json") {
            let after = &trimmed[fence + "```json".len()..];
            let content = match after.find("```") {
                Some(end) => &after[..end],
                None => after,
            };
            let c = content.trim();
            return if c.is_empty() {
                raw.to_string()
            } else {
                c.to_string()
            };
        }
        let chars: Vec<char> = trimmed.chars().collect();
        let mut start = None;
        for (i, c) in chars.iter().enumerate() {
            if *c == '{' || *c == '[' {
                start = Some(i);
                break;
            }
        }
        let start = match start {
            Some(s) => s,
            None => return raw.to_string(),
        };
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escaped = false;
        let mut end = chars.len();
        for (i, c) in chars.iter().enumerate().skip(start) {
            if in_string {
                if escaped {
                    escaped = false;
                } else if *c == '\\' {
                    escaped = true;
                } else if *c == '"' {
                    in_string = false;
                }
                continue;
            }
            match *c {
                '"' => in_string = true,
                '{' | '[' => depth += 1,
                '}' | ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        chars[start..end].iter().collect::<String>().trim().to_string()
    }

    fn trim_trailing_commas(&self, s: &str) -> String {
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len();
        let mut out = String::with_capacity(s.len());
        let mut i = 0;
        let mut in_string = false;
        let mut escaped = false;
        while i < n {
            let c = chars[i];
            if in_string {
                out.push(c);
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                i += 1;
                continue;
            }
            if c == '"' {
                in_string = true;
                out.push('"');
                i += 1;
                continue;
            }
            if c == ',' {
                let mut j = i + 1;
                while j < n && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < n && (chars[j] == '}' || chars[j] == ']') {
                    i += 1;
                    continue;
                }
            }
            out.push(c);
            i += 1;
        }
        out
    }

    fn close_unclosed(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 4);
        let mut stack: Vec<char> = Vec::new();
        let mut in_string = false;
        let mut escaped = false;
        for c in s.chars() {
            if in_string {
                out.push(c);
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                continue;
            }
            match c {
                '"' => {
                    in_string = true;
                    out.push('"');
                }
                '{' => {
                    stack.push('{');
                    out.push('{');
                }
                '[' => {
                    stack.push('[');
                    out.push('[');
                }
                '}' => {
                    if stack.last() == Some(&'{') {
                        stack.pop();
                        out.push('}');
                    } else if stack.last() == Some(&'[') {
                        stack.pop();
                        out.push(']');
                        stack.pop();
                        out.push('}');
                    }
                }
                ']' => {
                    if stack.last() == Some(&'[') {
                        stack.pop();
                        out.push(']');
                    } else if stack.last() == Some(&'{') {
                        stack.pop();
                        out.push('}');
                        stack.pop();
                        out.push(']');
                    }
                }
                _ => out.push(c),
            }
        }
        while let Some(open) = stack.pop() {
            out.push(match open {
                '{' => '}',
                _ => ']',
            });
        }
        out
    }

    pub fn heal_count(&self) -> u64 {
        self.heal_count
    }

    pub fn unrepairable_count(&self) -> u64 {
        self.unrepairable_count
    }
}

// ═══════════════════════════════════════════════════════════════════
// Pool Health — LLM 池健康探测器
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct PoolHealthReport {
    pub total: usize,
    pub free: usize,
    pub paid: usize,
    pub local: usize,
    pub model_locked: usize,
    pub sufficient: bool,
    pub min_free: usize,
}

pub struct LlmPoolHealth;

impl LlmPoolHealth {
    pub fn evaluate(gw: &GatewayV2, min_free: usize) -> PoolHealthReport {
        let providers = gw.providers();
        let total = providers.len();
        let free = gw.available_free_providers().len();
        let model_locked = gw.model_locked_count();
        let sufficient = gw.is_pool_sufficient(min_free);
        let local = providers
            .iter()
            .filter(|n| n.contains("ollama") || n.contains("vllm") || n.contains("llama"))
            .count();
        PoolHealthReport {
            total,
            free,
            paid: total.saturating_sub(free),
            local,
            model_locked,
            sufficient,
            min_free,
        }
    }

    pub fn summarize(gw: &GatewayV2, min_free: usize) -> String {
        let r = Self::evaluate(gw, min_free);
        format!(
            "LLM pool health: total={} free={} paid={} local={} model_locked={} sufficient={} (min_free={})",
            r.total, r.free, r.paid, r.local, r.model_locked, r.sufficient, r.min_free
        )
    }
}

// ═══════════════════════════════════════════════════════════════════
// GatewayV2 Resilience Extensions — 韧性接线
// ═══════════════════════════════════════════════════════════════════

impl GatewayV2 {
    pub fn enable_response_cache(&mut self, enabled: bool) {
        self.response_cache_enabled = enabled;
    }

    pub fn response_cache_enabled(&self) -> bool {
        self.response_cache_enabled
    }

    pub fn response_cache_hits(&self) -> u64 {
        self.response_cache.lock().map(|c| c.hit_count()).unwrap_or(0)
    }

    pub fn response_cache_len(&self) -> usize {
        self.response_cache.lock().map(|c| c.len()).unwrap_or(0)
    }

    pub fn response_cache_prefetches(&self) -> u64 {
        self.response_cache_prefetches
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_response_healer(&mut self, enabled: bool) {
        self.response_healer_enabled = enabled;
    }

    pub fn response_healer_enabled(&self) -> bool {
        self.response_healer_enabled
    }

    pub fn response_healer_counters(&self) -> (u64, u64) {
        match self.response_healer.lock() {
            Ok(h) => (h.heal_count(), h.unrepairable_count()),
            Err(_) => (0, 0),
        }
    }

    pub(crate) fn _set_generation_classification(&mut self, enabled: bool) {
        self.generation_classification_enabled = enabled;
    }

    pub fn generation_classification_enabled(&self) -> bool {
        self.generation_classification_enabled
    }

    pub(crate) fn tag_generation(
        &self,
        request: &LlmRequest,
        response: &LlmResponse,
        provider_name: &str,
        latency_ms: f64,
        tokens: u32,
        success: bool,
    ) {
        if !self.generation_classification_enabled {
            return;
        }
        let prompt = request
            .messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n");
        let classification = match self.generation_classifier.lock() {
            Ok(c) => c.classify(&prompt, &response.content),
            Err(e) => {
                log::warn!("[gateway] generation_classifier poisoned: {}", e);
                return;
            }
        };
        let purpose = match classification.task_type {
            TaskType::ToolUse => LlmPurpose::ToolUse,
            TaskType::Summarization => LlmPurpose::Summarization,
            TaskType::Extraction => LlmPurpose::ToolUse,
            _ => LlmPurpose::AgentTurn,
        };
        let record = GenerationRecord {
            model: format!("{}/{}", provider_name, request.model),
            classification,
            prompt_len: prompt.len(),
            response_len: response.content.len(),
            latency_ms: latency_ms as u64,
            tokens,
            success,
            purpose,
        };
        if let Ok(mut analytics) = self.generation_analytics.lock() {
            analytics.record(&record);
        }
    }

    pub(crate) fn _generation_analytics_snapshot(&self) -> (u64, HashMap<String, u64>, HashMap<String, u64>, HashMap<String, u64>) {
        match self.generation_analytics.lock() {
            Ok(a) => (
                a.total,
                a.distribution("task_type"),
                a.distribution("complexity"),
                a.distribution("domain"),
            ),
            Err(_) => (0, HashMap::new(), HashMap::new(), HashMap::new()),
        }
    }

    pub fn maybe_re_evaluate(&self) -> bool {
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });
        let refs: Vec<&ProviderState> = states.values().collect();
        match self.market_router.lock() {
            Ok(mut router) => router.re_evaluate(&refs),
            Err(e) => {
                log::warn!("[gateway] market_router Mutex poisoned: {}", e);
                false
            }
        }
    }

    pub(crate) fn heal_and_cache_response(&self, request: &LlmRequest, response: LlmResponse) -> LlmResponse {
        let mut response = response;
        if self.response_healer_enabled {
            if let Ok(mut healer) = self.response_healer.lock() {
                if response.content.contains('{') || response.content.contains('[') {
                    response.content = healer.heal(&response.content);
                }
            }
        }
        if self.response_cache_enabled {
            let rc_key = ResponseCache::key_for_request(&request.model, &self.prompt_cache_key(request));
            if let Ok(mut rc) = self.response_cache.lock() {
                match serde_json::to_string(&response) {
                    Ok(serialized) => rc.insert(&rc_key, serialized),
                    Err(_) => rc.insert(&rc_key, response.content.clone()),
                }
                let hints = rc.lookahead_hints(&rc_key);
                if !hints.is_empty() {
                    let (_, _) = rc.prefetch_lookahead(&hints);
                    self.response_cache_prefetches.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
        response
    }

    pub(crate) fn prompt_text(&self, request: &LlmRequest) -> String {
        request
            .messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(crate) fn prompt_cache_key(&self, request: &LlmRequest) -> String {
        let content = self.prompt_text(request);
        let mut tools: Vec<&str> = request
            .tools
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>();
        tools.sort_unstable();
        let tools_fp = tools.join(",");
        let structured_fp = match &request.structured_output {
            Some(s) => serde_json::to_string(s).unwrap_or_default(),
            None => String::new(),
        };
        format!(
            "{}|max={}|think={:?}|tools=[{}]|struct={}|prefix={:?}",
            content,
            request.max_tokens,
            request.thinking_budget,
            tools_fp,
            structured_fp,
            request.cacheable_prefix_tokens
        )
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

    #[test]
    fn test_drift_normal_when_insufficient_samples() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        det.record(ProviderMetric {
            provider_id: "p1".into(),
            latency_ms: 100,
            cost_per_token: 0.001,
            success_rate: 1.0,
            quality_score: 0.9,
            timestamp: 1000,
        });
        det.record(ProviderMetric {
            provider_id: "p1".into(),
            latency_ms: 110,
            cost_per_token: 0.001,
            success_rate: 1.0,
            quality_score: 0.88,
            timestamp: 1001,
        });
        assert_eq!(det.detect("p1"), DriftStatus::Normal);
    }

    #[test]
    fn test_drift_no_drift_on_stable() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        for _ in 0..10 {
            det.record(ProviderMetric {
                provider_id: "p1".into(),
                latency_ms: 100,
                cost_per_token: 0.001,
                success_rate: 1.0,
                quality_score: 0.9,
                timestamp: 1000,
            });
        }
        assert_eq!(det.detect("p1"), DriftStatus::Normal);
    }

    #[test]
    fn test_drift_severe_latency() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        for _ in 0..9 {
            det.record(ProviderMetric {
                provider_id: "p1".into(),
                latency_ms: 100,
                cost_per_token: 0.001,
                success_rate: 1.0,
                quality_score: 0.9,
                timestamp: 1000,
            });
        }
        det.record(ProviderMetric {
            provider_id: "p1".into(),
            latency_ms: 500,
            cost_per_token: 0.001,
            success_rate: 1.0,
            quality_score: 0.9,
            timestamp: 1009,
        });
        let status = det.detect("p1");
        assert!(
            matches!(status, DriftStatus::SevereDrift { ref metric, .. } if metric == "latency"),
            "expected SevereDrift on latency, got {:?}",
            status
        );
    }

    #[test]
    fn test_pool_health_empty() {
        let gw = GatewayV2::new();
        let r = LlmPoolHealth::evaluate(&gw, 3);
        assert_eq!(r.total, 0);
        assert_eq!(r.free, 0);
        assert!(!r.sufficient);
        assert_eq!(r.min_free, 3);
    }

    #[test]
    fn test_pool_health_summary() {
        let gw = GatewayV2::new();
        let s = LlmPoolHealth::summarize(&gw, 3);
        assert!(s.contains("LLM pool health"));
        assert!(s.contains("total=0"));
    }
}
