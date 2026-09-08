# Iteration Batch 581 — Stream Processing / Event-Driven / Message Queue Defect Analysis

**Date**: 2026-09-06
**Iteration**: 581 of 10,000+
**Research Scope**: Stream processing, event-driven architecture, message queue technology landscape 2026
**Baseline**: Batch 580 (Tower middleware ordering, circuit breaker, MCP API governance, semantic cache, in-process rate limits)

---

## Executive Summary

Batch 581 identified **12 new defects** across three domains. All are **NEW vs Batch 580** — none overlap with Tower middleware ordering, circuit breaker budget, MCP API-product governance, semantic LLM cache, or in-process rate limits.

---

## I. STREAM PROCESSING DEFECTS (4 new findings)

### Defect 581-S1: Flink Checkpoint-Backpressure Feedback Loop (Bounded Error Budget)

**Source**: iotdigitaltwinplm.com (2026-06-27), alper-korukcu.medium.com (2026-03-14)

**Finding**: Flink's checkpoint mechanism has a vicious cycle under backpressure: when a job backpressures, checkpoints slow down → slow checkpoints timeout → recovery from stale state triggers → recovery causes more backpressure. Unaligned checkpoints help but don't eliminate the cycle. A VLDB 2025 paper showed checkpoints completing within 3 seconds regardless of state size, but that's under normal conditions — under sustained backpressure the cycle degrades into cascading recovery storms.

**Defect**: No bounded error budget for checkpoint timeout recovery cycles. When backpressure exceeds threshold, the system enters a degenerate recovery loop where each recovery attempt amplifies the load that caused the backpressure. This is structurally similar to Batch 580's circuit breaker finding but operates at the **infrastructure level** (stream processor internal recovery) rather than the application level (LLM API calls).

**Fix Required**: Implement checkpoint-backpressure circuit breaker: when checkpoint lag exceeds 2x the target interval for 3 consecutive checkpoints, the operator chain must shed load (drop non-critical windows, switch to approximate aggregation) to break the feedback loop.

---

### Defect 581-S2: Kafka 4.0 EOS-v1 Removal — Silent Dual-Write Corruption Window

**Source**: kafka.apache.org/40 (2026-03-20), blackflow.co.uk (2026-06-11), andrewbaker.ninja (2026-02-17)

**Finding**: Kafka 4.0 removed ZooKeeper and removed `exactly_once_v1` as a supported transactional mode — only `exactly_once_v2` remains. During the ZK-to-KRaft migration window (dual-write phase), the system writes metadata to both KRaft and ZooKeeper simultaneously. KAFKA-20488 documents a rollback that left partitions offline due to controller epoch mismatch. KAFKA-19480 documented a malformed `/migration` znode blocking migration.

**Defect**: The migration window creates a **transient dual-write semantic gap** where producer transactions can succeed on the KRaft side but fail on the ZooKeeper side (or vice versa), causing **silent offset corruption**. Consumer groups that commit offsets during this window may have offsets recognized by one authority but not the other, leading to replay duplication or event loss after finalization.

**Fix Required**: During KRaft migration, all producer transactions must include a migration-phase marker (metadata version check). Consumer offset commits must be validated against both authorities before acknowledgment. Post-migration, consumer offsets must be reconstructed from the KRaft authoritative log, not from in-memory state.

---

### Defect 581-S3: Spark Structured Streaming Latency Floor Creates Phantom SLA Compliance

**Source**: iotdigitaltwinplm.com (2026-06-27), streamkap.com (2026-03-12), narcismiclaus.com (2026-02-02)

**Finding**: Spark Structured Streaming's micro-batch model has a structural latency floor of 100ms-seconds (academic benchmarks: 510-570ms for basic workloads). Databricks introduced Real-Time Mode (public preview, August 2025) achieving P99 of 15-300ms, but it's limited to stateless Scala queries, not open source, and only on Databricks.

**Defect**: Systems that deploy Spark Structured Streaming for "real-time" workloads often achieve **SLA phantom compliance** — dashboards report sub-second latency on average, but P99 misses the target by 5-10x due to micro-batch scheduling overhead. The latency floor is architectural (physics, not configuration), but monitoring tools that report average latency mask the P99 violations. This creates a situation where the system is formally "compliant" but operationally broken for latency-sensitive consumers.

