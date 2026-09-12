# Model Reverse Engineering — Cycle 433

**Date:** 2026-09-12
**Focus:** Recent papers on efficient inference, attention, agent coordination, memory, self-improvement

---

## Paper 1: Procedural Graphs — Self-Evolving Execution Structures for LLM Agents

**arXiv:** 2609.09153 (Sep 8 2026)
**Key Innovation:** Organizes procedural knowledge into (procedure, relation, procedure) triplets — a "Procedural Graph" for what-to-do questions, analogous to knowledge graphs for what-is questions.

### Core Mechanism
- At each step, localizes agent's active node and translates surrounding subgraph into step-level situational guidance
- Guidance **biases** next action without dictating it (soft constraint, not hard state machine)
- Self-evolving: LLM refiner contrasts failed vs successful trajectories, edits graph topology
- Edits committed only if they preserve/improve held-out validation performance
- Rejected edits retained to discourage repetition

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-MIND** | Graph-as-memory for procedural knowledge | SEAL pipeline could store execution patterns as Procedural Graph nodes |
| **NT-CORE** | Guidance model translating subgraph → situational context | GWT salience could weight procedural graph nodes by task relevance |
| **NT-MEMORY** | Self-evolving graph topology from experience | experience-tree could evolve procedure graphs alongside distillation |

### Architecture Insight
The key insight is separating **what procedure exists** (graph structure) from **when to use it** (guidance model). This mirrors NeoTrix's separation of capability registry (what) from GWT attention routing (when). Procedural Graphs make this separation explicit at the execution level.

### Absorption Vector
**R-P42 compatible:** Enhance SEAL phase execution with procedural graph nodes. Each skill crystallization creates not just a skill entry but a procedure graph node with validation-gated edges to related procedures.

---

## Paper 2: PARSER — Read in Parallel, Reason in Depth for Long-Context LLM Agents

**arXiv:** 2609.06702 (Sep 6 2026)
**Key Innovation:** Decouples reading from reasoning via parallel subagent architecture. A bank of lightweight subagents each bound to a single chunk read the entire document in parallel, while a lead agent reasons in depth through iterative scatter-gather rounds.

### Core Mechanism
- Lead agent broadcasts queries to all subagents simultaneously
- Each subagent processes its assigned chunk independently
- Lead aggregates returned evidence, formulates deeper follow-up queries
- Subagents remain frozen; lead agent optimized with RL
- 4B backbone outperforms strongest sequential baseline by 5.7pp avg, 12.0pp at 896K tokens
- 11x latency reduction vs sequential processing

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-WORLD** | Parallel document processing | UnifiedCrawler could adopt scatter-gather for multi-source acquisition |
| **NT-MEMORY** | Lead agent as reasoning coordinator | KB query layer could use PARSER pattern for multi-hop retrieval |
| **NT-CORE** | GWT attention as lead agent role | Attention routing naturally maps to scatter-gather broadcast |

### Architecture Insight
PARSER's key insight: reading and reasoning are **different computational profiles**. Reading is embarrassingly parallel (each chunk independent); reasoning is sequential (each step depends on prior evidence). Coupling them forces serial processing even when chunks are independent.

### Absorption Vector
**NT-WORLD enhancement:** Multi-source crawl queries use PARSER pattern — each fetcher is a subagent, lead coordinator aggregates evidence across sources. Particularly effective for long-document extraction and multi-hop knowledge acquisition.

---

## Paper 3: CoSkill — Joint RL of Reasoning and Meta-Skill Agents for Hierarchical Skill Evolution

**arXiv:** 2609.04865 (Sep 4 2026)
**Key Innovation:** Recasts static meta-skill workflow as a learnable Meta-Skill Agent, jointly trained with Reasoning Agent over hierarchical skill library. Skills are not passive objects but active agents.

