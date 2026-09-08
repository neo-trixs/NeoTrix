# Iteration Batch 505 — Hardware Architecture Research Loop

**Date**: 2026-09-06
**Focus**: CPU architecture, memory hierarchy, parallel computing advances (2026)
**Research depth**: Hot Chips 2026, OSDI 26, MLSys 26, arxiv latest

---

## Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | Tom's Hardware — Arm AGI at Hot Chips 2026 | 2026-08-26 | Arm AGI dual-chiplet server CPU, 136 V3 cores, CMN-S3 mesh, UCIe 2TB/s D2D |
| S2 | ServeTheHome — Arm AGI Data Center CPU | 2026-08-24 | Arm first in-house CPU, 300W TDP, 12-ch DDR5, CXL 3.0 |
| S3 | ARM Feature Names v2026_06 | 2026-06 | Armv9.6 ISA: SME2p2, SVE2p2, FEAT_PCDPHINT, FEAT_LSFE, FEAT_MPAM_PE_BW_CTRL |
| S4 | Chips and Cheese — Fujitsu Monaka | 2026-08-26 | Monaka 144-core, 2nm core + 5nm SRAM/IO, 256-bit SVE2, 3 NUMA modes |
| S5 | ServeTheHome — Fujitsu Monaka | 2026-08-24 | Ultra-low voltage, 500W/350W SKUs, N2P+N5 die stacking |
| S6 | CNX Software — StarFive Dubhe-100 | 2026-08-20 | RVA23 RISC-V, 15-stage 6-wide OoO, 256-bit VLEN, TAGE predictor |
| S7 | SIXE — IBM Z Dual-ISA (Arm+Z) | 2026-08-27 | Native Arm+S390x execution, 36MB L2/core, 432MB virtual L3, SMT-2 |
| S8 | arxiv 2607.28824 — NUMA GPUs for LLM Inference | 2026-07-30 | Multi-partition GPU NUMA effects, 1.09×-1.79× latency penalty |
| S9 | arxiv 2608.00867 — NUNA | 2026-08-01 | Non-uniform network access in multi-die GPUs, NAR+NAP up to 80% reduction |
| S10 | arxiv 2606.25353 — Cache-Resident LLM Inference | 2026-06-24 | GB-scale LLC for LLM, weight-attention decoupled architecture, up to 13.9× speedup |
| S11 | OSDI 26 — DirectKV | 2026 | Zero-copy KV cache offloading via NVLink-C2C, CPU memory as GPU extension |
| S12 | arxiv 2607.28633 — TopKV Topology-Aware Disaggregated Inference | 2026-07 | 72× bandwidth hierarchy exploitation, CXL 3.0 as overflow tier |
| S13 | arxiv 2608.22613 — NOVA | 2026-08-23 | Near-memory processing for Attention-SSM-MoE hybrid inference |
| S14 | arxiv 2609.03905 — Einsummable | 2026-09-03 | Auto multi-GPU parallelism via relational join abstraction |
| S15 | arxiv 2608.19628 — FIBER | 2026-08-20 | Decoupled shared-register GPU execution, 1.8-2.3× speedup |
| S16 | MLSys 26 — Event Tensor | 2026 | Dynamic megakernel compiler abstraction for fused LLM inference |
| S17 | MLSys 26 — ParallelKittens | 2026 | Minimal multi-GPU kernel framework, up to 4.08× speedup |
| S18 | arxiv 2608.22602 — Async Distributed GPU Simulator | 2026-08-23 | Cycle-level MCM GPU simulation, validated 99% correlation on H100 |
| S19 | arxiv 2609.01864 — CREDIT | 2026-09-01 | DSMEM inter-CTA tiling, 1.47× on RTX 5090, 1.32× on H100 |
| S20 | OSDI 26 — MPK | 2026 | Mega-kernel compiler/runtime for multi-GPU, up to 1.7× latency reduction |
| S21 | Yenra — AI Parallel Computing 16 Advances | 2026 | NeuSight GPU prediction, OpenMP 6.0, Liger-Kernel 20% throughput |
| S22 | Shattered.io — Fujitsu Monaka Deep Dive | 2026-08-28 | 3D chiplet disaggregation by cost, ~30% area reduction |

---

## Defects Identified in NeoTrix Design

### DEFECT-1: No NUMA-Aware Workload Partitioning (CRITICAL)

**Source**: S8, S9, S10, S12
**Severity**: HIGH
**Category**: Memory Hierarchy / Parallel Computing

