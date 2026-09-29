# NeoTrix 五实体统一蓝图 v2

> ⚠️ **已废弃（SUPERSEDED）** —— 被 `FIVE-ENTITY-BLUEPRINT-V3.md`（v3.0.0 最终版，2026-09-22）替代。
> 实施以 v3 为准，本文仅作历史参考。

> 版本：v2.0.0（2026-09-22）
> 地位：**唯一正典**，替代并废弃以下三份 v1 文档：
> - `FIVE-ENTITY-BLUEPRINT.md`（v1，实体定义）
> - `FIVE-ENTITY-FUSION.md`（v1，融合协议）
> - `FIVE-ENTITY-ARCHITECTURE-FUSION.md`（v1，整体融合）
> v1 保留为历史参考，实施以本文为准。
>
> 审计依据：2026-09-22 全域审计（2392 .rs / 分层违规 / 五实体重度碎片 / 晶体平行 / 文档矛盾）。
> 吸收源：Kingdee Lingee EAOS（2026-05-20）五实体模型。
> 设计原则：人类定规则，AI 做执行。

---

## 0. 一句话总纲

**CrystalState 是单一事实源；五实体是它的五种投影；外围注册表降级为投影的序列化层。**

```
┌─────────────────────────────────────────────────────────────┐
│  CrystalState（单一事实源）                                   │
│    identity / knowledge / experience / evolution（四层不变）  │
│    capabilities: HashMap<String, f64>（能力坐标）             │
│    projections: workspace / agents / skills / tasks / tools   │
│    execution_trace: Vec<ExecutionRecord>（神经系统）          │
└──────────────────────────┬──────────────────────────────────┘
                           │ 投影（内在属性，非独立模块）
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  外围层（序列化 + 执行）                                      │
│    L0 WorkSpaceManager  ←→ workspace 投影                    │
│    L1 AgentCardRegistry ←→ agents 投影                        │
│    L0 SkillLoader       ←→ skills 投影                        │
│    L1 Scheduler         ←→ tasks 投影                         │
│    L1 McpRegistry       ←→ tools 投影                         │
└─────────────────────────────────────────────────────────────┘
```

### v1 教训（审计结论）

| v1 假设 | 现实（file:line 证据） | v2 对策 |
|---|---|---|
| 每实体只有一个现有类型 | 每实体 3–7 个重复定义 | E1 先选型，每实体指定唯一正典 |
| 分层干净，只需加字段 | L0 被击穿（`nt_core_error/mod.rs:170,195` 等 33 处 From 反向依赖；`nt_core_ws.rs:111` 直调 L5）＋ L3 信任环（`nt_shield_enforcer.rs:4,6,372`）＋ L5→L6 124 处（`l1_facade.rs:131-173` 90 个再导出） | E0 先止血，再加实体 |
| ConsciousnessCore 就是枢纽 | 两套 tick（`consciousness_core/core.rs:179` vs `crystal_state.rs:128`）＋晶体零外围调用 | 枢纽 = CrystalState；ConsciousnessCore tick 向它读写 |
| D/I/W/P 编号可执行 | D1-D15 重号、I/W/P 四套并存 | v2 统一 **E0–E4** 编号，与旧路线解耦 |

---

## E0. 分层止血（先修地基，再加实体）

> 不做 E0，后续一切加字段都会加剧违规。E0 无新功能，只有依赖倒置。

| # | 违规 | 修复 | 文件 |
|---|---|---|---|
| E0.0 | 基线红色：`cargo xl` 现已失败（`deny(warnings)` 下 `nt_tui_app.rs:297` 未用变量 `now`），E0 验收"通过"无从谈起 | 修该 1 行（前缀下划线或删除），恢复基线绿；此后 E0 每步以绿基线回归 | `neotrix-core/src/l1_action/nt_tui_app.rs:297`（`lib.rs:22` lint 门） |
| E0.1 | L0 `nt_core_error` 反向 `From` L2/L3/L5/L6（33 处） | `From` 实现上移至各上层（`impl From<XxxError> for NeoTrixError` 写在使用方层），L0 只保留错误枚举本身；**遵循既有先例**：`mod.rs:114` 已有 `L1 错误 moved to l1_action::error_conversions`，E0.1 即把该模式推广到 L2/L3/L5/L6/neotrix 各建 `error_conversions` | `l0_substrate/nt_core_error/mod.rs:117,123,136,170,195,222,254` |
| E0.2 | L0 `nt_core_ws` 经 L5 别名调用同层 `nt_core_state`（4 处） | **修正 v2 初稿"trait 注入"方案**：抽查证实 `nt_core_state` 本尊在 L0（`l0_substrate/nt_core_state.rs:67-119` save/save_with/load/load_with/delete），L5 `mod.rs:103` 只是 `pub use` 转出口 —— 同层调用走错门牌而已；修复＝把 4 处 `crate::l5_cognition::nt_core_state::` 改为 `crate::l0_substrate::nt_core_state::`（纯路径重写，E0 最小项）；`WorkspaceStore` trait 暂不引入（YAGNI，待真出现第二实现再抽） | `l0_substrate/nt_core_ws.rs:111,116,124,131` |
| E0.3 | L5 `l1_facade` 90 个 L6 再导出（循环枢纽） | **修正 v2 初稿"删除"方案**：抽查证实该 facade 是有意从 L0 迁移至 L5 的（`l1_facade.rs:125-128` 注释），且有 20+ 跨层消费者（L1/L3/L4/L5/L6，如 `nt_emotion_facade.rs`、`l6_meta/mod.rs`、`l3_embodiment/mod.rs`），直接删除会断裂 20 文件；改为**facade 收敛**：(a) 按域拆分为 `l1_facade_observer` / `l1_facade_meta` 子模块，(b) 每个再导出加 `// ALLOW: <消费者> 需字段直访（trait-object 不可行）` 注释建 allowlist，(c) 新增消费必须走 trait，存量逐步迁移 | `l5_cognition/l1_facade.rs:131-173` |
| E0.4 | L3 安全层依赖 L5/L6（信任环） | `ApprovalEngine/Laws/ToolRegistry` 下沉为 L0 trait，L3 经回调调用，上层注入实现 | `l3_embodiment/nt_shield_enforcer.rs:4,6,372`, `nt_sandboxed_shell.rs:1,85` |
| E0.5 | L1/L2 经 L5 路径引用共享类型（8 文件）＋ L1 直引 L5 CoT/Policy | **修正 v2 初稿"下沉类型"方案**：抽查证实 `CapabilityVector` 只有一份定义（`crates/neotrix-types/src/core/nt_core_cap.rs:121`，层中立 crate），L5 的 `capability/types.rs:215` 只是 `pub use` 转出口 —— 无需搬类型，只需把 8 处 import 从 `crate::l5_cognition::nt_core::capability::types::CapabilityVector` 改为 `neotrix_types::core::nt_core_cap::CapabilityVector`（纯路径重写，零语义变更）；剩余 `CoT/E8Policy`（`nt_core_task_dispatcher.rs:12,15,17`）经 `nt_action_facade` trait 抽象 | 8 文件见附录 A-A14 |

