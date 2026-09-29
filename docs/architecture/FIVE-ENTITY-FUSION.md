# NeoTrix 五实体融合架构

> ⚠️ **已废弃（SUPERSEDED）** —— 被 `FIVE-ENTITY-BLUEPRINT-V2.md`（v2.0.0，2026-09-22）替代。
> 实施以 v2（E3 节收录本文路由＋事件设计）为准，本文仅作历史参考。

> 日期：2026-09-22
> 前置文档：FIVE-ENTITY-BLUEPRINT.md

---

## 0. 融合问题

蓝图定义了五个独立实体。融合要回答：

1. **tick 融合** — tick() 循环如何同时感知五实体？
2. **路由融合** — CAPABILITY_ROUTES 如何从关键词匹配进化为实体感知？
3. **事件融合** — CoreEvent 如何表达实体生命周期？
4. **上下文融合** — 五实体如何共享工作空间上下文？

---

## 1. 现状分析

### 1.1 已有的融合机制

```
tick() 循环 (consciousness_core/core.rs)
    → run_growth_cycle()        // 意识树生长
    → set_branch_health()       // 分支健康
    → compute_iit_phi()         // 整合信息
    → persist_snapshot()        // KB 写回

CAPABILITY_ROUTES (dispatch.rs)
    → 2191 行硬编码映射: 关键词 → 能力 → 层 → Agent角色
    → "excel" → xlsx_consolidation → NT-ACT → CodeAnalyzer
    → 无实体感知，无工作空间感知

CoreEvent (nt_core_event.rs)
    → TaskSubmitted / AgentFeedback / GoalCompleted / ...
    → 无 Workspace/Skill/MCP 事件
```

### 1.2 融合断裂点

| 断裂点 | 现状 | 后果 |
|--------|------|------|
| tick 不感知 workspace | tick() 无 workspace 参数 | 无法按工作空间隔离实体 |
| CAPABILITY_ROUTES 硬编码 | 2191 行静态表 | 无法动态添加 Agent/Skill |
| CoreEvent 缺实体事件 | 无 Workspace/Skill/MCP 事件 | 无法追踪实体生命周期 |
| 无上下文传播 | Agent 不知道当前 workspace | 无法共享共享记忆 |

---

## 2. 融合设计

### 2.1 tick 融合 — Workspace 上下文注入

```
tick() 当前:
    ConsciousnessCoreHandle::tick(cycles)
        → run_growth_cycle()
        → persist_snapshot()

tick() 融合后:
    ConsciousnessCoreHandle::tick(cycles, workspace_id)
        │
        ├─ 1. 加载 Workspace 上下文
        │     WorkSpaceManager::get(ws_id)
        │     → workspace.agent_ids, skill_ids, mcp_servers
        │
        ├─ 2. 加载实体状态
        │     AgentCardRegistry::find_by_workspace(ws_id)  → agents
        │     SkillRegistry::find_by_workspace(ws_id)      → skills
        │     Scheduler::list_by_workspace(ws_id)          → tasks
        │     McpRegistry::tools_by_workspace(ws_id)       → tools
        │
        ├─ 3. 注入到 CoreSnapshot
        │     snapshot.workspace_context = WorkspaceContext {
        │         workspace_id,
        │         active_agents,
        │         active_skills,
        │         active_tasks,
        │         available_tools,
        │     }
        │
        ├─ 4. 运行意识树生长 (现有逻辑不变)
        │     run_growth_cycle()
        │
        └─ 5. 持久化 (现有逻辑不变)
              persist_snapshot()
```

**关键设计**：tick() 签名不变（向后兼容），workspace_id 通过 snapshot 传递。

```rust
// CoreSnapshot 新增字段
pub struct CoreSnapshot {
    // ── 现有字段不变 ──
    pub cycle: u64,
    pub phi: f64,
    pub coherence: f64,
    // ...
    
    // ── 新增：工作空间上下文 ──
    #[serde(default)]
    pub workspace_context: Option<WorkspaceContext>,
}

pub struct WorkspaceContext {
    pub workspace_id: String,
    pub active_agents: Vec<String>,    // Agent ID 列表
    pub active_skills: Vec<String>,    // Skill ID 列表
    pub active_tasks: Vec<String>,     // Task ID 列表
    pub available_tools: Vec<String>,  // MCP 工具名列表
}
```

### 2.2 路由融合 — 从硬编码到实体感知

#### 现有：静态 CAPABILITY_ROUTES

