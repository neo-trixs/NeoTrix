# Iteration Batch 383 — Distributed Training / Communication Efficiency / Scalable ML

**Date:** 2026-09-06
**Scope:** External research scan → NeoTrix design defect identification → suggestions

---

## 1. Sources Cited

### Distributed Training (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| D1 | Placement Semantics Framework, arXiv:2601.02311 (Jan 2026) | Unified formal framework: five placement modes (replicated/sharded/sharded-with-gather/materialized/offloaded) derive memory and communication from specifications alone. ZeRO-3 = 8× less memory than DP at 1.5× communication cost. Composition rules for TP+DP, PP+DP proven correct. |
| D2 | TorchTitan SPMD Types, PyTorch DevBlog (Aug 2026) | `spmd_types` replaces DTensor as compute backend: 42-46% eager throughput gain, erasure-mode validation (typecheck then strip), coupled FWD-BWD typing, global+local SPMD for MoE/TP. DTensor at rest, spmd_types in compute. |
| D3 | PyTorch TP Tutorial, PyTorch Docs (2026) | 2D parallelism: TP intra-host (NVLink) + FSDP inter-host. Sequence Parallel on LayerNorm/RMSNorm. Loss Parallel for vocabulary sharding. Handles >256 GPU scaling where FSDP alone hits ring latency wall. |
| D4 | TorchTitan System, arXiv:2410.06511 | 4D parallelism (DP+TP+PP+CP) modular composition. Float8 training + SymmetricMemory for hardware utilization. Elastic scaling with distributed checkpointing. Llama 3.1 optimized recipes. |
| D5 | Megatron Bridge Parallelisms, NVIDIA Docs (2026) | Multi-dimensional parallelism config: TP=2-8, PP=2-16, CP=2, EP for MoE. Expert Parallel + DeepEP/HybridEP for MoE token dispatching. Memory optimization: distributed optimizer + SP + context parallelism. |

### Communication Efficiency (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| C1 | FedCEF, arXiv:2603.07654 (Mar 2026) | Federated Composite Error Feedback for non-convex FCO. Decoupled proximal update: non-smooth terms handled locally, compressed info transmitted. Error feedback with control variates. Sublinear convergence under general non-convexity. Competitive at 1% compression ratio. |
| C2 | GradESTC, arXiv:2601.10491 (Jan 2026) | Spatio-temporal gradient compression: SVD captures spatial low-rank structure, dynamic basis update exploits temporal correlation. 39.79% uplink reduction vs strongest baseline. Preserves convergence speed and final accuracy vs uncompressed FedAvg. |
| C3 | FedSGM, arXiv:2601.16897 (Jan 2026) | Unified framework: functional constraints + bidirectional compression + multi-step local updates + partial participation. Projection-free primal-only updates. Bi-directional error feedback. O(1/√T) convergence rate. First framework unifying all four challenges. |
| C4 | Gradient-Aware EBLC, arXiv:2511.05770 (Nov 2025) | Error-bounded lossy compression for FL gradients. Cross-round EMA magnitude predictor + oscillation-aware sign predictor + kernel-level sign consistency. 1.53× better than SZ3, 76.1-96.2% communication time reduction in real-world APPFL framework. |
| C5 | Full Compression Pipeline (FCP), arXiv:2604.11146 (Apr 2026) | Pruning→Quantization→Huffman unified pipeline. 11× model size reduction with only 2% accuracy drop. Layer-wise single-pass execution. Complexity remains O(N) linear, matching standard FL. |
| C6 | Adaptive-STC, ETASR (Jun 2026) | Dynamic gradient sparsification thresholds learned from local gradient statistics per round. 18% communication cost reduction vs dense FedAvg and fixed-threshold STC. Less than 0.3% accuracy degradation. |
| C7 | SA-PEF, OpenReview (2026) | Step-Ahead Partial Error Feedback: combines step-ahead correction with partial EF. Interpolates between EF (α=0) and SAEF (α=1). Convergence to stationarity matching nonconvex FedSGD. Faster early-round progress, no late-stage plateaus. |
| C8 | E-3SFC, arXiv:2502.03092 (Feb 2025) | Model-as-decompressor: gradients compressed into synthetic features via backpropagation. Double-way compression (upload + download). Budget scheduler allocates more budget to early training. 111.6× communication cost reduction. |

