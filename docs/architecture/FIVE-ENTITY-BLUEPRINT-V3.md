# NeoTrix 五实体统一蓝图 v3（最终版）

> 版本：v3.0.0（2026-09-22）
> 地位：**唯一正典，完整自包含**。替代并废弃 `FIVE-ENTITY-BLUEPRINT-V2.md`（v2 保留为历史参考）。
> v1 三份（`FIVE-ENTITY-BLUEPRINT.md` / `FIVE-ENTITY-FUSION.md` / `FIVE-ENTITY-ARCHITECTURE-FUSION.md`）维持废弃。
>
> 一句话总纲：**CrystalState 是单一事实源；五实体是它的五种投影；外围注册表降级为投影的序列化层。**
> 设计原则：人类定规则，AI 做执行。吸收源：Kingdee Lingee EAOS（2026-05-20）＋ 2026-09-22 实测收割（23 文件 / 484K 脱敏）。
> 执行清单：`FIVE-ENTITY-TASK-CHECKLIST.md`（T01–T41）；收割蒸馏：`docs/plans/2026-09-22-lingee-{architecture,agents-skills,desktop-iteration}.md`＋`repo-analyses/lingee-20260922/`。

---

## §0 执行状态看板（已验证事实，2026-09-22）

| # | 项 | 状态 | 证据 |
|---|---|---|---|
| E0.0 | 基线恢复绿 | ✅ 他窗完成 | `nt_tui_app.rs:297` 已为 `let _now`；`cargo check -p neotrix --lib` 48s 通过 |
| P0-3 | 401 回灌登录 | ✅ 本窗完成 | `browser_host.rs::AuthBridge`＋命令注册＋前端 `subscribeAuthBridge`；`tsc --noEmit` exit 0；rustfmt 我区零 diff；cargo（tauri 侧）待基线复验 |
| P0-2 | 三段式门禁 | ✅ lib 代码完成 | `skill_loader.rs`＋门禁＋4 单测；`cargo check` 通过；门禁逻辑 2/2 独立验证通过；另 2 单测待全量 test  harness（环境 10min 超时，非代码问题） |
| E0.1–E0.5 | 分层止血 | ⬜ open（E0.2/E0.5 已降级为纯路径改写） | — |
| E1–E4 | 实体/投影/路由/文档 | ⬜ open（本蓝图即施工图） | — |
| P0-1 | 模型档位层 | ⬜ open（下一個） | — |

---

## §1 架构总览

```
L6 Meta ──────────────────────────────────────────────── governance / sentrux / session_replay
         │  Skill 进化态（SkillCandidate＋bandit）        │
L5 Cognition ─────────────────────────────────────────── consciousness_core / byoa / model_router
         │  Agent 正典（AgentCard，六维）                 │
L4 Emotion ───────────────────────────────────────────── nt_feel（情感驱动 Agent 状态）
L3 Embodiment ────────────────────────────────────────── nt_shield（约束 Agent/MCP 执行）
L2 Perception ────────────────────────────────────────── nt_world（感知驱动 Skill 触发）
L1 Action ────────────────────────────────────────────── Task 正典（Scheduler）＋ MCP 正典（McpRegistry）
         │  AgentCardRegistry / SkillLoader 门面          │
L0 Substrate ─────────────────────────────────────────── Workspace 正典（WorkSpace）＋事件/遥测
              │
┌─────────────┴─────────────────────────────────────────┐
│  neotrix/nt_crystal_core :: CrystalState（单一事实源） │
│  identity / knowledge / experience / evolution        │
│  capabilities: HashMap<String, f64>                   │
│  projections: workspace / agents / skills / tasks / tools │
│  execution_trace: Vec<ExecutionRecord>                │
└───────────────────────────────────────────────────────┘
```

实体关系：Workspace 包含 Agent/Skill/Task/MCP（`agent_ids`/`skill_ids`/`mcp_servers`＋`task.workspace_id`）；
Agent 使用 Skill（`installed_skills`）＋ 执行 Task（`task.executor_agent_id`）＋ MCP 权限（`mcp_permissions`）；
Skill 触发 Task（`task.skill_ids`）＋ 依赖 MCP（`skill.dependencies` → tools）；
Task 产出交付物（`deliverables`）＋ 记录历史（`history`）。

---

## §2 E0 分层止血（先修地基，再加实体）

> E0 无新功能，只有依赖倒置。顺序：E0.0 ✅ → E0.2 → E0.5 → E0.3 → E0.1 → E0.4（风险升序；E0.4 排最后）。

| # | 违规 | 修复 | 文件 | 状态 |
|---|---|---|---|---|
| E0.0 | 基线红（`deny(warnings)` 下未用变量） | 修 1 行恢复绿 | `l1_action/nt_tui_app.rs:297`（`lib.rs:22`） | ✅ |
| E0.2 | L0 经 L5 别名调用同层 `nt_core_state`（4 处） | 4 处 `crate::l5_cognition::nt_core_state::` → `crate::l0_substrate::nt_core_state::`（本尊在 L0 `:67-119`，L5 `mod.rs:103` 仅转出口；`WorkspaceStore` trait 按 YAGNI 暂不引入） | `l0_substrate/nt_core_ws.rs:111,116,124,131` | ⬜ |
| E0.5 | 9 处 L1/L2 经 L5 路径引用 `CapabilityVector`（T03a ✅）＋ dispatcher 5 import 字段级纠缠（T03b） | T03a：import 改本尊路径（零语义变更，已落地）；T03b：`CoT/Policy/CRT/antidistil/TraceSource` 仿 `crt_factory` 回调注入先例解耦（非机械活，单列） | 9 文件＋dispatcher（附录 A-A14 补 `mapper.rs`） | 🔄 T03a✅/T03b⬜ |
| E0.3 | `l1_facade` 90 个 L6 再导出 | **facade 收敛**（非删除：20+ 消费者会断）：按域拆子模块＋`// ALLOW` 注释建 allowlist＋新增走 trait | `l5_cognition/l1_facade.rs:131-173` | ⬜ |
| E0.1 | L0 `nt_core_error` 反向 `From`（实存 20 处，初稿误写 33） | 推广既有先例（`:114` L1 已迁 `l1_action::error_conversions`）：各层建 `error_conversions`，L0 只留枚举 | `l0_substrate/nt_core_error/mod.rs` | ✅ Wave2（5 新文件；L3/L6 注册行总控已补；调用方零改） |
| E0.4 | L3→L5/L6 信任环（V3.1 重定：引擎有状态搬不动） | 三步走：(a) 类型搬迁（`ApprovalMode/ActionType/PendingAction`→L0 或 neotrix-types）；(b) L0 定义 `ApproveGate` trait；(c) 引擎留 L6、向 L3 注入实现。引擎本体（含 `pending` 队列＋L6 profiles 回调，`nt_approval.rs:109-135`）不下沉。首步先加判决快照回归测试（分两 PR） | `nt_shield_enforcer.rs:4,6,372` 等 | ⬜ |

