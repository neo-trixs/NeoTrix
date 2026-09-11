# Model Reverse Engineering — Cycle 337

**Date**: 2026-09-11
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory systems

## Paper 1: Flux Attention (arXiv:2604.07394)

**Title**: Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference
**Venue**: arXiv April 2026 | **Authors**: Qiu et al.

### Core Mechanism
- **Layer-level dynamic routing**: A lightweight Layer Router is inserted into frozen pretrained LLMs. Each layer is adaptively routed to Full Attention (FA) or Sparse Attention (SA) based on input context.
- **Constraint optimization**: Training objective balances generation quality and computational efficiency. A dynamic penalty prevents the router from degenerating to all-FA mode.
- **Hardware alignment**: Unlike head-level sparsity (which causes load imbalance and sync long-tails), layer-level routing preserves contiguous memory access — translating theoretical sparsity into wall-clock speedups.
- **Results**: 2.8x prefill speedup, 2.0x decode speedup. Only 12 hours training on 8×A800 GPUs.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | Layer Router = consciousness-level attention modulation. Each layer decides "attend fully" or "attend sparsely" based on context salience. | Extend GWT salience to layer-granularity routing. Current GWT broadcasts globally; Flux shows layer-level is sufficient and faster. |
| **NT-MEMORY** | Sparse attention = selective memory retrieval. Only relevant "layers" of KB are queried. | Optimize KB retrieval to use layer-like partitioning — route queries to relevant namespace subsets. |
| **NT-SHIELD** | Dynamic penalty prevents router degeneration → analogous to guard preventing attention collapse. | Add degeneration detection to GWT — if routing becomes uniform, inject diversity penalty. |

### Actionable Insight
**Flux Attention's layer router is a mini-GWT per layer.** NeoTrix could implement a "micro-GWT" at the attention layer level, where each layer independently decides routing based on local context salience, rather than a single global broadcast. This is structurally identical to ConsciousnessTree's 6-stage loop but at the attention computation level.

---

## Paper 2: MKA — Memory-Keyed Attention (arXiv:2603.20586)

**Title**: MKA: Memory-Keyed Attention for Efficient Long-Context Reasoning
**Venue**: ACM Computing Frontiers 2026 (Oral) + ICML 2025 Long Context Workshop

### Core Mechanism
- **Hierarchical KV caches**: Three levels — local (current sequence), session (recent turns), long-term (consolidated). Attention routes dynamically across all three.
- **FastMKA variant**: Broadcast-routed fusion — memory sources are fused before attention computation, avoiding separate attention passes.
- **Results**: Comparable perplexity to MLA (Multi-Latent Attention) while 5x faster training throughput and 1.8x lower evaluation latency.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-MEMORY** | Three-tier KV = sensory/working/long-term memory. Direct map to experience-tree's write path. | Align KB storage tiers with MKA's local/session/long-term partition. |
| **NT-CORE (GWT)** | Dynamic routing across memory tiers = attention selecting which memory level to consult. | GWT salience should consider memory tier proximity — local memory gets higher salience boost. |
| **NT-MIND** | FastMKA's broadcast-routed fusion = distillation across memory tiers. | Extend SEAL distillation to fuse memory tiers before reasoning, not just within tiers. |

### Actionable Insight
**MKA proves that hierarchical KV routing outperforms flat attention.** NeoTrix's KB already has namespaces (domain_*), but lacks hierarchical routing. Adding a local/session/long-term routing layer to KB queries could yield significant retrieval quality and speed improvements.

---

## Paper 3: Gated-Memory Routing (arXiv:2609.00237)

**Title**: Learning What to Retain: Gated-Memory Routing for Efficient Collaboration in Multi-Agent LLM Systems
**Venue**: EMNLP 2026 Main Conference

### Core Mechanism
- **Memory Write Gate**: Commits only non-redundant reasoning steps to execution memory. Filters out duplicate/low-utility steps.
- **Retrieval Gate**: Supplies each agent a compact, relevant subset of execution memory — not the full history.
- **Adaptive Halting Controller**: Stops execution once memory contains sufficient evidence for answering.
- **Results**: Best average accuracy (+2.44 points over strongest baseline), 31.9% inference cost reduction on HumanEval.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-MEMORY** | Write Gate = experience-tree write filter. Retrieval Gate = lazy branch loading with relevance scoring. | Add a "write gate" to experience absorption — only commit non-redundant experiences. |
| **NT-ACT** | Adaptive Halting = orchestration termination when sufficient evidence accumulated. | Add evidence-threshold to production orchestrator — stop when confidence exceeds threshold. |
| **NT-MIND** | Execution memory = SEAL pipeline intermediate state. Gated writes prevent pollution. | Gate SEAL phase transitions — only propagate non-redundant findings between phases. |

### Actionable Insight
**The Write Gate + Retrieval Gate pattern is directly applicable to NeoTrix's experience-tree.** Currently, all session experiences are absorbed. A Write Gate would filter redundant experiences before KB write. A Retrieval Gate would serve only relevant experience slices to the current context, reducing token waste.

---

## Paper 4: RAGEN-2 — Template Collapse in Agentic RL (arXiv:2604.06268)

