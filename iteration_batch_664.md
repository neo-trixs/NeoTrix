# Iteration Batch 664 — Distributed Database, Consensus & Replication Landscape

**Date**: 2026-09-06  
**Sources**: 24 web sources (2026-dated where available)  
**Continuity**: Extends Batch 663 (supply-chain/signing/SLSA findings)

---

## 1. Distributed Database Landscape (2026)

### Sources
- sanj.dev, "CockroachDB vs TiDB vs YugabyteDB 2026" (updated 2026-03-17)
- PiStack, "CockroachDB vs YugabyteDB vs TiDB: Best Distributed SQL Database 2026" (2026-04-16)
- youngju.dev, "Distributed SQL / NewSQL 2026 Deep Dive" (2026-05-15)
- PingCAP, "Best Distributed SQL Databases 2026" (2026-03-23)

### Key Findings

| System | Architecture | Consensus | License (2026) |
|--------|-------------|-----------|----------------|
| CockroachDB v24 | Shared-nothing, Pebble (Rust KV) | Raft per range | BSL → Apache 2.0 after 3yr |
| TiDB 8 | TiKV (Rust) + TiFlash (columnar) + PD | Raft (TiKV) | Apache 2.0 |
| YugabyteDB | DocDB (RocksDB + Raft) + PG upper layers | Raft per tablet | Apache 2.0 |
| Google Spanner | TrueTime + Paxos | Paxos (per shard) | Proprietary |
| Aurora DSQL | Storage-compute split | Quorum | AWS managed |
| Neon (Databricks) | Storage-compute split, branching | Custom | Apache 2.0 |

### NEW Defect #664-D1: No Distributed-DB Threat Model for AI Agent Data
**Severity**: MEDIUM  
**Detail**: All 2026 comparison guides optimize for OLTP/HTAP workloads. None address threat models specific to AI agent state — multi-agent coordination writes, LLM-generated content deduplication, or vector embedding consistency across regions. NeoTrix KB is SQLite-backed with no distributed replication story. If NeoTrix scales to multi-region agent clusters, the single-node SQLite KB becomes a consensus bottleneck with no CRDT fallback.

### NEW Defect #664-D2: CockroachDB License Trap
**Severity**: LOW  
**Detail**: CockroachDB's BSL license (converts to Apache 2.0 after 3 years per release) means NeoTrix cannot embed it as a managed service without BSL constraints. YugabyteDB and TiDB are pure Apache 2.0. This limits NeoTrix's options if it needs to offer a hosted DB tier.

### NEW Defect #664-D3: No Cross-Region KV Consistency Benchmark for Agent Workloads
**Severity**: LOW  
**Detail**: The 2026 benchmarks (TPC-C, YCSB) don't model agent-specific workloads: high-frequency small writes (tool call logs), bursty embedding updates, or sparse-but-critical metadata mutations. No existing benchmark validates whether distributed SQL can handle NeoTrix's access patterns.

---

## 2. Consensus Algorithm Landscape (2026)

### Sources
- SysTutorials, "Paxos vs Raft: Consensus Algorithms Compared" (updated 2026-04-12)
- dev.to/narendars, "Distributed Consensus: Paxos vs Raft and Modern Implementations" (2025-05)
- codelucky.com, "Distributed Consensus: Raft and Paxos Algorithms Explained" (2025-08)
- codelit.io, "Distributed Consensus Explained: Raft, Paxos, and Beyond" (2026-03-28)
- techinterview.org, "LLD: Consensus Algorithms (Raft and Paxos)" (2026-04-18)

### Key Findings

| Algorithm | Leader Model | Fault Tolerance | Production Users (2026) |
|-----------|-------------|-----------------|------------------------|
| Raft | Strong leader | (n-1)/2 crash | etcd, TiKV, CockroachDB, KRaft (Kafka) |
| Multi-Paxos | Flexible proposers | (n-1)/2 crash | Spanner, Chubby |
| PBFT | Primary-backup | (n-1)/3 Byzantine | Hyperledger, Tendermint |
| KRaft | Quorum controller | (n-1)/2 crash | Kafka 4.0+ (ZK-free) |

### NEW Defect #664-D4: Raft Leader Lease Clock Drift Risk
**Severity**: HIGH  
**Detail**: Raft leader leases (used for read optimization without ReadIndex round-trip) rely on bounded clock drift between nodes. If clocks skew beyond the lease duration, the lease guarantee breaks. In NeoTrix's edge/multi-device deployment (NT-PHYSICAL sensors), clock discipline is unreliable. No existing NeoTrix module accounts for lease invalidation under clock skew.

### NEW Defect #664-D5: No Byzantine Tolerance for External Data Ingestion
**Severity**: MEDIUM  
**Detail**: NeoTrix NT-WORLD crawls external data sources. PBFT/Byzantine tolerance is not applied to data ingestion pipelines. A compromised or adversarial web source could inject contradictory data that propagates through the KB without detection. No data provenance chain or Byzantine-resistant merge exists.

