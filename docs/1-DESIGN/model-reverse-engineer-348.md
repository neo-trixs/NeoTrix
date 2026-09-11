# Model Reverse Engineer — Cycle 348

**Date**: 2026-09-11
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory architectures
**Sources**: arXiv (Sep 2026), ACL 2026 Findings, ICML 2026

## 5 New Papers (Not in Cycles 318-347)

### 1. Declarative Attention: Language Models Can Control Their Own Attention (arXiv:2609.02737)
- **URL**: https://arxiv.org/abs/2609.02737
- **Category**: Intrinsic Sparse Attention via Self-Declaration
- **Core Mechanism**: Model declares where it needs to attend within its chain-of-thought, partitioning generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). Inference engine parses these declarations like tool calls and skips most KV cache read. Zero-shot evaluation on off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B).
- **Key Insight**: "Wouldn't the model already know which parts of the context are relevant?" Instead of external proxy scores (still O(N) per step), the model intrinsically declares attention regions, enabling O(1) KV cache access for focused queries. 52% (Gemma-4-31B) and 31.1% (Qwen-3.6-27B) reduction in total attended tokens with modest accuracy drops (1.27pp, 2.75pp) that shrink with model scale.
- **Results**: Significant attention token reduction without any training. Unlocking a new axis of sparse attention with further potential under training-based methods.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (GWT)**: Declarative Attention = model-driven GWT attention routing — the model itself decides which sensory inputs deserve global broadcast vs local processing. `<global>` = full GWT broadcast, `<focus>` = selective attention to specific memory regions, `<local>` = recent working memory only. Validates and extends GWT with intrinsic (not proxy-scored) attention modulation.
  - **NT-MEMORY**: Attention declarations enable efficient KV cache management — only declared regions are loaded, enabling larger effective context windows with bounded compute. Maps to experience-tree lazy branch loading with model-guided prefetching.
  - **NT-IO**: Declaration parsing like tool calls = NT-IO structured output protocol for attention control signals.

### 2. CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention (arXiv:2609.07237)
- **URL**: https://arxiv.org/abs/2609.07237
- **Category**: Coarse-to-Fine Error-Aware Attention Routing
- **Core Mechanism**: Each semantic chunk contributes a cheap key-value summary to a residual attention path. Chunks with high estimated approximation error are expanded to exact token attention. Exact and summarized contributions combined in single softmax normalization (refinement replaces, not duplicates, coarse evidence). Output-error bound governed by within-chunk key/value dispersion. Variable refinement budget allocated per-query based on error estimate.
- **Key Insight**: Hard selection (sparse attention) assigns zero probability to omitted chunks — routing miss cannot be recovered. CEDAR keeps global coverage via residual summaries while selectively expanding high-error chunks. "Refinement replaces rather than duplicates coarse evidence" — avoiding the information duplication problem.
- **Results**: 98%+ reduction in reconstruction error vs hard dropping at equal exact-chunk budgets. ~3x kernel speedup at 128K context. Recovers most quality lost by hard sparse routing while maintaining speedup. Accepted at ICML 2026.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (GWT)**: Error-bounded routing = GWT salience scoring with formal quality guarantees. Variable refinement budget = cost-aware attention allocation (Axiom A1). Residual summaries = VSA HyperCube compressed state representations providing global coverage without full materialization.
  - **NT-MEMORY**: Chunk-level summaries = experience-tree hub index with lazy detail loading. Error-bound-governed expansion = quality-gated branch loading (load details only when summary accuracy is insufficient).
  - **NT-WORLD**: Semantic chunking for attention = NT-WORLD content segmentation for perception pipeline.

### 3. Inference-Time Graph Engineering for Multi-Agent LLM Workflows — ReActNet (arXiv:2609.05774)
- **URL**: https://arxiv.org/abs/2609.05774
- **Category**: Task-Conditioned Temporal Workflow Graphs
- **Core Mechanism**: Compiles query + role-specialized agents into sequence of directed communication graphs. Each snapshot = one reasoning stage, each edge = natural-language instruction specifying message from source to target agent. Graph compilation separated from graph execution — making coordination explicit, inspectable, and task-conditioned. Structured message passing: agents update reasoning states by integrating previous states with messages from controller-assigned neighbors. Final aggregator synthesizes states into answer.
- **Key Insight**: "Effective multi-agent orchestration depends not only on which agents communicate, but also on engineering executable workflow graphs that encode when, why, and how information should flow during reasoning." Training-free — no RL or gradient-based topology optimization needed.
- **Results**: Consistently improves over fixed-topology and learned-topology baselines across knowledge reasoning, math, code generation, and GAIA assistant tasks. Competitive inference cost.
- **NeoTrix Domain Mapping**:
  - **NT-ACT**: Temporal workflow graphs = NT-ACT orchestration with dynamic topology selection per task. Edge instructions = EventBus message contracts with natural-language semantics. Aggregator = NT-ACT result synthesis.
  - **NT-CORE (GWT)**: Task-conditioned graph compilation = GWT salience-driven attention routing that shapes communication topology. Each stage's graph = GWT broadcast pattern for that reasoning phase.
  - **NT-MIND**: Training-free graph engineering = SEAL pipeline with explicit topology design rather than learned topology — lighter, more inspectable, faster iteration.