```rust
// dispatch.rs — 2191 行硬编码
const CAPABILITY_ROUTES: &[(&str, &str, &str, &str)] = &[
    ("excel", "xlsx_consolidation", "NT-ACT", "CodeAnalyzer"),
    // ...
];
```

#### 融合：三层路由

```
┌─────────────────────────────────────────────────────────────┐
│                    三层路由架构                               │
│                                                             │
│  Layer 1: 静态路由 (CAPABILITY_ROUTES)                      │
│    关键词 → 能力 → 层 → Agent角色                           │
│    用途: 兜底路由，新实体未注册时的回退                       │
│                                                             │
│  Layer 2: Agent 动态路由 (AgentCardRegistry)                │
│    Agent 声明 capabilities → 按能力匹配                     │
│    用途: 已注册 Agent 的能力匹配                             │
│                                                             │
│  Layer 3: Skill 动态路由 (SkillRegistry)                    │
│    Skill 声明 triggers → 按触发条件匹配                     │
│    用途: 已安装 Skill 的触发匹配                             │
└─────────────────────────────────────────────────────────────┘
```

#### 路由优先级

```
用户输入
    │
    ▼
Layer 3: Skill 触发匹配 (最高优先)
    → 匹配成功 → 加载 Skill → 注入 Agent 执行
    │
    ▼ (未匹配)
Layer 2: Agent 能力匹配
    → 匹配成功 → 选择 Agent → 分配任务
    │
    ▼ (未匹配)
Layer 1: 静态路由 (兜底)
    → 匹配成功 → 走现有 dispatch 逻辑
    │
    ▼ (未匹配)
Layer 0: LLM 直接推理 (最低优先)
```

#### 路由融合实现

```rust
// dispatch.rs 新增函数
pub fn route_entity_aware(
    input: &str,
    workspace: &WorkspaceContext,
    agent_registry: &AgentCardRegistry,
    skill_registry: &SkillRegistry,
) -> RouteDecision {
    // Layer 3: Skill 触发匹配
    if let Some(skill) = skill_registry.match_trigger(input) {
        return RouteDecision::Skill {
            skill_id: skill.id.clone(),
            agent_id: skill.find_best_agent(workspace),
        };
    }
    
    // Layer 2: Agent 能力匹配
    if let Some(agent) = agent_registry.match_capabilities(input, workspace) {
        return RouteDecision::Agent {
            agent_id: agent.id.clone(),
        };
    }
    
    // Layer 1: 静态路由 (现有逻辑)
    if let Some((capability, layer, role)) = static_route(input) {
        return RouteDecision::Static {
            capability: capability.to_string(),
            layer: layer.to_string(),
            role: role.to_string(),
        };
    }
    
    // Layer 0: LLM 直接推理
    RouteDecision::DirectLlm
}

pub enum RouteDecision {
    Skill { skill_id: String, agent_id: Option<String> },
    Agent { agent_id: String },
    Static { capability: String, layer: String, role: String },
    DirectLlm,
}
```

### 2.3 事件融合 — 实体生命周期事件

```rust
// nt_core_event.rs 新增事件变体
pub enum CoreEvent {
    // ── 现有事件不变 ──
    TaskSubmitted { ... },
    AgentFeedback { ... },
    // ...
    
    // ── 新增：Workspace 事件 ──
    #[serde(rename = "workspace_switched")]
    WorkspaceSwitched {
        workspace_id: String,
        agent_count: usize,
        skill_count: usize,
    },
    
    #[serde(rename = "workspace_agent_added")]
    WorkspaceAgentAdded {
        workspace_id: String,
        agent_id: String,
    },
    
    #[serde(rename = "workspace_agent_removed")]
    WorkspaceAgentRemoved {
        workspace_id: String,
        agent_id: String,
    },
    
    // ── 新增：Agent 事件 ──
    #[serde(rename = "agent_status_changed")]
    AgentStatusChanged {
        agent_id: String,
        old_status: String,
        new_status: String,
    },
    
    #[serde(rename = "agent_task_delegated")]
    AgentTaskDelegated {
        from_agent: String,
        to_agent: String,
        task_id: String,
    },
    
    // ── 新增：Skill 事件 ──
    #[serde(rename = "skill_installed")]
    SkillInstalled {
        skill_id: String,
        workspace_id: String,
        version: u32,
    },
    
    #[serde(rename = "skill_executed")]
    SkillExecuted {
        skill_id: String,
        agent_id: String,
        success: bool,
        duration_ms: u64,
    },
    
    #[serde(rename = "skill_maturity_changed")]
    SkillMaturityChanged {
        skill_id: String,
        old_maturity: String,
        new_maturity: String,
    },
    
    // ── 新增：Task 事件 ──
    #[serde(rename = "task_assigned")]
    TaskAssigned {
        task_id: String,
        agent_id: String,
        workspace_id: String,
    },
    
    #[serde(rename = "task_completed")]
    TaskCompletedV2 {
        task_id: String,
        execution_id: String,
        success: bool,
        deliverables: Vec<String>,
    },
    
    // ── 新增：MCP 事件 ──
    #[serde(rename = "mcp_tool_called")]
    McpToolCalled {
        tool_name: String,
        server_name: String,
        agent_id: String,
        latency_ms: u64,
        success: bool,
    },
    
    #[serde(rename = "mcp_server_connected")]
    McpServerConnected {
        server_name: String,
        tool_count: usize,
    },
}
```

