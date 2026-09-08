# Agent 4: TaskTracker Patterns (Batch 872)

## Sources

1. **tokio::task::JoinSet** — `docs.rs/tokio/latest/tokio/task/struct.JoinSet.html` — Official Tokio docs: JoinSet aborts all tasks on drop, `shutdown()` method, `abort_all()`, `detach_all()`.
2. **tokio_util::task::TaskTracker** — `docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTracker.html` — Official docs: close + wait pattern, cloning support, memory-safe cleanup (tasks freed immediately on exit).
3. **Tokio Graceful Shutdown tutorial** — `tokio.rs/tokio/topics/shutdown` — CancellationToken + TaskTracker pattern, `biased;` select for shutdown-wins-ties.
4. **Microsoft RustTraining Ch.13** — `github.com/microsoft/RustTraining` — Structured concurrency with JoinSet/TaskTracker, `watch` channels for coordinated shutdown.
5. **tokio-graceful-shutdown crate** — `github.com/Finomnis/tokio-graceful-shutdown` — Subsystem nesting, partial shutdown, timeout-limited graceful exit.
6. **tokio-graceful crate** — `github.com/plabayo/tokio-graceful` — Guard-based reference counting for task lifecycle tracking.
7. **Rust Forum discussion** — `users.rust-lang.org` — JoinSet sharing is incorrect; CancellationToken + TaskTracker is the correct pattern for shared task sets.
8. **Tokio Discussion #1819** — `github.com/tokio-rs/tokio/discussions/1819` — TaskTracker as "a JoinSet that lets you wait for all tasks to complete during shutdown."
9. **rust-tokio-task-tracker crate** — `github.com/jwalton/rust-tokio-task-tracker` — TaskSpawner + TaskTracker + TaskWaiter pattern for cooperative cancellation.

## Defects

### D-TT-001: EventBus sync subscribers use spin-wait `std::thread::sleep(10ms)` polling loop — CPU burn under load
**File**: `neotrix-core/src/neotrix/nt_core_event_bus.rs:440-464`
**Severity**: Medium
**Source**: Microsoft RustTraining Ch.13, tokio docs
**Detail**: The sync EventBus subscribers (`subscribe_all_layers_sync`) use a tight `try_recv()` + `std::thread::sleep(10ms)` spin-poll loop on 9 threads. Under high event throughput, this wastes CPU cycles. The async variant (`subscribe_layer`) correctly uses `rx.recv().await` on a broadcast channel. The sync path should use `std::sync::mpsc::Receiver::recv()` (blocking) instead of `try_recv` + sleep, or migrate to `tokio::sync::watch` which naturally blocks.

### D-TT-002: `BackgroundLoop::spawn()` tasks are fire-and-forget — no graceful shutdown, no abort tracking
**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:6-11`
**Severity**: High
**Source**: tokio docs, tokio-graceful-shutdown
**Detail**: `BackgroundLoop::spawn()` pushes `JoinHandle` into `handles` Vec but the doc comment explicitly says "Such tasks will be aborted during shutdown without grace period." This means any ad-hoc task (e.g., absorption processing at `handlers_absorption.rs:181`) gets zero opportunity to flush state. The pattern should use `CancellationToken` + `TaskTracker` so spawned tasks can observe the shutdown signal and exit cleanly before the 5s deadline.

### D-TT-003: `BackgroundLoop::shutdown()` has a single shared deadline — slow task starves all others
**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:48-61`
**Severity**: High
**Source**: tokio-graceful-shutdown (per-subsystem timeout), Tokio docs
**Detail**: The `shutdown()` method creates a single `tokio::time::sleep(5s)` deadline and drains all handles sequentially. If the first handle blocks for 4.9s, the remaining handles share only 100ms. Each handler should get its own deadline. The correct pattern is: broadcast shutdown → collect all handles → `tokio::select!` each with its own timeout, or use `tokio::time::timeout` per-handle.

### D-TT-004: `EventBus::shutdown()` detaches remaining threads without abort — orphaned subscriber threads
**File**: `neotrix-core/src/neotrix/nt_core_event_bus.rs:228-243`
**Severity**: Medium
**Source**: tokio docs (`JoinSet::detach_all` docs: "tasks removed will continue to run in background")
**Detail**: When the 2s deadline expires, the loop `break`s and remaining threads in the `handles` Vec are dropped. Since these are `std::thread::JoinHandle`, dropping them detaches the threads — they continue running even after the EventBus is destroyed. The shutdown_flag is set, but there's a TOCTOU gap: the thread could be between the `shutdown.load()` check and the `try_recv()` call. Should `h.join()` the remaining threads unconditionally after deadline, or use a stronger signal (e.g., `thread::park_timeout`).

