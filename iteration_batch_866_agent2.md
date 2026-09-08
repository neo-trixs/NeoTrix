# Agent 2: Async Channel Patterns (Batch 866)

## Sources
1. https://docs.rs/tokio/latest/tokio/sync/ — Tokio sync module docs (mpsc, broadcast, watch, oneshot)
2. https://www.toolsku.com/en/blog/rust-tokio-channel-patterns-2026/ — 6 production channel patterns
3. https://chandanbhagat.com.np/async-rust-with-tokio-part-5-channels-and-communic/ — Channel communication patterns
4. https://fettblog.eu/rust-tokio-guide/channels/ — Tokio channel guide (mpsc+oneshot actor pattern)
5. https://tokio.rs/tokio/tutorial/channels — Official Tokio tutorial channels (backpressure & bounded)
6. https://rs4ts.dev/11-async/08-channels/ — Channel type comparison & common pitfalls
7. https://microsoft.github.io/RustTraining/async-book/ch13-production-patterns.html — Production patterns (shutdown, backpressure, structured concurrency)
8. https://rustycloud.org/data_pipelines_track/module-04-backpressure-and-flow-control/lesson-01-bounded-channels.html — Bounded channels & FlowPolicy
9. https://docs.rs/tokio/latest/tokio/sync/broadcast/ — Broadcast channel docs (Lagged semantics, slow receiver)
10. https://github.com/tokio-rs/tokio/issues/4625 — broadcast sender drop notification bug
11. https://github.com/tokio-rs/tokio/pull/8197 — broadcast channel contention sharding
12. https://docs.rs/tokio/latest/tokio/sync/watch/ — Watch channel docs (borrow vs borrow_and_update race)
13. https://github.com/tokio-rs/tokio/issues/3168 — watch channel multi-threaded misbehavior
14. https://github.com/tokio-rs/tokio/issues/5403 — watch channel subscriber contention
15. https://github.com/tokio-rs/tokio/issues/4225 — oneshot send/recv race leading to panic

## Defects

**D-CHAN-001: EventBus broadcast capacity hardcoded to 1024 with no per-edge sizing rationale**
File: `neotrix-core/src/neotrix/nt_core_event_bus.rs:62,86` | Severity: MEDIUM
The EventBus creates `broadcast::channel(1024)` in both `new()` and `with_persistence()`. The broadcast channel rounds capacity to the next power of two (Tokio internals), and the buffer is per-receiver ring buffer. With 9 layer subscribers + arbitrary `subscribe()` callers, the effective memory is `1024 × sizeof(CoreEvent) × receiver_count`. No rationale is documented for why 1024 specifically. Per the production patterns source, channel capacity should be sized per-edge based on `peak_rate × burst_duration × safety_factor`. For a consciousness architecture event bus carrying critical `GlobalHalt` and `SystemError` events, an undersized buffer causes silent event loss (Lagged), while an oversized one wastes memory. Source: toolsku.com, rustycloud.org.

**D-CHAN-002: EventBus `subscribe_all_layers_sync` uses blocking try_recv + sleep(10ms) polling — thread CPU waste**
File: `neotrix-core/src/neotrix/nt_core_event_bus.rs:440-464` | Severity: HIGH
The synchronous subscriber loop calls `rx.try_recv()` then `std::thread::sleep(Duration::from_millis(10))` on `TryRecvError::Empty`. This is a classic busy-poll anti-pattern: each of the 9 layer threads burns CPU spinning every 10ms even when idle. The correct pattern is `broadcast::Receiver::recv().await` (async) or, for sync contexts, `std::sync::mpsc` with blocking recv. The sleep(10ms) introduces up to 10ms latency for every event delivery and wastes ~9 threads × ~100 wakeups/sec of CPU. Source: tokio.rs tutorial ("Concurrency and queuing must be explicitly introduced"), rs4ts.dev.

**D-CHAN-003: ElementBus silently drops messages on `try_send` — no backpressure, no metrics**
File: `neotrix-core/src/unified/layers/cognition/nt_mind/consciousness/element/bus.rs:49` | Severity: HIGH
`let _ = sender.try_send(payload.clone())` discards the error when the mpsc buffer (capacity 64) is full. This means consciousness element messages (CapabilityUpdated, MemoryStored, GoalStateChanged) are silently lost under load with zero observability. Per production best practices, every `try_send` site must have a counter increment and structured log entry on `Full` (rustycloud.org FlowPolicy pattern). The element bus is a critical consciousness-internal communication path — silent drops can cause state desynchronization between elements. Source: rustycloud.org ("The trap is try_send with no logging"), toolsku.com.

