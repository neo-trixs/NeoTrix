# Iteration Batch #419 — Event Sourcing / CQRS / Streaming Research

**Date**: 2026-09-06
**Research Domain**: Event-driven architecture, CQRS, stream processing

---

## Sources Cited

1. **Event-Driven Architecture Deep Dive — Advanced Patterns (2026)** — tutorialq.com (Feb 2026)
   - Event sourcing, CQRS, sagas, outbox pattern integration
   - Anti-pattern: dual writes without atomicity → use transactional outbox or Debezium CDC

2. **Event-Driven Architecture & Message Queues: 2026 Reference** — digitalapplied.com (Jun 2026)
   - Fowler's 4 event patterns taxonomy (notification, carried state transfer, event sourcing, event-driven architecture)
   - Outbox pattern treated as table stakes, not advanced technique
   - EventBridge schema governance, archive, replay

3. **Event Sourcing + CQRS Implementation Guide** — youngju.dev (Mar 2026)
   - Snapshot strategy: every 100–1000 events, async creation, replay >100ms threshold
   - Event schema changes are MORE dangerous than DB migrations
   - Event versioning: maintain backward compatibility ≥2 versions, upcasting during replay

4. **Event-Driven Architecture + CQRS + Event Sourcing Implementation** — youngju.dev (Mar 2026)
   - Event order inversion destroys data consistency (Kafka partition key misconfiguration)
   - Saga compensating transaction failures require DLQ + manual recovery
   - Event store infinite growth must be managed (snapshot + archiving)

5. **Event Sourcing and CQRS Architecture Diagrams (2026)** — architecturediagram.ai (Jun 2026)
   - Write path: Command → handler → aggregate → event store (synchronous)
   - Read path: Query → handler → read DB (separate)
   - Projection pipeline: Event store → subscriber → projection → read DB (async, eventually consistent)

6. **HostMyCode: Event-Driven Architecture Patterns 2026** (Apr 2026)
   - Event versioning: include version in metadata, maintain ≥2 versions backward compat
   - Schema registry for validation, upcasting for old events during replay
   - Circuit breakers isolate failing downstream services

7. **CQRS Pattern: Complete Guide (2026)** — systemdesignhandbook.com (Jul 2026)
   - CQRS does NOT require separate databases; code-level separation is the starting point
   - Eventual consistency: UI must handle delay gracefully (progress indicators, optimistic UI)

8. **CQRS in .NET: Theory to Production (2026)** — steve-bang.com (Mar 2026)
   - Apply CQRS when domain has real business logic, not for simple CRUD
   - MediatR for handler separation; unit testability improves with single responsibility

9. **Stream Processing 2026 Deep Dive** — youngju.dev (May 2026)
   - Three paradigms: Code-defined topology (Flink/Kafka Streams), Streaming DB (RisingWave/Materialize), Managed SaaS
   - Flink 2.0: disaggregated state (ForSt on S3), materialized tables, ML_PREDICT in SQL
   - Kafka 4.0: KRaft default, queues (KIP-932), faster rebalances
   - Arroyo: Rust monolithic single-binary stream processor
   - Biggest trend: state moving to cloud storage (S3-backed disaggregated state)

10. **Kafka Flink in 2026: Building Real-Time Stream Processing** — angeloprea.com (Apr 2026)
    - Flink 2.2.0 (Dec 2025): AI capabilities, materialized tables
    - Flink CDC 3.6.0: Flink 1.20.x/2.2.x support
    - Confluent Flink: unified serverless, Tableflow (Iceberg/Delta materialization)

11. **Event-Driven Architecture with Kafka and Flink 2026** — devstarsj.github.io (Mar 2026)
    - Events as immutable facts, not commands or state snapshots
    - Self-describing events with schema evolution

12. **RisingWave: Event-Driven Architecture in 2026** (Apr 2026)
    - Streaming database layer gives agents a queryable, always-current view
    - Existing EDA microservices don't need to change; add streaming DB layer on top

13. **Top Trends for Data Streaming 2026** — Kai Waehner (Apr 2026)
    - Diskless Kafka and Apache Iceberg as new foundation
    - Real-time analytics shifting into the streaming layer
    - IBM acquiring Confluent for enterprise generative AI

