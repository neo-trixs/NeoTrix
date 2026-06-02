# Claude Code Dynamic Workflows — Architecture Reference & NeoTrix Mapping

> **Date**: 2026-05-29 | **Status**: Research Preview (released 2026-05-28)
> **Required**: Claude Code v2.1.154+ | **Plans**: Max/Team (on by default), Enterprise (admin), Pro (manual config)
> **Source**: https://code.claude.com/docs/en/workflows + https://claude.com/blog/introducing-dynamic-workflows-in-claude-code

---

## 1. Core Architecture Overview

Claude Code Dynamic Workflows introduces a **script-runtime separation** architecture that fundamentally decouples orchestration from execution:

```
┌──────────────────────────────────────────────────────┐
│  User Prompt ("workflow" keyword)                     │
│         │                                             │
│         ▼                                             │
│  Claude writes a JavaScript orchestration script      │
│  (saved as .claude/workflows/<name>.js)               │
│         │                                             │
│         ▼                                             │
│  ┌─ Workflow Runtime ─────────────────────────────┐   │
│  │  • Isolated from Claude's context window        │   │
│  │  • NO direct filesystem/shell access            │   │
│  │  • Intermediate results in script variables     │   │
│  │  • Tracks per-agent state for checkpoint/resume │   │
│  │  • Max 16 concurrent / 1000 total agents        │   │
│  └─────────────────────────────────────────────────┘   │
│         │                                               │
│         ▼                                               │
│  Subagents (always in acceptEdits mode)                 │
│  → Results verified → Adversarial cross-check           │
│  → Convergence check → Final report                     │
└──────────────────────────────────────────────────────┘
```

### Key Architectural Properties

| Property | Implementation | Significance |
|----------|---------------|--------------|
| **Script as plan** | Claude generates JS orchestration script from NL | Plan is executable code, not implicit in conversation |
| **Context isolation** | Intermediate results in script variables, not Claude's context | Prevents context saturation at scale |
| **Resumability** | Runtime tracks per-agent state; cached on pause | Survives interruption within session |
| **Scale** | 16 concurrent / 1000 total subagents per run | Order-of-magnitude jump from manual subagent patterns |
| **Security** | Script has zero filesystem/shell; spawned agents handle all I/O | Clear trust boundary between orchestration and execution |
| **Repeatability** | Script saved as `/name` command in `.claude/workflows/` | Orchestration is version-controlled and shareable |

---

## 2. Pattern Analysis (7 Architectural Patterns)

### Pattern 1: Script-Runtime Separation

**Claude**: The orchestration script runs in an isolated runtime with NO filesystem/shell access. All I/O happens through spawned subagents. This creates a clean **orchestrator-executor separation**.

**Why it matters**: Prevents the orchestration layer from becoming an attack surface. The script can only coordinate, never execute. Also prevents context pollution — the conversation window stays clean.

### Pattern 2: Adversarial Verification Loop

**Claude**: Independent subagents solve from different angles, then cross-critique each other's findings. Only conclusions surviving refutation reach the user. The run iterates until answers converge.

**Why it matters**: Single-pass verification misses self-consistent errors. Having agents actively try to disprove each other's claims is strictly stronger than majority voting.

### Pattern 3: Checkpoint/Resume with Cached Results

**Claude**: The runtime tracks every agent's state. Paused/interrupted runs resume with completed agents returning cached results; only incomplete agents re-run. Does NOT survive session restart.

**Why it matters**: Essential for long-running workflows (hours-days). Without this, a single interruption wastes all prior work. Trade-off: no cross-session persistence = simpler state management.

### Pattern 4: Dynamic Fan-Out via Generated Script

**Claude**: Claude generates the JS orchestration script at runtime based on the natural language request. The number of agents, branching logic, and verification strategy are all dynamically determined.

**Why it matters**: Contrasts with static DAGs or pre-defined agent teams. The decomposition is discovered at runtime, not designed upfront. Only feasible because Claude itself writes the orchestration.

### Pattern 5: Phased Approval with Graduated Permission Modes

**Claude**: Shows planned phases before execution. Options: Yes / Yes don't ask again / View raw script / No. Auto mode skips after first launch. `claude -p` and Agent SDK bypass entirely.

**Why it matters**: Enables trust calibration. First run is explicit; subsequent runs are frictionless. Enterprise can disable entirely via managed settings.

### Pattern 6: Token Cost Transparency

