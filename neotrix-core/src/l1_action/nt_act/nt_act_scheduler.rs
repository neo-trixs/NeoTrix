//! Scheduler — Cron/One-time/Event-driven Task Scheduling
//!
//! Implements the ChatGPT/Claude pattern: scheduled tasks that run in the cloud
//! without requiring the device to be awake.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │           Scheduler                          │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │  Cron    │  │  Once    │  │  Event   │  │
//! │  │ Schedule │  │ Schedule │  │ Trigger  │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │        Task Dispatcher               │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # Safety
//! - All scheduled tasks are validated before execution
//! - No unsafe code (R-P1)

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Core Types
// ============================================================================

/// Schedule type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScheduleType {
    /// Cron expression (5 fields: minute hour day month weekday)
    Cron { expression: String },
    /// One-time execution at specific time
    Once { at: String },
    /// Interval-based (every N seconds)
    Interval { seconds: u64 },
    /// Event-driven (triggered by external event)
    Event { event_type: String },
}

/// Task status
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    /// Task is registered but not yet scheduled
    Registered,
    /// Task is scheduled and waiting
    Scheduled,
    /// Task is currently executing
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed(String),
    /// Task is paused
    Paused,
    /// Task is cancelled
    Cancelled,
}

/// Scheduled task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduledTask {
    /// Unique task ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Task instruction
    pub instruction: String,
    /// Schedule configuration
    pub schedule: ScheduleType,
    /// Current status
    pub status: TaskStatus,
    /// Execution count
    pub execution_count: u64,
    /// Last executed at
    pub last_executed: Option<String>,
    /// Next scheduled execution
    pub next_execution: Option<String>,
    /// Created at
    pub created_at: String,
    /// Maximum retries
    pub max_retries: u32,
    /// Current retry count
    pub retry_count: u32,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Tags
    pub tags: Vec<String>,
    /// 执行代理 ID（T18：Task 实体正典字段）
    #[serde(default)]
    pub executor_agent_id: Option<String>,
    /// 关联技能 ID 列表
    #[serde(default)]
    pub skill_ids: Vec<String>,
    /// 所属工作区 ID
    #[serde(default)]
    pub workspace_id: Option<String>,
    /// 交付物规格列表
    #[serde(default)]
    pub deliverables: Vec<DeliverableSpec>,
    /// 依赖的任务 ID 列表
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// 创建者标识
    #[serde(default)]
    pub created_by: String,
    /// 任务内执行历史
    #[serde(default)]
    pub history: Vec<TaskExecution>,
}

/// Task execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskExecutionResult {
    /// Task ID
    pub task_id: String,
    /// Execution ID
    pub execution_id: String,
    /// Whether execution succeeded
    pub success: bool,
    /// Output
    pub output: String,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Executed at
    pub executed_at: String,
}

/// Scheduler statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchedulerStats {
    pub total_tasks: usize,
    pub scheduled: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub paused: usize,
}

/// 交付物类型（T18 新增）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeliverableKind {
    /// 文件交付物
    File,
    /// 报告交付物
    Report,
    /// 代码交付物
    Code,
    /// 数据交付物
    Data,
}

/// 交付物规格（T18 新增）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeliverableSpec {
    /// 交付物名称
    pub name: String,
    /// 交付物类型
    pub kind: DeliverableKind,
    /// 输出路径（可选）
    pub output_path: Option<PathBuf>,
}

/// 任务单次执行记录（T18 新增，与全局 TaskExecutionResult 区分：任务内历史）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskExecution {
    /// 执行 ID
    pub execution_id: String,
    /// 开始时间（unix 秒）
    #[serde(default)]
    pub started_at: u64,
    /// 结束时间（unix 秒，可选）
    #[serde(default)]
    pub finished_at: Option<u64>,
    /// 执行状态（复用 TaskStatus）
    pub status: TaskStatus,
    /// 输出（可选）
    #[serde(default)]
    pub output: Option<String>,
    /// 错误信息（可选）
    #[serde(default)]
    pub error: Option<String>,
    /// 消耗 token 数
    #[serde(default)]
    pub tokens_used: u64,
    /// 花费（美元）
    #[serde(default)]
    pub cost_usd: f64,
    /// 产出工件数
    #[serde(default)]
    pub artifact_count: u64,
    /// 积分消耗
    #[serde(default)]
    pub credits: f64,
}

/// 任务摘要：同步镜像行（T18 新增，供 tick 只读）
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskSummary {
    /// 任务 ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务状态
    pub status: TaskStatus,
    /// 所属工作区
    pub workspace_id: Option<String>,
    /// 执行代理
    pub executor_agent_id: Option<String>,
}

// ============================================================================
// Scheduler
// ============================================================================

