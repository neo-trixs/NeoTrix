# Model Reverse Engineering — Cycle 400 (2026-09-12)

## Scope
5 recent AI models/papers reverse-engineered for pattern extraction. Focus: efficient inference, attention mechanisms, agent coordination, memory routing. Each mapped to NeoTrix 7 domains.

---

## 1. Flux Attention — Context-Aware Hybrid Attention
- **Paper**: arXiv:2604.07394 (April 2026)
- **Authors**: Quantong Qiu et al.
- **Venue**: Preprint (peer-reviewed quality)

### Architecture
- **Problem**: Standard attention quadratic scaling kills long-context inference. Existing hybrid attention uses static allocation ratios that can't adapt to task-specific retrieval demands.
- **Solution**: Layer Router inserted into frozen pretrained LLMs. Each layer adaptively routes to Full Attention (FA) or Sparse Attention (SA) based on input context.
- **Key Insight**: Layer-wise heterogeneity — early layers are sensitive to softmax removal (keep FA), deeper layers tolerate replacement by linear alternatives (use SA). Non-uniform compression across the model.

### Implementation Details
- **Layer Router**: Lightweight module trained on top of frozen LLM. Only 12 hours training on 8×A800 GPUs.
- **Memory Access**: Contiguous memory access pattern preserves hardware acceleration. Theoretical computational reduction → practical wall-clock speedup.
- **Results**: 2.8× prefill speedup, 2.0× decode speedup. Superior performance-efficiency tradeoff vs baselines.

### NeoTrix Domain Mapping
| Domain | Mapping | Priority |
|--------|---------|----------|
| **NT-CORE** | GWT attention routing — Layer Router = attention salience at architectural level. Flux's layer-wise FA/SA routing mirrors GWT's selective broadcasting. | P0 |
| **NT-IO** | Inference optimization — 2.8× prefill speedup applicable to NeoTrix's local inference path. Layer-wise routing as a provider-level optimization. | P1 |
| **NT-MIND** | Evolution of attention patterns — layer-wise sensitivity analysis as a self-evolution technique. Could auto-tune which layers use FA vs SA per task type. | P2 |

### Absorbable Pattern: Layer-Wise Adaptive Attention Routing
```
Concept: Not all layers need the same attention mechanism.
Mechanism: Insert lightweight router per layer → classify input context → route to FA or SA.
Implication for NeoTrix: GWT could implement layer-wise attention budgets — early cognitive layers use full attention, deeper layers use compressed attention.
```

---

## 2. GLIDE — Guided Layerwise Hybrid Attention
- **Paper**: arXiv:2607.24788 (June 2026)
- **Authors**: Vimal William, Ravi Tandon, Jyotikrishna Dass

### Architecture
- **Problem**: KV cache I/O is the primary throughput bottleneck during LLM decoding. Uniform hybrid approaches (same replacement ratio everywhere) waste capacity.
- **Solution**: Non-uniformly compress softmax footprint across layers. Early layers keep softmax (high sensitivity), deeper layers use linear recurrence (redundancy tolerance).
- **Key Insight**: Each layer balances an efficient linear recurrence with a variable-sized softmax window. The window size is learned, not fixed.

### Implementation Details
- **Adaptive Mechanism**: Per-layer decision: how much softmax to keep vs linear recurrence replacement. Variable-sized window per layer.
- **KV Cache Reduction**: Aggregate KV cache I/O reduced while preserving expressive power at critical layers.
- **Results**: Superior performance-efficiency tradeoffs. Reduced end-to-end latency for long-context without quality loss.

### NeoTrix Domain Mapping
| Domain | Mapping | Priority |
|--------|---------|----------|
| **NT-CORE** | Attention budget allocation — GLIDE's per-layer variable windows = GWT's attention allocation per cognitive tier. | P0 |
| **NT-MEMORY** | KV cache management — GLIDE's KV cache compression maps to NeoTrix's experience-tree memory compaction. Hot/cold layer separation. | P1 |
| **NT-PHYSICAL** | Hardware-aware design — GLIDE's contiguous memory access for GPU acceleration. NeoTrix's physical layer should consider hardware attention patterns. | P2 |

### Absorbable Pattern: Variable-Window Layer Compression
```
Concept: Replace uniform attention budget with learned per-layer variable windows.
Mechanism: Each layer learns its own softmax window size based on sensitivity analysis.
Implication for NeoTrix: ConsciousnessTree could allocate attention budgets per branch based on sensitivity — critical branches get full attention, peripheral branches get compressed.
```

