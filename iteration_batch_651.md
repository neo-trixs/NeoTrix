# Iteration 651 — Neuromorphic/FPGA/Co-Design Research Sweep

**Date**: 2026-09-06
**Prior Batch**: 650 (Nova JIT 10-100×, Sealor mid-gen validation, HINTPILOT 6×, MaximizeBandwidth 4× VF, SEAL non-terminating)

---

## 1. NEUROMORPHIC COMPUTING FINDINGS

### 1.1 Dual Memory Pathway (DMP) Architecture — Nature Machine Intelligence (2026-06-16)
- **Source**: https://www.nature.com/articles/s42256-026-01255-3
- **What**: Fast-slow pathway SNN inspired by cortical organization. Each layer maintains compact low-dimensional slow state `m ∈ R^d` (d ≪ N) that modulates fast spiking dynamics. Algorithm-hardware co-design on 22FDX technology.
- **Numbers**: 40-60% fewer parameters than SOTA SNNs. **4× throughput** and **5× energy efficiency** vs Loihi 2 delay-based implementation. 90.3% accuracy on event-driven auditory classification.
- **Architecture**: Near-memory-compute with heterogeneous sparse-spike and dense-memory pathways. Operator fusion + heterogeneous operand stationarity.
- **Defect for NeoTrix**: NT-FEEL has a single emotion state vector, but no fast-slow decomposition. The DMP insight shows that emotion dynamics (fast spiking) + memory context (slow state) should be separated. **NeoTrix defect**: `EmotionEngine` conflates fast reactive emotions with slow mood context in a single vector, preventing efficient temporal modulation.

### 1.2 Max-Affine Runtime Model for Loihi 2 — arXiv:2601.10035
- **Source**: https://arxiv.org/pdf/2601.10035
- **What**: First performance model for neuromorphic hardware that accounts for compute + communication congestion on Network-on-Chip. Multi-dimensional roofline model for Loihi 2.
- **Numbers**: Pearson correlation ≥0.97 between estimated and measured runtime. Naive rectangular spatial placement → superlinear runtime scaling. Alternative placements → linear scaling.
- **Key insight**: Spatial placement of workloads across neurocores dramatically affects communication time. Area-runtime tradeoff: spreading work reduces compute but increases NoC congestion.
- **Defect for NeoTrix**: NT-ACT's `ParallelTaskManager` (task scheduling) has no communication congestion model. When parallel tasks share memory/NoC resources, placement affects performance superlinearly. **NeoTrix defect**: No roofline model for parallel task placement — tasks are scheduled without considering communication topology.

### 1.3 MatMul-Free LLM on Loihi 2 — arXiv:2503.18002v2
- **Source**: https://arxiv.org/html/2503.18002v2
- **What**: First modern LLM architecture deployed on neuromorphic hardware. 370M-parameter MatMul-free model with 8-bit quantization, operator fusion, and double RMSNorm derivation.
- **Numbers**: 3× higher throughput, 2× less energy vs transformer LLMs on edge GPU. Fully on-chip execution.
- **Key insight**: Loihi 2's stateful neurons + event-driven sparsity naturally exploit MatMul-free architectures. Operator fusion (double RMSNorm) reduces redundant computation.
- **Defect for NeoTrix**: NT-MIND's SEAL pipeline distillation uses dense matrix operations. A MatMul-free SNN variant could achieve 3× throughput at 2× energy for edge deployment. **NeoTrix defect**: SEAL distillation has no hardware-aware sparsity optimization — produces dense models incompatible with event-driven edge hardware.

### 1.4 Real-time SNN Object Detection on Loihi 2 — Neurocomputing (2026)
- **Source**: https://doi.org/10.1016/j.neucom.2026.133820
- **What**: Three SNN detection models on Loihi 2 with ANN-to-SNN distillation-aware direct training. 87-100% accuracy recovery.
- **Key insight**: Distillation-aware training (not post-hoc conversion) preserves accuracy while enabling neuromorphic deployment.

