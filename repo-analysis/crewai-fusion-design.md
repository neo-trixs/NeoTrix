# CrewAI ↔ NeoTrix Orchestrator Fusion Design

**Date**: 2026-05-28
**Source**: CrewAI v1.14.6a2 (52K⭐) — https://github.com/crewAIInc/crewAI
**Target**: NeoTrix `neotrix-core/src/neotrix/orchestrator/` (7 files)
**Author**: Agent analysis

---

## 1. Project Overview — CrewAI

CrewAI is a standalone Python framework (0 LangChain dependency) for orchestrating role-playing, autonomous AI agents. It exposes two complementary programming models:

| Layer | Abstraction | Purpose |
|-------|------------|---------|
| **Crews** | `Agent` + `Task` + `Crew` | Autonomous role-based teams with sequential/hierarchical collaboration |
| **Flows** | `Flow[State]` + `@start`/`@listen`/`@router` | Event-driven, production-grade workflow orchestration with state persistence |

### Core Types (from docs + source)

**Agent**: `role`, `goal`, `backstory`, `tools`, `llm`, `memory`, `knowledge_sources`, `allow_delegation`, `reasoning`, `max_iter`, `max_rpm`, `max_execution_time`, `respect_context_window`, `code_execution_mode`, `multimodal`, `system_template`, `prompt_template`, `response_template`

**Task**: `description`, `expected_output`, `agent`, `tools` (override), `context` (other tasks), `output_pydantic`, `output_json`, `async_execution`, `human_input`, `guardrail`/`guardrails`, `callback`, `output_file`, `create_directory`

**Crew**: `agents`, `tasks`, `process` (`Process.sequential` / `Process.hierarchical`), `verbose`, `memory`, `knowledge_sources`, `share_crew`, `function_calling_llm`

**Flow**: `Flow[State]`, `@start()`, `@listen(method)`, `@router(method)`, `@persist`, `@human_feedback`, `or_()`/`and_()`, `kickoff()`, `plot()`, `state.id` (auto UUID), `remember()`/`recall()`/`extract_memories()`

---

## 2. NeoTrix Orchestrator — Current Architecture

### File Layout (7 files)

| File | Role | Key Types |
|------|------|-----------|
| `mod.rs` | Entry point | `Orchestrator` struct (planner + worker + critic + graph + engine) |
| `types.rs` | Enums/structs | `NodeType`, `LatentState` |
| `planner.rs` | Task decomposition | `PlannerNode` — `decompose()`, cross-repo enrichment via `GroupManager` |
| `worker.rs` | Execution | `WorkerNode` — `execute_tasks()` via shell commands, parallel |
| `critic.rs` | Verification | `CriticNode` — HP@K protocol, cross-validation, `AbsorbValidator` |
| `state_graph.rs` | DAG state machine | `StateGraph` — topological sort, ready/mark_done, Kahn's algorithm |
| `group_integration_test.rs` | Tests | 5 integration tests for cross-repo planning |

### Execution Flow

```
Orchestrator::run_recursive_loop(goal)
  ├─ 1. PlannerNode::decompose(goal) → Vec<Task>
  ├─ 2. StateGraph::build_plan(goal, n)  (Proposal → Spec → Tasks → Review)
  ├─ 3. AutonomyLevel gating (Proposal/Bounded/Full)
  ├─ 4. DAG-driven loop:
  │      for ready_nodes:
  │        WorkerNode::execute_tasks()
  │        AgentTeam integration (if available)
  │        StateGraph::mark_done()
  ├─ 5. CriticNode::heavy_pass_verify() → HP@K score
  └─ 6. If score < 0.6: ReasoningEngine::reason() + self_iterate()
```

### Unique Strengths (Protect)

- **HP@K verification** (arXiv:2605.02396) — heavy-pass weighted top-k with diversity bonus
- **DAG state graph** with topological sort, ready-node detection, dependency cascade
- **Capability vector** integration — 22-dim `CapabilityVector` drives execution decisions
- **Cross-repo contract matching** — `GroupManager` enriches task descriptions with cross-repo references
- **Autonomy levels** — Proposal (dry-run) / Bounded (capability gate) / Full
- **ReasoningEngine** — LLM-based reasoning within the orchestrator loop
- **Cross-validation** — capability vector internal consistency checks
- **Perspective bias** — simulated independent evaluation between Worker and Critic
- **SEAL self-iteration** — `absorb()` + `self_iterate()` for continuous improvement

