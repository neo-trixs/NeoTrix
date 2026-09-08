# Iteration Batch #461 — Hardware Architecture Research Loop

> **Date**: 2026-09-06 | **Sources**: 15+ | **Defects Found**: 8 | **Suggestions**: 12

---

## 1. Research Sources

### CPU Architecture (2026)

| Source | Title | Date | Key Insight |
|--------|-------|------|-------------|
| [ServeTheHome](https://www.servethehome.com/arms-agi-data-center-cpu-at-hot-chips-2026/) | Arm AGI Data Center CPU at Hot Chips 2026 | 2026-08-24 | Dual-chiplet 136-core Neoverse V3, 2MB L2/core, CMN-S3 mesh, 2TB/s UCIe D2D, 12ch DDR5-8800, 300W |
| [CNX Software](https://www.cnx-software.com/2026/08/20/starfive-dubhe-100/) | StarFive Dubhe-100 RISC-V Core | 2026-08-20 | RVA23-compliant, 15-stage pipeline, 6-wide issue, 512-entry ROB, TAGE branch predictor, SPECint 15/GHz |
| [ServeTheHome](https://www.servethehome.com/fujitsus-arm-based-monaka-data-center-cpu-at-hot-chips-2026/) | Fujitsu Monaka at Hot Chips 2026 | 2026-08-24 | 2nm core die + 5nm SRAM/IO dies, 3D hybrid bonding, 144 cores, 256-bit SVE2, ultra-low voltage, LLC on separate 5nm tile |
| [Arm Docs](https://support.arm.com/documentation/109697/2026_06/) | Armv9.6 Architecture Extension | 2026-06 | FEAT_SME2p2, FEAT_MPAM_PE_BW_CTRL (bandwidth controls), FEAT_PCDPHINT (producer-consumer data placement hints), FEAT_RME_GDI |
| [Ranzware](https://ranzware.com/hot-chips-2026-arm-details-agi-server-cpu-with-two-70-core-n3p-chiplets-touts-2-tbs-ucie-fabric-link-and-12-channel-memory-controller) | Hot Chips 2026: Arm AGI Deep Dive | 2026-08-26 | 50B transistors/chiplet, snoop filtering, hierarchical caching (HN-S), NUMA-1/2 modes, programmable page policies, MPAM bandwidth partitioning |
| [arXiv:2608.28097](https://arxiv.org/abs/2608.28097) | Great Expectations: RVV 1.0 in HPC | 2026-08-28 | RVV 1.0 delivers significant improvement over scalar; hardware-specific implementation challenges remain for RISC-V HPC viability |

### Memory Hierarchy & Coherence (2026)

| Source | Title | Date | Key Insight |
|--------|-------|------|-------------|
| [ISPASS 2026](https://doi.org/10.1109/ispass69572.2026.00020) | Coherent Data Movement in Scaled-Out Shared Memory | 2026-04 | Coherence overheads dominated by small subset of high-impact pages; sparse but dominant tail of shared data drives cost |
| [arXiv:2608.05965](https://arxiv.org/html/2608.05965) | ShimGen: Automated Hierarchical Coherence Protocol Synthesis | 2026-08 | First synthesis tool supporting non-SWMR global protocols + scoped operations; CXL/CHI integration |
| [ISCA 2026](https://iacoma.cs.uiuc.edu/iacoma-papers/isca26_1.pdf) | Dorado: Clustered Hardware Cache Coherence | 2026 | Two-Level Homes, Dynamic Apportioning, SetOverflow for 1024+ cores; 1.36x speedup over limited-pointer, 46.1% load latency reduction |
| [arXiv:2607.19922](https://arxiv.org/html/2607.19922v1) | DGNA: GPU NUMA Architecture | 2026-07 | First paper to detail GPU L2/DRAM NUMA architecture; sub-NUMA nodes, SM-NUMA relationships, coherent write mechanisms |
| [PLDI 2026](https://users.cs.utah.edu/~vijay/papers/pldi26.pdf) | Formally Verified Foundation for Compositional Heterogeneous Coherence | 2026 | 26K-line mechanized proof: Synchronous Propagation + SWMR global interconnect → Compound Memory Consistency Model (CMCM) |
| [SOSP 2026](https://csyhua.github.io/csyhua/hua-sosp26-XTRA.pdf) | XTRA: Cache Coherence + Concurrency Control for CXL Pods | 2026 | Unifies coherence with OCC; lazy invalidation via version validation; CXL 3.0 HCC for coordination directory |
| [OSDI 2026](https://www.usenix.org/system/files/osdi26-yang-zhijun.pdf) | FORGE: Disaggregated Memory Caching | 2026 | Group-level synchronization amortizes overhead; FIFO-based lazy hotness sync; 4.5x throughput improvement |

### AI Accelerators (2026)

| Source | Title | Date | Key Insight |
|--------|-------|------|-------------|
| [Google Blog](https://blog.google/innovation-and-ai/infrastructure-and-cloud/google-cloud/eighth-generation-tpu-agentic-era/) | Google TPU 8t and TPU 8i | 2026-04 | First split training/inference chips; TPU 8t: 121 EFLOPS FP4, 9600 chips/superpod; TPU 8i: 384MB SRAM, 288GB HBM, 7-hop BoardFly topology |
| [i-scoop.eu](https://www.i-scoop.eu/google-tpu-8t-8i/) | Inside Google's TPU 8t and TPU 8i | 2026-04 | Virgo Network: 134K TPUs, 47Pb/s; TPUDirect Storage bypasses host CPU; Axion ARM hosts; FP4 native; CAE (Collectives Acceleration Engine) |
| [ServeTheHome](https://www.servethehome.com/googles-tpuv8s-for-training-and-inference-at-hot-chips-2026/) | Google TPUv8s at Hot Chips 2026 | 2026-08 | BoardFly fly topology (max 7 hops vs 16 for 3D Torus); collective ops in ICI IO die (5x latency reduction); FP4 doubles throughput; in-field unit testing |
| [IEEE Micro](https://arxiv.org/pdf/2606.15870) | Google TPU v2 to Ironwood: 5 Generations | 2026-06 | 100x node perf growth, 3600x supercomputer scaling; Megacore abstraction; architectural stability; HBM 10x scaling; optical circuit switches |
| [LavX News](https://news.lavx.hu/article/google-and-amd-may-explore-next-generation-tpu-with-on-package-cpu-cores) | Google + AMD TPU Collaboration | 2026-08-17 | Reported 10th-gen TPU with on-package AMD CPU cores for RL; CPU+accelerator hybrid packaging |
| [MediaTek](https://www.mediatek.com/tek-talk-blogs/demystifying-custom-silicon-in-the-data-center) | Custom Silicon in the Data Center | 2026-04 | XPU as architectural umbrella; heterogeneous chiplet integration; custom ASICs combine CPU+NPU+DSP+FPGA in shared package |

---

## 2. Defects Identified in NeoTrix Design

### DEFECT-461-1: No Coherence-Aware Module Isolation Model

**Severity**: HIGH | **Domain**: NT-CORE + NT-PHYSICAL

**Evidence from Research**: 
- ShimGen (arXiv:2608.05965) proves that heterogeneous systems (CPU+GPU+accelerator) require formal coherence protocol composition. Compound Memory Consistency Model (CMCM) is now the standard for correctness.
- PLDI 2026 paper provides 26K-line mechanized proof that Synchronous Propagation over SWMR interconnect guarantees CMCM.
- NeoTrix's Rust borrow checker enforces single-writer at compile time, but this only applies within a single compilation unit. Cross-domain modules (e.g., NT-CORE calling NT-ACT tools, NT-WORLD calling NT-MEMORY KB) communicate via message passing without formal coherence guarantees.

**Gap in Design Doc**: 
- `CONTEXT.md` defines module isolation via borrow checker + single-writer principle (`CONTEXT.md:190`), but this is a Rust-level guarantee, not an architectural coherence model.
- The Six-Layer Architecture (`CONTEXT.md:62`) defines `traits.rs` interface contracts per layer, but there is no specification of what coherence guarantees these contracts provide across layers.
- When NT-CORE's `SelectiveState` (L5) reads from NT-MEMORY's KB (L1), what consistency model governs that read? The design doc does not specify.

**Suggestion**: 
1. Define a **Cross-Domain Coherence Contract (CDCC)** in `CONTEXT.md` under Architecture Patterns, specifying per-layer-pair consistency guarantees (e.g., L5→L1 reads are eventually consistent, L1→L4 writes are linearizable).
2. Implement a `CoherenceSpec` trait in `core/traits.rs` that each layer's `traits.rs` must implement, declaring its write-read guarantees.
3. Model this after the Compound Memory Consistency Model from PLDI 2026: each NT-* domain has an internal "MCM", and cross-domain interactions compose via formal rules.

---

### DEFECT-461-2: Missing Chiplet/Disaggregated Compute Topology Abstraction

**Severity**: HIGH | **Domain**: NT-CORE + NT-PHYSICAL

**Evidence from Research**:
- Arm AGI (Hot Chips 2026): Dual-chiplet design with 2TB/s UCIe D2D link; each chiplet is a self-contained SoC with compute+memory+IO. NUMA-1 vs NUMA-2 modes trade latency for capacity.
- Fujitsu Monaka: 3D-stacked chiplets (2nm core die + 5nm SRAM/IO dies) with hybrid bonding. LLC on separate 5nm tile — the most aggressive disaggregation yet.
- Google TPU 8t/8i: Two distinct chiplet architectures per workload type (training vs inference). TPU 8i places collective operations in the ICI IO die to avoid compute-die round-trips.
- Google+AMD reported TPU 10: hybrid CPU+accelerator on-package for RL workloads.

**Gap in Design Doc**:
- NeoTrix's `HardwareProfile` (`core/deploy.rs`) detects hardware but does not model chiplet topology, die-to-die bandwidth, or heterogeneous compute domains.
- `BodyDescriptor` (mentioned in `unified-consciousness-embodiment-architecture.md:88-93`) is a flat description, not a topology-aware model.
- The `CapabilityRegistry` does not model which compute elements are on which chiplet/die, or the bandwidth cost of cross-die operations.

**Suggestion**:
1. Create a `ComputeTopology` struct that models: chiplet count, per-chiplet core count, D2D bandwidth, NUMA node count, memory channels per node.
2. Extend `HardwareProfile` to include `topology: ComputeTopology`.
3. Make `CapabilityRegistry` topology-aware: when routing a capability invocation, consider the NUMA distance between the caller's chiplet and the target's chiplet.
4. Model this after Arm's CMN-S3 mesh: each module gets a "home node" and cross-module calls track "hop count" as a proxy for latency cost.

---

### DEFECT-461-3: No Page-Level Coherence Cost Modeling

**Severity**: MEDIUM | **Domain**: NT-MEMORY + NT-CORE

**Evidence from Research**:
- ISPASS 2026 (Babaie et al.): Coherence overheads are dominated by a **small subset of high-impact pages** despite most pages exhibiting limited sharing. These pages combine wide socket span, high access frequency, and millisecond-scale temporal persistence.
- Dorado (ISCA 2026): Two-Level Homes with Temporary directory slices reduce load latency by 46.1% by exploiting intra-cluster locality. Dynamic Apportioning allows directory entries to share storage dynamically.
- XTRA (SOSP 2026): Unifies cache coherence with OCC by using version validation as lazy coherence signals — small coherent control plane coordinates TB-scale shared data.

**Gap in Design Doc**:
- NeoTrix's KB (SQLite-backed) treats all data uniformly. There is no concept of "hot pages" or "coherent shared state" that drives disproportionate overhead.
- The `HeartbeatAggregator` (`CONTEXT.md:70`) tracks system health but does not model which KB entities are "hot" in the coherence sense (frequently read/written by multiple domains).
- SEAL pipeline distillation writes to KB without tracking cross-domain read patterns, potentially creating the same "high-impact pages" pattern that the ISPASS paper identifies as the dominant coherence cost driver.

**Suggestion**:
1. Add a `CoherenceHeatTracker` to NT-MEMORY that monitors cross-domain read/write patterns on KB entities, analogous to hardware page access counters.
2. Implement "page placement" optimization: frequently cross-domain-accessed KB entities should be placed in a "coherent fast-path" cache, analogous to Dorado's Temporary home directories.
3. Model lazy invalidation (from XTRA) for KB cache: instead of eagerly invalidating all domain caches on write, use version stamps and lazy validation on read.

---

### DEFECT-461-4: Missing Training/Inference Specialization Split

**Severity**: MEDIUM | **Domain**: NT-MIND + NT-CORE

**Evidence from Research**:
- Google TPU 8 family: First time a hyperscaler splits into two purpose-built chips. TPU 8t (training) optimizes for FLOPS+bandwidth; TPU 8i (inference) optimizes for SRAM+latency. The reasoning: "Inference needs more HBM bandwidth per compute. Inference needs more SRAM, a higher percentage of SRAM than compute."
- TPU 8i: 384MB on-chip SRAM sized specifically for KV cache footprint of reasoning models at production scale. 7-hop BoardFly topology (vs 16 for 3D Torus) trades bandwidth for latency.
- TPU 8t: FP4 native doubles throughput. Virgo Network (47Pb/s) for scale-out. Optical circuit switches for dynamic topology reconfiguration.

**Gap in Design Doc**:
- NeoTrix's SEAL pipeline (`CONTEXT.md:14`) runs exploration→distillation→self-test→absorption cycles, but does not differentiate between "training-like" cycles (exploration, heavy KB writes, high bandwidth) and "inference-like" cycles (query, reasoning, low latency).
- The `AttentionManager` (`CONTEXT.md` "Dual Specialization") switches between CORE+WORLD (acquisition) and CORE+MIND (evolution), but this is a coarse binary — it doesn't model the spectrum from training-heavy to inference-heavy workloads.
- No precision-tier modeling: the system uses fixed VSA 4096-dim vectors regardless of whether the current phase benefits from higher or lower precision.

**Suggestion**:
1. Extend `DualSpecialization` to a **Multi-Mode Specialization** model with at least 3 modes: `Exploration` (training-heavy: high bandwidth, low precision OK), `Reasoning` (inference-heavy: low latency, high precision needed), `Balanced` (mixed).
2. Add precision-aware routing: during `Exploration` mode, use coarser VSA representations (e.g., 2048-dim) for faster processing; during `Reasoning` mode, use full 4096-dim or higher.
3. Model this after TPU 8's "no dark silicon" philosophy: each mode activates only the subsystems it needs, freeing power budget for the active subsystems.

---

### DEFECT-461-5: No Interconnect Topology-Aware Attention Routing

**Severity**: MEDIUM | **Domain**: NT-CORE (GWT)

**Evidence from Research**:
- Google TPU 8t uses 3D Torus (bandwidth-optimized for training); TPU 8i uses BoardFly (latency-optimized for inference, max 7 hops vs 16).
- BoardFly "fly networks in general have lower latency" — topology choice directly impacts performance characteristics.
- TPU 8i performs collective operations in the ICI IO die near networking hardware, avoiding round-trips through the compute die. This saves both transfer time and HBM accesses.
- Dorado (ISCA 2026): Two-Level Homes exploit intra-cluster locality to minimize high-latency remote accesses.

**Gap in Design Doc**:
- NeoTrix's GWT (`CONTEXT.md:12`) "broadcasts salient information across specialist modules, with resonance-based routing" but does not model the communication topology between modules.
- The `CompetitionArena` computes salience from urgency+novelty+coherence, but does not factor in the "communication cost" between the broadcasting module and receiving modules.
- The `BroadcastBus` (`architecture-v2.md:136`) uses fixed-history broadcast with prune, but does not consider which modules are "nearby" (same process, shared memory) vs "far away" (different process, IPC).

**Suggestion**:
1. Add a `CommunicationTopology` model to GWT that tracks: which modules share a process, which communicate via IPC, which cross machine boundaries.
2. Extend salience computation to include a "communication cost" term: `salience = urgency + novelty + coherence - λ * comm_cost`.
3. Implement topology-aware broadcast: when `IgnitionDetector` triggers, prioritize broadcasting to "nearby" (low comm_cost) modules first, then "far" modules asynchronously — analogous to TPU 8i's BoardFly reducing hop count for latency-sensitive operations.

---

### DEFECT-461-6: Missing RAS (Reliability, Availability, Serviceability) Model

**Severity**: LOW | **Domain**: NT-SHIELD + NT-REPAIR

**Evidence from Research**:
- Arm AGI: Chipkill-class protection, memory scrubbing, row-hammer mitigation, repair support, error injection, RAS error logging. MPAM bandwidth partitioning prevents starvation.
- Fujitsu Monaka: Confidential computing via Arm CCA. Ultra-low voltage operation requires special design tools for reliability.
- Google TPU 8: In-field unit testing during idle cycles detects failures before they disrupt jobs. "Working assumption is that the chips will be flaky, so the architecture needs to be able to account for this and work around failed chips."
- Ranzware: AGI implements "anti-starvation mechanisms" and "QoS-based traffic prioritization" for predictable service under heavy load.

**Gap in Design Doc**:
- NeoTrix's `HeartbeatAggregator` (`CONTEXT.md:70`) tracks logical health (compilation/test/KB/eventbus/module health) with time-decay, but does not model hardware-level reliability concerns.
- NT-REPAIR (repair-healer) handles software self-healing but does not model hardware failure modes (silent data corruption, intermittent failures, thermal throttling).
- No "in-field testing" equivalent: NeoTrix's SelfTest runs on-demand, not during idle cycles.

**Suggestion**:
1. Add a `HardwareReliabilityModel` to `HeartbeatAggregator` that tracks: error correction counts, memory scrubbing results, thermal state, power state transitions.
2. Implement "idle-cycle SelfTest": when the system detects idle time (no active tasks), run lightweight SelfTest checks — analogous to TPU 8's in-field unit testing.
3. Add "anti-starvation" to the EventBus: when one domain is flooding the bus, throttle its priority to prevent starving other domains — analogous to Arm AGI's MPAM bandwidth controls.

---

### DEFECT-461-7: No NUMA-Aware Knowledge Base Partitioning

**Severity**: LOW | **Domain**: NT-MEMORY

**Evidence from Research**:
- Fujitsu Monaka: 3 NUMA configurations (1/4/8 nodes) — 1 node for large memory, 8 nodes for high throughput. Each NUMA node split into 2 LLC regions.
- Arm AGI: Coherent NUMA with 2x 6-channel DDR5 subsystems. Cross-chiplet requests travel over UCIe D2D link with latency cost.
- DGNA (arXiv:2607.19922): GPU L2/DRAM NUMA architecture reveals sub-NUMA nodes; write mechanisms differ between home-SM and remote-SM (remote-SM only updates remote-NUMA if cache hits, otherwise updates all L2 NUMA nodes).
- XTRA (SOSP 2026): Small coherent control plane (Coordination Directory) coordinates TB-scale shared data via version validation.

**Gap in Design Doc**:
- NeoTrix's KB is a single SQLite database — no NUMA-aware partitioning.
- When multiple NT-* domains access the same KB, there is no modeling of which domain is "local" to which KB partition.
- The `Ordered Backend Router` (`CONTEXT.md:126`) routes searches across backends (DDG→Wikipedia) with ordered fallback, but does not consider NUMA locality for KB access.

**Suggestion**:
1. Design a NUMA-aware KB partitioning scheme where each NT-* domain has a "home partition" for its most-accessed entities, analogous to Monaka's per-NUMA-node LLC regions.
2. Implement "local-first reads": when NT-CORE queries KB, check its home partition first before cross-partition lookup — analogous to Dorado's Two-Level Homes reducing remote accesses.
3. Model this as a `KbPartitionTopology` struct: maps each NT-* domain to its home KB partition, tracks cross-partition access frequency, and triggers repartitioning when skew exceeds threshold.

---

### DEFECT-461-8: Missing SRAM/On-Chip Memory Capacity Planning

**Severity**: LOW | **Domain**: NT-PHYSICAL + NT-MEMORY

**Evidence from Research**:
- Google TPU 8i: 384MB on-chip SRAM, explicitly sized for KV cache footprint of reasoning models at production scale. "Breaking the memory wall: to stop processors sitting idle."
- Fujitsu Monaka: LLC on separate 5nm die — SRAM density scaling has slowed on newest nodes (2nm barely shrinks SRAM cells), so 5nm is more cost-effective.
- Google TPU 8t: SparseCore accelerator offloads irregular memory access patterns (embedding lookups), preventing "zero-op" bottlenecks.

**Gap in Design Doc**:
- NeoTrix does not model on-chip memory (SRAM) capacity as a constraint.
- The `ReasoningBank` (1805 lines in `core/memory.rs`) and `KnowledgeHyperCube` (4096-dim VSA) have no size budget — they grow unbounded.
- No concept of "hot working set" that must fit in fast memory vs "cold archive" that can be in slow storage.

**Suggestion**:
1. Define a `MemoryBudget` model: total SRAM budget (configurable), per-module allocation, eviction policy when budget exceeded.
2. Make `KnowledgeHyperCube` budget-aware: when inserting a new vector, check if the cube is within budget; if not, eject lowest-salience vectors (analogous to TPU 8i's SRAM management).
3. Model `ReasoningBank` with a two-tier structure: hot tier (fits in "SRAM budget", fast access) + cold tier (overflow to disk, slower access) — analogous to TPU 8i's on-chip SRAM + HBM split.

---

## 3. Summary Table

| ID | Defect | Severity | Domain | Key Research Source |
|----|--------|----------|--------|-------------------|
| 461-1 | No Coherence-Aware Module Isolation Model | HIGH | NT-CORE + NT-PHYSICAL | PLDI 2026, arXiv:2608.05965 |
| 461-2 | Missing Chiplet/Disaggregated Compute Topology | HIGH | NT-CORE + NT-PHYSICAL | Hot Chips 2026 (Arm AGI, Fujitsu Monaka) |
| 461-3 | No Page-Level Coherence Cost Modeling | MEDIUM | NT-MEMORY + NT-CORE | ISPASS 2026, ISCA 2026 (Dorado) |
| 461-4 | Missing Training/Inference Specialization Split | MEDIUM | NT-MIND + NT-CORE | Google TPU 8t/8i |
| 461-5 | No Interconnect Topology-Aware Attention Routing | MEDIUM | NT-CORE (GWT) | Google TPU 8i (BoardFly) |
| 461-6 | Missing RAS Model | LOW | NT-SHIELD + NT-REPAIR | Arm AGI, Google TPU 8 |
| 461-7 | No NUMA-Aware KB Partitioning | LOW | NT-MEMORY | Fujitsu Monaka, DGNA |
| 461-8 | Missing SRAM Capacity Planning | LOW | NT-PHYSICAL + NT-MEMORY | Google TPU 8i (384MB SRAM) |

---

## 4. Cross-Cutting Theme

All 8 defects share a single root cause: **NeoTrix's architecture assumes uniform compute/memory resources, but 2026 hardware is fundamentally heterogeneous and topology-aware.** The industry has converged on:

1. **Heterogeneous chiplets** (Arm AGI, Fujitsu Monaka, Google TPU 8) — different dies for different functions, connected by explicit D2D links
2. **Workload-specialized silicon** (TPU 8t vs 8i) — training and inference are architecturally distinct
3. **Coherence as a first-class concern** (ShimGen, Dorado, XTRA) — correctness of heterogeneous systems requires formal coherence models
4. **Topology-aware scheduling** (BoardFly, CMN-S3, NUMA) — performance depends on understanding the physical communication graph

NeoTrix's software architecture should model these hardware realities as architectural primitives, not leave them as implicit assumptions. The suggestions above progressively align NeoTrix's design with the physical reality it will run on.

---

## 5. Priority Recommendations

| Priority | Action | Effort | Impact |
|----------|--------|--------|--------|
| P0 | Add Cross-Domain Coherence Contract (DEFECT-461-1) | 2 days | Prevents silent data corruption across domains |
| P0 | Add ComputeTopology to HardwareProfile (DEFECT-461-2) | 3 days | Enables NUMA-aware scheduling and chiplet modeling |
| P1 | Add CoherenceHeatTracker to NT-MEMORY (DEFECT-461-3) | 2 days | Prevents hot-page bottleneck in KB |
| P1 | Extend DualSpecialization to Multi-Mode (DEFECT-461-4) | 3 days | Optimizes SEAL pipeline for workload type |
| P2 | Add CommunicationTopology to GWT (DEFECT-461-5) | 2 days | Reduces broadcast latency for attention routing |
| P2 | Add HardwareReliabilityModel to HeartbeatAggregator (DEFECT-461-6) | 1 day | Improves fault tolerance |
| P3 | Design NUMA-aware KB partitioning (DEFECT-461-7) | 3 days | Improves KB access latency at scale |
| P3 | Add MemoryBudget to KnowledgeHyperCube (DEFECT-461-8) | 2 days | Prevents unbounded memory growth |

**Total estimated effort**: 18 days | **Total defects closed**: 8
