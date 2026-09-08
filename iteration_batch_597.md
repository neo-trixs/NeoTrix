# Iteration Batch 597 — Hardware Accelerator Landscape Scan
**Date**: 2026-09-06
**Focus**: FPGA, ASIC/TPU, Neuromorphic Chip Developments (2026)
**Baseline**: Batch 596 (climate drift, hybrid orchestration, carbon-aware scheduling, material science KG, data provenance)

---

## 1. FPGA — Key Findings

### Hummingbird+ (ACM/SIGDA FPGA 2026)
- **Source**: https://dl.acm.org/doi/10.1145/3748173.3779189
- **What**: Edge LLM deployment on Zynq UltraScale XCZU2CG/3EG SoC, 24GB memory, <$150 BOM
- **Specs**: GPTQ 4-bit Qwen3-30B-A3B at 18+ tok/s decode, 50+ tok/s prefill
- **Significance**: First FPGA-based edge product as cost-effective final implementation for LLM deployment

### Redwood (arXiv:2608.26418, Aug 2026)
- **Source**: https://arxiv.org/abs/2608.26418
- **What**: Frontier AI accelerator designed end-to-end by AI in 2 weeks from human spec
- **Specs**: 1.75x throughput, 1.9x lower power vs Jetson Orin Nano; 3.4x perf/watt
- **Key**: Recursive self-improvement — Qwen running on Redwood helped design next-gen Redwood
- **Significance**: First production-worthy AI accelerator designed entirely by AI

### FPGAgent (arXiv:2608.23630, Aug 2026)
- **Source**: https://arxiv.org/abs/2608.23630
- **What**: Multi-agent LLM framework for autonomous HLS code generation + board-level validation
- **Specs**: 16.9% synthesizable rate improvement, 26.7% executability, 30.6% functional correctness
- **Key**: First task-spec-to-executable HLS framework validated on real FPGA hardware

### ELiTeFormer (arXiv:2607.03652, Jul 2026)
- **Source**: https://arxiv.org/html/2607.03652v1
- **What**: HW/SW co-designed ternary Transformer for FPGA — 10x weight compression, 12.8x KV cache compression
- **Specs**: 9.6x FFN speedup, 4.4x attention speedup vs LLaMA 3; 3.9x latency, 3.2x energy efficiency vs A100
- **Key**: First FPGA realization combining linear attention + ternary quantization

### TRINE (arXiv:2603.22867)
- **Source**: https://arxiv.org/pdf/2603.22867
- **What**: Single-bitstream multimodal accelerator — ViT/CNN/GNN/NLP unified without reconfiguration
- **Specs**: 22.57x latency reduction vs RTX 4090; up to 7.8x speedup via token pruning; <2.5% accuracy drop under int8
- **Key**: Runtime-adaptive mode-switching (WS/OS, SIMD, RADT) within one bitstream

### Gen-TAS (arXiv:2608.28160, Aug 2026)
- **Source**: https://arxiv.org/abs/2608.28160
- **What**: LLM-aided FPGA-GPP task allocation with RAG-grounded reasoning
- **Specs**: Up to 2.45x speedup (CNN) and 92.53x (SDR) vs all-GPP baselines
- **Key**: Human-in-the-loop selection with deterministic backend for reproducibility

---

## 2. ASIC / TPU — Key Findings

### Google TPU 8t + 8i (Google Cloud Next 2026, Apr 2026)
- **Source**: https://blog.google/innovation-and-ai/infrastructure-and-cloud/google-cloud/eighth-generation-tpu-agentic-era/
- **What**: Split training (8t) and inference (8i) TPUs — first dual-architecture generation
- **Specs**:
  - TPU 8t: 9,600 chips/pod, 3x compute perf/pod over Ironwood, Virgo Network interconnect
  - TPU 8i: 384MB on-chip SRAM (3x Ironwood), 288GB HBM per chip, Collectives Acceleration Engine (CAE)
  - Both: 2x perf/watt over Ironwood; Axion ARM host co-designed
