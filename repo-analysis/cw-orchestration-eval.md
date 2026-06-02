# CW Dynamic Workflows — NeoTrix Orchestration Evaluation

> **Date**: 2026-05-29 | **Analyzed against**: NeoTrix orchestrator v1 (14 files, ~2100 lines)
> **Source**: cc-dynamic-workflows-reference.md + `neotrix-core/src/neotrix/orchestrator/`

---

## 1. Current NeoTrix Orchestrator Architecture

NeoTrix orchestration is **turn-based DAG execution** with three fixed nodes:

```
run_recursive_loop(goal)
  ├── PlannerNode.decompose(goal)     # 3 hardcoded templates (design/code/general), 3-4 tasks each
  ├── StateGraph.build_plan()          # Proposal → Spec → Tasks(N) → Review
  ├── Loop: ready_nodes() → execute → mark_done()
  │     ├── WorkerNode.execute_tasks()  # ParallelExecutor (4 workers)
  │     └── AgentTeam.execute()         # Optional multi-agent delegation
  └── CriticNode.heavy_pass_verify()   # HP@K scoring, single-pass evaluation
```

Key characteristics:
- **Hardcoded decomposition**: `planner.rs:61-85` — exactly 3 templates, each returning Chinese strings. No LLM-driven planning.
- **DAG execution**: `state_graph.rs:191-237` — Kahn's topological sort, ready-node polling, per-node state machine (Pending/Ready/InProgress/Done/Blocked).
- **Single CriticNode**: `critic.rs:63` — one evaluator with `perspective_bias` for independence. HP@K protocol but no multi-agent adversarial refutation.
- **AdversarialVerifier exists but is keyword-based**: `adversarial.rs:91-174` — pattern-matches keywords, not LLM-driven. 5 perspectives but no actual agent spawning.
- **Flow/FlowRuntime exists**: `flow.rs:92-140` — trigger-based step execution with `StateManager<S>`. Generic, not workflow-native, not integrated with Orchestrator.
- **No checkpoint/resume**: `run_recursive_loop()` runs start-to-finish. Interruption = full restart.
- **No token tracking**: `telemetry.rs` exists but is not workflow-aware.
- **No cross-session persistence**: Orchestrations are ephemeral; no script saving.

---

## 2. CW Script-Runtime Pattern (Summary)

CW's core innovation is **decoupling orchestration logic (JS script) from execution (subagents)**:

1. Claude generates a `.js` orchestration script from natural language
2. The script runs in an isolated runtime (no filesystem/shell access)
3. The script coordinates subagents via IPC — up to 16 concurrent, 1000 total
4. Subagents produce results → script aggregates → adversarial cross-check → convergence
5. Scripts saved as `/commands` in `.claude/workflows/` → reusable and version-controlled
6. Runtime tracks per-agent state for checkpoint/resume (in-session only)
7. Token cost tracked per-phase with model-switching

| Property | CW | NeoTrix |
|----------|----|---------|
| Plan format | Executable JS script | Implicit in `run_recursive_loop()` |
| Context isolation | Script has zero I/O | Orchestrator shares engine memory |
| Decomposition | LLM-driven, dynamic | 3 hardcoded templates |
| Scale | 16 concurrent / 1000 total | 4 workers (ParallelExecutor) |
| Resume | Per-agent state caching | None |
| Verification | Multi-agent adversarial | Single CriticNode + HP@K |
| Reusability | Saved as `/commands` | Ephemeral |
| Cost tracking | Per-phase dashboard | None |

---

## 3. Gap Analysis

### What NeoTrix Gains by Adopting CW Patterns

| Pattern | What It Unlocks | Risk |
|---------|----------------|------|
| **Generative decomposition** | Replace 3 hardcoded templates with LLM-driven task DAG generation. Adapts to novel task types. Parity with CW's dynamic fan-out. | LLM cost per decomposition (~2K tokens). Generated DAGs may have cycles/redundancy → validation needed. |
| **Multi-agent adversarial verification** | Move from HP@K single-pass to cross-refutation. Catches self-consistent errors that single-pass misses. `adversarial.rs` already has the structure (5 perspectives), just not wired into Orchestrator's loop. | 2-3x token cost vs single CriticNode. Infinite refutation loops → need `max_rounds` hard limit. |
| **Per-agent checkpoint/resume** | Long-running SEAL loops survive interruption. Essential for cross-session workflows. | State machine drift (checkpoint vs actual execution divergence). WAL overhead. |
| **Script-based orchestration** | DAG plans become serializable and replayable. Enables "save plan → review → execute" workflow. Aligns with NeoTrix's existing `Flow`/`FlowRuntime` concept. | Another abstraction layer. Forced reuse of stale plans. |
| **Phase-level cost tracking** | Visibility into which phase burns tokens. Enables model-switching per phase (cheap model for planning, expensive for execution). | Telemetry infra changes. |

### What NeoTrix Loses by Adopting CW Patterns

| If We Adopt... | We Lose... | Mitigation |
|---------------|-----------|------------|
| Full script-runtime isolation | Shared engine context (ReasoningEngine, AgentTeam, CapabilityVector) in Orchestrator | Keep shared context, but gate script I/O behind AutonomyLevel |
| JS-based orchestration | Rust-native type safety | Use `rhai`-based DSL or serializable DAG struct (already have `StateGraph`) |
| 1000-agent scale obsession | Current intelligence-density focus (capability vectors, self-iteration) | Don't chase 1000 agents. NeoTrix's value is quality per agent, not quantity |

### What NeoTrix Has That CW Doesn't (No Loss)