**D-CHAN-004: ProxyKernel uses unbounded_channel for listener errors — potential unbounded memory growth**
File: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:125` | Severity: MEDIUM
`tokio::sync::mpsc::unbounded_channel::<String>()` is used for listener error reporting. Under sustained listener failures (e.g., port conflict storm, upstream proxy cascade), error strings accumulate without bound. The receiver is behind an `Arc<RwLock<>>` (line 109), adding further lock contention. Per production patterns, error reporting channels should be bounded with `try_send` + counter for load shedding. The unbounded channel is described as a "footgun for data-path edges" (rustycloud.org). Source: rustycloud.org, tokio.rs tutorial ("Unbounded queues will eventually fill up all available memory").

**D-CHAN-005: ProxyKernel main loop holds write lock on `listener_error_rx` during recv — blocks concurrent error sends**
File: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:254` | Severity: HIGH
`let mut error_rx = kernel.listener_error_rx.write().await` takes a write lock on the `Arc<RwLock<UnboundedReceiver>>` for the entire duration of the main select loop. While this lock is held, any clone of `listener_error_tx` that attempts to send will succeed (unbounded), but the receiver cannot be accessed by any other task. More critically, the `.write().await` on an `RwLock` blocks all `.read().await` callers, creating a priority inversion if readers exist. The channel should be moved out of the `RwLock` entirely — `mpsc::Receiver` is not `Clone` by design and should be owned by the single consumer task. Source: tokio.rs ("Only one Receiver is supported"), fettblog.eu.

**D-CHAN-006: Background loop catches `RecvError::Lagged` but does not update consciousness health indicators**
File: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:905-906` | Severity: MEDIUM
When the EventBus consumer lags, the handler only logs `warn!("[bg] event_bus consumer lagged {} events", n)` without feeding the lag count into the consciousness health monitoring system (ConsciousnessMonitor, HeartbeatAggregator). In a consciousness architecture, event lag is a direct signal of cognitive overload — the system is producing events faster than it can process them. This should propagate to `ConsciousnessTree.trunk.coherence` or trigger a self-healing response. The lag count `n` is the number of dropped events, which represents lost awareness. Source: tokio broadcast docs ("The caller may decide how to respond: either by aborting its task or by tolerating lost messages").

**D-CHAN-007: Hotreload and PluginRegistry both use unbounded channels for filesystem watch events — no backpressure**
File: `neotrix-core/src/unified/layers/action/nt_io/nt_io_hotreload/mod.rs:137` and `nt_io_plugin/registry.rs:436` | Severity: MEDIUM
Both use `tokio::sync::mpsc::unbounded_channel::<notify::Result<notify::Event>>()`. Filesystem watchers can generate rapid bursts of events (e.g., `git checkout` touching hundreds of files, build tools writing temp files). With no bound, the channel absorbs all events into memory. The hotreload system should use a bounded channel with `FlowPolicy::Shed` (drop excess events, log a counter) since intermediate filesystem states are not meaningful — only the final state matters. Source: rustycloud.org ("unbounded channels are the wrong default"), toolsku.com.

**D-CHAN-008: Factory gateway initialization spawns std::thread with std::sync::mpsc — panic causes permanent hang**
File: `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/factory.rs:1436-1450` | Severity: HIGH
The gateway factory spawns a `std::thread` that creates a new tokio runtime, runs the async init future, then sends the result via `std::sync::mpsc::channel()`. If the spawned thread panics (e.g., `Runtime::new()` fails, `block_on` panics), the `tx` is dropped without sending, and `rx.recv().unwrap_or_else(|_| GatewayV2::new())` returns an empty gateway — silently degrading the system. The `unwrap_or_else` masks the failure. Per production patterns, the thread should catch panics with `std::panic::catch_unwind` and send an error variant. The `std::mem::forget(rt)` on line 1447 also leaks the runtime permanently. Source: tokio.rs tutorial ("Unbounded queues will eventually fill up"), toolsku.com.

**D-CHAN-009: Behavioral verifier uses `std::sync::mpsc` + busy-polling thread with sleep(200ms) inside async context**
File: `neotrix-core/src/unified/layers/action/nt_act/nt_act_goal/behavioral_verifier.rs:26-38` | Severity: MEDIUM
A `std::sync::mpsc::channel` is created, then a `std::thread` polls `child.try_wait()` every 200ms in a loop. Using `std::sync::mpsc` inside an async application blocks the OS thread when recv is called. The 200ms polling interval wastes CPU and adds up to 200ms latency. The correct pattern is `tokio::process::Command` with `.output().await` or `tokio::task::spawn_blocking` with the blocking recv. Source: rs4ts.dev ("Using std::sync::mpsc inside async code causes silent stalls"), tokio.rs tutorial.

**D-CHAN-010: Watch channel `ShutdownCoordinator` sender leaked via `let _ = shutdown_tx.send(true)` — no delivery guarantee**
File: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:261,596` and `nt_mind_background_loop/handlers.rs:40` | Severity: MEDIUM
Multiple locations use `let _ = self.shutdown_tx.send(true)` or `let _ = coordinator.sender.send(true)` to broadcast shutdown. The `let _` discards the `SendError` that occurs when all receivers are dropped. If a receiver task has already exited (e.g., due to panic or early return), the shutdown signal is silently lost. While `watch` is designed for "latest value" semantics (receivers see the value on next `changed()` call regardless), if receivers have already terminated, the signal never reaches them. A more robust pattern is to combine `watch` shutdown with `JoinSet::abort_all()` for guaranteed termination. Source: tokio docs ("watch: only the most recent value is retained"), docs.rs/tokio/sync/watch.