### Scalable ML / Scaling Laws / MoE / Distributed Inference (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| S1 | Holistic MoE Scaling Framework, arXiv:2603.21862 (Mar 2026) | Joint constraint triad (FLOPs M, active params Na, total params N). Algebraic decoupling reduces O(n^16) search to O(n^3)+O(n^2). Near-optimal band widens with scale. Key finding: FLOPs/token alone is inadequate MoE metric — parameter inflation yields misleading gains. |
| S2 | MOSAIC: Compute-Optimal ≠ Cluster-Optimal, arXiv:2608.10605 (Aug 2026) | Systems-aware MoE scaling: architecture+systems co-design as single optimization. MoE sparsity optimum emerges only under cluster constraints, not from model FLOPs alone. Mixed-integer nonlinear program resolves boundary sparsity. |
| S3 | Comprehensive MoE Scaling Law, arXiv:2509.23678 (Sep 2025) | 5-factor MoE law: (D, N, Na, G, S). 446 controlled experiments. Optimal activated experts ≈7. Optimal Na/N ratio 20-43% (theoretical), 5-9% (practical efficiency-aware). Shared experts essential: 13-31% ratio optimal. |
| S4 | MoE Scaling: muP to MSSP, arXiv:2605.14200 (May 2026) | Maximally Scale-Stable Parameterization for MoE: three co-scaling regimes (N≍Ne, N≍M≍K, full proportional). DMFT analysis of limiting dynamics. muP fails for MoE — MSSP achieves robust LR transfer and monotonic improvement. |
| S5 | MegaScale-Infer, arXiv:2504.02263 (Apr 2025) | Disaggregated MoE serving: attention+FFN on separate GPUs. Ping-pong pipeline parallelism hides communication. 1.90× throughput improvement. Heterogeneous deployment: attention on memory-optimized GPUs, FFN on compute-optimized GPUs. |
| S6 | QEIL: Inference-time Scaling Laws, arXiv:2602.06057 (Feb 2026) | 5 architecture-agnostic theorems for inference scaling. Coverage: C(S)=1-exp(-αN^βS^β), β≈0.7. Heterogeneous orchestration yields superlinear efficiency gains invisible to homogeneous. Intelligence Per Watt 2.08-5.60× improvement. |
| S7 | MoE Generalization & Scaling Theory, arXiv:2604.09175 (Apr 2026) | Sup-norm covering-number bound decomposes into active-parameter term + routing combinatorial term (k·log(eM/k)). Dense scaling exponents recovered when measured against active budget Nact. Routing contributes logarithmic overhead only. |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-D1: No Multi-Dimensional Parallelism Composition

**Evidence:** `ParallelTaskManager` (`nt_act/parallel_task.rs:162`) manages task-level parallelism with a simple `max_parallel_tasks: u32` limit. The `InferenceRuntime` (`inference_runtime.rs:65`) has a `tensor_parallel_size: usize` field defaulting to 1. There is no mechanism to compose TP+DP+PP+CP parallelism strategies, no DeviceMesh abstraction, and no placement semantics.

**2026 Gap:** D1 proves five placement modes derive memory/communication from specifications alone. D2's `spmd_types` achieves 42-46% throughput gain over DTensor via erasure-mode validation. D3/D4 demonstrate that >256 GPU training requires 2D/4D parallelism composition. NeoTrix's parallel task system is flat — it cannot express or compose different parallelism strategies across model layers.

**Severity:** High — NeoTrix cannot orchestrate distributed training or inference across multi-GPU/multi-node deployments beyond trivial data parallelism.

**Suggestion:** Implement a `PlacementSpec` struct in NT-PHYSICAL encoding per-state placement mode (R/S/S*/M/O) for parameters, optimizer, gradients, and activations. Add a `ParallelismComposer` that validates composition rules (from D1 Theorems 5-7) and derives memory/communication budgets. Integrate with `spmd_types`-style erasure mode for validation-then-compute separation. The GWT attention router should modulate parallelism strategy selection based on current cluster topology from HeartbeatAggregator.

---

### DEFECT-D2: No Spatio-Temporal Gradient Compression

**Evidence:** NeoTrix has no gradient compression for any distributed computation. The SEAL pipeline's distillation and self-test stages operate on single-node. When deployed across multiple instances (e.g., multi-enterprise KB sharing), there is no mechanism to compress inter-node gradients or model updates.

**2026 Gap:** C2 (GradESTC) achieves 39.79% uplink reduction by exploiting spatial SVD low-rank structure + temporal correlation across rounds. C4 (EBLC) achieves 76-96% communication time reduction via cross-round EMA prediction. C8 (E-3SFC) achieves 111.6× reduction via model-as-decompressor with budget scheduling. NeoTrix has none of these capabilities.

**Severity:** High — multi-node NeoTrix deployments will be bottlenecked by communication; no mechanism to reduce inter-node data transfer.