---

## 3. Core Abstraction Comparison

| Dimension | CrewAI | NeoTrix Orchestrator | Gap |
|-----------|--------|---------------------|-----|
| **Agent model** | Role+Goal+Backstory+Tools+LLM | `AgentTeam` (opaque arc from `agent/team.rs`) | CrewAI has rich declarative agent spec |
| **Task model** | `Task(description, expected_output, agent, tools, context, output_pydantic, ...)` | `Task { id, input: Vec<f64>, priority }` | NeoTrix `Task` is a low-level data vector, no semantic description |
| **Process topology** | `Process.sequential` / `Process.hierarchical` | Custom DAG via `StateGraph::build_plan()` | CrewAI has named, reusable process patterns |
| **Workflow model** | Flows: `@start`/`@listen`/`@router` event-driven | `run_recursive_loop()` single monolithic DAG | NeoTrix lacks event-driven flow composition |
| **State management** | Typed `Flow[State]` with Pydantic, auto UUID, persistence | `LatentState { summary, task_state, confidence }` | NeoTrix state is minimal, untyped, no persistence |
| **Config format** | YAML (`agents.yaml`, `tasks.yaml`) | Programmatic Rust | CrewAI has cleaner separation of config vs logic |
| **Structured output** | `output_pydantic`, `output_json` (Pydantic model) | Raw shell stdout/stderr | NeoTrix has no structured output contract |
| **Guardrails** | Per-task `guardrail` (fn or LLM), `guardrails` list | None | Missing entirely |
| **Human-in-loop** | `@human_feedback`, `human_input=True` | None | Missing entirely |
| **Async execution** | `async_execution=True` on tasks | All parallel via `ExecMode::Parallel` | Different model — NeoTrix uses shell parallelism |
| **Task callbacks** | `callback(output)` after each task | `feedback_callback` at orchestrator level | NeoTrix has only end-to-end callback |
| **Memory** | Agent memory + Flow `remember()`/`recall()` | None | Missing entirely |
| **Persistence** | `@persist` with SQLite backend | None | Missing entirely |
| **Flow visualization** | `flow.plot("name")` → interactive HTML | `graph.summary()` → text | NeoTrix has only text summary |
| **Verification** | Task guardrails | HP@K protocol, cross-validation | CrewAI's guardrails ≠ NeoTrix's HP@K (complementary) |
| **Cross-repo context** | Tools (SerperDevTool, etc.) | `GroupManager::match_cross_repo()` | NeoTrix has unique multi-repo contract matching |
| **Capability vectors** | None | 22-dim `CapabilityVector` with `normalize()` | NeoTrix unique |
| **Self-iteration** | None | SEAL loop: `generate_self_edit()` → `absorb()` | NeoTrix unique |
| **Autonomy levels** | None (always execute) | Proposal / Bounded / Full | NeoTrix unique |

---

## 4. Gap Matrix

### 4.1 What CrewAI Has That NeoTrix Doesn't (Absorption Candidates)

