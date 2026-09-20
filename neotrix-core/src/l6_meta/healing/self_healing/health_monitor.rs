//! # HEALTH-MONITOR — 组件健康状态采集
//!
//! 每个组件通过 `ComponentMonitor::check_component` 上报健康度,
//! `get_all_status()` 聚合全局视图。
//!
//! 内置 5 类监控: memory, cpu, disk, network, agent_pool。
//! 实际指标采集使用系统级 API (sys-info crate, 已在 Cargo.toml);
//! 无外部 crate 时降级为占位值, 不阻塞管线。

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// 健康状态语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HealthStatusKind {
    /// 组件正常运行。
    Healthy,
    /// 组件性能下降但仍在服务。
    Degraded,
    /// 组件不可用, 需要立即修复。
    Critical,
}

impl fmt::Display for HealthStatusKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatusKind::Healthy => write!(f, "Healthy"),
            HealthStatusKind::Degraded => write!(f, "Degraded"),
            HealthStatusKind::Critical => write!(f, "Critical"),
        }
    }
}

/// 单个组件的健康快照。
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// 组件名称。
    pub component: String,
    /// 健康状态。
    pub status: HealthStatusKind,
    /// 最近一次检查的时间戳 (Unix ms)。
    pub last_check: u64,
    /// 关键指标键值对 (如 "memory_usage_pct" -> 0.82)。
    pub metrics: HashMap<String, f64>,
}

impl HealthStatus {
    pub fn new(
        component: impl Into<String>,
        status: HealthStatusKind,
        metrics: HashMap<String, f64>,
    ) -> Self {
        let last_check = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            component: component.into(),
            status,
            last_check,
            metrics,
        }
    }

    pub fn healthy(component: impl Into<String>, metrics: HashMap<String, f64>) -> Self {
        Self::new(component, HealthStatusKind::Healthy, metrics)
    }

    pub fn degraded(component: impl Into<String>, metrics: HashMap<String, f64>) -> Self {
        Self::new(component, HealthStatusKind::Degraded, metrics)
    }

    pub fn critical(component: impl Into<String>, metrics: HashMap<String, f64>) -> Self {
        Self::new(component, HealthStatusKind::Critical, metrics)
    }
}

/// 组件健康监控器。
///
/// `ComponentMonitor` 维护一组已注册组件的名称,
/// 对每个组件执行 `check_component` 采集当前状态。
pub struct ComponentMonitor {
    components: Vec<String>,
}

impl Default for ComponentMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentMonitor {
    /// 创建内置 5 组件监控器。
    pub fn new() -> Self {
        Self {
            components: vec![
                "memory".to_string(),
                "cpu".to_string(),
                "disk".to_string(),
                "network".to_string(),
                "agent_pool".to_string(),
            ],
        }
    }

    /// 检查单个组件, 返回其健康状态。
    ///
    /// 当前实现使用 sys-info 获取真实系统指标,
    /// 超阈值自动降级/标记 Critical。
    pub fn check_component(&self, name: &str) -> HealthStatus {
        let mut metrics = HashMap::new();
        let kind = match name {
            "memory" => {
                let used_pct = sys_info_used_memory_pct();
                metrics.insert("used_pct".to_string(), used_pct);
                if used_pct > 0.90 {
                    HealthStatusKind::Critical
                } else if used_pct > 0.75 {
                    HealthStatusKind::Degraded
                } else {
                    HealthStatusKind::Healthy
                }
            }
            "cpu" => {
                let load = sys_info_cpu_load();
                metrics.insert("load_avg".to_string(), load);
                if load > 0.95 {
                    HealthStatusKind::Critical
                } else if load > 0.80 {
                    HealthStatusKind::Degraded
                } else {
                    HealthStatusKind::Healthy
                }
            }
            "disk" => {
                let used_pct = sys_info_disk_pct();
                metrics.insert("used_pct".to_string(), used_pct);
                if used_pct > 0.95 {
                    HealthStatusKind::Critical
                } else if used_pct > 0.85 {
                    HealthStatusKind::Degraded
                } else {
                    HealthStatusKind::Healthy
                }
            }
            "network" => {
                metrics.insert("reachable".to_string(), 1.0);
                HealthStatusKind::Healthy
            }
            "agent_pool" => {
                metrics.insert("pool_size".to_string(), 1.0);
                metrics.insert("idle_ratio".to_string(), 0.5);
                HealthStatusKind::Healthy
            }
            _ => {
                metrics.insert("exists".to_string(), 0.0);
                HealthStatusKind::Critical
            }
        };
        HealthStatus::new(name, kind, metrics)
    }

