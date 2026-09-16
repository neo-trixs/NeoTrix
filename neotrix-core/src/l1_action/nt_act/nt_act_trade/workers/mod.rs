//! Trade Worker Pool — 外贸任务工作池
//!
//! 基于 trait object 的异步工作池，支持 Extract / Analyze / Write / Send / Track 五种工作类型。
//! 每个 worker 实现 `TradeWorker` trait，由 `WorkerPool` 统一注册、查找、调度。

#![forbid(unsafe_code)]

pub mod analyze_worker;
pub mod extract_worker;
pub mod send_worker;
pub mod track_worker;
pub mod write_worker;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use analyze_worker::AnalyzeWorker;
pub use extract_worker::ExtractWorker;
pub use send_worker::SendWorker;
pub use track_worker::TrackWorker;
pub use write_worker::WriteWorker;

// ============================================================
// 1. Worker 类型枚举
// ============================================================

/// 工作类型 — 标识 worker 的职责域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerType {
    /// 数据提取 (平台抓取、文件解析)
    Extract,
    /// 数据分析 (统计、评分、洞察)
    Analyze,
    /// 文档/报告生成
    Write,
    /// 邮件/通知发送
    Send,
    /// 进度跟踪与汇报
    Track,
}

impl std::fmt::Display for WorkerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Extract => write!(f, "Extract"),
            Self::Analyze => write!(f, "Analyze"),
            Self::Write => write!(f, "Write"),
            Self::Send => write!(f, "Send"),
            Self::Track => write!(f, "Track"),
        }
    }
}

// ============================================================
// 2. WorkerTask / WorkerResult
// ============================================================

/// 任务输入 — 传递给 worker 的执行上下文
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkerTask {
    /// 任务唯一 ID
    pub task_id: String,
    /// 任务类型字符串 (如 "extract_orders", "analyze_kpi")
    pub task_type: String,
    /// 关联的订单/客户 ID
    pub entity_id: Option<String>,
    /// 任务参数 (key-value)
    pub params: HashMap<String, String>,
    /// 任务优先级 (0 = 最低, 100 = 最高)
    pub priority: u8,
}

/// 任务输出 — worker 执行后的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
    /// 任务 ID
    pub task_id: String,
    /// 是否成功
    pub success: bool,
    /// 输出数据 (JSON payload)
    pub data: serde_json::Value,
    /// 执行耗时 (毫秒)
    pub duration_ms: u64,
    /// 错误信息 (仅失败时)
    pub error: Option<String>,
    /// 附加元数据
    pub metadata: HashMap<String, String>,
}

