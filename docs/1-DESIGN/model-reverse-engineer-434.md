# Model Reverse Engineering — Cycle 434

**Date**: 2026-09-12
**Scope**: 5 recent papers/models on efficient inference, attention, agent coordination
**Method**: Pattern extraction → NeoTrix 7-domain mapping
**Exclusion**: Papers covered in cycle 431 (Gated-Memory Routing, AGAO, UNISON, RouteRelay, Procedural Graphs)

---

## 1. CEDAR — Error-Bounded Residual Routing for Long-Context Attention (arXiv:2609.07237)

**Paper**: *CEDAR: Coarse-to-fine Error-aware Dynamic Attention Routing*
**Published**: 2026-09-07

### Core Insight
Hard sparse attention assigns zero probability to omitted chunks — a routing miss cannot be recovered. CEDAR keeps global coverage via **residual summaries** with variable refinement budget based on approximation error.

### Mechanism
- **Residual KV summaries**: Each semantic chunk contributes a cheap summary to a residual attention path
- **Error-aware refinement**: Chunks with high estimated approximation error are expanded to exact token attention
- **Single softmax normalization**: Exact + summarized contributions combined — refinement replaces, not duplicates, coarse evidence
- **Output-error bound**: Governed by within-chunk key/value dispersion → variable refinement budget per query
- **Coarse-to-fine**: Frozen LM, no training required

### Results
- Residual summaries reduce reconstruction error by 98%+ vs hard dropping at equal budgets
- 3x kernel speedup at 128K context with near-dense quality
- Recovers most quality lost by hard sparse routing

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Error-bounded refinement → GWT attention refinement. When salience estimation has high uncertainty (high dispersion), allocate more computation. CEDAR's error bound = GWT confidence threshold for attention allocation. |
| **NT-MEMORY** | Residual summaries = lightweight KB node summaries. Full retrieval only when summary error exceeds threshold. Maps to experience-tree lazy branch loading — load summary first, expand only when needed. |
| **NT-MIND** | Coarse-to-fine refinement → SEAL pipeline's graduated stages. cheap summary at Phase-0, full analysis only at Phase-3+ when error warrants it. Resource-aware evolution. |
| **NT-WORLD** | Semantic chunking → crawl pipeline content segmentation. Error-aware routing → prioritize high-information segments for full parsing, skip low-value boilerplate. |

### Actionable Pattern for NeoTrix
**Error-Bounded KB Retrieval**: When querying experience namespace, load node summaries first. Compute dispersion score. Only expand to full text when dispersion > threshold. This reduces retrieval cost for routine queries while preserving quality for ambiguous queries. Maps directly to Axiom A2 (Context as Scarce Resource).

---

## 2. Declarative Attention — Models Control Their Own Attention (arXiv:2609.02737)

**Paper**: *Language Models Can Control Their Own Attention*
**Published**: 2026-09-02

### Core Insight
Models already know which context parts are relevant — instead of external proxy scoring (still O(N)), let the model **declare** where to attend via chain-of-thought. Three modes: `<global>`, `<focus>`, `<local>`.

### Mechanism
- **Declarative protocol**: Model emits `<global>`, `<focus>`, `<local>` tags in chain-of-thought
- **Engine parses declarations**: Like tool calls — skips most KV cache read
- **Zero-shot**: No training required, works on off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B)
- **Intrinsic routing**: Model decides attention scope from its own reasoning, not external scorer

### Results
- 52.0% reduction in total attended tokens (Gemma-4-31B), 31.1% (Qwen-3.6-27B)
- Modest accuracy drops (1.27pp, 2.75pp) that shrink with model scale
- Unlocks new axis of sparse attention: model-intrinsic vs extrinsic scoring

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Declarative Attention = model-intrinsic GWT. Current GWT uses external salience scoring; DA suggests the model itself should declare attention scope. ConsciousnessTree's awareness_score() could become model-declared, not computed. |
| **NT-MEMORY** | `<focus>` mode = targeted KB retrieval. `<local>` = session-only context. `<global>` = full namespace scan. Model declares which memory tier to access based on its own reasoning needs. |
| **NT-IO** | Declaration parsing = tool-call-like protocol. Agent emits attention scope declarations that the runtime engine interprets. Maps to NT-IO's protocol layer design. |
| **NT-MIND** | Self-declared attention → SEAL stage selection driven by model's own assessment of task difficulty, not fixed schedule. |

### Actionable Pattern for NeoTrix
**Model-Declared Attention Scopes**: Extend GWT to accept model-declared attention scope declarations. When the model emits `<focus>` on a specific domain, GWT allocates full computation to that domain and uses lightweight summaries for others. This inverts current GWT — model drives attention, not salience scores.

