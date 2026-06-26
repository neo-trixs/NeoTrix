use crate::neotrix::nt_core_parallel::executor::{OptimalTaskAllocator, ParallelExecutor};
use crate::neotrix::nt_core_parallel::types::{AgentId, AllocationStrategy, Task};
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;
use std::sync::{Arc, Mutex};

pub struct ParallelAgentResult {
    pub agent_id: String,
    pub task_index: usize,
    pub output: String,
    pub success: bool,
}

pub struct MultiAgentCoordinator {
    _executor: ParallelExecutor,
    allocator: OptimalTaskAllocator,
    pub agents: Vec<AgentConfig>,
    engine: Option<Arc<Mutex<ReasoningEngine>>>,
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

    pub fn with_engine(mut self, engine: Arc<Mutex<ReasoningEngine>>) -> Self {
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
    pub async fn execute_tasks(&self, tasks: &[Task]) -> Vec<ParallelAgentResult> {
        if tasks.is_empty() {
            return Vec::new();
        }

        let agent_refs: Vec<_> = self
            .agents
            .iter()
            .map(|a| (a.id.clone(), a.capability.clone(), a.throughput))
            .collect();
        let agents: Vec<_> = agent_refs
            .iter()
            .map(|(id, _, _tp)| crate::neotrix::nt_core_parallel::types::Agent::new(id.clone()))
            .collect();

        let allocation = self.allocator.allocate(tasks, &agents);
        let mut results = Vec::new();
        let engine = self.engine.clone();

        // Build parallel task list
        let mut handles = Vec::new();
        for (agent_id, task_indices) in &allocation {
            for &ti in task_indices {
                if let Some(task) = tasks.get(ti) {
                    let desc = String::from_utf8_lossy(
                        &task.input.iter().map(|&b| b as u8).collect::<Vec<_>>(),
                    )
                    .to_string();
                    let aid = agent_id.clone();
                    let eng = engine.clone();

                    handles.push(tokio::spawn(async move {
                        if let Some(ref engine_arc) = eng {
                            let mut guard = match engine_arc.lock() {
                                Ok(g) => g,
                                Err(poisoned) => poisoned.into_inner(),
                            };
                            match guard.reason(&format!("[parallel:{}] {}", aid, desc)) {
                                Ok(output) => ParallelAgentResult {
                                    agent_id: aid,
                                    task_index: ti,
                                    output,
                                    success: true,
                                },
                                Err(e) => ParallelAgentResult {
                                    agent_id: aid,
                                    task_index: ti,
                                    output: e.to_string(),
                                    success: false,
                                },
                            }
                        } else {
                            ParallelAgentResult {
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
    pub fn summarize(&self, results: &[ParallelAgentResult]) -> String {
        let total = results.len();
        let success = results.iter().filter(|r| r.success).count();
        format!(
            "Parallel execution: {}/{} tasks succeeded across {} agents",
            success,
            total,
            self.agents.len()
        )
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
            ParallelAgentResult {
                agent_id: "a1".to_string(),
                task_index: 0,
                output: "ok".to_string(),
                success: true,
            },
            ParallelAgentResult {
                agent_id: "a2".to_string(),
                task_index: 1,
                output: "fail".to_string(),
                success: false,
            },
        ];
        let summary = coord.summarize(&results);
        assert!(summary.contains("1/2"));
    }
}
