# Model Reverse Engineering — Cycle 443

**Date**: 2026-09-12
**Source**: arXiv (Sep 2026), ICLR 2026, EMNLP 2026, ICML 2026, WWW 2026, ACL 2026
**Focus**: Efficient inference, attention mechanisms, agent coordination, reasoning frameworks

---

## Model 1: Procedural Graphs (arXiv:2609.09153, Sep 2026)

### Paper
**"Procedural Graphs: Self-Evolving Execution Structures for LLM Agents"**

### Core Insight
Agents lose track of objectives as trajectories lengthen because procedural knowledge (what-to-do) is implicit in generation. Procedural Graphs make it explicit: organize procedures into `(procedure, relation, procedure)` triplets — analogous to knowledge graphs for facts but for action sequences.

### Architecture
```
Procedure Graph: (procedure_node) →[prerequisite]→ (procedure_node)
                                        ↓[condition]
                                  (procedure_node)

At each step:
1. Localize active node in graph
2. Guidance model translates subgraph → situational guidance
3. Bias solver's next action (not dictate)
4. Self-evolution: LLM refiner contrasts failed vs successful trajectories
5. Edits graph topology, retaining rejected edits to prevent repetition
```

### Key Mechanisms
- **Graph-as-Execution-Prior**: Instead of unconstrained generation, agent follows graph structure
- **Self-Evolution Loop**: Failed trajectories → graph edits → validate on held-out set → commit if improved
- **Minimal Skeleton Bootstrap**: Start from minimal graph, build up through execution traces
- **Subgraph Guidance**: At each step, the local subgraph provides situational context

### Results
- Matches or surpasses hand-designed graphs across multiple datasets
- Can repair flawed expert priors
- Consistent gains over memory-based baselines

### NeoTrix Mapping

| NeoTrix Component | Procedural Graph Pattern | Integration |
|-------------------|------------------------|-------------|
| **ConsciousnessTree** | Execution graph as meta-cognition scaffold | C6: ConsciousnessTree could evolve execution graphs for SEAL pipeline stages |
| **SEAL Pipeline** | Self-evolving stage transitions | Each SEAL phase (Soil→Roots→Trunk→Branches→Fruits→Core) could be a graph node with prerequisites |
| **NT-MIND** | Failed trajectory → graph edit | SEAL cycle failure → modify pipeline graph topology (not just parameters) |
| **NT-ACT** | Procedure graph for tool orchestration | Multi-step tool calls organized as prerequisite graph, not flat sequence |
| **GWT** | Subgraph-guided attention | At each reasoning step, active graph neighborhood biases attention routing |

**Priority**: P0 — Direct mapping to SEAL pipeline evolution and ConsciousnessTree stage transitions.

---

## Model 2: PARSER (arXiv:2609.06702, Sep 2026)

### Paper
**"PARSER: Read in Parallel, Reason in Depth for Long-Context LLM Agents"**

### Core Insight
Sequential memory agents couple document traversal to reasoning depth. PARSER decouples them: subagents read chunks in parallel while a lead agent reasons in depth through iterative scatter-gather rounds.

### Architecture
```
Document → N subagents (each bound to one chunk)
                ↓
    Lead Agent broadcasts query to ALL subagents
                ↓
    Subagents return evidence (parallel)
                ↓
    Lead Agent aggregates → formulates deeper follow-up query
                ↓
    Repeat for K rounds (scatter-gather)
                ↓
    Final answer with depth > sequential reading
```

### Key Mechanisms
- **Decoupled Read/Reason**: Reading (parallel subagents) separate from reasoning (lead agent)
- **Iterative Scatter-Ground**: Each round: broadcast query → aggregate evidence → deeper query
- **Frozen Subagents + Trained Lead**: Subagents are off-the-shelf; only lead agent optimized via RL
- **Position-Invariant**: Robust to evidence placement, order, distance

