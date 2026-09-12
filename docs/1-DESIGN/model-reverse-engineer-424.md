# Model Reverse Engineering — Cycle 424 (2026-09-12)

## Selection Criteria
- Recent papers (September 2026) on efficient inference, attention, agent coordination
- Novel patterns that map to NeoTrix 7 domains
- Must address fundamental bottlenecks: memory, attention, routing, self-evolution

---

## Paper 1: Declarative Attention (DA) — Self-Declared Sparse Attention

### Metadata
| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2609.02737](https://arxiv.org/abs/2609.02737) |
| **Published** | 2026-09-02 |
| **Task** | Long-context inference efficiency |

### Core Mechanism
Language models declare where they need to attend within their chain-of-thought, partitioning generation into three modes:
- `<global>` — full context scan
- `<focus>` — specific region (reduced KV read)
- `<local>` — recent output only (minimal KV read)

The inference engine parses these declarations like tool calls and skips most KV cache reads.

### Key Results
| Model | Tokens Reduced | Accuracy Drop |
|-------|---------------|---------------|
| Gemma-4-31B | 52.0% | 1.27pp |
| Qwen-3.6-27B | 31.1% | 2.75pp |

Shrinks with model scale — larger models make better declarations.

### Novel Pattern
**Intrinsic attention control** — the model itself decides which context regions matter, rather than external proxy scores. This is fundamentally different from extrinsic sparse attention (routing via lightweight scorer models) because it's zero-shot, requires no training, and the model's own reasoning state guides the selection.

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE** | GWT salience modulation — the model self-declares attention focus, mapping to GWT's broadcast/receive pattern. `<focus>` mode = salient channel activation, `<local>` mode = peripheral suppression. |
| **NT-MEMORY** | KV cache skip optimization — directly extends KVMem paged KV virtualization. DA declarations could guide page eviction/retention policies. |
| **NT-MIND** | Meta-cognitive self-regulation — the ability to declare attention scope is a form of introspective control, aligning with ConsciousnessTree's self-monitoring. |

### Absorption Pattern
**Axiom contribution:** Validates A2 (Context as Scarce Resource) — context window is the bottleneck, and the model itself can help manage it. DA provides a concrete mechanism for GWT attention routing: instead of computing salience externally, let the reasoning model declare its focus regions.

**Implementation sketch:**
```
GWT Enhancement:
1. During chain-of-thought, model emits <global>/<focus>/<local> tags
2. GWT router reads these declarations to modulate broadcast scope
3. <focus> regions get priority in attention channel allocation
4. <local> regions skip broadcast entirely (peripheral suppression)
```

---

## Paper 2: Codebook Agent — Amortized Topology Design

### Metadata
| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2609.02264](https://arxiv.org/abs/2609.02264) |
| **Published** | 2026-09-02 |
| **Task** | Multi-agent topology optimization |

### Core Mechanism
Three key empirical findings:
1. Topologies collapse to ~6 distinct graphs even at 64-entry codebook capacity
2. Edge count negatively correlates with token cost (Pearson r ≈ -0.4)
3. Message-passing scorer is adjacency-invariant when agents share profiles

Solution: Vector-quantized autoencoder compresses successful topologies into 16-entry query-independent codebook. Reward-weighted MLP maps query embedding to code distribution. MLP proxy reranks candidates in single batched forward pass.

### Key Results
| Metric | Codebook Agent | Best Prior |
|--------|---------------|------------|
| Avg accuracy (6 benchmarks) | 84.6 | 83.0 |
| Topology generation time | 2.4ms | ~seconds (search-based) |
| Token savings | 21.9-33.2% | baseline |

### Novel Pattern
**Topology compression as vocabulary** — treating agent communication graphs like a learned vocabulary. The codebook is a "lexicon" of effective interaction patterns. This amortizes the cost of topology search: instead of solving a combinatorial problem per query, you select from a pre-learned vocabulary.

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE** | E8 hexagram state compression — the 16-entry codebook maps directly to E8's 64-element hexagonal grid. Each codebook entry = a hexagram state encoding a communication topology. |
| **NT-ACT** | Capability registry routing — topology codes become routing tables for capability dispatch. |
| **NT-MIND** | Pattern crystallization — successful topologies compress into reusable "skill crystals" in the SEAL pipeline. |

### Absorption Pattern
**Axiom contribution:** New axiom candidate — **Topology Compression** (A4): effective agent communication patterns collapse to a small vocabulary. Implication: E8 hexagram grid doesn't need to represent all possible states, only the ~6-16 that actually emerge in practice.

**Implementation sketch:**
```
E8 Enhancement:
1. Maintain topology codebook (16 entries) as hexagram vocabulary
2. Query embedding → code distribution → topology selection
3. Successful topologies auto-promote into codebook via VQ-VAE
4. Failed topologies get negative reward, eventually pruned
5. Codebook size self-adjusts: grows when novel patterns emerge, compresses when patterns stabilize
```

---

## Paper 3: BeaconKV — KV Cache Compression for Reasoning Models

### Metadata
| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2609.04971](https://arxiv.org/abs/2609.04971) |
| **Published** | 2026-09-04 |
| **Task** | KV cache compression for long reasoning traces |

### Core Mechanism
Discovers **Thought Revisiting Tokens (TRT)** — decoding steps that re-attend to distant previous context (e.g., task plans from early in the trace). TRT queries cluster into similarity groups in embedding space. BeaconKV maintains compact beacon queries (representatives for each cluster) to anticipate which KV pairs will be revisited without storing entire query history.

### Key Results
| Metric | Value |
|--------|-------|
| Memory reduction | Up to 5.8× |
| Accuracy | Nearly preserves full cache |
| Throughput improvement | Over 4.3× |

### Novel Pattern
**Anticipatory cache management** — instead of reactive compression (evict what seems unimportant), predict what will be revisited and keep those KV pairs. The beacon queries act as "probes" that forecast future attention patterns based on past revisitation clusters.

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-MEMORY** | Memory retention policy — beacon queries as "importance predictors" for KB entries. Repeatedly revisited knowledge gets promoted in cache. |
| **NT-CORE** | GWT attention prediction — beacon clusters predict which knowledge channels will be activated in future reasoning cycles. |
| **NT-MIND** | Experience prioritization — TRT detection identifies which past experiences are worth crystallizing into reusable skills. |

### Absorption Pattern
**Axiom contribution:** Extends A2 (Context as Scarce Resource) with **anticipatory retention** — don't just compress what's old, predict what will be needed again. The beacon query pattern maps to a new mechanism for ConsciousnessTree: predicting which modules will need attention in the next growth cycle.

**Implementation sketch:**
```
KVMem Enhancement:
1. Track Thought Revisiting Token patterns per conversation
2. Cluster TRT queries into similarity groups
3. Maintain beacon query per cluster (compact representatives)
4. At each decode step, score KV pairs against beacon queries
5. Keep high-scoring pairs resident, compress low-scoring ones
6. Beacon clusters evolve as conversation progresses
```

---

## Paper 4: PARSER — Parallel Read, Deep Reason for Long-Context Agents

### Metadata
| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2609.06702](https://arxiv.org/abs/2609.06702) |
| **Published** | 2026-09-06 |
| **Task** | Long-context multi-hop QA |

### Core Mechanism
Decouples reading from reasoning. A bank of lightweight subagents (frozen off-the-shelf models) each bound to a single chunk read the entire document in parallel. A lead agent reasons in depth through iterative scatter-gather rounds: broadcasts query to all subagents, aggregates evidence, formulates deeper follow-up query. Lead agent optimized via RL; subagents remain frozen.

### Key Results
| Metric | PARSER (4B) | Best Sequential Baseline |
|--------|------------|------------------------|
| Multi-hop QA avg | +5.7pp | baseline |
| At 896K tokens | +12.0pp | baseline |
| PARSER (9B) vs DeepSeek-V4-Pro | +6.3pp | — |
| Inference latency | Up to 11× reduction | — |

Robust to evidence position, order, and distance perturbations.

### Novel Pattern
**Read-reason separation with RL-optimized lead** — frozen parallel readers handle I/O, RL-trained lead handles reasoning. This is a natural hierarchy: cheap bulk reading + expensive deep reasoning. The lead agent's RL training optimizes the scatter-gather query strategy, not the reading itself.

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-WORLD** | Parallel content ingestion — subagent bank as UnifiedCrawler parallel fetchers |
| **NT-CORE** | GWT lead agent optimization — RL-trained lead = GWT attention router, scatter-gather = broadcast pattern |
| **NT-ACT** | Capability execution hierarchy — frozen executors + trained orchestrator |
| **NT-MEMORY** | Chunked knowledge assembly — parallel readers build distributed memory fragments, lead assembles coherent context |

### Absorption Pattern
**Architecture contribution:** Validates the 3-layer architecture (L5 Consciousness + L1 Capability Network). PARSER proves that separating "reading infrastructure" (capability layer) from "reasoning intelligence" (consciousness layer) yields both efficiency and quality gains. The RL-optimized lead maps directly to GWT's attention router learning optimal broadcast strategies.

**Implementation sketch:**
```
GWT + World Enhancement:
1. Parallel subagent readers handle document chunking (L1 capability)
2. GWT lead agent broadcasts queries, collects evidence (L5 consciousness)
3. Lead uses RL to optimize query formulation across scatter-gather rounds
4. Evidence assembly builds structured context in NT-MEMORY
5. Lead's learned policy becomes reusable skill crystal (NT-MIND)
```

---

## Paper 5: Bilevel Coordinated Reflection — Game-Theoretic Multi-Agent Memory

### Metadata
| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2609.02750](https://arxiv.org/abs/2609.02750) |
| **Published** | 2026-09-02 |
| **Task** | Multi-agent LLM coordination and memory improvement |

### Core Mechanism
Models orchestrator-worker interaction as a **bilevel coordination game**:
- Workers' local-update game is an approximate potential game
- Equilibrium slack controlled by decomposition quality
- Reflection modeled as stochastic movement over semantic memory states

Introduces **Stochastic Reflective Memory Ascent (SRMA)**: accepts candidate memory only after grounded evaluation shows risk strictly decreases. Converges exactly, geometrically or polynomially.

### Key Results
| Metric | Value |
|--------|-------|
| SWE-bench (500 instances) | 72.2% (vs 70.8% reference) |
| Convergence guarantee | Exact under calibration + non-degenerate corrective mass |
| Impossibility result | No transcript-only gate can improve uniformly; environment-grounded gate required |

### Novel Pattern
**Environment-grounded memory validation** — a gate that only observes the generated transcript cannot distinguish good from bad memories in text-indistinguishable environments. You need external grounding (test results, user feedback, world state) to validate memory quality. This is an information-theoretic impossibility result.

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-MEMORY** | Memory quality gating — SRMA as KB experience validation policy |
| **NT-CORE** | Bilevel coordination — orchestrator-worker game maps to ConsciousnessTree ↔ domain module interaction |
| **NT-MIND** | Reflection as stochastic process — SEAL pipeline reflection modeled as Markov chain over memory states |
| **NT-SHIELD** | Grounded validation — environment signals (build status, test results) as security audit ground truth |

### Absorption Pattern
**Axiom contribution:** New axiom candidate — **Grounded Memory Validation** (A5): memory quality gates must observe environment state, not just generated text. Implication: KB experience entries require external validation signals (build pass, test pass, user confirmation) before promotion to crystallized skills.

**Implementation sketch:**
```
SEAL Pipeline Enhancement:
1. Candidate experience generated (reflection phase)
2. SRMA gate evaluates: does this memory decrease risk?
3. Risk assessment uses environment signals:
   - Build status (cargo check/test)
   - Test results (unit/integration)
   - User feedback (explicit confirmation)
4. Only grounded-validated memories get crystallized
5. Transcript-only memories stay in staging, never promoted
6. Stochastic convergence ensures eventual quality improvement
```

---

## Cross-Paper Synthesis

### Common Patterns Across All 5 Papers

| Pattern | Papers | NeoTrix Integration |
|---------|--------|-------------------|
| **Self-declared attention** | DA, Codebook, PARSER | GWT learns to route by observation, not just computation |
| **Amortized optimization** | Codebook, SMC, BeaconKV | Pre-compute patterns, select at runtime (E8 hexagram vocabulary) |
| **Hierarchical execution** | PARSER, SMC, BCR | Frozen cheap workers + RL-trained expensive orchestrator |
| **Grounded validation** | BCR, Harden AIF | Memory/decisions require environment signals, not just text |
| **Budget-aware allocation** | DA, BeaconKV, MetaKV | Resources (tokens, memory, compute) allocated per-query |

### NeoTrix Architecture Implications

1. **GWT Enhancement:** Add self-declared attention mode — reasoning model outputs focus declarations that modulate broadcast scope
2. **E8 Codebook:** Topology compression vocabulary (16-entry) for hexagram state selection
3. **SEAL Pipeline:** SRMA-style grounded validation for experience crystallization
4. **KVMem:** Beacon query anticipation for cache retention policy
5. **NT-ACT:** Speculative macro commit for tool calling latency reduction

### Priority Absorption Order

| Priority | Paper | Pattern | Effort |
|----------|-------|---------|--------|
| P0 | Declarative Attention | Self-declared attention routing | Medium |
| P0 | Codebook Agent | Topology compression vocabulary | Low |
| P1 | BeaconKV | Anticipatory cache management | Medium |
| P1 | PARSER | Parallel read + RL lead | High |
| P2 | BCR/SRMA | Grounded memory validation | Medium |
