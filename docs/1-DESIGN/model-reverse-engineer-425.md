# Model Reverse Engineering — Cycle 425

**Date**: 2026-09-12
**Focus**: Recent papers on efficient inference, attention, agent coordination
**Sources**: arXiv (Sep 2026), ACL 2026
**Cross-referenced against cycles 318–424 for novelty**

---

## Paper 1: CEDAR — Error-Bounded Residual Routing for Long-Context Attention

**URL**: https://arxiv.org/abs/2609.07237
**Date**: 2026-09-07

### Core Contribution
Coarse-to-fine method for sparse attention that preserves global coverage while maintaining ~3x kernel speedup at 128K context. Each semantic chunk contributes a cheap key-value summary to a **residual attention path**; chunks with high estimated approximation error are expanded to exact token attention. Exact and summarized contributions combined in single softmax normalization — refinement replaces rather than duplicates coarse evidence.

### Key Insight
> "Hard selection assigns zero probability to every omitted chunk: a routing miss cannot be recovered, and a fixed expansion budget spends the same work on easy and ambiguous queries."

The residual path is the critical innovation — instead of binary include/exclude, every chunk contributes a compressed summary, and only high-error chunks get expanded. This is **probabilistic routing with guaranteed error bounds**.

### Architecture Mapping to NeoTrix

| CEDAR Component | NeoTrix Domain | Pattern |
|-----------------|---------------|---------|
| Residual attention path | NT-CORE (GWT) | GWT salience as residual summary — every module contributes, high-salience gets exact attention |
| Error-bounded refinement | NT-MIND (SEAL) | SEAL convergence check (converge_check) with error bounds — not just pass/fail, but quantified degradation |
| Variable refinement budget | NT-CORE (A1) | Cost-Aware Routing — allocate exact attention only where approximation error exceeds threshold |
| Chunk-level KV summaries | NT-MEMORY (KB) | KB embeddings as compressed summaries, query-time expansion on high-relevance results |

### Absorption Candidates
- **GWT Enhancement**: Add residual path to salience computation. Currently GWT either includes or excludes modules from attention broadcast. CEDAR shows how to include all modules with compressed summaries, expanding only high-error ones. This is a GWT refinement from binary to graduated attention.
- **Convergence Check with Error Bounds**: `converge_check()` currently returns pass/fail. CEDAR's error-bound methodology could quantify how far each branch deviates from health baseline, enabling prioritized self-healing.
- **KB Retrieval Tiering**: Residual summaries map to KB embedding tiers — cheap BM25 summary for all results, exact vector expansion only for high-relevance candidates.

### Implementation Priority: P1
Direct enhancement to GWT salience computation. Low coupling, high impact. The residual path pattern is broadly applicable.

---

## Paper 2: Faster Flash Decoding (FFD) — Hardware-Algorithm Co-Design for Attention Sparsity

**URL**: https://arxiv.org/abs/2609.00097
**Date**: 2026-08-31 (Accepted at ICML 2026)

### Core Contribution
Hardware-algorithm co-design framework that breaks the memory wall in long-context decoding. FFD integrates selector and computer into a **fully fused kernel**, replacing external metadata indices with content-aware scanning via low-bit quantization. Introduces the **top-delta strategy** that dynamically filters blocks to achieve distribution-adaptive sparsity without global synchronization.

### Key Insight
> "To overcome the inherent trade-offs between the memory overhead of metadata-based metrics and the computational inefficiency of adaptive selection strategies."

The top-delta strategy is novel: instead of fixed top-k selection, it adapts to the actual distribution of attention scores at runtime. This is **data-dependent sparsity** that requires no training and is plug-and-play.

### Architecture Mapping to NeoTrix

| FFD Component | NeoTrix Domain | Pattern |
|---------------|---------------|---------|
| Fully fused kernel (selector+computer) | NT-PHYSICAL | Hardware-software co-design for inference efficiency |
| Top-delta dynamic filtering | NT-CORE (GWT) | GWT attention routing with distribution-adaptive thresholds |
| Content-aware scanning | NT-WORLD | Perception layer with content-dependent processing depth |
| Low-bit quantization for metadata | NT-MEMORY | KB index compression — metadata in low-bit, data in full precision |

### Absorption Candidates
- **GWT Distribution-Adaptive Routing**: Current GWT uses static salience thresholds. FFD's top-delta shows how to make thresholds adapt to the actual distribution of attention scores — if most modules are low-salience, the threshold tightens; if many are high, it loosens. This is dynamic attention budget allocation.
- **NT-PHYSICAL Co-Design**: FFD demonstrates that hardware-algorithm co-design yields 11.6x kernel speedup. NeoTrix's physical layer could adopt similar co-design principles for inference optimization.
- **KB Index Optimization**: Low-bit quantization for metadata (module health scores, attention weights) while keeping full precision for actual data. This is a memory optimization pattern for the KB.