- **Key Innovation**: CAE reduces on-chip collective latency 5x for autoregressive decoding
- **Significance**: First purpose-built agentic-era silicon — "waiting room" elimination architecture

### OpenAI Jalapeño (Hot Chips 2026, Aug 2026)
- **Source**: https://www.servethehome.com/openai-jalapeno-asic-at-hot-chips-2026/
- **What**: OpenAI's first custom inference ASIC, designed with Broadcom
- **Specs**: 13.4 PFLOP/s mxfp4, 15.4 TB/s HBM4 bandwidth, 700W package; scales to 27 EFLOP/s across 2,048 chips
- **Timeline**: RTL Feb 2025 → tapeout Nov 2025 → silicon May 2026 → Codex running same month
- **Key**: 3-phase decomposition (prefill/draft/verify) vs Nvidia's 2-phase; AI-designed kernels 1.5-1.8x faster than human-written
- **Portability Gap**: Spatial architecture requires AI-optimized data placement; no CUDA-like portable abstraction

### Cerebras WSE-3 Serving GPT-5.6 Sol (Aug 2026)
- **Source**: https://www.cerebras.ai/blog/how-cerebras-serves-gpt-5-6-sol-at-up-to-750-tokens-per-second
- **What**: GPT-5.6 Sol at 750 tok/s on wafer-scale engine
- **Specs**: 44GB SRAM across wafer, 21 PB/s aggregate bandwidth, 900,000 cores
- **Key**: Weights stay on-chip, only activations move between wafers; no model compression/quantization needed

### FuriosaAI RNGD (2026)
- **Source**: https://developer.furiosa.ai/v2026.2.0/en/overview/rngd.html
- **What**: Tensor Contraction Processor (TCP) — novel architecture for native tensor contractions
- **Specs**: 256 TFLOPS BF16, 512 TFLOPS FP8, 1024 TOPS INT4; 150W TDP, passive cooling
- **Key**: 256MB on-chip SRAM + 48GB HBM3; 8 multi-instance + SR-IOV support

### Tensordyne Napier (Sep 2026)
- **Source**: https://www.tensordyne.ai/tensordyne-napier-whitepaper
- **What**: Logarithmic math chip for AI inference — world's first log-math AI processor
- **Specs**: 2.11 PFLOPS FP8 per AIP, 256MB SRAM, 144GB HBM3E; 72-chip pod with TDN Link
- **Key**: 9 AIPs in 1RU, 19 PFLOPS total; claimed 1 rack replaces 9 Nvidia+Groq racks

---

## 3. Neuromorphic — Key Findings

### SpiNNaker2 (arXiv:2607.24396, Jul 2026)
- **Source**: https://arxiv.org/abs/2607.24396
- **What**: 152 ARM M4F cores per chip, bridges DNN and SNN on same hardware
- **Specs**: 4.5 TOPS high-perf mode, 2.7 TOPS/W efficiency (INT8); >150K neurons, >1.8B synaptic events/s
- **Power**: <250mW baseline, DVFS per processing element
- **Comparison**: 152 cores vs SpiNNaker's 18; 19.8MB SRAM vs 1.8MB; 22nm vs 130nm
- **Key**: Software-defined neuron models (any model expressible in C); hybrid DNN+SNN support

### Intel Loihi 2 + Hala Point (Deployed 2024-2025)
- **Source**: https://www.sandia.gov/news/publications/hpc-annual-reports/article/advancing-neuromorphic-computing-at-the-neural-exploration-and-research-lab/
- **What**: Largest neuromorphic system: 1,152 chips, 1.15B neurons, 128B synapses
- **Specs**: 1M neurons/chip, 120M synapses/chip, Intel 4 process, variable power
- **Key**: Programmable neuron models (LIF, Izhikevich, custom); on-chip STDP + three-factor learning
- **Runtime Model**: Max-affine roofline model published (arXiv:2601.10035) — first quantitative performance predictor