验收：`rg 'crate::l[56]_' neotrix-core/src/l0_substrate --type rust` 零命中（注释除外）；`l1_facade` 拆分子模块完成且每个再导出有 ALLOW 注释；`cargo xl` 通过。

---

## E1. 正典选型（一实体一正典，其余 deprecated）

> 选型标准：字段最全 ∪ 被引用最多 ∪ 方向正确（上层不污染下层）。

### E1.1 Workspace → `l0_substrate::nt_core_ws::WorkSpace`（正典）

- 现状 11 字段（id/name/created_at/last_active/project_root/description/tags/memory_count/goal_count/skill_count）＋ `WorkSpaceManager`（create/list/switch/delete/active/get/rename/save/load，KB 持久化已通）。
- 新增字段（全部 `#[serde(default)]`，旧快照兼容）：
  `kind: WorkspaceKind`（Local/Remote/Sandbox）、`agent_ids: Vec<String>`、`skill_ids: Vec<String>`、`mcp_servers: Vec<McpServerBinding>`、`shared_memory_keys: Vec<String>`、`config: WorkspaceConfig`。
- 废弃（标记 `#[deprecated]`，不删除）：
  `file_centric_state.rs:13 WorkspaceSnapshot`（任务快照语义并入 TaskProjection.history）、`nt_act_workspace_isolator.rs:78 IsolatedWorkspace`（降级为 WorkspaceKind::Sandbox 的执行器）、`gwt_router/broadcast.rs:68 GlobalWorkspace`（与 `nt_core_gwt/workspace.rs:37` 同名冲突，改名 `GwtBroadcastBus`）、`ctm.rs:52 Workspace`（改名 `CtmShortTerm`）、`ffi/types.rs:173 WorkspaceState`（改名 `FfiWorkspaceView`）。
- 明确保留：`nt_core_gwt/workspace.rs:37 GlobalWorkspace`（16 文件引用，L0/L2/L5/L6）—— 它是**注意力广播** concern，与实体 Workspace（上下文容器）正交，不合并、不改名；两者边界：GWT 管"广播什么"，Workspace 管"谁在场"。

### E1.2 Agent → `l1_action::nt_infra_agent_card::AgentCard`（正典）

- 现状 12 字段＋ `AgentCardRegistry`（register/get/find_by_capability/find_by_tag/alive_agents/cleanup）＋全局 `GLOBAL_CARDS` —— 唯一带存活心跳与能力索引的注册表。
- 新增字段（`#[serde(default)]`）：`role: AgentRole`（level/org_unit/reports_to）、`knowledge_domains: Vec<String>`、`task_templates: Vec<String>`、`deliverable_types: Vec<String>`、`installed_skills: Vec<String>`、`execution: ExecutionPolicy`、`workspace_id: Option<String>`、`mcp_permissions: Vec<McpPermission>`、`model_preference: ModelPreference`、`status: AgentStatus`。
- 降级为协议适配器（保留，做 `From` 转换，不做能力注册）：
  `capability/a2a.rs:120 AgentCard`（A2A 协议格式：agent_name/endpoints/input_modes —— 只做协议序列化）＋ `A2ARegistry:188`（只做 A2A 会话）、`nt_io_protocol_bridge.rs:83 AgentCard`（只做协议桥接）。
- 降级为画廊数据（保留展示，不做调度）：
  `nt_agent_identity.rs:15 AgentPersona`（17 字段人设 —— 只做展示/画廊）、`nt_agent_gallery` 不变。
- 合并：`agent.rs:208 AgentRole` 与 `nt_mind_dual_track.rs:43` 重复 AgentRole —— 以 E1.2 新 `AgentRole` 为准，旧两处标记 deprecated。
- 预置模板库（新增 `l5_cognition/nt_agent/presets.rs`）：code-reviewer / doc-writer / data-analyst / security-auditor / test-engineer（id/name/description/role/knowledge/skills 六元组）。
- 预置模板**馈入画廊**（不另起平行清单）：抽查证实 `AgentGallery.register_builtin_presets`（`nt_agent_gallery.rs:67-72`）是从 persona specialty 派生的，无独立 curated 目录；`presets.rs` 作为 curated 源，同时馈入 `AgentCardRegistry`（注册）与 `AgentGallery`（展示），画廊 `install()` 保持唯一安装入口。
- 运行实例注册分工：`l0 nt_core_platform::AgentRegistry`（`Box<dyn Agent>` 运行实例，按层/域索引）保留 —— 它管"活的实例"，`AgentCardRegistry` 管"能力描述"；E2 接线时加桥：实例 register → 自动发布/更新对应 AgentCard（心跳对齐）。`nt_act_typed_agent.rs:592 TypedAgentRegistry`（name→handle 执行分发）保留，只做执行，不做描述。

### E1.3 Skill → `skill_loader::ResolvedSkill`（正典）＋ `SkillCandidate`（进化态）

- `ResolvedSkill` 已有 name/description/path/category/**tags/triggers/dependencies**/exists —— 天然具备触发与依赖语义，选为**运行时正典**。
- `coordination/skill_evolution.rs:44 SkillCandidate`（id/name/description/version/performance_history/evaluation_count/maturity）选为**进化态正典**；新增 `author/tags/triggers/required_permissions/dependencies/source/security_audit`（`#[serde(default)]`）。
- 两态关系：`ResolvedSkill`（发现/加载）→ `SkillCandidate`（评分/进化），以 `name==id` 关联；`SkillRegistry` 接口以 `ResolvedSkill` 为出参、`SkillCandidate` 为进化入参。
- 废弃/降级：`agent.rs:292 SkillsEngine`（1 字段空壳 —— 保留为 `SkillLoader` 门面，方法透传）、`nt_mind_skill_engine.rs:20` 同名 `SkillEntry`（改名 `SkillDocEntry`，只做 Markdown 解析）、`nt_memory_knowledge_assets.rs:13` 第三个 `SkillEntry`（改名 `KbSkillAsset`）、`nt_core_model_skills.rs:22 ModelSkillRegistry`（实为模型能力表，改名 `ModelCapabilityTable`）。

### E1.4 Task → `l1_action::nt_act::nt_act_scheduler::ScheduledTask`（正典）

- 现状 14 字段＋ `Scheduler`（register/create_cron/tick/history）＋ `TaskExecutionResult` —— 唯一带 cron/重试/超时的调度器。
- 新增字段（`#[serde(default)]`）：`executor_agent_id: Option<String>`、`skill_ids: Vec<String>`、`workspace_id: Option<String>`、`deliverables: Vec<DeliverableSpec>`、`depends_on: Vec<String>`、`created_by: String`、`history: Vec<TaskExecution>`。
- 新增方法：`list_by_workspace` / `list_by_agent` / `list_by_status` / `execution_history` / `pause` / `resume`。
- **同步/异步墙（第四轮审计发现）**：`ConsciousnessCoreHandle::tick` 为同步（`core.rs:179,238`），而本 `Scheduler` 全方法异步（tokio RwLock，`:148,155,260,266,272,283`）—— tick 内不可 `.await`，`tick` 改 async 会炸 MCP/CLI 共用方；方案：E1.4 在 `Scheduler` 内加同步镜像 `snapshot: std::sync::RwLock<Vec<TaskSummary>>`（每次 async 变更方法内同步更新），tick 只读镜像；新增的 `list_by_workspace` 等一律提供 sync 版（读镜像）＋ async 版（读源）双接口。
- 降级：`nt_core_task_dispatcher.rs:123 SubTask`（LLM 派发中间态 —— 只做分解输出，不做调度）、`recursive_controller.rs:14 TaskNode`（ROMA 递归控制节点 —— 只做递归展开）、`decompose.rs:15` 空 `TaskDecomposer`（改名 `AntidistilGuard`，它与拆解无关）。
- 第二调度器分工：`l6 nt_core_scheduler/engine.rs:48 SchedulerEngine`（ScheduledJob：handler＋context_gate＋heartbeat＋claim_pool，内部作业池）保留 —— 它管"内部作业"，`Scheduler` 管"用户任务"；两者字段已验证正交（Job 无 instruction/deliverables，Task 无 handler/context_gate），E2 之后再议 Task→Job 桥，不在本蓝图合并。

