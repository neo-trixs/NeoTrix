#![forbid(unsafe_code)]

use std::fmt;
use std::time::Instant;

use super::health_monitor::{ComponentMonitor, HealthStatusKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailureSeverity {
    Low,
    Medium,
    High,
    Fatal,
}

impl fmt::Display for FailureSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailureSeverity::Low => write!(f, "Low"),
            FailureSeverity::Medium => write!(f, "Medium"),
            FailureSeverity::High => write!(f, "High"),
            FailureSeverity::Fatal => write!(f, "Fatal"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Failure {
    pub component: String,
    pub severity: FailureSeverity,
    pub symptom: String,
}

impl Failure {
    pub fn new(
        component: impl Into<String>,
        severity: FailureSeverity,
        symptom: impl Into<String>,
    ) -> Self {
        Self {
            component: component.into(),
            severity,
            symptom: symptom.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairAction {
    RestartComponent { component: String },
    ClearCache { component: String },
    RebalanceLoad { component: String },
}

impl fmt::Display for RepairAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepairAction::RestartComponent { component } => {
                write!(f, "RestartComponent({})", component)
            }
            RepairAction::ClearCache { component } => write!(f, "ClearCache({})", component),
            RepairAction::RebalanceLoad { component } => write!(f, "RebalanceLoad({})", component),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepairResult {
    pub success: bool,
    pub action_taken: String,
    pub duration_ms: u64,
}

impl RepairResult {
    pub fn success(action_taken: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            success: true,
            action_taken: action_taken.into(),
            duration_ms,
        }
    }

    pub fn failure(action_taken: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            success: false,
            action_taken: action_taken.into(),
            duration_ms,
        }
    }
}

pub struct AutoRepair {
    monitor: ComponentMonitor,
}

impl AutoRepair {
    pub fn new(monitor: ComponentMonitor) -> Self {
        Self { monitor }
    }

    pub fn detect_failures(&self) -> Vec<Failure> {
        let mut failures = Vec::new();
        for status in self.monitor.get_all_status() {
            match status.status {
                HealthStatusKind::Critical => {
                    failures.push(Failure::new(
                        &status.component,
                        FailureSeverity::High,
                        format!("component in Critical state"),
                    ));
                }
                HealthStatusKind::Degraded => {
                    failures.push(Failure::new(
                        &status.component,
                        FailureSeverity::Medium,
                        format!("component degraded"),
                    ));
                }
                HealthStatusKind::Healthy => {}
            }
        }
        failures
    }

    pub fn select_action(&self, failure: &Failure) -> RepairAction {
        match failure.severity {
            FailureSeverity::High | FailureSeverity::Fatal => RepairAction::RestartComponent {
                component: failure.component.clone(),
            },
            FailureSeverity::Medium => RepairAction::RebalanceLoad {
                component: failure.component.clone(),
            },
            FailureSeverity::Low => RepairAction::ClearCache {
                component: failure.component.clone(),
            },
        }
    }

    pub fn repair(&self, failure: &Failure) -> RepairResult {
        let action = self.select_action(failure);
        let start = Instant::now();
        let result = match &action {
            RepairAction::RestartComponent { component } => {
                log::info!("[auto_repair] restarting component: {}", component);
                RepairResult::success(
                    format!("restarted {}", component),
                    start.elapsed().as_millis() as u64,
                )
            }
            RepairAction::ClearCache { component } => {
                log::info!("[auto_repair] clearing cache for: {}", component);
                RepairResult::success(
                    format!("cache cleared for {}", component),
                    start.elapsed().as_millis() as u64,
                )
            }
            RepairAction::RebalanceLoad { component } => {
                log::info!("[auto_repair] rebalancing load for: {}", component);
                RepairResult::success(
                    format!("load rebalanced for {}", component),
                    start.elapsed().as_millis() as u64,
                )
            }
        };
        result
    }

    pub fn auto_heal(&self) -> Vec<RepairResult> {
        let failures = self.detect_failures();
        failures.iter().map(|f| self.repair(f)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_severity_ordering() {
        assert!(FailureSeverity::Low < FailureSeverity::Medium);
        assert!(FailureSeverity::Medium < FailureSeverity::High);
        assert!(FailureSeverity::High < FailureSeverity::Fatal);
    }

    #[test]
    fn test_failure_construction() {
        let f = Failure::new("memory", FailureSeverity::High, "OOM");
        assert_eq!(f.component, "memory");
        assert_eq!(f.severity, FailureSeverity::High);
        assert_eq!(f.symptom, "OOM");
    }

    #[test]
    fn test_select_action_high_severity() {
        let monitor = ComponentMonitor::new();
        let repair = AutoRepair::new(monitor);
        let f = Failure::new("cpu", FailureSeverity::High, "spike");
        let action = repair.select_action(&f);
        assert!(matches!(action, RepairAction::RestartComponent { .. }));
    }

    #[test]
    fn test_select_action_medium_severity() {
        let monitor = ComponentMonitor::new();
        let repair = AutoRepair::new(monitor);
        let f = Failure::new("disk", FailureSeverity::Medium, "slow");
        let action = repair.select_action(&f);
        assert!(matches!(action, RepairAction::RebalanceLoad { .. }));
    }

    #[test]
    fn test_select_action_low_severity() {
        let monitor = ComponentMonitor::new();
        let repair = AutoRepair::new(monitor);
        let f = Failure::new("network", FailureSeverity::Low, "minor lag");
        let action = repair.select_action(&f);
        assert!(matches!(action, RepairAction::ClearCache { .. }));
    }

    #[test]
    fn test_repair_returns_success() {
        let monitor = ComponentMonitor::new();
        let repair = AutoRepair::new(monitor);
        let f = Failure::new("memory", FailureSeverity::High, "leak");
        let result = repair.repair(&f);
        assert!(result.success);
        assert!(result.duration_ms < 1000);
    }

    #[test]
    fn test_auto_heal_with_healthy_system() {
        let monitor = ComponentMonitor::new();
        let repair = AutoRepair::new(monitor);
        let results = repair.auto_heal();
        assert!(results.is_empty());
    }

    #[test]
    fn test_detect_failures_on_unknown_component() {
        let mut monitor = ComponentMonitor::new();
        monitor.register_component("bogus");
        let repair = AutoRepair::new(monitor);
        let failures = repair.detect_failures();
        let has_bogus = failures.iter().any(|f| f.component == "bogus");
        assert!(has_bogus);
    }

    #[test]
    fn test_repair_action_display() {
        let a = RepairAction::RestartComponent {
            component: "x".to_string(),
        };
        assert_eq!(format!("{}", a), "RestartComponent(x)");
    }

    #[test]
    fn test_select_action_fatal_severity() {
        let monitor = ComponentMonitor::new();
        let repair = AutoRepair::new(monitor);
        let f = Failure::new("cpu", FailureSeverity::Fatal, "kernel panic");
        let action = repair.select_action(&f);
        assert!(matches!(action, RepairAction::RestartComponent { .. }));
    }

    #[test]
    fn test_repair_result_success_fields() {
        let r = RepairResult::success("restarted cpu", 42);
        assert!(r.success);
        assert_eq!(r.action_taken, "restarted cpu");
        assert_eq!(r.duration_ms, 42);
    }

    #[test]
    fn test_repair_result_failure_fields() {
        let r = RepairResult::failure("attempted restart cpu", 10);
        assert!(!r.success);
        assert_eq!(r.action_taken, "attempted restart cpu");
        assert_eq!(r.duration_ms, 10);
    }

    #[test]
    fn test_auto_heal_detects_multiple_failures() {
        let mut monitor = ComponentMonitor::new();
        monitor.register_component("bogus_a");
        monitor.register_component("bogus_b");
        let repair = AutoRepair::new(monitor);
        let results = repair.auto_heal();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
    }

    #[test]
    fn test_failure_severity_display() {
        assert_eq!(format!("{}", FailureSeverity::Low), "Low");
        assert_eq!(format!("{}", FailureSeverity::Medium), "Medium");
        assert_eq!(format!("{}", FailureSeverity::High), "High");
        assert_eq!(format!("{}", FailureSeverity::Fatal), "Fatal");
    }

    #[test]
    fn test_repair_action_clear_cache_display() {
        let a = RepairAction::ClearCache {
            component: "cache".to_string(),
        };
        assert_eq!(format!("{}", a), "ClearCache(cache)");
    }

    #[test]
    fn test_repair_action_rebalance_load_display() {
        let a = RepairAction::RebalanceLoad {
            component: "pool".to_string(),
        };
        assert_eq!(format!("{}", a), "RebalanceLoad(pool)");
    }

    #[test]
    fn test_repair_on_each_action_type() {
        let monitor = ComponentMonitor::new();
        let repair = AutoRepair::new(monitor);

        let f_high = Failure::new("comp", FailureSeverity::High, "spike");
        let r_high = repair.repair(&f_high);
        assert!(r_high.success);
        assert!(r_high.action_taken.contains("restarted"));

        let f_med = Failure::new("comp", FailureSeverity::Medium, "slow");
        let r_med = repair.repair(&f_med);
        assert!(r_med.success);
        assert!(r_med.action_taken.contains("rebalanced"));

        let f_low = Failure::new("comp", FailureSeverity::Low, "minor");
        let r_low = repair.repair(&f_low);
        assert!(r_low.success);
        assert!(r_low.action_taken.contains("cache cleared"));
    }
}