2026 research demonstrates that NUMA effects now manifest *intra-package* in multi-partition GPUs (AMD MI300X, NVIDIA B200). The cost of NUMA in LLM serving ranges from 1.09× (compute-bound) to 1.79× (memory-bound GQA decode). NeoTrix's NT-PHYSICAL domain (`nt_physical::parallel_task`) and NT-ACT (`nt_act::parallel_task`) have no NUMA topology detection or partition-aware scheduling. The `ParallelTaskManager` (alias `TaskScheduler`) manages GPU memory and batch scheduling but treats all GPU partitions as uniform — a flat abstraction that silently creates cross-partition traffic.

**Gap**: No `NUNA-aware routing` (NAR) or `NUNA-aware placement` (NAP) concept exists in the architecture. The HeartbeatAggregator doesn't model memory locality as a health signal.

**Suggestion**: Introduce a `NumaTopology` trait in L3 Embodiment that:
1. Probes GPU partition hierarchy at startup (NVLink domain, PCIe switch tree, NUMA node)
2. Exposes locality scores to GWT for attention routing
3. Feeds into ParallelTaskManager for partition-aware workgroup scheduling

---

### DEFECT-2: Missing Weight-Attention Decoupled Execution Model (CRITICAL)

**Source**: S10, S13
**Severity**: HIGH
**Category**: Memory Hierarchy / Architecture Pattern

Cache-resident LLM inference (arxiv 2606.25353) proves that separating weight-centric operators from attention/KV-cache management into dedicated resource domains yields 2.04×–13.9× TPOT speedup. The "KV-Cache Pressure Paradox" — where deeper pipelines increase KV footprint proportionally to weight sharding — is unresolved in NeoTrix's SEAL pipeline design.

**Gap**: The SEAL pipeline (`seal_pipeline.rs`) treats all operators uniformly. There is no concept of weight nodes vs. attention nodes, no cache residency tracking, and no sub-operator dependency relaxation (the paper shows operator-boundary synchronization costs are disproportionate in cache-resident regimes).

**Suggestion**: Add to NT-CORE or NT-MIND:
- `WeightResidencyManager` — tracks which model weights are LLC-resident
- `AttentionDomain` — isolated KV-cache management decoupled from weight execution
- `SubOperatorDependency` — relaxed synchronization model (replaces operator-boundary barriers)

---

### DEFECT-3: No CXL 3.0 Memory Expansion Tier (HIGH)

**Source**: S7, S11, S12
**Severity**: HIGH
**Category**: Memory Hierarchy / Architecture

CXL 3.0 Type 3 memory expanders provide 512GB additional capacity at 150ns latency and 64GB/s bandwidth — 86× lower latency than NVMe and 9× higher bandwidth. IBM Z (S7) already ships with CXL support. Arm AGI (S1) supports CXL 3.0 on 96 PCIe 6.0 lanes. Fujitsu Monaka (S4) has CXL 3.0 production support for 2027.

**Gap**: NeoTrix's NT-MEMORY domain (`nt_memory`) models KB as SQLite-backed with no concept of tiered memory (HBM → DDR → CXL → NVMe). The `HeartbeatAggregator` tracks compilation/test/KB/eventbus health but has no memory hierarchy signal. When NeoTrix runs as a daemon on modern servers, it cannot leverage CXL-expanded memory for KB caching or embedding index residency.

**Suggestion**: Extend NT-MEMORY with a `MemoryTierManager` that:
1. Detects available CXL endpoints via `cxl-cli` or sysfs
2. Implements a 3-tier cache: hot (DDR) → warm (CXL) → cold (NVMe)
3. Exposes tier-aware eviction policies for KB embeddings and BM25 index

---

### DEFECT-4: Missing ISA Feature Detection and Adaptation (MEDIUM)

**Source**: S3, S6, S7
**Severity**: MEDIUM
**Category**: CPU Architecture / Self-Adaptation

Armv9.6 (S3) introduces FEAT_SME2p2 (structured sparsity outer products), FEAT_SVE2p2 (BF16 scaling), FEAT_PCDPHINT (producer-consumer data placement hints), and FEAT_MPAM_PE_BW_CTRL (per-core bandwidth controls). RISC-V RVA23 (S6) brings 256-bit VLEN vector processing. IBM Z (S7) executes dual-ISA natively with SMT-2.

**Gap**: NeoTrix has no runtime ISA feature detection. The E8 Hexagram reasoning engine, VSA HyperCube operations, and GWT attention routing are implemented as pure Rust with no SIMD specialization. On Arm AGI with 128-bit SVE2 or Fujitsu Monaka with 256-bit SVE2, NeoTrix leaves significant vector throughput unused. On RISC-V with 256-bit VLEN, the same opportunity is missed.