### E1.5 MCP → `agent.rs::tool::mcp::McpRegistry`（正典）

- 现状 `McpToolDef`（name/description/input_schema/transport/server_name）＋ `McpRegistry::register_stdio` ＋ `ToolOrchestrator::call`。
- 必修死链：`all_native_tools()` 当前返回空 Vec —— 改为从全局 `McpRegistry` 重建 NativeTool 列表（真实路径，非空壳）。
- 新增字段（`#[serde(default)]`）：`McpToolDef.required_permission` / `risk_level`（Low/Medium/High/Critical）/ `usage_count` / `avg_latency_ms`；新增 `register_sse`、`tools_for_agent(agent_id)`、`tools_by_risk`、`usage_stats`。
- 降级：`nt_io_mcp_bridge.rs:43 McpBridge`（只做 IO 桥接）、`mcp_server.rs:20` 与 `tool_endpoint.rs:46` 同名 `McpToolRegistry`（前者改名 `McpHttpRegistry`，后者改名 `McpEndpointRegistry`，两者只做传输层）。

### E1 选型总表

| 实体 | 正典 | 废弃/降级数 | 关键动作 |
|---|---|---|---|
| Workspace | `nt_core_ws::WorkSpace` | 5 废弃改名 | +6 字段 |
| Agent | `nt_infra_agent_card::AgentCard` | 2 协议适配＋1 画廊＋1 合并 | +10 字段（六维） |
| Skill | `ResolvedSkill`（运行）＋`SkillCandidate`（进化） | 1 门面＋2 改名＋1 改名 | 双态关联 |
| Task | `ScheduledTask` | 2 中间态＋1 改名 | +7 字段＋6 方法 |
| MCP | `tool::mcp::McpRegistry` | 1 修死链＋2 改名＋1 桥接 | +4 字段＋4 方法 |

---

## E2. 单一事实源（CrystalState ＋五投影）

> 枢纽只认一个：**CrystalState**。ConsciousnessCore 的 tick 向它读写，不另起 Hub。

### E2.1 CrystalState 新增投影字段

```rust
pub struct CrystalState {
    // ── 现有不变 ──
    pub identity: CrystalIdentity,
    pub tick: u64,
    pub knowledge: CrystalKnowledge,
    pub experience: CrystalExperience,
    pub evolution: CrystalEvolution,
    pub attention_focus: Option<String>,
    pub current_goal: Option<String>,
    pub capabilities: HashMap<String, f64>,
    pub execution_trace: Vec<ExecutionRecord>,
    // ── 新增：五投影（全部 #[serde(default)]）──
    #[serde(default)] pub workspace: Option<WorkspaceProjection>,
    #[serde(default)] pub agents: Vec<AgentProjection>,   // 已有，补字段
    #[serde(default)] pub skills: Vec<SkillProjection>,   // 新增
    #[serde(default)] pub tasks: Vec<TaskProjection>,     // 新增
    #[serde(default)] pub tools: Vec<ToolProjection>,     // 新增
}
```

- `AgentProjection`（现有 `crystal_state.rs:63`）补字段：`workspace_id`、`status`、`installed_skills`、`mcp_permissions`（与 E1.2 对齐）。
- `SkillProjection`：id/name/triggers/source_knowledge_ids/source_experience_ids/effectiveness/use_count/maturity。
- `TaskProjection`：id/name/instruction/executor_agent_id/skill_ids/status/history（复用 `ExecutionRecord`）。
- `ToolProjection`：name/server_name/capability_tag/risk_level/usage_count/avg_latency_ms。
- `WorkspaceProjection`：id/name/root/agent_ids/skill_ids/tool_names/shared_memory_keys。
- 投影住址（nt_ 合规，就近原则）：五投影全部与 `CrystalState` 同文件（`neotrix/nt_crystal_core/crystal_state.rs`）—— 投影是状态的字段，与状态同生共死，不另起模块；`WorkspaceProjection` 的 L0 运行时形态仍是 `WorkSpace`（E1.1），两者以 id 关联。
- 构造更新清单：`crystal_state.rs` 三处 `Self{…}` 字面量（`new:108` / `from_core:195` / `from_consciousness:204`）同步加字段（`#[serde(default)]` 保旧快照可读）；`summary()` 加 `active_tasks` 计数。

### E2.2 tick 接线（不改签名）

- `ConsciousnessCoreHandle::tick(cycles)` 签名不变；tick 内：读 active workspace → 加载五实体 → 写入 `CoreSnapshot.workspace_context`（E2.3）→ 运行现有 `run_growth_cycle` → 写回 KB。
- `CrystalState::advance_tick` 与 `ConsciousnessCoreHandle::tick` 建立调用链：后者每 cycle 调用前者推进晶体时钟（终结"两套 tick"）。
- 首条真实数据流（E2 最小闭环）：**修正 v2 初稿"Skill 执行结束"表述**：抽查证实仓库无中央 Skill 执行器（SkillLoader/SkillsEngine 只有发现/加载，无 execute；`agent.rs:489` 的 execute 是 ToolOrchestrator 调工具）—— Skill 以 SKILL.md 上下文形态被 Agent 消费，执行点在 `entry/mod.rs` 与 `entry/headless.rs` 两处 `SkillsEngine` 调用方；故首条闭环＝在这两处 load-use 点包裹 `CrystalState::record_execution`（已存在 `crystal_state.rs:146`，首次被外围调用）→ `execution_trace` 增长 → `SkillCandidate.performance_history` 反馈 → bandit 更新 maturity。附带：L6 `SkillEvolver` 现仅自文件引用（`nt_mind_skill_engine.rs:869` 的 `_SkillCandidate` 是同名无关体）—— Evolver 反馈线与 record_execution 同一批接通，否则 maturity 永不更新。

### E2.3 CoreSnapshot 上下文（snapshot 透传）

```rust
#[serde(default)] pub workspace_context: Option<WorkspaceContext>
// WorkspaceContext { workspace_id, active_agents, active_skills, active_tasks, available_tools }
```

### E2.4 KB 命名空间（每实体独立）

