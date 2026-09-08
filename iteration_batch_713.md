# Iteration Batch 713 — Event Sourcing / CQRS / Event Schema Research

**Date**: 2026-09-06
**Prior Batch**: 712 (Continuous Training trigger, three-layer serving, content-addressable immutability, agent context feature type, output schema validation)
**Sources**: 8 event sourcing references, 6 CQRS references, 7 event schema/versioning references

---

## 1. Event Sourcing — NEW Defects

### DEFECT-ES-1: SEAL Pipeline Is Not Event-Sourced (State Beyond Recovery)

**Finding**: Every 2026 production event sourcing guide (Youngju.dev, Calmops, HiZidan, AppScale, Skillions) confirms: event sourcing stores state changes as immutable append-only events; current state is derived by replay. NeoTrix SEAL pipeline stages (explore→distill→absorb→close) mutate `SEALPipelineInner` state in-memory with no durable event log. If the process crashes between stages, all progress is lost. The pipeline has no ability to answer "what happened during cycle N?"

**Impact**: No temporal audit of evolution cycles. No ability to replay or debug past SEAL runs. No crash recovery — a crash mid-cycle loses the entire cycle's work.

**Action**: Design `SealEvent` enum (SealStageStarted, SealStageCompleted, SealStageFailed, SealFruitHarvested) with append-only storage in KB `event_store` namespace. Each SEAL cycle becomes an aggregate with `cycle_id` as stream key. Snapshots at cycle boundaries for fast restore.

---

### DEFECT-ES-2: ConsciousnessTree Growth Cycles Lack Aggregate Boundaries

**Finding**: Event sourcing requires well-defined aggregate boundaries — the invariant consistency unit. ConsciousnessTree runs 6-stage growth cycles (Soil→Roots→Trunk→Branches→Fruits→Core) but treats the entire tree as a single mutable object (`ConsciousnessTreeInner`). There is no aggregate boundary per growth cycle. Multiple concurrent growth cycles (e.g., background loop + manual tick) can interleave mutations without optimistic concurrency control.

**Impact**: Race conditions in tree state updates. No `expectedVersion` concurrency guard (the core mechanism in event sourcing per Calmops/Youngju). Cannot safely replay or snapshot individual growth cycles.

**Action**: Each growth cycle should be a separate aggregate with `cycle_id`. Apply optimistic concurrency via `expectedVersion` on the event store append. Failed concurrent appends get `WrongExpectedVersionError` and retry with fresh state replay.

---

### DEFECT-ES-3: EventBus Is Fire-and-Forget, Not Event Store

**Finding**: NeoTrix `EventBus` (`nt_core_event_bus.rs`) is a tokio broadcast channel — transient, in-memory, no persistence guarantee. Event sourcing requires durable append-only storage where events are never lost. The 2026 KurrentDB/Kafka split is explicit: "Use Kafka as transport, not as the store. Pair it with a purpose-built event store" (Sujeet Jaiswal). NeoTrix has the transport (EventBus) but no event store.

**Impact**: Events lost on restart. No ability to rebuild projections from history. No temporal queries. No disaster recovery via event replay.

**Action**: Introduce `EventStore` trait in NT-MEMORY with append-only semantics. EventBus becomes the distribution layer (transport). EventStore becomes the source of truth. Projection handlers subscribe to EventBus but checkpoint against EventStore positions.

---

### DEFECT-ES-4: No Snapshot Strategy for Aggregate Optimization

**Finding**: When aggregates accumulate thousands of events, replay becomes expensive. Every 2026 guide recommends snapshotting: save materialized state at intervals, replay only the tail. NeoTrix ConsciousnessTree can accumulate hundreds of growth cycles. `SEALPipelineInner` accumulates stage results. No snapshot mechanism exists.

**Impact**: Degraded performance as system runs longer. Full replay of all historical events on every load becomes prohibitive.

**Action**: Implement `AggregateSnapshot` trait. Snapshots stored alongside events in EventStore. On load: find latest snapshot, replay only events after snapshot position. Snapshot schema versioning for forward compatibility (per HiZidan AppScale benchmark: snapshot at every 100 events is optimal).