/// Task scheduler for cron, one-time, and event-driven tasks
pub struct Scheduler {
    /// Registered tasks
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
    /// Execution history
    history: Arc<RwLock<Vec<TaskExecutionResult>>>,
    /// 同步镜像：tick 只读，供同步查询使用（T18 新增）
    snapshot: std::sync::RwLock<Vec<TaskSummary>>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            snapshot: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// 由任务构造摘要（T18 内部辅助，同步）
    fn to_summary(task: &ScheduledTask) -> TaskSummary {
        TaskSummary {
            id: task.id.clone(),
            name: task.name.clone(),
            status: task.status.clone(),
            workspace_id: task.workspace_id.clone(),
            executor_agent_id: task.executor_agent_id.clone(),
        }
    }

    /// 同步更新镜像行（upsert，锁中毒时静默跳过）
    fn upsert_snapshot(&self, task: &ScheduledTask) {
        let summary = Self::to_summary(task);
        if let Ok(mut snap) = self.snapshot.write() {
            let mut found = false;
            for row in snap.iter_mut() {
                if row.id == summary.id {
                    *row = summary.clone();
                    found = true;
                    break;
                }
            }
            if !found {
                snap.push(summary);
            }
        }
    }

    /// 按工作区过滤（同步读镜像，供 tick 使用）
    pub fn list_by_workspace(&self, workspace_id: &str) -> Vec<TaskSummary> {
        if let Ok(snap) = self.snapshot.read() {
            snap.iter()
                .filter(|s| s.workspace_id.as_deref() == Some(workspace_id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 按执行代理过滤（同步读镜像，供 tick 使用）
    pub fn list_by_agent(&self, agent_id: &str) -> Vec<TaskSummary> {
        if let Ok(snap) = self.snapshot.read() {
            snap.iter()
                .filter(|s| s.executor_agent_id.as_deref() == Some(agent_id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 按状态过滤（同步读镜像，供 tick 使用）
    pub fn list_by_status(&self, status: &TaskStatus) -> Vec<TaskSummary> {
        if let Ok(snap) = self.snapshot.read() {
            snap.iter()
                .filter(|s| &s.status == status)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 任务内执行历史（读任务自带 history）
    pub async fn execution_history(&self, task_id: &str) -> Vec<TaskExecution> {
        let tasks = self.tasks.read().await;
        if let Some(task) = tasks.get(task_id) {
            task.history.clone()
        } else {
            Vec::new()
        }
    }

    /// Register a new scheduled task
    pub async fn register(&self, task: ScheduledTask) -> Result<(), SchedulerError> {
        // 同步更新镜像（T18：tick 只读镜像）
        self.upsert_snapshot(&task);
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task);
        Ok(())
    }

    /// Create and register a cron task
    pub async fn create_cron(
        &self,
        name: String,
        instruction: String,
        cron_expression: String,
    ) -> Result<String, SchedulerError> {
        // Validate cron expression
        let parts: Vec<&str> = cron_expression.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(SchedulerError::InvalidSchedule(
                "Cron expression must have 5 fields".to_string()
            ));
        }

        let task_id = format!("task-{}", uuid::Uuid::new_v4());
        let task = ScheduledTask {
            id: task_id.clone(),
            name,
            instruction,
            schedule: ScheduleType::Cron { expression: cron_expression },
            status: TaskStatus::Scheduled,
            execution_count: 0,
            last_executed: None,
            next_execution: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            max_retries: 3,
            retry_count: 0,
            timeout_secs: 300,
            tags: Vec::new(),
            // T18 新增正典字段：默认空（镜像经 register 同步更新）
            executor_agent_id: None,
            skill_ids: Vec::new(),
            workspace_id: None,
            deliverables: Vec::new(),
            depends_on: Vec::new(),
            created_by: String::new(),
            history: Vec::new(),
        };

        self.register(task).await?;
        Ok(task_id)
    }

    /// Create and register a one-time task
    pub async fn create_once(
        &self,
        name: String,
        instruction: String,
        at: String,
    ) -> Result<String, SchedulerError> {
        // Validate ISO 8601
        if chrono::DateTime::parse_from_rfc3339(&at).is_err() {
            return Err(SchedulerError::InvalidSchedule(
                "Invalid ISO 8601 timestamp".to_string()
            ));
        }

        let task_id = format!("task-{}", uuid::Uuid::new_v4());
        let task = ScheduledTask {
            id: task_id.clone(),
            name,
            instruction,
            schedule: ScheduleType::Once { at },
            status: TaskStatus::Scheduled,
            execution_count: 0,
            last_executed: None,
            next_execution: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            max_retries: 1,
            retry_count: 0,
            timeout_secs: 300,
            tags: Vec::new(),
            // T18 新增正典字段：默认空
            executor_agent_id: None,
            skill_ids: Vec::new(),
            workspace_id: None,
            deliverables: Vec::new(),
            depends_on: Vec::new(),
            created_by: String::new(),
            history: Vec::new(),
        };

        self.register(task).await?;
        Ok(task_id)
    }

    /// Create and register an interval task
    pub async fn create_interval(
        &self,
        name: String,
        instruction: String,
        seconds: u64,
    ) -> Result<String, SchedulerError> {
        if seconds == 0 {
            return Err(SchedulerError::InvalidSchedule(
                "Interval must be greater than 0".to_string()
            ));
        }

        let task_id = format!("task-{}", uuid::Uuid::new_v4());
        let task = ScheduledTask {
            id: task_id.clone(),
            name,
            instruction,
            schedule: ScheduleType::Interval { seconds },
            status: TaskStatus::Scheduled,
            execution_count: 0,
            last_executed: None,
            next_execution: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            max_retries: 3,
            retry_count: 0,
            timeout_secs: 300,
            tags: Vec::new(),
            // T18 新增正典字段：默认空
            executor_agent_id: None,
            skill_ids: Vec::new(),
            workspace_id: None,
            deliverables: Vec::new(),
            depends_on: Vec::new(),
            created_by: String::new(),
            history: Vec::new(),
        };

        self.register(task).await?;
        Ok(task_id)
    }

    /// Get a task by ID
    pub async fn get(&self, task_id: &str) -> Option<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// List all tasks
    pub async fn list(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// Pause a task
    pub async fn pause(&self, task_id: &str) -> Result<(), SchedulerError> {
        let snapshot_task = {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.status = TaskStatus::Paused;
                task.clone()
            } else {
                return Err(SchedulerError::NotFound(task_id.to_string()));
            }
        };
        // 同步更新镜像（T18）
        self.upsert_snapshot(&snapshot_task);
        Ok(())
    }

    /// Resume a task
    pub async fn resume(&self, task_id: &str) -> Result<(), SchedulerError> {
        let snapshot_task = {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.status = TaskStatus::Scheduled;
                task.clone()
            } else {
                return Err(SchedulerError::NotFound(task_id.to_string()));
            }
        };
        // 同步更新镜像（T18）
        self.upsert_snapshot(&snapshot_task);
        Ok(())
    }

    /// Cancel a task
    pub async fn cancel(&self, task_id: &str) -> Result<(), SchedulerError> {
        let snapshot_task = {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.status = TaskStatus::Cancelled;
                task.clone()
            } else {
                return Err(SchedulerError::NotFound(task_id.to_string()));
            }
        };
        // 同步更新镜像（T18 缺口补齐：cancel 改状态必须可见）
        self.upsert_snapshot(&snapshot_task);
        Ok(())
    }

    /// Execute a task manually
    pub async fn execute_now(&self, task_id: &str) -> Result<TaskExecutionResult, SchedulerError> {
        let task = self.get(task_id).await
            .ok_or(SchedulerError::NotFound(task_id.to_string()))?;

        let start = std::time::Instant::now();
        let execution_id = format!("exec-{}", uuid::Uuid::new_v4());

        // Simplified execution
        let result = TaskExecutionResult {
            task_id: task_id.to_string(),
            execution_id,
            success: true,
            output: format!("Executed: {}", task.instruction),
            error: None,
            duration_ms: start.elapsed().as_millis() as u64,
            executed_at: chrono::Utc::now().to_rfc3339(),
        };

        // Update task stats
        let snapshot_task = {
            let mut tasks = self.tasks.write().await;
            if let Some(t) = tasks.get_mut(task_id) {
                t.execution_count += 1;
                t.last_executed = Some(chrono::Utc::now().to_rfc3339());
                Some(t.clone())
            } else {
                None
            }
        };
        // 同步更新镜像（T18 缺口补齐：execute_now 改计数必须可见）
        if let Some(ref snapshot_task) = snapshot_task {
            self.upsert_snapshot(snapshot_task);
        }

        // Store in history
        {
            let mut history = self.history.write().await;
            history.push(result.clone());
            let history_len = history.len();
            if history_len > 1000 {
                history.drain(0..history_len - 1000);
            }
        }

        Ok(result)
    }

    /// Get execution history
    pub async fn history(&self, limit: Option<usize>) -> Vec<TaskExecutionResult> {
        let history = self.history.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.clone(),
        }
    }

    /// Get scheduler statistics
    pub async fn stats(&self) -> SchedulerStats {
        let tasks = self.tasks.read().await;
        let total = tasks.len();
        let scheduled = tasks.values().filter(|t| t.status == TaskStatus::Scheduled).count();
        let running = tasks.values().filter(|t| t.status == TaskStatus::Running).count();
        let completed = tasks.values().filter(|t| t.status == TaskStatus::Completed).count();
        let failed = tasks.values().filter(|t| matches!(t.status, TaskStatus::Failed(_))).count();
        let paused = tasks.values().filter(|t| t.status == TaskStatus::Paused).count();

        SchedulerStats {
            total_tasks: total,
            scheduled,
            running,
            completed,
            failed,
            paused,
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Scheduler errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum SchedulerError {
    #[error("task not found: {0}")]
    NotFound(String),

    #[error("invalid schedule: {0}")]
    InvalidSchedule(String),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_cron_task() {
        let scheduler = Scheduler::new();
        let task_id = scheduler.create_cron(
            "Daily Report".to_string(),
            "Generate daily report".to_string(),
            "0 9 * * *".to_string(),
        ).await.unwrap();

        let task = scheduler.get(&task_id).await.unwrap();
        assert_eq!(task.name, "Daily Report");
        assert_eq!(task.status, TaskStatus::Scheduled);
    }

    #[tokio::test]
    async fn test_create_interval_task() {
        let scheduler = Scheduler::new();
        let task_id = scheduler.create_interval(
            "Health Check".to_string(),
            "Check system health".to_string(),
            60,
        ).await.unwrap();

        let task = scheduler.get(&task_id).await.unwrap();
        assert_eq!(task.name, "Health Check");
    }

    #[tokio::test]
    async fn test_pause_resume() {
        let scheduler = Scheduler::new();
        let task_id = scheduler.create_once(
            "One-time Task".to_string(),
            "Do something".to_string(),
            "2026-12-31T23:59:59Z".to_string(),
        ).await.unwrap();

        scheduler.pause(&task_id).await.unwrap();
        let task = scheduler.get(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Paused);

        scheduler.resume(&task_id).await.unwrap();
        let task = scheduler.get(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Scheduled);
    }

    /// T18：register 后同步镜像可见
    #[tokio::test]
    async fn test_t18_snapshot_visible_after_register() {
        let scheduler = Scheduler::new();
        let task = ScheduledTask {
            id: "task-t18-a".to_string(),
            name: "T18 A".to_string(),
            instruction: "do a".to_string(),
            schedule: ScheduleType::Interval { seconds: 60 },
            status: TaskStatus::Scheduled,
            execution_count: 0,
            last_executed: None,
            next_execution: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            max_retries: 1,
            retry_count: 0,
            timeout_secs: 60,
            tags: Vec::new(),
            // T18 新字段
            executor_agent_id: Some("agent-1".to_string()),
            skill_ids: Vec::new(),
            workspace_id: Some("ws-1".to_string()),
            deliverables: Vec::new(),
            depends_on: Vec::new(),
            created_by: "t18".to_string(),
            history: Vec::new(),
        };
        scheduler.register(task).await.unwrap();
        // 同步读镜像：register 后立即可见
        let rows = scheduler.list_by_workspace("ws-1");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "task-t18-a");
        let by_agent = scheduler.list_by_agent("agent-1");
        assert_eq!(by_agent.len(), 1);
        let by_status = scheduler.list_by_status(&TaskStatus::Scheduled);
        assert_eq!(by_status.len(), 1);
    }

    /// T18：list_by_workspace 过滤正确
    #[tokio::test]
    async fn test_t18_list_by_workspace_filter() {
        let scheduler = Scheduler::new();
        for (id, ws) in [("task-t18-w1", "ws-1"), ("task-t18-w2", "ws-2")] {
            let task = ScheduledTask {
                id: id.to_string(),
                name: id.to_string(),
                instruction: "do".to_string(),
                schedule: ScheduleType::Interval { seconds: 30 },
                status: TaskStatus::Scheduled,
                execution_count: 0,
                last_executed: None,
                next_execution: None,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                max_retries: 1,
                retry_count: 0,
                timeout_secs: 30,
                tags: Vec::new(),
                executor_agent_id: None,
                skill_ids: Vec::new(),
                workspace_id: Some(ws.to_string()),
                deliverables: Vec::new(),
                depends_on: Vec::new(),
                created_by: "t18".to_string(),
                history: Vec::new(),
            };
            scheduler.register(task).await.unwrap();
        }
        let w1 = scheduler.list_by_workspace("ws-1");
        assert_eq!(w1.len(), 1);
        assert_eq!(w1[0].id, "task-t18-w1");
        let w2 = scheduler.list_by_workspace("ws-2");
        assert_eq!(w2.len(), 1);
        assert_eq!(w2[0].id, "task-t18-w2");
        let empty = scheduler.list_by_workspace("ws-missing");
        assert!(empty.is_empty());
    }
}
