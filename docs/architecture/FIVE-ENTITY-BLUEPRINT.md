# NeoTrix 五大实体蓝图

> ⚠️ **已废弃（SUPERSEDED）** —— 被 `FIVE-ENTITY-BLUEPRINT-V2.md`（v2.0.0，2026-09-22）替代。
> 实施以 v2 为准，本文仅作历史参考。
> 废弃原因：全域审计发现选型不明/分层违规/枢纽三套说法/编号打架，v2 已全部收敛。

> 设计原则：基于 NeoTrix 自身架构公理，不照搬 Lingee
> 日期：2026-09-22

---

## 0. 设计决策

### 决策 1: 实体边界 — 扩展现有类型

NeoTrix 已有基础类型，但碎片化。选择**扩展**而非重写：

| 实体 | 现有类型 | 位置 | 扩展方式 |
|------|---------|------|---------|
| Workspace | `WorkSpace` | `l0_substrate/nt_core_ws.rs` | 增加字段 |
| Agent | `AgentCard` | `l1_action/nt_infra_agent_card.rs` | 增加六维字段 |
| Skill | `SkillCandidate` | `l6_meta/coordination/skill_evolution.rs` | 增加元数据 |
| Task | `ScheduledTask` | `l1_action/nt_act/nt_act_scheduler.rs` | 增加关联字段 |
| MCP | `McpRegistry` | `agent.rs::tool::mcp` | 增加认证/权限 |

### 决策 2: 枢纽 — ConsciousnessCore 就是枢纽

NeoTrix 已有 `ConsciousnessCore`，它通过 tick 循环分配注意力。**不新建 Hub 实体**，而是让 ConsciousnessCore 成为五实体的自然路由：

```
用户输入 → ConsciousnessCore (tick)
                │
                ├── 意图识别 → 选择 Agent
                ├── Skill 加载 → 按需注入
                ├── Task 调度 → 按优先级排队
                ├── MCP 调用 → 工具执行
                └── Workspace → 上下文容器
```

### 决策 3: MCP 分层 — L1 执行 + L5 发现

- **L1 `nt_infra_agent_card.rs`**: AgentCard + McpRegistry（执行层，工具调用）
- **L5 `nt_core_model_skills.rs`**: SkillRegistry（发现层，技能匹配）
- 两者通过 Agent 的 `installed_skills` 和 `mcp_permissions` 字段桥接

---

## 1. Workspace（工作空间）

### 现状

```rust
// l0_substrate/nt_core_ws.rs — C2 成熟度
pub struct WorkSpace {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub project_root: Option<PathBuf>,
    pub description: String,
    pub tags: Vec<String>,
    pub memory_count: u32,
    pub goal_count: u32,
    pub skill_count: u32,
}
```

### 扩展设计

```rust
// 在 nt_core_ws.rs 中增加（不改现有字段，只追加）
pub struct WorkSpace {
    // ── 现有字段保持不变 ──
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub project_root: Option<PathBuf>,
    pub description: String,
    pub tags: Vec<String>,
    pub memory_count: u32,
    pub goal_count: u32,
    pub skill_count: u32,
    
    // ── 新增字段 ──
    /// 工作空间类型
    pub kind: WorkspaceKind,
    /// 所属 Agent ID 列表
    pub agent_ids: Vec<String>,
    /// 已安装 Skill ID 列表
    pub skill_ids: Vec<String>,
    /// MCP 服务器配置（名称→命令）
    pub mcp_servers: Vec<McpServerBinding>,
    /// 共享记忆键（跨 Agent 共享的 KB key）
    pub shared_memory_keys: Vec<String>,
    /// 工作空间级别配置
    pub config: WorkspaceConfig,
}

pub enum WorkspaceKind {
    Local,                          // 当前目录
    Remote { url: String },         // 远程
    Sandbox,                        // 隔离沙箱
}

pub struct McpServerBinding {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub enabled: bool,
}

pub struct WorkspaceConfig {
    pub model_tier_preference: Option<String>,  // fast/auto/deep/expert/ultra
    pub max_concurrent_agents: u32,
    pub auto_save: bool,
    pub governance_level: String,               // off/monitor/enforce
}
```

### 数据流

```
WorkSpaceManager::create()
    → WorkSpace { kind, agent_ids, skill_ids, mcp_servers, ... }
    → save() → KB "workspaces"

ConsciousnessCore::tick()
    → 读取 active workspace
    → 加载 agent_ids, skill_ids, mcp_servers
    → 注入到 tick 上下文
```