| # | Feature | CrewAI Detail | NeoTrix Impact | Priority |
|---|---------|---------------|----------------|----------|
| G1 | **Role-based Agent spec** | `Agent(role, goal, backstory, tools, llm, ...)` | Enables rich agent personality, specialization, tool binding | **P0** |
| G2 | **Semantic Task model** | `Task(description, expected_output, agent, context, tools)` | Transforms `Task` from `(id, Vec<f64>, i32)` to meaningful spec | **P0** |
| G3 | **Sequential/Hierarchical process** | `Process.sequential` / `Process.hierarchical` | Named process patterns (manager-agent for hierarchical) | **P0** |
| G4 | **Event-driven Flows** | `@start`/`@listen`/`@router` + `or_()`/`and_()` | Enables multi-crew event-driven composition | **P0** |
| G5 | **Typed state management** | `Flow[PydanticState]` with auto UUID | Clean state contracts between workflow steps | **P0** |
| G6 | **YAML declarative config** | `agents.yaml`, `tasks.yaml` with template vars | Separate config from code | **P1** |
| G7 | **Structured output** | `output_pydantic`, `output_json` | Type-safe task output contracts | **P1** |
| G8 | **Task guardrails** | Function/LLM-based validation per task | Output quality gate before next task | **P1** |
| G9 | **Flow persistence** | `@persist` → SQLite backend | Resume/restart long-running orchestrations | **P1** |
| G10 | **Human-in-the-loop** | `@human_feedback`, `human_input=True` | Approval gates, manual review checkpoints | **P1** |
| G11 | **Flow-level memory** | `remember()`/`recall()`/`extract_memories()` | Knowledge accumulation across runs | **P2** |
| G12 | **Task callbacks** | Per-task `callback(output)` | Fine-grained progress monitoring | **P2** |
| G13 | **Flow visualization** | `flow.plot("name")` → HTML | Debugging/communication aid | **P2** |
| G14 | **Template variables** | `{topic}` in YAML, resolved via `kickoff(inputs=...)` | Dynamic task parameters | **P2** |
| G15 | **Agent delegation** | `allow_delegation=True` | Agents can sub-delegate to peers | **P2** |
| G16 | **Context window mgmt** | `respect_context_window` with auto-summary | Graceful handling of token limits | **P2** |

### 4.2 What NeoTrix Has That CrewAI Doesn't (Differentiators — Protect)

| # | Feature | NeoTrix Location | Value |
|---|---------|------------------|-------|
| N1 | **HP@K verification** | `critic.rs:22` — heavy-pass weighted top-k | arXiv-grounded multi-trace evaluation |
| N2 | **DAG state graph** | `state_graph.rs` — topological sort, ready/mark_done | Reproducible artifact dependency tracking |
| N3 | **CapabilityVector 22-dim** | `reasoning_brain/core.rs` | Neuro-symbolic competence representation |
| N4 | **Cross-repo contracts** | `planner.rs:35` — `GroupManager::match_cross_repo()` | Multi-repo context enrichment |
| N5 | **Autonomy levels** | `mod.rs:95` — Proposal/Bounded/Full | Safety-gated execution modes |
| N6 | **SEAL self-iteration** | `self_iterating.rs` — `absorb()` + `generate_self_edit()` | Continuous capability evolution |
| N7 | **Cross-validation** | `critic.rs:127` — capability consistency check | Detects capability vector pathologies |
| N8 | **Perspective bias** | `critic.rs:65` — simulated independence | Reduces hallucination in evaluation |
| N9 | **ReasoningEngine integration** | `mod.rs:117` — `eng.reason_task()` + `eng.reason()` | LLM reasoning within orchestration loop |
| N10 | **Shell command execution** | `worker.rs:27` — `ParallelExecutor::execute_shell()` | Can run any OS command as task |

---

## 5. Priority Classification & Implementation Plan

### P0 — Immediate (Core Orchestration Upgrade)

| Gap | Effort | Key Files | Dependencies |
|-----|--------|-----------|-------------|
| G1 Role-based Agent | 3d | New: `orchestrator/agent_spec.rs`, Mod: `orchestrator/types.rs` | None |
| G2 Semantic Task | 2d | New: `orchestrator/task_spec.rs`, Mod: `orchestrator/types.rs`, `planner.rs` | None |
| G3 Process patterns | 2d | New: `orchestrator/process.rs`, Mod: `orchestrator/mod.rs` | G1, G2 |
| G4 Event-driven Flows | 5d | New: `orchestrator/flow.rs`, Mod: `orchestrator/mod.rs` | G5 |
| G5 Typed state | 2d | New: `orchestrator/flow_state.rs`, Mod: `types.rs` | None |

**Total P0**: ~14d — delivers the core CrewAI-equivalent orchestration model on top of NeoTrix's DAG + HP@K foundation.

### P1 — Quality & Integration

| Gap | Effort | Key Files | Dependencies |
|-----|--------|-----------|-------------|
| G6 YAML config | 2d | New: `orchestrator/yaml_config.rs` | G1, G2 |
| G7 Structured output | 2d | Mod: `orchestrator/types.rs`, `worker.rs` | G2 |
| G8 Task guardrails | 3d | New: `orchestrator/guardrail.rs` | G2 |
| G9 Flow persistence | 3d | New: `orchestrator/persistence.rs` | G5 |
| G10 Human-in-loop | 3d | New: `orchestrator/human_feedback.rs` | G2 |

