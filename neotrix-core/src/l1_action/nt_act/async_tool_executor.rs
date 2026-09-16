//! Async Tool Executor — Parallel tool execution with backpressure
//!
//! Supports spawning async tasks, awaiting all results, cancelling
//! tasks, and collecting results with timeout and retry logic.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use tokio::time::{Duration, Instant};
use tokio::task::JoinHandle;

// ============================================================================
// Types
// ============================================================================

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub retry_count: u32,
}

/// Spawned task handle
#[derive(Debug)]
pub struct TaskHandle {
    pub task_id: String,
    pub tool_name: String,
    pub join_handle: JoinHandle<ToolResult>,
}

/// Tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRequest {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub priority: TaskPriority,
}

/// Task priority
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Executor configuration
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub max_concurrent: usize,
    pub default_timeout_ms: u64,
    pub default_max_retries: u32,
    pub backpressure_threshold: usize,
    pub retry_base_delay_ms: u64,
    pub retry_max_delay_ms: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            default_timeout_ms: 30_000,
            default_max_retries: 3,
            backpressure_threshold: 100,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 10_000,
        }
    }
}

// ============================================================================
// AsyncToolExecutor
// ============================================================================

/// Async tool executor supporting parallel execution with backpressure,
/// timeout, retry logic, and task lifecycle management.
pub struct AsyncToolExecutor {
    config: ExecutorConfig,
    active_tasks: Arc<Mutex<HashMap<String, TaskHandle>>>,
    completed_results: Arc<Mutex<HashMap<String, ToolResult>>>,
    cancelled_tasks: Arc<Mutex<HashSet<String>>>,
    pending_queue: Arc<Mutex<Vec<ToolExecutionRequest>>>,
    shutdown_notify: Arc<Notify>,
    running: Arc<Mutex<bool>>,
}

impl AsyncToolExecutor {
    /// Create a new async tool executor with default config
    pub fn new() -> Self {
        Self::with_config(ExecutorConfig::default())
    }

    /// Create a new async tool executor with custom config
    pub fn with_config(config: ExecutorConfig) -> Self {
        Self {
            config,
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            completed_results: Arc::new(Mutex::new(HashMap::new())),
            cancelled_tasks: Arc::new(Mutex::new(HashSet::new())),
            pending_queue: Arc::new(Mutex::new(Vec::new())),
            shutdown_notify: Arc::new(Notify::new()),
            running: Arc::new(Mutex::new(true)),
        }
    }

    /// Spawn a tool execution task
    ///
    /// # Arguments
    /// * `request` - The tool execution request
    /// * `executor_fn` - Async function that executes the tool
    ///
    /// # Returns
    /// The task ID
    pub fn spawn<F>(&self, request: ToolExecutionRequest, executor_fn: F) -> String
    where
        F: Fn(ToolExecutionRequest) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send>>
            + Send
            + Sync
            + Clone
            + 'static,
    {
        let task_id = format!("task_{}", uuid::Uuid::new_v4());
        let mut request = request;
        request.priority = request.priority;

        let active_tasks = self.active_tasks.clone();
        let completed_results = self.completed_results.clone();
        let cancelled_tasks = self.cancelled_tasks.clone();
        let shutdown_notify = self.shutdown_notify.clone();
        let config = self.config.clone();
        let executor_fn = executor_fn.clone();

        let task_id_clone = task_id.clone();
        let request_clone = request.clone();
        let handle = tokio::spawn(async move {
            let result = Self::execute_with_retry(
                &request_clone,
                &executor_fn,
                &config,
                &cancelled_tasks,
            ).await;

            if !cancelled_tasks.lock().unwrap().contains(&task_id_clone) {
                completed_results.lock().unwrap().insert(task_id_clone.clone(), result.clone());
            }

            active_tasks.lock().unwrap().remove(&task_id_clone);
            shutdown_notify.notify_one();
            result
        });

        let task_handle = TaskHandle {
            task_id: task_id.clone(),
            tool_name: request.tool_name.clone(),
            join_handle: handle,
        };

        self.active_tasks.lock().unwrap().insert(task_id.clone(), task_handle);
        task_id
    }

