//! Workflow Engine — 任务编排引擎
//!
//! 吸收 KB 经验:
//! - DAG (有向无环图) 任务编排
//! - 条件分支/并行执行
//! - 错误恢复/重试
//! - 任务依赖管理
//! - 进度追踪

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 工作流引擎
pub struct WorkflowEngine {
    workflows: HashMap<String, Workflow>,
    running: HashMap<String, WorkflowInstance>,
    task_executors: HashMap<String, Box<dyn TaskExecutor>>,
    stats: WorkflowStats,
}

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tasks: Vec<TaskDefinition>,
    pub edges: Vec<TaskEdge>,
    pub config: WorkflowConfig,
}

/// 工作流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WorkflowConfig {
    pub max_parallel: usize,
    pub timeout: Option<u64>,
    pub retry_policy: RetryPolicy,
    pub error_handling: ErrorHandling,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_parallel: 4,
            timeout: Some(3600), // 1 hour
            retry_policy: RetryPolicy::default(),
            error_handling: ErrorHandling::FailFast,
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

/// 错误处理
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorHandling {
    FailFast,
    SkipAndContinue,
    RetryThenFail,
    Custom(String),
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TaskDefinition {
    pub id: String,
    pub name: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub timeout: Option<u64>,
    pub retries: Option<u32>,
}

/// 任务边 (依赖关系)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEdge {
    pub from: String,
    pub to: String,
    pub condition: Option<String>,
}

/// 工作流实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub status: WorkflowStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub task_states: HashMap<String, TaskState>,
    pub context: HashMap<String, serde_json::Value>,
}

/// 工作流状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Skipped,
    Retrying,
}

/// 任务执行器 trait
pub(crate) trait TaskExecutor: Send + Sync {
    fn execute(&self, task: &TaskDefinition, context: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String>;
    fn name(&self) -> &str;
}

/// 工作流统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WorkflowStats {
    pub total_workflows: u64,
    pub running_workflows: u64,
    pub completed_workflows: u64,
    pub failed_workflows: u64,
    pub avg_execution_time: u64,
}