**Suggestion**: Add an `IsaCapabilities` module in NT-PHYSICAL that:
1. Probes CPU features at startup (via `std::arch::is_x86_feature_detected!` / `std::arch::is_aarch64_feature_detected!` / RISC-V HPM)
2. Provides dispatch hooks for VSA embedding operations (dot products, cosine similarity) to use native SIMD
3. Exposes MPAM partitioning hints for NT-MEMORY KB operations

---

### DEFECT-5: No Topology-Aware Inter-GPU Communication (HIGH)

**Source**: S9, S14, S17, S18
**Severity**: HIGH
**Category**: Parallel Computing / Communication

Modern GPU scale-up networks expose 72×–144× bandwidth hierarchy (NVLink 4.0: 900GB/s, IB: 50GB/s, TCP: 12.5GB/s). NUNA-aware routing (S9) reduces collective latency up to 80%. ParallelKittens (S17) achieves 4.08× speedup for sequence-parallel workloads with just 50 lines of device code by understanding transfer mechanisms (copy engines vs. TMA vs. register-level).

**Gap**: NeoTrix's `ParallelTaskManager` and `BatchProductionManager` have no concept of GPU interconnect topology. Multi-GPU operations (if ever deployed) would use uniform communication patterns, ignoring that bandwidth varies by 72× depending on physical relationship between GPUs.

**Suggestion**: Add `InterconnectTopology` to NT-PHYSICAL that:
1. Probes NVLink/PCIe/RDMA hierarchy at startup
2. Classifies GPU pairs by bandwidth tier
3. Feeds topology data to task placement in NT-ACT

---

### DEFECT-6: No Megakernel / Persistent Kernel Execution Model (MEDIUM)

**Source**: S16, S20
**Severity**: MEDIUM
**Category**: Parallel Computing / Execution Model

Event Tensor (S16) and MPK (S20) demonstrate that fusing multiple operators into a single persistent megakernel eliminates kernel launch overhead and enables inter-kernel parallelism. MPK achieves up to 1.7× latency reduction. Event Tensor handles dynamic shapes and data-dependent control flow (MoE expert routing) within megakernels.

**Gap**: NeoTrix's execution model is kernel-per-operator (standard CUDA execution). The SEAL pipeline stages are sequential with operator-boundary synchronization. For any GPU-accelerated NeoTrix workloads (e.g., VSA embedding computation, E8 reasoning on GPU), this creates unnecessary launch overhead and prevents fine-grained overlap.

**Suggestion**: Define a `MegakernelExecutor` trait in L1 Action that:
1. Accepts fused operator graphs
2. Manages persistent kernel lifecycle across SMs
3. Supports dynamic shape dispatch (Event Tensor semantics)

---

### DEFECT-7: Missing Zero-Copy Heterogeneous Memory Model (MEDIUM)

**Source**: S11
**Severity**: MEDIUM
**Category**: Memory Hierarchy

DirectKV (OSDI 26) proves zero-copy KV cache offloading is practical on GH200 via NVLink-C2C (900GB/s), achieving 50% transfer reduction and 43% GPU memory savings. The key insight: use shared memory (SMEM) to shift bandwidth pressure from CPU-GPU interconnect to HBM, making CPU memory a practical extension of GPU capacity.

**Gap**: NeoTrix assumes CPU and GPU memory are separate domains. NT-MEMORY (SQLite KB) runs entirely in CPU memory. NT-PHYSICAL has no mechanism to make KB embeddings accessible to GPU kernels without staging buffers. This limits any future GPU-accelerated knowledge retrieval.

**Suggestion**: Add a `HeterogeneousMemoryBridge` that:
1. Exposes KB embeddings as CUDA-accessible memory via NVLink-C2C or PCIe P2P
2. Implements SMEM-aware tiling for cross-device matrix operations
3. Tracks residency across CPU DDR, GPU HBM, and CXL tiers

---

### DEFECT-8: No Energy-Delay Product Optimization (LOW)

**Source**: S4, S5, S21
**Severity**: LOW
**Category**: Architecture / Power Management

Fujitsu Monaka (S4/S5) achieves 30% lower voltage than comparable designs through ultra-low-voltage operation, making power the primary optimization target. Modern schedulers (S21) optimize for energy-delay product (EDP) rather than single-metric performance. Arm AGI (S1) uses MPAM bandwidth controls and QoS-based congestion feedback.

