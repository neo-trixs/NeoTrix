//! Boot and resource metrics tracking for AI infrastructure.

use serde::{Deserialize, Serialize};

/// Metrics for environment boot performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootMetrics {
    pub boot_time_ms: u64,
    pub pre_alloc_time_ms: u64,
    pub restore_time_ms: u64,
    pub layer_load_time_ms: u64,
    pub success: bool,
    pub timestamp: u64,
}

/// Resource usage metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub disk_io_bytes: u64,
    pub network_io_bytes: u64,
    pub timestamp: u64,
}

/// Fleet-wide metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetMetrics {
    pub total_isolates: usize,
    pub active_isolates: usize,
    pub avg_boot_time_ms: f64,
    pub total_memory_mb: u64,
    pub total_cpu_cores: f64,
    pub uptime_seconds: u64,
}

/// Metrics collector for tracking boot and resource usage.
pub struct MetricsCollector {
    boot_metrics: Vec<BootMetrics>,
    resource_metrics: Vec<ResourceMetrics>,
    start_time: u64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            boot_metrics: Vec::new(),
            resource_metrics: Vec::new(),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    /// Record boot metrics.
    pub fn record_boot(&mut self, metrics: BootMetrics) {
        self.boot_metrics.push(metrics);
    }

    /// Record resource metrics.
    pub fn record_resource(&mut self, metrics: ResourceMetrics) {
        self.resource_metrics.push(metrics);
    }

    /// Get average boot time.
    pub fn avg_boot_time_ms(&self) -> f64 {
        if self.boot_metrics.is_empty() {
            return 0.0;
        }
        let total: u64 = self.boot_metrics.iter().map(|m| m.boot_time_ms).sum();
        total as f64 / self.boot_metrics.len() as f64
    }

    /// Get boot success rate.
    pub fn boot_success_rate(&self) -> f64 {
        if self.boot_metrics.is_empty() {
            return 0.0;
        }
        let successes = self.boot_metrics.iter().filter(|m| m.success).count();
        successes as f64 / self.boot_metrics.len() as f64
    }

    /// Get latest resource metrics.
    pub fn latest_resource(&self) -> Option<&ResourceMetrics> {
        self.resource_metrics.last()
    }

    /// Get uptime in seconds.
    pub fn uptime_seconds(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        (now - self.start_time) / 1000
    }

    /// Generate fleet metrics summary.
    pub fn fleet_summary(&self, active_isolates: usize) -> FleetMetrics {
        FleetMetrics {
            total_isolates: self.boot_metrics.len(),
            active_isolates,
            avg_boot_time_ms: self.avg_boot_time_ms(),
            total_memory_mb: self
                .resource_metrics
                .iter()
                .map(|m| m.memory_usage_mb)
                .sum(),
            total_cpu_cores: self
                .resource_metrics
                .iter()
                .map(|m| m.cpu_usage_percent)
                .sum(),
            uptime_seconds: self.uptime_seconds(),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();

        collector.record_boot(BootMetrics {
            boot_time_ms: 150,
            pre_alloc_time_ms: 50,
            restore_time_ms: 50,
            layer_load_time_ms: 50,
            success: true,
            timestamp: 0,
        });

        assert_eq!(collector.avg_boot_time_ms(), 150.0);
        assert_eq!(collector.boot_success_rate(), 1.0);

        let summary = collector.fleet_summary(1);
        assert_eq!(summary.total_isolates, 1);
        assert_eq!(summary.active_isolates, 1);
    }
}
