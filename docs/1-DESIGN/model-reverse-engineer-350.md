# Model Reverse Engineering — Cycle 350 (2026-09-12)

## Methodology

Selected 5 recent papers (Jun–Sep 2026) on efficient inference, attention mechanisms, and agent coordination. Mapped patterns to NeoTrix 7 domains using axioms A1 (Cost-Aware Routing), A2 (Context as Scarce Resource), A3 (Skill as Production Template).

---

## Paper 1: AGAO — Adaptive Goal-aware Attention Orchestration

**arXiv**: 2607.23678 (Jul 2026)
**Authors**: Mingzhou Fan et al.
**Code**: https://github.com/MingzhouFan97/AGAO

### Core Innovation

Extends attention from token-level representation learning to **workflow-level agent coordination**. Three complementary attention mechanisms:

1. **Goal-aware Attention**: Semantic relevance between user objectives and agent capabilities
2. **Topology-aware Attention**: Structural dependencies within agent graphs
3. **Resource-aware Attention**: Dynamic allocation of computational budgets and execution priorities

Key insight: Agents as **dynamically selectable computational units** rather than fixed workflow operators. Attention becomes an **execution-level control mechanism**.

### NeoTrix Mapping

| AGAO Component | NeoTrix Domain | Mapping |
|----------------|----------------|---------|
| Goal-aware Attention | NT-CORE (GWT) | GWT salience scoring — measure semantic alignment between task goal and module capabilities |
| Topology-aware Attention | NT-CORE (E8) | E8 Hexagram graph structure — execution paths through reasoning state space |
| Resource-aware Attention | NT-ACT (Cost-Aware) | Axiom A1: cost-weighted routing. Token budget allocation per module based on attention scores |
| Adaptive Graph Routing | NT-MIND (SEAL) | SEAL pipeline stage activation — dynamically enable/disable stages based on task demands |

### Integration Pattern

```
Task Goal → Goal-aware Attention (semantic scoring)
         → Topology-aware Attention (graph dependencies)
         → Resource-aware Attention (budget allocation)
         → Dynamic agent activation
```

This maps directly to GWT refinement: instead of static salience, use **goal-conditioned attention** to route across NT-* domains.

### Key Result

AGAO improves task effectiveness while reducing unnecessary computation, latency, and token consumption. On MBPP: pass@1 from 0.875 to 1.000 with goal scoring enabled.

---

## Paper 2: ReActNet — Inference-Time Graph Engineering

**arXiv**: 2609.05774 (Sep 2026)
**Authors**: Not specified in abstract

### Core Innovation

Training-free framework that **compiles** a query and role-specialized agents into a **sequence of directed communication graphs**. Each graph snapshot = one reasoning stage. Each edge = natural-language instruction specifying message content.

Key insight: **Separates graph compilation from graph execution**. Coordination becomes explicit, inspectable, and task-conditioned without RL or gradient-based topology optimization.

### NeoTrix Mapping

| ReActNet Component | NeoTrix Domain | Mapping |
|--------------------|----------------|---------|
| Query → Graph Compilation | NT-CORE (E8) | Query interpretation → Hexagram state selection → execution path |
| Role-specialized Agents | NT-* Domains | Each NT-* domain as a specialized agent with defined role |
| Edge Instructions | NT-ACT (MCP) | PTC: typed tool invocation with explicit message contracts |
| Temporal Graph Sequence | NT-MIND (SEAL) | SEAL pipeline as temporal graph — each phase = graph snapshot |
| Final Aggregator | NT-CORE (GWT) | GWT broadcast: aggregate reasoning states into coherent output |

### Integration Pattern

```
Query → Compile temporal graph (no training)
      → Execute structured message passing
      → Agents update states from controller-assigned neighbors
      → Final aggregator synthesizes answer
```

This validates our SEAL pipeline architecture: each stage is a graph snapshot, edges carry typed messages, final stage aggregates.

### Key Result

Consistently improves over fixed-topology and learned-topology baselines while maintaining competitive inference cost. Training-free.

---

## Paper 3: Codebook Agent — Amortized Topology Design

