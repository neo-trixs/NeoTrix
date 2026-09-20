//! DAG Monitor — tracks execution status of DAG nodes at runtime.
//!
//! Provides real-time status tracking per node and aggregated DAG status.
//! Follows R-P125 (typed interfaces) and R-P126 (observability).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of a single node during execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Not yet started
    Pending,
    /// Currently executing
    Running,
    /// Completed successfully
    Complete,
    /// Failed during execution
    Failed,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeStatus::Pending => write!(f, "Pending"),
            NodeStatus::Running => write!(f, "Running"),
            NodeStatus::Complete => write!(f, "Complete"),
            NodeStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// Aggregated status of an entire DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagStatus {
    /// Total number of nodes in the DAG
    pub total_nodes: usize,
    /// Nodes that have completed successfully
    pub completed: usize,
    /// Nodes that have failed
    pub failed: usize,
    /// Nodes currently executing
    pub in_progress: usize,
    /// Nodes still pending
    pub pending: usize,
}

impl DagStatus {
    /// True if all nodes are complete (no failures)
    pub fn is_complete(&self) -> bool {
        self.completed == self.total_nodes && self.failed == 0
    }

    /// True if any node has failed
    pub fn has_failure(&self) -> bool {
        self.failed > 0
    }

    /// Completion percentage (0.0 - 100.0)
    pub fn completion_pct(&self) -> f64 {
        if self.total_nodes == 0 {
            return 100.0;
        }
        (self.completed as f64 / self.total_nodes as f64) * 100.0
    }
}

/// Monitors execution state of a DAG, tracking per-node status.
///
/// Thread-safe via interior mutability (all state is in the HashMap).
pub struct DagMonitor {
    /// DAG id this monitor is watching
    pub dag_id: String,
    /// Per-node status
    node_status: HashMap<String, NodeStatus>,
    /// History of status transitions for audit
    history: Vec<StatusTransition>,
}

/// A recorded status transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusTransition {
    pub node_id: String,
    pub from: NodeStatus,
    pub to: NodeStatus,
    pub timestamp: i64,
}

impl DagMonitor {
    /// Create a new monitor for a DAG, initializing all nodes to Pending
    pub fn new(dag_id: impl Into<String>, node_ids: &[String]) -> Self {
        let mut node_status = HashMap::new();
        for id in node_ids {
            node_status.insert(id.clone(), NodeStatus::Pending);
        }
        Self {
            dag_id: dag_id.into(),
            node_status,
            history: Vec::new(),
        }
    }

