use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// PILOT Live Steering — 实时自改进 supervisor-worker 架构
///
/// 参考: arXiv:2608.26530 "PILOT: Live Self-Improvement for Long-Horizon Agents"
/// 核心思想: 在任务执行期间实时监控并重定向 worker, 而非执行后异步处理。
///
/// 架构:
/// - Supervisor: 独立监控 worker 执行, 支持 abort/redirect
/// - Worker: 接受 supervisor 重定向
/// - SharedMemory: 失败模式 → 可复用技能

/// Worker 执行状态
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerState {
    /// 空闲
    Idle,
    /// 执行中
    Running { task_id: String, started_at: Instant },
    /// 被 supervisor 重定向
    Redirected { from_task: String, to_task: String },
    /// 完成
    Completed { task_id: String, duration: Duration },
    /// 失败
    Failed { task_id: String, error: String },
}

/// 失败模式记录
#[derive(Debug, Clone)]
pub struct FailurePattern {
    pub pattern_id: String,
    pub description: String,
    pub frequency: u32,
    pub last_seen: Instant,
    pub suggested_fix: Option<String>,
    pub related_skills: Vec<String>,
}

/// Worker 执行轨迹点
#[derive(Debug, Clone)]
pub struct TracePoint {
    pub timestamp: Instant,
    pub action: String,
    pub result: TraceResult,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceResult {
    Success,
    Failure(String),
    Redirected(String),
}

/// Supervisor 决策
#[derive(Debug, Clone)]
pub enum SupervisorDecision {
    /// 继续执行
    Continue,
    /// 重定向到新任务
    Redirect { new_task: String, reason: String },
    /// 终止执行
    Abort { reason: String },
    /// 记录失败模式
    RecordFailure { pattern: FailurePattern },
}

/// Supervisor 配置
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    /// 最大连续失败次数 (触发重定向)
    pub max_consecutive_failures: u32,
    /// 失败模式最小频率 (触发技能提取)
    pub min_pattern_frequency: u32,
    /// 监控间隔
    pub monitoring_interval: Duration,
    /// 最大执行时间
    pub max_execution_time: Duration,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            max_consecutive_failures: 3,
            min_pattern_frequency: 2,
            monitoring_interval: Duration::from_millis(100),
            max_execution_time: Duration::from_secs(300),
        }
    }
}

/// PILOT Supervisor — 实时监控并重定向 worker
pub struct PilotSupervisor {
    config: SupervisorConfig,
    worker_state: Arc<Mutex<WorkerState>>,
    traces: Arc<Mutex<Vec<TracePoint>>>,
    failure_patterns: Arc<Mutex<HashMap<String, FailurePattern>>>,
    consecutive_failures: u32,
}

impl PilotSupervisor {
    pub fn new(config: SupervisorConfig) -> Self {
        Self {
            config,
            worker_state: Arc::new(Mutex::new(WorkerState::Idle)),
            traces: Arc::new(Mutex::new(Vec::new())),
            failure_patterns: Arc::new(Mutex::new(HashMap::new())),
            consecutive_failures: 0,
        }
    }

    /// 创建 supervisor 并启动 worker 任务
    pub fn launch_worker(&mut self, task_id: String) {
        let mut state = self.worker_state.lock().unwrap();
        *state = WorkerState::Running {
            task_id,
            started_at: Instant::now(),
        };
        self.consecutive_failures = 0;
    }

    /// 记录 worker 执行轨迹点
    pub fn record_trace(&self, point: TracePoint) {
        let mut traces = self.traces.lock().unwrap();
        traces.push(point);
    }

    /// 评估当前状态并做出决策
    pub fn evaluate(&mut self) -> SupervisorDecision {
        let state = self.worker_state.lock().unwrap().clone();

        match state {
            WorkerState::Idle => SupervisorDecision::Continue,
            WorkerState::Running { task_id, started_at } => {
                // 检查超时
                if started_at.elapsed() > self.config.max_execution_time {
                    return SupervisorDecision::Abort {
                        reason: format!("Task {} timed out after {:?}", task_id, self.config.max_execution_time),
                    };
                }

                // 分析轨迹中的失败模式
                let traces = self.traces.lock().unwrap();
                let recent_failures: Vec<_> = traces
                    .iter()
                    .rev()
                    .take(10)
                    .filter(|t| t.result == TraceResult::Success)
                    .collect();

                if recent_failures.len() < 3 {
                    // 检查是否有连续失败
                    let consecutive = traces
                        .iter()
                        .rev()
                        .take_while(|t| matches!(t.result, TraceResult::Failure(_)))
                        .count() as u32;

                    if consecutive >= self.config.max_consecutive_failures {
                        self.consecutive_failures = consecutive;
                        return SupervisorDecision::Redirect {
                            new_task: format!("recovery_{}", task_id),
                            reason: format!("{} consecutive failures detected", consecutive),
                        };
                    }
                }

                SupervisorDecision::Continue
            }
            WorkerState::Redirected { .. } => SupervisorDecision::Continue,
            WorkerState::Completed { .. } | WorkerState::Failed { .. } => SupervisorDecision::Continue,
        }
    }

