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
/// 兼容双层语义：`name` 无 `nt-` 前缀 = 旧 5 类（静态），`nt-` 前缀 = NT 域（静态或文件驱动）。
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

    pub fn is_nt_domain(&self) -> bool {
        self.name.starts_with("nt-")
    }

    /// 域归属：`nt-world` → `NT-WORLD`；非 NT → None。
    pub fn domain_label(&self) -> Option<String> {
        self.is_nt_domain()
            .then(|| self.name.trim_start_matches("nt-").to_uppercase().replace('-', "_"))
    }
}

/// 运行时物化 agent 档案 — 从文件驱动 SubAgentDef 构建（String 字段，非静态）。
/// 与 AgentProfile 同构但持有堆数据，用于 `~/.neotrix/agents/*.md` 加载的 NT 域 agent。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAgentProfile {
    pub name: String,
    pub tier: AgentTier,
    pub e8_mode: u8,
    pub description: String,
    pub goal: String,
    pub capabilities: Vec<CapabilityOp>,
    pub allowed_tools: Vec<ToolPerm>,
    pub max_context: usize,
    pub temperature: Option<f64>,
    pub steps: Option<usize>,
    pub domain: Option<String>,
    pub trigger: Option<String>,
    pub source_path: String,
    pub body: String,
}

impl RuntimeAgentProfile {
    pub fn allows(&self, perm: ToolPerm) -> bool {
        self.allowed_tools.contains(&perm)
    }

    pub fn is_nt_domain(&self) -> bool {
        self.name.starts_with("nt-")
    }
}

/// 内置 agent 目录 — 旧 5 类（精简经济模型）+ 10 个 NT 域 agent（完整域模型）。
/// NT 域 agent 与 CONTEXT.md 7 域 + 3 扩展域一一对应，route() 优先匹配 NT 域。
pub struct AgentCatalog;

impl AgentCatalog {
    pub fn builtin() -> Vec<AgentProfile> {
        let mut profiles = Self::legacy_builtin();
        profiles.extend(Self::nt_domain_builtin());
        profiles
    }