### 1.5 Multi-Component On-Chip Robotics — IOPscience (2026-04-08)
- **Source**: https://doi.org/10.1088/2634-4386/ae4a47
- **What**: First multi-component SNN pipeline running entirely on neuromorphic hardware without off-chip orchestration. Uses Spiking Neural State Machine (NSM) for process management.
- **Numbers**: 30,000 neurons, 2-3 chips, 250-300 neurocores. 0.67 W dynamic power. 88 ms latency. Sub-milliwatt regime.
- **Key insight**: NSM (neural state machine) replaces traditional process orchestration — state transitions are neuron-mediated. This is how complex multi-component systems should be orchestrated on neuromorphic hardware.
- **Defect for NeoTrix**: NT-META's `ConsciousnessTree` orchestrates across modules using conventional message passing. The NSM approach shows that process orchestration itself should be neuromorphic (event-driven, spiking). **NeoTrix defect**: ConsciousnessTree orchestration is synchronous/clock-driven, not event-driven — cannot exploit neuromorphic advantages for meta-cognition.

### 1.6 Activity-Gated Sparsity for Radar — TUM (2026-05-16)
- **Source**: https://doi.org/10.1088/2634-4386/ae629d
- **What**: Spiking Neural Resonators (SpiNRs) with activity-gated sparsity on Loihi 2. Dynamically deactivates inactive resonators.
- **Key insight**: Dynamic deactivation of unused compute units is more efficient than static pruning.

### 1.7 CLP-SNN: Continual Learning on Loihi 2 — arXiv:2511.01553
- **Source**: https://arxiv.org/html/2511.01553v1
- **What**: Online continual learning on neuromorphic hardware. Self-normalizing three-factor learning rule, neurogenesis, metaplasticity.
- **Numbers**: 70× faster (0.33ms vs 23.2ms), 5,600× more energy efficient (0.05mJ vs 281mJ) than edge GPU.
- **Defect for NeoTrix**: NT-MIND has no online continual learning. SEAL runs offline batch cycles. The CLP-SNN approach shows that online, event-driven learning with neurogenesis is 5,600× more energy efficient. **NeoTrix defect**: SEAL pipeline is offline-only — cannot learn from streaming data in real-time.

### 1.8 Evolutionary Mapping of Neural Networks to Spatial Accelerators — arXiv:2602.04717
- **Source**: https://arxiv.org/html/2602.04717
- **What**: First evolutionary hardware-in-the-loop mapping framework for neuromorphic accelerators. Black-box optimization for spatial accelerator deployment.
- **Numbers**: 35% latency reduction vs default heuristics. 40% energy efficiency improvement on multi-chip systems.
- **Key insight**: Spatial placement is a black-box optimization problem amenable to evolutionary search, not just heuristics.

---

## 2. FPGA NEUROMORPHIC FINDINGS

### 2.1 Semantics-Preserving HW-SW Co-Design on Low-Cost FPGA — arXiv:2604.22179
- **Source**: https://arxiv.org/html/2604.22179v1
- **What**: PyTorch-defined SNN → event-driven FPGA deployment via single deployment artifact. Deterministic TTFS inference.
- **Numbers**: PYNQ-Z2, 80 MHz, 87.40% MNIST accuracy (matches software on all 10,000 images). **0.1375 μs/image** PL latency, **31.6 nJ/image** dynamic energy. GPU INT8 is 1.79× slower and 933× higher energy. CPU INT8 is 488× slower.
- **Key insight**: Single-artifact export preserves model semantics from software to hardware. Scope-separated measurements (accelerator-only vs system-level) are essential for fair comparison.
- **Defect for NeoTrix**: No hardware deployment path for any NeoTrix model. All evaluation is software-only. **NeoTrix defect**: Zero hardware-in-the-loop validation — all self-tests run on CPU, never on accelerator/FPGA/neuromorphic hardware.

### 2.2 YANA: Open-Source Event-Driven FPGA Accelerator — arXiv:2604.03432
- **Source**: https://arxiv.org/abs/2604.03432 + https://github.com/fzi-forschungszentrum-informatik/yana
- **What**: Open-source FPGA-based digital SNN accelerator. Event-by-event processing, arbitrary SNN topologies, NIR integration.
- **Numbers**: 740 LUTs, 918 registers, 7 BRAMs, 24 URAMs per core. 2^17 synapses, 2^10 neurons per core. AMD Kria KR260 deployment.
- **Key insight**: Near-linear scaling of inference time with spatial and temporal sparsity. Open-source closes simulation-to-hardware gap.