### 4. CondenseFlow: Scalable Latent Space Collaboration via Semantic Compression (ACL Findings 2026)
- **URL**: https://aclanthology.org/2026.findings-acl.669.pdf
- **Category**: Semantic Compression for Multi-Agent Latent Communication
- **Core Mechanism**: Latent Thought Condenser (LTC) — lightweight module using learnable semantic probes to compress KV caches into fixed-size representations. Achieves O(1) communication complexity regardless of context length. Cross-attention aggregation: probes compute semantic relevance with key vectors, discovering valuable information patterns through end-to-end learning (not heuristic pruning). Compression error bounded by attention concentration (ρ): error scales with (1 − ρ) where ρ = fraction of attention mass captured by top-K positions. K=64 default based on effective rank analysis (typical KV caches have effective rank < 50).
- **Key Insight**: "The core challenge is not retrieval alone, but managing the knowledge lifecycle: deciding what to externalize, update, or ultimately internalize." Text-based multi-agent communication loses semantic richness; full KV cache transfer is memory-prohibitive. CondenseFlow bridges the gap with learned compression preserving high-level intent.
- **Results**: 99%+ KV cache memory reduction, ~20% inference latency reduction, negligible accuracy degradation (<2%). Outperforms text-based methods by 1.7pp average across 7 benchmarks and 6 models. Cumulative error grows at most linearly over rounds (empirically sub-linear).
- **NeoTrix Domain Mapping**:
  - **NT-MEMORY**: Semantic compression = experience-tree entry compression for cross-agent knowledge sharing. Fixed-size representations = VSA HyperCube fixed-dimensional embeddings. Attention-concentration error bound = formal quality guarantee for experience-tree branch loading.
  - **NT-CORE (GWT)**: Cross-agent latent communication = GWT broadcast with compressed state vectors. Probes = attention mechanisms discovering which memory dimensions matter for downstream reasoning.
  - **NT-ACT**: O(1) communication complexity = NT-ACT inter-agent coordination with bounded overhead regardless of accumulated context. Enables scaling to deep multi-agent interaction without linear memory growth.

### 5. PARSER: Read in Parallel, Reason in Depth for Long-Context LLM Agents (arXiv:2609.06702)
- **URL**: https://arxiv.org/abs/2609.06702
- **Category**: Decoupled Reading/Reasoning for Long-Context Agents
- **Core Mechanism**: Bank of lightweight subagents (each bound to single chunk) read entire document in parallel. Lead agent reasons in depth through iterative scatter-gather rounds: broadcasts query to all subagents, aggregates returned evidence, formulates deeper follow-up query conditioned on findings. Decoupled design: all learnable behavior in lead agent (optimized with RL), subagents remain frozen off-the-shelf models. Unlike sequential memory agents that couple traversal to reasoning depth.
- **Key Insight**: "Sequential memory agents process long documents by reading chunks one after another while maintaining a compact memory state, coupling document traversal to reasoning depth. This coupling introduces sensitivity to evidence placement and ties inference latency linearly to document length." PARSER decouples reading from reasoning — parallel reads, serial depth.
- **Results**: 4B backbone outperforms strongest sequential baseline by 5.7pp average, 12.0pp at 896K tokens. 9B backbone surpasses DeepSeek-V4-Pro by 6.3pp. Robust to evidence position/order/distance perturbations. Up to 11x latency reduction vs sequential methods.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (GWT)**: Scatter-gather reasoning = GWT attention routing with parallel perception (subagents) and serial deliberation (lead agent). Lead agent = GWT global workspace; subagents = specialist modules broadcasting salient evidence.
  - **NT-WORLD**: Parallel document reading = NT-WORLD sensory integration with parallel fetchers. Chunk-bound subagents = domain-specific perception modules.
  - **NT-MEMORY**: Iterative evidence aggregation = experience-tree multi-hop retrieval with progressive query refinement. Position-invariant robustness = experience-tree retrieval independent of entry ordering.

## Cross-Cutting Patterns (Cycle 348)

| Pattern | Papers | NeoTrix Mapping |
|---------|--------|-----------------|
| **Intrinsic Attention Control** | Declarative Attention, CEDAR | Model-driven (not proxy-scored) attention routing with formal quality bounds — extends GWT with self-declared focus regions |
| **Error-Bounded Approximation** | CEDAR, CondenseFlow | Formal quality guarantees for compressed representations — error bounded by attention concentration, enabling trust in sparse summaries |
| **Decoupled Perception/Reasoning** | PARSER, ReActNet | Parallel perception (subagents/chunks) with serial deliberation (lead agent/aggregator) — GWT broadcast pattern for multi-agent coordination |
| **Semantic Compression for Communication** | CondenseFlow, Declarative Attention | Fixed-size latent representations replace full KV cache transfer — O(1) communication complexity for multi-agent systems |
| **Training-Free Coordination** | ReActNet, PARSER | No RL or gradient optimization needed for topology design — explicit, inspectable, task-conditioned coordination |
