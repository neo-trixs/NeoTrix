//! Coordinator Monitoring
//!
//! Tracks assignment history and computes coordination metrics:
//! total tasks, average wait time, per-agent utilization.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::coordinator::Assignment;
use super::load_balancer::AgentStats;

/// Aggregated coordination metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationMetrics {
    pub total_tasks: u64,
    pub avg_wait_time: f64,
    pub agent_utilization: HashMap<String, f64>,
}

impl CoordinationMetrics {
    /// Create empty metrics
    pub fn new() -> Self {
        Self {
            total_tasks: 0,
            avg_wait_time: 0.0,
            agent_utilization: HashMap::new(),
        }
    }
}

impl Default for CoordinationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks assignment history and computes coordination metrics
pub struct CoordinatorMonitor {
    assignments: Vec<TrackedAssignment>,
}

#[derive(Debug, Clone)]
struct TrackedAssignment {
    assignment: Assignment,
    /// Monotonic sequence number for ordering
    sequence: u64,
}

impl CoordinatorMonitor {
    pub fn new() -> Self {
        Self {
            assignments: Vec::new(),
        }
    }

    /// Record an assignment for tracking
    pub fn track_assignment(&mut self, assignment: Assignment) {
        let sequence = self.assignments.len() as u64;
        self.assignments.push(TrackedAssignment {
            assignment,
            sequence,
        });
    }

    /// Get coordination metrics based on tracked assignments
    pub fn get_metrics(&self) -> CoordinationMetrics {
        let total_tasks = self.assignments.len() as u64;

        // Average wait time = average estimated_duration across all assignments
        let avg_wait_time = if total_tasks == 0 {
            0.0
        } else {
            let total_duration: f64 = self
                .assignments
                .iter()
                .map(|a| a.assignment.estimated_duration)
                .sum();
            total_duration / total_tasks as f64
        };

        // Per-agent utilization: fraction of total tasks assigned to each agent
        let mut agent_counts: HashMap<String, u64> = HashMap::new();
        for tracked in &self.assignments {
            *agent_counts
                .entry(tracked.assignment.agent_id.clone())
                .or_insert(0) += 1;
        }

        let agent_utilization: HashMap<String, f64> = agent_counts
            .into_iter()
            .map(|(id, count)| {
                let utilization = if total_tasks > 0 {
                    count as f64 / total_tasks as f64
                } else {
                    0.0
                };
                (id, utilization)
            })
            .collect();

        CoordinationMetrics {
            total_tasks,
            avg_wait_time,
            agent_utilization,
        }
    }

    /// Get utilization for a specific agent
    pub fn get_agent_utilization(&self, agent_id: &str) -> f64 {
        let total = self.assignments.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        let count = self
            .assignments
            .iter()
            .filter(|a| a.assignment.agent_id == agent_id)
            .count() as f64;
        count / total
    }

    /// Number of tracked assignments
    pub fn len(&self) -> usize {
        self.assignments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assignments.is_empty()
    }

    /// Get all tracked assignments (for inspection)
    pub fn assignments(&self) -> Vec<&Assignment> {
        self.assignments.iter().map(|t| &t.assignment).collect()
    }
}