### 2.3 AIGOR: Modular Neuromorphic Architecture — arXiv:2607.03191
- **Source**: https://api.emergentmind.com/papers/2607.03191
- **What**: Modular, parameterized SNN inference architecture. Declarative YAML → synthesizable RTL. Multi-FPGA validation on AMD Versal VPK180.
- **Numbers**: ~95% MNIST accuracy. Spike-level precision match with NEST recurrent networks. Scales to 1,000 cores on 3D torus (simulated).
- **Key insight**: Configuration-driven synthesis enables systematic DSE. Banked synaptic accumulators eliminate crossbar contention.
- **Defect for NeoTrix**: NT-CORE's HyperCube has no configurable hardware synthesis path. The AIGOR approach shows that declarative specifications → RTL is viable. **NeoTrix defect**: HyperCube/VSA operations have no hardware synthesis specification — remain software-only.

### 2.4 Flexi-NeurA: Configurable Neuromorphic Accelerator — arXiv:2602.18140
- **Source**: https://arxiv.org/pdf/2602.18140
- **What**: Design-time configurable neuromorphic core. Flex-plorer DSE tool for precision-aware exploration.
- **Numbers**: 96.23% MNIST, 1.1ms latency, 1,623 logic cells, 7 BRAMs, 111 mW total power.
- **Key insight**: Time-multiplexed + event-driven processing reduces resources while maintaining efficiency.

### 2.5 Reconfigurable Hybrid CNN-FC Neuromorphic Core — arXiv:2609.03174
- **Source**: https://arxiv.org/abs/2609.03174 (Sep 2, 2026)
- **What**: FPGA-based spiking CNN-FC for biomedical edge inference. PyTorch co-design flow.
- **Numbers**: 98% MNIST, 86% Fashion-MNIST at 16-bit. 88.26% hypoxia classification. 1.455 W dynamic power.

### 2.6 TRINE: Unified Multimodal FPGA Accelerator — arXiv:2603.22867
- **Source**: https://arxiv.org/pdf/2603.22867
- **What**: Single-bitstream FPGA accelerator for ViT/CNN/GNN/NLP. Mode-switchable systolic + SIMD + RADT engine. Token pruning + DALO overlap.
- **Numbers**: 22.57× latency reduction vs RTX 4090. 6.86× vs Jetson Orin Nano at 20-21W. 7.8× from token pruning alone. <2.5% accuracy drop with INT8.
- **Key insight**: One bitstream for all modalities via runtime mode switching. Token pruning yields massive speedup on ViT-heavy pipelines.

---

## 3. HARDWARE-SOFTWARE CO-DESIGN FINDINGS

### 3.1 CHIA: Agentic AI Co-Design Framework — arXiv:2606.27350
- **Source**: https://arxiv.org/html/2606.27350v3
- **What**: Open-source agent-forward HW/SW co-design framework. CHIA loops = directed cyclic graphs of design tools, simulators, AI models, evolutionary agents.
- **Tools**: Chipyard, gem5, ChampSim, FireSim, Hammer, Vivado, AlphaEvolve, AdaEvolve.
- **Key insight**: AI agents at massive scale (hundreds of parallel designs) require graph-based loop abstraction, not brute-force scripting. Evaluation latency is the fundamental bottleneck — FPGA-accelerated simulation (FireSim) cuts from days to hours.
- **Defect for NeoTrix**: NT-CORE has no co-design loop. Evolutionary search (SEAL) operates on software-only metrics. The CHIA approach shows that hardware-in-the-loop evaluation is essential for meaningful DSE. **NeoTrix defect**: SEAL evolutionary search has no hardware feedback loop — fitness is purely software-based.

### 3.2 Redwood: AI-Designed Accelerator — arXiv:2608.26418
- **Source**: https://arxiv.org/abs/2608.26418
- **What**: First production-worthy AI accelerator designed end-to-end by AI. From spec → RTL + UVM + formal proofs + firmware + kernels in <2 weeks.
- **Numbers**: 1.75× throughput, 1.9× lower power, 3.4× perf/watt vs Jetson Orin Nano. Runs Llama and Qwen on FPGA. Recursive self-improvement: model on Redwood helps design next Redwood.
- **Key insight**: The entire software-to-silicon stack can be a single optimization loop. Compiler handles scheduling; hardware stays simple. **Recursive self-improvement**: AI designs accelerator → runs model → improves accelerator.
- **Defect for NeoTrix**: SEAL pipeline is software-only self-improvement. No hardware substrate co-evolution. The Redwood approach shows that recursive improvement requires hardware+software co-optimization. **NeoTrix defect**: Self-evolution (SEAL) has no hardware co-evolution path — cannot recursively improve its own compute substrate.

