use crate::neotrix::nt_core_parallel::types::{Agent, AgentId, AllocationStrategy, Task};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub enum ExecMode {
    Sequential,
    Parallel,
}

#[derive(Debug)]
pub struct ParallelExecutor {
    _max_agents: usize,
    mode: ExecMode,
    tasks: Vec<(String, Vec<f64>, i32)>,
}

impl ParallelExecutor {
    pub fn new(max_agents: usize) -> Self {
        Self {
            _max_agents: max_agents,
            mode: ExecMode::Sequential,
            tasks: Vec::new(),
        }
    }

    pub fn set_mode(&mut self, mode: ExecMode) {
        self.mode = mode;
    }

    pub fn add_task(&mut self, agent_id: String, input: Vec<f64>, priority: i32) {
        self.tasks.push((agent_id, input, priority));
    }

    pub async fn execute(&self) -> Vec<Vec<f64>> {
        match self.mode {
            ExecMode::Sequential => self
                .tasks
                .iter()
                .map(|(_, input, _)| input.clone())
                .collect(),
            ExecMode::Parallel => {
                let mut results = Vec::new();
                for (_, input, _) in &self.tasks {
                    let input = input.clone();
                    let handle = tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        input
                    });
                    if let Ok(res) = handle.await {
                        results.push(res);
                    }
                }
                results
            }
        }
    }

    /// 安全 shell 执行：仅运行白名单命令，禁止 sh -c 模式
    pub fn execute_shell(command: &str) -> Result<String, String> {
        let allowed = [
            "echo", "ls", "cat", "pwd", "date", "whoami", "uname", "head", "tail", "wc", "sort",
        ];
        let cmd_name = command.split_whitespace().next().unwrap_or("");
        if !allowed.contains(&cmd_name) {
            return Err(format!("命令 '{}' 不在白名单中: {:?}", cmd_name, allowed));
        }
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("空命令".to_string());
        }
        let mut cmd = std::process::Command::new(parts[0]);
        for arg in &parts[1..] {
            cmd.arg(arg);
        }
        let output = cmd.output().map_err(|e| format!("执行失败: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if output.status.success() {
            Ok(stdout)
        } else {
            Err(format!(
                "exit={} stderr={}",
                output.status.code().unwrap_or(-1),
                stderr
            ))
        }
    }

    /// 从 Vec<f64> 解码命令字符串（仅供调试，不执行）
    pub fn decode_command(input: &[f64]) -> String {
        input
            .iter()
            .take(64)
            .map(|&b| (b as u8).clamp(32, 126) as char)
            .collect::<String>()
            .trim_end()
            .to_string()
    }
}

pub struct OptimalTaskAllocator {
    strategy: AllocationStrategy,
    capability_weight: f64,
    load_balance_weight: f64,
    throughput_weight: f64,
}

impl OptimalTaskAllocator {
    pub fn new(strategy: AllocationStrategy) -> Self {
        Self {
            strategy,
            capability_weight: 0.5,
            load_balance_weight: 0.3,
            throughput_weight: 0.2,
        }
    }

    pub fn with_weights(mut self, cap: f64, load: f64, throughput: f64) -> Self {
        self.capability_weight = cap;
        self.load_balance_weight = load;
        self.throughput_weight = throughput;
        self
    }

    pub fn allocate(&self, tasks: &[Task], agents: &[Agent]) -> Vec<(AgentId, Vec<usize>)> {
        if agents.is_empty() || tasks.is_empty() {
            return Vec::new();
        }
        match self.strategy {
            AllocationStrategy::CapabilityFirst => self.allocate_by_capability(tasks, agents),
            AllocationStrategy::LoadBalance => self.allocate_by_load_balance(tasks, agents),
            AllocationStrategy::ThroughputFirst => self.allocate_by_throughput(tasks, agents),
            AllocationStrategy::Hybrid => self.allocate_hybrid(tasks, agents),
        }
    }

    fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum();
        let nb: f64 = b.iter().map(|x| x * x).sum();
        if na == 0.0 || nb == 0.0 {
            return 0.0;
        }
        dot / (na.sqrt() * nb.sqrt())
    }

    fn allocate_by_capability(
        &self,
        tasks: &[Task],
        agents: &[Agent],
    ) -> Vec<(AgentId, Vec<usize>)> {
        let mut allocation: Vec<(AgentId, Vec<usize>)> =
            agents.iter().map(|a| (a.id.clone(), Vec::new())).collect();
        let mut scores: Vec<(usize, usize, f64)> = Vec::new();
        for (ti, task) in tasks.iter().enumerate() {
            for (ai, agent) in agents.iter().enumerate() {
                if agent.busy {
                    continue;
                }
                scores.push((
                    ti,
                    ai,
                    Self::cosine_similarity(&task.input, &agent.capability),
                ));
            }
        }
        scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal));
        let mut assigned_tasks = vec![false; tasks.len()];
        for (ti, ai, _) in &scores {
            if !assigned_tasks[*ti] {
                allocation[*ai].1.push(*ti);
                assigned_tasks[*ti] = true;
            }
        }
        allocation
    }

    fn allocate_by_load_balance(
        &self,
        tasks: &[Task],
        agents: &[Agent],
    ) -> Vec<(AgentId, Vec<usize>)> {
        let mut allocation: Vec<(AgentId, Vec<usize>)> =
            agents.iter().map(|a| (a.id.clone(), Vec::new())).collect();
        for (ti, _) in tasks.iter().enumerate() {
            let (min_ai, _) = allocation
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.1.len().cmp(&b.1.len()))
                .unwrap_or((0, &(String::new(), vec![])));
            allocation[min_ai].1.push(ti);
        }
        allocation
    }

    fn allocate_by_throughput(
        &self,
        _tasks: &[Task],
        agents: &[Agent],
    ) -> Vec<(AgentId, Vec<usize>)> {
        let mut sorted: Vec<(usize, &Agent)> = agents.iter().enumerate().collect();
        sorted.sort_by(|(_, a), (_, b)| {
            b.throughput
                .partial_cmp(&a.throughput)
                .unwrap_or(Ordering::Equal)
        });
        let mut allocation: Vec<(AgentId, Vec<usize>)> =
            agents.iter().map(|a| (a.id.clone(), Vec::new())).collect();
        for (rank, (ai, _)) in sorted.iter().enumerate() {
            allocation[*ai].1.push(rank);
        }
        allocation
    }

    fn allocate_hybrid(&self, tasks: &[Task], agents: &[Agent]) -> Vec<(AgentId, Vec<usize>)> {
        let mut combined: Vec<(AgentId, Vec<usize>)> =
            agents.iter().map(|a| (a.id.clone(), Vec::new())).collect();
        for (ti, task) in tasks.iter().enumerate() {
            let best = agents
                .iter()
                .enumerate()
                .map(|(ai, agent)| {
                    let cap = Self::cosine_similarity(&task.input, &agent.capability)
                        * self.capability_weight;
                    let load =
                        (1.0 / (combined[ai].1.len() as f64 + 1.0)) * self.load_balance_weight;
                    let tp = agent.throughput * self.throughput_weight;
                    (ai, cap + load + tp)
                })
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            if let Some((ai, _)) = best {
                combined[ai].1.push(ti);
            }
        }
        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_shell_echo() {
        let result = ParallelExecutor::execute_shell("echo hello");
        assert!(result.is_ok());
        assert_eq!(result.expect("result should be ok in test"), "hello");
    }

    #[test]
    fn test_execute_shell_fail() {
        let result = ParallelExecutor::execute_shell("exit 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_command() {
        let cmd: Vec<f64> = "ls -la".bytes().map(|b| b as f64).collect();
        let decoded = ParallelExecutor::decode_command(&cmd);
        assert_eq!(decoded, "ls -la");
    }
}
