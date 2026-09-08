# Iteration Batch 587 — Database Transactions, Concurrency Control & Distributed Transactions

**Date**: 2026-09-06  
**Predecessor**: Batch 586 (interpretability attack surface, false-positive XAI, attention≠causal, no formal validity guarantees, EU XAI compliance gap)  
**Domain**: Persistence Layer — Transaction Safety & Distributed Coordination

---

## Batch 586 Recap (Context)

Batch 586 established five defects in the AI interpretability layer:
1. Interpretability methods are adversarial attack surfaces
2. Standard attribution methods (SHAP, LIME, GradCAM) produce false-positive explanations (Nature 2026)
3. Attention weights ≠ causal attribution — paradigm shift needed
4. No formal validity guarantees on GWT attention routing
5. EU AI Act / GDPR compliance gap for XAI requirements

---

## Category 1: Database Transactions & ACID

### Finding 1.1: Write Skew Survives Snapshot Isolation
**Source**: ndlab.blog (Aug 2026), AppScale (Jul 2026), Databricks concurrency guide  
**Defect**: Snapshot Isolation (SI) — the default MVCC mode in PostgreSQL/MySQL — does NOT prevent **write skew**, where two transactions read overlapping data and write based on stale reads, producing a state neither would generate alone. Only Serializable Snapshot Isolation (SSI) catches this.  
**NEW vs batch 586**: Batch 586 analyzed XAI false-positives; this is a *data-layer false-positive*: the database claims consistency under SI while silently permitting write skew corruption.

### Finding 1.2: Isolation Level Selection Is a Hidden Architectural Decision
**Source**: ByteLedger (Apr 2026), JOptimize (May 2026), Databricks  
**Defect**: Most Spring Boot / ORM applications use `@Transactional` with the default isolation level (READ COMMITTED) without explicit selection. The default is wrong for financial writes, inventory mutations, or any multi-row invariant. **70%+ of production databases run at an isolation level weaker than they need.**  
**NEW vs batch 586**: Batch 586 identified missing formal guarantees on GWT; this is the persistence equivalent: missing explicit isolation-level governance across modules.

### Finding 1.3: Serializability Throughput Cost Is Non-Linear Under Contention
**Source**: ndlab blog, ByteLedger, NDLab  
**Defect**: SERIALIZABLE isolation costs 10–30% throughput under low contention, but under high contention (hot rows), the cost becomes non-linear — retry storms cascade. NeoTrix KB writes (experience hub, KV store, domain namespaces) could hit this under concurrent SEAL pipeline writes.  
**NEW vs batch 586**: Identified a concrete scaling risk for NeoTrix KB concurrent writes during multi-domain evolution cycles.

---

## Category 2: Concurrency Control (MVCC / OCC)

### Finding 2.1: VACUUM/Version Chain Bloat Under Write-Heavy Workloads
**Source**: Databricks (2026), datalakehousehub.com (Apr 2026), Dremio/Iceberg analysis  
**Defect**: PostgreSQL MVCC heap-stored versions require VACUUM to reclaim dead tuples. Under sustained high write throughput (e.g., SEAL pipeline absorbing 50+ sessions), autovacuum falls behind → table bloat → query latency degrades → VACUUM storms compete for I/O. No automatic reclaim in write-heavy analytical workloads.  
**NEW vs batch 586**: Concrete operational risk for NeoTrix KB under heavy SEAL absorption load — version chain bloat is an invisible performance cliff.

### Finding 2.2: Optimistic Concurrency Control Retry Waste Under Conflict
**Source**: datalakehousehub.com (Apr 2026), CockroachDB/TiDB analysis  
**Defect**: OCC assumes low conflict. When conflicts are frequent (concurrent writes to shared KB nodes), transactions execute fully then abort at validation → wasted CPU + I/O + token cost. CockroachDB/Spanner hybrid approach helps but adds latency.  
**NEW vs batch 586**: NeoTrix's KB node contention (multiple domains writing to shared experience hub) could trigger OCC retry storms, wasting compute on failed absorption cycles.

### Finding 2.3: Lakehouse Snapshot Isolation ≠ Database Snapshot Isolation
**Source**: datalakehousehub.com (Apr 2026), Iceberg/Delta Lake analysis  
**Defect**: Apache Iceberg/Delta Lake "snapshot isolation" operates on immutable files with atomic pointer swaps — fundamentally different from PostgreSQL's MVCC row versioning. NeoTrix KB uses SQLite (MVCC-based), but if analytics/warehouse layers (DuckDB, Spark) are added, snapshot semantics differ silently. Cross-system isolation assumptions are a false analogy.  
**NEW vs batch 586**: Architecture hazard: mixing SQLite (row MVCC) with analytical engines (file-snapshot) creates hidden isolation boundary mismatches.

