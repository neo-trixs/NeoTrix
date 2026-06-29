use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub struct AgentBusConfig {
    pub max_messages: usize,
}

impl Default for AgentBusConfig {
    fn default() -> Self {
        Self { max_messages: 5000 }
    }
}

pub type WorkerId = String;

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BusTopic {
    TaskRequest,
    TaskClaim,
    TaskResult,
    SupervisorBroadcast,
    WorkerHeartbeat,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct BusTask {
    pub id: u64,
    pub supervisor_id: String,
    pub description: String,
    pub required_capabilities: Vec<String>,
    pub context: HashMap<String, String>,
    pub created_at: Instant,
}

impl BusTask {
    pub fn new(supervisor_id: &str, description: &str) -> Self {
        Self {
            id: NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst),
            supervisor_id: supervisor_id.to_string(),
            description: description.to_string(),
            required_capabilities: Vec::new(),
            context: HashMap::new(),
            created_at: Instant::now(),
        }
    }

    pub fn with_capability(mut self, cap: &str) -> Self {
        self.required_capabilities.push(cap.to_string());
        self
    }

    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct TaskClaim {
    pub task_id: u64,
    pub worker_id: String,
    pub worker_capabilities: Vec<String>,
    pub claimed_at: Instant,
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: u64,
    pub worker_id: String,
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AgentBusMessage {
    TaskAvailable(BusTask),
    TaskClaimed(TaskClaim),
    TaskCompleted(TaskResult),
    SupervisorDirective(String),
    WorkerStatus {
        worker_id: String,
        busy: bool,
        capabilities: Vec<String>,
    },
    Heartbeat {
        agent_id: String,
        timestamp: Instant,
    },
    Custom {
        topic: String,
        payload: String,
    },
}

impl AgentBusMessage {
    fn is_expired(&self, max_age: Duration) -> bool {
        let cutoff = Instant::now() - max_age;
        let ts = match self {
            AgentBusMessage::Heartbeat { timestamp, .. } => *timestamp,
            AgentBusMessage::TaskAvailable(ref t) => t.created_at,
            AgentBusMessage::TaskClaimed(ref c) => c.claimed_at,
            _ => return false,
        };
        ts < cutoff
    }
}

#[derive(Debug, Clone, Default)]
pub struct BusStats {
    pub tasks_published: u64,
    pub tasks_claimed: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub workers_registered: u64,
    pub active_workers: u64,
    pub messages_cleaned: u64,
    pub message_queue_len: usize,
}

const MAX_SUBSCRIBERS: usize = 100;

pub struct AgentBus {
    messages: Arc<RwLock<Vec<(BusTopic, AgentBusMessage)>>>,
    subscriptions: Arc<RwLock<HashMap<BusTopic, Vec<String>>>>,
    stats: Arc<RwLock<BusStats>>,
    workers: Arc<RwLock<HashMap<String, Vec<String>>>>,
    max_messages: usize,
}

impl AgentBus {
    pub fn new() -> Self {
        Self::with_config(AgentBusConfig::default())
    }

    pub fn with_config(config: AgentBusConfig) -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(BusStats::default())),
            workers: Arc::new(RwLock::new(HashMap::new())),
            max_messages: config.max_messages,
        }
    }

    pub fn publish(&self, topic: BusTopic, msg: AgentBusMessage) {
        if let Ok(mut msgs) = self.messages.write() {
            msgs.push((topic.clone(), msg.clone()));
        }
        if let Ok(mut stats) = self.stats.write() {
            match &msg {
                AgentBusMessage::TaskAvailable(_) => stats.tasks_published += 1,
                AgentBusMessage::TaskClaimed(_) => stats.tasks_claimed += 1,
                AgentBusMessage::TaskCompleted(r) => {
                    if r.success {
                        stats.tasks_completed += 1;
                    } else {
                        stats.tasks_failed += 1;
                    }
                }
                AgentBusMessage::WorkerStatus { .. } => {
                    stats.active_workers += 1;
                }
                _ => {}
            }
        }
    }

    pub fn subscribe(&self, agent_id: &str, topic: BusTopic) {
        if let Ok(mut subs) = self.subscriptions.write() {
            let entries = subs.entry(topic).or_default();
            if entries.len() >= MAX_SUBSCRIBERS {
                return;
            }
            entries.push(agent_id.to_string());
        }
        if let Ok(mut workers) = self.workers.write() {
            workers.entry(agent_id.to_string()).or_default();
        }
        if let Ok(mut stats) = self.stats.write() {
            stats.workers_registered = self.workers.read().map(|w| w.len() as u64).unwrap_or(0);
        }
    }

    pub fn register_worker(&self, worker_id: &str, capabilities: Vec<String>) {
        if let Ok(mut workers) = self.workers.write() {
            workers.insert(worker_id.to_string(), capabilities.clone());
        }
        if let Ok(mut stats) = self.stats.write() {
            stats.workers_registered = self.workers.read().map(|w| w.len() as u64).unwrap_or(0);
        }
    }

    pub fn unregister_worker(&self, worker_id: &str) {
        if let Ok(mut workers) = self.workers.write() {
            workers.remove(worker_id);
        }
        if let Ok(mut subs) = self.subscriptions.write() {
            for v in subs.values_mut() {
                v.retain(|id| id != worker_id);
            }
        }
    }

    pub fn poll_topic(&self, topic: &BusTopic) -> Vec<AgentBusMessage> {
        if let Ok(msgs) = self.messages.read() {
            msgs.iter()
                .filter(|(t, _)| t == topic)
                .map(|(_, m)| m.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Dispatch a task to the worker with the best EFC efficiency for the required capability
    pub fn dispatch_by_efc(
        &self,
        topic: &str,
        payload: &str,
        required_capability: &str,
    ) -> Result<(), String> {
        let workers = self
            .workers
            .read()
            .map_err(|e| format!("lock error: {}", e))?;
        let worker_ids: Vec<String> = workers.keys().cloned().collect();
        for id in &worker_ids {
            if let Some(caps) = workers.get(id) {
                if caps.iter().any(|c| c == required_capability) {
                    self.publish(
                        BusTopic::Custom(topic.to_string()),
                        AgentBusMessage::Custom {
                            topic: topic.to_string(),
                            payload: payload.to_string(),
                        },
                    );
                    return Ok(());
                }
            }
        }
        Err(format!(
            "no worker found with capability: {}",
            required_capability
        ))
    }

    pub fn find_worker_for_task(&self, required: &[String]) -> Option<(String, Vec<String>)> {
        if let Ok(workers) = self.workers.read() {
            let best = workers
                .iter()
                .filter(|(_, caps)| required.iter().all(|r| caps.contains(r)))
                .max_by_key(|(_, caps)| caps.len());
            best.map(|(id, caps)| (id.clone(), caps.clone()))
        } else {
            None
        }
    }

    pub fn all_workers(&self) -> Vec<String> {
        self.workers
            .read()
            .map(|w| w.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn worker_capabilities(&self, worker_id: &str) -> Option<Vec<String>> {
        self.workers
            .read()
            .ok()
            .and_then(|w| w.get(worker_id).cloned())
    }

    pub fn stats(&self) -> BusStats {
        self.stats
            .read()
            .as_ref()
            .map(|s| BusStats {
                tasks_published: s.tasks_published,
                tasks_claimed: s.tasks_claimed,
                tasks_completed: s.tasks_completed,
                tasks_failed: s.tasks_failed,
                workers_registered: s.workers_registered,
                active_workers: s.active_workers,
                messages_cleaned: s.messages_cleaned,
                message_queue_len: s.message_queue_len,
            })
            .unwrap_or_default()
    }

    pub fn clear_old_messages(&self, max_age: Duration) {
        if let Ok(mut msgs) = self.messages.write() {
            let before = msgs.len();
            msgs.retain(|(_, m)| !m.is_expired(max_age));
            let n = msgs.len();
            if n > self.max_messages {
                msgs.drain(0..n - self.max_messages);
            }
            if let Ok(mut stats) = self.stats.write() {
                stats.messages_cleaned += (before - msgs.len()) as u64;
                stats.message_queue_len = msgs.len();
            }
        }
    }
}

impl Default for AgentBus {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SupervisorAgent {
    id: String,
    bus: Arc<AgentBus>,
    workers: HashMap<WorkerId, Vec<String>>,
    worker_efc: HashMap<WorkerId, f64>,
}

impl SupervisorAgent {
    pub fn new(id: &str, bus: Arc<AgentBus>) -> Self {
        bus.subscribe(id, BusTopic::TaskClaim);
        bus.subscribe(id, BusTopic::TaskResult);
        bus.subscribe(id, BusTopic::WorkerHeartbeat);
        Self {
            id: id.to_string(),
            bus,
            workers: HashMap::new(),
            worker_efc: HashMap::new(),
        }
    }

    pub fn register_worker(&mut self, worker_id: &str, capabilities: Vec<String>) {
        self.workers.insert(worker_id.to_string(), capabilities);
        self.worker_efc.entry(worker_id.to_string()).or_insert(0.0);
    }

    pub fn update_worker_efc(&mut self, worker_id: &str, avg_efficiency: f64) {
        self.worker_efc
            .insert(worker_id.to_string(), avg_efficiency);
    }

    /// Select the best worker for a task, preferring high-efficiency workers
    pub fn select_worker_by_efc(&self, required_capability: &str) -> Option<WorkerId> {
        self.workers
            .iter()
            .filter(|(_, caps)| caps.iter().any(|c| c == required_capability))
            .max_by(|(a_id, _), (b_id, _)| {
                let ea = self.worker_efc.get(*a_id).copied().unwrap_or(0.0);
                let eb = self.worker_efc.get(*b_id).copied().unwrap_or(0.0);
                ea.partial_cmp(&eb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, _)| id.clone())
    }

    pub fn dispatch_task(&self, description: &str) -> BusTask {
        let task = BusTask::new(&self.id, description);
        self.bus.publish(
            BusTopic::TaskRequest,
            AgentBusMessage::TaskAvailable(task.clone()),
        );
        task
    }

    pub fn dispatch_task_with_capabilities(
        &self,
        description: &str,
        capabilities: &[&str],
    ) -> BusTask {
        let mut task = BusTask::new(&self.id, description);
        for cap in capabilities {
            task = task.with_capability(cap);
        }
        self.bus.publish(
            BusTopic::TaskRequest,
            AgentBusMessage::TaskAvailable(task.clone()),
        );
        task
    }

    pub fn broadcast(&self, directive: &str) {
        self.bus.publish(
            BusTopic::SupervisorBroadcast,
            AgentBusMessage::SupervisorDirective(directive.to_string()),
        );
    }

    pub fn poll_results(&self) -> Vec<TaskResult> {
        self.bus
            .poll_topic(&BusTopic::TaskResult)
            .into_iter()
            .filter_map(|m| match m {
                AgentBusMessage::TaskCompleted(r) => Some(r),
                _ => None,
            })
            .collect()
    }

    pub fn poll_claims(&self) -> Vec<TaskClaim> {
        self.bus
            .poll_topic(&BusTopic::TaskClaim)
            .into_iter()
            .filter_map(|m| match m {
                AgentBusMessage::TaskClaimed(c) => Some(c),
                _ => None,
            })
            .collect()
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

pub struct WorkerAgent {
    id: String,
    capabilities: Vec<String>,
    bus: Arc<AgentBus>,
    current_task: Option<u64>,
    /// Worker's EFC history (last N efficiency values)
    pub efc_history: VecDeque<f64>,
    /// Average EFC efficiency (η) for this worker
    pub avg_efficiency: f64,
    /// Total tasks completed
    pub tasks_completed: u64,
}

impl WorkerAgent {
    pub fn new(id: &str, capabilities: Vec<String>, bus: Arc<AgentBus>) -> Self {
        bus.register_worker(id, capabilities.clone());
        bus.subscribe(id, BusTopic::TaskRequest);
        bus.subscribe(id, BusTopic::SupervisorBroadcast);
        Self {
            id: id.to_string(),
            capabilities,
            bus,
            current_task: None,
            efc_history: VecDeque::with_capacity(100),
            avg_efficiency: 0.0,
            tasks_completed: 0,
        }
    }

    pub fn poll_for_tasks(&self) -> Vec<BusTask> {
        let msgs = self.bus.poll_topic(&BusTopic::TaskRequest);
        msgs.into_iter()
            .filter_map(|m| match m {
                AgentBusMessage::TaskAvailable(task) => Some(task),
                _ => None,
            })
            .collect()
    }

    pub fn claim_task(&mut self, task: &BusTask) -> bool {
        if self.current_task.is_some() {
            return false;
        }
        let has_all_caps = task
            .required_capabilities
            .iter()
            .all(|r| self.capabilities.contains(r));
        if !has_all_caps {
            return false;
        }
        self.current_task = Some(task.id);
        let claim = TaskClaim {
            task_id: task.id,
            worker_id: self.id.clone(),
            worker_capabilities: self.capabilities.clone(),
            claimed_at: Instant::now(),
        };
        self.bus
            .publish(BusTopic::TaskClaim, AgentBusMessage::TaskClaimed(claim));
        true
    }

    pub fn complete_task(&mut self, success: bool, output: &str, error: Option<&str>) {
        if let Some(task_id) = self.current_task.take() {
            let result = TaskResult {
                task_id,
                worker_id: self.id.clone(),
                success,
                output: output.to_string(),
                duration_ms: 0,
                error: error.map(|e| e.to_string()),
            };
            self.bus
                .publish(BusTopic::TaskResult, AgentBusMessage::TaskCompleted(result));
        }
    }

    pub fn send_heartbeat(&self) {
        self.bus.publish(
            BusTopic::WorkerHeartbeat,
            AgentBusMessage::Heartbeat {
                agent_id: self.id.clone(),
                timestamp: Instant::now(),
            },
        );
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    pub fn is_busy(&self) -> bool {
        self.current_task.is_some()
    }

    pub fn record_task_completion(&mut self, efc: f64, raw_cost: f64) {
        let eta = if raw_cost > 0.0 { efc / raw_cost } else { 0.0 };
        self.efc_history.push_back(eta);
        if self.efc_history.len() > 100 {
            self.efc_history.pop_front();
        }
        self.tasks_completed += 1;
        let sum: f64 = self.efc_history.iter().sum();
        self.avg_efficiency = sum / (self.efc_history.len() as f64).max(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_publish_and_poll() {
        let bus = Arc::new(AgentBus::new());
        let task = BusTask::new("supervisor-1", "analyze code");
        bus.publish(
            BusTopic::TaskRequest,
            AgentBusMessage::TaskAvailable(task.clone()),
        );

        let msgs = bus.poll_topic(&BusTopic::TaskRequest);
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_bus_stats() {
        let bus = Arc::new(AgentBus::new());
        let task = BusTask::new("supervisor-1", "test");
        bus.publish(BusTopic::TaskRequest, AgentBusMessage::TaskAvailable(task));
        bus.publish(
            BusTopic::TaskResult,
            AgentBusMessage::TaskCompleted(TaskResult {
                task_id: 0,
                worker_id: "w1".into(),
                success: true,
                output: "ok".into(),
                duration_ms: 10,
                error: None,
            }),
        );
        bus.publish(
            BusTopic::TaskResult,
            AgentBusMessage::TaskCompleted(TaskResult {
                task_id: 1,
                worker_id: "w1".into(),
                success: false,
                output: "".into(),
                duration_ms: 5,
                error: Some("fail".into()),
            }),
        );

        let stats = bus.stats();
        assert_eq!(stats.tasks_published, 1);
        assert_eq!(stats.tasks_completed, 1);
        assert_eq!(stats.tasks_failed, 1);
    }

    #[test]
    fn test_worker_registration() {
        let bus = Arc::new(AgentBus::new());
        bus.register_worker("worker-1", vec!["analysis".into(), "code".into()]);
        bus.register_worker("worker-2", vec!["review".into()]);

        let all = bus.all_workers();
        assert_eq!(all.len(), 2);

        let caps = bus.worker_capabilities("worker-1");
        assert!(caps.is_some());
        assert_eq!(caps.unwrap().len(), 2);
    }

    #[test]
    fn test_find_worker_for_task() {
        let bus = Arc::new(AgentBus::new());
        bus.register_worker("worker-a", vec!["analysis".into(), "code".into()]);
        bus.register_worker("worker-b", vec!["review".into()]);

        let found = bus.find_worker_for_task(&["analysis".to_string(), "code".to_string()]);
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, "worker-a");
    }

    #[test]
    fn test_supervisor_dispatch() {
        let bus = Arc::new(AgentBus::new());
        let supervisor = SupervisorAgent::new("sup-1", bus.clone());

        let task = supervisor.dispatch_task("find bugs");
        assert!(task.id > 0);

        let msgs = bus.poll_topic(&BusTopic::TaskRequest);
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_supervisor_broadcast() {
        let bus = Arc::new(AgentBus::new());
        let supervisor = SupervisorAgent::new("sup-1", bus.clone());

        supervisor.broadcast("stop all work");
        let msgs = bus.poll_topic(&BusTopic::SupervisorBroadcast);
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_worker_claims_and_completes() {
        let bus = Arc::new(AgentBus::new());
        let supervisor = SupervisorAgent::new("sup-1", bus.clone());
        let mut worker = WorkerAgent::new("worker-1", vec!["analysis".into()], bus.clone());

        let task = supervisor.dispatch_task("analyze data");
        let claimed = worker.claim_task(&task);
        assert!(claimed);
        assert!(worker.is_busy());

        worker.complete_task(true, "analysis done", None);
        assert!(!worker.is_busy());

        let results = supervisor.poll_results();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(results[0].output, "analysis done");
    }

    #[test]
    fn test_worker_rejects_missing_capabilities() {
        let bus = Arc::new(AgentBus::new());
        let supervisor = SupervisorAgent::new("sup-1", bus.clone());
        let mut worker = WorkerAgent::new("worker-1", vec!["review".into()], bus.clone());

        let task = supervisor.dispatch_task_with_capabilities("analyze", &["analysis"]);
        let claimed = worker.claim_task(&task);
        assert!(!claimed, "worker without capability should reject");
    }

    #[test]
    fn test_worker_cannot_claim_two_tasks() {
        let bus = Arc::new(AgentBus::new());
        let supervisor = SupervisorAgent::new("sup-1", bus.clone());
        let mut worker = WorkerAgent::new("worker-1", vec!["analysis".into()], bus.clone());

        let t1 = supervisor.dispatch_task("task 1");
        let t2 = supervisor.dispatch_task("task 2");

        assert!(worker.claim_task(&t1));
        assert!(!worker.claim_task(&t2), "should not claim while busy");
    }

    #[test]
    fn test_heartbeat() {
        let bus = Arc::new(AgentBus::new());
        let worker = WorkerAgent::new("worker-1", vec![], bus.clone());
        worker.send_heartbeat();

        let heartbeats = bus.poll_topic(&BusTopic::WorkerHeartbeat);
        assert_eq!(heartbeats.len(), 1);
    }

    #[test]
    fn test_subscribe_and_unregister() {
        let bus = Arc::new(AgentBus::new());
        bus.register_worker("w1", vec![]);
        assert_eq!(bus.all_workers().len(), 1);

        bus.unregister_worker("w1");
        assert_eq!(bus.all_workers().len(), 0);
    }

    #[test]
    fn test_supervisor_polls_claims() {
        let bus = Arc::new(AgentBus::new());
        let supervisor = SupervisorAgent::new("sup-1", bus.clone());
        let mut worker = WorkerAgent::new("worker-1", vec!["analysis".into()], bus.clone());

        let task = supervisor.dispatch_task("analyze");
        worker.claim_task(&task);

        let claims = supervisor.poll_claims();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].worker_id, "worker-1");
    }

    #[test]
    fn test_task_with_context() {
        let task = BusTask::new("sup-1", "fix bug")
            .with_context("file", "src/main.rs")
            .with_context("line", "42");
        assert_eq!(task.context.get("file").unwrap(), "src/main.rs");
        assert_eq!(task.context.get("line").unwrap(), "42");
    }

    #[test]
    fn test_multiple_workers_and_worker_selection() {
        let bus = Arc::new(AgentBus::new());
        bus.register_worker("coder", vec!["code".into(), "analysis".into()]);
        bus.register_worker("reviewer", vec!["review".into()]);
        bus.register_worker(
            "fullstack",
            vec!["code".into(), "review".into(), "analysis".into()],
        );

        let found = bus.find_worker_for_task(&["code".to_string(), "review".to_string()]);
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, "fullstack");
    }

    #[test]
    fn test_clear_old_messages() {
        let bus = Arc::new(AgentBus::new());
        let worker = WorkerAgent::new("w1", vec![], bus.clone());
        worker.send_heartbeat();
        assert_eq!(bus.poll_topic(&BusTopic::WorkerHeartbeat).len(), 1);

        bus.clear_old_messages(Duration::from_secs(0));
        assert_eq!(bus.poll_topic(&BusTopic::WorkerHeartbeat).len(), 0);
    }

    #[test]
    fn test_custom_topic_messages() {
        let bus = Arc::new(AgentBus::new());
        bus.publish(
            BusTopic::Custom("log".into()),
            AgentBusMessage::Custom {
                topic: "log".into(),
                payload: "info message".into(),
            },
        );
        let msgs = bus.poll_topic(&BusTopic::Custom("log".into()));
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_bus_stats_workers_registered() {
        let bus = Arc::new(AgentBus::new());
        assert_eq!(bus.stats().workers_registered, 0);
        bus.register_worker("w1", vec![]);
        assert_eq!(bus.stats().workers_registered, 1);
        bus.register_worker("w2", vec![]);
        assert_eq!(bus.stats().workers_registered, 2);
    }

    #[test]
    fn test_task_capability_filtering() {
        let bus = Arc::new(AgentBus::new());
        let supervisor = SupervisorAgent::new("sup-1", bus.clone());
        let mut worker = WorkerAgent::new("dev", vec!["code".into(), "test".into()], bus.clone());

        let task = supervisor.dispatch_task_with_capabilities("write tests", &["code", "test"]);
        assert!(worker.claim_task(&task));

        let task2 = supervisor.dispatch_task_with_capabilities("design", &["design"]);
        assert!(!worker.claim_task(&task2));
    }

    #[test]
    fn test_worker_complete_with_error() {
        let bus = Arc::new(AgentBus::new());
        let supervisor = SupervisorAgent::new("sup-1", bus.clone());
        let mut worker = WorkerAgent::new("w1", vec![], bus.clone());

        let task = supervisor.dispatch_task("risky task");
        worker.claim_task(&task);
        worker.complete_task(false, "", Some("something went wrong"));

        let results = supervisor.poll_results();
        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
        assert_eq!(results[0].error.as_deref(), Some("something went wrong"));
    }

    #[test]
    fn test_dispatch_with_capabilities() {
        let bus = Arc::new(AgentBus::new());
        let supervisor = SupervisorAgent::new("sup-1", bus.clone());
        let task =
            supervisor.dispatch_task_with_capabilities("audit security", &["security", "review"]);
        assert_eq!(task.required_capabilities.len(), 2);
        assert!(task.required_capabilities.contains(&"security".to_string()));
    }

    #[test]
    fn test_unique_task_ids() {
        let bus = Arc::new(AgentBus::new());
        let sup = SupervisorAgent::new("sup-1", bus.clone());
        let t1 = sup.dispatch_task("task A");
        let t2 = sup.dispatch_task("task B");
        assert_ne!(t1.id, t2.id);
    }
}
