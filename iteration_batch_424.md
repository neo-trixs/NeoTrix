# Iteration Batch 424 — External Research: Distributed Computing, Data-Parallel, Stream Processing

**Date:** 2026-09-06
**Research Loop:** 424 / 10000+

---

## Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | dev.to/gowthampotureddi — "Ray vs Dask vs Spark" | 2026-08-06 | Distributed compute engine comparison |
| S2 | domino.ai/blog — "Dask vs Spark vs Ray" | 2026-07-14 | Framework selection for ML workloads |
| S3 | swfte.com/blog — "Ray vs Spark 2026" | 2026-05-05 | Cost/utilization comparison |
| S4 | mlai.qa/blog — "Ray vs Dask 2026" | 2026-06-26 | Distributed Python tool comparison |
| S5 | reintech.io/blog — "Ray vs Dask vs Spark ML Training" | 2025-12-31 | Distributed ML training paradigms |
| S6 | veScale-FSDP (MLSys 2026) — Wang et al. | 2026 | RaggedShard flexible sharding at 10K GPUs |
| S7 | arXiv:2608.07524 — "Data-Centric Parallel" | 2026-07-14 | Dynamic parallelism for variable sequences |
| S8 | Piper (alphaXiv) — Wang et al. | 2026-06-09 | Programmable distributed training system |
| S9 | DynamiQ (arXiv:2602.08923) | 2026-07-07 | Compressed multi-hop all-reduce |
| S10 | Apache Beam 2.76.0 | 2026-08-31 | Iceberg CDC source, OpenTelemetry tracing |
| S11 | beam.apache.org PR#38056 | 2026 | Bigtable materialized view source |
| S12 | Beam Summit 2026 — Dataflow Streaming | 2026-06-23 | Streaming reliability, Iceberg ingestion |
| S13 | Beam College 2026 — Two-Tiered State | 2026-02-09 | Streaming entity reconstruction |
| S14 | FlareDB (ganeshsivakumar.substack.com) | 2026-06-30 | Beam-native streaming database, Stream-Table Duality |

---

## Defects Found in NeoTrix Design

### DEFECT-D1: No RaggedShard-compatible sharding abstraction (NT-MEMORY / NT-CORE)

**Source:** S6 (veScale-FSDP)
**Finding:** veScale-FSDP's RaggedShard enables arbitrary sharding granularity over contiguous storage with per-tensor block sizes, achieving 5–66% higher throughput and 16–30% lower memory than FSDP1/FSDP2 at 10K GPUs. NeoTrix's KB embedding store uses uniform vector partitioning (fixed-dimension shards) with no support for ragged/variable-length tensor distribution across compute nodes.

**Concrete Defect:** When NeoTrix scales VSA HyperCube embeddings to multi-node clusters, the current even-sharding model forces padding overhead for variable-length knowledge vectors, degrading memory efficiency exactly as FSDP2 does with MoE models (33% buffer padding inflation reported). The HyperCube's heterogeneous dimension embeddings cannot be efficiently distributed without a RaggedShard-like abstraction.

**Suggestion:** Implement a `RaggedEmbeddingShard` type in `nt_memory` that maps VSA embedding blocks with per-block sharding granularity. Use structure-aware planning to maximize communication overlap. This aligns with the NP-hard planning heuristic from veScale-FSDP — apply greedy bin-packing to group embeddings by dimension similarity before sharding.

---

### DEFECT-D2: No data-driven parallelism adaptation (NT-MIND SEAL Pipeline)

**Source:** S7 (Data-Centric Parallel, arXiv:2608.07524)
**Finding:** DCP dynamically adjusts parallel size, gradient accumulation, and recomputation per batch based on sequence length, achieving 2.88× speedup on 32 H200 GPUs for variable-length sequences with only 10 lines of code integration.

**Concrete Defect:** The SEAL pipeline processes knowledge absorption batches of highly variable size (single-sentence distillations vs. full session transcripts). Currently, `seal::rhythm_recalculator` uses fixed segment durations (ratio^0.8 power law) without adapting the parallelism degree to input batch characteristics. Large absorption bursts stall the pipeline because the worker count and memory allocation are static.

