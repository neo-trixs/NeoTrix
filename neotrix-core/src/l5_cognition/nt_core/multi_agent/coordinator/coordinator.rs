//! Multi-Agent Coordinator
//!
//! Central coordinator that manages agent registration, capability-based
//! task assignment, and load-aware dispatch. Follows the R-P125 pattern
//! for type-safe agent interfaces.

#![forbid(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::load_balancer::AgentLoadBalancer;
use super::monitor::CoordinatorMonitor;
use super::task_routing::{TaskDescription, TaskRouter};

/// A registered agent entry with its capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEntry {
    pub agent_id: String,
    pub capabilities: Vec<String>,
}

impl AgentEntry {
    pub fn new(agent_id: impl Into<String>, capabilities: Vec<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            capabilities,
        }
    }
}

/// Result of assigning a task to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub agent_id: String,
    pub task: TaskDescription,
    pub estimated_duration: f64,
}

/// Central multi-agent coordinator
///
/// Manages agent registration, capability-based task assignment,
/// load balancing, and assignment monitoring.
pub struct MultiAgentCoordinator {
    agents: Vec<AgentEntry>,
    agent_map: HashMap<String, usize>,
    load_balancer: AgentLoadBalancer,
    monitor: CoordinatorMonitor,
}

impl MultiAgentCoordinator {
    /// Create a new coordinator
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            agent_map: HashMap::new(),
            load_balancer: AgentLoadBalancer::new(),
            monitor: CoordinatorMonitor::new(),
        }
    }

    /// Register an agent with its capabilities
    pub fn register_agent(&mut self, agent_id: &str, capabilities: Vec<String>) {
        if self.agent_map.contains_key(agent_id) {
            return;
        }
        let idx = self.agents.len();
        self.agents.push(AgentEntry::new(agent_id, capabilities));
        self.agent_map.insert(agent_id.to_string(), idx);
        self.load_balancer.register_agent(agent_id);
    }

    /// Remove a registered agent
    pub fn unregister_agent(&mut self, agent_id: &str) {
        if let Some(_idx) = self.agent_map.remove(agent_id) {
            self.agents.retain(|a| a.agent_id != agent_id);
            // Rebuild index map
            self.agent_map.clear();
            for (i, agent) in self.agents.iter().enumerate() {
                self.agent_map.insert(agent.agent_id.clone(), i);
            }
            self.load_balancer.unregister_agent(agent_id);
        }
    }

    /// Assign a single task to the best-matching agent.
    ///
    /// Uses capability overlap scoring with load-aware tiebreaking.
    pub fn assign_task(&mut self, task: TaskDescription) -> Option<Assignment> {
        let agents = self.agents.clone();
        let assignments = TaskRouter::route(&[task], &agents);

        if let Some(mut assignment) = assignments.into_iter().next() {
            // Adjust duration based on load
            let load = self.load_balancer.get_load(&assignment.agent_id);
            assignment.estimated_duration *= 1.0 + load;

            self.load_balancer.task_started(&assignment.agent_id);
            self.monitor.track_assignment(assignment.clone());
            Some(assignment)
        } else {
            None
        }
    }

    /// Assign multiple tasks, returning all successful assignments
    pub fn assign_tasks(&mut self, tasks: Vec<TaskDescription>) -> Vec<Assignment> {
        let agents = self.agents.clone();
        let mut assignments = TaskRouter::route(&tasks, &agents);

        for assignment in &mut assignments {
            let load = self.load_balancer.get_load(&assignment.agent_id);
            assignment.estimated_duration *= 1.0 + load;

            self.load_balancer.task_started(&assignment.agent_id);
            self.monitor.track_assignment(assignment.clone());
        }

        assignments
    }

    /// Mark a task as completed (updates load balancer stats)
    pub fn complete_task(&mut self, agent_id: &str, duration: f64) {
        self.load_balancer.task_completed(agent_id, duration);
    }

    /// Get load for a specific agent (0.0–1.0)
    pub fn get_agent_load(&self, agent_id: &str) -> f64 {
        self.load_balancer.get_load(agent_id)
    }

    /// Rebalance: return agent IDs sorted by load (lowest first)
    pub fn rebalance(&self) -> Vec<String> {
        self.load_balancer.rebalance()
    }

    /// Get agent stats from the load balancer
    pub fn get_agent_stats(&self) -> Vec<super::load_balancer::AgentStats> {
        self.load_balancer.get_agent_stats()
    }

    /// Get coordination metrics from the monitor
    pub fn get_metrics(&self) -> super::monitor::CoordinationMetrics {
        self.monitor.get_metrics()
    }

    /// Get references to registered agents
    pub fn get_agents(&self) -> &[AgentEntry] {
        &self.agents
    }

    /// Number of registered agents
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

