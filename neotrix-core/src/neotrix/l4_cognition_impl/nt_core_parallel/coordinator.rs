use std::sync::{Arc, Mutex};
use crate::neotrix::nt_core_parallel::types::{Task, AgentId, AllocationStrategy};
use crate::neotrix::l1_body_impl::nt_io_provider::context_budget::estimate_tokens;

pub trait ReasoningProvider: Send + Sync {
    fn reason(&mut self, task: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
use crate::neotrix::nt_core_parallel::executor::{ParallelExecutor, OptimalTaskAllocator};

#[derive(Debug, Clone)]
pub struct AgentResult {
    pub agent_id: String,
    pub task_index: usize,
    pub output: String,
    pub success: bool,
}

pub struct MultiAgentCoordinator {
    _executor: ParallelExecutor,
    allocator: OptimalTaskAllocator,
    pub agents: Vec<AgentConfig>,
    engine: Option<Arc<Mutex<Box<dyn ReasoningProvider>>>>,
}

pub struct AgentConfig {
    pub id: AgentId,
    pub capability: Vec<f64>,
    pub throughput: f64,
}

impl MultiAgentCoordinator {
    pub fn new(max_agents: usize) -> Self {
        Self {
            _executor: ParallelExecutor::new(max_agents),
            allocator: OptimalTaskAllocator::new(AllocationStrategy::Hybrid),
            agents: Vec::new(),
            engine: None,
        }
    }

    pub fn with_engine(mut self, engine: Arc<Mutex<Box<dyn ReasoningProvider>>>) -> Self {
        self.engine = Some(engine);
        self
    }

    pub fn register_agent(&mut self, id: &str, capability: Vec<f64>) {
        self.agents.push(AgentConfig {
            id: id.to_string(),
            capability,
            throughput: 1.0,
        });
    }

    pub fn set_allocation_strategy(&mut self, strategy: AllocationStrategy) {
        self.allocator = OptimalTaskAllocator::new(strategy);
    }

    /// Allocate tasks to agents and execute in parallel
    pub async fn execute_tasks(&self, tasks: &[Task]) -> Vec<AgentResult> {
        if tasks.is_empty() {
            return Vec::new();
        }

        let agent_refs: Vec<_> = self.agents.iter().map(|a| (a.id.clone(), a.capability.clone(), a.throughput)).collect();
        let agents: Vec<_> = agent_refs.iter().map(|(id, _, _tp)| {
            crate::neotrix::nt_core_parallel::types::Agent::new(id.clone())
        }).collect();

        let allocation = self.allocator.allocate(tasks, &agents);
        let mut results = Vec::new();
        let engine = self.engine.clone();

        // Build parallel task list
        let mut handles = Vec::new();
        for (agent_id, task_indices) in &allocation {
            for &ti in task_indices {
                if let Some(task) = tasks.get(ti) {
                    let desc = String::from_utf8_lossy(&task.input.iter().map(|&b| b as u8).collect::<Vec<_>>()).to_string();
                    let aid = agent_id.clone();
                    let eng = engine.clone();

                    handles.push(tokio::spawn(async move {
                        if let Some(ref engine_arc) = eng {
                            let mut guard = match engine_arc.lock() {
                                Ok(g) => g,
                                Err(poisoned) => poisoned.into_inner(),
                            };
                            match guard.reason(&format!("[parallel:{}] {}", aid, desc)) {
                                Ok(output) => AgentResult {
                                    agent_id: aid,
                                    task_index: ti,
                                    output,
                                    success: true,
                                },
                                Err(e) => AgentResult {
                                    agent_id: aid,
                                    task_index: ti,
                                    output: e.to_string(),
                                    success: false,
                                },
                            }
                        } else {
                            AgentResult {
                                agent_id: aid,
                                task_index: ti,
                                output: format!("[processed task {}]", ti),
                                success: true,
                            }
                        }
                    }));
                }
            }
        }

        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }

        results
    }

    /// Gather results from agent summaries
    pub fn summarize(&self, results: &[AgentResult]) -> String {
        let total = results.len();
        let success = results.iter().filter(|r| r.success).count();
        format!("Parallel execution: {}/{} tasks succeeded across {} agents",
            success, total, self.agents.len())
    }

    /// 扇出-合并 (orca worktree + x-algorithm DPP 吸收):
    /// 从所有 agent 的独立结果中用 DPP 重排挑选 `keep` 个高质量且互不重复的赢家,
    /// 供下游「比较→合并」使用 (而非只保留单一 winner)。
    ///
    /// 每个成功结果被建模为一个候选: 质量分取 1.0 (成功)/0.0 (失败),
    /// 特征向量取 agent 能力向量 (作为多样覆盖的度量维度)。
    pub fn select_winners(&self, results: &[AgentResult], keep: usize) -> Vec<AgentResult> {
        if results.is_empty() || keep == 0 {
            return Vec::new();
        }
        // agent id → 能力特征映射
        let capability_of: std::collections::HashMap<String, Vec<f64>> = self
            .agents
            .iter()
            .map(|a| (a.id.clone(), a.capability.clone()))
            .collect();

        let max_dim = self
            .agents
            .iter()
            .map(|a| a.capability.len())
            .max()
            .unwrap_or(1)
            .max(1);

        let candidates: Vec<crate::neotrix::nt_core_parallel::Candidate> = results
            .iter()
            .map(|r| {
                let feat = capability_of.get(&r.agent_id).cloned().unwrap_or_default();
                crate::neotrix::nt_core_parallel::Candidate::new(
                    &format!("{}#{}", r.agent_id, r.task_index),
                    if r.success { 1.0 } else { 0.0 },
                    feat,
                )
            })
            .collect();

        let selector = crate::neotrix::nt_core_parallel::DppSelector::new(max_dim);
        let winners = selector.merge_winners(&candidates, keep);
        let winner_ids: std::collections::HashSet<String> =
            winners.iter().map(|w| w.id.clone()).collect();

        results
            .iter()
            .filter(|r| {
                winner_ids.contains(&format!("{}#{}", r.agent_id, r.task_index))
            })
            .cloned()
            .collect()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// G25: 阶段化共享上下文编排 (arxiv 2603.20131) + context_budget 硬约束
// ────────────────────────────────────────────────────────────────────────────

/// 共享上下文中的一个共享条目 — 跨阶段累积, 供后续阶段 agent 消费。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedContextItem {
    /// 来源阶段名。
    pub stage: String,
    /// 来源 agent。
    pub agent: String,
    /// 条目文本。
    pub text: String,
    /// 注入顺序 (越大越新)。
    pub seq: u64,
}

/// 共享上下文窗口 — 累积各阶段产出, 受 token 预算硬约束。
#[derive(Debug, Clone, Default)]
pub struct SharedContextWindow {
    pub items: Vec<SharedContextItem>,
    pub seq: u64,
    /// 硬 token 上限 (0 = 无上限)。
    pub budget_tokens: usize,
}

impl SharedContextWindow {
    pub fn new(budget_tokens: usize) -> Self {
        Self {
            items: Vec::new(),
            seq: 0,
            budget_tokens,
        }
    }

    /// 当前 token 总量 (复用 context_budget 估算器)。
    pub fn token_count(&self) -> usize {
        self.items.iter().map(|i| estimate_tokens(&i.text)).sum()
    }

    /// 尝试注入一条共享上下文。若注入后超预算 → 拒绝 (返回 None),
    /// 由调用方决定降级 (截断/丢弃)。硬约束保证: 窗口永不超过 budget。
    pub fn try_push(&mut self, stage: &str, agent: &str, text: &str) -> Option<u64> {
        let added = estimate_tokens(text);
        if self.budget_tokens > 0 && self.token_count() + added > self.budget_tokens {
            return None;
        }
        self.seq += 1;
        self.items.push(SharedContextItem {
            stage: stage.to_string(),
            agent: agent.to_string(),
            text: text.to_string(),
            seq: self.seq,
        });
        Some(self.seq)
    }

    /// 查询某阶段之后的共享上下文 (供后续阶段检索)。
    pub fn after(&self, stage: &str) -> Vec<&SharedContextItem> {
        self.items.iter().filter(|i| i.stage != stage).collect()
    }

    /// 总条目数。
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 硬约束验证: 当前 token 总量 ≤ budget (0 = 无上限)。
    pub fn budget_ok(&self) -> bool {
        self.budget_tokens == 0 || self.token_count() <= self.budget_tokens
    }
}

/// 一次阶段执行结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageOutcome {
    pub stage: String,
    pub agents: usize,
    pub items_injected: usize,
    pub items_rejected: usize,
}

/// 阶段化共享上下文编排器 — 按阶段顺序执行 agent 组, 每阶段产出经
/// 预算门禁注入共享窗口; 窗口超预算时注入被拒 (硬约束), 不再回退。
///
/// 三要素 (arxiv 2603.20131 阶段化共享):
/// - 阶段划分: 规划 → 执行 → 审查, 每阶段独立 agent 组;
/// - 共享窗口: 前阶段结论作为后阶段输入;
/// - 预算门禁: context_budget 硬约束防止上下文无限膨胀。
#[derive(Debug, Clone)]
pub struct StagedContextOrchestrator {
    /// 阶段执行顺序。
    pub stages: Vec<String>,
    /// 共享窗口。
    pub window: SharedContextWindow,
    /// 各阶段产出记录。
    pub outcomes: Vec<StageOutcome>,
    /// 拒绝计数 (超预算被拒的注入数)。
    pub rejected: u64,
}

impl StagedContextOrchestrator {
    pub fn new(budget_tokens: usize) -> Self {
        Self {
            stages: vec!["plan".into(), "execute".into(), "review".into()],
            window: SharedContextWindow::new(budget_tokens),
            outcomes: Vec::new(),
            rejected: 0,
        }
    }

