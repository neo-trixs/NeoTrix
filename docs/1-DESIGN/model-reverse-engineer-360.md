# Model Reverse Engineering — Cycle 360 (2026-09-12)

## Selection Criteria
Recent papers (2025-2026) on efficient inference, attention mechanisms, agent coordination, and memory routing. Mapped to NeoTrix 7 domains (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD). Focus: papers not covered in cycles 318-359.

---

## 1. TITAN — Memory-Augmented Attention with Neural Long-Term Memory

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2501.00663 (Jan 2025, Google Research) |
| **Authors** | Behrouz Behrouz, Peilin Zhong, Vahab Mirrokni |
| **Key Innovation** | Three complementary memory types: (1) short-term (attention, in-context), (2) long-term (neural memory module, compressed), (3) persistent (model parameters). Neural memory module uses a gating mechanism that controls information flow from attention to long-term storage. Monotonic associative memory stores history while preserving temporal locality. Core contribution: memory that compresses as it ages — information density increases with time, not just retention. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Three Memory Types** | Short-term: standard attention over recent tokens. Long-term: neural memory module with compressed representation. Persistent: frozen model parameters. Each type has different access patterns and update frequencies. |
| **Neural Long-Term Memory** | Linear attention-based module that maintains compressed state. Gating mechanism controls when new information enters long-term storage. Not all attention output goes to long-term — only information passing the gate. |
| **Monotonic Associative Memory** | Associates current context with compressed history. Temporal locality preserved: recent history has higher resolution, older history has lower resolution but broader coverage. |
| **Training** | Memory module is trainable while preserving attention mechanism. End-to-end training of memory + attention jointly. 1.5B TITAN model matches 2.8B LLM on 500M-token context tasks. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-MEMORY** | Three memory types = NT-MEMORY's namespace hierarchy: short-term (ephemeral), long-term (KB), persistent (experience-tree hub). Neural gating = experience-tree's branch importance scoring — not all interactions enter long-term memory. Monotonic associative = KB's temporal index with recency weighting. |
| **NT-CORE** | Gating mechanism = GWT's attention gating. The gate decides what enters long-term memory from short-term attention — exactly what GWT does for consciousness. Three memory types = ConsciousnessTree's three attention scopes (immediate, session, cross-session). |
| **NT-MIND** | Compressed aging = SEAL pipeline's distillation. Old experiences are compressed (distilled) as they age. Information density increases with time — the system gets more efficient at storing knowledge. |
| **NT-NEXUS** | Persistent memory = model parameters that survive across sessions. The agent's learned behavior patterns are stored in persistent memory, not just recalled from KB. |

### Key Takeaway for NeoTrix
**Memory that compresses with age** — not just retention but progressive compression. TITAN's insight is that older information should be more compressed, not discarded. NeoTrix's KB should implement three tiers: raw (recent, full fidelity), compressed (older, distilled), and structural (persistent, parameter-level). The gating mechanism is the architecture for experience-tree's distillation filter.

---

## 2. Mixture-of-Agents (MoA) — LLM as Backbone of Multi-Agent Collaboration

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2406.04692 (Jun 2024, Together AI) |
| **Authors** | Junlin Wang, Jue Wang, Ben Athiwaratkun, Ce Zhang, James Zou |
| **Key Innovation** | Uses multiple LLMs in a layered collaborative architecture where each model acts as both proposer and aggregator. Proposition layer generates diverse responses. Aggregation layer synthesizes using stronger models. Two key insights: (1) even weaker models improve when used as aggregators over multiple proposers, (2) collaboration amplifies capability beyond any single model. Outperforms GPT-4 on multiple benchmarks. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Proposition-Aggregation Layers** | Two-layer architecture: proposers (any model, including weak) generate candidate responses, aggregators (stronger models) synthesize. Multiple rounds possible. |
| **Collaborative Amplification** | Weaker models perform better when aggregating over diverse proposals. The aggregation process itself is capability-boosting — not just averaging but contextually selecting and combining. |
| **Reference Model Selection** | Each aggregator receives a specific subset of proposer outputs. Not all proposers feed all aggregators — selective routing based on relevance. |
| **No Training Required** | Pure inference-time collaboration. No fine-tuning or training needed. Works with any combination of existing models. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Proposition-aggregation = GWT's broadcast-and-refine. Propositions = domain broadcasts, aggregation = selective attention refinement. Layered collaboration = ConsciousnessTree's multi-stage processing. |
| **NT-ACT** | Multi-model orchestration = NT-ACT's tool selection with multiple candidates. Reference model selection = GWT's salience-based tool routing. |
| **NT-IO** | Model routing = NT-IO's ordered backend with aggregation intelligence. Weaker models as aggregators = even cheap providers contribute value when processing diverse inputs. |
| **NT-MIND** | Collaborative amplification = SEAL pipeline's multi-perspective distillation. Multiple reasoning perspectives synthesized into better conclusions. |

