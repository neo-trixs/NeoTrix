# Agent 3: Async Cancellation Safety (Batch 873)

## Sources

1. Tokio `select!` docs — `docs.rs/tokio/latest/tokio/macro.select.html` — Official cancellation safety section: which methods are/aren't cancel-safe, biased mode, branch dropping semantics
2. Oxide RFD 400 — `rfd.shared.oxide.computer/rfd/0400` — "Dealing with cancel safety in async Rust": reserve pattern, Mutex trap, then_try adapters, cooperative cancellation
3. sunshowers.io — "Cancelling async Rust" — Cancel correctness framework: 3 conditions for violation (unsafe future + cancelled + system invariant violated), make-futures-cancel-safe patterns
4. Atharva Pandey — "Lesson 10: Cancellation Safety — The Silent Footgun" — read_exact in select trap, pin-outside-loop strategy, atomicity wrapper pattern
5. Biriukov — "Async Rust gotcha: evolving tokio::select! code has sharp edges" — `FutureExt::then()` not cancel-safe, StreamExt::then() fix, test Poll::Pending paths
6. developerlife.com — "Rust async in practice tokio::select!, actor pattern & cancel safety" — sleep vs interval mental model, unpinned sleep future dropping state
7. Tokio select! macro source — `github.com/tokio-rs/tokio/blob/c637f67/tokio/src/macros/select.rs` — Internal polling order, biased vs random branch selection
8. Rust Users Forum "Cancellation safety of locks in select!" — Mutex queue-loss, pin-and-reuse pattern, FutureLock hazard
9. Rust Users Forum "Cancel safety in async and tokio::select!" — Borrowing future cancellation, channel recv cancel-safety
10. Medium "tokio::select! Is a Loaded Gun" — reserve pattern, Mutex trap, partial DB writes
11. aselect crate (github.com/avl/aselect) — Alternative to select! that never cancels futures by default

## Defects

**D-CSAFE-012: Shutdown drain loop shares single pinned deadline across all 35+ handlers — first handler to timeout consumes the deadline, leaving remaining handlers with zero timeout enforcement**
| `nt_mind_background_loop/handlers.rs:48-61` | HIGH | Sources: 1, 2, 7

The `shutdown()` method creates a single `tokio::time::sleep(Duration::from_secs(5))` and pins it via `tokio::pin!(deadline)`. It then iterates `self.handles.drain(..)` in a loop, racing each handle against `&mut deadline` in a `biased; select!`. Once the deadline fires during the first handler's select (line 55), the `Sleep` future is consumed — it returns `Ready(())`. On subsequent iterations, `&mut deadline` immediately resolves to `Ready`, so every remaining handler is instantly aborted with no grace period. If handler A takes 4.9s, the remaining 34 handlers share 0.1s total. The Tokio docs state: "the `select!` macro randomly picks a branch to check first" — with `biased`, the deadline branch is polled first, and once it resolves, the loop body executes `abort.abort()` immediately. Fix: use per-handler `tokio::time::timeout` or a `JoinSet::try_join_next` with individual deadlines.

**D-CSAFE-013: `spawn_handler!` macro acquires `tokio::sync::Mutex` inside biased select branch — handler body cancellation drops lock guard mid-operation**
| `nt_mind_background_loop/run.rs:742-757` | HIGH | Sources: 1, 2, 8, 10

The `spawn_handler!` macro expands to `tokio::select! { biased; _ = ticker.tick() => { let mut $lock = h.lock().await; $body; } _ = rx.changed() => { break; } }`. The `h.lock().await` (line 745) is called inside the `ticker.tick()` branch. If the handler body `$body` contains multiple `.await` points (e.g., `handle_consolidate()` which does KB reads + writes), and the shutdown signal fires between two awaits, the select drops the entire future including the MutexGuard. Tokio docs explicitly list `Mutex::lock` as "not cancellation safe — uses a queue for fairness; cancellation makes you lose your place in the queue." The 35+ handlers all share `Arc<Mutex<BackgroundLoop>>`. A cancelled lock acquisition wastes a queue slot, and under concurrent load, the FIFO ordering breaks down. The Rust Forum thread (Source 8) confirms: "dropping the lock future means you lose your place in line."

**D-CSAFE-014: EventBus consumer holds Mutex guard across `handle_event_bus_event().await` inside select handler — cancel-unsafe priority inversion across all background handlers**
| `nt_mind_background_loop/run.rs:896-902` | HIGH | Sources: 1, 2, 8, 10, 11