**arXiv**: 2609.02264 (Sep 2026)
**Authors**: Not specified in abstract

### Core Innovation

Vector-quantized autoencoder compresses successful topologies into a **query-independent 16-entry codebook**. At test time: query embedding → distribution over codes → MLP proxy reranks top candidates in single forward pass. No iterative search, no message passing at test time.

Key insights:
- Topologies collapse to ~6 distinct graphs even with 64-entry codebook
- Edge count **negatively correlated** with token consumption (sparser = more expensive)
- Message-passing scorer is **adjacency-invariant** when agents share profiles

### NeoTrix Mapping

| Codebook Agent Component | NeoTrix Domain | Mapping |
|--------------------------|----------------|---------|
| 16-entry Codebook | NT-CORE (E8) | E8 Hexagram as fixed topology codebook — 64 hexagrams = reusable execution patterns |
| Query → Code Distribution | NT-CORE (GWT) | GWT attention: query selects which hexagram/codebook entry to activate |
| MLP Reranker | NT-MIND (Skill Crystallization) | Skill selection: map task type to optimal execution topology |
| Token Efficiency | Axiom A2 | Context as Scarce Resource — minimize token consumption through topology optimization |

### Integration Pattern

```
Query → Embed → Sample from 16-entry codebook
      → MLP reranks top-k candidates (single forward pass)
      → Execute selected topology
      → 21.9-33.2% fewer LLM tokens
```

This directly implements our Rune Socketing concept: fixed configuration slots (16 entries) that produce emergent effects (topology) based on input. The "codebook" is our Constellation maturity ladder compressed into a lookup table.

### Key Result

84.6 average accuracy vs 83.0 for strongest prior. Topology emitted in 2.4ms. 21.9-33.2% fewer tokens.

---

## Paper 4: SANTA — Stochastic Sparse Attention

**arXiv**: 2605.01910v2 (May 2026)
**Authors**: Kyle Lee, Corentin Delacour et al. (OPUSLab)
**Code**: https://github.com/OPUSLab/SANTA.git

### Core Innovation

**Stochastic Additive No-mulT Attention**: Sparsifies value-cache access by sampling S << n_k indices from post-softmax distribution. Replaces value-stage multiply-accumulates with **gather-and-add**. Unbiased estimator of post-softmax value aggregation.

Complementary technique: **Bernoulli qK^T sampling** for score-stage sparsification.

Key insight: Autoregressive decoding becomes bandwidth-limited at long contexts. SANTA reduces memory-bound latency without accuracy loss.

### NeoTrix Mapping

| SANTA Component | NeoTrix Domain | Mapping |
|-----------------|----------------|---------|
| Stochastic KV Sampling | NT-CORE (HyperCube) | VSA embedding sparsification — select high-signal dimensions, skip noise |
| Gather-and-Add | NT-MEMORY | KB embedding retrieval — sparse access patterns for large knowledge stores |
| qK^T Score Sparsification | NT-CORE (GWT) | GWT attention sparsification — only compute attention for salient modules |
| Memory-Bound Optimization | Axiom A2 | Context as Scarce Resource — bounded GPU memory regardless of workspace size |

### Integration Pattern

```
Long context → Sample S indices from post-softmax distribution
             → Gather-and-add (no multiply-accumulate)
             → Bernoulli qK^T for score-stage sparsification
             → 1.5x decode-step attention speedup
             → 1.25x end-to-end decode latency improvement
```

This directly optimizes our HyperCube access patterns. For large KB embeddings, stochastic sparse access reduces memory pressure while maintaining retrieval quality.

### Key Result

Matches baseline accuracy. Up to 1.5x attention kernel speedup. Up to 1.25x end-to-end decode latency improvement on RTX 6000 Ada.

---

## Paper 5: PSMAS — Phase-Scheduled Multi-Agent Systems

**arXiv**: 2604.17400 (Apr 2026)
**Authors**: Mohit Dubey (Open Gigantic)

### Core Innovation

Reconceptualizes agent activation as **continuous control over shared attention space** modeled on circular manifold S¹. Each agent assigned fixed angular phase θ_i from task dependency topology. Global sweep signal φ(t) activates only agents within angular window ε.