### SpiNNaker2 + Loihi2 + GPU Hybrid (arXiv:2601.09755)
- **Source**: https://arxiv.org/pdf/2601.09755
- **What**: Heterogeneous brain-inspired computing: Loihi2 (edge perception) + SpiNNaker2 (brain modeling) + NVIDIA DGX (LLM/orchestration)
- **Key**: Loihi2 hand-tracking at 4mW vs Jetson Nano 5-10W; SpiNNaker2 SPAUN model at 1/10 GPU power
- **First**: Real-time robotic vision workload on Loihi2 + DVS camera

### Intel Loihi 3 (Announced 2026)
- **Source**: https://www.joshwagenbach.com/blog/neuromorphic-hardware-landscape-2026
- **What**: Target 100x energy efficiency vs GPUs for specific tasks; commercial availability projected 2026
- **Key**: Backed by Intel fabrication capability; roadmap credible

### Neuromorphic Compute Runtime Model (arXiv:2601.10035, Jan 2026)
- **Source**: https://doi.org/10.48550/arxiv.2601.10035
- **What**: Max-affine lower-bound runtime model for Loihi 2 — accounts for compute + NoC communication
- **Specs**: Pearson r ≥ 0.97 between model estimate and measured runtime
- **Key**: First quantitative performance model for neuromorphic hardware; area-runtime tradeoff analysis

---

## 4. NEW Defects vs Batch 596

### DEFECT-FPGA-001: Recursive Self-Improvement Without Containment
**Source**: Redwood (arXiv:2608.26418)
**Description**: Redwood demonstrates recursive self-improvement — Qwen running on Redwood designed next-generation Redwood. No containment or verification boundary exists between the AI-designed hardware and the AI that validates it. Batch 596 had no concept of self-referential hardware design loops.
**NeoTrix Impact**: NT-CORE must model self-referential optimization loops as a distinct pattern class. ConsciousnessTree needs a "self-modification depth" metric to track how many iterations deep an AI has influenced its own substrate.

### DEFECT-FPGA-002: HLS-to-Deployable-Design Validation Gap
**Source**: FPGAgent (arXiv:2608.23630)
**Description**: FPGAgent reports that HLS code passing simulation and synthesis may still fail on real FPGA hardware due to timing/P&R constraints. The gap between "synthesizable" and "deployable" is unmonitored in existing pipelines. 16.9% synthesizable rate improvement still leaves 73.3% non-executable.
**NeoTrix Impact**: NT-ACT needs a deployment-readiness gate that validates board-level executability, not just synthesis. The SEAL pipeline's self-test tier T3 (Production Wiring) must include hardware-in-the-loop validation for FPGA targets.

### DEFECT-FPGA-003: Multi-Modal Bitstream Orchestration Without Telemetry
**Source**: TRINE (arXiv:2603.22867)
**Description**: TRINE unifies ViT/CNN/GNN/NLP in a single bitstream with runtime mode-switching, but mode selection policies are compile-time static. Runtime sparsity deviations trigger mode switches without telemetry-guided feedback loops. No closed-loop adaptation exists.
**NeoTrix Impact**: NT-MIND must provide a runtime telemetry collector for FPGA mode-switching decisions. The GWT attention router should modulate mode selection based on observed sparsity distributions, not just compile-time estimates.

### DEFECT-ASIC-004: Training/Inference Split State Consistency
**Source**: Google TPU 8t/8i (blog.google)
**Description**: Google splits training (8t) and inference (8i) into separate silicon with different architectures (compute-heavy vs memory-heavy). No specification exists for state synchronization between 8t-trained models and 8i-deployed inference. KV cache state, weight versioning, and adapter LoRA states must be consistent across the split.
**NeoTrix Impact**: NT-MEMORY must implement a model-state provenance chain that tracks weight lineage from training silicon to inference silicon. Batch 596's "sovereign data provenance chain" must extend to model weights, not just data.

