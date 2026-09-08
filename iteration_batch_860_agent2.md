# Agent 2: Data Race Prevention (Batch 860)

## Sources

1. **Rustonomicon: Send and Sync** — Official Rust reference on marker traits, auto-derivation, UnsafeCell/Rc/RawPointer exclusions, and soundness proof obligations for manual impls.
2. **Rustonomicon: Races** — Defines data races (concurrent access + at least one write + unsynchronized), distinguishes data races from general race conditions, confirms UB classification.
3. **Rustonomicon: Atomics** — C++20 memory model inheritance, ordering semantics (Relaxed/Acquire/Release/AcqRel/SeqCst), happens-before relationships, hardware weak/strong ordering implications.
4. **Rust std::sync::atomic docs** — Mixed-size access UB, read-only memory atomic UB, lock-free guarantees, architectural cost differences.
5. **Mara Bos: Rust Atomics and Locks Ch.3** — Formal happens-before model, fence semantics, SeqCst total order, release-acquire pair mechanics.
6. **Loom (tokio-rs)** — Concurrency permutation testing under C11 model, state reduction, SeqCst false-positive limitation, relaxed ordering blind spots, preemption bounds.
7. **Crossbeam ecosystem** — Epoch-based GC for lock-free structures, ArrayQueue/SegQueue, work-stealing deques, ShardedLock, known ThreadSanitizer false-positive in deque volatile reads.
8. **Corrode: Rust Prevents Data Races, Not Race Conditions** — Critical distinction: compiler prevents UB data races but not logic race conditions (deadlock, lost updates, check-then-act).
9. **Corrode: Pitfalls of Safe Rust** — Race conditions as logic bugs, TOCTOU, oversubscription, Mutex poisoning cascades, static analysis limitations.
10. **Hivebook: Arc<Mutex> Deadlocks** — Guard temporary lifetime gotcha (end-of-statement not end-of-expression), non-reentrant deadlock, poisoning cascade.
11. **Juan Torchia: Mutex Deadlock Production Diagnosis** — Three deadlock patterns (Classic Embrace, Reentrant Lock, Inverted Order Under Pressure), tokio-console diagnosis, timeout-on-lock pattern.
12. **Surelock: Hierarchical Mutex** — Compile-time lock ordering via type system, encoding acquisition level in types, async integration gaps.
13. **Microsoft Rust Training: Concurrency Patterns** — SeqLock UnsafeCell caveat (technically UB on Rust abstract machine), lock-free counter patterns, Arc<Mutex> vs channel decision matrix.
14. **DevPro Portal: Mastering Rust Concurrency** — Critical section scoping, atomic ordering selection (Relaxed for counters vs SeqCst for global timeline), oversubscription costs.
15. **Deno SendPtr Issue #34453** — Unsound `unsafe impl Send for SendPtr<T>` without `T: Send` bound, NAPI pointer crossing thread boundaries.
16. **Rudra SendSyncVariance** — Automated detection of suspicious `unsafe impl Sync` without proper generic bounds, NAIVE_SYNC_FOR_SYNC pattern.
17. **Rust-clippy #6638** — Lint for Sync types generic over T without `T: Send` bound, soundness via Mutex/Channel patterns requiring `T: Send`.
18. **HackerOne: Rust Safety Concurrency** — Ownership preventing data races at compile-time, Arc<Mutex> patterns, ecosystem tools (rayon, tokio, crossbeam).
19. **Fearless Concurrency Patterns** — Detached JoinHandle resource leaks, RwLock starvation, atomic ordering selection guide, channel shutdown signal patterns.
20. **NeoTrix codebase: nt_core_event_bus.rs** — EventBus with broadcast channel, sync_handlers Mutex, hooks Mutex, handles Mutex, seq AtomicU64, Clone impl dropping hooks/handlers.

## Defects

### D-RACE-001: EventBus Clone Drops Registered Hooks and Sync Handlers
**File:** `neotrix-core/src/neotrix/nt_core_event_bus.rs:40-51`
**Severity:** HIGH (logical race / silent data loss)
**Source:** Rustonomicon Send/Sync + Corrode "Race Conditions Are Logic Bugs"
**Description:** The `Clone` impl for `EventBus` creates fresh empty `Mutex<Vec>` instances for `handles` and `sync_handlers`, and clones the `Arc` for `hooks`. While the `Arc<Dispatcher>` clone correctly shares hooks across clones, the `sync_handlers` field is replaced with a new empty `Vec`. If a cloned EventBus registers a sync handler, it will be invisible to the original's `emit_from()` call path (line 134-138), and vice versa. This creates a logical split: cloned EventBus instances silently diverge in their handler sets. In a multi-domain consciousness architecture where EventBus clones are passed to NT-CORE, NT-MIND, NT-WORLD, etc., a sync handler registered on one clone won't execute on emit from another, breaking the "two-phase sync" contract documented at line 35-36.

