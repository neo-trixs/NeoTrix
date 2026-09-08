# Iteration Batch 443 — Message Queues, Pub/Sub, and RPC (2026 Advances)

**Date**: 2026-09-06
**Focus**: External research → design defects in NeoTrix messaging/communication layer

---

## Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | NATS Server v2.15.0-preview.1 (GitHub) | 2026-08-24 | JetStream desired-state metalayer, evacuate endpoints, stream backup v2, cancel move |
| S2 | NATS Server v2.14.6-RC.1 (GitHub) | 2026-08-25 | Isolated stream reads, filestore I/O semaphore, flow control fixes |
| S3 | "NATS vs Kafka vs Redis Streams for Java Microservices" (JavaCodeGeeks) | 2026-03 | Operational comparison: NATS sub-ms p99.7, Kafka 4.0 KRaft-only, Redis Streams pragmatic middle |
| S4 | "Kafka vs NATS vs Redis Streams: Choosing the Event Backbone for AI Agent Systems" (dreaming.press) | 2026-06-30 | Agent-system-specific messaging: durable ordered replayable log as first-class, sliding-window flow control |
| S5 | "Real-Time Event Streaming: Kafka vs Redis Streams vs NATS in 2026" (DEV.to) | 2026-03-22 | Benchmarks: NATS 820K msg/s, Kafka 1.2M msg/s, Redis 480K msg/s; p99 latencies |
| S6 | "Pub/Sub Messaging Patterns: Redis vs NATS" (DEV.to) | 2026-03-21 | Dead letter handling, competing consumers, wildcard routing, queue groups |
| S7 | BroSettlement JetStream wallet backbone (NATS blog) | 2026-08-17 | Outbox pattern, idempotent consumers, versioned subjects, correlation IDs |
| S8 | Dapr pub/sub routing docs | 2026-08-03 | CEL-based content routing, declarative/programmatic subscriptions |
| S9 | Solace topic architecture best practices | 2026 | Domain/Noun/Verb/Version/Properties hierarchy, versioning in topics |
| S10 | event-io (GitHub) | 2026 | Rust in-process event bus: wildcard subscriptions, priority delivery, backpressure, dead-letter, replay |
| S11 | KDCube Data Bus architecture | 2026-07-16 | Durable inbound bus (Redis Stream) + transient outbound relay, per-object serialization, scene bridge |
| S12 | ConnectRPC v1.19.1 (Go) | 2026-04-20 | Stable: gRPC + gRPC-Web + Connect protocol, streaming, health checks |
| S13 | connect-rust joins Connect project (Buf blog) | 2026-08-26 | Anthropic's ConnectRPC for Rust: Tower-based, zero-copy Buffa protobuf, production at Anthropic |
| S14 | connectrpc/connect-rust (GitHub) | 2026 | 3,600 server + 6,872 client conformance tests, 1/3 more throughput than tonic at high concurrency |
| S15 | "gRPC-Connect Protocol: Go Microservice Unified Frontend-Backend Communication 2026" | 2026-02-15 | One Proto for both sides, Buf ecosystem, production gateway patterns |
| S16 | Cap'n Proto RPC | Current | Promise pipelining, capability-based RPC, zero-copy, time-traveling RPC |
| S17 | connect-es (GitHub) | Current | TypeScript Connect: Protobuf-ES, gRPC-Web, browser-native |

---

## Defects Found

### DEFECT-443-1: EventBus Has No Sliding-Window Flow Control (CRITICAL)

**Component**: `nt_core_event_bus.rs:25-34` (EventBus struct)

**Current State**: `EventBus` uses `tokio::sync::broadcast(1024)` — a fixed-capacity ring buffer. When a consumer lags, it receives `Lagged(n)` and the event is silently dropped. There is no flow control mechanism to slow producers when consumers fall behind.

**2026 Research**: Source S4 (dreaming.press, 2026-06-30) demonstrates that **per-subscription sliding-window flow control** is the critical differentiator for agent systems: "a single agent step can trigger a slow, expensive tool call and you need the planner not to bury the executors." NATS JetStream has this built into the protocol (S1, S2). Source S10 (event-io) implements bounded per-subscriber queues with explicit dropping semantics for slow consumers.