### Results
- 4B backbone beats strongest sequential baseline by 5.7 points avg, 12.0 points at 896K tokens
- 9B backbone surpasses DeepSeek-V4-Pro by 6.3 points
- 11x latency reduction vs sequential methods

### NeoTrix Mapping

| NeoTrix Component | PARSER Pattern | Integration |
|-------------------|---------------|-------------|
| **NT-MEMORY KB** | Parallel chunk reading | KB query could dispatch subagents per partition, lead agent synthesizes |
| **GWT** | Scatter-gather attention | GWT broadcast → specialist modules return evidence → consciousness synthesizes |
| **NT-WORLD** | Parallel document extraction | Crawl pipeline could parallelize chunk extraction with lead agent reasoning |
| **ConsciousnessTree** | Lead agent = consciousness | ConsciousnessTree acts as lead agent; 7 domains act as parallel subagents |
| **Axiom A2** | Position-invariant retrieval | Evidence placement doesn't affect reasoning quality — critical for long sessions |

**Priority**: P0 — Scatter-gather pattern directly maps to GWT broadcast + specialist module coordination.

---

## Model 3: ReActNet (arXiv:2609.05774, Sep 2026)

### Paper
**"Inference-Time Graph Engineering for Multi-Agent LLM Workflows"**

### Core Insight
Effective multi-agent orchestration depends not only on WHICH agents communicate but on engineering WHEN, WHY, and HOW information flows. ReActNet compiles task → temporal workflow graph with edge-level communication semantics.

### Architecture
```
Query + Role-Specialized Agents
        ↓
Graph Compilation:
  Stage 1: Directed graph (who talks to whom)
  Stage 2: Edge semantics (WHAT message source → target)
  Stage 3: Temporal sequence (graph snapshots per reasoning stage)
        ↓
Execution via structured message passing:
  Agent state = prev_state + messages_from_neighbors
        ↓
Final answer
```

### Key Mechanisms
- **Graph Compilation ≠ Graph Execution**: Separate compilation (offline) from execution (online)
- **Edge-Level Semantics**: Each edge carries natural-language instruction for message content
- **Task-Conditioned Temporal Graph**: Different graph topology per reasoning stage
- **Training-Free**: No RL or gradient optimization needed

### Results
- Consistently improves over fixed-topology and learned-topology baselines
- Competitive inference cost
- Works across knowledge reasoning, math, code generation, GAIA tasks

### NeoTrix Mapping

| NeoTrix Component | ReActNet Pattern | Integration |
|-------------------|-----------------|-------------|
| **GWT** | Temporal attention routing | GWT could dynamically reconfigure broadcast topology per reasoning stage |
| **NT-ACT** | Edge-level communication specs | Agent-to-agent messages carry structured intent, not just raw output |
| **ConsciousnessTree** | Stage-specific graph topology | Each consciousness stage (Soil→Core) uses different inter-domain wiring |
| **NT-META** | Separation of compilation/execution | Meta-cognition compiles strategy; execution follows compiled graph |
| **Dual Specialization** | Weapon Set graph switching | Weapon Set I (acquisition) vs II (evolution) = different temporal graphs |

**Priority**: P0 — Separation of graph compilation from execution is the missing pattern in NeoTrix's current GWT design.

---

## Model 4: NeuralFSM (ACL 2026)

### Paper
**"NeuralFSM: Adaptive Multi-Agent Coordination via Learning Finite-State Execution Policy"**

### Core Insight
Multi-agent coordination can be formulated as a finite-state execution process. NeuralFSM learns both state transition distribution and inter-agent communication weights from interaction traces, using a temporal coordination controller.

### Architecture
```
Finite State Machine (FSM) Backbone:
  States = coordination phases (explore → converge → verify → output)
  Transitions = learned probabilistic distribution

Temporal Coordination Controller (TGN):
  Input: task representation + execution context
  Output: state transition + communication routing weights

Dual-Defense Protection:
  Training: graph regularization (anomalous communication penalty)
  Runtime: trust-aware message attenuation (centrality + anomaly score)
```

