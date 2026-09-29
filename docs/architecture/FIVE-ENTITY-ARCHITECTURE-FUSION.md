# NeoTrix 五实体整体架构融合

> ⚠️ **已废弃（SUPERSEDED）** —— 被 `FIVE-ENTITY-BLUEPRINT-V2.md`（v2.0.0，2026-09-22）替代。
> 实施以 v2（E2 单一事实源＋附表路线对位）为准，本文仅作历史参考。

> 日期：2026-09-22
> 前置文档：FIVE-ENTITY-BLUEPRINT.md / FIVE-ENTITY-FUSION.md

---

## 0. 核心命题

五实体（Workspace/Agent/Skill/Task/MCP）不是 NeoTrix 的新层，而是让现有六层架构**作为有机体运转**的骨骼。

问题：NeoTrix 有 2426 个 .rs 文件、56 个项目对标、6 层架构，但它们是**散装零件**。五实体融合要回答：这些零件如何组装成一台能自我进化的机器？

---

## 1. 架构现状：散装零件

```
L6 Meta    ─── governance / sentrux / session_replay / otel_bridge
L5 Cognition ── consciousness_core / byoa / model_router / skill_registry
L4 Emotion  ─── nt_feel / cognitive_bridge
L3 Embodiment ─ nt_shield / pentest_swarm
L2 Perception ─ nt_world / crawl / nexus / sense
L1 Action   ─── nt_act / nt_io / nt_memory / agent_card / scheduler
L0 Substrate ── workspace / event / ecs / telemetry / error
```

**断裂点**：

| 断裂 | 表现 | 后果 |
|------|------|------|
| L0 workspace 和 L5 agent 无连接 | workspace 不知道有哪些 agent | 无法按工作空间隔离 agent |
| L1 scheduler 和 L5 consciousness 无连接 | tick() 不感知任务 | 任务调度和意识脱节 |
| L1 agent_card 和 L5 skill_registry 无连接 | agent 不知道自己有哪些技能 | 技能匹配靠硬编码 |
| L1 mcp_registry 和 L5 capability 无连接 | 工具调用不感知能力 | 无法按能力分配工具 |
| L6 governance 和 L1 action 无连接 | 治理规则不约束执行 | 安全规则形同虚设 |

---

## 2. 融合设计：五实体作为骨骼

### 2.1 实体 = 层间的骨骼

```
L6 Meta ────────────────────────────────────────────────
         │                                              │
         │  Skill (进化单元)                             │
         │  ├── 质量评分 → 触发进化                      │
         │  ├── 安全审计 → 治理门控                      │
         │  └── 使用统计 → 反馈循环                      │
         │                                              │
L5 Cognition ───────────────────────────────────────────
         │                                              │
         │  Agent (认知单元)                             │
         │  ├── 六维定义 → 能力边界                      │
         │  ├── 模型偏好 → 路由决策                      │
         │  └── 状态机 → 注意力分配                      │
         │                                              │
L4 Emotion ─────────────────────────────────────────────
         │                                              │
         │  (情感驱动 Agent 状态转换)                    │
         │                                              │
L3 Embodiment ──────────────────────────────────────────
         │                                              │
         │  (安全约束 Agent/MCP 执行)                    │
         │                                              │
L2 Perception ──────────────────────────────────────────
         │                                              │
         │  (感知驱动 Skill 触发)                        │
         │                                              │
L1 Action ──────────────────────────────────────────────
         │                                              │
         │  Task (执行单元)                              │
         │  ├── 调度 → 按 Agent/Skill 分配               │
         │  ├── 依赖 → 任务间关联                        │
         │  └── 交付物 → 结果追踪                        │
         │                                              │
         │  MCP (工具单元)                               │
         │  ├── 权限 → 按 Agent 分配                     │
         │  ├── 风险 → 按级别分类                        │
         │  └── 统计 → 反馈到 Skill 质量                 │
         │                                              │
L0 Substrate ───────────────────────────────────────────
              │
              │  Workspace (容器单元)
              │  ├── 包含 → Agent/Skill/Task/MCP
              │  ├── 共享记忆 → 跨实体上下文
              │  └── 配置 → 实体行为参数
```

### 2.2 关系矩阵：谁包含谁

```
Workspace ──contains──→ Agent (workspace.agent_ids)
Workspace ──contains──→ Skill (workspace.skill_ids)
Workspace ──contains──→ Task (task.workspace_id)
Workspace ──contains──→ MCP  (workspace.mcp_servers)

Agent ──uses──→ Skill (agent.installed_skills)
Agent ──executes──→ Task (task.executor_agent_id)
Agent ──permissions──→ MCP (agent.mcp_permissions)

Skill ──triggers──→ Task (task.skill_ids)
Skill ──depends_on──→ MCP (skill.dependencies → tools)

Task ──produces──→ Deliverables (task.deliverables)
Task ──records──→ Execution History (task.history)
```

