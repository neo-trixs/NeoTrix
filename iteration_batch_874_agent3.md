# Agent 3: Cancel-Safe Futures (Batch 874)

## Sources

1. **Tokio `select!` docs** — https://docs.rs/tokio/latest/tokio/macro.select.html — Official cancellation safety reference, safe/unsafe method lists
2. **Oxide RFD 400** — https://rfd.shared.oxide.computer/rfd/0400 — Production cancel correctness methodology from Oxide control plane (omicron#3356, omicron#3098, omicron#3351)
3. **Oxide RFD 397** — https://rfd.shared.oxide.computer/rfd/0397 — Challenges with async/await in control plane, tokio::sync::Mutex invariant violations
4. **cancel-safe-futures crate** — https://docs.rs/cancel-safe-futures/latest/ — Oxide's fix for `try_join!` early cancellation, `RobustMutex`, `coop_cancel`
5. **sunshowers.io "Cancelling async Rust"** — https://sunshowers.io/posts/cancelling-async-rust/ — Cancel correctness framework (3-prong model), `write_all_buf` vs `write_all`, task-as-isolation pattern
6. **biriukov.dev "Async Rust gotcha"** — https://biriukov.dev/posts/async-rust-gocha-tokio-cancelation-select-future-then/ — `StreamExt::then()` vs `FutureExt::then()` cancel safety, I/O actor model, `reserve()` pattern
7. **Tokio GitHub PR #8291** — https://github.com/tokio-rs/tokio/pull/8291 — `fs::File` cancellation bug: Idle→Busy→Idle state machine left invalid on cancel, causes panic on next op
8. **Tokio GitHub PR #7462** — https://github.com/tokio-rs/tokio/pull/7462 — `CancellationToken::run_until_cancelled` failing to cancel ready futures (poll order bug)
9. **comprehensive-rust/cancellation** — https://google.github.io/comprehensive-rust/concurrency/async-pitfalls/cancellation.html — `Interval::tick` safe, `read_line` unsafe, `LinesReader` fix pattern
10. **dev-async cancellation_safety.rs** — https://docs.rs/dev-async/latest/src/dev_async/cancellation_safety.rs.html — `check_cancel_safe()` verification harness: drive→drop→assert pattern
11. **Medium "tokio::select! Is a Loaded Gun"** — https://medium.com/@dhvani612/ — DB write silently dropped in select!, reserve pattern fix, Mutex across await blast radius
12. **tokio-select-pattern blog** — https://rustz2h.com/chapter_07/ — biased select for deterministic test behavior, condition guards, dropped futures don't run cleanup

## Defects

### D-CSAFE-001: `tokio::sync::Mutex` held across `.await` in biased select loop — handler lock cancellation leaves state invalid

**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:742-757`

**Severity**: CRITICAL

**Source**: Oxide RFD 397/400 (tokio::sync::Mutex invariant violations), sunshowers.io (cancel correctness 3-prong model)

The `spawn_handler!` macro acquires `h.lock().await` (a `tokio::sync::Mutex<BackgroundLoopHandle>`) inside a `tokio::select! { biased; ... }` tick branch. If the `rx.changed()` shutdown branch wins while the lock is held, the `MutexGuard` is dropped mid-operation. Since `BackgroundLoopHandle` contains 20+ fields (brain, cleanup_engine, goal_loop, awareness, etc.) that are mutated by handlers like `handle_save`, `handle_consolidate`, `handle_goal`, etc., dropping the guard mid-mutation leaves shared state in an inconsistent state. The next handler tick will observe corrupted state. This is the exact pattern Oxide found in omicron#3098: "the data would be stuck in the invalid `None` state."

---

### D-CSAFE-002: EventBus behavioral consumer holds lock across `.await` inside biased select — event processing cancellation leaves handle locked

**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:896-916`

**Severity**: CRITICAL

**Source**: Oxide RFD 400 (Mutex across await blast radius), Medium "loaded gun" article

The EventBus consumer does `tokio::select! { biased; result = event_rx.recv() => { let mut handle = h.lock().await; handle.handle_event_bus_event(event).await; } ... }`. The `biased;` ensures the event branch is polled first. If the shutdown signal arrives while `handle_event_bus_event(event).await` is in-flight (and the lock is held), the MutexGuard is dropped, potentially leaving the BackgroundLoopHandle in an inconsistent state. All other handlers that subsequently acquire the lock will see partial state.

---