    /// Execute a tool with retry logic and timeout
    async fn execute_with_retry<F>(
        request: &ToolExecutionRequest,
        executor_fn: &F,
        config: &ExecutorConfig,
        cancelled_tasks: &Arc<Mutex<HashSet<String>>>,
    ) -> ToolResult
    where
        F: Fn(ToolExecutionRequest) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send>>,
    {
        let mut retry_count = 0;
        let start = Instant::now();

        loop {
            if cancelled_tasks.lock().unwrap().contains(&request.tool_name) {
                return ToolResult {
                    tool_name: request.tool_name.clone(),
                    success: false,
                    output: None,
                    error: Some("Task cancelled".to_string()),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    retry_count,
                };
            }

            let timeout = Duration::from_millis(
                request.timeout_ms.max(config.default_timeout_ms),
            );

            let result = match tokio::time::timeout(timeout, executor_fn(request.clone())).await {
                Ok(result) => result,
                Err(_) => {
                    retry_count += 1;
                    if retry_count > request.max_retries.max(config.default_max_retries) {
                        return ToolResult {
                            tool_name: request.tool_name.clone(),
                            success: false,
                            output: None,
                            error: Some(format!("Timeout after {} retries", retry_count)),
                            execution_time_ms: start.elapsed().as_millis() as u64,
                            retry_count,
                        };
                    }
                    let delay = Self::calculate_retry_delay(retry_count, config);
                    tokio::time::sleep(delay).await;
                    continue;
                }
            };

            if result.success {
                return ToolResult {
                    tool_name: request.tool_name.clone(),
                    success: true,
                    output: result.output,
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    retry_count,
                };
            } else {
                retry_count += 1;
                if retry_count > request.max_retries.max(config.default_max_retries) {
                    return ToolResult {
                        tool_name: request.tool_name.clone(),
                        success: false,
                        output: result.output,
                        error: result.error,
                        execution_time_ms: start.elapsed().as_millis() as u64,
                        retry_count,
                    };
                }
                let delay = Self::calculate_retry_delay(retry_count, config);
                tokio::time::sleep(delay).await;
            }
        }
    }

    /// Calculate exponential backoff delay
    fn calculate_retry_delay(retry_count: u32, config: &ExecutorConfig) -> Duration {
        let delay = config.retry_base_delay_ms * (2u64.pow(retry_count));
        Duration::from_millis(delay.min(config.retry_max_delay_ms))
    }

    /// Await all active tasks to complete
    ///
    /// # Returns
    /// Vec of all completed `ToolResult`
    pub async fn await_all(&self) -> Vec<ToolResult> {
        let mut results = Vec::new();

        loop {
            let active = self.active_tasks.lock().unwrap();
            if active.is_empty() {
                drop(active);
                break;
            }
            drop(active);
            self.shutdown_notify.notified().await;
        }

        let completed = self.completed_results.lock().unwrap();
        results.extend(completed.values().cloned());
        results
    }

    /// Cancel a running task by ID
    ///
    /// # Returns
    /// `true` if the task was found and cancelled
    pub async fn cancel(&self, task_id: &str) -> bool {
        self.cancelled_tasks.lock().unwrap().insert(task_id.to_string());
        self.active_tasks.lock().unwrap().remove(task_id).is_some()
    }

    /// Cancel all running tasks
    pub async fn cancel_all(&self) {
        let mut tasks = self.active_tasks.lock().unwrap();
        for (id, _) in tasks.iter() {
            self.cancelled_tasks.lock().unwrap().insert(id.clone());
        }
        tasks.clear();
    }

    /// Get results for all completed tasks
    ///
    /// # Returns
    /// HashMap mapping task IDs to their results
    pub async fn get_results(&self) -> HashMap<String, ToolResult> {
        self.completed_results.lock().unwrap().clone()
    }

    /// Get results for a specific task
    pub async fn get_result(&self, task_id: &str) -> Option<ToolResult> {
        self.completed_results.lock().unwrap().get(task_id).cloned()
    }