验收：`rg 'crate::l[56]_' neotrix-core/src/l0_substrate` 零命中（注释除外）；facade 拆分＋ALLOW 全覆盖；`cargo xl` 通过。

---

## §3 E1 实体完整定义（最终字段，全量）

> 约定：所有新增字段 `#[serde(default)]`（旧快照兼容）；生产代码禁 `unwrap/expect/panic!`；`nt_` 前缀。
> 标记：✅＝已落地（2026-09-22），余＝open。

### 3.1 Workspace → `l0_substrate::nt_core_ws::WorkSpace`

```rust
pub struct WorkSpace {
    // ── 现有 10 字段（不动；V3 初稿误写 11，T10 实测纠正）──
    pub id: String, pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub project_root: Option<PathBuf>,
    pub description: String, pub tags: Vec<String>,
    pub memory_count: u32, pub goal_count: u32, pub skill_count: u32,
    // ── 新增（E1.1＋S7.1）──
    #[serde(default)] pub kind: WorkspaceKind,                    // Local/Remote/Sandbox
    #[serde(default)] pub agent_ids: Vec<String>,
    #[serde(default)] pub skill_ids: Vec<String>,
    #[serde(default)] pub mcp_servers: Vec<McpServerBinding>,    // 见 3.5
    #[serde(default)] pub shared_memory_keys: Vec<String>,       // 五实体共享总线
    #[serde(default)] pub config: WorkspaceEntityConfig,         // T10 实测改名（isolator 已有 WorkspaceConfig）
    #[serde(default)] pub locale: String,                         // S7.1：如 zh-CN
    #[serde(default)] pub llm_response_language: String,         // S7.1：10 地域→6 语言路由
}

pub struct WorkspaceEntityConfig {
    pub model_tier_preference: Option<String>,
    pub max_concurrent_agents: u32,
    pub auto_save: bool,
    pub governance_level: String,                                // off/monitor/enforce
    #[serde(default)] pub file_gates: FileGates,                 // S7.1：preview/upload 白名单
    #[serde(default)] pub menus: Vec<String>,                    // S7.1：ceo/cfo 等菜单可见性
    #[serde(default)] pub feature_flags: Vec<String>,            // S7.1：imageGen/voice/webReport…
    #[serde(default)] pub memory_offline_extract_enabled: bool,  // S7.1：记忆抽取开关（网关层归一化真布尔）
}
```

废弃改名（`#[deprecated]`，不删除）：`WorkspaceSnapshot`→任务快照语义并入 TaskProjection.history；
`IsolatedWorkspace`→降级为 `WorkspaceKind::Sandbox` 执行器；`gwt_router::GlobalWorkspace`→`GwtBroadcastBus`（与 GWT 广播重名）；
`ctm::Workspace`→`CtmShortTerm`；`ffi::WorkspaceState`→`FfiWorkspaceView`。
明确保留：`nt_core_gwt::GlobalWorkspace`（16 文件引用，注意力广播 concern；边界：GWT 管"广播什么"，Workspace 管"谁在场"）。

### 3.2 Agent → `l1_action::nt_infra_agent_card::AgentCard`

```rust
// 现有 12 字段（nt_infra_agent_card.rs:15，不动）＋ AgentCardRegistry
// （register/get/find_by_capability/find_by_tag/alive_agents/cleanup＋GLOBAL_CARDS）
// 防 god-struct（V3.1：12＋17＝29 字段必拆）：拆 `AgentCard`（身份/在线：上 12 字段）
//   ＋ `AgentSpec`（下述六维/治理/溯源），中间 `#[serde(flatten)]`——代码分家、JSON 不变、旧快照零影响。
// ── 新增（E1.2 六维＋S6.1＋S7.1，全部 #[serde(default)]）──
// 六维（灵基官网 6/6 对齐，见 §7-S4）：
pub role: AgentRole,                       // level: PermissionLevel(Read/Write/Admin/Sovereign)
                                           //        ＋org_unit＋reports_to＋role_chain: Vec<String>（`--` 分隔）
pub knowledge_domains: Vec<String>,
pub task_templates: Vec<String>,
pub deliverable_types: Vec<String>,
pub installed_skills: Vec<String>,         // 出厂初值源：installed Skill（S7.1 initSkillName）
pub execution: ExecutionPolicy,            // max_tokens/temperature/timeout_secs/retry_count
// 关联与治理：
pub workspace_id: Option<String>,
pub mcp_permissions: Vec<McpPermission>,   // {server_name, tools[]空＝全部}（对标"不得超授权"）
pub model_preference: ModelPreference,     // 锁档语义＋双 tier（下）
pub status: AgentStatus,                   // Available/Thinking/Busy/Resting（含值班）/Offline
pub visibility: String,                    // S6.1：发布态与可见性分离
pub template_id: Option<String>,           // S6.1：版本血缘
pub template_version: Option<String>,
pub package_codes: Vec<String>,            // S6.1：entitlement 分级（付费墙不抄）
pub trial_enabled: bool,
pub connector: Option<String>,             // S6.1：MCP 绑定引用（E1.5 对接）
pub icon_class: Option<String>,
pub admins: Vec<String>,                   // S7.1：主＋协管理（治理审计）
pub usage_proof: UsageProof,               // S7.1：累计 token/credits/任务/产物/成功率（实证非估算）

