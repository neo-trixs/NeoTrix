# Iteration Batch 805 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### Embedded Databases (12)
- libmdbx-rs: Idiomatic Rust MDBX bindings, MVCC+CoW, zero-maintenance, no WAL
- mdbx-rs: Pure Rust libmdbx reimplementation, 15% faster GET, 37% faster CURSOR
- redb: Modern pure-Rust ACID KV store, compile-time typed tables
- fjall: LSM-tree embedded KV, used as TSDB
- TurboKV: Async embedded KV, WAL, compression, background compaction, beats redb/fjall
- SQLRite: Embedded SQL + vector from scratch: HNSW + BM25 + MVCC concurrent writes + MCP
- AgentVec: Embedded vector DB with TTL, in-place ACID updates, incremental index rebuild
- LanceDB: Serverless embedded vector DB, columnar, zero-copy, versioned, Arrow-backed
- tinyvector: Tiny embedding DB (pure Rust, ~600 LOC)
- RuVector: Agent memory substrate with graph relationships
- emdb-rs: Bitcask-style KV, lock-free reads/writes
- Turso/Limbo: SQLite rewrite in Rust, async I/O, io_uring, WASM-friendly

### Type System Advances (12)
- Async traits stable since Rust 1.75, but dyn dispatch gap remains
- GATs stable since 1.65, not object-safe (use static dispatch)
- Const generics approaching "full" (ADT const params, min_generic_const_args)
- corophage: Algebraic effects on stable Rust via coroutines + frunk, ~10ns/yield
- effing-mad: Coroutine-based effects on nightly
- id_effect: Functional ZIO-style effects with typed context/layers, STM
- Coenobita (IWACO 2026): Author/provider/lattice annotations, 3% compile-time overhead
- PermRust: ZST capability tokens, pub(crate) unforgeable constructors
- typesec: Capability<P,R> phantom types for agentic AI, SecureValue
- capsec: #[requires]/#[deny] proc macros for compile-time I/O capability enforcement
- Keyword Generics Initiative: effect async, const clauses coming to language

### Edge AI Inference (10)
- Precision format fragmentation: INT8/INT4/FP8/NVFP4/MXFP4 vendor divergence
- ExecuTorch: PyTorch on-device runtime, 12+ backends, powers Meta at scale
- NVFP4 + speculative decoding: 6.28× decode throughput on Jetson
- Calibration is #1 failure point for PTQ
- Apple Core AI (WWDC26): New on-device inference framework, 4-bit block-wise palettization
- QNN EP plugin model: Qualcomm acceleration as separate pip package
- QDQ format recommended since ORT 1.11, CPU kernels optimized for QDQ not QOperator

### Distributed Consensus (10)
- Antithesis: Found bugs in EVERY Raft implementation tested (HashiCorp, Aeron, OpenRaft, MicroRaft)
- TRM-Raft: Byzantine Raft, 40% Byzantine nodes dominate 80% leadership
- Bluestreak: DAG BFT scales to 400 validators
- Composing CRDTs (OOPSLA 2026): Composition can break convergence without careful design
- CRDT History: Eg-walker closing OT/CRDT gap; Loro rising as Rust-native CRDT
- Vector clocks: O(N) per message, tombstones accumulate, Dotted VVs fix sibling explosion
- Self-stabilizing VCs: First wait-free self-stabilizing vector clock
- Vulnerability assessment: Message replay + forgery attacks on Raft authentication gaps

---

## Defects Identified (38+)

### Embedded Databases (12)
| ID | Defect | Severity |
|----|--------|----------|
| D-DB-1 | HNSW index is stateless (rebuilt from SQLite every startup) | Critical |
| D-DB-2 | KB Bridge uses HashMap + linear scan (O(n) cosine) | Critical |
| D-DB-3 | No embedded KV store for high-frequency writes (hand-rolled JSONL) | High |
| D-DB-4 | SQLite single-writer bottleneck (no MVCC writes) | High |
| D-DB-5 | No persistent vector index (lost on process death) | High |
| D-DB-6 | BM25 is in-memory only, rebuilt from scratch each query | Medium |
| D-DB-7 | Embedding dimension mismatch risk (384/768/1024 coexist) | High |
| D-DB-8 | No MDBX for transactional KV | Medium |
| D-DB-9 | No incremental HNSW updates (bulk rebuild only) | Medium |
| D-DB-10 | VSA local embeddings are semantically meaningless (random projections) | High |
| D-DB-11 | No TTL/expiry for embeddings | Low |
| D-DB-12 | No compression for vector blobs (raw f32 at 1.5KB each) | Medium |

