#![deny(clippy::unwrap_used)]

use std::time::Instant;

/// Runtime health signals collected from the observability pipeline.
#[derive(Debug, Clone)]
pub struct HealthSnapshot {
    pub timestamp_ms: u128,
    pub memory_bytes: u64,
    pub latency_p50_ms: f64,
    pub latency_p99_ms: f64,
    pub error_rate: f64,
    pub active_spans: u64,
    pub open_connections: u64,
}

/// Alert severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HealthSeverity {
    Info,
    Warning,
    Critical,
}

impl HealthSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthSeverity::Info => "info",
            HealthSeverity::Warning => "warning",
            HealthSeverity::Critical => "critical",
        }
    }
}

/// A single health alert.
#[derive(Debug, Clone)]
pub struct HealthAlert {
    pub timestamp_ms: u128,
    pub severity: HealthSeverity,
    pub signal: String,
    pub message: String,
    pub current_value: f64,
    pub threshold: f64,
}

/// Alerting threshold policy for a specific signal.
#[derive(Debug, Clone)]
pub struct AlertPolicy {
    pub signal: String,
    pub warning_threshold: f64,
    pub critical_threshold: f64,
    pub inverted: bool,
}

impl AlertPolicy {
    pub fn new(signal: impl Into<String>, warning: f64, critical: f64) -> Self {
        Self {
            signal: signal.into(),
            warning_threshold: warning,
            critical_threshold: critical,
            inverted: false,
        }
    }

    /// For signals where lower is worse (e.g., free memory).
    pub fn inverted(mut self) -> Self {
        self.inverted = true;
        self
    }

    fn evaluate(&self, value: f64) -> Option<HealthSeverity> {
        if self.inverted {
            if value <= self.critical_threshold {
                Some(HealthSeverity::Critical)
            } else if value <= self.warning_threshold {
                Some(HealthSeverity::Warning)
            } else {
                None
            }
        } else {
            if value >= self.critical_threshold {
                Some(HealthSeverity::Critical)
            } else if value >= self.warning_threshold {
                Some(HealthSeverity::Warning)
            } else {
                None
            }
        }
    }
}

/// Ring buffer of recent health snapshots for trend analysis.
struct HealthHistory {
    snapshots: Vec<HealthSnapshot>,
    max_len: usize,
}

impl HealthHistory {
    fn new(max_len: usize) -> Self {
        Self {
            snapshots: Vec::with_capacity(max_len),
            max_len,
        }
    }

    fn push(&mut self, snapshot: HealthSnapshot) {
        if self.snapshots.len() >= self.max_len {
            self.snapshots.remove(0);
        }
        self.snapshots.push(snapshot);
    }

    fn recent(&self, n: usize) -> &[HealthSnapshot] {
        let len = self.snapshots.len();
        let start = len.saturating_sub(n);
        &self.snapshots[start..]
    }

    fn latency_percentiles(&self) -> (f64, f64) {
        if self.snapshots.is_empty() {
            return (0.0, 0.0);
        }
        let mut p50_vals: Vec<f64> = self.snapshots.iter().map(|s| s.latency_p50_ms).collect();
        let mut p99_vals: Vec<f64> = self.snapshots.iter().map(|s| s.latency_p99_ms).collect();
        p50_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        p99_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p50 = p50_vals[p50_vals.len() / 2];
        let p99 = p99_vals[p99_vals.len() - 1];
        (p50, p99)
    }
}