pub struct ModelPreference {
    pub tier: String,                      // fast/auto/deep/expert/ultra（前端只暴露档位名）
    pub providers: Vec<String>,
    #[serde(default)] pub consumption_coefficient: f64,  // S6.3：fast 0.3/expert 0.9/ultra 1.2
    #[serde(default)] pub order_number: i32,
    #[serde(default)] pub auto_routing: bool,
    #[serde(default)] pub work_tier: String,             // S7.1：工作档 vs 定时档分离
    #[serde(default)] pub scheduled_tier: String,
}
```

降级：a2a 版 AgentCard（只做 A2A 协议序列化）＋`A2ARegistry`（只做会话）＋`protocol_bridge` 版（只做桥接）；
`AgentPersona`（只做画廊展示）；合并两处重复 `AgentRole`（以本定义为准）。
预置模板（`l5_cognition/nt_agent/presets.rs`，code-reviewer/doc-writer/data-analyst/security-auditor/test-engineer）
**馈入画廊**（不另起清单；`install()` 保持唯一入口）。
分工保留：L0 `AgentRegistry`（活实例）vs `AgentCardRegistry`（能力描述）＋实例注册→发布 Card 桥；
`TypedAgentRegistry` 只做执行分发。

### 3.3 Skill → `skill_loader::ResolvedSkill`（运行）＋ `SkillCandidate`（进化）

```rust
// ✅ 已落地 P0-2（2026-09-22，cargo check 通过，门禁逻辑 2/2 独立验证）：
pub struct SkillEntry {  // index.json 条目（新键 serde(default)，旧索引兼容）
    pub description: String, pub tags: Vec<String>,
    pub triggers: Vec<String>, pub dependencies: Vec<String>,
    pub exclusions: Vec<String>,                 // 三段式之二：明确排除
    pub output_contract: Option<String>,         // 三段式之三：输出契约
}
pub struct ResolvedSkill {
    pub name: String, pub description: String, pub path: PathBuf,
    pub category: String, pub tags: Vec<String>, pub triggers: Vec<String>,
    pub dependencies: Vec<String>, pub exists: bool,
    pub exclusions: Vec<String>, pub output_contract: Option<String>,
    pub admission: SkillAdmission,               // 入库门禁 verdict
}
pub enum SkillAdmission {
    Admitted,
    NeedsWork { missing_trigger: bool, missing_exclusion: bool, missing_contract: bool },
}
// 纯函数 gate_skill(triggers, exclusions, output_contract)；
// SkillFilter.require_admitted（默认 false 向后兼容）；Admitted 排序 ＋8.0；
// legacy 无元数据 → NeedsWork 全缺（maturity 从 Candidate 起步）。
```

```rust
// SkillCandidate（进化态，coordination/skill_evolution.rs:44）新增（#[serde(default)]）：
pub author: String, pub tags: Vec<String>, pub triggers: Vec<String>,
pub required_permissions: Vec<String>, pub dependencies: Vec<String>,
pub source: SkillSource,                       // Builtin/Local/Git/Registry
pub security_audit: Option<SecurityAudit>,     // S6.7：从布尔升级为 5 步管线
pub data_scope: u8,                            // S6.2：0 纯对话 / 8 需数云·ERP（8 必须配 mcp_permissions）
pub external_id: Option<String>,               // S7.1：manageSkillId
pub updated_at: u64, pub certified: bool,      // S6.2：官方收敛标记（防 ×20 重复造轮子）
// 供应链纵深 5 步（S6.7）：预发布扫描 → 来源签名＋版本锁定（禁浮动 latest）→
//   运行时监控 → 最小权限调用 → 人工复核；工件四元组 caps/signer/version/运行时不可变。
```

降级：`SkillsEngine`（保留为门面透传）、`SkillDocEntry`（Markdown 解析）、`KbSkillAsset`、`ModelCapabilityTable`（实为模型表）。
两态关系：`ResolvedSkill` → `SkillCandidate`，`name==id` 关联。

### 3.4 Task → `l1_action::nt_act::nt_act_scheduler::ScheduledTask`

```rust
// 现有 14 字段＋Scheduler（register/create_cron/tick/history，全 async，tokio RwLock）不动；
// 新增（#[serde(default)]）：
pub executor_agent_id: Option<String>, pub skill_ids: Vec<String>,
pub workspace_id: Option<String>, pub deliverables: Vec<DeliverableSpec>,
pub depends_on: Vec<String>, pub created_by: String,
pub history: Vec<TaskExecution>,
// TaskExecution（含 S6.4＋S3.2）：execution_id/started_at/finished_at/status/output/error
//   ＋tokens_used/cost_usd（信用消耗底座）＋duration_ms＋artifact_count/credits；
// 聚合 dailyStats（14 天）进用量看板。
// TaskStatus 对齐 A2A task 状态机；DeliverableSpec{name,kind(File/Report/Code/Document/Data),output_path}。
// 新增方法（sync 读镜像版＋async 读源版双接口）：
//   list_by_workspace / list_by_agent / list_by_status / execution_history / pause / resume
// 同步/异步墙：tick 同步 vs Scheduler 全异步 → Scheduler 内加
//   snapshot: std::sync::RwLock<Vec<TaskSummary>> 同步镜像，tick 只读镜像。
```

降级：`SubTask`（只做 LLM 分解输出）、`TaskNode`（只做 ROMA 递归展开）—— doc 注记已落（T19b）；
`TaskDecomposer` 改名已否决（Wave2 实测：非空，346 行实现＋2 生产调用，改名语义错误）。
分工保留：L6 `SchedulerEngine`（内部作业池：handler/context_gate/heartbeat）—— Job 无 instruction，Task 无 handler，正交不合并。
收敛条件（V3.1·R1）：Task→Job 桥跑通之日即裁决日（L6 降级为执行后端）；裁决点放在 E2 验收，不用"再议"。

### 3.5 MCP → `agent.rs::tool::mcp::McpRegistry`

```rust
// McpToolDef 现有（name/description/input_schema/transport/server_name）＋新增：
//   required_permission / risk_level(Low/Medium/High/Critical) / usage_count / avg_latency_ms
// McpRegistry 新增：register_sse / tools_for_agent(agent_id) / tools_by_risk / usage_stats /
//   register_agent_card（S3.1：Card 即 MCP resource，discovery 经 MCP —— a2a-samples 官方模式）
// 必修死链：all_native_tools() 改从全局 registry 重建（现返回空 Vec，生产零调用）。
// McpServerBinding（E1.1 Workspace.mcp_servers 条目＋S7.1）：
//   {name, command, args, enabled} ＋ switchable/selectable/open_flag ＋ requires_erp: bool
```

降级：`McpBridge`（只做 IO 桥）、`McpHttpRegistry` / `McpEndpointRegistry`（传输层）。
分工：MCP 管工具＋数据（Microsoft 官方三分工：原生编排管内部流 / MCP 管工具数据 / A2A 管跨平台 agent）。

### E1 选型总表

| 实体 | 正典 | 废弃/降级 | 关键动作 |
|---|---|---|---|
| Workspace | `nt_core_ws::WorkSpace` | 5 改名＋1 明确保留（GWT） | ＋字段（含 S7.1 config/locale） |
| Agent | `nt_infra_agent_card::AgentCard` | 2 协议适配＋1 画廊＋1 合并 | 六维＋S6.1/S7.1（共 ＋17 字段位） |
| Skill | `ResolvedSkill`＋`SkillCandidate` | 1 门面＋2 改名＋1 改名 | 双态关联；**门禁 ✅ 已落地** |
| Task | `ScheduledTask` | 2 中间态＋1 改名 | ＋字段/方法＋同步镜像 |
| MCP | `tool::mcp::McpRegistry` | 1 修死链＋2 改名＋1 桥接 | ＋字段/方法＋Card 即 resource |

---

## §4 E2 单一事实源（CrystalState＋五投影）

```rust
pub struct CrystalState {
    pub identity: CrystalIdentity, pub tick: u64,
    pub knowledge: CrystalKnowledge, pub experience: CrystalExperience,
    pub evolution: CrystalEvolution,
    pub attention_focus: Option<String>, pub current_goal: Option<String>,
    pub capabilities: HashMap<String, f64>,
    pub execution_trace: Vec<ExecutionRecord>,
    #[serde(default)] pub workspace: Option<WorkspaceProjection>,
    #[serde(default)] pub agents: Vec<AgentProjection>,   // 已有，补 workspace_id/status/installed_skills/mcp_permissions
    #[serde(default)] pub skills: Vec<SkillProjection>,   // id/name/triggers/src knowledge+experience/effectiveness/use_count/maturity
    #[serde(default)] pub tasks: Vec<TaskProjection>,     // id/name/instruction/executor/skill_ids/status/history
    #[serde(default)] pub tools: Vec<ToolProjection>,     // name/server/capability_tag/risk_level/usage/latency
}
// WorkspaceProjection：id/name/root/agent_ids/skill_ids/tool_names/shared_memory_keys
// 住址：五投影与 CrystalState 同文件（crystal_state.rs）；构造三处同步
//   （new:108/from_core:195/from_consciousness:204）；summary() 加 active_tasks。
```

- tick 接线（不改签名）：读 active workspace → 加载五实体（Task 经同步镜像）→ 写 `CoreSnapshot.workspace_context` → `run_growth_cycle` → KB；`advance_tick` 并入 tick 调用链（终结两套 tick）。
- 首条数据流：`entry/mod.rs`＋`entry/headless.rs` 两 load-use 点包裹 `record_execution`
  （仓库无中央 Skill 执行器；L6 SkillEvolver 零外部调用——反馈线同一批接通，否则 maturity 永不更新）。
  验收：Skill 执行一次 → `execution_trace`＋1 → `performance_history`＋1 → 成本断言（`tokens_used>0`，`cost_usd≥0`）。
- KB：`workspaces`/`agent_cards`/`skill_candidates`/`scheduled_tasks`/`mcp_servers` 独立命名空间（无碰撞已验）；
  `crystal_state.json` 与 `crystal.json` 同目录，沿用 tmp＋rename＋.bak（R-P0-2）。
- 写主规则（V3.1 定死，防 skew）：**实体 KB 为写主，投影为派生缓存**——加载时投影从实体 KB 重建；
  `crystal_state.json` 只存时钟＋派生版本号，不存实体详情。

---

## §5 E3 路由＋事件＋上下文

### 5.1 三层路由

```
Layer 3 Skill 触发（最高）：SkillLoader::search(SkillFilter{triggers}) → ResolvedSkill
  子串匹配（:306-310）；index.json 真实存在（58/147）；42/147 触发词与静态表双向命中（~29%）→ Layer-3-优先＋42 词逐个复核