    /// Track a status change for a node
    pub fn track_execution(&mut self, node_id: &str, status: NodeStatus) -> Result<(), String> {
        let current = self
            .node_status
            .get(node_id)
            .ok_or_else(|| format!("unknown node: {}", node_id))?;

        if *current == status {
            return Ok(()); // no-op transition
        }

        let transition = StatusTransition {
            node_id: node_id.to_string(),
            from: *current,
            to: status,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.node_status.insert(node_id.to_string(), status);
        self.history.push(transition);
        Ok(())
    }

    /// Get the status of a specific node
    pub fn get_node_status(&self, node_id: &str) -> Option<NodeStatus> {
        self.node_status.get(node_id).copied()
    }

    /// Get aggregated DAG status
    pub fn get_dag_status(&self) -> DagStatus {
        let total_nodes = self.node_status.len();
        let completed = self
            .node_status
            .values()
            .filter(|s| **s == NodeStatus::Complete)
            .count();
        let failed = self
            .node_status
            .values()
            .filter(|s| **s == NodeStatus::Failed)
            .count();
        let in_progress = self
            .node_status
            .values()
            .filter(|s| **s == NodeStatus::Running)
            .count();
        let pending = self
            .node_status
            .values()
            .filter(|s| **s == NodeStatus::Pending)
            .count();

        DagStatus {
            total_nodes,
            completed,
            failed,
            in_progress,
            pending,
        }
    }

    /// Get the full transition history
    pub fn history(&self) -> &[StatusTransition] {
        &self.history
    }

    /// Get all node ids being monitored
    pub fn node_ids(&self) -> Vec<&str> {
        self.node_status.keys().map(|s| s.as_str()).collect()
    }

    /// Get list of failed node ids
    pub fn failed_nodes(&self) -> Vec<&str> {
        self.node_status
            .iter()
            .filter(|(_, s)| **s == NodeStatus::Failed)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Get list of running node ids
    pub fn running_nodes(&self) -> Vec<&str> {
        self.node_status
            .iter()
            .filter(|(_, s)| **s == NodeStatus::Running)
            .map(|(id, _)| id.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_monitor() -> DagMonitor {
        let ids: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        DagMonitor::new("dag-1", &ids)
    }

    #[test]
    fn initial_status_all_pending() {
        let mon = test_monitor();
        let status = mon.get_dag_status();
        assert_eq!(status.total_nodes, 3);
        assert_eq!(status.pending, 3);
        assert_eq!(status.completed, 0);
        assert!(!status.is_complete());
    }

    #[test]
    fn track_execution_basic() {
        let mut mon = test_monitor();
        mon.track_execution("a", NodeStatus::Running).unwrap();
        assert_eq!(mon.get_node_status("a"), Some(NodeStatus::Running));

        mon.track_execution("a", NodeStatus::Complete).unwrap();
        assert_eq!(mon.get_node_status("a"), Some(NodeStatus::Complete));

        let status = mon.get_dag_status();
        assert_eq!(status.completed, 1);
        assert_eq!(status.pending, 2);
    }

    #[test]
    fn track_execution_failure() {
        let mut mon = test_monitor();
        mon.track_execution("b", NodeStatus::Running).unwrap();
        mon.track_execution("b", NodeStatus::Failed).unwrap();

        assert_eq!(mon.get_node_status("b"), Some(NodeStatus::Failed));
        let status = mon.get_dag_status();
        assert!(status.has_failure());
        assert_eq!(status.failed, 1);
    }

    #[test]
    fn track_execution_unknown_node_rejects() {
        let mut mon = test_monitor();
        let err = mon.track_execution("missing", NodeStatus::Running);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("unknown node"));
    }

    #[test]
    fn same_status_noop() {
        let mut mon = test_monitor();
        mon.track_execution("a", NodeStatus::Pending).unwrap(); // no-op
        assert!(mon.history().is_empty());
    }

    #[test]
    fn history_tracking() {
        let mut mon = test_monitor();
        mon.track_execution("a", NodeStatus::Running).unwrap();
        mon.track_execution("a", NodeStatus::Complete).unwrap();
        assert_eq!(mon.history().len(), 2);
        assert_eq!(mon.history()[0].from, NodeStatus::Pending);
        assert_eq!(mon.history()[0].to, NodeStatus::Running);
        assert_eq!(mon.history()[1].from, NodeStatus::Running);
        assert_eq!(mon.history()[1].to, NodeStatus::Complete);
    }

    #[test]
    fn dag_status_completion_pct() {
        let mut mon = test_monitor();
        mon.track_execution("a", NodeStatus::Complete).unwrap();
        mon.track_execution("b", NodeStatus::Complete).unwrap();
        let status = mon.get_dag_status();
        assert!((status.completion_pct() - 66.666).abs() < 0.1);
    }

    #[test]
    fn dag_status_empty() {
        let mon = DagMonitor::new("empty", &[]);
        let status = mon.get_dag_status();
        assert!(status.is_complete());
        assert_eq!(status.completion_pct(), 100.0);
    }

    #[test]
    fn failed_and_running_nodes() {
        let mut mon = test_monitor();
        mon.track_execution("a", NodeStatus::Running).unwrap();
        mon.track_execution("b", NodeStatus::Failed).unwrap();

        assert_eq!(mon.running_nodes(), vec!["a"]);
        assert_eq!(mon.failed_nodes(), vec!["b"]);
    }

    #[test]
    fn node_ids_list() {
        let mon = test_monitor();
        let mut ids = mon.node_ids();
        ids.sort();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn dag_status_display() {
        let status = DagStatus {
            total_nodes: 5,
            completed: 3,
            failed: 1,
            in_progress: 1,
            pending: 0,
        };
        assert!(!status.is_complete());
        assert!(status.has_failure());
    }

    #[test]
    fn node_status_display() {
        assert_eq!(NodeStatus::Pending.to_string(), "Pending");
        assert_eq!(NodeStatus::Running.to_string(), "Running");
        assert_eq!(NodeStatus::Complete.to_string(), "Complete");
        assert_eq!(NodeStatus::Failed.to_string(), "Failed");
    }

    #[test]
    fn monitor_serialization_roundtrip() {
        let mut mon = test_monitor();
        mon.track_execution("a", NodeStatus::Running).unwrap();
        let json = serde_json::to_string(&mon).unwrap();
        let back: DagMonitor = serde_json::from_str(&json).unwrap();
        assert_eq!(back.dag_id, "dag-1");
        assert_eq!(back.history().len(), 1);
    }

    #[test]
    fn track_full_lifecycle() {
        let mut mon = test_monitor();
        mon.track_execution("a", NodeStatus::Running).unwrap();
        mon.track_execution("a", NodeStatus::Complete).unwrap();
        mon.track_execution("b", NodeStatus::Running).unwrap();
        mon.track_execution("b", NodeStatus::Failed).unwrap();
        mon.track_execution("c", NodeStatus::Running).unwrap();
        mon.track_execution("c", NodeStatus::Complete).unwrap();

        let status = mon.get_dag_status();
        assert_eq!(status.completed, 2);
        assert_eq!(status.failed, 1);
        assert_eq!(status.pending, 0);
        assert!(!status.is_complete());
    }

    #[test]
    fn dag_status_completion_pct_full() {
        let mut mon = test_monitor();
        mon.track_execution("a", NodeStatus::Complete).unwrap();
        mon.track_execution("b", NodeStatus::Complete).unwrap();
        mon.track_execution("c", NodeStatus::Complete).unwrap();
        let status = mon.get_dag_status();
        assert!((status.completion_pct() - 100.0).abs() < 0.01);
        assert!(status.is_complete());
    }

    #[test]
    fn failed_nodes_multiple() {
        let mut mon = test_monitor();
        mon.track_execution("a", NodeStatus::Failed).unwrap();
        mon.track_execution("b", NodeStatus::Failed).unwrap();
        mon.track_execution("c", NodeStatus::Complete).unwrap();
        let mut failed = mon.failed_nodes();
        failed.sort();
        assert_eq!(failed, vec!["a", "b"]);
    }

    #[test]
    fn running_nodes_empty() {
        let mon = test_monitor();
        assert!(mon.running_nodes().is_empty());
    }

    #[test]
    fn history_records_transitions() {
        let mut mon = test_monitor();
        mon.track_execution("a", NodeStatus::Running).unwrap();
        mon.track_execution("a", NodeStatus::Failed).unwrap();
        mon.track_execution("a", NodeStatus::Running).unwrap();
        mon.track_execution("a", NodeStatus::Complete).unwrap();
        assert_eq!(mon.history().len(), 4);
        assert_eq!(mon.history()[0].from, NodeStatus::Pending);
        assert_eq!(mon.history()[3].to, NodeStatus::Complete);
    }

    #[test]
    fn status_transition_serialization() {
        let t = StatusTransition {
            node_id: "n1".into(),
            from: NodeStatus::Pending,
            to: NodeStatus::Running,
            timestamp: 12345,
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: StatusTransition = serde_json::from_str(&json).unwrap();
        assert_eq!(back.from, NodeStatus::Pending);
        assert_eq!(back.to, NodeStatus::Running);
    }

    #[test]
    fn dag_status_serialization() {
        let status = DagStatus {
            total_nodes: 5,
            completed: 3,
            failed: 1,
            in_progress: 1,
            pending: 0,
        };
        let json = serde_json::to_string(&status).unwrap();
        let back: DagStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_nodes, 5);
        assert!(!back.is_complete());
    }
}