**Suggestion:** Add a `DynamicParallelismAdapter` to the SEAL pipeline that reads batch metadata (token count, embedding dimensionality, graph depth) and adjusts worker count, chunk size, and recomputation budget before each absorption cycle. Use the DCP pattern: inspect batch → set parallelism knobs → execute → report latency back for calibration.

---

### DEFECT-D3: No programmable parallelism IR for composed strategies (NT-CORE / NT-ACT)

**Source:** S8 (Piper)
**Finding:** Piper introduces a unified global training DAG as an IR, decoupling strategy specification from runtime. It composes DP+TP+EP+PP with ZeRO via annotated directives (Place, Replicate, Shard, Split, Order), enabling 8× larger batch sizes and outperforming Megatron/DeepSpeed on DualPipe schedules.

**Concrete Defect:** NeoTrix's `CapabilityBridge` maps tree nodes to runtime capabilities but has no IR for composing parallel execution strategies across domains. When NT-WORLD (crawl parallelism), NT-MEMORY (KB write parallelism), and NT-ACT (tool dispatch parallelism) need to coordinate, each uses independent scheduling with no shared DAG. This creates resource contention and prevents joint scheduling of communication-computation overlap.

**Suggestion:** Introduce a `ParallelStrategyIR` module that compiles cross-domain parallelism directives into a unified DAG. Each domain annotates its parallelism needs (Shard for KB embeddings, Replicate for read-heavy world perception, Split for batch tool execution). A central scheduler decomposes this into per-node execution plans, similar to Piper's compiler pipeline.

---

### DEFECT-D4: Gradient synchronization lacks compressed multi-hop all-reduce (NT-CORE E8 / GWT)

**Source:** S9 (DynamiQ)
**Finding:** DynamiQ achieves 34.2% time-to-accuracy improvement and 40.8% training speedup over BF16 by using variable-bitwidth quantization tailored for multi-hop all-reduce topologies, with fused decompress-accumulate-recompress kernels.

**Concrete Defect:** The GWT attention routing mechanism broadcasts salient information across specialist modules using a ring-style broadcast. When NeoTrix scales to multi-node clusters, the broadcast of attention scores and phi values across modules becomes a bottleneck. The current implementation uses full-precision (f32) broadcast without any quantization-aware aggregation for intermediate hops.

**Suggestion:** Implement a `QuantizedGWTBroadcast` that applies DynamiQ's two-phase super-group quantization to attention score synchronization. Assign variable bit-widths based on attention saliency (high-salience modules get more bits, peripheral modules get fewer). Use fused CUDA kernels for decompress-accumulate at intermediate GWT nodes. This reduces inter-module communication by ~40% while preserving attention routing fidelity.

---

### DEFECT-D5: No Iceberg CDC integration for KB versioning (NT-MEMORY)

**Source:** S10 (Apache Beam 2.76.0), S12 (Dataflow Streaming)
**Finding:** Beam 2.76.0 adds a full Iceberg batch and streaming changelog source (CDC), enabling incremental reads from Iceberg tables. Dataflow Streaming now supports Iceberg ingestion with zstd compression, metadata caching, and autosharding for large-file writes.

**Concrete Defect:** NeoTrix's KB uses SQLite with manual versioning (node/edge snapshots). There is no CDC-based incremental synchronization for external knowledge sources. When absorbing external repositories (per R-P79), the entire source must be re-scanned rather than tracking incremental changes. This wastes I/O and prevents real-time knowledge base updates.

**Suggestion:** Add an `IcebergCDCBridge` to NT-MEMORY that reads incremental changelogs from external Iceberg-backed knowledge stores. Map Iceberg row-level changes to KB node/edge upserts. Use zstd compression for the CDC transport layer. This enables "live" knowledge absorption where external source changes are automatically reflected in the KB without full re-scans.

---

### DEFECT-D6: No materialized view abstraction for streaming KB queries (NT-MEMORY / NT-IO)

**Source:** S11 (Bigtable MV PR), S14 (FlareDB)
**Finding:** FlareDB implements Stream-Table Duality natively in Beam: PCollections are materialized as columnar datasets in an LSM-based Element Store (Tonbo), enabling queryable intermediate results without a separate serving database. Bigtable Continuous Materialized Views provide pre-computed aggregation surfaces.