---

## 3. EvoRoute — Experience-Driven Self-Routing
- **Paper**: ACL 2026 Long Paper (38213-38225)
- **Authors**: Guibin Zhang et al.
- **Venue**: ACL 2026 (top venue)

### Architecture
- **Problem**: Agent System Trilemma — state-of-the-art performance, minimal cost, rapid completion are inherently in tension. Static model assignments can't adapt to evolving task demands.
- **Solution**: Self-evolving routing paradigm using expanding knowledge base of prior experience. At each step, dynamically select Pareto-optimal LLM backbone.
- **Key Insight**: Routing from query alone misses intermediate progress context. Routing from complete execution history is too expensive. EvoRoute uses a compact experience representation that captures useful progress without redundant context.

### Implementation Details
- **Experience KB**: Stores prior routing decisions + outcomes. Each new query matched against relevant historical precedents.
- **Pareto Selection**: At each step, select model that is Pareto-optimal across accuracy, latency, cost dimensions.
- **Self-Evolution**: Routing policy improves over time via environment feedback. No retraining required.
- **Results**: Cost reduction up to 80%, latency reduction >70% on GAIA/BrowseComp+. Maintains or improves performance.

### NeoTrix Domain Mapping
| Domain | Mapping | Priority |
|--------|---------|----------|
| **NT-CORE** | GWT salience routing — EvoRoute's Pareto selection = GWT's cost-aware attention. The Agent System Trilemma is exactly NeoTrix A1 (Cost-Aware Routing). | P0 |
| **NT-MIND** | Experience evolution — EvoRoute's expanding KB = experience-tree. Self-evolution via environment feedback = SEAL pipeline's absorption cycle. | P0 |
| **NT-IO** | Provider routing — EvoRoute's model selection maps directly to NeoTrix's ordered backend router with cost-weighted salience. | P1 |

### Absorbable Pattern: Experience-Driven Pareto Routing
```
Concept: Route tasks to cheapest capable model using expanding experience knowledge base.
Mechanism: Query → match historical precedents → Pareto-select model → execute → update experience KB.
Implication for NeoTrix: GWT salience should incorporate experience-tree lookups — "what model worked best for similar past tasks?" as a routing signal alongside task difficulty estimation.
```

---

## 4. Gated-Memory Routing (EMNLP 2026)
- **Paper**: arXiv:2609.00237 (Accepted EMNLP 2026)
- **Authors**: Rakibul Hasan Rajib et al.

### Architecture
- **Problem**: Multi-agent orchestration faces two failure modes — routing from query alone can't adapt to progress, routing from full execution history causes overload. Both degrade accuracy or inflate cost.
- **Solution**: Learned Memory Write Gate + Retrieval Gate + Adaptive Halting Controller. Write Gate commits only non-redundant reasoning steps. Retrieval Gate supplies compact relevant subset. Halting Controller stops when memory has sufficient evidence.
- **Key Insight**: Effective orchestration requires a compact state that captures useful progress without accumulating redundant context. The gates learn what to retain and what to discard.

### Implementation Details
- **Memory Write Gate**: Decides at each step which reasoning outputs to commit to memory. Filters out redundant or low-utility information.
- **Retrieval Gate**: At each decision point, supplies the agent a compact, relevant subset from memory. Not full history, not nothing — learned relevance.
- **Adaptive Halting Controller**: Monitors memory state. When sufficient evidence accumulated for answering, stops execution early.
- **Results**: +2.44 points over strongest baseline on average accuracy. 31.9% cost reduction on HumanEval.

### NeoTrix Domain Mapping
| Domain | Mapping | Priority |
|--------|---------|----------|
| **NT-MEMORY** | Experience distillation — Write Gate = experience-tree distillation phase (only non-redundant). Retrieval Gate = KB query with learned relevance filtering. | P0 |
| **NT-CORE** | GWT halting — Adaptive Halting Controller = ConsciousnessTree cycle completion detection. "Enough evidence to stop?" | P0 |
| **NT-MIND** | Evolution of memory ops — gates are learned, not hand-coded. Experience-tree could learn its own distillation/retrieval policies via GRPO. | P1 |