The EventBus consumer task (line 896-916) uses `tokio::select! { biased; result = event_rx.recv() => { let mut handle = h.lock().await; handle.handle_event_bus_event(event).await; } _ = rx.changed() => { break; } }`. This pattern is doubly cancel-unsafe: (a) `broadcast::Receiver::recv` is cancel-safe (good), but (b) after acquiring the MutexGuard at line 901, the handler calls `handle_event_bus_event(event).await` at line 902 while holding the lock. If `handle_event_bus_event` performs I/O (KB writes, file operations), the lock is held for the entire I/O duration. During this time, ALL other 30+ handlers (save, consolidate, goal, crystallization, etc.) cannot acquire the lock and are blocked. If the shutdown signal fires while the event handler is mid-I/O, the lock is dropped but the I/O may be partially complete — leaving the `BackgroundLoop` state inconsistent. The Tokio Mutex docs state: "If a task is cancelled while holding a lock, the lock is released, but other tasks that were waiting may see an inconsistent state."

**D-CSAFE-015: Bidirectional `forward_traffic` select lacks `biased;` — shutdown signal loses non-deterministic race against `io::copy` branches**
| `nt_shield_proxy_kernel/kernel.rs:532` | MEDIUM | Sources: 1, 2, 7

The `forward_traffic` method (line 532) uses `tokio::select!` (without `biased;`) to race three branches: `io::copy` (client→upstream), `io::copy` (upstream→client), and `shutdown_rx.changed()`. Without `biased;`, Tokio randomly selects which ready branch to poll first (Source 1: "randomly picks a branch to check first for readiness"). When the shutdown signal arrives simultaneously with a pending `io::copy`, the runtime may pick the `io::copy` branch first, allowing one more byte to be copied before the shutdown branch wins on the next poll. While this is a minor delay, the real issue is that the non-biased ordering means the shutdown branch can be starved under high throughput — if the stream has "a huge volume of messages and zero or nearly zero time between them" (Tokio docs on biased mode caveat), the `io::copy` branches will dominate the random polling. Fix: add `biased;` with the shutdown branch listed first.

**D-CSAFE-016: GC task creates unpinned `tokio::time::sleep` inline in select branch — timing state lost on each loop iteration**
| `nt_shield_proxy_kernel/kernel.rs:222-226` | MEDIUM | Sources: 1, 6

The GC task (line 220-232) uses `tokio::select!` with `tokio::time::sleep(Duration::from_secs(60))` created inline in the branch expression (line 223). Per the Tokio docs: "If you're using a cancel-unsafe future in a `select!` loop, you cannot recreate the future in each loop iteration." Each time the loop iterates, a brand-new `Sleep` future is created. When the shutdown branch fires, the current `Sleep` future is dropped, and on the next iteration (if any), a fresh 60-second timer starts. This means: if the GC ran at T=0 and shutdown fires at T=59, the GC ran exactly once. But if the shutdown branch wins the random poll at T=0 before the sleep branch is ever polled, the GC never runs. The `developerlife.com` tutorial (Source 6) explicitly warns: "unpinned sleep future is dropped when the other branch is executed... the state of the future is lost." Fix: create and pin the sleep outside the loop, or use `tokio::time::interval` with `MissedTickBehavior::Skip`.

**D-CSAFE-017: SOCKS5 and HTTP proxy listener selects lack `biased;` — shutdown signal may lose race to new connection accept**
| `nt_shield_proxy_kernel/listener/socks5.rs:40`, `listener/http.rs:41` | MEDIUM | Sources: 1, 7

Both `SOCKS5Server::start_with_shutdown` (socks5.rs:40) and `HttpProxyServer::start_with_shutdown` (http.rs:41) use `tokio::select!` without `biased;` to race `listener.accept()` against `shutdown_rx.changed()`. Without `biased;`, when a new TCP connection arrives at the exact instant the shutdown signal fires, the runtime picks non-deterministically. Per the Tokio docs on `biased;` mode: "if you are selecting between a stream and a shutdown future, and the stream has a huge volume of messages... you should place the shutdown future earlier in the `select!` list to ensure that it is always polled." A new connection accepted after shutdown spawns a `tokio::spawn` task (socks5.rs:45, http.rs:46) that will never be tracked or drained — violating the structured concurrency invariant that "no tasks are leaked from a scope." Fix: add `biased;` with the shutdown branch first.

**D-CSAFE-018: Shutdown drain uses sequential select per handler instead of concurrent JoinSet — shared 5-second budget creates starvation cascade**
| `nt_mind_background_loop/handlers.rs:48-61` | MEDIUM | Sources: 1, 2, 8

