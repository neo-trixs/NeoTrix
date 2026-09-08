# Iteration Batch 325 — External Research × Design Gap Analysis

**Date**: 2026-09-06  
**Domains**: Cloud Native/K8s, WebAssembly, Streaming/Real-time  
**Method**: 9 parallel web searches → design doc defect identification → suggestion generation

---

## 1. Sources Cited

### Cloud Native & Kubernetes (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| S1 | NVIDIA KAI Scheduler (GitHub, KubeCon EU 2026) | HAMi-core integration for physically-enforced GPU sharing (v0.16.4, June 2026). DRA driver donated to CNCF at KubeCon EU 2026. |
| S2 | Kubernetes GPU Scheduling Best Practices (kubex.ai, 2026-08-06) | HAMi-core isolation reuses driver-interception layer for memory/compute enforcement. KAI Scheduler now integrates HAMi-core as built-in feature. DRA GA in K8s 1.34 (Aug 2025). |
| S3 | Kubernetes 1.37 "Garhwal" (TechTimes, 2026-08-28) | Gang scheduling (KEP-4671) graduated to Beta. Workload-aware preemption (KEP-5710) Beta. HPA scale-to-zero Beta (enabled by default). `minCount` field for elastic training. `CompositePodGroups` for hierarchical gang scheduling (Alpha). |
| S4 | Kubernetes 1.37 (ADTmag, 2026-08-31) | 67 enhancements. 16 Stable, 23 Beta, 27 Alpha. Workload API + PodGroup concept for gang scheduling. |
| S5 | Kubernetes GPU Orchestration 2026 (Spheron, 2026-04-15) | Grove: NVIDIA's K8s API for inference workloads. PodClique/PodCliqueScalingGroup/PodCliqueSet/ClusterTopology/PodGang abstractions for disaggregated inference. |
| S6 | KAI Scheduler (kai-scheduler.dev) | Hierarchical queues, elastic workloads, GPU sharing (time-slicing/MPS/MIG), topology-aware scheduling. |

### WebAssembly (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| S7 | WASP: Stateful Serverless Wasm (arXiv 2607.25493, 2026) | Configurable WASM framework for edge-cloud continuum. Swappable runtime (Wasmtime/Wazero/WasmEdge) + datastore (Redis/PostgreSQL). Negligible runtime overhead. |
| S8 | WASM Edge-Cloud Review (IEEE, 2023) | WASM as next-gen isolation primitive after V8 isolates. |
| S9 | CWASI: Runtime Shim (arXiv 2504.21503, 2025) | 3-mode inter-function communication: Function Embedding / Local Buffer / Networked Buffer. Up to 95% latency reduction, 30x throughput increase for co-located functions. |
| S10 | WASM Migration over Cloud-Edge (UniPD Thesis, 2023/2024) | Live migration of WASM modules via pre-copy + post-copy techniques. Runtime code adaptation for heterogeneous hardware. |
| S11 | WebAssembly Server-Side 2026 Deep Dive (youngju.dev, 2026-05-25) | WASI P2 GA. All major runtimes ship P2 (Wasmtime 26+, WasmEdge 0.14+, Wasmer 5+, jco 1.x). wasmCloud v2.0 (2026-03-23): removed capability providers, host plugins in-process, explicit distributed networking. WasmEdge: AOT via LLVM, K8s RuntimeClass, AI plugins (ggml/PyTorch/TFLite/OpenVINO). |
| S12 | wasmCloud v2.0 (wasmcloud.com, 2026-03-23) | Kubernetes-native CRD orchestration. Host plugins run in-process (near-native speed). Deny-by-default capability model. WASI P2 native. P3 support planned. |
| S13 | Component Model 1.0 Roadmap (Bytecode Alliance, 2026-06-08) | WASI P3 imminent (native async). Component Model 1.0 target: lazy ABI, multi-value, error contexts, WASM GC. Cooperative threads via `wasi-libc` pthreads. Stream splicing. Guest/Host C-ABI for easier implementation. |
| S14 | Rust Wasm Optimization (rustwasm.github.io) | LTO, `opt-level='s'/'z'`, `wasm-opt`, `twiggy` profiler. Trait objects vs generics for size. `wee_alloc` allocator. |
| S15 | Rust 1.85 Wasm Optimization (johal.in, 2026-05-08) | 12 Wasm-specific optimization passes in MIR pipeline. `-Z wasm-opt` integrated passes. 32% size reduction, 27% throughput improvement. LLVM 18 backend. |
| S16 | WASM Size Optimization Series Part 5 (infinilabs, 2026-05-24) | `no_std + alloc` pure mode: 185KB gzipped search engine. `build-std` recompiling std from source. 4-tier architecture (nano/micro/mini/ultra). Feature-gated cost bands. |