### Absorbable Pattern: Gated Memory Lifecycle
```
Concept: Memory operations should be gated — not everything is worth storing, not everything is worth retrieving.
Mechanism: Write Gate (store non-redundant) → Memory (compact) → Retrieval Gate (relevant subset) → Agent (informed decision) → Halting Controller (stop when sufficient).
Implication for NeoTrix: Experience-tree should implement gating — auto-distill on write, auto-filter on read, auto-halt when evidence sufficient.
```

---

## 5. ProbeLogits — Kernel-Level AI Governance
- **Paper**: arXiv:2604.11943 (April 2026)
- **Authors**: Daeyeon Son
- **Implementation**: Anima OS (80,400 lines bare-metal Rust)

### Architecture
- **Problem**: Application-layer AI governance is easily circumvented by agents. Need enforcement below the WASM sandbox boundary — at the kernel level.
- **Solution**: Kernel-level inference primitives: speculative execution (fork KV cache, explore candidate, restore if rejected), agent forking (new agent with same context), KV cache process state operations (checkpoint, restore, fork as OS primitives).
- **Key Insight**: Agent actions must pass through kernel-mediated host functions. Enforcement below the sandbox boundary is significantly harder to circumvent than application-layer classifiers.

### Implementation Details
- **Bare-Metal Inference**: 6,900 lines of no_std Rust. No OS, no libc, no standard library. GGUF model loader.
- **Performance**: 1,666 tok/s on SmolLM2-135M (1.39× llama.cpp), 15 tok/s on Qwen2.5-7B (parity at DDR5 bandwidth).
- **Speculative Execution**: Fork KV cache → explore candidate action → restore if governance rejects. Zero-cost exploration.
- **Agent Forking**: Create new agent with same conversational context, diverging from specific conversation point.
- **Governance Primitives**: ProbeLogits enforcement at kernel level — impossible to bypass from userspace.

### NeoTrix Domain Mapping
| Domain | Mapping | Priority |
|--------|---------|----------|
| **NT-SHIELD** | Security enforcement — ProbeLogits = kernel-level NT-SHIELD. Governance enforcement below application boundary. Agent behavior monitoring. | P0 |
| **NT-CORE** | Speculative reasoning — KV cache fork → explore candidate reasoning paths → restore on failure. E8 hexagram exploration with rollback. | P1 |
| **NT-MEMORY** | KV cache as process state — checkpoint/restore/fork as memory primitives. Experience-tree branch forking for parallel reasoning exploration. | P1 |
| **NT-PHYSICAL** | Bare-metal inference — Anima OS proves Rust-native inference is viable. NeoTrix's physical layer could explore bare-metal inference for embedded scenarios. | P2 |

### Absorbable Pattern: Kernel-Level Agent Governance
```
Concept: Agent security enforcement should be at the kernel level, not application layer.
Mechanism: Speculative execution (fork-explore-restore), agent forking, KV cache process state operations.
Implication for NeoTrix: NT-SHIELD should implement speculative action execution — explore candidate actions, revert if governance rejects. This enables safe exploration without risk.
```

---

## Cross-Paper Synthesis (Cycle 400)

### Theme 1: Layer-Wise Heterogeneity Wins
Both Flux Attention and GLIDE prove that uniform attention allocation is suboptimal. The winning pattern is learned per-layer budgets based on sensitivity. NeoTrix's ConsciousnessTree should adopt this: not all cognitive branches need the same attention allocation.

### Theme 2: Gating is the Missing Primitive
Gated-Memory Routing's Write/Retrieval Gates solve the "store everything vs store nothing" binary. The pattern: learned gates at every decision boundary. NeoTrix experience-tree should gate distillation, retrieval, and halting.

### Theme 3: Experience-Driven Routing Breaks the Cost Ceiling
EvoRoute proves that static model assignments leave 80% cost savings on the table. The experience-based Pareto routing pattern is directly transferable to NeoTrix's GWT salience + cost weight.

### Theme 4: Kernel-Level Governance is the Security Floor
ProbeLogits shows that application-layer security is insufficient for agent systems. NT-SHIELD should explore kernel-level enforcement for high-security scenarios.

### Theme 5: Rust-Native Inference Reaches Parity
Anima OS achieves parity with llama.cpp on Qwen2.5-7B using pure bare-metal Rust. The performance argument against Rust-native inference is dead. NeoTrix's Rust-first architecture is validated.