### 实现路径

| 步骤 | 内容 | 复杂度 |
|------|------|--------|
| W1 | WorkSpace 增加新字段 + Default 实现 | 低 |
| W2 | McpServerBinding 类型 | 低 |
| W3 | WorkspaceConfig 类型 | 低 |
| W4 | WorkSpaceManager 增加 agent/skill 管理方法 | 中 |
| W5 | 单元测试 | 低 |

---

## 2. Agent（智能体）

### 现状

```rust
// l1_action/nt_infra_agent_card.rs — C4 成熟度
pub struct AgentCard {
    pub schema_version: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<AgentCapability>,
    pub tags: Vec<String>,
    pub endpoint: Option<String>,
    pub auth_type: Option<String>,
    pub max_concurrent: u32,
    pub created_at: u64,
    pub last_heartbeat: u64,
}

// l6_meta/nt_agent_identity.rs — AgentPersona
pub struct AgentPersona {
    pub id: String,
    pub name: String,
    pub role: String,
    pub backstory: String,
    pub tools: Vec<String>,
}
```

### 扩展设计（吸收 Lingee 六维）

```rust
// 在 nt_infra_agent_card.rs 中增加
pub struct AgentCard {
    // ── 现有字段保持不变 ──
    pub schema_version: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<AgentCapability>,
    pub tags: Vec<String>,
    pub endpoint: Option<String>,
    pub auth_type: Option<String>,
    pub max_concurrent: u32,
    pub created_at: u64,
    pub last_heartbeat: u64,
    
    // ── 维度1: 角色身份 ──
    pub role: AgentRole,
    // ── 维度2: 知识领域 ──
    pub knowledge_domains: Vec<String>,
    // ── 维度3: 任务模板 ──
    pub task_templates: Vec<String>,  // TaskTemplate ID 列表
    // ── 维度4: 交付物 ──
    pub deliverable_types: Vec<String>,
    // ── 维度5: 技能编排 ──
    pub installed_skills: Vec<String>,  // Skill ID 列表
    // ── 维度6: 执行策略 ──
    pub execution: ExecutionPolicy,
    // ── 关联 ──
    pub workspace_id: Option<String>,
    pub mcp_permissions: Vec<McpPermission>,
    pub model_preference: ModelPreference,
    pub status: AgentStatus,
}

pub struct AgentRole {
    pub level: PermissionLevel,      // Read / Write / Admin / Sovereign
    pub org_unit: Option<String>,
    pub reports_to: Option<String>,
}

pub enum AgentStatus {
    Available,
    Thinking,
    Busy,
    Resting,
    Offline,
}

pub struct ExecutionPolicy {
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_secs: u64,
    pub retry_count: u32,
}

pub struct ModelPreference {
    pub tier: String,                 // fast / auto / deep / expert / ultra
    pub providers: Vec<String>,
}

pub struct McpPermission {
    pub server_name: String,
    pub tools: Vec<String>,           // 空 = 全部
}

pub enum PermissionLevel {
    Read,
    Write,
    Admin,
    Sovereign,
}
```

### 预置 Agent 模板

```rust
// 新增: l5_cognition/nt_agent/presets.rs
pub struct PresetAgent {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub role: PermissionLevel,
    pub knowledge: Vec<&'static str>,
    pub skills: Vec<&'static str>,
}

pub const PRESET_AGENTS: &[PresetAgent] = &[
    PresetAgent { id: "code-reviewer", name: "代码审查官", ... },
    PresetAgent { id: "doc-writer", name: "文档工程师", ... },
    PresetAgent { id: "data-analyst", name: "数据分析师", ... },
    PresetAgent { id: "security-auditor", name: "安全审计员", ... },
    PresetAgent { id: "test-engineer", name: "测试工程师", ... },
];
```

### 数据流

```
AgentCardRegistry::register(card)
    → card 包含六维 + workspace_id
    → save() → KB "agent_cards"

ConsciousnessCore::tick()
    → 读取 workspace.agent_ids
    → AgentCardRegistry::find_by_workspace(ws_id)
    → 匹配 Agent 能力 vs 任务需求
    → 分配执行
```

### 实现路径