**Claude**: Real-time token tracking in `/workflows` dashboard per phase. Can switch models for less critical phases. Explicit recommendations to start scoped.

**Why it matters**: Workflows burn 15x+ tokens vs single-agent. Without built-in cost visibility, users get bill shock. The model-switching per-phase is a novel cost optimization pattern.

### Pattern 7: Codified Orchestration as Repeatable Command

**Claude**: Workflow scripts saved to `.claude/workflows/` (project) or `~/.claude/workflows/` (global) become `/name` slash commands. Shareable via git.

**Why it matters**: Transforms ephemeral AI generation into reusable automation. Same pattern as shell scripts but generated by Claude. Lowers barrier to creating automation.

---

## 3. NeoTrix Mapping

| Pattern | NeoTrix Module | Current Maturity | Gap |
|---------|---------------|-----------------|-----|
| **P1: Script-Runtime Separation** | `orchestrator/planner.rs` decomposes goals into DAG, but run_recursive_loop interleaves plan + execution in same function | 🟡 Basic separation (PlannerNode/WorkerNode/CriticNode exist) | Planner shares engine context; no isolated runtime. `Flow`/`FlowRuntime` exist but are generic not workflow-native |
| **P2: Adversarial Verification** | `orchestrator/critic.rs` — HeavyPass@K protocol with `cross_validate()` | 🟡 Strong mathematical basis (HP@K/HM@K/Vote@K) | Single CriticNode, not multi-agent cross-critique. No independent adversarial agents refuting findings |
| **P3: Checkpoint/Resume** | `reasoning_brain/memory/pipeline.rs` — PipelineScheduler, but no agent-level checkpoint | 🔴 Missing | Pipeline memory exists (L1/L2/L3 extraction). No per-subagent state caching, no resume protocol |
| **P4: Dynamic Fan-Out** | `orchestrator/planner.rs::decompose()` — hardcoded task templates (plan_design/plan_code/plan_general) | 🔴 Static decomposition | Decomposition is 3-4 hardcoded Chinese strings per task type. No runtime script generation, no dynamic agent count |
| **P5: Phased Approval** | `orchestrator/mod.rs:101-121` — AutonomyLevel (Proposal/Bounded/Full) | 🟢 Good | AutonomyLevel system maps well. Proposal mode generates plan preview. Lacks per-phase granular approval |
| **P6: Token Cost Transparency** | None dedicated. `neotrix/telemetry.rs` exists but not workflow-aware | 🔴 Missing | No per-phase token tracking. No model-switching per sub-task. No `/workflows` dashboard equivalent |
| **P7: Codified Orchestration** | `cli/commands/mod.rs` + `agent/workflow.rs` — some workflow command infrastructure | 🟡 Partial | No equivalent to saving generated orchestration scripts as reusable commands. AgentWorkflow exists but is turn-based |

### Maturity Scale
- 🟢 **Good**: NeoTrix has working implementation, comparable to CC
- 🟡 **Basic**: NeoTrix has partial implementation, needs expansion
- 🟠 **Minimal**: Stub or placeholder exists only
- 🔴 **Missing**: No implementation exists

---

## 4. Gap Analysis

### What NeoTrix Has That CC Workflows Doesn't

| NeoTrix Feature | CC Workflows Equivalent | Advantage |
|-----------------|------------------------|-----------|
| **ReasoningEngine** with 4 reasoning types | CC has Claude's native reasoning | NeoTrix can run reasoning offline/with local models |
| **CapabilityVector** 22-dim + self-iteration | No equivalent | NeoTrix learns across sessions via vector evolution |
| **GoalLoop** with lifetime management | No goal tracking | CC workflows are stateless per-run |
| **Stagnation detection** + avoidance | No equivalent | NeoTrix monitors convergence health |
| **AgentProtocol** (UDP discovery + TCP) | No multi-machine | CC is single-machine only |
| **Background ticks** (goal_ticker, thinking_ticker) | No background scheduling | NeoTrix runs autonomously |
| **WAL + persistence** | No persistent state | CC workflows don't survive session exit |
| **VSA Hypercube + GWT** | No equivalent | NeoTrix has structured reasoning substrate |

### What CC Workflows Has That NeoTrix Doesn't

