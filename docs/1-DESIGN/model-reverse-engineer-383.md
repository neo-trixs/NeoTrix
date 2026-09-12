# Model Reverse Engineering — Cycle 383

**Date:** 2026-09-12
**Focus:** Efficient KV cache, agent RL stability, hybrid attention, Mixture of Experts optimization
**Sources:** arXiv (Apr-Sep 2026), ACL 2026, ICML 2026, EuroSys 2026

---

## 1. GLIDE — Guided Layerwise Hybrid Attention

**Source:** arXiv:2607.24788, June 2026

### Key Claims
- Non-uniform hybrid attention: early layers keep softmax, deeper layers aggressively linearized
- 45×–62× lower KV cache I/O than baseline LLaMA while retaining 92%–96% accuracy
- Exploits layer-wise heterogeneity — early layers sensitive to softmax removal, deep layers tolerant
- Variable-sized softmax window per layer with linear recurrence aggregation

### Architecture Insights
- **Problem**: Uniform hybrid approaches (SWA, LoLCats) apply same policy across all layers → either over-compress early layers (losing accuracy) or under-compress deep layers (wasting efficiency)
- **Solution**: Sensitivity-guided allocation — measure each layer's sensitivity to softmax removal, allocate softmax budget non-uniformly
- **Key mechanism**: Cache sparsity parameter α ∈ [0,1] per layer: α·w tokens for linear attention, (1-α)·w for softmax attention
- **Pareto frontier**: GLIDE configurations occupy the favorable top-left region of performance-efficiency space, outperforming all uniform alternatives

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Layer-level routing in reasoning | GWT salience computation could use GLIDE's sensitivity-guided allocation — cheap layers for routine broadcasts, full attention for novel signals |
| NT-IO | KV cache optimization | GLIDE's non-uniform compression directly reduces inference cost — aligns with Axiom A2 (context as scarce resource) |
| NT-MEMORY | Tiered memory by layer depth | Early layers = precise context (softmax); deep layers = compressed representation (linear). Maps to NT-MEMORY's KB embedding (precise) vs session compression (cheap) |

### Absorption
- **GLIDE validates Flux Attention's layer-level routing** (prev cycle 381): Both papers converge on layer-level routing as superior to head-level. GLIDE goes further by showing *sensitivity-guided* allocation beats uniform allocation.
- **KV cache I/O as first-class bottleneck**: GLIDE shifts focus from "compute" to "memory I/O" — the real bottleneck at scale. NeoTrix's KVMem integration should prioritize I/O reduction, not just compute reduction.
- **Layer-wise heterogeneity as design principle**: Not all layers are equal. NeoTrix's module architecture should apply the same principle — some modules need full precision, others can be compressed.

---

## 2. RAGEN-2 — Reasoning Collapse in Agentic RL

**Source:** arXiv:2604.06268, April 2026 (ICML 2026 Oral)

### Key Claims
- **Template collapse**: RL-trained agents produce reasoning that looks diverse (high entropy) but is input-agnostic (low mutual information). Invisible to entropy monitoring
- Mutual information (MI) between input and reasoning correlates with task performance far more strongly than entropy
- Signal-to-noise ratio (SNR) mechanism explains collapse: low reward variance → weak task gradients → regularization dominates → cross-input differences erased
- SNR-Aware Filtering selects high-signal prompts per iteration using reward variance as lightweight proxy

### Architecture Insights
- **Problem**: Standard RL stability monitoring (entropy + reward) misses template collapse — agents produce fluent but generic reasoning
- **Diagnosis**: Decompose reasoning quality into H(Z|X) (within-input diversity) and I(X;Z) (cross-input distinguishability). Template collapse = high H(Z|X) + low I(X;Z)
- **Mechanism**: When within-input reward variance Var(R|X) is low, task gradients weaken, input-agnostic regularizers (KL, entropy) dominate, and cross-input reasoning differences are flattened
- **Fix**: SNR-Aware Filtering — prioritize high-variance prompts where reward signal is strong, discard low-information rollouts

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Reasoning quality monitoring | E8 hexagram transitions could suffer template collapse — use MI proxy to detect when reasoning becomes input-agnostic |
| NT-MIND | SEAL pipeline stability | SEAL cycles could collapse into generic templates. SNR-Aware Filtering = prioritize high-variance experiences for distillation |
| NT-REPAIR | Self-healing signal detection | Template collapse is a self-healing failure mode — reasoning degrades silently. MI-based detection enables proactive repair |

### Absorption
- **MI as SEAL cycle health metric**: SEAL's experience distillation could use MI to detect when cycles produce generic templates instead of input-specific reasoning. High MI = good (reasoning responds to context), low MI = collapse warning.
- **SNR-Aware Filtering for experience selection**: When absorbing external knowledge (R-P79), prioritize high-signal, high-variance sources. Low-variance sources produce template knowledge.
- **Entropy is necessary but insufficient**: RAGEN-2 proves entropy monitoring alone misses critical failure modes. NeoTrix's health monitoring should add MI-based diagnostics alongside entropy tracking.

---

## 3. HybridKV — Hybrid KV Cache Compression for MLLMs

