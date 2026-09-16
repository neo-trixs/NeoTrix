# NeoTrix v19 — Fusion Evolution Roadmap

> **Absorption Cycle**: 2026-09-15 | **Sources**: 278+ repos/papers → 7 major frameworks + 5 Rust memory systems
> **Core Principle**: 聚焦冗余 + 扁平缺陷 + 跨域错位 → 冗余清理 + 合理架构重构

---

## 1. External Technology Absorption Summary

### 1.1 Long-Horizon Agent Runtimes (7 frameworks)

| Framework | Key Innovation | NeoTrix Mapping | Absorption Priority |
|-----------|---------------|-----------------|-------------------|
| **Prime Agent** | Recursive Language Model (RLM) + Continual Harness + persistent IPython REPL | `nt_mind/self_iterating/` → RLM abstraction, session persistence | **A1** — direct integration |
| **InfiAgent** | File-centric state abstraction, O(1) context with bounded reasoning | `nt_memory_kb/tiered_store.rs` → file-centric state, hot/warm/cold | **A1** — already partially aligned |
| **Argus** | Manager/Planner/Engineer/Reviewer 4-role + verified pivoting + Dense-Intelligence Density | `nt_core_consciousness/` → role separation, `nt_mind/` → verified evolution | **A1** — maps to SEAL pipeline |
| **ROMA** | Recursive Atomizer→Planner→Executor→Aggregator + context compression | `nt_task_orchestrator` → recursive decomposition, `nt_mind/harness/` | **A2** — extend orchestrator |
| **AutoAgent** | Evolving Cognition + Elastic Memory Orchestrator + On-the-fly Decision | `nt_mind/seal_core/` → cognitive evolution, `nt_memory_kb/` → elastic memory | **A1** — core alignment |
| **AgentFlow** | Planner→Executor→Verifier→Generator + Flow-GRPO training | `nt_mind/seal_core/self_iterating/` → verifier loop, `nt_core_self/` | **A2** — add verifier module |
| **DeepAgent** | Autonomous Memory Folding (episodic/working/tool) + ToolPO | `nt_memory_kb/` → memory folding, `nt_act/` → tool selection | **A2** — memory architecture |

### 1.2 Rust Memory Systems (5 systems)

| System | Key Innovation | NeoTrix Mapping | Absorption Priority |
|--------|---------------|-----------------|-------------------|
| **memrust** | HNSW + BM25 + entity graph + recency fusion, MCP, multi-agent visibility | `nt_memory_kb/` → hybrid retrieval, `nt_memory_search.rs` | **A1** — already has similar design |
| **hirn** | 4-layer memory (working/episodic/semantic/procedural) + graph-native recall + symbolic temporal | `nt_memory_typed/` → typed memory layers, `nt_core_bank/` | **A1** — direct map |
| **agentic-memory** | Cognitive event graph + 16 query types + tiered storage + BLAKE3 integrity | `nt_memory_kb/` → graph queries, `tiered_store.rs` | **A2** — extend query types |
| **arcagent-state** | Append-only event log + git-native persistence + GASP protocol | `nt_nexus/` → cross-session memory, `experience-tree/` | **A3** — event sourcing |
| **Memory Genome Engine** | Marker-first retrieval + sealed binary pages + ContextPacket | `nt_memory_kb/` → marker-based retrieval, `nt_memory_types.rs` | **A3** — compact storage |

---

## 2. NeoTrix v19 Architecture Evolution

### 2.1 Current State (v18.9)

```
L6 Meta-Cognition (57 files)   → nt_meta + nt_repair + nt_nexus
L5 Cognition (408 files)       → nt_core + nt_mind + nt_core_industrial
L4 Emotion (6 files)           → nt_feel (underdeveloped)
L3 Embodiment (194 files)      → nt_physical + nt_shield + nt_feel
L2 Perception (212 files)      → nt_world + nt_sense
L1 Action (440 files)          → nt_act + nt_io + nt_memory
Core (356 files)               → E8, HyperCube, GWT, SEAL, KB
```

### 2.2 Evolution Target (v19)