| CC Pattern | NeoTrix Gap | Impact |
|------------|------------|--------|
| **Script-runtime isolation** | Orchestrator shares engine memory | Security risk; context pollution |
| **Adversarial multi-reviewer** | Single CriticNode | Missing refutation-driven quality |
| **Checkpoint/resume per-agent** | No agent-level state caching | Long runs restart entirely on interrupt |
| **Generated JS orchestration** | Hardcoded decompose() templates | Can't handle novel task decompositions |
| **Token cost per-phase dashboard** | No cost visibility | Can't optimize compute allocation |
| **1000 agents / 16 concurrent** | ParallelExecutor capped at 4 workers | Scale ceiling |
| **Saved workflows as /commands** | No equivalent | Orchestrations are ephemeral |

### Not Applicable

- **AcceptEdits mode for subagents**: CC requires this because the runtime has no permissions. NeoTrix's AutonomyLevel + permission system is more granular.
- **JS as orchestration language**: NeoTrix is Rust-native. JS makes sense for CC ecosystem; Rust-native DSL or embedded Rhai/rhai would be NeoTrix's equivalent.
- **No cross-session resume**: CC's design choice (simplicity). NeoTrix's WAL + persistence could enable cross-session resume, which is strictly better.

---

## 5. Integration Recommendations

### R1: Checkpoint/Resume → pipeline.rs (P0)

**Target**: `neotrix-core/src/neotrix/reasoning_brain/memory/pipeline.rs`

Add per-agent checkpoint state alongside existing L1/L2/L3 pipeline:

```rust
// New: AgentCheckpoint alongside PipelineState
struct AgentCheckpoint {
    agent_id: String,
    phase: String,
    completed_subtasks: Vec<String>,
    partial_results: Vec<serde_json::Value>,
    cached_since: i64,
}

impl PipelineScheduler {
    // Save agent state at checkpoint boundaries
    pub async fn checkpoint_agent(&self, checkpoint: AgentCheckpoint) {
        // Store in WAL-compatible format
        // Key: agent_id + phase, Value: serialized partial state
    }

    // Resume: skip completed subtasks, restore partial context
    pub async fn resume_agent(&self, agent_id: &str) -> Option<AgentCheckpoint> {
        // Load from checkpoint store
        // Return cached result if fully completed
    }
}
```

**Implementation steps**:
1. Define `AgentCheckpoint` struct in `pipeline.rs`
2. Add `checkpoint_dir: PathBuf` to `PipelineScheduler`
3. Serialize/deserialize via `serde_json` (Rust-native, no protobuf overhead)
4. Integrate with `ParallelExecutor::execute_shell` — wrap each command with checkpoint save
5. Add `resume_mode` flag to `PipelineConfig`

### R2: Multi-Reviewer Adversarial Verification → orchestrator/critic.rs (P0)

**Target**: `neotrix-core/src/neotrix/orchestrator/critic.rs`

Extend CriticNode from single evaluator to multi-agent adversarial panel:

```rust
struct AdversarialPanel {
    reviewers: Vec<CriticNode>,        // Independent reviewer instances
    refuters: Vec<CriticNode>,         // Agents trying to disprove findings
    convergence_threshold: f64,        // Default: 0.8
    max_rounds: usize,                 // Default: 3
}

struct AdversarialResult {
    survived_claims: Vec<Claim>,
    refuted_claims: Vec<RefutedClaim>,
    convergence_score: f64,            // HP@K across iterations
}

impl AdversarialPanel {
    fn new(reviewer_count: usize, refuter_count: usize) -> Self;
    fn run_verification(&self, findings: &[Claim]) -> AdversarialResult;
    async fn spawn_refutation_agents(&self, findings: &[Claim]) -> Vec<Refutation>;
}
```

**Implementation steps**:
1. Define `Claim` and `RefutedClaim` structs
2. Create `AdversarialPanel` with configurable reviewer/refuter counts
3. Seeds: each reviewer gets different perspective_bias (0.05–0.3)
4. Refuters get negative bias + strict_mode = true
5. integrate with `Orchestrator::run_recursive_loop()` after DAG execution
6. Gate behind `AutonomyLevel::Full` (skip for Bounded)

### R3: Script-Based Orchestration → orchestrator/planner.rs (P1)

**Target**: `neotrix-core/src/neotrix/orchestrator/planner.rs`

Replace hardcoded `plan_design()`/`plan_code()`/`plan_general()` with a generated orchestration approach:

