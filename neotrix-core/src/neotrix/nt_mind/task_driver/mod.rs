use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::core::TaskType;
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;
use crate::neotrix::nt_mind::SelfIteratingBrain;

mod todo_store;
mod verifier;

pub use todo_store::{DependencyGraph, ItemPriority, ItemStatus, TodoItem, TodoStore};
pub use verifier::{CargoVerifier, VerifyResult};

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub attempted: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<TaskResult>,
    pub verify_summary: String,
}

pub struct TaskDriver {
    pub todo: TodoStore,
    pub verifier: CargoVerifier,
    pub brain: Option<Arc<Mutex<SelfIteratingBrain>>>,
    pub engine: Option<Arc<Mutex<ReasoningEngine>>>,
    pub todo_path: String,
}

impl TaskDriver {
    pub fn new(todo_path: &str, project_dir: &str) -> Result<Self, String> {
        let todo = TodoStore::load(todo_path)?;
        let verifier = CargoVerifier::new(project_dir);
        Ok(Self {
            todo,
            verifier,
            brain: None,
            engine: None,
            todo_path: todo_path.to_string(),
        })
    }

    pub fn with_brain(mut self, brain: Arc<Mutex<SelfIteratingBrain>>) -> Self {
        self.brain = Some(brain);
        self
    }

    pub fn with_engine(mut self, engine: Arc<Mutex<ReasoningEngine>>) -> Self {
        self.engine = Some(engine);
        self
    }

    pub fn reload(&mut self) -> Result<(), String> {
        self.todo = TodoStore::load(&self.todo_path)?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        self.todo.save()
    }

    pub fn ready_task_ids(&self) -> Vec<String> {
        let done_ids: HashSet<String> = self
            .todo
            .items
            .iter()
            .filter(|i| i.status == ItemStatus::Done)
            .map(|i| i.id.clone())
            .collect();
        let mut ready: Vec<String> = self
            .todo
            .items
            .iter()
            .filter(|i| i.status == ItemStatus::Pending)
            .filter(|i| i.depends_on.iter().all(|d| done_ids.contains(d)))
            .map(|i| i.id.clone())
            .collect();
        ready.sort_by(|a, b| {
            let pa = self.todo.get(a).map(|i| i.priority_rank()).unwrap_or(2);
            let pb = self.todo.get(b).map(|i| i.priority_rank()).unwrap_or(2);
            pa.cmp(&pb)
        });
        ready
    }

    pub fn execute_ready_batch(&mut self, max_tasks: usize) -> BatchResult {
        let ready_ids = self.ready_task_ids();
        let batch_ids: Vec<String> = ready_ids.into_iter().take(max_tasks).collect();

        if batch_ids.is_empty() {
            return BatchResult {
                attempted: 0,
                succeeded: 0,
                failed: 0,
                results: vec![],
                verify_summary: "没有就绪的任务".to_string(),
            };
        }

        let mut results = Vec::new();
        for id in &batch_ids {
            let task_clone = self.todo.get(id).cloned();
            if let Some(task) = task_clone {
                let result = self.execute_single_task(&task);
                let _ = self.save();
                results.push(result);
            }
        }

        let verify_summary = self.verifier.summary();
        let succeeded = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();

        BatchResult {
            attempted: batch_ids.len(),
            succeeded,
            failed,
            results,
            verify_summary,
        }
    }

    pub fn execute_single_task(&mut self, task: &TodoItem) -> TaskResult {
        let _ = self.todo.update_status(&task.id, ItemStatus::InProgress);
        let _ = self.save();

        if let Err(e) = self.exec_impl(task) {
            let _ = self.todo.update_status(&task.id, ItemStatus::Blocked);
            return TaskResult {
                task_id: task.id.clone(),
                success: false,
                error: Some(format!("执行失败: {}", e)),
            };
        }

        let verify = self.verifier.verify_all();
        let all_pass = verify.iter().all(|(_, r)| r.passed);

        if all_pass {
            let _ = self.todo.update_status(&task.id, ItemStatus::Done);
            TaskResult {
                task_id: task.id.clone(),
                success: true,
                error: None,
            }
        } else {
            let _ = self.todo.update_status(&task.id, ItemStatus::Blocked);
            let err_first: String = verify
                .iter()
                .find(|(_, r)| !r.passed)
                .map(|(name, r)| format!("{}: {:?}", name, r.errors))
                .unwrap_or_default();
            TaskResult {
                task_id: task.id.clone(),
                success: false,
                error: Some(format!("验证失败: {}", err_first)),
            }
        }
    }

