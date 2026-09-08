# Iteration 734 — Research Batch: Functional / Reactive / Actor Model (2026)

**Date**: 2026-09-07 | **Batch**: 734 | **Sources**: 24 results across 3 domains

---

## 1. FUNCTIONAL PROGRAMMING — Rust FP Landscape 2026

### New Findings

| Finding | Source | Novel Defect |
|---------|--------|-------------|
| **fp-library v0.17** — Stable-Rust HKT emulation via Brand pattern + type-level defunctionalization. GAT-free, zero-cost static dispatch. | [crates.io/fp-library](https://crates.io/crates/fp-library), updated 2026-04-19 | NeoTrix has **no HKT abstraction layer** — perception pipeline operators (map/filter/fold) are hand-rolled per module instead of generic over monadic context. Brand-inference pattern (turbofish-free dispatch) could unify NT-WORLD/NT-MEMORY transform chains. |
| **OrdoFP v0.1.0** — Full FP toolbelt: HList, profunctor optics, monad transformers, row-typed effects (`nexus`), `ParFlumen` (rayon-backed parallel), 15 algebraic law families verified via property-test crate (`ordofp_laws`). MSRV 1.97 stable. | [github.com/ordokr/ordofp](https://github.com/ordokr/ordofp) | **NeoTrix has no algebraic law verification** for its SEAL pipeline stage transitions. OrdoFP's law-checking pattern (contract enforcement via property tests) could be adapted for NT-MIND evolution pipeline correctness. |
| **rsmonad v0.2.4** — `monad!` macro auto-derives Functor/Applicative/Alternative/Monoid + property-based monad-law tests. Non-lifetime binder feature pending in Rust. | [crates.io/rsmonad](https://crates.io/crates/rsmonad) | **Missing monadic composition for confidence propagation** — NeoTrix's `Confidence` type (brain.py) has no bind/flatMap; each trait implementation manually threads confidence. `rsmonad`'s `monad!` macro pattern could auto-derive `ConfidenceM` monad with law guarantees. |
| **fp-library Rayon integration** — `par_*` functions use true parallel fold/map via rayon, falling back to sequential without feature. | fp-library docs | **NT-MEMORY knowledge graph traversal is single-threaded**. No parallel fold over KB nodes/edges. `ParFunctor`/`ParFoldable` trait pattern could parallelize `subgraph()` traversal without manual rayon spawning. |

### Defects Identified

1. **D-FP-01**: No HKT-aware operator abstraction — each module re-implements map/filter/fold for its own container types
2. **D-FP-02**: No algebraic law verification for SEAL pipeline stage transitions (Functor/Monad laws unchecked)
3. **D-FP-03**: Confidence type lacks monadic bind — manual confidence threading violates DRY and risks inconsistent propagation
4. **D-FP-04**: KB graph traversal not parallelized — single-threaded fold bottleneck for large subgraphs

---

## 2. REACTIVE PROGRAMMING — RxRust & Event-Driven 2026

### New Findings

| Finding | Source | Novel Defect |
|---------|--------|-------------|
| **rxRust 1.0.0-rc.5** (2026-05-08) — Context-trait architecture: `Local` (Rc/RefCell) vs `Shared` (Arc/Mutex) context propagation via GAT `With<T>`. Compile-time DI for scheduler+ownership. | [crates.io/rxrust](https://crates.io/crates/rxrust) | **NeoTrix EventBus has no compile-time context differentiation** — EventBus uses runtime enum dispatch (Local/Shared) instead of type-level context. rxRust's GAT-based `Context::With<T>` pattern eliminates runtime branch on thread-safety. |
| **rxRust CoreObservable trait** — Pure environment-agnostic observable logic separated from Context. Operators implement `CoreObservable<C: Context>` — logic vs environment cleanly split. | [rxRust architecture deep dive](https://github.com/rxRust/rxRust/blob/master/guide/advanced/architecture_deep_dive.md) | **NT-WORLD crawl pipeline conflates observable logic with execution context** — `UnifiedCrawler` embeds tokio runtime decisions inside stream operators instead of abstracting via Context. |
| **rx_event_bus v0.2.2** (2026-04-03) — Reactive event bus built on rxRust. LocalEventBus/SharedEventBus with typed validation, async subscription via `observe_on(SharedScheduler)`. | [crates.io/rx_event_bus](https://crates.io/crates/rx_event_bus) | **NeoTrix EventBus lacks event validation** — rx_event_bus validates events at publish-time (generic `Fn(T) -> Result<T,E>`). NT EventBus accepts any event without schema validation. |
| **rx-rust (rx-rust crate)** — Multi-scheduler support: tokio, async-std, thread-pool, local-pool. Feature-gated scheduler selection. | [lib.rs/rx-rust](https://lib.rs/crates/rx-rust) | **NeoTrix EventBus is tokio-hardcoded** — no runtime-agnostic scheduler abstraction. Cannot run on async-std or custom executor without EventBus rewrite. |

### Defects Identified

1. **D-RX-01**: EventBus uses runtime enum dispatch instead of compile-time context (rxRust GAT pattern)
2. **D-RX-02**: Crawl pipeline conflates observable logic with execution environment (CoreObservable separation missing)
3. **D-RX-03**: No event validation at publish-time (rx_event_bus validation pattern absent)
4. **D-RX-04**: EventBus tokio-hardcoded — no multi-runtime scheduler abstraction

---

## 3. ACTOR MODEL — Actix, Riker, Distributed Actors 2026

### New Findings

| Finding | Source | Novel Defect |
|---------|--------|-------------|
| **Actix-Telepathy** — Distributed actor extension for Actix: transparent remote messaging, cluster membership, typed serialization for remote messages. Published 2023, still the only viable distributed actor lib for Rust. | [doi.org/10.1145/3623506.3623575](https://doi.org/10.1145/3623506.3623575) | **No distributed actor support in NeoTrix** — consciousness modules (E8/GWT/SEAL) communicate via shared EventBus only. No transparent remote messaging for multi-node deployment. Actix-Telepathy's typed remote serialization pattern is applicable. |
| **Actix 0.13.5** — Actor trait with lifecycle hooks (started/stopping/stopped), typed messages, `Addr` handles, `Recipient` for type-erased send, supervisor trees. | [actix.rs](https://actix.rs/docs/actix/actor/) | **NeoTrix has no supervisor tree** — actor failure in NT-WORLD/NT-ACT is not cascading-recovery. Actix's supervision strategy pattern could wrap consciousness modules. |
| **Riker 0.4.2** — Actor system with Props (factory config), ActorRef (cheap clone), at-most-once delivery, mailbox ordering guarantees. Event sourcing + CQRS support. | [riker.rs](https://riker.rs/) | **No mailbox ordering guarantees in NT EventBus** — messages are unordered broadcast. Riker's per-actor mailbox ordering could ensure sequential processing for single-consumer actors (e.g., NT-MEMORY version chain). |
| **Actix mailboxes with backpressure** — Mailbox size monitoring, bounded queues prevent OOM under load. | [developers-heaven.net](https://developers-heaven.net/blog/implementing-actor-based-models-with-actix-or-similar-frameworks/) | **NT EventBus is unbounded** — no backpressure. Under burst load (e.g., crawl storm), EventBus can OOM. Bounded mailbox with backpressure essential. |
| **Actix supervision trees** — Supervisor actors monitor worker health, restart on failure, configurable restart strategies. | Same source | **No failure isolation between consciousness modules** — crash in NT-FEEL propagates to NT-CORE via shared EventBus. Supervision tree could isolate and restart individual modules. |

### Defects Identified

1. **D-ACT-01**: No distributed actor support (Actix-Telepathy pattern missing)
2. **D-ACT-02**: No supervision tree — consciousness module crashes propagate globally
3. **D-ACT-03**: EventBus has no per-consumer ordering guarantees (Riker mailbox pattern absent)
4. **D-ACT-04**: EventBus is unbounded — no backpressure under burst load
5. **D-ACT-05**: No failure isolation between consciousness modules via supervision

---

## Cross-Domain Synthesis

| Pattern | FP Source | Reactive Source | Actor Source | NeoTrix Gap |
|---------|-----------|----------------|-------------|-------------|
| **Compile-time safety** | HKT/Brand inference | GAT Context::With<T> | Typed messages | Runtime enum dispatch everywhere |
| **Law verification** | Monad law property tests | CoreObservable contract | Message delivery guarantees | No algebraic contracts on any pipeline |
| **Backpressure** | Lazy evaluation / Thunk | Observable backpressure | Bounded mailboxes | Unbounded EventBus |
| **Distributed** | — | — | Actix-Telepathy | Single-node only |
| **Supervision** | — | — | Actix supervision trees | No crash recovery hierarchy |

---

## Summary: 13 New Defects Found

| ID | Domain | Defect | Severity |
|----|--------|--------|----------|
| D-FP-01 | FP | No HKT-aware operator abstraction | Medium |
| D-FP-02 | FP | No algebraic law verification for SEAL | High |
| D-FP-03 | FP | Confidence type lacks monadic bind | Medium |
| D-FP-04 | FP | KB traversal single-threaded | Low |
| D-RX-01 | Reactive | EventBus runtime dispatch vs compile-time | High |
| D-RX-02 | Reactive | Crawl pipeline conflates logic/context | Medium |
| D-RX-03 | Reactive | No event validation at publish | High |
| D-RX-04 | Reactive | Tokio-hardcoded, no multi-runtime | Medium |
| D-ACT-01 | Actor | No distributed actor support | High |
| D-ACT-02 | Actor | No supervision tree | Critical |
| D-ACT-03 | Actor | No per-consumer ordering guarantees | Medium |
| D-ACT-04 | Actor | EventBus unbounded, no backpressure | Critical |
| D-ACT-05 | Actor | No failure isolation between modules | Critical |

---

## Sources Cited

1. fp-library v0.17 — https://crates.io/crates/fp-library (2026-04-19)
2. OrdoFP v0.1.0 — https://github.com/ordokr/ordofp
3. rsmonad v0.2.4 — https://crates.io/crates/rsmonad
4. Monads in Rust (SoftwarePatternsLexicon) — https://softwarepatternslexicon.com/rust/functional-programming-patterns-in-rust/monads-and-monad-like-patterns-in-rust/
5. rxRust 1.0.0-rc.5 — https://crates.io/crates/rxrust (2026-05-08)
6. rxRust Architecture Deep Dive — https://github.com/rxRust/rxRust/blob/master/guide/advanced/architecture_deep_dive.md
7. rx_event_bus v0.2.2 — https://crates.io/crates/rx_event_bus (2026-04-03)
8. rx-rust — https://lib.rs/crates/rx-rust
9. Actix 0.13.5 — https://actix.rs/docs/actix/actor/
10. Actix GitHub — https://github.com/actix/actix
11. Riker 0.4.2 — https://riker.rs/
12. Riker docs — https://docs.rs/crate/riker/latest
13. Actix-Telepathy — https://doi.org/10.1145/3623506.3623575 (2023)
14. Actix Actor Models Guide — https://developers-heaven.net/blog/implementing-actor-based-models-with-actix-or-similar-frameworks/ (2026-06-25)
