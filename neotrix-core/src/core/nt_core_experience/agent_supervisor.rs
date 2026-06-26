#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

/// 会话状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Paused,
    Idle,
    Completed,
    Failed,
    Terminated,
}

impl SessionStatus {
    pub fn name(&self) -> &'static str {
        match self {
            SessionStatus::Active => "active",
            SessionStatus::Paused => "paused",
            SessionStatus::Idle => "idle",
            SessionStatus::Completed => "completed",
            SessionStatus::Failed => "failed",
            SessionStatus::Terminated => "terminated",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            SessionStatus::Completed | SessionStatus::Failed | SessionStatus::Terminated
        )
    }
}

/// 后台任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn name(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
        }
    }
}

/// Agent 身份标识
#[derive(Debug, Clone)]
pub struct AgentIdentity {
    pub id: String,
    pub name: String,
    pub role: String,
    pub created_at: u64,
    pub last_active: u64,
    pub session_count: u32,
}

/// 后台任务定义
#[derive(Debug, Clone)]
pub struct BackgroundTask {
    pub id: String,
    pub description: String,
    pub priority: u8,
    pub status: TaskStatus,
    pub created_at: u64,
    pub deadline: Option<u64>,
}

/// 会话执行结果
#[derive(Debug, Clone)]
pub struct SessionResult {
    pub task_id: String,
    pub outcome: String,
    pub success: bool,
    pub duration_ms: u64,
    pub timestamp: u64,
}

/// Agent 会话
#[derive(Debug, Clone)]
pub struct AgentSession {
    pub session_id: String,
    pub agent_id: String,
    pub status: SessionStatus,
    pub started_at: u64,
    pub context_summary: String,
    pub task_queue: VecDeque<BackgroundTask>,
    pub result_log: Vec<SessionResult>,
}

impl AgentSession {
    fn idle_cycles(&self, current_cycle: u64, _started_at: u64) -> u64 {
        if self.started_at >= current_cycle {
            0
        } else {
            current_cycle.saturating_sub(self.started_at)
        }
    }
}

/// 监督器配置
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    pub max_agents: usize,
    pub max_sessions_per_agent: u32,
    pub idle_timeout_cycles: u64,
    pub heartbeat_interval: u64,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            max_agents: 10,
            max_sessions_per_agent: 5,
            idle_timeout_cycles: 100,
            heartbeat_interval: 10,
        }
    }
}

/// 监督器汇总统计
#[derive(Debug, Clone)]
pub struct SupervisorSummary {
    pub agent_count: usize,
    pub active_session_count: usize,
    pub total_tasks: usize,
    pub completion_rate: f64,
    pub heartbeat_count: u64,
}

/// AgentSupervisor — 后台 Agent 会话管理与持久化
///
/// 受 Claude Code agents 功能启发：管理持久化 agent 会话，
/// 每个会话拥有独立身份、记忆和后台任务队列。
pub struct AgentSupervisor {
    agents: HashMap<String, AgentIdentity>,
    sessions: HashMap<String, AgentSession>,
    config: SupervisorConfig,
    next_agent_id: u64,
    next_session_id: u64,
    heartbeat_count: u64,
}

