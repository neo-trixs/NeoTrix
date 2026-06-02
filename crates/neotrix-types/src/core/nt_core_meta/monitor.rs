use super::self_model::{DebtSeverity, SelfModel};

/// Continuous health monitor that tracks project state over time,
/// generates alerts, and identifies trends.
#[derive(Debug, Clone)]
pub struct MetaMonitor {
    pub self_model: SelfModel,
    pub alerts: Vec<MetaAlert>,
    pub check_history: Vec<HealthCheck>,
}

impl MetaMonitor {
    pub fn new(model: SelfModel) -> Self {
        Self {
            self_model: model,
            alerts: Vec::new(),
            check_history: Vec::new(),
        }
    }

    pub fn run_check(&mut self) -> HealthCheck {
        let check = HealthCheck {
            timestamp: chrono::Utc::now(),
            compilation_ok: self.self_model.compilation.errors == 0,
            test_count: self.self_model.test_coverage.total_tests,
            weakness_count: 0,
            alert_count: self.alerts.len(),
        };
        self.check_history.push(check.clone());
        check
    }

    pub fn generate_alerts(&mut self) -> Vec<MetaAlert> {
        let mut new_alerts = Vec::new();

        for module in &self.self_model.modules {
            if module.total_lines > 1500 {
                new_alerts.push(MetaAlert {
                    timestamp: chrono::Utc::now(),
                    severity: AlertSeverity::Warning,
                    message: format!("Module '{}' has {} lines, exceeding 1500-line threshold", module.name, module.total_lines),
                    module: Some(module.name.clone()),
                    suggestion: "Consider refactoring into sub-modules".into(),
                });
            }
            if !module.has_tests && module.total_lines > 300 {
                new_alerts.push(MetaAlert {
                    timestamp: chrono::Utc::now(),
                    severity: AlertSeverity::Warning,
                    message: format!("Module '{}' has {} lines but zero tests", module.name, module.total_lines),
                    module: Some(module.name.clone()),
                    suggestion: "Add test coverage for this module".into(),
                });
            }
            if module.unsafe_count > 5 {
                new_alerts.push(MetaAlert {
                    timestamp: chrono::Utc::now(),
                    severity: AlertSeverity::Critical,
                    message: format!("Module '{}' has {} unsafe blocks, review required", module.name, module.unsafe_count),
                    module: Some(module.name.clone()),
                    suggestion: "Audit and minimize unsafe usage".into(),
                });
            }
        }

        for file in &self.self_model.files {
            if file.lines > 800 && !file.is_test_file {
                new_alerts.push(MetaAlert {
                    timestamp: chrono::Utc::now(),
                    severity: AlertSeverity::Info,
                    message: format!("File '{}' has {} lines, consider splitting", file.path, file.lines),
                    module: Some(file.module.clone()),
                    suggestion: "Split into smaller focused files".into(),
                });
            }
        }

        if !self.self_model.compilation.features_tested.is_empty() {
            let all_ok = self.self_model.compilation.features_tested.iter()
                .all(|_f| self.self_model.compilation.errors == 0);
            if !all_ok {
                new_alerts.push(MetaAlert {
                    timestamp: chrono::Utc::now(),
                    severity: AlertSeverity::Critical,
                    message: "Compilation errors detected in some feature combinations".into(),
                    module: None,
                    suggestion: "Run cargo check --all-features to diagnose".into(),
                });
            }
        }

        if self.self_model.tech_debt.items.iter().filter(|i| i.severity == DebtSeverity::Critical).count() > 10 {
            new_alerts.push(MetaAlert {
                timestamp: chrono::Utc::now(),
                severity: AlertSeverity::Warning,
                message: "Critical tech debt threshold exceeded (10+ items)".into(),
                module: None,
                suggestion: "Schedule a tech debt reduction sprint".into(),
            });
        }

        self.alerts.extend(new_alerts.clone());
        new_alerts
    }

    pub fn trend_analysis(&self) -> HealthTrend {
        if self.check_history.len() < 2 {
            return HealthTrend {
                compilation_stable: true,
                test_count_trend: 0,
                alert_trend: 0,
                overall: "insufficient_data".into(),
            };
        }

        let last = self.check_history.last().expect("check_history.len() >= 2 checked above");
        let first = &self.check_history[0];

        HealthTrend {
            compilation_stable: last.compilation_ok,
            test_count_trend: if last.test_count >= first.test_count { 1 } else { -1 },
            alert_trend: if last.alert_count <= first.alert_count { 1 } else { -1 },
            overall: if last.compilation_ok && last.test_count >= first.test_count {
                "improving".into()
            } else if last.compilation_ok {
                "stable".into()
            } else {
                "regressing".into()
            },
        }
    }

    pub fn weaknesses_to_alerts(&mut self, report: &super::weakness::WeaknessReport) {
        for w in &report.weaknesses {
            let severity = match w.severity {
                DebtSeverity::Critical => AlertSeverity::Critical,
                DebtSeverity::Major => AlertSeverity::Warning,
                _ => AlertSeverity::Info,
            };
            self.alerts.push(MetaAlert {
                timestamp: chrono::Utc::now(),
                severity,
                message: w.description.clone(),
                module: w.file.as_ref().map(|f| self.self_model.modules.iter()
                    .find(|m| f.contains(&m.name))
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| "unknown".into())),
                suggestion: w.suggestion.clone(),
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetaAlert {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: AlertSeverity,
    pub message: String,
    pub module: Option<String>,
    pub suggestion: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub compilation_ok: bool,
    pub test_count: usize,
    pub weakness_count: usize,
    pub alert_count: usize,
}

#[derive(Debug, Clone)]
pub struct HealthTrend {
    pub compilation_stable: bool,
    pub test_count_trend: i32,
    pub alert_trend: i32,
    pub overall: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_meta::self_model::*;

    fn make_test_model() -> SelfModel {
        let mut model = SelfModel::new();
        model.modules.push(ModuleInfo {
            name: "huge_module".into(), path: "src/huge/".into(), file_count: 1,
            total_lines: 2000, test_count: 0, has_tests: false,
            unsafe_count: 10, unwrap_count: 5, todo_count: 3,
            public_api_count: 20, description: "".into(),
        });
        model.compilation.errors = 0;
        model.compilation.features_tested = vec!["default".into(), "full".into()];
        model.test_coverage.total_tests = 100;
        model
    }

    #[test]
    fn test_monitor_generates_alerts_for_large_modules() {
        let model = make_test_model();
        let mut monitor = MetaMonitor::new(model);
        let alerts = monitor.generate_alerts();
        let large_module_alerts: Vec<_> = alerts.iter()
            .filter(|a| a.message.contains("huge_module"))
            .collect();
        assert!(!large_module_alerts.is_empty());
    }

    #[test]
    fn test_monitor_trend_insufficient_data() {
        let model = make_test_model();
        let monitor = MetaMonitor::new(model);
        let trend = monitor.trend_analysis();
        assert_eq!(trend.overall, "insufficient_data");
    }

    #[test]
    fn test_monitor_trend_with_history() {
        let model = make_test_model();
        let mut monitor = MetaMonitor::new(model);
        monitor.run_check();
        monitor.self_model.test_coverage.total_tests = 150;
        monitor.run_check();
        let trend = monitor.trend_analysis();
        assert_eq!(trend.test_count_trend, 1);
    }

    #[test]
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Info as u8 > AlertSeverity::Warning as u8);
        assert!(AlertSeverity::Warning as u8 > AlertSeverity::Critical as u8);
        assert!(AlertSeverity::Critical as u8 == 0);
    }
}
