# Iteration Batch 347 — Hardware Architecture Research (2026-09-06)

## Sources Cited

| # | Source | Date | Key Signal |
|---|--------|------|------------|
| S1 | [Promwad: Embedded Systems Trends 2026](https://promwad.com/news/embedded-systems-trends-2026-chiplets-risc-v-edge-ai) | 2026-01-05 | Chiplet-based embedded architectures go mainstream; RISC-V moves from curiosity to core architecture; edge generative AI arrives on embedded hardware |
| S2 | [arXiv:2509.18355v5 — Chiplet-Based RISC-V SoC with Modular AI Acceleration](https://arxiv.org/abs/2509.18355v5) | 2026-04-07 | 4 innovations: adaptive cross-chiplet DVFS, AI-aware UCIe protocol extensions with streaming flow control units, distributed cryptographic security across heterogeneous chiplets, intelligent sensor-driven load migration |
| S3 | [PatSnap: Near Memory Computing PIM/CIM/LIM 2026](https://www.patsnap.com/resources/blog/articles/near-memory-computing-pim-cim-and-lim-in-2026/) | 2026-04-23 | Von Neumann bottleneck now a crisis; 60%+ energy wasted on data movement; CIM/PIM market $1.16B (2026) → $6.54B (2031); Samsung LPDDR5X-PIM at 614 GB/s internal bandwidth |
| S4 | [PatSnap: PIM Architecture Landscape 2026](https://www.patsnap.com/resources/blog/articles/pim-architecture-landscape-tech-clusters-and-ip-for-2026/) | 2026-04-23 | RISC-V as standard PIM instruction framework; chiplet-based PIM integration via 2.5D/3D packaging; near-storage processing as lower-risk insertion point |
| S5 | [Chips & Cheese: Samsung PIM at Hot Chips 2026](https://chipsandcheese.com/p/hot-chips-2026-samsungs-processing) | 2026-08-29 | Samsung LPDDR5X-PIM MAC units per DRAM bank; 614 GB/s internal bandwidth; mode switches and cache rules complicate adoption |
| S6 | [IPValueLabs: ASIC vs FPGA for Edge AI Inference 2026](https://ipvaluelabs.com/insights/asic-vs-fpga-edge-ai-inference) | 2026-03-19 | Axelera Metis 214 TOPS/15 TOPS/W; RISC-V as control plane for AI accelerators; in-memory computing goes mainstream; heterogeneous compute (CPU+GPU+NPU+FPGA) standard |
| S7 | [FPGACenter: Are FPGAs Still Relevant for AI in 2026?](https://fpgacenter.com/blog/fpga-vs-gpu-asic-edge-ai) | 2026-08-29 | FPGAs strongest for deterministic latency, sensor-side processing, custom I/O, functional safety; AMD Versal AI Edge Gen 2; Intel Agilex 5 with native AI Tensor blocks |
| S8 | [Mordor Intelligence: CIM/PIM Market Report](https://www.mordorintelligence.com/industry-reports/compute-in-memory-cim-and-processing-in-memory-pim-market) | 2026-07-22 | CIM/PIM $6.54B by 2031 (41.33% CAGR); HBM4E 3.6 TB/s per stack (Samsung May 2026); BrainChip Akida AKD1500 commercial shipments; Mythic acquires Videantis for hybrid CIM |
| S9 | [GPUAdvisor: GPU Roadmap 2026-2028](https://gpuadvisor.com/gpu-roadmap) | 2026 | NVIDIA B300 Ultra: 7000 FP8 TFLOPS, 288GB HBM3e, 10 TB/s bandwidth; AMD MI350X: CDNA4, 4600 FP8 TFLOPS; AMD RDNA5 unified architecture with Neural Arrays |
| S10 | [TechAnnouncer: AMD GPU Roadmap 2026](https://techannouncer.com/amd-gpu-roadmap-2026-what-to-expect-from-next-gen-radeon-graphics) | 2026-04-13 | AMD RDNA5: clean-slate design, unified RDNA+CDNA, Radiance Cores (RT), Neural Arrays (AI rendering), Universal Compression; chiplet designs with CoWoS-L |
| S11 | [Barrack.ai: NVIDIA Rubin at GTC 2026](https://blog.barrack.ai/nvidia-rubin-specs-architecture-2026/) | 2026-03-14 | Rubin: 336B transistors, HBM4, 22 TB/s bandwidth, 10x inference cost reduction over Blackwell |
| S12 | [SiliconToSoftware: Edge AI Chips 2026](https://www.silicontosoftware.com/edge-ai-chips-cloud-intelligence/) | 2026-09-04 | Burst vs sustained performance is defining tradeoff; LPDDR6 emerging; thermal engineering is the real edge compute problem |
| S13 | [RapidCircuitry: TinyML MCU Hardware Guide 2026](https://www.rapidcircuitry.com/blogs/tinyml-mcu-hardware-guide-2026-npus-memory-power) | 2026-05-18 | TinyML MCUs with dedicated NN accelerators; single-digit microjoule per inference; always-on wake-on-event patterns; duty cycling standard |
| S14 | [AIMultiple: Top 15 Edge AI Chip Makers 2026](https://aimultiple.com/edge-ai-chips) | 2026-06-04 | NVIDIA Jetson AGX Orin 275 TOPS/15-60W; Hailo-10H 40 TOPS/5W for generative AI; Kneron KL730 7 TOPS/0.5-2W |
| S15 | [arXiv:2511.21232 — RISC-V TinyML Accelerator for Depthwise Separable Convolutions](https://arxiv.org/html/2511.21232v1) | 2025-11-26 | CFU Playground: RISC-V custom function units for tight HW/SW co-design; fused streaming architecture eliminates intermediate buffers |
| S16 | [Semiconductor Insight: RISC-V Multi-Core AI Accelerator Chiplet Market](https://semiconductorinsight.com/report/risc-v-multi-core-ai-accelerator-chiplet-architecture-market/) | 2026-06-18 | Market $0.73B (2026) → $2.15B (2034), 12.3% CAGR; SiFive+Nvidia RISC-V GPU integration; Alibaba Xuantie AI cores |
| S17 | [Luca Berton: RISC-V AI Accelerators & Open Silicon 2026](https://lucaberton.com/blog/risc-v-ai-accelerators-open-silicon-2026/) | 2026-06-11 | RISC-V as control plane of AI accelerators; RISC-V Summit Europe 2026; vector extensions for DSP/AI |
| S18 | [Semiconductor Insight: TinyML Chip Market 2026](https://semiconductorinsight.com/report/tinyml-chip-market) | 2026-06-01 | Market $0.78B (2026) → $3.12B (2034), 16.8% CAGR; ARM Cortex-M + custom ASICs; sub-milliwatt inference |
| S19 | [Emergentmind: PIM Architectures](https://www.emergentmind.com/topics/processing-in-memory-pim-architectures) | 2026-01-29 | AttAcc PIM Backend for Transformer Inference; near-bank PIM architectures; heterogeneous chiplet PIM |
| S20 | [youngju.dev: Edge AI & TinyML 2026 Deep Dive](https://www.youngju.dev/blog/culture/2026-05-16-edge-ai-tinyml-2026-litert-executorch-edge-impulse-jetson-coral-hailo-sipeed-k230-llama-cpp-deep-dive.en) | 2026-05-16 | ExecuTorch, LiteRT as deployment frameworks; Coral Edge TPU now legacy (2018 silicon); Hailo/Sipeed/Rockchip taking share |

## Defects Found

### DEFECT-347-01: Missing Hardware Abstraction Layer (HAL) in Embodiment Layer

**Research signal**: Chiplet architectures (S1, S2) with UCIe interconnects, heterogeneous compute (CPU+NPU+FPGA+PIM) now standard for edge AI. Samsung LPDDR5X-PIM (S5) places MAC units per DRAM bank. Market projections: CIM/PIM $6.54B by 2031 (S8).

**Current architecture**: `EmbodimentLayer` trait (`neotrix-core/src/unified/layers/embodiment/traits.rs:130`) only models pentest-oriented state (hosts, vulnerabilities, credentials, access levels). Zero abstraction for:
- Compute fabric topology (chiplet count, interconnect bandwidth, die-to-die latency)
- Memory hierarchy (PIM vs conventional, HBM vs LPDDR, bandwidth-per-watt)
- Hardware accelerator inventory (TOPS/W per device, supported precisions INT4/INT8/FP8)
- Power domain partitioning (per-chiplet DVFS, thermal envelope, sustained vs burst TOPS)

**Impact**: NeoTrix cannot reason about its own hardware substrate. The ConsciousnessTree (D13-D16 meta-cognition) cannot detect memory wall bottlenecks or evaluate PIM insertion opportunities. GWT attention routing cannot factor hardware topology into task placement.

**Suggestion**: Add `HardwareSubstrate` struct to `EmbodimentLayer` with fields: `chiplets: Vec<ChipletInfo>`, `memory_hierarchy: MemoryHierarchy`, `accelerator_inventory: Vec<AcceleratorProfile>`, `power_domains: Vec<PowerDomain>`. Add trait method `fn hardware_topology(&self) -> &HardwareSubstrate`. Wire into `HeartbeatAggregator` for hardware health signals.

---

### DEFECT-347-02: No Memory Wall / PIM Strategy in Knowledge Architecture

**Research signal**: Von Neumann bottleneck is THE defining constraint in 2026 — 60%+ energy wasted on CPU-DRAM data movement (S3, S4). Samsung PIM achieves 614 GB/s internal bandwidth by placing MAC units inside DRAM (S5). HBM4E at 3.6 TB/s per stack (S8). RISC-V is becoming the standard PIM ISA extension framework (S4).

**Current architecture**: KB is "SQLite-backed persistent store" with "nodes, edges, embeddings, BM25 index" (CONTEXT.md). No consideration of:
- Near-storage processing (Harvard RecSSD: 2x latency reduction for recommendation inference — S3)
- Compute-in-memory for VSA HyperCube vector operations (embeddings are the most memory-bound workload)
- Memory-side compute for GWT attention broadcasting (broadcasting = memory bandwidth)

**Impact**: KB embedding operations (`nt_core_vector_store`) will hit the memory wall as knowledge base grows. VSA HyperCube associative recall (the core reasoning engine) is bandwidth-bound. No path to exploit PIM for the most data-intensive operations.

**Suggestion**: Add `nt_memory_pim` module to NT-MEMORY domain:
1. Abstract PIM/CIM interface for vector similarity search (exact match for HyperCube recall)
2. Near-storage processing for FTS5 BM25 queries (embedding-indexed retrieval)
3. RISC-V PIM ISA extension trait for custom in-memory compute instructions
4. Memory bandwidth telemetry to `HeartbeatAggregator` for memory wall detection

---

### DEFECT-347-03: No Heterogeneous Compute Dispatch Layer

**Research signal**: 2026 edge AI is heterogeneous CPU+GPU+NPU+FPGA+PIM (S6, S7). FPGAs win on deterministic latency and sensor fusion (S7). ASICs win on throughput/watt (Axelera Metis 214 TOPS/15W — S6). The dispatch problem is now the binding engineering challenge (S6, S20).

**Current architecture**: `nt_core_scheduler` handles CPU thread scheduling. `ParallelTaskManager` manages GPU VRAM and batch scheduling (CONTEXT.md). No abstraction for:
- Multi-device workload partitioning (which layers run on which accelerator?)
- Runtime dispatch across CPU/GPU/NPU/FPGA/PIM based on model topology
- Mixed-precision routing (INT4 on NPU, INT8 on FPGA, FP16 on GPU)
- Sensor-to-AI-to-control pipeline (FPGA for preprocessing → NPU for inference → CPU for postprocessing)

**Impact**: NeoTrix cannot orchestrate inference across heterogeneous hardware. The SEAL pipeline runs on CPU only. Video post-processing (`nt_physical::video_post_processor`) cannot exploit GPU/NPU acceleration. The ParallelTaskManager assumes homogeneous GPU fleet.

**Suggestion**: Add `HeterogeneousDispatch` to NT-ACT or a new `nt_act_compute_orchestrator`:
1. `AcceleratorProfile` enum: CPU(chip), GPU(model, VRAM), NPU(TOPS, precision), FPGA(TOPS, reconfigurable), PIM(bandwidth, MAC count)
2. `WorkloadPartitioner`: given model graph + available accelerators, produce per-layer device assignment
3. `DispatchRuntime`: execute partitioned workload with device-to-device data transfer scheduling
4. Wire into SEAL pipeline for hardware-aware self-evolution (evolution adapts to available hardware)

---

### DEFECT-347-04: No Chiplet Awareness in Module Topology

**Research signal**: Chiplet-based SoCs with adaptive cross-chiplet DVFS, AI-aware UCIe streaming flow control, and compression-aware transfers (S2). Chiplet market for RISC-V AI accelerators: $0.73B in 2026 (S16). Modular compute tiles that evolve independently (S1).

**Current architecture**: The Six-Layer Architecture models software modules (`nt_*` domains) with trait-based interfaces. No concept of:
- Physical die boundaries (which modules run on which chiplet?)
- Die-to-die interconnect bandwidth as a constraint (UCIe protocol)
- Chiplet-level DVFS (adaptive voltage/frequency per compute tile)
- Heterogeneous process nodes (5nm AI engine + 12nm I/O tile — S6)

**Impact**: NeoTrix's module topology is purely logical. When deployed on chiplet-based hardware, it cannot reason about inter-chiplet communication costs, optimize module placement for data locality, or exploit chiplet-level power management.

**Suggestion**: Add `ChipletTopology` to `HardwareSubstrate`:
1. `ChipletInfo { id, process_node, compute_types, interconnect_protocol, bandwidth_gb_s }`
2. `ModulePlacement` mapping: which `nt_*` modules are pinned to which chiplets
3. `InterChipletCost` model: latency/bandwidth penalties for cross-chiplet calls
4. Integrate with GWT attention routing: prefer same-chiplet module co-activation

---

### DEFECT-347-05: No Thermal/Power Budget Abstraction

**Research signal**: Burst vs sustained performance is the defining edge AI tradeoff (S12). Edge chips run LPDDR5/5X/LPDDR6 with thermal constraints (S12). Power gating per chiplet is standard (S1). TinyML targets sub-milliwatt always-on patterns (S13, S18). NPUs in consumer SoCs: 45-80+ TOPS but bounded by thermal envelope (S6).

**Current architecture**: `nt_physical::health_checker` exists but models pentest health (host availability, vuln scan status). No abstraction for:
- Thermal envelope per compute domain (TDP, junction temperature, throttling curve)
- Power budget allocation across accelerators (NPU vs GPU vs PIM share of power)
- Sustained TOPS vs burst TOPS distinction (critical for edge deployment)
- Duty cycling for always-on inference (TinyML wake-on-event patterns — S13)

**Impact**: NeoTrix cannot reason about its own power consumption or thermal limits. The HeartbeatAggregator tracks software health but not hardware thermal state. Self-healing (NT-REPAIR) cannot detect thermal throttling as a degradation signal.

**Suggestion**: Add `PowerThermalModel` to `HardwareSubstrate`:
1. `ThermalZone { id, current_temp, tdp_watts, throttle_curve }`
2. `PowerBudget { total_watts, per_domain_allocation, duty_cycle }`
3. Wire into `HeartbeatAggregator` as `thermal_health` signal
4. Wire into SEAL pipeline: evolution avoids hardware-damaging configurations

---

### DEFECT-347-06: No RISC-V ISA Extension Awareness

**Research signal**: RISC-V is becoming the control plane of AI accelerators (S17). RISC-V vector extensions enable real-time AI workloads (S1). Custom ISA extensions for PIM (RISC-Vlim framework — S4). SiFive P870/X280 cores paired with custom NN accelerators (S6). CFU Playground enables RISC-V custom function units for tight HW/SW co-design (S15).

**Current architecture**: NT-CORE models E8 reasoning and GWT attention as pure software abstractions. No concept of:
- RISC-V vector ISA extensions for VSA HyperCube vector operations
- Custom function units (CFUs) for domain-specific acceleration
- PIM ISA extensions for in-memory compute
- ISA-level privilege modes for security (relevant to NT-SHIELD)

**Impact**: NeoTrix cannot exploit RISC-V custom extensions when deployed on RISC-V silicon. VSA HyperCube vector operations could be 10-100x faster with RISC-V V-extension but the architecture has no path to exploit this. The open-source RISC-V ecosystem (S16) is the natural substrate for NeoTrix but no ISA-aware layer exists.

**Suggestion**: Add `RiscVExtensionAwareness` to NT-CORE or a new `nt_core_isa`:
1. `IsaProfile { base_isa, vector_extensions, custom_cfus, pim_extensions }`
2. `VectorizableOps` trait: mark which NeoTrix operations can map to RISC-V V-extension
3. `CfuRegistry`: register custom function units for domain-specific ops (VSA multiply, GWT broadcast)
4. Compile-time feature flags: `#[cfg(target_feature = "v")]` for vector-optimized paths

---

### DEFECT-347-07: No Near-Storage Processing for Knowledge Base

**Research signal**: Harvard RecSSD: 2x latency reduction for recommendation inference embedding tables via near-storage processing (S3). Near-storage processing is a lower-risk insertion point than full CIM (S3, S4). Computational Storage / In-Storage Computing emerging as third PIM sub-domain (S4).

**Current architecture**: KB is SQLite on local disk (CONTEXT.md). Embedding vectors stored in SQLite BLOBs. No consideration of:
- Near-storage processing for embedding similarity search (the most KB-intensive operation)
- Computational SSDs that can filter/rank embeddings without CPU round-trips
- HBM-backed KB for datacenter deployments (Samsung HBM4E — S8)
- Split computing: embed on storage, rank on CPU, reason on NPU

**Impact**: As KB grows, embedding search becomes I/O bound. Each VSA HyperCube lookup requires reading embedding vectors from SSD → RAM → CPU cache. Near-storage processing could eliminate the SSD→RAM hop entirely.

**Suggestion**: Add `nt_memory_near_storage` module:
1. `NearStorageCompute` trait: abstract computational storage operations
2. `EmbeddingIndexOffload`: push HNSW/IVF index to computational SSD
3. `SplitComputePipeline`: storage-side coarse filter → CPU-side fine rank → NPU-side reasoning
4. Hardware capability detection: probe for computational SSD support at startup

---

### DEFECT-347-08: Dual Specialization Missing Hardware Context

**Research signal**: Heterogeneous compute (CPU+GPU+NPU+FPGA) is the standard 2026 deployment model (S6, S7). Each accelerator class has different strengths: FPGAs for deterministic latency/sensor fusion (S7), ASICs for throughput/watt (S6), GPUs for flexibility (S9). Hardware-software co-design via NAS achieves 4.4x EDP reduction (S6 — NAAS).

**Current architecture**: Dual Specialization routes between CORE+WORLD (acquisition) and CORE+MIND (evolution) software modes. No hardware context in routing decisions:
- Acquisition mode might prefer FPGA for sensor preprocessing + NPU for inference
- Evolution mode might prefer GPU for training-like distillation + PIM for embedding updates
- No runtime hardware capability detection to adjust routing

**Impact**: The AttentionManager cannot optimize for actual hardware availability. If running on a PIM-enabled system, it should prefer memory-intensive operations. If running on an FPGA-rich system, it should prefer low-latency sensor processing.

**Suggestion**: Extend `AttentionManager` with `HardwareContext`:
1. `HardwareProfile`: current device capabilities, thermal state, power budget
2. `RouteOptimizer`: select software route (CORE+WORLD vs CORE+MIND) based on hardware strengths
3. `FallbackChain`: if preferred accelerator unavailable, degrade gracefully to next-best device
4. Wire into Dual Specialization: Weapon Set switching includes hardware-aware dispatch

---

### DEFECT-347-09: No Edge Deployment / TinyML Abstraction

**Research signal**: TinyML market $0.78B in 2026 → $3.12B by 2034 (S18). Sub-microjoule inference on MCUs (S13). Always-on wake-on-event patterns standard (S13). RISC-V TinyML accelerators with fused streaming architectures (S15). ExecuTorch/LiteRT as deployment frameworks (S20).

**Current architecture**: NeoTrix is a heavyweight Rust application. No abstraction for:
- Model quantization pipeline (INT4/INT8/INT2 for TinyML)
- MCU-class deployment (sub-100KB flash, sub-10KB RAM — S13)
- Duty-cycled inference (always-on keyword spotting, wake-on-event vision)
- Energy harvesting constraints (solar/kinetic powered AI sensors)

**Impact**: NeoTrix cannot deploy its reasoning capabilities to ultra-low-power edge devices. The NT-PHYSICAL domain has no path to constrain itself to milliwatt budgets. The SEAL pipeline has no "tiny" evolution mode.

**Suggestion**: Add `nt_physical_tinymode` module:
1. `TinyModeProfile { max_flash_kb, max_ram_kb, max_power_mw, duty_cycle }`
2. `ModelDistiller`: distill large NeoTrix models to MCU-compatible sizes
3. `WakeOnEvent`: event-driven inference activation (sensor trigger → NPU wake → inference → sleep)
4. Feature flag: `#[cfg(feature = "tinymode")]` to compile without heavyweight dependencies

---

### DEFECT-347-10: No RDNA5/Neural Array Awareness for Visual Processing

**Research signal**: AMD RDNA5 introduces Neural Arrays (CU collections for neural rendering), Radiance Cores (dedicated RT hardware), and Universal Compression (bandwidth reduction — S10). NVIDIA Rubin: 22 TB/s HBM4 bandwidth, 10x inference cost reduction (S11). Video post-processing at hardware level is standard (S7 — FPGA preprocessing pipeline).

**Current architecture**: `nt_physical::video_post_processor` exists for frame-level processing (color alignment, temporal stabilization, super-resolution). But:
- No abstraction for GPU-accelerated video pipelines (CUDA/Metal/Vulkan compute shaders)
- No awareness of Neural Array / Tensor Core hardware for inference-heavy video ops
- No bandwidth-aware scheduling for 4K/8K video (22 TB/s bandwidth on Rubin, but still constrained)
- Universal Compression concept missing: no data reduction before GPU→CPU transfer

**Impact**: Video post-processing runs on CPU, missing 10-100x acceleration available on GPU/Neural Array hardware. The `VideoPostProcessor` cannot exploit RDNA5's Universal Compression to reduce memory traffic.

**Suggestion**: Add hardware-accelerated video pipeline to `nt_physical`:
1. `GpuVideoPipeline`: dispatch video ops to GPU via wgpu/vulkan compute shaders
2. `NeuralArrayAware`: detect AMD RDNA5 Neural Array availability, use for inference-heavy ops
3. `CompressionPrePass`: apply Universal Compression concept before data transfer
4. Wire into `ResourceBudgetManager`: GPU memory budget for video workloads
