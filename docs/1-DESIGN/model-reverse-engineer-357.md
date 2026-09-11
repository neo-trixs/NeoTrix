# Model Reverse Engineering — Cycle 357 (2026-09-12)

## Selection Criteria
Recent papers (2026) on efficient inference, attention mechanisms, agent coordination, and memory routing. Mapped to NeoTrix 7 domains (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD).

---

## 1. AgentInfer — Co-Design of Inference Architecture and System

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2512.18337v2 (Feb 2026) |
| **Authors** | Weizhe Lin, Hui-Ling Zhen, et al. (Huawei) |
| **Key Innovation** | Unified framework for end-to-end agent acceleration with 4 synergistic components: AgentCollab (dual-model reasoning), AgentSched (cache-aware scheduling), AgentSAM (suffix-automaton speculative decoding), AgentCompress (semantic compression). 1.8-2.5x speedup, 50%+ token reduction. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **AgentCollab** | Hierarchical dual-model: large model plans, small model executes. Dynamic escalation/de-escalation based on self-evaluation. Not fixed routing — the agent decides when to "call in the expert." |
| **AgentSched** | Cache-aware hybrid scheduler. Minimizes latency under heterogeneous request patterns by scheduling cache-hits together and cache-misses together. |
| **AgentSAM** | Suffix automaton for speculative decoding across agent sessions. Reuses multi-session semantic memory — if two sessions share prefix patterns, the automaton reuses token predictions. |
| **AgentCompress** | Asynchronous context summarization. Distills and reorganizes agent memory without disrupting ongoing reasoning. Compresses at reasoning-boundary granularity, not token granularity. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | AgentCollab dual-model = GWT attention routing with cost-aware escalation (Axiom A1). The "dynamic escalation" maps to ConsciousnessTree's adaptive cycle depth — simple tasks get shallow cycles, complex tasks trigger deep cycles. |
| **NT-MIND** | AgentCompress = SEAL pipeline distillation stage. Reasoning-boundary compression maps to experience-tree's 5-stage absorption (snapshot→distill→classify→persist→feedback). |
| **NT-MEMORY** | AgentSAM suffix automaton = cross-session pattern reuse. Maps to NT-NEXUS's cross-session memory — if two sessions share similar trajectories, reuse prefix computations. |
| **NT-IO** | AgentSched cache-aware scheduling = NT-IO's provider routing with request batching. Shared-prefix requests should be routed to same provider for cache locality. |
| **NT-SHIELD** | AgentCollab's self-evaluation = NT-SHIELD's RiskAssessor. Before escalating to expensive model, the small model evaluates whether escalation is warranted. |

### Key Takeaway for NeoTrix
The **reasoning-boundary compression** insight is critical: don't compress at token granularity (lossy) or turn granularity (too coarse). Compress at reasoning-phase boundaries — when the agent transitions from "thinking" to "acting" or vice versa. NeoTrix's ConsciousnessTree phases (Soil→Roots→Trunk→Branches→Fruits→Core) provide natural compression boundaries.

---

## 2. Attention-MoA — Inter-Agent Semantic Attention

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2601.16596 (Jan 2026) |
| **Authors** | Jianyu Wen, Yang Wei, et al. (Meituan) |
| **Key Innovation** | Mixture-of-Agents with inter-agent semantic attention. Agents don't just vote — they attend to each other's intermediate representations. Adaptive early stopping prevents information degradation in deep layers. 91.15% LC Win Rate on AlpacaEval 2.0. Small models ensemble beats Claude-4.5-Sonnet. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Inter-Agent Attention** | Each agent's hidden state is treated as a "token" in a cross-agent attention layer. Agents attend to each other's reasoning, not just their outputs. |
| **Residual Module** | Inter-layer residual connections prevent information degradation across deep agent layers. Each layer's output is added to the next layer's input. |
| **Adaptive Early Stopping** | The system monitors when additional agent layers stop improving quality. Terminates early when consensus is reached, avoiding redundant computation. |
| **Small Model Ensemble** | 3-4 small open-source models (7B-70B) with semantic attention can outperform single 400B+ models at lower total cost. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Inter-agent attention = E8 hexagram cross-domain reasoning. Each hexagram "attends" to other hexagrams' intermediate states, not just final outputs. Adaptive early stopping = ConsciousnessTree cycle termination when phi stabilizes. |
| **NT-MIND** | Small model ensemble = SEAL pipeline skill nodes. Rather than one massive model, use specialized small models that attend to each other. Each skill node is a specialist that contributes to ensemble reasoning. |
| **NT-ACT** | Agent coordination without central orchestrator = NT-ACT's tool orchestration where tools collaborate via shared state, not master-slave dispatch. |
| **NT-PHYSICAL** | Residual connections = NT-PHYSICAL's body schema integrity. Each layer preserves information from previous layers, preventing "amnesia" in deep reasoning chains. |

