use std::collections::VecDeque;

use crate::core::nt_core_self_test::{SelfTest, SelfTestResult};

struct TaskRecord {
    id: String,
    task: String,
    source: String,
    priority: u8,
    success: Option<bool>,
}

pub struct DelegateEngine {
    tasks: VecDeque<TaskRecord>,
    total_completed: u64,
    total_succeeded: u64,
    next_id: u64,
}

impl DelegateEngine {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            total_completed: 0,
            total_succeeded: 0,
            next_id: 0,
        }
    }

    pub fn delegate(&mut self, task: &str, source: &str, priority: u8) -> Option<String> {
        let id = format!("del-{}", self.next_id);
        self.next_id += 1;
        self.tasks.push_back(TaskRecord {
            id: id.clone(),
            task: task.to_string(),
            source: source.to_string(),
            priority,
            success: None,
        });
        Some(id)
    }

    pub fn synchronize(&mut self) -> u64 {
        let mut pending = 0u64;
        for task in &mut self.tasks {
            if task.success.is_none() {
                task.success = Some(true);
                self.total_completed += 1;
                self.total_succeeded += 1;
            } else {
                pending += 1;
            }
        }
        self.tasks.retain(|t| t.success.is_none());
        pending
    }

    pub fn total_tasks(&self) -> u64 {
        self.total_completed + self.tasks.len() as u64
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_completed == 0 {
            return 1.0;
        }
        self.total_succeeded as f64 / self.total_completed as f64
    }

    pub fn pending_count(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for DelegateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTest for DelegateEngine {
    fn name(&self) -> &'static str {
        "DelegateEngine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        Ok(())
    }
}