    /// 提取失败模式并记录到共享内存
    pub fn extract_failure_patterns(&self) -> Vec<FailurePattern> {
        let traces = self.traces.lock().unwrap();
        let mut patterns = self.failure_patterns.lock().unwrap();
        let mut new_patterns = Vec::new();

        // 按错误类型聚合失败
        let mut error_groups: HashMap<String, Vec<&TracePoint>> = HashMap::new();
        for trace in traces.iter() {
            if let TraceResult::Failure(ref error) = trace.result {
                error_groups
                    .entry(error.clone())
                    .or_default()
                    .push(trace);
            }
        }

        for (error, points) in error_groups {
            let frequency = points.len() as u32;
            if frequency >= self.config.min_pattern_frequency {
                let pattern_id = format!("fp_{}_{}", error.len(), frequency);
                let pattern = FailurePattern {
                    pattern_id: pattern_id.clone(),
                    description: error,
                    frequency,
                    last_seen: points.last().unwrap().timestamp,
                    suggested_fix: None,
                    related_skills: Vec::new(),
                };
                patterns.insert(pattern_id.clone(), pattern.clone());
                new_patterns.push(pattern);
            }
        }

        new_patterns
    }

    /// 获取所有已知失败模式
    pub fn get_failure_patterns(&self) -> Vec<FailurePattern> {
        self.failure_patterns.lock().unwrap().values().cloned().collect()
    }

    /// 获取当前 worker 状态
    pub fn get_worker_state(&self) -> WorkerState {
        self.worker_state.lock().unwrap().clone()
    }

    /// 获取执行轨迹
    pub fn get_traces(&self) -> Vec<TracePoint> {
        self.traces.lock().unwrap().clone()
    }
}

/// PILOT Worker — 接受 supervisor 重定向的任务执行器
pub struct PilotWorker {
    id: String,
    current_task: Option<String>,
    skills: HashMap<String, String>, // skill_id -> skill_content
}

impl PilotWorker {
    pub fn new(id: String) -> Self {
        Self {
            id,
            current_task: None,
            skills: HashMap::new(),
        }
    }

    /// 接受新任务
    pub fn accept_task(&mut self, task_id: String) {
        self.current_task = Some(task_id);
    }

    /// 从失败模式中学习并更新技能
    pub fn learn_from_failure(&mut self, pattern: &FailurePattern) {
        if let Some(ref fix) = pattern.suggested_fix {
            self.skills.insert(pattern.pattern_id.clone(), fix.clone());
        }
    }

    /// 获取当前任务
    pub fn get_current_task(&self) -> Option<&str> {
        self.current_task.as_deref()
    }

    /// 获取已学技能
    pub fn get_skills(&self) -> &HashMap<String, String> {
        &self.skills
    }
}

/// 共享内存 — 失败模式和技能的共享存储
pub struct SharedMemory {
    failure_patterns: HashMap<String, FailurePattern>,
    skills: HashMap<String, String>,
    pattern_to_skill: HashMap<String, String>, // pattern_id -> skill_id
}

impl SharedMemory {
    pub fn new() -> Self {
        Self {
            failure_patterns: HashMap::new(),
            skills: HashMap::new(),
            pattern_to_skill: HashMap::new(),
        }
    }

    /// 存储失败模式
    pub fn store_failure_pattern(&mut self, pattern: FailurePattern) {
        self.failure_patterns.insert(pattern.pattern_id.clone(), pattern);
    }

    /// 从失败模式提取技能
    pub fn extract_skill(&mut self, pattern_id: &str, skill_content: String) -> Option<String> {
        let skill_id = format!("skill_{}", pattern_id);
        self.skills.insert(skill_id.clone(), skill_content);
        self.pattern_to_skill.insert(pattern_id.to_string(), skill_id.clone());
        Some(skill_id)
    }

    /// 获取失败模式对应的技能
    pub fn get_skill_for_pattern(&self, pattern_id: &str) -> Option<&str> {
        self.pattern_to_skill
            .get(pattern_id)
            .and_then(|skill_id| self.skills.get(skill_id))
            .map(|s| s.as_str())
    }

    /// 获取所有失败模式
    pub fn get_failure_patterns(&self) -> &HashMap<String, FailurePattern> {
        &self.failure_patterns
    }

    /// 获取所有技能
    pub fn get_skills(&self) -> &HashMap<String, String> {
        &self.skills
    }
}

/// PILOT 系统 — 整合 supervisor, worker, 共享内存
pub struct PilotSystem {
    pub supervisor: PilotSupervisor,
    pub workers: Vec<PilotWorker>,
    pub shared_memory: SharedMemory,
}

impl PilotSystem {
    pub fn new(supervisor_config: SupervisorConfig, worker_count: usize) -> Self {
        let supervisor = PilotSupervisor::new(supervisor_config);
        let workers = (0..worker_count)
            .map(|i| PilotWorker::new(format!("worker_{}", i)))
            .collect();
        let shared_memory = SharedMemory::new();

        Self {
            supervisor,
            workers,
            shared_memory,
        }
    }