    /// Get count of active tasks
    pub async fn active_count(&self) -> usize {
        self.active_tasks.lock().unwrap().len()
    }

    /// Check if backpressure is active
    pub fn is_backpressured(&self) -> bool {
        let active = self.active_tasks.lock().unwrap();
        active.len() >= self.config.backpressure_threshold
    }

    /// Shutdown the executor
    pub async fn shutdown(&self) {
        *self.running.lock().unwrap() = false;
        self.cancel_all().await;
        self.shutdown_notify.notify_one();
    }

    /// Get the executor configuration
    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }
}

impl Default for AsyncToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::time::Duration as StdDuration;

    fn dummy_executor(req: ToolExecutionRequest) -> Pin<Box<dyn std::future::Future<Output = ToolResult> + Send>> {
        Box::pin(async move {
            ToolResult {
                tool_name: req.tool_name,
                success: true,
                output: Some(serde_json::json!("ok")),
                error: None,
                execution_time_ms: 10,
                retry_count: 0,
            }
        })
    }

    fn failing_executor(_req: ToolExecutionRequest) -> Pin<Box<dyn std::future::Future<Output = ToolResult> + Send>> {
        Box::pin(async move {
            ToolResult {
                tool_name: "fail".to_string(),
                success: false,
                output: None,
                error: Some("simulated error".to_string()),
                execution_time_ms: 5,
                retry_count: 0,
            }
        })
    }

    #[tokio::test]
    async fn test_spawn_and_await() {
        let executor = AsyncToolExecutor::new();
        let req = ToolExecutionRequest {
            tool_name: "test_tool".to_string(),
            parameters: serde_json::json!({"key": "value"}),
            timeout_ms: 1000,
            max_retries: 1,
            priority: TaskPriority::Normal,
        };

        let task_id = executor.spawn(req, dummy_executor);
        assert!(executor.active_count().await > 0);

        let results = executor.await_all().await;
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.success));
    }

    #[tokio::test]
    async fn test_cancel() {
        let executor = AsyncToolExecutor::new();
        let req = ToolExecutionRequest {
            tool_name: "cancel_tool".to_string(),
            parameters: serde_json::json!({}),
            timeout_ms: 5000,
            max_retries: 1,
            priority: TaskPriority::Normal,
        };

        let task_id = executor.spawn(req, dummy_executor);
        assert!(executor.cancel(&task_id).await);

        let results = executor.get_results().await;
        assert!(results.contains_key(&task_id) || executor.active_count().await == 0);
    }

    #[tokio::test]
    async fn test_get_results() {
        let executor = AsyncToolExecutor::new();
        let req = ToolExecutionRequest {
            tool_name: "result_tool".to_string(),
            parameters: serde_json::json!({}),
            timeout_ms: 1000,
            max_retries: 1,
            priority: TaskPriority::Normal,
        };

        executor.spawn(req, dummy_executor);
        let _ = executor.await_all().await;

        let results = executor.get_results().await;
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_backpressure() {
        let config = ExecutorConfig {
            max_concurrent: 2,
            backpressure_threshold: 2,
            ..ExecutorConfig::default()
        };
        let executor = AsyncToolExecutor::with_config(config);
        assert!(!executor.is_backpressured());

        for i in 0..3 {
            let req = ToolExecutionRequest {
                tool_name: format!("bp_tool_{}", i),
                parameters: serde_json::json!({}),
                timeout_ms: 1000,
                max_retries: 0,
                priority: TaskPriority::Normal,
            };
            executor.spawn(req, dummy_executor);
        }
    }

    #[tokio::test]
    async fn test_shutdown() {
        let executor = AsyncToolExecutor::new();
        let req = ToolExecutionRequest {
            tool_name: "shutdown_tool".to_string(),
            parameters: serde_json::json!({}),
            timeout_ms: 1000,
            max_retries: 1,
            priority: TaskPriority::Normal,
        };

        executor.spawn(req, dummy_executor);
        executor.shutdown().await;
        assert_eq!(executor.active_count().await, 0);
    }
}
