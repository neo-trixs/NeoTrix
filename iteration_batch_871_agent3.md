# Agent 3: Cancel-Safe Futures (Batch 871)

## Sources
1. Tokio `select!` macro docs — cancellation safety section, biased mode, branch dropping semantics
2. sunshowers.io "Cancelling async Rust" — cancel safety vs cancel correctness, Tokio mutex pain, partial-progress bugs
3. Medium "tokio::select! Is a Loaded Gun" — reserve pattern, Mutex trap, partial DB writes
4. developerlife.com "Rust async cancellation safety" — interval vs sleep cancel safety, stream cancellation loss
5. Rust Users Forum "Cancel safety in async and tokio::select!" — borrowing future cancellation, nested select safety
6. Rust Users Forum "Cancellation safety of locks in select!" — Mutex queue-loss, pin-and-reuse pattern
7. Rust Users Forum "Is tokio's select! cancel safe?" — nested select cancellation transitivity
8. Stanza "Cancellation Safety" course — BufReader::read_line loss, resume pattern, cancel-safe recv
9. Rust async-book "Cancellation and cancellation safety" — internal vs external cancellation, halt safety
10. Rustify "tokio::select! Guide 2026" — select vs join semantics, shutdown + work loop pattern
11. Google comprehensive-rust "async pitfalls: cancellation" — compiler doesn't enforce cancel safety
12. kindatechnical.com "Async Pitfalls: Blocking, Cancellation, Backpressure" — MutexGuard across await, Tokio Mutex vs std Mutex
13. blog.asteromorph.com "Cancel safety in Rust's Future" — CancelSafeFuture trait proposal, futures are passive
14. github.com/sunshowers/cancelling-async-rust — Oxide RFD 397/400, structured concurrency, panic=cancel analogy
15. lobste.rs "Dealing with cancel safety in async Rust" — select-in-loop idiom criticism, lossy cancellation
16. microsoft/RustTraining async-book ch12 — select starvation, fairness, Mutex deadlock patterns

## Defects

D-CSAFE-001: Biased select with deadline-first ordering causes premature abort of all remaining handlers during shutdown. In the shutdown loop (handlers.rs:53), `biased;` checks `deadline` before `handle`. Once deadline fires on the first slow handler, every subsequent handle in `drain(..)` immediately hits the expired deadline and is aborted — even handlers that were mere milliseconds from completing. Should use non-biased or pin-and-reuse deadline. | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:53` | medium | Tokio docs, sunshowers.io

D-CSAFE-002: EventBus behavioral consumer holds MutexGuard across event handler await inside select loop. At run.rs:901, `h.lock().await` acquires the background loop Mutex, then `handle_event_bus_event(event).await` holds it across an arbitrary async handler. If the handler blocks on I/O or is slow, other handlers starve on lock acquisition. If shutdown wins the select, the spawned task is dropped mid-guard — the lock is held until tokio fully drops the future, causing a temporary lockout. | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:896-903` | high | kindatechnical.com, Rust Users Forum lock-in-select

D-CSAFE-003: Bidirectional tunnel select drops in-progress io::copy on shutdown without flushing. At kernel.rs:532, `tokio::select!` races `io::copy` (client→upstream) vs `io::copy` (upstream→client) vs `shutdown_rx`. When shutdown fires, both copy futures are dropped mid-stream. While kernel TCP buffers may hold some data, the async runtime provides no flush guarantee on drop. Upstream or client may receive truncated responses. No explicit `shutdown()` or `flush()` is performed before dropping the split halves. | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:530-552` | medium | Tokio docs (write_all not cancel-safe), sunshowers.io

D-CSAFE-004: Signal handler uses process::exit after async disable — skips async cleanup. At system_proxy.rs:128, `proxy.disable().await` performs async OS proxy restoration, then `std::process::exit(0)` is called. This skips all Drop impls, pending tokio tasks, and buffered I/O flushes. If disable() involves spawned cleanup tasks or channel sends, those are silently lost. The Tokio runtime's graceful shutdown is entirely bypassed. | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/system_proxy.rs:122-129` | high | sunshowers.io (async cleanup), microsoft/RustTraining ch12

D-CSAFE-005: DNS intercept select lacks biased — inconsistent shutdown semantics. At dns_intercept.rs:50, `tokio::select!` (non-biased) races `socket.recv_from` vs `shutdown_rx.changed()`. Without `biased;`, the runtime randomly selects which ready branch to poll first. Under load, a burst of DNS queries may delay shutdown recognition by several poll cycles. Inconsistent with proxy kernel which uses `biased;` for the same shutdown pattern. | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/dns_intercept.rs:50` | low | Tokio docs biased mode, Rustify guide

D-CSAFE-006: SOCKS5 handler uses read_exact (cancel-unsafe) without buffering. At socks5.rs:76, `stream.read_exact(&mut buf).await` is cancel-unsafe — if the future is dropped mid-read, partially-read bytes are lost from the buffer. While this code is inside `tokio::spawn` (not directly in select!), if the task is aborted during shutdown, the partial read is lost. The 2-byte buffer is small, but the pattern is fragile if the handler is ever moved into a select! branch. | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/listener/socks5.rs:76` | low | Tokio docs (read_exact not cancel-safe), Stanza course

D-CSAFE-007: spawn_handler! macro acquires Mutex inside select branch — body cancellation leaves lock held. At run.rs:745, `h.lock().await` is called inside the `ticker.tick()` branch of a `biased; select!`. If the handler body contains multiple await points and the shutdown signal fires between them, the task is dropped while holding the Arc<Mutex> guard. The lock remains held until tokio completes future destruction. With 20+ handlers competing for the same lock, a shutdown during a slow handler cascades into delayed cleanup. | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:742-757` | medium | kindatechnical.com, sunshowers.io (Tokio mutex pain)

## Key Insights

1. **Cancel safety is a local property, cancel correctness is global** (sunshowers.io): NeoTrix's proxy and background loop systems have locally cancel-unsafe futures (read_exact, write_all, Mutex across await) that may be globally acceptable if cancellation never violates a system invariant — but this is not audited or documented per-site.

2. **`biased;` is a correctness tool, not just a performance hint**: The proxy kernel uses `biased;` for shutdown-first, but dns_intercept and signal handlers don't. Inconsistent biased usage means shutdown timing varies across subsystems — some shut down deterministically, others randomly.

3. **Tokio Mutex across await inside select is a double hazard**: It combines (a) lock starvation when the handler body is slow, and (b) delayed lock release on cancellation. NeoTrix's background loop uses a single shared Mutex for 20+ handlers — a worst-case contention scenario.

4. **process::exit bypasses all async cleanup**: The system proxy signal handler calls `process::exit(0)` after async disable, which means any in-flight proxy connections, buffered channel messages, or spawned cleanup tasks are silently dropped. This is a correctness hazard for OS proxy settings restoration.

5. **No CancelSafeFuture marker trait exists in Rust**: The ecosystem relies on per-method documentation. NeoTrix has no internal audit or documentation of which futures are cancel-safe, making it easy for future code to introduce cancellation bugs in select! loops.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 7 |
| Sources consulted | 16 |
| Critical (P0) | 0 |
| High severity | 2 |
| Medium severity | 3 |
| Low severity | 2 |
