//! Task Description and Priority-Based Routing
//!
//! Defines task descriptors with capability requirements and priority levels,
//! plus a router that assigns tasks to agents using priority-queue ordering.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use serde::{Deserialize, Serialize};

use super::coordinator::{AgentEntry, Assignment};

/// Priority level for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl TaskPriority {
    /// Numeric weight for scoring (higher = more urgent)
    pub fn weight(&self) -> u32 {
        match self {
            TaskPriority::Low => 1,
            TaskPriority::Medium => 2,
            TaskPriority::High => 4,
            TaskPriority::Critical => 8,
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Critical => write!(f, "Critical"),
        }
    }
}

/// Description of a task to be routed to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDescription {
    /// Unique task identifier
    pub id: String,
    /// Capabilities required to execute this task
    pub required_capabilities: Vec<String>,
    /// Task priority
    pub priority: TaskPriority,
    /// Optional deadline as Unix timestamp (None = no deadline)
    pub deadline: Option<i64>,
}

impl TaskDescription {
    /// Create a new task with the given ID and required capabilities
    pub fn new(id: impl Into<String>, required_capabilities: Vec<String>) -> Self {
        Self {
            id: id.into(),
            required_capabilities,
            priority: TaskPriority::Medium,
            deadline: None,
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_deadline(mut self, deadline: i64) -> Self {
        self.deadline = Some(deadline);
        self
    }
}

/// Wrapper for priority-queue ordering (max-heap by priority weight)
struct PrioritizedTask {
    task: TaskDescription,
    priority_weight: u32,
}

impl PartialEq for PrioritizedTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority_weight == other.priority_weight
    }
}

impl Eq for PrioritizedTask {}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority_weight.cmp(&other.priority_weight)
    }
}

/// Task router using priority-queue based assignment
pub struct TaskRouter;

impl TaskRouter {
    /// Route a batch of tasks to registered agents.
    ///
    /// Tasks are dequeued in priority order (Critical first). Each task is
    /// assigned to the best-matching agent based on capability overlap score.
    /// Returns `Assignment` for each successfully routed task.
    pub fn route(tasks: &[TaskDescription], agents: &[AgentEntry]) -> Vec<Assignment> {
        if tasks.is_empty() || agents.is_empty() {
            return Vec::new();
        }

        // Build priority queue
        let mut heap: BinaryHeap<PrioritizedTask> = tasks
            .iter()
            .map(|t| PrioritizedTask {
                task: t.clone(),
                priority_weight: t.priority.weight(),
            })
            .collect();

        let mut assignments = Vec::new();
        let mut agent_loads: Vec<(String, u32)> =
            agents.iter().map(|a| (a.agent_id.clone(), 0u32)).collect();

        while let Some(PrioritizedTask { task, .. }) = heap.pop() {
            if let Some(best_agent) = Self::find_best_agent(&task, agents, &agent_loads) {
                let estimated_duration = Self::estimate_duration(&task, &best_agent);
                let agent_id = best_agent.agent_id.clone();
                assignments.push(Assignment {
                    agent_id: agent_id.clone(),
                    task,
                    estimated_duration,
                });

                // Update load counter for this agent
                if let Some((_, load)) = agent_loads.iter_mut().find(|(id, _)| *id == agent_id) {
                    *load += 1;
                }
            }
        }

        assignments
    }

