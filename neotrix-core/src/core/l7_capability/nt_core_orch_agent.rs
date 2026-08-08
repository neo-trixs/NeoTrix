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
    /// Heartbeat expired (port of sync_todos.py SubagentTracker::check_stale).
    Stale,
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

    /// Register a new subagent bound to a TODO task (port of sync_todos.py register).
    pub fn register_for_task(&mut self, task_id: &str, session: &str) -> String {
        // Timestamp alone can collide when several tasks register within the same
        // second (two ids would overwrite each other in the map), so append a
        // monotonic counter.
        let id = format!("ses_{}_{}", unix_secs(), self.next_id);
        self.next_id += 1;
        let now = unix_secs();
        let mut config = SubagentConfig {
            name: format!("subagent for {task_id}"),
            e8_mode: 1,
            description: String::new(),
            goal: task_id.to_string(),
            capabilities: vec![],
            max_context: 8192,
            autostart: false,
        };
        if !session.is_empty() {
            config.description = format!("session: {session}");
        }
        let agent = SubagentInstance {
            id: id.clone(),
            config,
            status: SubagentStatus::Running { task: task_id.to_string(), started_at: now },
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

    /// Touch a subagent's heartbeat (port of sync_todos.py heartbeat).
    pub fn heartbeat(&mut self, id: &str, result: Option<String>) -> bool {
        let Some(agent) = self.agents.get_mut(id) else { return false };
        agent.last_active = unix_secs();
        if let Some(r) = result {
            agent.status = SubagentStatus::Completed { result: r };
        }
        true
    }

    /// Mark subagents whose heartbeat is older than `timeout_secs` as Stale.
    /// Returns the stale ids (port of sync_todos.py check_stale).
    pub fn check_stale(&mut self, timeout_secs: u64) -> Vec<String> {
        let now = unix_secs();
        let mut stale = Vec::new();
        for agent in self.agents.values_mut() {
            if matches!(agent.status, SubagentStatus::Completed { .. } | SubagentStatus::Failed { .. } | SubagentStatus::Stale) {
                continue;
            }
            if now.saturating_sub(agent.last_active) > timeout_secs {
                agent.status = SubagentStatus::Stale;
                stale.push(agent.id.clone());
            }
        }
        stale
    }

    /// Release a subagent (task finished) — port of sync_todos.py release.
    pub fn release(&mut self, id: &str, result: &str) -> bool {
        let Some(agent) = self.agents.get_mut(id) else { return false };
        agent.status = SubagentStatus::Completed { result: result.to_string() };
        agent.last_active = unix_secs();
        true
    }

    // ── KB persistence (sync_todos.py sessions/subagents.yml) ──

    /// Persist subagent states to the KB `subagents` namespace.
    pub fn save_to_kb(&self, kb: &crate::neotrix::nt_memory_kb::KnowledgeBase) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.agents)
            .map_err(|e| format!("SubagentManager serialize: {e}"))?;
        kb.kv_set("subagents", "registry", &json)
    }

    /// Load subagent states from the KB `subagents` namespace, merging into current state.
    pub fn load_from_kb(&mut self, kb: &crate::neotrix::nt_memory_kb::KnowledgeBase) -> Result<usize, String> {
        let json = kb.kv_get("subagents", "registry")?;
        let Some(json) = json else { return Ok(0) };
        let loaded: HashMap<String, SubagentInstance> = serde_json::from_str(&json)
            .map_err(|e| format!("SubagentManager deserialize: {e}"))?;
        let count = loaded.len();
        self.agents.extend(loaded);
        Ok(count)
    }
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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

// ---------------------------------------------------------------------------
// AgentCatalog — 内置 agent 目录 + 工具权限矩阵 + 路由。
// 设计对标：
//   - Claude Code 的 5-agent 经济模型（Explore 只读 / Plan 先研究），避免过度专门化；
//   - Codex 的 model 分级（haiku=explore, sonnet=plan, 旗舰=general）；
//   - Kun 的 builtin-agent-catalog + subagent-router（目录即路由来源）。
// 反模式警惕：Claude 官方明示 "too many specialist agents fails"，目录保持精简
// （5 类），宁可让 general-purpose 兜底，也不为每个域造专属 agent。
// ---------------------------------------------------------------------------

