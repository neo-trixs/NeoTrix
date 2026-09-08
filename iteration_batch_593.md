# Iteration Batch 593 — Scientific Computing, Monte Carlo & GPU Compute

## Sources Consulted

| # | Source | Date | Domain |
|---|--------|------|--------|
| 1 | Tu et al., "FP64 Tensor Cores for High-Order FEM at Extreme Scale" (ISC 2026) | 2026-06 | Scientific computing |
| 2 | MARUT: Exascale-Ready GPU-Accelerated CFD Framework (Julia) | 2026 | Scientific computing |
| 3 | TopSim: Plugin-Based Parallel Simulation Framework (Springer) | 2026-08 | Scientific computing |
| 4 | Trixi.jl vs FLUXO: Julia Parallel Scaling to 61K Cores | 2026-07 | Scientific computing |
| 5 | Omega Ocean Model: Kokkos Portability on Frontier/Aurora/Perlmutter | 2026 | Scientific computing |
| 6 | GALÆXI: DGSEM on AMD MI300A APUs (CPX partitioning) | 2026-06 | Scientific computing |
| 7 | "Peak-to-Operator Attenuation Cascade" for Matrix Engines (SPECFEM3D) | 2026-08 | Scientific computing |
| 8 | Kim et al., "Neural Network-Driven Importance Sampling for Rare Events" | 2026-02 | Monte Carlo |
| 9 | Moreau-Yosida Envelopes for MCMC Importance Sampling | 2025-01 | Monte Carlo |
| 10 | Niching Importance Sampling for Multi-modal Rare-event Simulation | 2026-04 | Monte Carlo |
| 11 | IS for Bayesian Inference: Bochner Integrability Bounds | 2026-04 | Monte Carlo |
| 12 | Entropic Mirror Monte Carlo (EM2C) | 2026-02 | Monte Carlo |
| 13 | "CUDA vs ROCm vs Vulkan vs Metal: GPU Compute in 2026" (orchestrator.dev) | 2026-05 | GPU compute |
| 14 | "CUDA vs ROCm vs SYCL vs Metal vs Vulkan 2026" (MyAIHardware) | 2026-05 | GPU compute |
| 15 | ROCm vs CUDA 2026: Benchmarks, Gaps & Real Cost (Spheron) | 2026-04 | GPU compute |
| 16 | ROCm vs CUDA GPU Computing (ThunderCompute) | 2026-09 | GPU compute |
| 17 | AMD ROCm 10 + ROCm.AI Announcement | 2026-08 | GPU compute |
| 18 | Meganeura: Portable GPU Training/Inference via Vulkan+Metal | 2026-08 | GPU compute |
| 19 | ROCm vs CUDA for Local AI 2026 (insiderllm) | 2026 | GPU compute |

---

## What's NEW vs Batch 592

Batch 592 identified: (1) agentic review lacks verification feedback loop, (2) 94% single-PR merge serialization, (3) AI code breaks main 2.4x less, (4) repo-wide context as table stakes, (5) stacked PRs needed, (6) batch bisection missing.

Batch 593 adds **6 NEW defects** drawn from scientific computing, Monte Carlo, and GPU compute research:

| # | New Defect | Source |
|---|-----------|--------|
| N1 | **Runtime-composable architecture missing** — TopSim proves plugin-based frameworks with unified data protocols outperform rigid APIs for scientific simulation. NeoTrix SEAL pipeline lacks runtime-composable module replacement. | [3] TopSim |
| N2 | **Bias potential learning absent** — Neural network-driven MCMC importance sampling learns optimal bias potentials to escape metastable states. NeoTrix attention routing has no dynamic bias mechanism for exploration vs exploitation tradeoff. | [8] Kim et al. |
| N3 | **Operator-path attenuation not modeled** — The 4x SME peak advantage decays to 1.1x full-operator speedup due to pointwise computation, indirect field movement, and coefficient delivery overhead. NeoTrix has no model for how architecture advantages degrade through the full processing path. | [7] SPECFEM3D |
| N4 | **Vendor-agnostic compute abstraction absent** — Meganeura proves Vulkan+Metal can deliver competitive training/inference on consumer GPUs. NeoTrix is CUDA-locked with no Vulkan/Metal fallback for local Apple/AMD hardware. | [18] Meganeura |
| N5 | **Agentic GPU optimization exists in production** — ROCm.AI Hyperloom is an autonomous agentic system optimizing end-to-end inference across host+GPU kernels. NeoTrix has no equivalent self-optimizing compute scheduler. | [17] AMD ROCm 10 |
| N6 | **Niching/diversity for multi-modal search missing** — Niching importance sampling maintains diversity across multiple failure modes. NeoTrix exploration has no niching mechanism, risking convergence to single optima in multi-modal fitness landscapes. | [10] NIS |