```rust
// New: OrchestrationScript — a serializable DAG with agent specs
struct OrchestrationScript {
    phases: Vec<Phase>,
    agent_counts: HashMap<String, usize>,
    verification_schedule: Vec<VerificationStep>,
}

impl PlannerNode {
    // Dynamic decomposition: use LLM (if engine available) or fallback to templates
    fn generate_script(&self, goal: &str, engine: Option<&ReasoningEngine>) -> OrchestrationScript {
        if let Some(eng) = engine {
            // LLM generates the decomposition dynamically
            let plan = eng.reason_task(goal).unwrap_or_default();
            self.parse_script_from_llm_output(&plan)
        } else {
            // Fallback to existing templates
            self.fallback_decompose(goal)
        }
    }
}
```

**Implementation steps**:
1. Define `Phase` struct with `agent_count`, `model_hint` (for cost optimization)
2. Add `generate_script()` method with LLM-driven decomposition
3. Add `fallback_decompose()` preserving existing hardcoded patterns
4. Integrate with `ReasoningEngine.reason_task()` for LLM decomposition
5. Cache generated scripts as files in `<project>/.neotrix/scripts/`
6. Add `save_script_as_command()` for repeatable orchestrations (P2)

### R4: Token Cost Dashboard + Phase-Level Model Routing (P2)

**Target**: `neotrix-core/src/neotrix/telemetry.rs`

```rust
struct WorkflowCostTracker {
    phase_costs: Vec<PhaseCost>,
    total_tokens: u64,
    estimated_usd: f64,
}

struct PhaseCost {
    phase_name: String,
    agent_count: usize,
    model: String,
    tokens_consumed: u64,
    duration: Duration,
}
```

### R5: Cross-Session Resume via WAL (P2)

**Target**: `neotrix-core/src/core/wal.rs`

CC workflows don't survive session restart. NeoTrix's existing WAL + `brain.json` persistence enables cross-session resume:

```rust
// In WAL, log every completed agent subtask
// On session restart, scan WAL for interrupted workflows
// Restore checkpoint state and resume incomplete agents
```

---

## 6. Priority Assessment

