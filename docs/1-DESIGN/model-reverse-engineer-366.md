# Model Reverse Engineering — Cycle 366 (2026-09-12)

## 5 New AI Models/Papers (Sep 2026)

---

### 1. FFD — Faster Than Flash: Hardware-Algorithm Co-Design for Sparse Decoding
**Paper**: "Faster Than Flash: Exploiting Attention Sparsity for Efficient Long-Context Decoding" (arXiv:2609.00097)
**Date**: Aug 31 2026 | **Accepted**: ICML 2026

#### Core Mechanism
Hardware-algorithm co-design that fuses selector + computer into a single kernel:
1. Replaces external metadata indices with **content-aware scanning** via low-bit quantization
2. **Top-delta strategy**: dynamically filters blocks for distribution-adaptive sparsity
3. No global synchronization required — fully fused kernel
4. Reuses scanning results for computation (scan → compute overlap)
5. Training-free, plug-and-play

#### Key Results
- **11.6× kernel-level speedup** (end-to-end: 2.37× throughput)
- Scales to **256K context length**
- Maintains accuracy on RULER and LongBench benchmarks
- Works with existing models (no retraining)

#### Why It Matters
FFD breaks the memory wall in long-context decoding by eliminating the two-phase scan-then-compute bottleneck. The top-delta strategy is the key innovation: instead of a fixed sparsity ratio, it dynamically selects the sparsity level per attention head based on the actual score distribution. This means easy attention heads get high sparsity (fast), hard heads get low sparsity (accurate).

#### NeoTrix Domain Mapping

| Domain | Mapping | Specific Component |
|--------|---------|-------------------|
| **NT-CORE** | Top-delta = adaptive reasoning depth. Easy queries skip deep reasoning; hard queries get full E8 exploration. | `nt_core_e8::adaptive_depth` — distribution-aware reasoning depth |
| **NT-MEMORY** | Fused scan-compute = KB embedding search + retrieval in single pass. No separate index-then-query. | `kb::fused_search` — single-pass embedding search + retrieval |
| **NT-IO** | Content-aware scanning = provider-side token filtering before context injection. | `nt_io::provider` — pre-filter irrelevant tokens at provider level |
| **NT-MIND** | Distribution-adaptive = SEAL pipeline allocates effort by task difficulty, not fixed budget. | `seal::adaptive_budget` — difficulty-aware evolution resource allocation |

#### Absorption Candidate
- **Pattern**: Distribution-adaptive sparsity (top-delta filtering)
- **Application**: GWT attention routing could use score-distribution analysis to allocate processing depth per query
- **Priority**: P1 — directly applicable to GWT attention optimization

---

### 2. EFQ-Softmax — Exp-Free Quantization for Attention
**Paper**: "EFQ-Softmax: Exp-Free Quantization for Softmax" (arXiv:2609.09721)
**Date**: Sep 9 2026

#### Core Mechanism
Eliminates the exp-then-quantize bottleneck in low-bit attention:
1. **Direct mapping** from shifted attention scores to E2M1 (4-bit) operands
2. Per-block exponent-only scale selection from local maximum
3. Single affine rule generates nonnegative probability codes
4. FlashAttention row-maximum update preserved unchanged
5. Compatible with MXFP4/FP8 microscaling

#### Key Results
- **Qwen3-8B**: mean improvement from 0.6749 → 0.6773 (+0.24 pp) on 7 tasks
- **Qwen3-VL-8B**: mean from 0.7826 → 0.8000 (+1.74 pp) on 9 tasks
- **40.33% latency reduction** on A5 vector unit for probability generation
- Maintains quality on WAN2.2 video generation (temporal consistency)

#### Why It Matters
The exp-then-quantize path is the hidden tax on quantized attention: you compute high-precision exponentials, then immediately downcast. EFQ-Softmax eliminates this by generating low-bit probability codes directly. The insight is that softmax probabilities have a specific structure (nonnegative, row-sum-1) that can be exploited for direct quantization without the exp intermediate step.

#### NeoTrix Domain Mapping

| Domain | Mapping | Specific Component |
|--------|---------|-------------------|
| **NT-CORE** | Direct score→action mapping. E8 hexagram transitions could skip intermediate representation. | `nt_core_e8::direct_transition` — score-to-action without intermediate |
| **NT-MEMORY** | Direct probability generation = KB relevance scoring without embedding→cosine→threshold pipeline. | `kb::direct_relevance` — skip embedding similarity step |
| **NT-IO** | Low-bit probability at provider level = faster tool-call argument validation. | `nt_io::arg_validator` — fast-path tool call validation |
| **NT-MIND** | Efficiency without quality loss = SEAL pipeline optimization without regression risk. | `seal::lossless_optimize` — efficiency gains without quality tradeoff |

#### Absorption Candidate
- **Pattern**: Direct low-bit generation (skip intermediate representation)
- **Application**: GWT salience scoring could bypass cosine similarity step for fast-path attention routing
- **Priority**: P2 — applicable but requires specific hardware context