---

### DEFECT-ES-5: No Idempotency Guarantee on Projection Handlers

**Finding**: Event sourcing subscribers will see duplicates (restart, rebalance, replay). Projection handlers MUST be idempotent — the single most emphasized requirement across all 2026 sources. NeoTrix background loop handlers (`handlers_consciousness.rs`, `handlers_maintenance.rs`) process EventBus events without idempotency keys or `ON CONFLICT` logic.

**Impact**: Duplicate processing corrupts read models. Duplicate experience entries. Duplicate skill crystallization. Duplicate health calculations.

**Action**: Every projection handler must: (1) extract `eventId` from event metadata, (2) use `ON CONFLICT(event_id) DO NOTHING` or equivalent upsert, (3) checkpoint position atomically with projection write.

---

## 2. CQRS — NEW Defects

### DEFECT-CQRS-1: No Read/Write Model Separation in KB

**Finding**: CQRS separates command (write) models from query (read) models. NeoTrix KB (`nt_memory_kb`) uses a single SQLite database for both writes (node creation, edge updates, experience absorption) and reads (semantic search, health queries, capability lookups). The write schema is normalized; read queries require denormalized joins. GitHub achieves 5.5M QPS by separating reads from writes at schema-domain level (HLD Handbook 2026).

**Impact**: Read contention on write transactions. Complex joins for dashboard queries. Cannot independently scale read vs write workload.

**Action**: Define `WriteModel` (normalized KB nodes/edges) and `ReadModel` projections (denormalized materialized views for health dashboard, capability lookup, experience search). Projections built asynchronously from event stream. Read from projections; write to event store.

---

### DEFECT-CQRS-2: ConsciousnessTask Has No Command/Query Split

**Finding**: `consciousness_task` accepts a single `instruction` string and returns a result. CQRS requires: Commands express intent to change state (handled by write model), Queries return DTOs without mutation (handled by read model). The current API conflates both — a single endpoint handles both "update my todo" and "what's my consciousness state?"

**Impact**: Cannot optimize read path independently. Read queries that only need status waste resources on write-path validation. Write commands that need validation cannot be separated from read-side display logic.

**Action**: Split into `ConsciousnessCommand` (tick, absorb, evolve — mutates state) and `ConsciousnessQuery` (status, tree, history — returns projections). Commands go through event store; queries go through read models.

---

### DEFECT-CQRS-3: No Read Model Projections for Health Dashboard

**Finding**: CQRS read models are "freely designed according to projection requirements" (Youngju.dev). NeoTrix consciousness status queries (`consciousness_status`) reconstruct health from raw KB data on every call. No pre-computed materialized views exist for the dashboard.

**Impact**: O(n) scan on every status query. Dashboard latency grows with KB size. Cannot serve multiple consumers (CLI, Tauri, API) with different projection shapes.

**Action**: Create `HealthProjection` — a denormalized view updated by async projection from consciousness events. Fields: `phi`, `coherence`, `fog_level`, `mars_system1_active`, `branch_health[11]`, `fruit_count`, `last_cycle_timestamp`. Rebuilt from event replay if corrupted.

---

### DEFECT-CQRS-4: No Transactional Outbox for EventBus Publishing

**Finding**: "Writing the event then publishing in two operations is the classic distributed bug — append succeeds, publish fails, downstream silently misses the event" (Sujeet Jaiswal 2026). NeoTrix writes to KB and emits to EventBus as separate operations. If KB write succeeds but EventBus emit fails (e.g., channel full), the event is lost forever.

**Impact**: Silent data loss. Projections drift from source of truth. Event sourcing becomes unreliable — cannot rebuild state from incomplete event history.

**Action**: Implement transactional outbox pattern: write event to `outbox` table in same transaction as KB mutation. Background poller reads outbox, publishes to EventBus, marks as published. Guarantees at-least-once delivery with idempotent consumers.

---

## 3. Event Schema Versioning — NEW Defects

### DEFECT-SCHEMA-1: No Event Schema Registry for Cross-Session Consistency