`workspaces` / `agent_cards` / `skill_candidates` / `scheduled_tasks` / `mcp_servers` —— 各实体独立持久化，互不依赖；CrystalState 快照存 `crystal_state`。
- 落盘位置：`crystal_root()`（`~/.neotrix/crystal_core/`）下 `crystal_state.json`，与 `crystal.json` 同目录；写模式沿用既有 R-P0-2（tmp＋rename 原子写＋`.bak` 轮转，见 `nt_crystal_core/mod.rs:118-145`），不自创持久化协议。

---

## E3. 路由＋事件（沿用 v1 FUSION 设计，以 E1 选型为准）

### E3.1 三层路由（取代硬编码首选）

```
Layer 3 Skill 触发（最高）：SkillLoader::search(SkillFilter{ triggers }) → ResolvedSkill
    语义已验证：子串匹配（`skill_loader.rs:306-310`：`skill.triggers.any(|st| st.contains(t))`），
    且 `./skills/index.json` 真实存在（58 skills / 147 triggers）—— Layer 3 首日即可跑通，无需先建索引
    冲突率已量化：147 triggers 中 42 个与静态表关键词双向子串命中（~29%，如 安全/审计/架构/测试/漏洞/文档）——
    Layer-3-优先规则即为此而设；E3 验收加一项：42 个重叠词逐个复核归属（Skill 专属 vs 静态兜底），清单见附录 A-A20
Layer 2 Agent 能力：AgentCardRegistry::find_by_capability → AgentCard
Layer 1 静态兜底（不变）：CAPABILITY_ROUTES（`dispatch.rs:12-570` 约 560 行静态表，非此前误称的 2191 行 —— 2191 是 dispatch.rs 全文件行数；表内容原样保留，只降优先级）
Layer 0 LLM 直推（最低）
```

- `route_entity_aware(input, workspace_ctx)` 返回 `RouteDecision::{Skill, Agent, Static, DirectLlm}`（定义见 v1 FUSION §2.2，原样采用）。
- `AutoOrchestrator::IntentClassifier` 新增 Skill 感知：分类前先查 `SkillFilter{ triggers }`，命中则直返 Skill 路由（终结"意图分类不感知技能"）。

### E3.2 CoreEvent 实体事件（+11 变体，全部 `#[serde(tag)]` 风格延续）

WorkspaceSwitched / WorkspaceAgentAdded / WorkspaceAgentRemoved / AgentStatusChanged / AgentTaskDelegated / SkillInstalled / SkillExecuted / SkillMaturityChanged / TaskAssigned / TaskCompletedV2 / McpToolCalled / McpServerConnected（载荷见 v1 FUSION §2.3，原样采用）。
- 爆破面已验证：抽查全部 `CoreEvent::` match 站点均有 `_ =>` 通配臂（`nt_core_event_bus.rs:146,311,340,410`、`handlers_consciousness.rs:884,1069,1281` 等），新增变体非破坏；唯一 `_ => panic!` 在 `nt_core_event_bus.rs:540` 测试内（`#[test]`），生产无碍。

### E3.3 上下文总线

`Workspace.shared_memory_keys` 为五实体共享数据总线；`ExecutionContext{ workspace_id, agent_id, task_id, skill_id, shared_memory, available_tools }` 为单次执行上下文（定义见 v1 FUSION §2.4）。

---

## E4. 文档同步＋编号统一

- 本文为唯一正典；三份 v1 头部已标注废弃（本轮由作者手工标，见"地位"节）。
- 编号统一为 **E0–E4**；旧 D/I/W/P 编号冻结，不再新增。
- `docs/architecture/ARCHITECTURE.md` 补一节"五实体投影"（CrystalState 五投影＋E1 正典表），与其 C4 分层图对齐。
- 迭代记录走 KB experience-tree（沿用 MAP-ROADMAP-V2 §5 约定）。

---

## 实施顺序与验收

| 阶段 | 内容 | 验收命令/标准 |
|---|---|---|
| E0 | 分层止血 E0.1–E0.5 | `rg 'crate::l[56]_' src/l0_substrate` 零命中；`cargo xl` 通过 |
| E1 | 正典选型＋字段＋改名＋死链修复 | `rg 'struct AgentCard\|struct SkillEntry\|struct McpToolRegistry\|struct WorkspaceSnapshot'` 仅正典＋适配器存活；`all_native_tools` 非空 |
| E2 | CrystalState 五投影＋tick 接线＋首条数据流 | Skill 执行一次 → `execution_trace` +1 → `performance_history` +1（单测断言） |
| E3 | 三层路由＋11 事件＋上下文总线 | 输入"合并 Excel"命中 Layer 3 Skill 路由（单测断言 RouteDecision::Skill） |
| E4 | ARCHITECTURE.md 同步＋v1 标注废弃 | 文档检查清单全勾 |

### 硬约束（沿用 R-P 系）

- `#![forbid(unsafe_code)]`；生产代码禁 `unwrap/expect/panic!`（测试除外）。
- 新增字段一律 `#[serde(default)]`；不改 `tick()` 签名；不动 `CAPABILITY_ROUTES` 内容（只降优先级）。
- `nt_` 前缀；编辑后重读验证（R-P16）；结构性改动后 `cargo clean && cargo build` 两遍。

---

## 附：与现有路线图的对位

| 本蓝图 | MAP-ROADMAP-V2 / MASTER-BLUEPRINT 对位 |
|---|---|
| E0 分层止血 | MASTER D-07/D-08（P0 编译＋越层＋SDB）—— 同一件事，E0 即其五实体前置条件 |
| E1 选型 | I0.1（MCP 协议）＋ I0.2（技能编排）的前置 —— 先定正典再对标 56 项目 |
| E2 单一事实源 | I1.1（统一记忆层：Workspace.shared_memory 即其 3-Tier 落点） |
| E3 路由＋事件 | I1.3（OTel：实体事件即 trace 事件源）＋ I4.2（多 Agent 协作：AgentTaskDelegated 即委托事件） |
| E4 文档同步 | R-P111-R-P115 架构管理规则 |

> 本文未覆盖 L2/L3/L4 独有能力（E8/VSA/NT-FEEL/Shield）—— 它们是能力纵深，非实体骨骼，按原路线图推进，不在本蓝图范围内。

---

## 补篇：联网搜索补齐（2026-09-22）

### S1. 灵基新实证（发布后 3 个月）

| 事实 | 来源 | 对 v2 的意义 |
|---|---|---|
| 六层架构官方口径：模型接入 / Agent Runtime / Skill / Tool / Data / Governance | 2026-07-23 北京品鉴沙龙 | 与 v2 五实体＋治理约束同构；Governance 即 E0.4/E1.2 权限分级的外部印证 |
| 三种工作模式：对话 / 工作 / 开发；Build 自然语言生成企业应用，Skill 原子化沉淀为数字资产 | 同上＋金蝶官网 | Skill 双态（ResolvedSkill 运行＋SkillCandidate 进化）即"原子化数字资产"的技术落点 |
| AI 原生产品收入 2.96 亿（+189%），灵基签约 26 家、上线 40+ 智能体；费用审核释放 20–30% 人力；采购付款小时级→分钟级；关账 +30%；益客 Build 研发效率 +65% | 2026-08-13 中期业绩 | E2 首条数据流＋E3 路由的商业价值锚点：TaskExecution.tokens_used/cost_usd（v1 FUSION 已有）即"信用消耗"计费底座 |
| 商业模式"订阅费＋信用消耗"；安全通过信通院 Q/KXY CS101-2026《企业级类 Claw 智能体安全能力要求》 | 大摩研报 / 深圳发改委转载 | 成本追踪（cost_usd）＋风险分级（risk_level）＋安全审计（security_audit）三字段全部有商业出处 |
| AI 套件开放 API / MCP / CLI 三通道；DataCloud 元数据语义化＋商业本体 | 2026-06-05 AI 套件 | MCP 三传输（stdio/SSE/HTTP）＋Workspace.shared_memory_keys（本体口径）即其开源侧对应物 |

