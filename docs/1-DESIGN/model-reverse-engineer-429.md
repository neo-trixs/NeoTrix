# Model Reverse Engineering — Cycle 429

**Date**: 2026-09-12
**Focus**: Recent papers on efficient inference, attention, agent coordination
**Sources**: arXiv (Apr–Sep 2026), ICML 2026, MLSP 2026, EMNLP 2026
**Cross-referenced against cycles 318–428 for novelty**

---

## Paper 1: Flux Attention — Context-Aware Hybrid Attention for Efficient LLM Inference

**URL**: https://arxiv.org/abs/2604.07394
**Date**: 2026-04-08
**Venue**: arXiv (under review)

### Core Contribution
Introduces Flux Attention — a context-aware framework that dynamically optimizes attention computation at the layer level. Integrates a lightweight Layer Router into frozen pretrained LLMs, adaptively routing each layer to Full Attention (FA) or Sparse Attention (SA) based on input context. Layer-wise routing preserves high-fidelity information retrieval while ensuring contiguous memory access. Parameter-efficient: only 12 hours training on 8×A800 GPUs. Achieves 2.8× prefill speedup and 2.0× decode speedup.

### Key Insight
> "Existing hybrid attention methods rely on static allocation ratios that fail to accommodate variable retrieval demands of different tasks. Head-level dynamic sparsity introduces computational load imbalance and synchronization long-tails."

The critical innovation is **layer-level** (not head-level) routing. Early layers need full attention for local patterns; deeper layers can use sparse attention for long-range dependencies. This avoids the load imbalance of head-level sparsity while still capturing layer heterogeneity.

### Architecture Mapping to NeoTrix

| Flux Attention Component | NeoTrix Domain | Pattern |
|--------------------------|---------------|---------|
| Layer Router (lightweight classifier) | NT-CORE (GWT) | GWT salience router — lightweight module deciding attention allocation |
| Layer-wise FA/SA routing | NT-CORE (HyperCube) | HyperCube could route layers to different knowledge representation modes |
| Context-dependent routing | NT-CORE (E8) | E8 hexagram selection depends on input context, not static config |
| Contiguous memory access | NT-MEMORY | KB access patterns optimized for locality |
| 12-hour training budget | NT-MIND (SEAL) | SEAL distillation with constrained compute budget |

### Absorption Candidates
- **GWT Layer-Aware Routing**: GWT currently routes at the module level. Flux Attention suggests routing at finer granularity — within a module, different "layers" of processing could use different attention modes. For E8 reasoning, early "layers" (pattern matching) use full attention; later "layers" (synthesis) use sparse attention.
- **HyperCube Multi-Mode Storage**: HyperCube could store knowledge in multiple modes (dense for frequent access, sparse for long-tail) and switch based on query context. This is Flux Attention's layer-level routing applied to knowledge representation.
- **SEAL Compute Budget Distillation**: SEAL cycles could adopt Flux Attention's constrained training approach — distill experiences within a fixed compute budget, prioritizing high-impact knowledge over exhaustive coverage.

### Implementation Priority: P0
Direct enhancement to GWT routing efficiency and HyperCube storage optimization.

---

## Paper 2: Latent Action Reparameterization (LAR) — Efficient Agent Inference

**URL**: https://arxiv.org/abs/2605.18597
**Date**: 2026-05-18
**Venue**: arXiv (under review)

### Core Contribution
Proposes Latent Action Reparameterization (LAR) — learns a compact latent action space where each latent action corresponds to a multi-step semantic behavior. Reparameterizes agent actions into latent units, enabling decision making over shorter effective horizon while preserving expressiveness. Unlike hand-crafted macros or hierarchical controllers, latent actions are learned from agent trajectories and integrated directly into the model.

### Key Insight
> "A key bottleneck lies in the representation of the action space itself. LAR enables decision making over a shorter effective horizon while preserving the expressiveness of the original action space."

The fundamental insight is that **action representation** is a critical and underexplored factor in scaling efficient agent inference. Current agents use low-level textual actions (tool calls, code edits) which create long decision horizons. LAR compresses these into learned semantic units.

### Architecture Mapping to NeoTrix

| LAR Component | NeoTrix Domain | Pattern |
|---------------|---------------|---------|
| Latent action space | NT-ACT | Tool call abstraction — semantic action units replace raw tool invocations |
| Multi-step semantic behaviors | NT-ACT (CapabilityBridge) | CapabilityBridge maps tree nodes to runtime capabilities = semantic action clusters |
| Learned from trajectories | NT-MEMORY (experience-tree) | Experience absorption distills trajectories into latent action patterns |
| Shorter decision horizon | NT-CORE (GWT) | GWT broadcast compressed to semantic units, not individual signals |
| Integrated into model | NT-MIND (SEAL) | SEAL crystallization bakes learned actions into production capabilities |