**Title**: RAGEN-2: Reasoning Collapse in Agentic RL
**Venue**: ICML 2026 Oral Presentation

### Core Mechanism
- **Template collapse**: Even with stable entropy (diversity within same input), models develop reasoning templates that are input-agnostic. MI (mutual information) between input and reasoning trace drops while entropy stays high.
- **MI diagnostic**: In-batch cross-scoring — each reasoning trace is used as a query to retrieve its source input from a minibatch. Accuracy drops toward chance under collapse.
- **SNR mechanism**: Low within-input reward variance → weak task gradients → input-agnostic regularizers dominate → cross-input reasoning differences erased.
- **SNR-Aware Filtering**: Prioritize high-variance (high-signal) prompts per iteration. Simple, effective.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-MIND (SEAL)** | Template collapse = SEAL pipeline producing generic distillations, not input-specific insights. MI = quality metric for distilled experiences. | Add MI-based quality check to SEAL distillation. If MI drops, inject high-signal prompts. |
| **NT-CORE (ConsciousnessTree)** | Reasoning quality = tree branch health. Template collapse = branches producing similar fruits regardless of input soil. | Monitor ConsciousnessTree fruit diversity. If fruits become templated, trigger soil replenishment (new stimuli). |
| **NT-MEMORY** | Execution memory accumulation = RL training trajectory. Write Gate prevents template pollution. | Experience-tree should track input-specificity of stored experiences. Deduplicate templated experiences. |

### Actionable Insight
**MI is a better quality metric than entropy for NeoTrix's reasoning.** The ConsciousnessTree currently monitors coherence/phi but not input-specificity. Adding an MI-like metric — "does this reasoning trace actually depend on the input?" — would detect a failure mode invisible to current monitors. SNR-Aware Filtering maps directly to experience absorption quality gating.

---

## Paper 5: EvoRoute — Self-Evolving Model Routing (ACL 2026)

**Title**: EvoRoute: Experience-Driven Self-Routing LLM Agent Systems
**Venue**: ACL 2026 Main Conference (Long Paper)

### Core Mechanism
- **Agent System Trilemma**: Tension among performance, cost, and latency. No static routing solves all three.
- **Experience-driven routing**: Expanding knowledge base of prior execution experiences. Each new task queries experience KB for similar past tasks, then selects Pareto-optimal model.
- **Pareto-optimal selection**: Multi-objective optimization — not cheapest, not best, but the model that dominates on the cost-performance-latency frontier.
- **Self-evolution**: Routing policy improves with each execution via experience accumulation.
- **Results**: 80% cost reduction, 70%+ latency reduction, while sustaining or improving performance.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | EvoRoute = experience-weighted GWT salience. Currently GWT uses static salience; EvoRoute shows experience history should weight routing decisions. | Extend GWT salience function with experience-history terms. Routes that worked well before get higher prior. |
| **NT-CORE (SelfModel)** | Experience KB = self-model performance history. Pareto selection = multi-objective self-model optimization. | SelfModel should track Pareto frontier of past routing decisions, not just per-model metrics. |
| **NT-MIND** | Self-evolution of routing policy = SEAL-style learning loop applied to model selection. | Add SEAL-style distillation for routing experiences — compress routing history into routing skills. |

### Actionable Insight
**EvoRoute's Pareto-optimal experience-weighted routing is the next evolution of NeoTrix's cost-aware routing (A1).** Current GWT uses static cost weights; EvoRoute shows that accumulated experience should dynamically shift the Pareto frontier. Implementing an experience KB for routing decisions could yield 80% cost reduction at matched quality.

---

## Synthesis: Cross-Paper Patterns

| Pattern | Papers | NeoTrix Integration |
|---------|--------|---------------------|
| **Hierarchical Routing** | Flux Attention, MKA, EvoRoute | Layer-level → memory-tier-level → experience-level. NeoTrix should implement routing at all three granularities. |
| **Write-Side Filtering** | Gated-Memory, RAGEN-2 | Filter before KB write, not just after retrieval. Reduces noise at source. |
| **MI as Quality Metric** | RAGEN-2 | Replace entropy-only monitoring with MI-based input-specificity tracking. |
| **Experience-Weighted Selection** | EvoRoute, Gated-Memory | Accumulate routing/execution history to inform future decisions. |
| **Adaptive Termination** | Gated-Memory | Stop processing when sufficient evidence accumulated. Reduces wasted compute. |

## Priority Actions

| # | Action | Source | Domain | Effort |
|---|--------|--------|--------|--------|
| 1 | Add MI diagnostic to ConsciousnessTree | RAGEN-2 | NT-MIND | Medium |
| 2 | Implement Write Gate for experience-tree | Gated-Memory | NT-MEMORY | Low |
| 3 | Extend GWT with experience-weighted Pareto routing | EvoRoute | NT-CORE | High |
| 4 | Add hierarchical KV routing to KB queries | MKA | NT-MEMORY | Medium |
| 5 | Implement layer-level micro-GWT | Flux Attention | NT-CORE | High |
| 6 | Add adaptive halting to orchestrator | Gated-Memory | NT-ACT | Low |