### S2. 业界模式印证（Microsoft + A2A/MCP 官方）

| 模式 | 来源 | v2 采纳 |
|---|---|---|
| 内部流走平台原生编排，工具数据走 MCP，跨平台 agent 走 A2A；最小权限＋可审计＋治理 | Microsoft Learn《Multi-agent patterns》 | E1.2 `mcp_permissions`＋E1.5 `risk_level`＋E3 事件即该原则的代码化；v2 不自创协议 |
| **MCP server 作为 A2A Agent Card 中央可查询仓库**：Card 存 MCP resource，discovery 经 MCP，运行时直连 A2A，MCP 不参与运行时交互 | a2a-samples 官方 `a2a_mcp` | E1.5 追加：`McpRegistry` 支持以 resource 形式托管 Agent Card（`register_agent_card`），discovery 走 MCP、执行走原生调用 —— 与业界标准对齐 |
| Agent Card 四件套：capabilities / skills / endpoints / metadata；task 生命周期 submitted/working/completed/failed | A2A 协议＋MCP×A2A 论文（2025-06） | E1.2 `AgentCard` 字段已覆盖；E1.4 `TaskStatus` 对齐 A2A task 状态机 |
| 本地 registry `~/.a2a-agents/`（agents.json＋tasks.json）：create/register/discover/send_task/get_task_status | a2a-protocol-mcp-server | E2.4 KB 命名空间即其生产级对应物（`agent_cards`＋`scheduled_tasks`）；CLI 调试可复用该五命令形态 |

### S3. v2 增补项（搜索后新增，仅 2 条）

- **S3.1** E1.5 追加 `McpRegistry::register_agent_card`（Agent Card 即 MCP resource，discovery 经 MCP）—— 业界标准模式，不自创。
- **S3.2** E2 验收追加成本断言：Task 执行一次 → `tokens_used > 0` 且 `cost_usd >= 0`（"信用消耗"计费底座，灵基商业模式印证）。

### S4. 灵基六维官方定义 × E1.2 一一印证（第二波，2026-09-22）

> 来源：金蝶灵基全球官网（2026-08）："Lingee defines enterprise AI agents across six dimensions — role definition, knowledge, task, deliverable management, skill orchestration, and execution plans."

| 灵基六维（官方） | v2 E1.2 字段 | 状态 |
|---|---|---|
| role definition（岗位定义） | `role: AgentRole` | ✅ 已有 |
| knowledge（知识） | `knowledge_domains` | ✅ 已有 |
| task（任务） | `task_templates` | ✅ 已有 |
| deliverable management（产物管理） | `deliverable_types` | ✅ 已有 |
| skill orchestration（技能调度） | `installed_skills` | ✅ 已有 |
| execution plans（执行计划） | `execution: ExecutionPolicy` | ✅ 已有 |

v2 E1.2 与官方定义 6/6 对齐 —— 六维不是自创，是跨厂商收敛。另：官网 "agents invoke **approved** ERP and business skills" 即 `mcp_permissions`＋`risk_level` 的外部印证；灵基四运营指标（处理时长/异常发现/任务闭环/复用次数）即 E2 验收的业务语言（`TaskExecution.duration_ms` / 事件 / `deliverables` / `use_count`）。

### S5. 合规映射（第二波，2026-09-22）

> v2 的权限/风控字段不是"设计偏好"，是合规刚需。按下表，开工即合规。

| 合规源 | 要求 | v2 落点 |
|---|---|---|
| 三部委《智能体规范应用与创新发展实施意见》（2026-05-08） | 三类决策边界（仅本人/需授权/自主）＋知情权＋不得超授权 | `PermissionLevel`（Read/Write/Admin/Sovereign）＋`mcp_permissions`＋高风险人工确认（E1.2/E1.5） |
| 国标《智能体应用安全基本要求》（20263116-Q-252，强制，TC260） | 身份标识/权限调用/工具调用/数据使用/高风险人工介入/日志留存监测/异常阻断关停 | Agent 身份＋`risk_level`＋`TaskExecution.history`＋CoreEvent＋circuit breaker（E1.2/E1.4/E1.5/E3） |
| 信通院《AI Agent安全实践指引》（2026-03） | 五类风险（权限/供应链/输入/隔离/审计）＋六要六不要＋三步走 | E0.4/E1.2/E1.3(`security_audit`)/E1.5/E3；ClawHub 实证 2857 skills 含 341 恶意（12%）—— `security_audit` 非可选 |
| OWASP Agentic AI Top 10（2026） | Least Agency＋Intent Capsule＋JIT token＋工具级最小特权＋语义防火墙（ASI01–04/09/10） | `risk_level`＋工具级权限＋高风险动作显式确认（E1.5/E3）；`AgentStatus` 支撑 Rogue Agent 熔断 |
| 信通院可信评估 2.0（八大维度） | 基础设施/数据资源/核心组件/Skills/编排/平台支撑/关键能力/应用/运营/价值 | E2 成本断言（价值评价）＋E3 事件（运营管理）＋双态 Skill（核心组件） |

---

## S6. 实测收割回填（2026-09-22，`repo-analyses/lingee-20260922/` 23 文件 / 484K 脱敏）

> 只收设计层（字段模型/计数/模式），不搬原文内容。蒸馏文档见 `docs/plans/2026-09-22-lingee-{architecture,agents-skills,desktop-iteration}.md`。

### S6.1 Agent 字段模型 → E1.2 增补（实测 89 条，17 键）

| 实测键 | v2 增补 | 说明 |
|---|---|---|
| `templateId` / `templateVersion` / `currentVersion` | `template_id` / `template_version`（`version` 已有） | 版本血缘；实测存在版本漂移（装 1.0.5 → 市 1.0.6），E2 加升级提示事件 |
| `availablePackageTypeCodes` / `trialEnabled` | `package_codes: Vec<String>` / `trial_enabled: bool` | entitlement 分级（Trial/Professional/Gift/Ultra）；付费墙本身不抄 |
| `installed` / `installable` | 注册表态（`workspace.agent_ids`＋安装记录），不进 Card 字段 | 安装态是关系，不是属性 |
| `connector` | `connector: Option<String>` | MCP 绑定引用（E1.5 对接） |
| `isAdmin` / `agentStatus` | `visibility` / `status`（E1.2 已有 status，加 visibility） | 可见性与发布态分离 |
| `icon` / `iconClass` / `tags` | `icon_class: Option<String>`（tags 已有） | 展示层提示 |