### 2.4 上下文融合 — 共享记忆

```
┌─────────────────────────────────────────────────────────────┐
│                    上下文传播路径                             │
│                                                             │
│  Workspace.shared_memory_keys                               │
│       │                                                     │
│       ▼                                                     │
│  ConsciousnessCore::tick()                                  │
│       │                                                     │
│       ├──→ Agent execution context                          │
│       │         │                                           │
│       │         ├──→ Skill parameters (从共享记忆读取)       │
│       │         │                                           │
│       │         ├──→ MCP tool arguments (从共享记忆读取)     │
│       │         │                                           │
│       │         └──→ Task deliverables (写入共享记忆)        │
│       │                                                     │
│       └──→ KB 持久化 (共享记忆落盘)                         │
└─────────────────────────────────────────────────────────────┘
```

#### 共享记忆类型

```rust
// Workspace 增加
pub struct Workspace {
    // ...
    pub shared_memory_keys: Vec<String>,  // KB key 列表
}

// 执行时上下文
pub struct ExecutionContext {
    pub workspace_id: String,
    pub agent_id: String,
    pub task_id: Option<String>,
    pub skill_id: Option<String>,
    pub shared_memory: serde_json::Value,  // 从 KB 加载的共享数据
    pub available_tools: Vec<McpToolDef>,
}
```

---

## 3. 融合流程图

```
用户输入: "帮我合并这个 Excel 文件"
    │
    ▼
┌──────────────────────────────────────────────────────┐
│ 1. 意图识别 (dispatch.rs)                            │
│    关键词: "合并" "Excel"                             │
│    Layer 3: Skill 触发匹配                           │
│    → Skill "xlsx_consolidation" 触发条件匹配         │
│    → 返回 RouteDecision::Skill { skill_id, agent_id }│
└──────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────┐
│ 2. Agent 选择                                        │
│    Skill 声明: required_agent = "CodeAnalyzer"       │
│    AgentCardRegistry::find_by_capability("xlsx")     │
│    → 选择 Agent "code-analyzer"                      │
│    → Event::AgentStatusChanged { Busy }              │
└──────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────┐
│ 3. Skill 加载                                        │
│    SkillRegistry::load("xlsx_consolidation")         │
│    → SkillCandidate { triggers, parameters, ... }    │
│    → Event::SkillExecuted { started }                │
└──────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────┐
│ 4. MCP 工具调用                                      │
│    Skill 依赖: MCP tool "file_read"                  │
│    McpRegistry::tools_for_agent("code-analyzer")     │
│    → 权限检查通过                                     │
│    → ToolOrchestrator::call("file_read", args)       │
│    → Event::McpToolCalled { latency_ms, success }    │
└──────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────┐
│ 5. Task 跟踪                                         │
│    Scheduler::create_one_shot(                       │
│        name: "合并 Excel",                            │
│        executor_agent_id: "code-analyzer",           │
│        skill_ids: ["xlsx_consolidation"],            │
│        workspace_id: "ws-abc123",                    │
│    )                                                 │
│    → Event::TaskAssigned { task_id, agent_id }       │
└──────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────┐
│ 6. 执行与交付                                        │
│    Agent 执行 Skill 逻辑                              │
│    → 生成 deliverable: "merged_output.xlsx"          │
│    → 写入 Workspace.shared_memory                    │
│    → Event::TaskCompletedV2 { deliverables }         │
│    → Event::SkillMaturityChanged { +evaluation }     │
└──────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────┐
│ 7. 反馈循环                                          │
│    ConsciousnessCore 读取事件                         │
│    → 更新 Skill.maturity (bandit feedback)           │
│    → 更新 Agent.metrics (usage stats)                │
│    → 更新 MCP.tools (latency stats)                  │
│    → 写回 KB                                         │
└──────────────────────────────────────────────────────┘
```

