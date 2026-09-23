# NeoTrix 五实体架构图

> 图表即代码（Mermaid）。正典蓝图：`FIVE-ENTITY-BLUEPRINT-V3.md`；任务清单：`FIVE-ENTITY-TASK-CHECKLIST.md`。
> 对应代码版本：2026-09-22（含 P0-3/P0-2/E0.0 已落地项）。

---

## 图1 全景：六层＋晶体内核＋桌面端

```mermaid
flowchart TB
    User([用户]) <--> Desktop[桌面端 Tauri<br/>BrowserHost / AuthBridge]
    Desktop <--> L6["L6 Meta<br/>治理 / 哨兵 / 回放"]
    L6 <--> L5["L5 Cognition<br/>意识核心 / AgentCard / Skill"]
    L5 <--> L4["L4 Emotion<br/>情感引擎"]
    L4 <--> L3["L3 Embodiment<br/>护盾 / 审批"]
    L3 <--> L2["L2 Perception<br/>世界模型 / 爬虫"]
    L2 <--> L1["L1 Action<br/>调度器 / MCP / 工具"]
    L1 <--> L0["L0 Substrate<br/>Workspace / 事件 / 遥测"]
    L0 <--> Crystal["CrystalState（单一事实源）<br/>四层＋能力坐标＋五投影＋执行追踪"]
    Crystal <--> KB[(KB 持久化<br/>workspaces / agent_cards /<br/>skill_candidates / scheduled_tasks /<br/>mcp_servers / crystal_state.json)]
```

## 图2 五实体关系（ER）

```mermaid
erDiagram
    WORKSPACE ||--o{ AGENT : "agent_ids 包含"
    WORKSPACE ||--o{ SKILL : "skill_ids 包含"
    WORKSPACE ||--o{ MCP : "mcp_servers 包含"
    WORKSPACE ||--o{ TASK : "workspace_id 归属"
    AGENT ||--o{ SKILL : "installed_skills 使用"
    AGENT ||--o{ TASK : "executor_agent_id 执行"
    AGENT ||--o{ MCP : "mcp_permissions 授权"
    SKILL ||--o{ TASK : "skill_ids 触发"
    SKILL ||--o{ MCP : "dependencies 依赖"
    TASK ||--o{ DELIVERABLE : "deliverables 产出"
    CRYSTAL ||--|| WORKSPACE : "workspace 投影"
    CRYSTAL ||--o{ AGENT : "agents 投影"
    CRYSTAL ||--o{ SKILL : "skills 投影"
    CRYSTAL ||--o{ TASK : "tasks 投影"
    CRYSTAL ||--o{ MCP : "tools 投影"
```

## 图3 tick 接线时序（E2）

```mermaid
sequenceDiagram
    participant Tick as ConsciousnessCore.tick (同步)
    participant WS as WorkSpaceManager
    participant Reg as 注册表群 (Card/Loader/镜像/MCP)
    participant CS as CrystalState
    participant Tree as 意识树 growth_cycle
    participant KB as KB
    Tick->>WS: 读 active workspace
    WS-->>Tick: agent_ids / skill_ids / mcp_servers
    Tick->>Reg: 加载五实体 (Task 只读同步镜像)
    Reg-->>Tick: agents / skills / tasks / tools
    Tick->>CS: advance_tick + 写 workspace_context
    Tick->>Tree: run_growth_cycle (原有逻辑不变)
    Tree-->>Tick: phi / coherence
    Tick->>KB: persist_snapshot + crystal_state.json
```

## 图4 三层路由（E3）

```mermaid
flowchart TD
    In([用户输入]) --> L3{Layer 3<br/>Skill 触发子串匹配}
    L3 -- 命中 --> RS[RouteDecision::Skill<br/>加载 Skill → 注入 Agent]
    L3 -- 未命中 --> L2{Layer 2<br/>Agent 能力匹配}
    L2 -- 命中 --> RA[RouteDecision::Agent<br/>选择 Agent → 分配任务]
    L2 -- 未命中 --> L1{Layer 1<br/>CAPABILITY_ROUTES 静态兜底}
    L1 -- 命中 --> RT[RouteDecision::Static<br/>走现有 dispatch]
    L1 -- 未命中 --> LLM[RouteDecision::DirectLlm]
    RS --> EX[执行]
    RA --> EX
    RT --> EX
    LLM --> EX
```

## 图5 首条数据流（E2 最小闭环）

```mermaid
sequenceDiagram
    participant Entry as entry/mod.rs · headless.rs
    participant Loader as SkillLoader
    participant CS as CrystalState
    participant Cand as SkillCandidate
    participant Bandit as bandit (maturity)
    Entry->>Loader: load/use skill (SKILL.md 上下文)
    Entry->>CS: record_execution (首次被外围调用)
    CS-->>CS: execution_trace +1
    CS->>Cand: performance_history 反馈
    Cand->>Bandit: 更新 maturity
    Bandit-->>Entry: 下次路由参考 (Admitted +8.0)
```

## 图6 401 回灌链（P0-3 ✅已落地）

```mermaid
sequenceDiagram
    participant BE as 后端 (vault/engine)
    participant Bridge as AuthBridge
    participant FE as 前端 (subscribeAuthBridge)
    BE->>Bridge: 401 / token 失效
    Bridge->>FE: emit token-expired {reason, at_ms}
    FE->>FE: 静默续签
    alt 续签成功
        FE-->>BE: 继续会话 (用户无感)
    else 续签失败
        Bridge->>FE: emit require-login
        FE-->>FE: 跳转登录页
    end
```

## 图7 实施路线依赖（T01–T37）

```mermaid
flowchart LR
    T01([T01 基线✅]) --> T02([T02 改门牌✅])
    T02 --> T03([T03 import重写])
    T03 --> T09([T09 模型档位])
    T03 --> T04([T04 facade])
    T04 --> T05([T05 From上移])
    T05 --> T06([T06 信任环])
    T03 --> E1([T10–T21<br/>E1 实体正典])
    T08([T08 门禁✅]) --> E1
    E1 --> E2([T22–T26<br/>E2 投影接线])
    E2 --> E3([T27–T29<br/>E3 路由事件])
    E3 --> E4([T30✅/T31–T37<br/>文档＋桌面后续])
    T07([T07 回灌✅]) --> T09
    T07 --> T31([T31–T34 P1])
    T31 --> T35([T35–T37 P2])
```

---

## 图例与状态（2026-09-22）

- ✅ 已落地：T01 / T07 / T08 / T30（P0-2 门禁逻辑 2/2 独立通过；P0-3 tsc exit 0）
- ⬜ 下一個：T02 ✅（本窗刚完成）→ T03 → T09
- 约束：tick 同步（Scheduler 异步墙→同步镜像）；新增字段 `#[serde(default)]`；生产禁 unwrap；`nt_` 前缀。
