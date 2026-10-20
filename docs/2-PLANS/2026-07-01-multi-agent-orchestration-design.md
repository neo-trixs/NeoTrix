# Multi-Agent Orchestration Layer (L7 Capability): Detailed Design

**Date**: 2026-07-01
**Layer**: L7 Capability
**Status**: Design v1

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    L7 Capability (能力层)                       │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              nt_cap_orch_graph (图引擎)                  │   │
│  │  ┌─────────┐  ┌──────────┐  ┌───────────┐              │   │
│  │  │AgentNode│→│AgentEdge │→│AgentReducer│              │   │
│  │  └─────────┘  └──────────┘  └───────────┘              │   │
│  └──────────┬──────────────────────────────────────────────┘   │
│             │ 使用                                │             │
│     ┌───────▼──────────┐            ┌────────────▼──────────┐  │
│     │ nt_cap_orch_gate │            │ nt_cap_orch_budget    │  │
│     │ (质量门控)       │            │ (预算控制器)         │  │
│     └──────────────────┘            └───────────────────────┘  │
│                                                                 │
│  ┌───────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │nt_cap_orch_handoff│  │nt_cap_orch_fan │  │nt_cap_orch_hitl│ │
│  │(交接协议)         │  │(扇出/扇入)    │  │(人机协同)      │ │
│  └───────────────────┘  └────────────────┘  └────────────────┘ │
│                                                                 │
│  ┌───────────────────┐  ┌────────────────┐                      │
│  │nt_cap_orch_debate │  │nt_cap_orch_trace│                      │
│  │(多轮讨论)         │  │(追踪跨度)      │                      │
│  └───────────────────┘  └────────────────┘                      │
└─────────────────────────────────────────────────────────────────┘
         │                          │
         ▼                          ▼
    L4 Cognition              L1 Body / L0 Substrate
    (E8, PRM, SAE)            (IO, MCP, StateGraph)
```

## 2. Module Specifications

### 2.1 `nt_cap_orch_graph` — Graph-Based Multi-Agent Workflow

**Purpose**: Connect StateGraph (L0) with AgentTeam (L1 agent layer). Wrap `AgentRole` as `AgentNode`, define `AgentEdge` for result-based routing, `AgentReducer` for output merging.

**Key types**:

```rust
/// An agent wrapped as a graph node
pub struct AgentNode {
    pub role: AgentRole,
    pub system_prompt: String,
    pub handler: Arc<dyn Fn(&mut AgentState, &AgentRole) -> Result<AgentOutput, String>>,
}

/// Edge with output-based routing
pub struct AgentEdge {
    pub source: NodeId,
    pub target: NodeId,
    pub condition: Box<dyn Fn(&AgentOutput) -> bool>,
}

/// State flowing through the graph
pub struct AgentState {
    pub task: String,
    pub accumulated_output: HashMap<String, AgentOutput>,
    pub turn_count: usize,
    pub metadata: HashMap<String, String>,
}

/// Compiled multi-agent workflow
pub struct AgentGraph {
    pub graph: StateGraph<AgentState>,
    pub agents: HashMap<NodeId, AgentNode>,
    pub edges: Vec<AgentEdge>,
}
```

**Integration with existing team.rs**:
- `AgentTeam::compile()` → returns `AgentGraph`
- `SwarmMode::BossTeam` → generates Graph: entry → decompose → fan_out → fan_in → synthesize
- `SwarmMode::ChainRefine` → generates Graph: A → B → C → D (linear)
- `SwarmMode::DevilsAdvocate` → generates Graph: propose → challenge → resolve

### 2.2 `nt_cap_orch_handoff` — Agent Handoff Protocol

**Purpose**: OpenAI Agents SDK-style handoff where agent A decides to pass control to agent B with full context.

```rust
pub struct HandoffPayload {
    pub source_agent: String,
    pub target_agent: String,
    pub context: String,
    pub accumulated_state: HashMap<String, String>,
    pub handoff_reason: String,
}

pub trait HandoffStrategy: Send + Sync {
    fn should_handoff(&self, output: &AgentOutput, state: &AgentState) -> Option<String>;
    fn select_target(&self, candidates: &[AgentRole], context: &str) -> Option<String>;
}
```

**Integration with AgentBus**:
- `SupervisorAgent.dispatch_handoff()` publishes to `BusTopic::HandoffRequest`
- Workers subscribe to `HandoffRequest` and claim if capable
- Cross-process handoff via TCP transport (future)

### 2.3 `nt_cap_orch_gate` — Supervisor Quality Gate

**Purpose**: Worker output → Gate → pass/fail/revise decision before next stage.

```rust
pub struct QualityGate {
    pub name: String,
    pub criteria: Vec<GateCriterion>,
    pub pass_score: f64,
}