    fn exec_impl(&self, task: &TodoItem) -> Result<(), String> {
        if let Some(ref engine) = self.engine {
            let mut eng = engine.lock().map_err(|e| format!("引擎锁失败: {}", e))?;
            let task_type = infer_task_type(task);
            let prompt = format!(
                "任务: {}\n文件: {:?}\n依赖: {:?}\n备注: {}",
                task.title, task.files, task.depends_on, task.notes
            );
            match task_type {
                TaskType::Design => {
                    eng.reason_task(&prompt).map_err(|e| e.to_string())?;
                }
                TaskType::CodeGeneration => {
                    eng.reason_task(&prompt).map_err(|e| e.to_string())?;
                }
                TaskType::CodeReview => {
                    eng.reason_task(&prompt).map_err(|e| e.to_string())?;
                }
                _ => {
                    eng.reason_task(&prompt).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    pub fn run_autonomous(&mut self, max_tasks: usize) -> (BatchResult, String) {
        let result = self.execute_ready_batch(max_tasks);
        let mut log = format!(
            "# TaskDriver 自动执行报告\n\n尝试: {}, 成功: {}, 失败: {}\n\n",
            result.attempted, result.succeeded, result.failed
        );
        for r in &result.results {
            let icon = if r.success { "✅" } else { "❌" };
            log.push_str(&format!(
                "{} {}: {}\n",
                icon,
                r.task_id,
                r.error.as_deref().unwrap_or("ok")
            ));
        }
        log.push_str(&format!("\n## 验证摘要\n\n{}", result.verify_summary));
        (result, log)
    }

    pub fn task_queue_status(&self) -> String {
        let total = self.todo.items.len();
        let done = self
            .todo
            .items
            .iter()
            .filter(|i| i.status == ItemStatus::Done)
            .count();
        let pending = self
            .todo
            .items
            .iter()
            .filter(|i| i.status == ItemStatus::Pending)
            .count();
        let blocked = self
            .todo
            .items
            .iter()
            .filter(|i| i.status == ItemStatus::Blocked)
            .count();
        let ready = self.ready_task_ids().len();
        format!(
            "任务队列: {}/{} 完成 | {} 待办 | {} 阻塞 | {} 就绪",
            done, total, pending, blocked, ready
        )
    }
}

fn infer_task_type(task: &TodoItem) -> TaskType {
    let title_lower = task.title.to_lowercase();
    if title_lower.contains("design") || title_lower.contains("设计") {
        TaskType::Design
    } else if title_lower.contains("code")
        || title_lower.contains("实现")
        || title_lower.contains("implement")
    {
        TaskType::CodeGeneration
    } else if title_lower.contains("review")
        || title_lower.contains("分析")
        || title_lower.contains("安全")
    {
        TaskType::CodeReview
    } else {
        TaskType::General
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_root_todo() -> String {
        let manifest = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()),
        );
        manifest
            .parent()
            .expect("value should be ok in test")
            .join("TODO.yml")
            .to_string_lossy()
            .to_string()
    }

    #[test]
    fn test_ready_task_ids_format() {
        let path = project_root_todo();
        let driver = TaskDriver::new(&path, ".").expect("value should be ok in test");
        let ids = driver.ready_task_ids();
        for id in &ids {
            assert!(!id.is_empty());
        }
    }

    #[test]
    fn test_task_queue_status_format() {
        let path = project_root_todo();
        let driver = TaskDriver::new(&path, ".").expect("value should be ok in test");
        let status = driver.task_queue_status();
        assert!(status.contains("任务队列"));
    }

    #[test]
    fn test_infer_task_type_design() {
        let item = TodoItem {
            id: "x".into(),
            title: "Design HTTP Proxy".into(),
            status: ItemStatus::Pending,
            priority: ItemPriority::High,
            created: "2026-01-01".into(),
            updated: "2026-01-01".into(),
            session: None,
            files: vec![],
            depends_on: vec![],
            notes: "".into(),
        };
        assert_eq!(infer_task_type(&item), TaskType::Design);
    }

    #[test]
    fn test_infer_task_type_code() {
        let item = TodoItem {
            id: "y".into(),
            title: "Implement PoC Engine".into(),
            status: ItemStatus::Pending,
            priority: ItemPriority::High,
            created: "2026-01-01".into(),
            updated: "2026-01-01".into(),
            session: None,
            files: vec![],
            depends_on: vec![],
            notes: "".into(),
        };
        assert_eq!(infer_task_type(&item), TaskType::CodeGeneration);
    }

    #[test]
    fn test_batch_result_default() {
        let r = BatchResult {
            attempted: 0,
            succeeded: 0,
            failed: 0,
            results: vec![],
            verify_summary: "".into(),
        };
        assert_eq!(r.attempted, 0);
    }
}
