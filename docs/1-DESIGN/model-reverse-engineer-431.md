# Model Reverse Engineering — Cycle 431

**Date**: 2026-09-12  
**Scope**: 5 recent papers/models on efficient inference, attention, agent coordination  
**Method**: Pattern extraction → NeoTrix 7-domain mapping

---

## 1. Gated-Memory Routing (arXiv:2609.00237)

**Paper**: *Learning What to Retain: Gated-Memory Routing for Efficient Collaboration in Multi-Agent LLM Systems*  
**Published**: 2026-08-31

### Core Insight
Multi-agent routing from query-only is brittle; routing from full history is expensive. Solution: **learned gated execution memory** that conditions every decision on a compact, filtered state.

### Mechanism
- **Memory Write Gate**: Commits only non-redundant reasoning steps (relevance + novelty against stored records)
- **Retrieval Gate**: Step-adaptive selection — decides per-record whether to surface it
- **Adaptive Halting Controller**: Stops execution when memory holds sufficient evidence (GRU-based recurrent state)
- **Single jointly trained router**: Role allocation, backbone selection, write, retrieval, halting — all co-optimized

### Results
- Best average accuracy across 5 benchmarks (+2.44 over strongest baseline)
- 31.9% inference cost reduction on HumanEval vs full-history routing

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-MEMORY** | Direct analog: Gated Write Gate → KB write policy (suppress redundant nodes). Retrieval Gate → `query --kw` filtered recall. Maps to experience-tree "distill" stage. |
| **NT-CORE** | Adaptive Halting → GWT salience threshold. When consciousness coherence exceeds θ, halt evolution cycle. ConsciousnessTree's "Fruits" stage = halting decision. |
| **NT-MIND** | Joint router → SEAL pipeline stage selection. Memory-as-control-signal mirrors SEAL's "phase deps" — each stage reads filtered prior output, not raw history. |
| **NT-ACT** | Role allocation → NT-ACT tool selection. History-aware allocator prevents repeating failed tool calls (maps to SEAL retry policy). |

### Actionable Pattern for NeoTrix
**Implement Gated Write for KB**: Before inserting to `experience` namespace, compute novelty score against existing entries. If redundancy > λ, skip write. This addresses the "experience table bloat" problem in AGENTS.md pointer conservation.

---

## 2. AGAO — Adaptive Goal-aware Attention Orchestration (arXiv:2607.23678)

**Paper**: *Focus Is All You Need: Adaptive Goal-aware Attention Orchestration for Multi-Agent Graph Systems*  
**Published**: 2026-07-26

### Core Insight
Extend attention mechanism from token-level to **workflow-level agent coordination**. Treat agents as dynamically selectable computational units, not fixed workflow operators.

### Mechanism
- **Goal-aware Attention**: Semantic alignment between user objective and agent capabilities
- **Topology-aware Attention**: Graph structural dependencies + critical path weighting
- **Resource-aware Attention**: Translates attention scores → execution priorities, model selection, token budgets
- **Adaptive Graph Routing**: Attention distributions update based on intermediate execution feedback

### Results
- Improved task effectiveness + reduced computation across diverse multi-agent workloads
- Establishes "Attention Engineering" as new paradigm for multi-agent systems

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | GWT salience = goal-aware attention. ConsciousnessTree branches as agent nodes. E8 hexagram reasoning as topology-aware routing. |
| **NT-MIND** | SEAL pipeline stages as graph nodes. Resource-aware attention → budget allocation per stage (Phase-0 through Phase-5). |
| **NT-WORLD** | Crawler nodes as agent graph. Topology-aware attention → prioritize high-information sources. Goal-aware → match crawl strategy to user objective. |
| **NT-IO** | Resource-aware attention → model routing per task difficulty. Cost-Aware Routing (Axiom A1) formalized. |

### Actionable Pattern for NeoTrix
**Formalize GWT as Attention Orchestration**: GWT's salience computation should incorporate (1) goal alignment, (2) module dependency graph, and (3) resource budget. Current GWT implements (2) partially; (1) and (3) are ad-hoc.

---

## 3. UNISON — Near-Memory KV Scheduler for Agent Sessions (arXiv:2609.09643)