### D-RACE-002: EventBus emit_from Holds Two Locks Simultaneously (hooks + log_file)
**File:** `neotrix-core/src/neotrix/nt_core_event_bus.rs:132-169`
**Severity:** MEDIUM (deadlock potential)
**Source:** Surelock hierarchical mutex + Juan Torchia "Inverted Order Under Pressure"
**Description:** `emit_from()` acquires `self.sync_handlers.lock()` (line 134), then `self.hooks.lock()` (line 141-144), then `self.log_file.lock()` (line 157). Three separate Mutex guards are acquired in sequence without an explicit ordering contract. If any sync_handler or hook callback attempts to emit another event on the same EventBus (reentrant emit), it will deadlock on `sync_handlers` (std::sync::Mutex is not reentrant). Even without reentrancy, holding `hooks` lock while acquiring `log_file` lock creates a lock ordering dependency that must be globally consistent across all EventBus instances. Under high event throughput (SEAL pipeline ticks, background loop), this triple-lock sequence becomes a contention hotspot.

### D-RACE-003: Inconsistent Atomic Ordering on bm25_dirty Flag
**File:** `crates/neotrix-types/src/core/nt_core_bank/bank/bank_impl/store.rs:88,137`
**Severity:** MEDIUM (memory ordering violation)
**Source:** Rustonomicon Atomics + Mara Bos Ch.3 Memory Ordering
**Description:** In `bank_impl/store.rs`, `bm25_dirty.store(true, Ordering::SeqCst)` at line 88 uses SeqCst, while line 137 uses `bm25_dirty.store(true, Ordering::Relaxed)`. The same flag is loaded with `Ordering::SeqCst` in `search.rs:40`. Mixing SeqCst stores with Relaxed stores on the same AtomicBool creates an inconsistent synchronization protocol. A Relaxed store on ARM/weakly-ordered hardware may not be visible to a subsequent SeqCst load from another thread in the expected order, potentially causing stale BM25 index reads. The flag's purpose (dirty-check for lazy rebuild) requires at minimum Acquire/Release pairing to ensure data written before the store is visible after the load.

### D-RACE-004: Panoramic dirty Flag Uses Release/Acquire but Lacks Store-Load Ordering
**File:** `crates/neotrix-types/src/core/panoramic.rs:148,184,259,263`
**Severity:** MEDIUM (stale read under contention)
**Source:** Mara Bos Ch.3 + Crossbeam deque volatile read issue #646
**Description:** `Panoramic` uses `dirty.store(true, Ordering::Release)` for writes and `dirty.load(Ordering::Acquire)` for reads. While this correctly establishes happens-before for data mutation visibility, it does not prevent a TOCTOU race: Thread A calls `scored()` which loads `dirty=true` (Acquire), then begins recomputation. Thread B simultaneously calls `mark_dirty()` which stores `true` (Release). Thread A's recomputation may use stale inputs because the Release store from B doesn't synchronize with A's Acquire load of the dirty flag — they're on different atomic variables. The Panoramic struct should either use SeqCst for the dirty flag or wrap the check-then-recompute in a Mutex to prevent concurrent recomputation.

### D-RACE-005: nt_shield Guardrails Tool Call Counter Uses SeqCst for Independent Counter
**File:** `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield/guardrails.rs:223,246`
**Severity:** LOW (performance, correctness risk)
**Source:** Rustonomicon Atomics + DevPro Portal atomic ordering selection
**Description:** `tool_call_count.fetch_add(1, Ordering::SeqCst)` and `tool_call_count.store(0, Ordering::SeqCst)` use the strongest ordering for an independent counter that is only checked locally. Per Rustonomicon guidance, independent counters should use `Ordering::Relaxed` — SeqCst imposes unnecessary global synchronization barriers on every tool call, which is on the hot path for NT-ACT tool execution. More critically, if this counter is ever read from another thread (e.g., during GWT attention routing in NT-CORE), the SeqCst ordering creates a false sense of synchronization that may mask a missing happens-before relationship with other shared state.

### D-RACE-006: nt_core_event_bus seq Counter May Overlap with File Write
**File:** `neotrix-core/src/neotrix/nt_core_event_bus.rs:149-166`
**Severity:** LOW (monotonicity gap)
**Source:** Crossbeam volatile read issue + Corrode "Pitfalls of Safe Rust"
**Description:** The `seq` counter is incremented atomically (line 150) before the event is written to the log file (line 157-163). If the file write fails or the process crashes between increment and write, the seq will have a gap. While the seq is documented as "monotonically increasing" (line 14), the gap means `replay_and_broadcast` may skip events that were assigned seq numbers but never persisted. The `EventEnvelope` struct (line 11-21) stores `seq` for replay ordering, so a gap breaks the event chain reconstruction guarantee.