**Impact**: NeoTrix's consciousness loop (E8 reasoning → GWT broadcast → downstream modules) can flood slow modules (NT-SHIELD security scans, NT-WORLD crawls). When a consumer lags, the `Lagged(n)` error loses events silently — no DLQ, no retry, no producer notification. This violates the "The Spice Must Flow" axiom: data must flow cleanly from input to output with no disconnects.

**Suggestion**: Replace the broadcast channel with a per-subscriber bounded queue model:
- Implement `FlowControlledBus` with per-layer `tokio::sync::mpsc::channel` (bounded) + a fan-out publisher task
- Add `backpressure_signal: broadcast::Sender<()>` that producers can optionally await
- Implement `PendingCount` metric per layer for monitoring (per S5's "monitor consumer lag is #1 cause of streaming failures")
- Add DLQ channel for events that exceed max-redelivery

---

### DEFECT-443-2: No Topic-Based Routing or Wildcard Subscriptions (HIGH)

**Component**: `nt_core_event_bus.rs:122` (`subscribe()` returns a single `broadcast::Receiver<CoreEvent>`)

**Current State**: All subscribers receive ALL `CoreEvent` variants. There is no topic filtering, no wildcard matching, no content-based routing. Every module must filter events in application code after receiving them.

**2026 Research**: Source S8 (Dapr, 2026-08-03) implements CEL-based content routing on CloudEvent fields. Source S9 (Solace) defines a `Domain/Noun/Verb/Version/Properties` topic hierarchy with wildcard subscriptions. Source S10 (event-io) implements `user.*`, `user.**`, `order.created` glob subscriptions natively. Source S6 (DEV.to) shows NATS wildcard subjects (`orders.>`, `orders.*`) as fundamental for microservice routing.

**Impact**: NeoTrix has 11+ domains, each subscribing to the full event stream and filtering in Rust match arms. This wastes CPU on serialization/deserialization of irrelevant events and prevents the EventBus from being used as a true message bus (per R-P36: "EventBus 触发行为改变"). Content-based routing would enable GWT to dynamically subscribe to specific event patterns based on current attention state.

**Suggestion**: Add topic taxonomy to `CoreEvent` variants:
- Define topic as `"{domain}.{noun}.{verb}.v{version}"` (per Solace S9 best practice)
- Implement `subscribe_pattern("nt_core.reasoning.*")` with glob matching
- Map existing `CoreEvent` enum variants to topic strings
- Enable `subscribe_all_with_filter(|env| env.topic.matches("nt_*.*.broadcast"))` for backward compatibility
- Consider integrating with NATS JetStream for distributed deployment (per S4 recommendation)

---

### DEFECT-443-3: No Dead Letter Queue or Failed-Event Recovery (HIGH)

**Component**: `nt_core_event_bus.rs:115-117` (error handling on broadcast send)

**Current State**: When `self.sender.send(event)` fails (no receivers, channel closed), the event is logged as a warning and dropped. When `emit_from` writes to the log file but broadcast fails, the event persists on disk but is never delivered to live subscribers. No DLQ, no retry, no circuit breaker.

**2026 Research**: Source S7 (BroSettlement, 2026-08-17) explicitly handles this: "the outbox closes the producer-side consistency gap, but it deliberately produces at-least-once... duplicates are therefore a normal recovery condition." Source S6 (DEV.to) shows NATS JetStream's built-in max delivery attempts with advisory messages on `$JS.EVENT.ADVISORY.CONSUMER.MAX_DELIVERIES`. Source S10 (event-io) implements a `dead-letter` feature with a configurable sink.

**Impact**: If NT-ACT emits a task-completion event but the subscriber for NT-MEMORY (knowledge persistence) is temporarily down, the event is lost. Per R-P25: "CritiqueResult 必须经过 EventBus 分发...不能只有局部日志" — lost events mean lost critique results, lost knowledge updates, and silent behavioral regressions.

**Suggestion**: Implement DLQ pattern:
- Add `dlq: Option<mpsc::Sender<CoreEventEnvelope>>` to EventBus
- On broadcast failure after N retries (configurable), route to DLQ
- Add `dlq_replay()` method for manual or automated re-delivery
- Add `FailedEventMetric` to HeartbeatAggregator for system health visibility
- Consider outbox pattern (S7): write to persistent store first, then publish; on replay, store is source of truth

---

### DEFECT-443-4: EventBus Persistence Is Not Crash-Safe (MEDIUM)

**Component**: `nt_core_event_bus.rs:97-113` (emit_from persistence path)

**Current State**: `emit_from()` writes to a JSONL file with `writeln!()` — no fsync, no WAL, no crash recovery guarantee. The file write and broadcast send are NOT atomic: if crash happens between them, events can be lost or duplicated on replay.

**2026 Research**: Source S4 (2026-06-30) states: "a durable, ordered, replayable log is a first-class requirement, not a nice-to-have" for agent systems. Source S11 (KDCube, 2026-07-16) uses Redis Streams (append-only log with XADD/XREADGROUP) for durable inbound bus. Source S7 (BroSettlement) uses PostgreSQL transactional outbox for crash-safe event publication.

**Impact**: On crash recovery, `replay_enveloped()` loads the entire JSONL file — but if the file is partially written (crash mid-write), the replay may fail or produce incorrect state. No fsync means OS can buffer writes for seconds, increasing data loss window.

**Suggestion**: Adopt WAL-like persistence:
- Write-ahead to a dedicated WAL file with fsync before broadcast send
- On recovery, replay WAL → rebuild in-memory state
- Add periodic compaction (S1: NATS stream source indexing for instant lookup)
- Add idempotency keys in `EventEnvelope` for dedup on replay
- Consider SQLite WAL mode (already used by KB) as the persistence backend

---

### DEFECT-443-5: No Versioned Subject Routing for Schema Evolution (MEDIUM)

**Component**: `nt_core_event_bus.rs:12-21` (EventEnvelope struct)

**Current State**: `EventEnvelope` has no version field. `CoreEvent` enum variants are not versioned. Adding/removing fields from a variant is a breaking change for all subscribers. No blue/green or canary deployment possible for event schema changes.

**2026 Research**: Source S9 (Solace, 2026) defines versioning as a mandatory topic level: `Domain/Noun/Verb/Version/Properties`. Source S7 (BroSettlement) uses `version` field and versioned subjects like `deposit.account.created.v1` for additive evolution. Source S5 (DEV.to, 2026-03-22) notes NATS JetStream streams capture subjects into streams — versioned subjects enable canary consumers.

**Impact**: When `CoreEvent` gains a new variant or field, all existing subscribers break at compile time. No gradual migration path. Cannot run v1 and v2 consumers simultaneously during transitions. This is particularly problematic for NT-SHIELD (security events) where downtime during schema migration is unacceptable.

**Suggestion**: Add versioning infrastructure:
- Add `schema_version: u16` to `EventEnvelope`
- Add topic string to `EventEnvelope` (e.g., `"nt_core.reasoning.e8_transition.v2"`)
- Implement `subscribe_pattern("nt_core.reasoning.*.v*")` for multi-version consumption
- Support additive evolution: new fields are optional, old consumers ignore unknown fields
- Add `SchemaRegistry` trait that maps event types to their current version and migration functions

---

### DEFECT-443-6: No RPC Layer for Synchronous Inter-Domain Calls (HIGH)

**Component**: Cross-cutting — no RPC implementation exists

**Current State**: NeoTrix domains communicate only via EventBus (async pub/sub). There is no mechanism for synchronous request-reply between domains. When NT-CORE needs to query NT-MEMORY for a specific knowledge lookup and await the result, it must use ad-hoc patterns (channels, shared state, or direct function calls).

**2026 Research**: Source S13 (Buf blog, 2026-08-26) shows ConnectRPC for Rust: Tower-based, passes full conformance suite (3,600 server + 6,872 client tests), production at Anthropic. Source S14 benchmarks show 1/3 more throughput than tonic. Source S15 (2026-02-15) demonstrates one Proto definition generating both backend (gRPC) and frontend (TypeScript) code. Source S16 (Cap'n Proto) offers promise pipelining for zero-latency RPC.

**Impact**: The consciousness loop requires synchronous interactions: E8 reasoning needs KB lookups (NT-CORE → NT-MEMORY), emotion evaluation needs world state (NT-FEEL → NT-WORLD), action planning needs tool availability (NT-ACT → NT-ACT internal). Without a proper RPC layer, these are either:
1. Blocking direct calls (violates async consciousness loop)
2. EventBus request-reply patterns (heavyweight, no type safety)
3. Shared mutable state (violates R-P1 zero-unsafe, concurrency safety)

**Suggestion**: Adopt ConnectRPC (S13/S14) as the RPC layer:
- Define service interfaces in `.proto` files
- Generate Rust server traits + client stubs via `protoc-gen-connect-rust`
- Mount ConnectRouter as a `tower::Service` alongside existing Hyper server
- Use Tower middleware for auth, rate limiting, tracing (built-in)
- Support both Connect protocol (HTTP/1.1, browser-friendly) and gRPC (backend-to-backend)
- Define services: `KnowledgeService` (NT-MEMORY), `ReasoningService` (NT-CORE), `ActionService` (NT-ACT)

---

### DEFECT-443-7: No Backpressure-Aware Event Priority (MEDIUM)

**Component**: `nt_core_event_bus.rs:46-49` (broadcast channel capacity)

**Current State**: All `CoreEvent` variants are treated equally in the broadcast channel. Security-critical events (NT-SHIELD) have the same priority as routine telemetry. When the bus is congested, all events are equally likely to be dropped via `Lagged(n)`.

**2026 Research**: Source S4 (2026-06-30) states: "that backpressure is not a luxury when a single agent step can trigger a slow, expensive tool call and you need the planner not to bury the executors." Source S10 (event-io) implements priority-based delivery with separate queues per priority level. Source S11 (KDCube) uses per-object serialization to prevent one slow object from blocking others.

**Impact**: During high-load periods (e.g., NT-WORLD crawling many pages simultaneously), security audit events from NT-SHIELD may be dropped, creating a blind spot. The HeartbeatAggregator (per CONTEXT.md) aggregates health signals but cannot distinguish dropped-security-events from dropped-telemetry-events.

**Suggestion**: Implement priority-aware routing:
- Add `priority: EventPriority` (Critical/High/Medium/Low) to `EventEnvelope`
- Use separate bounded channels per priority level
- Critical events (security, safety) use unbounded or large-capacity channel
- Low events (telemetry, debug) use small-capacity channel with aggressive dropping
- Monitor drop rate per priority level for HeartbeatAggregator

---

## Summary Table

| ID | Defect | Severity | Component | 2026 Research Gap |
|----|--------|----------|-----------|-------------------|
| 443-1 | No sliding-window flow control | CRITICAL | EventBus | NATS JetStream protocol-level flow control (S1/S2/S4) |
| 443-2 | No topic-based routing / wildcards | HIGH | EventBus | Dapr CEL routing, NATS wildcards, Solace topic hierarchy (S6/S8/S9) |
| 443-3 | No DLQ or failed-event recovery | HIGH | EventBus | NATS max-delivery advisories, outbox pattern (S6/S7) |
| 443-4 | Persistence not crash-safe | MEDIUM | EventBus | Redis Streams append-only, PostgreSQL outbox (S7/S11) |
| 443-5 | No versioned subject routing | MEDIUM | EventBus | Solace versioning, BroSettlement v1/v2 subjects (S7/S9) |
| 443-6 | No RPC layer for sync calls | HIGH | Cross-cutting | ConnectRPC-Rust: Tower-based, Anthropic production (S13/S14/S15) |
| 443-7 | No backpressure-aware priority | MEDIUM | EventBus | event-io priority queues, KDCube per-object serialization (S10/S11) |

---

## Architectural Suggestions

### Short-term (next iteration):
1. Add `EventPriority` enum and priority-aware channel separation to EventBus
2. Add `dlq` field and retry logic to EventBus
3. Add `schema_version` and `topic` to `EventEnvelope`

### Medium-term (1-2 cycles):
4. Adopt ConnectRPC for inter-domain synchronous RPC (S13/S14)
5. Implement topic-based subscription with glob matching on EventBus
6. Add WAL-based crash-safe persistence

### Long-term (architecture evolution):
7. Make EventBus transport-pluggable (in-process / SQLite / NATS / Redis Streams) — per S4 recommendation for agent systems
8. Implement full NATS JetStream integration for multi-node deployment with subject-based routing, consumer groups, and flow control