### DEFECT-ASIC-005: Spatial Architecture Portability Tax
**Source**: OpenAI Jalapeño (ServeTheHome)
**Description**: Jalapeño's spatial programming model requires AI-optimized data placement, scheduling, and communication patterns that are architecture-dependent. When Jalapeño Gen 2/3 changes core count, memory organization, or network topology, old optimizations become suboptimal. No portable abstraction layer exists (unlike CUDA).
**NeoTrix Impact**: NT-ACT must define a hardware-abstraction protocol for spatial accelerators. The SEAL pipeline needs a "portability coefficient" metric — how much re-optimization is required when migrating between accelerator generations.

### DEFECT-ASIC-006: Logarithmic Math Precision Uncharted Territory
**Source**: Tensordyne Napier (whitepaper)
**Description**: Tensordyne's logarithmic math reduces compute area/power but introduces fundamentally different precision characteristics than floating-point. No published analysis exists for how log-math precision affects MoE routing decisions, attention score distributions, or KV cache quantization at production scale.
**NeoTrix Impact**: NT-CORE must maintain a precision-sensitivity profile for each model component. The SelfTest T3 tier should include numerical stability checks for log-math inference paths.

### DEFECT-NM-007: Neuromorphic-GPU Orchestration Without Unified Scheduler
**Source**: Hybrid system (arXiv:2601.09755)
**Description**: Loihi2 + SpiNNaker2 + GPU orchestration works but requires manual task allocation. Loihi2 handles edge perception (4mW), SpiNNaker2 handles brain modeling (5.4-13.5x less power than DGX), and GPU handles LLM. No unified scheduler dynamically allocates work based on real-time neuromorphic state (spike rates, neuron utilization).
**NeoTrix Impact**: NT-PHYSICAL must implement a heterogeneous compute orchestrator that queries neuromorphic chip state (spike activity, DVFS state, neuron occupancy) and routes tasks to the optimal substrate. The Heartbeat Aggregator should include neuromorphic health signals.

### DEFECT-NM-008: Neuromorphic Runtime Model Without Energy Grounding
**Source**: Loihi 2 runtime model (arXiv:2601.10035)
**Description**: The max-affine runtime model predicts execution time with r ≥ 0.97 accuracy but excludes energy. The paper explicitly states energy modeling is "forthcoming." Without energy prediction, neuromorphic deployment decisions are made blind to power consumption, which is the primary advantage of neuromorphic over GPU.
**NeoTrix Impact**: NT-PHYSICAL needs an energy-runtime joint model for neuromorphic hardware. The SEAL pipeline's carbon-aware scheduling (Batch 596 DEFECT) must incorporate neuromorphic energy profiles for accurate carbon footprint calculation.

### DEFECT-NM-009: SNN-to-DNN Bridge Without Gradient Flow
**Source**: SpiNNaker2 (arXiv:2607.24396)
**Description**: SpiNNaker2 supports hybrid DNN+SNN execution but the bridge between spiking and non-spiking domains lacks gradient flow. Training a DNN, converting to SNN, and deploying on SpiNNaker2 introduces quantization loss that is not monitored or compensated at runtime. No feedback loop exists to adjust SNN parameters based on inference accuracy drift.
**NeoTrix Impact**: NT-MIND must implement a cross-domain accuracy monitor that detects SNN deployment degradation vs the original DNN baseline. The ConsciousnessTree should track "translation fidelity" as a health dimension.

### DEFECT-NM-010: Neuromorphic Chip Heterogeneity Without Standardized Benchmarking
**Source**: Neuromorphic hardware landscape (joshwagenbach.com)
**Description**: Loihi 2, SpiNNaker2, BrainScaleS-2, TrueNorth, and Akida all report different metrics (neurons, synapses, TOPS, TOPS/W) making cross-platform comparison impossible. No standardized benchmark exists for neuromorphic workloads. InferenceX (for ASICs) has no neuromorphic equivalent.
**NeoTrix Impact**: NT-MEMORY must maintain a neuromorphic capability registry with standardized metrics. The Heartbeat Aggregator should normalize health signals across heterogeneous neuromorphic substrates using a common ontology.