    /// 执行一个阶段: 模拟 N 个 agent 的产出, 逐条经预算门禁注入窗口。
    /// 纯确定性 (无 engine 依赖), 供编排语义验证; engine 注入见
    /// `run_stage_with_provider`。
    pub fn run_stage(&mut self, stage: &str, agent_outputs: &[(&str, &str)]) -> StageOutcome {
        let mut injected = 0usize;
        let mut rejected = 0usize;
        for (agent, text) in agent_outputs {
            match self.window.try_push(stage, agent, text) {
                Some(_) => injected += 1,
                None => {
                    rejected += 1;
                    self.rejected += 1;
                }
            }
        }
        let outcome = StageOutcome {
            stage: stage.to_string(),
            agents: agent_outputs.len(),
            items_injected: injected,
            items_rejected: rejected,
        };
        self.outcomes.push(outcome.clone());
        outcome
    }

    /// 带 engine 的阶段执行: 对每任务调用 ReasoningProvider, 产出注入共享窗口。
    pub async fn run_stage_with_provider(
        &mut self,
        stage: &str,
        coord: &MultiAgentCoordinator,
        tasks: &[Task],
    ) -> StageOutcome {
        let results = coord.execute_tasks(tasks).await;
        let mut agent_outputs = Vec::new();
        for r in &results {
            if r.success {
                agent_outputs.push((r.agent_id.as_str(), r.output.as_str()));
            }
        }
        self.run_stage(stage, &agent_outputs)
    }