| 步骤 | 内容 | 复杂度 |
|------|------|--------|
| A1 | AgentCard 增加六维字段 | 中 |
| A2 | AgentRole / ExecutionPolicy / ModelPreference 类型 | 低 |
| A3 | AgentCardRegistry::find_by_workspace() | 低 |
| A4 | 预置 Agent 模板库 | 低 |
| A5 | Agent 协作（委托/结果聚合） | 高 |

---

## 3. Skill（技能）

### 现状

```rust
// l6_meta/coordination/skill_evolution.rs — C2 成熟度
pub struct SkillCandidate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub performance_history: Vec<f64>,
    pub evaluation_count: u32,
    pub last_evaluated: Option<u64>,
    pub maturity: SkillMaturity,
}

// skill_loader.rs — SkillEntry (文件系统发现)
pub struct SkillEntry {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
}
```

### 扩展设计

```rust
// 在 skill_evolution.rs 中增加（不改现有字段）
pub struct SkillCandidate {
    // ── 现有字段保持不变 ──
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub performance_history: Vec<f64>,
    pub evaluation_count: u32,
    pub last_evaluated: Option<u64>,
    pub maturity: SkillMaturity,
    
    // ── 新增元数据 ──
    /// 作者
    pub author: String,
    /// 标签（用于发现）
    pub tags: Vec<String>,
    /// 触发条件
    pub triggers: Vec<TriggerCondition>,
    /// 所需权限
    pub required_permissions: Vec<String>,
    /// 依赖的其他 Skill ID
    pub dependencies: Vec<String>,
    /// 技能来源
    pub source: SkillSource,
    /// 安全检查结果
    pub security_audit: Option<SecurityAudit>,
}

pub struct TriggerCondition {
    pub kind: TriggerKind,
    pub pattern: String,
}

pub enum TriggerKind {
    Keyword,      // 关键词匹配
    FilePattern,  // 文件路径匹配
    ToolCall,     // 工具调用触发
    Schedule,     // 定时触发
    Manual,       // 手动触发
}

pub enum SkillSource {
    Builtin,
    Local { path: PathBuf },
    Git { url: String, branch: String },
    Registry { url: String, skill_id: String },
}

pub struct SecurityAudit {
    pub passed: bool,
    pub checked_at: u64,
    pub issues: Vec<String>,
}
```

### Skill Registry（统一发现）

```rust
// l5_cognition/nt_core_model_skills.rs — 增强现有
impl SkillRegistry {
    // ── 现有方法保持不变 ──
    
    // ── 新增方法 ──
    
    /// 按工作空间过滤技能
    pub fn find_by_workspace(&self, ws_id: &str) -> Vec<&SkillCandidate>;
    
    /// 按 Agent 过滤技能（Agent 安装了哪些）
    pub fn find_by_agent(&self, agent_id: &str) -> Vec<&SkillCandidate>;
    
    /// 按触发条件匹配
    pub fn match_trigger(&self, trigger: &TriggerCondition) -> Vec<&SkillCandidate>;
    
    /// 安装技能到工作空间
    pub fn install_to_workspace(&mut self, skill_id: &str, ws_id: &str) -> Result<(), SkillError>;
    
    /// 从工作空间卸载技能
    pub fn uninstall_from_workspace(&mut self, skill_id: &str, ws_id: &str) -> Result<(), SkillError>;
}
```

### 数据流

```
SkillRegistry::list_skills()
    → 从 KB 加载所有 SkillCandidate
    → 按 workspace_id / agent_id 过滤
    → 返回匹配列表

Agent 执行任务时:
    → SkillRegistry::match_trigger(触发条件)
    → 选择最佳技能
    → 注入到 Agent 上下文
```

### 实现路径

| 步骤 | 内容 | 复杂度 |
|------|------|--------|
| S1 | SkillCandidate 增加元数据字段 | 中 |
| S2 | TriggerCondition / SkillSource 类型 | 低 |
| S3 | SkillRegistry::find_by_workspace() | 低 |
| S4 | SkillRegistry::match_trigger() | 中 |
| S5 | 安装/卸载工作空间技能 | 中 |

---

## 4. Task（任务）

### 现状

```rust
// l1_action/nt_act/nt_act_scheduler.rs — C4 成熟度
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub instruction: String,
    pub schedule: ScheduleType,
    pub status: TaskStatus,
    pub execution_count: u64,
    pub last_executed: Option<String>,
    pub next_execution: Option<String>,
    pub created_at: String,
    pub max_retries: u32,
    pub retry_count: u32,
    pub timeout_secs: u64,
    pub tags: Vec<String>,
}

// l6_meta/coordination/nt_task_orchestrator/ — 递归控制
pub struct TaskNode { ... }
```

