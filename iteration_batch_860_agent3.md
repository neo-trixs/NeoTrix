# Agent 3: Async Cancellation Safety (Batch 860)

## Sources
1. [Cancellation and cancellation safety — Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/part-reference/cancellation.html)
2. [RFD 400 Dealing with cancel safety in async Rust — Oxide](https://rfd.shared.oxide.computer/rfd/0400)
3. [cancel_safe_futures crate](https://docs.rs/cancel-safe-futures/latest/cancel_safe_futures/)
4. [Rain: "Cancelling Async Rust" | RustConf 2025](https://www.youtube.com/watch?v=zrv5Cy1R7r4)
5. [Cancelling async Rust | Engineered.at](https://engineered.at/articles/cancelling-async-rust)
6. [Graceful Shutdown — Tokio](https://tokio.rs/tokio/topics/shutdown)
7. [CancellationToken — tokio_util](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html)
8. [Rust tokio task cancellation patterns](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/)
9. [Structured concurrency — Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/part-reference/structured.html)
10. [Tree-Structured Concurrency — Yoshua Wuyts](https://blog.yoshuawuyts.com/tree-structured-concurrency/index)
11. [structured_spawn crate](https://docs.rs/structured-spawn/latest/structured_spawn/)
12. [The Scoped Task trilemma — without.boats](https://without.boats/blog/the-scoped-task-trilemma/)
13. [Select — Tokio docs](https://tokio.rs/tokio/tutorial/select)
14. [tokio::select! cancellation safety docs](https://docs.rs/tokio/latest/tokio/macro.select.html)

## Defects

**D-CANCEL-001: `tokio::sync::Mutex` lock held across `await` inside `select!` branches — cancel-unsafe priority inversion across 35+ background handlers**
`neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:742-757`
Severity: HIGH
Source: Tokio select! cancellation safety docs — `tokio::sync::Mutex::lock` is explicitly listed as **not cancel-safe** because it uses a FIFO queue; cancelling drops your place in line. The `spawn_handler!` macro acquires `h.lock().await` inside the biased `select!` tick branch. When the shutdown branch fires (or a competing handler tick completes), the lock-acquisition future is dropped. Because all 35+ handlers share the same `Arc<Mutex<BackgroundLoop>>`, a cancelled lock acquisition wastes a queue slot and introduces non-deterministic starvation under concurrent load. The RustConf 2025 talk (Rain) confirms: "Tokio mutexes are very prone to cancel correctness issues" — the `cancel-safe-futures` crate exists specifically to address this with `RobustMutex`.

---

**D-CANCEL-002: `write_all` used in proxy hot paths — 73 occurrences of cancel-unsafe I/O**
`nt_shield_proxy_kernel/listener/socks5.rs:65,81,126,145,156,165,178` | `nt_shield_proxy_kernel/listener/http.rs:72,97,121,142,147,155,161` | `nt_shield_traffic/mitm.rs:262,289,360,383` | `nt_shield_stealth_net/tor_client.rs:155,173,314,323,331` | `nt_shield_stealth_net/local_proxy.rs:115,148,166,331,351`
Severity: HIGH
Source: Tokio docs explicitly list `AsyncWriteExt::write_all` as **not cancellation safe** — "can lead to loss of data". If a `write_all` future is dropped mid-operation (e.g., during shutdown select! or timeout), the remote peer sees a truncated write. For SOCKS5 handshakes this corrupts the connection state machine; for HTTP CONNECT tunnels it leaves a half-established tunnel. The `dev_async::cancellation_safety` docs note: "Futures that buffer writes but flush mid-await" are a classic cancel-unsafe pattern. Every `write_all` in the proxy path is vulnerable to this during shutdown or `select!` race.

---

**D-CANCEL-003: Zero structured concurrency — 49+ `tokio::spawn` calls with no `JoinSet` or `CancellationToken`**
`neotrix-core/src/` (49+ files)
Severity: HIGH
Source: The Rust async book's [Structured concurrency chapter](https://rust-lang.github.io/async-book/part-reference/structured.html) states: "The easiest way to follow structured concurrency is to use futures and concurrent composition rather than tasks and spawning." Yoshua Wuyts' tree-structured concurrency post: "Neither async-std nor tokio provide a `spawn` function which is structured." The `structured_spawn` crate docs: "When a `TaskHandle` is dropped, the underlying task is cancelled." NeoTrix has **zero** uses of `JoinSet` and **zero** uses of `CancellationToken`. All 49+ spawned tasks are detached — their `JoinHandle`s are either stored in a `Vec` (background loop) or immediately dropped (proxy kernel, providers, web server). This means: (a) no cancellation propagation from parent to children, (b) no error propagation from children to parents, (c) spawned tasks can outlive their creating scope. The scoped task trilemma (without.boats) confirms this is a fundamental architectural gap in Rust async.

---

**D-CANCEL-004: ProxyKernel `forward_traffic` — dual `tokio::io::copy` in `select!` without cancellation-safe half-close**
`nt_shield_proxy_kernel/kernel.rs:530-552`
Severity: MEDIUM
Source: The `forward_traffic` method uses `tokio::select!` with two `tokio::io::copy` branches (client→upstream and upstream→client). When one direction completes or the shutdown signal fires, the other copy future is dropped. While `tokio::io::copy` is cancellation-safe at the byte level (it tracks internal cursor state), the dropped direction does NOT send a TCP FIN/half-close to the remote peer. The Oxide RFD 400 notes: "cancellation is an example of 'spooky action at a distance'" — here, a shutdown signal silently kills one direction while the other side may continue writing into a void. Combined with `tokio::time::timeout` wrapping the entire select, a stale connection can persist for up to `idle_timeout_secs` after shutdown begins.

---

**D-CANCEL-005: Background loop shutdown uses a single shared deadline — abort leaves KB writes mid-operation**
`nt_mind_background_loop/handlers.rs:48-61`
Severity: MEDIUM
Source: The `shutdown` method creates a single 5-second `deadline` and iterates through all handles, racing each `handle` against the shared deadline. Once the first task exceeds the deadline, `abort()` is called. But `abort()` only **schedules** cancellation — the task may still be mid-KB-write when the runtime drops it. The Tokio docs state: "aborting a task does not guarantee that it fails with a cancelled error, since it may complete normally first." Since multiple handlers (save, consolidate, kb_guard, kb_backup) perform SQLite writes via the shared KB, an abort during a write transaction can leave the KB in an inconsistent state. The next startup's `handle_pending_absorption` may read corrupted data. No WAL checkpoint or transaction rollback is performed before abort.

---

**D-CANCEL-006: EventBus consumer holds `BackgroundLoop` mutex lock while processing events — blocks all other 35+ handlers**
`nt_mind_background_loop/run.rs:896-916`
Severity: MEDIUM
Source: The EventBus consumer task acquires `h.lock().await` inside a `select!` branch (line 901). If event processing (`handle_event_bus_event`) involves I/O (KB writes, file reads), the lock is held for the duration. During this time, **all other background handlers** (save, consolidate, goal, crystallization, consciousness_tick, etc.) cannot acquire the lock and are blocked. With 35+ handlers all competing for the same lock, a slow event handler creates a priority inversion. The `spawn_handler!` macro's `biased` select already prioritizes shutdown over tick, but the EventBus consumer is a separate task competing for the same lock without bias. This is a cancel-correctness concern: if the event handler future is cancelled mid-lock-acquisition (via select!), the lock state may be inconsistent.

---

**D-CANCEL-007: `system_proxy` signal handler calls `std::process::exit(0)` — bypasses all Drop implementations**
`nt_shield_stealth_net/system_proxy.rs:121-129`
Severity: MEDIUM
Source: The signal handler spawns a task that calls `std::process::exit(0)` after restoring proxy settings. This immediately terminates the process without running any destructors — including Tokio's runtime drop (which would cancel all tasks), SQLite connection pool cleanup, file sync, and KB WAL checkpoint. The Tokio graceful shutdown docs explicitly warn: "shutting down a Tokio runtime (e.g. by returning from `#[tokio::main]`) immediately cancels all tasks on it" — but `process::exit` bypasses even this. For a consciousness system with persistent KB state, this can corrupt the SQLite database on every SIGTERM/SIGINT. The production patterns doc (microsoft.github.io) recommends: "Always have a timeout on the shutdown wait — a hung task shouldn't prevent process exit" — but this skips the entire shutdown sequence.

---

**D-CANCEL-008: Proxy kernel `select!` checks listener `JoinHandle` via `&mut` — does not await completion of in-flight connections**
`nt_shield_proxy_kernel/kernel.rs:256-272`
Severity: LOW
Source: The main loop uses `tokio::select!` with `&mut socks5_handle` and `&mut http_handle`. When one listener exits (or panics), the select breaks and the main loop falls through to the drain phase. However, the drain phase (line 279+) only checks `active_connections` — it does NOT track or await the connection-handling tasks spawned inside the listener's `tokio::spawn` calls. The `active_connections` counter is an `AtomicUsize` that may not accurately reflect in-flight tasks if any connection handler panics (counter not decremented on panic path). The structured concurrency docs state: "No tasks are leaked from a scope" — here, connection tasks spawned by the dying listener can outlive the drain deadline.

---

**D-CANCEL-009: Background loop `spawn_handler!` macro recreates futures on each tick — loss of partial progress on cancellation**
`nt_mind_background_loop/run.rs:739-744`
Severity: LOW
Source: The `spawn_handler!` macro creates a fresh `ticker` inside each spawned task, but the actual handler body (e.g., `h.handle_consolidate().await`) is called as a fresh async expression each tick. If a handler's execution is partially complete when the `select!` picks the shutdown branch, all intermediate state is lost. The Tokio select! docs note: "Resume futures in `select!` loops rather than recreating them" as a key cancel-safety strategy. While the handlers themselves are designed to be idempotent, the pattern of "select between tick and shutdown" means the handler body is always a fresh future — it cannot resume from partial progress. For handlers like `handle_novel_ingest` (which drains a queue), this means partially processed items are re-processed on next tick (if any), which is wasteful but not corrupting. However, for `handle_wisdom_tick` (which does value learning + narrative integration), partial progress loss means learning state is reset each time shutdown fires during execution.

---

**D-CANCEL-010: No cooperative cancellation — all shutdown relies on `abort()` which is immediate and uncooperative**
`nt_mind_background_loop/handlers.rs:56` | `nt_core_prm/learner.rs:302`
Severity: LOW
Source: The codebase uses `abort()` in two places: background handler shutdown (line 56) and PRM learner cancellation (line 302). The `cancel-safe-futures` crate docs explain: "Executors like Tokio support forcible cancellation for async tasks via facilities like `JoinHandle::abort`. However, this causes cancellations at any arbitrary await point. This is often not desirable because it can lead to invariant violations." The Tokio graceful shutdown docs recommend CancellationTokens for cooperative cancellation where tasks can "run a shutdown procedure before terminating, such as flushing data to a file or database." NeoTrix has zero CancellationToken usage. The entire background loop and all spawned tasks rely on abort-on-deadline rather than cooperative cancellation with drain.

## Key Insights

1. **The core architectural gap is the absence of structured concurrency.** With 49+ `tokio::spawn` calls and zero `JoinSet` or `CancellationToken` usage, NeoTrix has no mechanism for cancellation propagation (parent→child), error propagation (child→parent), or guaranteed ordering of operations. This is the single most impactful finding — every other defect flows from this.

2. **`tokio::sync::Mutex` is the wrong primitive for the background loop.** The 35+ handlers all compete for a single `Arc<Mutex<BackgroundLoop>>`, and the lock is held across `await` points inside `select!` branches. The Tokio docs, Oxide RFD 400, Rain's RustConf talk, and the `cancel-safe-futures` crate all converge on the same conclusion: `tokio::sync::Mutex` should be avoided in cancel-sensitive code. Consider `std::sync::Mutex` (if locks are short), `tokio::sync::RwLock` (for read-heavy workloads), or restructuring to use per-handler state via channels.

3. **73 `write_all` calls in the proxy/traffic path are ticking time bombs.** Every one of these can corrupt protocol state machines (SOCKS5 handshake, HTTP CONNECT, Tor SOCKS5 auth) if the future is dropped mid-operation. The `write_all`→`reserve` pattern (documented in the Tokio async book) should be applied: reserve a buffer slot first (cancel-safe), then write the data (now infallible).

4. **`std::process::exit(0)` in the system proxy signal handler is a data corruption vector.** For a system with a SQLite-backed KB, bypassing Drop means no WAL checkpoint, no transaction rollback, and no connection pool cleanup. This should be replaced with a proper signal-driven graceful shutdown that flows through the runtime's own drop sequence.

5. **The shared 5-second shutdown deadline is a race.** All 35+ handlers are given a collective 5 seconds to finish, but they are drained sequentially (one `select!` per handle). If the first handler takes 4.9 seconds, the remaining 34 handlers get 0.1 seconds total before being aborted. The `ShutdownManager` pattern from the production patterns article (phased shutdown with per-component timeout) would be more appropriate.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| D-CANCEL range | D-CANCEL-001 through D-CANCEL-010 |
| HIGH severity | 3 |
| MEDIUM severity | 4 |
| LOW severity | 3 |
| Sources consulted | 14 |
| Files examined in codebase | 15+ |