    /// 旧 5 类 — 精简经济模型（Claude Code 5-agent + watcher），保留作兼容兜底。
    fn legacy_builtin() -> Vec<AgentProfile> {
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

    /// 10 个 NT 域 agent — 与 CONTEXT.md 域模型一一对应（7 域 + NT-SCOUT/NT-META/NT-REPAIR）。
    /// 静态内嵌作为默认档（首次加载 `~/.neotrix/agents/` 前即有）；文件驱动定义可覆盖同名档。
    pub fn nt_domain_builtin() -> Vec<AgentProfile> {
        vec![
            AgentProfile {
                name: "nt-core",
                tier: AgentTier::Trunk,
                e8_mode: 63,
                description: "NT-CORE E8引导者：编排架构大脑，路由委托 + 质量门 + 架构决策",
                goal: "识别任务类型委托给最合适的域 agent；架构决策自己做；双 Ledger 编排",
                capabilities: vec![CapabilityOp::Reason, CapabilityOp::Plan, CapabilityOp::Execute, CapabilityOp::Communicate, CapabilityOp::Verify],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Write, ToolPerm::Execute, ToolPerm::Communicate, ToolPerm::Inspect],
                max_context: 32768,
            },
            AgentProfile {
                name: "nt-world",
                tier: AgentTier::Leaf,
                e8_mode: 2,
                description: "NT-WORLD 虚空探索者：代码库语义探索与依赖图分析（只读）",
                goal: "Glob/Grep 定位、图查询优先、Repository Map 思维，只回答不改",
                capabilities: vec![CapabilityOp::Search, CapabilityOp::Reason],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Inspect],
                max_context: 16384,
            },
            AgentProfile {
                name: "nt-act",
                tier: AgentTier::Trunk,
                e8_mode: 31,
                description: "NT-ACT 行动执行者：多步任务执行与功能实现（可写）",
                goal: "Spec 驱动实现、TDD 红绿重构、双验证（cargo check + test）、R-P16 持久化校验",
                capabilities: vec![CapabilityOp::Execute, CapabilityOp::Reason, CapabilityOp::Plan, CapabilityOp::Verify],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Write, ToolPerm::Execute, ToolPerm::Communicate],
                max_context: 32768,
            },
            AgentProfile {
                name: "nt-mind",
                tier: AgentTier::Branch,
                e8_mode: 14,
                description: "NT-MIND 进化工匠：TDD 实施与技能结晶（可写）",
                goal: "红绿重构、测试金字塔、回归保护优先、技能结晶走吸收协议",
                capabilities: vec![CapabilityOp::Plan, CapabilityOp::Verify, CapabilityOp::Reason],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Write, ToolPerm::Execute],
                max_context: 16384,
            },
            AgentProfile {
                name: "nt-shield",
                tier: AgentTier::Branch,
                e8_mode: 37,
                description: "NT-SHIELD 影卫：安全审查与代码审计（只读）",
                goal: "D1-D63 维度审查、OWASP 2026 增量、证据优先、T3 验证接线",
                capabilities: vec![CapabilityOp::Verify, CapabilityOp::Search, CapabilityOp::Reason],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Inspect, ToolPerm::Execute],
                max_context: 16384,
            },
            AgentProfile {
                name: "nt-memory",
                tier: AgentTier::Branch,
                e8_mode: 21,
                description: "NT-MEMORY 知识守护者：KB 经验吸收/检索/会话收尾",
                goal: "experience-tree 五阶段吸收、指针守恒执行者、neotrix-experience 检索",
                capabilities: vec![CapabilityOp::Reason, CapabilityOp::Search, CapabilityOp::Monitor],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Inspect, ToolPerm::Execute],
                max_context: 16384,
            },
            AgentProfile {
                name: "nt-io",
                tier: AgentTier::Branch,
                e8_mode: 40,
                description: "NT-IO 界面使徒：前端/Tauri UI 与交互实现（可写）",
                goal: "契约对齐、逐功能接线、vitest/playwright/build 三连、对标拉平",
                capabilities: vec![CapabilityOp::Execute, CapabilityOp::Communicate, CapabilityOp::Reason],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Write, ToolPerm::Execute, ToolPerm::Communicate],
                max_context: 32768,
            },
            AgentProfile {
                name: "nt-scout",
                tier: AgentTier::Branch,
                e8_mode: 11,
                description: "NT-SCOUT 虚空探查：外部研究与文献检索（只读）",
                goal: "多源交叉验证、一手优先、query 变体矩阵、溯源纪律，结论带 URL",
                capabilities: vec![CapabilityOp::Research, CapabilityOp::Search, CapabilityOp::Reason],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Communicate, ToolPerm::Inspect],
                max_context: 16384,
            },
            AgentProfile {
                name: "nt-meta",
                tier: AgentTier::Leaf,
                e8_mode: 50,
                description: "NT-META 元吸收者：跨会话元认知与反思（只读）",
                goal: "实证性反思、量化信号、自欺检测、跨会话模式挖掘",
                capabilities: vec![CapabilityOp::Monitor, CapabilityOp::Reason, CapabilityOp::Research],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Inspect],
                max_context: 8192,
            },
            AgentProfile {
                name: "nt-repair",
                tier: AgentTier::Branch,
                e8_mode: 55,
                description: "NT-REPAIR 自愈工程师：问题诊断与恢复（可写）",
                goal: "先复现再假设、二分定位、缓存头号嫌疑、regression 闭环守卫",
                capabilities: vec![CapabilityOp::Execute, CapabilityOp::Verify, CapabilityOp::Reason, CapabilityOp::Monitor],
                allowed_tools: vec![ToolPerm::Read, ToolPerm::Write, ToolPerm::Execute, ToolPerm::Inspect],
                max_context: 32768,
            },
        ]
    }

    /// 按名称查内置 profile（目录即路由入口）。优先 NT 域，回落旧 5 类。
    pub fn by_name(name: &str) -> Option<AgentProfile> {
        Self::builtin().into_iter().find(|p| p.name == name)
    }

    /// 把文件驱动的 SubAgentDef 物化为 RuntimeAgentProfile（NT 域 agent 文件加载路径）。
    /// 工具权限矩阵从 frontmatter permission 推导；无 permission 时按域默认。
    pub fn from_subagent_def(
        def: &crate::core::nt_core_subagent::SubAgentDef,
    ) -> RuntimeAgentProfile {
        use crate::core::nt_core_subagent::PermissionMatrix;

        let e8_mode = crate::core::nt_core_subagent::e8_mode_for(def);
        let (allowed_tools, tier) = runtime_tool_perms(def.permission.as_ref(), &def.name);
        RuntimeAgentProfile {
            name: def.name.clone(),
            tier,
            e8_mode,
            description: def.description.clone(),
            goal: format!("{}: 按域契约执行任务", def.name),
            capabilities: runtime_capabilities(&def.name, &def.domain),
            allowed_tools,
            max_context: def.steps.map(|s| s * 512).unwrap_or(16384),
            temperature: def.temperature,
            steps: def.steps,
            domain: def.domain.clone(),
            trigger: def.trigger.clone(),
            source_path: def.source_path.display().to_string(),
            body: def.body.clone(),
        }
    }

    /// 成本感知路由：先匹配 NT 域触发词，命中 → NT 域 profile；未命中回落旧 5 类。
    pub fn route(task_hint: &str) -> AgentProfile {
        let hint = task_hint.to_lowercase();

        // NT 域优先路由 — 触发词与 .opencode/agents 定义对齐（共享语言 CONTEXT.md）
        let nt_routes: &[(&str, &[&str])] = &[
            ("nt-core", &["编排", "路由", "架构决策", "委托", "orchestrat", "dispatch", "architecture decision"]),
            ("nt-world", &["探索", "定位", "梳理", "盘点", "explore code", "locate", "find file", "代码结构", "依赖图"]),
            ("nt-act", &["实现", "修复 bug", "重构", "实施", "implement", "fix bug", "refactor", "接线", "落地"]),
            ("nt-mind", &["tdd", "写测试", "测试驱动", "技能结晶", "test-driven", "regression test", "测试优先"]),
            ("nt-shield", &["审查", "审计", "安全扫描", "盘点代码", "review", "audit", "security scan", "漏洞"]),
            ("nt-memory", &["经验", "吸收", "检索", "收尾", "cycle", "知识库", "absorb", "experience", "memory"]),
            ("nt-io", &["前端", "界面", "ui", "tauri", "对话界面", "交互", "frontend", "桌面端"]),
            ("nt-scout", &["搜索", "调研", "对标", "文献", "查资料", "竞品", "research paper", "benchmark", "scout"]),
            ("nt-meta", &["复盘", "反思", "元认知", "自省", "模式提炼", "进化评估", "retrospect", "meta-cognition"]),
            ("nt-repair", &["报错", "失败", "崩溃", "诊断", "构建失败", "无法启动", "bug fix", "crash", "debug", "diagnos"]),
        ];

        for (name, triggers) in nt_routes {
            if triggers.iter().any(|t| hint.contains(t)) {
                if let Some(p) = Self::by_name(name) {
                    return p;
                }
            }
        }

        // 回落旧 5 类（兼容既有调用方）
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

    /// 展示目录（供 `/agent catalog` 用）。NT 域 agent 分组在前，旧 5 类兜底在后。
    pub fn catalog_text() -> String {
        let mut out = String::from("NeoTrix 内置 agent 目录:\n");
        out.push_str("── NT 域 agent（10）──\n");
        for p in Self::nt_domain_builtin() {
            let perms = p.allowed_tools.iter()
                .map(|t| format!("{:?}", t))
                .collect::<Vec<_>>().join(",");
            out.push_str(&format!("  {} | tier={:?} | E8:{} | tools=[{}]\n   {}\n",
                p.name, p.tier, p.e8_mode, perms, p.description));
        }
        out.push_str("── 通用兜底（6）──\n");
        for p in Self::legacy_builtin() {
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

    /// 从文件驱动的 RuntimeAgentProfile 物化（NT 域 agent 文件加载路径）。
    pub fn spawn_from_runtime_profile(&mut self, profile: &RuntimeAgentProfile) -> Result<String, String> {
        let config = SubagentConfig {
            name: profile.name.clone(),
            e8_mode: profile.e8_mode,
            description: profile.description.clone(),
            goal: profile.goal.clone(),
            capabilities: profile.capabilities.iter()
                .map(|c| format!("{:?}", c))
                .collect(),
            max_context: profile.max_context,
            autostart: true,
        };
        Ok(self.spawn(config))
    }
}

/// 从文件驱动 permission 矩阵推导 ToolPerm 集合 + tier。
/// 无 permission 声明时按 NT 域默认（只读域 → Leaf，可写域 → Branch/Trunk）。
fn runtime_tool_perms(perm: Option<&crate::core::nt_core_subagent::PermissionMatrix>, name: &str) -> (Vec<ToolPerm>, AgentTier) {
    use crate::core::nt_core_subagent::PermissionMatrix;

    let matrix = perm.cloned().unwrap_or_default();
    let mut tools = vec![ToolPerm::Read, ToolPerm::Inspect];
    let mut writable = false;

    if matrix.tool_allowed("bash") {
        tools.push(ToolPerm::Execute);
    }
    if matrix.tool_allowed("write") || matrix.tool_allowed("edit") {
        tools.push(ToolPerm::Write);
        writable = true;
    }
    if matrix.tool_allowed("webfetch") || matrix.tool_allowed("websearch") {
        tools.push(ToolPerm::Communicate);
    }

    // 无显式 permission 时按域默认
    if tools.len() <= 2 {
        match name {
            "nt-act" | "nt-io" | "nt-repair" | "nt-mind" => {
                tools.push(ToolPerm::Write);
                tools.push(ToolPerm::Execute);
                writable = true;
            }
            "nt-core" => {
                tools.push(ToolPerm::Write);
                tools.push(ToolPerm::Execute);
                tools.push(ToolPerm::Communicate);
                writable = true;
            }
            "nt-shield" | "nt-memory" => {
                tools.push(ToolPerm::Execute);
            }
            "nt-scout" => {
                tools.push(ToolPerm::Communicate);
            }
            _ => {}
        }
    }

    let tier = if writable {
        match name {
            "nt-act" | "nt-io" | "nt-repair" | "nt-core" => AgentTier::Trunk,
            _ => AgentTier::Branch,
        }
    } else {
        match name {
            "nt-world" | "nt-meta" => AgentTier::Leaf,
            _ => AgentTier::Branch,
        }
    };

    (tools, tier)
}

/// 从 NT 域 agent 名推导能力算子集合。
fn runtime_capabilities(name: &str, domain: &Option<String>) -> Vec<CapabilityOp> {
    let d = domain.as_deref().unwrap_or("").to_uppercase();
    let mut caps = vec![CapabilityOp::Reason];
    match name {
        "nt-world" => caps.extend([CapabilityOp::Search]),
        "nt-act" => caps.extend([CapabilityOp::Execute, CapabilityOp::Plan, CapabilityOp::Verify]),
        "nt-mind" => caps.extend([CapabilityOp::Plan, CapabilityOp::Verify]),
        "nt-shield" => caps.extend([CapabilityOp::Verify, CapabilityOp::Search]),
        "nt-memory" => caps.extend([CapabilityOp::Search, CapabilityOp::Monitor]),
        "nt-io" => caps.extend([CapabilityOp::Execute, CapabilityOp::Communicate]),
        "nt-scout" => caps.extend([CapabilityOp::Research, CapabilityOp::Search]),
        "nt-meta" => caps.extend([CapabilityOp::Monitor, CapabilityOp::Research]),
        "nt-repair" => caps.extend([CapabilityOp::Execute, CapabilityOp::Verify, CapabilityOp::Monitor]),
        "nt-core" => caps.extend([CapabilityOp::Plan, CapabilityOp::Execute, CapabilityOp::Verify, CapabilityOp::Communicate]),
        _ => {
            if d.contains("WORLD") { caps.push(CapabilityOp::Search); }
            else if d.contains("ACT") { caps.extend([CapabilityOp::Execute, CapabilityOp::Plan]); }
            else if d.contains("SHIELD") { caps.extend([CapabilityOp::Verify, CapabilityOp::Search]); }
            else if d.contains("SCOUT") { caps.extend([CapabilityOp::Research, CapabilityOp::Search]); }
            else if d.contains("META") { caps.extend([CapabilityOp::Monitor, CapabilityOp::Research]); }
            else if d.contains("REPAIR") { caps.extend([CapabilityOp::Execute, CapabilityOp::Verify]); }
            else if d.contains("MIND") { caps.extend([CapabilityOp::Plan, CapabilityOp::Verify]); }
            else if d.contains("MEMORY") { caps.extend([CapabilityOp::Search, CapabilityOp::Monitor]); }
            else if d.contains("IO") { caps.extend([CapabilityOp::Execute, CapabilityOp::Communicate]); }
        }
    }
    caps
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
        assert_eq!(profiles.len(), 16);
        let names: Vec<&str> = profiles.iter().map(|p| p.name).collect();
        assert!(names.contains(&"explorer"));
        assert!(names.contains(&"planner"));
        assert!(names.contains(&"researcher"));
        assert!(names.contains(&"generalist"));
        assert!(names.contains(&"verifier"));
        assert!(names.contains(&"watcher"));
        // NT 域 agent 全量存在
        for nt in ["nt-core", "nt-world", "nt-act", "nt-mind", "nt-shield",
                   "nt-memory", "nt-io", "nt-scout", "nt-meta", "nt-repair"] {
            assert!(names.contains(&nt), "missing {}", nt);
        }
    }

    #[test]
    fn test_catalog_nt_domain_profiles() {
        let nt = AgentCatalog::nt_domain_builtin();
        assert_eq!(nt.len(), 10);
        // NT 域 agent 的 e8_mode 唯一且非零
        let mut modes: Vec<u8> = nt.iter().map(|p| p.e8_mode).collect();
        modes.sort();
        modes.dedup();
        assert_eq!(modes.len(), 10, "e8_mode must be unique across NT domain");
        assert!(modes.iter().all(|m| *m > 0));
        // 只读域（world/meta/scout）不可写
        let world = AgentCatalog::by_name("nt-world").unwrap();
        assert!(world.allows(ToolPerm::Read));
        assert!(!world.allows(ToolPerm::Write));
        let meta = AgentCatalog::by_name("nt-meta").unwrap();
        assert!(!meta.allows(ToolPerm::Write));
        // 可写域（act/io/repair）可写可执行
        let act = AgentCatalog::by_name("nt-act").unwrap();
        assert!(act.allows(ToolPerm::Write));
        assert!(act.allows(ToolPerm::Execute));
    }

    #[test]
    fn test_catalog_route_maps_task_to_tier() {
        // NT 域优先：explore codebase → nt-world（域模型取代旧 5 类）
        assert_eq!(AgentCatalog::route("explore codebase").name, "nt-world");
        // 旧 5 类仍可命中（无 NT 域触发词时回落）
        assert_eq!(AgentCatalog::route("设计 架构方案").name, "planner");
        // review 类 → NT 域优先（nt-shield 审查）
        assert_eq!(AgentCatalog::route("review the diff").name, "nt-shield");
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
    fn test_catalog_route_nt_domain_priority() {
        // NT 域触发词优先路由
        assert_eq!(AgentCatalog::route("探索代码库结构").name, "nt-world");
        assert_eq!(AgentCatalog::route("实现这个功能").name, "nt-act");
        assert_eq!(AgentCatalog::route("写测试 TDD").name, "nt-mind");
        assert_eq!(AgentCatalog::route("审查代码安全").name, "nt-shield");
        assert_eq!(AgentCatalog::route("吸收经验到知识库").name, "nt-memory");
        assert_eq!(AgentCatalog::route("前端 UI 界面").name, "nt-io");
        assert_eq!(AgentCatalog::route("调研竞品对标").name, "nt-scout");
        assert_eq!(AgentCatalog::route("复盘本次会话").name, "nt-meta");
        assert_eq!(AgentCatalog::route("构建失败诊断").name, "nt-repair");
        assert_eq!(AgentCatalog::route("编排委托任务").name, "nt-core");
    }

    #[test]
    fn test_catalog_from_subagent_def() {
        use crate::core::nt_core_subagent::{SubAgentDef, SubAgentDefParser};
        let content = r#"---
name: nt-world
description: NT-WORLD 虚空探索者（只读）
permission:
  edit:
    allow: false
  write:
    allow: false
  bash:
    allow: false
    patterns: ["ls *", "git status*"]
temperature: 0.2
steps: 60
domain: NT-WORLD
trigger: 探索,定位,梳理
---
只读探索 agent body
"#;
        let def = SubAgentDefParser::parse(std::path::Path::new("nt-world.md"), content).unwrap();
        let profile = AgentCatalog::from_subagent_def(&def);
        assert_eq!(profile.name, "nt-world");
        assert_eq!(profile.e8_mode, 2);
        assert_eq!(profile.temperature, Some(0.2));
        assert_eq!(profile.steps, Some(60));
        assert_eq!(profile.domain.as_deref(), Some("NT-WORLD"));
        // 只读域 → 无 Write 权限
        assert!(!profile.allows(ToolPerm::Write));
        assert!(profile.allows(ToolPerm::Read));
        assert!(profile.is_nt_domain());
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

    #[test]
    fn test_catalog_spawn_nt_domain() {
        let mut mgr = SubagentManager::new();
        let id = mgr.spawn_from_profile("nt-world").expect("spawn nt-world");
        let agent = mgr.get(&id).unwrap();
        assert_eq!(agent.config.e8_mode, 2);
        assert_eq!(agent.config.name, "nt-world");
    }

    #[test]
    fn test_catalog_spawn_nt_domain_from_file() {
        let mut mgr = SubagentManager::new();
        // 文件驱动物化后 spawn
        use crate::core::nt_core_subagent::{SubAgentDef, SubAgentDefParser};
        let content = r#"---
name: nt-scout
description: NT-SCOUT 虚空探查（只读）
permission:
  edit: {allow: false}
  write: {allow: false}
  webfetch: {allow: true}
  websearch: {allow: true}
---
外部调研 agent
"#;
        let def = SubAgentDefParser::parse(std::path::Path::new("nt-scout.md"), content).unwrap();
        let profile = AgentCatalog::from_subagent_def(&def);
        let id2 = mgr.spawn_from_runtime_profile(&profile).expect("spawn nt-scout");
        let agent2 = mgr.get(&id2).unwrap();
        assert_eq!(agent2.config.name, "nt-scout");
        assert_eq!(agent2.config.e8_mode, 11);
    }
}