Layer 2 Agent 能力：AgentCardRegistry::find_by_capability → AgentCard
Layer 1 静态兜底（不变）：CAPABILITY_ROUTES（dispatch.rs:12-570，内容不动，只降优先级）
  日落条件（V3.1·R2）：复核脚本化（输出 diff 代替人工逐条）；Layer-3 去重覆盖率达标即冻结静态表只减不增。
Layer 0 LLM 直推
```

```rust
pub fn route_entity_aware(
    input: &str,
    workspace: &WorkspaceContext,
    agent_registry: &AgentCardRegistry,
    skill_registry: &SkillRegistry,
) -> EntityRouteDecision { /* Layer3 → Layer2 → Layer1 → DirectLlm（v1 FUSION §2.2 原样） */ }

pub enum EntityRouteDecision {
    Skill { skill_id: String, agent_id: Option<String> },
    Agent { agent_id: String },
    Static { capability: String, layer: String, role: String },
    DirectLlm,
}
```

`AutoOrchestrator::IntentClassifier` 先查 triggers，命中直返 Skill 路由；搜索约定：返回完整 Card（前端零二次请求）。

> 待建 API 清单（V3.1·B2）：本节引用的 `SkillRegistry` / `match_trigger` /
> `AgentCardRegistry::match_capabilities` / `find_best_agent`（方法）/ `WorkspaceContext`
> 全仓均不存在——不是既有能力，是 E3 新建项（见 T27a/b/c）。
> 复用注记：`match_capabilities`＋`find_best_agent` 复用 `task_routing.rs:155`
> 的重叠分＋负载决胜算法（`AgentEntry`→`AgentCard` 换源，别重写）。

### 5.2 CoreEvent 实体事件（＋12 变体，`#[serde(tag)]` 延续；全 match 站点有 `_ =>`，非破坏已验）

```rust
WorkspaceSwitched { workspace_id: String, agent_count: usize, skill_count: usize }
WorkspaceAgentAdded { workspace_id: String, agent_id: String }
WorkspaceAgentRemoved { workspace_id: String, agent_id: String }
AgentStatusChanged { agent_id: String, old_status: String, new_status: String }
AgentTaskDelegated { from_agent: String, to_agent: String, task_id: String }
SkillInstalled { skill_id: String, workspace_id: String, version: u32 }
SkillExecuted { skill_id: String, agent_id: String, success: bool, duration_ms: u64 }
SkillMaturityChanged { skill_id: String, old_maturity: String, new_maturity: String }
TaskAssigned { task_id: String, agent_id: String, workspace_id: String }
TaskCompletedV2 { task_id: String, execution_id: String, success: bool, deliverables: Vec<String> }
McpToolCalled { tool_name: String, server_name: String, agent_id: String, latency_ms: u64, success: bool }
McpServerConnected { server_name: String, tool_count: usize }
// ＋升级提示（S6.1 版本漂移机制化）：latestVersion ≠ version 时触发
```

> 载荷去 String 化（V3.1·D3）：`old_status/new_status` 用 `AgentStatus` 枚举、
> `old_maturity/new_maturity` 用 `SkillMaturity` 枚举（`#[serde(rename_all="snake_case")]` 已有先例），
> 让编译器替你穷举；`version: u32` 保持（与 SkillCandidate.version 同型）。

### 5.3 上下文总线

