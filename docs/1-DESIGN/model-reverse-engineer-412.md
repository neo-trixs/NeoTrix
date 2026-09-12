# Model Reverse Engineering — Cycle 412

**Date**: 2026-09-12
**Period**: Feb-Sep 2026
**Focus**: Efficient inference, attention mechanisms, agent coordination, KV cache optimization

---

## Paper 1: Sketch&Walk Sparse Attention — Training-Free 6x Inference Speedup

**Source**: arXiv:2602.07397 (Feb 2026)
**Authors**: Hoang Anh Duy Le et al. (Rice University, Stevens Institute)
**Benchmarks**: Wide range of models and tasks

### Core Innovation
Training-free sparse attention using Hadamard sketching + deterministic walk:
1. **Hadamard Sketching**: Lightweight approximation of attention scores via randomized transforms — O(n log n) instead of O(n²)
2. **Walk Mechanism**: Aggregates attention influence across layers via deterministic walk — captures indirect token relationships beyond direct attention
3. **Dynamic Sparsity**: Top-k attention block selection at runtime — 20% attention density with near-lossless accuracy
4. **Custom Kernels**: Fused sparse attention kernels for both prefill and decode phases

### Key Results
- Near-lossless accuracy at 20% attention density
- Up to 6x inference speedup
- Slightly outperforms dense attention in some settings (regularization effect)
- Uniformly applies to both prefill and decode phases

### NeoTrix Domain Mapping

| Component | NeoTrix Domain | Implementation |
|-----------|---------------|----------------|
| Hadamard sketching | NT-CORE | HyperCube sketching — lightweight approximate attention via structured transforms |
| Walk mechanism | NT-CORE | GWT indirect influence propagation — attention influence beyond direct token pairs |
| Dynamic sparsity | NT-IO | Cost-aware attention routing (Axiom A1) — allocate compute where it matters |
| Custom kernels | NT-PHYSICAL | Hardware-aware attention execution |

### Absorbed Pattern
`sketch-walk-attention` — Two-phase sparse attention: (1) fast Hadamard sketch to identify candidate tokens, (2) walk-based aggregation to capture indirect influence. Key insight: attention influence is transitive — token A attends to B, B attends to C, so A should attend to C even if direct score is low. Maps to GWT's resonance-based routing where salient signals propagate through module chains.

---

## Paper 2: PackInfer — Compute- and I/O-Efficient Batched Attention

**Source**: arXiv:2602.06072 (Feb 2026)
**Authors**: Rui Ning, Wei Zhang, Fan Lai
**Benchmarks**: Real-world heterogeneous workloads

### Core Innovation
Kernel-level attention framework for heterogeneous batched inference:
1. **Load-Balanced Execution Groups**: Orchestrates batched requests into balanced groups — eliminates stragglers from heterogeneous sequence lengths
2. **Packed Query-Key Regions**: Attention kernels directly over packed regions — eliminates redundant computation
3. **I/O-Aware Grouping**: Co-locates shared-prefix requests, reorganizes KV caches into group-contiguous layouts
4. **Memory Fragmentation Reduction**: Group-contiguous KV cache layout reduces redundant data movement

### Key Results
- 13-20% latency reduction vs FlashAttention
- 20% throughput improvement
- Eliminates stragglers from heterogeneous sequence lengths
- Reduces memory fragmentation in KV cache

### NeoTrix Domain Mapping

| Component | NeoTrix Domain | Implementation |
|-----------|---------------|----------------|
| Load-balanced groups | NT-ACT | Work scheduling with heterogeneous request awareness |
| Packed query-key regions | NT-CORE | Fused attention execution — minimize redundant computation |
| I/O-aware grouping | NT-MEMORY | KV cache locality optimization — co-locate related entries |
| Memory fragmentation reduction | NT-PHYSICAL | Efficient memory layout for agent serving |

### Absorbed Pattern
`batch-aware-attention` — Treat batched heterogeneous requests as first-class optimization target, not just individual request optimization. Key insight: in production agent serving, request heterogeneity (different sequence lengths, shared prefixes) creates systemic inefficiencies that per-request optimization misses. Maps to NT-ACT's work scheduling with cache locality awareness.

---

## Paper 3: Flux Attention — Context-Aware Hybrid Attention

**Source**: arXiv:2604.07394 (Apr 2026)
**Authors**: Quantong Qiu et al.
**Benchmarks**: Long-context and mathematical reasoning benchmarks

### Core Innovation
Layer-level dynamic routing between Full Attention (FA) and Sparse Attention (SA):
1. **Lightweight Layer Router**: Trained router (12 hours on 8×A800) that decides FA vs SA per layer based on input context
2. **Layer-Wise Routing**: Preserves high-fidelity information retrieval while ensuring contiguous memory access
3. **Context-Adaptive**: Different layers use different attention mechanisms based on retrieval demands
4. **Parameter-Efficient**: Only router parameters trained — frozen pretrained LLM weights

### Key Results
- 2.8x prefill speedup, 2.0x decode speedup
- Superior performance-speed tradeoff vs baselines
- Mathematical reasoning preserved (critical for agent tool-use)
- Only 12 hours training on 8×A800

### NeoTrix Domain Mapping

| Component | NeoTrix Domain | Implementation |
|-----------|---------------|----------------|
| Layer Router | NT-CORE | GWT layer-level attention routing — decide full vs sparse per layer |
| Context-adaptive routing | NT-CORE | Dynamic salience-based attention allocation |
| Frozen LLM + thin router | NT-MIND | Skill overlay pattern — thin adaptation layer over frozen base model |
| Memory access optimization | NT-PHYSICAL | Hardware-aware attention execution |