/// 工具权限令牌 — 构建工具集权限矩阵（Explore 只读，Plan 只出方案等）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolPerm {
    Read,
    Write,
    Execute,
    Communicate,
    Inspect,
}

/// E8 编排能力算子的抽象名称（映射到既定 E8 模式位域的能力表达）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityOp {
    Reason,
    Search,
    Research,
    Plan,
    Execute,
    Refactor,
    Verify,
    Monitor,
    Communicate,
}

/// agent 分级 — 对应 Codex 的 model gradation 与成本感知路由。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentTier {
    /// 最小成本，只读探索（对应 haiku / Explore）
    Leaf,
    /// 中成本，先研究/出方案（对应 sonnet / Plan）
    Branch,
    /// 旗舰，通用执行兜底（对应 general-purpose / Codex 主 agent）
    Trunk,
}

/// 内置 agent 档案 — 目录里的一条。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub name: &'static str,
    pub tier: AgentTier,
    pub e8_mode: u8,
    pub description: &'static str,
    pub goal: &'static str,
    pub capabilities: Vec<CapabilityOp>,
    pub allowed_tools: Vec<ToolPerm>,
    pub max_context: usize,
}

impl AgentProfile {
    pub fn allows(&self, perm: ToolPerm) -> bool {
        self.allowed_tools.contains(&perm)
    }
}

/// 内置 agent 目录 — 参照 Claude Code 5 种内置 agent 精简为 5 类。
pub struct AgentCatalog;

