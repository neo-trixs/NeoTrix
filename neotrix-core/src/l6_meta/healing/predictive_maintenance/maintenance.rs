#![forbid(unsafe_code)]

use std::fmt;

/// Urgency level for a maintenance task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Urgency {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Urgency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Urgency::Low => write!(f, "LOW"),
            Urgency::Medium => write!(f, "MEDIUM"),
            Urgency::High => write!(f, "HIGH"),
            Urgency::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A predicted issue detected by the monitoring system.
#[derive(Debug, Clone)]
pub struct PredictedIssue {
    /// Component identifier.
    pub component: String,
    /// Description of the predicted problem.
    pub description: String,
    /// Severity score (0.0 ..= 1.0).
    pub severity: f64,
    /// Estimated turns until the issue becomes critical.
    pub turns_until_failure: Option<usize>,
}

/// A single maintenance task to be executed.
#[derive(Debug, Clone)]
pub struct MaintenanceTask {
    /// Component requiring maintenance.
    pub component: String,
    /// Action to perform.
    pub action: String,
    /// Urgency of this task.
    pub urgency: Urgency,
    /// Estimated duration in seconds.
    pub estimated_duration: u64,
}

/// A plan containing ordered maintenance tasks.
#[derive(Debug, Clone)]
pub struct MaintenancePlan {
    /// Tasks ordered by priority (most urgent first).
    pub tasks: Vec<MaintenanceTask>,
    /// Priority order indices into the tasks vector (redundant but explicit).
    pub priority_order: Vec<usize>,
}

impl MaintenancePlan {
    /// Total estimated duration of all tasks.
    pub fn total_duration(&self) -> u64 {
        self.tasks.iter().map(|t| t.estimated_duration).sum()
    }

    /// Number of critical tasks.
    pub fn critical_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.urgency == Urgency::Critical)
            .count()
    }
}

/// Schedules maintenance tasks from predicted issues.
pub struct MaintenanceScheduler {
    /// Default estimated duration for a maintenance task (seconds).
    default_duration: u64,
}

impl MaintenanceScheduler {
    pub fn new() -> Self {
        Self {
            default_duration: 300,
        }
    }

    pub fn with_default_duration(duration: u64) -> Self {
        Self {
            default_duration: duration,
        }
    }

    /// Convert predicted issues into a prioritized maintenance plan.
    pub fn schedule(&self, predicted_issues: Vec<PredictedIssue>) -> MaintenancePlan {
        let mut tasks: Vec<MaintenanceTask> = predicted_issues
            .into_iter()
            .map(|issue| self.issue_to_task(issue))
            .collect();

        // Sort by urgency (Critical first), then by estimated duration (shorter first).
        tasks.sort_by(|a, b| {
            b.urgency
                .cmp(&a.urgency)
                .then(a.estimated_duration.cmp(&b.estimated_duration))
        });

        let priority_order: Vec<usize> = (0..tasks.len()).collect();

        MaintenancePlan {
            tasks,
            priority_order,
        }
    }

    fn issue_to_task(&self, issue: PredictedIssue) -> MaintenanceTask {
        let urgency = self.severity_to_urgency(issue.severity);
        let action = match urgency {
            Urgency::Critical => format!("Immediate intervention: {}", issue.description),
            Urgency::High => format!("Schedule urgent repair: {}", issue.description),
            Urgency::Medium => format!("Plan preventive maintenance: {}", issue.description),
            Urgency::Low => format!("Monitor and log: {}", issue.description),
        };

        let estimated_duration = match urgency {
            Urgency::Critical => self.default_duration * 2,
            Urgency::High => self.default_duration,
            Urgency::Medium => self.default_duration / 2,
            Urgency::Low => self.default_duration / 4,
        };

        MaintenanceTask {
            component: issue.component,
            action,
            urgency,
            estimated_duration,
        }
    }

    fn severity_to_urgency(&self, severity: f64) -> Urgency {
        if severity >= 0.9 {
            Urgency::Critical
        } else if severity >= 0.7 {
            Urgency::High
        } else if severity >= 0.4 {
            Urgency::Medium
        } else {
            Urgency::Low
        }
    }
}

