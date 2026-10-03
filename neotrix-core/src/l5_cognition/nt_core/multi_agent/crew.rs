//! Crew Orchestration (crewAI pattern)
//!
//! Orchestrates multiple role-configured agents with configurable
//! execution strategies: Sequential, Parallel, or Hierarchical.

use serde::{Deserialize, Serialize};

use super::role::RoleConfig;
use super::delegation::DelegationRequest;

/// Unique identifier for an agent handle within a crew
pub type AgentHandle = String;

/// Execution strategy for a crew
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrewStrategy {
    /// Tasks run one after another, each consuming prior output
    Sequential,
    /// Tasks run concurrently via tokio::spawn
    Parallel,
    /// Manager agent delegates to worker agents, aggregates results
    Hierarchical,
}

impl std::fmt::Display for CrewStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrewStrategy::Sequential => write!(f, "Sequential"),
            CrewStrategy::Parallel => write!(f, "Parallel"),
            CrewStrategy::Hierarchical => write!(f, "Hierarchical"),
        }
    }
}

/// A single agent within a crew — role config + runtime identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewAgent {
    pub handle: AgentHandle,
    pub config: RoleConfig,
}

impl CrewAgent {
    pub fn new(handle: impl Into<String>, config: RoleConfig) -> Self {
        Self {
            handle: handle.into(),
            config,
        }
    }
}

/// Task assigned to a crew
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewTask {
    pub id: String,
    pub description: String,
    /// Agent handles eligible to execute this task (empty = any)
    pub eligible_agents: Vec<AgentHandle>,
    /// Priority (higher = more urgent)
    pub priority: i32,
    /// Input context from prior tasks
    pub context: String,
}

impl CrewTask {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.into(),
            eligible_agents: Vec::new(),
            priority: 0,
            context: String::new(),
        }
    }

    pub fn with_eligible_agents(mut self, handles: Vec<AgentHandle>) -> Self {
        self.eligible_agents = handles;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = context.into();
        self
    }
}

/// Result from a single agent's task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewResult {
    pub agent_handle: AgentHandle,
    pub task_id: String,
    pub output: String,
    pub success: bool,
    pub iterations_used: u32,
}

/// The Crew — a team of agents executing tasks with a chosen strategy.
///
/// # Strategies
///
/// - **Sequential**: Tasks are executed in order; each agent's output feeds
///   the next task's context (pipeline).
/// - **Parallel**: All eligible agents execute tasks concurrently.
/// - **Hierarchical**: A manager agent delegates subtasks to workers via
///   the delegation protocol, then aggregates results.
pub struct Crew {
    pub name: String,
    pub agents: Vec<CrewAgent>,
    pub strategy: CrewStrategy,
    pub max_iterations: u32,
}

impl Crew {
    pub fn new(name: impl Into<String>, strategy: CrewStrategy) -> Self {
        Self {
            name: name.into(),
            agents: Vec::new(),
            strategy,
            max_iterations: 10,
        }
    }

    /// Add an agent to the crew
    pub fn add_agent(mut self, agent: CrewAgent) -> Self {
        self.agents.push(agent);
        self
    }

    /// Set max iterations for the entire crew
    pub fn with_max_iterations(mut self, n: u32) -> Self {
        self.max_iterations = n;
        self
    }

    /// Find an agent by handle
    pub fn get_agent(&self, handle: &str) -> Option<&CrewAgent> {
        self.agents.iter().find(|a| a.handle == handle)
    }

    /// Find agents eligible for a task
    fn eligible_agents(&self, task: &CrewTask) -> Vec<&CrewAgent> {
        if task.eligible_agents.is_empty() {
            return self.agents.iter().collect();
        }
        self.agents
            .iter()
            .filter(|a| task.eligible_agents.contains(&a.handle))
            .collect()
    }

    /// Execute a set of tasks according to the crew's strategy.
    ///
    /// Returns results in execution order. For Sequential, each task receives
    /// accumulated context from prior results.
    pub async fn execute(&self, tasks: &[CrewTask]) -> Vec<CrewResult> {
        match self.strategy {
            CrewStrategy::Sequential => self.execute_sequential(tasks).await,
            CrewStrategy::Parallel => self.execute_parallel(tasks).await,
            CrewStrategy::Hierarchical => self.execute_hierarchical(tasks).await,
        }
    }

    /// Sequential: pipeline execution with context accumulation
    async fn execute_sequential(&self, tasks: &[CrewTask]) -> Vec<CrewResult> {
        let mut results = Vec::new();
        let mut accumulated_context = String::new();

        for task in tasks {
            let mut task_with_ctx = task.clone();
            if !accumulated_context.is_empty() {
                task_with_ctx.context = format!(
                    "{}\n---\n{}",
                    accumulated_context, task.context
                );
            }

            let eligible = self.eligible_agents(&task_with_ctx);
            if let Some(agent) = eligible.first() {
                let result = self.execute_single(agent, &task_with_ctx).await;
                if result.success {
                    accumulated_context.push_str(&result.output);
                    accumulated_context.push('\n');
                }
                results.push(result);
            }
        }
        results
    }