**Source:** ACL 2026 (2026.acl-long.594)

### Key Claims
- Head classification into static vs dynamic types using text-centric attention
- Top-down budget allocation: hierarchically assigns KV budgets across heads
- Static heads compressed by text-prior pruning; dynamic heads by chunk-wise retrieval
- 7.9× KV cache memory reduction, 1.52× faster decoding, near-zero performance loss on Qwen2.5-VL-7B

### Architecture Insights
- **Problem**: Existing KV compression treats all attention heads uniformly — but heads exhibit heterogeneous behaviors (some static, some dynamic)
- **Solution**: Classify heads first (text-centric attention reveals static vs dynamic behavior), then apply different compression strategies to each type
- **Static heads**: Compress via text-prior pruning (predictable patterns)
- **Dynamic heads**: Compress via chunk-wise retrieval (unpredictable, need on-demand reconstruction)
- **Budget allocation**: Top-down hierarchical — total budget → per-layer → per-head, ensuring critical heads get more budget

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-MEMORY | Head-type-aware memory management | Not all memory is equal — some facts are static (compress aggressively), some are dynamic (keep accessible). HybridKV's classification maps to KB (static) vs session (dynamic) |
| NT-IO | Content-type-aware compression | Claw Compactor's content-type stages + HybridKV's head-type classification = same principle: classify first, compress differently per type |
| NT-CORE | Heterogeneous attention allocation | GWT could classify attention heads by function — some for routine processing (compress), some for novel signals (preserve) |

### Absorption
- **Classify-then-compress as universal pattern**: HybridKV (heads), Claw Compactor (content types), GLIDE (layers) all converge on: classify components first, apply different strategies per type. This should be a NeoTrix architectural principle.
- **Static vs dynamic memory**: The static/dynamic head distinction maps directly to NT-MEMORY's architecture — long-term KB facts (static, compress aggressively) vs session context (dynamic, keep accessible for retrieval).
- **Top-down budget allocation**: Hierarchical budget distribution (total → layer → head) is the right model for NT-MEMORY's memory budget allocation across modules.

---

## 4. AsymCache — Computation-Latency-Aware KV Cache Management

**Source:** arXiv:2606.02964, June 2026 (EuroSys 2026)

### Key Claims
- Multi-Segment Attention (MSA): decomposes attention into non-contiguous segments for fine-grained KV cache control
- Computational-Aware Block Evictor: prioritizes eviction based on each block's marginal contribution to expected attention latency (not just access frequency)
- 1.90–2.03× TTFT reduction, 1.62–1.71× TPOT reduction over latest baselines
- Seamless integration with agent serving systems (18.1% average job latency reduction)

### Architecture Insights
- **Problem**: Existing KV cache eviction uses access frequency or positional heuristics — ignores how different KV blocks affect GPU attention kernel execution latency
- **Solution**: Model the marginal latency contribution of each KV block. Evict blocks that contribute least to kernel performance, not just those accessed least recently
- **MSA key insight**: Attention computation on non-contiguous KV segments is possible and enables fine-grained control over which blocks participate in GPU computation
- **Agent-aware**: AsymCache integrates with agent serving systems, reducing multi-turn agent latency by 18.1%

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-IO | Latency-aware resource management | Evict based on marginal latency impact, not just frequency. NT-IO's resource management should model actual kernel performance, not just cache hit rates |
| NT-MEMORY | Position-aware memory eviction | Position matters — early tokens in long contexts have different latency impact than late tokens. NT-MEMORY's eviction policy should be position-aware |
| NT-ACT | Agent serving optimization | AsymCache's 18.1% agent latency reduction validates agent-specific cache optimization — not just generic LLM serving |

### Absorption
- **Latency-aware eviction > frequency-based eviction**: Access frequency is a proxy for importance. Actual kernel latency impact is the real metric. NeoTrix's cache management should model performance impact, not just access patterns.
- **MSA as attention primitive**: Non-contiguous segment attention enables fine-grained KV cache control. This is the hardware-level implementation of what GLIDE does at the algorithm level — both enable selective attention participation.
- **Agent-specific serving**: Generic LLM serving doesn't account for multi-turn agent patterns. AsymCache's 18.1% agent speedup proves agent-specific optimization is worthwhile.

---

## 5. HookMoE — Single-Layer Compensation for Efficient MoE Inference

**Source:** ACL 2026 (Xueshuo Xie et al.)

### Key Claims
- Plug-and-play single-layer compensation framework for MoE models
- Reduces activated experts by >50% with only 2.5% average performance degradation
- 1.42× inference speedup during prefill stage
- Upper layers require fewer active experts — actionable insight for dynamic expert selection

### Architecture Insights
- **Problem**: MoE top-k routing reduces compute via expert sparsity, but aggressive sparsity degrades performance. Existing approaches require full retraining to recover
- **Solution**: Insert lightweight trainable Hook module before selected transformer blocks. Hook module compensates for information lost by reducing active experts
- **Key insight**: Upper layers require fewer active experts — the model's representations are already mature, less refinement needed
- **Training**: Only small post-training calibration set needed — no full retraining

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Layer-depth-aware computation | Upper reasoning layers need less compute — E8 hexagram transitions at higher abstraction levels could use fewer "experts" (modules) |
| NT-MIND | Lightweight adaptation without retraining | HookMoE's plug-and-play compensation = NeoTrix modules should be improvable at C4 (integrated) without dropping to C0 (recompile) |
| NT-IO | MoE efficiency | Direct optimization for MoE-based models in NeoTrix's inference paths |