### 扩展设计

```rust
// 在 nt_act_scheduler.rs 中增加
pub struct ScheduledTask {
    // ── 现有字段保持不变 ──
    pub id: String,
    pub name: String,
    pub instruction: String,
    pub schedule: ScheduleType,
    pub status: TaskStatus,
    pub execution_count: u64,
    pub last_executed: Option<String>,
    pub next_execution: Option<String>,
    pub created_at: String,
    pub max_retries: u32,
    pub retry_count: u32,
    pub timeout_secs: u64,
    pub tags: Vec<String>,
    
    // ── 新增关联 ──
    /// 执行者 Agent ID
    pub executor_agent_id: Option<String>,
    /// 使用的 Skill ID 列表
    pub skill_ids: Vec<String>,
    /// 所属工作空间 ID
    pub workspace_id: Option<String>,
    /// 交付物定义
    pub deliverables: Vec<DeliverableSpec>,
    /// 依赖的任务 ID
    pub depends_on: Vec<String>,
    /// 创建者
    pub created_by: String,
    /// 执行历史
    pub history: Vec<TaskExecution>,
}

pub struct DeliverableSpec {
    pub name: String,
    pub kind: DeliverableKind,
    pub output_path: Option<PathBuf>,
}

pub enum DeliverableKind {
    File,
    Report,
    Code,
    Document,
    Data,
}

pub struct TaskExecution {
    pub execution_id: String,
    pub started_at: u64,
    pub finished_at: Option<u64>,
    pub status: TaskStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub tokens_used: u64,
    pub cost_usd: f64,
}
```

### Task Manager（统一调度）

```rust
// l1_action/nt_act/nt_act_scheduler.rs — 增强
impl Scheduler {
    // ── 现有方法保持不变 ──
    
    // ── 新增方法 ──
    
    /// 按工作空间过滤任务
    pub fn list_by_workspace(&self, ws_id: &str) -> Vec<&ScheduledTask>;
    
    /// 按 Agent 过滤任务
    pub fn list_by_agent(&self, agent_id: &str) -> Vec<&ScheduledTask>;
    
    /// 按状态过滤任务
    pub fn list_by_status(&self, status: &TaskStatus) -> Vec<&ScheduledTask>;
    
    /// 获取任务执行历史
    pub fn execution_history(&self, task_id: &str) -> Vec<&TaskExecution>;
    
    /// 暂停任务
    pub async fn pause(&self, task_id: &str) -> Result<(), SchedulerError>;
    
    /// 恢复任务
    pub async fn resume(&self, task_id: &str) -> Result<(), SchedulerError>;
}
```

### 数据流

```
用户创建任务:
    → Scheduler::create_cron(name, instruction, cron, agent_id, skill_ids, ws_id)
    → ScheduledTask { executor_agent_id, skill_ids, workspace_id, ... }
    → register() → KB "scheduled_tasks"

ConsciousnessCore::tick():
    → Scheduler::tick()
    → 检查到期任务
    → 匹配 Agent 能力
    → 执行任务
    → 记录 TaskExecution
    → 更新 deliverables
```

### 实现路径

| 步骤 | 内容 | 复杂度 |
|------|------|--------|
| T1 | ScheduledTask 增加关联字段 | 低 |
| T2 | DeliverableSpec / TaskExecution 类型 | 低 |
| T3 | Scheduler::list_by_workspace() | 低 |
| T4 | Scheduler::pause() / resume() | 中 |
| T5 | 任务依赖图（depends_on 解析） | 高 |

---

## 5. MCP（工具链）

### 现状

```rust
// agent.rs::tool::mcp — C3 成熟度
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub transport: McpTransport,
    pub server_name: String,
    pub schema_version: Option<String>,
}

pub struct McpRegistry {
    servers: Vec<McpServer>,
}

struct McpServer {
    name: String,
    command: String,
    args: Vec<String>,
    tools: Vec<McpToolDef>,
}
```

### 扩展设计