pub enum GateCriterion {
    Completeness { min_length: usize, required_sections: Vec<String> },
    Coherence { min_coherence: f64 },
    Safety { blocked_patterns: Vec<String> },
    Custom { name: String, check: Box<dyn Fn(&str) -> f64> },
}

pub enum GateDecision {
    Pass(f64),
    Revise { score: f64, feedback: Vec<String> },
    Fail { reason: String },
}
```

**Pattern**: AgentOutput → QualityGate → [Pass → next] | [Revise → worker (max N retries)] | [Fail → escalation]

### 2.4 `nt_cap_orch_budget` — Orchestration Budget Controller

**Purpose**: Cap LLM calls per orchestration to prevent cost explosion.

```rust
pub struct Budget {
    pub max_llm_calls: usize,
    pub max_tokens: usize,
    pub max_cost_usd: f64,
    pub max_wall_time_ms: u64,
}

pub struct BudgetTracker {
    pub llm_calls: usize,
    pub tokens_used: usize,
    pub cost_usd: f64,
    pub elapsed_ms: u64,
    pub exceeded: HashSet<BudgetExceeded>,
}

pub enum BudgetExceeded {
    LlmCalls,
    Tokens,
    Cost,
    Time,
}
```

**Integration**: AgentTeam + AgentGraph carry a `Budget`. AgentTeam.execute() checks `tracker.exceeded.is_empty()` after each call. StateGraph.run() halts on budget exceeded.

### 2.5 `nt_cap_orch_fanout` — Fan-out / Fan-in

**Purpose**: Broadcast to N workers in parallel, collect and merge results.

```rust
pub enum FanOutStrategy {
    Broadcast,       // All workers get the same task
    Partition { key_fn: String }, // Split input by key
    RoundRobin,
}

pub enum FanInStrategy {
    Concat,
    Vote,             // Majority vote
    MaxScore,         // Pick best by score
    WeightedMerge { weights: Vec<f64> },
    LLMSynthesize { prompt_template: String },
}

pub struct FanOutResult {
    pub workers: Vec<(String, AgentOutput)>,
    pub aggregated: String,
    pub strategy_used: FanInStrategy,
}
```

### 2.6 `nt_cap_orch_hitl` — Human-in-the-Loop

**Purpose**: Pause orchestration, present output to human, wait for decision, resume.

```rust
pub struct HITLConfig {
    pub pause_points: Vec<PauseTrigger>,
    pub timeout_minutes: u64,
    pub escalation_email: Option<String>,
}

pub enum PauseTrigger {
    BeforeGate { gate_name: String },
    AfterNFailures { n: usize },
    BudgetThreshold { fraction: f64 },
    ManualBreakpoint { node_id: String },
}

pub enum HumanDecision {
    Approve,
    RejectWithFeedback(String),
    Modify(String),
    Escalate,
}
```

**Integration with StateGraph**:
- `Checkpoint` stores state at pause point
- HITL resume = `CompiledGraph::run_with_recovery(checkpoint_id, human_input)`
- Pending decisions stored on `CheckpointStore` with TTL

### 2.7 `nt_cap_orch_debate` — Multi-Round Debate Engine

**Purpose**: Iterative debate with convergence detection. Upgrade from one-shot `DevilsAdvocate`.

```rust
pub struct DebateConfig {
    pub max_rounds: usize,
    pub participants: Vec<String>,
    pub roles: HashMap<String, String>, // "proposer" | "challenger" | "synthesizer"
    pub convergence_threshold: f64,     // JS divergence under this = converged
    pub min_rounds: usize,
}

pub struct DebateRound {
    pub round: usize,
    pub proposals: HashMap<String, String>,
    pub critiques: HashMap<String, String>,
    pub synthesis: Option<String>,
    pub divergence: f64,
}

pub struct DebateResult {
    pub rounds: Vec<DebateRound>,
    pub converged: bool,
    pub final_synthesis: String,
    pub consensus_score: f64,
}
```

### 2.8 `nt_cap_orch_trace` — Orchestration Telemetry Spans

**Purpose**: Wrap every agent call, edge traversal, and gate decision with telemetry spans.

```rust
pub struct AgentSpan {
    pub trace_id: String,
    pub parent_id: Option<String>,
    pub span_type: SpanType,
    pub agent_name: String,
    pub start: Instant,
    pub duration_ms: u64,
    pub tokens_used: usize,
    pub cost_usd: f64,
    pub status: SpanStatus,
}