### S6.2 Skill 字段模型 → E1.3 增补（实测 266 条，9 键，12 分类）

| 实测键 | v2 增补 | 说明 |
|---|---|---|
| `dataScope`（0=纯对话 94 条 / 8=需数云·ERP 172 条） | `data_scope: u8` | 数据依赖分级，直连权限要求（dataScope=8 必须配 mcp_permissions） |
| `installStatus` / `manageSkillId` | 安装态（注册表）/ `external_id` | 同 S6.1：安装态是关系 |
| `developer`（null=官方 94 条 / 余 UGC 172 条） | `certified: bool`（官方收敛标记） | 反面教材：简历筛选 ×20+、销售订单查询 ×20+ 跨租户重复造轮子——市场必须有官方收敛版＋去重门禁 |
| `currentVersion` / `updateTime` | 版本元数据（E1.3 已有 version，加 `updated_at`） | 0.1.0 占 143（一次性共创产物）→ maturity Candidate 起步 |
| description 三段式 | Skill 入库门禁（E3 Layer 3 前置） | 触发条件＋明确排除＋输出契约（三档状态/五维度加权/JSON·HTML）；查询类强制"条件不全主动澄清"子句 |

### S6.3 模型档位 → ModelPreference 增补（实测 work/task 双域 11 档）

- 四元组 `modelLevel / autoRouting / consumptionCoefficient / orderNumber`：前端只暴露档位名，后端映射真实模型（换模型不改 UI）—— v2 `ModelPreference` 加 `consumption_coefficient: f64` / `order_number: i32` / `auto_routing: bool`。
- 实测系数锚点：fast 0.3 / expert 0.9（多数 Agent 默认锁档）/ ultra 1.2 / Flash 类 0.1–0.3 / 推理类 3.3 —— E2 成本断言的系数来源。
- 单 Agent 可锁档（`.../agent/model-level?assistantId=`）→ `AgentCard.model_preference` 即锁档语义，无需新机制。

### S6.4 用量 schema → TaskExecution 增补（`runtime-data` 实测键）

- `dataScope / period / taskCount / artifactCount / token / credits / dailyStats（14 天）` → `TaskExecution` 加 `artifact_count` / `credits`，聚合视图 `dailyStats` 进用量看板（E2 验收业务语言：处理时长/任务闭环/复用次数）。
- OTLP traces/logs/metrics＋首 token 延迟（TTFT）→ E3 事件即 trace 事件源（已在路线对位表，实测确认字段齐）。

### S6.5 网关/启动/桌面桥模式（设计层引用）

- 单基址＋三头路由（`X-Kwork-Api-Module: work|chat`＋业务类型头）→ NeoTrix provider 路由参考：module 头选网关，业务头选策略。
- 启动链 `tenant/list → auth/bootstrap`（locale/plan/水印一次下发）`→ auth/session` → NeoTrix 启动序列参考：一次下发代替多次往返。
- 桌面桥事件表（`getConfig / emit / notify`＋`token-expired` 回灌；401 先走 bridge 再回退登录）→ Tauri `BrowserHost` 桥接事件表（E3，桌面迭代清单 P0-3/P1-4 已立项，此处只记接口）。
- 不抄：WAF 绑定架构、entitlement 付费墙、sessionStorage 存 token（关窗即焚，改走安全存储）。

### S6.6 市场治理教训（反面模式，进蓝图门禁）

1. 约 30 个 workshop/共创/测试 Agent 混入市场（`0908/自进化/bwtest` 前缀，多 v1.0.0）→ 市场准入 maturity 门：Candidate 不得进默认列表。
2. Skill 版本 `0.1.0` 占 143 → `maturity` 初值 Candidate＋`TRUSTED_MIN_SCORE` 不变，未验证不得 Trusted。
3. 发布只收 zip（`upload_zip_only`）→ NeoTrix Skill 上传格式门：单文件 SKILL.md 或 zip 二选一，定死一种。

### S6.7 供应链纵深防御（外部，2026 实证压强）

> S6.6 三条门禁是起点；2026 上半年实证要求升级为纵深体系（否则等于把 ClawHavoc 重演一遍）。

- 实证压强：ClawHavoc 2026-01/02 向 ClawHub 投毒 **1,184** 恶意 Skill；Cisco 抽 31k skills **26%** 含可利用漏洞；Snyk 审 3,984 skills 确认 76 恶意载荷＋36% 提示注入；社区注册 3 个月破 98k 且无评审；Trail of Bits/CSA 结论：**任何单扫描器都不可做信任门**（"Don't outsource trust to a scanner"）。
- v2 落点（`security_audit` 从布尔升级为管线，E1.3/E1.5）：
  1. 预发布扫描（静态＋secret＋语义）→ 2. 来源签名＋版本锁定（防 rug-pull，禁止浮动 latest）→
  3. 运行时监控（工具调用围栏）→ 4. 最小权限调用（独立于开发者权限）→ 5. 人工复核（敏感上下文）。
- 可验证工件四元组（对照，不自创）：`caps`（声明能力有限词表）/ `signer`（信任根密钥）/ `version`（单调整数，旧版重放拒收）/ 运行时不可变（载入后 skill 内容＋manifest 冻结，agent 不得改）。
- 与 OWASP Agentic Skills Top 10 对齐（S5 已有 ASI04 条目，此处落实为上述 5 步）。

---

## S7. 剩余吸收清单（2026-09-22，23 raw 全过＋桌面代码核查）

> 结论：raw 已全覆盖（无未开封文件）；桌面 P0–P2 **0/10 落地**（1 项部分）—— 吸收缺口全在执行侧，不在情报侧。

### S7.1 本轮新榨（此前未入蓝图）