---

## Category 3: Distributed Transactions (2PC / Saga)

### Finding 3.1: 2PC Coordinator Single Point of Failure Is Worse Than Documented
**Source**: enterprise-software-review (Jul 2026), designgurus.io (Jun 2026), NDLab (Aug 2026)  
**Defect**: 2PC's blocking window (between PREPARE and COMMIT) leaves all participants holding locks. If the coordinator fails, the system is **frozen** — not just slow, but unable to commit or rollback. In-doubt transaction recovery requires manual intervention or operational tooling that most teams lack. 2026 production incidents confirm this is under-estimated.  
**NEW vs batch 586**: Batch 586 identified XAI compliance gaps; this is a distributed transaction compliance gap: DORA (EU Digital Operational Resilience Act) now mandates auditable state transitions, and in-doubt 2PC transactions are a violation.

### Finding 3.2: Saga Intermediate State Exposure Is a Security/Privacy Risk
**Source**: designgurus.io (Jun 2026), codelit.io (Mar 2026), hirenodejs.com (May 2026)  
**Defect**: Sagas pass through visible intermediate states (e.g., "payment charged but order not created"). In NeoTrix's multi-domain architecture, a Saga spanning NT-WORLD→NT-MEMORY→NT-ACT could expose partial KB writes to concurrent readers before compensation completes. This is a data leak vector.  
**NEW vs batch 586**: Security angle not in batch 586: intermediate state visibility during Saga compensation = information disclosure risk in multi-domain KB.

### Finding 3.3: Outbox Pattern + Kafka Transactional Producer Is the 2026 Standard
**Source**: hirenodejs.com (May 2026), enterprise-software-review (Jul 2026), AppScale (Apr 2026)  
**Defect**: NeoTrix's EventBus (two-layer: in-process + persistent) lacks the **transactional outbox** pattern for cross-service event delivery. Current implementation likely loses events on failure boundary between write and publish. The 2026 standard is write-to-outbox-in-same-transaction, then poll/push to Kafka with exactly-once semantics.  
**NEW vs batch 586**: Concrete EventBus improvement: NeoTrix needs transactional outbox to guarantee event delivery consistency.

### Finding 3.4: Saga Compensation Idempotency Is Non-Trivial
**Source**: hirenodejs.com (May 2026), NDLab (Aug 2026), AppScale (Apr 2026)  
**Defect**: Compensating actions must be **idempotent** — retries must not double-compensate. Designing idempotent compensation for KB operations (e.g., "undo experience absorption") requires version tracking on every write. Without it, Saga retries corrupt KB state.  
**NEW vs batch 586**: Concrete design requirement for NeoTrix SEAL pipeline: every KB write needs a version identifier for idempotent compensation.

---

## Category 4: Cross-Cutting Defects

### Finding 4.1: Write Skew + Attention Routing = Silent Corruption in NeoTrix KB
**Source**: Synthesis of Finding 1.1 + batch 586 Finding 3 (attention≠causal)  
**Defect**: If NeoTrix KB runs at Snapshot Isolation (PostgreSQL default for read-heavy workloads) while GWT attention routing directs concurrent writes from multiple domains, write skew can silently corrupt the experience hub's index. The corrupted data then feeds back into GWT attention decisions → cascading false-attention cycle.  
**Severity**: CRITICAL — silent data corruption + attention corruption = self-reinforcing failure mode.

### Finding 4.2: MVCC Version Chain + VSA HyperCube = Storage Explosion
**Source**: Synthesis of Finding 2.1 + CONTEXT.md  
**Defect**: VSA HyperCube stores high-dimensional vectors for concept representation. MVCC retains multiple versions. Under concurrent writes, version chain × vector dimension = potential storage explosion. PostgreSQL TOAST + autovacuum may not keep up with VSA vector churn.  
**Severity**: HIGH — storage cost spiral under concurrent evolution cycles.

### Finding 4.3: Saga Compensation Across NeoTrix Domains Requires Domain-Aware Rollback
**Source**: Synthesis of Finding 3.2-3.4 + CONTEXT.md  
**Defect**: NeoTrix's 7-domain architecture means a Saga across NT-WORLD→NT-MEMORY→NT-ACT→NT-SHIELD requires domain-specific compensation logic. Generic Saga orchestration (Temporal, Inngest) doesn't understand NT-* domain semantics. Compensation must be domain-aware.  
**Severity**: MEDIUM — architectural debt if not addressed before cross-domain operations mature.

---