impl AgentCatalog {
    pub fn builtin() -> Vec<AgentProfile> {
        vec![
            AgentProfile {
                name: "explorer",
                tier: AgentTier::Leaf,
                e8_mode: 1,
                description: "只读探索 agent：定位文件、语义搜索、依赖图、只回答不改",
                goal: "只读探查代码库并还原事实，绝不写文件或执行变更",
                capabilities: vec![CapabilityOp::Search, CapabilityOp::Reason],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Inspect],
                max_context: 8192,
            },
            AgentProfile {
                name: "planner",
                tier: AgentTier::Branch,
                e8_mode: 9,
                description: "先研究再出：产出实施计划，构建前锁定方案",
                goal: "调研约束并输出可执行计划，写字面方案不动生产代码",
                capabilities: vec![CapabilityOp::Plan, CapabilityOp::Reason, CapabilityOp::Search],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Inspect],
                max_context: 16384,
            },
            AgentProfile {
                name: "researcher",
                tier: AgentTier::Branch,
                e8_mode: 12,
                description: "搜索研究 agent：统一有序后端搜索，聚合多源为可吸收结论",
                goal: "先用有序搜索后端 (DDG→Wikipedia) 检索，再聚合为证据接地的结论，结果可落 KB",
                capabilities: vec![CapabilityOp::Research, CapabilityOp::Search, CapabilityOp::Plan, CapabilityOp::Reason],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Inspect, ToolPerm::Communicate],
                max_context: 16384,
            },
            AgentProfile {
                name: "generalist",
                tier: AgentTier::Trunk,
                e8_mode: 24,
                description: "旗舰通用执行 agent，兜底所有未路由任务",
                goal: "全权执行：读、写、构建、验证、沟通",
                capabilities: vec![
                    CapabilityOp::Reason, CapabilityOp::Search, CapabilityOp::Plan,
                    CapabilityOp::Execute, CapabilityOp::Verify, CapabilityOp::Communicate,
                ],
                allowed_tools: vec![
                    ToolPerm::Read, ToolPerm::Write, ToolPerm::Execute, ToolPerm::Communicate,
                ],
                max_context: 32768,
            },
            AgentProfile {
                name: "verifier",
                tier: AgentTier::Branch,
                e8_mode: 33,
                description: "审查/验证 agent：对照基因判据，专做回归与 EDV 校验",
                goal: "以测试与构建判据审查产物，阻止自确认陷阱，回滚失败改动",
                capabilities: vec![CapabilityOp::Verify, CapabilityOp::Search],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Inspect, ToolPerm::Execute],
                max_context: 16384,
            },
            AgentProfile {
                name: "watcher",
                tier: AgentTier::Leaf,
                e8_mode: 47,
                description: "常驻监控 agent：健康度、心跳、过期心跳标 Stale",
                goal: "后台监控系统健康，标记失效 agent 并为修复报告证据",
                capabilities: vec![CapabilityOp::Monitor, CapabilityOp::Reason],
                allowed_tools: vec![ToolPerm::Inspect, ToolPerm::Read],
                max_context: 4096,
            },
        ]
    }

    /// 按名称查内置 profile（目录即路由入口）。
    pub fn by_name(name: &str) -> Option<AgentProfile> {
        Self::builtin().into_iter().find(|p| p.name == name)
    }

    /// 成本感知路由：轻量任务（只读/探索）→ Leaf，研究型 → Branch，
    /// 其余 → Trunk 兜底。对应 Codex 的 model gradation 意图。
    pub fn route(task_hint: &str) -> AgentProfile {
        let hint = task_hint.to_lowercase();
        // 研究/搜索/聚合任务 → researcher（统一有序后端，非单点探索）。
        // 必须在 explorer 之前判断: "research" 含 "search" 子串, 否则被误夺。
        if hint.contains("research") || hint.contains("研究") || hint.contains("find")
            || hint.contains("aggregate") || hint.contains("synthesize") {
            return Self::by_name("researcher").unwrap_or_else(|| Self::fallback_profile());
        }
        if hint.contains("explore") || hint.contains("search") || hint.contains("read")
            || hint.contains("inspect") || hint.contains("audit") || hint.contains("分析")
            || hint.contains("查找") {
            return Self::by_name("explorer").unwrap_or_else(|| Self::fallback_profile());
        }
        if hint.contains("plan") || hint.contains("design") || hint.contains("方案")
            || hint.contains("调研") || hint.contains("architecture") {
            return Self::by_name("planner").unwrap_or_else(|| Self::fallback_profile());
        }
        if hint.contains("verify") || hint.contains("test") || hint.contains("审查")
            || hint.contains("review") || hint.contains("回滚") {
            return Self::by_name("verifier").unwrap_or_else(|| Self::fallback_profile());
        }
        if hint.contains("monitor") || hint.contains("watch") || hint.contains("health")
            || hint.contains("监控") || hint.contains("心跳") {
            return Self::by_name("watcher").unwrap_or_else(|| Self::fallback_profile());
        }
        Self::by_name("generalist").unwrap_or_else(|| Self::fallback_profile())
    }

    /// 兜底 profile：目录意外缺名时保证 route() 恒有返回值（内部不变量防御）。
    fn fallback_profile() -> AgentProfile {
        Self::builtin().into_iter().next().unwrap_or_else(|| AgentProfile {
            name: "generalist",
            tier: AgentTier::Trunk,
            e8_mode: 1,
            description: "内置兜底 agent：目录损坏时的最后防线",
            goal: "在目录缺失时保持最小可用能力",
            capabilities: vec![CapabilityOp::Reason],
            allowed_tools: vec![ToolPerm::Read],
            max_context: 2048,
        })
    }

    /// 展示目录（供 `/agent catalog` 用）。
    pub fn catalog_text() -> String {
        let mut out = String::from("NeoTrix 内置 agent 目录:\n");
        for p in Self::builtin() {
            let perms = p.allowed_tools.iter()
                .map(|t| format!("{:?}", t))
                .collect::<Vec<_>>().join(",");
            out.push_str(&format!("  {} | tier={:?} | E8:{} | tools=[{}]\n   {}\n",
                p.name, p.tier, p.e8_mode, perms, p.description));
        }
        out
    }
}

/// 把一个内置 agent 档案物化进 SubagentManager（生产接线，非死代码）。
impl SubagentManager {
    pub fn spawn_from_profile(&mut self, name: &str) -> Result<String, String> {
        let profile = AgentCatalog::by_name(name)
            .ok_or_else(|| format!("unknown agent profile '{}'", name))?;
        let config = SubagentConfig {
            name: profile.name.to_string(),
            e8_mode: profile.e8_mode,
            description: profile.description.to_string(),
            goal: profile.goal.to_string(),
            capabilities: profile.capabilities.iter()
                .map(|c| format!("{:?}", c))
                .collect(),
            max_context: profile.max_context,
            autostart: true,
        };
        Ok(self.spawn(config))
    }
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