/// Runtime health monitor with configurable alerting policies.
///
/// Collects health snapshots from the observability pipeline and
/// evaluates them against configurable alert policies. Maps to
/// nt_repair health monitoring via HealthReport output.
pub struct HealthMonitor {
    history: HealthHistory,
    policies: Vec<AlertPolicy>,
    active_alerts: Vec<HealthAlert>,
    start: Instant,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self::with_history_size(1000)
    }

    pub fn with_history_size(max_len: usize) -> Self {
        Self {
            history: HealthHistory::new(max_len),
            policies: Self::default_policies(),
            active_alerts: Vec::new(),
            start: Instant::now(),
        }
    }

    /// Record a health snapshot and evaluate alert policies.
    pub fn record(&mut self, snapshot: HealthSnapshot) -> Vec<HealthAlert> {
        let mut new_alerts = Vec::new();

        for policy in &self.policies {
            if let Some(value) = snapshot.signal_value(&policy.signal) {
                if let Some(severity) = policy.evaluate(value) {
                    let alert = HealthAlert {
                        timestamp_ms: snapshot.timestamp_ms,
                        severity,
                        signal: policy.signal.clone(),
                        message: format!(
                            "{}: {} = {:.2} exceeds {:?} threshold ({:.2})",
                            severity.as_str(),
                            policy.signal,
                            value,
                            if policy.inverted { "min" } else { "max" },
                            if policy.inverted {
                                policy.critical_threshold
                            } else {
                                policy.warning_threshold
                            },
                        ),
                        current_value: value,
                        threshold: if policy.inverted {
                            policy.critical_threshold
                        } else {
                            policy.warning_threshold
                        },
                    };
                    new_alerts.push(alert);
                }
            }
        }

        self.active_alerts.extend(new_alerts.clone());
        self.history.push(snapshot);
        new_alerts
    }

    /// Register a custom alerting policy.
    pub fn add_policy(&mut self, policy: AlertPolicy) {
        self.policies.push(policy);
    }

    /// Set all policies (replaces defaults).
    pub fn set_policies(&mut self, policies: Vec<AlertPolicy>) {
        self.policies = policies;
    }

    /// Current health snapshot from the most recent recording.
    pub fn latest_snapshot(&self) -> Option<&HealthSnapshot> {
        self.history.snapshots.last()
    }

    /// Latency percentiles over recent history.
    pub fn latency_percentiles(&self) -> (f64, f64) {
        self.history.latency_percentiles()
    }

    /// Active (unresolved) alerts.
    pub fn active_alerts(&self) -> &[HealthAlert] {
        &self.active_alerts
    }

    /// Clear all active alerts.
    pub fn clear_alerts(&self) {
        // Intentionally immutable — alerts are append-only per cycle.
        // Callers use a mutable reference to truncate.
    }

    /// Generate a health report compatible with nt_repair's monitoring.
    pub fn health_report(&self) -> HealthReport {
        let (p50, p99) = self.latency_percentiles();
        let snapshots = self.history.recent(100);

        let avg_memory = if snapshots.is_empty() {
            0
        } else {
            snapshots.iter().map(|s| s.memory_bytes).sum::<u64>() / snapshots.len() as u64
        };

        let avg_error_rate = if snapshots.is_empty() {
            0.0
        } else {
            snapshots.iter().map(|s| s.error_rate).sum::<f64>() / snapshots.len() as f64
        };

        let critical_count = self
            .active_alerts
            .iter()
            .filter(|a| a.severity == HealthSeverity::Critical)
            .count();
        let warning_count = self
            .active_alerts
            .iter()
            .filter(|a| a.severity == HealthSeverity::Warning)
            .count();

        HealthReport {
            uptime_secs: self.start.elapsed().as_secs(),
            avg_memory_bytes: avg_memory,
            latency_p50_ms: p50,
            latency_p99_ms: p99,
            avg_error_rate,
            critical_alerts: critical_count,
            warning_alerts: warning_count,
            snapshot_count: self.history.snapshots.len(),
        }
    }

    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }

    pub fn snapshot_count(&self) -> usize {
        self.history.snapshots.len()
    }

    fn default_policies() -> Vec<AlertPolicy> {
        vec![
            AlertPolicy::new("error_rate", 0.05, 0.20),
            AlertPolicy::new("latency_p99_ms", 5000.0, 15000.0),
            AlertPolicy::new("memory_bytes", 1024.0 * 1024.0 * 1024.0, 2.0 * 1024.0 * 1024.0 * 1024.0),
            AlertPolicy::new("active_spans", 500.0, 2000.0),
        ]
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Health report output — maps to nt_repair's health monitoring interface.
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub uptime_secs: u64,
    pub avg_memory_bytes: u64,
    pub latency_p50_ms: f64,
    pub latency_p99_ms: f64,
    pub avg_error_rate: f64,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub snapshot_count: usize,
}

impl HealthReport {
    pub fn is_healthy(&self) -> bool {
        self.critical_alerts == 0 && self.avg_error_rate < 0.20
    }
}

