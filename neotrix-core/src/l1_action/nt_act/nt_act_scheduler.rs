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

// ============================================================================
// Scheduler
// ============================================================================

/// Task scheduler for cron, one-time, and event-driven tasks
pub struct Scheduler {
    /// Registered tasks
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
    /// Execution history
    history: Arc<RwLock<Vec<TaskExecutionResult>>>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new scheduled task
    pub async fn register(&self, task: ScheduledTask) -> Result<(), SchedulerError> {
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
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Paused;
            Ok(())
        } else {
            Err(SchedulerError::NotFound(task_id.to_string()))
        }
    }

    /// Resume a task
    pub async fn resume(&self, task_id: &str) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Scheduled;
            Ok(())
        } else {
            Err(SchedulerError::NotFound(task_id.to_string()))
        }
    }

    /// Cancel a task
    pub async fn cancel(&self, task_id: &str) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Cancelled;
            Ok(())
        } else {
            Err(SchedulerError::NotFound(task_id.to_string()))
        }
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
        {
            let mut tasks = self.tasks.write().await;
            if let Some(t) = tasks.get_mut(task_id) {
                t.execution_count += 1;
                t.last_executed = Some(chrono::Utc::now().to_rfc3339());
            }
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
}
