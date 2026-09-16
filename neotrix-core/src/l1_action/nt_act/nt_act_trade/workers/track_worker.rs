//! Track Worker — 进度跟踪工作者
//!
//! 跟踪外贸订单全链路进度，汇总各阶段状态，生成进度报告。

#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{TradeWorker, WorkerResult, WorkerTask, WorkerType};

/// 单个任务的进度记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEntry {
    /// 任务 ID
    pub task_id: String,
    /// 当前状态
    pub status: TaskStatus,
    /// 进度百分比 (0.0 ~ 1.0)
    pub progress: f64,
    /// 状态消息
    pub message: String,
    /// 更新时间戳 (Unix 毫秒)
    pub updated_at: u64,
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Running => write!(f, "Running"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

/// 进度跟踪器
#[derive(Debug, Clone, Default)]
pub struct ProgressTracker {
    entries: Vec<ProgressEntry>,
}

impl ProgressTracker {
    /// 创建空跟踪器
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新进度
    pub fn update(&mut self, task_id: &str, status: TaskStatus, progress: f64, message: &str) {
        let now = now_millis();
        if let Some(entry) = self.entries.iter_mut().find(|e| e.task_id == task_id) {
            entry.status = status;
            entry.progress = progress;
            entry.message = message.to_string();
            entry.updated_at = now;
        } else {
            self.entries.push(ProgressEntry {
                task_id: task_id.to_string(),
                status,
                progress,
                message: message.to_string(),
                updated_at: now,
            });
        }
    }

    /// 获取所有进度条目
    pub fn entries(&self) -> &[ProgressEntry] {
        &self.entries
    }

    /// 获取指定任务进度
    pub fn get(&self, task_id: &str) -> Option<&ProgressEntry> {
        self.entries.iter().find(|e| e.task_id == task_id)
    }

    /// 总体进度 (所有任务平均)
    pub fn overall_progress(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.entries.iter().map(|e| e.progress).sum();
        sum / self.entries.len() as f64
    }

    /// 清除已完成的条目
    pub fn clear_completed(&mut self) {
        self.entries
            .retain(|e| e.status != TaskStatus::Completed);
    }
}

/// 进度跟踪工作者
pub struct TrackWorker {
    worker_id: String,
    progress: Arc<Mutex<ProgressTracker>>,
}

impl TrackWorker {
    /// 创建跟踪工作者
    pub fn new(worker_id: &str) -> Self {
        Self {
            worker_id: worker_id.to_string(),
            progress: Arc::new(Mutex::new(ProgressTracker::new())),
        }
    }

    /// 创建共享进度跟踪器的工作者
    pub fn with_tracker(worker_id: &str, tracker: Arc<Mutex<ProgressTracker>>) -> Self {
        Self {
            worker_id: worker_id.to_string(),
            progress: tracker,
        }
    }

    /// 获取进度快照
    pub fn snapshot(&self) -> ProgressTracker {
        self.progress.lock().unwrap().clone()
    }
}

#[async_trait]
impl TradeWorker for TrackWorker {
    fn worker_id(&self) -> &str {
        &self.worker_id
    }

    fn worker_type(&self) -> WorkerType {
        WorkerType::Track
    }

    async fn execute(&self, task: WorkerTask) -> Result<WorkerResult, String> {
        let start = Instant::now();

        let action = task
            .params
            .get("action")
            .cloned()
            .unwrap_or_else(|| "update".to_string());

        let mut tracker = self.progress.lock().map_err(|e| e.to_string())?;

        match action.as_str() {
            "update" => {
                let status_str = task
                    .params
                    .get("status")
                    .cloned()
                    .unwrap_or_else(|| "running".to_string());
                let status = match status_str.as_str() {
                    "pending" => TaskStatus::Pending,
                    "running" => TaskStatus::Running,
                    "completed" => TaskStatus::Completed,
                    "failed" => TaskStatus::Failed,
                    _ => TaskStatus::Running,
                };
                let progress = task
                    .params
                    .get("progress")
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let message = task
                    .params
                    .get("message")
                    .cloned()
                    .unwrap_or_default();

                let task_id_display = task.entity_id.as_deref().unwrap_or(&task.task_id);
                tracker.update(task_id_display, status, progress, &message);

                let data = serde_json::json!({
                    "action": "updated",
                    "overall_progress": tracker.overall_progress(),
                    "tracked_tasks": tracker.entries().len(),
                });

                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(WorkerResult {
                    task_id: task.task_id,
                    success: true,
                    data,
                    duration_ms,
                    error: None,
                    metadata: HashMap::new(),
                })
            }
            "snapshot" => {
                let data = serde_json::json!({
                    "action": "snapshot",
                    "overall_progress": tracker.overall_progress(),
                    "entries": tracker.entries().len(),
                });

                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(WorkerResult {
                    task_id: task.task_id,
                    success: true,
                    data,
                    duration_ms,
                    error: None,
                    metadata: HashMap::new(),
                })
            }
            "clear" => {
                tracker.clear_completed();
                let data = serde_json::json!({
                    "action": "cleared",
                    "remaining": tracker.entries().len(),
                });

                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(WorkerResult {
                    task_id: task.task_id,
                    success: true,
                    data,
                    duration_ms,
                    error: None,
                    metadata: HashMap::new(),
                })
            }
            _ => {
                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(WorkerResult::failure(
                    &task.task_id,
                    &format!("unknown track action: {}", action),
                    duration_ms,
                ))
            }
        }
    }

    fn can_handle(&self, task_type: &str) -> bool {
        matches!(
            task_type,
            "track_progress" | "track_order" | "track_production" | "track_shipment"
        )
    }
}

use std::collections::HashMap;

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_worker() -> TrackWorker {
        TrackWorker::new("track_test")
    }

    #[test]
    fn test_worker_id_and_type() {
        let w = make_worker();
        assert_eq!(w.worker_id(), "track_test");
        assert_eq!(w.worker_type(), WorkerType::Track);
    }

    #[test]
    fn test_can_handle() {
        let w = make_worker();
        assert!(w.can_handle("track_progress"));
        assert!(w.can_handle("track_order"));
        assert!(!w.can_handle("extract_orders"));
        assert!(!w.can_handle("send_email"));
    }

    #[test]
    fn test_progress_tracker_basics() {
        let mut tracker = ProgressTracker::new();
        assert_eq!(tracker.overall_progress(), 0.0);

        tracker.update("t1", TaskStatus::Running, 0.5, "halfway");
        assert_eq!(tracker.entries().len(), 1);
        assert_eq!(tracker.overall_progress(), 0.5);

        tracker.update("t2", TaskStatus::Completed, 1.0, "done");
        assert_eq!(tracker.entries().len(), 2);
        // avg of 0.5 and 1.0
        assert!((tracker.overall_progress() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_tracker_update_existing() {
        let mut tracker = ProgressTracker::new();
        tracker.update("t1", TaskStatus::Running, 0.3, "started");
        tracker.update("t1", TaskStatus::Running, 0.7, "almost there");

        assert_eq!(tracker.entries().len(), 1);
        let entry = tracker.get("t1").unwrap();
        assert_eq!(entry.progress, 0.7);
        assert_eq!(entry.message, "almost there");
    }

    #[test]
    fn test_progress_tracker_clear_completed() {
        let mut tracker = ProgressTracker::new();
        tracker.update("t1", TaskStatus::Completed, 1.0, "done");
        tracker.update("t2", TaskStatus::Running, 0.5, "working");

        tracker.clear_completed();
        assert_eq!(tracker.entries().len(), 1);
        assert!(tracker.get("t1").is_none());
        assert!(tracker.get("t2").is_some());
    }

    #[tokio::test]
    async fn test_track_update() {
        let w = make_worker();
        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "track_progress".into(),
            entity_id: Some("ord_123".into()),
            params: [
                ("action".into(), "update".into()),
                ("status".into(), "running".into()),
                ("progress".into(), "0.6".into()),
                ("message".into(), "production in progress".into()),
            ]
            .into_iter()
            .collect(),
            priority: 50,
        };

        let result = w.execute(task).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["action"], "updated");
    }

    #[tokio::test]
    async fn test_track_snapshot() {
        let w = make_worker();
        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "track_progress".into(),
            entity_id: None,
            params: [("action".into(), "snapshot".into())]
                .into_iter()
                .collect(),
            priority: 50,
        };

        let result = w.execute(task).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["action"], "snapshot");
    }

    #[tokio::test]
    async fn test_track_unknown_action() {
        let w = make_worker();
        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "track_progress".into(),
            entity_id: None,
            params: [("action".into(), "bogus".into())]
                .into_iter()
                .collect(),
            priority: 50,
        };

        let result = w.execute(task).await.unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::Pending.to_string(), "Pending");
        assert_eq!(TaskStatus::Running.to_string(), "Running");
        assert_eq!(TaskStatus::Completed.to_string(), "Completed");
        assert_eq!(TaskStatus::Failed.to_string(), "Failed");
    }
}