### Key Takeaway for NeoTrix
**Collaborative amplification without training** — the architecture itself creates capability beyond individual components. NeoTrix's 7 domains should be viewed as a MoA: each domain is a "model" that proposes and aggregates. The GWT aggregation layer synthesizes domain proposals into coherent decisions. The key insight: even weak specialized domains improve the whole when they contribute diverse perspectives.

---

## 3. Differential Transformer — Differential Attention for Noise Reduction

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2410.05258 (Oct 2024, Microsoft Research) |
| **Authors** | Tianzhu Ye, Li Dong, Yuqing Yang, Furu Wei, et al. |
| **Key Innovation** | Replaces standard softmax attention with differential attention — computing the difference between two sets of attention weights. Two softmax maps with learnable noise: subtract one from the other to cancel noise while amplifying signal. Core insight: attention maps contain significant noise from irrelevant tokens. Differential subtraction removes this noise. 3.1x improvement on key benchmarks, stronger in-context learning, less attention on irrelevant tokens. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Dual Attention Maps** | Two softmax maps with different noise patterns. Each map attends to the full context but with different noise added. The difference between maps reveals true signal. |
| **Noise Cancellation** | Noise that appears in both maps is cancelled by subtraction. Signal that differs between maps is amplified. Similar to noise-cancelling headphones but for attention. |
| **Learnable Noise** | Noise patterns are learned during training, not random. The model learns what kinds of noise are most common and optimizes subtraction accordingly. |
| **Performance** | 3.1x improvement over standard transformer on key benchmarks. Stronger in-context learning — fewer examples needed for good performance. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Differential attention = GWT's noise filtering. GWT broadcasts salient information but some broadcast content is noise. Differential attention provides the mathematical framework for GWT to distinguish signal from noise in domain broadcasts. |
| **NT-MEMORY** | Noise cancellation = KB query result filtering. Search results contain noise (irrelevant matches). Differential scoring between two retrieval methods (BM25 vs embedding) could cancel noise and amplify signal. |
| **NT-SHIELD** | Noise detection = NT-SHIELD's anomaly detection. Differential attention identifies anomalous patterns by subtracting expected behavior from observed behavior. |
| **NT-FEEL** | Emotion signal from noise = differential attention applied to emotional signals. True emotional state vs surface noise in user interactions. |

### Key Takeaway for NeoTrix
**Noise cancellation for attention** — differential attention provides a principled way to filter noise from signal. NeoTrix's GWT currently selects salient information; differential attention suggests a refinement: broadcast two versions of domain state with different noise profiles, subtract to get true signal. This could be applied to KB search (differential retrieval) and domain health monitoring (differential diagnostics).

---

## 4. Gated Linear Attention (GLA) — Efficient Recurrent Architecture with Hardware-Aware Training

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2407.05237 (Jul 2024, CMU + Google DeepMind) |
| **Authors** | Yaozong Cha, Tri Dao, et al. |
| **Key Innovation** | Recurrent architecture that combines linear attention (constant KV cache, O(n) inference) with data-dependent gating. Gating controls information retention/forgetting at each step — the gate decides what to remember and what to forget. Hardware-aware chunk-wise parallel training for efficient GPU utilization. Competitive with Transformers on language modeling while being O(n) for inference. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Gated Linear Recurrence** | At each step, the new input is multiplied by a data-dependent gate and added to the compressed state. Gate values close to 0 forget old information; values close to 1 retain it. |
| **Linear Attention Foundation** | Core attention mechanism is linear (O(n) complexity) with constant KV cache. No quadratic attention cost. The gating adds selective retention on top of linear attention. |
| **Chunk-Wise Training** | GPU training uses chunk-wise parallelism — processing blocks of tokens in parallel within chunks, sequentially across chunks. Balances training efficiency (parallelism) with recurrence (sequential). |
| **KV Cache Efficiency** | Constant-size KV cache regardless of sequence length. State is O(d²) where d is model dimension, not O(n) where n is sequence length. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-MEMORY** | Gated linear recurrence = KB's temporal state management. Each interaction updates a compressed state with a gate controlling retention. Old interactions are forgotten (gate→0), important ones retained (gate→1). This is the architecture for NT-MEMORY's session state. |
| **NT-CORE** | Linear attention = GWT's efficient attention for routine monitoring. Not all GWT processing needs full quadratic attention. Linear attention for heartbeat monitoring, module health tracking — routine attention that needs O(n) not O(n²). |
| **NT-NEXUS** | Constant KV cache = cross-session memory compression. Regardless of how many sessions occur, the cross-session state remains bounded. New sessions update the gate; old sessions are compressed. |
| **NT-IO** | Hardware-aware training = NeoTrix's hardware-aware optimization. Different execution environments (GPU, CPU, edge) need different attention strategies. GLA's chunk-wise approach adapts to hardware constraints. |