Key insight: Scheduling and compression are **independent sources of gain**. Scheduling alone accounts for 18-20pp of token reduction.

### NeoTrix Mapping

| PSMAS Component | NeoTrix Domain | Mapping |
|-----------------|----------------|---------|
| Circular Manifold S¹ | NT-CORE (E8) | E8 Hexagram as circular reasoning space — periodic recurrence for iterative pipelines |
| Phase Assignment | NT-CORE (GWT) | GWT attention routing — agents assigned phases based on role in attention hierarchy |
| Sweep Signal | NT-MIND (SEAL) | SEAL pipeline cycle — sweep activates stages in dependency order |
| Angular Window ε | NT-CORE (ConsciousnessTree) | ConsciousnessTree activation window — which branches are active in current cycle |
| Idle Agent Compression | NT-MEMORY | Experience-tree lazy loading — compressed summaries for inactive knowledge branches |

### Integration Pattern

```
Agent dependency DAG → Assign phases on S¹ (topological sort)
                     → Global sweep φ(t) = (ωt) mod 2π
                     → Only agents within ε of sweep are active
                     → Active: full context; Idle: compressed summary (α ≈ 0.12)
                     → 27.3% mean token reduction (21.4-34.8% range)
```

This implements our experience-tree lazy branch loading: inactive branches get compressed summaries, active branches get full context. The circular manifold ensures periodic recurrence without restart overhead.

### Key Result

27.3% mean token reduction. Performance within 2.1pp of full activation baseline. Scheduling alone = 18-20pp reduction.

---

## Cross-Paper Synthesis

### Pattern Convergence

| Pattern | Papers | NeoTrix Implementation |
|---------|--------|----------------------|
| **Attention as Control** | AGAO, PSMAS | GWT salience as execution-level control, not just representation |
| **Topology as Skill** | Codebook Agent, ReActNet | E8 Hexagram as reusable topology codebook |
| **Sparse Access** | SANTA, Codebook Agent | HyperCube sparse retrieval, KV cache optimization |
| **Phase-based Activation** | PSMAS, AGAO | ConsciousnessTree cycle activation, SEAL stage scheduling |
| **Training-free Coordination** | ReActNet, Codebook Agent | Static topology lookup + query-conditioned selection |

### NeoTrix Axiom Validation

| Axiom | Validation |
|-------|------------|
| **A1: Cost-Aware Routing** | AGAO resource-aware attention + PSMAS 27.3% token reduction + Codebook Agent 21.9-33.2% fewer tokens |
| **A2: Context as Scarce Resource** | SANTA memory-bound optimization + PSMAS idle compression (α≈0.12) |
| **A3: Skill as Production Template** | Codebook Agent 16-entry codebook + ReActNet graph compilation |

### Implementation Priorities

| Priority | Pattern | Target | Estimated Gain |
|----------|---------|--------|----------------|
| P0 | Goal-aware attention scoring | GWT refinement | 15-20% better task routing |
| P0 | Circular manifold activation | ConsciousnessTree | 27% token reduction |
| P1 | Topology codebook | E8 Hexagram | 22-33% fewer tokens |
| P1 | Stochastic sparse KV | HyperCube retrieval | 1.5x attention speedup |
| P2 | Graph compilation from query | SEAL pipeline | Training-free adaptation |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| Codebook Agent: sparser topology = MORE tokens (Pearson r ≈ -0.4) | Counter-intuitive but real: sparse graphs require more inter-agent messages. Use PSMAS phase scheduling to compensate. |
| SANTA: stochastic = unbiased but high variance | Stratified + systematic sampling variants reduce variance. For production: use systematic sampling. |
| AGAO: goal-aware vs topology-aware tradeoff | On MBPP, removing goal scoring improves pass@1 but hurts HotpotQA. Task-type routing needed (our Dual Specialization pattern). |
| PSMAS: circular manifold is necessary topology | Real line R lacks periodicity; discrete ring Z/nZ lacks differentiability. S¹ is unique topology satisfying all three requirements. |