    /// 执行一轮 supervisor-worker 循环
    pub fn tick(&mut self) -> Vec<SupervisorDecision> {
        let mut decisions = Vec::new();

        // 1. Supervisor 评估
        let decision = self.supervisor.evaluate();
        decisions.push(decision.clone());

        // 2. 执行决策
        match &decision {
            SupervisorDecision::Redirect { new_task, reason: _ } => {
                // 重定向第一个空闲 worker
                if let Some(worker) = self.workers.iter_mut().find(|w| w.get_current_task().is_none()) {
                    worker.accept_task(new_task.clone());
                }
                // 记录重定向
                let mut state = self.supervisor.worker_state.lock().unwrap();
                if let WorkerState::Running { task_id, .. } = &*state {
                    let from = task_id.clone();
                    *state = WorkerState::Redirected {
                        from_task: from,
                        to_task: new_task.clone(),
                    };
                }
            }
            SupervisorDecision::RecordFailure { pattern } => {
                // 存储到共享内存
                self.shared_memory.store_failure_pattern(pattern.clone());
                // 尝试提取技能
                if let Some(fix) = &pattern.suggested_fix {
                    self.shared_memory.extract_skill(&pattern.pattern_id, fix.clone());
                }
            }
            _ => {}
        }

        // 3. 提取失败模式
        let new_patterns = self.supervisor.extract_failure_patterns();
        for pattern in new_patterns {
            self.shared_memory.store_failure_pattern(pattern.clone());
            // 让 worker 学习
            for worker in &mut self.workers {
                worker.learn_from_failure(&pattern);
            }
        }

        decisions
    }

    /// 获取系统状态快照
    pub fn snapshot(&self) -> PilotSnapshot {
        PilotSnapshot {
            worker_states: self.workers.iter().map(|w| {
                (w.id.clone(), w.get_current_task().map(|t| t.to_string()))
            }).collect(),
            failure_pattern_count: self.shared_memory.failure_patterns.len(),
            skill_count: self.shared_memory.skills.len(),
            trace_count: self.supervisor.get_traces().len(),
        }
    }
}

/// PILOT 系统状态快照
#[derive(Debug, Clone)]
pub struct PilotSnapshot {
    pub worker_states: Vec<(String, Option<String>)>,
    pub failure_pattern_count: usize,
    pub skill_count: usize,
    pub trace_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pilot_supervisor_launch_and_evaluate() {
        let mut supervisor = PilotSupervisor::new(SupervisorConfig::default());
        supervisor.launch_worker("task_001".to_string());

        let decision = supervisor.evaluate();
        assert!(matches!(decision, SupervisorDecision::Continue));
    }

    #[test]
    fn test_pilot_supervisor_timeout_abort() {
        let config = SupervisorConfig {
            max_execution_time: Duration::from_millis(1), // 1ms timeout
            ..Default::default()
        };
        let mut supervisor = PilotSupervisor::new(config);
        supervisor.launch_worker("task_001".to_string());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(10));

        let decision = supervisor.evaluate();
        assert!(matches!(decision, SupervisorDecision::Abort { .. }));
    }

    #[test]
    fn test_failure_pattern_extraction() {
        let mut supervisor = PilotSupervisor::new(SupervisorConfig::default());
        supervisor.launch_worker("task_001".to_string());

        // Record multiple failures with same error
        for i in 0..5 {
            supervisor.record_trace(TracePoint {
                timestamp: Instant::now(),
                action: format!("action_{}", i),
                result: TraceResult::Failure("timeout error".to_string()),
                context: HashMap::new(),
            });
        }

        let patterns = supervisor.extract_failure_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns[0].frequency >= 2);
    }

    #[test]
    fn test_shared_memory_skill_extraction() {
        let mut memory = SharedMemory::new();
        let pattern = FailurePattern {
            pattern_id: "fp_001".to_string(),
            description: "test error".to_string(),
            frequency: 3,
            last_seen: Instant::now(),
            suggested_fix: Some("retry with backoff".to_string()),
            related_skills: vec![],
        };

        memory.store_failure_pattern(pattern);
        let skill_id = memory.extract_skill("fp_001", "retry with backoff".to_string());
        assert!(skill_id.is_some());

        let skill = memory.get_skill_for_pattern("fp_001");
        assert_eq!(skill, Some("retry with backoff"));
    }

    #[test]
    fn test_worker_learn_from_failure() {
        let mut worker = PilotWorker::new("worker_0".to_string());
        let pattern = FailurePattern {
            pattern_id: "fp_001".to_string(),
            description: "test error".to_string(),
            frequency: 3,
            last_seen: Instant::now(),
            suggested_fix: Some("use fallback".to_string()),
            related_skills: vec![],
        };

        worker.learn_from_failure(&pattern);
        assert_eq!(worker.get_skills().len(), 1);
        assert_eq!(worker.get_skills().get("fp_001").map(|s| s.as_str()), Some("use fallback"));
    }

    #[test]
    fn test_pilot_system_tick() {
        let mut system = PilotSystem::new(SupervisorConfig::default(), 2);
        system.supervisor.launch_worker("task_001".to_string());

        let decisions = system.tick();
        assert!(!decisions.is_empty());
    }
}