    /// Parallel: all eligible agents execute concurrently
    async fn execute_parallel(&self, tasks: &[CrewTask]) -> Vec<CrewResult> {
        let mut handles = Vec::new();

        for task in tasks {
            let eligible = self.eligible_agents(task);
            // ⚠️ 2026-10-02 修正（**真实缺陷**，由 177 个从未运行的测试抓出）：
            // 原实现对**每个 task × 每个合格 agent**各 spawn 一次
            // ⇒ 结果数 = 任务数 × 合格 agent 数（实测 2 × 3 = 6）
            // ⇒ 工作量 O(|tasks| × |agents|)，且同一任务被重复执行、返回重复结果。
            //
            // 而 `CrewStrategy::Parallel` 的字段自述是
            // 「**Tasks** run concurrently via tokio_underscore spawn」⇒
            // 并发单位是**任务**，每个任务应产出**一个**结果。
            //
            // 修法：每个任务只选**一个** agent 执行。
            // ⛔ 选择策略暂用「合格者中第一个」（按注册序，**确定性**）——
            //    「最优 agent 选择」需要 coordinator 的 `TaskRouter`（其
            //    `find_best_agent` 目前是私有的），属跨模块设计，不在此臆造。
            // ⚛ 先克隆再取：`eligible.first()` 会借用自临时 `eligible`，
            // 而 spawn 要求 'static 所有权 ⇒ 直接用会触发 E0521。
            let chosen = eligible.first().cloned();
            if let Some(agent) = chosen {
                let agent_clone = agent.clone();
                let task_clone = task.clone();
                let max_iter = self.max_iterations;
                handles.push(tokio::spawn(async move {
                    Self::execute_single_static(&agent_clone, &task_clone, max_iter).await
                }));
            }
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }
        results
    }

    /// Hierarchical: manager delegates to workers, then aggregates
    async fn execute_hierarchical(&self, tasks: &[CrewTask]) -> Vec<CrewResult> {
        let mut results = Vec::new();

        // Manager is first agent; workers are the rest
        let manager = match self.agents.first() {
            Some(m) => m,
            None => return results,
        };
        let workers: Vec<&CrewAgent> = self.agents[1..].iter().collect();

        for task in tasks {
            // Manager creates delegation requests for each worker
            let mut worker_results = Vec::new();
            for worker in &workers {
                let delegation = DelegationRequest::new(
                    &manager.handle,
                    &worker.handle,
                    &task.description,
                );
                let result = self.execute_delegation(&delegation, worker, task).await;
                worker_results.push(result);
            }

            // Manager aggregates worker results (uses first-valid by default)
            //
            // ⚠️ 2026-10-02 修正（E0382，且是**真逻辑 bug** 不只是编译错）：
            // 原写法在两个分支里各调用一次 `worker_results.into_iter()`，
            // 第一次就**消耗**了它 ⇒ `else if` 分支根本不可达。
            // 而本函数意图是「优先取首个 success，否则退回首个结果」，
            // 这需要**先定位再取走**，不能靠两次迭代。
            if !worker_results.is_empty() {
                let pick = worker_results
                    .iter()
                    .position(|r| r.success)
                    .unwrap_or(0);
                results.push(worker_results.swap_remove(pick));
            }
        }
        results
    }

    /// Execute a single agent on a task (instance method)
    async fn execute_single(&self, agent: &CrewAgent, task: &CrewTask) -> CrewResult {
        Self::execute_single_static(agent, task, self.max_iterations).await
    }

    /// Execute a single agent on a task (static, for spawning)
    async fn execute_single_static(
        agent: &CrewAgent,
        task: &CrewTask,
        max_iterations: u32,
    ) -> CrewResult {
        // Simulate agent execution — in production this hooks into
        // the reasoning engine via ReasoningProvider trait.
        let iterations = max_iterations.min(task.description.len() as u32 / 10 + 1);
        let output = format!(
            "[{}] processed: {}",
            agent.config.display_name(),
            task.description
        );
        CrewResult {
            agent_handle: agent.handle.clone(),
            task_id: task.id.clone(),
            output,
            success: true,
            iterations_used: iterations,
        }
    }

    /// Execute via delegation protocol (for hierarchical strategy)
    async fn execute_delegation(
        &self,
        delegation: &DelegationRequest,
        agent: &CrewAgent,
        task: &CrewTask,
    ) -> CrewResult {
        let _ = delegation; // delegation context available for audit trail
        Self::execute_single_static(agent, task, self.max_iterations).await
    }