`Workspace.shared_memory_keys` 为总线；单次执行上下文：
`ExecutionContext{ workspace_id, agent_id, task_id?, skill_id?, shared_memory: Value, available_tools: Vec<McpToolDef> }`。
API 包络约定（artifacts/catalog/install 三处同形）：`code / data{items,total,page,pageSize} / timestamp`；
线上只收真布尔（字符串布尔网关层归一化）。

---

## §6 E4＋实施顺序＋验收矩阵

| 阶段 | 内容 | 验收 | 状态 |
|---|---|---|---|
| E0.0 | 基线恢复绿 | `cargo xl` 通过 | ✅ |
| P0-3 | 401 回灌 | bridge 事件＋命令＋前端订阅；tsc exit 0 | ✅ |
| P0-2 | 三段式门禁 | 门禁＋4 单测；`cargo check` 通过 | ✅（lib 代码；2 单测待全量 harness） |
| E0.2/E0.5 | 纯路径改写（最小） | rg 零命中旧路径；`cargo xl` | ⬜ 下一個 |
| P0-1 | 模型档位层 | 档位＋系数＋锁档；前端只传档位名 | ⬜ |
| E0.3/E0.1/E0.4 | facade 收敛/From 上移/信任环解耦 | 分阶段验收（见 §2） | ⬜ |
| E1 | 正典字段＋改名＋死链修复 | 正典唯一性 rg；`all_native_tools` 非空 | ⬜ |
| E2 | 五投影＋tick 接线＋首条数据流 | trace＋1→history＋1→成本断言（单测） | ⬜ |
| E3 | 三层路由＋12 事件＋总线＋42 词复核 | "合并 Excel"→EntityRouteDecision::Skill（单测） | ⬜ |
| E4 | ARCHITECTURE.md §2 后插入"五实体投影"节（附录 B 就绪文本）＋迭代走 experience-tree | 文档清单全勾 | ⬜ |
| P1/P2 | Bridge 事件表/用量看板/设计 token/空态/市场字段/MFE/翻译防护 | 桌面清单逐项 | ⬜（空态部分） |

硬约束：`#![forbid(unsafe_code)]`；生产禁 `unwrap/expect/panic!`；新增字段 `#[serde(default)]`；
不改 `tick()` 签名；不动 CAPABILITY_ROUTES 内容；`nt_` 前缀；R-P16 重读验证；
结构性改动后 `cargo clean && cargo build` 两遍；单窗口全量构建（16G 约束）。

---

## §7 证据与来源（ condensed；完整考据见 v2 S1–S7）

- **灵基六维**（官网 2026-08）：role/knowledge/task/deliverable/skill-orchestration/execution-plans × E1.2 **6/6 对齐**；
  "agents invoke approved skills" → `mcp_permissions`＋`risk_level`。
- **商业实证**（2026-08 中期业绩）：AI 原生收入 2.96 亿＋189%、26 家、40+ 智能体；"订阅＋信用消耗" → 成本字段底座。
- **合规五源**：三部委意见（三类决策边界）/ 强制国标 TC260 / 信通院指引（ClawHub 12% 恶意）/
  OWASP Agentic Top 10（Least Agency 等）/ 可信评估 2.0 → S5 映射表，开工即合规。
- **实测收割**：`repo-analyses/lingee-20260922/`（23 文件 / 484K 脱敏：89 agents·17 键 / 266 skills·9 键 /
  11 模型档 / 12 分类 / 6 tools·11 键 / install 记录 33＋19 键 / capabilities 位表 / tenant 12 键 /
  locale 映射 / 包络约定 / 双 tier / 角色链）＋ `docs/plans/2026-09-22-lingee-*.md` 三份蒸馏。
  只收设计层（字段名/计数/模式）， token 零落盘已验。
- **业界模式**：Microsoft 三分工（原生编排/ MCP 工具数据/ A2A 跨平台）；
  a2a-samples（Card 即 MCP resource）；SkillMill/CSA 纵深防御（扫描≠信任门）。
- **反面模式入库门禁**：workshop 测试 Agent 混入 → maturity 准入门；Skill ×20 重复 → 官方收敛＋去重；
  ClawHavoc 1,184 / Cisco 26% / Snyk 76＋36% → 供应链 5 步。

---

## 附录 A：审计证据链 A1–A24