impl WorkflowEngine {
    /// 创建新的工作流引擎
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            running: HashMap::new(),
            task_executors: HashMap::new(),
            stats: WorkflowStats {
                total_workflows: 0,
                running_workflows: 0,
                completed_workflows: 0,
                failed_workflows: 0,
                avg_execution_time: 0,
            },
        }
    }

    /// 注册工作流
    pub fn register_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.id.clone(), workflow);
    }

    /// 注册任务执行器
    pub fn register_executor(&mut self, task_type: &str, executor: Box<dyn TaskExecutor>) {
        self.task_executors.insert(task_type.to_string(), executor);
    }

    /// 启动工作流
    pub fn start_workflow(&mut self, workflow_id: &str) -> Result<String, String> {
        let workflow = self.workflows.get(workflow_id)
            .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

        let instance_id = uuid::Uuid::new_v4().to_string();
        let mut task_states = HashMap::new();

        // 初始化所有任务状态
        for task in &workflow.tasks {
            task_states.insert(task.id.clone(), TaskState::Pending);
        }

        // 标记没有前驱的任务为 Ready
        for task in &workflow.tasks {
            let has_predecessor = workflow.edges.iter().any(|e| e.to == task.id);
            if !has_predecessor {
                task_states.insert(task.id.clone(), TaskState::Ready);
            }
        }

        let instance = WorkflowInstance {
            id: instance_id.clone(),
            workflow_id: workflow_id.to_string(),
            status: WorkflowStatus::Running,
            started_at: chrono::Utc::now(),
            completed_at: None,
            task_states,
            context: HashMap::new(),
        };

        self.running.insert(instance_id.clone(), instance);
        self.stats.total_workflows += 1;
        self.stats.running_workflows += 1;

        Ok(instance_id)
    }

    /// 执行下一步
    pub fn step(&mut self, instance_id: &str) -> Result<Vec<(String, TaskState)>, String> {
        let instance = self.running.get_mut(instance_id)
            .ok_or_else(|| format!("Instance {} not found", instance_id))?;

        let workflow = self.workflows.get(&instance.workflow_id)
            .ok_or_else(|| format!("Workflow {} not found", instance.workflow_id))?;

        let mut completed_tasks = Vec::new();

        // 找到所有 Ready 的任务
        let ready_tasks: Vec<String> = instance.task_states.iter()
            .filter(|(_, state)| **state == TaskState::Ready)
            .map(|(id, _)| id.clone())
            .collect();

        // 限制并行数
        let running_count = instance.task_states.values()
            .filter(|s| **s == TaskState::Running)
            .count();

        let available_slots = workflow.config.max_parallel.saturating_sub(running_count);

        for task_id in ready_tasks.into_iter().take(available_slots) {
            // 找到任务定义
            if let Some(task_def) = workflow.tasks.iter().find(|t| t.id == task_id) {
                // 执行任务
                if let Some(executor) = self.task_executors.get(&task_def.task_type) {
                    instance.task_states.insert(task_id.clone(), TaskState::Running);

                    match executor.execute(task_def, &instance.context) {
                        Ok(result) => {
                            instance.task_states.insert(task_id.clone(), TaskState::Completed);
                            instance.context.insert(task_id.clone(), result);
                            completed_tasks.push((task_id.clone(), TaskState::Completed));

                            // 检查后继任务是否可以执行
                            for edge in &workflow.edges {
                                if edge.from == task_id {
                                    let target_state = instance.task_states.get(&edge.to).cloned().unwrap_or(TaskState::Pending);
                                    if target_state == TaskState::Pending {
                                        // 检查所有前驱是否完成
                                        let all_predecessors_done = workflow.edges.iter()
                                            .filter(|e| e.to == edge.to)
                                            .all(|e| {
                                                instance.task_states.get(&e.from) == Some(&TaskState::Completed)
                                            });

                                        if all_predecessors_done {
                                            instance.task_states.insert(edge.to.clone(), TaskState::Ready);
                                        }
                                    }
                                }
                            }
                        }
                        Err(_e) => {
                            instance.task_states.insert(task_id.clone(), TaskState::Failed);
                            completed_tasks.push((task_id.clone(), TaskState::Failed));

                            match workflow.config.error_handling {
                                ErrorHandling::FailFast => {
                                    instance.status = WorkflowStatus::Failed;
                                    instance.completed_at = Some(chrono::Utc::now());
                                    self.stats.running_workflows -= 1;
                                    self.stats.failed_workflows += 1;
                                    return Ok(completed_tasks);
                                }
                                ErrorHandling::SkipAndContinue => {
                                    // 继续执行其他任务
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // 检查工作流是否完成
        let all_done = instance.task_states.values().all(|s| {
            *s == TaskState::Completed || *s == TaskState::Skipped || *s == TaskState::Failed
        });

        if all_done {
            instance.status = WorkflowStatus::Completed;
            instance.completed_at = Some(chrono::Utc::now());
            self.stats.running_workflows -= 1;
            self.stats.completed_workflows += 1;
        }

        Ok(completed_tasks)
    }

    /// 获取实例状态
    pub fn get_instance(&self, instance_id: &str) -> Option<&WorkflowInstance> {
        self.running.get(instance_id)
    }

    /// 取消工作流
    pub fn cancel_workflow(&mut self, instance_id: &str) -> Result<(), String> {
        if let Some(instance) = self.running.get_mut(instance_id) {
            instance.status = WorkflowStatus::Cancelled;
            instance.completed_at = Some(chrono::Utc::now());
            self.stats.running_workflows -= 1;
            Ok(())
        } else {
            Err(format!("Instance {} not found", instance_id))
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> &WorkflowStats {
        &self.stats
    }
}
