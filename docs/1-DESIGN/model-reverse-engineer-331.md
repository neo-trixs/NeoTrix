# Model Reverse Engineer — Cycle 331

**Date**: 2026-09-11
**Focus**: Efficient inference, attention mechanisms, agent coordination
**Papers**: 5 new papers (not in cycles 318–330)

---

## Paper 1: PackInfer — Compute- and I/O-Efficient Attention for Batched LLM Inference

**Source**: arXiv:2602.06072 (Feb 2026)
**Authors**: Rui Ning, Wei Zhang, Fan Lai
**Venue**: Distributed, Parallel, and Cluster Computing

### Core Innovation
Production LLM serving batches requests with highly heterogeneous sequence lengths. Prior work (FlashAttention) optimizes single-request attention but ignores the batch-level problem: this mismatch induces severe computation and I/O imbalance, exacerbates stragglers, and underutilizes GPU resources.

PackInfer is a kernel-level attention framework that addresses the **batch-level** problem:
1. **Load-balanced execution groups**: Orchestrates batched requests into balanced groups, saturating GPU utilization by packing multiple requests into unified kernel launches
2. **Packed query-key attention**: Constructs attention kernels directly over packed query-key regions, eliminating redundant computation and balancing thread-block execution
3. **I/O-aware grouping**: Co-locates shared-prefix requests and reorganizes KV caches into group-contiguous layouts, reducing memory fragmentation and redundant data movement

### Results
- 13-20% latency reduction vs FlashAttention
- 20% throughput improvement
- Particularly effective for heterogeneous workloads (mixed long/short sequences)

### NeoTrix Domain Mapping

| Domain | Integration Pattern |
|--------|-------------------|
| **NT-IO** | Batch-aware inference optimization for multi-domain workloads. Different NeoTrix modules (NT-CORE reasoning, NT-WORLD parsing, NT-MEMORY retrieval) generate heterogeneous sequence lengths — PackInfer's grouping strategy addresses this directly |
| **NT-CORE** | GWT attention routing could adopt I/O-aware grouping — co-locate requests with shared context (same domain knowledge) to maximize KV cache reuse |
| **NT-MEMORY** | Shared-prefix co-location aligns with experience-tree branch loading — sessions sharing the same domain context share KV cache pages |

### Key Architectural Insight
The real bottleneck in LLM serving is not single-request efficiency but **cross-request resource contention**. PackInfer treats the batch as a first-class optimization target. This validates NeoTrix's holistic architecture: optimizing individual modules in isolation misses system-level gains.

### Actionable Pattern
**Group-by-prefix for KV cache optimization**: When multiple NeoTrix modules share domain context (e.g., NT-CORE and NT-MIND both need NT-MEMORY knowledge), co-locate their KV cache entries. Shared prefixes = shared cache pages = reduced memory pressure.

---

## Paper 2: Select-then-Solve — Paradigm Routing as Inference-Time Optimization for LLM Agents

**Source**: arXiv:2604.06753 (Apr 2026)
**Authors**: Multiple (4 LLMs × 10 benchmarks × ~18k runs)
**Venue**: ACL/EMNLP track

### Core Innovation
Six inference-time reasoning paradigms (Direct, CoT, ReAct, Plan-Execute, Reflection, ReCode) are studied across four frontier LLMs and ten benchmarks. Key finding: **paradigms have complementary strengths — no single paradigm dominates**.

The paper proposes **select-then-solve**: before answering each task, a lightweight embedding-based router selects the most suitable paradigm. The router:
- Achieves near-optimal performance with <1% compute overhead
- Uses embedding-based similarity to historical task-paradigm pairs
- Dynamically selects per-task, not per-session

### Results
- Router captures ~95% of the best-paradigm performance
- <1% overhead (embedding lookup, not inference)
- Complementary paradigm strengths are consistent across LLMs

### NeoTrix Domain Mapping

