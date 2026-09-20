#![forbid(unsafe_code)]

use std::fmt;

/// Severity of a health signal, ordered from least to most severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A single health signal emitted by a monitored component.
///
/// Each signal captures a metric reading from a specific component at a point in time.
/// The diagnostician consumes a batch of these to determine root causes.
#[derive(Debug, Clone)]
pub struct HealthSignal {
    /// Component identifier (e.g. "nt_core_cache", "nt_world_crawl").
    pub component: String,
    /// Metric name (e.g. "latency_ms", "error_rate", "memory_bytes").
    pub metric_name: String,
    /// Numeric value of the metric.
    pub value: f64,
    /// Unix timestamp in milliseconds when this signal was recorded.
    pub timestamp: u64,
    /// Severity classification.
    pub severity: Severity,
}

impl HealthSignal {
    pub fn new(
        component: impl Into<String>,
        metric_name: impl Into<String>,
        value: f64,
        timestamp: u64,
        severity: Severity,
    ) -> Self {
        Self {
            component: component.into(),
            metric_name: metric_name.into(),
            value,
            timestamp,
            severity,
        }
    }

    /// Convenience: create a Critical signal.
    pub fn critical(
        component: impl Into<String>,
        metric_name: impl Into<String>,
        value: f64,
        timestamp: u64,
    ) -> Self {
        Self::new(component, metric_name, value, timestamp, Severity::Critical)
    }

    /// Convenience: create a Warning signal.
    pub fn warning(
        component: impl Into<String>,
        metric_name: impl Into<String>,
        value: f64,
        timestamp: u64,
    ) -> Self {
        Self::new(component, metric_name, value, timestamp, Severity::Warning)
    }

    /// Convenience: create an Info signal.
    pub fn info(
        component: impl Into<String>,
        metric_name: impl Into<String>,
        value: f64,
        timestamp: u64,
    ) -> Self {
        Self::new(component, metric_name, value, timestamp, Severity::Info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Critical);
    }

    #[test]
    fn test_health_signal_construction() {
        let s = HealthSignal::warning("nt_core_cache", "hit_rate", 0.3, 1000);
        assert_eq!(s.component, "nt_core_cache");
        assert_eq!(s.severity, Severity::Warning);
    }

    #[test]
    fn test_critical_convenience() {
        let s = HealthSignal::critical("nt_world_crawl", "error_rate", 0.95, 2000);
        assert_eq!(s.severity, Severity::Critical);
    }

    #[test]
    fn test_info_convenience() {
        let s = HealthSignal::info("nt_core", "uptime", 3600.0, 5000);
        assert_eq!(s.severity, Severity::Info);
        assert_eq!(s.component, "nt_core");
        assert_eq!(s.value, 3600.0);
        assert_eq!(s.timestamp, 5000);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Info.to_string(), "INFO");
        assert_eq!(Severity::Warning.to_string(), "WARNING");
        assert_eq!(Severity::Critical.to_string(), "CRITICAL");
    }

    #[test]
    fn test_health_signal_clone() {
        let s = HealthSignal::warning("comp", "metric", 1.0, 100);
        let cloned = s.clone();
        assert_eq!(s.component, cloned.component);
        assert_eq!(s.metric_name, cloned.metric_name);
        assert_eq!(s.value, cloned.value);
        assert_eq!(s.timestamp, cloned.timestamp);
        assert_eq!(s.severity, cloned.severity);
    }

    #[test]
    fn test_health_signal_metric_name() {
        let s = HealthSignal::new("db", "query_latency_ms", 42.5, 1000, Severity::Warning);
        assert_eq!(s.metric_name, "query_latency_ms");
    }

    #[test]
    fn test_severity_hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Severity::Info);
        set.insert(Severity::Warning);
        set.insert(Severity::Critical);
        assert_eq!(set.len(), 3);
    }
}
