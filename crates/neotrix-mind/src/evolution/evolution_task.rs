use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical = 0,
    High = 1,
    Medium = 2,
    Low = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Pending,
    Active,
    Verifying,
    Committed,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetLayer {
    Self_,
    Mind,
    Core,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    ParameterTuning,
    HeuristicMutation,
    ArchitectureChange,
    CapabilityAddition,
    CapabilityRemoval,
    StrategySwitch,
}

#[derive(Debug, Clone)]
pub struct ModuleTarget {
    pub layer: TargetLayer,
    pub module_name: String,
}

#[derive(Debug, Clone)]
pub struct TaskProposal {
    pub change_type: ChangeType,
    pub expected_impact: f64,
    pub estimated_risk: f64,
    pub parameters: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub success: bool,
    pub metric_delta: HashMap<String, f64>,
    pub health_after: f64,
    pub health_before: f64,
}

#[derive(Debug, Clone)]
pub struct EvolutionTask {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub priority: TaskPriority,
    pub state: TaskState,
    pub target: ModuleTarget,
    pub proposal: TaskProposal,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub result: Option<TaskResult>,
}

impl EvolutionTask {
    pub fn new(id: u64, name: &str, target: ModuleTarget, proposal: TaskProposal, priority: TaskPriority) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            id,
            name: name.to_string(),
            description: String::new(),
            priority,
            state: TaskState::Pending,
            target,
            proposal,
            created_at: now,
            completed_at: None,
            result: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskScheduler {
    tasks: Vec<EvolutionTask>,
    next_id: u64,
    max_active: usize,
    active_count: usize,
}

impl TaskScheduler {
    pub fn new(max_active: usize) -> Self {
        Self { tasks: Vec::new(), next_id: 1, max_active, active_count: 0 }
    }

    pub fn submit(&mut self, name: &str, target: ModuleTarget, proposal: TaskProposal, priority: TaskPriority) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.push(EvolutionTask::new(id, name, target, proposal, priority));
        id
    }

    pub fn next_pending(&mut self) -> Option<EvolutionTask> {
        if self.active_count >= self.max_active {
            return None;
        }
        self.tasks.sort_by_key(|t| t.priority);
        let pos = self.tasks.iter().position(|t| t.state == TaskState::Pending);
        if let Some(idx) = pos {
            self.tasks[idx].state = TaskState::Active;
            self.active_count += 1;
            Some(self.tasks[idx].clone())
        } else {
            None
        }
    }

    pub fn complete(&mut self, id: u64, result: TaskResult) -> Option<EvolutionTask> {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.state = if result.success { TaskState::Committed } else { TaskState::Failed };
            task.completed_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            );
            task.result = Some(result);
            self.active_count = self.active_count.saturating_sub(1);
        }
        self.tasks.iter().find(|t| t.id == id).cloned()
    }

    pub fn pending_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.state == TaskState::Pending).count()
    }

    pub fn tasks_by_state(&self, state: TaskState) -> Vec<&EvolutionTask> {
        self.tasks.iter().filter(|t| t.state == state).collect()
    }

    pub fn report(&self) -> String {
        let total = self.tasks.len();
        let pending = self.pending_count();
        let active = self.tasks.iter().filter(|t| t.state == TaskState::Active).count();
        let committed = self.tasks.iter().filter(|t| t.state == TaskState::Committed).count();
        let failed = self.tasks.iter().filter(|t| t.state == TaskState::Failed).count();
        format!("Scheduler[tasks={} pending={} active={} committed={} failed={}]", total, pending, active, committed, failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_target() -> ModuleTarget {
        ModuleTarget { layer: TargetLayer::Body, module_name: "tls_fingerprint".into() }
    }

    fn sample_proposal() -> TaskProposal {
        TaskProposal { change_type: ChangeType::HeuristicMutation, expected_impact: 0.6, estimated_risk: 0.3, parameters: HashMap::new() }
    }

    #[test]
    fn test_submit_and_pending() {
        let mut sched = TaskScheduler::new(5);
        let id = sched.submit("test", sample_target(), sample_proposal(), TaskPriority::Medium);
        assert_eq!(sched.pending_count(), 1);
        let task = sched.next_pending();
        assert!(task.is_some());
        assert_eq!(task.unwrap().id, id);
    }

    #[test]
    fn test_max_active_limit() {
        let mut sched = TaskScheduler::new(2);
        sched.submit("a", sample_target(), sample_proposal(), TaskPriority::Low);
        sched.submit("b", sample_target(), sample_proposal(), TaskPriority::Low);
        sched.submit("c", sample_target(), sample_proposal(), TaskPriority::Low);
        assert!(sched.next_pending().is_some());
        assert!(sched.next_pending().is_some());
        assert!(sched.next_pending().is_none());
    }

    #[test]
    fn test_complete_releases_slot() {
        let mut sched = TaskScheduler::new(1);
        let id = sched.submit("t", sample_target(), sample_proposal(), TaskPriority::High);
        let task = sched.next_pending().unwrap();
        sched.complete(id, TaskResult { success: true, metric_delta: HashMap::new(), health_after: 0.9, health_before: 0.7 });
        let next = sched.next_pending();
        assert!(next.is_none());
    }

    #[test]
    fn test_priority_ordering() {
        let mut sched = TaskScheduler::new(5);
        sched.submit("low", sample_target(), sample_proposal(), TaskPriority::Low);
        sched.submit("high", sample_target(), sample_proposal(), TaskPriority::High);
        sched.submit("critical", sample_target(), sample_proposal(), TaskPriority::Critical);
        assert_eq!(sched.next_pending().unwrap().name, "critical");
        assert_eq!(sched.next_pending().unwrap().name, "high");
        assert_eq!(sched.next_pending().unwrap().name, "low");
    }

    #[test]
    fn test_report_format() {
        let mut sched = TaskScheduler::new(3);
        sched.submit("a", sample_target(), sample_proposal(), TaskPriority::Medium);
        let report = sched.report();
        assert!(report.contains("pending=1"));
    }

    #[test]
    fn test_tasks_by_state_filter() {
        let mut sched = TaskScheduler::new(3);
        let id = sched.submit("t", sample_target(), sample_proposal(), TaskPriority::Medium);
        assert_eq!(sched.tasks_by_state(TaskState::Pending).len(), 1);
        sched.complete(id, TaskResult { success: true, metric_delta: HashMap::new(), health_after: 1.0, health_before: 0.5 });
        assert_eq!(sched.tasks_by_state(TaskState::Committed).len(), 1);
    }

    #[test]
    fn test_task_creation_timestamp() {
        let target = sample_target();
        let proposal = sample_proposal();
        let task = EvolutionTask::new(42, "test_task", target, proposal, TaskPriority::Medium);
        assert_eq!(task.id, 42);
        assert!(task.created_at > 0);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_complete_failed_task() {
        let mut sched = TaskScheduler::new(3);
        let id = sched.submit("fail", sample_target(), sample_proposal(), TaskPriority::Low);
        sched.next_pending();
        let completed = sched.complete(id, TaskResult { success: false, metric_delta: HashMap::new(), health_after: 0.3, health_before: 0.5 });
        assert!(completed.unwrap().result.unwrap().success == false);
    }
}
