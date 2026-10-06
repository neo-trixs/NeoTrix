//! Async Tool Executor — Parallel tool execution with backpressure
//!
//! Supports spawning async tasks, awaiting all results, cancelling
//! tasks, and collecting results with timeout and retry logic.

// 2026-10-05 修锁中毒放大（判据来自 `codewhale-hq/Codewhale`
// `docs/ARCHITECTURE.md`「Key Design Decisions」第 7 条的**判别标准**）。
//
// ## 原缺陷
// 本模块用 `std::sync::Mutex`（**可中毒**），且 `tokio::spawn` 块内调用
// **调用方提供的工具闭包**（`execute_with_retry` 的 `F: Fn(ToolExecutionRequest) -> …`）。
// ⇒ 工具 panic 时若持有锁 ⇒ 锁中毒 ⇒ 此后**全部** `.lock().unwrap()` 都 panic，
//   包括 `submit` / `cancel` / `results` / `stats` 等 10 处对外 API
//   ⇒ **单个工具的 panic 放大成执行器永久失效且无法自愈**。
//   实测本文件原有 **16 处** `.lock().unwrap()`。
//
// ## 为什么可以 `into_inner()` 而不是留 `expect()`
// Codewhale 的判别标准是 **「能否容忍陈旧状态」**（能否容忍 half-updated state）：
// · 该标准下 `into_inner()` 是正解 —— 因为本模块的锁装的是
//   `HashMap` / `HashSet` / `Vec`（**任务账本**），中毒只意味着
//   「上一次 panic 时正在改动这个集合」⇒ 集合本身仍是**合法 Rust 值**，
//   **不是半个字节流、不是半个文件**⇒ 读它不会崩。
// · 反例（本模块**没有**的）：若锁里是「写了一半的磁盘文件」或
//   「算了一半的校验和」，那才是必须 fail-stop 的场景。
//
// ## 同仓已有正确写法可对照（不是外来口味）
// `neotrix-core/src/l0_substrate/nt_core_di.rs` 早已统一用
// `lock().unwrap_or_else(|e| e.into_inner())`（10 处）⇒
// 本笔是把**同仓既有正确实践**铺到这 16 处，不是引入新写法。
//
// ⚠️ 与本仓硬规则的关系（未改动规则本身）：
// 规则是「生产禁 `unwrap`/`expect`」。本笔按同仓既有实践统一为
// `unwrap_or_else(|e| e.into_inner())` —— 它不是 `unwrap`/`expect`，
// 且**在类型层面就要求你显式面对中毒**（闭包里必须写 `e`）。
// ⛔ 本笔**不**为「已命名的中毒锁」开 `expect()` 例外 —— 那是规则级决策，
//   不该在一笔缺陷修复里顺手改。

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
/// 任务登记的**必跑清理 guard**（2026-10-05）。
///
/// ## 为什么需要它（这是比锁中毒更严重的一个缺陷）
/// 清理 `active_tasks` + 唤醒 `await_all` 这两个动作，**无论任务正常结束
/// 还是 panic 都必须发生**。原实现把它们排在
/// `execute_with_retry(..).await` **之后**且不在任何 guard 下
/// ⇒ 工具闭包 panic 时 async 块整体 unwind，那两行**永不执行**
/// ⇒ `active_tasks` 里那条永久滞留 ⇒ `await_all()` 的
/// `if active.is_empty()` 永假、`shutdown_notify.notified()` 永不被唤醒
/// ⇒ **整个执行器永久挂死**（实测：反向锁跑超 60s 不返回）。
///
/// ## 为什么放在 `Drop` 里
/// unwind 路径与正常路径**共用同一份清理代码**，
/// 不存在「只覆盖了正常路径」的可能 —— 那正是原缺陷的形状。
struct ActiveTaskGuard {
    active_tasks: Arc<Mutex<HashMap<String, TaskHandle>>>,
    task_id: String,
    shutdown_notify: Arc<Notify>,
}