**Gap**: NeoTrix's HeartbeatAggregator tracks health but has no power/energy dimension. The `CostManager` tracks token cost but not energy cost. When running on Arm AGI (300W TDP) or Monaka (350W-500W), NeoTrix cannot make energy-aware scheduling decisions.

**Suggestion**: Extend HeartbeatAggregator with an `EnergySignal` that:
1. Reads RAPL/ARM-energy-counters at periodic intervals
2. Feeds EDP into GWT attention modulation
3. Enables energy-aware task placement in NT-ACT

---

### DEFECT-9: No Dynamic Parallelism Scaling for Mixed-Phase Workloads (MEDIUM)

**Source**: S15
**Severity**: MEDIUM
**Category**: Parallel Computing / GPU Architecture

FIBER (S15) demonstrates that decoupling thread register ownership from execution instances enables dynamic parallelism scaling — the hardware can adjust active fiber count at runtime to match phase-dependent needs (memory-bound vs. tensor-bound vs. vector-bound). This achieves 1.8–2.3× speedup across GPU generations.

**Gap**: NeoTrix's GPU execution model assumes static thread allocation. The `ParallelTaskManager` allocates GPU resources at task launch time with fixed thread counts. For mixed-phase workloads (e.g., VSA embedding computation that alternates between GEMM and elementwise operations), this static allocation cannot adapt.

**Suggestion**: Add a `PhaseAdaptiveExecutor` in NT-PHYSICAL that:
1. Detects compute phase transitions (memory → tensor → vector)
2. Adjusts active thread/fiber count dynamically
3. Coordinates with FIBER-style shared register access when available

---

### DEFECT-10: No Dual-ISA / Multi-Architecture Awareness (LOW)

**Source**: S7, S6
**Severity**: LOW
**Category**: CPU Architecture / Portability

IBM Z (S7) executes Arm and S390x natively on every core with SMT-2. RISC-V (S6) is emerging as a third ISA for server workloads. NeoTrix is a Rust project targeting x86-64 and aarch64, but has no awareness of multi-ISA environments.

**Gap**: When NeoTrix runs on IBM Z (dual-ISA) or future multi-ISA platforms, it cannot leverage the ability to execute different workload phases on the most efficient ISA. This is a forward-looking concern but worth tracking as IBM Z Arm support matures in 2026-2027.

**Suggestion**: Add a note to NT-PHYSICAL's body schema concept: multi-ISA awareness for workload migration across ISA boundaries. Low priority but architecturally interesting.

---

## Summary

| # | Defect | Severity | Domain | Source |
|---|--------|----------|--------|--------|
| 1 | No NUMA-aware workload partitioning | CRITICAL | NT-PHYSICAL, NT-ACT | S8,S9,S10,S12 |
| 2 | Missing weight-attention decoupled execution | CRITICAL | NT-CORE, NT-MIND | S10,S13 |
| 3 | No CXL 3.0 memory expansion tier | HIGH | NT-MEMORY | S7,S11,S12 |
| 4 | Missing ISA feature detection/adaptation | MEDIUM | NT-PHYSICAL | S3,S6,S7 |
| 5 | No topology-aware inter-GPU communication | HIGH | NT-PHYSICAL, NT-ACT | S9,S14,S17 |
| 6 | No megakernel/persistent kernel execution | MEDIUM | L1 Action | S16,S20 |
| 7 | Missing zero-copy heterogeneous memory | MEDIUM | NT-MEMORY, NT-PHYSICAL | S11 |
| 8 | No energy-delay product optimization | LOW | NT-PHYSICAL, Heartbeat | S4,S5,S21 |
| 9 | No dynamic parallelism scaling | MEDIUM | NT-PHYSICAL | S15 |
| 10 | No dual-ISA awareness | LOW | NT-PHYSICAL | S7,S6 |

## Priority Recommendations

1. **Immediate (Sprint 1)**: DEFECT-1 (NUMA) + DEFECT-2 (Weight-Attention Decoupling) — these yield the highest performance impact (up to 13.9× for cache-resident inference)
2. **Short-term (Sprint 2-3)**: DEFECT-3 (CXL) + DEFECT-5 (Topology-Aware Communication) — infrastructure for modern server deployment
3. **Medium-term**: DEFECT-4 (ISA Detection) + DEFECT-6 (Megakernel) + DEFECT-7 (Zero-Copy) + DEFECT-9 (Dynamic Parallelism) — performance optimization layer
4. **Long-term**: DEFECT-8 (Energy) + DEFECT-10 (Dual-ISA) — future platform readiness