### Key Takeaway for NeoTrix
**Gated linear attention for bounded memory** — the gate provides a learnable forgetting mechanism on top of linear attention. NeoTrix's KB should use gated linear recurrence for session state: each interaction updates a compressed state, with the gate deciding what to retain. This gives O(1) state size regardless of interaction count — solving the context bottleneck for long-running agents.

---

## 5. SAND — Structured Alternation of Narrow and Deep Attention

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2504.16492 (Apr 2025) |
| **Authors** | Ahmet Üstün, et al. |
| **Key Innovation** | Alternates between two attention types: (1) Narrow attention — each token attends to a small window (local context), and (2) Deep attention — a bottleneck where selected tokens attend to the full context. The alternation creates a multi-scale reasoning process: local processing (narrow) followed by global integration (deep). Fewer parameters than standard attention while maintaining or improving performance. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Narrow-Deep Alternation** | Each layer alternates: narrow attention (small window, cheap) → deep attention (full context but bottlenecked, expensive). Not all tokens get full context — only selected ones through the bottleneck. |
| **Bottleneck Token Selection** | Deep attention uses a learned selection mechanism to choose which tokens get full context access. Most tokens only see local context. The bottleneck is a form of information compression. |
| **Multi-Scale Reasoning** | Narrow layers do local processing (within-sentence reasoning). Deep layers do global integration (cross-sentence reasoning). The alternation creates hierarchical reasoning. |
| **Parameter Efficiency** | Narrow layers are cheap (fewer parameters per token). Deep layers are expensive but rare. Total parameter count lower than standard full attention at same depth. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Narrow-deep alternation = GWT's tiered attention. Narrow = domain-internal processing (each NT-* domain works locally). Deep = cross-domain GWT broadcast (selected information shared globally). The alternation creates hierarchical reasoning: local expertise → global integration. |
| **NT-MEMORY** | Bottleneck selection = KB query compression. Most queries don't need full context — narrow retrieval for routine lookups, deep retrieval for complex multi-hop reasoning. The bottleneck selects which queries escalate to full KB scan. |
| **NT-WORLD** | Multi-scale perception = NT-WORLD's perception pipeline. Narrow = single-page extraction (local context). Deep = cross-page synthesis (global context). The alternation creates hierarchical web perception. |
| **NT-ACT** | Bottleneck = NT-ACT's task escalation. Routine tasks stay narrow (local context). Complex tasks escalate through the bottleneck to full context access. |

### Key Takeaway for NeoTrix
**Alternating local and global attention** — not all processing needs full context. SAND's insight is that most tokens (and most agent operations) only need local context. Full context should be reserved for bottleneck operations — selected tokens that carry cross-domain information. NeoTrix's GWT should implement this: narrow attention for domain-internal processing, deep attention for cross-domain integration through a bottleneck.

---

## Cross-Cutting Synthesis (Cycle 360)

| Theme | Papers | NeoTrix Integration |
|-------|--------|-------------------|
| **Memory Compression with Age** | TITAN | Three-tier memory (raw→compressed→structural) with progressive compression. |
| **Collaborative Amplification** | MoA | 7 domains as proposers, GWT as aggregator. Weak specialists strengthen the whole. |
| **Differential Noise Cancellation** | Differential Transformer | Dual-retrieval KB search with noise subtraction. Differential GWT broadcasts. |
| **Gated Linear Recurrence** | GLA | O(1) session state with learnable forgetting gate. Bounded memory for long agents. |
| **Narrow-Deep Alternation** | SAND | Local domain processing → global GWT integration through bottleneck. Hierarchical attention. |

## Novel vs Incremental

| Paper | Novelty | NeoTrix Priority |
|-------|---------|-----------------|
| **TITAN** | High — three memory types with compression aging, neural gating | P0 — architecture for NT-MEMORY three-tier KB with progressive compression |
| **GLA** | High — gated linear recurrence with hardware-aware training | P0 — bounded memory architecture for long-running agent sessions |
| **SAND** | High — narrow-deep alternation with bottleneck selection | P1 — hierarchical attention architecture for GWT tiered processing |
| **Differential Transformer** | Medium — differential attention for noise cancellation | P1 — noise filtering framework for GWT and KB retrieval |
| **MoA** | Medium — collaborative amplification without training | P2 — multi-domain collaboration pattern for NeoTrix's 7-domain architecture |

## Implementation Roadmap

| Phase | Action | Paper |
|-------|--------|-------|
| **Immediate** | Design three-tier KB: raw (recent) → compressed (aged) → structural (persistent) | TITAN |
| **Week 2** | Implement gated linear recurrence for session state (O(1) state size) | GLA |
| **Month 1** | Prototype narrow-deep attention alternation for GWT tiered processing | SAND |
| **Month 2** | Add differential attention for KB noise cancellation (dual retrieval subtraction) | Differential Transformer |
| **Quarter** | Evaluate MoA-style domain collaboration (7 domains as proposers, GWT as aggregator) | MoA |
