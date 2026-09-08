# Agent 4: Structured Concurrency (Batch 873)

## Sources

1. **Tokio JoinSet docs** — `docs.rs/tokio/latest/tokio/task/struct.JoinSet.html` — Drop aborts all tasks; return values accumulate if `join_next` not called; same return type required.
2. **Tokio TaskTracker docs** — `docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html` — JoinSet retains return values causing OOM under fire-and-forget; TaskTracker drops completed task memory immediately; TaskTracker does not abort on drop.
3. **Rust Async Book: Structured Concurrency** — `rust-lang.github.io/async-book/part-reference/structured.html` — Select/race macros are unstructured cancellation sources; dropping JoinHandle releases task without handling result; spawned tasks cannot propagate cancellation to children.
4. **Oxide RFD 400: Cancel Safety** — `rfd.shared.oxide.computer/rfd/0400` — Task aborts should be avoided in normal control flow; Tokio mutex + cancellation = invalid state; cancel-unsafe futures in select loops must be recreated outside the loop.
5. **Rust FAQ: Structured Concurrency in Rust** — `www.rustfaq.org` — Detached tasks are a leak waiting to happen; JoinSet is workhorse for dynamic task counts; `scope` auto-waits on drop.
6. **Tokio GitHub: JoinSet Discussion #5910** — `github.com/tokio-rs/tokio/discussions/5910` — JoinSet retains return values indefinitely; under continuous spawning without `join_next`, memory grows unbounded.
7. **Tokio GitHub: Structured Concurrency Issue #1879** — `github.com/tokio-rs/tokio/issues/1879` — Forceful drop of WaitSet cancels all children; async destructors (async drop) still not supported; graceful cancellation needs explicit token.
8. **Medium: Structured Concurrency in Rust with Tokio** — `medium.com/@adamszpilewicz` — Select! + CancellationToken for coordinated shutdown; `shutdown()` ensures no zombie futures.
9. **Async Rust with Tokio Part 4** — `chandanbhagat.com.np` — Structured concurrency: every spawned task awaited before JoinSet dropped; fire-and-forget loses lifecycle control.
10. **Tokio Graceful Shutdown docs** — `tokio.rs/tokio/topics/shutdown` — TaskTracker + CancellationToken is the production pattern for graceful shutdown.

## Defects

**D-SCON-001: Zero JoinSet usage — entire background loop uses raw Vec\<JoinHandle\> with manual lifecycle** | `nt_mind_background_loop/mod.rs:96` | HIGH | Sources 1,3,5,9

`BackgroundLoop.handles: Vec<JoinHandle<()>>` collects raw JoinHandles instead of using `JoinSet`. This means: (a) no structured lifecycle — dropping `BackgroundLoop` does not abort orphaned tasks; (b) return values (errors/panics) are never drained, causing silent error accumulation; (c) no per-task abort handles for selective cancellation; (d) the `spawn()` method in `handlers.rs:11` pushes handles that may outlive the parent scope. The entire `spawn_handler!` macro system is a hand-rolled JoinSet that lacks abort safety and cooperative shutdown guarantees.

**D-SCON-002: Fire-and-forget spawning in accept loops — unbounded task leak** | `nt_shield_proxy_kernel/kernel.rs:220,256`, `nt_shield_traffic/mitm.rs:123`, `nt_shield_proxy_kernel/listener/socks5.rs:45`, `nt_shield_proxy_kernel/listener/http.rs:46` | HIGH | Sources 3,5,6,9

Multiple accept loops (`mitm.rs:123`, `socks5.rs:45`, `http.rs:46`, `kernel.rs:220`) spawn tasks via `tokio::spawn(async move { ... })` without any containment. These tasks are fire-and-forget: their `JoinHandle` is immediately dropped, meaning (a) errors/panics are silently swallowed; (b) under load, unbounded tasks can exhaust memory; (c) there is no graceful shutdown — a cancelled parent leaves orphaned connection handlers running indefinitely. Per Rust FAQ: "Detached tasks are a leak waiting to happen."

**D-SCON-003: ParallelExecutor spawns tasks in sequential loop with no concurrency guarantee** | `nt_core_parallel/executor.rs:34-44` | MEDIUM | Sources 1,6,9

`ParallelExecutor::execute()` in `ExecMode::Parallel` spawns tasks in a sequential `for` loop and immediately awaits each `handle` before spawning the next. This eliminates all concurrency — tasks execute sequentially despite being "parallel". The `JoinHandle` is awaited inline, so the pattern is structured but functionally broken. A `JoinSet` with `join_next()` loop would enable true concurrent execution and proper error aggregation.

**D-SCON-004: JoinSet absent from batch operations — download chunks use Vec\<JoinHandle\> with no abort** | `nt_io_download/engine.rs:127-156` | MEDIUM | Sources 1,3,5

The download engine collects `Vec<JoinHandle>` for chunk downloads, awaits each sequentially. If one chunk fails or the caller cancels, remaining chunks have no abort mechanism — their JoinHandles are dropped but tasks continue running to completion, wasting bandwidth. Should use `JoinSet` with `abort_all()` on first failure or external cancellation.

**D-SCON-005: `futures::future::join_all` used instead of JoinSet — loses abort control** | `nt_shield_stealth_net/proxy_pool.rs:624` | MEDIUM | Sources 1,6,9

`proxy_pool.rs` uses `futures::future::join_all(handles)` to await health checks. Unlike `JoinSet`, `join_all` does not support aborting individual or all tasks on timeout or failure. If a health check hangs (even with the timeout), the remaining checks cannot be cancelled. Should use `JoinSet` with `shutdown()` for coordinated abort.

