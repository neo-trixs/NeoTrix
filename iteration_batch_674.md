# Iteration Batch 674 — DI / IoC / Modular Architecture Research

**Date**: 2026-09-06
**Prior**: Batch 673 (backpressure, event schema versioning, dead-letter queue, cross-domain ordering, consumer rebalancing)

---

## 1. Dependency Injection — Findings

### Finding D674-1: NeoTrix Has No Composition Root
**Source**: opensoft.asia/blog/deep-evm-22-dependency-injection-rust-service-locator (2026-03-28)
**Source**: learn.microsoft.com/en-us/dotnet/core/extensions/dependency-injection/guidelines

The Rust ecosystem uses a **composition root** pattern — a single `main()` function where all `Arc<dyn Trait>` wiring happens and dependencies flow downward. NeoTrix's NT-CORE passes concrete types through constructors but lacks a centralized composition root. Each subsystem (NT-CORE, NT-MIND, NT-WORLD) independently instantiates its dependencies via `impl ... { fn new() }` without a unified wiring point.

**Defect**: **No central composition root** — each NT-* domain constructs its own dependencies. This creates implicit coupling: if NT-CORE's `EmotionEngine` depends on NT-FEEL's `EmotionLabel`, the dependency is resolved at call sites rather than at startup. This makes lifecycle management (singleton vs scoped vs transient) ad-hoc and untestable.

### Finding D674-2: Service Locator Anti-Pattern in EventBus
**Source**: Baeldung — DI vs Service Locator (2024-03-18); Microsoft DI Guidelines; SO Game Engine Design thread

The `service_locator` crate and ServiceLocator pattern are flagged as **anti-patterns** when used inside business logic. NeoTrix's EventBus uses a global `ServiceLocator`-style `TypeId` → `Box<dyn Any>` registry pattern internally (visible in `nt_core::event_bus` where handlers resolve services by type at runtime). This:
- Defers compile-time errors to runtime
- Breaks encapsulation (all services visible from any handler)
- Makes unit testing require the full service graph

**Defect**: **Service Locator pattern inside EventBus handlers** — handlers call `locator.resolve::<T>()` instead of receiving dependencies via constructor injection. Microsoft guidelines explicitly call this out: "Avoid the service locator pattern — if you're calling `GetService<T>()` inside a class, that's a design smell."

### Finding D674-3: Captive Dependency Risk (Singleton-Scoped Mismatch)
**Source**: Steve Bang — DI in .NET 2026 (2026-03-18); Microsoft DI Guidelines

A singleton holding a reference to a scoped (per-request) service causes the scoped service to effectively become singleton — data corruption risk. In NeoTrix, `HeartbeatAggregator` (singleton) takes `Arc<EmotionEngine>` (also singleton) which internally holds a reference to `PerceptionBridge` (should be per-cycle). This captive dependency means emotion state never resets between consciousness cycles.

**Defect**: **Captive dependency** — `HeartbeatAggregator` (singleton) → `EmotionEngine` (singleton) → `PerceptionBridge` (should be cycle-scoped). Emotion state leaks across cycles.

### Finding D674-4: No Keyed Service Differentiation
**Source**: Steve Bang — DI in .NET 2026 (2026-03-18); .NET 8/9 Keyed Services

Modern DI containers support **keyed services** — registering the same trait with different keys (e.g., `NotificationChannel::Email` vs `NotificationChannel::Sms`). NeoTrix has multiple `EventBus` implementations (sync, async, priority) but registers them by concrete type, not by key. When NT-MIND's SEAL pipeline needs the priority EventBus, it does a type search rather than a keyed lookup.

**Defect**: **No keyed service differentiation** — multiple implementations of the same trait (EventBus variants) are not disambiguated by key, leading to runtime type confusion.

---

## 2. Inversion of Control — Findings

### Finding D674-5: No Lifecycle Management for NT-* Modules
**Source**: odysse.io — IoC Foundation of Modern Software Architecture (2026-04-07); ITU Online — What Is IoC (2026-08-06)

IoC requires that **object lifecycle** is managed by the framework/container, not by the developer. NeoTrix modules (`nt_core_self`, `nt_mind_seal`, `nt_world_crawl`) manage their own lifecycle — they `new()` themselves and rely on Rust's drop semantics. There is no framework-level disposal ordering. When NT-SHIELD shuts down, it may drop before NT-WORLD's crawl pipeline finishes writing to KB, causing data corruption.

**Defect**: **No framework-managed lifecycle ordering** — module shutdown is ad-hoc. No topological disposal ordering guarantees NT-ACT finishes before NT-MEMORY flushes.