    /// Find the best agent for a task based on capability overlap score,
    /// breaking ties by current load (lower load preferred).
    fn find_best_agent<'a>(
        task: &TaskDescription,
        agents: &'a [AgentEntry],
        agent_loads: &[(String, u32)],
    ) -> Option<&'a AgentEntry> {
        let mut best: Option<(&AgentEntry, f64, u32)> = None;

        for agent in agents {
            let overlap =
                Self::capability_overlap(&task.required_capabilities, &agent.capabilities);
            if overlap <= 0.0 {
                continue;
            }

            let load = agent_loads
                .iter()
                .find(|(id, _)| *id == agent.agent_id)
                .map(|(_, l)| *l)
                .unwrap_or(0);

            match best {
                None => best = Some((agent, overlap, load)),
                Some((_, best_overlap, best_load)) => {
                    // Prefer higher overlap; break ties with lower load
                    if overlap > best_overlap || (overlap == best_overlap && load < best_load) {
                        best = Some((agent, overlap, load));
                    }
                }
            }
        }

        best.map(|(agent, _, _)| agent)
    }

    /// Compute overlap score between required capabilities and agent capabilities.
    ///
    /// Returns the fraction of required capabilities the agent covers (0.0–1.0).
    /// If the task requires no capabilities, returns 1.0.
    pub fn capability_overlap(required: &[String], agent_caps: &[String]) -> f64 {
        if required.is_empty() {
            return 1.0;
        }
        let matched = required
            .iter()
            .filter(|req| agent_caps.iter().any(|ac| ac == req.as_str()))
            .count();
        matched as f64 / required.len() as f64
    }

    /// Estimate task duration based on task complexity and agent capabilities.
    fn estimate_duration(task: &TaskDescription, agent: &AgentEntry) -> f64 {
        let base_duration = 1.0; // seconds baseline
        let cap_factor = Self::capability_overlap(&task.required_capabilities, &agent.capabilities);
        let priority_factor = 1.0 / task.priority.weight() as f64;
        base_duration * (1.0 + (1.0 - cap_factor)) * (1.0 + priority_factor)
    }
}

#[cfg(test)]
mod tests {
    use super::super::coordinator::MultiAgentCoordinator;
    use super::*;

    fn make_coordinator_with_agents() -> MultiAgentCoordinator {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("agent-a", vec!["code".into(), "review".into()]);
        coord.register_agent("agent-b", vec!["research".into(), "code".into()]);
        coord.register_agent("agent-c", vec!["review".into(), "test".into()]);
        coord
    }