### Key Mechanisms
- **FSM as Coordination Skeleton**: Reusable across tasks; transitions learned per-task
- **Temporal Graph Networks**: Dynamic interaction graph where nodes=agents, edges=communications
- **Trust-Aware Attenuation**: Betweenness centrality + PageRank → trust score → message weight
- **Sparse Routing**: Only active agents communicate; others stay silent

### Results
- 6.74%–19.39% improvement over baselines across 6 benchmarks
- Reduced token consumption via sparse routing
- 1.82% performance drop under adversarial attack (with protection layer)

### NeoTrix Mapping

| NeoTrix Component | NeuralFSM Pattern | Integration |
|-------------------|------------------|-------------|
| **ConsciousnessTree** | FSM backbone for 6-stage cycle | Soil→Roots→Trunk→Branches→Fruits→Core = FSM states with learned transitions |
| **GWT** | Trust-aware attention routing | Module trust scores modulate broadcast weight (not binary attention) |
| **NT-SHIELD** | Dual-defense protection | Training-time regularization + runtime trust attenuation for adversarial robustness |
| **NT-ACT** | Sparse agent routing | Only active domains communicate; silent domains save tokens |
| **NT-FEEL** | Centrality-based trust | Module centrality in interaction graph = trust signal for coordination |

**Priority**: P1 — FSM backbone for ConsciousnessTree + trust-aware GWT are high-value but require architectural adaptation.

---

## Model 5: CEDAR (arXiv:2609.07237, Sep 2026)

### Paper
**"CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention"**

### Core Insight
Hard sparse attention (route query to subset of tokens) fails when routing misses salient chunks. CEDAR adds a residual attention path: each chunk contributes a cheap KV summary; chunks with high approximation error are expanded to exact attention. Error-bounded refinement budget allocation.

### Architecture
```
For each query q:
1. Compute KV summary per semantic chunk (cheap)
2. Estimate within-chunk key/value dispersion → error bound
3. Allocate refinement budget: high-error chunks → exact attention
4. Low-error chunks → keep summarized (residual path)
5. Combine exact + summarized in single softmax normalization
6. Total: ~3x kernel speedup at 128K context
```

### Key Mechanisms
- **Residual Attention Path**: Summarized chunks contribute to softmax (not dropped)
- **Error-Bounded Budget**: Variable refinement based on within-chunk dispersion
- **Single Normalization**: Exact + summarized combined in one softmax (no duplication)
- **Coarse-to-Fine**: Start with all summarized, refine only where needed

### Results
- 98%+ reduction in reconstruction error vs hard dropping at equal budgets
- ~3x kernel speedup at 128K context
- Preserves quality lost by hard sparse routing

### NeoTrix Mapping

| NeoTrix Component | CEDAR Pattern | Integration |
|-------------------|--------------|-------------|
| **GWT** | Error-bounded attention routing | GWT could allocate attention budget based on information-theoretic error, not just salience |
| **NT-CORE kv_cache** | Residual KV summaries | KB nodes could maintain summarized embeddings; high-error (high-information) nodes expanded on-demand |
| **Axiom A2** | Error-bounded context budget | Context window allocated by approximation error, not uniform token counting |
| **VSA HyperCube** | Within-chunk dispersion | Concept clusters with high internal variance get more attention budget |
| **ConsciousnessTree** | Coarse-to-fine attention | Consciousness starts with broad summaries, refines where uncertainty is high |

**Priority**: P0 — Error-bounded attention is the mathematical foundation for Axiom A2 (context as scarce resource). Direct implementation path.

---

## Cross-Model Pattern Synthesis

### 1. Graph-Structured Execution Is Universal
Procedural Graphs (execution), ReActNet (communication), NeuralFSM (coordination) all use graph structures. The graph is NOT static — it evolves per task or per reasoning stage. NeoTrix's ConsciousnessTree + GWT should adopt this: static tree topology + dynamic attention wiring.