### Implementation Priority: P2
Requires kernel-level engineering for GWT. High impact but moderate coupling to hardware.

---

## Paper 3: RouteRelay — Event-Triggered Cross-Layer Route Reuse

**URL**: https://arxiv.org/abs/2609.07306
**Date**: 2026-09-07

### Core Contribution
Router-agnostic method that reuses route metadata across Transformer depth. Anchor layers perform full routing; intermediate layers rescore the previous top-k route and a compact sentinel set of near-miss and randomly probed chunks. A query row is rerouted only when a sentinel challenges its weakest selected chunk.

### Key Insight
> "The sparse attention kernel avoids most token interactions, but the router still rebuilds a chunk-chunk score matrix layer after layer, even when the selected routes change little."

RouteRelay exploits **cross-layer route stability** — in most layers, the routing decisions from the previous layer are still valid. This reduces full routing cost from O(L) per layer to O(L) per anchor layer + O(sentinel) per intermediate layer.

### Architecture Mapping to NeoTrix

| RouteRelay Component | NeoTrix Domain | Pattern |
|---------------------|---------------|---------|
| Anchor vs intermediate layers | NT-CORE (ConsciousnessTree) | CT cycle phases: full evaluation at cycle boundaries, lightweight monitoring between |
| Sentinel-based rerouting | NT-SHIELD | Anomaly detection — only investigate when sentinel detects deviation |
| Cross-layer route reuse | NT-MEMORY | Cross-session knowledge reuse — don't re-derive patterns that are still valid |
| Row-selective GPU execution | NT-PHYSICAL | Selective computation — only process modules where routing has changed |

### Absorption Candidates
- **ConsciousnessTree Phase Optimization**: CT currently runs full 6-stage cycle every iteration. RouteRelay shows how to run full evaluation at anchor points (every N cycles) with lightweight sentinel monitoring between. This reduces CT overhead while maintaining coverage.
- **Cross-Session Route Reuse**: NT-NEXUS currently re-derives patterns each session. RouteRelay's reuse pattern suggests caching routing decisions and only re-routing when sentinel detects drift.
- **Selective Module Health Checks**: Instead of checking all module health every cycle, check full health at anchor cycles and sentinel-based checks between. This is a performance optimization for HeartbeatAggregator.

### Implementation Priority: P1
Direct optimization for ConsciousnessTree cycle frequency. Low coupling, moderate impact.

---

## Paper 4: Bilevel Coordinated Reflection — Game-Theoretic Multi-Agent LLM Systems

**URL**: https://arxiv.org/abs/2609.02750
**Date**: 2026-09-02

### Core Contribution
Models orchestrator-worker interaction as a **bilevel coordination game**: under bounded coupling, workers' local-update game is an approximate potential game whose equilibrium slack is controlled by decomposition quality. Introduces **Stochastic Reflective Memory Ascent (SRMA)** — accepts candidate memory only after grounded evaluation risk strictly decreases.

### Key Insight
> "No gate that observes only the generated transcript can improve uniformly over text-indistinguishable environments, whereas an environment-grounded gate can."

This is an **impossibility result** for transcript-only evaluation and a proof that **environment grounding** is necessary for reliable memory improvement. SRMA provides the mechanism: accept memory only when evaluation risk strictly decreases.

### Architecture Mapping to NeoTrix

| BCR Component | NeoTrix Domain | Pattern |
|---------------|---------------|---------|
| Bilevel coordination game | NT-CORE (ConsciousnessTree) | CT as bilevel game: branches (workers) + core (orchestrator) |
| Equilibrium slack | NT-MIND (SEAL) | SEAL pipeline balance — decomposition quality controls convergence speed |
| SRMA (risk-decreasing acceptance) | NT-MEMORY (KB) | KB write gate — accept experience only when confidence increases |
| Environment-grounded evaluation | NT-WORLD | Perception-grounded validation — not just LLM self-evaluation |

### Absorption Candidates
- **KB Write Gate with Risk Reduction**: Currently KB accepts all experience entries. SRMA shows how to gate writes: only accept when grounded evaluation confidence increases. This is a quality filter for experience accumulation.
- **ConsciousnessTree as Bilevel Game**: CT branches as workers, core as orchestrator. Decomposition quality (how well CT decomposes system health into branch-level assessments) controls the "equilibrium slack" — how far branches can deviate from health baseline.
- **Environment-Grounded Self-Test**: SelfTest currently uses LLM evaluation. BCR proves transcript-only evaluation is insufficient. SelfTest should ground evaluation in actual system state (compilation results, test output, KB state) not just LLM judgment.

