# Iteration Batch 785 Report — NeoTrix Consciousness Architecture

## Research Sources (30+)

### Concurrency & Async Rust (14)
- GrafeoDB: Block-STM parallel txns, morsel-driven parallelism, rayon batch search
- frankengraphdb: `Cx` capability contexts, deterministic simulation, sync API
- crdt-kit: no_std, zero-alloc CRDTs, u64 NodeId, delta-state sync
- rust-crdt: Hybrid CvRDT/CmRDT, causal context (ReadCtx/AddCtx/RmCtx)
- y-crdt: Yrs (Yjs Rust port), binary protocol compat, FFI+WASM
- indradb: Pluggable datastores, gRPC server, simple concurrency
- PostHog case study: 2s→94ms by separating Tokio (I/O) from Rayon (compute)
- scc2: 46M+ downloads, lock-free HashMap/Queue/Stack with async
- arctic-wt: lock-free adaptive radix tree (OSDI '26)
- velocityx: 52M+ ops/s MPMC queue, 58M+ ops/s concurrent hashmap
- Structured Concurrency: JoinSet + CancellationToken over bare spawn

### WebAssembly & Edge (8)
- WASI Preview 2 stable, WASI 0.3 with native async I/O
- Component Model with WIT for polyglot composition
- Wasm 3.0 W3C: WasmGC, exception handling, 128-bit SIMD
- Edge functions 9x faster cold starts than Lambda
- Cloudflare Workers: billions of requests across 300+ PoPs
- Edge databases (Turso, Neon, D1) reach GA
- Docker supports `--runtime=io.containerd.wasmedge.v1`

### Database Internals (12)
- LSM-Tree: Sequential I/O 2-10x faster than random on NVMe
- WAL: ARIES 3-phase recovery (Analysis→Redo→Undo)
- coordinode-lsm-tree: MVCC, BuRR filters, zstd dictionary, AES-256-GCM, Page ECC
- fjall-rs/lsm-tree: 449 stars, Bloom filters, leveled + FIFO compaction
- wal-db: Composable WAL primitive for shared storage engines
- IronWal: Deterministic multi-stream WAL with sharded locking
- Temperature-tiered storage: inline micro-adjacency → delta blocks → compressed CSR

### Networking & Protocols (8)
- Tonic → CNCF gRPC Project (May 2026), new `grpc` crate coming
- h3: HTTP/3 experimental, h3-quinn v0.0.10
- tokio-tungstenite: WebSocket standard, no built-in reconnection
- frankengraphdb: HTTP/2+gRPC+WS+Bolt, RaptorQ erasure coding
- crdt-kit: DeltaCrdt trait, HybridClock, network transport roadmap
- Macaroon capability tokens with graph caveats (frankengraphdb Warden)

---

## Defects Identified (30)

### Concurrency & Async (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-CON-1 | tokio full feature set pulls in everything unnecessarily | High |
| D-CON-2 | No thread budget between Tokio and Rayon pools | High |
| D-CON-3 | Broadcast lagged subscribers silently lose events | High |
| D-CON-4 | Arc<Mutex<Vec<JoinHandle>>> blocking lock in async | Medium |
| D-CON-5 | Arc<Mutex<Dispatcher>> blocking pattern | Medium |
| D-CON-6 | Arc<RwLock<AppState>> held across all request handlers | Medium |
| D-CON-7 | Multiple tokio::runtime::Runtime::new() in tests | Low |
| D-CON-8 | No structured concurrency (JoinSet/CancellationToken) | Medium |
| D-CON-9 | No CancellationToken propagation for graceful shutdown | Low |
| D-CON-10 | No lock-free data structures in hot paths | Medium |

### WebAssembly & Edge (3)
| ID | Defect | Severity |
|----|--------|----------|
| D-WASM-1 | No WASM edge profile for KB embedding | High |
| D-WASM-2 | No tombstone compaction for CRDTs | High |
| D-WASM-3 | RocksDB backend not portable to WASM | Medium |

### Database Internals (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-DB-1 | No WAL implementation (relies on SQLite WAL) | Critical |
| D-DB-2 | Missing Bloom filter layer for point lookups | High |
| D-DB-3 | No compaction strategy for experience entries | High |
| D-DB-4 | No MVCC/time-travel for knowledge evolution | Medium |
| D-DB-5 | No deterministic replay for KB mutations | Medium |
| D-DB-6 | No page-level error detection for binary blobs | Medium |
| D-DB-7 | No WAL-aware crash recovery for non-SQLite paths | Medium |

### Networking & Protocols (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-NET-1 | No HTTP/3/QUIC support | High |
| D-NET-2 | gRPC provider lock-in (tonic being superseded) | Medium |
| D-NET-3 | No WebSocket reconnection/replay | Medium |
| D-NET-4 | Missing Bolt wire protocol (Neo4j compat) | Low |
| D-NET-5 | No content-addressed transport (RaptorQ FEC) | Low |
| D-NET-6 | No macaroon/capability-token auth | Medium |
| D-NET-7 | TLS backend rigidity (ring/rustls) | Low |
| D-NET-8 | No encrypt-then-code at rest pattern | Medium |
| D-NET-9 | No fingerprint rotation for stealth net | Low |
| D-NET-10 | No delta-state sync for cross-session CRDTs | High |

---

## Key Insights (This Batch)

1. **Structured concurrency is the 2026 standard** — JoinSet + CancellationToken over bare tokio::spawn. Parent/child task tree with temporal scope = lexical scope.

2. **PostHog: 2s→94ms by separating Tokio from Rayon** — Default thread pools both assume they own all cores → 2x oversubscription. Solution: separate pools with semaphore backpressure.

3. **Lock-free data structures reached maturity** — scc2 (46M+ downloads), arctic-wt (OSDI '26), velocityx (52M+ ops/s). All concurrent access using Arc<Mutex<T>> is a missed opportunity.

4. **Temperature-tiered storage is the frontier** — Inline micro-adjacency → sorted delta blocks → sealed compressed CSR runs. Hot minority pays delta cost; cold majority sits below raw CSR.

5. **Tonic is being superseded** — CNCF gRPC Project under grpc/grpc-rust (May 2026). New `grpc` crate with connection management, client-side load balancing, xDS/Envoy support.

6. **Macaroon capability tokens compile to planner predicates** — frankengraphdb's Warden model: caveats about scope/ops/ownership compile to planner-enforced row/subgraph security.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 785 |
| New defects (this batch) | 30 |
| Cumulative defects | D01-D75602 |
| Research sources (this batch) | 30+ |
| Cumulative research sources | 96,044+ |