| Domain | Integration Pattern |
|--------|-------------------|
| **NT-CORE** | E8 hexagram reasoning states can encode paradigm choices. Each hexagram line represents a reasoning paradigm dimension. Router selects optimal hexagram configuration per task |
| **GWT** | Attention routing adopts paradigm routing: salience scoring includes which reasoning paradigm best fits the incoming task |
| **NT-MIND** | SEAL pipeline distillation learns paradigm-task mappings from experience. After absorbing a skill, the system records which paradigm worked best |
| **Axiom A1** | Cost-Aware Routing: different paradigms have different token costs. Router optimizes for quality/cost ratio, not just quality |

### Key Architectural Insight
The paradigm routing finding validates NeoTrix's **Dual Specialization** (Weapon Set I/II). Rather than committing to one reasoning mode, the system should dynamically switch paradigms based on task characteristics. The <1% overhead of embedding-based routing means this can be a universal layer, not a special-case optimization.

### Actionable Pattern
**Embedding-based paradigm router**: For each incoming task, compute embedding similarity against historical paradigm-success pairs. Route to the paradigm that succeeded on similar tasks. Cost-weight: simpler paradigms (Direct) for simple tasks, complex paradigms (Reflection) for hard tasks. This is Axiom A1 (Cost-Aware Routing) applied to reasoning paradigms.

---

## Paper 3: Sketch&Walk — Sketch-and-Walk Sparse Attention for Efficient LLM Inference

**Source**: arXiv:2602.07397 (Feb 2026)
**Authors**: Hoang Anh Duy Le et al. (Rice University, Stevens)
**Venue**: Machine Learning

### Core Innovation
Training-free sparse attention that achieves **6x speedup** at 20% attention density with near-lossless accuracy. Two-stage approach:

1. **Hadamard Sketching**: Lightweight (O(n)) approximations of attention scores using randomized projections. No learned parameters — pure mathematical transformation
2. **Walk-based Aggregation**: Accumulates sketch scores across layers via a walk mechanism that captures attention influence beyond direct token-to-token interactions. Layer L's importance incorporates information from layers 1..L-1

The walk mechanism is the key insight: direct attention scores miss indirect influence. Token A may not attend to token C, but A→B→C creates indirect importance. Walk-based aggregation captures this.

### Results
- 6x inference speedup at 20% attention density
- Near-lossless accuracy across models and tasks
- Slightly outperforms dense attention in some settings (noise reduction)
- Training-free: zero additional compute for sparsity decisions

### NeoTrix Domain Mapping

| Domain | Integration Pattern |
|--------|-------------------|
| **NT-CORE** | GWT attention routing uses sketch-based salience: lightweight Hadamard sketches determine broadcast recipients, walk-based propagation determines multi-hop influence |
| **NT-MEMORY** | Experience-tree lazy loading adopts walk-based importance: a branch's importance includes indirect references from other branches, not just direct mentions |
| **NT-META** | ConsciousnessTree cycle health scoring uses walk-based propagation: module health influences depend on transitive dependencies, not just direct connections |
| **NT-WORLD** | Perception filtering: sketch-based attention determines which sensory events reach consciousness, walk-based aggregation captures cascading influence |

### Key Architectural Insight
**Indirect influence via walk aggregation** is the breakthrough. In NeoTrix's domain graph, NT-CORE influences NT-MEMORY which influences NT-MIND — but NT-CORE's effect on NT-MIND is indirect. Walk-based scoring captures this transitive influence without explicit modeling. This is how ConsciousnessTree should weight cross-domain health signals.

### Actionable Pattern
**Walk-based importance for experience-tree**: When loading experience branches, compute not just direct relevance (keyword match) but transitive relevance (branches that reference this branch, branches that this branch references, and their references). This captures the "indirect influence" that makes some experiences more important than their direct relevance suggests.

---

## Paper 4: AgentInfer — Co-Design of Inference Architecture and System for Efficient Agents