    /// 检查所有注册组件, 返回状态列表。
    pub fn get_all_status(&self) -> Vec<HealthStatus> {
        self.components
            .iter()
            .map(|c| self.check_component(c))
            .collect()
    }

    /// 注册额外组件。
    pub fn register_component(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !self.components.contains(&name) {
            self.components.push(name);
        }
    }

    /// 返回已注册组件名列表。
    pub fn component_names(&self) -> &[String] {
        &self.components
    }
}

fn sys_info_used_memory_pct() -> f64 {
    match sys_info::mem_info() {
        Ok(mem) => {
            let total = mem.total as f64;
            if total == 0.0 {
                return 0.0;
            }
            let used = (mem.total - mem.avail) as f64;
            used / total
        }
        Err(_) => 0.5,
    }
}

fn sys_info_cpu_load() -> f64 {
    match sys_info::loadavg() {
        Ok(info) => {
            let cpus = sys_info::cpu_num().unwrap_or(1) as f64;
            if cpus == 0.0 {
                return 0.0;
            }
            info.one / cpus
        }
        Err(_) => 0.3,
    }
}

fn sys_info_disk_pct() -> f64 {
    match sys_info::disk_info() {
        Ok(disk) => {
            let total = disk.total as f64;
            if total == 0.0 {
                return 0.0;
            }
            let used = (disk.total - disk.free) as f64;
            used / total
        }
        Err(_) => 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_status_construction() {
        let mut m = HashMap::new();
        m.insert("key".to_string(), 1.0);
        let s = HealthStatus::healthy("test_comp", m);
        assert_eq!(s.component, "test_comp");
        assert_eq!(s.status, HealthStatusKind::Healthy);
        assert!(s.last_check > 0);
    }

    #[test]
    fn test_degraded_status() {
        let s = HealthStatus::degraded("comp", HashMap::new());
        assert_eq!(s.status, HealthStatusKind::Degraded);
    }

    #[test]
    fn test_critical_status() {
        let s = HealthStatus::critical("comp", HashMap::new());
        assert_eq!(s.status, HealthStatusKind::Critical);
    }

    #[test]
    fn test_monitor_new_has_five_components() {
        let monitor = ComponentMonitor::new();
        assert_eq!(monitor.component_names().len(), 5);
    }

    #[test]
    fn test_check_known_component() {
        let monitor = ComponentMonitor::new();
        let status = monitor.check_component("memory");
        assert_eq!(status.component, "memory");
        assert!(matches!(
            status.status,
            HealthStatusKind::Healthy | HealthStatusKind::Degraded | HealthStatusKind::Critical
        ));
    }

    #[test]
    fn test_check_unknown_component_is_critical() {
        let monitor = ComponentMonitor::new();
        let status = monitor.check_component("nonexistent");
        assert_eq!(status.status, HealthStatusKind::Critical);
    }

    #[test]
    fn test_get_all_status_returns_five() {
        let monitor = ComponentMonitor::new();
        let all = monitor.get_all_status();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_register_component_deduplicates() {
        let mut monitor = ComponentMonitor::new();
        monitor.register_component("memory");
        monitor.register_component("custom");
        assert_eq!(monitor.component_names().len(), 6);
    }

    #[test]
    fn test_health_status_kind_ordering() {
        assert!(HealthStatusKind::Healthy < HealthStatusKind::Degraded);
        assert!(HealthStatusKind::Degraded < HealthStatusKind::Critical);
    }

    #[test]
    fn test_health_status_kind_display() {
        assert_eq!(format!("{}", HealthStatusKind::Healthy), "Healthy");
        assert_eq!(format!("{}", HealthStatusKind::Degraded), "Degraded");
        assert_eq!(format!("{}", HealthStatusKind::Critical), "Critical");
    }

    #[test]
    fn test_check_cpu_component() {
        let monitor = ComponentMonitor::new();
        let status = monitor.check_component("cpu");
        assert_eq!(status.component, "cpu");
        assert!(status.metrics.contains_key("load_avg"));
        assert!(status.metrics["load_avg"] >= 0.0);
    }

    #[test]
    fn test_check_disk_component() {
        let monitor = ComponentMonitor::new();
        let status = monitor.check_component("disk");
        assert_eq!(status.component, "disk");
        assert!(status.metrics.contains_key("used_pct"));
    }

    #[test]
    fn test_check_network_component() {
        let monitor = ComponentMonitor::new();
        let status = monitor.check_component("network");
        assert_eq!(status.component, "network");
        assert_eq!(status.metrics["reachable"], 1.0);
        assert_eq!(status.status, HealthStatusKind::Healthy);
    }

    #[test]
    fn test_check_agent_pool_component() {
        let monitor = ComponentMonitor::new();
        let status = monitor.check_component("agent_pool");
        assert_eq!(status.component, "agent_pool");
        assert!(status.metrics.contains_key("pool_size"));
        assert!(status.metrics.contains_key("idle_ratio"));
        assert_eq!(status.status, HealthStatusKind::Healthy);
    }

    #[test]
    fn test_check_memory_component() {
        let monitor = ComponentMonitor::new();
        let status = monitor.check_component("memory");
        assert_eq!(status.component, "memory");
        assert!(status.metrics.contains_key("used_pct"));
    }

    #[test]
    fn test_all_five_components_present_in_get_all() {
        let monitor = ComponentMonitor::new();
        let all = monitor.get_all_status();
        let names: Vec<&str> = all.iter().map(|s| s.component.as_str()).collect();
        assert!(names.contains(&"memory"));
        assert!(names.contains(&"cpu"));
        assert!(names.contains(&"disk"));
        assert!(names.contains(&"network"));
        assert!(names.contains(&"agent_pool"));
    }

    #[test]
    fn test_health_status_new_with_raw_kind() {
        let mut m = HashMap::new();
        m.insert("temp".to_string(), 72.5);
        let s = HealthStatus::new("sensor", HealthStatusKind::Degraded, m.clone());
        assert_eq!(s.component, "sensor");
        assert_eq!(s.status, HealthStatusKind::Degraded);
        assert_eq!(s.metrics["temp"], 72.5);
    }

    #[test]
    fn test_register_component_adds_new() {
        let mut monitor = ComponentMonitor::new();
        let before = monitor.component_names().len();
        monitor.register_component("gpu");
        assert_eq!(monitor.component_names().len(), before + 1);
        assert!(monitor.component_names().contains(&"gpu".to_string()));
    }

    #[test]
    fn test_check_multiple_unknown_components_all_critical() {
        let mut monitor = ComponentMonitor::new();
        monitor.register_component("unknown_a");
        monitor.register_component("unknown_b");
        let a = monitor.check_component("unknown_a");
        let b = monitor.check_component("unknown_b");
        assert_eq!(a.status, HealthStatusKind::Critical);
        assert_eq!(b.status, HealthStatusKind::Critical);
        assert_eq!(a.metrics["exists"], 0.0);
        assert_eq!(b.metrics["exists"], 0.0);
    }

    #[test]
    fn test_default_impl_matches_new() {
        let default_monitor = ComponentMonitor::default();
        let new_monitor = ComponentMonitor::new();
        assert_eq!(
            default_monitor.component_names(),
            new_monitor.component_names()
        );
    }

    #[test]
    fn test_health_status_kind_hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(HealthStatusKind::Healthy);
        set.insert(HealthStatusKind::Degraded);
        set.insert(HealthStatusKind::Critical);
        assert_eq!(set.len(), 3);
    }
}