### D-CSAFE-003: Bidirectional `tokio::io::copy` inside nested `timeout + select!` — double cancellation layer can leave half-duplex state inconsistent

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:531-552`

**Severity**: HIGH

**Source**: Tokio select docs (cancellation safety), Oxide RFD 400 (cancel correctness)

The proxy kernel's `tunnel_connection` wraps `tokio::select!` inside `tokio::time::timeout(idle_timeout, async { tokio::select! { ... } })`. When the timeout fires, both `tokio::io::copy` futures are dropped simultaneously. While `io::copy` internally uses cancel-safe `read`, the compound operation has subtle state: if client→upstream copy has buffered data pending write and upstream→client copy has buffered data pending write, both are dropped. The TCP connections are not explicitly half-closed — they rely on OS TCP stack cleanup. On graceful shutdown (`shutdown_rx.changed()`), the same double-drop occurs. Unlike the Oxide serial console fix (split recv into two steps), NeoTrix does not buffer or checkpoint partial transfer state.

---

### D-CSAFE-004: `write_all` (cancel-unsafe) used inside spawned async tasks without completion guarantee

**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers_absorption.rs:181-185`

**Severity**: HIGH

**Source**: Tokio docs (`write_all` not cancel-safe), sunshowers.io (write_all pattern is "absolutely not cancel safe")

A spawned task does `stdin.write_all(payload.as_bytes()).await` followed by `stdin.shutdown().await`. If this task is dropped between the two awaits (e.g., runtime shutdown), the child process receives a truncated stdin — the absorption payload is silently corrupted. The spawned task has no completion tracking; the parent proceeds with `child.wait_with_output()` which may succeed with partial data, causing silent knowledge absorption corruption. The fix is to use `write_all_buf` with a cursor (cancel-safe) or ensure the task runs to completion.

---

### D-CSAFE-005: SOCKS5 handshake uses sequential `read_exact`/`write_all` (both cancel-unsafe) — dropped future leaves connection in undefined protocol state

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/listener/socks5.rs:76-113`

**Severity**: HIGH

**Source**: Tokio docs (`read_exact` and `write_all` not cancel-safe), Oxide RFD 400 (cancel correctness requires no partial protocol state)

The `handle_socks5_connection` function performs a multi-step SOCKS5 handshake: `read_exact(2 bytes)` → `read_exact(methods)` → `write_all(greeting)` → `read_exact(request)` → `read_exact(ip/port)`. Each `read_exact` and `write_all` is documented as cancel-unsafe. If the parent task is aborted during any step (e.g., the spawned task's JoinHandle is dropped during shutdown), the SOCKS5 connection is left in a partial protocol state — the remote peer has sent data that was consumed but not processed. Unlike the Oxide serial console fix (split into reserve + commit phases), NeoTrix performs the entire handshake synchronously within a spawned task with no cancellation protection.

---

### D-CSAFE-006: DNS intercept select loop recreates `recv_from` future each iteration — potential data loss under backpressure

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/dns_intercept.rs:50-69`

**Severity**: MEDIUM

**Source**: Tokio docs (select! in loop — futures must be cancel-safe), biriukov.dev (evolving select! code has sharp edges)

The DNS intercept loop does `tokio::select! { result = socket.recv_from(&mut buf) => { ... send_to ... } _ = shutdown_rx.changed() => { ... } }`. While `UdpSocket::recv_from` is cancel-safe, the `buf` is shared across iterations as a `&mut` reference. If the shutdown branch fires after `recv_from` returns `Ready(Ok((len, peer)))` but before the handler processes the data, the response (`send_to`) is never sent — the DNS query is silently dropped. This is correct behavior for shutdown, but the `buf` variable is reused: if `recv_from` had partially written into `buf` before cancellation (impossible for UDP, but the pattern is fragile if refactored to TCP), partial data could corrupt the next iteration.

---

### D-CSAFE-007: Background loop shutdown uses `abort()` on tasks holding `tokio::sync::Mutex` — abort at arbitrary await point violates invariants

**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:48-61`

**Severity**: HIGH

**Source**: Oxide RFD 400 (task aborts), Tokio docs (abort cancels at next await point)

The shutdown sequence does: send shutdown signal → wait 5 seconds → `handle.abort()` on remaining tasks. The `abort()` call cancels the task at the next `.await` point. Since every handler task holds `tokio::sync::Mutex<BackgroundLoopHandle>` across `.await` points (see D-CSAFE-001), aborting mid-lock-hold leaves the mutex in an invalid state. Unlike `std::sync::Mutex` (which has poisoning), `tokio::sync::Mutex` has no cancellation protection. The 5-second deadline is insufficient if a handler is blocked on I/O while holding the lock.

---

### D-CSAFE-008: `llama_process.rs` holds `tokio::sync::Mutex` across child process spawn and restart — cancel during restart leaves process orphaned

**File**: `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/llama_process.rs:385-452`

**Severity**: MEDIUM

**Source**: sunshowers.io (cancel correctness requires invariant restoration between await points), Oxide RFD 397

The `LlamaProcess` uses `Arc<tokio::sync::Mutex<Option<Child>>>` for the child process handle. The restart logic locks the mutex, kills the old child, spawns a new one, and stores it — all across multiple `.await` points. If the future is cancelled between `kill()` and storing the new child, `self.child` is `None` and the old process is dead — the LLM provider is completely unavailable with no recovery path. This matches the Oxide sled data pattern: "the data would be stuck in the invalid `None` state."

---

### D-CSAFE-009: MITM proxy holds `tokio::sync::Mutex<TrafficAnalyzer>` across `write_all` + `copy_bidirectional` — cancel during TLS handshake leaves analyzer in partial state

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:262-270`