pub enum SpanType {
    AgentCall,
    GateCheck,
    Handoff,
    FanOut_Wait,
    HITL_Pause,
    Debate_Round,
}
```

**Integration with `nt_io_telemetry::Tracer`**:
- `Tracer::start_span()` at each AgentNode entry
- `Tracer::end_span()` at exit
- Aggregate cost + latency per orchestration

## 3. Integration Plan

### Step 1: Connect StateGraph <> AgentTeam (1 day)
- Add `AgentGraph` struct wrapping `StateGraph<AgentState>` + `HashMap<NodeId, AgentNode>`
- Implement `AgentRole::as_node()` → `GraphNode<AgentState>`
- Implement `SwarmMode` → generates pre-wired graph templates
- Keep `AgentTeam::execute()` as facade that compiles → runs

### Step 2: Port existing patterns (1 day)
- BossTeam → Orchestrator-Worker template
- ChainRefine → Sequential template  
- DevilsAdvocate → Debate template (upgrade to multi-round)
- AllVote → FanOut + FanIn(Vote) template

### Step 3: Add new patterns (2 days)
- nt_cap_orch_gate (quality gate)
- nt_cap_orch_handoff (handoff protocol)
- nt_cap_orch_budget (budget controller)
- nt_cap_orch_hitl (human-in-the-loop)

### Step 4: Telemetry + optimization (1 day)
- nt_cap_orch_trace (spans)
- nt_cap_orch_fanout + nt_cap_orch_debate
- Router upgrade from keyword → CapabilityVector similarity

## 4. Migration Path for Existing Code

| Current Code | Migrates To | Change |
|-------------|-------------|--------|
| `AgentTeam::execute()` | `AgentGraph::run()` | Add compile step; keep facade for backward compat |
| `SwarmMode::DevilsAdvocate` | `DebateEngine::run()` | Add multi-round, convergence |
| `Coordinator::route_task()` | `CapabilityRouter::route()` | Replace keyword match with vector similarity |
| `run_sequential()` → `run_hierarchical()` | Template: `AgentGraph::pipeline()` / `AgentGraph::supervisor()` | Auto-generate from mode |

## 5. Testing Strategy

- **Unit tests**: Each module in isolation (gate scoring, budget tracking, handoff routing)
- **Integration tests**: AgentGraph compile → run → verify step count + outputs
- **Pattern tests**: SwarmMode → Graph template → expected topology
- **Cost tests**: Budget halts after N calls
- **HITL tests**: Checkpoint at pause → human input → resume from checkpoint
- **Debate tests**: N rounds → converge → result contains synthesis

---

## 6. 补强：外部接地验证门（External-Grounded Gate）— 2026-08 增补

> 依据：Huang et al. (ICLR 2024) 证明内在自纠错无外部接地时**降级**；MAST 分析 1600+ 轨迹，79% 失败源于规格歧义(41.8%)+协调断裂(36.9%)，验证缺口仅 21.3%。现有 `QualityGate`（§2.3）是**文本质量门**（Completeness/Coherence/Safety），缺可执行证据。

### 6.1 问题
现有 gate 只做文本启发式判断，无法验证"代码真的能编译/测试通过"。对编码类子 agent 输出，必须用**可执行证据**接地。

### 6.2 设计：`GroundedGate`（外部接地验证门）

```rust
/// 外部接地验证门 — 用可执行证据（编译/测试/lint）验证子 agent 输出，
/// 而非仅文本启发式。对应 Huang 2024 的"外部接地"要求。
pub enum GroundedCheck {
    /// cargo check 编译通过（Rust 子任务）
    Compile { crate_name: String },
    /// cargo test 指定测试通过
    Test { crate_name: String, filter: String },
    /// 静态 lint（clippy / 自定义）
    Lint { tool: String, args: Vec<String> },
    /// 检索/工具输出对比（非代码任务）
    ToolOutput { expected: String },
}

pub struct GroundedGate {
    pub name: String,
    pub checks: Vec<GroundedCheck>,
    pub max_retries: u8,          // 失败后允许 worker 重试次数
    pub timeout_secs: u64,       // 沙箱超时
}

