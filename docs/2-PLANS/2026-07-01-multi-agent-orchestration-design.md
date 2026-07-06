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