impl Drop for ActiveTaskGuard {
    fn drop(&mut self) {
        self.active_tasks
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.task_id);
        self.shutdown_notify.notify_one();
    }
}

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
            // 清理动作交给 guard ⇒ panic 路径也会跑（见 ActiveTaskGuard 文档）
            let _cleanup = ActiveTaskGuard {
                active_tasks: active_tasks.clone(),
                task_id: task_id_clone.clone(),
                shutdown_notify: shutdown_notify.clone(),
            };

            let result = Self::execute_with_retry(
                &request_clone,
                &executor_fn,
                &config,
                &cancelled_tasks,
            ).await;

            if !cancelled_tasks.lock().unwrap_or_else(|e| e.into_inner()).contains(&task_id_clone) {
                completed_results.lock().unwrap_or_else(|e| e.into_inner()).insert(task_id_clone.clone(), result.clone());
            }

            result
        });

        let task_handle = TaskHandle {
            task_id: task_id.clone(),
            tool_name: request.tool_name.clone(),
            join_handle: handle,
        };

        self.active_tasks.lock().unwrap_or_else(|e| e.into_inner()).insert(task_id.clone(), task_handle);
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
            if cancelled_tasks.lock().unwrap_or_else(|e| e.into_inner()).contains(&request.tool_name) {
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
            let active = self.active_tasks.lock().unwrap_or_else(|e| e.into_inner());
            if active.is_empty() {
                drop(active);
                break;
            }
            drop(active);
            self.shutdown_notify.notified().await;
        }

        let completed = self.completed_results.lock().unwrap_or_else(|e| e.into_inner());
        results.extend(completed.values().cloned());
        results
    }

    /// Cancel a running task by ID
    ///
    /// # Returns
    /// `true` if the task was found and cancelled
    pub async fn cancel(&self, task_id: &str) -> bool {
        self.cancelled_tasks.lock().unwrap_or_else(|e| e.into_inner()).insert(task_id.to_string());
        self.active_tasks.lock().unwrap_or_else(|e| e.into_inner()).remove(task_id).is_some()
    }

    /// Cancel all running tasks
    pub async fn cancel_all(&self) {
        let mut tasks = self.active_tasks.lock().unwrap_or_else(|e| e.into_inner());
        for (id, _) in tasks.iter() {
            self.cancelled_tasks.lock().unwrap_or_else(|e| e.into_inner()).insert(id.clone());
        }
        tasks.clear();
    }

    /// Get results for all completed tasks
    ///
    /// # Returns
    /// HashMap mapping task IDs to their results
    pub async fn get_results(&self) -> HashMap<String, ToolResult> {
        self.completed_results.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Get results for a specific task
    pub async fn get_result(&self, task_id: &str) -> Option<ToolResult> {
        self.completed_results.lock().unwrap_or_else(|e| e.into_inner()).get(task_id).cloned()
    }

    /// Get count of active tasks
    pub async fn active_count(&self) -> usize {
        self.active_tasks.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    /// Check if backpressure is active
    pub fn is_backpressured(&self) -> bool {
        let active = self.active_tasks.lock().unwrap_or_else(|e| e.into_inner());
        active.len() >= self.config.backpressure_threshold
    }

    /// Shutdown the executor
    pub async fn shutdown(&self) {
        *self.running.lock().unwrap_or_else(|e| e.into_inner()) = false;
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

    /// 反向锁：**一个工具 panic 不得毒死整个执行器**。
    ///
    /// ## 缺陷形状（2026-10-05 修）
    /// 本模块用 `std::sync::Mutex`（**可中毒**），且 `tokio::spawn` 块内调用
    /// **调用方提供的工具闭包**。⇒ 工具 panic 且持锁 ⇒ 锁中毒 ⇒
    /// 此后全部 `.lock().unwrap()` 都 panic，**含 10 处对外 API**
    /// （`spawn` / `await_all` / `cancel` / `cancel_all` / `get_results` /
    /// `get_result` / `active_count` / `is_backpressured` / `shutdown` / `config`）
    /// ⇒ 单个工具的 panic 放大成执行器**永久失效且无法自愈**。
    ///
    /// ## 本测试怎么做
    /// 不靠「锁中毒」这种间接推断，而是**真的让工具 panic**，
    /// 然后逐个调用对外 API —— 任何一个 panic 就说明执行器被毒死了。
    ///
    /// ⚠️ 修复前：`.lock().unwrap()` 会在**第二次** `spawn`（锁已中毒）时 panic。
    #[tokio::test]
    async fn panicking_tool_must_not_poison_the_executor() {
        /// 故意 panic 的工具：模拟真实插件/工具崩在用户代码里
        fn exploding_executor(_req: ToolExecutionRequest) -> Pin<Box<dyn std::future::Future<Output = ToolResult> + Send>> {
            Box::pin(async move { panic!("工具内部 panic（模拟真实工具崩溃）") })
        }

        let req = |name: &str| ToolExecutionRequest {
            tool_name: name.to_string(),
            parameters: serde_json::json!({}),
            timeout_ms: 1000,
            max_retries: 0,
            priority: TaskPriority::Normal,
        };

        let executor = AsyncToolExecutor::new();

        // 1) 第一个工具 panic（它会持有并污染锁）
        executor.spawn(req("boom"), exploding_executor);

        // 2) 等它真的跑完（panic 发生在 spawn 内的 async 块里）
        let _ = executor.await_all().await;

        // 3) 核心断言：此后执行器必须**仍然可用**。
        //    修复前这里会在 `active_tasks.lock().unwrap()` 处 panic。
        executor.spawn(req("healthy"), dummy_executor);
        let results = executor.await_all().await;

        assert!(
            results.iter().any(|r| r.tool_name == "healthy" && r.success),
            "panic 之后健康工具仍必须能跑完（锁中毒会毁掉这一点）：{results:?}"
        );

        // 4) 全部对外 API 逐个触碰 —— 任一 panic 即为未修复
        assert!(executor.active_count().await <= 1);
        let _ = executor.get_results().await;
        let _ = executor.get_result("healthy").await;
        let _ = executor.cancel("nonexistent").await;
        let _ = executor.is_backpressured();
        let _ = executor.config();
    }
}