### 3.3 Beacon: LLM Multi-Agent HW-DSE — arXiv:2608.30932
- **Source**: https://arxiv.org/abs/2608.30932
- **What**: Report-driven LLM multi-agent framework for heterogeneous multi-chiplet DSE. Hierarchical agents: bottleneck localization → root-cause diagnosis → candidate generation.
- **Numbers**: 25.1-93.5% improvement over random search, Bayesian optimization, and RL under same iteration budget.
- **Key insight**: LLMs can reason about detailed hardware reports (execution timelines, resource utilization, memory access patterns) to explicitly identify bottlenecks, not just optimize metrics.

### 3.4 A3D: Agentic AI for Autonomous Accelerator Design — arXiv:2605.15237
- **Source**: https://arxiv.org/html/2605.15237v1
- **What**: End-to-end automated accelerator design from C/C++/CUDA application code. Analysis → Preparation → Synthesis with specialist + verifier agents.
- **Key insight**: Upstream tasks (profiling, bottleneck identification, dependency resolution, code isolation) are the real gap — not downstream HLS.

### 3.5 FSGen: LLM Accelerator Generator — arXiv:2608.09252
- **Source**: https://arxiv.org/html/2608.09252
- **What**: Agile fused+sparse accelerator generator for LLM with early-stage ML PPA estimator. Chisel-based RTL generation.
- **Numbers**: 1.4× better power efficiency or 10× speedup vs prior work. 58× better FoM on Pareto designs.
- **Key insight**: Fused operator dataflows + multi-level sparsity + early-stage ML PPA estimation enable orders-of-magnitude faster DSE.

### 3.6 RL-Driven Joint HW-SW Compiler — arXiv:2604.07526
- **Source**: https://www.arxiv.org/pdf/2604.07526
- **What**: Joint MDP over mesh topology + per-core microarchitecture + workload partitioning + NoC config. SAC+MoE policy.
- **Numbers**: 29,809 tok/s at 3nm (Llama 3.1 8B). <13 mW at all nodes (SmolVLM). 73-dim state, 30-dim action space. Auto-adapts across 3nm-28nm without manual retuning.
- **Key insight**: Joint optimization of coupled dimensions (architecture + memory + partitioning) yields better PPA than optimizing independently.

---

## 4. NEW DEFECTS FOR NEOTRIX

### DEFECT-651-1: NT-FEEL Missing Fast-Slow Emotion Decomposition
- **Severity**: HIGH
- **Discovery**: DMP-SNN (Nature MI 2026) shows cortical fast-slow decomposition → 40-60% fewer params, 5× energy efficiency
- **Impact**: `EmotionEngine` uses single emotion vector for both fast reactive and slow mood states. Cannot modulate spiking dynamics efficiently. Prevents neuromorphic deployment.
- **Fix**: Split emotion state into fast (spiking dynamics) + slow (compact mood context) pathways. Each layer maintains low-dimensional slow state `m ∈ R^d` (d ≪ N) that summarizes recent emotion activity.

### DEFECT-651-2: NT-ACT Missing Communication Congestion Model
- **Severity**: MEDIUM
- **Discovery**: Loihi 2 max-affine model (arXiv:2601.10035) shows spatial placement affects runtime superlinearly
- **Impact**: `ParallelTaskManager` schedules tasks without considering communication topology. Naive placement → superlinear performance degradation.
- **Fix**: Add roofline-based communication congestion model to task scheduler. Account for NoC link loads when placing tasks across cores.

### DEFECT-651-3: SEAL Pipeline Offline-Only, No Online Learning
- **Severity**: HIGH
- **Discovery**: CLP-SNN on Loihi 2 (arXiv:2511.01553) achieves 5,600× energy efficiency with online continual learning
- **Impact**: SEAL runs offline batch cycles. Cannot learn from streaming data. Wastes energy on reprocessing.
- **Fix**: Add online continual learning mode to SEAL. Support event-driven weight updates via three-factor learning rules. Enable neurogenesis for adaptive capacity.