impl Default for MultiAgentCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::task_routing::TaskPriority;
    use super::*;

    fn make_coordinator() -> MultiAgentCoordinator {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("coder", vec!["code".into(), "review".into()]);
        coord.register_agent("researcher", vec!["research".into(), "code".into()]);
        coord
    }

    #[test]
    fn new_coordinator_is_empty() {
        let coord = MultiAgentCoordinator::new();
        assert_eq!(coord.agent_count(), 0);
    }

    #[test]
    fn register_agent() {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("a1", vec!["code".into()]);
        assert_eq!(coord.agent_count(), 1);
    }

    #[test]
    fn register_agent_dedup() {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("a1", vec!["code".into()]);
        coord.register_agent("a1", vec!["review".into()]);
        assert_eq!(coord.agent_count(), 1);
    }

    #[test]
    fn unregister_agent() {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("a1", vec!["code".into()]);
        coord.register_agent("a2", vec!["review".into()]);
        coord.unregister_agent("a1");
        assert_eq!(coord.agent_count(), 1);
    }

    #[test]
    fn assign_task_to_matching_agent() {
        let mut coord = make_coordinator();
        let task =
            TaskDescription::new("t1", vec!["code".into()]).with_priority(TaskPriority::High);
        let assignment = coord.assign_task(task);
        assert!(assignment.is_some());
        let a = assignment.unwrap();
        assert_eq!(a.task.id, "t1");
        assert!(!a.agent_id.is_empty());
    }

    #[test]
    fn assign_task_no_match() {
        let mut coord = make_coordinator();
        let task = TaskDescription::new("t1", vec!["deploy".into()]);
        let assignment = coord.assign_task(task);
        assert!(assignment.is_none());
    }

    #[test]
    fn assign_tasks_batch() {
        let mut coord = make_coordinator();
        let tasks = vec![
            TaskDescription::new("t1", vec!["code".into()]),
            TaskDescription::new("t2", vec!["research".into()]),
        ];
        let assignments = coord.assign_tasks(tasks);
        assert_eq!(assignments.len(), 2);
    }

    /// ⛔ **原断言失败的原因是测试的夹具假设错误，不是产品缺陷**（2026-10-02 查清）。
    ///
    /// 探针实测（临时 test + `--nocapture`）：
    /// ```
    /// assign_task -> true
    /// get_agent_load(a1) = 0.1      ← 正是 > 0.0
    /// ```
    /// 差异在于探针**只注册了 a1**，而 `make_coordinator()` 已注册：
    /// `"coder"["code","review"]` 与 `"researcher"["research","code"]`
    /// ⇒ **三个 agent 都能接 "code"**。
    /// `find_best_agent` 的平局规则是 `load < best_load`（**严格小于**）
    /// ⇒ 平局**保留先注册者** ⇒ `coder` 胜出 ⇒ `a1` 从未接到任务
    /// ⇒ `task_started("a1")` 从未被调用 ⇒ load 0.0。
    /// ⇒ **产品行为正确**（平局偏向先注册者是合理且稳定的策略）。
    ///
    /// 修法：给 a1 一个**只有它有**的能力，使测试不再依赖平局结果。
    #[test]
    fn complete_task_updates_load() {
        let mut coord = make_coordinator();
        // ⛔ 能力必须是 **a1 独有**：`make_coordinator()` 里 coder/researcher
        // 都带 "code"，原测试用 "code" 会命中平局分支（见上方说明）。
        coord.register_agent("a1", vec!["rust".into()]);

        let task = TaskDescription::new("t1", vec!["rust".into()]);
        let assigned = coord.assign_task(task).expect("a1 独有 rust，应接到任务");
        assert_eq!(assigned.agent_id, "a1");

        // Agent should now have some load
        assert!(
            coord.get_agent_load("a1") > 0.0,
            "实际 load = {}",
            coord.get_agent_load("a1")
        );

        // Complete the task
        coord.complete_task("a1", 3.0);
        let stats = coord.get_agent_stats();
        let a1_stats = stats.iter().find(|s| s.agent_id == "a1").unwrap();
        assert_eq!(a1_stats.completed_tasks, 1);
    }

    #[test]
    fn rebalance_returns_agents() {
        let coord = make_coordinator();
        let order = coord.rebalance();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn get_metrics_after_assignment() {
        let mut coord = make_coordinator();
        let task = TaskDescription::new("t1", vec!["code".into()]);
        coord.assign_task(task);

        let metrics = coord.get_metrics();
        assert_eq!(metrics.total_tasks, 1);
        assert!(metrics.avg_wait_time > 0.0);
    }

    #[test]
    fn default_is_empty() {
        let coord = MultiAgentCoordinator::default();
        assert_eq!(coord.agent_count(), 0);
    }

    #[test]
    fn agent_entry_builder() {
        let entry = AgentEntry::new("a1", vec!["code".into()]);
        assert_eq!(entry.agent_id, "a1");
        assert_eq!(entry.capabilities, vec!["code".to_string()]);
    }

    #[test]
    fn assignment_serialization_roundtrip() {
        let task = TaskDescription::new("t1", vec!["code".into()]);
        let assignment = Assignment {
            agent_id: "a1".into(),
            task,
            estimated_duration: 2.5,
        };
        let json = serde_json::to_string(&assignment).unwrap();
        let back: Assignment = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, "a1");
        assert!((back.estimated_duration - 2.5).abs() < 0.01);
    }

    #[test]
    fn register_then_unregister_preserves_integrity() {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("a1", vec!["code".into()]);
        coord.register_agent("a2", vec!["code".into()]);
        coord.register_agent("a3", vec!["code".into()]);
        coord.unregister_agent("a2");
        assert_eq!(coord.agent_count(), 2);
        // Remaining agents should still be accessible
        let ids: Vec<&str> = coord.get_agents().iter().map(|a| a.agent_id.as_str()).collect();
        assert!(ids.contains(&"a1"));
        assert!(ids.contains(&"a3"));
        assert!(!ids.contains(&"a2"));
    }

    #[test]
    fn unregister_nonexistent_agent() {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("a1", vec!["code".into()]);
        coord.unregister_agent("ghost");
        assert_eq!(coord.agent_count(), 1);
    }

    #[test]
    fn assign_task_no_agents() {
        let mut coord = MultiAgentCoordinator::new();
        let task = TaskDescription::new("t1", vec!["code".into()]);
        assert!(coord.assign_task(task).is_none());
    }

    #[test]
    fn assign_task_load_increases_duration() {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("a1", vec!["code".into()]);

        // First task
        let task1 = TaskDescription::new("t1", vec!["code".into()]);
        let a1 = coord.assign_task(task1).unwrap();
        let base_duration = a1.estimated_duration;

        // Second task on same agent — load should increase duration
        let task2 = TaskDescription::new("t2", vec!["code".into()]);
        let a2 = coord.assign_task(task2).unwrap();
        assert!(a2.estimated_duration >= base_duration);
    }

    #[test]
    fn get_agent_load_unknown_agent() {
        let coord = MultiAgentCoordinator::new();
        assert_eq!(coord.get_agent_load("unknown"), 0.0);
    }

    #[test]
    fn rebalance_empty_coordinator() {
        let coord = MultiAgentCoordinator::new();
        assert!(coord.rebalance().is_empty());
    }

    #[test]
    fn get_metrics_empty() {
        let coord = MultiAgentCoordinator::new();
        let metrics = coord.get_metrics();
        assert_eq!(metrics.total_tasks, 0);
        assert_eq!(metrics.avg_wait_time, 0.0);
    }

    #[test]
    fn agent_entry_serialization() {
        let entry = AgentEntry::new("a1", vec!["code".into(), "review".into()]);
        let json = serde_json::to_string(&entry).unwrap();
        let back: AgentEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, "a1");
        assert_eq!(back.capabilities.len(), 2);
    }

    #[test]
    fn complete_task_multiple_times() {
        let mut coord = make_coordinator();
        coord.register_agent("a1", vec!["code".into()]);

        coord.assign_task(TaskDescription::new("t1", vec!["code".into()]));
        coord.assign_task(TaskDescription::new("t2", vec!["code".into()]));

        coord.complete_task("a1", 2.0);
        coord.complete_task("a1", 4.0);

        let stats = coord.get_agent_stats();
        let a1 = stats.iter().find(|s| s.agent_id == "a1").unwrap();
        assert_eq!(a1.completed_tasks, 2);
        // Rolling average: (2+4)/2 = 3.0
        assert!((a1.avg_duration - 3.0).abs() < 0.01);
    }

    #[test]
    fn assign_tasks_empty_list() {
        let mut coord = make_coordinator();
        let assignments = coord.assign_tasks(vec![]);
        assert!(assignments.is_empty());
    }
}