### Streaming & Real-time (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| S17 | Event-Driven Architecture Complete Guide (RisingWave, 2026-03-20) | EDA is baseline in 2026. Streaming databases (RisingWave) as queryable state layer on top of Kafka. |
| S18 | EDA & Message Queues 2026 Reference (digitalapplied, 2026-06-02) | Fowler's 4 EDA patterns remain canonical. Three messaging primitives: queue/pub-sub/stream. Exactly-once is myth; idempotent consumers + at-least-once is production answer. Kafka 4.0 removed ZooKeeper (KRaft only). Transactional outbox pattern. |
| S19 | EDA in 2026 (encore.dev, 2026-05-03) | EDA pays for itself with 3+ consumers of same event. Avoid for simple CRUD. Broker selection: Kafka/Redpanda/NATS/RabbitMQ/SNS+SQS/Pub/Sub. |
| S20 | EDA in 2026: Queues, Streams, Resilience (substack, 2026-06-04) | Queues for work distribution, streams for event history. Hybrid architecture (queues for commands, streams for events) is sweet spot. Backpressure is critical. DLQ, circuit breakers, sagas, outbox pattern all mandatory. |
| S21 | EDA in 2026: Kafka, Streaming SQL, AI Layer (RisingWave, 2026-04-08) | 2026 gap: Kafka is excellent at storing events but not at answering "what is current state?" Streaming database (RisingWave) fills the gap. AI agents need queryable live state via MCP, not raw event consumption. |
| S22 | EDA in 2026: Kafka (Sensussoft, 2026-06-22) | Kafka operational complexity is real: partitions, consumer groups, rebalances, KRaft config. Start with one high-value flow, grow deliberately. |
| S23 | Kafka Alternatives 2026 (Domo, 2026-07-27) | Redpanda: Kafka-compatible, C++, no JVM. Pulsar: multi-tenancy, geo-replication, tiered storage. NATS JetStream: lightweight streaming. Diskless Kafka emerging (AutoMQ, WarpStream). |
| S24 | Kafka vs Pulsar vs Redpanda 2026 (simplifiedlearning, 2026-01-28) | Kafka: largest ecosystem, KRaft default. Pulsar: compute-storage separation, superior for K8s. Redpanda: 30-50% cost savings, sub-ms latency, WASM transforms in broker. |
| S25 | Apache Kafka Alternatives 7 Options (Redisson, 2026-07-22) | NATS JetStream lightest streaming option. Diskless Kafka (AutoMQ, WarpStream) for cloud cost. |
| S26 | Kafka Alternatives Compared (tinybird, 2026-08-11) | Streaming databases serve real-time analytics APIs. Kafka for distribution, streaming DB for queryable state. |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-01: No Gang Scheduling Awareness for Multi-GPU Inference Workloads
**Severity**: HIGH  
**Evidence**: K8s 1.37 gang scheduling is now Beta (S3, S4). KAI Scheduler + HAMi-core provides physically-enforced GPU sharing with hierarchical queues (S1, S2). NeoTrix's `ParallelTaskManager` and `TaskScheduler` (CONCEPT.md:160-170) manage GPU scheduling but have no concept of gang scheduling, PodGroup atomicity, or `minCount` elastic training.  
**Gap**: If NeoTrix deploys multi-GPU inference (e.g., disaggregated prefill/decode as in Grove's PodClique model S5), partial placement could waste GPUs. No awareness of KAI Scheduler's fair-share queues or workload-aware preemption.  
**Suggestion**: Extend `nt_act::parallel_task` with a `GangSchedulingPolicy` enum: `Atomic` (all-or-nothing), `Elastic` (min-count), `None`. Integrate with KAI Scheduler's PodGroup CRD for distributed training workloads.

### DEFECT-02: Missing WASM Component Model for Plugin Sandboxing
**Severity**: HIGH  
**Evidence**: WASI P2 is GA across all major runtimes (S11). wasmCloud v2.0 uses deny-by-default capability model with host plugins (S12). Component Model 1.0 roadmap includes lazy ABI, cooperative threads, stream splicing (S13). WASP framework demonstrates swappable WASM runtime + storage for edge-cloud continuum (S7).  
**Gap**: NeoTrix has no WASM-based plugin isolation model. The "Rune Socketing" system (CONTEXT.md:65) uses in-process configuration, not sandboxed execution. Third-party skills/extensions run with full host privileges.  
**Suggestion**: Define a `WasmPluginLayer` in NT-PHYSICAL or NT-SHIELD for untrusted extension sandboxing. Use Component Model WIT interfaces for typed capability grants. Target: `wasm32-wasip2` compilation with deny-by-default host access.

### DEFECT-03: No Edge-Cloud Continuum Deployment Model
**Severity**: MEDIUM  
**Evidence**: WASM edge-cloud continuum research is active (S7, S8, S10). WASP deploys unchanged from server to Raspberry Pi (S7). Live migration of WASM modules between cloud and edge via pre/post-copy (S10). CWASI achieves 95% latency reduction for co-located functions (S9).  
**Gap**: NeoTrix architecture assumes single-process or single-cluster deployment. No concept of edge nodes, fog layer, or heterogeneous hardware targets. NT-PHYSICAL mentions sensors/motors but no deployment topology model.  
**Suggestion**: Add a `DeploymentTopology` enum to NT-PHYSICAL: `SingleNode`, `Clustered`, `EdgeCloudContinuum`. Define `WasmEdgeTarget` for constrained devices (RPi, IoT gateways). Add runtime adaptation mechanism for heterogeneous WASM runtimes.

### DEFECT-04: EventBus Lacks Streaming Database Integration for Live Queryable State
**Severity**: HIGH  
**Evidence**: 2026 EDA gap: Kafka stores events but can't answer "what is current state?" (S21). AI agents need queryable live state via MCP, not raw event consumption (S21). Streaming databases (RisingWave) maintain continuously updated materialized views (S17, S21).  
**Gap**: NeoTrix EventBus (mentioned in AGENTS.md audit dimensions D26-D36) is a message bus, not a queryable state layer. ConsciousnessTree and GWT need to query current system state, not replay event logs. The "Heartbeat Aggregator" (CONTEXT.md:70) computes snapshots but isn't a persistent queryable view.  
**Suggestion**: Add a `StreamingStateLayer` to NT-MEMORY that maintains materialized views of EventBus events. Expose via SQL-compatible interface (or internal query API) so GWT attention routing can query "current system health" without replaying events. Consider embedding a lightweight streaming DB (e.g., SQLite materialized views with triggers, or a WASM-native streaming engine).

### DEFECT-05: No Backpressure or Dead-Letter-Queue Pattern in Event System
**Severity**: MEDIUM  
**Evidence**: Backpressure is "the quiet hero of resilient systems" in 2026 EDA (S20). DLQ, circuit breakers, sagas, outbox pattern are mandatory for production EDA (S18, S20). Exactly-once is a myth; idempotent consumers + at-least-once is the production answer (S18).  
**Gap**: NeoTrix EventBus design mentions "Two-layer EventBus" (D31-D36) but no backpressure mechanism, no DLQ for poison messages, no idempotency keys. GWT attention broadcast could overwhelm slow consumers.  
**Suggestion**: Implement `BackpressurePolicy` in EventBus: `Buffer` (bounded queue), `DropOldest`, `DropNewest`, `Block`. Add `DeadLetterQueue` with alerting. Tag all events with `idempotency_key` for consumer dedup.

### DEFECT-06: WASM Optimization Pipeline Not Defined for NeoTrix Binaries
**Severity**: LOW  
**Evidence**: Rust 1.85 introduces 12 Wasm-specific optimization passes in MIR pipeline (S15). `no_std + alloc` reduces WASM to 185KB gzipped (S16). Feature-gated cost bands (nano/micro/mini/ultra) enable tiered deployment (S16).  
**Gap**: NeoTrix build process uses standard `cargo build` (AGENTS.md). No WASM optimization pipeline (`wasm-opt`, `twiggy` profiling, `build-std` for `no_std`). If NeoTrix modules are ever compiled to WASM for edge deployment, binary size and cold start will be unoptimized.  
**Suggestion**: Define a `WasmBuildProfile` in Cargo workspace: `edge-nano` (no_std, opt-level='z', LTO), `edge-micro` (std, geo/vector/graph), `edge-mini` (full std + JSON), `edge-ultra` (aggregations + scripting). Add `twiggy` to CI for size regression detection.

### DEFECT-07: No WASM Inter-Function Communication Model for Co-Located Modules
**Severity**: MEDIUM  
**Evidence**: CWASI's 3-mode model (Function Embedding / Local Buffer / Networked Buffer) achieves 95% latency reduction for co-located WASM functions (S9). wasmCloud v2.0 defaults to in-process calls in nanoseconds, explicit wiring for distributed (S12).  
**Gap**: NeoTrix inter-domain communication (NT-CORE ↔ NT-MIND ↔ NT-WORLD etc.) uses in-process Rust calls or EventBus. If domains are compiled as WASM components, there's no communication model defined for co-located vs. distributed placement.  
**Suggestion**: Define `InterComponentCommunicationPolicy`: `InProcess` (nanosecond, shared memory), `LocalBuffer` (Unix socket, co-located), `NetworkedBuffer` (NATS/mTCP, distributed). Auto-select based on deployment topology.

### DEFECT-08: Missing DRA (Dynamic Resource Allocation) Integration for GPU Workloads
**Severity**: MEDIUM  
**Evidence**: DRA is GA since K8s 1.34 (S2). NVIDIA + Google donated DRA drivers to CNCF at KubeCon EU 2026 (S5). DRA replaces the decade-old device plugin with structured resource attributes, MIG profiles, NVLink topology (S5).  
**Gap**: NeoTrix's `ParallelTaskManager` and `ResourceBudgetManager` manage GPU resources but don't express requirements via DRA ResourceClaims. Still relies on integer GPU counts.  
**Suggestion**: Extend `nt_act::resource_budget` to emit DRA `ResourceClaim` specs: GPU type, MIG profile, NVLink topology constraints. Enable topology-aware placement for multi-GPU inference.

### DEFECT-09: No Schema Evolution Strategy for Event Contracts
**Severity**: MEDIUM  
**Evidence**: Schema registries enforce event contracts in production EDA (S17, S22). Event schema evolution without breaking consumers is a core challenge (S17). Confluent Schema Registry + Avro/Protobuf is standard (S23).  
**Gap**: NeoTrix EventBus has no schema registry, no event versioning, no forward/backward compatibility guarantees. Adding new consumers or changing event shapes risks silent breakage.  
**Suggestion**: Add `EventSchemaRegistry` to NT-MEMORY with: schema ID in event header, version compatibility modes (BACKWARD/FORWARD/FULL), Avro/Protobuf/JSON Schema support. Integrate with EventBus for automatic validation.

### DEFECT-10: No WASM Cooperative Thread Model for Async Consciousness Processing
**Severity**: LOW  
**Evidence**: WASI P3 adds native async support (S13). Cooperative threads via `wasi-libc` pthreads are shipping (S13). Component Model async ABI supports both stackless (callback) and stackful concurrency (S13).  
**Gap**: NeoTrix ConsciousnessTree runs a 6-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core) synchronously. If WASM-compiled, the synchronous model doesn't leverage P3's native async. GWT attention broadcasting could benefit from cooperative threading for parallel specialist evaluation.  
**Suggestion**: Design ConsciousnessTree loop stages as `async` WASM exports. Use Component Model `future`/`stream` types for GWT broadcast. Allow parallel specialist evaluation via cooperative threads when compiled to WASM.

