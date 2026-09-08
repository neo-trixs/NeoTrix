# Agent 3: Cancel-Safe Futures (Batch 869)

## Sources
1. Tokio docs — `select` macro cancellation safety: https://docs.rs/tokio/latest/tokio/macro.select.html
2. Oxide RFD 400 — Dealing with cancel safety in async Rust: https://rfd.shared.oxide.computer/rfd/0400
3. Tokio tutorial — select: https://tokio.rs/tokio/tutorial/select
4. `cancel-safe-futures` crate docs: https://docs.rs/cancel-safe-futures/latest/cancel_safe_futures/
5. Biruikov — Async Rust gotcha: evolving tokio::select! code has sharp edges (Feb 2026): https://biriukov.dev/posts/async-rust-gocha-tokio-cancelation-select-future-then/
6. Tokio — Graceful Shutdown: https://tokio.rs/tokio/topics/shutdown
7. `tokio-cancel-guard` crate docs: https://docs.rs/tokio-cancel-guard/latest/tokio_cancel_guard/
8. Medium — tokio::select! Is a Loaded Gun (Apr 2026): https://medium.com/@dhvani612/tokio-select-is-a-loaded-gun-understanding-cancel-safety-before-it-silently-destroys-your-data-ceb74688eee8
9. Barafael — Stop Worrying and Learn to Loop-Select (Feb 2025): https://barafael.github.io/posts/stop-worrying-and-learn-to-loop-select/
10. Sunshowers — Cancelling async Rust (Oct 2025): https://sunshowers.io/posts/cancelling-async-rust/
11. Comprehensive Rust — Cancellation: https://google.github.io/comprehensive-rust/concurrency/async-pitfalls/cancellation.html
12. Rust Forum — Cancellation safety of locks in `select!` (Dec 2025): https://users.rust-lang.org/t/cancellation-safety-of-locks-in-select/137188
13. `rules/async-cancel-safety.md`: https://github.com/leonardomso/rust-skills/blob/HEAD/rules/async-cancel-safety.md

## Defects

**D-CSAFE-001**: `biased;` in `spawn_handler!` macro starves shutdown signal — all 30+ background handlers use `biased;` with `ticker.tick()` first, meaning if a handler's ticker fires continuously the `rx.changed()` shutdown branch is never polled. Tokio docs explicitly warn: "It becomes your responsibility to ensure that polling order is fair." During graceful shutdown, handlers that run at high frequency (e.g., `always_on`, `scheduler`) may never observe the cancellation signal, causing the 5-second abort deadline to trigger and forcibly kill in-progress work. | `nt_mind_background_loop/run.rs:742-758` | high | Sources 1, 5, 9

**D-CSAFE-002**: Single `Arc<Mutex<BackgroundLoopHandle>>` serializes all 30+ independent handler tasks — despite being spawned as separate tokio tasks, every handler contends on the same `tokio::sync::Mutex` (`h`). `tokio::sync::Mutex::lock()` is not cancellation-safe (documented: "cancellation makes you lose your place in the queue"). When two handlers race for the lock, the loser's lock future is dropped and it must re-queue from scratch on the next loop iteration. This creates priority inversion: a slow KB-absorb handler blocks the emergency kb_guard handler from running. | `nt_mind_background_loop/run.rs:738-759` | high | Sources 1, 2, 12

**D-CSAFE-003**: Event bus consumer holds `Mutex` guard across async handler — the event_bus consumer task (line 896-915) acquires `h.lock().await` inside the `select!` handler block, then calls `handle_event_bus_event(event).await` while holding the guard. If `handle_event_bus_event` blocks on I/O (KB write, network), the mutex is held for the duration, starving all other 30+ handlers that need the same lock. This transforms the "independent task" architecture into effectively single-threaded execution. | `nt_mind_background_loop/run.rs:900-902` | high | Sources 2, 10, 12

**D-CSAFE-004**: Bidirectional `io::copy` select drops TCP half-close — in `proxy_connection`, when one direction's `tokio::io::copy` completes (client sends FIN), the other direction's copy future is cancelled via `select!`. While `io::copy` itself is cancel-safe, the TCP socket is dropped without calling `shutdown()` on the write half. The peer (e.g., upstream server) never receives a proper FIN, leaving the connection in a TIME_WAIT-like state. On shutdown, both directions are killed simultaneously via `shutdown_rx.changed()`, discarding any in-flight kernel write buffers. | `nt_shield_proxy_kernel/kernel.rs:531-552` | medium | Sources 1, 10

