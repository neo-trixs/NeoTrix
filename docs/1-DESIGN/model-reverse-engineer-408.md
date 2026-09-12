# Model Reverse Engineering — Cycle 408

**Date**: 2026-09-12
**Focus**: Efficient inference, attention sparsity, agent coordination architectures
**Sources**: arXiv, ICML 2026, ACL 2026, ACL Anthology

---

## Paper 1: Faster Flash Decoding (ICML 2026)

**Paper**: "Faster Than Flash: Exploiting Attention Sparsity for Efficient Long-Context Decoding"
**Authors**: Not specified | **Venue**: ICML 2026 | **arXiv**: 2609.00097

### Core Mechanism
Hardware-algorithm co-design that fuses selector and computer into a single CUDA kernel. Replaces external metadata indices with content-aware scanning via low-bit quantization. Top-delta strategy dynamically filters blocks for distribution-adaptive sparsity without global synchronization.

### Key Innovation
- **Fully Fused Kernel**: Selector and attention computation merged into one kernel — no separate index step
- **Top-Delta Strategy**: Dynamically filters KV blocks based on distribution-adaptive sparsity. No global synchronization needed
- **Low-Bit Quantization for Scanning**: Content-aware scanning using quantized representations replaces metadata indices
- **11.6x kernel-level speedup**, 2.37x end-to-end throughput, scales to 256K context

### Pattern Extracted
**Fuse-then-Filter**: Instead of separate index → compute steps, fuse selection and computation into one kernel. The top-delta strategy acts as a learned hardware-aware filter that adapts to input distribution.

### NeoTrix Mapping

| Component | Integration Point | Pattern |
|-----------|-------------------|---------|
| **GWT** | Attention broadcast | Fuse salience scoring with routing into single pass |
| **KVMem** | KV cache management | Fuse cache scoring + eviction into one operation |
| **Heartbeat Aggregator** | Health signal collection | Fuse signal collection + aggregation into one pass |
| **Cost-Aware Routing (A1)** | Model selection | Fuse capability matching + cost estimation into one decision |

**Actionable**: Apply fuse-then-filter pattern to GWT — instead of scoring salience then routing, perform fused salience-routing in one pass. Reduces two O(N) passes to one O(N) pass with fused kernel.

---

## Paper 2: CEDAR — Coarse-to-fine Error-aware Dynamic Attention Routing

**Paper**: "CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention"
**Authors**: Not specified | **arXiv**: 2609.07237

### Core Mechanism
Coarse-to-fine attention with error-bounded refinement. Each semantic chunk contributes a cheap key-value summary to a residual attention path. Chunks with high estimated approximation error are expanded to exact token attention. Exact and summarized contributions combined in single softmax normalization — refinement replaces rather than duplicates coarse evidence.

### Key Innovation
- **Residual Attention Path**: Cheap KV summaries run in parallel with exact attention — refinement replaces, not duplicates
- **Error-Bounded Refinement**: Output error bound governed by within-chunk key/value dispersion. Variable refinement budget allocated per chunk
- **98% reduction in reconstruction error** vs hard dropping at equal budget. ~3x kernel speedup at 128K context

### Pattern Extracted
**Residual Approximation with Error Bounds**: Maintain a cheap approximate path alongside expensive exact path. Only promote chunks to exact computation when approximation error exceeds a bound. The key insight: refinement replaces coarse evidence rather than adding to it.

### NeoTrix Mapping

| Component | Integration Point | Pattern |
|-----------|-------------------|---------|
| **GWT** | Attention allocation | Cheap approximate routing + expensive exact routing per task |
| **SEAL Pipeline** | Phase execution | Approximate exploration first, exact execution only when error bound exceeded |
| **KB** | Query processing | Approximate retrieval first, exact search only for high-error queries |
| **HyperCube** | VSA embedding | Approximate vector similarity first, exact computation for ambiguous matches |

**Actionable**: Implement residual attention for GWT — maintain cheap approximate salience scores per domain, only compute exact salience when approximation error exceeds threshold. Budget-aware attention allocation.

---

## Paper 3: Declarative Attention

**Paper**: "Language Models Can Control Their Own Attention"
**Authors**: Not specified | **arXiv**: 2609.02737

### Core Mechanism
Models declare WHERE they need to attention within their chain-of-thought. Partition generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). Inference engine parses these declarations like tool calls and skips most KV cache reads.

### Key Innovation
- **Self-Declared Attention Regions**: Model explicitly declares attention scope via structured tags during generation
- **Three-Mode Partitioning**: global/focus/local — model chooses mode per generation step
- **52% reduction in attended tokens** (Gemma-4-31B), 31.1% (Qwen-3.6-27B) with modest accuracy drops (1.27pp, 2.75pp)
- **Zero-shot on off-the-shelf models** — no training required

### Pattern Extracted
**Intrinsic Attention Control**: Instead of external proxies scoring token importance, the model itself declares where attention is needed. This is fundamentally different from sparse attention methods that use external scoring — the model becomes the router.

### NeoTrix Mapping

| Component | Integration Point | Pattern |
|-----------|-------------------|---------|
| **GWT** | Self-directed attention | Agent declares its own attention scope per reasoning step |
| **ConsciousnessTree** | Branch selection | Agent declares which branches need full processing vs approximate |
| **NT-MIND** | Self-evolution | Agent declares its own evolution scope — full self-analysis vs targeted improvement |
| **SEAL Pipeline** | Phase routing | Agent declares which SEAL phases need full execution vs skip |

**Actionable**: Implement declarative attention routing for NeoTrix agents — agents declare `<global>` (full context processing), `<focus>` (specific domain), or `<local>` (recent interactions only) per reasoning step. Enables agent-controlled compute allocation.