### Core Mechanism
- Reasoning Agent and Meta-Skill Agent share a single backbone
- Reasoning Agent conditions actions on retrieved task skill and step skills from child set
- Task performance guides Meta-Skill Agent in refining step skills
- End-to-end co-adaptation: skill evolution and policy optimization happen together
- 98.4% success on ALFWorld (+3.5pp), 90.6% on WebShop (+6.2pp)

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-MIND** | Skills as active agents, not passive templates | SEAL skill nodes could become self-improving agents |
| **NT-ACT** | Reasoning Agent = task executor, Meta-Skill = skill curator | NT-ACT orchestration could split into execution and curation |
| **NT-CORE** | Shared backbone for reasoning + skill management | E8 reasoning and skill registry share computational substrate |

### Architecture Insight
Traditional skill libraries treat skills as static templates to retrieve and execute. CoSkill treats skill management as a **learnable agent** that improves alongside the reasoning agent. This is the missing piece in SEAL — skill crystallization currently produces static artifacts, not self-improving ones.

### Absorption Vector
**R-P79/R-P42 synthesis:** When crystallizing skills in SEAL, create not just skill entries but a Meta-Skill Agent that can refine those entries based on execution feedback. The Meta-Skill Agent learns which step-level instructions are reusable across tasks.

---

## Paper 4: MEMO — Multimodal Evidence Memory Organization for Long-Horizon LLM Agents

**arXiv:** 2609.07471 (Sep 7 2026)
**Key Innovation:** Addresses the fundamental tension between continuous memory accumulation and limited context capacity by selecting evidence and choosing optimal presentation modality (textual, visual, or dual-channel).

### Core Mechanism
- Trained evidence extractor selects relevant memory blocks
- Query-conditioned memory manager assigns each unit to textual, visual, or dual-channel carrier
- Selects layout matching evidence structure
- Memory manager trained with offline reader feedback measuring guided memory plan utility
- 10-40% fewer memory tokens vs baselines with better downstream performance

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-MEMORY** | Multi-modal memory presentation | KB retrieval could choose text vs structured representation |
| **NT-FEEL** | Evidence modality affects emotional processing | Visual memory triggers different affective responses |
| **NT-IO** | Presentation adaptation for different interfaces | CLI vs web vs voice get different memory formats |

### Architecture Insight
The core realization: different types of evidence are better conveyed through different modalities. Text preserves fidelity but costs uniform tokens. Visual rendering exposes structure but loses detail. The optimal choice depends on both the evidence type and the downstream task.

### Absorption Vector
**Axiom A2 refinement:** Context compression is not just token reduction but **modality selection**. Some memories are better as structured data, some as text, some as visual summaries. NT-MEMORY could adopt a memory manager that chooses presentation modality per evidence unit.

---

## Paper 5: PSMAS — Phase-Scheduled Multi-Agent Systems for Token-Efficient Coordination

**arXiv:** 2604.17400 (revised 2026)
**Key Innovation:** Reconceptualizes agent activation as continuous control over shared attention space on circular manifold S¹. Each agent assigned fixed angular phase; global sweep signal rotates at velocity ω, activating only agents within angular window ε.

### Core Mechanism
- Agents placed on S¹ by topological phase derived from task dependency topology
- Global sweep signal φ(t) activates only agents within angular window ε
- Idle agents receive compressed context summaries (not full context)
- Scheduling and compression are **independent gains**: scheduling saves 18-20pp alone
- 27.3% mean token reduction (up to 34.8%) while preserving 99% task performance
- Proves circular manifold is necessary topology (not linear, not grid)

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** | GWT attention as sweep signal | Attention routing could use phase-based activation |
| **NT-ACT** | Agent activation scheduling | Multi-domain task execution with phase-ordered activation |
| **NT-MEMORY** | Context compression for idle agents | Agents not currently needed get summary, not full context |
| **L6 Meta** | Control theory for agent coordination | Meta-cognition layer could manage activation phases |

### Architecture Insight
PSMAS proves that multi-agent coordination is a **continuous control problem**, not a discrete scheduling problem. The circular manifold S¹ is the natural topology because agent pipelines are inherently periodic (refinement loops, verification cycles). Linear scheduling has boundary problems; circular scheduling has none.

### Absorption Vector
**GWT enhancement:** Current GWT attention routing could incorporate phase-based activation. Instead of broadcasting to all specialist modules equally, rotate activation through a sweep signal that prioritizes modules based on task-phase alignment. Idle modules receive compressed context (heartbeat-level signals only).

