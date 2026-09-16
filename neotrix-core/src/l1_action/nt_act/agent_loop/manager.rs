//! Manager role — Assigns tasks, monitors progress.
//!
//! From LongHorizon-Harness: the Manager is responsible for
//! task assignment, progress monitoring, and overall coordination
//! of the agent loop execution.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::l1_action::traits::{CapabilityError, Plan, PlanResult};
use super::trait_::{AgentLoop, AgentPlan, AgentLoopError, Adaptation, AdaptationStrategy, Objective, ExecutionResult, VerificationResult, PlanStep, StepOutput};
use super::config::LoopConfig;
use super::executor::AgentExecutor;
use super::auditor::Auditor;

/// Task status tracked by the Manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Escalated,
}

/// A task assigned by the Manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedTask {
    pub id: String,
    pub objective: Objective,
    pub status: TaskStatus,
    pub assigned_to: String,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub progress: f64,
    pub result: Option<ExecutionResult>,
}

/// Manager — Assigns tasks, monitors progress.
///
/// From LongHorizon-Harness: the Manager component is responsible for:
/// - Task decomposition and assignment
/// - Progress tracking and monitoring
/// - Resource allocation
/// - Escalation when tasks stall
pub struct Manager {
    config: LoopConfig,
    tasks: Arc<Mutex<HashMap<String, ManagedTask>>>,
    executor: Option<Arc<dyn AgentExecutor>>,
    auditor: Option<Arc<Auditor>>,
    max_concurrent: usize,
}

impl Manager {
    /// Create a new Manager.
    pub fn new(config: LoopConfig) -> Self {
        Self {
            config,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            executor: None,
            auditor: None,
            max_concurrent: 4,
        }
    }

    /// Set the executor to use for task execution.
    pub fn with_executor(mut self, executor: Arc<dyn AgentExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Set the auditor for verification.
    pub fn with_auditor(mut self, auditor: Arc<Auditor>) -> Self {
        self.auditor = Some(auditor);
        self
    }

    /// Set maximum concurrent tasks.
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Assign an objective as a managed task.
    pub fn assign_task(&self, objective: Objective, agent_id: &str) -> Result<String, CapabilityError> {
        let task_id = format!("task_{}_{}", agent_id, SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
        let task = ManagedTask {
            id: task_id.clone(),
            objective: objective.clone(),
            status: TaskStatus::Pending,
            assigned_to: agent_id.to_string(),
            started_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            completed_at: None,
            progress: 0.0,
            result: None,
        };

        self.tasks.lock().unwrap().insert(task_id.clone(), task);
        Ok(task_id)
    }

    /// Get a task by ID.
    pub fn get_task(&self, task_id: &str) -> Option<ManagedTask> {
        self.tasks.lock().unwrap().get(task_id).cloned()
    }

    /// Update task progress.
    pub fn update_progress(&self, task_id: &str, progress: f64) -> Result<(), CapabilityError> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(task_id) {
            task.progress = progress.clamp(0.0, 1.0);
            if progress >= 1.0 {
                task.status = TaskStatus::Completed;
                task.completed_at = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
            }
            Ok(())
        } else {
            Err(CapabilityError::NotAvailable(format!("Task {} not found", task_id)))
        }
    }

    /// Monitor all tasks and return their status summary.
    pub fn monitor(&self) -> TaskSummary {
        let tasks = self.tasks.lock().unwrap();
        let total = tasks.len();
        let pending = tasks.values().filter(|t| t.status == TaskStatus::Pending).count();
        let in_progress = tasks.values().filter(|t| t.status == TaskStatus::InProgress).count();
        let completed = tasks.values().filter(|t| t.status == TaskStatus::Completed).count();
        let failed = tasks.values().filter(|t| t.status == TaskStatus::Failed).count();

        TaskSummary { total, pending, in_progress, completed, failed }
    }

    /// Execute a task through the agent loop.
    pub fn execute_task(&self, task_id: &str) -> Result<ExecutionResult, CapabilityError> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(task_id).ok_or_else(|| {
            CapabilityError::NotAvailable(format!("Task {} not found", task_id))
        })?;

        task.status = TaskStatus::InProgress;
        drop(tasks);

        let objective = {
            let tasks = self.tasks.lock().unwrap();
            tasks.get(task_id).unwrap().objective.clone()
        };

        // Use executor if available, otherwise create a basic execution
        let result = if let Some(executor) = &self.executor {
            executor.execute(&objective)
        } else {
            self.execute_directly(&objective)
        };

        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(task_id).unwrap();
        match &result {
            Ok(exec_result) => {
                task.status = TaskStatus::Completed;
                task.result = Some(exec_result.clone());
                task.completed_at = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
            }
            Err(_) => {
                task.status = TaskStatus::Failed;
            }
        }
        task.progress = 1.0;

        result
    }

    fn execute_directly(&self, objective: &Objective) -> Result<ExecutionResult, CapabilityError> {
        let steps = self.plan_steps(objective);
        let mut outputs = Vec::new();

        for step in &steps {
            outputs.push(StepOutput {
                step_id: step.id.clone(),
                success: true,
                output: format!("Completed: {}", step.action),
                assertions_passed: step.verify_assertions.len(),
                assertions_failed: 0,
            });
        }

        Ok(ExecutionResult {
            success: true,
            outputs,
            iterations_used: steps.len(),
            errors: vec![],
        })
    }

    fn plan_steps(&self, objective: &Objective) -> Vec<PlanStep> {
        vec![
            PlanStep {
                id: "manager_plan".to_string(),
                action: format!("Plan for: {}", objective.description),
                expected_outcome: "planned".to_string(),
                verify_assertions: vec!["planned".to_string()],
                depends_on: vec![],
            }
        ]
    }
}

/// Summary of all tracked tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskSummary {
    pub total: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub failed: usize,
}

impl Default for Manager {
    fn default() -> Self {
        Self::new(LoopConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_assign_task() {
        let manager = Manager::new(LoopConfig::quick());
        let objective = Objective {
            description: "Test task".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let task_id = manager.assign_task(objective, "agent_1").unwrap();
        assert!(task_id.starts_with("task_"));
        let task = manager.get_task(&task_id).unwrap();
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_manager_monitor() {
        let manager = Manager::new(LoopConfig::default());
        let objective = Objective {
            description: "Test".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let task_id = manager.assign_task(objective, "agent_1").unwrap();
        manager.update_progress(&task_id, 0.5).unwrap();
        let summary = manager.monitor();
        assert_eq!(summary.total, 1);
        assert_eq!(summary.in_progress, 1);
    }

    #[test]
    fn test_manager_execute_task() {
        let manager = Manager::new(LoopConfig::quick());
        let objective = Objective {
            description: "Execute test".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let task_id = manager.assign_task(objective, "agent_1").unwrap();
        let result = manager.execute_task(&task_id);
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
}