    #[test]
    fn test_register_for_task_heartbeat_release() {
        let mut mgr = SubagentManager::new();
        let id = mgr.register_for_task("S-TASK-9", "cycle-201");
        assert!(id.starts_with("ses_"));
        assert!(mgr.get(&id).is_some());
        assert!(mgr.running_count() >= 1);
        assert!(mgr.heartbeat(&id, None));
        assert!(mgr.release(&id, "done well"));
        match mgr.get(&id).unwrap().status {
            SubagentStatus::Completed { ref result } => assert_eq!(result, "done well"),
            _ => panic!("expected Completed after release"),
        }
    }

    #[test]
    fn test_check_stale_flags_expired() {
        let mut mgr = SubagentManager::new();
        let fresh = mgr.register_for_task("S-TASK-1", "");
        let stale = mgr.register_for_task("S-TASK-2", "");
        {
            let agent = mgr.get_mut(&stale).unwrap();
            agent.last_active = unix_secs().saturating_sub(3600 * 3); // 3h old
        }
        let marked = mgr.check_stale(3600);
        assert!(!marked.contains(&fresh));
        assert!(marked.contains(&stale));
        assert!(matches!(mgr.get(&stale).unwrap().status, SubagentStatus::Stale));
    }

    #[test]
    fn test_kb_subagents_roundtrip() {
        let dir = std::env::temp_dir().join(format!("nt_orch_agent_{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        let kb = crate::neotrix::nt_memory_kb::KnowledgeBase::open(Some(dir.join("orch.db")))
            .expect("open kb");
        let mut mgr = SubagentManager::new();
        mgr.register_for_task("S-TASK-5", "cycle-202");
        mgr.save_to_kb(&kb).expect("save");

        let mut reloaded = SubagentManager::new();
        let count = reloaded.load_from_kb(&kb).expect("load");
        assert_eq!(count, 1);
        assert!(reloaded.list().iter().any(|a| a.config.goal == "S-TASK-5"));
    }

    #[test]
    fn test_catalog_has_six_profiles() {
        let profiles = AgentCatalog::builtin();
        assert_eq!(profiles.len(), 6);
        let names: Vec<&str> = profiles.iter().map(|p| p.name).collect();
        assert!(names.contains(&"explorer"));
        assert!(names.contains(&"planner"));
        assert!(names.contains(&"researcher"));
        assert!(names.contains(&"generalist"));
        assert!(names.contains(&"verifier"));
        assert!(names.contains(&"watcher"));
    }

    #[test]
    fn test_catalog_route_maps_task_to_tier() {
        assert_eq!(AgentCatalog::route("explore codebase").name, "explorer");
        assert_eq!(AgentCatalog::route("设计 架构方案").name, "planner");
        assert_eq!(AgentCatalog::route("review the diff").name, "verifier");
        assert_eq!(AgentCatalog::route("监控系统健康").name, "watcher");
        // 研究/聚合任务 → researcher（统一有序搜索后端）
        assert_eq!(AgentCatalog::route("research the latest papers").name, "researcher");
        assert_eq!(AgentCatalog::route("研究该主题").name, "researcher");
        assert_eq!(AgentCatalog::route("synthesize findings").name, "researcher");
        // 未知任务兜底到 generalist（旗舰通用）
        assert_eq!(AgentCatalog::route("随便做点什么").name, "generalist");
        // 工具权限矩阵：explorer 只读，generalist 可写/执行
        let explorer = AgentCatalog::by_name("explorer").unwrap();
        assert!(explorer.allows(ToolPerm::Read));
        assert!(!explorer.allows(ToolPerm::Write));
        assert!(!explorer.allows(ToolPerm::Execute));
        let generalist = AgentCatalog::by_name("generalist").unwrap();
        assert!(generalist.allows(ToolPerm::Write));
        assert!(generalist.allows(ToolPerm::Execute));
        // researcher 工具权限：只读 + 通信，无写入（研究不污染生产）
        let researcher = AgentCatalog::by_name("researcher").unwrap();
        assert!(researcher.allows(ToolPerm::Read));
        assert!(!researcher.allows(ToolPerm::Write));
    }

    #[test]
    fn test_catalog_spawn_from_profile() {
        let mut mgr = SubagentManager::new();
        let id = mgr.spawn_from_profile("explorer").expect("spawn explorer");
        let agent = mgr.get(&id).unwrap();
        assert_eq!(agent.config.e8_mode, 1);
        assert_eq!(agent.config.max_context, 8192);
        assert!(mgr.spawn_from_profile("nonexistent").is_err());
    }
}