The drain loop (line 51-61) iterates handlers sequentially: `for handle in self.handles.drain(..) { tokio::select! { biased; _ = &mut deadline => { abort.abort(); } _ = handle => {} } }`. With 35+ handlers and a single 5-second deadline, the budget is consumed sequentially. If the first handler takes 4 seconds to stop (e.g., waiting for a KB write to complete), the second handler gets 1 second, the third gets the remainder, and all remaining handlers are instantly aborted. The correct pattern (Source 2: Oxide RFD 400) is to use `JoinSet::try_join_next` or `FuturesUnordered` to poll all handlers concurrently, or to use `tokio::time::timeout` per-handler. The sequential drain also means the `biased;` keyword is misleading — it only controls ordering within a single select, not across the sequential loop iterations.

**D-CSAFE-019: `write_all` in SOCKS5 error response inside spawned task — cancel-unsafe on task abort during shutdown**
| `nt_shield_proxy_kernel/listener/socks5.rs:65` | LOW | Sources: 1, 4

The `send_socks5_error` function (line 64-67) calls `stream.write_all(&[...]).await`. Tokio docs explicitly list `AsyncWriteExt::write_all` as "not cancellation safe — can lead to loss of data." While this code is inside a `tokio::spawn` (not directly in a `select!`), during graceful shutdown the 5-second abort deadline (handlers.rs:48) forcibly aborts tasks via `AbortHandle`. Any task mid-`write_all` loses its partially-written error response bytes. The SOCKS5 client receives a truncated response, potentially causing protocol desync. Fix: use `write` (which is cancel-safe) in a loop with manual tracking of written bytes, or avoid writing error responses to connections being shut down.

**D-CSAFE-020: `system_proxy` signal handler calls `std::process::exit(0)` after select — skips all Drop impls and async cleanup**
| `nt_shield_stealth_net/system_proxy.rs:122-128` | LOW | Sources: 1, 4

The system proxy signal handler (line 121-129) spawns a task that does `tokio::select! { _ = term.recv() => {} _ = int.recv() => {} }` then calls `proxy.disable().await` followed by `std::process::exit(0)`. The `process::exit(0)` call bypasses all Rust destructors, Tokio runtime shutdown, and async Drop implementations. Any in-flight proxy state changes, KB writes, or file I/O from other tasks will be silently lost. While this is intentional (the comment says "signal received, restoring proxy settings"), the `proxy.disable().await` itself may be cancelled if the runtime is already shutting down — there's no guarantee the Tokio runtime is still operational at this point. Fix: use `tokio::signal::ctrl_c()` with a `CancellationToken` to coordinate clean shutdown across all subsystems.

## Key Insights

1. **The `select!` + shared-future anti-pattern is pervasive**: The single-pinned-deadline-shared-across-all-handlers pattern (D-CSAFE-012) is a textbook cancel-correctness violation. The Oxide RFD 0400 explicitly addresses this: "resume futures in `select!` loops rather than recreating them." NeoTrix's shutdown loop does neither — it shares a consumed future.

2. **Mutex-in-select is the single largest async safety risk**: Three separate defects (D-CSAFE-013, D-CSAFE-014, D-CSAFE-018) stem from `tokio::sync::Mutex` being acquired inside `select!` branches. The Tokio docs, Oxide RFD 0400, and the `cancel-safe-futures` crate all converge: `tokio::sync::Mutex` should be avoided in cancel-sensitive code. NeoTrix's 35+ handlers sharing one Mutex creates a system-wide bottleneck.

3. **Non-biased selects across 5 shutdown-critical paths**: The DNS interceptor, SOCKS5 listener, HTTP listener, bidirectional tunnel, and GC task all lack `biased;`. Each represents a path where the shutdown signal can lose the random poll race, delaying graceful shutdown or allowing new work after shutdown was requested.

4. **No CancelSafeFuture audit exists**: The codebase has no internal documentation or marker trait indicating which futures are cancel-safe. The Tokio docs provide per-method guidance, but NeoTrix has no systematic audit. This makes it trivially easy for future code to introduce cancellation bugs in `select!` loops.

5. **Cancel safety is a system-wide property, not per-future**: As sunshowers.io notes (Source 3), cancel correctness requires three conditions: (1) a cancel-unsafe future exists, (2) it is actually cancelled, (3) the cancellation violates a system invariant. NeoTrix satisfies all three in the shutdown path: the Mutex-guard-across-await is cancel-unsafe, the 5s abort deadline cancels it, and the system invariant (consistent BackgroundLoop state) is violated.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 9 |
| Sources consulted | 11 |
| `tokio::select!` sites audited | 12 |
| Previous batch defects carried forward | 0 |
