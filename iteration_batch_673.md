# Iteration 673 — Message Queue / Event Streaming / Pub-Sub 2026 Landscape

## Prior Context (Batch 672)
- experience-tree B-tree vs LSM workload mismatch
- KB statistics never refreshed → planner stale
- No hybrid row/column for dual access pattern
- No cost telemetry feedback loop
- Learned indexes pragmatic but unvalidated

---

## NEW Findings from 2026 Landscape

### Defect 1: NeoTrix EventBus Has No Backpressure Protocol

**Source**: youngju.dev (2026-03-22), dev.to (2026-06-11), index.dev comparison

2026 broker comparison reveals three distinct backpressure models:
- **Kafka**: Pull-based consumption — consumers control pace via `fetch.min.bytes`/`fetch.max.wait.ms`
- **RabbitMQ**: Push with `prefetch` limit — broker pauses delivery when consumer unACKed count hits ceiling
- **NATS JetStream**: `max_deliver` + `ack_wait` — dead-letter after N retries

NeoTrix EventBus (if custom) likely has none of these. Without backpressure, a slow consumer under GWT broadcast causes unbounded memory growth → OOM kill during consciousness cycle.

**Impact**: L5 Cognition broadcast storms can crash the system when one specialist module is slow.

**Defect Category**: Production readiness — missing flow control.

---

### Defect 2: No Event Schema Versioning or Compatibility Checks

**Source**: blobstreaming.org (2026-04-07), javacodegeeks.com (2026-01-05)

2026 consensus: "Schema design is the single most impactful decision in event streaming." Both Kafka and Pulsar now mandate Avro/Protobuf + Schema Registry with compatibility modes (BACKWARD/FORWARD/FULL).

NeoTrix EventBus events (ConsciousnessTick, GWT broadcast, SEAL phase transitions) have no schema registry. When modules evolve independently (NT-MIND distillation output changes shape), consumers crash silently or corrupt state.

**Impact**: Cross-domain event contracts break silently during evolution cycles.

**Defect Category**: Data contract enforcement — no schema evolution safety.

---

### Defect 3: No Message-Level TTL or Expiration Policy

**Source**: youngju.dev (2026-03-22) — NATS 2.11 feature

NATS 2.11 introduced per-message TTL (`--max-msg-ttl`). RabbitMQ has per-message expiration headers. Kafka has retention-based expiry.

NeoTrix EventBus events like `HeartbeatAggregator` health signals or `ConsciousnessTree` soil-phase broadcasts are time-sensitive. A stale health signal delivered 10 minutes late is worse than no signal — it triggers wrong self-healing decisions.

**Impact**: Stale events poison decision-making in self-healing and meta-cognition loops.

**Defect Category**: Event freshness enforcement — no TTL mechanism.

---

### Defect 4: No Dead-Letter Queue or Poison Message Handling

**Source**: codingdroplets.com (2026), index.dev comparison

RabbitMQ has DLX (Dead-Letter Exchange). Azure Service Bus has built-in DLQ. Kafka has `dead letter queue` patterns via error topics.

NeoTrix EventBus: if a consumer panics (e.g., NT-SHIELD audit detects anomaly but handler throws), the message is lost or retried infinitely. No poison-pill detection, no circuit breaker, no dead-letter capture for post-mortem.

**Impact**: One broken consumer can stall an entire domain's event processing.

**Defect Category**: Fault isolation — no poison message quarantine.

---

### Defect 5: No Multi-Tenancy or Namespace Isolation

**Source**: blobstreaming.org (2026-04-07), automq.com (2026-06-01)

Pulsar's native multi-tenancy: `tenant/namespace/topic` hierarchy with per-namespace quotas, ACLs, and isolation. Kafka 4.x share groups provide logical separation.

NeoTrix has 7 domains (NT-CORE through NT-FEEL). All share one EventBus. NT-SHIELD security events should not be observable by NT-IO (external interface). NT-MIND evolution events should not be visible to NT-WORLD (perception). No namespace isolation exists.

**Impact**: Information leakage across security boundaries; one domain's flood starves others.

**Defect Category**: Security architecture — missing domain isolation.

---

### Defect 6: No Geo-Replication or Multi-Region Event Log

**Source**: blobstreaming.org (2026-04-07), automq.com (2026-06-01)

Pulsar: built-in geo-replication as configuration. Kafka: MirrorMaker 2 + replicated topics. Both treat multi-region as first-class.

NeoTrix is desktop-first (Tauri), but NT-NEXUS (cross-session memory) and NT-MEMORY (KB) imply distributed scenarios. No event log replication means a local crash loses all in-flight consciousness events since last KB commit.

**Impact**: Cross-session learning is lossy; recovery from crash loses uncommitted evolution state.

**Defect Category**: Durability — no event log replication.

---

### Defect 7: Consumer Group Rebalancing Not Handled

**Source**: techbytes.app (2026-04-07), youngju.dev (2026-03-22)

Kafka's consumer group rebalancing is a known pain point (stop-the-world during rebalance). 2026 solutions: Kafka cooperative sticky assignor, Pulsar's subscription cursor management.

NeoTrix modules can start/stop dynamically (skill activation/deactivation). If EventBus uses a consumer-group model, module restart causes full rebalance → all other consumers pause. If no consumer-group model, no parallel consumption at all.

**Impact**: Module hot-reload or crash recovery blocks entire event pipeline.

**Defect Category**: Operational resilience — no graceful consumer rebalancing.

---