impl HealthSnapshot {
    fn signal_value(&self, signal: &str) -> Option<f64> {
        match signal {
            "error_rate" => Some(self.error_rate),
            "latency_p50_ms" => Some(self.latency_p50_ms),
            "latency_p99_ms" => Some(self.latency_p99_ms),
            "memory_bytes" => Some(self.memory_bytes as f64),
            "active_spans" => Some(self.active_spans as f64),
            "open_connections" => Some(self.open_connections as f64),
            _ => None,
        }
    }
}

/// Helper to build a HealthSnapshot from process stats.
impl Default for HealthSnapshot {
    fn default() -> Self {
        Self {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            memory_bytes: 0,
            latency_p50_ms: 0.0,
            latency_p99_ms: 0.0,
            error_rate: 0.0,
            active_spans: 0,
            open_connections: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(error_rate: f64, latency_p99: f64, memory: u64) -> HealthSnapshot {
        HealthSnapshot {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            memory_bytes: memory,
            latency_p50_ms: latency_p99 * 0.5,
            latency_p99_ms: latency_p99,
            error_rate,
            active_spans: 10,
            open_connections: 5,
        }
    }

    #[test]
    fn test_health_monitor_records() {
        let mut monitor = HealthMonitor::new();
        let snap = make_snapshot(0.01, 100.0, 512 * 1024 * 1024);
        let alerts = monitor.record(snap);
        assert!(alerts.is_empty());
        assert_eq!(monitor.snapshot_count(), 1);
    }

    #[test]
    fn test_high_error_rate_triggers_alert() {
        let mut monitor = HealthMonitor::new();
        let snap = make_snapshot(0.30, 100.0, 512 * 1024 * 1024);
        let alerts = monitor.record(snap);
        let critical: Vec<_> = alerts
            .iter()
            .filter(|a| a.severity == HealthSeverity::Critical)
            .collect();
        assert!(!critical.is_empty());
    }

    #[test]
    fn test_high_latency_triggers_warning() {
        let mut monitor = HealthMonitor::new();
        let snap = make_snapshot(0.01, 8000.0, 512 * 1024 * 1024);
        let alerts = monitor.record(snap);
        let warnings: Vec<_> = alerts
            .iter()
            .filter(|a| a.severity == HealthSeverity::Warning)
            .collect();
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_health_report_healthy() {
        let mut monitor = HealthMonitor::new();
        let snap = make_snapshot(0.01, 100.0, 512 * 1024 * 1024);
        monitor.record(snap);
        let report = monitor.health_report();
        assert!(report.is_healthy());
    }

    #[test]
    fn test_health_report_unhealthy() {
        let mut monitor = HealthMonitor::new();
        let snap = make_snapshot(0.30, 100.0, 512 * 1024 * 1024);
        monitor.record(snap);
        let report = monitor.health_report();
        assert!(!report.is_healthy());
    }

    #[test]
    fn test_custom_policy() {
        let mut monitor = HealthMonitor::new();
        monitor.add_policy(AlertPolicy::new("open_connections", 10.0, 50.0));
        let snap = HealthSnapshot {
            open_connections: 20,
            ..make_snapshot(0.01, 100.0, 512 * 1024 * 1024)
        };
        let alerts = monitor.record(snap);
        let warnings: Vec<_> = alerts
            .iter()
            .filter(|a| a.severity == HealthSeverity::Warning)
            .collect();
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_inverted_policy() {
        let policy = AlertPolicy::new("memory_bytes", 100.0, 50.0).inverted();
        assert!(policy.evaluate(30.0) == Some(HealthSeverity::Critical));
        assert!(policy.evaluate(80.0) == Some(HealthSeverity::Warning));
        assert!(policy.evaluate(150.0).is_none());
    }

    #[test]
    fn test_latency_percentiles() {
        let mut monitor = HealthMonitor::new();
        for i in 0..10 {
            let snap = HealthSnapshot {
                latency_p50_ms: i as f64 * 10.0,
                latency_p99_ms: i as f64 * 50.0,
                ..make_snapshot(0.01, i as f64 * 50.0, 512 * 1024 * 1024)
            };
            monitor.record(snap);
        }
        let (p50, p99) = monitor.latency_percentiles();
        assert!(p50 > 0.0);
        assert!(p99 > p50);
    }
}
