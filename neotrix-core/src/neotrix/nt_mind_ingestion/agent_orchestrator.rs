use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct AgentTask {
    pub id: String,
    pub description: String,
    pub priority: u8,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct AgentResult {
    pub task_id: String,
    pub output: String,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct WorkerAgent {
    pub id: String,
    pub specialty: String,
    pub task_queue: VecDeque<AgentTask>,
    pub is_busy: bool,
}

impl WorkerAgent {
    pub fn new(id: &str, specialty: &str) -> Self {
        Self {
            id: id.to_string(),
            specialty: specialty.to_string(),
            task_queue: VecDeque::new(),
            is_busy: false,
        }
    }

    pub fn assign(&mut self, task: AgentTask) {
        self.task_queue.push_back(task);
        self.is_busy = true;
    }

    pub fn tick(&mut self) -> Option<AgentResult> {
        let task = self.task_queue.pop_front()?;
        self.is_busy = !self.task_queue.is_empty();
        Some(AgentResult {
            task_id: task.id,
            output: format!("processed by {} ({})", self.id, self.specialty),
            success: true,
        })
    }

    pub fn queue_len(&self) -> usize {
        self.task_queue.len()
    }
}

#[derive(Debug, Clone)]
pub struct ManagerOrchestrator {
    pub workers: Vec<WorkerAgent>,
    pub task_queue: VecDeque<AgentTask>,
}

impl ManagerOrchestrator {
    pub fn new() -> Self {
        Self {
            workers: Vec::new(),
            task_queue: VecDeque::new(),
        }
    }

    pub fn add_worker(&mut self, specialty: &str) {
        let id = format!("worker_{}", self.workers.len() + 1);
        self.workers.push(WorkerAgent::new(&id, specialty));
    }

    pub fn dispatch(&mut self, task: AgentTask) -> Result<(), String> {
        if self.workers.is_empty() {
            return Err("No workers available".to_string());
        }
        self.task_queue.push_back(task);
        Ok(())
    }

    pub fn tick(&mut self) -> Vec<AgentResult> {
        let mut results = Vec::new();
        for worker in &mut self.workers {
            if let Some(result) = worker.tick() {
                results.push(result);
            }
        }
        let mut still_pending = VecDeque::new();
        while let Some(task) = self.task_queue.pop_front() {
            match self.workers.iter_mut().min_by_key(|w| w.queue_len()) {
                Some(worker) => worker.assign(task),
                None => still_pending.push_back(task),
            }
        }
        self.task_queue = still_pending;
        results
    }

    pub fn collect_results(&mut self) -> Vec<AgentResult> {
        self.tick()
    }

    pub fn workers_by_specialty(&self, specialty: &str) -> Vec<&WorkerAgent> {
        self.workers.iter().filter(|w| w.specialty == specialty).collect()
    }
}

impl Default for ManagerOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_dispatch() {
        let mut worker = WorkerAgent::new("test_worker", "testing");
        assert!(!worker.is_busy);
        let task = AgentTask {
            id: "task_1".into(),
            description: "test task".into(),
            priority: 1,
            created_at: 0,
        };
        worker.assign(task);
        assert!(worker.is_busy);
        let result = worker.tick().unwrap();
        assert_eq!(result.task_id, "task_1");
        assert!(result.success);
        assert!(!worker.is_busy);
    }

    #[test]
    fn test_task_fifo_order() {
        let mut orchestrator = ManagerOrchestrator::new();
        orchestrator.add_worker("testing");
        for i in 0..3 {
            orchestrator.dispatch(AgentTask {
                id: format!("task_{}", i),
                description: format!("task {}", i),
                priority: 1,
                created_at: i,
            }).unwrap();
        }
        let r1 = orchestrator.tick();
        assert_eq!(r1[0].task_id, "task_0");
        let r2 = orchestrator.tick();
        assert_eq!(r2[0].task_id, "task_1");
        let r3 = orchestrator.tick();
        assert_eq!(r3[0].task_id, "task_2");
    }

    #[test]
    fn test_collect_results() {
        let mut orchestrator = ManagerOrchestrator::new();
        orchestrator.add_worker("alpha");
        orchestrator.add_worker("beta");
        orchestrator.dispatch(AgentTask {
            id: "t1".into(), description: "".into(), priority: 1, created_at: 0,
        }).unwrap();
        orchestrator.dispatch(AgentTask {
            id: "t2".into(), description: "".into(), priority: 1, created_at: 0,
        }).unwrap();
        let results = orchestrator.collect_results();
        assert_eq!(results.len(), 2);
        let ids: Vec<&str> = results.iter().map(|r| r.task_id.as_str()).collect();
        assert!(ids.contains(&"t1"));
        assert!(ids.contains(&"t2"));
    }

    #[test]
    fn test_orchestrator_tick_distributes_across_workers() {
        let mut orchestrator = ManagerOrchestrator::new();
        orchestrator.add_worker("alpha");
        orchestrator.add_worker("beta");
        orchestrator.dispatch(AgentTask {
            id: "a1".into(), description: "".into(), priority: 1, created_at: 0,
        }).unwrap();
        orchestrator.dispatch(AgentTask {
            id: "a2".into(), description: "".into(), priority: 1, created_at: 0,
        }).unwrap();
        orchestrator.dispatch(AgentTask {
            id: "a3".into(), description: "".into(), priority: 1, created_at: 0,
        }).unwrap();
        let r1 = orchestrator.tick();
        assert_eq!(r1.len(), 2);
        let r2 = orchestrator.tick();
        assert_eq!(r2.len(), 1);
    }

    #[test]
    fn test_workers_by_specialty() {
        let mut orchestrator = ManagerOrchestrator::new();
        orchestrator.add_worker("code_review");
        orchestrator.add_worker("architecture");
        orchestrator.add_worker("code_review");
        let reviewers = orchestrator.workers_by_specialty("code_review");
        assert_eq!(reviewers.len(), 2);
        let architects = orchestrator.workers_by_specialty("architecture");
        assert_eq!(architects.len(), 1);
        let none = orchestrator.workers_by_specialty("research");
        assert!(none.is_empty());
    }

    #[test]
    fn test_dispatch_no_workers_returns_error() {
        let mut orchestrator = ManagerOrchestrator::new();
        let result = orchestrator.dispatch(AgentTask {
            id: "orphan".into(), description: "".into(), priority: 1, created_at: 0,
        });
        assert!(result.is_err());
    }
}