**Source**: arXiv:2512.18337 (Dec 2025, revised Feb 2026)
**Authors**: Weizhe Lin et al. (Huawei, multiple institutions)
**Venue**: Computation and Language

### Core Innovation
LLM-based agents suffer from systemic latency not from isolated model inference but from **accumulated latency across reasoning loops, context growth, and heterogeneous tool interactions**. AgentInfer co-designs the agent architecture with the inference system through four synergistic components:

1. **AgentCollab**: Hierarchical dual-model reasoning — large model plans, small model executes. Dynamic role assignment based on task complexity
2. **AgentSched**: Cache-aware hybrid scheduler minimizes latency under heterogeneous request patterns
3. **AgentSAM**: Suffix-automaton-based speculative decoding that **reuses multi-session semantic memory** — past conversations accelerate future inference. Low-overhead because suffixes are pre-computed
4. **AgentCompress**: Semantic compression that asynchronously distills and reorganizes agent memory **without disrupting ongoing reasoning**

Together these form a **Self-Evolution Engine** sustaining efficiency through long-horizon reasoning tasks.

### Results
- 50%+ reduction in ineffective token consumption
- 1.8-2.5x overall speedup with preserved accuracy
- Tested on BrowseComp-zh and DeepDiver benchmarks

### NeoTrix Domain Mapping

| Domain | Integration Pattern |
|--------|-------------------|
| **NT-CORE + NT-MIND** | AgentCollab's hierarchical dual-model reasoning maps to our E8 reasoning: large-model planning (deep hexagram analysis) + small-model execution (fast tool calls). Dynamic role assignment = Adaptive routing based on task complexity |
| **NT-MEMORY** | AgentSAM's multi-session memory reuse is exactly experience-tree lazy loading: past session knowledge accelerates future inference. Suffix-automaton = experience branch lookup by suffix pattern |
| **NT-MIND** | AgentCompress's async distillation is SEAL pipeline distillation: compress experience without blocking active reasoning. "Without disrupting ongoing reasoning" = background cycle execution |
| **NT-IO** | AgentSched's cache-aware scheduling informs our inference scheduling across LLM providers |

### Key Architectural Insight
**Co-design, not optimization-in-isolation.** The paper's core argument — optimizing inference without considering agent architecture misses systemic gains — directly validates NeoTrix's Six-Layer Architecture. Each layer's design should consider inference implications. AgentSAM's insight that "past conversations accelerate future ones" is the theoretical foundation for experience-tree.

### Actionable Pattern
**Suffix-automaton for experience lookup**: When a new task arrives, compute its suffix (last N tokens of context). Look up which experience branches had similar suffixes in the past. Those branches are most relevant for accelerating current inference. This is a concrete algorithm for experience-tree lazy loading.

---

## Paper 5: Squeezed Attention — Accelerating Long Context Length LLM Inference

**Source**: ACL 2025 (Long Papers)
**Authors**: Coleman Hooper et al. (UC Berkeley, Stanford)
**Venue**: ACL 2025

### Core Innovation
For long-context applications, a large portion of the input context is **fixed** (system prompts, documents, knowledge bases). Squeezed Attention exploits this structure:

1. **Offline K-means clustering**: Groups keys for the fixed context by semantic similarity, representing each cluster with a single centroid value
2. **Query-centroid comparison**: At inference, query tokens compare against centroids (not all keys) to predict which keys are relevant
3. **Hierarchical compression**: Reduces attention complexity from O(n) to O(log n) for fixed context length
4. **Sparse FlashAttention**: Custom kernels for centroid comparison and sparse attention over important keys only

### Results
- 3.1x KV budget reduction with no accuracy loss
- Up to 8x KV budget reduction with only 0.5 point accuracy gap
- 4x+ speedup in both prefill and generation phases
- Tested on LLaMA-2-7B-32K, LWM-Text-Chat-1M, Longchat-7B-v1.5-32K

### NeoTrix Domain Mapping