---

## 3. 融合脉络：四条进化主线

### 主线 1: 意识驱动（ConsciousnessCore → 五实体）

```
tick() 循环
    │
    ├─ 读取 Workspace 上下文
    │   → workspace.agent_ids / skill_ids / mcp_servers
    │
    ├─ 加载实体状态
    │   → Agent 状态 (Available/Thinking/Busy/Resting)
    │   → Skill 成熟度 (Candidate/Provisional/Trusted/Retired)
    │   → Task 状态 (Pending/Running/Paused/Completed/Failed)
    │   → MCP 工具统计 (usage_count/avg_latency)
    │
    ├─ 意识树生长 (现有逻辑)
    │   → run_growth_cycle()
    │   → compute_iit_phi()
    │
    └─ 输出: 注意力分配决策
        → 哪个 Agent 醒着
        → 哪个 Skill 被选中
        → 哪个 Task 被调度
        → 哪个 MCP 工具被调用
```

**进化方向**：tick() 从"自省循环"进化为"实体调度器"。

### 主线 2: 能力路由（CAPABILITY_ROUTES → 三层路由）

```
现有: 2191 行硬编码映射
    "excel" → xlsx_consolidation → NT-ACT → CodeAnalyzer

进化后: 三层动态路由
    Layer 3: Skill 触发匹配 (最高优先)
        → Skill 声明 triggers → 按触发条件匹配
        → 新 Skill 注册即生效，无需改代码

    Layer 2: Agent 能力匹配
        → Agent 声明 capabilities → 按能力匹配
        → 新 Agent 注册即生效，无需改代码

    Layer 1: 静态路由 (兜底)
        → 现有 CAPABILITY_ROUTES 不变
        → 向后兼容
```

**进化方向**：从硬编码到声明式，新实体注册即路由。

### 主线 3: 事件驱动（CoreEvent → 实体生命周期）

```
现有事件:
    TaskSubmitted / AgentFeedback / GoalCompleted / ...

新增事件:
    Workspace: WorkspaceSwitched / WorkspaceAgentAdded / ...
    Agent:     AgentStatusChanged / AgentTaskDelegated / ...
    Skill:     SkillInstalled / SkillExecuted / SkillMaturityChanged / ...
    Task:      TaskAssigned / TaskCompletedV2 / ...
    MCP:       McpToolCalled / McpServerConnected / ...

事件消费:
    SkillMaturityChanged → 更新 bandit 反馈 → Skill 进化
    AgentStatusChanged → 调整注意力分配 → tick() 决策
    McpToolCalled → 更新使用统计 → 工具质量评分
    TaskCompleted → 记录交付物 → 追踪产出
```

**进化方向**：从无状态到事件驱动，实体状态变更可追踪。

### 主线 4: 知识进化（SEAL 管线 → 实体反馈）

```
SEAL 管线现有:
    领域知识 → 蒸馏 → 渗透 → 消化 → 果实

融合后:
    Skill 执行 → 质量评分 → bandit 反馈 → Skill 进化
    Agent 执行 → 成本/延迟统计 → Agent 优化
    MCP 调用 → 使用/错误统计 → 工具质量
    Task 完成 → 交付物质量 → 任务模板优化

反馈循环:
    执行结果 → 实体质量评分 → 进化决策 → 实体升级
```

**进化方向**：从单向蒸馏到双向反馈，实体在使用中进化。

---

## 4. 与现有迭代路线的对齐

### 现有路线图

```
Phase 0: 基础协议对齐 (MCP + 技能编排)
Phase 1: 记忆与可观测
Phase 2: 安全加固
Phase 3: OSINT 扩展
Phase 4: 认知增强
Phase 5: 独有能力闭环
```

### 五实体融合如何嵌入

```
Phase 0 (基础协议)
    I0.1 MCP 协议层 → 直接对接 MCP 实体设计
    I0.2 技能编排   → 直接对接 Skill 实体设计
    新增: Workspace 实体基础 (L0)

Phase 1 (记忆与可观测)
    I1.1 统一记忆层 → Workspace.shared_memory
    I1.3 OTel 可观测 → 实体事件 → CoreEvent 扩展
    新增: Agent 六维定义 (L5)

Phase 2 (安全加固)
    I2.1-I2.5 安全   → MCP.risk_level + Skill.security_audit
    新增: Agent 权限分级 + MCP 权限控制

Phase 3 (OSINT 扩展)
    I3.1-I3.4 OSINT  → Agent 按能力路由到 OSINT 技能
    新增: Skill 触发匹配 + 三层路由

Phase 4 (认知增强)
    I4.1 状态图工作流 → Task 依赖图 + 调度器
    I4.2 角色多 Agent → Agent 协作 + 委托
    新增: Task 完整生命周期

Phase 5 (独有能力闭环)
    I5.1 自愈诊断    → Task 失败 → 自动重试/修复
    I5.2 情感-认知耦合 → Agent 状态受情感驱动
    I5.3 跨会话编织   → Workspace 跨会话记忆
    新增: 五实体完整融合 + 反馈循环
```