**Paper**: *UNISON: A Co-Designed Near-Memory Scheduler of Session KV Residency for LLM Agents*  
**Published**: 2026-09-09

### Core Insight
Agent loops (plan→tool→resume) stress KV cache differently than multi-turn chat. Existing eviction based on recency/timeout misses the **loop structure** — a live wait is not a cold discard.

### Mechanism
- **SPEAR** (Survival-Penalty Eviction for Agent Return-gap): Gap average + turn-indexed hazard → eviction scoring
- **TIDE** (Tiering in Idle-window DMA Events): Uses observed wait as DMA budget for tier placement
- **Unified ranking**: Single live ranking across both mechanisms
- **Hardware**: 28nm CMOS core, 0.169mm², 13.6mW — negligible overhead

### Results
- Hit rate +0.3% to +23.1%, AMAT reduction 22%–51%, TTFT reduction 58%–89%
- Best non-oracle entry on every trace across 3 model families, 1,415 sessions

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | KV residency scheduling → GWT attention buffer management. Agent return-gap → ConsciousnessTree cycle boundary (when to flush/reload awareness). |
| **NT-MEMORY** | KV cache eviction → KB node lifecycle management. SPEARC's gap-awareness → experience node retention policy (don't evict nodes mid-cycle). |
| **NT-PHYSICAL** | Hardware scheduling core → validates NT-PHYSICAL's power management for constrained devices. 13.6mW budget → embedded embodiment feasibility. |
| **NT-SHIELD** | Tier placement → trust-tiered cache (Trusted=GPU, Contracted=Host, Untrusted=Disk). Aligns with Egress Privacy Guard tiers. |

### Actionable Pattern for NeoTrix
**Session-Aware KB Eviction**: Replace simple TTL-based node eviction with loop-aware scoring. During active SEAL cycles, suppress eviction of nodes referenced in current cycle's working set. UNISON's SPEARC heuristic (gap average + turn hazard) maps directly.

---

## 4. RouteRelay — Cross-Layer Route Reuse for Sparse Attention (arXiv:2609.07306)

**Paper**: *RouteRelay: Event-Triggered Cross-Layer Route Reuse for Efficient Dynamic Sparse Attention*  
**Published**: 2026-09-07

### Core Insight
Dynamic sparse attention rebuilds chunk-chunk score matrix at every layer even when routes barely change. Reuse route metadata across depth; reroute only when sentinel chunks challenge weakest selection.

### Mechanism
- **Anchor layers**: Full routing
- **Intermediate layers**: Rescore previous top-k + compact sentinel set (near-miss + random probe chunks)
- **Reroute only on challenge**: Query row rerouted only when sentinel challenges weakest selected chunk
- **Row-selective GPU execution**: Only challenged rows get full rerouting

### Results
- 99.99%+ route recall retained
- Reroutes only 25%–78% of rows depending on drift
- Evaluates 38.4%–51.6% of full-routing score pairs

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Route reuse → GWT attention route caching. Anchor layers → ConsciousnessTree "Soil" stage (full re-evaluation). Intermediate → incremental updates. |
| **NT-MIND** | SEAL pipeline stage reuse. If prior stage output hasn't drifted beyond sentinel threshold, skip full re-evaluation. Maps to Convergence Check optimization. |
| **NT-IO** | Model routing reuse. If current model selection remains valid (no sentinel challenge), skip re-routing. Reduces routing overhead. |
| **NT-WORLD** | Crawl route reuse. If source quality hasn't changed (sentinel = new data signal), reuse cached crawl strategy. |

### Actionable Pattern for NeoTrix
**GWT Route Caching**: Cache salience routing decisions. At each ConsciousnessTree cycle, only re-evaluate modules where input state has drifted beyond sentinel threshold. Expected: 50%+ reduction in GWT salience recomputation per cycle.

---

## 5. Procedural Graphs — Self-Evolving Execution Structures (arXiv:2609.09153)

**Paper**: *Procedural Graphs: Self-Evolving Execution Structures for LLM Agents*  
**Published**: 2026-09-08