**D-CSAFE-005**: SOCKS5 handler uses `read_exact` (cancel-unsafe) in connection handler — `handle_socks5_connection` calls `stream.read_exact()` 6+ times (lines 76, 80, 85, 95, 97, 102, 104, 111, 113). `read_exact` is explicitly documented as NOT cancel-safe: "partially filled buffer is lost." While each connection is spawned as a separate task (not inside `select!`), during graceful shutdown the 5-second abort deadline (handlers.rs:48-61) forcibly aborts tasks via `AbortHandle`. Any task mid-`read_exact` loses its partially-read SOCKS5 request bytes, potentially causing protocol desync on the client side. | `nt_shield_proxy_kernel/listener/socks5.rs:76-120` | medium | Sources 1, 11, 13

**D-CSAFE-006**: `TrafficAnalyzer` behind `tokio::sync::Mutex` — `TrafficAnalyzer` is wrapped in `Arc<Mutex<TrafficAnalyzer>>` across 3 files (traffic/mod.rs:26, api_proxy.rs:41, mitm.rs:79). Any code that holds the analyzer guard across an `.await` point makes the mutex cancellation-unsafe. If the holding future is cancelled (task abort, timeout, select!), the mutex guard is dropped, but the analyzer's internal state (captured request buffers, session tracking) may be left in an inconsistent state — e.g., a request was partially recorded but the response handler never ran. | `nt_shield_traffic/mod.rs:26`, `api_proxy.rs:41`, `mitm.rs:79` | medium | Sources 2, 8, 10

**D-CSAFE-007**: No cooperative cancellation tokens anywhere in NT-SHIELD — the proxy kernel, SOCKS5 listener, HTTP listener, DNS intercept, and system proxy all use `watch::Receiver<bool>` for shutdown. Unlike `CancellationToken` (which supports fan-out and has drop semantics), `watch::Receiver` requires polling `changed()` to observe shutdown. Combined with D-CSAFE-001 (biased starves the shutdown branch), this means NT-SHIELD components can't cooperatively cancel in-progress I/O operations. The `cancel-safe-futures` crate's `coop_cancel` module is the recommended pattern for this, but is not used. | `nt_shield_proxy_kernel/kernel.rs:219`, `listener/socks5.rs:33`, `listener/http.rs:34`, `dns_intercept.rs:41` | medium | Sources 2, 4, 6

**D-CSAFE-008**: `write_all` in SOCKS5 handler is cancel-unsafe under task abort — `send_socks5_error` (socks5.rs:65) and multiple SOCKS5 response writes (lines 81, 97, 126, 145, 156, 165, 178) use `AsyncWriteExt::write_all`. Tokio docs explicitly list `write_all` as NOT cancel-safe: "if the future is dropped before completion, you have no idea how much of this buffer was written out." During shutdown, task abort can leave a partial SOCKS5 response on the wire, causing the client to read garbage bytes and desync from the protocol state machine. | `nt_shield_proxy_kernel/listener/socks5.rs:65-178` | medium | Sources 1, 5, 13

## Key Insights

1. **The single-Mutex architecture defeats concurrency**: The 30+ background handlers in `run.rs` are spawned as independent tasks but all contend on one `Arc<Mutex<BackgroundLoopHandle>>`. This is the classic Tokio anti-pattern: `tokio::sync::Mutex` held across `.await` in a select loop creates serialized execution with cancellation-unsafe queue starvation. The fix is either per-handler state (message-passing actors) or at minimum splitting the lock into domain-specific sub-locks.

2. **`biased;` without fairness guarantees is a shutdown bomb**: Tokio's `biased;` mode puts the burden of fairness on the developer. With 30+ handlers all using `biased;` with tick-first, high-frequency handlers can starve shutdown signals indefinitely. The `cancel-safe-futures` crate's `coop_cancel` pattern or explicit `CancellationToken` would be more robust.

3. **Task abort during `read_exact`/`write_all` is a protocol corruption vector**: The SOCKS5 and HTTP proxy handlers use cancel-unsafe I/O operations (`read_exact`, `write_all`) in spawned tasks that get forcibly aborted during shutdown. The 5-second deadline in `handlers.rs` is too aggressive for protocol-level handlers that need clean disconnection. These should use cooperative cancellation with drain periods.

4. **Missing `std::io::AsyncWriteExt::shutdown()` on proxy teardown**: The bidirectional proxy in `kernel.rs:531-552` drops TCP sockets without calling `shutdown()` on the write halves. This prevents the peer from receiving FIN, leading to resource leaks and connection pool exhaustion under load.

5. **No use of `cancel-safe-futures` crate anywhere**: The codebase has zero adoption of the `cancel-safe-futures` library (which provides `then_try` adapters, `RobustMutex`, and `coop_cancel`). The `tokio::sync::Mutex` usage throughout NT-SHIELD and NT-MIND would benefit from `RobustMutex` which doesn't have the cancellation pitfalls of Tokio's built-in mutex.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Sources consulted | 13 |
| Files analyzed | 11 |
| `tokio::select!` sites audited | 11 |
| `tokio::sync::Mutex` usage sites audited | 11 |