**D-CHAN-011: No unified channel backpressure strategy across NeoTrix subsystems — inconsistent FlowPolicy**
File: Multiple (all channel creation sites) | Severity: HIGH
NeoTrix uses at least 5 different channel creation patterns with no consistent backpressure strategy:
- `broadcast::channel(1024)` (EventBus) — Lagged error on slow consumers
- `mpsc::channel(64)` (ElementBus, providers, web API) — backpressure via send().await
- `mpsc::unbounded_channel()` (ProxyKernel errors, hotreload, plugin registry) — no backpressure
- `std::sync::mpsc::channel()` (factory, behavioral verifier) — blocking, no async integration
- `oneshot::channel()` (web API) — fire-and-forget

Production systems should encode a per-edge `FlowPolicy` enum (Backpressure/Shed/Timed) as recommended by the bounded channels source. The absence of a unified policy means some paths silently drop (ElementBus), some block (mpsc), some grow unbounded (ProxyKernel), and some block threads (std::sync::mpsc). This makes it impossible to reason about system-wide load behavior. Source: rustycloud.org ("The three send semantics are operationally distinct"), toolsku.com.

**D-CHAN-012: EventBus `emit_from` calls sync_handlers under lock while also holding broadcast Sender — potential priority inversion**
File: `neotrix-core/src/neotrix/nt_core_event_bus.rs:134-168` | Severity: MEDIUM
`emit_from` executes `sync_handlers.lock()` (std::sync::Mutex) synchronously, then calls `self.sender.send(event)` (broadcast). The sync_handlers are described as "Phase 1: must complete before broadcast" — if any sync_handler blocks or takes a long time (e.g., KB write), the broadcast is delayed and all subscribers starve. Additionally, the `hooks.lock()` and `log_file.lock()` are also std::sync::Mutex, creating a chain of lock acquisitions in the critical path. This violates the "don't hold locks across await" principle — though these are sync locks, the broadcast `send` internally acquires a tail mutex, creating potential contention with receivers. Source: tokio.rs tutorial ("Mutex deadlock across await"), toolsku.com ("Lock held while calling .await").

## Key Insights

1. **The broadcast channel `Lagged` error is underutilized**: NeoTrix catches it in 3 locations (EventBus sync, EventBus async, background loop) but only logs warnings. In a consciousness architecture, lag is a first-class health signal — it should feed into the HeartbeatAggregator and ConsciousnessTree coherence metrics.

2. **Unbounded channels are a systemic risk**: 3 locations use `unbounded_channel` (ProxyKernel errors, hotreload, plugin registry). These are all "convenience" choices that bypass backpressure. The rustycloud.org source makes a compelling case that unbounded channels should be treated as footguns requiring explicit justification.

3. **The ElementBus `try_send` silent drop is the most dangerous defect**: The consciousness element bus carries state-change events between cognitive elements. Silent message loss means elements can permanently desynchronize — one element thinks a capability was updated while another never received the notification. This is exactly the class of bug that `FlowPolicy::Shed` with counters is designed to catch.

4. **Watch channels for shutdown are correct but incomplete**: The `ShutdownCoordinator` pattern is idiomatic but needs `JoinSet::abort_all()` as a fallback. The watch channel only works for tasks that are actively polling `changed()` — tasks that have already terminated or are blocked elsewhere will never see the signal.

5. **std::sync::mpsc inside async code is a latent deadlock**: The factory and behavioral verifier use blocking channels. If these are called from within a tokio runtime context (which they are — the factory explicitly checks `Handle::try_current()`), the blocking recv can starve the tokio worker thread, especially under load when multiple gateway initializations compete for threads.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Sources consulted | 15 |
| Files examined in NeoTrix | 12 |
| Channel creation sites found | 33 |
| Unbounded channel sites | 3 |
| Silent try_send drops | 1 |
| Blocking-in-async sites | 2 |