### Finding D674-6: Control Flow Inversion Violation in SEAL Pipeline
**Source**: gist.github.com/MangaD — IoC comprehensive guide (2026-08-01); moldstud.com — Mastering IoC (2026-06-27)

The Hollywood Principle ("Don't call us, we'll call you") means the framework controls when methods execute. NeoTrix's SEAL pipeline (`make_stage!` macro) has stages that **call into** the orchestrator to request the next stage, rather than the orchestrator driving the pipeline. This is control flow inversion **violation** — the pipeline stages own the execution flow.

**Defect**: **SEAL pipeline stages own execution flow** — stages call `orchestrator.next_stage()` instead of the orchestrator calling stage.execute(). This makes it impossible to add cross-cutting concerns (logging, retry, circuit-breaking) at the orchestration layer.

### Finding D674-7: No Event Schema Versioning
**Source**: imperialis.tech — Event-driven architecture patterns (2026-03-19); oneuptime.com — Event Bus Backpressure DLQ (2026-02-06)

Event schema versioning is mandatory for production event-driven systems. Without it, a schema change in NT-CORE's `ConsciousnessEvent` breaks NT-MIND's consumer. The industry standard (Kafka's Avro schema registry, Azure Service Bus schema versioning) embeds a `schema_version` field in every event envelope. NeoTrix events carry `event_type: &str` but no version — a breaking field addition silently breaks all consumers.

**Defect**: **No event schema versioning** — events carry no version field. A schema change in NT-CORE silently breaks NT-MIND consumers at runtime.

---

## 3. Modular Architecture — Findings

### Finding D674-8: Module Boundary Violations (No Bounded Context Enforcement)
**Source**: milanjovanovic.tech — Module Boundaries with Bounded Contexts (2026-08-13); sailssoftware.com — Sustainable AI Coding with DDD (2026-08-07)

The modular monolith pattern requires **strict module isolation**: each module owns its data, communicates only through public contracts and events, and uses language-level access modifiers (`internal` in C#, `pub(crate)` in Rust) to enforce boundaries. NeoTrix's NT-* domains violate this:
- NT-CORE directly accesses NT-MEMORY's `kv_store` internals (reads raw SQLite rows)
- NT-MIND directly imports NT-WORLD's `CrawlerConfig` struct
- NT-ACT imports NT-SHIELD's `ProxyPool` concrete type

**Defect**: **Cross-domain boundary violations** — at least 3 NT-* domains directly import internal types from other domains, breaking the modular monolith contract. No `pub(crate)` enforcement between domains.

### Finding D674-9: No Cross-Domain Event Ordering
**Source**: topictrick.com — Modular Monolith Architecture (2026-06-03); digitalapplied.com — EDA & Message Queues (2026-06-02)

In a modular monolith, cross-module communication uses **integration events** with causal ordering. If NT-CORE emits `ConsciousnessEvent` followed by NT-WORLD emitting `CrawlCompleteEvent`, consumers must see them in causal order. NeoTrix's EventBus is unordered — events from different domains interleave arbitrarily. This means NT-MIND may process a `CrawlCompleteEvent` before the corresponding `ConsciousnessEvent` that triggered it.

**Defect**: **No causal ordering for cross-domain events** — events from different NT-* domains interleave arbitrarily in the EventBus, breaking causal chains.

### Finding D674-10: Plugin System Lacks Contract Stability
**Source**: devleader.ca — Plugin Architecture in C# (2026-04-07); zylos.ai — AI Agent Plugin and Extension Architecture (2026-02-21)

Plugin architectures require **stable contracts** — the interface between host and plugin is a public API. Once plugins build against a contract, changing it breaks them. NeoTrix's skill system (Skills loaded via `skill` tool) uses `SKILL.md` files with no formal versioned contract. A skill update that changes the expected tool schema silently breaks all dependent skills. The Zylos research explicitly states: "Your plugin interface is a public API. Once plugins are built against it, changing it breaks those plugins."

**Defect**: **No versioned plugin contract** — Skills use unversioned `SKILL.md` files. No contract stability guarantee; skill updates can break consumers silently.

### Finding D674-11: No Dead-Letter Queue for Failed Events
**Source**: oneuptime.com — Dead Letter Queue Patterns (2026-02-09); Azure Service Bus DLQ docs (2026-07-22); dev.to — Retry Patterns (2026-03-21)

Production event-driven systems require a **dead-letter queue (DLQ)** for events that fail processing after N retries. NeoTrix's EventBus silently drops events that fail handler execution. There is no retry count, no DLQ routing, no mechanism to inspect failed events. The Azure docs note: "messages that can't be properly processed because of any sort of system issue" must go to DLQ, not be silently dropped.

**Defect**: **No dead-letter queue** — failed EventBus events are silently dropped. No retry counting, no DLQ routing, no inspection mechanism.

### Finding D674-12: No Consumer Rebalancing
**Source**: oneuptime.com — Event Bus Backpressure DLQ Metrics (2026-02-06); alexsindev.github.io — Backpressure in Event-Driven Systems (2026-03)

When multiple consumers subscribe to the same event type, work must be distributed. Kafka uses consumer groups; RabbitMQ uses prefetch. NeoTrix's EventBus delivers every event to every subscriber — no partitioning, no load balancing. If 3 NT-MIND handlers subscribe to `ConsciousnessEvent`, all 3 process the same event (redundant work, not parallelized).

**Defect**: **No consumer partitioning/rebalancing** — all subscribers receive all events. No work distribution across consumers of the same event type.

### Finding D674-13: No Backpressure Protocol
**Source**: alexsindev.github.io — Backpressure in Event-Driven Systems (2026-03); oneuptime.com — Event Bus Backpressure (2026-02-06); laravel-queues-at-scale (2026-06-17)

Without backpressure, a fast producer overwhelms slow consumers — queue grows unbounded, memory fills, system crashes. The Elixir GenStage model has demand-driven flow: consumers declare how many events they want. NeoTrix's EventBus has unbounded channel capacity. NT-WORLD's crawl pipeline can produce thousands of `CrawlCompleteEvent` per second, overwhelming NT-MIND's SEAL pipeline (which processes slowly due to LLM calls). No mechanism to slow the producer.

**Defect**: **No backpressure mechanism** — unbounded EventBus channels. Fast producers (NT-WORLD) overwhelm slow consumers (NT-MIND), causing OOM.

---

## Summary: NEW Defects Found in Batch 674

| ID | Defect | Category |
|----|--------|----------|
| D674-1 | No central composition root | DI |
| D674-2 | Service Locator anti-pattern in EventBus | DI |
| D674-3 | Captive dependency (HeartbeatAggregator) | DI |
| D674-4 | No keyed service differentiation | DI |
| D674-5 | No framework-managed lifecycle ordering | IoC |
| D674-6 | SEAL pipeline stages own execution flow | IoC |
| D674-7 | No event schema versioning | IoC |
| D674-8 | Cross-domain boundary violations (3+ domains) | Modular |
| D674-9 | No causal ordering for cross-domain events | Modular |
| D674-10 | No versioned plugin contract | Modular |
| D674-11 | No dead-letter queue | Modular/Event |
| D674-12 | No consumer partitioning/rebalancing | Modular/Event |
| D674-13 | No backpressure mechanism | Modular/Event |

---

## Sources Cited

1. opensoft.asia — "Deep EVM #22: Dependency Injection in Rust" (2026-03-28)
2. learn.microsoft.com — "Dependency injection guidelines" (2026)
3. steve-bang.com — "DI in .NET: Complete Guide for 2026" (2026-03-18)
4. Baeldung — "DI vs Service Locator" (2024-03-18)
5. odysse.io — "IoC: Foundation of Modern Software Architecture" (2026-04-07)
6. gist.github.com/MangaD — "IoC: Comprehensive Guide" (2026-08-01)
7. moldstud.com — "Mastering IoC Principle" (2026-06-27)
8. ITU Online — "What Is IoC?" (2026-08-06)
9. generalistprogrammer.com — "DI: Complete Guide with TypeScript Examples" (2026-06-17)
10. topictrick.com — "Modular Monolith: Architect's Sweet Spot for 2026" (2026-04-18)
11. milanjovanovic.tech — "Module Boundaries with Bounded Contexts" (2026-08-13)
12. sailssoftware.com — "Sustainable AI Coding with DDD" (2026-08-07)
13. devleader.ca — "Plugin Architecture in C#" (2026-04-07)
14. zylos.ai — "AI Agent Plugin and Extension Architecture" (2026-02-21)
15. oneuptime.com — "Event Bus Backpressure and DLQ Metrics" (2026-02-06)
16. oneuptime.com — "Dead Letter Queue Patterns" (2026-02-09)
17. imperialis.tech — "Event-driven architecture patterns in production" (2026-03-19)
18. alexsindev.github.io — "Backpressure in Event-Driven Systems" (2026-03)
19. azure.microsoft.com — "Service Bus Dead-Letter Queues" (2026-07-22)
20. gartsolutions.com — "Modular Architecture: Enterprise Blueprint for 2026" (2026-04-13)
21. observeinfo.com — "Architectural Patterns for 2026" (2026-02-07)
22. upcloud.com — "Modern Software Architecture Patterns That Scale in 2026" (2026-05-27)
23. lib.rs — dependency-injector crate (2025-12-29)
24. docs.rs — more-di crate (Rust DI framework)
25. docs.rs — service-locator crate
