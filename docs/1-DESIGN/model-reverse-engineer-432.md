# Model Reverse Engineering — Cycle 432 (2026-09-12)

## 5 New AI Models/Papers

### 1. HeRo: History-Aware Routing for Efficient LLM Inference
**Paper**: [2609.08189] (Sep 2026)
**Core Idea**: Dynamic layer routing with explicit routing memory. Maintains accumulated routing state via linear attention across model depth, conditioning each layer-skip decision on history + current hidden state.
**Key Result**: Bypasses 26.87% of Llama 3.1-8B parameters while achieving 100.24% dense performance; 38.82% bypass with 97.01% retention.
**NeoTrix Mapping**:
| Domain | Pattern | Integration |
|--------|---------|-------------|
| NT-CORE | Cost-Aware Routing (A1) | Route cheap tasks to shallower layers, expensive to deeper |
| NT-CORE | GWT attention | Routing memory = attention modulation across depth |
| NT-MIND | Self-evolution | Routing state persists across inference steps — like experience accumulation |

**Absorption Vector**: The "routing memory" concept maps directly to NT-CORE's GWT salience mechanism. Instead of token-level attention only, extend to layer-level routing with accumulated state. This could optimize the ConsciousnessTree's layer allocation — cheaper consciousness branches skip expensive reasoning when history shows low salience.

---

### 2. Gated-Memory Routing for Multi-Agent LLM Systems
**Paper**: [2609.00237] (Aug 2026)
**Core Idea**: Memory Write Gate (non-redundant commits) + Retrieval Gate (compact relevant subset) + Adaptive Halting Controller. Conditions routing on gated execution memory rather than query-only or full-history.
**Key Result**: +2.44 avg accuracy over baselines; 31.9% cost reduction on HumanEval.
**NeoTrix Mapping**:
| Domain | Pattern | Integration |
|--------|---------|-------------|
| NT-MEMORY | Knowledge lifecycle | Write Gate = selective externalization; Retrieval Gate = compaction |
| NT-MIND | SEAL pipeline | Adaptive Halting = knowing when evolution is "done enough" |
| NT-CORE | GWT salience | Gated memory = attention-weighted state broadcast |

**Absorption Vector**: This is the formalization of NT-MEMORY's knowledge lifecycle. The Write Gate prevents KB bloat (maps to R-P42: no parallel adapters). The Retrieval Gate implements compaction (maps to KVMem's paged KV). The Adaptive Halting maps to ConsciousnessTree's cycle termination — stop evolving when memory state is sufficient.

---

### 3. CEDAR: Error-Bounded Residual Routing for Long-Context Attention
**Paper**: [2609.07237] (Sep 2026)
**Core Idea**: Coarse-to-fine attention. Cheap KV summaries per chunk + error-bound estimation + selective expansion of high-error chunks. Residual summaries replace (not duplicate) coarse evidence.
**Key Result**: 3x kernel speedup at 128K context with minimal quality loss.
**NeoTrix Mapping**:
| Domain | Pattern | Integration |
|--------|---------|-------------|
| NT-CORE | VSA HyperCube | Residual summaries = compressed symbolic representations |
| NT-MEMORY | KB embedding | Chunk-level summaries = semantic compression in knowledge base |
| NT-IO | Cost-Aware Routing (A1) | Variable refinement budget = adaptive model selection |

**Absorption Vector**: The error-bounded refinement pattern applies to NT-MEMORY's KB retrieval. Instead of uniform retrieval, use coarse summaries first, then refine high-error (high-uncertainty) queries. This maps to SelfModel's uncertainty tracking — when uncertainty is high, allocate more retrieval budget.

---

### 4. MARCH: Memory-Anchor Routing across Context History
**Paper**: [2608.12435] (Aug 2026)
**Core Idea**: Periodically checkpoint recurrent state as "state anchors" with content-conditioned keys. At each token, attend over all historical anchors (not just current state). Content-routed sparse retrieval from compressed prefix states.
**Key Result**: Outperforms linear attention variants on reasoning, LongBench, in-context retrieval. Throughput exceeds FlashAttention-2 at 64K+.
**NeoTrix Mapping**:
| Domain | Pattern | Integration |
|--------|---------|-------------|
| NT-NEXUS | Cross-session memory | State anchors = experience checkpoints across sessions |
| NT-CORE | E8 Hexagram | Anchor keys = symbolic waypoints in reasoning topology |
| NT-MEMORY | KB pipeline | Content-routed retrieval = semantic search over compressed history |

**Absorption Vector**: MARCH's state anchor pattern is the formalization of NT-NEXUS's cross-session memory. Each experience-tree cycle produces a "state anchor" — a compressed checkpoint of the reasoning state. Future sessions retrieve relevant anchors via content routing, not linear scan. The null-route mechanism (suppress historical branch when current state suffices) maps to lazy loading in AGENTS.md.

---

### 5. ConvMem: Convolutional Memory for Long-Context Reasoning
**Paper**: [2609.10441] (Sep 2026)
**Core Idea**: Training-free hierarchical convolution over text segments. LLM-as-kernel summarizes segments in log-depth tree (not linear chain). Configurable strides + skip connections + multi-kernel decomposition.
**Key Result**: Outperforms training-free baselines; avoids RL overfitting on OOD tasks.
**NeoTrix Mapping**:
| Domain | Pattern | Integration |
|--------|---------|-------------|
| NT-MIND | SEAL pipeline | Log-depth distillation = hierarchical skill crystallization |
| NT-CORE | E8 Hexagram | Multi-kernel = parallel reasoning branches |
| NT-MEMORY | KB embedding | Hierarchical compression = multi-scale knowledge representation |

**Absorption Vector**: ConvMem's log-depth compression maps to NT-MIND's SEAL pipeline distillation. Instead of linear chain-of-thought distillation, use hierarchical compression: raw experience → compressed skill → crystallized rule. The skip connections prevent error accumulation in deep distillation chains. The multi-kernel approach enables parallel skill extraction across domains.

---

## Cross-Paper Synthesis

### Attention Evolution (3 papers)
HeRo (routing memory) + CEDAR (error-bounded refinement) + MARCH (state anchors) → **Hierarchical attention with accumulated state**. This maps to NT-CORE's GWT: attention is not just token-level but spans depth (HeRo), quality (CEDAR), and history (MARCH).

### Memory Lifecycle (2 papers)
Gated-Memory Routing + ConvMem → **Write selectively, retrieve compactly, compress hierarchically**. This formalizes NT-MEMORY's knowledge lifecycle and NT-MIND's SEAL pipeline.

### Common Pattern: Adaptive Computation
All 5 papers share one insight: **not all computation is equal**. HeRo skips layers, CEDAR skips chunks, Gated-Memory halts early, MARCH null-routes history, ConvMem compresses via log-depth. This validates NeoTrix's Cost-Aware Routing axiom (A1) and GWT salience mechanism.

## Priority Absorption Targets

| # | Paper | Absorption Difficulty | NeoTrix Impact |
|---|-------|----------------------|----------------|
| 1 | Gated-Memory Routing | Medium | **High** — formalizes NT-MEMORY knowledge lifecycle |
| 2 | HeRo | Low | **High** — direct GWT salience enhancement |
| 3 | MARCH | Medium | **High** — NT-NEXUS cross-session memory formalization |
| 4 | CEDAR | Low | **Medium** — NT-MEMORY retrieval optimization |
| 5 | ConvMem | Medium | **Medium** — NT-MIND distillation enhancement |