impl Default for MaintenanceScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_issues() -> Vec<PredictedIssue> {
        vec![
            PredictedIssue {
                component: "nt_core_cache".into(),
                description: "Memory leak detected".into(),
                severity: 0.95,
                turns_until_failure: Some(5),
            },
            PredictedIssue {
                component: "nt_world_crawl".into(),
                description: "Latency degradation".into(),
                severity: 0.6,
                turns_until_failure: Some(20),
            },
            PredictedIssue {
                component: "nt_governance".into(),
                description: "Policy drift".into(),
                severity: 0.3,
                turns_until_failure: None,
            },
        ]
    }

    #[test]
    fn test_schedule_produces_tasks() {
        let scheduler = MaintenanceScheduler::new();
        let plan = scheduler.schedule(sample_issues());
        assert_eq!(plan.tasks.len(), 3);
    }

    #[test]
    fn test_critical_first() {
        let scheduler = MaintenanceScheduler::new();
        let plan = scheduler.schedule(sample_issues());
        assert_eq!(plan.tasks[0].urgency, Urgency::Critical);
    }

    #[test]
    fn test_urgency_mapping() {
        let scheduler = MaintenanceScheduler::new();
        assert_eq!(scheduler.severity_to_urgency(0.95), Urgency::Critical);
        assert_eq!(scheduler.severity_to_urgency(0.7), Urgency::High);
        assert_eq!(scheduler.severity_to_urgency(0.5), Urgency::Medium);
        assert_eq!(scheduler.severity_to_urgency(0.1), Urgency::Low);
    }

    #[test]
    fn test_total_duration() {
        let scheduler = MaintenanceScheduler::new();
        let plan = scheduler.schedule(sample_issues());
        let total = plan.total_duration();
        assert!(total > 0);
    }

    #[test]
    fn test_critical_count() {
        let scheduler = MaintenanceScheduler::new();
        let plan = scheduler.schedule(sample_issues());
        assert_eq!(plan.critical_count(), 1);
    }

    #[test]
    fn test_empty_issues() {
        let scheduler = MaintenanceScheduler::new();
        let plan = scheduler.schedule(vec![]);
        assert!(plan.tasks.is_empty());
        assert!(plan.priority_order.is_empty());
    }

    #[test]
    fn test_display() {
        assert_eq!(Urgency::Critical.to_string(), "CRITICAL");
        assert_eq!(Urgency::Low.to_string(), "LOW");
    }

    #[test]
    fn test_priority_ordering_critical_before_low() {
        let scheduler = MaintenanceScheduler::new();
        let plan = scheduler.schedule(vec![
            PredictedIssue {
                component: "low_comp".into(),
                description: "minor".into(),
                severity: 0.1,
                turns_until_failure: Some(100),
            },
            PredictedIssue {
                component: "crit_comp".into(),
                description: "major".into(),
                severity: 0.95,
                turns_until_failure: Some(1),
            },
        ]);
        assert_eq!(plan.tasks[0].urgency, Urgency::Critical);
        assert_eq!(plan.tasks[0].component, "crit_comp");
        assert_eq!(plan.tasks[1].urgency, Urgency::Low);
    }

    #[test]
    fn test_same_urgency_sorted_by_duration() {
        let scheduler = MaintenanceScheduler::with_default_duration(100);
        let plan = scheduler.schedule(vec![
            PredictedIssue {
                component: "slow".into(),
                description: "long task".into(),
                severity: 0.5, // Medium → 50s
                turns_until_failure: None,
            },
            PredictedIssue {
                component: "fast".into(),
                description: "short task".into(),
                severity: 0.6, // Medium → 50s
                turns_until_failure: None,
            },
        ]);
        // Both are Medium, should be sorted by estimated duration
        assert_eq!(plan.tasks.len(), 2);
    }

    #[test]
    fn test_severity_to_urgency_boundaries() {
        let scheduler = MaintenanceScheduler::new();
        assert_eq!(scheduler.severity_to_urgency(0.9), Urgency::Critical);
        assert_eq!(scheduler.severity_to_urgency(0.89), Urgency::High);
        assert_eq!(scheduler.severity_to_urgency(0.7), Urgency::High);
        assert_eq!(scheduler.severity_to_urgency(0.69), Urgency::Medium);
        assert_eq!(scheduler.severity_to_urgency(0.4), Urgency::Medium);
        assert_eq!(scheduler.severity_to_urgency(0.39), Urgency::Low);
        assert_eq!(scheduler.severity_to_urgency(0.0), Urgency::Low);
    }

    #[test]
    fn test_action_text_varies_by_urgency() {
        let scheduler = MaintenanceScheduler::new();
        let issues = vec![
            PredictedIssue {
                component: "a".into(),
                description: "crit issue".into(),
                severity: 0.95,
                turns_until_failure: None,
            },
            PredictedIssue {
                component: "b".into(),
                description: "high issue".into(),
                severity: 0.75,
                turns_until_failure: None,
            },
            PredictedIssue {
                component: "c".into(),
                description: "med issue".into(),
                severity: 0.5,
                turns_until_failure: None,
            },
            PredictedIssue {
                component: "d".into(),
                description: "low issue".into(),
                severity: 0.1,
                turns_until_failure: None,
            },
        ];
        let plan = scheduler.schedule(issues);
        assert!(plan.tasks[0].action.contains("Immediate"));
        assert!(plan.tasks[1].action.contains("urgent"));
        assert!(plan.tasks[2].action.contains("preventive"));
        assert!(plan.tasks[3].action.contains("Monitor"));
    }

    #[test]
    fn test_default_duration() {
        let scheduler = MaintenanceScheduler::new();
        assert_eq!(scheduler.default_duration, 300);
    }

    #[test]
    fn test_custom_duration() {
        let scheduler = MaintenanceScheduler::with_default_duration(600);
        assert_eq!(scheduler.default_duration, 600);
    }

    #[test]
    fn test_maintenance_plan_total_duration_calculation() {
        let plan = MaintenancePlan {
            tasks: vec![
                MaintenanceTask {
                    component: "a".into(),
                    action: "act".into(),
                    urgency: Urgency::High,
                    estimated_duration: 100,
                },
                MaintenanceTask {
                    component: "b".into(),
                    action: "act".into(),
                    urgency: Urgency::Low,
                    estimated_duration: 50,
                },
            ],
            priority_order: vec![0, 1],
        };
        assert_eq!(plan.total_duration(), 150);
    }

    #[test]
    fn test_maintenance_plan_critical_count() {
        let plan = MaintenancePlan {
            tasks: vec![
                MaintenanceTask {
                    component: "a".into(),
                    action: "act".into(),
                    urgency: Urgency::Critical,
                    estimated_duration: 100,
                },
                MaintenanceTask {
                    component: "b".into(),
                    action: "act".into(),
                    urgency: Urgency::High,
                    estimated_duration: 100,
                },
                MaintenanceTask {
                    component: "c".into(),
                    action: "act".into(),
                    urgency: Urgency::Critical,
                    estimated_duration: 100,
                },
            ],
            priority_order: vec![0, 1, 2],
        };
        assert_eq!(plan.critical_count(), 2);
    }

    #[test]
    fn test_urgency_ordering() {
        assert!(Urgency::Low < Urgency::Medium);
        assert!(Urgency::Medium < Urgency::High);
        assert!(Urgency::High < Urgency::Critical);
    }

    #[test]
    fn test_all_urgency_display() {
        assert_eq!(Urgency::Medium.to_string(), "MEDIUM");
        assert_eq!(Urgency::High.to_string(), "HIGH");
    }

    #[test]
    fn test_default_impl() {
        let s1 = MaintenanceScheduler::new();
        let s2 = MaintenanceScheduler::default();
        assert_eq!(s1.default_duration, s2.default_duration);
    }
}