```
L6 Meta-Cognition → +arcagent-state event sourcing, +GASP persistence
L5 Cognition      → +ROMA recursive decomposition, +AgentFlow verifier, +Argus 4-role
L4 Emotion        → +hirn emotional tagging, +affective memory consolidation
L3 Embodiment     → +PrimeAgent RLM sessions, +InfiAgent bounded context
L2 Perception     → +DeepAgent memory folding, +memrust hybrid retrieval
L1 Action         → +agentic-memory graph queries, +Memory Genome compact storage
Core              → +event log as single source of truth, +append-only persistence
```

---

## 3. Core Roadmap Tasks

### Phase A: Immediate Fusion (Week 1-2)

#### A1: File-Centric State Abstraction (InfiAgent + arcagent-state)
- **What**: Externalize all persistent state to file system, separate from reasoning context
- **Where**: `nt_memory_kb/tiered_store.rs` → extend to file-centric model
- **How**:
  1. Each task gets a dedicated workspace directory
  2. Context reconstructed from workspace snapshot + fixed action window
  3. O(1) context size regardless of task duration
  4. Append-only event log as single source of truth
- **Impact**: Eliminates context window bottleneck for long-horizon tasks
- **Files**: `tiered_store.rs`, `nt_memory_pipeline.rs`, `nt_memory_search.rs`

#### A2: Elastic Memory Orchestrator (AutoAgent)
- **What**: Dynamic memory compression with raw records → compressed trajectories → reusable episodic abstractions
- **Where**: `nt_memory_kb/` → add `memory_orchestrator.rs`
- **How**:
  1. Raw records: full fidelity storage
  2. Compressed trajectories: redundant info filtered
  3. Episodic abstractions: reusable patterns extracted
  4. Token overhead reduced while preserving decision-critical evidence
- **Impact**: 60-80% token reduction for long-horizon reasoning
- **Files**: new `nt_memory_orchestrator.rs`, `nt_memory_pipeline.rs`

#### A3: Verified Pivoting (Argus)
- **What**: Separate stable user intent from operational objectives; admit revisions only after role-owned review
- **Where**: `nt_mind/seal_core/` → extend SEAL pipeline with verification gates
- **How**:
  1. Manager role: commits to overall objective, determines campaign stage
  2. Planner role: breaks into bounded tasks with dependencies
  3. Engineer role: executes, modifies, self-reviews
  4. Reviewer role: independent inspection, issues verdicts
  5. Verified pivoting: when evidence contradicts, Manager admits revision
- **Impact**: Prevents goal drift in long campaigns
- **Files**: `seal_core/core/evaluator.rs`, `seal_core/self_iterating/pipeline.rs`

### Phase B: Architectural Deepening (Week 3-4)

#### B1: Recursive Meta-Agent Framework (ROMA)
- **What**: Atomizer→Planner→Executor→Aggregator recursive control loop
- **Where**: `nt_task_orchestrator/` → extend with recursive decomposition
- **How**:
  1. Atomizer: decides if task is atomic
  2. Planner: MECE subtask graph decomposition
  3. Executor: parallel atomic execution
  4. Aggregator: compress + verify + return to parent
  5. Context compression at each aggregation level
- **Impact**: Transparent, hierarchical execution traces
- **Files**: `nt_task_orchestrator/mod.rs`, new `recursive_controller.rs`

#### B2: Hybrid Retrieval Stack (memrust + hirn)
- **What**: HNSW + BM25 + entity graph + recency fusion with pre-filtering
- **Where**: `nt_memory_kb/nt_memory_search.rs` → extend retrieval
- **How**:
  1. Vector index: HNSW with SQ8 quantization
  2. Text index: BM25 inverted index
  3. Entity graph: co-occurrence edges, 1-hop traversal
  4. Fusion: reciprocal rank fusion + exponential recency decay
  5. Pre-filtering inside every index (not post-hoc)
- **Impact**: 27% accuracy improvement over pure vector search
- **Files**: `nt_memory_search.rs`, `nt_memory_types.rs`, new `retrieval_fusion.rs`

