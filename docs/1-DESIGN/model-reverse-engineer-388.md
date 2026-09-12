# Model Reverse Engineering — Cycle 388 (2026-09-12)

## Cycle Metadata
- **Cycle**: 388
- **Date**: 2026-09-12
- **Focus**: Efficient inference, attention mechanisms, agent coordination, memory routing

---

## 5 Papers Selected

### Paper 1: Flux Attention — Context-Aware Hybrid Attention
- **arXiv**: 2604.07394 (Apr 2026)
- **Authors**: Quantong Qiu et al.
- **Venue**: Preprint (ML + CL)

**Core Mechanism**:
A lightweight Layer Router is inserted into frozen pretrained LLMs. At inference time, each layer is adaptively routed to either Full Attention (FA) or Sparse Attention (SA) based on input context. Unlike head-level dynamic sparsity (which causes load imbalance), layer-level routing preserves contiguous memory access for hardware acceleration.

**Key Results**:
- 2.8x prefill speedup, 2.0x decode speedup
- Only 12 hours training on 8×A800 GPUs
- Superior performance on long-context and math reasoning benchmarks

**Reverse-Engineered Pattern**:
```
Input Context → Layer Router (lightweight MLP) → Per-Layer Decision (FA/SA)
                                                    ↓
                                        Contiguous memory access
                                                    ↓
                                        Hardware-aligned acceleration
```

**NeoTrix Domain Mapping**:

| Domain | Pattern | Implementation |
|--------|---------|---------------|
| **NT-CORE** | Layer-wise routing ↔ GWT salience per attention head | ConsciousnessTree branch routing uses layer-level granularity, not head-level |
| **NT-MIND** | Parameter-efficient adaptation (12h training) ↔ SEAL lightweight evolution | SEAL distillation can use similar lightweight routers for mode switching |
| **NT-PHYSICAL** | Hardware-aware memory access ↔ physical embodiment constraints | kv_cache_optimizer.rs should enforce contiguous memory blocks |

**Absorption Verdict**: **Strong signal**. Layer-level routing is the missing piece between GWT (attention routing) and kv_cache_optimizer (KV cache management). Absorb as `nt_core_gwt::layer_router`.

---

### Paper 2: GLIDE — Guided Layerwise Hybrid Attention
- **arXiv**: 2607.24788 (Jun 2026)
- **Authors**: Vimal William, Ravi Tandon, Jyotikrishna Dass
- **Venue**: Preprint (AI + CL + ML)

**Core Mechanism**:
Exploits layer-wise heterogeneity: early layers are sensitive to softmax removal (need full attention), deeper layers tolerate aggressive replacement by linear recurrence. Non-uniformly compresses softmax footprint across model depth.

**Key Insight**:
Early layers = high sensitivity to attention changes (preserve softmax). Deep layers = redundant (replace with linear recurrence). This is the opposite of uniform hybrid approaches.

**Reverse-Engineered Pattern**:
```
Layer 0-N (early):   Sliding-window softmax (high fidelity)
Layer N-M (middle):  Hybrid softmax + linear recurrence
Layer M-L (deep):    Pure linear recurrence (aggressive compression)
```

**NeoTrix Domain Mapping**:

| Domain | Pattern | Implementation |
|--------|---------|---------------|
| **NT-CORE** | Layer-depth sensitivity profiling ↔ E8 hexagram depth analysis | Different E8 hexagram branches operate at different "depths" of reasoning |
| **NT-MEMORY** | Non-uniform compression ↔ KB tiered storage | Hot/warm/cold KB tiers with different retention policies |
| **NT-REPAIR** | Graceful degradation in deep layers ↔ self-healing priority | Deep components can be replaced with lightweight alternatives under load |

**Absorption Verdict**: **Medium signal**. The depth-sensitivity insight is valuable but overlaps with Flux Attention. Absorb the heterogeneity principle as a refinement to `layer_router`.

---

### Paper 3: Gated-Memory Routing for Multi-Agent LLM Systems
- **arXiv**: 2609.00237 (Aug 2026, EMNLP 2026)
- **Authors**: Rakibul Hasan Rajib et al.
- **Venue**: EMNLP 2026 Main Conference