### Absorption Candidates
- **NT-ACT Latent Action Library**: NT-ACT currently exposes individual tool calls. LAR suggests learning multi-step action patterns from execution trajectories and storing them as latent actions. A "Scry" Runeword (data→filter→transform→persist) could be a single latent action, not 4 separate tool calls.
- **Experience-Tree Action Distillation**: The experience-tree absorption pipeline could extract latent action patterns from successful trajectories. Instead of storing raw tool call sequences, distill them into semantic action units. This reduces experience storage size and accelerates retrieval.
- **GWT Compressed Broadcasting**: GWT currently broadcasts individual salient signals. LAR suggests broadcasting compressed semantic units — "execute data pipeline" instead of "fetch→parse→filter→store". This reduces broadcast overhead.

### Implementation Priority: P0
Foundational enhancement to NT-ACT action representation and experience-tree compression.

---

## Paper 3: GLIDE — Guided Layerwise Hybrid Attention for Efficient LLM Inference

**URL**: https://arxiv.org/abs/2607.24788
**Date**: 2026-06-26
**Venue**: arXiv (under review)

### Core Contribution
Proposes GLIDE — a Guided Layerwise Hybrid Attention that strategically integrates sliding-window softmax attention with linear recurrent aggregation. Motivated by layer-wise heterogeneity: early layers exhibit high sensitivity to softmax removal, while deeper layers demonstrate redundancy and tolerate aggressive replacement by linear alternatives. Each layer balances an efficient linear recurrence with a variable-sized softmax window.

### Key Insight
> "GLIDE non-uniformly compresses the softmax footprint across the model, reducing aggregate KV cache I/O while preserving expressive power where most vital."

The key insight is **non-uniform compression**: not all layers need the same attention mechanism. Early layers are sensitive (keep softmax); deep layers are redundant (replace with linear recurrence). This is a more nuanced version of Flux Attention's layer-level routing.

### Architecture Mapping to NeoTrix

| GLIDE Component | NeoTrix Domain | Pattern |
|-----------------|---------------|---------|
| Layer-wise heterogeneous attention | NT-CORE (GWT) | GWT salience weights vary by module depth — core modules need full attention, leaf modules can use compressed |
| Variable-sized softmax window | NT-MEMORY | KB retrieval uses full attention for recent entries, compressed for historical |
| Linear recurrence for deep layers | NT-MEMORY (kv_cache_optimizer) | KV cache optimization: hot data in GPU, warm in CPU, cold compressed |
| Non-uniform compression | NT-MIND (SEAL) | SEAL distillation depth varies by experience importance |

### Absorption Candidates
- **GWT Heterogeneous Module Attention**: GWT should apply different attention mechanisms to different module depths. NT-CORE (critical path) gets full attention; NT-IO (leaf nodes) gets compressed linear attention. This mirrors GLIDE's layer-wise heterogeneity.
- **KB Tiered Retrieval**: KB retrieval could adopt GLIDE's tiered approach — recent experiences use full vector search; historical experiences use compressed BM25. Variable retrieval quality based on recency.
- **SEAL Adaptive Distillation Depth**: SEAL distillation could compress more aggressively for low-impact experiences (deep layers) while preserving full detail for high-impact experiences (early layers).

### Implementation Priority: P1
Enhancement to GWT routing and KB retrieval optimization.

---

## Paper 4: ActiveMem — Distributed Active Memory for Long-Horizon LLM Reasoning

**URL**: https://arxiv.org/abs/2606.10532
**Date**: 2026-06-09
**Venue**: arXiv (under review)

### Core Contribution
Proposes ActiveMem — a heterogeneous framework that decouples agent memory from the core reasoning process. Inspired by functional synergy between prefrontal cortex (reasoning) and hippocampus (memory). Two modules: (1) Planner handles reasoning and top-down query generation over compact context; (2) Distributed Memory System replaces monolithic context buffers with parallelized, sharded architecture. Three components: Memorizers (concurrent document processing), Memory Shards (partitioned storage), and consolidation across task lifecycle.

### Key Insight
> "Structurally decoupling memory systems from reasoning processes" — the Planner issues retrieval queries and integrates returned semantic gists, while the Distributed Memory System handles all storage and consolidation independently.

