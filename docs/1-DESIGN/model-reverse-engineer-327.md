# Model Reverse Engineering — Cycle 327

**Date**: 2026-09-11
**Papers**: 5 recent papers on multi-agent coordination, efficient inference, and self-evolving agent structures

---

## 1. ReActNet: Inference-Time Graph Engineering for Multi-Agent LLM Workflows

**Source**: arXiv:2609.05774, Sep 2026
**Domain**: Multi-agent orchestration

### Core Contribution
Synthesizes **task-conditioned temporal workflow graphs** that jointly specify agent connectivity and edge-level communication semantics. Rather than optimizing a static topology, compiles a query and set of role-specialized agents into a sequence of directed communication graphs.

### Key Mechanisms
1. **Temporal Graph Compilation**: Query + agents → sequence of directed graphs, each snapshot = one reasoning stage
2. **Edge-Level Instructions**: Each edge carries a natural-language instruction specifying what message a source agent provides to a target agent
3. **Separation of Compilation and Execution**: Graph compilation (offline) separated from graph execution (online), making coordination explicit and inspectable
4. **Structured Message Passing**: Agents update reasoning states by integrating previous states with messages from controller-assigned neighbors
5. **Final Aggregator**: Synthesizes resulting agent states into answer

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-ACT** | Temporal graph compilation for task-specific orchestration | `nt_act::orchestrator::temporal_graph` |
| **NT-CORE** | GWT attention as graph snapshot — each reasoning stage broadcasts different salient information | `nt_core_gwt::stage_attention` |
| **NT-GOVERNANCE** | Edge-level instructions = policy enforcement per communication channel | `nt_governance::edge_policy` |
| **NT-MIND** | Temporal graph = SEAL pipeline stage transitions with explicit message semantics | `seal::stage_messages` |

### Actionable Insight
Replace flat GWT broadcast with temporal graph snapshots. Each SEAL pipeline stage compiles a different attention graph — Stage 1 (Soil) broadcasts raw data, Stage 5 (Fruits) broadcasts distilled insights. Edge instructions encode what each domain specialist should contribute at each stage.

---

## 2. Codebook Agent: Amortized Topology Design for Multi-Agent Systems

**Source**: arXiv:2609.02264, Sep 2026
**Domain**: Multi-agent topology optimization

### Core Contribution
Vector-quantized autoencoder compresses successful topologies into a **query-independent 16-entry codebook**. A reward-weighted MLP maps query embedding to code distribution. An MLP proxy reranks candidates in a single batched forward pass. No iterative search, no message passing at test time.

### Key Mechanisms
1. **Topology Codebook**: 16-entry learned codebook from successful topologies — topologies collapse to ~6 distinct graphs regardless of codebook capacity (8→64)
2. **Amortized Design**: Single forward pass generates topology, not iterative search. 2.4ms per topology generation
3. **Edge Count Paradox**: Edge count is negatively correlated with token consumption (r ≈ -0.4) — sparser graphs are MORE expensive because they require more tokens per edge
4. **Adjacency-Invariance**: Message-passing scorer cannot rank candidates when agents share profiles (default benchmark configuration)
5. **Token Reduction**: 21.9-33.2% fewer LLM tokens than prior methods

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-ACT** | Codebook for orchestration topology — pre-learned coordination patterns | `nt_act::orchestrator::codebook` |
| **NT-CORE** | E8 hexagram as topology codebook — 64 hexagrams as learned coordination primitives | `nt_core_e8::hexagram_codebook` |
| **NT-MEMORY** | Codebook as memory-efficient topology representation | `nt_memory::topology_cache` |
| **NT-MIND** | Codebook evolution — SEAL pipeline learns which topologies work for which task classes | `seal::topology_evolution` |

### Actionable Insight
The "topologies collapse to ~6 distinct graphs" finding maps directly to our Constellation maturity model — most coordination patterns are variants of a few archetypes. The edge count paradox warns against naive topology sparsification. The codebook approach could compress our CapabilityRegistry into ~16 canonical coordination patterns, queried by task embedding.

---

## 3. Speculative Macro Commit for Faster Tool-Using Agents

**Source**: arXiv:2609.03236, MLSP 2026
**Domain**: Agent inference acceleration

### Core Contribution
Two-tier agent system: large authoritative actor + fast speculative drafter. Drafter predicts and executes future action chains on isolated environment snapshots. Macro library mines recurring multi-action skeletons from training traces. When actor's next tool call matches drafted action, pre-executed steps commit.

### Key Mechanisms
1. **Macro Library**: Recurring multi-action skeletons extracted from training traces and stored as reusable patterns
2. **Speculative Drafting**: Fast 4B model predicts action chains ahead of authoritative 27B model
3. **Isolated Environment Snapshots**: Drafted actions execute on isolated copies, no pollution of official trajectory
4. **Match-and-Commit**: When actor's first tool call matches draft, all pre-executed steps + observations commit
5. **Latency Reduction**: 10.23% over Speculative Actions baseline, 18.59% over sequential on TeleBench; 44.9% over sequential on AppWorld

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-ACT** | Macro library = skill library for action prediction | `nt_act::macro_library` |
| **NT-PHYSICAL** | Speculative execution on isolated snapshots | `nt_physical::speculative_executor` |
| **NT-MIND** | Macro mining from successful trajectories = SEAL pattern extraction | `seal::macro_extraction` |
| **NT-REPAIR** | Macro commit failure → trigger re-planning | `nt_repair::macro_recovery` |

