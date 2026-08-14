use std::sync::{Arc, Mutex};
use crate::neotrix::nt_core_parallel::types::{Task, AgentId, AllocationStrategy};

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
}
