# Model Reverse Engineering — Cycle 423

**Date**: 2026-09-12
**Focus**: Recent papers on efficient inference, attention, agent coordination
**Sources**: arXiv, GitHub

---

## Paper 1: AGAO — Adaptive Goal-aware Attention Orchestration

**URL**: https://arxiv.org/html/2607.23678v1
**Code**: https://github.com/MingzhouFan97/AGAO
**Date**: 2026-07-26

### Core Contribution
Extends attention from token-level representation learning to **workflow-level agent coordination**. Three complementary attention mechanisms:
1. **Goal-aware Attention**: Semantic alignment between user objectives and agent capabilities
2. **Topology-aware Attention**: Graph structural dependencies in agent workflows
3. **Resource-aware Attention**: Adaptive computational budget allocation

### Key Insight
> "Future agentic systems require not only reasoning capability but also the ability to dynamically allocate computational focus."

### Architecture Mapping to NeoTrix

| AGAO Component | NeoTrix Domain | Pattern |
|----------------|---------------|---------|
| Goal-aware Attention | NT-CORE (GWT) | GWT salience already computes relevance; AGAO adds objective-level scoring |
| Topology-aware Attention | NT-CORE (ConsciousnessTree) | CT branch dependencies = agent graph topology |
| Resource-aware Attention | NT-MIND (SEAL pipeline) | SEAL stage budgets map to resource allocation |
| Adaptive Graph Routing | NT-ACT (orchestration) | Dynamic agent selection per task |

### Absorption Candidates
- **GWT Enhancement**: Add goal-aware scoring layer on top of existing salience computation. Current GWT uses static salience; AGAO shows how to make it objective-dependent.
- **ConsciousnessTree Refinement**: CT branch health scoring could incorporate topology-aware attention — branches with more downstream dependencies get higher attention weight.
- **SEAL Stage Budgeting**: Resource-aware attention maps to SEAL phase token budgets — allocate more tokens to critical phases (e.g., Phase-3 execution vs Phase-0 convergence check).

### Implementation Priority: P1
Direct enhancement to GWT salience computation. Low coupling, high impact.

---

## Paper 2: ReActNet — Inference-Time Graph Engineering

**URL**: https://arxiv.org/abs/2609.05774
**Date**: 2026-09-04

### Core Contribution
Training-free framework that compiles a query and role-specialized agents into a **sequence of directed communication graphs**. Each graph snapshot = one reasoning stage. Edges carry natural-language instructions specifying what message source provides to target.

### Key Insight
> "Effective multi-agent orchestration depends not only on which agents communicate, but also on engineering executable workflow graphs that encode when, why, and how information should flow."

### Architecture Mapping to NeoTrix

| ReActNet Concept | NeoTrix Domain | Pattern |
|------------------|---------------|---------|
| Temporal workflow graphs | NT-CORE (ConsciousnessTree) | CT 6-stage feedback loop = temporal graph snapshots |
| Edge-level communication semantics | NT-MEMORY (KB edges) | KB edges as communication channels between domains |
| Graph compilation → execution separation | NT-MIND (SEAL pipeline) | SEAL design phase → execution phase |
| Message passing state updates | NT-CORE (E8 Hexagram) | Hexagram state transitions = message-passing updates |

### Absorption Candidates
- **SEAL Pipeline Enhancement**: Separate graph compilation (design) from graph execution (runtime). Currently SEAL stages are sequential; ReActNet shows how to make them parallel with explicit message semantics.
- **KB Edge Semantics**: KB edges currently store relations; ReActNet's edge-level instructions suggest enriching edges with execution semantics (what action the edge triggers).
- **ConsciousnessTree as Temporal Graph**: Each CT cycle could be viewed as a graph snapshot where branches are nodes and health signals are edges. This formalizes CT's implicit graph structure.

### Implementation Priority: P2
Requires SEAL pipeline restructuring. High impact but moderate coupling.

---

## Paper 3: Agent-Radar — Attention Steering for Multi-Agent Communication

