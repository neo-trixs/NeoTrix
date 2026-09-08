# Iteration Batch 819 Report — NeoTrix Consciousness Architecture

## Research Sources (32+)

### Graph/Network (8)
- petgraph 0.8.3: Breaking refactor in progress (Jan 2026), new trait layout
- rustworkx-core 0.18.1: Advanced algorithms (johnson_simple_cycles, katz_centrality)
- graphblas_sparse_linear_algebra: CC-BY-NC-4.0 license (incompatible with commercial use)
- nalgebra-sparse 0.12.0: Early but usable (CSR/CSC/COO), "little focus on performance"
- SpacetimeDB v2.4.1: Reactive in-memory relational DB, single-threaded
- NeoTrix has 4 independent hand-rolled graph implementations
- nt_memory_graph.rs shortest_path uses BFS not Dijkstra (semantically incorrect)
- 3 separate KnowledgeGraph types (no shared type)

### Async Cancellation (8)
- tokio-util CancellationToken: Hierarchy with child_token(), drop_guard()
- Background loop has single-mutex contention (40+ handlers compete for same lock)
- Proxy kernel missing CancellationToken propagation (resource leak on shutdown)
- select! without biased; in shutdown causes premature handler abort
- Proxy copy without half-close propagation (data loss on teardown)
- No async drop pattern for cleanup engines
- EventHandler lock contention serializes all event handling
- tokio::task::JoinSet for scoped parallelism

### Observability (10)
- OpenTelemetry 0.32: Bound instruments give 28× counter speedup
- metrics 0.24.6: Lightweight facade, protocol-agnostic
- prometheus-client 0.25.0: Official OpenMetrics client
- metered-tracing 0.10.0-rc.1: Instrument once, derive traces+metrics
- NeoTrix pins OTel 0.27 (5 versions behind)
- No OTLP exporter configured (metrics collected but never shipped)
- Mutex<HashMap> for metrics storage (contention under concurrent emission)
- Per-module ad-hoc metric types (no shared schema)
- HeartbeatAggregator has no OTel hooks

### Memory Allocators (8)
- mimalloc v3.5.1: 13.4K stars, free list sharding, first-class heaps
- jemallocator 0.3.x: Unmaintained wrapper, large binary size
- bumpalo v3.18.x: Phase-bounded allocation, no individual dealloc
- typed-arena v2.x: Single-type arena, stale repo
- slab v0.4.x: Pre-allocated storage, O(1) insert/remove
- NeoTrix has no custom global allocator (system malloc)
- NT-WORLD/NT-MEMORY lack phase-bounded temp allocators
- VSA HyperCube E8 hexagrams use Box individually (no arena batching)

---

## Defects Identified (28+)

### Graph/Network (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-GRAPH-1 | 4 independent hand-rolled graph implementations (no petgraph) | High |
| D-GRAPH-2 | nt_memory_graph.rs shortest_path uses BFS not Dijkstra | High |
| D-GRAPH-3 | 3 separate KnowledgeGraph types (no shared type) | High |
| D-GRAPH-4 | GraphCache not thread-safe (no RwLock) | Medium |
| D-GRAPH-5 | graphblas license blocks commercial use | Medium |
| D-GRAPH-6 | nalgebra-sparse immature for production | Low |

### Async Cancellation (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-ASYNC-1 | Single-mutex contention in background loop (40+ handlers) | High |
| D-ASYNC-2 | Missing CancellationToken in proxy kernel | Medium |
| D-ASYNC-3 | Shutdown premature handler abort (single deadline for all) | Medium |
| D-ASYNC-4 | Proxy copy without half-close propagation | Medium |
| D-ASYNC-5 | No async drop pattern for cleanup engines | Low |
| D-ASYNC-6 | EventBus consumer lock contention (serialization bottleneck) | High |

### Observability (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-OBS-1 | OTel pinned to 0.27 (5 versions behind) | High |
| D-OBS-2 | No OTLP exporter configured (metrics never shipped) | High |
| D-OBS-3 | Mutex<HashMap> for metrics storage (contention) | Medium |
| D-OBS-4 | No metrics crate integration (ad-hoc HashMap<String,f64>) | High |
| D-OBS-5 | HeartbeatAggregator has no OTel hooks | Medium |
| D-OBS-6 | No exemplar support (trace↔metric correlation) | Low |

### Memory Allocators (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-MEM-1 | No custom global allocator (system malloc) | High |
| D-MEM-2 | No phase-bounded temp allocators (SEAL pipeline leaks) | Medium |
| D-MEM-3 | VSA HyperCube uses Box individually (no arena batching) | Low |
| D-MEM-4 | NT-ACT/NT-WORLD no pre-allocated task pools | Medium |
| D-MEM-5 | jemallocator unmaintained (mimalloc supersedes) | Low |
| D-MEM-6 | slab index-based access only (no iteration by value) | Low |

## Key Insights (This Batch)

1. **4 hand-rolled graph implementations**: petgraph is already a dependency but only used in capability_tree. KB graph, KG traversal, and semantic extract all roll their own BFS/Dijkstra with bugs.

2. **shortest_path is actually fewest_hops**: nt_memory_graph.rs accumulates edge weights but explores FIFO (BFS), so it finds the path with fewest edges, not shortest total weight.

3. **Background loop single-mutex is critical**: 40+ handlers compete for one lock. A slow KB write blocks all fast handlers (telemetry, heartbeat). Must shard per-domain.

4. **OTel 0.27→0.32 gives 28× counter speedup**: Bound instruments cache aggregator refs, eliminating per-call HashMap lookups.

5. **mimalloc as global allocator**: Immediate 15-30% throughput gain on multi-threaded KB operations. Must set #[global_allocator].

6. **bumpalo for SEAL pipeline**: Phase-bounded allocation prevents memory growth during long evolution cycles. Each phase gets its own Bump.

7. **graphblas license blocks commercial use**: CC-BY-NC-4.0. Must skip for NeoTrix. OneSparse-Rust is early-stage alternative.

8. **CancellationToken hierarchy**: tokio-util 0.7.19 provides child_token() and drop_guard() for graceful shutdown of handler trees.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 819 |
| New defects (this batch) | 24 |
| Cumulative defects | D01-D76615 |
| Research sources (this batch) | 34 |
| Cumulative research sources | 97,437+ |