---

## 3. HeRo — History-Aware Routing with Router Memory (arXiv:2609.08189)

**Paper**: *Do Dynamic Routers Need Memory? HeRo: History-Aware Routing for Efficient LLM Inference*
**Published**: 2026-09-08

### Core Insight
Dynamic layer routing treats each decision as local (current hidden state only). But earlier routing decisions shape downstream representations — the problem is **sequential and path-dependent**. Solution: **explicit routing memory** via linear attention.

### Mechanism
- **Router memory**: Linear attention incrementally aggregates preceding routing scores + residual updates into compact history
- **Joint conditioning**: Each routed layer conditions on accumulated state + current hidden representation
- **Frozen backbone**: Only lightweight routers and adapters trained, no pretrained parameter modification
- **Token-wise FFN routing**: Each token independently decides to skip or execute each FFN layer

### Results
- Llama 3.1-8B: 26.87% parameter bypass at 100.24% dense performance
- 38.82% bypass at 97.01% performance under tighter budget
- Removing routing history degrades most on multistep reasoning and code generation

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Router memory → GWT attention history. Current GWT doesn't condition on prior routing decisions. HeRo proves this matters — GWT should maintain a compact representation of recent attention allocations. |
| **NT-MIND** | History-aware routing → SEAL stage selection should condition on which stages were selected in recent cycles. Path-dependent evolution: earlier stage choices affect which stages are useful next. |
| **NT-MEMORY** | Routing memory = execution memory within a cycle. Not just "what happened" but "what was routed where" — a control signal, not just a log. |
| **NT-ACT** | Token-wise routing → per-task tool skip/execute decisions. If a tool was recently effective for similar tokens, bias toward reuse. |

### Actionable Pattern for NeoTrix
**GWT Routing Memory**: Add a lightweight linear-attention memory to GWT that tracks recent attention allocations. Each salience computation conditions on this memory, enabling path-dependent attention. This prevents GWT from oscillating between domains when a stable allocation would be better. Critical for multistep reasoning tasks.

---

## 4. Faster Than Flash — Fused Sparse Attention Decoding (arXiv:2609.00097)

**Paper**: *Faster Than Flash: Exploiting Attention Sparsity for Efficient Long-Context Decoding*
**Published**: 2026-08-31 (ICML 2026 accepted)

### Core Insight
Metadata-based sparse attention has memory overhead; adaptive selection has compute inefficiency. Solution: **fuse selector and computer into a single kernel** with content-aware scanning via low-bit quantization.

### Mechanism
- **Fused kernel**: Selector + computer in one CUDA kernel, no external metadata indices
- **Content-aware scanning**: Low-bit quantization (FP8/FP4) for in-kernel relevance scoring
- **Top-delta strategy**: Dynamically filters blocks for distribution-adaptive sparsity without global synchronization
- **Scan reuse**: Scanning results reused for computation — no redundant passes
- **Training-free, plug-and-play**: No model modification required

### Results
- 11.6x kernel-level speedup, 2.37x end-to-end throughput improvement
- Scales to 256K context length
- Maintains model accuracy on RULER and LongBench

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Fused attention kernel → GWT fused salience computation. Current GWT: compute salience, then route. FFD pattern: fuse scoring + routing into one pass. |
| **NT-MEMORY** | Top-delta filtering → KB retrieval top-k with adaptive k. Distribution-adaptive: when query is ambiguous, retrieve more; when focused, retrieve fewer. |
| **NT-PHYSICAL** | Hardware-algorithm co-design → NT-PHYSICAL's compute optimization for constrained devices. Low-bit scanning maps to quantized inference on edge hardware. |
| **NT-IO** | Training-free plug-and-play → NT-IO's provider-agnostic integration. Any model, any provider, same fused attention optimization. |

### Actionable Pattern for NeoTrix
**Fused GWT Salience+Routing**: Fuse GWT's salience computation and module routing into a single pass. Currently two separate steps (compute salience → route based on salience). FFD proves fusing eliminates metadata overhead and enables scan reuse. Critical for real-time attention routing in long sessions.

---

## 5. ConvMem — Convolutional Memory for Long-Context Reasoning (arXiv:2609.10441)

**Paper**: *ConvMem: Convolutional Memory for Long-Context Reasoning*
**Published**: 2026-09-09

### Core Insight
Sequential memory approaches (like MemAgent) have high latency and require costly RL training. ConvMem reformulates long-context reasoning as **hierarchical convolution** — LLM as kernel, text segments as input feature maps.