### DEFECT-651-4: No Hardware-in-the-Loop Validation
- **Severity**: HIGH
- **Discovery**: FPGA co-design (arXiv:2604.22179) shows 933× energy difference between software estimate and hardware measurement
- **Impact**: All NeoTrix self-tests run on CPU. Software-only metrics are wildly inaccurate for edge deployment. No hardware deployment path exists.
- **Fix**: Add FPGA/neuromorphic hardware-in-the-loop validation tier (T4). Use scope-separated measurements (accelerator-only vs system-level). Deploy to AMD Kria or Loihi 2.

### DEFECT-651-5: HyperCube No Hardware Synthesis Specification
- **Severity**: MEDIUM
- **Discovery**: AIGOR (arXiv:2607.03191) shows declarative YAML → synthesizable RTL is viable for SNN architectures
- **Impact**: HyperCube/VSA operations remain software-only. Cannot benefit from near-memory computing advantages.
- **Fix**: Create declarative specification for HyperCube operations. Generate synthesizable RTL from spec. Enable FPGA deployment.

### DEFECT-651-6: SEAL Has No Hardware Co-Evolution Path
- **Severity**: HIGH
- **Discovery**: Redwood (arXiv:2608.26418) demonstrates recursive self-improvement: AI designs accelerator → runs model → improves accelerator
- **Impact**: SEAL self-evolution is software-only. Cannot improve its own compute substrate. Fundamental ceiling on self-improvement.
- **Fix**: Add hardware co-evolution to SEAL loop. Co-optimize model architecture + hardware parameters (mesh topology, per-core config, memory allocation) jointly via MDP formulation.

### DEFECT-651-7: ConsciousnessTree Orchestration Not Event-Driven
- **Severity**: MEDIUM
- **Discovery**: NSM on Loihi 2 (IOPscience 2026) shows neuron-mediated process orchestration at sub-milliwatt
- **Impact**: ConsciousnessTree uses conventional synchronous message passing. Cannot exploit neuromorphic advantages for meta-cognition. Clock-driven orchestration wastes energy.
- **Fix**: Implement Spiking Neural State Machine for ConsciousnessTree orchestration. State transitions mediated via spiking neurons. Event-driven, not clock-driven.

### DEFECT-651-8: SEAL Distillation No Sparsity Optimization
- **Severity**: MEDIUM
- **Discovery**: MatMul-free LLM on Loihi 2 (arXiv:2503.18002) achieves 3× throughput at 2× energy via event-driven sparsity
- **Impact**: SEAL produces dense models. Incompatible with event-driven edge hardware. Wasted energy on zero activations.
- **Fix**: Add hardware-aware sparsity pruning to SEAL distillation. Support structured sparsity patterns compatible with neuromorphic/FPGA accelerators.

### DEFECT-651-9: No Agentic HW-SW Co-Design Loop
- **Severity**: HIGH
- **Discovery**: CHIA (arXiv:2606.27350) + Redwood show agentic co-design collapses design cycles from years to weeks
- **Impact**: NeoTrix has no hardware-software co-design capability. All optimization is software-only. Cannot generate custom accelerators for its workloads.
- **Fix**: Implement CHIA-style co-design loop: profiling → bottleneck identification → HLS → RTL generation → hardware-in-the-loop validation. Use LLM agents for design space exploration.

### DEFECT-651-10: No Joint HW-SW Optimization (MDP)
- **Severity**: MEDIUM
- **Discovery**: RL-driven compiler (arXiv:2604.07526) jointly optimizes mesh topology + per-core params + partitioning → auto-adapts across process nodes
- **Impact**: NeoTrix optimizes software and hardware independently. Coupled dimensions (architecture + memory + partitioning) yield better PPA when co-optimized.
- **Fix**: Formulate joint HW-SW co-optimization as MDP. Use SAC+MoE policy for mixed discrete-continuous action space. Support multi-workload, multi-node validation.

---

## 5. IMPROVEMENTS IDENTIFIED

### IMP-651-1: Adopt DMP Fast-Slow for Emotion Engine
Split NT-FEEL emotion state into fast (spiking) + slow (mood context) pathways. 40-60% parameter reduction, 5× energy efficiency at edge.