impl AgentSupervisor {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            sessions: HashMap::new(),
            config: SupervisorConfig::default(),
            next_agent_id: 1,
            next_session_id: 1,
            heartbeat_count: 0,
        }
    }

    pub fn with_config(config: SupervisorConfig) -> Self {
        Self {
            agents: HashMap::new(),
            sessions: HashMap::new(),
            config,
            next_agent_id: 1,
            next_session_id: 1,
            heartbeat_count: 0,
        }
    }

    /// 注册一个新 agent，返回 agent_id
    pub fn register_agent(&mut self, name: &str, role: &str) -> String {
        if self.agents.len() >= self.config.max_agents {
            return String::new();
        }
        let id = format!("agent_{}", self.next_agent_id);
        self.next_agent_id += 1;
        let now = now_secs();
        self.agents.insert(
            id.clone(),
            AgentIdentity {
                id: id.clone(),
                name: name.to_string(),
                role: role.to_string(),
                created_at: now,
                last_active: now,
                session_count: 0,
            },
        );
        id
    }

    /// 为指定 agent 启动新会话，返回 session_id
    pub fn start_session(
        &mut self,
        agent_id: &str,
        context_summary: &str,
    ) -> Result<String, String> {
        if !self.agents.contains_key(agent_id) {
            return Err(format!("agent {} not found", agent_id));
        }
        let agent_session_count = self
            .sessions
            .values()
            .filter(|s| s.agent_id == agent_id && !s.status.is_terminal())
            .count() as u32;
        if agent_session_count >= self.config.max_sessions_per_agent {
            return Err(format!(
                "agent {} has reached max sessions ({})",
                agent_id, self.config.max_sessions_per_agent
            ));
        }
        let session_id = format!("session_{}", self.next_session_id);
        self.next_session_id += 1;
        let now = now_secs();
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.session_count = agent.session_count.saturating_add(1);
            agent.last_active = now;
        }
        self.sessions.insert(
            session_id.clone(),
            AgentSession {
                session_id: session_id.clone(),
                agent_id: agent_id.to_string(),
                status: SessionStatus::Active,
                started_at: now,
                context_summary: context_summary.to_string(),
                task_queue: VecDeque::new(),
                result_log: Vec::new(),
            },
        );
        Ok(session_id)
    }

    /// 入队一个后台任务，返回 task_id
    pub fn enqueue_task(
        &mut self,
        session_id: &str,
        description: &str,
        priority: u8,
        deadline: Option<u64>,
    ) -> Result<String, String> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;
        if session.status.is_terminal() {
            return Err(format!(
                "session {} is terminal ({})",
                session_id,
                session.status.name()
            ));
        }
        let task_id = format!("task_{}_{}", session_id, session.task_queue.len() + 1);
        let now = now_secs();
        session.task_queue.push_back(BackgroundTask {
            id: task_id.clone(),
            description: description.to_string(),
            priority: priority.min(5).max(1),
            status: TaskStatus::Pending,
            created_at: now,
            deadline,
        });
        Ok(task_id)
    }

    /// 标记任务完成，记录结果
    pub fn complete_task(
        &mut self,
        session_id: &str,
        task_id: &str,
        outcome: &str,
        success: bool,
    ) -> bool {
        let session = match self.sessions.get_mut(session_id) {
            Some(s) => s,
            None => return false,
        };
        let now = now_secs();
        let mut found = false;
        let mut duration_ms = 0u64;
        for task in &mut session.task_queue {
            if task.id == task_id {
                task.status = if success {
                    TaskStatus::Completed
                } else {
                    TaskStatus::Failed
                };
                duration_ms = now.saturating_sub(task.created_at).saturating_mul(1000);
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
        session.result_log.push(SessionResult {
            task_id: task_id.to_string(),
            outcome: outcome.to_string(),
            success,
            duration_ms,
            timestamp: now,
        });
        true
    }

    /// 心跳：更新活跃时间，检测空闲，超时终止
    pub fn heartbeat(&mut self, cycle: u64) {
        self.heartbeat_count += 1;
        if self.heartbeat_count % self.config.heartbeat_interval != 0 {
            return;
        }
        let threshold_idle = 20;
        let threshold_terminate = self.config.idle_timeout_cycles;
        let now = now_secs();
        let session_ids: Vec<String> = self.sessions.keys().cloned().collect();
        for sid in session_ids {
            let session = match self.sessions.get_mut(&sid) {
                Some(s) => s,
                None => continue,
            };
            match session.status {
                SessionStatus::Active | SessionStatus::Paused => {
                    let idle = cycle.saturating_sub(session.started_at);
                    if idle >= threshold_terminate {
                        session.status = SessionStatus::Terminated;
                    } else if idle >= threshold_idle {
                        session.status = SessionStatus::Idle;
                    }
                }
                SessionStatus::Idle => {
                    let idle = cycle.saturating_sub(session.started_at);
                    if idle >= threshold_terminate {
                        session.status = SessionStatus::Terminated;
                    }
                }
                SessionStatus::Completed | SessionStatus::Failed | SessionStatus::Terminated => {}
            }
            if let Some(agent) = self.agents.get_mut(&session.agent_id) {
                agent.last_active = now;
            }
        }
    }

    /// 列出所有 agent
    pub fn list_agents(&self) -> &HashMap<String, AgentIdentity> {
        &self.agents
    }

    /// 查询某 agent 的所有非终止会话
    pub fn agent_sessions(&self, agent_id: &str) -> Vec<&AgentSession> {
        self.sessions
            .values()
            .filter(|s| s.agent_id == agent_id && !s.status.is_terminal())
            .collect()
    }

    /// 查询某会话的任务队列
    pub fn session_task_queue(&self, session_id: &str) -> Option<&VecDeque<BackgroundTask>> {
        self.sessions.get(session_id).map(|s| &s.task_queue)
    }

    /// 汇总统计
    pub fn summary(&self) -> SupervisorSummary {
        let agent_count = self.agents.len();
        let active_session_count = self
            .sessions
            .values()
            .filter(|s| s.status == SessionStatus::Active)
            .count();
        let mut total_tasks = 0usize;
        let mut completed_tasks = 0usize;
        for session in self.sessions.values() {
            for task in &session.task_queue {
                total_tasks += 1;
                if task.status == TaskStatus::Completed {
                    completed_tasks += 1;
                }
            }
            completed_tasks += session.result_log.iter().filter(|r| r.success).count();
        }
        let completion_rate = if total_tasks > 0 {
            completed_tasks as f64 / total_tasks as f64
        } else {
            1.0
        };
        SupervisorSummary {
            agent_count,
            active_session_count,
            total_tasks,
            completion_rate,
            heartbeat_count: self.heartbeat_count,
        }
    }

    /// 终止指定会话
    pub fn terminate_session(&mut self, session_id: &str) -> bool {
        match self.sessions.get_mut(session_id) {
            Some(s) => {
                s.status = SessionStatus::Terminated;
                true
            }
            None => false,
        }
    }

    /// 恢复指定会话到活跃状态
    pub fn resume_session(&mut self, session_id: &str) -> bool {
        match self.sessions.get_mut(session_id) {
            Some(s) if s.status == SessionStatus::Idle || s.status == SessionStatus::Paused => {
                s.status = SessionStatus::Active;
                s.started_at = now_secs();
                true
            }
            _ => false,
        }
    }

    /// 清理终止会话：移除超过 500 cycle 的已完成/失败/终止会话
    pub fn prune(&mut self) {
        let now = now_secs();
        self.sessions.retain(|_sid, s| {
            if s.status.is_terminal() && now.saturating_sub(s.started_at) > 500 {
                return false;
            }
            true
        });
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_agent() {
        let mut sup = AgentSupervisor::new();
        let id = sup.register_agent("explorer", "researcher");
        assert!(!id.is_empty());
        assert!(id.starts_with("agent_"));
        assert!(sup.agents.contains_key(&id));
        let agent = &sup.agents[&id];
        assert_eq!(agent.name, "explorer");
        assert_eq!(agent.role, "researcher");
        assert_eq!(agent.session_count, 0);
    }

    #[test]
    fn test_start_session() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("worker", "executor");
        let sid = sup.start_session(&aid, "research phase 1").unwrap();
        assert!(sid.starts_with("session_"));
        let session = &sup.sessions[&sid];
        assert_eq!(session.agent_id, aid);
        assert_eq!(session.status, SessionStatus::Active);
        assert_eq!(session.context_summary, "research phase 1");
        assert!(session.task_queue.is_empty());
    }

    #[test]
    fn test_start_session_unknown_agent() {
        let mut sup = AgentSupervisor::new();
        let result = sup.start_session("agent_999", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_enqueue_task() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("bot", "worker");
        let sid = sup.start_session(&aid, "tasks").unwrap();
        let tid = sup.enqueue_task(&sid, "process data", 3, None).unwrap();
        assert!(tid.contains(&sid));
        let queue = sup.session_task_queue(&sid).unwrap();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].priority, 3);
        assert_eq!(queue[0].status, TaskStatus::Pending);
    }

    #[test]
    fn test_complete_task() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("bot", "worker");
        let sid = sup.start_session(&aid, "tasks").unwrap();
        let tid = sup.enqueue_task(&sid, "process data", 2, None).unwrap();
        let ok = sup.complete_task(&sid, &tid, "done successfully", true);
        assert!(ok);
        let session = &sup.sessions[&sid];
        let task = &session.task_queue[0];
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(session.result_log.len(), 1);
        assert_eq!(session.result_log[0].task_id, tid);
        assert!(session.result_log[0].success);
        assert_eq!(session.result_log[0].outcome, "done successfully");
    }

    #[test]
    fn test_heartbeat_marks_idle() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("sleeper", "idle test");
        let sid = sup.start_session(&aid, "idle").unwrap();
        // set started_at so idle_cycles looks large
        if let Some(s) = sup.sessions.get_mut(&sid) {
            s.started_at = 0;
        }
        // bigger cycle should trigger idle
        sup.heartbeat(100);
        assert_eq!(sup.sessions[&sid].status, SessionStatus::Idle);
    }

    #[test]
    fn test_idle_timeout_terminates() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("terminator", "timeout test");
        let sid = sup.start_session(&aid, "timeout").unwrap();
        if let Some(s) = sup.sessions.get_mut(&sid) {
            s.started_at = 0;
        }
        // cycle past idle_timeout
        sup.heartbeat(150);
        assert_eq!(sup.sessions[&sid].status, SessionStatus::Terminated);
    }

    #[test]
    fn test_summary() {
        let mut sup = AgentSupervisor::new();
        let a1 = sup.register_agent("alpha", "miner");
        let a2 = sup.register_agent("beta", "analyst");
        let s1 = sup.start_session(&a1, "mine").unwrap();
        let s2 = sup.start_session(&a2, "analyze").unwrap();
        let t1 = sup.enqueue_task(&s1, "dig", 3, None).unwrap();
        let t2 = sup.enqueue_task(&s2, "compute", 5, None).unwrap();
        sup.complete_task(&s1, &t1, "found ore", true);
        // s2 task still pending
        let sum = sup.summary();
        assert_eq!(sum.agent_count, 2);
        assert_eq!(sum.active_session_count, 2);
        assert!(sum.total_tasks >= 2);
        assert!(sum.completion_rate > 0.0 && sum.completion_rate <= 1.0);
        assert_eq!(sum.heartbeat_count, 0);
    }

    #[test]
    fn test_max_agents_limit() {
        let config = SupervisorConfig {
            max_agents: 2,
            ..SupervisorConfig::default()
        };
        let mut sup = AgentSupervisor::with_config(config);
        let id1 = sup.register_agent("a", "x");
        let id2 = sup.register_agent("b", "y");
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        let id3 = sup.register_agent("c", "z");
        assert!(id3.is_empty());
    }

    #[test]
    fn test_prune() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("old", "prune test");
        let sid = sup.start_session(&aid, "old session").unwrap();
        if let Some(s) = sup.sessions.get_mut(&sid) {
            s.status = SessionStatus::Completed;
            s.started_at = 0;
        }
        sup.prune();
        // started_at=0, now - 0 will be huge > 500 → pruned
        assert!(!sup.sessions.contains_key(&sid));
    }

    #[test]
    fn test_terminate_session() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("term", "test");
        let sid = sup.start_session(&aid, "term").unwrap();
        assert!(sup.terminate_session(&sid));
        assert_eq!(sup.sessions[&sid].status, SessionStatus::Terminated);
    }

    #[test]
    fn test_resume_session() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("resume", "test");
        let sid = sup.start_session(&aid, "pause").unwrap();
        sup.terminate_session(&sid);
        // terminated → cannot resume
        assert!(!sup.resume_session(&sid));
        // create a paused one
        let aid2 = sup.register_agent("resume2", "test");
        let sid2 = sup.start_session(&aid2, "pause2").unwrap();
        sup.sessions.get_mut(&sid2).unwrap().status = SessionStatus::Paused;
        assert!(sup.resume_session(&sid2));
        assert_eq!(sup.sessions[&sid2].status, SessionStatus::Active);
    }

    #[test]
    fn test_enqueue_into_terminal_session() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("dead", "test");
        let sid = sup.start_session(&aid, "dead").unwrap();
        sup.terminate_session(&sid);
        let result = sup.enqueue_task(&sid, "should fail", 1, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_sessions_filter() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("multi", "test");
        let s1 = sup.start_session(&aid, "s1").unwrap();
        let s2 = sup.start_session(&aid, "s2").unwrap();
        sup.terminate_session(&s1);
        let sessions = sup.agent_sessions(&aid);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, s2);
    }

    #[test]
    fn test_summary_heartbeat_count() {
        let mut sup = AgentSupervisor::new();
        sup.heartbeat(1);
        sup.heartbeat(2);
        // interval=10, <10 invocations → heartbeat_count increments but actual logic only fires on multiples
        let sum = sup.summary();
        assert_eq!(sum.heartbeat_count, 2);
    }

    #[test]
    fn test_task_priority_clamped() {
        let mut sup = AgentSupervisor::new();
        let aid = sup.register_agent("clamp", "test");
        let sid = sup.start_session(&aid, "clamp").unwrap();
        sup.enqueue_task(&sid, "low", 0, None).unwrap();
        sup.enqueue_task(&sid, "high", 10, None).unwrap();
        let q = sup.session_task_queue(&sid).unwrap();
        assert_eq!(q[0].priority, 1);
        assert_eq!(q[1].priority, 5);
    }
}