| Domain | Integration Pattern |
|--------|-------------------|
| **NT-MEMORY** | KB knowledge is "fixed context" — Squeezed Attention's centroid-based compression applies directly. Pre-compute centroids for domain knowledge clusters, use query-centroid matching at retrieval time |
| **NT-CORE** | System prompts and domain context (7 factions, 6 layers) are fixed context — compress via centroids, retrieve via query-centroid matching |
| **NT-WORLD** | Crawled document stores are fixed context — Squeezed Attention's clustering enables efficient retrieval from large document corpora |
| **KVMem** | Complementary to KVMem's paged KV — Squeezed Attention compresses fixed context, KVMem pages dynamic context. Combined: compressed fixed + paged dynamic |

### Key Architectural Insight
**Fixed vs. dynamic context separation.** The paper identifies that long-context efficiency comes from treating fixed context (knowledge base, system prompt) differently from dynamic context (user query, conversation). Fixed context can be pre-compressed (K-means clustering), dynamic context needs real-time processing. This separation is directly applicable to NeoTrix: domain knowledge is fixed, session context is dynamic.

### Actionable Pattern
**Centroid-based KB compression**: Pre-compute K-means centroids for each domain's knowledge clusters in KB. At query time, compare query against centroids to identify relevant clusters, then retrieve only from those clusters. This reduces KB retrieval from O(all nodes) to O(clusters + relevant nodes). Combined with experience-tree lazy loading, this provides two-level compression: branch-level (lazy) + cluster-level (centroid).

---

## Cross-Paper Synthesis

### Unified Insight: The Three Inference Efficiency Pillars

| Pillar | Papers | NeoTrix Application |
|--------|--------|-------------------|
| **Batch/Resource Optimization** | PackInfer, AgentSched | Group-by-prefix for multi-domain inference; cache-aware scheduling across LLM providers |
| **Paradigm/Route Selection** | Select-then-Solve, AgentCollab | Embedding-based paradigm router; dynamic role assignment for dual-model reasoning |
| **Context Compression** | Sketch&Walk, Squeezed Attention, AgentSAM, AgentCompress | Walk-based importance for experience-tree; centroid-based KB compression; suffix-automaton for experience lookup |

### NeoTrix Architecture Implications

1. **GWT Enhancement**: Add paradigm routing to salience scoring — each broadcast candidate includes which reasoning paradigm is optimal, not just which information is salient

2. **Experience-Tree Algorithm**: Combine three techniques:
   - Walk-based importance (Sketch&Walk) for transitive relevance
   - Suffix-automaton (AgentSAM) for suffix-pattern matching
   - Centroid-based compression (Squeezed Attention) for cluster-level retrieval

3. **Inference Scheduling**: Adopt PackInfer's batch-aware grouping for NeoTrix's multi-domain workloads — group requests by shared context prefix, co-locate KV cache entries

4. **Co-Design Principle**: AgentInfer's core lesson — optimize the agent architecture and inference system jointly, not in isolation. This validates our Six-Layer Architecture where each layer considers inference implications

---

## Action Items

| Technique | Paper | Target Module | Implementation Priority |
|-----------|-------|--------------|----------------------|
| Embedding-based paradigm router | Select-then-Solve | NT-CORE (E8 reasoning) | P1 |
| Walk-based experience importance | Sketch&Walk | NT-MEMORY (experience-tree) | P1 |
| Suffix-automaton experience lookup | AgentSAM | NT-MEMORY (experience-tree) | P1 |
| Centroid-based KB compression | Squeezed Attention | NT-MEMORY (KB retrieval) | P1 |
| Batch-aware KV cache grouping | PackInfer | NT-IO (inference scheduling) | P2 |
| Dual-model hierarchical reasoning | AgentCollab | NT-CORE + NT-MIND | P2 |
| Async memory distillation | AgentCompress | NT-MIND (SEAL pipeline) | P2 |
| Hierarchical attention sparsity | Sketch&Walk | NT-CORE (GWT attention) | P3 |