**Core Mechanism**:
Three learned components:
1. **Memory Write Gate**: Commits only non-redundant reasoning steps (filters noise)
2. **Retrieval Gate**: Supplies each agent a compact, relevant subset (not full history)
3. **Adaptive Halting Controller**: Stops execution once memory contains sufficient evidence

**Key Results**:
- +2.44 accuracy over strongest baseline
- -31.9% inference cost on HumanEval
- Prevents execution-history overload

**Reverse-Engineered Pattern**:
```
Step t: Agent generates reasoning step
         ↓
    Memory Write Gate (MLP) → [commit / discard]
         ↓
    Memory Store (compact, non-redundant)
         ↓
Step t+1: Agent needs context
         ↓
    Retrieval Gate (MLP) → [compact relevant subset]
         ↓
    Adaptive Halting Controller → [continue / stop]
```

**NeoTrix Domain Mapping**:

| Domain | Pattern | Implementation |
|--------|---------|---------------|
| **NT-MEMORY** | Write Gate ↔ KB admission control | `neotrix-kb` write path should filter redundant entries via learned gate |
| **NT-MEMORY** | Retrieval Gate ↔ KB query optimization | BM25 + vector hybrid retrieval with learned compact subset selection |
| **NT-CORE** | Adaptive Halting ↔ GWT attention budget | ConsciousnessTree stops evolution cycle when phi threshold reached |
| **NT-MIND** | Non-redundancy filtering ↔ SEAL distillation | Experience tree distillation step filters redundant experiences before KB write |

**Absorption Verdict**: **Strong signal**. This paper directly addresses the "experience accumulation without redundancy" problem. The Write Gate + Retrieval Gate pattern maps 1:1 to NT-MEMORY admission and query paths. Absorb as `nt_memory::gated_routing`.

---

### Paper 4: DecentMem — Decentralized Self-Evolving Memory
- **arXiv**: 2605.22721 (May 2026)
- **Authors**: Guangya Hao, Yunbo Long, Zhuokai Zhao
- **Venue**: Preprint (Multi-Agent Systems)

**Core Mechanism**:
Each agent maintains private dual-pool memory:
- **Exploitation Pool (E-pool)**: Consolidated successful trajectories
- **Exploration Pool (X-pool)**: LLM-generated candidates for unseen contexts

Online routing between pools via stochastic bandit. LLM-as-judge provides stage-wise feedback for pool reweighting.

**Key Results**:
- +23.8% over centralized memory baselines
- -49% token usage
- O(log T) cumulative regret (matches bandit lower bound)
- Works across AutoGen, DyLAN, AgentNet frameworks

**Reverse-Engineered Pattern**:
```
Agent i:
  E-pool: [trajectory_1, trajectory_2, ...] (consolidated, high-confidence)
  X-pool: [candidate_1, candidate_2, ...]   (generated, exploratory)
         ↓
  Online Router (bandit) → [select from E or X]
         ↓
  LLM-as-Judge → [stage-wise feedback → reweight pools]
```

**NeoTrix Domain Mapping**:

| Domain | Pattern | Implementation |
|--------|---------|---------------|
| **NT-MEMORY** | Dual-pool memory ↔ KB experience namespace | experience-tree E-pool = consolidated cycles, X-pool = pending explorations |
| **NT-CORE** | Bandit routing ↔ GWT cost-aware attention | AttentionManager routes between exploitation (known patterns) and exploration (new patterns) |
| **NT-MIND** | LLM-as-judge feedback ↔ SEAL self-test | ConsciousnessTree uses judge feedback to reweight skill branches |
| **NT-SHIELD** | Agent-private memory ↔ per-domain KB isolation | Each NT-* domain has private KB namespace (already implemented) |

**Absorption Verdict**: **Critical signal**. Validates NeoTrix's per-domain memory architecture. The dual-pool (exploit/explore) pattern should be formalized in the experience-tree protocol: consolidated cycles = E-pool, pending absorptions = X-pool.

---

### Paper 5: SparDA — Sparse Decoupled Attention
- **arXiv**: 2606.04511 (Jun 2026)
- **Authors**: Yaosheng Fu et al. (Song Han group)
- **Venue**: Preprint (CL + ML)

**Core Mechanism**:
Introduces a fourth per-layer projection called **Forecast** (alongside Q, K, V). Forecast predicts which KV blocks the next layer needs, enabling lookahead selection that overlaps CPU-to-GPU prefetch with current-layer execution.