impl Default for CoordinatorMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::coordinator::{Assignment, MultiAgentCoordinator};
    use super::super::task_routing::{TaskDescription, TaskPriority};
    use super::*;

    fn sample_assignment(agent_id: &str, task_id: &str, duration: f64) -> Assignment {
        Assignment {
            agent_id: agent_id.to_string(),
            task: TaskDescription::new(task_id, vec!["code".into()])
                .with_priority(TaskPriority::Medium),
            estimated_duration: duration,
        }
    }

    #[test]
    fn metrics_empty() {
        let m = CoordinatorMonitor::new();
        let metrics = m.get_metrics();
        assert_eq!(metrics.total_tasks, 0);
        assert_eq!(metrics.avg_wait_time, 0.0);
        assert!(metrics.agent_utilization.is_empty());
    }

    #[test]
    fn track_single_assignment() {
        let mut m = CoordinatorMonitor::new();
        m.track_assignment(sample_assignment("a1", "t1", 2.0));

        let metrics = m.get_metrics();
        assert_eq!(metrics.total_tasks, 1);
        assert!((metrics.avg_wait_time - 2.0).abs() < 0.01);
        assert_eq!(metrics.agent_utilization.get("a1"), Some(&1.0));
    }

    #[test]
    fn track_multiple_assignments() {
        let mut m = CoordinatorMonitor::new();
        m.track_assignment(sample_assignment("a1", "t1", 2.0));
        m.track_assignment(sample_assignment("a1", "t2", 4.0));
        m.track_assignment(sample_assignment("a2", "t3", 6.0));

        let metrics = m.get_metrics();
        assert_eq!(metrics.total_tasks, 3);
        assert!((metrics.avg_wait_time - 4.0).abs() < 0.01);
        assert!((metrics.agent_utilization["a1"] - 2.0 / 3.0).abs() < 0.01);
        assert!((metrics.agent_utilization["a2"] - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn agent_utilization_specific() {
        let mut m = CoordinatorMonitor::new();
        m.track_assignment(sample_assignment("a1", "t1", 1.0));
        m.track_assignment(sample_assignment("a1", "t2", 1.0));
        m.track_assignment(sample_assignment("a2", "t3", 1.0));

        assert!((m.get_agent_utilization("a1") - 2.0 / 3.0).abs() < 0.01);
        assert!((m.get_agent_utilization("a2") - 1.0 / 3.0).abs() < 0.01);
        assert_eq!(m.get_agent_utilization("unknown"), 0.0);
    }

    #[test]
    fn assignments_returns_references() {
        let mut m = CoordinatorMonitor::new();
        m.track_assignment(sample_assignment("a1", "t1", 1.0));
        m.track_assignment(sample_assignment("a2", "t2", 2.0));

        let all = m.assignments();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].agent_id, "a1");
        assert_eq!(all[1].agent_id, "a2");
    }

    #[test]
    fn monitor_len() {
        let mut m = CoordinatorMonitor::new();
        assert!(m.is_empty());
        m.track_assignment(sample_assignment("a1", "t1", 1.0));
        assert_eq!(m.len(), 1);
        assert!(!m.is_empty());
    }

    #[test]
    fn metrics_serialization_roundtrip() {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("a1", vec!["code".into()]);
        let mut m = CoordinatorMonitor::new();
        m.track_assignment(sample_assignment("a1", "t1", 2.0));

        let metrics = m.get_metrics();
        let json = serde_json::to_string(&metrics).unwrap();
        let back: CoordinationMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_tasks, 1);
        assert!((back.avg_wait_time - 2.0).abs() < 0.01);
    }

    #[test]
    fn monitor_default() {
        let m = CoordinatorMonitor::default();
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn agent_utilization_single_agent() {
        let mut m = CoordinatorMonitor::new();
        m.track_assignment(sample_assignment("solo", "t1", 1.0));
        assert_eq!(m.get_agent_utilization("solo"), 1.0);
    }

    #[test]
    fn metrics_empty_after_clear() {
        let mut m = CoordinatorMonitor::new();
        m.track_assignment(sample_assignment("a1", "t1", 1.0));
        assert_eq!(m.get_metrics().total_tasks, 1);
        // Simulate new monitor
        let m2 = CoordinatorMonitor::new();
        assert_eq!(m2.get_metrics().total_tasks, 0);
    }

    #[test]
    fn assignments_returns_all() {
        let mut m = CoordinatorMonitor::new();
        m.track_assignment(sample_assignment("a1", "t1", 1.0));
        m.track_assignment(sample_assignment("a2", "t2", 2.0));
        m.track_assignment(sample_assignment("a3", "t3", 3.0));
        let all = m.assignments();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].agent_id, "a1");
        assert_eq!(all[1].agent_id, "a2");
        assert_eq!(all[2].agent_id, "a3");
    }

    #[test]
    fn metrics_agent_utilization_balanced() {
        let mut m = CoordinatorMonitor::new();
        m.track_assignment(sample_assignment("a1", "t1", 1.0));
        m.track_assignment(sample_assignment("a2", "t2", 1.0));
        m.track_assignment(sample_assignment("a1", "t3", 1.0));
        m.track_assignment(sample_assignment("a2", "t4", 1.0));
        let metrics = m.get_metrics();
        assert!((metrics.agent_utilization["a1"] - 0.5).abs() < 0.01);
        assert!((metrics.agent_utilization["a2"] - 0.5).abs() < 0.01);
    }

    #[test]
    fn coordination_metrics_default() {
        let m = CoordinationMetrics::default();
        assert_eq!(m.total_tasks, 0);
        assert!(m.agent_utilization.is_empty());
    }
}
