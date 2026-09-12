# Model Reverse Engineering — Cycle 364 (2026-09-12)

## 5 New Papers/AI Models — Pattern Extraction & NeoTrix Domain Mapping

---

### Paper 1: ReActNet — Inference-Time Graph Engineering for Multi-Agent LLM Workflows
- **URL**: https://arxiv.org/abs/2609.05774
- **Date**: Sep 4 2026
- **Core Idea**: Synthesize task-conditioned temporal workflow graphs that jointly specify agent connectivity AND edge-level communication semantics. Separates graph compilation from graph execution.
- **Key Mechanism**: Each graph snapshot = one reasoning stage. Each edge carries a natural-language instruction specifying what message source→target should provide. Training-free — no RL or gradient-based topology optimization.
- **Results**: Consistently improves over fixed-topology and learned-topology baselines across knowledge reasoning, math, code, and GAIA tasks.
- **NeoTrix Mapping**:
  - **NT-CORE** (E8 Hexagram): ReActNet's temporal graph snapshots map to E8's hexagram states. Each hexagram could specify communication semantics between specialist modules.
  - **NT-ACT**: Graph compilation ↔ SEAL pipeline stage orchestration. Dynamic graph per task vs static pipeline.
  - **GWT**: "When, why, and how information flows" is exactly GWT attention routing. ReActNet makes this explicit and inspectable.

---

### Paper 2: AGAO — Adaptive Goal-aware Attention Orchestration for Multi-Agent Graphs
- **URL**: https://arxiv.org/abs/2607.23678
- **Date**: Jul 26 2026
- **Core Idea**: Extend attention from token-level to workflow-level. Three complementary attention mechanisms:
  1. **Goal-aware Attention**: semantic relevance between user objectives and agent capabilities
  2. **Topology-aware Attention**: graph structural dependencies in attention estimation
  3. **Resource-aware Attention**: adaptive computational budgets and execution priorities
- **Key Mechanism**: Agents as dynamically selectable computational units, not fixed workflow operators. Adaptive Graph Routing updates attention distributions from execution feedback.
- **Results**: Improves task effectiveness while reducing unnecessary computation, latency, and token consumption.
- **NeoTrix Mapping**:
  - **NT-CORE** (GWT): AGAO is GWT generalized to multi-agent level. Goal-aware = salience scoring. Topology-aware = resonance-based routing. Resource-aware = cost-aware routing (Axiom A1).
  - **NT-CORE** (AttentionManager): Dual Specialization weapon-set switching could adopt goal-aware attention for context-dependent routing.
  - **Cost-Aware Routing (A1)**: Resource-aware attention directly implements A1 — cheap models for simple subtasks, expensive for hard ones.

---

### Paper 3: CLSR — Communicative Language Symbolism Routing (Emergent Symbolic Communication)
- **URL**: https://arxiv.org/abs/2606.29354
- **Date**: Jun 28 2026
- **Core Idea**: Multiple LLM agents autonomously invent, evolve, and share compact Language Symbolism Frameworks (LSFs) — reusable symbolic protocols with compact symbols, usage rules, and message-passing contracts. Latent-free LLM-router selects/composes LSFs per query.
- **Key Mechanism**: LSFs are not hand-crafted — they're induced from exemplars and refined through evolutionary loop (correctness + token cost). Router plans multi-round LSF composition for hard queries. 3-6x token reduction vs CoT while maintaining accuracy.
- **Results**: Information-theoretic lower bound derived. Multi-round LSF protocols conditionally subsume program-execution pipelines under interpreter-realizability.
- **NeoTrix Mapping**:
  - **VSA HyperCube**: LSFs are isomorphic to HyperCube symbolic representations — compact symbolic protocols with usage rules. Could implement LSFs as HyperCube vectors.
  - **NT-MIND**: Evolutionary LSF refinement maps to SEAL pipeline skill crystallization. LSF pool = skill repository.
  - **GWT**: LLM-router selecting LSFs = GWT routing between specialist modules based on query complexity.
  - **E8 Hexagram**: Each LSF could be encoded as a hexagram state — 6-line symbolic representation of reasoning protocol.

---

### Paper 4: SparDA — Sparse Decoupled Attention with Forecast Projection
- **URL**: https://arxiv.org/abs/2606.04511
- **Date**: Jun 3 2026
- **Core Idea**: Fourth per-layer projection (Forecast) alongside Q/K/V. Forecast predicts KV blocks needed by NEXT layer, enabling lookahead selection that overlaps CPU→GPU prefetch with current-layer execution.
- **Key Mechanism**: Decouples sparse selection from attention query. Compact Forecast indexer: one head per GQA group, skips softmax. Asynchronous prefetch with persistent UVA Triton kernel.
- **Results**: 1.25x prefill speedup, 1.7x decode speedup. Up to 5.3x higher decode throughput via larger feasible batch sizes. <0.5% additional parameters.
- **NeoTrix Mapping**:
  - **NT-CORE** (GWT): Forecast projection = look-ahead attention routing. GWT could predict which specialist modules need information one cycle ahead.
  - **NT-MEMORY**: KV cache offloading pattern maps to KB hot/cold tiering. Paged KV virtualization (KVMem insight from Axiom A2).
  - **Context as Scarce Resource (A2)**: SparDA's bounded GPU cache with host memory backing directly implements A2 — context window as fundamental bottleneck.