### Key Takeaway for NeoTrix
The **inter-agent semantic attention** pattern is a direct implementation of what GWT should be: not just broadcasting salient information, but allowing specialist modules to attend to each other's intermediate representations. This is "GWT with depth" — not flat broadcast but structured cross-attention.

---

## 3. PackInfer — Compute- and I/O-Efficient Batched Attention

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2602.06072 (Feb 2026) |
| **Authors** | Rui Ning, Wei Zhang, Fan Lai |
| **Key Innovation** | Kernel-level attention framework for heterogeneous batched inference. Packs multiple requests into unified kernel launches, eliminating redundant computation. I/O-aware grouping co-locates shared-prefix requests. 13-20% latency reduction, 20% throughput improvement over FlashAttention. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Load-Balanced Execution Groups** | Batched requests are orchestrated into groups that saturate GPU utilization. Short sequences packed with short, long with long. |
| **Packed Query-Key Regions** | Attention kernels operate directly over packed regions, eliminating redundant computation and balancing thread-block execution. |
| **I/O-Aware Grouping** | Shared-prefix requests are co-located in memory. KV caches reorganized into group-contiguous layouts, reducing memory fragmentation and data movement. |
| **Prefix-Aware Batching** | Requests sharing system prompts or few-shot examples are batched together so KV cache pages are shared, not duplicated. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Load-balanced execution = GWT attention allocation. Not all attention heads get equal resources — some get more compute, others more memory. Maps to "selective attention" where high-salience items get full processing, low-salience items get minimal. |
| **NT-MEMORY** | I/O-aware grouping = KB query optimization. Similar queries should hit the same cache pages. Maps to KB's BM25 + embedding hybrid index where similar queries share prefix computations. |
| **NT-IO** | Prefix-aware batching = NT-IO's provider routing. Requests sharing system context should route to same provider for KV cache reuse. Validates cost-aware routing (Axiom A1). |
| **NT-PHYSICAL** | GPU utilization saturation = NT-PHYSICAL's power/thermal management. Efficient batching means less wasted energy per inference. |

### Key Takeaway for NeoTrix
**Prefix-aware batching** is the infrastructure-level equivalent of NeoTrix's session-level experience reuse. When multiple sessions share similar prefixes (same AGENTS.md, same CONTEXT.md, similar task types), their KV cache pages should be shared at the GPU level. This is hardware-level NT-NEXUS.

---

## 4. Active Inference as Context Acquisition

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2608.19202 (Jun 2026) |
| **Authors** | Sanchayan Dutta, Sai Niranjan Ramachandran, Suvrit Sra (MIT) |
| **Key Innovation** | Formulates the "should I ask or should I assume?" tradeoff as active inference. Inner step updates beliefs over latent task state; outer step selects context action (ask/act/stop) to minimize expected free energy. Model-agnostic design principle for the context-acquisition layer. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Active Inference Framework** | Agent models uncertainty over task state. Each potential action (ask question, retrieve, assume default, act) is scored by expected information gain minus token cost. |
| **Optimal Question Asking (OQA)** | Exact posteriors with dynamic programming oracle. Benchmarks frontier LLMs on binary and multiway tasks (25-300 candidates). |
| **Free Energy Minimization** | The agent minimizes expected free energy = expected loss + information cost. Balances exploration (gaining context) vs exploitation (acting with existing context). |
| **Token Budget Optimization** | Clarification before generation and automated prompt optimization under token budgets. Not just "what to ask" but "how many tokens to spend asking." |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Active inference = GWT attention allocation with uncertainty. The agent's "awareness_score()" in PerceptionBridge is exactly this: score = expected information gain × relevance − cost. Low awareness → skip perception event. |
| **NT-MIND** | Free energy minimization = SEAL pipeline stage selection. Each stage has an expected "information gain" and a token cost. The pipeline selects stages that minimize total free energy. |
| **NT-MEMORY** | Context acquisition = KB query strategy. When uncertain about what to retrieve, the agent should model which query will maximally reduce uncertainty at minimal cost. |
| **NT-IO** | Token budget = NT-IO's cost-aware routing (Axiom A1). Active inference provides the theoretical foundation for routing decisions: spend tokens on context acquisition (asking) or on execution (acting). |

### Key Takeaway for NeoTrix
This paper provides the **theoretical foundation for GWT's salience calculation**. Currently GWT uses heuristic salience scoring. Active inference formalizes it: salience = expected information gain − cost. This could replace NeoTrix's ad-hoc attention allocation with a principled Bayesian framework.

