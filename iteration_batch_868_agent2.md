# Agent 2: Async Cancellation Patterns (Batch 868)

## Sources
1. Medium — "Async & Observability: Cancellation Patterns That Actually Work" (2026-02-20)
2. Tokio docs — Graceful Shutdown (tokio.rs/tokio/topics/shutdown)
3. docs.rs — CancellationToken (tokio-util 0.7)
4. docs.rs — JoinSet (tokio 1.x)
5. GitHub tokio-rs/tokio — Discussion #1819: Graceful shutdown of tokio-based server
6. lib.rs — tokio-graceful-shutdown crate
7. rust-skills/SKILL.md — async-cancellation-token, async-structured-concurrency rules
8. Microsoft RustTraining — ch13 Production Patterns (JoinSet, TaskTracker, watch channels)
9. rustz2h.com — Chapter 7: Cancellation, Timeouts, and Graceful Shutdown series
10. GitHub codeandsolder/rust-web-fullstack — Pattern 15: CancellationToken + JoinSet + biased select

## Defects

**D-CANCEL-001**: `std::process::exit(0)` in signal handler bypasses all task cleanup | `system_proxy.rs:128` | HIGH | Sources: 1,2,8,10
> After restoring proxy settings on SIGTERM/SIGINT, the handler calls `std::process::exit(0)` which immediately kills the process. All other tokio tasks (background handlers, EventBus consumers, proxy connections) are terminated without any grace period. Per rust-skills rule `async-cancellation-token`, the correct pattern is to fire a CancellationToken and let the runtime drain. The self-review PA009 check already flags this pattern but the instance persists.

**D-CANCEL-002**: Shared deadline in shutdown loop steals time from later handlers | `handlers.rs:48-61` | MEDIUM | Sources: 1,6,7,10
> The shutdown loop iterates `self.handles.drain(..)` with a single `tokio::pin!(deadline)` at 5 seconds. If handler N takes 4.9s to exit, handler N+1 receives only 0.1s before being aborted. Each handler should get its own independent grace window, or the total deadline should be per-handler. As-is, the first slow handler cannibalizes the budget for all subsequent ones.

**D-CANCEL-003**: Proxy listener `select!` lacks `biased;` — shutdown signal may lose race | `socks5.rs:40`, `http.rs:41` | MEDIUM | Sources: 2,7,9,10
> Both SOCKS5 and HTTP proxy listener loops use `tokio::select!` without `biased;`. When a new connection arrives at the same instant as the shutdown signal, tokio picks randomly. This means a new connection may be accepted *after* shutdown was requested, spawning a task that will never be tracked or drained. Per rust-skills `async-structured-concurrency`, the shutdown branch must be checked first via `biased;` to guarantee no new work is accepted after shutdown.

**D-CANCEL-004**: EventBus subscriber tasks have no shutdown mechanism — fire-and-forget leak | `nt_core_event_bus.rs:362` | HIGH | Sources: 1,6,8
> `subscribe_layer()` spawns a `tokio::spawn(async move { loop { rx.recv().await ... } })` that runs forever. The returned `JoinHandle` is ignored by callers (no handle stored, no abort). These tasks have no cancellation token or shutdown signal. On process exit they're killed abruptly; if the runtime is reused or the subscriber is re-registered, orphan tasks accumulate. Each subscriber should accept a CancellationToken or the handle should be tracked in a JoinSet.

**D-CANCEL-005**: Proxy kernel drain loop polls counter but never waits for connection close | `kernel.rs:279-291` | MEDIUM | Sources: 2,6,8
> After sending shutdown, the drain loop checks `active_connections` every 100ms and breaks when zero or timed out. But nothing signals the *active* connections to close — the `run_bidirectional` tasks observe `shutdown_rx.changed()` and drop their streams, but the counter decrement happens asynchronously. The loop may see `conns > 0` even after all connections are logically closed, or see `conns == 0` while a close is still in flight. The drain should await a JoinSet/TaskTracker of connection tasks rather than polling an atomic counter.

**D-CANCEL-006**: MITM listener has no shutdown signal — infinite accept loop | `mitm.rs:116-137` | MEDIUM | Sources: 1,2,8
> `start_mitm()` runs `loop { listener.accept().await ... tokio::spawn(...) }` with no shutdown check. When the proxy kernel shuts down, this listener keeps accepting connections indefinitely. There is no CancellationToken, no watch channel, and no break condition. Spawned handler tasks are also fire-and-forget with no tracking. This creates a resource leak on shutdown.

**D-CANCEL-007**: `std::process::exit(1)` on runtime creation failure leaks tokio runtime | `factory.rs:1442,1457` | LOW | Sources: 1,8
> In `create_gateway_v2_in_new_runtime`, if `Runtime::new()` fails, `std::process::exit(1)` is called. While this is an edge case (runtime creation rarely fails), the hard exit bypasses any cleanup of previously initialized state. The function already has a pattern of `std::mem::forget(rt)` to avoid drop issues, suggesting awareness of runtime lifecycle — but the error path doesn't follow the same care.

**D-CANCEL-008**: Background handler spawn macro lacks cancellation-safety annotation | `run.rs:734-760` | LOW | Sources: 1,7,9
> The `spawn_handler!` macro creates tasks with `tokio::select! { biased; _ = ticker.tick() => { ... } _ = rx.changed() => { break; } }`. While `biased;` is correctly used, the handler body acquires `h.lock().await` *inside* the select branch. If the handler panics while holding the lock, the lock is held until the MutexGuard is dropped by the runtime. The macro doesn't enforce that handler bodies are cancellation-safe or that lock duration is bounded. A slow handler tick can delay the shutdown observation on the next loop iteration.

## Key Insights
- NeoTrix uses `watch::channel` (not `CancellationToken`) for shutdown coordination across the proxy kernel and background loop. This is functional but misses the hierarchical parent/child token semantics of `CancellationToken` — child tasks cannot independently trigger shutdown of their subtree.
- The codebase has 64 `tokio::spawn` calls but only the background loop handlers (9 tasks) are tracked in a `Vec<JoinHandle>`. All other spawned tasks are fire-and-forget, creating a systematic orphan-task leak across the proxy, EventBus, download, and traffic modules.
- The `biased;` keyword is correctly used in the background loop handler macro but absent from all proxy kernel listeners (SOCKS5, HTTP, DNS intercept), creating a shutdown race in the network layer.
- The `process::exit` pattern appears in 3 production paths (system_proxy, factory, nt-lang main) despite PA009 self-review already flagging it. This is a recurring anti-pattern that should be gated at the architecture level.
- No module in the codebase uses `JoinSet` or `TaskTracker` from tokio-util. The structured concurrency pattern recommended by tokio docs and rust-skills is entirely absent — all task management is ad-hoc via `Vec<JoinHandle>` or fire-and-forget spawns.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| HIGH severity | 2 |
| MEDIUM severity | 4 |
| LOW severity | 2 |
| Sources consulted | 10 |