### NEW Defect #664-D6: KRaft Migration Blind Spot
**Severity**: LOW  
**Detail**: Kafka's ZK→KRaft migration is now mandatory (ZooKeeper removed in Kafka 4.0). If NeoTrix uses Kafka for EventBus, the migration path must be validated. No existing test confirms EventBus compatibility with KRaft-only mode.

---

## 3. Replication & CRDT Landscape (2026)

### Sources
- martinuke0, "Implementing CRDTs for Eventual Consistency in Distributed Systems" (2026-05-13)
- martinuke0, "Implementing CRDTs for Eventual Consistency in Distributed Collaborative Systems" (2026-05-13)
- JavaCodeGeeks, "Delta-State CRDTs: Solving the Bandwidth Problem" (2026-09-04)
- JavaCodeGeeks, "CRDTs: The Data Structure That Makes Distributed Consistency Optional" (2026-04-13)
- zylos.ai, "CRDTs and Distributed State Synchronization for Multi-Agent AI Systems" (2026-03-17)
- zylos.ai, "CRDTs and Real-Time Collaboration" (2026-01-29)
- INRIA/DAIS 2024, "Synql: CRDT-powered SQLite sync" (cited in 2026)
- ISA AFP, "Framework for Establishing Strong Eventual Consistency for CRDTs" (2026-08-03)

### Key Findings

| CRDT Type | Use Case | Merge Complexity | Bandwidth |
|-----------|----------|-----------------|-----------|
| G-Counter | Increment-only counters | O(N) per replica | Low |
| PN-Counter | Bidirectional counters | O(N) | Low |
| OR-Set | Set with removals | O(ops) | Medium |
| RGA | Ordered sequences (text) | O(log N) | High |
| LWW-Map | Key-value with timestamps | O(1) per key | Low |
| Delta-state CRDTs | Any — bandwidth-optimized | O(delta) | Very Low |
| JSON CRDT (Automerge/Yjs) | Document collaboration | O(N) | Medium-High |

### Performance (2026)
- **Automerge 2.0**: 600ms processing for 260,000 keystrokes (down from 2s/char)
- **Yjs**: 26K–156K ops/sec sustained throughput
- **cr-sqlite / Synql**: CRDT as SQLite extension — multi-writer replication without app-layer awareness
- **ESP32 CRDTs**: CRDTs running on microcontrollers (520KB SRAM) — ECOOP 2025

### NEW Defect #664-D7: CRDT Metadata Bloat in Long-Running Agent Sessions
**Severity**: HIGH  
**Detail**: OR-Set and RGA CRDTs accumulate tombstones and causal metadata (version vectors) unboundedly. In NeoTrix's multi-day agent sessions with high-frequency state mutations (tool calls, memory writes, emotion state updates), metadata growth will eventually dominate payload size. No garbage-collection horizon or compaction strategy exists in NeoTrix's KB.

### NEW Defect #664-D8: No Delta-State CRDT for KB Embedding Sync
**Severity**: MEDIUM  
**Detail**: Delta-state CRDTs (JavaCodeGeeks 2026-09) solve the bandwidth problem by sending only changed state deltas rather than full state. NeoTrix KB embeddings (vector representations) are high-dimensional and expensive to transmit. No delta-state CRDT implementation exists for vector embedding synchronization across NeoTrix nodes.

### NEW Defect #664-D9: CRDT Cannot Enforce Global Invariants
**Severity**: MEDIUM  
**Detail**: CRDTs guarantee convergence but cannot enforce global invariants (e.g., "total budget across all agents ≤ $100", "no two agents hold exclusive lock on same resource"). NeoTrix's ResourceBudgetManager and ParallelTaskManager rely on global invariants that CRDTs cannot provide. A hybrid CRDT+consensus approach is needed but not designed.

### NEW Defect #664-D10: No Formal Verification of NeoTrix State Merges
**Severity**: LOW  
**Detail**: ISA AFP 2026 published a modular framework for formally verifying CRDT correctness (commutativity, associativity, idempotence). NeoTrix's merge logic (KB writes, experience absorption, module state sync) has no formal verification. Property-based testing (proptest/Hypothesis) is recommended but not implemented for merge operations.

### NEW Defect #664-D11: Local-First Agent Architecture Gap
**Severity**: LOW  
**Detail**: FOSDEM 2026 hosted its first "Local First, sync engines and CRDTs" devroom. The trend is toward local-first software with CRDT-based sync. NeoTrix's architecture is server-centric (single SQLite KB). No local-first agent architecture exists where agents operate offline and sync via CRDTs when reconnected.

---

## 4. Cross-Cutting Defects (Distributed DB × Consensus × CRDT)

### NEW Defect #664-D12: No Multi-Region KB Consensus Strategy
**Severity**: CRITICAL  
**Detail**: NeoTrix has no strategy for multi-region KB replication. Options exist (Raft for strong consistency, CRDTs for availability, Spanner for external consistency) but none are designed. If NeoTrix deploys agent clusters across regions, the single-node SQLite KB is a single point of failure with no consensus or replication layer.