### Defect 8: No Tiered Storage for Long-Term Event Retention

**Source**: blobstreaming.org (2026-04-07), artificialintelligence.sg (2026-02-28)

"Tiered storage is now table stakes on both sides." Kafka and Pulsar both offload old segments to S3/GCS/Blob. This enables infinite retention without scaling the streaming layer.

NeoTrix events (ConsciousnessTree growth cycles, SEAL pipeline phases, GWT broadcasts) represent the system's operational history. Current EventBus likely discards after consumption. This means:
- Cannot audit why a self-healing decision was made
- Cannot replay a consciousness cycle to debug phi/coherence anomalies
- Cannot train learned indexes on historical event patterns

**Impact**: No operational audit trail; no historical data for ML/learned index training.

**Defect Category**: Observability — no event history retention.

---

### Defect 9: Missing OpenTelemetry Integration for Event Tracing

**Source**: youngju.dev (2026-03-22) — NATS 2.11 distributed tracing

NATS 2.11 added OpenTelemetry integration. Kafka has `kafka-tracing` library. Pulsar has built-in tracing.

NeoTrix events flow across 6 layers (L1-L6). When a consciousness tick triggers GWT broadcast → NT-MIND distillation → NT-MEMORY write, there's no trace ID linking these. Debugging requires manual log correlation across 7+ files.

**Impact**: Cross-domain event chains are untraceable; debugging consciousness cycles requires grepping logs.

**Defect Category**: Observability — no distributed tracing for event chains.

---

### Defect 10: EventBus Has No Ordered Delivery Guarantees Across Domains

**Source**: dev.to (2026-08-21), index.dev comparison

Kafka: ordering per partition. Pulsar: ordering per topic partition. NATS: ordering per subject. All provide explicit ordering semantics.

NeoTrix EventBus: if events from NT-CORE (E8 reasoning) must be processed before NT-MIND (distillation) which must precede NT-MEMORY (KB write), there's no ordering guarantee. Out-of-order delivery → distillation writes before reasoning completes → corrupted KB state.

**Impact**: Cross-domain event ordering violations cause data corruption in KB.

**Defect Category**: Consistency — no causal ordering across domain events.

---

## Sources Cited

| # | Source | Date | Key Insight |
|---|--------|------|-------------|
| 1 | dev.to/kafka-vs-rabbitmq-vs-nats-2026 | 2026-08-21 | Three distinct broker models: streaming/broker/messaging |
| 2 | youngju.dev/message-queue-comparison-2025 | 2026-03-22 | NATS 2.11 message TTL + OTel tracing; Pulsar 300x P99 improvement |
| 3 | dev.to/message-brokers-comparison-2026 | 2026-06-11 | Redis Streams as 4th option; practical decision framework |
| 4 | blobstreaming.org/event-streaming-platforms-2026 | 2026-04-10 | Tiered storage table stakes; schema design = most impactful decision |
| 5 | techbytes.app/kafka-vs-pulsar-vs-nats-2026 | 2026-04-07 | Kafka share groups; architectural center of gravity differs |
| 6 | artificialintelligence.sg/kafka-vs-pulsar-2026 | 2026-02-8 | Operations > benchmarks; org shape decides |
| 7 | codingdroplets.com/rabbitmq-vs-azure-service-bus-vs-kafka | 2026 | Messaging vs streaming distinction; MassTransit abstraction |
| 8 | automq.com/pulsar-vs-kafka | 2026-06-01 | Protocol/tooling/ops shift cost analysis |
| 9 | index.dev/kafka-vs-rabbitmq-vs-nats | 2026 | No universal winner; each gets defined responsibility |
| 10 | sota.io/eu-message-broker-comparison-2026 | 2026-05-18 | CLOUD Act jurisdiction vs data residency; EU-native alternatives |

---

## Defect Summary

| # | Defect | Severity | Domain Affected | Category |
|---|--------|----------|-----------------|----------|
| 1 | No backpressure protocol | HIGH | All (broadcast storms) | Production readiness |
| 2 | No event schema versioning | HIGH | Cross-domain contracts | Data contract |
| 3 | No message-level TTL | MEDIUM | Time-sensitive signals | Freshness |
| 4 | No dead-letter queue | HIGH | Fault isolation | Fault isolation |
| 5 | No multi-tenancy/namespace | HIGH | Security boundaries | Security |
| 6 | No geo-replication | MEDIUM | Cross-session memory | Durability |
| 7 | No consumer rebalancing | HIGH | Module hot-reload | Operational resilience |
| 8 | No tiered storage for events | MEDIUM | Audit trail / ML training | Observability |
| 9 | No OpenTelemetry event tracing | MEDIUM | Cross-domain debugging | Observability |
| 10 | No ordered delivery across domains | HIGH | KB consistency | Consistency |

---

## NEW vs Prior Batches

| Prior (672) | NEW (673) |
|-------------|-----------|
| B-tree vs LSM workload mismatch | Backpressure protocol missing |
| KB statistics never refreshed | Event schema versioning absent |
| No hybrid row/column | Message TTL not enforced |
| No cost telemetry | Dead-letter queue absent |
| Learned indexes pragmatic | Multi-tenancy/namespace missing |
| | Geo-replication absent |
| | Consumer rebalancing unhandled |
| | Tiered storage missing |
| | OpenTelemetry tracing absent |
| | Cross-domain ordering absent |

**Total new defects**: 10
**Cumulative across batches**: 672 + 10 = 682 findings tracked