**Total P1**: ~13d

### P2 — Polish & Advanced Features

| Gap | Effort | Key Files | Dependencies |
|-----|--------|-----------|-------------|
| G11 Memory | 3d | New: `orchestrator/flow_memory.rs` | G5 |
| G12 Callbacks | 1d | Mod: `orchestrator/mod.rs` | G2 |
| G13 Visualization | 2d | Mod: `state_graph.rs` (export to Mermaid/DOT) | None |
| G14 Templates | 1d | Mod: `planner.rs` | G2 |
| G15 Delegation | 2d | Mod: `worker.rs` | G1 |

**Total P2**: ~9d

---

## 6. Integration Points

For each gap feature, the specific NeoTrix file(s) that require changes:

### G1 — Role-based Agent Spec

```
Modify:  orchestrator/types.rs
         Add AgentSpec struct { role, goal, backstory, tools, llm_config }
Create: orchestrator/agent_spec.rs
         AgentSpecBuilder, from_yaml(), from_code()
Modify: orchestrator/mod.rs
         Orchestrator gets agents: Vec<AgentSpec>, agent_lookup: HashMap<String, usize>
Integration: orchestrator/planner.rs
         Plan nodes reference AgentSpec by name
Integration: orchestrator/worker.rs
         WorkerNode selects agent based on AgentSpec
Integration: agent/team.rs (cross-module)
         AgentSpec → AgentTeam member mapping
```

### G2 — Semantic Task Model

```
Create: orchestrator/task_spec.rs
         TaskSpec struct { id, description, expected_output, agent_ref, tools, context_refs, output_format }
Modify: orchestrator/types.rs
         NodeType gets Analysis/Generation/Review variants mapped from TaskSpec
Modify: orchestrator/planner.rs
         decompose() returns Vec<TaskSpec> instead of Vec<Task> from parallel::types
Integration: parallel/types.rs
         Task gets optional task_spec: Option<TaskSpec> field
```

### G3 — Sequential/Hierarchical Process

```
Create: orchestrator/process.rs
         enum ProcessType { Sequential, Hierarchical, CustomDag }
         trait ProcessDefinition { fn build_graph(&self, tasks, agents) → StateGraph }
         SequentialProcess: linear chain
         HierarchicalProcess: manager-agent split with delegation
Modify: orchestrator/mod.rs
         Orchestrator has process: ProcessType + process definition
         run_recursive_loop() dispatches to process.build_graph()
```

### G4 — Event-driven Flows

```
Create: orchestrator/flow.rs
         FlowState<T: Serialize> { id: Uuid, data: T, metadata: HashMap }
         FlowNode { method_name, kind: Start | Listen(target) | Router(target) }
         FlowGraph { nodes, edges } — wraps StateGraph with event semantics
         enum FlowTrigger { or_(Vec<FlowTrigger>), and_(Vec<FlowTrigger>), method(String) }
Modify: orchestrator/mod.rs
         Orchestrator can run in Flow mode or Crew mode
         run_flow() → event-driven execution loop
         run_recursive_loop() → preserved as crew mode
```

### G5 — Typed State Management

```
Create: orchestrator/flow_state.rs
         trait ConfigState: Serialize + Deserialize + Clone
         FlowStateId: newtype over Uuid
         StateManager { current: Box<dyn ConfigState>, history: Vec, id: FlowStateId }
Modify: orchestrator/types.rs
         LatentState extended with state_id: Uuid
```

### G6 — YAML Config

```
Create: orchestrator/yaml_config.rs
         CrewConfig { agents: Vec<AgentYaml>, tasks: Vec<TaskYaml> }
         parser using serde_yaml
         template_variable resolution ({topic} → value)
Integration: reasoning_brain/ (new KnowledgeSource)
         KnowledgeSource::YamlConfig with capability_vector
```

### G7 — Structured Output

```
Modify: orchestrator/worker.rs
         WorkerNode gets output_format: OutputFormat enum
         execute_task_with_format() — wraps shell output in typed struct
Modify: orchestrator/task_spec.rs
         output_pydantic: Option<TypeId>, output_json: Option<TypeId>
```