14. **CQRS: Read-Write Separation Design Pattern** — bibekkakati.com (Sep 2026)
    - Commands represent business intent, not property changes
    - Read models: optimized projections (relational, search, time-series)

15. **NoSQLSummer: Event Sourcing and CQRS for Distributed Systems 2026** (Jul 2026)
    - Events appended to log partitioned by aggregate identifier
    - Read models consume same stream, build independent projections
    - Snapshots at 1200 events keep query latency <5ms

---

## Defects Found in NeoTrix Architecture

### DEFECT-419-1: No Event Schema Versioning or Upcasting
**Severity**: HIGH
**Component**: `nt_core_event_bus.rs`
**2026 Finding**: Event schema changes are MORE dangerous than DB migrations. Production systems must maintain ≥2 versions backward compatibility and implement upcasting during replay.
**Current State**: `EventEnvelope` has `seq`, `source`, `timestamp_ms`, `event: CoreEvent` — no schema version field. `CoreEvent` is a Rust enum; serde deserialization will fail on version mismatch. No upcast pipeline exists.
**Gap**: When `CoreEvent` variants change (add fields, rename), old JSONL logs become undecipherable. The `replay()` and `replay_enveloped()` functions have no fallback for schema drift beyond the current enum definition.
**Suggestion**: Add `schema_version: u16` to `EventEnvelope`. Implement a registry of upcasters `(u16, u16) -> fn(CoreEvent) -> CoreEvent` that transform old events to current schema during replay.

### DEFECT-419-2: No Snapshot/Compaction Strategy for Event Log
**Severity**: MEDIUM
**Component**: `nt_core_event_bus.rs`
**2026 Finding**: Production event stores require snapshot strategy (every 100–1000 events) and archiving to prevent infinite growth. Replay time >100ms should trigger snapshot creation.
**Current State**: `EventBus::with_persistence()` appends to a JSONL file with no rotation, compaction, or snapshot mechanism. The `replay_enveloped()` loads the entire file into memory.
**Gap**: Long-running sessions produce unbounded event logs. No mechanism to compact old events into aggregate state summaries. Memory pressure grows linearly with event count.
**Suggestion**: Implement periodic snapshot creation (every N events or T seconds) that summarizes aggregate state. Archive old event segments to separate files. Replay starts from latest snapshot + delta.

### DEFECT-419-3: Broadcast Channel Has No Backpressure or Dead Letter Queue
**Severity**: MEDIUM
**Component**: `nt_core_event_bus.rs`
**2026 Finding**: Production event systems require DLQ for poison messages, retry logic, and idempotent handlers. Circuit breakers isolate failing consumers.
**Current State**: `broadcast::channel(1024)` — if consumers lag, they receive `Lagged(n)` and events are dropped silently. No DLQ, no retry, no circuit breaker for failing subscribers.
**Gap**: A slow or panicking consumer causes silent event loss. No mechanism to capture and reprocess failed events. No way to isolate a misbehaving layer subscriber.
**Suggestion**: Add optional DLQ channel for events that fail delivery. Implement subscriber health monitoring. Add circuit breaker pattern per-layer to temporarily disable failing subscribers.

### DEFECT-419-4: No CQRS Separation in Read/Write Paths
**Severity**: HIGH (architectural gap)
**Component**: Global architecture
**2026 Finding**: CQRS separates command and query concerns at the architectural level. Even code-level separation (without separate DBs) provides significant benefits for complex domains.
**Current State**: NeoTrix uses a single `CoreEvent` enum for all events. No separation between command events (state-mutating) and query events (read-only). No projection pipeline from events to read-optimized views.
**Gap**: All consumers process the same event stream regardless of whether they need write-side or read-side semantics. No materialized views or read-optimized projections derived from events.
**Suggestion**: Classify `CoreEvent` variants into Command (TaskSubmitted, GoalCompleted, BudgetExceeded) vs Query (status queries, metrics). Build projection functions that materialize event data into query-optimized caches.