#### B3: Cognitive Event Graph (agentic-memory)
- **What**: Typed cognitive event graph with 16 query types
- **Where**: `nt_memory_kb/` → extend graph model
- **How**:
  1. Nodes: facts, decisions, inferences, corrections, skills, episodes
  2. Edges: causal, belief revision, temporal ordering
  3. Queries: traversal, pattern matching, temporal comparison, causal impact, reasoning gap detection
  4. Supersession: corrections don't delete, they supersede
- **Impact**: Explainable reasoning chains, self-correcting memory
- **Files**: `nt_memory_types.rs`, `nt_memory_graphrag/`

### Phase C: Memory Architecture Overhaul (Week 5-6)

#### C1: Four-Layer Memory (hirn alignment)
- **What**: Working → Episodic → Semantic → Procedural with biologically-grounded transitions
- **Where**: `nt_memory_typed/` + `nt_core_bank/` → restructure
- **How**:
  1. Working memory: TTL-based, auto-expire
  2. Episodic: session-bound, timestamped events
  3. Semantic: consolidated summaries, graph relationships
  4. Procedural: skills, patterns, reusable procedures
  5. Surprise-gated admission to semantic layer
  6. Spaced repetition for procedural consolidation
- **Impact**: Biologically plausible memory, better consolidation
- **Files**: `nt_memory_typed/`, `nt_core_bank/mem.rs`, `tiered_store.rs`

#### C2: Event Sourcing Core (arcagent-state)
- **What**: Append-only event log as single source of truth, fold to rebuild state
- **Where**: `nt_nexus/` + `nt_core_event_bus.rs` → event sourcing
- **How**:
  1. All mutations are append-only events
  2. State rebuilt by folding event log
  3. Crash recovery: replay from last checkpoint
  4. Git-native persistence for portability
  5. Lineage queries: "why does this state exist?"
- **Impact**: Deterministic crash recovery, portable agent state
- **Files**: `nt_core_event_bus.rs`, `nt_nexus/`, new `event_store.rs`

#### C3: Symbolic Temporal Reasoning (hirn)
- **What**: Dates resolved and intervals computed in Rust, handed to reader as evidence
- **Where**: `nt_memory_types.rs` → add temporal reasoning
- **How**:
  1. Temporal index: interval-exact queries
  2. "AS OF" queries: state at any point in time
  3. Causal temporal edges: "what happened before X?"
  4. +17 accuracy on temporal questions (p=0.0015)
- **Impact**: Precise temporal reasoning without LLM hallucination
- **Files**: `nt_memory_types.rs`, `nt_memory_search.rs`

### Phase D: Redundancy Cleanup (Week 7-8)

#### D1: Duplicate Module Consolidation
- **SearchResult types**: 6+ definitions → 1 canonical `KbSearchResult`
- **RiskLevel types**: consolidate to 1 canonical
- **GraphNode/GraphEdge**: 6+ definitions → 3 canonical
- **TaskStatus**: 8 definitions → 1 superset
- **KnowledgeSource**: 2 definitions (neotrix + neotrix_types) → 1

#### D2: Unused Code Elimination
- Dead dispatch routes (17 already fixed, scan for more)
- Unused imports across test targets (~120 errors)
- Orphaned modules not connected to any capability

#### D3: Cross-Layer Dependency Cleanup
- L4 Emotion (6 files) → either expand or merge into L3/L5
- L6 Meta (57 files) → verify all re-exports are wired
- Core → ensure no circular dependencies

### Phase E: Production Hardening (Week 9-10)

#### E1: Test Target Compilation (120 errors)
- Fix all `cargo check --tests` errors
- Add integration tests for new modules
- Benchmark new retrieval stack

#### E2: Documentation Update
- Architecture diagrams for v19
- Module documentation for new additions
- API documentation for public interfaces

#### E3: Performance Benchmarking
- Retrieval latency: target <1ms for hot, <10ms for warm
- Memory consolidation: target <100ms per cycle
- Token reduction: target 60-80% for long-horizon

---

## 4. Absorption Matrix: External → NeoTrix