### Implementation Priority: P2
Requires KB write gate redesign. High impact, moderate coupling.

---

## Paper 5: NeuralFSM — Finite-State Execution Policy for Multi-Agent Coordination

**URL**: https://aclanthology.org/2026.acl-long.1543.pdf
**Date**: ACL 2026

### Core Contribution
Formulates multi-agent problem solving as a **finite-state execution process**. Learns both state transition distribution and inter-agent communication weights from interaction traces using a Temporal Coordination Controller (TGN-based). Includes dual-defense protection layer: training-time graph regularization + runtime trust-aware message attenuation.

### Key Insight
> "Rather than prioritizing explicit structure generation, the proposed framework uses task context to modulate transition and routing decisions, enabling flexible coordination without manual protocol design."

NeuralFSM doesn't generate fixed workflows — it learns a **reusable FSM** that adapts transitions based on task context. The FSM is the coordination substrate, not the task-specific workflow.

### Architecture Mapping to NeoTrix

| NeuralFSM Component | NeoTrix Domain | Pattern |
|---------------------|---------------|---------|
| Finite-state execution process | NT-CORE (ConsciousnessTree) | CT 6-stage loop as FSM — states = stages, transitions = phase progression |
| Temporal Coordination Controller (TGN) | NT-NEXUS | Cross-session temporal graph for coordination pattern learning |
| Trust-aware message attenuation | NT-SHIELD | Message filtering based on source trust score |
| Graph regularization | NT-MIND (SEAL) | Training-time regularization for skill crystallization quality |

### Absorption Candidates
- **ConsciousnessTree as Learned FSM**: CT currently has fixed 6-stage progression. NeuralFSM shows how to learn transition probabilities from interaction traces — which stages are most often followed by which, under what conditions CT should skip stages, when to repeat stages.
- **Trust-Aware Domain Communication**: NT domains currently communicate without trust scoring. NeuralFSM's trust attenuation suggests weighting inter-domain messages by source reliability — e.g., NT-WORLD perception data weighted higher than NT-MIND speculation.
- **Temporal Coordination Graph**: NT-NEXUS could learn a temporal graph of cross-session coordination patterns — which domain interactions recur, which are one-shot, which have temporal dependencies.

### Implementation Priority: P2
Requires FSM learning infrastructure. High impact, moderate coupling.

---

## Cross-Cutting Patterns (Cycle 425)

| Pattern | Papers | NeoTrix Mapping |
|---------|--------|----------------|
| **Residual/graduated attention** | CEDAR | GWT from binary to graduated attention |
| **Distribution-adaptive sparsity** | FFD, RouteRelay | Dynamic attention thresholds |
| **Environment-grounded evaluation** | BCR | SelfTest grounded in system state |
| **Finite-state coordination** | NeuralFSM | CT as learned FSM |
| **Cross-layer route reuse** | RouteRelay | Cross-session pattern reuse |
| **Risk-gated memory writes** | BCR (SRMA) | KB quality filter |

---

## Absorption Priority Matrix

| # | Paper | Pattern | Target Domain | Impact | Effort |
|---|-------|---------|---------------|--------|--------|
| 1 | **CEDAR** | Residual attention path | NT-CORE (GWT) | 🔴 Very High | Low |
| 2 | **RouteRelay** | Anchor+sentinel optimization | NT-CORE (CT) | 🟠 High | Low |
| 3 | **BCR (SRMA)** | Risk-gated KB writes | NT-MEMORY (KB) | 🔴 Very High | Medium |
| 4 | **NeuralFSM** | Learned FSM coordination | NT-CORE (CT) + NT-NEXUS | 🟠 High | Medium |
| 5 | **FFD (Top-Delta)** | Distribution-adaptive routing | NT-CORE (GWT) | 🟠 High | Medium |

---

## Source Registry

| Paper | arXiv | Date | Venue |
|-------|-------|------|-------|
| CEDAR | 2609.07237 | 2026-09-07 | Preprint |
| Faster Flash Decoding | 2609.00097 | 2026-08-31 | ICML 2026 |
| RouteRelay | 2609.07306 | 2026-09-07 | Preprint |
| Bilevel Coordinated Reflection | 2609.02750 | 2026-09-02 | Preprint |
| NeuralFSM | ACL 2026 Long | 2026 | ACL 2026 |