### Core Insight
Agents lose track of objectives as trajectories lengthen. Solution: organize procedural knowledge into (procedure, relation, procedure) triplets — a "knowledge graph for what-to-do."

### Mechanism
- **Procedural Graph**: Nodes = procedures, Edges = procedural relations (order, condition, dependency)
- **Active node localization**: At each step, find agent's position in graph
- **Guidance model**: Translates surrounding subgraph → step-level situational guidance (biases, not dictates)
- **Self-evolution**: LLM refiner contrasts failed vs successful trajectories → edits graph topology + attributes
- **Minimal skeleton bootstrap**: Start with minimal graph → loop builds to match/surpass hand-designed graphs

### Results
- Consistent gains over memory-based baselines across multiple datasets/task types/LLMs
- Self-evolution further improves without manual engineering
- Can repair flawed expert priors

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Procedural Graph → E8 Hexagram reasoning as procedural knowledge. Each hexagram = procedure node. Relations = yijing line transformations. Self-evolution → SEAL auto-refinement. |
| **NT-MIND** | Self-evolving graph → SEAL pipeline's skill crystallization. Failed trajectories → repair/reject. Successful → promote to C4/C5. This IS the SEAL pipeline formalized as graph evolution. |
| **NT-ACT** | Guidance model → NT-ACT tool selection bias. Not dictating but biasing next action. Maps to NT-ACT's "action recommendations" with confidence scores. |
| **NT-MEMORY** | Procedural Graph as memory structure → KB edges become procedural relations. Temporal ordering → `created_at` / `updated_at` timestamps. |

### Actionable Pattern for NeoTrix
**SEAL-as-Procedural-Graph**: Model each SEAL stage as a procedure node. Track which stages succeed/fail for specific task types. Self-evolving graph → automatically adjust stage ordering based on historical success rates. This formalizes the "phase deps" pattern in SEAL.

---

## Cross-Paper Synthesis

### Emergent Pattern: Memory as Control Signal (3 papers)

| Paper | Memory Role |
|-------|-------------|
| Gated-Memory Routing | Memory as routing control — who acts next, what they read, when to stop |
| UNISON | KV residency as session-level control — eviction is a loop-structure problem |
| Procedural Graph | Procedural memory as guidance — biases action without dictating |

**NeoTrix Implication**: NT-MEMORY should not be a passive store. It should be an active control signal that influences NT-CORE's attention routing, NT-MIND's evolution decisions, and NT-ACT's action selection. Current KB is too passive.

### Emergent Pattern: Attention at Workflow Level (2 papers)

| Paper | Attention Scope |
|-------|----------------|
| AGAO | Goal + topology + resource attention over agent graph |
| RouteRelay | Cross-layer route reuse with sentinel-triggered rerouting |

**NeoTrix Implication**: GWT should operate at two scales — token-level (current) and workflow-level (proposed). The ConsciousnessTree already models workflow-level attention conceptually but lacks the mathematical formalization AGAO provides.

### Emergent Pattern: Self-Evolution via Trajectory Contrast (2 papers)

| Paper | Evolution Mechanism |
|-------|-------------------|
| Procedural Graph | Contrast failed vs successful → edit graph |
| GenericAgent (from trending) | Morphling mode: absorb external → decide call/rewrite/discard |

**NeoTrix Implication**: SEAL's self-evolution should explicitly contrast successful vs failed runs at each stage. Current SEAL tracks success rates but doesn't use trajectory contrast to modify the pipeline itself.

---

## Priority Actions

| Priority | Action | Domain | Effort |
|----------|--------|--------|--------|
| P0 | Implement Gated Write for KB experience namespace | NT-MEMORY | Medium |
| P0 | Formalize GWT as 3-component attention orchestration | NT-CORE | High |
| P1 | Session-aware KB eviction (SPEARC heuristic) | NT-MEMORY | Medium |
| P1 | GWT route caching with sentinel triggers | NT-CORE | Medium |
| P2 | SEAL-as-Procedural-Graph modeling | NT-MIND | High |
| P2 | Memory-as-control-signal architecture | NT-MEMORY | High |

---

*Generated by NeoTrix iteration loop, cycle 431*