---

## 4. 融合协议

### 4.1 实体间通信协议

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│Workspace│────→│  Agent  │────→│  Skill  │
│         │←────│         │←────│         │
└────┬────┘     └────┬────┘     └────┬────┘
     │               │               │
     │               ▼               │
     │         ┌─────────┐          │
     │         │  Task   │          │
     │         │         │          │
     │         └────┬────┘          │
     │              │               │
     ▼              ▼               ▼
┌─────────────────────────────────────────┐
│              MCP (工具执行)              │
│  Workspace.mcp_servers → 工具调用       │
│  Agent.mcp_permissions → 权限控制       │
│  Skill.dependencies → 工具依赖          │
└─────────────────────────────────────────┘
```

### 4.2 数据流协议

```
写入协议:
    Agent 写入 → Workspace.shared_memory (via KB)
    Task 完成 → Skill.maturity (via bandit feedback)
    MCP 调用 → McpToolDef.usage_count (via stats)

读取协议:
    Agent 读取 ← Workspace.shared_memory
    Skill 参数 ← Workspace.shared_memory
    Task 上下文 ← Workspace.context

持久化协议:
    每个实体独立持久化到 KB namespace
    Workspace → "workspaces"
    Agent → "agent_cards"
    Skill → "skill_candidates"
    Task → "scheduled_tasks"
    MCP → "mcp_servers"
```

### 4.3 错误传播协议

```
MCP 工具失败
    → McpToolCalled { success: false }
    → Skill 感知失败 → 重试/降级
    → Agent 感知失败 → 记录到 history
    → Task 感知失败 → retry_count += 1
    → Workspace 感知失败 → 通知用户

Agent 超时
    → AgentStatusChanged { Resting }
    → Task 重新分配 → 另一个 Agent
    → Event::AgentTaskDelegated { from, to, task_id }
```

---

## 5. 实施路径

### Phase 1: tick 融合 (3 天)

| 日 | 内容 | 复杂度 |
|----|------|--------|
| D1 | CoreSnapshot 增加 WorkspaceContext 字段 | 低 |
| D2 | tick() 加载 workspace 上下文 | 中 |
| D3 | 单元测试 + 向后兼容验证 | 低 |

### Phase 2: 路由融合 (5 天)

| 日 | 内容 | 复杂度 |
|----|------|--------|
| D4 | RouteDecision 枚举定义 | 低 |
| D5 | route_entity_aware() 实现 Layer 3 (Skill) | 中 |
| D6 | route_entity_aware() 实现 Layer 2 (Agent) | 中 |
| D7 | 集成到 dispatch.rs | 中 |
| D8 | 路由测试 + 回归验证 | 中 |

### Phase 3: 事件融合 (3 天)

| 日 | 内容 | 复杂度 |
|----|------|--------|
| D9 | CoreEvent 新增实体事件变体 | 低 |
| D10 | 事件发布点 (tick/execute/complete) | 中 |
| D11 | 事件消费点 (maturity/metrics/stats) | 中 |

### Phase 4: 上下文融合 (4 天)

| 日 | 内容 | 复杂度 |
|----|------|--------|
| D12 | ExecutionContext 类型定义 | 低 |
| D13 | 共享记忆读写 API | 中 |
| D14 | Agent/Skill/Task 集成 ExecutionContext | 高 |
| D15 | 全链路集成测试 | 中 |

---

## 6. 成功指标

| 指标 | 融合前 | 融合后 |
|------|--------|--------|
| tick 感知实体数 | 0 | 5 |
| 路由层数 | 1 (静态) | 3 (静态+Agent+Skill) |
| 事件类型 | 11 | 22 |
| 上下文传播 | 无 | Workspace→Agent→Skill→Task→MCP |
| 实体间通信 | 无 | 事件驱动 |

---

> 融合原则:
> 1. 不改现有 tick() 签名 (向后兼容)
> 2. 不改现有 CAPABILITY_ROUTES (兜底不变)
> 3. 不改现有 CoreEvent (向后兼容)
> 4. 新增字段全部 #[serde(default)] (反序列化兼容)
>
> 吸收源: Kingdee Lingee EAOS 五实体模型 (2026-05-20)
> 设计原则: 人类定规则，AI 做执行
