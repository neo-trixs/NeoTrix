//! Skill Chaining — 技能编排链
//!
//! 吸收 KB 经验:
//! - 技能组合/编排
//! - 数据流管道
//! - 条件执行
//! - 回滚机制

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 技能链管理器
pub struct SkillChainManager {
    chains: HashMap<String, SkillChain>,
    executors: HashMap<String, Box<dyn SkillExecutor>>,
    #[allow(dead_code)]
    config: ChainConfig,
    stats: ChainStats,
}

/// 链配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub max_parallel: usize,
    pub timeout: u64,
    pub retry_policy: RetryPolicy,
    pub enable_rollback: bool,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            max_parallel: 4,
            timeout: 3600,
            retry_policy: RetryPolicy::default(),
            enable_rollback: true,
        }
    }
}

/// 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_ms: u64,
    pub exponential: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_ms: 1000,
            exponential: true,
        }
    }
}

/// 技能链
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillChain {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<ChainStep>,
    pub config: ChainConfig,
    pub metadata: ChainMetadata,
}

/// 链步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStep {
    pub id: String,
    pub skill_id: String,
    pub config: serde_json::Value,
    pub dependencies: Vec<String>,
    pub condition: Option<String>,
    pub rollback_action: Option<String>,
}

/// 链元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetadata {
    pub author: Option<String>,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
}

/// 链执行器
pub struct ChainExecutor {
    chain: SkillChain,
    state: ChainState,
    results: HashMap<String, StepResult>,
}

/// 链状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainState {
    pub status: ChainStatus,
    pub current_step: Option<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

/// 链状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChainStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RollingBack,
}

/// 步骤结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub status: StepStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// 步骤状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    RolledBack,
}

/// 技能执行器 trait
pub trait SkillExecutor: Send + Sync {
    fn execute(&self, config: &serde_json::Value, input: Option<&serde_json::Value>) -> Result<serde_json::Value, String>;
    fn rollback(&self, config: &serde_json::Value, output: &serde_json::Value) -> Result<(), String>;
    fn name(&self) -> &str;
}

/// 链统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub total_chains: u64,
    pub running_chains: u64,
    pub completed_chains: u64,
    pub failed_chains: u64,
    pub avg_execution_time: f64,
}

/// 链执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExecutionResult {
    pub chain_id: String,
    pub status: ChainStatus,
    pub results: HashMap<String, StepResult>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl SkillChainManager {
    /// 创建新的技能链管理器
    pub fn new(config: ChainConfig) -> Self {
        Self {
            chains: HashMap::new(),
            executors: HashMap::new(),
            config,
            stats: ChainStats {
                total_chains: 0,
                running_chains: 0,
                completed_chains: 0,
                failed_chains: 0,
                avg_execution_time: 0.0,
            },
        }
    }

    /// 注册技能链
    pub fn register_chain(&mut self, chain: SkillChain) {
        self.chains.insert(chain.id.clone(), chain);
    }

    /// 注册技能执行器
    pub fn register_executor(&mut self, skill_id: &str, executor: Box<dyn SkillExecutor>) {
        self.executors.insert(skill_id.to_string(), executor);
    }

    /// 创建执行器
    pub fn create_executor(&self, chain_id: &str) -> Result<ChainExecutor, String> {
        let chain = self.chains.get(chain_id)
            .ok_or_else(|| format!("Chain {} not found", chain_id))?;

        Ok(ChainExecutor {
            chain: chain.clone(),
            state: ChainState {
                status: ChainStatus::Pending,
                current_step: None,
                started_at: None,
                completed_at: None,
                error: None,
            },
            results: HashMap::new(),
        })
    }

    /// 获取统计信息
    pub fn stats(&self) -> &ChainStats {
        &self.stats
    }
}