### G8 — Task Guardrails

```
Create: orchestrator/guardrail.rs
         trait TaskGuardrail: Send + Sync { fn validate(&self, output) → Result }
         FnGuardrail(Box<dyn Fn(TaskOutput) → Result>)
         LlmGuardrail { criteria: String, llm: String }
         GuardrailChain { guardrails: Vec<Box<dyn TaskGuardrail>> }
Modify: orchestrator/worker.rs
         run_guardrails() between task execution and mark_done
```

---

## 7. KnowledgeSource Registration Scheme

Each new abstraction creates a KnowledgeSource for the ReasoningBrain:

| Feature | KnowledgeSource | CapabilityVector Impact | Registration Point |
|---------|----------------|------------------------|-------------------|
| G1 AgentSpec | `RoleBasedAgent` | `planning: +0.15`, `delegation: +0.2` | `reasoning_brain/self_iterating.rs:absorb()` |
| G2 TaskSpec | `SemanticTask` | `analysis: +0.12`, `code_generation: +0.1` | Same |
| G3 Process | `SequentialHierarchical` | `planning: +0.18`, `delegation: +0.15` | Same |
| G4 Flows | `EventDrivenFlow` | `planning: +0.2`, `verification: +0.1` | Same |
| G5 State | `TypedState` | `analysis: +0.08` | Same |
| G6 YAML | `YamlConfig` | `planning: +0.1` | Same |
| G7 StructOutput | `StructuredOutput` | `verification: +0.15` | Same |
| G8 Guardrails | `TaskGuardrail` | `verification: +0.2`, `quality_gates: +0.18` | Same |

### Seed Knowledge (ReasoningBank pre-inject)

```rust
// 5 initial memories for RoleBasedAgent
vec![
    ReasoningMemory {
        trace: "Agent with role='Senior Researcher', goal='Analyze papers', backstory='expert in ML'...",
        vector: cap_vec![planning: 0.85, analysis: 0.9, delegation: 0.3],
        task_type: TaskType::CodeAnalysis,
    },
    ReasoningMemory {
        trace: "Hierarchical process with manager delegating to 3 specialist agents...",
        vector: cap_vec![planning: 0.9, delegation: 0.88, verification: 0.7],
        task_type: TaskType::CodeGeneration,
    },
    ReasoningMemory {
        trace: "Flow with @start → @listen → @router: conditional branching on confidence > 0.8...",
        vector: cap_vec![planning: 0.92, analysis: 0.8, verification: 0.75],
        task_type: TaskType::Design,
    },
    // ...
]
```

---

## 8. Test Strategy

### Unit Tests (per module)

| Module | Test Cases | Coverage Target |
|--------|-----------|-----------------|
| `agent_spec.rs` | `test_agent_from_yaml`, `test_agent_spec_validation`, `test_tool_binding` | 90% |
| `task_spec.rs` | `test_task_output_pydantic`, `test_task_context_deps`, `test_task_async_flag` | 90% |
| `process.rs` | `test_sequential_execution`, `test_hierarchical_delegation`, `test_process_type_switch` | 85% |
| `flow.rs` | `test_start_listen_chain`, `test_router_conditional`, `test_or_trigger`, `test_and_trigger`, `test_flow_state_id` | 85% |
| `flow_state.rs` | `test_state_serialize`, `test_state_history`, `test_uuid_generation` | 95% |
| `guardrail.rs` | `test_fn_guardrail_pass`, `test_fn_guardrail_fail`, `test_guardrail_chain`, `test_retry_on_fail` | 90% |
| `persistence.rs` | `test_sqlite_save_load`, `test_flow_resume`, `test_restore_state_id` | 85% |
| `human_feedback.rs` | `test_human_feedback_emit`, `test_approval_gate`, `test_timeout` | 80% |

### Integration Tests