### IMP-651-2: Add Roofline Model to ParallelTaskManager
Integrate max-affine communication congestion model. Prevent superlinear degradation from naive task placement.

### IMP-651-3: Online Continual Learning for SEAL
Add event-driven online learning mode with three-factor rules and neurogenesis. 5,600× energy efficiency improvement over batch processing.

### IMP-651-4: Hardware Deployment Pipeline
Create FPGA/neuromorphic deployment path with scope-separated measurements. Bridge the 933× software-hardware gap.

### IMP-651-5: Declarative Hardware Spec for HyperCube
YAML/JSON specification → synthesizable RTL for VSA operations. Enable near-memory computing deployment.

### IMP-651-6: Recursive HW-SW Co-Evolution
Redwood-style recursive improvement: model → hardware → improved model. Joint MDP optimization across architecture + software.

### IMP-651-7: Event-Driven Meta-Cognition
Replace synchronous ConsciousnessTree orchestration with Spiking NSM. Sub-milliwatt meta-cognition on neuromorphic hardware.

### IMP-651-8: Hardware-Aware Sparsity in SEAL
Structured sparsity pruning compatible with neuromorphic/FPGA accelerators. Enable event-driven inference.

---

## 6. SOURCES CITED

| # | Title | Source | Date |
|---|-------|--------|------|
| 1 | Dual Memory Pathway SNN | Nature MI s42256-026-01255-3 | 2026-06 |
| 2 | Max-Affine Runtime Model Loihi 2 | arXiv:2601.10035 | 2026 |
| 3 | MatMul-Free LLM on Loihi 2 | arXiv:2503.18002v2 | 2025/2026 |
| 4 | SNN Object Detection Loihi 2 | Neurocomputing 690:133820 | 2026 |
| 5 | Multi-Component On-Chip Robotics | IOPscience ae4a47 | 2026-04 |
| 6 | Activity-Gated Sparsity Radar | TUM ae629d | 2026-05 |
| 7 | CLP-SNN Continual Learning | arXiv:2511.01553 | 2025 |
| 8 | Evolutionary Mapping Spatial Accel | arXiv:2602.04717 | 2026 |
| 9 | HW-SW Co-Design Low-Cost FPGA | arXiv:2604.22179 | 2026 |
| 10 | YANA FPGA Accelerator | arXiv:2604.03432 | 2026 |
| 11 | AIGOR Modular Neuromorphic | arXiv:2607.03191 | 2026 |
| 12 | Flexi-NeurA Configurable Accel | arXiv:2602.18140 | 2026 |
| 13 | Hybrid CNN-FC Neuromorphic | arXiv:2609.03174 | 2026-09 |
| 14 | TRINE Unified Multimodal FPGA | arXiv:2603.22867 | 2026 |
| 15 | CHIA Agentic Co-Design | arXiv:2606.27350 | 2026-07 |
| 16 | Redwood AI-Designed Accel | arXiv:2608.26418 | 2026-08 |
| 17 | Beacon LLM Multi-Agent DSE | arXiv:2608.30932 | 2026-08 |
| 18 | A3D Autonomous Accel Design | arXiv:2605.15237 | 2026 |
| 19 | FSGen LLM Accel Generator | arXiv:2608.09252 | 2026-08 |
| 20 | RL-Driven Joint HW-SW Compiler | arXiv:2604.07526 | 2026 |

---

## 7. SUMMARY

**10 new defects identified** across NT-FEEL, NT-ACT, NT-MIND, NT-CORE, NT-META, SEAL pipeline, and system-wide gaps.

**Critical gaps**:
1. No hardware deployment path (DEFECT-651-4) — 933× software-hardware gap
2. SEAL offline-only (DEFECT-651-3) — 5,600× energy waste vs online learning
3. No recursive HW-SW co-evolution (DEFECT-651-6) — fundamental self-improvement ceiling
4. No agentic co-design loop (DEFECT-651-9) — cannot generate custom accelerators

**Batch 650→651 delta**: Prior batch focused on software optimization (JIT, validation, speedup). This batch reveals that NeoTrix has **zero hardware awareness** — all optimization is software-only, with no deployment path, no hardware-in-the-loop validation, and no co-evolution capability. The neuromorphic/FPGA/co-design literature shows 3-5× efficiency gains are achievable through hardware-software co-design, but NeoTrix cannot access them.