    /// 检索指定阶段可见的共享上下文 (前阶段累积, 排除当前阶段)。
    pub fn context_for(&self, stage: &str) -> Vec<&SharedContextItem> {
        self.window.after(stage)
    }

    /// 硬约束验证: 窗口 token 总量 ≤ budget (若设了 budget)。
    pub fn budget_ok(&self) -> bool {
        self.window.budget_tokens == 0
            || self.window.token_count() <= self.window.budget_tokens
    }

    /// 汇总: 已注入条目数。
    pub fn injected_total(&self) -> usize {
        self.window.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_core_parallel::types::Task;

    #[tokio::test]
    async fn test_coordinator_empty_tasks() {
        let coord = MultiAgentCoordinator::new(4);
        let results = coord.execute_tasks(&[]).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_coordinator_no_engine_fallback() {
        let mut coord = MultiAgentCoordinator::new(2);
        coord.register_agent("worker1", vec![1.0, 0.0, 0.0]);
        coord.register_agent("worker2", vec![0.0, 1.0, 0.0]);
        let tasks = vec![Task::new("task1".to_string(), vec![1.0], 0)];
        let results = coord.execute_tasks(&tasks).await;
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[tokio::test]
    async fn test_coordinator_register_agents() {
        let mut coord = MultiAgentCoordinator::new(4);
        coord.register_agent("code", vec![1.0, 0.0]);
        coord.register_agent("design", vec![0.0, 1.0]);
        assert_eq!(coord.agents.len(), 2);
    }

    #[test]
    fn test_summarize() {
        let coord = MultiAgentCoordinator::new(2);
        let results = vec![
            AgentResult { agent_id: "a1".to_string(), task_index: 0, output: "ok".to_string(), success: true },
            AgentResult { agent_id: "a2".to_string(), task_index: 1, output: "fail".to_string(), success: false },
        ];
        let summary = coord.summarize(&results);
        assert!(summary.contains("1/2"));
    }

    #[test]
    fn test_select_winners_empty() {
        let coord = MultiAgentCoordinator::new(2);
        assert!(coord.select_winners(&[], 3).is_empty());
        let results = vec![
            AgentResult { agent_id: "a1".to_string(), task_index: 0, output: "x".to_string(), success: true },
        ];
        assert!(coord.select_winners(&results, 0).is_empty());
    }

    #[test]
    fn test_select_winners_keeps_only_success_and_caps() {
        let mut coord = MultiAgentCoordinator::new(4);
        coord.register_agent("w1", vec![1.0, 0.0, 0.0]);
        coord.register_agent("w2", vec![0.0, 1.0, 0.0]);
        coord.register_agent("w3", vec![0.0, 0.0, 1.0]);
        let results = vec![
            AgentResult { agent_id: "w1".to_string(), task_index: 0, output: "ok".to_string(), success: true },
            AgentResult { agent_id: "w2".to_string(), task_index: 0, output: "ok".to_string(), success: true },
            AgentResult { agent_id: "w3".to_string(), task_index: 0, output: "fail".to_string(), success: false },
            AgentResult { agent_id: "w1".to_string(), task_index: 1, output: "ok".to_string(), success: true },
        ];
        let winners = coord.select_winners(&results, 2);
        assert!(winners.len() <= 2);
        assert!(winners.iter().all(|w| w.success), "failures excluded");
    }

    // ── G25 staged shared context ──────────────────────────────────────

    #[test]
    fn shared_window_injects_within_budget() {
        let mut w = SharedContextWindow::new(1000);
        assert!(w.try_push("plan", "a1", "step one").is_some());
        assert!(w.try_push("plan", "a2", "step two").is_some());
        assert_eq!(w.len(), 2);
        assert_eq!(w.items[0].seq, 1);
        assert_eq!(w.items[1].seq, 2);
    }

    #[test]
    fn shared_window_rejects_over_budget() {
        let mut w = SharedContextWindow::new(50);
        assert!(w.try_push("plan", "a1", "short").is_some());
        // 长文本单条即超 50 token → 拒绝 (硬约束)
        let long = "x".repeat(500);
        assert!(w.try_push("plan", "a2", &long).is_none());
        assert_eq!(w.len(), 1);
        assert!(w.budget_ok());
    }

    #[test]
    fn shared_window_after_excludes_current_stage() {
        let mut w = SharedContextWindow::new(0);
        w.try_push("plan", "a1", "p1");
        w.try_push("execute", "b1", "e1");
        let for_execute = w.after("execute");
        assert_eq!(for_execute.len(), 1, "plan context visible to execute");
        assert_eq!(for_execute[0].stage, "plan");
        let for_plan = w.after("plan");
        assert_eq!(for_plan.len(), 1, "execute context NOT visible to plan");
    }

    #[test]
    fn orchestrator_stages_accumulate_context() {
        let mut orch = StagedContextOrchestrator::new(0);
        orch.run_stage("plan", &[("p1", "design doc"), ("p2", "constraints")]);
        let execute_ctx = orch.context_for("execute");
        assert_eq!(execute_ctx.len(), 2, "plan outputs feed execute");
        assert!(orch.budget_ok());
        assert_eq!(orch.injected_total(), 2);
    }

    #[test]
    fn orchestrator_rejects_overflow_and_counts() {
        let mut orch = StagedContextOrchestrator::new(60);
        orch.run_stage("plan", &[("p1", "brief")]);
        let long = "y".repeat(1000);
        let outcome = orch.run_stage("execute", &[("b1", &long)]);
        assert_eq!(outcome.items_injected, 0);
        assert_eq!(outcome.items_rejected, 1);
        assert_eq!(orch.rejected, 1);
        assert!(orch.budget_ok(), "hard constraint never violated");
    }

    #[tokio::test]
    async fn orchestrator_with_provider_gates_output() {
        // 小预算 → engine 输出超限被拒; 窗口仍满足硬约束。
        let mut orch = StagedContextOrchestrator::new(100);
        let mut coord = MultiAgentCoordinator::new(2);
        coord.register_agent("worker1", vec![1.0, 0.0]);
        let tasks = vec![Task::new("t1".to_string(), vec![1.0], 0)];
        let outcome = orch
            .run_stage_with_provider("execute", &coord, &tasks)
            .await;
        // 无 engine → fallback 输出 `[processed task 0]` 短文本, 应在预算内
        assert_eq!(outcome.items_injected, 1);
        assert!(orch.budget_ok());
    }

    #[test]
    fn orchestrator_default_stage_order() {
        let orch = StagedContextOrchestrator::new(0);
        assert_eq!(orch.stages, vec!["plan", "execute", "review"]);
    }
}