### 2. Decouple Reading from Reasoning
PARSER's scatter-gather + CEDAR's residual summaries both decouple information gathering from information synthesis. The lead agent (consciousness) reasons in depth while subagents (domains) read in parallel. This is the production architecture for NeoTrix's 7-domain coordination.

### 3. Error-Bounded Attention Allocation
CEDAR's within-chunk dispersion → error bound → variable refinement budget is the missing mathematical framework for GWT. Instead of binary attention (salient/not-salient), allocate budget proportional to information-theoretic uncertainty. This operationalizes Axiom A2.

### 4. Trust-Aware Coordination
NeuralFSM's dual-defense (training regularization + runtime trust attenuation) addresses a gap NeoTrix hasn't tackled: adversarial agents or degraded modules. NT-SHIELD should adopt trust scores modulating inter-module communication.

### 5. Self-Evolution Through Graph Editing
Procedural Graphs' self-evolution (failed trajectory → graph topology edit) is the pattern for SEAL pipeline evolution. Instead of just tuning parameters, modify the pipeline's execution graph structure when failures occur.

---

## Integration Roadmap

### Phase 1: Foundation (Immediate)
| Pattern | Paper | Implementation | Effort |
|---------|-------|---------------|--------|
| Error-bounded attention | CEDAR | GWT salience scoring with dispersion metric | 2 weeks |
| Scatter-gather GWT | PARSER | Parallel domain queries + consciousness synthesis | 3 weeks |
| Graph compilation/execution split | ReActNet | GWT topology compiled per task, executed per step | 2 weeks |

### Phase 2: Evolution (1-2 months)
| Pattern | Paper | Implementation | Effort |
|---------|-------|---------------|--------|
| Procedural Graphs for SEAL | Procedural Graphs | SEAL pipeline as self-evolving execution graph | 4 weeks |
| FSM Consciousness | NeuralFSM | ConsciousnessTree stages as learned FSM transitions | 3 weeks |
| Trust-aware GWT | NeuralFSM | Module trust scores modulate attention weight | 2 weeks |

### Phase 3: Integration (2-3 months)
| Pattern | Paper | Implementation | Effort |
|---------|-------|---------------|--------|
| Parallel KB reading | PARSER | NT-MEMORY subagent parallelization | 3 weeks |
| Residual KV summaries | CEDAR | KB node summarization + expansion on-demand | 4 weeks |
| Graph-edited evolution | Procedural Graphs | SEAL pipeline topology self-modification | 4 weeks |

---

## Axiom Implications

| Axiom | New Validation | Refinement |
|-------|---------------|------------|
| **A1 Cost-Aware Routing** | ponytail (-54% code), PARSER (11x latency) | Cost-awareness applies to code generation AND attention allocation |
| **A2 Context as Scarce Resource** | CEDAR (error-bounded), OpenAI Agents API (compaction) | Context budget = information-theoretic error, not token count |
| **A3 Skill as Production Template** | Webwright Skill Factory, ponytail skill levels | Skills are parameterized programs, not prompts |
| **NEW: A4 Graph-Structured Execution** | Procedural Graphs, ReActNet, NeuralFSM | Agent execution follows explicit graph structure, not implicit generation |
| **NEW: A5 Trust-Aware Coordination** | NeuralFSM dual-defense | Inter-module trust scores modulate communication weight |

---

## Cross-Domain Energy Flow

```
CEDAR (error-bounded attention)
    ↓ provides mathematical framework for
GWT (attention routing)
    ↓ modulated by
NeuralFSM (trust-aware coordination)
    ↓ organized as
ReActNet (temporal workflow graphs)
    ↓ executed via
PARSER (scatter-gather reasoning)
    ↓ evolved through
Procedural Graphs (self-evolving execution structures)
```

This forms a complete attention→coordination→execution→evolution stack that directly maps to NeoTrix's 6-layer architecture.