### D-RACE-007: No Loom Coverage for Critical Concurrent Structures
**File:** NeoTrix project-wide (crossbeam not in Cargo.toml, no loom tests found)
**Severity:** HIGH (verification gap)
**Source:** Loom documentation + matklad "Properly Testing Concurrent Data Structures"
**Description:** NeoTrix has 56+ `Arc::new(Mutex::new(...))` sites, 100+ atomic ordering usages, 72+ `tokio::spawn` / `std::thread::spawn` sites, but zero Loom-based concurrency tests. Loom can exhaustively explore all valid interleavings under the C11 memory model for instrumented concurrent structures. Without Loom coverage, the EventBus broadcast+lock pattern, the KB write guard sequence counter, the shield proxy kernel's AtomicU32 IP allocator, and the background loop's Arc<Mutex<BackgroundLoopHandle>> are all untested for concurrency correctness. Loom's state reduction can handle structures with up to ~5 concurrent threads with preemption bound of 2-3.

### D-RACE-008: nt_shield Traffic Analyzer Arc<Mutex> Crosses Async Boundary
**File:** `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mod.rs:34` and `api_proxy.rs:55`
**Severity:** MEDIUM (async stall risk)
**Source:** Hivebook "Arc<Mutex> Deadlocks" + Juan Torchia "Mutex Deadlock Production Diagnosis"
**Description:** `TrafficAnalyzer` is wrapped in `Arc<Mutex<TrafficAnalyzer>>` and used in async contexts (tokio::spawn at api_proxy.rs:353). If the Mutex is held across an `.await` point (e.g., during HTTP request processing in the proxy), it blocks the executor thread entirely with `std::sync::Mutex`. The production diagnosis pattern shows this caused full runtime freezes. The fix is either: (a) use `tokio::sync::Mutex` if lock must span await, or (b) extract data from lock, drop guard, then proceed with async work.

### D-RACE-009: Background Loop Arc<Mutex<BackgroundLoopHandle>> Potential Reentrant Lock
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:598`
**Severity:** MEDIUM (deadlock under reentrant call)
**Source:** Surelock + Juan Torchia "Pattern 2: The Reentrant Lock"
**Description:** `BackgroundLoopHandle` is wrapped in `Arc<Mutex>` and passed to spawned tasks (run.rs:738, 894 via tokio::spawn). If any spawned task's handler callback (handlers.rs:11) re-enters the background loop API (e.g., calling `self.handles.push()` while the outer Mutex is still held), it will deadlock. std::sync::Mutex is not reentrant. The clone-then-operate pattern ("Take-Swap-Operate") should be used: clone the Arc, drop the lock, then operate on the clone.

### D-RACE-010: NT-CORE Parallel Executor Uses tokio::spawn Without JoinHandle Tracking
**File:** `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/executor.rs:37`
**Severity:** MEDIUM (resource leak + silent failure)
**Source:** Fearless Concurrency "Detached JoinHandle" + Corrode "Pitfalls of Safe Rust"
**Description:** `tokio::spawn` returns a `JoinHandle` but in several locations (executor.rs:37, coordinator.rs:89, and others) the handles are either dropped immediately or not awaited. A dropped JoinHandle continues running but its result is silently discarded. If the spawned task panics, the panic is lost unless `catch_unwind` is used. For the parallel executor that decomposes tasks into AtomicUnits, a silent panic in a parallel sub-task would produce incomplete results without any error signal. The coordinator at coordinator.rs:89 does collect handles, but the executor at line 37 does not.

## Key Insights

1. **Rust prevents data races but not race conditions.** NeoTrix's 56+ Arc<Mutex> sites and 72+ spawn sites are data-race-free at compile time, but the EventBus triple-lock pattern, background loop reentrant risk, and traffic analyzer async boundary are all *logic* race conditions that compile cleanly.

2. **Atomic ordering inconsistency is the hidden UB vector.** Mixing SeqCst and Relaxed stores on the same AtomicBool (bm25_dirty) creates architecture-dependent behavior that works on x86 (strongly ordered) but may fail on ARM (weakly ordered). The project targets macOS (Apple Silicon ARM) — testing on x86 alone won't catch this.

3. **Clone semantics diverge from user expectations.** EventBus::clone() silently drops sync_handlers, creating a split-brain where different clones have different handler sets. This is a documentation + API design issue, not a memory safety issue, but it breaks the consciousness architecture's two-phase sync contract.

4. **Missing Loom coverage is the biggest risk.** With zero Loom tests across 56+ concurrent structures, the project relies entirely on code review and stress testing to find concurrency bugs. Loom can exhaustively verify structures with ≤5 threads and preemption bound 2-3, covering the critical path structures (EventBus, KB write guard, shield proxy kernel).

5. **The SeqCst-default habit hides real synchronization needs.** Many atomic operations use SeqCst as a "safe default" (guardrails, event bus seq, plugin registry counters), which is correct but obscures the actual happens-before relationships. This makes it harder to reason about which operations actually need synchronization and which are over-synchronized.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| D-RACE-001 through D-RACE-010 | 10 |
| HIGH severity | 2 |
| MEDIUM severity | 6 |
| LOW severity | 2 |
| Sources consulted | 20 |