impl ChainExecutor {
    /// 执行链
    pub fn execute(&mut self, initial_input: Option<&serde_json::Value>) -> Result<ChainExecutionResult, String> {
        self.state.status = ChainStatus::Running;
        self.state.started_at = Some(chrono::Utc::now());

        let mut current_input = initial_input.cloned();

        // 拓扑排序步骤
        let sorted_steps = self.topological_sort();

        for step_id in &sorted_steps {
            let step = self.chain.steps.iter().find(|s| &s.id == step_id)
                .ok_or_else(|| format!("Step {} not found", step_id))?;

            // 检查条件
            if let Some(ref condition) = step.condition {
                if !self.evaluate_condition(condition) {
                    self.results.insert(step_id.clone(), StepResult {
                        step_id: step_id.clone(),
                        status: StepStatus::Skipped,
                        output: None,
                        error: None,
                        duration_ms: 0,
                    });
                    continue;
                }
            }

            // 检查依赖
            let deps_met = step.dependencies.iter().all(|dep| {
                self.results.get(dep).map(|r| r.status == StepStatus::Completed).unwrap_or(false)
            });

            if !deps_met {
                return Err(format!("Dependencies not met for step {}", step_id));
            }

            // 执行步骤
            self.state.current_step = Some(step_id.clone());

            // executors field not yet implemented on ChainExecutor
            tracing::warn!("ChainExecutor.executors not yet implemented; using placeholder");
            {
                let start = std::time::Instant::now();
                // Placeholder: pass through current input as output
                let output = current_input.clone().unwrap_or(serde_json::json!(null));
                let duration = start.elapsed().as_millis() as u64;
                self.results.insert(step_id.clone(), StepResult {
                    step_id: step_id.clone(),
                    status: StepStatus::Completed,
                    output: Some(output.clone()),
                    error: None,
                    duration_ms: duration,
                });
                current_input = Some(output);
            }
        }

        self.state.status = ChainStatus::Completed;
        self.state.completed_at = Some(chrono::Utc::now());

        Ok(self.build_result())
    }

    /// 拓扑排序
    fn topological_sort(&self) -> Vec<String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        for step in &self.chain.steps {
            in_degree.entry(step.id.clone()).or_insert(0);
            graph.entry(step.id.clone()).or_insert_with(Vec::new);

            for dep in &step.dependencies {
                graph.entry(dep.clone()).or_insert_with(Vec::new).push(step.id.clone());
                *in_degree.entry(step.id.clone()).or_insert(0) += 1;
            }
        }

        let mut queue: Vec<String> = in_degree.iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut sorted = Vec::new();

        while let Some(node) = queue.pop() {
            sorted.push(node.clone());
            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        sorted
    }

    /// 评估条件
    fn evaluate_condition(&self, _condition: &str) -> bool {
        // 简化版: 总是返回 true
        true
    }

    /// 回滚
    #[allow(dead_code)]
    fn rollback(&mut self) -> Result<(), String> {
        self.state.status = ChainStatus::RollingBack;

        // 按逆序回滚已完成的步骤
        let completed_steps: Vec<String> = self.results.iter()
            .filter(|(_, r)| r.status == StepStatus::Completed)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        for step_id in completed_steps {
            let step = self.chain.steps.iter().find(|s| s.id == step_id);
            if let Some(_step) = step {
            // executors field not yet implemented on ChainExecutor
            tracing::warn!("ChainExecutor.executors not yet implemented; using placeholder");
                if let Some(result) = self.results.get(&step_id) {
                    if let Some(ref _output) = result.output {
                        // let _ = executor.rollback(&step.config, output);
                        self.results.insert(step_id, StepResult {
                            status: StepStatus::RolledBack,
                            ..result.clone()
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// 构建结果
    fn build_result(&self) -> ChainExecutionResult {
        let duration = if let (Some(start), Some(end)) = (self.state.started_at, self.state.completed_at) {
            end.signed_duration_since(start).num_milliseconds() as u64
        } else {
            0
        };

        ChainExecutionResult {
            chain_id: self.chain.id.clone(),
            status: self.state.status.clone(),
            results: self.results.clone(),
            output: self.results.values()
                .last()
                .and_then(|r| r.output.clone()),
            error: self.state.error.clone(),
            duration_ms: duration,
        }
    }
}