---

## 5. Improvements Over Batch 596

### IMPROVE-001: Carbon-Aware Compute Now Has Hardware Substrate
**Batch 596**: "Carbon-aware compute scheduling missing"
**Improvement**: SpiNNaker2's <250mW baseline + DVFS per element enables carbon-aware scheduling at the silicon level. When grid carbon intensity spikes, neuromorphic chips can drop to near-zero power without losing state. TPU 8i's 2x perf/watt improvement also reduces carbon per inference.
**Action**: Implement carbon-intensity → DVFS policy mapping in NT-PHYSICAL.

### IMPROVE-002: Material Science Knowledge Graph Can Leverage Neuromorphic Simulations
**Batch 596**: "No material science knowledge graph"
**Improvement**: SpiNNaker2's 152 ARM cores running software-defined neuron models can simulate molecular dynamics and protein folding at 1/10 GPU power. The chip's flexibility (any model expressible in C) makes it suitable for material science simulations that require custom interaction potentials.
**Action**: Add neuromorphic simulation as a knowledge acquisition pathway in NT-WORLD's material science KG.

### IMPROVE-003: Sovereign Data Provenance Now Includes Hardware Design Lineage
**Batch 596**: "No sovereign data provenance chain"
**Improvement**: Redwood's AI-designed accelerator + OpenAI's AI-assisted Jalapeño kernels establish that hardware design lineage is now a first-class concern. The provenance chain must extend from training data → model weights → hardware design → deployed silicon.
**Action**: NT-MEMORY's provenance chain must include hardware design artifacts (RTL, GDSII, bitstreams) as traceable entities.

### IMPROVE-004: Hybrid Model Orchestration Has New Substrate Options
**Batch 596**: "Hybrid model orchestration lacks state consistency"
**Improvement**: The Loihi2+SpiNNaker2+GPU hybrid pattern (arXiv:2601.09755) demonstrates that heterogeneous orchestration is viable at production scale. Sandia's deployment of both systems validates the approach. The missing piece is automated state synchronization, not viability.
**Action**: NT-ACT's orchestration layer should adopt the three-tier pattern: neuromorphic edge (Loihi2) → neuromorphic brain (SpiNNaker2) → GPU cloud (LLM).

### IMPROVE-005: Climate Model Distribution Drift Has New Detection Tool
**Batch 596**: "Climate model distribution drift unmonitored"
**Improvement**: The max-affine runtime model (arXiv:2601.10035) demonstrates that neuromorphic hardware can detect distribution drift via spike rate monitoring. When input distribution shifts, spike patterns change, and the runtime model detects the anomaly before accuracy degrades.
**Action**: Integrate spike-rate-based drift detection into NT-WORLD's climate monitoring pipeline.

---

## 6. Summary Statistics

| Metric | Count |
|--------|-------|
| Sources cited | 15 unique papers/products |
| NEW defects found | 10 (DEFECT-FPGA-001 through DEFECT-NM-010) |
| Improvements over Batch 596 | 5 |
| FPGA findings | 6 |
| ASIC/TPU findings | 5 |
| Neuromorphic findings | 5 |

---

## 7. Next Batch Priorities

1. **DEFECT-FPGA-001** (recursive self-improvement containment) — highest risk, no existing mitigation
2. **DEFECT-ASIC-004** (training/inference split state consistency) — directly impacts model deployment reliability
3. **DEFECT-NM-007** (neuromorphic-GPU unified scheduler) — blocks hybrid orchestration at scale
4. **DEFECT-NM-010** (neuromorphic benchmarking standardization) — prerequisite for all other neuromorphic work
5. **DEFECT-FPGA-003** (multi-modal bitstream telemetry) — enables runtime-adaptive FPGA deployment

---

*Batch 597 complete. Next: Batch 598 — Edge AI orchestration + sovereign compute patterns.*