---

## Paper 4: EvoSparse — Evolving Sparsity with Token Importance Dynamics

**Paper**: "Evolving Sparsity: Leveraging Token Importance Dynamics for Efficient LLM Decoding with Sparse Attention"
**Authors**: Ruizi Han, Miao Zhang et al. | **Venue**: ACL 2026 | **Code**: github.com/iLearn-Lab/ACL26-EvoSparse

### Core Mechanism
Models token importance as a dynamic process that evolves over decoding steps and propagates through model layers. Two mechanisms: (1) Cross-Step Accumulation — incrementally maintains long-term importance via decayed accumulation of sparse attention scores, avoiding recomputation; (2) Cross-Layer Propagation — Retrieval Heads compute query-aware indices and propagate them to Standard Heads.

### Key Innovation
- **Temporal Consistency**: Token importance is not transient — significant tokens serve as Long-term Anchors, adjacent steps exhibit Short-term Reuse
- **Head-Level Capability Gap**: Retrieval Heads accurately locate key info; Standard Heads fail independently but can be guided by Retrieval Head indices
- **5.36x attention speedup**, 2.33x end-to-end decoding speedup
- Approaches full attention performance in most settings

### Pattern Extracted
**Evolved Importance with Cross-Layer Guidance**: Token importance evolves over time (not recomputed from scratch each step) and propagates across layers (specialist heads guide general heads). The key insight: importance is a dynamic state, not a static score.

### NeoTrix Mapping

| Component | Integration Point | Pattern |
|-----------|-------------------|---------|
| **GWT** | Salience evolution | Salience scores evolve over time, not recomputed each cycle |
| **ConsciousnessTree** | Branch health tracking | Branch health evolves over cycles, propagated across branches |
| **KB** | Query relevance | Query relevance accumulates over sessions, cross-query guidance |
| **NT-MIND** | Skill importance | Skill importance evolves based on usage patterns, cross-skill guidance |

**Actionable**: Implement evolved salience for GWT — maintain accumulated importance scores per domain that evolve over attention cycles, with specialist domains (Retrieval Heads) guiding general domains. Avoids redundant salience recomputation.

---

## Paper 5: BIGMAS — Brain-Inspired Graph Multi-Agent Systems

**Paper**: "Brain-Inspired Graph Multi-Agent Systems for LLM Reasoning"
**Authors**: Guangfu Hao et al. | **arXiv**: 2603.15371

### Core Mechanism
Inspired by Global Workspace Theory. Specialized LLM agents organized as nodes in a dynamically constructed directed graph. GraphDesigner constructs task-specific agent topologies per problem. Global Orchestrator leverages complete shared workspace for routing decisions. All agents coordinate through centralized shared workspace maintaining globally consistent task state.

### Key Innovation
- **Per-Problem Graph Construction**: GraphDesigner builds different topologies for different problems — routine tasks get compact pipelines, complex tasks recruit broader coalitions
- **Shared Workspace Coordination**: All agents see complete shared state (not local views). Eliminates information fragmentation across agents
- **Orthogonal to Model Scaling**: Multi-agent architectural coordination provides gains complementary to model-level reasoning improvements
- **Routing count as difficulty proxy**: Number of routing decisions naturally indicates instance difficulty without explicit scheduling logic

### Pattern Extracted
**Dynamic Coalition via Shared Workspace**: Instead of fixed agent topologies, construct per-problem agent graphs. All agents coordinate through a shared workspace that maintains global state. Architecture provides gains orthogonal to model capability.

### NeoTrix Mapping

| Component | Integration Point | Pattern |
|-----------|-------------------|---------|
| **GWT** | Dynamic attention coalitions | Per-task domain coalitions formed dynamically, not fixed topology |
| **ConsciousnessTree** | Branch coordination | Branch coalitions form per-cycle based on task demands |
| **NT-ACT** | Multi-agent routing | Agent topologies constructed per-problem, not predefined |
| **Heartbeat Aggregator** | System state | Shared workspace = global health state visible to all agents |

**Actionable**: Implement dynamic coalition formation for GWT — instead of fixed domain routing, construct per-task attention coalitions via a lightweight GraphDesigner. Shared workspace ensures all domains see complete system state. Architecture provides gains orthogonal to model scaling.

## Meta-Pattern: The Attention-Coordination Convergence

All five papers converge on a single meta-pattern: **attention and coordination are the same problem at different scales**.

| Scale | Attention Problem | Coordination Problem |
|-------|-------------------|---------------------|
| **Token** | Which tokens to attend to | Which agents to activate |
| **Layer** | Which layers need full computation | Which coalitions need full state |
| **Session** | Which context regions matter | Which domains need attention |
| **Cross-Session** | What knowledge persists | What learning transfers |

The papers suggest that the same algorithmic patterns work across scales:
1. **Fuse-then-filter** (FFD): Merge scoring and selection into one pass
2. **Residual approximation with error bounds** (CEDAR): Cheap path + expensive path, promote only when error exceeds bound
3. **Intrinsic control** (Declarative Attention): Let the system declare its own attention scope
4. **Evolved importance with cross-layer guidance** (EvoSparse): Importance evolves over time, specialists guide generalists
5. **Dynamic coalition via shared workspace** (BIGMAS): Per-problem agent graphs with global state visibility

**NeoTrix Implication**: These five patterns can be applied uniformly across NeoTrix's architecture — from token-level KV cache management to domain-level attention routing to cross-session learning. The GWT attention mechanism is the natural integration point, as it already operates as a cross-domain attention router.