---

## 5. 进化脉络图

```
                    NeoTrix 进化主线
                    
Phase 0 ──── Phase 1 ──── Phase 2 ──── Phase 3 ──── Phase 4 ──── Phase 5
  │              │              │              │              │              │
  ▼              ▼              ▼              ▼              ▼              ▼
┌──────┐    ┌──────┐    ┌──────┐    ┌──────┐    ┌──────┐    ┌──────┐
│Workspace│  │Agent │    │MCP   │    │Skill │    │Task  │    │完整  │
│基础    │  │六维  │    │权限  │    │触发  │    │依赖  │    │融合  │
│容器    │  │定义  │    │分级  │    │匹配  │    │图    │    │反馈  │
└──────┘    └──────┘    └──────┘    └──────┘    └──────┘    └──────┘
  │              │              │              │              │              │
  │              │              │              │              │              │
  ▼              ▼              ▼              ▼              ▼              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        意识驱动 (tick)                                  │
│  tick() 加载 Workspace → 感知五实体 → 注意力分配 → 驱动执行              │
└─────────────────────────────────────────────────────────────────────────┘
  │              │              │              │              │              │
  ▼              ▼              ▼              ▼              ▼              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        能力路由 (三层)                                  │
│  Layer 3: Skill 触发 → Layer 2: Agent 能力 → Layer 1: 静态兜底           │
└─────────────────────────────────────────────────────────────────────────┘
  │              │              │              │              │              │
  ▼              ▼              ▼              ▼              ▼              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        事件驱动 (CoreEvent)                             │
│  实体状态变更 → 事件发布 → 其他实体感知 → 调整行为                       │
└─────────────────────────────────────────────────────────────────────────┘
  │              │              │              │              │              │
  ▼              ▼              ▼              ▼              ▼              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        知识进化 (SEAL + bandit)                         │
│  执行结果 → 质量评分 → bandit 反馈 → 实体进化 → 能力提升                  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 6. 关键设计约束

### 约束 1: 向后兼容

所有新增字段 `#[serde(default)]`，不破坏现有反序列化。

```
现有代码读取 WorkSpace → 正常工作 (新字段用默认值)
新代码读取 WorkSpace → 使用完整字段
```

### 约束 2: 不改 tick() 签名

```rust
// 现有签名不变
pub fn tick(cycles: usize) -> CoreSnapshot

// workspace_id 通过 snapshot 传递
pub struct CoreSnapshot {
    // ... 现有字段
    #[serde(default)]
    pub workspace_context: Option<WorkspaceContext>,
}
```

### 约束 3: 不改 CAPABILITY_ROUTES

现有 2191 行硬编码表作为 Layer 1 兜底，新路由在其之上。

### 约束 4: 实体独立持久化

每个实体独立 KB namespace，不互相依赖：

```
workspaces  → Workspace
agent_cards → Agent
skill_candidates → Skill
scheduled_tasks → Task
mcp_servers → MCP
```

---

## 7. 进化终点

```
Phase 0-2: 实体类型定义 + 基础方法
    → 五实体有完整的类型系统
    → 每个实体有 CRUD + 查询方法

Phase 3-4: 实体融合 + 事件驱动
    → tick() 感知五实体
    → 三层路由取代硬编码
    → 事件驱动实体间通信

Phase 5: 实体进化 + 反馈闭环
    → 实体在使用中进化
    → bandit 反馈优化 Skill
    → 任务执行反馈优化 Agent
    → 工具使用反馈优化 MCP

终点: NeoTrix 成为一个
    → 有骨骼 (五实体)
    → 有神经 (事件系统)
    → 有血液 (共享记忆)
    → 有免疫 (治理约束)
    → 能自我进化 (SEAL + bandit)
    的有机体
```

---

> 融合原则:
> 1. 五实体是骨骼，不是新器官
> 2. 融合发生在 tick() / 路由 / 事件 / 进化四条主线
> 3. 每条主线对应一个现有机制的增强
> 4. 向后兼容，不破坏现有代码
>
> 吸收源: Kingdee Lingee EAOS 五实体模型 (2026-05-20)
> 设计原则: 人类定规则，AI 做执行
