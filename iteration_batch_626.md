# Iteration Batch 626 — Event Sourcing / CQRS / Domain Events Research

**Date**: 2026-09-06  
**Predecessor**: Batch 625 (5 defects: no adaptive search space, no HW-aware optimization, no multi-objective scoring, no automated retraining/drift, no decision lineage)  
**Sources**: 12 web sources (2026) on event sourcing, CQRS, domain events, schema versioning

---

## Key 2026 Findings

### 1. Event Sourcing Maturity (2026)
- EventStoreDB rebranded to **KurrentDB** (v26.1): SQL access + new projection engine ([youngju.dev](https://www.youngju.dev/blog/architecture/2026-03-05-architecture-event-sourcing-cqrs-implementation.en), [calmops.com](https://calmops.com/software-engineering/cqrs-event-sourcing-practical-implementation/))
- **Apache Kafka 4.0+**: KRaft mode (no ZooKeeper), exactly-once semantics, ksqlDB for stream processing ([calmops.com](https://calmops.com/software-engineering/cqrs-event-sourcing-practical-implementation/))
- **Kafka is NOT an event store** — it's a streaming bus. Topic granularity fails for millions of aggregates. No cheap per-entity reads. No optimistic concurrency primitive ([sujeet.pro](https://sujeet.pro/articles/event-sourcing-deep-dive))
- **PostgreSQL 17**: native ANSI SQL MERGE, enhanced JSONB indexing (+40% vs PG16) ([johal.in](https://johal.in/implement-event-sourcing-kafka-37-postgresql-17-2026))
- **EU DORA regulation (2026)**: financial institutions must provide full immutable audit trail within 4 hours — event sourcing is the only compliant pattern ([johal.in](https://johal.in/implement-event-sourcing-kafka-37-postgresql-17-2026))

### 2. CQRS Production Patterns (2026)
- **Freshness Budget per read surface** — not one global lag metric. Each projection needs its own SLO ([abstractalgorithms.dev](https://abstractalgorithms.dev/cqrs-pattern-read-write-model-separation))
- **Transactional Outbox Pattern**: atomic write + event publication. Dual-write from request path is the #1 anti-pattern ([abstractalgorithms.dev](https://abstractalgorithms.dev/cqrs-pattern-read-write-model-separation), [Microsoft](https://learn.microsoft.com/en-us/azure/architecture/patterns/cqrs))
- **Read-After-Write Consistency**: return command results directly instead of querying read model ([oneuptime.com](https://oneuptime.com/blog/post/2026-01-30-cqrs-pattern-microservices/view))
- **CQRS without Event Sourcing is valid**: dual databases, read replicas, materialized views. Don't conflate the two patterns ([usamaqamar.space](https://www.usamaqamar.space/writing/cqrs-event-sourcing-production-architecture-guide))
- **Projection without owner, checkpoint, and recovery plan = operational debt** ([abstractalgorithms.dev](https://abstractalgorithms.dev/cqrs-pattern-read-write-model-separation))

### 3. Domain Event Schema Evolution (2026)
- **Events are permanent contracts** — never modify, only append ([proteanhq.com](https://docs.proteanhq.com/patterns/event-versioning-and-evolution/))
- **4 schema evolution strategies**: (1) Upcasting at consumer, (2) Lazy migration, (3) Dual-write transition, (4) Copy-transform ([letsbuildsolutions.com](https://letsbuildsolutions.com/blog/system-design/event-schema-versioning-in-practice-evolution-strategies-registry-patterns-and-breaking-change-management-for-production-event-systems/))
- **Envelopes + Metadata**: every event needs `event_id`, `correlation_id`, `causation_id`, `schema_version`, `source` ([oneuptime.com](https://oneuptime.com/blog/post/2026-01-30-event-schema-design/view))
- **Internal vs External events**: never expose raw domain events to external systems — use Anti-Corruption Layer ([dataexpert.io](https://www.dataexpert.io/blog/managing-domain-events-in-event-driven-architectures))
- **Event ownership governance**: every event type needs declared owner (team), deprecation with sunset dates, CI sunset checker ([letsbuildsolutions.com](https://letsbuildsolutions.com/blog/system-design/event-schema-versioning-in-practice-evolution-strategies-registry-patterns-and-breaking-change-management-for-production-event-systems/))
- **Tolerant Reader pattern**: consumers must handle unknown fields gracefully — forward compatibility as non-negotiable ([adhdecode.com](https://adhdecode.com/api-architecture/event-driven-and-reactive-apis/event-schema-design-evolution/))

---

## NEW Defects Found (Beyond Batch 625)

### DEFECT-626-01: No Event-Sourced Decision Log (Severity: CRITICAL)
**Evidence**: NeoTrix SEAL pipeline runs exploration/distillation/absorption cycles but has no append-only immutable event log of decisions. The KB stores snapshots but not the decision lineage as events.  
**2026 Pattern**: Every state change must be an immutable event. Current state is a derived view from replay. ([youngju.dev](https://www.youngju.dev/blog/architecture/2026-03-05-architecture-event-sourcing-cqrs-implementation.en))  
**Impact**: Cannot rebuild historical reasoning state. Cannot answer "what was the system thinking at iteration N?" Cannot replay past sessions for debugging.  
**Fix**: Introduce `NtEventStore` as append-only log. Each SEAL cycle emits events: `CycleStarted`, `ExplorationCompleted`, `DistillationApplied`, `SelfTestRan`, `AbsorptionCommitted`. Current system health is a projection, not stored state.

### DEFECT-626-02: No Optimistic Concurrency Control (Severity: HIGH)
**Evidence**: No version-stamp mechanism on KB writes. Multiple agents can overwrite each other's state.  
**2026 Pattern**: `expectedVersion` parameter prevents concurrent corruption. First write wins, second gets `WrongExpectedVersionError` and retries. ([youngju.dev](https://www.youngju.dev/blog/architecture/2026-03-05-architecture-event-sourcing-cqrs-implementation.en), [calmops.com](https://calmops.com/software-engineering/cqrs-event-sourcing-practical-implementation/))  
**Impact**: Race conditions on KB state. Lost updates under concurrent agent writes.  
**Fix**: Add `expected_version` field to all KB write operations. Aggregate-level version checks. Retry with reload on version mismatch.

### DEFECT-626-03: No Idempotency Guarantees on Event Processing (Severity: HIGH)
**Evidence**: No event deduplication mechanism. Replays can corrupt state.  
**2026 Pattern**: "The most important aspect of projection implementation is idempotency. Without idempotent handling, data becomes corrupted." ([youngju.dev](https://www.youngju.dev/blog/architecture/2026-03-05-architecture-event-sourcing-cqrs-implementation.en))  
**Impact**: Duplicate processing during replayer restarts or bus redelivery causes data corruption.  
**Fix**: Every event has `event_id` (UUID). Projection checkpoint table tracks last processed event ID. Processors must use `ON CONFLICT ... DO UPDATE` or equivalent.

### DEFECT-626-04: No Event Envelope Standard (Severity: MEDIUM)
**Evidence**: No standardized event envelope with metadata. No `correlation_id`, `causation_id`, `schema_version` fields.  
**2026 Pattern**: Standard envelope fields: `event_id`, `event_type`, `timestamp`, `source`, `correlation_id`, `schema_version`, `payload` ([oneuptime.com](https://oneuptime.com/blog/post/2026-01-30-event-schema-design/view))  
**Impact**: Cannot trace causal chains across modules. Cannot debug cross-domain interactions. No audit trail metadata.  
**Fix**: Define `NtEventEnvelope` struct: `{ event_id: Uuid, event_type: String, timestamp: DateTime<Utc>, source: NtDomain, correlation_id: Uuid, causation_id: Option<Uuid>, schema_version: Semver, payload: serde_json::Value }`.

### DEFECT-626-05: No Upcasting for Schema Evolution (Severity: MEDIUM)
**Evidence**: No mechanism to handle old event formats when schema changes. KB structures evolve but old records aren't transformed.  
**2026 Pattern**: Upcasting transforms old events to new schema at read time. Store is never modified. ([youngju.dev](https://www.youngju.dev/blog/architecture/2026-03-05-architecture-event-sourcing-cqrs-implementation.en), [proteanhq.com](https://docs.proteanhq.com/patterns/event-versioning-and-evolution/))  
**Impact**: Schema changes break old records. Cannot replay historical events after structure changes.  
**Fix**: Define `NtUpcaster` trait. Chain of upcasters: v1→v2→v3. Applied during deserialization. Each upcaster is a pure function transforming raw payload.

### DEFECT-626-06: No Projection Pattern for Read Models (Severity: MEDIUM)
**Evidence**: All reads go through the same path as writes. No denormalized read-optimized views.  
**2026 Pattern**: Projections build query-optimized views from event streams. Multiple projections for different use cases (dashboard, analytics, search). ([youngju.dev](https://www.youngju.dev/blog/architecture/2026-03-05-architecture-event-sourcing-cqrs-implementation.en))  
**Impact**: Performance degrades as data grows. No ability to optimize reads independently.  
**Fix**: Define `NtProjection` trait. Each projection subscribes to event stream, builds its own read model. Checkpoint tracking per projection.

### DEFECT-626-07: No Freshness Budget Definition (Severity: MEDIUM)
**Evidence**: No SLO for read-after-write consistency. Users may see stale data without knowing.  
**2026 Pattern**: "Freshness budgets and replay tooling matter as much as schema design. Projection-specific lag is a better signal than one global event-bus metric." ([abstractalgorithms.dev](https://abstractalgorithms.dev/cqrs-pattern-read-write-model-separation))  
**Impact**: UI can show stale state. Users don't know if data is current.  
**Fix**: Define `FreshnessBudget { surface: String, max_lag_ms: u64 }`. Expose lag metric per projection. Return 409 CONFLICT if read model not caught up within budget.

### DEFECT-626-08: No Internal/External Event Separation (Severity: MEDIUM)
**Evidence**: No Anti-Corruption Layer between domain events and external consumers. Raw domain events could leak to external systems.  
**2026 Pattern**: "Never expose raw domain events directly to external systems. Use a handler to transform internal domain events into integration events." ([dataexpert.io](https://www.dataexpert.io/blog/managing-domain-events-in-event-driven-architectures))  
**Impact**: Internal schema changes break external consumers. Tight coupling between bounded contexts.  
**Fix**: Define `NtIntegrationEvent` with stable public schema. ACL translates internal events to integration events. Internal events can evolve freely.

### DEFECT-626-09: No Transactional Outbox Pattern (Severity: LOW)
**Evidence**: No mechanism to atomically write state + publish event. Dual-write risk.  
**2026 Pattern**: "Dual-writing the read model from the request path removes most of CQRS's safety benefits." Transactional outbox ensures atomic write + event publication. ([abstractalgorithms.dev](https://abstractalgorithms.dev/cqrs-pattern-read-write-model-separation))  
**Impact**: Write succeeds but event publication fails → downstream silently misses event.  
**Fix**: Write event to outbox table in same transaction as state change. CDC or poller publishes from outbox.

### DEFECT-626-10: No Compensating Event Pattern (Severity: LOW)
**Evidence**: No mechanism to reverse or correct past events through new events.  
**2026 Pattern**: "The only way to update an entity or undo a change is to add a compensating event to the event store." ([Microsoft](https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing))  
**Impact**: Cannot correct erroneous state without overwriting history.  
**Fix**: Define compensating events: `SelfTestFailed → SelfTestResultCorrected`. Never modify original event. Append correction.

---

## Defect Summary

| # | Defect | Severity | Novelty |
|---|--------|----------|---------|
| 626-01 | No event-sourced decision log | CRITICAL | NEW — extends defect (5) decision lineage from batch 625 |
| 626-02 | No optimistic concurrency control | HIGH | NEW — not in batch 625 |
| 626-03 | No idempotency guarantees | HIGH | NEW — not in batch 625 |
| 626-04 | No event envelope standard | MEDIUM | NEW — not in batch 625 |
| 626-05 | No upcasting for schema evolution | MEDIUM | NEW — not in batch 625 |
| 626-06 | No projection pattern for read models | MEDIUM | NEW — not in batch 625 |
| 626-07 | No freshness budget definition | MEDIUM | NEW — not in batch 625 |
| 626-08 | No internal/external event separation | MEDIUM | NEW — not in batch 625 |
| 626-09 | No transactional outbox pattern | LOW | NEW — not in batch 625 |
| 626-10 | No compensating event pattern | LOW | NEW — not in batch 625 |

---

## Sources Cited

1. youngju.dev — Event Sourcing + CQRS Implementation Guide (2026-03-05)
2. calmops.com — CQRS + Event Sourcing Practical Implementation (2026-03-06)
3. sujeet.pro — Event Sourcing Deep Dive (2026-02-04)
4. hizidan.com — Event Sourcing vs EDA Production Patterns (2026-06-06)
5. skillions.in — Event Sourcing in 2026 (2026-08-11)
6. Microsoft Azure — Event Sourcing Pattern (2026-03-27)
7. appscale.blog — Event Sourcing Snapshots & Projections 2026 (2026-05-30)
8. johal.in — Event Sourcing with Kafka 3.7 + PostgreSQL 17 (2026-05-04)
9. martinfowler.com — CQRS (2011 reference)
10. Microsoft Azure — CQRS Pattern (2025-02-21)
11. abstractalgorithms.dev — CQRS Read/Write Model Separation (2026-03-13)
12. iamamanuragh.in — CQRS Read/Write Models (2026-06-14)
13. oneuptime.com — CQRS in Microservices (2026-01-30)
14. usamaqamar.space — CQRS + Event Sourcing Production Guide (2026-05-22)
15. letsbuildsolutions.com — Event Schema Versioning in Practice (2026-05-03)
16. proteanhq.com — Event Versioning and Evolution
17. oneuptime.com — Event Schema Design (2026-01-30)
18. fullstackdeveloper.it — Event Design Contracts (2026-02-19)
19. softwarepatternslexicon.com — Versioning Strategies (2026-03-23)
20. adhdecode.com — Event Schema Design and Evolution (2026-03-19)
21. dataexpert.io — Managing Domain Events (2026-06-07)