**URL**: https://arxiv.org/html/2605.30136
**Date**: 2026-05-30

### Core Contribution
Training-free method that **steers each agent's attention** toward relevant context using temporal and spatial decay. Three signals: semantic relevance, spatial distance (graph topology), temporal recency. Uses Selective Prompt Anchoring (SPA) for attention amplification.

### Key Insight
> "Shift from context compression or pruning to selectively steering agents' attention toward relevant context."

### Architecture Mapping to NeoTrix

| Agent-Radar Concept | NeoTrix Domain | Pattern |
|---------------------|---------------|---------|
| Spatial decay (topology) | NT-CORE (GWT) | GWT resonance already encodes spatial relationships between modules |
| Temporal decay | NT-MEMORY (experience decay) | Experience-tree already has time-decay on experience relevance |
| Semantic relevance | NT-WORLD (perception) | PerceptionBridge scores awareness of sensory events |
| SPA attention steering | NT-CORE (E8 reasoning) | E8 hexagram state transitions could use attention steering |

### Absorption Candidates
- **PerceptionBridge Enhancement**: Agent-Radar's spatial+temporal decay model could improve PerceptionBridge's `awareness_score()`. Currently AwarenessBridge uses simple threshold; Agent-Radar shows how to weight by topology distance + recency.
- **Experience Relevance Scoring**: Agent-Radar's joint scoring (semantic × spatial × temporal) could enhance experience-tree's retrieval scoring. Current scoring is primarily semantic + recency; adding spatial (module proximity) would improve relevance.
- **GWT Attention Steering**: Agent-Radar's SPA mechanism could be applied to GWT broadcasting — instead of broadcasting uniformly, steer attention toward goal-relevant specialist modules.

### Implementation Priority: P1
Direct enhancement to PerceptionBridge and experience-tree retrieval. Low coupling, high impact.

---

## Paper 4: SpecBox — Speculative Sandbox Scheduling

**URL**: https://arxiv.org/abs/2607.23933
**Date**: 2026-07-27

### Core Contribution
Runtime for LLM agent serving with **speculative sandbox preallocation**. Keyword matching + streaming semantic embedding for intent-driven prewarming. Overlaps sandbox bootstrapping with model inference. Zero-copy shared-memory transport. Semantic result cache prunes redundant invocations.

### Key Insight
> "Disaggregated sandbox deployment introduces a fundamental tension between resource utilization and interactive tail latency."

### Architecture Mapping to NeoTrix

| SpecBox Concept | NeoTrix Domain | Pattern |
|-----------------|---------------|---------|
| Speculative prewarming | NT-ACT (tool execution) | Tool execution could pre-warm resources based on intent prediction |
| Intent-driven preallocation | NT-WORLD (perception) | PerceptionBridge intent detection → resource prewarming |
| Zero-copy transport | NT-PHYSICAL (embodiment) | Physical layer data transfer optimization |
| Semantic result cache | NT-MEMORY (KB cache) | KB query result caching with semantic dedup |
| Dependency graph prefetching | NT-CORE (GWT) | GWT could prefetch attention resources based on predicted next tasks |

### Absorption Candidates
- **Tool Execution Optimization**: SpecBox's intent-driven prewarming could optimize NT-ACT tool execution — predict which tools will be needed based on current task trajectory and pre-initialize them.
- **KB Query Caching**: Semantic result cache pattern could enhance NT-MEMORY query layer — cache results at semantic level (not just exact query match) to prune redundant KB lookups.
- **GWT Prefetching**: SpecBox's context-aware stochastic prefetching could be applied to GWT — prefetch specialist module activations based on current attention trajectory.

### Implementation Priority: P2
Infrastructure optimization. Moderate coupling, moderate impact.

---

## Paper 5: Codebook Agent — Amortized Topology Design

**URL**: https://arxiv.org/abs/2609.02264
**Date**: 2026-09-02

### Core Contribution
Vector-quantized autoencoder compresses successful topologies into a **query-independent 16-entry codebook**. Reward-weighted MLP maps query embedding to distribution over codes. MLP proxy reranks candidates in single batched forward pass. No iterative search, no message passing at test time.