**Finding**: "Schema drift in event-driven systems is one of the most common sources of production incidents" (Let's Build 2026). "The core problem is that events are not API calls... the schema contract between producer and consumer has no enforcement point by default." NeoTrix events (`CoreEvent` enum) have no schema registry, no version tracking, no compatibility checks. Cross-session event contracts are implicit.

**Impact**: Adding a field to `CoreEvent` silently breaks downstream consumers. No way to detect schema incompatibility before deployment. No migration path for old events.

**Action**: Implement `EventSchemaRegistry` in NT-MEMORY. Each event type registered with: schema definition (JSON Schema or Rust type), version number, compatibility mode (BACKWARD by default). CI gate checks compatibility before merge. Schema ID embedded in event envelope for consumer resolution.

---

### DEFECT-SCHEMA-2: No Upcasting Mechanism for Event Evolution

**Finding**: "The most important principle: never modify existing events in the store. Transform at read time (upcasting) or define new event types" (Calmops 2026). NeoTrix has no upcasting chain. If `CoreEvent` gains a new variant or field, old serialized events in KB will fail to deserialize correctly.

**Impact**: Historical events become unreadable after schema changes. Cannot replay old SEAL cycles. Event store grows stale and unusable.

**Action**: Implement `Upcaster` trait: `fn upcast(event: RawEvent, from_version: u32, to_version: u32) -> Result<RawEvent>`. Chain of upcasters registered per event type. Applied transparently during event deserialization. Keep upcasters colocated with schema definitions (per Let's Build 2026 recommendation).

---

### DEFECT-SCHEMA-3: No CloudEvents Metadata on Domain Events

**Finding**: CloudEvents specification provides standardized envelope: `specversion`, `id`, `source`, `type`, `time`, `datacontenttype`, `traceparent`. NeoTrix events lack `correlationId` and `causationId` (the two most critical fields for distributed tracing per Youngju.dev checklist). Without these, cannot trace which command caused which event.

**Impact**: Cannot reconstruct causal chains across modules. Debugging distributed event flows becomes impossible. No request tracing across ConsciousnessTree → SEAL → KB pipeline.

**Action**: Define `EventMetadata` struct with: `event_id` (UUID), `correlation_id` (ties related events), `causation_id` (event that caused this one), `timestamp`, `source_module`, `schema_version`. Embedded in every event envelope. Required field — missing metadata = rejected event.

---

### DEFECT-SCHEMA-4: No CI Schema Validation Gate

**Finding**: "Schema validation belongs in the pipeline before events reach the broker" (Let's Build 2026). Confluent Schema Registry rejects incompatible schemas at registration time. NeoTrix has no CI check for event schema compatibility. A breaking change to `CoreEvent` can merge and deploy silently.

**Impact**: Runtime deserialization failures. Silent data corruption. Consumer drift without detection.

**Action**: Add CI step: extract event schemas from Rust types, check backward compatibility against previous version (add optional field = OK, remove required field = FAIL, change type = FAIL). Fail build on incompatible changes. Schema changes require PR with compatibility rationale.

---

### DEFECT-SCHEMA-5: No Dead Letter Queue for Failed Projection Events

**Finding**: "Any rejected or schema-invalid message should land in a monitored DLQ with structured metadata. Track metrics: schema-registration errors, compatibility failures, publish rejections, and consumer deserialization errors" (DEV Community 2026). NeoTrix EventBus has no DLQ mechanism. Failed event processing is silently dropped.

**Impact**: Permanent data loss on projection failures. No visibility into which events failed processing. Cannot retry or manually fix failed events.

**Action**: Implement `DeadLetterQueue` for EventBus. Events that fail projection (schema validation, handler panic, timeout) routed to DLQ with: original event, error reason, retry count, timestamp. DLQ monitor alerts on accumulation. Manual retry or discard via CLI.

---

## Summary Table

| ID | Category | Defect | Severity | Effort |
|---|---|---|---|---|
| DEFECT-ES-1 | Event Sourcing | SEAL not event-sourced | CRITICAL | L |
| DEFECT-ES-2 | Event Sourcing | No aggregate boundaries on ConsciousnessTree | HIGH | M |
| DEFECT-ES-3 | Event Sourcing | EventBus is transport, not event store | CRITICAL | L |
| DEFECT-ES-4 | Event Sourcing | No snapshot strategy | MEDIUM | M |
| DEFECT-ES-5 | Event Sourcing | No idempotency on projections | HIGH | S |
| DEFECT-CQRS-1 | CQRS | KB has no read/write separation | HIGH | L |
| DEFECT-CQRS-2 | CQRS | ConsciousnessTask has no command/query split | MEDIUM | M |
| DEFECT-CQRS-3 | CQRS | No health dashboard projections | MEDIUM | M |
| DEFECT-CQRS-4 | CQRS | No transactional outbox | HIGH | M |
| DEFECT-SCHEMA-1 | Schema | No event schema registry | CRITICAL | L |
| DEFECT-SCHEMA-2 | Schema | No upcasting mechanism | HIGH | M |
| DEFECT-SCHEMA-3 | Schema | No CloudEvents metadata | HIGH | S |
| DEFECT-SCHEMA-4 | Schema | No CI schema validation | HIGH | S |
| DEFECT-SCHEMA-5 | Schema | No DLQ for failed events | MEDIUM | S |

---

## Sources Cited

1. **Youngju.dev** (2026-03-05) — "Event Sourcing and CQRS Pattern Implementation Guide" — EventStoreDB/Kurrent, snapshot strategies, upcasting, idempotent projections
2. **Calmops/Larry Qu** (2026-03-06) — "CQRS and Event Sourcing: Practical Implementation Patterns 2026" — KurrentDB 26.1, Kafka 4.0 KRaft, snapshot schema versioning
3. **Sujeet Jaiswal** (2026-02-04) — "Event Sourcing — Deep Dive" — LMAX/Netflix/Uber production patterns, Kafka as transport not store, deterministic replay
4. **HiZidan** (2026-06-06) — "Event Sourcing vs Event-Driven Architecture" — PostgreSQL event store schema, EDA vs ES distinction, GDPR erasure
5. **AppScale Blog** (2026-05-30) — "Event Sourcing in Production: Snapshots & Projections" — Production benchmarks, snapshot cadence
6. **Skillions** (2026-08-11) — "Event Sourcing in 2026" — Architecture components, production readiness
7. **Encore.dev** (2026-05-03) — "Event-Driven Architecture in 2026" — Broker comparison, when not to use EDA
8. **Microsoft Azure** (2025-02-21) — "CQRS Pattern" — Read/write model separation, transactional outbox
9. **ScopeForged** (2026-08-02) — "CQRS Pattern Explained" — Mild vs strong CQRS, CDC via Debezium
10. **HLD Handbook** (2026-05-11) — "CQRS: Separating Reads from Writes" — GitHub 5.5M QPS, schema-domain partitioning, ProxySQL
11. **Usama Qamar** (2026-05-22) — "CQRS + Event Sourcing: Production Architecture Guide" — Combined pattern, projection rebuilds
12. **Let's Build** (2026-05-03) — "Event Schema Versioning in Practice" — Upcasting, lazy migration, CI validation, sunset checker
13. **Confluent Docs** (2026-08-20) — "Schema Evolution & Compatibility Types" — BACKWARD/FORWARD/FULL modes, transitive compatibility
14. **OneUptime** (2026-01-30) — "How to Implement Event Versioning Strategies" — Version in metadata, upcaster registry, weak schema
15. **Engineer Palsu** (2026-06-19) — "Designing Event Schemas and Event Versioning" — JSON Schema/Avro, semantic versioning, backward compatibility
16. **Codelit** (2026-03-29) — "Schema Registry — Event Schema Evolution" — Confluent/Apicurio, CI validation, never rename fields
17. **DEV Community/beefedai** (2026-05-09) — "Event Contracts: Schema Design, Versioning & Governance" — AsyncAPI, CloudEvents, governance workflow