**D-SCON-006: No CancellationToken in any codebase file — cooperative shutdown impossible** | Global (0 matches for `CancellationToken`) | HIGH | Sources 3,7,8,10

Zero occurrences of `CancellationToken` in the entire codebase. The background loop uses `watch::channel<bool>` for shutdown signaling, but spawned tasks outside this loop (accept loops, health checks, streaming helpers, etc.) have no cancellation mechanism. When the process shuts down, all orphaned tasks run until process exit or until they hit an error. The Tokio docs recommend `TaskTracker + CancellationToken` as the production pattern for graceful shutdown.

**D-SCON-007: `select!` branches with `JoinSet::join_next` not cancel-safe — shared handler lock held across select** | `nt_mind_background_loop/run.rs:742` | MEDIUM | Sources 3,4

The `spawn_handler!` macro uses `tokio::select! { biased; _ = ticker.tick() => { let mut lock = h.lock().await; ... } }`. If a handler tick holds the `Arc<RwLock<...>>` lock while awaiting a slow operation inside the handler body, and the `select!` fires the shutdown branch, the task is cancelled mid-lock. The `RwLock` guard is dropped but the state may be in an inconsistent state. Oxide RFD 400 documents this as a primary source of async cancel-safety bugs: "if a future holding a mutex is cancelled, the state guarded by the mutex is likely invalid."

**D-SCON-008: Shutdown deadline shared across all handlers — first handler consumes full 5s budget** | `nt_mind_background_loop/handlers.rs:48-61` | MEDIUM | Sources 3,5

The `shutdown()` method creates a single `tokio::time::sleep(Duration::from_secs(5))` deadline and iterates through all handles. The `tokio::select! { biased; _ = &mut deadline => { abort } }` shares the same pinned deadline across all handlers. If the first handler takes 5 seconds, all remaining handlers are immediately aborted without any grace period. Should use per-handler deadlines or a `JoinSet::shutdown()` which gives each task its own abort window.

**D-SCON-009: ParallelExecutor silently drops task errors** | `nt_core_parallel/executor.rs:41-43` | LOW | Sources 1,3

`handle.await` result is consumed with `if let Ok(res) = handle.await { results.push(res); }` — any `JoinError` (panic or abort) is silently discarded. In a consciousness architecture, a panicked parallel reasoning task should propagate upward. The error is lost and the result vector has fewer entries with no indication of failure.

**D-SCON-010: `spawn_blocking` used for network I/O — can exhaust blocking thread pool** | `nt_io_provider/free_catalog.rs:89`, `nt_io_provider/gateway/selection.rs:365`, `nt_io_provider/factory.rs:1295` | MEDIUM | Sources 1,6

Multiple files use `tokio::task::spawn_blocking` for `reqwest::blocking::Client` network calls. The blocking thread pool defaults to 512 threads. Under load (e.g., `free_catalog.rs` spawning up to 8 blocking network probes concurrently via semaphore, but without JoinSet), a burst of catalog refreshes can exhaust the pool, starving other blocking operations (KB maintenance, file I/O). Should use async reqwest instead, or use JoinSet to bound concurrent blocking tasks with proper abort semantics.

## Key Insights

1. **NeoTrix has zero structured concurrency primitives.** No `JoinSet`, no `TaskTracker`, no `CancellationToken`. Every spawned task is either fire-and-forget or collected into a raw `Vec<JoinHandle>` with manual lifecycle management. This is the single largest category of defect found across all agents.

2. **The background loop is a hand-rolled task manager.** The `spawn_handler!` macro in `run.rs` implements a custom task lifecycle with `watch::channel` for shutdown signaling. While functional, it lacks JoinSet's abort-on-drop guarantee, cooperative cancellation, and return-value handling. The 5-second shared deadline is particularly fragile.

3. **Accept loops are unbounded.** Every network-facing service (SOCKS5, HTTP proxy, MITM interceptor, DNS) spawns tasks in an infinite `loop { tokio::spawn(...) }` with no backpressure, no JoinSet, and no graceful shutdown. Under connection storms, this exhausts memory with no recovery path.

4. **Cancellation is structurally impossible.** Without `CancellationToken`, tasks cannot cooperatively cancel each other. The `watch::channel<bool>` in the background loop is a crude approximation — it requires each handler to explicitly check `rx.changed()`, which many spawned tasks (accept loops, streaming helpers, health checks) do not.

5. **The `select!` + RwLock pattern is a ticking bomb.** Holding an async RwLock guard across a `select!` branch that can be cancelled means the lock may be dropped in an inconsistent state. This is documented as a primary async cancel-safety bug class by Oxide (RFD 400).

6. **Memory growth under continuous spawning is unbounded.** `JoinSet` retains return values until `join_next` is called. NeoTrix's `Vec<JoinHandle>` pattern is even worse — `JoinHandle` values are never consumed, so their return values accumulate indefinitely. The download engine and batch processor are at risk of OOM under sustained load.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| HIGH severity | 3 |
| MEDIUM severity | 6 |
| LOW severity | 1 |
| Unique source files affected | 15+ |
| Categories: Lifecycle | 4 (D-SCON-001,004,008,010) |
| Categories: Fire-and-forget | 2 (D-SCON-002,009) |
| Categories: Missing primitives | 2 (D-SCON-006,007) |
| Categories: Wrong abstraction | 2 (D-SCON-003,005) |