**Suggestion:** Implement `GradientCompressor` in NT-ACT with: (1) SVD-based spatial decomposition (C2) for batch gradient compression, (2) EMA-based temporal predictor (C4) for cross-round residual compression, (3) adaptive sparsification thresholds (C6) learned from local gradient statistics. Add `BudgetScheduler` (C8) that allocates more communication budget to early training phases. Wire into the EventBus for inter-node gradient synchronization.

---

### DEFECT-D3: No Systems-Aware Scaling for MoE Routing

**Evidence:** The VSA HyperCube uses fixed-dimension high-dimensional vectors for knowledge representation. The E8 Hexagram uses a fixed 64-element grid. There is no Mixture-of-Experts architecture where tokens are dynamically routed to specialized sub-networks. The SEAL pipeline's skill crystallization produces monolithic skill artifacts.

**2026 Gap:** S1 establishes that FLOPs/token alone is inadequate for MoE — the joint triad (M, Na, N) is required. S2 (MOSAIC) proves that MoE sparsity optimum emerges only under cluster constraints. S3 identifies optimal Na/N ratio at 20-43% (theoretical), with optimal activated experts ≈7. S7 proves routing overhead scales as k·log(eM/k) — logarithmic in expert count. NeoTrix has no MoE architecture for any subsystem.

**Severity:** High — NeoTrix cannot leverage sparse computation to decouple model capacity from inference cost; all processing is dense.

**Suggestion:** Implement `MoERouter` in NT-CORE that: (1) defines expert specialization domains aligned with NeoTrix's 7 factions (NT-CORE=logic, NT-MIND=evolution, NT-MEMORY=storage, etc.), (2) implements top-k gating with the k·log(eM/k) overhead budget, (3) enforces optimal Na/N ratio from S3 (20-43% active). Add a `SharedExpert` pool for cross-domain knowledge (13-31% of active experts per S3). Wire to GWT attention: salience score determines routing weight, not just routing binary.

---

### DEFECT-D4: No Disaggregated Attention/FFN for Inference

**Evidence:** `InferenceRuntime` (`inference_runtime.rs`) loads a single model instance with `model_path` and `backend`. No separation between attention computation and feed-forward computation. No ping-pong pipeline parallelism. No heterogeneous deployment (different hardware for different compute phases).

**2026 Gap:** S5 (MegaScale-Infer) demonstrates 1.90× throughput via disaggregated attention/FFN with ping-pong pipeline parallelism. Attention on memory-optimized GPUs, FFN on compute-optimized GPUs. S6 (QEIL) shows heterogeneous orchestration yields superlinear efficiency gains invisible to homogeneous baselines. NeoTrix's inference runtime is monolithic — single GPU, no phase disaggregation.

**Severity:** High — NeoTrix cannot optimize inference for MoE-scale models; cannot exploit hardware heterogeneity.

**Suggestion:** Add `DisaggregatedInferenceEngine` to NT-PHYSICAL that: (1) separates attention and FFN onto different GPU pools, (2) implements ping-pong micro-batch pipeline (S5) to hide communication, (3) routes attention to memory-optimized GPUs and FFN to compute-optimized GPUs, (4) uses M2N communication library for zero-copy GPU-to-GPU transfer. The HeartbeatAggregator should report per-GPU utilization to inform disaggregation decisions.

---

### DEFECT-D5: No Bidirectional Compression for Federated Knowledge Sharing

**Evidence:** NeoTrix's KB synchronization across instances uses no compression. The `nt_memory` module stores embeddings and BM25 index but has no mechanism to compress or differentially sync across distributed instances. Egress guard redacts source code but does not compress outbound model updates.

**2026 Gap:** C3 (FedSGM) unifies bidirectional compression with functional constraints and multi-step local updates. C8 (E-3SFC) achieves double-way compression (upload + download) with budget scheduling. C1 (FedCEF) enables compressed federated composite optimization with error feedback at 1% compression ratio. NeoTrix has no bidirectional compression for any distributed synchronization.

**Severity:** Medium — multi-instance NeoTrix deployments waste bandwidth on uncompressed KB sync; limits scalability to bandwidth-constrained environments.

**Suggestion:** Implement `BidirectionalCompressor` in NT-MEMORY that: (1) applies SVD-based compression (C2) for KB embedding sync (upload), (2) applies EMA prediction (C4) for model weight distribution (download), (3) integrates error feedback (C1/C7) to maintain accuracy under aggressive compression. Add `FedSyncCoordinator` that manages partial participation and non-IID data distributions across NeoTrix instances.