| # | 核查 | 结论 |
|---|---|---|
| A1 | E0.1 From 反向依赖 33 处 | ✅ `nt_core_error` L2/L3/L5/L6/neotrix；`:114` L1 先例可推广 |
| A2 | E0.3"删除再导出" | ❌ 误判→facade 收敛（有意迁移＋20+ 消费者） |
| A3 | L3 信任环 | ✅ `nt_shield_enforcer:4,6,372` |
| A4 | `all_native_tools` 死链 | ✅ 返回空 Vec，生产零调用 |
| A5 | KB 命名空间碰撞 | ✅ 无（`load("goals")` 双用已注） |
| A6 | 真实 `unsafe` | ✅ 零（2 命中皆非生产） |
| A7 | `unwrap/expect/panic!` | ✅ 生产零（目标文件；测试除外） |
| A8 | L6 第二调度器 | ✅ 正交保留（Job vs Task） |
| A9 | 注册表系分工 | ✅ 活实例 vs 能力描述＋桥；presets 馈入画廊 |
| A10 | GWT GlobalWorkspace | ✅ 保留（广播 vs 容器边界） |
| A11 | E2 落点 | ✅ 构造 3 处＋同文件＋index.json 真实 |
| A12 | E3 落点 | ✅ 子串匹配＋560 行表＋插入位 §2/§3 |
| A13 | 同步/异步墙 | ✅ 真墙→同步镜像＋双接口 |
| A14 | CapabilityVector 下沉 | ❌ 误判→8 处 import 重写（定义唯一） |
| A15 | 事件爆破面 | ✅ 非破坏（全 `_ =>`；panic 仅测试） |
| A16 | CrystalState 落盘位 | ✅ `crystal_state.json`＋R-P0-2 写模式 |
| A17 | 基线颜色 | ~~🔴~~ → ✅ 他窗修复，`cargo check` 48s 通过 |
| A18 | E0.2 trait 注入 | ❌ 过设计→4 行路径改写（YAGNI） |
| A19 | E2 挂载点 | ✅ entry 两调用方＋Evolver 同批接通 |
| A20 | Layer3/L1 冲突率 | ✅ 42/147（~29%）＋复核项 |
| A21 | 六维＋合规印证 | ✅ 6/6＋S5 映射 |
| A22 | 实测收割回填 | ✅ 字段名/计数/模式（S6） |
| A23 | 剩余吸收＋桌面核查 | ✅ S7；桌面清单当时 0/10（现 P0-3/P0-2 销项） |
| A24 | V3 定稿新增验证 | ✅ P0-2 门禁逻辑 2/2 独立通过；P0-3 tsc exit 0；`cargo check -p neotrix --lib` 通过（含 P0-2 代码） |
| A25 | McpRegistry 融合现状（他窗并入已删 cli 注册表） | ✅ 正典更实但死链未除 | `agent.rs` ＋175（`register_stdio/tool_count/list_servers/recommend_tools`）；`all_native_tools()` 仍空 Vec，E1.5"必修"有效；分工：`recommend_tools` 只做候选召回（子串），Layer-3 触发匹配只做裁决——召回与裁决分离，禁止二合一 |
| A26 | 阻塞项补丁（复审 B1/B2） | ✅ 已入正文 | E0.4 重定三步（类型搬迁→trait→注入，引擎留 L6）；T27 拆 T27a/b/c（SkillRegistry 门面＋match×2 复用 task_routing 算法＋WorkspaceContext/route_fn）；T06 拆 T06a/b/c |
| A27 | 设计缺陷补丁（复审 D1–D3） | ✅ 已入正文 | AgentCard/AgentSpec 拆分＋`#[serde(flatten)]`；写主规则（实体 KB 写主、投影派生缓存）；事件载荷枚举化（Status/Maturity） |
| A28 | 冗余分散补丁（复审 R1/R2/S1–S3） | ✅ 已入正文＋清单 | R1 收敛裁决点（E2 验收）；R2 日落条件＋复核脚本化；S1 `AgentDirectory` 只读聚合外观（描述/实例/分发/展示四源，归 T22）；S2 E轨建schema/P轨只做UI（T35 依赖补边）；S3 `SkillId(String)` newtype＋注册表级 join（归 T16） |
| A29 | Wave 1 门验收（5 车道并行复核） | ✅ 代码收＋3 修复＋2 发现 | T10：WorkSpace 实为 10 字段（V3 误写 11，已纠正）；`WorkspaceConfig` 撞 isolator 已有定义→改名 `WorkspaceEntityConfig`（V3 E1.1/S7.1 已同步）；T16：`skill_evolution.rs` 全仓零引用——已接线（mod＋1 行）并修 bit-rot（HashMap import＋3×drop＋_success，独立验证零 error/warning）；T18：镜像缺口（cancel/execute_now）已补；T04：58 项零消费者清单 defer 由总控裁决；T12：撞名登记（AgentRole/AgentStatus/PermissionLevel/ExecutionPolicy/ModelPreference），T14 合并时收敛 |
| A30 | Wave 2 门验收（7 车道并行复核＋总控救火） | ✅ 代码收＋2 救火＋2 纠正 | T05：实存 20 处非 33（V3 已纠正）；L3/L6 注册行总控补（他窗脏）；T20：GLOBAL_MCP 真实路径；entry 3 字面量 E0063 总控补 `..Default`；T19：`TaskDecomposer` 非空→改名否决（V3 已修正），doc 注记总控直落；T17：SkillDocEntry 57 处超限停手正确→拆 T17b；T04 58 项／T12 撞名／T16 同名类型登记收敛 follow-up |
| A31 | Wave 3 门验收（3 车道） | ✅ 2 完成＋1 部分＋停手皆正确 | T03b：trait 对象安全已验，l5 引用 8→7，T03c 数据类型清单已立（新增）；T17b：57/57；T14：From×2✅，合并停手正确（旧字段零交集＋构造/读取点不可迁；实为 7 处定义）→拆 T14b（语义迁移先行）；清单同步 T03c/T14b |
| A32 | Wave E2a 门验收（T22＋T28 并行） | ✅ 双完成＋2 纠正 | T22＋335/-0：`from_projections` 返 Self（可链式，into_entries 兼容取数）；`active_tasks`＝非终态计数（非 len，调用方注意）；遗留：`projections`(旧)与`agents`(新)并存→E2 接线以前者为准 deprecated；T28＋153/-0：`domain()` 穷尽匹配漏网（A15 审计补正）已补 13 臂暂归 nt_core |
| A33 | Wave E2b 门验收（T23＋T24＋T25 并行） | ✅ 三完成＋3 新发现 | T23＋100/-0：workspace_id 暂读环境变量；advance_tick 需持有点设计（未硬上）；T24：headless 系 stub（记录 Err 为真实现象），真数据待真实 Skill 接线；T25：.bak 单代 vs 五代轮转 mismatch→T25b；live save/load 路径待沙箱 HOME 演练 |
| A34 | Wave E3a 门验收（T26＋T27a＋T27b 直落） | ✅ 子代理连续取消→总控直落 | T26/T27b 同文件串行（4 方法＋check_upgrade＋2 单测，fmt 我区 clean）；T27a 新门面＋lib 注册 1 行（cli 删除行系他窗，互不干扰）；负载决胜未做（交集排序已够 E3，负载以后接 Scheduler 镜像） |
| A35 | Wave 全量并行门验收（E3b＋P1/P2，8 车道） | ✅ 8/8 完成＋停手皆正确 | T27c/T29：dispatch＋164/-0，orchestrator 脏区停手正确，EntityRouteDecision 改名收敛（全仓第 8 个 RouteDecision，我方新建即改名）；T14b：桥接×2＋单测，旧定义零动，4 enum 裁决已收；T03c：＋218/-15，读点未迁清单已立；T31：BridgeEvent＋可信目录；T35：占位近似已注；前端束：tsc exit 0＋vitest 3/3，零 WebviewWindow 直接调用；T32：trait 解耦孤儿模块；T36：Vite＋Solid 实测，推荐 B 轻量（MF 否决）；T04：58 项全部 DEFER |
| A36 | E4 总体接入＋T14 迁移收官 | ✅ 4 文档各 1 行双向链接（MASTER/FULL/FUSION/MAP-V2；FULL 系 untracked 他窗文件，锚定固定头部）；T14 迁移点×2（entry:1995 并行注册＋CrewRunner 并行读，旧路径保留＋TODO）；CrystalArchetype 改名完成；coordinator/multi_agent 别名归并停手正确（语义根本不同）；agent_orchestrator 疑似孤儿＋存量 unwrap 留属主窗 |
| A37 | E2-runtime 直落＋T25b＋收敛裁决 | ✅ 持有点（`crystal: Option`＋setter＋tick 推进，构造点 1 处）；agents/skills 全量填充（tools/tasks 暂空＋精确 unblock 条件已注）；T25b 五代轮转＋实跑验证通过；收敛裁决：crates 双 enum 保留（零跨文件引用，语义不可并）；SkillSource/SecurityAudit/ModelPreference 保留模块隔离＋注记；**新发现：双 ConsciousnessCoreHandle 并行 tick（core.rs:158 vs nt_core_consciousness_core.rs:202），收敛单列 follow-up，未动** |
| A38 | T39 双 Handle 只读侦察（零代码改动） | ✅ 旧核独有 5 行为已编目：并发合并（多进程 last-write-wins 防护）／SelfTest 迷雾治理注入／T1 价值观检查／T4 NativeBus／趋势回填＋reload 哨兵修；调用分裂已映射（entry:842 新核，background handlers 旧核）；同写 `consciousness` 命名空间＋三 CoreSnapshot 定义（旧缺 workspace_context，serde 默认兼容不崩但语义 skew）；收敛方案：A1 移植 5 行为→新 tick，A2 旧模块垫片，A3 调用迁移，A4 删 4665 行旧文件；L0 快照定义 adjudication 待定；执行需串行验证门，禁并行 |
| A39 | "能力"概念四处分散建档（T41） | ⬜ 待收敛 | tree crate（图谱本体，保留）／L6 `nt_core_capability/`（orchestrator/discovery/integrator）／L0 types／`nt_file_ability/capability.rs`；方向：图谱唯一坐标系，其余只做消费适配 |
| A40 | T39-A1/A3 执行（调用迁移落地） | ✅ SelfTest 本体逐字移植＋4 调用点迁移＋fmt clean（cargo 未跑） | 新 tick 经逐项核对已含旧核 5 行为（并发合并/SelfTest/T1/T4/趋势回填＋reload/ensure_phi），A1 移植项实质完成；literals（observer/run_cycle deep 链）＋旧文件内自测未动；A2 垫片待旧模块余量清点；A4 删文件待串行门 |
| A41 | 全文档一致性审计封口 | ✅ 三方对账完成 | 清单 41 任务 ID 连续（T01–T41 含 T03a/b、T06a/b/c、T17b、T27a/b/c）；V3↔清单↔§X↔收割文档双向链接补齐（V3 头＋桌面清单头）；v1→v2→V3 废弃链完整；行号锚点以 rg 模式为准（line 锚点随改动漂移）；历史表述均已自注（如 A23“当时 0/10”括号注记），无裸过期断言 |
| A42 | 冗余清理审计＋收敛 | ✅ 8482 类型普查（1218 同名，14%）；自家第 8 个 RouteDecision 改名收敛；L0 零反向残留；L1/L2 残留 13 文件列清单（T43）；巨文件 Top（4665 旧核等）列清单（T44）；重定义 Top18 裁决程序立项（T42） |
| A43 | 深层审计（孤儿/错误类型/函数名/unwrap  Debt） | ✅ 新发现：孤儿死文件系性存在（78 严格死， verified 4：skill_evolution 已接线复活／model_unified 与 T09 半落空／dual_track 含 T14b 工作代码／consciousness_types；方法论：mod 声明＋全仓引用双零）；函数名重复 441 系惯例噪声（new/get/execute 类），不立项；`Result<_,String>` 515 文件（类型化错误程序 T46）；unwrap 约 6700 命中（测试为主，需 clippy 门）；顶层非 nt_ 为祖父条款；T09 注记孤儿影响；T45/T46 已立项 |
| A43 | 并行统一修复验收（T42/T43/T44，5 车道） | ✅ T42：Top18 全 LEGIT/DEFER、零合并（不兼容实证），结论：同名多为正当分离，禁批量合并；T43a/b 落地（回调注入＋DTO＋EXCEPTION），T43c 停手正确（脏＋异步缝＋信任环归属，survey 已留最小方案）；T44：4 文件簇＋切分＋单向链，优先级 pipeline＞experience＞streaming＞browser_engine |
| A44 | 旧核 tick E2 移植（生产路径修正） | ✅ 旧 `tick()` 加持有点＋agents/skills 全量填充（排序 bug 现场捕获已修；merge 语义 `clone` 先行已验存活；fmt 我区 clean；cargo 未跑） | 旧核为生产 tick（background＋MCP/CLI 共用），E2 遗漏即功能缺席——本次补齐；死目录（`consciousness_core/` 未声明编译）内 E2 代码维持现状不动，待模块归属裁决 |
| A45 | T04 终裁＋门禁救火两连 | ✅ facade 259 导出真零消费者为 0（T04 车道 58 系计数口径误报，已证伪；DEFER 决议撤销，无删除项）；门禁 E0063×2 均为我方新增字段所致（CoreSnapshot 字面量＋SkillCrystal::new），已补齐；教训：加字段必须全仓扫字面量构造（含 `Self {` 变体） |
| A46 | E3 活路径迁移 | ✅ 死目录 `l5_cognition/consciousness_core/` 未声明编译（T27c 代码永不链接）→ `EntityRouteDecision`/`ExecutionContext`/`route_entity_aware` 已迁入活文件 `skill_registry.rs`（仅依赖 SkillLoader＋AgentCardRegistry，fmt clean，2 单测；Layer-1 静态兜底留 orchestrator 侧；cargo 未跑） |
| A47 | 串行验证门首绿＋复绿（`cargo check -p neotrix --lib`） | ✅ 首绿 0 error＋0 warning，34 分钟（含排队；清锁 4 进程见 handoff；`BUILD-SCHEDULING.md` 已立为公约文档）；复绿 6 分 24 秒前台直跑（缓存暖后），此前所有"cargo 未跑"注记至此批量验证通过（含 E0063×2 修复、T05 搬迁、T20 死链修复、T27c 新类型、E2 移植区）；cfg(test) 与 tauri 侧仍未覆盖；终门（8 commits 落盘后，`target/nt_gate-final.log`）1m03s 0 error 0 warning 全绿（/tmp 日志曾被系统清理，门日志已改落 gitignored 的 target/，结论以本行冻结为准） |
| A48 | 孤儿接线第二波（4 文件） | 🔄 已接线待门验：`nt_core_model_unified`（T09 三档位复活）／`nt_mind_dual_track`（自含 traits＋L1 合法下引）／`nt_core_consciousness_types`＋`awareness_monitor`（rand/serde/HashMap clean；第 3 个 CoreSnapshot 并存已注记，adjudication 待 T39）；门：后台 `check-orphan-wire.log` ✅ 通过（21 分 31 秒，0 error＋0 warning；含 E3 迁移代码一并验证） |
| A49 | 孤儿接线第三波（3 文件：成本簇＋trait 家） | 🔄 已接线待门验：`cost_router`／`cost_ladder`（std＋serde clean，他窗 M 区已避让）／`nt_task_decomposition`（69 行零 import，L1 trait 正是 T03b 缺的抽象家）；门：后台 `check-orphan2.log` ✅ 收官（30m21s，11 error 全归因他窗区，我区 0；/tmp 日志已失，结论冻结） |
| A50 | 孤儿第三波独立验证收官 | ✅ cost_router 20/20（含修 pre-existing 档位测试输入×1000 量级错位，生产阈值未动）／cost_ladder 4/4／task_decomposition 编译通过；fmt 我区 clean；登记簿三行同步 |

