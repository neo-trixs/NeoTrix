# GenericAgent → NeoTrix Fusion Design: Skill Crystallization

> **Source**: [lsdefine/GenericAgent](https://github.com/lsdefine/GenericAgent) — 12.2K⭐, ~3.3K lines Python seed code  
> **Target**: NeoTrix `SelfEvolver` + `ReasoningBrain` + `SelfIteratingBrain`  
> **Focus**: Skill auto-crystallization mechanism (GA's core innovation)  
> **Paper**: arXiv 2604.17091 — "Contextual Information Density Maximization"  
> **Date**: 2026-05-28

---

## 1. Project Overview: GenericAgent Skill Crystallization

GenericAgent's defining innovation is **don't preload skills — evolve them.** Every time it solves a task, the execution path is automatically crystallized into a reusable Skill (SOP + executable code), written to memory, and directly recalled on similar tasks.

### The Self-Evolution Pipeline

```
New Task → Autonomous Exploration (install deps, write scripts, debug & verify)
         → Crystallize Execution Path into Skill
         → Write to Memory Layer (L3: Task Skills/SOPs)
         → Direct Recall on Next Similar Task (token cost drops 89.6%)
```

### Key Architectural Components

| Component | Lines | Function |
|-----------|-------|----------|
| Agent Loop (`agent_loop.py`) | ~100 | Perceive → Reason → Execute → Memory → Loop |
| LLM Core (`llmcore.py`) | ~500 | Multi-model LLM client abstraction |
| 9 Atomic Tools | ~800 | `code_run`, `file_read`, `file_write`, `file_patch`, `web_scan`, `web_execute_js`, `ask_user`, `update_working_checkpoint`, `start_long_term_update` |
| Layered Memory | ~400 | L0 Meta Rules → L1 Insight Index → L2 Global Facts → L3 Skills/SOPs → L4 Session Archive |
| Skill Crystallization | ~200 | Extract tool-call trajectory → distill SOP → verify → persist |
| Goal Mode (`reflect/goal_mode.py`) | ~500 | Time-budget-driven self-driven loop |
| Morphling Mode (`memory/morphling_sop.md`) | ~300 | Project-level skill absorption from external repos |
| Conductor | ~150 | Sub-agent orchestration |
| Goal Hive | ~200 | Multi-worker cooperative goals |

### Context Information Density Maximization (Paper's Core Principle)

The paper argues long-horizon performance is determined not by context length but by **decision-relevant information within a finite budget**. Four mechanisms:
1. **Minimal atomic tool set** — keeps tool descriptions tiny
2. **Hierarchical on-demand memory** — small high-level view by default, drill-down on need
3. **Skill crystallization** — trajectories → SOPs + code (extreme compression)
4. **Context truncation/compression** — maintain density during long executions

**Benchmark results**: 100% completion rate with 222K input tokens (27.7% of Claude Code); 9-round GitHub task token consumption dropped 89.6%; calls converged from 32 to 5.

---

## 2. Core Abstraction Comparison Table

| Dimension | GenericAgent | NeoTrix SelfEvolver |
|-----------|-------------|---------------------|
| **Evolution trigger** | Every task execution end | External URL / manual `evolve_from_url()` |
| **Learning representation** | Executable Python script + SOP (text) | 23-dim CapabilityVector adjustments (MicroEdit) |
| **Memory structure** | 5-layer hierarchical (L0–L4) | Flat ReasoningBank + MemoryTier enum |
| **Skill format** | Full executable code + structured SOP doc | Vector weights only (no executable artifacts) |
| **Skill recall** | Semantic index → direct code execution | Cosine similarity on CapabilityVector → no code recall |
| **Atomic tool interface** | 9 fixed tools, 1 `code_run` escape hatch | MCP tools (external), no minimal core |
| **Context management** | Truncation + compression layer | None |
| **Self-bootstrap** | Full: agent built the entire repo | Partial: SelfEvolver only adjusts vectors |
| **External absorption** | Morphling mode (extract goal+tests, per-component decide) | ThreeStream analysis (code/docs/insights) |
| **Sub-agent orchestration** | Conductor (spawn/supervise/cleanup) | AgentTeam (different abstraction) |
| **Verification** | Execution-based (run the skill, check output) | Algebraic (SEAL) via seal_algebra.rs |
| **Learning rate decay** | Implicit via skill tree maturity | Fixed `learning_rate: 0.05` |
| **Token optimization** | Core design principle | Not addressed |
| **Capability vector** | None (relies on skill tree + layered memory) | 23-dim CapabilityVector + extension dimensions |
| **Knowledge sources** | Not applicable (no vector model) | Enum-based + HashMap custom_sources |
| **Metacognition** | Not present | Full metacognition system |
| **Multi-agent coordination** | Goal Hive + Conductor | Orchestrator + AgentTeam |

---

## 3. Gap Matrix

### What GenericAgent Has That NeoTrix Doesn't

| # | Feature | GA Mechanism | Impact | Priority |
|---|---------|-------------|--------|----------|
| G1 | **Skill crystallization from execution paths** | After each task, extract tool-call trajectory → distill SOP → verify → persist as executable skill | Foundational self-evolution. Without this, SelfEvolver is read-only (external URL analysis only) | **P0** |
| G2 | **Executable skill artifacts** | Skills are real Python code + structured SOP docs, not vector adjustments | Vector-only learning cannot produce runnable code. NeoTrix evolves capability estimates but not actual tools | **P0** |
| G3 | **Hierarchical on-demand memory (L0-L4)** | Meta Rules (L0) → Insight Index (L1) → Global Facts (L2) → Task Skills (L3) → Session Archive (L4). Only relevant layer loaded per context | NeoTrix ReasoningBank is flat; no compression/tiered-loading strategy. Context will bloat like Claude Code | **P0** |
| G4 | **Context truncation/compression** | Explicit layer that truncates low-density context, compresses tool descriptions | Without this, long-running loops hit context limits. NeoTrix has no equivalent mechanism | **P0** |
| G5 | **Autonomous task-driven exploration** | `code_run` + `file_write` + debug loop creates new tools autonomously | NeoTrix cannot autonomously write and install its own tools — it can only adjust existing vectors | **P1** |
| G6 | **Skill tree formation & convergence** | Skills accumulate into a tree; repeated tasks converge to 5 calls from 32 | NeoTrix has no skill-tree structure; repeated external analysis yields no cost reduction | **P1** |
| G7 | **Million-scale skill library** | Pre-built library of community skills (from Sophub + Datawhale) | NeoTrix seed memory injection is hand-crafted, not scalable | **P1** |
| G8 | **Morphling mode** | Extract goal + tests from external repo; per-component decide: call/rewrite/discard | NeoTrix ThreeStream analysis is read-only — it extracts insights but doesn't generate executable components | **P1** |
| G9 | **Goal mode (time-budget-driven)** | `reflect/goal_mode.py`: "keep optimizing X for N hours" with deadline-enforced quality | NeoTrix has GoalLoop but no time-budget-driven self-driving loop | **P2** |
| G10 | **Token efficiency as architecture principle** | Every design decision measured by token cost vs. task completion | NeoTrix has no token budget tracking or optimization anywhere in the loop | **P2** |

### What NeoTrix Has That GenericAgent Doesn't

| # | Feature | NeoTrix Mechanism | Value to GA |
|---|---------|-------------------|-------------|
| N1 | **CapabilityVector (23-dim + extension)** | Dense vector representation of agent capabilities across 23 fixed + N extension dimensions | Could give GA a similarity-based skill retrieval (vs. index-based) |
| N2 | **SEAL algebraic verification** | `seal_algebra.rs` — spectrum-radius-based edit coherence verification | Could verify skill convergence before committing |
| N3 | **Metacognition system** | Self-scanning + weakness analysis + evolution planning | Could give GA self-awareness of skill tree gaps |
| N4 | **KnowledgeSource enum + custom registration** | 12+ typed knowledge sources with explicit capability_vector() profiles | Could seed GA's L2 Global Facts with structured knowledge |
| N5 | **GWT attention routing** | Global Workspace Theory competitive salience selection | Could improve skill selection in Goal Hive mode |
| N6 | **Hypercube VSA knowledge representation** | 4096-dim hyperdimensional computing for bind/bundle/permute | Could give GA compositional skill operations |
| N7 | **ReasoningEngine with 4 reasoning types** | Conversation/TaskSolving/ErrorDebugging/KnowledgeQuery | Could make GA's reasoning traces distillable to capability vectors |
| N8 | **Cognitive map (15 LLM↔NeoTrix mappings)** | Bridges LLM behavior patterns to system internals | Could help GA reason about its own architecture |
| N9 | **Multi-model orchestration (Orchestrator + AgentTeam)** | PlannerNode/WorkerNode/CriticNode with debate/sequential patterns | Could give GA structured multi-agent with validation |
| N10 | **GoalLoop with RateLimiter + CircuitBreaker** | Production-grade goal management with load shedding | Could prevent GA from runaway execution loops |

---

## 4. Priority Classification & Implementation Plan

### P0 — Foundation (Must have before any other work)

```
G1 + G2 + G3 + G4: Skill Crystallization Pipeline + Layered Memory + Context Compression
                                                                        │
                                    ┌───────────────────────────────────┴───────────────────────────────────┐
                                    │                                                                      │
                              G1+G2: SkillCrystallizer                                              G3: LayeredMemory
                              self_iterating/skill_crystallizer.rs                                   core/layered_memory.rs
                                    │                                                                      │
                                    ▼                                                                      ▼
                            Capture task execution                                                L0: MetaRulesVec
                            → distill SOP + extract code                                         L1: InsightIndex
                            → verify (compile/run)                                                L2: GlobalFactStore
                            → persist as SkillArtifact                                            L3: SkillArchive
                                                                                                  L4: SessionVault
                                                                                                        │
                                    └───────────────────────────────────┬───────────────────────────────────┘
                                                                        ▼
                                                              G4: ContextCompressor
                                                              core/context_compressor.rs
                                                              Truncation + compression layer
                                                              for SelfEvolver + ReasoningBank
```

**Estimated work**: 3 files, ~800 lines total

### P1 — Core Enhancement

```
G5: AutonomousToolWriter           G6: SkillTree              G7: SkillLibrarySeed
self_iterating/                    self_iterating/             memory/
tool_writer.rs                     skill_tree.rs               skill_library.rs
  │                                  │                           │
  ▼                                  ▼                           ▼
Write + install self-               Tree structure              Bulk seed from GA's
generated tools via                  with convergence            published library
code_run equivalent                  detection                   (~100 skills)
```

### P2 — Quality of Life

```
G8: MorphlingBridge                    G9: GoalTimeBudget              G10: TokenTracker
reasoning_brain/                        goal_loop/                      core/
morphling_bridge.rs                     time_budget.rs                  token_tracker.rs
Per-component decision                  Time-budget-driven              Per-turn token budget
(call/rewrite/discard)                  self-driving loop               + compression decisions
```

---

## 5. Integration Points

For each gap feature, the exact NeoTrix files that need changes:

### P0 Integration Points

| Feature | NeoTrix Files to Modify/Add | Nature of Change |
|---------|---------------------------|------------------|
| **G1+G2: SkillCrystallizer** | **New**: `reasoning_brain/self_iterating/skill_crystallizer.rs` | Core new module |
| | **New**: `core/skill.rs` — `SkillArtifact` struct with `code: String`, `sop: String`, `tool_calls: Vec<ToolCall>`, `entry_point: String`, `language: SkillLang` | New data type |
| | **Modify**: `reasoning_brain/self_edit.rs` — add `SkillArtifact` to `MicroEdit::CrystallizeSkill(SkillArtifact)` | Extend MicroEdit enum |
| | **Modify**: `reasoning_brain/self_iterating/brain_impl.rs` — `ReasoningBrain` gains `skill_tree: SkillTree` field | New field on main brain struct |
| | **Modify**: `reasoning_brain/self_iterating/brain_impl.rs` — `absorb()` gains `crystallize_skill()` call at end | Hook into existing absorb path |
| | **Modify**: `reasoning_brain/self_iterating/persist_impl.rs` — serialize skills alongside brain.json | Persistence |
| **G3: LayeredMemory** | **New**: `core/layered_memory.rs` — `LayeredMemory<L0, L1, L2, L3, L4>` generic struct with `on_demand_load(layer: LayerId) -> &[Memory]` | New memory architecture |
| | **New**: `core/meta_rules.rs` — L0 MetaRules as typed constraint vector | L0 implementation |
| | **New**: `core/insight_index.rs` — L1 lightweight inverted index for fast routing | L1 implementation |
| | **Modify**: `reasoning_brain/memory.rs` — `ReasoningBank` wraps `LayeredMemory` instead of flat `Vec<ReasoningMemory>` | Refactor existing memory |
| | **Modify**: `reasoning_brain/self_iterating/brain_impl.rs` — all memory reads go through layered load | Replace all direct Vec reads |
| **G4: ContextCompressor** | **New**: `core/context_compressor.rs` — `Compressor::truncate(&mut messages)`, `Compressor::compress_tool_descs()`, `Compressor::compress_memory_layers()` | New utility module |
| | **Modify**: `reasoning_engine/reasoning_engine.rs` — add compression step before LLM call | Hook into existing engine |
| | **Modify**: `self_iterating/loop_impl/` — add `compressor.compress()` call at loop top | Hook into execution loop |

### P1 Integration Points

| Feature | NeoTrix Files to Modify/Add | Nature of Change |
|---------|---------------------------|------------------|
| **G5: AutonomousToolWriter** | **New**: `reasoning_brain/self_iterating/tool_writer.rs` — `write_tool_code()`, `install_deps()`, `verify_tool()` | New module |
| | **Modify**: `agent/tools/mod.rs` — `register_builtin()` gains dynamic registration path | Hook for runtime tool install |
| | **Modify**: `mcp_tools.rs` — add `install_dynamic_tool` MCP handler | Expose via MCP |
| **G6: SkillTree** | **New**: `core/skill_tree.rs` — `SkillTree { root: SkillNode, convergence_map: HashMap<String, ConvergenceRecord> }` | New data structure |
| | **Modify**: `reasoning_brain/self_iterating/brain_impl.rs` — `ReasoningBrain.skill_tree` field, `recall_skill()` method | Extend brain |
| **G7: SkillLibrarySeed** | **New**: `reasoning_brain/memory/skill_library.rs` — bulk seed from GA published library | New module |
| | **Modify**: `reasoning_brain/memory.rs` — `initialize_with_ga_skills()` callback | Extend initialization |

### P2 Integration Points

| Feature | NeoTrix Files to Modify/Add | Nature of Change |
|---------|---------------------------|------------------|
| **G8: MorphlingBridge** | **New**: `reasoning_brain/morphling_bridge.rs` — wraps existing SelfEvolver.analyze_three_streams() with per-component decision | Bridge module |
| | **Modify**: `reasoning_brain/self_evolver.rs` — add `morphling_absorb()` method | Extend SelfEvolver |
| **G9: GoalTimeBudget** | **New**: `goal_loop/time_budget.rs` — `TimeBudget { total_seconds, elapsed, hard_deadline }` | New module |
| | **Modify**: `reasoning_brain/goal_loop.rs` — GoalState gains `time_budget: Option<TimeBudget>` | Extend GoalState |
| **G10: TokenTracker** | **New**: `core/token_tracker.rs` — `TokenBudget { per_turn_limit, total_limit, spent }` | New module |
| | **Modify**: `reasoning_engine/reasoning_engine.rs` — track per-call tokens | Hook into engine |

---

## 6. KnowledgeSource Registration Scheme

Integrating GenericAgent's skill crystallization introduces 3 new knowledge domains. Register as follows:

### New KnowledgeSource Enum Variants

In `core/knowledge.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeSource {
    // ... existing sources ...
    GenericAgentCrystallization,  // P0 — Skill crystallization from execution paths
    GenericAgentLayeredMemory,    // P0 — Hierarchical on-demand memory (L0-L4)
    GenericAgentMorphling,        // P1 — Project-level skill absorption
}
```

### CapabilityVector Mapping

```rust
impl KnowledgeSource {
    pub fn capability_vector(&self) -> CapabilityVector {
        match self {
            // Each GA source maps to existing dimensions + new extension dimensions
            KnowledgeSource::GenericAgentCrystallization => CapabilityVector::from_values(
                0.0, 0.0, 0.0, 0.0,    // typography, grid, color, whitespace
                0.0, 0.0, 0.0, 0.0,    // data_viz, emotion, minimalism, experimental
                0.65,                    // inference_depth — GA's task decomposition
                0.50,                    // creativity — GA's autonomous tool writing
                0.70,                    // analysis — GA's trajectory analysis
                0.80,                    // synthesis — GA's SOP distillation
                0.55,                    // domain_specificity
                0.0, 0.0, 0.0, 0.0,     // accessibility, compound_composition, tailwind, react_aria
                0.0, 0.0, 0.0,          // bem_naming, figma_integration, ai_native_states
                0.0,                     // semantic_layer
                0.85,                    // quality_gates — GA's verification
                0.80,                    // verification — GA's test-driven approach
            ),
            // ... other GA sources with similar mappings
        }
    }
}
```

### Extension Dimensions (named, beyond the 23 fixed)

| Dimension Name | Source | Description |
|---------------|--------|-------------|
| `skill_crystallization` | GenericAgentCrystallization | Ability to distill execution paths into reusable skills |
| `sop_distillation` | GenericAgentCrystallization | Quality of SOP generation from trajectories |
| `context_compression` | GenericAgentLayeredMemory | Ability to compress/truncate context while preserving decision signals |
| `layered_recall` | GenericAgentLayeredMemory | Hierarchical memory load (relevant layer only) |
| `morphling_absorb` | GenericAgentMorphling | Per-component decide: call/rewrite/discard |
| `autonomous_code_gen` | GenericAgentCrystallization | Write + install own code-based tools |
| `convergence_rate` | GenericAgentCrystallization | Speed at which repeated tasks converge to optimal cost |

### Seed Memories for ReasoningBank

Inject 5 seed memories:

```rust
fn initialize_with_ga_skills(&mut self) {
    let ga_knowledge = vec![
        ("GenericAgent Skill Crystallization: After each task, extract tool-call trajectory, distill into SOP + executable code, verify by running, persist to L3 memory. 9-round LangChain study: token consumption drops 89.6%, calls converge from 32→5.",
         TaskType::CodeGeneration, 0.95),
        ("GenericAgent Layered Memory: L0 MetaRules (behavior constraints), L1 InsightIndex (fast routing), L2 GlobalFacts (stable long-term), L3 Skills/SOPs (reusable task workflows), L4 SessionArchive (distilled session records). Only load relevant layer per context to maintain density.",
         TaskType::General, 0.94),
        ("GenericAgent Context Compression: Truncate low-density tool descriptions, compress redundant observations, deactivate layers not needed for current task. Core principle: information density maximization within finite context budget.",
         TaskType::General, 0.93),
        ("GenericAgent Morphling Mode: Given external repo, extract goal + tests. Per-component decision: call existing tool, rewrite for context, or discard. Project-level skill absorption with zero human intervention.",
         TaskType::CodeAnalysis, 0.91),
        ("GenericAgent Atomic Tool Design: 9 tools max. code_run as universal escape hatch (install packages, write scripts, call APIs, control hardware). Tool descriptions kept minimal (<50 chars each) to preserve context density.",
         TaskType::General, 0.90),
    ];
    for (desc, task_type, reward) in ga_knowledge {
        self.store(ReasoningMemory::new(desc, task_type, &[], reward));
    }
}
```

---

## 7. Test Strategy

### P0 Tests (Critical Path)

| Test | File | Description | Verification |
|------|------|-------------|--------------|
| `test_crystallize_toolcall_trajectory` | `skill_crystallizer.rs` | Given `Vec<ToolCall>` + `outputs`, produce `SkillArtifact` with valid SOP + code | `crystallize()` returns `Ok(SkillArtifact)` with non-empty `code` and `sop` |
| `test_crystallize_empty_trajectory` | `skill_crystallizer.rs` | Empty trajectory returns error | `Err` |
| `test_crystallize_verification_gate` | `skill_crystallizer.rs` | Failing verification prevents persistence | `crystallize()` fails when verification returns false |
| `test_skill_recall_exact` | `skill_tree.rs` | Exact task match returns skill | `recall_skill("read wechat")` returns cached skill |
| `test_skill_recall_semantic` | `skill_tree.rs` | Semantic match via CapabilityVector cosine similarity | similarity > 0.8 threshold |
| `test_skill_convergence` | `skill_tree.rs` | N repeated calls converge on minimal skill | Call count drops 50%+ by 3rd call |
| `test_layered_memory_load_l3` | `layered_memory.rs` | Task with skill match loads only L3 | L0/L1/L2/L4 not loaded (verified via access counters) |
| `test_layered_memory_default_l0l1` | `layered_memory.rs` | New task loads only L0 + L1 | Only L0 and L1 loaded into context |
| `test_context_compressor_truncate_tools` | `context_compressor.rs` | Tool descs >50 chars truncated | Output descs ≤50 chars; essential info preserved |
| `test_context_compressor_full_pipeline` | `context_compressor.rs` | Full compression: tool descs + memory layers + old turns | Context size reduced 40%+, task completion unchanged |

### P1 Tests

| Test | File | Description | Verification |
|------|------|-------------|--------------|
| `test_tool_writer_generate_python` | `tool_writer.rs` | Given spec "scrape webpage" → valid Python script | Script runs without import errors |
| `test_tool_writer_vulnerability_gate` | `tool_writer.rs` | Spec with `rm -rf /` blocked | Returns Err with SecurityViolation |
| `test_skill_tree_persist_roundtrip` | `skill_tree.rs` | Skills save/load from disk | All fields match after deserialization |
| `test_skill_library_bulk_seed` | `skill_library.rs` | Bulk import 100 GA skills | 100 skills in tree, no duplicates |

### P2 Tests

| Test | File | Description | Verification |
|------|------|-------------|--------------|
| `test_morphling_component_decision_call` | `morphling_bridge.rs` | Existing tool matches → return Call | Decision::Call |
| `test_morphling_component_decision_rewrite` | `morphling_bridge.rs` | Similar but not identical → return Rewrite | Decision::Rewrite |
| `test_morphling_component_decision_discard` | `morphling_bridge.rs` | Irrelevant → return Discard | Decision::Discard |
| `test_goal_time_budget_hard_deadline` | `time_budget.rs` | Budget exceeded at turn N → auto-pause | GoalState becomes BudgetLimited |
| `test_token_tracker_per_turn_limit` | `token_tracker.rs` | Single turn exceeds budget → circuit breaker | CircuitBreaker trips |

---

## 8. Architecture Diagram (Post-Fusion)

```
                              ┌─────────────────────────────────────┐
                              │         NeoTrix SelfEvolver          │
                              │   (existing: evolve_from_url)         │
                              │   (new: evolve_from_trajectory)       │
                              └──────────┬──────────────────────────┘
                                         │
                    ┌────────────────────┼────────────────────┐
                    │                    │                    │
                    ▼                    ▼                    ▼
          ┌─────────────────┐  ┌──────────────────┐  ┌──────────────────┐
          │ G1+G2:          │  │ G3:              │  │ G4:              │
          │ SkillCrystallizer│  │ LayeredMemory    │  │ ContextCompressor │
          │ (new module)    │  │ (new module)     │  │ (new module)     │
          ├─────────────────┤  ├──────────────────┤  ├──────────────────┤
          │ ∙ ToolCall       │  │ ∙ L0: MetaRules  │  │ ∙ Truncate tools  │
          │   trajectory     │  │ ∙ L1: InsightIdx │  │ ∙ Compress layers │
          │ ∙ SOP extract    │  │ ∙ L2: GlobalFact │  │ ∙ Compact turns   │
          │ ∙ Code gen       │  │ ∙ L3: SkillArc   │  │                  │
          │ ∙ Verify+Persist │  │ ∙ L4: SessionV   │  │                  │
          └────────┬────────┘  └────────┬─────────┘  └────────┬─────────┘
                   │                    │                     │
                   └────────────┬───────┴──────────┬──────────┘
                                │                  │
                                ▼                  ▼
                     ┌──────────────────┐  ┌──────────────────┐
                     │ SelfIteratingBrain│  │ ReasoningEngine  │
                     │ (existing: P0+P1) │  │ (existing:       │
                     │  + SkillTree G6)  │  │  + TokenTracker) │
                     └──────────────────┘  └──────────────────┘
```

---

## 9. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-----------|--------|------------|
| GA's skill format (Python) doesn't match NeoTrix's Rust runtime | High | Medium | Store skills in language-agnostic Artifact format; add Rust code-gen path for native skills |
| Context compression may degrade quality on complex tasks | Medium | High | Implement as optional layer; benchmark task completion before/after; fallback if score drops |
| Skill crystallization from LLM trajectories produces non-deterministic results | Medium | Medium | Add verification step (compile/run/test) before commit; rejected skills retry with stricter guard |
| Layered memory increases code complexity without proportional benefit for NeoTrix's use case | Medium | Low | GA's benchmarks show 89.6% token reduction; port those benchmarks to prove ROI |
| Convergence detection may false-positive on genuinely novel tasks | Low | Low | Threshold tuning per task type; user confirmation for first N skills |

---

## 10. Timeline Estimate

| Phase | Features | Files | Est. Lines | Est. Sessions |
|-------|----------|-------|-----------|---------------|
| P0-1 | G1+G2: SkillCrystallizer + SkillArtifact | 4 new + 3 modify | ~500 | 2 |
| P0-2 | G3: LayeredMemory | 3 new + 2 modify | ~400 | 2 |
| P0-3 | G4: ContextCompressor + integration | 1 new + 3 modify | ~250 | 1 |
| P1-1 | G5: AutonomousToolWriter | 1 new + 2 modify | ~350 | 2 |
| P1-2 | G6: SkillTree + G7: SkillLibrarySeed | 2 new + 2 modify | ~300 | 1 |
| P2 | G8+G9+G10: Morphling+TimeBudget+TokenTracker | 3 new + 3 modify | ~300 | 1 |
| **Total** | **10 features** | **14 new + 15 modify** | **~2100** | **9** |

*Each session includes: implementation + `cargo check --lib` + `cargo check --features full --lib` + `cargo test --lib` + TODO sync + session log.*
