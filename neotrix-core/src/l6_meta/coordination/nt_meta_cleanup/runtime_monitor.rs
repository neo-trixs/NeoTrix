#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub uptime_secs: u64,
}

pub struct RuntimeMonitor {
    cpu_warn: f64,
    cpu_crit: f64,
    mem_warn: f64,
    mem_crit: f64,
}

impl RuntimeMonitor {
    pub fn new() -> Self {
        Self {
            cpu_warn: 70.0,
            cpu_crit: 90.0,
            mem_warn: 80.0,
            mem_crit: 95.0,
        }
    }

    pub fn check(&self, m: &SystemMetrics) -> Vec<String> {
        let mut alerts = Vec::new();
        if m.cpu_usage >= self.cpu_crit {
            alerts.push(format!("CPU CRITICAL: {:.1}%", m.cpu_usage));
        } else if m.cpu_usage >= self.cpu_warn {
            alerts.push(format!("CPU WARNING: {:.1}%", m.cpu_usage));
        }
        if m.memory_usage >= self.mem_crit {
            alerts.push(format!("MEMORY CRITICAL: {:.1}%", m.memory_usage));
        } else if m.memory_usage >= self.mem_warn {
            alerts.push(format!("MEMORY WARNING: {:.1}%", m.memory_usage));
        }
        alerts
    }

    pub fn is_healthy(&self, m: &SystemMetrics) -> bool {
        m.cpu_usage < self.cpu_crit && m.memory_usage < self.mem_crit
    }
}

impl Default for RuntimeMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy() {
        let mon = RuntimeMonitor::new();
        assert!(mon.is_healthy(&SystemMetrics {
            cpu_usage: 50.0,
            memory_usage: 60.0,
            disk_usage: 30.0,
            uptime_secs: 100,
        }));
    }

    #[test]
    fn test_critical() {
        let mon = RuntimeMonitor::new();
        assert!(!mon.is_healthy(&SystemMetrics {
            cpu_usage: 95.0,
            memory_usage: 60.0,
            disk_usage: 30.0,
            uptime_secs: 100,
        }));
    }
}