### Actionable Insight
The macro library concept directly extends our experience-tree — successful multi-step execution traces become speculative macros. The 4B drafter + 27B authority split maps to our cost-aware routing (Axiom A1): cheap model for prediction, expensive model for execution. The isolated snapshot pattern prevents speculative failures from corrupting state.

---

## 4. PARSER: Read in Parallel, Reason in Depth for Long-Context Agents

**Source**: arXiv:2609.06702, Sep 2026
**Domain**: Long-context agent reasoning

### Core Contribution
Decouples reading from reasoning. A bank of lightweight subagents each bound to a single chunk read the entire document in parallel, while a lead agent reasons in depth through iterative scatter-gather rounds.

### Key Mechanisms
1. **Parallel Reading Bank**: Lightweight subagents each handle one chunk, read entire document in parallel
2. **Scatter-Gather Rounds**: Lead agent broadcasts query → subagents return evidence → lead aggregates → deeper follow-up query
3. **Lead Agent Specialization**: All learnable behavior concentrated in lead agent (RL-trained); subagents remain frozen
4. **Robustness to Perturbation**: Robust to evidence position, order, and distance changes — conditions that cause large accuracy swings in sequential methods
5. **Latency Reduction**: Up to 11x inference latency reduction vs sequential approaches

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** | GWT scatter-gather — lead agent broadcasts queries, specialists return evidence | `nt_core_gwt::scatter_gather` |
| **NT-MEMORY** | Parallel KB read with iterative refinement | `nt_memory::parallel_retrieval` |
| **NT-ACT** | Subagent bank for parallel task execution | `nt_act::parallel_subagents` |
| **NT-WORLD** | Parallel web scraping with lead agent synthesis | `nt_world::parallel_fetch` |

### Actionable Insight
PARSER validates our GWT scatter-gather pattern — the lead agent broadcasts attention queries, specialist modules return evidence, lead synthesizes. The "all learnability in lead agent" finding suggests our ConsciousnessTree should be the only RL-trained component, with domain specialists frozen. The 11x latency reduction is relevant to real-time agent coordination.

---

## 5. Procedural Graphs: Self-Evolving Execution Structures for LLM Agents

**Source**: arXiv:2609.09153, Sep 2026
**Domain**: Agent procedural knowledge management

### Core Contribution
Organizes procedural knowledge into (procedure, relation, procedure) triplets — analogous to how knowledge graphs organize factual knowledge. At each decision step, localizes agent's active node and translates surrounding subgraph into step-level situational guidance.

### Key Mechanisms
1. **Procedural Knowledge Graph**: (procedure, relation, procedure) triplets for "what-to-do" questions, complementing (entity, relation, entity) for "what-is" questions
2. **Active Node Localization**: At each step, identify current position in procedural graph
3. **Subgraph-to-Guidance Translation**: Guidance model translates local subgraph neighborhood into situational instructions that bias next action
4. **Self-Evolution via Contrast**: LLM refiner contrasts failed vs successful trajectories, edits graph topology and attributes
5. **Held-Out Validation**: Edits only committed if they preserve/improve held-out validation performance

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-MEMORY** | Procedural knowledge graph for experience organization | `nt_memory::procedural_graph` |
| **NT-CORE** | Active node = current reasoning state in E8 hexagram space | `nt_core_e8::active_node` |
| **NT-MIND** | Self-evolution via trajectory contrast = SEAL stage evaluation | `seal::procedural_evolution` |
| **NT-REPAIR** | Failed trajectory → graph edit → validation = self-healing loop | `nt_repair::procedural_repair` |

### Actionable Insight
The procedural graph concept maps directly to our experience-tree — experiences should be organized as (procedure, relation, procedure) triplets, not flat summaries. The self-evolution via trajectory contrast is exactly our SEAL pipeline's quality gate: compare successful vs failed executions, edit procedural graph, validate on held-out tasks. The "held-out validation before commit" pattern is critical for safe self-evolution.

---

## Cross-Paper Synthesis

| Theme | Papers | NeoTrix Integration |
|-------|--------|---------------------|
| **Temporal Coordination** | ReActNet, Procedural Graphs | SEAL pipeline as temporal graph sequence |
| **Topology Learning** | Codebook Agent, ReActNet | E8 hexagram as learned topology codebook |
| **Speculative Execution** | Speculative Macro Commit | Macro library from experience-tree traces |
| **Parallel Reading** | PARSER, Codebook Agent | GWT scatter-gather with parallel subagent banks |
| **Self-Evolution** | Procedural Graphs, Codebook Agent | Trajectory contrast → graph edit → held-out validation |

## Priority Absorption Matrix

| Paper | Pattern | Priority | Integration Point | Difficulty |
|-------|---------|----------|-------------------|------------|
| ReActNet | Temporal graph compilation | P1 | GWT stage-conditional attention | Medium |
| Codebook Agent | Topology codebook | P1 | E8 hexagram codebook compression | Medium |
| Speculative Macro Commit | Macro library | P1 | Experience-tree macro extraction | Low |
| PARSER | Parallel scatter-gather | P2 | GWT lead-subagent coordination | Medium |
| Procedural Graphs | Procedural knowledge graph | P1 | Experience-tree procedural organization | Low |