**Fix Required**: For any stream processing path with a P99 latency SLA, enforce a hard architectural gate: if SLA < 500ms, Spark Structured Streaming is excluded from the technology set. Flink or Kafka Streams required. Latency monitoring must report P99, P99.9, and max — never just average.

---

### Defect 581-S4: RisingWave/Materialize Streaming DB Bypasses Stream Processor Governance

**Source**: youngju.dev (2026-05-16), devstarsj.github.io (2026-03-29)

**Finding**: 2026's fastest-growing pattern is "streaming databases" — RisingWave (Postgres wire protocol) and Materialize (Differential Dataflow) let backend engineers write `CREATE MATERIALIZED VIEW` to get real-time incremental views without writing any stream processing code. This bypasses the stream processor layer entirely.

**Defect**: Streaming databases introduce a **shadow data pipeline** that circumvents the governance, monitoring, and error-handling infrastructure built around Flink/Kafka Streams. Teams write `CREATE MATERIALIZED VIEW` without realizing they've created a stateful streaming job with unbounded state growth, no checkpoint governance, and no backpressure visibility. The Materialized View becomes a black box that the platform team can't monitor, scale, or failover.

**Fix Required**: All streaming database objects (RisingWave materialized views, Materialize materialized views) must be registered in the same capability registry as Flink jobs. State size, consumer lag, and refresh rate must be monitored through the same heartbeat aggregator. Materialized views exceeding 10GB state must trigger governance review.

---

## II. EVENT-DRIVEN ARCHITECTURE DEFECTS (5 new findings)

### Defect 581-E1: Event Sourcing Projection Gap — Silent Permanent Data Loss

**Source**: youngju.dev (2026-03-07), blog.ecotone.tech (2026-04-27), kloudvin.com (2026-06-08)

**Finding**: In event-sourced CQRS systems, projections that use a global sequence number for position tracking can **permanently and silently lose events** under concurrent writes. When events from different aggregates interleave in a global stream, concurrent transactions create gaps in the visible sequence. A projection that reads `global_position` and advances past a gap (because the gap is invisible at read time) **permanently skips the event that created the gap**. No error, no log entry, no exception. The read model diverges from the Event Store forever.

**Defect**: This is a **silent permanent data loss** defect — the most dangerous class of bug. Unlike Batch 580's rate limiting leak (which leaks traffic), this leaks **data integrity** permanently. The projection will never recover unless manually rebuilt from scratch. Most event sourcing libraries have this as default behavior because dev environments run one request at a time and never expose the race condition.

**Fix Required**: 
1. Projections must track position per aggregate (partitioned tracking), not globally. Within a single aggregate, optimistic locking guarantees strict version ordering — no gaps possible.
2. If global tracking is mandatory, implement gap detection: if `next_expected_version != observed_version`, block until the gap fills (with timeout + DLQ fallback).
3. All projection handlers must include an idempotency guard using `eventId` as dedup key, even if the projection library claims ordering guarantees.

---

### Defect 581-E2: Event Schema Immutability vs GDPR Right-to-Erasure Conflict

**Source**: youngju.dev (2026-03-07), pnguyen.au (2026-03-07), sujeet.pro (2026-02-04)

**Finding**: Event sourcing's core contract is **events are immutable and never deleted**. GDPR's right-to-erasure (Article 17) requires deletion of personal data on request. These are structurally contradictory. The standard solution is "crypto-shredding" — encrypt PII with a per-user key, then delete the key. But this requires every event type to have been designed with crypto-shredding in mind from day one.

**Defect**: Systems that adopted event sourcing without designing for crypto-shredding from the start face an **irreconcilable compliance gap**. Events containing PII (customer names, email addresses, shipping addresses) are stored in plain text in the event store. Retroactive application of crypto-shredding requires replaying the entire event history, re-encrypting every event that contains the user's PII, and updating all projections — an operation that can take hours to days on large stores and creates a new version of the event history that no longer matches the original append-only contract.

**Fix Required**: 
1. PII must be stored in a separate sidecar table referenced by event ID, never embedded in the event payload itself.
2. If PII is already in event payloads, implement a PII quarantine zone: flag events containing PII for a dedicated projection that writes PII to the sidecar table and replaces the in-event PII with a hash reference. Schedule this as a mandatory migration for all event-sourced domains.

---

### Defect 581-E3: Kafka-as-Event-Store Anti-Pattern — No Optimistic Concurrency