```
┌─────────────────────────────────────────────────────────────────┐
│                    NeoTrix v19 Absorption Matrix                │
├──────────────────┬──────────────────────────────────────────────┤
│  Prime Agent     │  RLM abstraction → nt_mind/self_iterating    │
│                  │  Continual Harness → session persistence     │
│                  │  Recursive subagents → nt_task_orchestrator  │
├──────────────────┼──────────────────────────────────────────────┤
│  InfiAgent       │  File-centric state → tiered_store.rs        │
│                  │  Bounded context → memory_orchestrator.rs    │
│                  │  External attention → nt_world/crawl/        │
├──────────────────┼──────────────────────────────────────────────┤
│  Argus           │  4-role separation → SEAL pipeline           │
│                  │  Verified pivoting → evaluator.rs            │
│                  │  Dense-Intelligence Density → metrics        │
├──────────────────┼──────────────────────────────────────────────┤
│  ROMA            │  Recursive decomposition → task_orchestrator │
│                  │  Context compression → aggregation.rs        │
│                  │  Execution traces → nt_nexus/                │
├──────────────────┼──────────────────────────────────────────────┤
│  AutoAgent       │  Elastic memory → memory_orchestrator.rs     │
│                  │  Cognitive evolution → seal_core/             │
│                  │  Skill distillation → skill_engine/          │
├──────────────────┼──────────────────────────────────────────────┤
│  memrust         │  HNSW+BM25+graph → nt_memory_search.rs      │
│                  │  Pre-filtering → retrieval pipeline          │
│                  │  Multi-agent visibility → KB scoping         │
├──────────────────┼──────────────────────────────────────────────┤
│  hirn            │  4-layer memory → nt_memory_typed/           │
│                  │  Graph-native recall → spreading activation  │
│                  │  Symbolic temporal → nt_memory_types.rs      │
├──────────────────┼──────────────────────────────────────────────┤
│  agentic-memory  │  Cognitive graph → nt_memory_graphrag/       │
│                  │  16 query types → query engine               │
│                  │  Supersession → version management           │
├──────────────────┼──────────────────────────────────────────────┤
│  arcagent-state  │  Event sourcing → nt_core_event_bus.rs       │
│                  │  Git persistence → nt_nexus/                 │
│                  │  Lineage queries → traceability              │
└──────────────────┴──────────────────────────────────────────────┘
```

---

## 5. Version Evolution Plan

| Version | Phase | Key Deliverable | Target Date |
|---------|-------|----------------|-------------|
| v19.0 | A1-A3 | File-centric state + Elastic memory + Verified pivoting | Week 2 |
| v19.1 | B1-B3 | ROMA recursive + Hybrid retrieval + Cognitive graph | Week 4 |
| v19.2 | C1-C3 | 4-layer memory + Event sourcing + Temporal reasoning | Week 6 |
| v19.3 | D1-D3 | Redundancy cleanup + Dead code elimination | Week 8 |
| v19.4 | E1-E3 | Test compilation + Documentation + Benchmarks | Week 10 |

---

## 6. Anti-Patterns to Avoid

1. **Don't bolt memory onto prompt** — externalize to file system (InfiAgent lesson)
2. **Don't use flat multi-agent** — hierarchical DAG reduces error propagation (InfiAgent)
3. **Don't assume static context** — working contracts change as knowledge grows (Argus)
4. **Don't compress without verification** — verified pivoting prevents goal drift (Argus)
5. **Don't use single retrieval leg** — hybrid fusion outperforms pure vector by 27% (memrust)
6. **Don't delete — supersede** — corrections preserve history (agentic-memory)
7. **Don't skip lineage** — explainable state requires append-only events (arcagent-state)

---

## 7. Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Retrieval accuracy | ~70% (pure vector) | >95% (hybrid fusion) | Benchmark suite |
| Long-horizon stability | Context-dependent | O(1) context | Task completion rate |
| Memory consolidation | Manual | Automatic | Cycle time |
| Crash recovery | Checkpoint-based | Event-sourced | Recovery time |
| Token efficiency | 1x baseline | 3-5x reduction | Token count per task |
| Explainability | Limited | Full lineage | Query trace depth |