### D-TT-005: `subscribe_all_layers()` returns `Vec<JoinHandle>` but caller drops it — silent task abort
**File**: `neotrix-core/src/neotrix/nt_core_event_bus.rs:401-413`
**Severity**: Medium
**Source**: Tokio docs ("When JoinSet is dropped, all tasks are immediately aborted")
**Detail**: `subscribe_all_layers()` returns 9 `tokio::task::JoinHandle<()>` in a Vec, but at `run.rs:415` the result of `subscribe_all_layers_sync(&event_bus)` is `()`. For the async path, any caller dropping the returned Vec drops the JoinHandles, which aborts the tasks. This is a silent correctness bug — the async subscriber tasks vanish on drop. Should store handles in a struct with explicit lifecycle management, or use `TaskTracker` which survives drops.

### D-TT-006: `factory.rs` leaks two tokio Runtimes via `std::mem::forget(rt)` — unbounded resource leak
**File**: `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/factory.rs:1446-1447, 1461-1462`
**Severity**: High
**Source**: tokio docs, Tokio Discussion #1819
**Detail**: `create_gateway()` calls `std::mem::forget(rt)` on two code paths to avoid `BlockingPool::shutdown` deadlock. This leaks the entire Runtime including its blocking thread pool, I/O driver, and timer. Each call to `create_gateway()` from a new runtime context leaks another Runtime. The comment acknowledges this but provides no mitigation. Should use `Runtime::shutdown_timeout()` with a bounded wait, or restructure to avoid nested runtimes entirely (use `Handle::current()` and propagate the parent runtime).

### D-TT-007: `proxy_pool.rs` health check tasks use `futures::future::join_all` without cancellation safety
**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/proxy_pool.rs:613-631`
**Severity**: Low
**Source**: Tokio docs ("Tasks spawned on a JoinSet are aborted on drop; join_all has no abort mechanism")
**Detail**: Health checks are spawned into a local `Vec<JoinHandle>` and awaited with `futures::future::join_all`. If the calling task is aborted (e.g., during shutdown), these health check tasks become orphaned. Should use `JoinSet::abort_all()` or `TaskTracker` with a CancellationToken so health checks are properly cancelled during shutdown.

### D-TT-008: No `biased;` in `dns_intercept.rs` `tokio::select!` — shutdown signal may be delayed by one DNS query
**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/dns_intercept.rs:49-69`
**Severity**: Low
**Source**: Microsoft RustTraining Ch.13, tokio-rs/tokio discussion
**Detail**: The `tokio::select!` in the DNS interceptor loop does not use `biased;`. Without it, when a shutdown signal and a DNS query arrive simultaneously, the runtime picks non-deterministically — the shutdown branch may lose. The Tokio docs and production patterns recommend `biased;` so shutdown always wins ties. The `BackgroundLoop::shutdown()` at `handlers.rs:54` correctly uses `biased;`, but `dns_intercept.rs` does not.

## Key Insights

1. **No TaskTracker anywhere in the codebase**: NeoTrix uses raw `Vec<JoinHandle>` everywhere instead of `tokio_util::task::TaskTracker`. This misses the close/wait lifecycle, automatic memory cleanup, and clone-based sharing that TaskTracker provides.

2. **No CancellationToken usage**: Despite having `tokio::sync::watch` for shutdown signaling, no task uses `CancellationToken`. The shutdown pattern is ad-hoc: broadcast bool via watch → each task checks in its select. This misses hierarchical cancellation (parent/child tokens) and the `with_cancellation_token_owned` timeout pattern.

3. **Sync/async split creates lifecycle gaps**: The EventBus has both sync (std::thread) and async (tokio::spawn) subscriber paths, but neither has proper lifecycle management. The sync path spin-polls, the async path gets silently aborted on drop.

4. **Pattern mismatch**: Tokio's recommended pattern is `JoinSet` for same-type task groups + `TaskTracker` for mixed/sharable task sets + `CancellationToken` for shutdown signaling. NeoTrix uses none of these, instead reinventing lifecycle management with `Vec<JoinHandle>` + `AtomicBool` + `watch::channel`.

5. **Runtime leaking as a deliberate workaround**: The `factory.rs` runtime leak is documented but not mitigated. This means every process that calls `create_gateway()` from a tokio context leaks ~2 runtimes with their blocking pools. Under long-running scenarios, this accumulates orphaned threads.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Sources consulted | 9 |
| Critical (High) defects | 3 |
| Medium defects | 3 |
| Low defects | 2 |