| raw 源 | 设计点 | v2 落点 |
|---|---|---|
| agent-install 33 键（`totalTokenUsage/totalCredits/taskCount/artifactCount/successRate`） | 用量实证字段（非估算） | `AgentCard` 加 `usage_proof`（累计 token/credits/任务/产物/成功率）；`successRate` 进 maturity 反馈 |
| 同上（`latestVersion` vs `version`） | 升级信号 | E2 加升级提示事件（S6.1 版本漂移的机制化） |
| 同上（`effectiveDataScope` / `requestedDataScope`） | 安装时数据权限握手（申请 vs 实授） | E1.2/E1.4：安装记录存双 scope；超授阻断（对标三部委"不得超授权"） |
| 同上（`initSkillName` / `personality` / `position` / `department` / `onDutyDays`） | Agent 出厂技能＋人设＋值班 | `installed_skills` 初值源；`AgentRole.org_unit` 语义确认；值班态进 `AgentStatus` |
| 同上（`isAssistantAdmin` / `primaryAdmin`） | 双管理角色 | `AgentCard` 加 `admins`（主＋协），治理审计用 |
| skill-install 19 键（`catalogId/developerType/openSourceAuthor/source/sourceUrl/license/status/timeCreated/timeUpdated`） | 安装溯源 | 安装记录形状（registry 级）；`license` 进合规 |
| tool 11 键（`toolCode/toolId/status/openFlag/source/switchable/selectable/requiresErpEnvironment`） | 三开关＋ERP 门 | E1.5 `McpServerBinding` 加 `switchable/selectable/open_flag`＋`requires_erp: bool` |
| `capabilities.json`（skills/plugins/mcp/commands/config 读写位＋`multiWorkspace/approval/audit/auth/readOnly`） | 能力位表 | 特性开关设计（E0.4/E3）：按域读写位＋全局只读闸 |
| `tenant_config.json` 12 键（扩展名白名单 ×2＋`ceo/cfoMenuVisible`＋`imageGen/volcVoice/webReport/financialReport`） | 租户能力旗 | `Workspace.config` 加 `file_gates`（preview/upload 白名单）＋`menus`＋`feature_flags` |
| `enums_locale.json`（10 地域→6 回复语言） | locale→回复语言路由 | `Workspace.locale`＋`llm_response_language`（E1.2 `ModelPreference` 旁路） |
| `tenants.json`（`error_code`＋`items{tenantCode/tenantName/defaultTenant}`） | 多租户切换＋错误包络 | 租户切换形状；`code/data/timestamp` 包络为 API 约定（artifacts/catalog/install 三处同形已验证） |
| `user_settings.json`（`memory_offline_extract_enabled:"true"` 字符串布尔！） | 记忆抽取开关＋类型坑 | Workspace 隐私旗；约定：线上只收真布尔，字符串布尔在网关层归一化 |
| `server_status.json`（`status/version/readOnly/devMode`） | 健康形状 | 健康端点四键（E3 可观测） |
| `auth_bootstrap.json`（`workModelLevel`＋`scheduledModelLevel` 双默认档！） | 工作档 vs 定时档分离 | `ModelPreference` 拆 `work_tier` / `scheduled_tier`（S6.3 补丁：此前只建单 tier） |
| `auth_session.json`（role 复合串 `member--role_*--audit_admin--M001`） | 角色组合串 | `AgentRole` 加 `role_chain: Vec<String>`（`--` 分隔解析） |
| search 返回全卡（17 键同 listing） | 搜索无精简 DTO | E3 搜索约定：搜索返回完整 Card（前端零二次请求） |

### S7.2 桌面 P0–P2 落地核查（代码实测，0/10）

| # | 项 | 状态 | 证据 |
|---|---|---|---|
| 1 | 模型档位层 | ❌ | 全仓无 `consumptionCoefficient/autoRouting/orderNumber` |
| 2 | 三段式门禁 | ❌ | triggers 存在但无"排除＋契约"校验 |
| 3 | 401 回灌 | ✅ 已落地 2026-09-22 | `browser_host.rs::AuthBridge`＋`auth_bridge_get_config` 命令＋前端 `subscribeAuthBridge`；tsc exit 0 |
| 4 | Bridge 事件表 | ❌ | 同上 |
| 5 | 用量看板 | ❌ | 无 dailyStats/artifactCount |
| 6 | 设计 token | ❌ | 无 `--lg-*`/gold token 体系 |
| 7 | 空错加载态 | ⚠️ 部分 | 通用空态有，对标项（`skillCenter.empty.*`/`loadFailed*`）无 |
| 8 | 市场字段 | ❌ | 无 templateId/entitlement/upload-zip |
| 9 | MFE 预研 | ❌ | 命中 `federation.rs` 均为 nt_mind 内部联邦，非前端 MFE |
| 10 | 翻译防护 | ❌ | 无 React #11538 防护 |

→ 十项其余九项仍为 open 工作（P0-3 已于 2026-09-22 销项）；S7.1 新增字段与十项无冲突（模型档位项 #1 即 S6.3＋S7.1 双 tier 的执行位）。

---

## 附录 A：点审计证据链（2026-09-22 第二轮）

> 上一轮为子代理广度扫描；本轮逐条抽查原文，修正 1 处 v2 初稿误判（E0.3）。