---

## 5. Supra Cognitive Modes — Routed Architecture for Agent Memory

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2607.19096 (Jul 2026) |
| **Authors** | Joshua Tobkin, David Yang (Supra Research) |
| **Key Innovation** | Per-query control interface for agent memory. Maps explicit or automatically selected semantic modes to retrieval and synthesis payloads over one shared substrate. 4 modes: single-fact lookup, time-anchored lookup, latest-version resolution, long-form synthesis. 86% on LongMemEval (+29% over baseline). |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Mode-Payload-Procedure Interface** | Three-layer abstraction: semantic mode (what does the query want?) → payload (what retrieval/synthesis config?) → procedure (how to execute?). Separates intent from implementation. |
| **Shared Asynchronous Substrate** | All memory families (direct, graph, long-form) read from one unified ingest substrate. No separate stores per query type. |
| **Four Semantic Modes** | (1) Single-fact lookup — fused lexical+dense retrieval. (2) Time-anchored lookup — temporal metadata filtering. (3) Latest-version resolution — triple versioning. (4) Long-form synthesis — stratified chunk assembly. |
| **Runtime Gates** | Procedural dispatch: frozen semantic classifier routes queries; runtime gates can change the executed procedure mid-execution if initial path is insufficient. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-MEMORY** | Mode-payload-procedure = KB query routing. NeoTrix's KB already has multiple namespaces (experience, domain_*, knowledge). SCM's modes map to namespace selection: factoid → domain_*, temporal → experience, synthesis → cross-namespace. |
| **NT-CORE** | Runtime gates = ConsciousnessTree adaptive depth. If initial retrieval is insufficient, the system escalates to deeper reasoning — exactly the ConsciousnessTree's Soils→Core progression. |
| **NT-WORLD** | Shared async substrate = NT-WORLD's UnifiedCrawler. Multiple perception modalities (text, image, audio) share one ingestion pipeline, not separate crawlers per type. |
| **NT-MIND** | Long-form synthesis = SEAL pipeline's Fruits→Core stage. Raw experiences are synthesized into high-level insights. |

### Key Takeaway for NeoTrix
The **mode-payload-procedure** abstraction is exactly what NeoTrix's KB query layer needs. Currently KB queries are ad-hoc. SCM provides a clean interface: declare the intent mode, the system selects the optimal retrieval/synthesis payload, and executes the appropriate procedure. This could replace NeoTrix's manual query construction with intent-driven retrieval.

---

## Cross-Cutting Synthesis (Cycle 357)

| Theme | Papers | NeoTrix Integration |
|-------|--------|-------------------|
| **Compression at Boundaries** | AgentInfer, claw-compactor | Compress at ConsciousnessTree phase boundaries, not token/turn granularity |
| **Agent Self-Evaluation** | AgentInfer, Attention-MoA | Agents should evaluate their own confidence before escalating or terminating |
| **Shared Substrate, Multiple Modes** | SCM, PackInfer | One unified memory/cache layer with multiple query modes, not separate stores |
| **Inter-Agent Attention** | Attention-MoA | GWT should be structured cross-attention, not flat broadcast |
| **Active Inference for Routing** | Active Inference | Bayesian framework for GWT salience: salience = info_gain − cost |
| **Prefix Sharing** | PackInfer, AgentSAM | Cross-session KV cache reuse for shared AGENTS.md/CONTEXT.md prefixes |

## Novel vs Incremental

| Paper | Novelty | NeoTrix Priority |
|-------|---------|-----------------|
| **AgentInfer** | High — 4 synergistic components, reasoning-boundary compression | P0 — reasoning-boundary compression for ConsciousnessTree |
| **Attention-MoA** | High — inter-agent semantic attention, small-ensemble beats big models | P1 — GWT refinement: structured cross-attention |
| **Active Inference** | High — theoretical framework for context acquisition | P1 — Bayesian foundation for GWT salience |
| **SCM** | Medium — clean interface over existing primitives | P2 — mode-payload-procedure for KB queries |
| **PackInfer** | Medium — infrastructure optimization | P3 — prefix-aware batching for provider routing |

## Implementation Roadmap

| Phase | Action | Paper |
|-------|--------|-------|
| **Immediate** | Design reasoning-boundary compression for ConsciousnessTree phases | AgentInfer |
| **Week 2** | Prototype GWT salience with active inference (expected info gain − cost) | Active Inference |
| **Month 1** | Add mode-payload-procedure interface to KB query layer | SCM |
| **Month 2** | Implement inter-agent semantic attention for E8 hexagram reasoning | Attention-MoA |
| **Quarter** | Prefix-aware batching for cross-session KV cache sharing | PackInfer |