| Pattern | Priority | Rationale | Effort |
|---------|----------|-----------|--------|
| **R1: Checkpoint/Resume** | **P0** | Without it, long-running SEAL loops restart entirely on interrupt. Also unlocks cross-session resume via WAL | ~3 days |
| **R2: Adversarial Verification** | **P0** | Directly improves output quality for orchestrator. HeavyPass@K already exists as foundation. Multi-agent refutation is a clean extension | ~4 days |
| **R3: Script-Based Orchestration** | **P1** | Replacing hardcoded decompose() unlocks dynamic fan-out. Lower priority because templates work for most current use cases | ~5 days |
| **R5: Cross-Session Resume** | **P1** | Differentiation from CC (CC doesn't have it). Leverages existing WAL | ~2 days |
| **R4: Token Cost Dashboard** | **P2** | Nice-to-have for production. High value for enterprise adoption but no architectural dependency | ~3 days |
| **P7: Codified Orchestration** | **P2** | Reusable commands are a UX polish. Low effort once R3 is done | ~1 day |
| **P6: Script-Runtime Isolation** | **P2** | Security hardening. Current shared-engine approach is adequate for single-user | ~5 days |

### Implementation Order

```
Sprint 1 (P0): Checkpoint/Resume (R1) + Adversarial Verification (R2)
  → 7 days total, 5 files modified
  → Unlocks: reliable long runs + verifiable quality

Sprint 2 (P1): Script-Based Orchestration (R3) + Cross-Session Resume (R5)
  → 7 days total, 4 files modified
  → Unlocks: dynamic decomposition + survivable workflows

Sprint 3 (P2): Cost Dashboard (R4) + Codified Orchestration + Isolation
  → 9 days total, 6 files modified
  → Unlocks: production readiness
```

---

## 7. Risk Analysis

### Token Cost Risk

| Risk | Severity | Mitigation |
|------|----------|------------|
| Adversarial verification doubles token burn | High | Gate behind AutonomyLevel::Full; Bounded uses single CriticNode |
| Dynamic decomposition via LLM costs ~2K tokens per call | Medium | Cache generated scripts; reuse for same task type |
| Checkpoint serialization overhead | Low | Async WAL writes; configurable checkpoint frequency |

### Complexity Risk

| Risk | Severity | Mitigation |
|------|----------|------------|
| Checkpoint state machine diverges from actual execution | High | Use WAL as source of truth; replay from last consistent checkpoint |
| Adversarial agents enter infinite refutation loops | Medium | Convergence threshold + max_rounds hard limit |
| Script-based orchestration produces invalid DAGs | Medium | Validate graph before execution (cycle detection, reachability) |

### Architectural Coupling Risk

| Risk | Severity | Mitigation |
|------|----------|------------|
| Checkpoint/resume tightly couples to ParallelExecutor internals | Medium | Trait interface: `Checkpointable { fn save_state(&self); fn restore(id) -> Self }` |
| AdversarialPanel depends on CriticNode's perspective_bias API | Low | API already stable; extend via trait, not inheritance |
| Script-based planner may conflict with existing GroupManager | Low | GroupManager enhancement is additive, not modifying |

### Architectural Fit

```
CC Workflows (JS runtime)          NeoTrix (Rust native)
┌────────────────────┐            ┌──────────────────────────┐
│ Script coordinates  │            │ Orchestrator runs DAG     │
│ agents via IPC      │            │ + ReasonEngine reasoning  │
│                      │            │ + Capability self-iterate │
│ + scale (1000)      │     vs     │ + GoalLoop + WAL persist  │
│ + checkpoint        │            │ + stagnation detection    │
│ - no learning       │            │ - hardcoded decompose()   │
│ - no self-iteration │            │ - no checkpoint/resume    │
└────────────────────┘            └──────────────────────────┘
```

**Key insight**: CC Workflows is optimized for **horizontal scale** (many agents, single run). NeoTrix is optimized for **vertical intelligence** (self-iteration, capability evolution, memory). The recommended adoption strategy is **additive** — borrow CC's patterns (checkpoint, adversarial, dynamic planning) without copying its architecture (no JS runtime, no 1000-agent obsession). Focus on patterns that amplify NeoTrix's existing strengths.

---

## 8. Summary Decision Matrix

| Decision | Recommended | Rationale |
|----------|------------|-----------|
| Add JS orchestration runtime? | ❌ **No** | Rust-native DSL via `rhai` or scriptable DAG instead. JS adds unnecessary dependency + security surface |
| Match 1000-agent scale? | ❌ **No** | NeoTrix's value is intelligence density, not raw agent count. 16-32 concurrent with capability-based routing > 1000 dumb agents |
| Implement checkpoint/resume? | ✅ **Yes** (P0) | Directly unlocks reliable long-running SEAL loops. Low effort, high ROI |
| Implement adversarial verification? | ✅ **Yes** (P0) | Built on existing HP@K. Natural quality improvement for orchestrator output |
| Implement dynamic script generation? | ⏸️ **Defer** (P1) | Wait until R1+R2 stabilize. Then replace hardcoded decompose() with LLM-driven planning |
| Implement cost dashboard? | ⏸️ **Defer** (P2) | Valuable for adoption but no architectural dependency |

---

## 9. File Change Summary

| File | Change | Lines | Sprint |
|------|--------|-------|--------|
| `neotrix-core/src/neotrix/reasoning_brain/memory/pipeline.rs` | Add AgentCheckpoint + resume logic | ~120 | S1 |
| `neotrix-core/src/neotrix/orchestrator/critic.rs` | Add AdversarialPanel + multi-agent refutation | ~200 | S1 |
| `neotrix-core/src/neotrix/orchestrator/mod.rs` | Wire AdversarialPanel into run_recursive_loop | ~30 | S1 |
| `neotrix-core/src/neotrix/orchestrator/planner.rs` | Add generate_script() + LLM decomposition | ~150 | S2 |
| `neotrix-core/src/core/wal.rs` | Add workflow checkpoint records | ~80 | S2 |
| `neotrix-core/src/neotrix/telemetry.rs` | Add WorkflowCostTracker | ~100 | S3 |
| `neotrix-core/src/neotrix/cli/commands/mod.rs` | Add workflow save/load commands | ~60 | S3 |

---

## 10. References

- [CC Dynamic Workflows Docs](https://code.claude.com/docs/en/workflows)
- [CC Blog Announcement](https://claude.com/blog/introducing-dynamic-workflows-in-claude-code)
- [CC Dynamic Workflows Guide](https://claudefa.st/blog/guide/development/dynamic-workflows)
- [NeoTrix Orchestrator](neotrix-core/src/neotrix/orchestrator/mod.rs)
- [NeoTrix CriticNode](neotrix-core/src/neotrix/orchestrator/critic.rs)
- [NeoTrix PipelineScheduler](neotrix-core/src/neotrix/reasoning_brain/memory/pipeline.rs)
- [NeoTrix ParallelExecutor](neotrix-core/src/neotrix/parallel/executor.rs)