### Key Insight
> "Topologies that survive a reward filter collapse to about six distinct graphs even when codebook capacity grows from 8 to 64."

### Architecture Mapping to NeoTrix

| Codebook Agent Concept | NeoTrix Domain | Pattern |
|------------------------|---------------|---------|
| Topology codebook | NT-CORE (E8 Hexagram) | E8 64 hexagrams = natural topology codebook |
| Query-to-code routing | NT-CORE (GWT) | GWT salience → module activation pattern |
| Vector-quantized compression | NT-MEMORY (KB embeddings) | KB embedding compression for efficient retrieval |
| Amortized design | NT-MIND (SEAL) | SEAL pattern library = amortized design knowledge |

### Absorption Candidates
- **E8 as Topology Codebook**: Codebook Agent's 16-entry codebook maps directly to E8's 64 hexagrams — each hexagram represents a distinct topology. This formalizes E8's role as a topology library.
- **SEAL Pattern Library**: Codebook Agent's "evidence-backed design rules" (Preserve/Modify/Avoid) map to SEAL's pattern library. Success/failure feedback → pattern evolution.
- **GWT Module Selection**: Codebook Agent's query-to-code routing could optimize GWT — instead of computing salience from scratch each time, route through a learned codebook of activation patterns.

### Implementation Priority: P1
E8 codebook formalization is low-effort, high-impact. Direct enhancement to reasoning engine.

---

## Cross-Paper Synthesis

### Pattern: Attention as Coordination Primitive
All 5 papers treat attention as a coordination mechanism, not just a representation learning tool:
- AGAO: workflow-level attention allocation
- ReActNet: graph-level message attention
- Agent-Radar: context-level attention steering
- SpecBox: inference-level intent attention
- Codebook: topology-level pattern attention

**NeoTrix Implication**: GWT should be the unified attention coordination layer across all 6 layers. Current GWT operates at L5 (consciousness); papers show attention engineering should extend to L1-L4.

### Pattern: Compile-then-Execute Separation
ReActNet and Codebook Agent both separate design (compile) from execution (execute):
- Design: generate topology/graph/plan
- Execute: run with message passing/resource allocation

**NeoTrix Implication**: SEAL pipeline already has design→execute separation; papers validate this and add feedback loops (ReActNet's adaptive routing, Codebook's reward-weighted selection).

### Pattern: Local-First, No-Training Inference
All 5 papers achieve strong results without fine-tuning the base model:
- AGAO: training-free attention orchestration
- ReActNet: training-free graph compilation
- Agent-Radar: training-free context steering
- SpecBox: runtime-only optimization
- Codebook: 16-entry codebook, single forward pass

**NeoTrix Implication**: Reinforces R-P1 (no unsafe code) and the pattern of lightweight runtime layers over frozen base models. NeoTrix's Rust-native approach aligns with this efficiency principle.

### Pattern: Explicit is Better than Implicit
Every paper makes orchestration explicit and inspectable:
- AGAO: attention scores visible per agent
- ReActNet: graph snapshots per reasoning stage
- Agent-Radar: context selection auditable
- SpecBox: prewarming decisions logged
- Codebook: codebook entries inspectable

**NeoTrix Implication**: ConsciousnessTree's 6-stage feedback loop should expose per-stage attention scores and decision rationale. This aligns with NT-SHIELD audit requirements.

---

## Priority Summary

| Paper | Priority | Target Domain | Key Change |
|-------|----------|--------------|------------|
| AGAO | P1 | NT-CORE (GWT) | Goal-aware salience scoring |
| Codebook Agent | P1 | NT-CORE (E8) | E8 as topology codebook |
| Agent-Radar | P1 | NT-MEMORY + NT-WORLD | Multi-signal relevance scoring |
| ReActNet | P2 | NT-MIND (SEAL) | Temporal graph execution |
| SpecBox | P2 | NT-ACT + NT-MEMORY | Intent-driven prewarming |