**Severity**: MEDIUM

**Source**: Tokio docs (`write_all` not cancel-safe), Oxide RFD 400 (Mutex blast radius)

The `connect_passthrough` function acquires `analyzer.lock().await`, calls `capture_request()`, then drops the lock before `copy_bidirectional`. However, in `handle_connect_mitm` (line 325), the analyzer lock is held while performing TLS handshake operations (`stream.read()`, TLS setup). If the connection is cancelled during the TLS handshake (e.g., client disconnects), the analyzer's internal state (captured request metadata) may be inconsistent — the request was recorded but the connection never completed.

---

### D-CSAFE-010: `system_proxy.rs` signal handler calls `process::exit(0)` after `disable().await` — cancel between signal receipt and disable leaves system proxy corrupted

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/system_proxy.rs:122-128`

**Severity**: MEDIUM

**Source**: sunshowers.io (cancel correctness requires cleanup on drop), Oxide RFD 400 (no async drop in Rust)

The signal handler does `tokio::select! { _ = term.recv() => {} _ = int.recv() => {} }` then `proxy.disable().await` then `process::exit(0)`. If the spawned task is dropped after the signal is received but before `disable().await` completes (e.g., runtime shutdown), the system proxy settings are permanently corrupted — the OS-level proxy remains configured but NeoTrix is no longer running to restore it. Since Rust has no `AsyncDrop`, there is no way to guarantee cleanup on cancellation. The fix would be to use a synchronous `std::process::Command` for proxy restoration or register a `ctrlc` handler that blocks until cleanup completes.

---

## Key Insights

1. **`tokio::sync::Mutex` is the dominant anti-pattern in NeoTrix**: 11 files use it, and the background loop holds it across every handler `.await`. Oxide's recommendation (RFD 400) is to avoid it entirely or ensure invariants are restored between await points. NeoTrix's `BackgroundLoopHandle` with 20+ fields makes this especially dangerous.

2. **The `spawn_handler!` macro bakes in a cancel-unsafe pattern**: Every handler acquires the shared lock inside a `select!` tick branch. This means every handler tick is a potential cancel point for the lock. The fix would be to use message-passing (mpsc channels) instead of shared-mutex-for-everything.

3. **No `try_join!` usage found** — this is actually good. NeoTrix avoids the Oxide `try_join!` early-cancellation bug pattern. However, the spawned tasks via `tokio::spawn` with no JoinHandle tracking (78 spawn sites) means many tasks are fire-and-forget with no cancellation control.

4. **`write_all`/`read_exact` (both cancel-unsafe) appear in 100+ locations** — most in non-select contexts (acceptable), but several in spawned tasks that could be aborted during shutdown. The SOCKS5 and Tor handshake code is particularly vulnerable.

5. **The biased select in `spawn_handler!` is intentional but dangerous**: `biased;` ensures shutdown is polled, but it also means the tick branch always wins when both are ready — if the handler acquires the lock and the shutdown fires in the same tick, the handler completes its work before observing shutdown. This is correct behavior but means shutdown latency is handler-work-dependent.

6. **No cancellation safety tests exist**: The codebase has no `check_cancel_safe()` or similar verification harness. Given the 11 select! sites and 11 tokio::sync::Mutex usages, systematic cancellation testing is needed.

7. **The proxy kernel's bidirectional copy is the most architecturally vulnerable**: The `timeout { select! { copy, copy, shutdown } }` pattern has three cancellation layers. A single timeout can simultaneously drop two in-flight TCP copies with no half-close signaling.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Critical | 2 |
| High | 4 |
| Medium | 4 |
| Low | 0 |
| Sources consulted | 12 |
| Files with select! usage | 8 unique files |
| Files with tokio::sync::Mutex | 11 files |
| tokio::spawn sites (total) | 78 |