---

### DEFECT-D6: No Scale-Stable Parameterization for Self-Evolving Models

**Evidence:** SEAL pipeline distills skills into fixed-parameter artifacts. No mechanism ensures that hyperparameters scale correctly as model capacity grows. The E8 Hexagram grid is fixed at 64 elements regardless of system scale.

**2026 Gap:** S4 (MSSP) proves that muP fails for MoE architectures — learning rate transfer breaks under scale. MSSP achieves robust LR transfer across three co-scaling regimes via DMFT-analyzed limiting dynamics. NeoTrix's skill distillation has no scale-stable parameterization; increasing model capacity requires manual hyperparameter retuning.

**Severity:** Medium — NeoTrix cannot scale its self-evolving models without manual intervention; scaling breaks learned hyperparameters.

**Suggestion:** Implement `ScaleStableParameterizer` in NT-MIND that: (1) applies MSSP (S4) for any MoE-based skill module, (2) ensures learning rate transfer across co-scaling regimes (N≍Ne, N≍M≍K, full), (3) uses DMFT-derived limiting dynamics to predict training stability at target scale. Wire into SEAL pipeline's distillation stage to automatically adjust parameters when scaling capacity.

---

### DEFECT-D7: No Inference-Time Scaling Law for Resource Allocation

**Evidence:** `ResourceBudgetManager` tracks Token/GPU/cost budgets but uses no scaling law to predict how coverage, energy, or latency will change with model size or sample budget. The `InferenceConfig` has static parameters with no principled scaling.

**2026 Gap:** S6 (QEIL) proves 5 architecture-agnostic theorems: coverage scales as C(S)=1-exp(-αN^βS^β) with β≈0.7, energy scales sub-linearly, cost follows device-specific multipliers, latency scales with parallelism. Heterogeneous orchestration achieves 2.08-5.60× Intelligence Per Watt. NeoTrix has no scaling law to predict inference efficiency or allocate resources optimally.

**Severity:** Medium — NeoTrix allocates inference resources by static heuristics, not principled scaling predictions; wastes energy and compute.

**Suggestion:** Implement `InferenceScalingLaw` in NT-PHYSICAL that: (1) encodes the 5 theorems from S6 as differentiable cost models, (2) predicts coverage/energy/latency/cost for any (N, S, T) configuration, (3) uses heterogeneous orchestration to route operations to cost-optimal devices (CPU/GPU/NPU). Wire into ResourceBudgetManager for principled budget allocation based on predicted scaling rather than static limits.

---

### DEFECT-D8: No Error Feedback for Distributed Self-Test Validation

**Evidence:** SelfTest tier T2 (Registration) and T3 (Production Wiring) validate modules but have no mechanism to handle communication errors in distributed validation. The `converge_check()` function operates on local state only.

**2026 Gap:** C7 (SA-PEF) demonstrates that step-ahead partial error feedback achieves faster early-round progress without late-stage plateaus. C1 (FedCEF) shows error feedback with control variates maintains convergence under aggressive compression. NeoTrix's self-test validation has no error feedback mechanism — if distributed validation messages are lost or compressed, results may be incorrect.

**Severity:** Medium — distributed self-test may produce incorrect health assessments under communication failures.

**Suggestion:** Add `ErrorFeedbackLayer` to NT-REPAIR that: (1) maintains residual state for distributed SelfTest validation messages, (2) applies SA-PEF (C7) for early-round accuracy, (3) degrades gracefully under partial participation. Integrate with HeartbeatAggregator to report communication-quality-adjusted health scores.

---

## 3. Summary

| Category | Defects | Severity Distribution |
|----------|---------|----------------------|
| Distributed Training | D1, D6 | 1 High, 1 Medium |
| Communication Efficiency | D2, D5, D8 | 1 High, 1 Medium, 1 Medium |
| Scalable ML / MoE / Inference | D3, D4, D7 | 2 High, 1 Medium |

**Total: 8 defects identified, 4 High, 4 Medium**

**Priority Actions:**
1. **D1 (Multi-Dimensional Parallelism)** — foundational for all distributed computation; blocks D3, D4, D5
2. **D3 (MoE Routing)** — enables sparse computation; decouples capacity from cost; critical for scaling
3. **D4 (Disaggregated Inference)** — enables heterogeneous hardware utilization; 1.9× throughput gain proven
4. **D2 (Spatio-Temporal Gradient Compression)** — reduces inter-node communication by 40-96%; enables bandwidth-constrained deployment
5. **D7 (Inference-Time Scaling Law)** — provides principled resource allocation; replaces static heuristics with predictive models
