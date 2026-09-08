# Agent 3: Async Channel Patterns (Batch 870)

## Sources
1. https://www.toolsku.com/en/blog/rust-tokio-channel-patterns-2026 — 6 production channel patterns, pitfalls, capacity sizing
2. https://rustz2h.com/chapter_07_mastering_async_rust_and_tokio/series_03_async_channels_and_synchronization/backpressure_patterns_async — bounded/unbounded backpressure, Semaphore patterns, cascading backpressure
3. https://brandonwie.dev/posts/rust-async-channels — stream-vs-handoff decision rule, bounded default
4. https://www.rustfaq.org/en/how-to-handle-backpressure-in-async-rust — backpressure as default, load shedding with try_send
5. https://tokio.rs/tokio/tutorial/channels — official tutorial, channel selection guide
6. https://docs.rs/tokio/latest/tokio/sync/watch/index.html — watch channel borrow_and_update race condition warning
7. https://github.com/openclaw/skills/blob/main/skills/anderskev/rust-code-review/references/async-concurrency.md — fire-and-forget tasks, cancellation safety
8. https://rustycloud.org/foundation_track/module-03-message-passing-patterns/lesson-01-mpsc.html — MPSC capacity sizing, drop-original-sender pattern

## Defects

### D-CHAN-001: EventBus Clone drops sync_handlers and handles, breaking two-phase emit
**File:** `neotrix-core/src/neotrix/nt_core_event_bus.rs:40-51`
**Severity:** HIGH
**Source:** Source 1 (Pitfall 1: dropping sender), Source 6 (watch race condition — same anti-pattern for sync handlers)

The `Clone` impl for `EventBus` creates **empty** `Vec` for both `handles` and `sync_handlers`:
```rust
impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            // ...
            handles: std::sync::Mutex::new(Vec::new()),    // EMPTY
            sync_handlers: std::sync::Mutex::new(Vec::new()), // EMPTY
        }
    }
}
```
A cloned `EventBus` (used extensively — e.g., `run.rs:645` passes `event_bus.as_ref().clone()`) will emit events **without** executing sync handlers (KB persistence, state updates) and **without** tracking subscriber thread handles for shutdown. This silently breaks the two-phase emit guarantee (Phase 1: sync handlers → Phase 2: broadcast). The `run.rs:404` `register_sync_handler` call targets the original bus, but clones used elsewhere skip Phase 1 entirely.

### D-CHAN-002: ElementBus publish uses try_send — silent message loss without backpressure
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/consciousness/element/bus.rs:49`
**Severity:** MEDIUM
**Source:** Source 4 (try_send for load shedding, not for default path), Source 1 (bounded channel backpressure)

`ElementBus::publish` uses `try_send` on every subscriber:
```rust
let _ = sender.try_send(payload.clone());
```
If any subscriber's buffer (capacity 64) is full, the event is silently dropped. Per best practice (Source 4), `try_send` should only be used for load-shedding of low-priority items, not as the default publish path. For the consciousness element bus — where `CapabilityUpdated`, `MemoryStored`, and `GoalStateChanged` events carry critical cross-module coordination — silent loss creates inconsistency between producers and consumers. The `_ =` discard also masks `Closed` errors, meaning dead subscribers are never cleaned up.

### D-CHAN-003: ElementBus holds std::sync::Mutex across try_send — deadlock risk under contention
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/consciousness/element/bus.rs:46-51`
**Severity:** MEDIUM
**Source:** Source 1 (Pitfall 4: Mutex lock across await), Source 7 (sync mutex in async code)

`publish()` locks a `std::sync::Mutex<HashMap<...>>` and then calls `try_send` on each subscriber's mpsc sender. While `try_send` is non-blocking, if a subscriber channel is full, the `Full` error path still executes under the lock. More critically, the lock is held during the entire fan-out iteration. Under high contention (many concurrent publishers from different tokio tasks), this `std::sync::Mutex` can block the tokio worker thread, degrading runtime scheduling. Should use `tokio::sync::Mutex` or a lock-free concurrent map.

### D-CHAN-004: Unbounded channels in hotreload and plugin registry — no backpressure on filesystem events
**File:** `neotrix-core/src/unified/layers/action/nt_io/nt_io_hotreload/mod.rs:137`, `nt_io_plugin/registry.rs:436`
**Severity:** MEDIUM
**Source:** Source 2 (unbounded = OOM risk), Source 3 (bounded as default)