```rust
// 在 agent.rs::tool::mcp 中增加
pub struct McpToolDef {
    // ── 现有字段保持不变 ──
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub transport: McpTransport,
    pub server_name: String,
    pub schema_version: Option<String>,
    
    // ── 新增元数据 ──
    /// 所需权限级别
    pub required_permission: Option<String>,
    /// 安全风险等级
    pub risk_level: RiskLevel,
    /// 使用统计
    pub usage_count: u64,
    /// 平均延迟
    pub avg_latency_ms: f64,
}

pub enum RiskLevel {
    Low,       // 只读操作
    Medium,    // 写操作
    High,      // 删除/执行操作
    Critical,  // 系统级操作
}

// McpRegistry 增强
impl McpRegistry {
    // ── 现有方法保持不变 ──
    
    // ── 新增方法 ──
    
    /// 按 Agent 权限过滤工具
    pub fn tools_for_agent(&self, agent_id: &str) -> Vec<&McpToolDef>;
    
    /// 按风险等级过滤工具
    pub fn tools_by_risk(&self, risk: &RiskLevel) -> Vec<&McpToolDef>;
    
    /// 获取工具使用统计
    pub fn usage_stats(&self) -> Vec<ToolUsageStats>;
    
    /// 注册 SSE 传输服务器
    pub fn register_sse(&mut self, name: &str, url: &str, tools: Vec<McpToolDef>);
}

pub struct ToolUsageStats {
    pub tool_name: String,
    pub server_name: String,
    pub call_count: u64,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
}
```

### 数据流

```
McpRegistry::register_stdio(name, command, args, tools)
    → McpServer { name, command, args, tools }
    → tools 中每个 McpToolDef 带 risk_level

Agent 执行任务时:
    → McpRegistry::tools_for_agent(agent_id)
    → 过滤权限
    → ToolOrchestrator::call(name, args)
    → 记录 usage_count, avg_latency_ms
```

### 实现路径

| 步骤 | 内容 | 复杂度 |
|------|------|--------|
| M1 | McpToolDef 增加元数据字段 | 低 |
| M2 | RiskLevel 类型 | 低 |
| M3 | McpRegistry::tools_for_agent() | 中 |
| M4 | McpRegistry::register_sse() | 中 |
| M5 | 工具使用统计 | 中 |

---

## 6. 实体关系图

```
┌─────────────────────────────────────────────────────────────────┐
│                    ConsciousnessCore (枢纽)                      │
│                    注意力路由 + 意图识别                          │
│                                                                 │
│  tick() → 检查五实体状态 → 分配注意力 → 驱动执行               │
└─────────┬───────────────┬───────────────┬───────────────┬───────┘
          │               │               │               │
          ▼               ▼               ▼               ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│  Workspace   │ │    Agent     │ │    Skill     │ │    Task      │
│  ──────────  │ │  ──────────  │ │  ──────────  │ │  ──────────  │
│  id          │ │  id          │ │  id          │ │  id          │
│  agent_ids ──┼─┤  workspace_id├─┤  source      │ │  workspace_id│
│  skill_ids ──┼─┤  installed_  │ │  triggers    │ │  executor_   │
│  mcp_servers │ │    skills ───┼─┤  maturity    │ │    agent_id ─┼─┤
│  shared_     │ │  mcp_perm ───┼─┤  security    │ │  skill_ids ──┼─┤
│    memory    │ │  role        │ │  audit       │ │  depends_on  │
│  config      │ │  execution   │ │  tags        │ │  deliverables│
│              │ │  status      │ │  author      │ │  history     │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
                              │
                              ▼
                      ┌──────────────┐
                      │     MCP      │
                      │  ──────────  │
                      │  name        │
                      │  transport   │
                      │  risk_level  │
                      │  usage_count │
                      │  tools ──────┼──→ ToolOrchestrator
                      └──────────────┘
```

### 关系矩阵

| 实体 A | 实体 B | 关系 | 字段 |
|--------|--------|------|------|
| Workspace | Agent | 包含 | `workspace.agent_ids` ↔ `agent.workspace_id` |
| Workspace | Skill | 包含 | `workspace.skill_ids` ↔ `skill.source` |
| Workspace | Task | 包含 | `task.workspace_id` |
| Workspace | MCP | 包含 | `workspace.mcp_servers` |
| Agent | Skill | 使用 | `agent.installed_skills` ↔ `skill.id` |
| Agent | Task | 执行 | `task.executor_agent_id` ↔ `agent.id` |
| Agent | MCP | 权限 | `agent.mcp_permissions` ↔ `mcp.tools` |
| Skill | Task | 触发 | `task.skill_ids` ↔ `skill.id` |
| Skill | MCP | 依赖 | `skill.dependencies` → MCP tools |