### NEW Defect #664-D13: No Consensus-Backed Experience Absorption
**Severity**: HIGH  
**Detail**: The experience-tree absorption protocol writes to a local KB. In a multi-agent scenario, concurrent absorption of experiences by multiple agents could produce contradictory entries. No consensus mechanism (Raft, Paxos, or CRDT) protects the experience namespace from concurrent write conflicts.

### NEW Defect #664-D14: EventBus ≠ Consensus
**Severity**: MEDIUM  
**Detail**: NeoTrix's EventBus provides pub/sub delivery but not consensus. Event ordering is not guaranteed across partitions. If two agents emit conflicting events during a partition, the merge order is non-deterministic. This is acceptable for notification events but not for state-mutation events.

---

## 5. Improvement Opportunities

| # | Opportunity | Source Inspiration | NeoTrix Module |
|---|------------|-------------------|----------------|
| I1 | Adopt cr-sqlite for multi-writer KB sync | INRIA/DAIS 2024 Synql | nt_memory |
| I2 | Implement delta-state CRDT for embedding sync | JavaCodeGeeks 2026-09 | nt_memory |
| I3 | Add version vectors to KB writes for causal ordering | CRDT literature (Shapiro et al.) | nt_memory |
| I4 | Design hybrid CRDT+Raft for invariant-critical state | zylos.ai 2026-03 | nt_core + nt_memory |
| I5 | Property-based merge testing with proptest | ISA AFP 2026 | nt_memory tests |
| I6 | Local-first agent mode with CRDT sync | FOSDEM 2026 devroom | nt_world + nt_memory |
| I7 | Byzantine-resistant data ingestion for NT-WORLD | PBFT concepts | nt_world + nt_shield |
| I8 | KRaft compatibility test for EventBus | Kafka 4.0 migration | nt_io |

---

## 6. Sources Cited

1. sanj.dev — "CockroachDB vs TiDB vs YugabyteDB 2026" (2026-03-17)
2. PiStack — "CockroachDB vs YugabyteDB vs TiDB: Best Distributed SQL Database 2026" (2026-04-16)
3. youngju.dev — "Distributed SQL / NewSQL 2026 Deep Dive" (2026-05-15)
4. PingCAP — "Best Distributed SQL Databases 2026" (2026-03-23)
5. SysTutorials — "Paxos vs Raft: Consensus Algorithms Compared" (2026-04-12)
6. dev.to/narendars — "Distributed Consensus: Paxos vs Raft" (2025-05)
7. codelucky.com — "Distributed Consensus: Raft and Paxos Explained" (2025-08)
8. codelit.io — "Distributed Consensus Explained: Raft, Paxos, and Beyond" (2026-03-28)
9. techinterview.org — "LLD: Consensus Algorithms" (2026-04-18)
10. martinuke0 — "Implementing CRDTs for Eventual Consistency" (2026-05-13)
11. martinuke0 — "CRDTs for Eventual Consistency in Collaborative Systems" (2026-05-13)
12. JavaCodeGeeks — "Delta-State CRDTs: Solving the Bandwidth Problem" (2026-09-04)
13. JavaCodeGeeks — "CRDTs: The Data Structure That Makes Consistency Optional" (2026-04-13)
14. zylos.ai — "CRDTs and Distributed State Sync for Multi-Agent AI" (2026-03-17)
15. zylos.ai — "CRDTs and Real-Time Collaboration" (2026-01-29)
16. ISA AFP — "Framework for Establishing SEC for CRDTs" (2026-08-03)
17. INRIA/DAIS 2024 — Synql (cited in zylos.ai 2026-03)
18. ACM/PODC — "Paxos vs Raft: Have we reached consensus?" (Howard et al.)

---

## Summary

**14 NEW defects** identified (1 CRITICAL, 3 HIGH, 5 MEDIUM, 5 LOW):
- **CRITICAL**: No multi-region KB consensus strategy (#664-D12)
- **HIGH**: Raft leader lease clock drift (#664-D4), CRDT metadata bloat (#664-D7), no consensus-backed experience absorption (#664-D13)
- **MEDIUM**: No DB threat model for AI agents (#664-D1), no Byzantine ingestion (#664-D5), no delta-state CRDT for embeddings (#664-D8), CRDT can't enforce global invariants (#664-D9), EventBus ≠ consensus (#664-D14)
- **LOW**: CockroachDB license trap (#664-D2), no agent workload benchmark (#664-D3), KRaft migration blind spot (#664-D6), no formal merge verification (#664-D10), local-first gap (#664-D11)

**8 improvement opportunities** identified, top 3:
1. Adopt cr-sqlite for multi-writer KB sync (I1)
2. Implement delta-state CRDT for embedding sync (I2)
3. Design hybrid CRDT+Raft for invariant-critical state (I4)