- **CapabilityVector + self-iteration** — CW has no cross-session learning
- **GoalLoop** with lifetime management — CW workflows are stateless per-run
- **WAL + persistence** — CW doesn't survive session exit; NeoTrix could do cross-session resume
- **ReasoningEngine** with 4 reasoning types — CW has Claude-only
- **AgentProtocol** (UDP discovery + TCP) — CW is single-machine
- **Background ticks** (goal_ticker, thinking_ticker) — CW has no background scheduling
- **VSA Hypercube + GWT** — CW has no structured reasoning substrate

### Verdict

**Net architectural advantage**: NeoTrix already has the right foundation (`Flow`, `AdversarialVerifier`, `StateGraph`, `AutonomyLevel`). The gaps are in *integration* and *generative depth*, not missing primitives.

---

## 4. Recommendation: Cherry-pick

**Full adoption is not justified.** A JS runtime, 1000-agent scale, and full context isolation are misaligned with NeoTrix's intelligence-density value proposition.

**Cherry-picking 3 patterns is justified** because they directly amplify existing infrastructure with moderate effort (~7 days total).

### 4.1 What to SKIP (not worth it)

| Pattern | Effort | Reason to Skip |
|---------|--------|---------------|
| JS runtime integration | ~5 days | Rhai/JS adds attack surface. NeoTrix already has `Flow` + `StateGraph` for DSL |
| 1000-agent scale | Large | NeoTrix differentiates on capability per agent, not raw count. 16-32 with capability routing > 1000 dumb agents |
| Full context isolation | ~5 days | Security hardening is P2-P3. Current shared-engine is adequate for single-user development |
| Cross-session resume via WAL | ~2 days | CW doesn't have this either. Differentiator but not urgent. Defer to P2 |

### 4.2 What to ADOPT (cherry-pick)

| # | Pattern | CW Source | NeoTrix Target | Effort |
|---|---------|-----------|----------------|--------|
| **C1** | **Generative decomposition** | CW's LLM-generated orchestration script | `planner.rs` — replace `match task_type` with `ReasoningEngine.reason_task()` | ~2 days |
| **C2** | **Wired adversarial panel** | CW's cross-refutation between agents | `critic.rs` + `adversarial.rs` — spawn 3-5 perspective agents per run, cross-validate findings | ~2 days |
| **C3** | **Script-saving** | CW's `.claude/workflows/` | `planner.rs` + `neotrix/cli/commands/` — serialize DAG plan as YAML, save/reload via CLI | ~1 day |

**Total cherry-pick effort: ~5 days** (vs ~17 days for full adoption).

### 4.3 Integration Points

#### C1: Generative Decomposition

```
planner.rs:26-34
  match task_type {                          →    fn generate_script(goal, engine?) -> Vec<Task>
    Design => plan_design()                       if engine: LLM generates task list dynamically
    Code => plan_code()                           else: fallback to existing templates (same as today)
    _ => plan_general()
  }
```

- Keep existing templates as fallback (no regression)
- Accept `&Option<ReasoningEngine>` — if None, use hardcoded paths
- Cache generated scripts in `~/.neotrix/scripts/<goal-hash>.yaml`
- No new dependencies needed (uses existing `ReasoningEngine.reason_task()`)

#### C2: Multi-Agent Adversarial Verification

```
critic.rs + adversarial.rs
  Current:                                   →    New:
    let scores = vec![critic.evaluate(...)]         let panel = AdversarialPanel::new(
    let hp_result = critic.heavy_pass_verify(           reviewers: 3, refuters: 2
      &scores                                       );
    );                                            let result = panel.run_verification(findings);
```

- `adversarial.rs` already has `Perspective` (5 variants), `AdversarialVerifier`, `consensus_report()`
- Gap: `verify()` is keyword-based, not LLM-driven → replace with `ReasoningEngine.reason_as_reviewer()` per perspective
- Wire into `run_recursive_loop()` after DAG execution (line 164-174)
- Gate behind `AutonomyLevel::Full` (skip for Bounded)

#### C3: Script Saving

```
planner.rs (new methods):
  fn save_plan(&self, plan: &[Task], goal: &str) -> PathBuf
  fn load_plan(&self, name: &str) -> Option<Vec<Task>>
```

- Serialize as YAML (serde already in deps)
- Save to `<project>/.neotrix/plans/` and `~/.neotrix/plans/`
- Add `neotrix plan save <name>` and `neotrix plan list` CLI commands

---

## 5. Implementation Decision Matrix

| Pattern | Adopt? | Effort | ROI | Dependency |
|---------|--------|--------|-----|------------|
| **Generative decomposition** | ✅ | 2 days | High (removes hardcoded ceiling) | None |
| **Multi-agent adversarial** | ✅ | 2 days | High (quality improvement on existing HP@K) | None |
| **Script saving** | ✅ | 1 day | Medium (UX polish, plan reuse) | C1 |
| Per-agent checkpoint | ⏸️ Defer | 3 days | Medium | WAL integration |
| Phase cost tracking | ⏸️ P2 | 3 days | Medium | Telemetry existing |
| Script-runtime isolation | ❌ Skip | 5 days | Low | — |
| 1000-agent scale | ❌ Skip | Large | Negative | — |

**Recommended sprint**: C1 + C2 (4 days) → cargo check → C3 (1 day) → cargo check + test.

### Why Not Wait for CW to Mature?

CW's patterns are already validated by production use. The script-runtime pattern is not novel (DAG-based orchestration is decades old). What CW contributes is: (1) LLM-generated decomposition, (2) adversarial multi-reviewer at scale. NeoTrix is structurally ready for both. Waiting would only widen the feature gap on something that takes ~5 days to close.