### Mechanism
- **LLM as convolutional kernel**: Prompt + query = kernel that summarizes text segments hierarchically
- **Logarithmic tree**: Reasoning path shortened from linear chain to log-depth tree
- **Configurable Strides + Skip Connections**: Robust evidence capture and propagation
- **Multi-Kernel Convolution**: Decomposes complex queries into disentangled semantic channels
- **Training-free, massively parallel**: Across both text segments and reasoning threads

### Results
- Outperforms training-free baselines on RULER-HotpotQA and RULER-2WikiMultiHopQA
- Avoids overfitting to parametric priors (common in RL-trained models on OOD tasks)
- Parallelizable across segments and threads

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-MEMORY** | Hierarchical convolution → KB retrieval as multi-scale summarization. Level 1: chunk summaries. Level 2: section summaries. Level 3: document summaries. Log-depth retrieval instead of flat search. |
| **NT-CORE** | Multi-kernel convolution → E8 hexagram multi-perspective reasoning. Different "kernels" (reasoning styles) applied to same data, results merged. Semantic channel decomposition → hexagram line analysis. |
| **NT-MIND** | Logarithmic reasoning tree → SEAL pipeline's stage decomposition. Instead of linear Phase-0→Phase-5, use hierarchical stages with skip connections. Early termination when evidence sufficient. |
| **NT-WORLD** | Configurable strides → crawl pipeline segmentation. Large documents processed in overlapping chunks with hierarchical summarization. |

### Actionable Pattern for NeoTrix
**Hierarchical KB Retrieval**: Implement log-depth retrieval for experience namespace. Level 1: node summaries (100 tokens each). Level 2: hub summaries (500 tokens each). Level 3: full text. Query hits Level 1 first; only drill down when summary is insufficient. Reduces average retrieval cost from O(N) to O(log N) with skip connections for robust evidence propagation.

---

## Cross-Paper Synthesis

### Emergent Pattern: Model-Intrinsic Attention Control (2 papers)

| Paper | Mechanism | Training Required |
|-------|-----------|-------------------|
| Declarative Attention | Model declares `<global>/<focus>/<local>` in CoT | No (zero-shot) |
| HeRo | Router memory conditions on routing history | Lightweight adapters only |

**NeoTrix Implication**: GWT should evolve from externally-computed salience to model-intrinsic attention declaration. The model itself knows what it needs — GWT should interpret declarations, not compute scores from scratch. HeRo adds the memory dimension: declarations should condition on routing history, not be stateless.

### Emergent Pattern: Hierarchical Coarse-to-Fine (3 papers)

| Paper | Coarse | Fine | Error Bound |
|-------|--------|------|-------------|
| CEDAR | Residual KV summaries | Exact token attention | Within-chunk dispersion |
| FFD | Low-bit quantized scanning | Full-precision computation | Distribution-adaptive |
| ConvMem | Segment summaries | Full text | Skip connection propagation |

**NeoTrix Implication**: All three prove that **summary-first, expand-on-demand** is the dominant efficiency pattern. NT-MEMORY should implement this as a first-class retrieval strategy: always load summaries first, expand to full text only when error/dispersion exceeds threshold.

### Emergent Pattern: Fused Computation (2 papers)

| Paper | What's Fused | Benefit |
|-------|-------------|---------|
| FFD | Selector + Computer in one kernel | Eliminates metadata overhead, enables scan reuse |
| ConvMem | Reasoning + Retrieval in hierarchical pass | Reduces latency from linear to logarithmic |

**NeoTrix Implication**: NeoTrix modules currently separate concerns (compute salience → route → execute). Fused computation eliminates intermediate state and enables reuse. GWT should explore fusing salience computation + module routing + execution dispatch into fewer passes.

---

## Priority Actions

| Priority | Action | Domain | Effort |
|----------|--------|--------|--------|
| P0 | Implement summary-first KB retrieval (CEDAR-inspired) | NT-MEMORY | Medium |
| P0 | Add routing memory to GWT (HeRo-inspired) | NT-CORE | Medium |
| P1 | Model-declared attention scopes in GWT (DA-inspired) | NT-CORE | High |
| P1 | Fused GWT salience+routing (FFD-inspired) | NT-CORE | High |
| P2 | Hierarchical log-depth KB retrieval (ConvMem-inspired) | NT-MEMORY | High |
| P2 | Multi-kernel E8 reasoning (ConvMem-inspired) | NT-CORE | High |

---

*Generated by NeoTrix iteration loop, cycle 434*
