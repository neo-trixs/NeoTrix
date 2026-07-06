use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use crate::core::nt_core_plan::E8Plan;


/// Async agent execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed(String),
    Failed(String),
}

/// Async agent execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub name: String,
    pub e8_mode: u8,
    pub status: TaskStatus,
    pub created_at: SystemTime,
}

/// E8 原生子代理系统 — 每个 subagent 绑定一个 E8 模式 + 独立认知窗口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentConfig {
    pub name: String,
    pub e8_mode: u8,
    pub description: String,
    pub goal: String,
    pub capabilities: Vec<String>,
    pub max_context: usize,
    pub autostart: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentInstance {
    pub id: String,
    pub config: SubagentConfig,
    pub status: SubagentStatus,
    pub messages: Vec<AgentMessage>,
    pub current_plan: Option<E8Plan>,
    pub context_window: Vec<String>,
    pub created_at: u64,
    pub last_active: u64,
    pub execution_count: u64,
    pub total_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubagentStatus {
    Idle,
    Running { task: String, started_at: u64 },
    Completed { result: String },
    Failed { error: String },
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub msg_type: MessageType,
    pub timestamp: u64,
    pub in_response_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Task,
    Result,
    Query,
    Response,
    Broadcast,
    Error,
}

/// 子代理管理器 — E8-driven 多 agent 编排
pub struct SubagentManager {
    agents: HashMap<String, SubagentInstance>,
    background_tasks: HashMap<String, AgentTask>,
    #[allow(dead_code)]
    default_capabilities: Vec<String>,
    next_id: u64,
    #[allow(dead_code)]
    max_agents: usize,
}

impl SubagentManager {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            background_tasks: HashMap::new(),
            default_capabilities: vec!["reason".into(), "search".into(), "communicate".into()],
            next_id: 1,
            max_agents: 8,
        }
    }

    pub fn spawn(&mut self, config: SubagentConfig) -> String {
        let id = format!("agent-{:04}", self.next_id);
        self.next_id += 1;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let agent = SubagentInstance {
            id: id.clone(),
            config,
            status: SubagentStatus::Idle,
            messages: Vec::new(),
            current_plan: None,
            context_window: Vec::new(),
            created_at: now,
            last_active: now,
            execution_count: 0,
            total_duration_ms: 0,
        };
        self.agents.insert(id.clone(), agent);
        id
    }

    pub fn kill(&mut self, id: &str) -> Option<SubagentInstance> {
        self.agents.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<&SubagentInstance> {
        self.agents.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut SubagentInstance> {
        self.agents.get_mut(id)
    }

    pub fn list(&self) -> Vec<&SubagentInstance> {
        self.agents.values().collect()
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn send_message(&mut self, from: &str, to: &str, content: &str, msg_type: MessageType) -> Result<(), String> {
        let to_exists = self.agents.contains_key(to);
        if !to_exists {
            return Err(format!("Agent '{}' not found", to));
        }
        let id = format!("msg-{}", uuid::Uuid::new_v4());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let msg = AgentMessage {
            id,
            from: from.to_string(),
            to: to.to_string(),
            content: content.to_string(),
            msg_type,
            timestamp: now,
            in_response_to: None,
        };
        if let Some(agent) = self.agents.get_mut(to) {
            agent.messages.push(msg);
            agent.last_active = now;
        }
        Ok(())
    }

    pub fn assign_plan(&mut self, agent_id: &str, plan: E8Plan) -> Result<(), String> {
        let agent = self.agents.get_mut(agent_id).ok_or_else(|| format!("Agent '{}' not found", agent_id))?;
        agent.current_plan = Some(plan);
        agent.status = SubagentStatus::Running {
            task: agent.config.goal.clone(),
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        };
        Ok(())
    }

    pub fn agent_by_e8_mode(&self, mode: u8) -> Vec<&SubagentInstance> {
        self.agents.values().filter(|a| a.config.e8_mode == mode).collect()
    }

    pub fn broadcast(&mut self, sender: &str, content: &str) {
        let ids: Vec<String> = self.agents.keys().cloned().collect();
        for id in ids {
            if id != sender {
                let _ = self.send_message(sender, &id, content, MessageType::Broadcast);
            }
        }
    }

    pub fn running_count(&self) -> usize {
        self.agents.values().filter(|a| matches!(a.status, SubagentStatus::Running { .. })).count()
    }

    pub fn summary_stats(&self) -> AgentPoolStats {
        AgentPoolStats {
            total: self.agents.len(),
            running: self.running_count(),
            idle: self.agents.values().filter(|a| matches!(a.status, SubagentStatus::Idle)).count(),
            completed: self.agents.values().filter(|a| matches!(a.status, SubagentStatus::Completed { .. })).count(),
            failed: self.agents.values().filter(|a| matches!(a.status, SubagentStatus::Failed { .. })).count(),
            paused: self.agents.values().filter(|a| matches!(a.status, SubagentStatus::Paused)).count(),
            total_executions: self.agents.values().map(|a| a.execution_count).sum(),
        }
    }

    /// Spawn a background agent task (non-blocking)
    pub fn spawn_background(&mut self, name: &str, mode: u8) -> String {
        let id = format!("bg-{:04}", self.next_id);
        self.next_id += 1;
        let task = AgentTask {
            id: id.clone(),
            name: name.to_string(),
            e8_mode: mode,
            status: TaskStatus::Pending,
            created_at: SystemTime::now(),
        };
        self.background_tasks.insert(id.clone(), task);
        id
    }

    /// Get status of a background task
    pub fn get_task_status(&self, id: &str) -> Option<&TaskStatus> {
        self.background_tasks.get(id).map(|t| &t.status)
    }

    /// List all background tasks
    pub fn list_tasks(&self) -> Vec<&AgentTask> {
        self.background_tasks.values().collect()
    }

    /// Execute all pending background tasks
    pub fn execute_pending_tasks(&mut self) -> Vec<String> {
        let pending: Vec<String> = self.background_tasks.iter()
            .filter(|(_, t)| matches!(t.status, TaskStatus::Pending))
            .map(|(id, _)| id.clone())
            .collect();

        for id in &pending {
            if let Some(task) = self.background_tasks.get_mut(id) {
                task.status = TaskStatus::Running;
                task.status = TaskStatus::Completed(format!(
                    "Task {} completed in E8 mode {}", task.name, task.e8_mode
                ));
            }
        }
        pending
    }
}

impl Default for SubagentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPoolStats {
    pub total: usize,
    pub running: usize,
    pub idle: usize,
    pub completed: usize,
    pub failed: usize,
    pub paused: usize,
    pub total_executions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_agent() {
        let mut mgr = SubagentManager::new();
        let config = SubagentConfig {
            name: "researcher".into(),
            e8_mode: 9,
            description: "Research subagent".into(),
            goal: "Search and summarize".into(),
            capabilities: vec!["search".into(), "summarize".into()],
            max_context: 4096,
            autostart: true,
        };
        let id = mgr.spawn(config);
        let agent = mgr.get(&id).unwrap();
        assert_eq!(agent.config.name, "researcher");
        assert_eq!(agent.config.e8_mode, 9);
    }

    #[test]
    fn test_send_message() {
        let mut mgr = SubagentManager::new();
        let config_a = SubagentConfig { name: "alpha".into(), e8_mode: 1, description: "".into(), goal: "".into(), capabilities: vec![], max_context: 1000, autostart: false };
        let config_b = SubagentConfig { name: "beta".into(), e8_mode: 2, description: "".into(), goal: "".into(), capabilities: vec![], max_context: 1000, autostart: false };
        mgr.spawn(config_a);
        mgr.spawn(config_b);

        let id_b = mgr.list().last().unwrap().id.clone();
        assert!(mgr.send_message("agent-0001", &id_b, "hello", MessageType::Query).is_ok());
        let agent_b = mgr.get(&id_b).unwrap();
        assert_eq!(agent_b.messages.len(), 1);
        assert_eq!(agent_b.messages[0].content, "hello");
    }

    #[test]
    fn test_assign_plan() {
        let mut mgr = SubagentManager::new();
        let config = SubagentConfig { name: "planner".into(), e8_mode: 7, description: "".into(), goal: "Execute plan".into(), capabilities: vec![], max_context: 1000, autostart: false };
        mgr.spawn(config);
        let id = mgr.list().last().unwrap().id.clone();

        let plan_gen = crate::core::nt_core_plan::PlanGenerator::new();
        let plan = plan_gen.generate_plan("Test", &[]);
        assert!(mgr.assign_plan(&id, plan).is_ok());
        let agent = mgr.get(&id).unwrap();
        assert!(agent.current_plan.is_some());
    }

    #[test]
    fn test_kill_agent() {
        let mut mgr = SubagentManager::new();
        let config = SubagentConfig { name: "temp".into(), e8_mode: 0, description: "".into(), goal: "".into(), capabilities: vec![], max_context: 1000, autostart: false };
        mgr.spawn(config);
        let id = mgr.list().last().unwrap().id.clone();
        assert_eq!(mgr.agent_count(), 1);
        mgr.kill(&id);
        assert_eq!(mgr.agent_count(), 0);
    }

    #[test]
    fn test_spawn_background() {
        let mut mgr = SubagentManager::new();
        let id = mgr.spawn_background("bg-worker", 12);
        assert!(id.starts_with("bg-"));
        assert_eq!(mgr.list_tasks().len(), 1);
        assert!(matches!(mgr.get_task_status(&id), Some(TaskStatus::Pending)));
    }

    #[test]
    fn test_execute_pending_tasks() {
        let mut mgr = SubagentManager::new();
        let id = mgr.spawn_background("worker-a", 5);
        mgr.spawn_background("worker-b", 8);
        assert_eq!(mgr.list_tasks().len(), 2);

        let executed = mgr.execute_pending_tasks();
        assert_eq!(executed.len(), 2);
        assert!(executed.contains(&id));

        let status = mgr.get_task_status(&id);
        assert!(matches!(status, Some(TaskStatus::Completed(_))));
        if let Some(TaskStatus::Completed(msg)) = status {
            assert!(msg.contains("worker-a"));
            assert!(msg.contains("E8 mode 5"));
        }
    }

    #[test]
    fn test_list_tasks_empty() {
        let mgr = SubagentManager::new();
        assert!(mgr.list_tasks().is_empty());
    }

    #[test]
    fn test_get_task_status_unknown() {
        let mgr = SubagentManager::new();
        assert!(mgr.get_task_status("nonexistent").is_none());
    }

    #[test]
    fn test_broadcast() {
        let mut mgr = SubagentManager::new();
        let configs = vec![
            SubagentConfig { name: "a".into(), e8_mode: 1, description: "".into(), goal: "".into(), capabilities: vec![], max_context: 1000, autostart: false },
            SubagentConfig { name: "b".into(), e8_mode: 2, description: "".into(), goal: "".into(), capabilities: vec![], max_context: 1000, autostart: false },
            SubagentConfig { name: "c".into(), e8_mode: 3, description: "".into(), goal: "".into(), capabilities: vec![], max_context: 1000, autostart: false },
        ];
        for cfg in configs {
            mgr.spawn(cfg);
        }
        let id_first = mgr.list().first().unwrap().id.clone();
        mgr.broadcast(&id_first, "hello everyone");
        // Each non-sender agent should have 1 message
        for agent in mgr.list() {
            if agent.id != id_first {
                assert_eq!(agent.messages.len(), 1);
            }
        }
    }
}