---

## Detailed Findings

### 1. Scientific Computing — Runtime Composability & Performance Portability

**TopSim** [3] demonstrates a C++ framework with a **service-oriented plugin architecture** configured at runtime. Three key architectural features:
1. Runtime-composable plugins — swap numerical methods without modifying the kernel
2. **Unified TopS data structure** — serves as both compact topological model AND shared inter-plugin communication protocol
3. Transparent distributed-memory backbone (ParTopS, TopMat, TopVec)

**Key result**: TopSim-native distributed linear algebra outperforms PETSc at >10K elements/core workload, with superior weak scaling. Below 10K elements/core, PETSc's loose coupling wins.

**Defect N1**: NeoTrix SEAL pipeline has hardcoded phase transitions. No runtime-composable module replacement mechanism exists. The TopSim pattern — unified data protocol + plugin interface — directly addresses NeoTrix's rigidity in module composition.

**MARUT** [2] is a Julia-based multi-GPU CFD framework achieving:
- Near-linear strong scaling across multiple GPUs
- Native GPU-resident computations with MPI communication overlap
- Integration with differentiable programming and ML workflows (Julia's inherent capability)
- AMR (Adaptive Mesh Refinement) using Löhner-type indicators

**Defect (refines batch 592)**: MARUT's Julia implementation enables **differentiable simulation** — gradients flow through the physics solver itself. NeoTrix's SEAL pipeline has no differentiable programming capability. This is a concrete gap: the entire evolution loop could be gradient-optimized rather than heuristic-tuned.

**Trixi.jl vs FLUXO** [4] — Julia scales to 61,440 CPU cores with 0.83 parallel efficiency, matching Fortran. But a critical finding: **parallel code loading causes a bottleneck at 8,192+ cores**, solved by generating a pre-compiled system image.

**Defect (extends batch 592)**: NeoTrix module initialization at startup is analogous to Julia's code loading problem. At scale, NeoTrix's module system will hit similar loading bottlenecks. The solution — pre-compiled system images — maps to NeoTrix needing pre-compiled module snapshots for fast startup.

**Omega** [5] — DOE ocean model using **Kokkos** for performance portability across Frontier (AMD MI250X), Aurora (Intel Max), and Perlmutter (NVIDIA A100). Key finding: GPU-native redesign was necessary because OpenACC directives on the legacy Fortran code achieved only ~50% of expected GPU throughput due to small kernel sizes and frequent CPU↔GPU data transfers.

**GALÆXI on AMD MI300A** [6] — The **CPX (Compute Partitioning eXtended)** mode treats each APU as 6 logical GPUs, yielding **10% PID reduction** and **up to 3x speedup** over SPX mode. This is because memory-bound DGSEM benefits from better memory locality with finer-grained partitioning.

**Defect N3**: The **peak-to-operator attenuation cascade** from SPECFEM3D [7] is the most important finding. On Arm LX2 CPUs:
- 4x SME peak advantage → 2.2x for isolated contractions → **1.1x for full operator**
- Loss factors: pointwise computation, indirect field movement, synchronization, irregular coefficient delivery
- Recovery strategy: explicit SIMD + field-layout changes + vector-blocked AoSoV → **1.6x at high order**

**This maps directly to NeoTrix**: any "architectural advantage" (E8, GWT, HyperCube) will degrade through the full processing path. NeoTrix has **no model for this attenuation**. An advantage at the kernel level does not survive through the full operator pipeline.

### 2. Monte Carlo — Importance Sampling & Rare Event Simulation

**Neural Network-Driven Importance Sampling** [8] achieves:
- Bias potential learned via neural network in **logarithmic space** — avoids underflow near energy minima
- **Branching Random Walk (BRW)** reduces variance by ~8x vs standard importance sampling
- Validated on 2D (14D) systems, capturing transition pathways correctly
- Key: reweighting sampled paths yields unbiased transition rates **even when learned bias is imperfect**

**Defect N2**: NeoTrix attention routing uses static saliency thresholds. The MCMC importance sampling literature shows that **optimal bias potentials must be learned dynamically** — the exploration distribution must adapt to the target landscape. NeoTrix lacks:
1. A bias potential (exploration prior) that adapts based on observed fitness landscape
2. Variance reduction via branching (allocating more compute to promising attention branches)
3. Reweighting mechanism to correct for imperfect exploration biases

**Moreau-Yosida Envelopes** [9] — Smooth approximation enables gradient-based MCMC (MALA/HMC) for non-differentiable posteriors. Asymptotic normality with explicit covariance — enables variance estimation.

**Niching Importance Sampling** [10] — Integrates evolutionary niching into IS framework:
- Maintains **multiple importance modes** simultaneously
- Von Mises-Fisher-Nakagami mixture models capture directional + magnitude variation
- EM algorithm fits mixture without initialization
- Mutual information determines budget allocation

**Defect N6**: NeoTrix exploration has no niching mechanism. When the fitness landscape has multiple optima (multi-modal), single-mode importance sampling converges to one mode. NIS shows how to maintain diversity across modes — directly applicable to NeoTrix's multi-domain architecture (7 domains, 11 ConsciousnessTree branches).

**IS for Bayesian Inference** [11] — Proves that O(N^{-1/2}) convergence holds **if and only if** a model-dependent link function is square-integrable (Bochner integrable). For bounded observation models, error grows at most **polynomially** (not exponentially) with dimension.

**Defect (quantitative)**: NeoTrix's VSA HyperCube dimensionality may suffer from the curse of dimensionality in importance-weighted recall. This paper proves the condition under which IS remains efficient — NeoTrix has no such analysis.

**EM2C** [12] — Combines Entropic Mirror Descent contraction with Markovian exploration:
- First term: preserves contraction (exploitation of current proposal)
- Second term: Markov kernel exploration (reaching unexplored regions)
- Key result: consistently recovers all modes under initialization where AIS fails

### 3. GPU Compute — CUDA Gap, ROCm Viability, Metal 4, Vulkan Portability

**The "CUDA Gap"** [13][14] is now quantified:
- CUDA's real-world throughput exceeds raw TFLOPS prediction by 10-20% due to software stack maturity (cuDNN, TensorRT accumulated over 18 years)
- **CUDA Gap Score**: ranges from 28.7 to 99.1 across benchmarks — software advantage equivalent to 30-99% more hardware
- AMD MI300X has 32% more TFLOPS than H100 but CUDA throughput exceeds ROCm by 30-68% at various batch sizes
- At batch size 64-128, ROCm MI355X closes to 5-10% of H100 — near parity for memory-bound workloads

**ROCm 7 production viability** [15][16]:
- PyTorch, vLLM, SGLang all have official ROCm support
- TensorRT-LLM and FlashAttention 3: **no ROCm equivalent** — 20-40% gap
- Meta's 6GW AMD MI450 deployment ($60-100B) validates ROCm at scale
- MLPerf Inference 6.0: MI355X within ~5% of B200 on server inference

**ROCm.AI + Hyperloom** [17] — AMD's autonomous agentic optimization system:
- Optimizes end-to-end inference workloads across host code AND GPU kernels
- Natural language interface for deployment and optimization
- ROCm CLI provides version-agnostic experience across ROCm releases
- **This is a production implementation of self-optimizing compute** — directly relevant to NeoTrix's NT-ACT orchestration

**Metal 4** [13]:
- `MTL4MachineLearningCommandEncoder` — ML passes as GPU commands, synchronized with render/compute
- `MTLTensor` — native multi-dimensional data container for ML workloads
- Unified memory: zero-copy CPU↔GPU on Apple Silicon
- **Limitation**: no multi-GPU scaling, no FP8/INT4 tensor core support, no FlashAttention

**Defect N4**: NeoTrix has no Vulkan or Metal backend. Meganeura [18] proves portable GPU training/inference via Vulkan+Metal is viable:
- 48/50 device-workload-mode cells pass validation
- Median training gap vs PyTorch: **1.8x** (competitive)
- On AMD discrete GPU: 4/5 inference workloads within 1.10x of ROCm PyTorch
- Compilation: 0.1-2.4 seconds vs 6-96 seconds for PyTorch
- Stripped binary: **13 MiB**

**Vulkan beating ROCm on RDNA 4** [19] — On AMD RX 9070:
- Vulkan: 69.2 tok/s (Llama 3.1 8B) vs ROCm HIP: 60.9 tok/s → **+14%**
- For GPT-OSS 20B: Vulkan 152.7 vs ROCm 117.2 → **+30%**
- Prompt processing: Vulkan 2,888 tok/s vs ROCm 1,149 → **2.5x faster**

**Defect (quantitative)**: CUDA kernel efficiency gap — NVIDIA averages 0.13 tok/s per GB/s of bandwidth. AMD averages 0.06 → **2x software efficiency penalty**. This is not hardware — it's kernel optimization depth.

---

## Defect Summary vs Batch 592

| Batch 592 Defect | Status | Batch 593 Refinement |
|------------------|--------|---------------------|
| (1) Agentic review lacks verification feedback | Carried forward | — |
| (2) 94% single-PR merge serialization | Carried forward | — |
| (3) AI code breaks main 2.4x less | Carried forward | — |
| (4) Repo-wide context as table stakes | Carried forward | — |
| (5) Stacked PRs needed | Carried forward | — |
| (6) Batch bisection missing | Carried forward | — |
| **N1** Runtime-composable architecture | **NEW** | TopSim plugin pattern; SEAL phases should be swappable at runtime |
| **N2** Bias potential learning absent | **NEW** | MCMC IS proves adaptive exploration priors outperform static thresholds |
| **N3** Operator-path attenuation not modeled | **NEW** | 4x kernel advantage → 1.1x full operator; NeoTrix has no attenuation model |
| **N4** Vendor-agnostic compute absent | **NEW** | Meganeura + Vulkan prove portable GPU training viable at 1.8x PyTorch gap |
| **N5** Agentic GPU optimization exists | **NEW** | ROCm.AI Hyperloom is production self-optimizing compute; NeoTrix has none |
| **N6** Niching for multi-modal search missing | **NEW** | NIS maintains diversity across failure modes; NeoTrix lacks niching |

---

## Cross-Domain Synthesis

The deepest pattern across all three domains: **software ecosystem depth now exceeds hardware capability as the primary performance differentiator**.

- CUDA's 18-year kernel optimization library creates a 30-99% effective hardware advantage (CUDA Gap Score)
- ROCm has 90-95% of CUDA throughput for standard workloads but 20-40% gap for specialized kernels
- Scientific computing frameworks (TopSim, MARUT, Omega) are moving to **runtime-composable, plugin-based architectures** with unified data protocols
- Monte Carlo methods prove that **adaptive bias potentials + branching + niching** are essential for efficient exploration in multi-modal landscapes
- GPU compute is converging on a **multi-backend** future: CUDA for NVIDIA, ROCm for AMD, Metal for Apple, Vulkan for cross-platform

**Implication for NeoTrix**: The consciousness architecture needs (1) runtime-composable module replacement (TopSim pattern), (2) adaptive bias potentials for attention routing (MCMC IS pattern), (3) vendor-agnostic compute abstraction (Meganeura pattern), and (4) niching for multi-domain exploration (NIS pattern).

---

## Priority Defects for Iteration 594

1. **N3** (Attenuation cascade) — Highest. Without modeling how advantages degrade through the full path, all performance claims are suspect.
2. **N1** (Runtime composability) — High. SEAL pipeline rigidity is a structural bottleneck.
3. **N2** (Bias potential learning) — High. Static attention routing is provably suboptimal for multi-modal landscapes.
4. **N6** (Niching) — Medium. Multi-domain architecture requires diversity maintenance.
5. **N4** (Vendor-agnostic compute) — Medium. Apple Silicon support requires Vulkan/Metal path.
6. **N5** (Agentic GPU optimization) — Lower. Production pattern exists but NeoTrix is not yet at that scale.
