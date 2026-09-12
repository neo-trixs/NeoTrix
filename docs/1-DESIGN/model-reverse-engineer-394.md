# Model Reverse Engineering — Cycle 394 (2026-09-12)

## Papers & Models Analyzed

### 1. AGAO: Adaptive Goal-aware Attention Orchestration
**Paper**: arXiv:2607.23678 (Jul 2026)
**Core Idea**: Extend Transformer attention from token-level to workflow-level agent coordination.

**Three attention mechanisms**:
1. **Goal-aware attention**: Semantic relevance between user goals and agent capabilities
2. **Topology-aware attention**: Structural dependencies in agent graphs
3. **Resource-aware attention**: Budget allocation across heterogeneous agents

**NeoTrix Mapping**:
- **NT-CORE**: Direct maps to GWT salience calculation. AGAO's goal-aware attention = GWT's salience scoring with explicit goal vector. Can replace current GWT salience heuristic with AGAO's 3-component scoring.
- **NT-MIND**: Topology-aware attention = SEAL pipeline stage dependency modeling. Currently SEAL stages are sequential; AGAO suggests graph-based parallel execution with dependency-aware scheduling.
- **NT-ACT**: Resource-aware attention = capability registry load balancing. Current CapabilityRegistry uses static weights; AGAO proposes dynamic allocation based on computational constraints.

**Actionable**: Implement AGAO's 3-component attention scoring as `GwtAttentionScorer` trait, replacing current `salience()` function.

---

### 2. Agent-Radar: Attention Steering for Multi-Agent Communication
**Paper**: arXiv:2605.30136 (May 2026)
**Core Idea**: Score sentence-level context by semantic relevance weighted with temporal and spatial decay, then steer agent attention toward selected context during inference.

**Key Innovation**: Preserves full transcript and topology, but selectively attends to relevant history instead of compressing/rewriting context.

**NeoTrix Mapping**:
- **NT-CORE**: Agent-Radar's attention steering = GWT broadcast filtering. Currently GWT broadcasts all salient info; Agent-Radar suggests steering attention toward specific context regions. Can implement as `AttentionSteering` layer between GWT and specialist modules.
- **NT-MEMORY**: Temporal + spatial decay weighting for memory retrieval. Current KB query is BM25 + vector; can add decay weighting to prioritize recent and contextually close memories.
- **NT-NEXUS**: Cross-session memory relevance scoring. Agent-Radar's topology preservation maps to experience-tree branch weighting.

**Actionable**: Add `temporal_decay` and `spatial_decay` weights to KB query scoring in `nt_memory_search`.

---

### 3. PackInfer: Compute- and I/O-Efficient Batched Attention
**Paper**: arXiv:2602.06072 (Feb 2026)
**Core Idea**: Kernel-level attention framework for heterogeneous batched inference. Load-balanced execution groups, packed query-key regions, I/O-aware KV cache layout.

**Results**: 13-20% latency reduction, 20% throughput improvement over FlashAttention.

**NeoTrix Mapping**:
- **NT-IO**: PackInfer's batching strategy = provider-level request batching. Currently NeoTrix dispatches LLM calls individually; PackInfer shows batched execution groups can reduce latency. Can implement in `nt_io_llm` as request coalescing.
- **NT-PHYSICAL**: I/O-aware KV cache layout = GPU memory management for local inference. Relevant for oMLX integration and future on-device inference.

**Actionable**: Implement `BatchedInferenceManager` in nt_io for provider-level request batching when multiple concurrent LLM calls target the same provider.

---

### 4. AgentInfer: Co-Design of Inference Architecture and System
**Paper**: arXiv:2512.18337 (Dec 2025, revised Feb 2026)
**Core Idea**: Four synergistic components for end-to-end agent acceleration:
1. **AgentCollab**: Hierarchical dual-model reasoning (large + small model via dynamic role assignment)
2. **AgentSched**: Cache-aware hybrid scheduler
3. **AgentSAM**: Suffix-automaton speculative decoding reusing multi-session semantic memory
4. **AgentCompress**: Semantic compression that distills agent memory asynchronously

**Results**: >50% reduction in ineffective tokens, 1.8-2.5× speedup.

**NeoTrix Mapping**:
- **NT-CORE**: AgentCollab's dynamic role assignment = GWT's model routing (Axiom A1: Cost-Aware Routing). Large model for hard reasoning, small model for I/O tasks.
- **NT-MIND**: AgentSAM's semantic memory reuse = experience-tree's branch loading. Can cache successful SEAL pipeline runs as suffix patterns for speculative re-execution.
- **NT-MEMORY**: AgentCompress's async distillation = experience-tree's distillation phase. Currently synchronous; AgentInfer shows async compression improves throughput.

**Actionable**: Implement `DualModelRouter` in nt_core that dynamically assigns tasks to large/small models based on complexity estimation.

---

### 5. Learning from Failure: Inference-Time Self-Improvement
**Paper**: arXiv:2606.31270 (Jun 2026)
**Core Idea**: Failure-driven self-improvement loop. LLM diagnoses failure modes → proposes inference-time solutions → generates code patches (lightly verified by humans) → upgrades agent without retraining.

**Results**: 42.3% → 48.9% success rate on OSWorld (6.6pp gain) with zero training cost.

**NeoTrix Mapping**:
- **NT-MIND**: Failure diagnosis = SEAL pipeline's Phase-5 feedback. Currently feedback is binary (success/fail); this paper proposes granular failure mode classification.
- **NT-REPAIR**: Code patch generation for self-repair. Maps to NT-REPAIR's self-healing loop but at inference time (not training time).
- **NT-META**: Meta-learning from failure patterns across sessions. Failure modes become meta-patterns stored in KB.

**Actionable**: Add `FailureModeClassifier` to SEAL pipeline that categorizes failures into actionable types (reasoning error, tool misuse, context loss, etc.) and generates targeted patches.

---

## Cross-Paper Synthesis

| Pattern | Papers | NeoTrix Integration |
|---------|--------|---------------------|
| **Attention as Resource Allocation** | AGAO + Agent-Radar | GWT salience upgrade: 3-component scoring + temporal/spatial decay |
| **Memory as Computational Substrate** | AgentInfer + Agent-Radar | experience-tree: async distillation + suffix-based speculative reuse |
| **Failure as Learning Signal** | Learning from Failure + Reflexio (tool) | SEAL Phase-5: granular failure classification + code patch generation |
| **Batching as Infrastructure** | PackInfer + AgentInfer | nt_io: provider-level request coalescing + cache-aware scheduling |
| **Dual-Model Efficiency** | AgentInfer + AGAO | GWT: dynamic large/small model routing per task complexity |

## Priority Actions

1. **P0**: Implement AGAO's 3-component attention scoring for GWT salience
2. **P0**: Add temporal/spatial decay to KB memory retrieval
3. **P1**: Implement `DualModelRouter` for cost-aware model selection (Axiom A1)
4. **P1**: Add `FailureModeClassifier` to SEAL pipeline feedback
5. **P2**: Implement provider-level request batching in nt_io
6. **P2**: Async memory distillation in experience-tree