    #[test]
    fn task_description_builder() {
        let task = TaskDescription::new("t1", vec!["code".into()])
            .with_priority(TaskPriority::High)
            .with_deadline(1000);
        assert_eq!(task.id, "t1");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.deadline, Some(1000));
    }

    #[test]
    fn priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Medium);
        assert!(TaskPriority::Medium > TaskPriority::Low);
    }

    #[test]
    fn priority_weight() {
        assert_eq!(TaskPriority::Critical.weight(), 8);
        assert_eq!(TaskPriority::High.weight(), 4);
        assert_eq!(TaskPriority::Medium.weight(), 2);
        assert_eq!(TaskPriority::Low.weight(), 1);
    }

    #[test]
    fn capability_overlap_full() {
        let req = vec!["code".into(), "review".into()];
        let caps = vec!["code".into(), "review".into(), "test".into()];
        assert!((TaskRouter::capability_overlap(&req, &caps) - 1.0).abs() < 0.01);
    }

    #[test]
    fn capability_overlap_partial() {
        let req = vec!["code".into(), "review".into()];
        let caps = vec!["code".into()];
        assert!((TaskRouter::capability_overlap(&req, &caps) - 0.5).abs() < 0.01);
    }

    #[test]
    fn capability_overlap_none() {
        let req = vec!["code".into()];
        let caps = vec!["review".into()];
        assert!((TaskRouter::capability_overlap(&req, &caps)).abs() < 0.01);
    }

    #[test]
    fn capability_overlap_empty_required() {
        let req: Vec<String> = vec![];
        let caps = vec!["code".into()];
        assert!((TaskRouter::capability_overlap(&req, &caps) - 1.0).abs() < 0.01);
    }

    #[test]
    fn route_priority_order() {
        let coord = make_coordinator_with_agents();
        let agents = coord.get_agents();

        let tasks = vec![
            TaskDescription::new("low-task", vec!["code".into()]).with_priority(TaskPriority::Low),
            TaskDescription::new("critical-task", vec!["code".into()])
                .with_priority(TaskPriority::Critical),
            TaskDescription::new("medium-task", vec!["code".into()])
                .with_priority(TaskPriority::Medium),
        ];

        let assignments = TaskRouter::route(&tasks, agents);
        assert_eq!(assignments.len(), 3);
        // Critical should be routed first
        assert_eq!(assignments[0].task.id, "critical-task");
    }

    #[test]
    fn route_skips_unmatched() {
        let coord = make_coordinator_with_agents();
        let agents = coord.get_agents();

        // "deploy" is not in any agent's capabilities
        let tasks = vec![TaskDescription::new("deploy-task", vec!["deploy".into()])];
        let assignments = TaskRouter::route(&tasks, agents);
        assert!(assignments.is_empty());
    }

    #[test]
    fn route_empty_inputs() {
        assert!(TaskRouter::route(&[], &[]).is_empty());
        let tasks = vec![TaskDescription::new("t1", vec![])];
        assert!(TaskRouter::route(&tasks, &[]).is_empty());
    }

    #[test]
    fn priority_display() {
        assert_eq!(TaskPriority::Low.to_string(), "Low");
        assert_eq!(TaskPriority::Critical.to_string(), "Critical");
    }

    #[test]
    fn task_description_serialization_roundtrip() {
        let task = TaskDescription::new("t1", vec!["code".into()])
            .with_priority(TaskPriority::High)
            .with_deadline(9999);
        let json = serde_json::to_string(&task).unwrap();
        let back: TaskDescription = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "t1");
        assert_eq!(back.priority, TaskPriority::High);
        assert_eq!(back.deadline, Some(9999));
    }

    #[test]
    fn route_equal_priority_load_balances() {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("a1", vec!["code".into()]);
        coord.register_agent("a2", vec!["code".into()]);
        let agents = coord.get_agents();

        let tasks: Vec<TaskDescription> = (0..4)
            .map(|i| TaskDescription::new(format!("t{}", i), vec!["code".into()]))
            .collect();

        let assignments = TaskRouter::route(&tasks, agents);
        assert_eq!(assignments.len(), 4);
        // Both agents should get tasks due to load balancing
        let a1_count = assignments.iter().filter(|a| a.agent_id == "a1").count();
        let a2_count = assignments.iter().filter(|a| a.agent_id == "a2").count();
        assert!(a1_count >= 1 && a2_count >= 1);
    }

    #[test]
    fn route_single_task_single_agent() {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("solo", vec!["deploy".into()]);
        let agents = coord.get_agents();
        let tasks = vec![TaskDescription::new("t1", vec!["deploy".into()])];
        let assignments = TaskRouter::route(&tasks, agents);
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].agent_id, "solo");
    }

    #[test]
    fn capability_overlap_empty_agent_caps() {
        let req = vec!["code".into()];
        let caps: Vec<String> = vec![];
        assert!((TaskRouter::capability_overlap(&req, &caps)).abs() < 0.01);
    }

    #[test]
    fn capability_overlap_both_empty() {
        let req: Vec<String> = vec![];
        let caps: Vec<String> = vec![];
        assert!((TaskRouter::capability_overlap(&req, &caps) - 1.0).abs() < 0.01);
    }

    #[test]
    fn route_critical_before_low() {
        let mut coord = MultiAgentCoordinator::new();
        coord.register_agent("a1", vec!["code".into()]);
        let agents = coord.get_agents();

        let tasks = vec![
            TaskDescription::new("low", vec!["code".into()]).with_priority(TaskPriority::Low),
            TaskDescription::new("critical", vec!["code".into()])
                .with_priority(TaskPriority::Critical),
        ];
        let assignments = TaskRouter::route(&tasks, agents);
        assert_eq!(assignments[0].task.id, "critical");
    }
}