---

### 3. FastE — Readout-Triggered Token Compression for Embeddings
**Paper**: "FastE: Readout-Triggered Token Compression for LLM Embedding Inference" (arXiv:2609.08407)
**Date**: Sep 8 2026

#### Core Mechanism
Exploits depth-dependent prefix redundancy in embedding models:
1. **Readout-prefix alignment** as online heuristic for compression timing
2. Prefix states become increasingly compressible at deeper layers
3. **Attention-score ranking** determines which prefix states to retain
4. Shared fixed threshold — no per-sample calibration
5. Training-free, plug-and-play

#### Key Results
- **40.11% FLOPs reduction** on NarrativeQA (Qwen3-Embedding-0.6B) with 99.53% quality retention
- Works across 5 text embedding benchmarks, 2 backbone scales, 3 cross-modal retrieval tasks
- Quality-efficiency tradeoff directly customizable via max removal ratio
- No retraining required

#### Why It Matters
Embedding models waste computation reading prefix tokens that become irrelevant at deeper layers. FastE identifies when prefix states can be safely dropped based on their alignment with the readout position. The key insight: **compression depth is predictable from shallow-layer statistics** — you don't need to compute deep layers to know what can be dropped.

#### NeoTrix Domain Mapping

| Domain | Mapping | Specific Component |
|--------|---------|-------------------|
| **NT-MEMORY** | Readout-triggered compression = KB experience nodes summarized at retrieval depth. Shallow summaries for fast path, deep detail on demand. | `kb::depth_aware_compression` — layer-aware experience node summarization |
| **NT-CORE** | Alignment heuristic = ConsciousnessTree cycle depth awareness. Early cycles (shallow) retain full detail; late cycles (deep) compress. | `consciousness_tree::adaptive_compression` — cycle-depth-aware experience compression |
| **NT-MIND** | Prefix redundancy removal = SEAL pipeline stage pruning. Early stages retain full detail; later stages drop redundant information. | `seal::stage_pruner` — prune redundant information at later evolution stages |
| **NT-WORLD** | Content extraction with depth-dependent detail. Shallow extraction = overview; deep extraction = specifics. | `nt_world_crawl::adaptive_extraction` — depth-aware content extraction |

#### Absorption Candidate
- **Pattern**: Readout-triggered compression (predictable depth-dependent redundancy)
- **Application**: Experience-tree branch loading could compress at retrieval depth, not storage depth
- **Priority**: P1 — directly applicable to KB experience compression

---

### 4. AdaptOrch — Task-Adaptive Multi-Agent Orchestration
**Paper**: "AdaptOrch: Task-Adaptive Multi-Agent Orchestration in the Era of LLM Performance Convergence" (arXiv:2602.16873)
**Date**: Feb 18 2026 (updated Aug 2026)

#### Core Mechanism
Formal framework for topology-aware multi-agent orchestration:
1. **Performance Convergence Scaling Law**: As models converge, orchestration topology dominates model selection
2. **Topology Routing Algorithm**: O(|V|+|E|) DAG analysis → parallel/sequential/hierarchical/hybrid
3. **Adaptive Synthesis Protocol**: provable termination, consistency scoring for parallel outputs
4. Four canonical topologies selected by task dependency structure
5. Threshold-based routing: parallelism width (ω), coupling density (γ), critical path depth (δ)

#### Key Results
- **12–23% improvement** over static baselines (SWE-bench, GPQA, RAG tasks)
- Routing decision runs in **O(|V|+|E|)** — linear in task complexity
- 94% of tasks converge in ≤2 synthesis iterations
- Uses identical underlying models (orchestration-only improvement)

#### Why It Matters
The paper's central thesis: "when LLM capabilities converge, orchestration topology becomes the dominant lever." The scaling law proves this formally — variance from topology exceeds variance from model by Ω(1/ε²). This reframes the optimization problem from "which model?" to "which orchestration?" The DAG analysis is elegant: parallelism width predicts parallel execution, coupling density predicts hierarchical, critical path depth predicts sequential.

#### NeoTrix Domain Mapping

| Domain | Mapping | Specific Component |
|--------|---------|-------------------|
| **NT-ACT** | Topology routing = task decomposition → execution strategy selection. DAG analysis determines parallel/sequential/hierarchical execution. | `nt_act::topology_router` — task DAG → execution topology |
| **NT-CORE** | ConsciousnessTree phases as DAG. Soil→Root→Trunk→Branch→Fruit topology varies by task complexity. | `consciousness_tree::adaptive_topology` — task-dependent phase execution |
| **NT-MIND** | SEAL pipeline topology = parallel/sequential evolution stages. | `seal::adaptive_topology` — task-dependent evolution strategy |
| **NT-SHIELD** | Topology-aware policy enforcement. Hierarchical tasks need stricter controls; parallel tasks need coordination policies. | `nt_shield::topology_policy` — execution-strategy-dependent security policies |