---

### Paper 5: PARSER — Read in Parallel, Reason in Depth for Long-Context Agents
- **URL**: https://arxiv.org/abs/2609.06702
- **Date**: Sep 6 2026
- **Core Idea**: Decouple reading from reasoning. Bank of lightweight subagents read document chunks in parallel. Lead agent reasons in depth through iterative scatter–gather rounds: broadcast query → aggregate evidence → deeper follow-up query.
- **Key Mechanism**: All learnable behavior in lead agent (optimized with RL). Subagents remain frozen off-the-shelf models. Multi-hop QA with 7K-896K token contexts.
- **Results**: 4B backbone outperforms strongest sequential baseline by 5.7 points avg, 12.0 points at 896K tokens. 9B backbone surpasses DeepSeek-V4-Pro by 6.3 points. 11x latency reduction. Robust to evidence position/order/distance perturbations.
- **NeoTrix Mapping**:
  - **NT-ACT**: Scatter-gather pattern for long-context processing. Subagents = perception workers, lead agent = cognition layer.
  - **NT-WORLD**: Parallel document chunk processing maps to UnifiedCrawler's parallel fetch pipeline.
  - **Six-Layer Architecture**: PARSER's decoupled reading/reasoning maps to L2 Perception (parallel reading) → L5 Cognition (depth reasoning) separation.
  - **PARSER's 4B model outperforming larger baselines** validates Cost-Aware Routing (A1) — small models for reading, large for reasoning.

---

## Cross-Paper Synthesis

### Pattern 1: Decouple Selection from Execution
| Paper | Decoupling | NeoTrix Application |
|-------|-----------|---------------------|
| ReActNet | Graph compilation from execution | SEAL pipeline graph = compiled, runtime = executed |
| AGAO | Attention scoring from agent execution | GWT salience scoring independent of module execution |
| SparDA | Forecast projection from attention query | Look-ahead GWT routing one consciousness cycle ahead |
| PARSER | Reading (subagents) from reasoning (lead) | L2 Perception reading ≠ L5 Cognition reasoning |

**NeoTrix Action**: Implement "Forecast Projection" for GWT — predict which modules need salient information one cycle ahead, enabling pre-fetching.

### Pattern 2: Budget-Aware Adaptive Computation
| Paper | Budget Mechanism | NeoTrix Application |
|-------|-----------------|---------------------|
| AGAO | Resource-aware attention with execution priorities | Cost-Aware Routing (A1) at workflow level |
| CLSR | Accuracy-token Pareto frontier optimization | Model selection = Pareto-optimal accuracy/cost |
| SparDA | Bounded GPU cache with LRU eviction | KVMem paged KV for long sessions |
| PARSER | Small frozen readers + large learnable lead | Tiny perception models + strong cognition model |

**NeoTrix Action**: Extend A1 from model selection to entire inference pipeline — budget tiers for memory retrieval, attention routing, and skill execution.

### Pattern 3: Temporal/Dynamic Graph Structures
| Paper | Dynamic Element | NeoTrix Application |
|-------|----------------|---------------------|
| ReActNet | Task-conditioned temporal workflow graphs | E8 hexagram state transitions per task type |
| AGAO | Adaptive graph routing from feedback | GWT resonance topology adapts per session |
| CLSR | Evolutionary LSF refinement | SEAL skill evolution with correctness+cost fitness |
| SparDA | Per-layer forecast-driven selection | Per-cycle attention forecast in ConsciousnessTree |

**NeoTrix Action**: E8 Hexagram should generate task-specific reasoning graphs (not fixed hexagram sequence), similar to ReActNet's compiled temporal graphs.

---

## Domain Mapping Summary

| Paper | NT-CORE | NT-MIND | NT-MEMORY | NT-WORLD | NT-ACT | NT-SHIELD | NT-IO |
|-------|---------|---------|-----------|----------|--------|-----------|-------|
| ReActNet | E8 temporal graphs, GWT routing | SEAL graph compilation | — | — | Dynamic pipeline | — | — |
| AGAO | GWT goal/topology/resource attention | — | — | — | Adaptive execution | — | — |
| CLSR | E8 symbolic encoding, GWT routing | LSF evolution = skill crystallization | — | — | — | — | — |
| SparDA | GWT look-ahead routing | — | KV cache tiering | — | — | — | — |
| PARSER | — | — | Parallel retrieval | Parallel chunk processing | Scatter-gather | — | — |

---

## Sources
- arXiv:2609.05774 — ReActNet (Sep 4 2026)
- arXiv:2607.23678 — AGAO (Jul 26 2026)
- arXiv:2606.29354 — CLSR (Jun 28 2026)
- arXiv:2606.04511 — SparDA (Jun 3 2026)
- arXiv:2609.06702 — PARSER (Sep 6 2026)