## NEW vs Batch 586 Summary

| # | Defect | Category | Severity | Novel |
|---|--------|----------|----------|-------|
| 1.1 | Write skew survives Snapshot Isolation | Transaction | HIGH | ✅ Data-layer false-positive (parallel to XAI false-positives) |
| 1.2 | Default isolation level is wrong for financial/multi-row writes | Transaction | MEDIUM | ✅ Governance gap |
| 1.3 | Serializability cost non-linear under contention | Transaction | MEDIUM | ✅ Scaling risk |
| 2.1 | VACUUM/version chain bloat under write-heavy workloads | Concurrency | HIGH | ✅ Operational performance cliff |
| 2.2 | OCC retry waste under KB node contention | Concurrency | MEDIUM | ✅ Compute waste |
| 2.3 | Lakehouse snapshot ≠ database snapshot isolation | Concurrency | MEDIUM | ✅ False analogy hazard |
| 3.1 | 2PC coordinator SPOF + DORA compliance gap | Distributed | HIGH | ✅ Regulatory angle |
| 3.2 | Saga intermediate state = privacy/security risk | Distributed | HIGH | ✅ Security vector |
| 3.3 | Missing transactional outbox on NeoTrix EventBus | Distributed | HIGH | ✅ Concrete improvement |
| 3.4 | Saga compensation idempotency requires version tracking | Distributed | MEDIUM | ✅ Design requirement |
| 4.1 | Write skew + GWT attention = silent corruption cascade | Cross-cutting | CRITICAL | ✅ Novel failure mode |
| 4.2 | MVCC version chain × VSA vector = storage explosion | Cross-cutting | HIGH | ✅ Storage risk |
| 4.3 | Saga compensation must be domain-aware across NT-* | Cross-cutting | MEDIUM | ✅ Architecture debt |

**Total NEW defects**: 13  
**Critical**: 1 (4.1 — write skew + attention cascade)  
**High**: 5 (1.1, 2.1, 3.1, 3.2, 3.3, 4.2)  
**Medium**: 5 (1.2, 1.3, 2.2, 2.3, 3.4, 4.3)

---

## Sources Cited

1. ndlab.blog — "Transactions & ACID: Isolation Levels, Locking and the Bugs They Prevent" (Aug 2026)
2. AppScale — "ACID Transactions & Isolation Levels: A Practical Guide" (Jul 2026)
3. ByteLedger — "Database Transactions Explained 2026" (Apr 2026)
4. JOptimize — "Database Transactions in Spring Boot: Isolation Levels" (May 2026)
5. datalakehousehub.com — "Concurrency, Isolation, and MVCC: How Engines Handle Contention" (Apr 2026)
6. Databricks — "Concurrency Control in DBMS: Locking, MVCC & More" (2026)
7. enterprise-software-review — "Distributed Transactions in 2026: Saga Patterns vs Two-Phase Commit" (Jul 2026)
8. NDLab — "Distributed Transactions: Two-Phase Commit, Saga and When to Use Each" (Aug 2026)
9. designgurus.io — "Saga Pattern vs. Two-Phase Commit" (Jun 2026)
10. hirenodejs.com — "Node.js Saga Pattern in 2026: Distributed Transactions Guide" (May 2026)
11. AppScale — "Saga Orchestration Pattern" (Apr 2026)
12. codelit.io — "Distributed Transactions and the Saga Pattern" (Mar 2026)
13. kindatechnical.com — "Distributed Transactions and Two-Phase Commit" (Mar 2026)

---

## Recommended Actions for NeoTrix

1. **KB Write Isolation**: Explicitly set SERIALIZABLE for all multi-row KB mutations (experience hub, domain namespaces). Accept 10-30% throughput cost.
2. **EventBus Outbox**: Implement transactional outbox pattern on two-layer EventBus before cross-domain operations scale.
3. **Saga Versioning**: Add version identifiers to every KB write for idempotent compensation in cross-domain Sagas.
4. **VACUUM Monitoring**: Add autovacuum lag monitoring to NeoTrix KB health dashboard (HeartbeatAggregator integration).
5. **Write Skew Detection**: Implement write-skew detection for experience hub concurrent access (SELECT FOR UPDATE on hot rows).
6. **Domain-Aware Compensation**: Design domain-specific rollback handlers for NT-WORLD→NT-MEMORY→NT-ACT→NT-SHIELD Saga paths.

---

*Batch 587 complete. 13 new defects identified. Critical: write-skew + GWT attention cascade (silent self-reinforcing corruption). Next: batch 588 should investigate formal verification of NeoTrix KB isolation guarantees under concurrent SEAL pipeline loads.*