**Source**: sujeet.pro (2026-02-04), serialized.io (referenced in multiple 2026 articles)

**Finding**: Multiple 2026 sources confirm Kafka is widely misused as an event store for event sourcing. The Serialized.io article "Apache Kafka is not for Event Sourcing" is cited by at least 3 independent 2026 analyses. Kafka lacks: (1) entity-level stream reads ("load events for entity X"), (2) optimistic concurrency ("append only if version is still N"), (3) log compaction destroys history. Teams bolt on a coordinating database for concurrency, defeating the purpose.

**Defect**: Using Kafka as the event store creates a **false sense of event sourcing correctness**. The write path (Kafka append) and the concurrency control (external database) are in different systems, creating a split-brain scenario where Kafka has events that the database didn't authorize, or the database authorized events that Kafka didn't receive. The "Kafka is the source of truth" claim becomes a lie — the actual source of truth is split across two systems.

**Fix Required**: 
1. For event sourcing, use a purpose-built event store (KurrentDB) or PostgreSQL with an append-only events table. Kafka is the transport/event bus, not the event store.
2. If Kafka is already used as the event store, implement the transactional outbox pattern: write events to the database in the same transaction as state changes, then relay to Kafka asynchronously. Never write directly to Kafka as the primary event store.

---

### Defect 581-E4: Saga Compensation Stack Overflow Under Cascading Failures

**Source**: system-design.space (2026-05-01), youngju.dev (2026-03-06)

**Finding**: In Saga choreography, when a compensating transaction fails (e.g., payment refund fails because the payment gateway is down), the failure cascades backward through the Saga chain. Each failed compensation triggers another compensation attempt, creating a recursive chain. Under cascading failures, this can exhaust the DLQ, block all related aggregates, and require manual intervention for every stuck Saga instance.

**Defect**: Saga compensation lacks a **termination depth limit** — analogous to stack overflow in recursive functions. When compensation fails, the system has no bounded worst case. A 5-step Saga where step 4 fails triggers compensation for steps 3, 2, 1 — if step 3's compensation also fails, you get compensation-for-compensation attempts. With 100 concurrent Sagas all failing at the same step, you get 100 × 4 = 400 compensating transactions in flight, potentially overwhelming the DLQ and blocking the broker.

**Fix Required**: 
1. Implement compensation depth limit: maximum 2 levels of compensation nesting. If a compensation fails, mark the Saga as FAILED_MANUAL and stop retrying.
2. Compensating transactions must be idempotent and timeout-bounded (max 30s per attempt).
3. DLQ must have per-Saga-type capacity limits to prevent one failing Saga type from monopolizing the DLQ.

---

### Defect 581-E5: Event Projection Rebuild Time Explosion — Blue/Green Not Enough

**Source**: youngju.dev (2026-03-07), kloudvin.com (2026-06-08), blog.ecotone.tech (2026-04-27)

**Finding**: Netflix's event-sourced system on Cassandra required full projection rebuilds that took **days** during development. At 50M events, a global projection rebuild with single-worker sequential processing can take 4+ days. Blue/green projection deployment (stand up new projection, let it catch up, then swap) sounds safe but has a hidden flaw: the "catch up" phase requires the new projection to process all historical events at the same speed as the live event arrival rate — otherwise the catch-up never converges and the swap never happens.

**Defect**: Blue/green projection deployment has a **convergence deadlock** for high-volume streams. If the event production rate exceeds the projection processing rate during catch-up, the new projection's lag grows instead of shrinking, and the swap condition (lag < SLO) is never met. The old projection continues serving stale data indefinitely while the new projection runs forever in catch-up mode.

**Fix Required**: 
1. Projection rebuild must support **parallel partitioned processing** — track position per aggregate_id, not globally. This allows multiple workers to process different aggregates concurrently.
2. Implement a rebuild deadline: if catch-up hasn't converged within 2x the estimated time, force the swap with a degraded-read flag (return stale data with a "freshness unknown" indicator).
3. For projections > 100M events, implement incremental rebuild: only rebuild the projection for aggregates that have changed since the last snapshot, not the entire history.

---

## III. MESSAGE QUEUE DEFECTS (3 new findings)

### Defect 581-M1: RabbitMQ Classic Queue GC Regression — Unbounded Disk Growth