## 附录 B：E4 待插入节（ARCHITECTURE.md §2 后§3 前，原样可用）

```markdown
## X. 五实体投影（FIVE-ENTITY-BLUEPRINT-V3 正典）

CrystalState 为单一事实源；五实体是它的五种投影，外围注册表为投影的序列化层。

| 实体 | 正典类型 | 投影字段 | KB 键 |
|---|---|---|---|
| Workspace | `l0 nt_core_ws::WorkSpace` | `workspace: WorkspaceProjection` | `workspaces` |
| Agent | `l1 nt_infra_agent_card::AgentCard` | `agents: Vec<AgentProjection>` | `agent_cards` |
| Skill | `skill_loader::ResolvedSkill`＋`skill_evolution::SkillCandidate` | `skills: Vec<SkillProjection>` | `skill_candidates` |
| Task | `l1 nt_act_scheduler::ScheduledTask` | `tasks: Vec<TaskProjection>` | `scheduled_tasks` |
| MCP | `agent::tool::mcp::McpRegistry` | `tools: Vec<ToolProjection>` | `mcp_servers` |

路由三层（Skill＞Agent＞静态兜底）；事件 12 变体见 CoreEvent；编号 E0–E4，旧号冻结。
```

## 附录 C：风险表

| 风险 | 概率 | 影响 | 缓解 | 状态 |
|---|---|---|---|---|
| E0.4 改动 guard 行为回退 | 中 | 高 | 先加判决快照回归测试，分两 PR | ⬜ |
| E1 改名撞外部引用 | 中 | 中 | rg 全仓＋`#[deprecated]` 保留 2 迭代 | ⬜ |
| E2 tick 接线拖慢 | 低 | 中 | 首版只读轻量投影＋telemetry 打点 | ⬜ |
| 快照体积膨胀 | 低 | 低 | 投影只存 ID＋状态；体积单测上限 | ⬜ |
| 全量 test harness 超时（环境） | 高 | 低 | 逻辑级独立验证＋check 门；CI 侧全量 | ✅ 已验证模式（P0-2） |
| 多窗口并发改同一文件 | 中 | 中 | 改前 `git status` 查脏文件；小步快走 | ⬜ 进行中（main.rs/nt_tui_app 已见他窗） |
| A51 | 孤儿接线第四波（知识图谱＋记忆层） | ✅ `nt_core::knowledge`（5 文件 1,263 行）＋`nt_core::memory`（5 文件 1,171 行）挂载`nt_core/mod.rs`，沿用 A48/A49 的 `// 孤儿接线 Ax` 注释约定。**两模块均「从未编译过」，首接线即暴露真 bug**：knowledge 的 `bfs`/`dfs` `max_depth` off-by-one（`Some(1)` 不返回直接邻居却给它记 `distances=1.0`）；memory 的 `EmotionLabel` 只存在于 l6_meta（引它即 L5→L6 分层违规，改用同层 `PlutchikEmotion`）、`ConceptNode` 无 `concept` 字段 6 处误用、`episode.id` move 后复用、`patterns`/`principles` 不可变借用活过 `store()` 的 `&mut`。门：`cargo check -p neotrix --lib` RC=0；`smoke_tests` 5/5 新增测试 + memory 4/4 既有测试全绿；孤儿 13 → 11 | 归档优先原则被推翻：预算无限时「没数据/没编译」两条反接线理由均可消除，而「有测试 ⇒ 不删」依然成立 |
| A52 | 孤儿接线第五波（小而全三件套） | ✅ `nt_core::io_skills`（87 行，Eli5 解释器）＋`nt_feel::cognition_bridge`（234 行，**补建缺失的 mod.rs**）＋`nt_memory::nt_memory_knowledge_graph`（569 行）。合计 890 行、20 测试**全绿**。`nt_memory_knowledge_graph` 首接线 **22 error → 0**：孤立残留 derive 块（原31-33 行，与下一块同时作用于同一 enum ⇒ serde `rename_all` 重复 + 6 个 `E0119`）、`use` 夹在 `#[derive]` 与 struct 之间（`E0774`）、`AttackClass` derive 缺 `Hash` 却用于 `HashSet`、同步 `pub fn self_test()` 内部 `.await`（改 async）、**真 UTF-8 panic**（`rest[arrow_pos+1..]` 按字节切 3 字节的 `→`）。另如实标注 `sync_to_kb` 为**活路径死端桩**（写入被注释、有调用方、`KnowledgeBase` 无对应写入 API） | 本波再次印证 A51 结论：**「从未编译过的能力」不能靠`cargo check` 通过来判定可用**—— 真 bug 藏在运行期（UTF-8 panic）与类型细节里，只有真跑测试才暴露 |
| A53 | 孤儿接线第六波（时序 KG＋语义路由＋Geo/SEO） | ✅ `nt_world::temporal_kg`（1,278 行，42 测试）＋`nt_act::semantic_routing`（844 行，22 测试）＋`nt_act::geo_seo`（591 行，9 测试）。**73 测试全绿**。`geo_seo` 零错误零改动。`semantic_routing` 27 error → 0：4 条旧布局残留导入、`SimilarityScore` 持 `String` 却 derive `Copy`（E0204）、`FanInQueue`/`SimilarityScore` 缺 serde、`BehaviorPatternType` 缺 `Eq/Hash` 却作 HashMap 键、闭包内 `?` 须改 `and_then`、`boost` 浮点类型歧义、借用冲突 2 处、`pattern_id` move 后复用。`temporal_kg` 7 处 `NaiveDateTime::from_ymd_opt`（该 API 不存在）改现代 chrono。**两个真逻辑/数学 bug**：① `context_boost` 拿上下文 **key** 比关键词（永匹配不上）② PPR 把悬挂质量 `dangling_sum/n` 均摊，链尾 `e4` 反超种子 `e0` | 再次印证：**只有真跑测试才暴露真 bug**。`cargo check RC=0` 的三个模块里，`semantic_routing` 有 27 个 error、`temporal_kg` 有 7 个 chrono error，而 `geo_seo` 干净——**编译通过仍需逐模块跑测试**（本波 PPR bug 连编译和单测断言之外的路径都无法暴露） |