### Absorption
- **Layer-depth-compute correlation**: Upper layers need fewer experts. This validates the "sensitivity-guided" approach from GLIDE and Flux Attention — deeper layers are more tolerant of compression/sparsification.
- **Hook modules as adaptation pattern**: Lightweight trainable hooks inserted into frozen models — this is exactly how NeoTrix should handle external technology absorption (R-P79). Don't modify the core, add compensating hooks.
- **Prefill vs decode optimization**: HookMoE optimizes prefill (1.42×), GLIDE optimizes decode (45× KV cache I/O). Different stages need different optimizations — NeoTrix should have stage-specific optimization strategies.

---

## Cross-Paper Synthesis

### Unified Theme: Layer-Wise Heterogeneity

All 5 papers recognize that not all layers/heads/blocks are equal:

| Paper | What's Heterogeneous | Allocation Strategy |
|-------|---------------------|---------------------|
| GLIDE | Layer sensitivity to softmax removal | Sensitivity-guided softmax budget |
| RAGEN-2 | Prompt signal variance | SNR-Aware Filtering |
| HybridKV | Head static vs dynamic behavior | Type-specific compression |
| AsymCache | Block marginal latency contribution | Latency-aware eviction |
| HookMoE | Layer depth and expert requirement | Fewer experts in upper layers |

**NeoTrix implication**: The universal pattern is **classify-then-allocate**. Identify component characteristics first, then allocate resources (compute, memory, attention) proportionally to importance.

### Unified Theme: Lightweight Adaptation Without Retraining

| Paper | Adaptation Method | Training Requirement |
|-------|------------------|---------------------|
| GLIDE | Layer router insertion | 12 hours on 8 GPUs (prev Flux Attention) |
| RAGEN-2 | SNR-Aware Filtering | No model changes — data selection only |
| HybridKV | Head classification + budget | Calibration set only |
| AsymCache | MSA kernel + evictor | No model changes — system-level |
| HookMoE | Hook module insertion | Small calibration set |

**NeoTrix implication**: All 5 papers achieve improvement without full retraining. NeoTrix's C0-C6 constellation maturity model should guarantee that C4 (integrated) modules can be improved via lightweight hooks/calibration without dropping to C0.

### Unified Theme: Signal Quality Over Signal Quantity

| Paper | Signal Quality Metric | What It Replaces |
|-------|----------------------|------------------|
| RAGEN-2 | Mutual Information I(X;Z) | Entropy H(Z) |
| AsymCache | Marginal latency contribution | Access frequency |
| HookMoE | Layer expert requirement | Uniform expert allocation |
| GLIDE | Per-layer sensitivity | Uniform hybridization |
| HybridKV | Head-type classification | Uniform compression |

**NeoTrix implication**: NeoTrix's monitoring and optimization should use **quality-aware metrics** (MI, latency impact, sensitivity) rather than **quantity metrics** (entropy, frequency, count). This is a shift from "how much" to "how much impact."

---

## Recommendation for NeoTrix

**Immediate absorption:**
1. **HybridKV's classify-then-compress** — NT-MEMORY should classify knowledge as static (KB facts, compress aggressively) vs dynamic (session context, keep accessible) and apply different compression strategies
2. **RAGEN-2's MI monitoring** — SEAL pipeline health should use mutual information to detect template collapse, not just entropy. Add MI proxy to ConsciousnessTree health metrics
3. **HookMoE's hook pattern** — External technology absorption (R-P79) should use lightweight hooks inserted into frozen modules, not full retraining

**Research direction:**
4. **GLIDE's sensitivity-guided allocation** — GWT attention routing should evolve from uniform broadcast to sensitivity-guided allocation based on layer/module characteristics
5. **AsymCache's latency-aware eviction** — NT-MEMORY's eviction policy should model actual performance impact, not just access frequency. This requires kernel-level performance modeling.

---

## Cycle Continuity

**From Cycle 381 (prev model RE):**
- Flux Attention → GLIDE validates and extends layer-level routing with sensitivity guidance
- LISA → HybridKV extends two-branch thinking to head-level classification
- DUET → HookMoE validates reasoning-action separation with lightweight compensation
- AttnPO → RAGEN-2 extends attention-as-supervision to agent RL stability
- Reflection Steering → RAGEN-2's MI proxy enables reflection detection in reasoning traces

**Forward to Cycle 384:**
- Watch for: KV cache management systems that combine GLIDE's algorithmic approach with AsymCache's system-level optimization
- Watch for: Agent RL stabilization techniques that combine RAGEN-2's MI monitoring with actual deployment feedback loops
- Research direction: MoE expert selection as a NeoTrix routing problem — can E8 hexagram transitions serve as expert selection signals?
