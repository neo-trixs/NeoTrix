use super::message::AgentId;
use super::sub_agent::SubAgentCapability;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaskStatus {
    Pending,
    Claimed,
    InProgress,
    Complete,
    Failed,
    Blocked,
}

#[derive(Debug, Clone)]
pub struct SharedTask {
    pub id: usize,
    pub description: String,
    pub capability: SubAgentCapability,
    pub status: TaskStatus,
    pub claimed_by: Option<AgentId>,
    pub dependencies: Vec<usize>,
    pub result: Option<String>,
}

pub struct SharedTaskList {
    tasks: Vec<SharedTask>,
    next_id: usize,
}

impl SharedTaskList {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_task(
        &mut self,
        description: &str,
        capability: SubAgentCapability,
        deps: Vec<usize>,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.push(SharedTask {
            id,
            description: description.to_string(),
            capability,
            status: TaskStatus::Pending,
            claimed_by: None,
            dependencies: deps,
            result: None,
        });
        id
    }

    /// Agent calls this to pull an available task.
    pub fn claim_next(
        &mut self,
        agent: AgentId,
        capability: SubAgentCapability,
    ) -> Option<SharedTask> {
        let completed: HashSet<usize> = self
            .tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Complete | TaskStatus::Failed))
            .map(|t| t.id)
            .collect();

        let pos = self.tasks.iter().position(|t| {
            matches!(t.status, TaskStatus::Pending)
                && t.capability == capability
                && t.dependencies.iter().all(|d| completed.contains(d))
        });

        if let Some(idx) = pos {
            let task = &mut self.tasks[idx];
            task.status = TaskStatus::Claimed;
            task.claimed_by = Some(agent);
            Some(task.clone())
        } else {
            None
        }
    }

    pub fn complete_task(&mut self, id: usize, result: String) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.status = TaskStatus::Complete;
            task.result = Some(result);
        }
    }

    pub fn fail_task(&mut self, id: usize, error: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.status = TaskStatus::Failed;
            task.result = Some(error.to_string());
        }
    }

    pub fn unclaimed_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Pending))
            .count()
    }

    pub fn all_complete(&self) -> bool {
        self.tasks
            .iter()
            .all(|t| matches!(t.status, TaskStatus::Complete | TaskStatus::Failed))
    }

    pub fn summary(&self) -> String {
        let total = self.tasks.len();
        let done = self
            .tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Complete))
            .count();
        let failed = self
            .tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Failed))
            .count();
        let pending = self.unclaimed_count();
        format!(
            "SharedTaskList: {} tasks, {} done, {} failed, {} pending",
            total, done, failed, pending
        )
    }
}

impl Default for SharedTaskList {
    fn default() -> Self {
        Self::new()
    }
}