### DEFECT-419-5: No Schema Registry or Event Contract Enforcement
**Severity**: MEDIUM
**Component**: `nt_core_event.rs`
**2026 Finding**: Schema registries (Confluent Schema Registry, AWS Glue) validate events at produce time. Data contracts prevent schema drift between producers and consumers.
**Current State**: `CoreEvent` variants are defined in Rust source code. No external schema registry. No validation layer between event emission and consumption. `serde_json` deserialization is the only schema enforcement.
**Gap**: Multi-module or multi-crate event producers can emit events that consumers cannot handle. No compile-time or runtime validation of event contracts.
**Suggestion**: Add a schema registry pattern: each `CoreEvent` variant has a registered schema hash. Consumers declare which schemas they accept. Emit warnings on schema mismatches.

### DEFECT-419-6: No Outbox Pattern for Dual-Write Atomicity
**Severity**: LOW (single-process)
**Component**: `nt_core_event_bus.rs`
**2026 Finding**: The transactional outbox pattern is table stakes for production event systems. Dual writes to database and message broker without atomicity guarantees are an anti-pattern.
**Current State**: EventBus writes to both in-memory broadcast and optional JSONL file. The file write and broadcast send are NOT atomic — a crash between them can lose events or create duplicates.
**Gap**: If `emit_from()` crashes after file write but before broadcast send, subscribers miss the event. If crash happens after broadcast but before file write, replay loses the event.
**Suggestion**: Use write-ahead logging: write to file FIRST (with fsync), then broadcast. On replay, file is the source of truth. Implement idempotency keys in EventEnvelope to deduplicate on replay.

### DEFECT-419-7: No Integration with Streaming Infrastructure (Kafka/Flink)
**Severity**: HIGH (scalability gap)
**Component**: Global architecture
**2026 Finding**: Kafka 4.0 (KRaft, queues), Flink 2.2 (AI/ML_PREDICT, materialized tables), RisingWave (streaming SQL for agents), and streaming databases are the 2026 state of the art.
**Current State**: NeoTrix is a single-process Rust application. No Kafka, Flink, or streaming DB integration. Event bus is in-process `tokio::broadcast`.
**Gap**: Cannot scale event processing across processes or machines. No persistent event log with replay from arbitrary positions. No stream processing for real-time analytics over event streams.
**Suggestion**: Design a pluggable event store trait that can be backed by: (a) in-process broadcast (current), (b) SQLite WAL, (c) Kafka topic, (d) streaming DB (Materialize/RisingWave). This preserves the single-process UX while enabling distributed deployment.

### DEFECT-419-8: No Disaggregated State for Cloud-Native Deployment
**Severity**: LOW (future concern)
**Component**: Memory/Knowledge layer
**2026 Finding**: Flink 2.0 introduced ForSt (S3-backed disaggregated state). RisingWave uses S3-backed state. The trend is stateless compute + cloud object storage.
**Current State**: NeoTrix state is local (SQLite, JSONL files, in-memory). No cloud-native state separation.
**Gap**: Cannot deploy NeoTrix in a serverless or containerized environment without local persistent storage. No way to share state across multiple agent instances.
**Suggestion**: Abstract state access behind a trait that can be backed by local FS, SQLite, or S3/object storage. This aligns with the "state moving to cloud storage" trend.

---

## Summary

| # | Defect | Severity | Domain |
|---|--------|----------|--------|
| 419-1 | No event schema versioning/upcasting | HIGH | Event Sourcing |
| 419-2 | No snapshot/compaction for event log | MEDIUM | Event Sourcing |
| 419-3 | No DLQ or backpressure on broadcast | MEDIUM | Event Sourcing |
| 419-4 | No CQRS read/write separation | HIGH | CQRS |
| 419-5 | No schema registry for event contracts | MEDIUM | CQRS |
| 419-6 | No outbox pattern for atomicity | LOW | Event Sourcing |
| 419-7 | No streaming infrastructure integration | HIGH | Streaming |
| 419-8 | No disaggregated state for cloud | LOW | Streaming |

**Recommendations Priority**:
1. **P0** (this iteration): DEFECT-419-1 (schema versioning) — immediate risk to replay correctness
2. **P0** (this iteration): DEFECT-419-4 (CQRS separation) — architectural foundation
3. **P1** (next iteration): DEFECT-419-7 (streaming integration) — scalability prerequisite
4. **P2** (backlog): DEFECT-419-2, 419-3, 419-5, 419-6, 419-8