**Source**: github.com/rabbitmq/rabbitmq-server/issues/16141 (2026), rabbitmq.com (2024-08-28, 2026-04-23)

**Finding**: Commit 0278980ba0 (PR #13959) introduced a regression in RabbitMQ classic queue message store GC. Under sustained publish load with a slow-consumer queue sharing the same vhost as high-throughput queues, the GC compaction rate drops below the publish rate. Files accumulate faster than reclaimed, and disk usage grows without bound. The GC stall also causes consumer latency spikes: median 1.5s, P95 46s, P99 54s, max 568s (9.5 minutes).

**Defect**: This is a **cross-vhost resource contention** defect: one slow-consumer queue poisons the shared message store GC for all queues in the same vhost. The regression is in the core GC logic and is confirmed present in the `main` branch. Disk dropped 16GB in 100 minutes under the workload. The fix is a revert of the specific commit, but the architectural issue (shared message store across vhosts) remains.

**Fix Required**: 
1. Isolate queues with long consumer timeouts (ack hold > 60s) to dedicated vhosts. This gives them a separate message store instance.
2. Implement vhost-level disk usage alerts with auto-throttle: if vhost disk usage exceeds 80%, throttle publish rate for queues with unacked message count > 1000.
3. Monitor GC compaction rate as a first-class metric — if compaction rate < publish rate for > 5 minutes, trigger alert.

---

### Defect 581-M2: NATS JetStream at-Most-Once Default — Payment Event Loss Window

**Source**: medium.com/@rameshannanyt0078 (2026-07-06), johal.in (2026-04-27), dev.to (2026-03-22)

**Finding**: NATS Core delivers at-most-once by default — if a consumer is offline when a message arrives, the message is gone forever. NATS JetStream adds durability but requires explicit configuration (`replicas: 3`, `storage: file`). In benchmarks, NATS Core achieved 11M msg/s but with zero persistence. JetStream achieved 200K-400K msg/s with durability. Many teams adopt NATS for its simplicity and fail to enable JetStream for critical paths.

**Defect**: NATS's **two-tier durability model** creates a silent data loss window. Teams that start with NATS Core (at-most-once, zero persistence) and later add JetStream for critical paths often have a **mixed deployment** where some subjects are durable and some are not. The decision of which subjects get durability is often made ad-hoc, not through a governance policy. Critical events (payment confirmations, order state changes) can land on non-durable subjects if the team isn't careful about subject naming conventions.

**Fix Required**: 
1. All NATS subjects must be classified at deployment time: `critical` (JetStream with RF=3), `important` (JetStream with RF=1), `best-effort` (Core NATS). Classification must be in the capability registry.
2. NATS subject prefixes must encode durability: `critical.payments.`, `important.orders.`, `best-effort.metrics.`. Subjects without a durability prefix are rejected by the producer.
3. Implement a durability audit: periodically verify that all subjects with `critical.` prefix have JetStream enabled with RF >= 3.

---

### Defect 581-M3: Redis Streams OOM Kill — Cache-Broker Co-location Hazard

**Source**: medium.com/@rameshannanyt0078 (2026-07-06), messages.solutions (2026-06-11), devopsboys.com (2026-05-10)

**Finding**: Redis Streams stores all messages in RAM. When a consumer bug stops reading, messages pile up, Redis hits its memory limit, and the OOM killer steps in — killing the Redis instance that serves as both the message broker AND the application cache. One bug in a consumer brings down the entire cache layer. The benchmark (1KB messages) showed 950GB memory for 1B messages vs 280GB for Kafka with lz4 compression.

**Defect**: Redis Streams' in-memory model creates a **blast radius coupling** between cache availability and message processing. A single stuck consumer can OOM the Redis instance, taking down all dependent services (cache readers, session store, rate limiter) that share the same Redis. This is a **correlated failure mode** that doesn't exist with dedicated brokers (Kafka, RabbitMQ, NATS) because the broker and cache are separate processes.

**Fix Required**: 
1. Never use the same Redis instance for caching and message streaming. Deploy a dedicated Redis instance for Streams.
2. Implement `maxlen` on all Redis Streams with a hard cap: `XADD mystream MAXLEN ~ 100000`. Never allow unbounded stream growth.
3. Implement a consumer health watchdog: if any consumer group's pending entries (PEL) exceed 10,000 for > 60 seconds, auto-trim the stream and alert.

---

## IV. CROSS-DOMAIN DEFECTS (Synthesis)

### Defect 581-X1: No Unified Backpressure Taxonomy Across Stream + Queue + Event Layers

**Sources**: All 2026 sources

**Finding**: Each layer (stream processor, message queue, event store) has its own backpressure mechanism, but they don't compose:
- Flink: unaligned checkpoints + network buffer
- Kafka: consumer lag monitoring
- RabbitMQ: credit-based flow control
- NATS: subject-based rate limiting
- Event Sourcing: projection lag monitoring

**Defect**: When backpressure hits at one layer, the other layers are unaware. A Flink job backpressures → Kafka consumer lag spikes → RabbitMQ (used for downstream task dispatch) fills up → Saga compensations queue behind the blocked tasks → DLQ grows → manual intervention required. Each layer's monitoring sees "normal" (within its own thresholds) while the composite system is degrading.

**Fix Required**: Implement cross-layer backpressure propagation: when Flink checkpoint lag exceeds 2x target, signal Kafka to pause consumer offset commits, signal RabbitMQ to apply backpressure to downstream queues, and signal the Saga orchestrator to pause new Saga instances. All layers must share a unified backpressure signal via the heartbeat aggregator.

---

### Defect 581-X2: KRaft Migration + Event Sourcing + Projection = Triple Consistency Hazard

**Sources**: kafka.apache.org (2026), youngju.dev (2026-03-05), blackflow.co.uk (2026-06-11)

**Finding**: Systems running event sourcing on Kafka face a triple consistency hazard during Kafka 4.0 KRaft migration: (1) Kafka metadata is in flux (dual-write to ZK + KRaft), (2) event store append semantics may change during migration, (3) projections consuming from Kafka may see replayed or reordered events during the migration window. Each hazard alone is manageable; together they create a **consistency storm** where the event store, projections, and Kafka offsets are all potentially out of sync.

**Defect**: No documented procedure exists for running event sourcing projections during KRaft migration. The Kafka upgrade guide covers broker migration but not application-level projection behavior during the transition. Teams that rely on Kafka as their event bus for CQRS projections have no guidance on how to ensure projection position tracking survives the metadata transition.

**Fix Required**: 
1. During KRaft migration, projection position tracking must be paused and重建 from the event store's own position tracking (not Kafka offsets).
2. Post-migration, projection positions must be reconciled against the KRaft-authoritative offset log.
3. Implement a migration-specific projection replay: after migration finalization, replay all projections from the last known-good position (pre-migration) through the migration window events.

---

## V. SOURCES CITED

| # | Source | Date | URL |
|---|--------|------|-----|
| 1 | Flink vs Spark Streaming vs Kafka Streams (2026) | 2026-06-27 | iotdigitaltwinplm.com |
| 2 | Kafka vs Flink vs Spark Streaming (Medium) | 2026-03-14 | alper-korukcu.medium.com |
| 3 | Stream Processing 2026 Deep Dive | 2026-05-16 | youngju.dev |
| 4 | Stream Processing Tools Compared | 2026-03-12 | streamkap.com |
| 5 | Event-Driven Architecture with Kafka and Flink | 2026-03-29 | devstarsj.github.io |
| 6 | Event-Driven Architecture + CQRS + Event Sourcing | 2026-03-11 | youngju.dev |
| 7 | Event Sourcing and CQRS Implementation Guide | 2026-03-05 | youngju.dev |
| 8 | Event-Driven Architecture: Event Sourcing, CQRS, Saga | 2026-05-01 | system-design.space |
| 9 | Event-Driven Architecture in 2026 | 2026-05-03 | encore.dev |
| 10 | CQRS + Event Sourcing Production Architecture | 2026-05-22 | usamaqamar.space |
| 11 | CQRS + Event Sourcing Kafka Implementation | 2026-03-06 | youngju.dev |
| 12 | CQRS and Event Sourcing Practical Implementation 2026 | 2026-03-06 | calmops.com |
| 13 | Event Sourcing Production Anti-Patterns | 2026-03-07 | youngju.dev |
| 14 | Event Sourcing in Production: Aggregate Design | 2026-06-08 | kloudvin.com |
| 15 | Event Sourcing Event Versioning | 2026-03-07 | pnguyen.au |
| 16 | Event Sourcing Deep Dive | 2026-02-04 | sujeet.pro |
| 17 | Scaling Projections (Ecotone) | 2026-04-27 | blog.ecotone.tech |
| 18 | Fintech Event Sourcing Migration Case Study | 2026-06-25 | ecoaai.com |
| 19 | RabbitMQ vs NATS vs Redis Streams | 2026-06-11 | messages.solutions |
| 20 | Message Brokers Comparison 2026 | 2026-05-28 | dev.to |
| 21 | Message Queues and Event Streaming Architecture | 2026-01-29 | zylos.ai |
| 22 | Kafka vs RabbitMQ vs Redis Streams vs NATS 2026 | 2026-07-06 | medium.com |
| 23 | Kafka vs RabbitMQ vs NATS Performance 2026 | 2026-08-21 | dev.to |
| 24 | Kafka vs RabbitMQ vs Redis Streams 2026 | 2026-05-10 | devopsboys.com |
| 25 | Real-Time Event Streaming 2026 | 2026-03-22 | dev.to |
| 26 | Benchmark 2026 Message Queue Throughput | 2026-04-27 | johal.in |
| 27 | Real-Time Streaming 2026 - Kafka to AI | 2026-03-05 | insights.simon-cullen.com |
| 28 | Kafka 4.0 Upgrade Guide | 2026-03-20 | kafka.apache.org |
| 29 | KRaft vs ZooKeeper | 2026-05-22 | kafka.apache.org |
| 30 | Kafka 4.0 Dropped ZooKeeper | 2026-06-11 | blackflow.co.uk |
| 31 | Kafka 4.x Reality Check | 2026-02-17 | andrewbaker.ninja |
| 32 | Kafka 4.0 KRaft Migration Guide | 2026-05-03 | byteiota.com |
| 33 | Kafka 4.x Without ZooKeeper | 2026-02-13 | josedavidbaena.com |
| 34 | RabbitMQ Classic Queue GC Regression | 2026 | github.com/rabbitmq |
| 35 | RabbitMQ 4.0 Quorum Queue Features | 2024-08-28 | rabbitmq.com |
| 36 | RabbitMQ 4.3.0 Release Notes | 2026-04-23 | github.com/rabbitmq |
| 37 | RabbitMQ Quorum Queues Documentation | 2026 | rabbitmq.com |
| 38 | RabbitMQ 4.3 Highlights | 2026-04-23 | rabbitmq.com |

---

## VI. DEFECT SUMMARY TABLE

| ID | Domain | Severity | Title | Batch 580 Overlap |
|----|--------|----------|-------|-------------------|
| 581-S1 | Stream Processing | HIGH | Flink Checkpoint-Backpressure Feedback Loop | NO |
| 581-S2 | Stream Processing | CRITICAL | Kafka 4.0 EOS-v1 Removal — Dual-Write Corruption | NO |
| 581-S3 | Stream Processing | MEDIUM | Spark Latency Floor Phantom SLA | NO |
| 581-S4 | Stream Processing | HIGH | Streaming DB Shadow Pipeline | NO |
| 581-E1 | Event-Driven | CRITICAL | Projection Gap Silent Permanent Data Loss | NO |
| 581-E2 | Event-Driven | HIGH | Event Immutability vs GDPR Conflict | NO |
| 581-E3 | Event-Driven | HIGH | Kafka-as-Event-Store Anti-Pattern | NO |
| 581-E4 | Event-Driven | HIGH | Saga Compensation Stack Overflow | NO |
| 581-E5 | Event-Driven | MEDIUM | Projection Rebuild Convergence Deadlock | NO |
| 581-M1 | Message Queue | CRITICAL | RabbitMQ GC Regression Unbounded Disk | NO |
| 581-M2 | Message Queue | HIGH | NATS at-Most-Once Payment Loss Window | NO |
| 581-M3 | Message Queue | HIGH | Redis Streams OOM Cache Co-location | NO |
| 581-X1 | Cross-Domain | HIGH | No Unified Backpressure Taxonomy | NO |
| 581-X2 | Cross-Domain | CRITICAL | KRaft+EventSourcing+Projection Triple Hazard | NO |

**Total new defects**: 14 (4 stream + 5 event + 3 message + 2 cross-domain)
**Overlapping with Batch 580**: 0
**Critical severity**: 4 (581-S2, 581-E1, 581-M1, 581-X2)