### Absorbed Pattern
`flux-layer-routing` — Train a thin router that dynamically selects attention mechanism per layer. Key insight: not all layers need the same attention density — early layers may need full attention for token understanding, later layers can be sparse for efficiency. Maps to GWT's attention modulation where different system components receive different attention bandwidth based on task demands.

---

## Paper 4: HeteroPanacea — Disaggregated Prefill-Decode-Attention-FFN Serving

**Source**: arXiv:2608.03741 (Aug 2026)
**Authors**: Przemyslaw Forys et al. (Cambridge, Imperial)
**Benchmarks**: Agentic inference workloads

### Core Innovation
Cross-stack simulation framework for heterogeneous agentic serving:
1. **PDAF Disaggregation**: Prefill-Decode-Attention-FFN separated onto different hardware (GPUs + Groq LPUs)
2. **Disaggregated Quantization**: Different precision for different serving stages
3. **Automated Parallelization**: Intra- and inter-device parallelization scheduling
4. **4-Way NPU Specialization**: Custom NPUs optimized for specific attention/FFN operations

### Key Results
- 75% throughput improvement with Prefill-Decode disaggregation
- 4-way PDAF disaggregation most consistent across models
- Agentic inference (multi-turn, tool-calling) requires fundamentally different hardware than single-turn

### NeoTrix Domain Mapping

| Component | NeoTrix Domain | Implementation |
|-----------|---------------|----------------|
| PDAF disaggregation | NT-PHYSICAL | Hardware heterogeneity awareness — different stages on different hardware |
| Disaggregated quantization | NT-IO | Cost-aware model routing (Axiom A1) — different precision for different tasks |
| Automated parallelization | NT-ACT | Work scheduling with hardware topology awareness |
| Agentic workload characterization | NT-CORE | GWT workload-aware attention — agentic vs single-turn routing |

### Absorbed Pattern
`pdaf-disaggregation` — Agentic inference fundamentally differs from single-turn: prefill and decode have different compute/memory profiles, attention and FFN have different parallelization needs. Key insight: homogeneous GPU systems waste resources on agentic workloads. Maps to NT-PHYSICAL's sensor-motor architecture where different cognitive stages map to different physical resources.

---

## Paper 5: SIRI — Self-Internalizing RL with Intrinsic Skills

**Source**: arXiv:2606.02355 (Jun 2026)
**Authors**: Zhongyu He et al. (Alibaba)
**Benchmarks**: ALFWorld, WebShop

### Core Innovation
Three-phase skill internalization framework:
1. **Phase 1 — Warmup**: GiGPO policy acquires basic interaction ability, collects skill-free trajectories
2. **Phase 2 — Self-Skill Mining**: Policy summarizes compact skills from own successful rollouts, validates via paired skill-augmented vs skill-free rollouts
3. **Phase 3 — Skill Distillation**: Distill only beneficial skill-guided action tokens into plain policy using trajectory-level utility and action-level advantage

### Key Results
- 0.908 → 0.930 on ALFWorld, 0.728 → 0.813 on WebShop
- Outperforms prompt-based, RL-based, and memory-augmented baselines
- Self-mining achieves performance comparable to distillation with closed-source large model
- At inference: agent runs with original prompt only (no skill bank needed)

### NeoTrix Domain Mapping

| Component | NeoTrix Domain | Implementation |
|-----------|---------------|----------------|
| Warmup + trajectory collection | NT-MIND | SEAL Phase-1 (Soil) — baseline capability establishment |
| Self-skill mining | NT-MIND | SEAL Phase-3 (Branches) — skill extraction from successful experiences |
| Paired validation | NT-REPAIR | Skill validation — compare skill-augmented vs skill-free performance |
| Skill distillation into policy | NT-MIND | SEAL Phase-5 (Fruits) — crystallize beneficial patterns into base capability |
| No skill bank at inference | NT-CORE | Internalized skills — no external retrieval needed, pure forward pass |

### Absorbed Pattern
`siri-skill-internalization` — Three-phase cycle: (1) establish baseline, (2) mine skills from own successes with paired validation, (3) distill beneficial skills into base policy. Key insight: skills should be internalized into model weights, not stored externally — eliminates retrieval overhead at inference. Maps to NT-MIND's SEAL pipeline where experience is crystallized into persistent capability. The paired validation (skill-augmented vs skill-free) is a form of self-play quality assurance.

---

## Cross-Paper Synthesis

| Theme | Papers | Key Insight |
|-------|--------|-------------|
| **Sparse Attention** | Sketch&Walk, Flux | Two strategies: (1) training-free approximate sparsity, (2) learned layer-level routing |
| **Batch Efficiency** | PackInfer, HeteroPanacea | Production serving needs batch-aware and hardware-heterogeneous optimization |
| **Skill Internalization** | SIRI | External skill banks add latency; internalize into weights for inference efficiency |
| **Agentic Workloads** | HeteroPanacea, PackInfer | Agentic multi-turn fundamentally differs from single-turn — needs different architectures |

---

## Absorption Priority

| Priority | Pattern | NeoTrix Target |
|----------|---------|----------------|
| P0 | `siri-skill-internalization` | NT-MIND SEAL pipeline — three-phase skill mining + distillation |
| P0 | `flux-layer-routing` | NT-CORE GWT — layer-level attention mechanism routing |
| P1 | `sketch-walk-attention` | NT-CORE HyperCube — approximate attention with transitive influence |
| P1 | `pdaf-disaggregation` | NT-PHYSICAL — hardware-heterogeneous agent serving |
| P2 | `batch-aware-attention` | NT-ACT — heterogeneous request scheduling |