impl WorkerResult {
    /// 创建成功结果
    pub fn success(task_id: &str, data: serde_json::Value, duration_ms: u64) -> Self {
        Self {
            task_id: task_id.to_string(),
            success: true,
            data,
            duration_ms,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// 创建失败结果
    pub fn failure(task_id: &str, error: &str, duration_ms: u64) -> Self {
        Self {
            task_id: task_id.to_string(),
            success: false,
            data: serde_json::Value::Null,
            duration_ms,
            error: Some(error.to_string()),
            metadata: HashMap::new(),
        }
    }
}

// ============================================================
// 3. TradeWorker trait
// ============================================================

/// 工作节点 trait — 所有外贸 worker 的统一接口
#[async_trait]
pub trait TradeWorker: Send + Sync {
    /// Worker 唯一标识 (如 "extract_alibaba_01")
    fn worker_id(&self) -> &str;

    /// Worker 类型
    fn worker_type(&self) -> WorkerType;

    /// 执行任务
    async fn execute(&self, task: WorkerTask) -> Result<WorkerResult, String>;

    /// 是否能处理指定类型的任务
    fn can_handle(&self, task_type: &str) -> bool;
}

// ============================================================
// 4. WorkerPool
// ============================================================

/// 工作池 — 注册、查找、调度 worker
pub struct WorkerPool {
    /// worker_id → worker 实例
    workers: HashMap<String, Arc<dyn TradeWorker>>,
    /// worker_type → worker_id 列表 (按类型索引)
    type_index: HashMap<WorkerType, Vec<String>>,
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerPool {
    /// 创建空工作池
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// 注册 worker
    pub fn register(&mut self, worker: Arc<dyn TradeWorker>) {
        let id = worker.worker_id().to_string();
        let wtype = worker.worker_type();
        self.type_index
            .entry(wtype)
            .or_default()
            .push(id.clone());
        self.workers.insert(id, worker);
    }

    /// 按 worker_id 获取
    pub fn get_by_id(&self, worker_id: &str) -> Option<Arc<dyn TradeWorker>> {
        self.workers.get(worker_id).cloned()
    }

    /// 按类型获取第一个可用 worker
    pub fn get_worker(&self, worker_type: WorkerType) -> Option<Arc<dyn TradeWorker>> {
        self.type_index
            .get(&worker_type)
            .and_then(|ids| ids.first())
            .and_then(|id| self.workers.get(id).cloned())
    }

    /// 按类型获取所有 worker
    pub fn get_workers_by_type(&self, worker_type: WorkerType) -> Vec<Arc<dyn TradeWorker>> {
        self.type_index
            .get(&worker_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.workers.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 按任务类型查找能处理的 worker
    pub fn find_worker_for_task(&self, task_type: &str) -> Option<Arc<dyn TradeWorker>> {
        self.workers
            .values()
            .find(|w| w.can_handle(task_type))
            .cloned()
    }

    /// 执行任务 — 自动路由到匹配的 worker
    pub async fn execute_task(&self, task: WorkerTask) -> Result<WorkerResult, String> {
        let worker = self
            .find_worker_for_task(&task.task_type)
            .ok_or_else(|| format!("no worker can handle task type '{}'", task.task_type))?;
        worker.execute(task).await
    }

    /// 已注册 worker 数量
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    /// 列出所有 worker ID
    pub fn worker_ids(&self) -> Vec<&str> {
        self.workers.keys().map(|s| s.as_str()).collect()
    }
}

// ============================================================
// 5. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── Mock worker for testing ──

    struct MockWorker {
        id: String,
        wtype: WorkerType,
        supported: Vec<String>,
    }

    #[async_trait]
    impl TradeWorker for MockWorker {
        fn worker_id(&self) -> &str {
            &self.id
        }

        fn worker_type(&self) -> WorkerType {
            self.wtype
        }

        async fn execute(&self, task: WorkerTask) -> Result<WorkerResult, String> {
            let start = std::time::Instant::now();
            Ok(WorkerResult::success(
                &task.task_id,
                serde_json::json!({"worker": self.id, "task_type": task.task_type}),
                start.elapsed().as_millis() as u64,
            ))
        }

        fn can_handle(&self, task_type: &str) -> bool {
            self.supported.iter().any(|s| s == task_type)
        }
    }

    fn make_mock(id: &str, wtype: WorkerType, supported: Vec<&str>) -> Arc<dyn TradeWorker> {
        Arc::new(MockWorker {
            id: id.into(),
            wtype,
            supported: supported.into_iter().map(String::from).collect(),
        })
    }

    #[test]
    fn test_pool_register_and_len() {
        let mut pool = WorkerPool::new();
        assert!(pool.is_empty());

        pool.register(make_mock("w1", WorkerType::Extract, vec!["extract_orders"]));
        pool.register(make_mock("w2", WorkerType::Analyze, vec!["analyze_kpi"]));
        assert_eq!(pool.len(), 2);
        assert!(!pool.is_empty());
    }

    #[test]
    fn test_get_by_id() {
        let mut pool = WorkerPool::new();
        pool.register(make_mock("w1", WorkerType::Extract, vec!["extract_orders"]));
        assert!(pool.get_by_id("w1").is_some());
        assert!(pool.get_by_id("nonexistent").is_none());
    }

    #[test]
    fn test_get_worker_by_type() {
        let mut pool = WorkerPool::new();
        pool.register(make_mock("w1", WorkerType::Extract, vec!["extract_orders"]));
        pool.register(make_mock("w2", WorkerType::Analyze, vec!["analyze_kpi"]));

        let w = pool.get_worker(WorkerType::Extract);
        assert!(w.is_some());
        assert_eq!(w.unwrap().worker_id(), "w1");

        assert!(pool.get_worker(WorkerType::Write).is_none());
    }

    #[test]
    fn test_get_workers_by_type() {
        let mut pool = WorkerPool::new();
        pool.register(make_mock("w1", WorkerType::Extract, vec!["a"]));
        pool.register(make_mock("w2", WorkerType::Extract, vec!["b"]));
        pool.register(make_mock("w3", WorkerType::Analyze, vec!["c"]));

        let extract_workers = pool.get_workers_by_type(WorkerType::Extract);
        assert_eq!(extract_workers.len(), 2);
    }

    #[test]
    fn test_find_worker_for_task() {
        let mut pool = WorkerPool::new();
        pool.register(make_mock("w1", WorkerType::Extract, vec!["extract_orders"]));
        pool.register(make_mock("w2", WorkerType::Analyze, vec!["analyze_kpi"]));

        let w = pool.find_worker_for_task("extract_orders");
        assert!(w.is_some());
        assert_eq!(w.unwrap().worker_id(), "w1");

        assert!(pool.find_worker_for_task("unknown_task").is_none());
    }

    #[test]
    fn test_worker_ids() {
        let mut pool = WorkerPool::new();
        pool.register(make_mock("alpha", WorkerType::Extract, vec!["a"]));
        pool.register(make_mock("beta", WorkerType::Write, vec!["b"]));

        let mut ids = pool.worker_ids();
        ids.sort();
        assert_eq!(ids, vec!["alpha", "beta"]);
    }

    #[tokio::test]
    async fn test_execute_task() {
        let mut pool = WorkerPool::new();
        pool.register(make_mock("w1", WorkerType::Extract, vec!["extract_orders"]));

        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "extract_orders".into(),
            entity_id: Some("ord_123".into()),
            params: HashMap::new(),
            priority: 50,
        };

        let result = pool.execute_task(task).await.unwrap();
        assert!(result.success);
        assert_eq!(result.task_id, "t1");
    }

    #[tokio::test]
    async fn test_execute_task_no_worker() {
        let pool = WorkerPool::new();
        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "unknown".into(),
            entity_id: None,
            params: HashMap::new(),
            priority: 0,
        };

        let result = pool.execute_task(task).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no worker"));
    }

    #[test]
    fn test_worker_type_display() {
        assert_eq!(WorkerType::Extract.to_string(), "Extract");
        assert_eq!(WorkerType::Analyze.to_string(), "Analyze");
        assert_eq!(WorkerType::Write.to_string(), "Write");
        assert_eq!(WorkerType::Send.to_string(), "Send");
        assert_eq!(WorkerType::Track.to_string(), "Track");
    }

    #[test]
    fn test_worker_result_success() {
        let r = WorkerResult::success("t1", serde_json::json!({"ok": true}), 42);
        assert!(r.success);
        assert_eq!(r.task_id, "t1");
        assert_eq!(r.duration_ms, 42);
        assert!(r.error.is_none());
    }

    #[test]
    fn test_worker_result_failure() {
        let r = WorkerResult::failure("t1", "timeout", 100);
        assert!(!r.success);
        assert_eq!(r.error.unwrap(), "timeout");
    }
}