**Key Innovation**:
Forecast is decoupled from the attention query — it's a pure prediction head. GQA uses one Forecast head per group (not per head), reducing selection overhead.

**Key Results**:
- <0.5% parameter overhead
- 1.25x prefill speedup, 1.7x decode speedup over sparse-attention offload baseline
- 5.3x higher decode throughput via larger feasible batch sizes

**Reverse-Engineered Pattern**:
```
Layer i:
  Q, K, V: Standard attention projections
  Forecast: Predicts KV blocks needed by Layer i+1
         ↓
  Lookahead prefetch: CPU→GPU transfer overlaps with Layer i execution
         ↓
  Layer i+1: KV blocks already on GPU (zero wait)
```

**NeoTrix Domain Mapping**:

| Domain | Pattern | Implementation |
|--------|---------|---------------|
| **NT-CORE** | Forecast projection ↔ GWT anticipatory routing | GWT predicts next salient module before current processing completes |
| **NT-PHYSICAL** | Prefetch overlap ↔ physical sensor prediction | Physical embodiment anticipates next sensor read during motor execution |
| **NT-MEMORY** | KV block prediction ↔ KB prefetch | Query planner predicts which KB shards will be needed and prefetches |
| **NT-MIND** | Decoupled prediction head ↔ SEAL lookahead | SEAL pipeline predicts next evolution stage requirements |

**Absorption Verdict**: **High signal**. The Forecast projection pattern is elegant and generalizes beyond attention. Absorb as `nt_core_gwt::forecast_router` for anticipatory attention routing.

---

## Cross-Paper Pattern Synthesis

### Convergent Patterns (3+ papers agree)

| Pattern | Papers | NeoTrix Component |
|---------|--------|-------------------|
| **Layer-wise granularity** (not head-level) | Flux, GLIDE, SparDA | GWT attention routing |
| **Non-redundancy filtering** | Gated-Memory, DecentMem, GLIDE | KB admission control, SEAL distillation |
| **Anticipatory/predictive routing** | SparDA, Gated-Memory, Flux | Forecast router, adaptive halting |
| **Training-free/lightweight adaptation** | CoSA, Flux, SparDA | Zero-overhead embodiment |

### Novel Patterns (unique to this cycle)

| Pattern | Source | NeoTrix Opportunity |
|---------|--------|---------------------|
| **Dual-pool exploit/explore** | DecentMem | Formalize experience-tree E/X pools |
| **Forecast projection** | SparDA | Anticipatory GWT routing |
| **Adaptive halting controller** | Gated-Memory | ConsciousnessTree cycle termination |
| **Kernel-Aware Proxy** | CoSA | Proxy-kernel co-design for KB queries |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| Flux: layer routing needs training vs CoSA: training-free | Use CoSA's proxy-kernel for cold start, Flux's router after warmup |
| DecentMem: fully decentralized vs NeoTrix: KB hub | NeoTrix uses decentralized namespaces with centralized KB hub — hybrid validated |
| Gated-Memory: learned gates vs NeoTrix: heuristic gates | Start with heuristic gates (R-P1 zero unsafe), graduate to learned gates in NT-MIND |

---

## Priority Absorption Queue

| Priority | Paper | Component | Domain | Effort |
|----------|-------|-----------|--------|--------|
| P0 | Gated-Memory | `nt_memory::gated_routing` | NT-MEMORY | Medium |
| P0 | DecentMem | E/X pool formalization | NT-MEMORY + NT-MIND | Low |
| P1 | SparDA | `nt_core_gwt::forecast_router` | NT-CORE | Medium |
| P1 | Flux | `nt_core_gwt::layer_router` | NT-CORE | Medium |
| P2 | GLIDE | Depth-sensitivity refinement | NT-CORE | Low |

---

## References

1. Flux Attention — arXiv:2604.07394 (Qiu et al., Apr 2026)
2. GLIDE — arXiv:2607.24788 (William et al., Jun 2026)
3. Gated-Memory Routing — arXiv:2609.00237 (Rajib et al., EMNLP 2026)
4. DecentMem — arXiv:2605.22721 (Hao et al., May 2026)
5. SparDA — arXiv:2606.04511 (Fu et al., Jun 2026)