---

## 3. Suggestions Summary

| # | Defect | Priority | Suggested Action | Target Module |
|---|--------|----------|------------------|---------------|
| D01 | No gang scheduling | HIGH | Add `GangSchedulingPolicy` enum, integrate KAI Scheduler PodGroup | `nt_act::parallel_task` |
| D02 | No WASM plugin sandboxing | HIGH | Define `WasmPluginLayer` with Component Model WIT interfaces | `nt_physical` or `nt_shield` |
| D03 | No edge-cloud continuum | MEDIUM | Add `DeploymentTopology` enum, `WasmEdgeTarget` for constrained devices | `nt_physical` |
| D04 | No streaming state layer | HIGH | Add `StreamingStateLayer` with materialized views for GWT queries | `nt_memory` |
| D05 | No backpressure/DLQ | MEDIUM | Implement `BackpressurePolicy`, `DeadLetterQueue`, idempotency keys | EventBus core |
| D06 | No WASM optimization pipeline | LOW | Define `WasmBuildProfile` tiers, add `twiggy` to CI | Cargo workspace |
| D07 | No inter-function comm model | MEDIUM | Define `InterComponentCommunicationPolicy` (InProcess/Local/Networked) | Cross-domain |
| D08 | No DRA integration | MEDIUM | Emit DRA `ResourceClaim` specs from `resource_budget` | `nt_act::resource_budget` |
| D09 | No schema evolution | MEDIUM | Add `EventSchemaRegistry` with compatibility modes | `nt_memory` |
| D10 | No async consciousness model | LOW | Design ConsciousnessTree stages as `async` WASM exports | `nt_core` / ConsciousnessTree |

---

## 4. Cross-Domain Synthesis

The 2026 landscape reveals three convergent trends that NeoTrix should absorb:

1. **WASM as the universal sandbox**: Component Model P2→P3, cooperative threads, deny-by-default capabilities, edge-cloud portability. NeoTrix should adopt WASM not just as a deployment target but as the **isolation primitive** for untrusted extensions and cross-domain communication.

2. **K8s-native GPU orchestration is mature**: DRA GA, gang scheduling Beta, HAMi-core enforcement, HPA scale-to-zero. NeoTrix's GPU management layer must express workloads as DRA ResourceClaims and support gang scheduling for multi-GPU inference.

3. **Streaming databases bridge the event-to-state gap**: Kafka stores events; streaming DBs (RisingWave) maintain queryable live state. NeoTrix's EventBus needs a materialized view layer so GWT and ConsciousnessTree can query current system state without replaying event logs.

---

*Iteration 325 complete. 10 defects identified across 3 research domains. 26 sources cited.*