| Test | What It Validates |
|------|-------------------|
| `test_crew_sequential_full_flow` | AgentSpec → TaskSpec → SequentialProcess → WorkerNode → HP@K |
| `test_crew_hierarchical_with_delegation` | Manager agent delegates to subordinate agents |
| `test_flow_event_driven_chain` | @start → @listen → @listen chain executes in order |
| `test_flow_router_conditional_branch` | Router output determines which listener fires |
| `test_flow_persist_resume` | Run flow, stop, restore from SQLite, continue |
| `test_guardrail_rejects_bad_output` | Task output fails guardrail, triggers retry |
| `test_yaml_config_decomposition` | YAML agents.yaml + tasks.yaml → PlannerNode::decompose() |
| `test_cross_repo_flow_enrichment` | GroupManager enriches flow state with cross-repo context |
| `test_hp_at_k_verification_post_process` | After flow completion, CriticNode evaluates HP@K |

### Existing Test Integration

The 5 existing tests in `group_integration_test.rs` must be preserved and extended:

- `test_planner_enriches_cross_repo_tasks` → extend to test with new TaskSpec
- `test_orchestrator_builds_with_group_manager` → extend to test with Process topology
- Add: `test_planner_with_agent_spec_refs` (verify planner maps tasks to named agents)

### Compile & Feature Gates

```
cargo check --lib                          # baseline: 0 errors
cargo check --features full --lib          # gate: 0 errors
cargo test --lib orchestrator              # all orchestrator tests pass
cargo test --lib reasoning_brain           # existing brain tests unaffected
```

---

## 9. Architecture Decision Records

### ADR-1: Keep DAG StateGraph as Foundation, Layer Process on Top

The existing `StateGraph` with topological sort, ready-node detection, and dependency cascade is superior to CrewAI's flat sequential/hierarchical model for artifact tracking. Decision: `ProcessType` (G3) generates a `StateGraph`, not replaces it.

### ADR-2: Flows as Macro-Free DSL (No Procedural Macros)

CrewAI uses Python decorators (`@start`, `@listen`, `@router`). Rust cannot replicate this ergonomics without proc macros. Decision: Use a builder pattern instead:

```rust
// Instead of Python @start/@listen
Flow::define()
    .start("fetch_market_data", |ctx| { /* ... */ })
    .listen("fetch_market_data", "analyze_with_crew", |ctx| { /* ... */ })
    .router("analyze_with_crew", |ctx| {
        if ctx.state.confidence > 0.8 { "high_confidence" } else { "low_confidence" }
    })
    .listen(or_("medium_confidence", "low_confidence"), "request_analysis", |ctx| {})
    .build()
```

### ADR-3: YAML Config as Optional, Not Required

CrewAI strongly recommends YAML; NeoTrix should support YAML as an alternative to programmatic config, not a replacement. Decision: Add `yaml_config.rs` as a `feature-gated` module (`feature = "yaml_config"`).

### ADR-4: Structured Output via Existing Pydantic-like Mechanism

NeoTrix has no Pydantic equivalent. Decision: Use `serde::Serialize` + `typetype` for output contracts. Task output becomes `Option<Box<dyn Any + Send>>` with runtime type checking.

### ADR-5: HP@K as Post-Flow Validation (Not Per-Task Guardrail)

HP@K (G8) operates on multiple traces across the entire workflow; CrewAI guardrails operate on single task output. Decision: Both coexist — guardrails validate per-task, HP@K validates end-to-end.

---

## 10. Summary

| Category | Count | Key Takeaway |
|----------|-------|-------------|
| P0 gaps | 5 | Role-based agent + semantic task + process patterns + flows + typed state |
| P1 gaps | 5 | YAML config + structured output + guardrails + persistence + human-in-loop |
| P2 gaps | 6 | Memory + callbacks + visualization + templates + delegation + context |
| NeoTrix strengths | 10 | HP@K, DAG, CapabilityVector, cross-repo, autonomy, SEAL, cross-val, bias, engine, shell |
| New modules to create | 10 | `agent_spec`, `task_spec`, `process`, `flow`, `flow_state`, `yaml_config`, `guardrail`, `persistence`, `human_feedback`, `flow_memory` |
| Files to modify | 6 | `mod.rs`, `types.rs`, `planner.rs`, `worker.rs`, `state_graph.rs`, `agent/team.rs` |
| KnowledgeSources to add | 8 | One per absorbed feature |
| Tests to write | ~25 | 18 unit + 8 integration + extended existing |
| Total estimated effort | ~36d | 14 P0 + 13 P1 + 9 P2 |