---

## Cross-Paper Synthesis: 5 Meta-Patterns

### 1. Separation of Compilation from Execution
**Papers:** Procedural Graphs, ReActNet (from search), PSMAS
**Pattern:** Design-time topology compilation separate from runtime execution. The topology is task-conditioned but computed once at start, then executed through structured message passing.
**NeoTrix:** GWT attention routing currently does both simultaneously. Could benefit from pre-compiling attention topology per task type.

### 2. Parallel Reading, Sequential Reasoning
**Papers:** PARSER, AgentInfer (from search)
**Pattern:** Reading/processing is embarrassingly parallel; reasoning is sequential. Decoupling them yields massive latency reduction (11x) without quality loss.
**NeoTrix:** NT-WORLD crawl and NT-MEMORY retrieval could adopt this pattern. Each fetcher/retriever runs in parallel; reasoning coordinator aggregates sequentially.

### 3. Skills as Active Agents
**Papers:** CoSkill, SkillGLoW (from search)
**Pattern:** Skills are not passive templates but active agents with their own optimization loops. Meta-skill agents refine skill libraries based on execution feedback.
**NeoTrix:** SEAL skill crystallization produces static artifacts. Could evolve to produce self-improving skill agents that refine based on usage.

### 4. Modality-Adaptive Memory
**Papers:** MEMO, ConvMem (from search)
**Pattern:** Memory is not just text but can be text, visual, structured data, or hierarchical summaries. The optimal modality depends on evidence type and downstream task.
**NeoTrix:** NT-MEMORY could choose presentation modality per evidence unit. KB retrieval returns not just text but structured data when appropriate.

### 5. Phase-Based Activation Control
**Papers:** PSMAS, AGAO (from search)
**Pattern:** Agent activation as continuous control over attention space. Agents have phases; a sweep signal activates relevant phases for current task state.
**NeoTrix:** GWT could use phase-based activation instead of broadcasting to all modules. Reduces unnecessary computation while maintaining coverage.

---

## Implementation Priority Matrix

| Paper | Effort | Impact | NeoTrix Domain | Priority |
|-------|--------|--------|----------------|----------|
| Procedural Graphs | Medium | High | NT-MIND + NT-CORE | P1 — SEAL execution graphs |
| PARSER | Low | High | NT-WORLD + NT-MEMORY | P0 — Parallel crawl/retrieval |
| CoSkill | High | High | NT-MIND + NT-ACT | P2 — Self-improving skill nodes |
| MEMO | Medium | Medium | NT-MEMORY | P1 — Modality-adaptive retrieval |
| PSMAS | High | High | NT-CORE (GWT) | P2 — Phase-based attention routing |

---

## Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| PSMAS assumes periodic tasks; NeoTrix tasks may be aperiodic | PSMAS degrades gracefully on unstructured tasks (14-19% reduction vs 27-35% on structured). Use full PSMAS for known task types, fallback to simple scheduling for novel ones |
| CoSkill requires shared backbone; NeoTrix modules are independent | CoSkill's "shared backbone" maps to shared context window, not shared weights. Each NT-* module reads same context but processes independently |
| MEMO trained memory manager; NeoTrix is training-free | Use rule-based modality selection as proxy (text for facts, structured for relations, summary for long history). MEMO's insight is modality choice matters, not that it must be learned |
| PARSER requires frozen subagents; NT-WORLD fetchers are diverse | PATTERN: frozen subagents = stateless fetchers. Each NT-WORLD fetcher is naturally stateless. Lead coordinator = NT-MEMORY query layer |

---

## Axiom Alignment Check

| Axiom | Paper Alignment | Conflict |
|-------|----------------|----------|
| A1 Cost-Aware Routing | PSMAS proves phase-based routing saves 27% tokens | None — directly supports |
| A2 Context as Scarce Resource | PARSER (11x latency), MEMO (10-40% fewer tokens), PSMAS (idle compression) | None — all address context scarcity |
| A3 Skill as Production Template | CoSkill, SkillGLoW treat skills as evolving agents | Shifts skill from "template" to "agent" — may need Axiom A3b |