The biological inspiration is key: the hippocampus doesn't participate in active reasoning — it provides memories on demand. This decoupling allows each system to scale independently.

### Architecture Mapping to NeoTrix

| ActiveMem Component | NeoTrix Domain | Pattern |
|---------------------|---------------|---------|
| Planner (reasoning core) | NT-CORE (ConsciousnessTree) | ConsciousnessTree as Planner — generates queries, integrates results |
| Distributed Memory System | NT-MEMORY | KB as distributed memory — shards, consolidation, retrieval |
| Memorizers (concurrent processing) | NT-MEMORY (experience-tree) | Experience-tree absorption: concurrent distillation of documents |
| Memory Shards | NT-MEMORY (KB namespaces) | KB `experience` namespace = memory shard pattern |
| Consolidation across lifecycle | NT-MIND (SEAL) | SEAL pipeline = memory consolidation across evolution cycles |
| Decoupled memory-reasoning | NT-NEXUS | Nexus-weaver bridges memory and reasoning across sessions |

### Absorption Candidates
- **ConsciousnessTree-Memory Decoupling**: Currently, ConsciousnessTree and NT-MEMORY are tightly coupled. ActiveMem suggests explicit decoupling: ConsciousnessTree as pure Planner (reasoning + query generation), NT-MEMORY as pure storage (shards + consolidation). This allows independent scaling.
- **Experience-Tree Memorizer Pattern**: The experience-tree absorption could adopt ActiveMem's Memorizer pattern — concurrent processing of multiple experiences with parallelized distillation. Currently sequential; could parallelize across experience branches.
- **KB Shard Architecture**: KB could be explicitly sharded by domain (NT-* namespaces as shards) with cross-shard consolidation. This is already implicit in KB design; ActiveMem makes it explicit and optimizable.

### Implementation Priority: P0
Foundational architectural insight for NT-CORE/NT-MEMORY decoupling.

---

## Paper 5: Not All Thoughts Need HBM — Semantics-Aware Memory Hierarchy for LLM Reasoning

**URL**: https://arxiv.org/abs/2605.09490
**Date**: 2026-05-10
**Venue**: AdaptFM Workshop @ ICML 2026

### Core Contribution
Introduces a semantics-aware memory hierarchy that sorts KV cache tokens into four tiers — HBM, DDR, compressed, and evicted — using cumulative attention scoring. Low-importance tokens are moved to CPU memory rather than destroyed; before each attention step they are prefetched back at full precision. Formalizes zero-approximation-error offloading. Key finding: accuracy depends solely on eviction ratio (tokens permanently discarded), not on how many remain in HBM. At 3% eviction, retains 91% accuracy on GSM8K and 71% on MATH-500. 5-7% transfer overhead.

### Key Insight
> "Must every token live in HBM, or can some live elsewhere? Low-importance tokens are moved to CPU memory rather than destroyed."

The critical insight is that **offloading ≠ eviction**. Tokens moved to CPU and prefetched back contribute identically to tokens that never left GPU. Only permanent eviction (deletion) reduces accuracy. This creates a safe optimization space: move low-importance tokens to cheaper storage, prefetch when needed.

### Architecture Mapping to NeoTrix

| HBM Hierarchy Component | NeoTrix Domain | Pattern |
|-------------------------|---------------|---------|
| 4-tier memory hierarchy (HBM/DDR/compressed/evicted) | NT-MEMORY | KB tiered storage: hot (GPU/SSD) / warm (SQLite) / cold (compressed) / archived |
| Cumulative attention scoring | NT-CORE (GWT) | GWT salience = attention scoring for memory importance |
| Zero-approximation-error offloading | NT-MEMORY | Experience-tree: move low-priority experiences to cold storage, restore on query |
| Eviction ratio as accuracy knob | NT-MIND (SEAL) | SEAL distillation controls experience retention ratio |
| 5-7% transfer overhead | NT-IO | KB query latency budget — acceptable overhead for tiered retrieval |

### Absorption Candidates
- **KB Tiered Storage**: KB could implement explicit 4-tier storage: hot (recent, full detail) → warm (indexed, summary) → cold (compressed, searchable) → archived (deleted, recoverable). Currently implicit; this paper provides the theoretical framework.
- **Experience-Tree Eviction Policy**: Experience-tree currently retains all experiences. HBM hierarchy suggests controlled eviction: keep top-K experiences in hot storage, move rest to cold. Eviction ratio = quality knob. At 3% eviction, 91% accuracy retained — acceptable for most use cases.
- **GWT Memory Importance Scoring**: GWT salience could double as memory importance scoring. When GWT broadcasts a signal, mark related memories as "high importance" and keep them in hot storage. Low-salience memories naturally migrate to cold storage.