#### Absorption Candidate
- **Pattern**: DAG-structure-based topology routing (O(|V|+|E|))
- **Application**: SEAL pipeline could analyze task dependency DAGs to select parallel/sequential/hybrid evolution strategies
- **Priority**: P1 — directly applicable to SEAL pipeline orchestration

---

### 5. Orchestra-o1 — Omnimodal Agent Orchestration
**Paper**: "Orchestra-o1: Omnimodal Agent Orchestration" (arXiv:2606.13707)
**Date**: Jun 10 2026

#### Core Mechanism
Multi-modal multi-agent orchestration framework:
1. **Modality-aware task decomposition** — tasks split by modality (text/image/audio/video)
2. **Online sub-agent specialization** — agents specialize dynamically based on subtask modality
3. **Parallel sub-task execution** — different modalities processed concurrently
4. **Decision-Aligned GRPO (DA-GRPO)** — agentic RL training for orchestration policy
5. Unified orchestration mechanism across all modalities

#### Key Results
- **10.3% accuracy improvement** over second-best on OmniGAIA benchmark
- Orchestra-o1-8B achieves SOTA among open-source omnimodal agents
- DA-GRPO enables efficient RL training without massive compute
- Generalizes across text, image, audio, video tasks

#### Why It Matters
Most multi-agent orchestration assumes text-only agents. Orchestra-o1 extends the paradigm to heterogeneous modalities where different agents handle different sensory channels. The key insight: modality boundaries provide natural task decomposition axes. Text agents reason, image agents perceive, audio agents listen — and the orchestrator coordinates across these natural boundaries.

#### NeoTrix Domain Mapping

| Domain | Mapping | Specific Component |
|--------|---------|-------------------|
| **NT-ACT** | Modality-aware agent specialization = NT-ACT tool selection by input modality. | `nt_act::modality_router` — route tasks by input modality to specialized agents |
| **NT-WORLD** | Multi-modal perception = NT-WORLD multi-sensor integration. Text + image + audio + video processing. | `nt_world::multimodal_perception` — modality-aware content extraction |
| **NT-IO** | Modality-aware interface = NT-IO multi-modal output (text/image/audio). | `nt_io::multimodal_output` — output by modality |
| **NT-MIND** | DA-GRPO = SEAL pipeline training with modality-specific rewards. | `seal::modality_reward` — modality-aware evolution training |

#### Absorption Candidate
- **Pattern**: Modality-aware orchestration with natural decomposition boundaries
- **Application**: NT-WORLD perception pipeline could use modality boundaries as natural task decomposition axes for multi-sensor fusion
- **Priority**: P2 — applicable to multi-modal perception, less urgent for text-focused systems

---

## Cross-Paper Synthesis

### Dominant Theme: Adaptive Resource Allocation
Papers 1-3 (FFD, EFQ-Softmax, FastE) all optimize resource allocation based on distribution statistics — sparsity adapts to score distribution, quantization adapts to block structure, compression adapts to depth. The common principle: **don't allocate uniformly; allocate based on measured difficulty**.

### Secondary Theme: Topology as First-Class Optimization
Papers 4-5 (AdaptOrch, Orchestra-o1) both treat orchestration topology as a primary optimization target. AdaptOrch proves topology dominates model selection under convergence; Orchestra-o1 extends topology to multi-modal domains.

### Absorption Priority Matrix

| Pattern | Source Paper | NeoTrix Target | Priority | Complexity |
|---------|-------------|----------------|----------|------------|
| Distribution-adaptive sparsity | FFD (2609.00097) | GWT adaptive_depth | P1 | Medium |
| DAG-structure topology routing | AdaptOrch (2602.16873) | SEAL adaptive_topology | P1 | High |
| Readout-triggered compression | FastE (2609.08407) | KB depth_aware_compression | P1 | Low |
| Direct low-bit generation | EFQ-Softmax (2609.09721) | GWT direct_transition | P2 | Medium |
| Modality-aware orchestration | Orchestra-o1 (2606.13707) | NT-WORLD modality_router | P2 | High |

---

## Cross-Cycle Synthesis (365 → 366)

| Dimension | Cycle 365 | Cycle 366 | Evolution |
|-----------|-----------|-----------|-----------|
| **Attention** | Self-declared routing (DA), error-bounded (CEDAR) | Distribution-adaptive (FFD), direct quantization (EFQ) | From programmer-controlled → model-adaptive |
| **Memory** | Hierarchical convolution (ConvMem), query amortization (PIVOT) | Readout-triggered compression (FastE) | From query-side optimization → storage-side optimization |
| **Orchestration** | Temporal workflow graphs (ReActNet) | DAG topology routing (AdaptOrch), multimodal (Orchestra-o1) | From static compilation → dynamic adaptation |
| **Hardware** | Software-level optimization | Hardware-algorithm co-design (FFD) | Moving optimization closer to silicon |