---

## 7. 状态管理

### KB 持久化

每个实体独立持久化，不互相依赖：

```
KB Namespace       实体            Key 模式
─────────────────────────────────────────────
workspaces         Workspace       ws-{id}
agent_cards        Agent           agent-{id}
skill_candidates   Skill           skill-{id}
scheduled_tasks    Task            task-{id}
mcp_servers        MCP             mcp-{name}
```

### 内存缓存

```
ConsciousnessCore (tick 单例)
    ├── workspace: Option<WorkSpace>         // 当前活跃工作空间
    ├── agents: Vec<AgentCard>               // 当前工作空间的 Agent
    ├── skills: Vec<SkillCandidate>          // 当前工作空间的 Skill
    ├── tasks: Vec<ScheduledTask>            // 当前工作空间的 Task
    └── mcp_tools: Vec<McpToolDef>           // 当前工作空间的 MCP 工具
```

### 事件流

```
Workspace 切换
    → Event::WorkspaceSwitched { ws_id }
    → ConsciousnessCore 重新加载五实体

Agent 状态变更
    → Event::AgentStatusChanged { agent_id, old, new }
    → ConsciousnessCore 调整注意力分配

Task 完成
    → Event::TaskCompleted { task_id, execution }
    → ConsciousnessCore 更新 skill.maturity

MCP 工具调用
    → Event::McpToolCalled { tool_name, latency, success }
    → 更新 McpToolDef.usage_count
```

---

## 8. 实施路线

### Phase 1: 类型扩展 (1 周)

| 日 | 模块 | 内容 | 复杂度 |
|----|------|------|--------|
| D1 | `nt_core_ws.rs` | WorkSpace 增加新字段 | 低 |
| D2 | `nt_infra_agent_card.rs` | AgentCard 增加六维字段 | 中 |
| D3 | `skill_evolution.rs` | SkillCandidate 增加元数据 | 中 |
| D4 | `nt_act_scheduler.rs` | ScheduledTask 增加关联字段 | 低 |
| D5 | `agent.rs::tool::mcp` | McpToolDef 增加元数据 | 低 |

### Phase 2: 方法增强 (1 周)

| 日 | 模块 | 内容 | 复杂度 |
|----|------|------|--------|
| D6 | `nt_core_ws.rs` | WorkSpaceManager::add_agent() 等 | 低 |
| D7 | `nt_infra_agent_card.rs` | AgentCardRegistry::find_by_workspace() | 低 |
| D8 | `skill_evolution.rs` | SkillRegistry::find_by_workspace() | 中 |
| D9 | `nt_act_scheduler.rs` | Scheduler::list_by_workspace() | 低 |
| D10 | `agent.rs::tool::mcp` | McpRegistry::tools_for_agent() | 中 |

### Phase 3: 集成 (1 周)

| 日 | 模块 | 内容 | 复杂度 |
|----|------|------|--------|
| D11 | `consciousness_core/core.rs` | tick() 加载五实体上下文 | 中 |
| D12 | `nt_core_event` | 五实体事件类型 | 中 |
| D13 | `presets.rs` | 预置 Agent 模板库 | 低 |
| D14 | 全链路 | 单元测试 + 集成测试 | 中 |
| D15 | 文档 | ARCHITECTURE.md 更新 | 低 |

### 成功指标

| 指标 | 当前 | Phase 1 后 | Phase 2 后 | Phase 3 后 |
|------|------|-----------|-----------|-----------|
| Workspace 字段 | 10 | 16 | 16 | 16 |
| Agent 维度 | 2 | 8 | 8 | 8 |
| Skill 元数据 | 4 | 10 | 10 | 10 |
| Task 关联 | 0 | 5 | 5 | 5 |
| MCP 元数据 | 4 | 8 | 8 | 8 |
| 跨实体查询 | 0 | 0 | 5 | 5 |
| 事件类型 | 0 | 0 | 0 | 5 |

---

> 本蓝图遵循:
> - R-P111-R-P115 架构管理规则
> - R-P1 零 unsafe
> - nt_ 前缀规范
> - L0→L6 分层约束
> 
> 吸收源: Kingdee Lingee EAOS 五实体模型 (2026-05-20)
> 设计原则: 人类定规则，AI 做执行
