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
    /// 路由任务到 agent。
    ///
    /// ⚠️ 2026-10-02：本函数是**纯函数** —— 它只能看到本批内部的分配计数，
    /// **看不到 `LoadBalancer` 里跨批次的真实负载**（`agent_loads` 全初始为 0）。
    /// 实测（探针，2 个同能力 agent，连续 4 次 `assign_task`）：
    ///     批次0: a1 | a1=0.10 a2=0.00
    ///     批次1: a1 | a1=0.20 a2=0.00
    ///     批次2: a1 | a1=0.30 a2=0.00
    ///     批次3: a1 | a1=0.40 a2=0.00
    /// ⇒ **a2 全程空闲** ⇒ 跨批次负载均衡完全失效。
    ///
    /// ⛔ 因此**本函数保持原签名**（9 个调用点，含 7 处测试，牵一发动全身），
    ///    改为新增 [`Self::route_with_initial_load`] 供需要真实负载的调用方使用。
    pub fn route(tasks: &[TaskDescription], agents: &[AgentEntry]) -> Vec<Assignment> {
        Self::route_with_initial_load(tasks, agents, &[])
    }

    /// 与 [`Self::route`] 相同，但以 `initial_loads` 作为各 agent 的**起始负载计数**。
    ///
    /// `initial_loads` 只需包含关心的 agent（`(agent_id, load)`），
    /// 未列出的按 0 处理。用途：让路由能看到 `LoadBalancer` 的真实跨批次负载。
    pub fn route_with_initial_load(
        tasks: &[TaskDescription],
        agents: &[AgentEntry],
        initial_loads: &[(String, u32)],
    ) -> Vec<Assignment> {
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
        // ⚠️ 2026-10-02：以调用方提供的真实负载为起点（原先恒为全 0）。
        let mut agent_loads: Vec<(String, u32)> = agents
            .iter()
            .map(|a| {
                let start = initial_loads
                    .iter()
                    .find(|(id, _)| *id == a.agent_id)
                    .map(|(_, l)| *l)
                    .unwrap_or(0);
                (a.agent_id.clone(), start)
            })
            .collect();

        while let Some(PrioritizedTask { task, .. }) = heap.pop() {
            if let Some(best_agent) = Self::find_best_agent(&task, agents, &agent_loads) {
                let estimated_duration = Self::estimate_duration(&task, best_agent);
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
            // ⚠️ 2026-10-02 修正（**真实缺陷**，探针实测坐实）：
            // 原门槛是 `overlap > 0`（**沾边即合格**）⇒ 只要命中**任意一项**
            // 就算 eligible。实测（agent 分别只有 rust / sql，任务需 rust+sql+ml）：
            //     需要[rust,sql,ml] -> Some("only_rust")
            //     需要[rust,sql,ml] -> Some("only_sql")
            // ⇒ **任务被派给根本做不了的 agent**，且无任何信号（静默降级）。
            // 而字段自述是「Capabilities **required** to execute this task」
            // ⇒ required 即**必须全覆盖**。
            //
            // 修法：eligibility 改为**全覆盖**，overlap 退化为**排序**用。
            // ⛔ 保留 `capability_overlap` 本身与其 8 处测试
            //    （`capability_overlap_partial` 等明确断言部分覆盖返回 0.5
            //      ⇒ 那是对**函数**的正确断言，不是对**门槛**的背书）。
            if !Self::covers_all(&task.required_capabilities, &agent.capabilities) {
                continue;
            }
            let overlap =
                Self::capability_overlap(&task.required_capabilities, &agent.capabilities);

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

    /// 该 agent 是否**全覆盖**任务所需能力。
    ///
    /// 2026-10-02 新增：`TaskDescription::required_capabilities` 的字段自述是
    /// 「Capabilities **required** to execute this task」⇒ 缺任何一项都不该被派单。
    /// 与 `capability_overlap` 的区别：后者是**比例**（用于排序），
    /// 本函数是**资格**（用于筛除）。二者刻意分开，避免「0.5 也算命中」的门槛错误。
    ///
    /// 约定：
    /// · `required` 为空 ⇒ 任何 agent 都合格（无能力要求）。
    /// · `agent_caps` 为空而 `required` 非空 ⇒ 不合格。
    pub fn covers_all(required: &[String], agent_caps: &[String]) -> bool {
        if required.is_empty() {
            return true;
        }
        required
            .iter()
            .all(|req| agent_caps.iter().any(|ac| ac == req.as_str()))
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

    /// `covers_all`（资格）与 `capability_overlap`（排序）刻意分离的回归测试。
    #[test]
    fn covers_all_requires_every_capability() {
        let req = vec!["rust".to_string(), "sql".to_string()];
        // 全覆盖 ⇒ 合格
        assert!(TaskRouter::covers_all(
            &req,
            &["rust".to_string(), "sql".to_string(), "ml".to_string()]
        ));
        // 只覆盖一半 ⇒ **不合格**（但 overlap 仍是 0.5，说明它只是排序分）
        assert!(!TaskRouter::covers_all(&req, &["rust".to_string()]));
        assert!((TaskRouter::capability_overlap(&req, &["rust".to_string()]) - 0.5).abs() < 0.01);
        // 无能力要求 ⇒ 任何 agent 都合格
        assert!(TaskRouter::covers_all(&[], &[]));
        assert!(TaskRouter::covers_all(&[], &["rust".to_string()]));
        // agent 无能力而任务有要求 ⇒ 不合格
        assert!(!TaskRouter::covers_all(&req, &[]));
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