### Implementation Priority: P1
Enhancement to KB storage architecture and experience-tree retention policy.

---

## Cross-Cutting Patterns (Cycle 429)

### Pattern 1: Layer-Wise Heterogeneity
Flux Attention, GLIDE, and ActiveMem all exploit the insight that not all components need the same treatment. Early/critical components get full resources; deep/leaf components get compressed treatment. This is the "non-uniform compression" principle.

**NeoTrix mapping**: GWT should apply heterogeneous attention to different module depths. NT-CORE (critical path) gets full attention; NT-IO (leaf) gets compressed. KB retrieves full detail for recent experiences, compressed for historical.

### Pattern 2: Decouple Memory from Reasoning
ActiveMem's biological insight (hippocampus ≠ prefrontal cortex) and LAR's latent action space both suggest separating storage from computation. Memory systems should be independent modules, not embedded in reasoning loops.

**NeoTrix mapping**: ConsciousnessTree (reasoning) should be explicitly decoupled from NT-MEMORY (storage). They communicate via queries and results, not shared state. This enables independent scaling.

### Pattern 3: Learned Action Abstraction
LAR's latent actions and SMC's speculative execution both compress action spaces. Instead of reasoning over raw tool calls, reason over learned semantic units. This shortens decision horizons without losing expressiveness.

**NeoTrix mapping**: NT-ACT should learn multi-step action patterns from experience-tree trajectories. Store as latent actions in the skill tree. GWT broadcasts semantic action units, not individual tool calls.

### Pattern 4: Offload ≠ Evict
The HBM hierarchy paper proves that moving data to cheaper storage is safe; only permanent deletion reduces quality. This creates a large safe optimization space for memory management.

**NeoTrix mapping**: KB should implement explicit offloading (hot→warm→cold→archive) with prefetch-on-demand. Eviction ratio becomes a quality knob: trade storage cost for accuracy.

---

## Absorption Candidates Summary

| # | Candidate | Source Paper | Priority | NeoTrix Domain |
|---|-----------|-------------|----------|----------------|
| 1 | GWT Layer-Aware Routing | Flux Attention | P0 | NT-CORE |
| 2 | NT-ACT Latent Action Library | LAR | P0 | NT-ACT |
| 3 | ConsciousnessTree-Memory Decoupling | ActiveMem | P0 | NT-CORE / NT-MEMORY |
| 4 | GWT Heterogeneous Module Attention | GLIDE | P1 | NT-CORE |
| 5 | KB Tiered Storage (4-tier) | HBM Hierarchy | P1 | NT-MEMORY |
| 6 | Experience-Tree Action Distillation | LAR | P1 | NT-MEMORY |
| 7 | SEAL Adaptive Distillation Depth | GLIDE | P1 | NT-MIND |
| 8 | Experience-Tree Eviction Policy | HBM Hierarchy | P2 | NT-MEMORY |
| 9 | GWT Compressed Broadcasting | LAR | P2 | NT-CORE |
| 10 | KB Query Latency Budget | HBM Hierarchy | P2 | NT-MEMORY |

---

## Implementation Roadmap

### Phase 1 (Immediate — P0)
1. GWT Layer-Aware Routing: Implement heterogeneous attention routing within GWT salience computation
2. NT-ACT Latent Action Library: Learn multi-step action patterns from experience-tree trajectories
3. ConsciousnessTree-Memory Decoupling: Explicit interface between reasoning (Planner) and storage (Memory Shards)

### Phase 2 (Short-term — P1)
4. GWT Heterogeneous Module Attention: Different attention mechanisms per module depth
5. KB Tiered Storage: Hot/warm/cold/archive tiers with prefetch-on-demand
6. SEAL Adaptive Distillation Depth: Variable compression based on experience importance

### Phase 3 (Medium-term — P2)
7. Experience-Tree Eviction Policy: Controlled eviction with quality knob
8. GWT Compressed Broadcasting: Semantic action units instead of individual signals
9. KB Query Latency Budget: Acceptable overhead for tiered retrieval

---

**Cycle**: 429 | **Date**: 2026-09-12 | **Next**: Cycle 430 — deeper dive on layer-wise heterogeneity + latent action patterns