pub enum GroundedDecision {
    Pass,
    Revise { feedback: Vec<String> },  // 附编译错误/测试失败详情
    Fail { reason: String },           // 超时/重试耗尽 → 升级
}
```

**执行语义**：`AgentOutput → GroundedGate → [Pass → next] | [Revise → worker (max_retries)] | [Fail → escalation]`
与 §2.3 的 `QualityGate` 组合：**先文本门（快）→ 后接地门（准）**，控制成本。

### 6.3 三边界门控（文献洞见）
质量门应放在三个边界，跳过中间推理步的逐行判断以控成本：
1. **用户输出前** — 最终交付物验证
2. **不可逆工具执行前** — 写文件/发消息/部署前
3. **持久内存写入前** — KB/经验落盘前

### 6.4 复用现有资产
- 本地 SEAL 已有 `SelfReviewGate`（静态审查：panic/死代码/API 文档）— 作为**接地门的补充**（静态 + 动态）
- 本地 `cargo check`/`cargo test` 双验证纪律（R-P9/R-P16）即接地门的手动形态

---

## 7. 补强：类型化契约（Typed Contract）— 2026-08 外部修订

> 依据：MetaGPT (ICLR 2024) 核心洞见是"结构化工件 + 数据契约 + 可执行反馈"（+15.6% 成功率），
  而非"更多角色"。现有路由是 `Coordinator::route_task()` 关键词匹配（§4 迁移表），缺契约层。

### 7.1 设计：`AgentContract`（子 agent 契约）

```rust
/// 子 agent 类型化契约 — 定义输入/输出 schema 与成功标准。
/// 让编排器从"聊天式委托"升级为"契约式委托"（MetaGPT 模式）。
pub struct AgentContract {
    pub domain: Domain,              // NT-WORLD / NT-ACT / ...
    pub input_schema: Vec<Field>,    // 期望输入字段
    pub output_schema: Vec<Field>,   // 承诺输出字段
    pub success_criteria: Vec<GroundedCheck>, // 成功标准（可执行）
    pub upstream_watch: Vec<String>, // 上游工件 watch list
}

pub struct Field {
    pub name: String,
    pub ty: FieldType,               // String / Number / Json / File
    pub required: bool,
}
```

**委托协议**：`orchestrator 定义契约 → 子 agent 按契约产出 → GroundedGate 按 success_criteria 验证 → 采纳/重做`

### 7.2 委托原则（对齐外部洞见）
- **默认单 agent**：仅在 ①并行化 ②上下文保护 ③自主编排 三情形委托（arXiv:2604.02460）
- **委托收集、保留决策**：子 agent 只产出工件，综合/决策永远在 orchestrator
- **深度限制**：`max_spawn_depth`（默认 1=扁平），防失控递归（Hermes 模式）
- **预算门**：每委托设 token/时间预算，超限即停（对齐 §2.4 budget）

---

## 8. 单 agent 默认原则（2026-08 外部修订）

> 证据：单 agent 在等 token 预算下反超多 agent（arXiv:2604.02460）；Cognition "Don't Build Multi-Agents"；
  MAS 收益随模型能力递减。**多 agent 是"并行广度换 15× token + 14 种失败模式"的权衡，非默认答案。**

- **默认路径**：单 agent 直接执行（当前 `AgentTeam::execute()` 语义）
- **委托触发**：仅当任务可并行 / 需隔离上下文 / 需自主编排时，才走 `AgentGraph` 多 agent 路径
- **GWT 定位**：只当**注意力路由隐喻**（广播+瓶颈），勿作为"意识"功能投资（Theater of Mind 被审稿人批无实证；Anthropic 发现工作空间自发涌现）

---

## 9. 落地优先级（2026-08）

| 优先级 | 动作 | 依赖 |
|---|---|---|
| **P0** | 实现 `GroundedGate`（外部接地验证门）— 复用 cargo check/test | 无 |
| **P0** | 定义 `AgentContract` 类型化契约 + 委托协议 | 无 |
| **P1** | 实现 `nt_cap_orch_graph` 落地（现有设计 §2.1 是纸面蓝图，零实现） | P0 |
| **P1** | 委托加 `max_spawn_depth` + 预算门 | P0 |
| **P2** | 建委托基准（DecisionBench 式）+ MAST 分类学审计 | P1 |

> 注：现有 `nt_cap_orch_*` 8 模块（§2.1-2.8）**全部未实现**（2026-08 核实 find 无结果），
  本补强文档与 §2 设计互补，落地时统一实现。