Three locations use `tokio::sync::mpsc::unbounded_channel`:
- `nt_io_hotreload/mod.rs:137` — filesystem watcher events
- `nt_io_plugin/registry.rs:436` — plugin directory watcher events
- `nt_shield_proxy_kernel/kernel.rs:125` — listener error reporting

Per best practice (Source 2: "Unbounded channels risk memory exhaustion. The only exception is when message production is guaranteed to be bounded"). Filesystem watchers during bulk operations (git checkout, npm install) can generate hundreds of events per second. The hotreload consumer processes events sequentially with a 200ms sleep per reload (`nt_io_hotreload/mod.rs:177`), creating a growing backlog. Should use bounded channels with `try_send` for load shedding.

### D-CHAN-005: Gateway streaming uses channel(1) — blocks producer on single buffered message
**File:** `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/gateway/mod.rs:546,1198`, `nt_core_llm.rs:971,988,1009,1082`
**Severity:** LOW
**Source:** Source 1 (capacity sizing: too small = unnecessary backpressure), Source 8 (capacity 2-4x expected burst)

Seven locations create `mpsc::channel(1)` for LLM streaming responses. While these are test/mock implementations, the pattern is established in production-adjacent code. Channel capacity of 1 means the producer task (spawning the LLM response stream) blocks after sending the first token until the consumer reads it. For streaming responses with multiple chunks, this creates unnecessary synchronization. Should use capacity ≥4 for streaming scenarios to allow producer-consumer overlap.

### D-CHAN-006: subscribe_all_layers_sync uses busy-polling with 10ms sleep — wastes CPU
**File:** `neotrix-core/src/neotrix/nt_core_event_bus.rs:440-464`
**Severity:** LOW
**Source:** Source 5 (async recv vs try_recv), Source 8 (channel patterns)

The synchronous layer subscriber loop uses `try_recv()` with a 10ms `thread::sleep` on `Empty`:
```rust
Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
    std::thread::sleep(std::time::Duration::from_millis(10));
}
```
This is a spin-poll anti-pattern: 9 threads × 100 polls/sec = 900 unnecessary wakeups/sec. Each wakeup involves acquiring the broadcast mutex. Should use `tokio::sync::Notify` or `recv()` on a dedicated runtime, or at minimum increase the sleep interval. The async version (`subscribe_layer`) correctly uses `rx.recv().await`.

### D-CHAN-007: EventBus flood_guard uses discriminant-only dedup — loses per-field event identity
**File:** `neotrix-core/src/neotrix/nt_core_event_bus.rs:186-203`
**Severity:** LOW
**Source:** Source 1 (broadcast channel semantics), Source 2 (backpressure patterns)

The `flood_guard` deduplicates by `Discriminant<CoreEvent>`, meaning ALL events of the same variant within `min_interval` are dropped. For `SystemError { component, error, severity }`, two errors from different components (e.g., "kb:timeout" and "llm:refused") are collapsed into one. This can mask cascading failures in the consciousness loop. Should key on discriminant + critical field (e.g., component name) to allow distinct error sources through.

## Key Insights

1. **Clone semantics are the #1 channel defect vector** in NeoTrix. The EventBus Clone dropping registrations is a silent correctness bug — cloned buses emit without Phase 1 sync handlers, breaking KB persistence guarantees.

2. **Unbounded channels are a ticking OOM bomb** in the file-watching subsystem. The hotreload and plugin watcher patterns are classic "fast producer, slow consumer" with no backpressure valve.

3. **The ElementBus design conflicts with the EventBus design**. ElementBus uses mpsc (one consumer per subscription) while EventBus uses broadcast (all consumers see all events). ElementBus's `try_send` silently drops, EventBus's `Lagged` error provides a signal. This inconsistency makes consciousness element coordination unreliable.

4. **The sync subscriber polling pattern wastes resources**. 9 threads doing `try_recv + sleep(10ms)` is a classic busy-wait that should use `Notify` or condvar.

5. **Channel capacity sizing is inconsistent**. Most providers use 64, gateway tests use 1, hotreload uses unbounded, EventBus uses 1024. No capacity tuning documentation or rationale exists.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 7 |
| Sources consulted | 8 |
| Files analyzed | 12 |
