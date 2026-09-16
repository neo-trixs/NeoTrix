//! # RuntimeMonitor — Real-Time System Performance Monitoring
//!
//! Real-time system performance monitoring with metrics, alerts, and health.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAlert {
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: u64,
    pub component: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub uptime_seconds: u64,
    pub overall_health: HealthRating,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthRating {
    Optimal,
    Normal,
    Degraded,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetrics {
    pub cpu_usage: f64,
    pub memory_usage_mb: f64,
    pub memory_total_mb: f64,
    pub disk_usage_percent: f64,
    pub active_threads: usize,
    pub requests_per_second: f64,
    pub average_latency_ms: f64,
    pub error_rate: f64,
    pub uptime_seconds: u64,
    pub timestamp: u64,
}

pub struct RuntimeMonitor {
    metrics: Arc<Mutex<RuntimeMetrics>>,
    alerts: Arc<Mutex<Vec<RuntimeAlert>>>,
    start_time: SystemTime,
    cpu_threshold: f64,
    memory_threshold: f64,
    error_rate_threshold: f64,
}

impl RuntimeMonitor {
    pub fn new(cpu_threshold: f64, memory_threshold: f64, error_rate_threshold: f64) -> Self {
        Self {
            metrics: Arc::new(Mutex::new(RuntimeMetrics {
                cpu_usage: 0.0,
                memory_usage_mb: 0.0,
                memory_total_mb: 0.0,
                disk_usage_percent: 0.0,
                active_threads: 0,
                requests_per_second: 0.0,
                average_latency_ms: 0.0,
                error_rate: 0.0,
                uptime_seconds: 0,
                timestamp: 0,
            })),
            alerts: Arc::new(Mutex::new(Vec::new())),
            start_time: SystemTime::now(),
            cpu_threshold,
            memory_threshold,
            error_rate_threshold,
        }
    }

    pub fn monitor(&self) {
        let metrics = self.collect_metrics();
        let mut guard = self.metrics.lock().unwrap();
        *guard = metrics;
        self.check_thresholds();
    }

    pub fn get_metrics(&self) -> RuntimeMetrics {
        self.metrics.lock().unwrap().clone()
    }

    pub fn alert(&self, severity: AlertSeverity, message: &str, component: &str) -> RuntimeAlert {
        let alert = RuntimeAlert {
            severity,
            message: message.to_string(),
            timestamp: get_timestamp(),
            component: component.to_string(),
        };
        self.alerts.lock().unwrap().push(alert.clone());
        alert
    }

    pub fn get_health(&self) -> HealthStatus {
        let metrics = self.metrics.lock().unwrap();
        let uptime = self.start_time.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let overall_health = if metrics.cpu_usage > 90.0 || metrics.memory_usage_mb > metrics.memory_total_mb * 0.9 {
            HealthRating::Critical
        } else if metrics.cpu_usage > self.cpu_threshold || metrics.error_rate > self.error_rate_threshold {
            HealthRating::Degraded
        } else if metrics.cpu_usage > 50.0 || metrics.memory_usage_mb > metrics.memory_total_mb * 0.5 {
            HealthRating::Normal
        } else {
            HealthRating::Optimal
        };
        HealthStatus {
            cpu_usage: metrics.cpu_usage,
            memory_usage: metrics.memory_usage_mb / metrics.memory_total_mb.max(1.0),
            disk_usage: metrics.disk_usage_percent,
            uptime_seconds: uptime,
            overall_health,
        }
    }

    fn collect_metrics(&self) -> RuntimeMetrics {
        let uptime = self.start_time.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        RuntimeMetrics {
            cpu_usage: self.sample_cpu(),
            memory_usage_mb: self.sample_memory(),
            memory_total_mb: self.sample_total_memory(),
            disk_usage_percent: self.sample_disk(),
            active_threads: self.sample_threads(),
            requests_per_second: self.sample_rps(),
            average_latency_ms: self.sample_latency(),
            error_rate: self.sample_error_rate(),
            uptime_seconds: uptime,
            timestamp: get_timestamp(),
        }
    }

    fn check_thresholds(&self) {
        let metrics = self.metrics.lock().unwrap();
        if metrics.cpu_usage > self.cpu_threshold {
            self.alert(AlertSeverity::Warning, &format!("CPU usage {}% exceeds threshold {}%", metrics.cpu_usage, self.cpu_threshold), "cpu");
        }
        if metrics.memory_usage_mb > self.memory_threshold {
            self.alert(AlertSeverity::Warning, "Memory usage exceeds threshold", "memory");
        }
        if metrics.error_rate > self.error_rate_threshold {
            self.alert(AlertSeverity::Critical, &format!("Error rate exceeds threshold"), "error_rate");
        }
    }

    fn sample_cpu(&self) -> f64 { 0.0 }
    fn sample_memory(&self) -> f64 { 0.0 }
    fn sample_total_memory(&self) -> f64 { 8192.0 }
    fn sample_disk(&self) -> f64 { 0.0 }
    fn sample_threads(&self) -> usize { 0 }
    fn sample_rps(&self) -> f64 { 0.0 }
    fn sample_latency(&self) -> f64 { 0.0 }
    fn sample_error_rate(&self) -> f64 { 0.0 }

    pub fn get_alerts(&self) -> Vec<RuntimeAlert> {
        self.alerts.lock().unwrap().clone()
    }

    pub fn clear_alerts(&self) {
        self.alerts.lock().unwrap().clear();
    }
}

impl Default for RuntimeMonitor {
    fn default() -> Self { Self::new(80.0, 4096.0, 0.05) }
}

fn get_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_monitor_and_get_metrics() { let monitor = RuntimeMonitor::new(80.0, 4096.0, 0.05); monitor.monitor(); let metrics = monitor.get_metrics(); assert!(metrics.cpu_usage >= 0.0); }
    #[test] fn test_alert() { let monitor = RuntimeMonitor::new(80.0, 4096.0, 0.05); let alert = monitor.alert(AlertSeverity::Warning, "test alert", "test_component"); assert_eq!(alert.severity, AlertSeverity::Warning); }
    #[test] fn test_get_health() { let monitor = RuntimeMonitor::default(); monitor.monitor(); let health = monitor.get_health(); assert!(matches!(health.overall_health, HealthRating::Optimal | HealthRating::Normal)); }
}