    /// Aggregate results from all agents (delegates to aggregation module)
    pub fn aggregate_results(&self, results: &[CrewResult]) -> String {
        let successful: Vec<&CrewResult> = results.iter().filter(|r| r.success).collect();
        let total = results.len();
        let ok = successful.len();

        if ok == 0 {
            return format!("[{}] all {} tasks failed", self.name, total);
        }

        let outputs: Vec<&str> = successful.iter().map(|r| r.output.as_str()).collect();
        format!(
            "[{}] {}/{} tasks succeeded:\n{}",
            self.name,
            ok,
            total,
            outputs.join("\n---\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::role::AgentRole;

    fn test_crew(strategy: CrewStrategy) -> Crew {
        Crew::new("test-crew", strategy)
            .add_agent(CrewAgent::new("mgr", RoleConfig::new(AgentRole::Planner)))
            .add_agent(CrewAgent::new("coder", RoleConfig::new(AgentRole::Coder)))
            .add_agent(CrewAgent::new("reviewer", RoleConfig::new(AgentRole::Reviewer)))
            .with_max_iterations(5)
    }

    fn sample_tasks() -> Vec<CrewTask> {
        vec![
            CrewTask::new("implement feature A"),
            CrewTask::new("review feature A"),
        ]
    }

    #[tokio::test]
    async fn sequential_execution() {
        let crew = test_crew(CrewStrategy::Sequential);
        let results = crew.execute(&sample_tasks()).await;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
    }

    /// ✅ 2026-10-02 已修复（`execute_parallel` 的任务×agent 爆炸，详见该函数注释）。
    ///
    /// 当时症状是「期望 2 个结果、实得 6」：`sample_tasks()` 只有 2 个任务，
    /// 而 `execute_parallel` 对**每个 task × 每个合格 agent**各 spawn 一次
    /// ⇒ 2 × 3 = 6。
    #[tokio::test]
    async fn parallel_execution() {
        let crew = test_crew(CrewStrategy::Parallel);
        let results = crew.execute(&sample_tasks()).await;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
    }

    #[tokio::test]
    async fn hierarchical_execution() {
        let crew = test_crew(CrewStrategy::Hierarchical);
        let results = crew.execute(&sample_tasks()).await;
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.success));
    }

    #[test]
    fn get_agent_by_handle() {
        let crew = test_crew(CrewStrategy::Sequential);
        assert!(crew.get_agent("coder").is_some());
        assert!(crew.get_agent("nonexistent").is_none());
    }

    #[test]
    fn eligible_agents_respects_filter() {
        let crew = test_crew(CrewStrategy::Sequential);
        let task = CrewTask::new("task").with_eligible_agents(vec!["reviewer".into()]);
        let eligible = crew.eligible_agents(&task);
        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0].handle, "reviewer");
    }

    #[test]
    fn eligible_agents_returns_all_when_empty_filter() {
        let crew = test_crew(CrewStrategy::Sequential);
        let task = CrewTask::new("task");
        let eligible = crew.eligible_agents(&task);
        assert_eq!(eligible.len(), 3);
    }

    #[test]
    fn aggregate_results_all_failed() {
        let crew = test_crew(CrewStrategy::Sequential);
        let results = vec![CrewResult {
            agent_handle: "x".into(),
            task_id: "t1".into(),
            output: String::new(),
            success: false,
            iterations_used: 1,
        }];
        let agg = crew.aggregate_results(&results);
        assert!(agg.contains("failed"));
    }

    #[test]
    fn aggregate_results_mixed() {
        let crew = test_crew(CrewStrategy::Sequential);
        let results = vec![
            CrewResult {
                agent_handle: "a".into(),
                task_id: "t1".into(),
                output: "ok".into(),
                success: true,
                iterations_used: 1,
            },
            CrewResult {
                agent_handle: "b".into(),
                task_id: "t2".into(),
                output: "fail".into(),
                success: false,
                iterations_used: 1,
            },
        ];
        let agg = crew.aggregate_results(&results);
        assert!(agg.contains("1/2"));
        assert!(agg.contains("ok"));
    }

    #[test]
    fn crew_task_builder() {
        let task = CrewTask::new("do something")
            .with_priority(5)
            .with_context("prior context")
            .with_eligible_agents(vec!["coder".into()]);
        assert_eq!(task.priority, 5);
        assert_eq!(task.context, "prior context");
        assert_eq!(task.eligible_agents, vec!["coder".to_string()]);
    }

    #[test]
    fn strategy_display() {
        assert_eq!(CrewStrategy::Sequential.to_string(), "Sequential");
        assert_eq!(CrewStrategy::Parallel.to_string(), "Parallel");
        assert_eq!(CrewStrategy::Hierarchical.to_string(), "Hierarchical");
    }
}