| # | 核查项 | 结论 | 证据 |
|---|---|---|---|
| A1 | E0.1 From 反向依赖 33 处 | ✅ 属实 | `nt_core_error/mod.rs:117,123,136,170,176,182,188,195,214,222,229,235,241,253` L2/L3/L5/L6/neotrix `From`；且 `:114` 已有 L1 先例 `moved to l1_action::error_conversions` —— E0.1 即推广该模式 |
| A2 | E0.3"删除 90 个再导出" | ❌ 初稿误判，已修正为 facade 收敛 | `l1_facade.rs:125-128` 注释证明有意从 L0 迁至 L5（字段直访使 trait-object 不可行）；20+ 跨层消费者（`nt_emotion_facade.rs`、`l6_meta/mod.rs`、`l3_embodiment/mod.rs` 等），删则断裂 |
| A3 | L3 信任环 | ✅ 属实 | `nt_shield_enforcer.rs:4,6` 直引 `l6 nt_approval/nt_laws`；`:372` 返回 `&'static l5 nt_core_gate::ToolRegistry` |
| A4 | `all_native_tools` 死链 | ✅ 属实 | `agent.rs:494` 定义返回空 Vec；全仓唯一引用是其自测 `agent.rs:737-739`，生产零调用 |
| A5 | KB 命名空间碰撞 | ✅ 无碰撞 | `save("…")` 全仓键：`workspaces`（已用，正典复用）外，`agent_cards` / `skill_candidates` / `scheduled_tasks` / `mcp_servers` / `crystal_state` 均无现存键；注意 `load("goals")` 被两处复用（`always_on.rs` 与 `goal_loop/core.rs`）—— E2 新增键必须全局唯一，沿用 `<entity>s` 复数约定 |
| A6 | 真实 `unsafe` | ✅ 零（目标文件） | 2 处命中均为非生产：`nt_act_scheduler.rs:23` 是文档注释 `No unsafe code (R-P1)`；`nt_shield_enforcer.rs:677` 是测试字符串内的检测样本（L002 法规测试固件） |
| A7 | `unwrap/expect/panic!` | ✅ 生产零（目标文件） | `nt_core_ws.rs:9`＋`nt_act_scheduler.rs:9` 全部位于 `#[cfg(test)]`；`nt_infra_agent_card.rs` / `crystal_state.rs` / `skill_evolution.rs` 零命中 —— E1 加字段无需先还负债 |
| A8 | L6 第二调度器 | ✅ 与 L1 正典正交，保留不合并 | `nt_core_scheduler/engine.rs:48 SchedulerEngine`（ScheduledJob：handler/context_gate/heartbeat/claim_pool）vs L1 `ScheduledTask`（instruction/deliverables）—— 前者内部作业池，后者用户任务；E1.4 已立分工 |
| A9 | L0 AgentRegistry / TypedAgentRegistry / Gallery 与正典关系 | ✅ 分工已立 | L0 `AgentRegistry`（`Box<dyn Agent>` 活实例）vs L1 `AgentCardRegistry`（能力描述）＋桥（实例注册→发布 Card）；`TypedAgentRegistry` 只做执行分发；`presets.rs` 馈入画廊不另起清单（画廊现 presets 均派生自 persona，无 curated 源） |
| A10 | GWT GlobalWorkspace 去留 | ✅ 保留，边界已划 | 16 文件引用（L0/L2/L5/L6）；GWT 管"广播什么"，实体 Workspace 管"谁在场" —— E1.1 已明示 |
| A11 | E2 落点可行性 | ✅ 全部可落 | CrystalState 为普通 Serialize/Deserialize（`crystal_state.rs:27`），新字段 `#[serde(default)]` 兼容旧快照；3 处构造字面量（`new:108`/`from_core:195`/`from_consciousness:204`）待同步；投影与状态同文件；`./skills/index.json` 真实存在 |
| A12 | E3 落点可行性 | ✅ 全部可落 | 触发语义＝子串匹配（`skill_loader.rs:306-310`）；CAPABILITY_ROUTES 纠正为约 560 行（`dispatch.rs:12-570`）；E4 插入位确认为 ARCHITECTURE.md §2 后§3 前 |
| A13 | 同步/异步墙 | ✅ 真墙，已有桥方案 | tick 同步（`core.rs:179,238`）vs L1 Scheduler 全异步（`:148,155,260,266,272,283`）；其余注册表（AgentCard/SkillLoader/McpRegistry/L6 SchedulerEngine）皆同步 —— E1.4 加同步镜像＋双接口，tick 只读镜像 |
| A14 | CapabilityVector"下沉" | ❌ 初稿误判，已修正为 import 重写 | 定义唯一（`neotrix-types/.../nt_core_cap.rs:121`），L5 `types.rs:215` 仅转出口；8 处 L1/L2 经 L5 路径引用（`sources.rs:4`、`specialized.rs`、`general.rs`、`vectors_group_b/mod.rs`、`activation.rs`、`vectors_group_a.rs`、`nt_io_proxy_server.rs`、`crawl/unified.rs`）改为 `neotrix_types::…` 路径即可 |
| A15 | CoreEvent＋11 变体爆破面 | ✅ 非破坏 | 全 match 站点有 `_ =>`；唯一 panic 在测试内（`nt_core_event_bus.rs:540`） |
| A16 | CrystalState 落盘位 | ✅ 已定 | `crystal_state.json` 同 `crystal.json` 目录，沿用 tmp＋rename＋.bak（R-P0-2）；`ApprovalEngine`（`l6_meta/nt_approval.rs`，448 行 struct＋enums）trait 抽取排 E0 末位，先加判决快照测试（风险表已记） |
| A17 | 基线颜色 | 🔴 红（实测 `cargo xl` 失败） | `nt_tui_app.rs:297` 未用 `now` × `lib.rs:22 deny(warnings)` —— E0 新增 E0.0 先恢复绿基线，否则一切验收无意义 |
| A18 | E0.2"trait 注入" | ❌ 初稿过设计，已降级为 4 行路径改写 | `nt_core_state` 本尊在 L0（`:67-119`），L5 仅转出口（`mod.rs:103`）—— 同层走错门；`WorkspaceStore` trait 按 YAGNI 暂不引入 |
| A19 | E2 首条数据流挂载点 | ✅ 已定位（无中央执行器） | SkillLoader/SkillsEngine 无 execute；执行点＝`entry/mod.rs`＋`entry/headless.rs` 两处调用方；L6 SkillEvolver 零外部调用（`skill_engine.rs:869` 为同名无关体）—— record_execution 与 Evolver 反馈线同一批接通 |
| A20 | Layer3/Layer1 冲突率 | ✅ 已量化 42/147（~29%） | `skills/index.json`（58 skills/147 triggers）vs `dispatch.rs` 449 关键词双向子串命中 42；重叠词（安全/审计/架构/测试/漏洞/文档…）E3 逐个复核归属 |
| A21 | 六维＋合规外部印证 | ✅ 6/6 对齐＋映射表 | 灵基官网六维 × E1.2 全对齐；三部委意见/强制国标/信通院指引/OWASP Top10/可信 2.0 → S5 映射表，开工即合规 |
| A22 | 实测收割回填 | ✅ 23 文件结构已验（89 agents/266 skills/11 档/12 分类/6 tools） | 只取字段名＋计数＋模式（S6.1–S6.6）；E1.2＋6 字段/E1.3＋5 字段/ModelPreference＋3 字段/TaskExecution＋2 字段；市场门禁 3 条（maturity 准入/去重/上传格式）；源 `repo-analyses/lingee-20260922/`＋`docs/plans/2026-09-22-lingee-*.md` |
| A23 | 剩余吸收（23 全过＋桌面核查＋供应链外部） | ✅ 无未开封 raw；桌面 0/10 落地 | S7.1：agent-install 33 键金矿（用量实证/升级信号/双 scope 握手/双管理/出厂技能）＋tool 11 键（三开关＋ERP 门）＋能力位表＋租户 12 键＋locale 路由＋包络约定＋双 tier 拆分＋角色链；S7.2：P0–P2 全 open；S6.7：供应链纵深 5 步（ClawHavoc 1,184/26%/76 实证） |

## 附录 B：E4 待插入节（ARCHITECTURE.md 就绪文本）

> E4 执行时将下文插入 `docs/architecture/ARCHITECTURE.md` **§2 分层架构设计之后、§3 统一数据源架构之前**（已核对该文档 12 节结构），原样可用。

```markdown
## X. 五实体投影（FIVE-ENTITY-BLUEPRINT-V2 正典）

CrystalState 为单一事实源；五实体是它的五种投影，外围注册表为投影的序列化层。

| 实体 | 正典类型 | 投影字段 | KB 键 |
|---|---|---|---|
| Workspace | `l0 nt_core_ws::WorkSpace` | `workspace: WorkspaceProjection` | `workspaces` |
| Agent | `l1 nt_infra_agent_card::AgentCard` | `agents: Vec<AgentProjection>` | `agent_cards` |
| Skill | `skill_loader::ResolvedSkill` 运行＋`skill_evolution::SkillCandidate` 进化 | `skills: Vec<SkillProjection>` | `skill_candidates` |
| Task | `l1 nt_act_scheduler::ScheduledTask` | `tasks: Vec<TaskProjection>` | `scheduled_tasks` |
| MCP | `agent::tool::mcp::McpRegistry` | `tools: Vec<ToolProjection>` | `mcp_servers` |

路由三层（Skill 触发＞Agent 能力＞CAPABILITY_ROUTES 兜底）；事件 11 增体见 CoreEvent；
编号统一 E0–E4，旧 D/I/W/P 冻结。详见 FIVE-ENTITY-BLUEPRINT-V2.md。
```

## 附录 C：风险表

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| E0.4 L3 信任环解耦改动 guard 调用链，安全行为回退 | 中 | 高 | 先加 `nt_shield_enforcer` 回归测试快照当前放行/拦截判决，再做 trait 下沉；分两 PR（加测试→再重构） |
| E1 改名（7+ 处）撞外部引用（Tauri/CLI/前端） | 中 | 中 | 改名前 `rg` 全仓（含 `src-tauri/`、前端）引用计数；`#[deprecated]` 保留期 2 迭代，不直接删 |
| E2 tick 接线拖慢 tick（每 cycle 读五实体） | 低 | 中 | 首版只读 `workspace_context` 轻量投影（ID 列表），实体详情懒加载；tick 耗时打点进 telemetry |
| CrystalState 五投影字段膨胀快照体积 | 低 | 低 | 投影只存 ID＋状态，详情存各实体 KB 键；快照体积单测断言上限 |