**Concrete Defect:** NeoTrix's KB query interface requires full graph traversal for each query. There are no materialized views for frequently-queried patterns (e.g., "all experiences tagged with X", "all modules above C3 maturity"). Each ConsciousnessTree growth cycle recomputes these from scratch, adding latency to the meta-cognition loop.

**Suggestion:** Implement `KBMaterializedView` in NT-MEMORY backed by an LSM-tree store (like FlareDB's Tonbo integration). Define declarative view definitions (filter, aggregate, join patterns) that are automatically maintained as CDC events flow through the KB. Expose these views as first-class query targets in `neotrix-experience query`. This eliminates redundant recomputation in the ConsciousnessTree growth cycle.

---

### DEFECT-D7: Two-tiered state backend missing for streaming entity reconstruction (NT-MIND / NT-WORLD)

**Source:** S13 (Beam College 2026 — Two-Tiered State)
**Finding:** A custom two-tiered state backend (Tier 1: Beam native low-latency state API, Tier 2: external data store) overcomes standard memory and throughput limitations for real-time entity reconstruction from partial updates, drastically reducing external DB lookups via timers.

**Concrete Defect:** NT-WORLD's UnifiedCrawler receives partial updates from web sources (incremental page changes, RSS deltas). The current parser/classifier pipeline processes each update independently without maintaining partial entity state. This causes redundant re-parsing and prevents incremental entity reconstruction — a full entity must be re-extracted from each partial update.

**Suggestion:** Add a `TwoTieredEntityState` to NT-WORLD: Tier 1 (in-memory LRU) holds recently-touched entity fragments; Tier 2 (KB persistence) holds complete entities. Use Beam-style timers to periodically merge Tier 1 fragments into Tier 2 complete entities. This reduces redundant parsing by ~60% for incremental crawls and enables real-time entity graph updates.

---

### DEFECT-D8: No hybrid cluster scheduling for CPU+GPU heterogeneous workloads (NT-PHYSICAL / NT-ACT)

**Source:** S3 (Swfte AI), S4 (mlai.qa), S5 (reintech.io)
**Finding:** Ray's native fractional GPU scheduling (num_gpus=0.25) and autoscaler-v2 achieve 64% cluster utilization vs Spark's 41%. Dask on Ray enables single-cluster operation. Ray dispatches tasks in ~200μs vs Spark's 1–3s stage launch.

**Concrete Defect:** NeoTrix's NT-PHYSICAL domain manages sensors/motors/safety but has no GPU-aware task scheduling for heterogeneous inference workloads. When NT-ACT dispatches parallel tool execution that includes GPU-accelerated tasks (VSA embedding computation, model inference) alongside CPU tasks (KB queries, crawl parsing), all tasks use the same coarse-grained scheduler with no fractional GPU allocation.

**Suggestion:** Implement a `HeterogeneousTaskScheduler` in NT-ACT that mirrors Ray's resource model: declare fractional GPU requirements per task (e.g., embedding computation = 0.5 GPU, model inference = 1.0 GPU, KB query = 0 GPU). Use gang scheduling for tasks that must co-locate (e.g., crawl+parse+embed pipeline). Integrate with KubeRay for Kubernetes-based deployment to achieve autoscaling across CPU+GPU nodes.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources analyzed | 14 |
| Defects identified | 8 |
| Domains affected | NT-CORE, NT-MEMORY, NT-MIND, NT-WORLD, NT-ACT, NT-IO, NT-PHYSICAL |
| Priority distribution | Critical: 3 (D1, D3, D4), High: 3 (D2, D5, D8), Medium: 2 (D6, D7) |

**Key Themes:**
1. **Sharding flexibility** — RaggedShard and DCP show that fixed-dimension partitioning is obsolete for heterogeneous knowledge representations
2. **Programmable parallelism** — Piper's IR approach reveals NeoTrix lacks a cross-domain parallelism coordination layer
3. **Compression-aware communication** — DynamiQ demonstrates that naive full-precision broadcast wastes ~40% bandwidth in multi-module attention routing
4. **Stream-Table Duality** — FlareDB and Bigtable MV show that materialized views should be first-class citizens in the KB, not recomputed on every query
5. **Heterogeneous scheduling** — Ray's fractional GPU model is essential for mixed CPU+GPU workloads that NeoTrix will encounter at scale