### Type System (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-TYPE-1 | #[async_trait] crate used 71× (native async fn available since 1.75) | Medium |
| D-TYPE-2 | No GATs/lending iterators (zero-copy streaming not exploitable) | Medium |
| D-TYPE-3 | Const generics underutilized (only 4 uses in entire codebase) | Low |
| D-TYPE-4 | No capability/permission type system (authorization is runtime-only) | High |
| D-TYPE-5 | dyn Trait bypasses native async trait | Low |
| D-TYPE-6 | No effect system for SEAL pipeline composition | Medium |

### Edge AI Inference (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-AI-1 | No runtime quantization format negotiation (hardware native precision) | High |
| D-AI-2 | Missing calibration pipeline (PTQ quality depends on calibration data) | High |
| D-AI-3 | No mixed-precision graph partitioning (all-or-nothing quantization) | Medium |
| D-AI-4 | ExecuTorch not integrated as execution backend | High |
| D-AI-5 | No hardware capability probe (NPU vendor detection) | High |
| D-AI-6 | No speculative decoding support (6.28× throughput on Jetson) | Medium |
| D-AI-7 | No QDQ format awareness (ORT CPU kernels optimized for QDQ) | High |
| D-AI-8 | No thermal/power budget enforcement | Medium |
| D-AI-9 | No fleet-level model versioning (canary/rollback) | Medium |
| D-AI-10 | Missing Apple Core AI integration | Medium |

### Distributed Consensus (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-CON-1 | Raft synchronous-process assumption violated by async operations | Critical |
| D-CON-2 | CRDT composition breaks convergence without careful design | High |
| D-CON-3 | Vector clock O(N) scaling death (metadata bloat) | High |
| D-CON-4 | Tombstone accumulation degrades KB query performance | Medium |
| D-CON-5 | Byzantine vulnerability in crash-fault protocols | High |
| D-CON-6 | No Delta-CRDT for KB state sync | High |
| D-CON-7 | No HLC (hybrid logical clock) for causal ordering | Medium |
| D-CON-8 | No Dotted Version Vectors for crawl node coordination | Medium |
| D-CON-9 | No Schnorr signatures for security-critical consensus | Medium |
| D-CON-10 | No tombstone GC policy for experience tree | Medium |

## Key Insights (This Batch)

1. **HNSW index is stateless**: Rebuilt from SQLite every startup. At 390K vectors this is seconds of cold-start penalty. Must persist graph structure to disk.

2. **VSA local embeddings are random projections**: xorshift64 PRNG seeded by FNV hash produces deterministic but semantically meaningless vectors. Two semantically identical sentences produce orthogonal vectors. Must replace with real embedding model.

3. **Every Raft implementation has bugs**: Antithesis found bugs in ALL implementations tested. Async heartbeat races, leadership transfer deadlocks, snapshot livelocks. NeoTrix should use Delta-CRDT for KB sync instead.

4. **CRDT composition breaks convergence**: OOPSLA 2026 proves composing convergent CRDTs can break algebraic properties. Must use the 5 CRDT combinators (Product, MapState, Associate, Traverse, MapInterpretation) for safe composition.

5. **Capability types are compile-time provable**: Coenobita, typesec, capsec show ZST phantom types with pub(crate) constructors give zero-cost compile-time authorization. NeoTrix has runtime-only checks.

6. **Native async fn replaces #[async_trait]**: Since Rust 1.75, no macro needed for static dispatch. 71 uses in codebase are unnecessary overhead.

7. **ExecuTorch is the new standard**: Powers Meta at scale, 12+ hardware backends, .pte format. NeoTrix's inference path doesn't support it.

8. **MDBX is the gold standard for embedded KV**: ACID, MVCC, CoW, zero-maintenance, used by Ethereum. NeoTrix hand-rolls JSONL journals instead.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 805 |
| New defects (this batch) | 38 |
| Cumulative defects | D01-D76236 |
| Research sources (this batch) | 44+ |
| Cumulative research sources | 96,954+ |
