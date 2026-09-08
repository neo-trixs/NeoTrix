# Agent 3: Async Cancellation Safety (Batch 875)

## Sources

1. Tokio docs: `select!` macro — cancellation safety section (docs.rs/tokio/latest/tokio/macro.select.html)
2. Tokio tutorial: Select — cancelling futures (tokio.rs/tokio/tutorial/select)
3. Oxide RFD 400: Dealing with cancel safety in async Rust (rfd.shared.oxide.computer/rfd/0400)
4. Biriukov: Async Rust gotcha — evolving tokio::select! code has sharp edges (biriukov.dev, Feb 2026)
5. Atharva Pandey: Lesson 10 — Cancellation Safety, The Silent Footgun (atharvapandey.com)
6. Nazmul Idris: Build with Naz — Rust async in practice tokio::select! (developerlife.com)
7. cancel-safe-futures crate: Oxide cooperative cancellation (docs.rs/cancel-safe-futures)
8. users.rust-lang.org: Is tokio's select! cancel safe? discussion (2026-02)
9. Engineered.at: Cancelling async Rust (2025-10)
10. Barafael: Stop Worrying and Learn to Loop-Select (2025-02)
11. rust-skills/async-cancel-safety rules (github.com/leonardomso/rust-skills)
12. Cybernetist: Rust tokio task cancellation patterns (2024-04)

## Defects

### D-CSAFE-001: `tokio::io::copy` in select! — partial transfer loss on shutdown

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:531-551`
**Severity**: HIGH

The `run_bidirectional` function uses `tokio::select!` with two `tokio::io::copy` branches and a `shutdown_rx.changed()` branch. When the shutdown branch fires, both `copy` futures are dropped. Per Tokio docs, `tokio::io::copy` internally calls `read` then `write` in a loop — the `read` is cancel-safe but the subsequent `write_all` is not. If the select drops the copy future mid-write, the bytes already read from the source but not yet written to the sink are silently lost. The TCP stream state becomes inconsistent: the receiver side consumed bytes that the sender side never delivered.

**Reference**: Tokio docs list `write_all` as NOT cancel-safe. Oxide RFD 400 §select_loops documents the exact pattern where `send(value)` (analogous to write) causes data loss when cancelled.

### D-CSAFE-002: `write_all` in handler tasks without cancel-safety — silent data loss in SOCKS5/HTTP proxy listeners

**Files**: `listener/socks5.rs:65,80-81,85,95-113,126,145,156,165,178` and `listener/http.rs:72,97,121,142,147,155,161`
**Severity**: MEDIUM

Both SOCKS5 and HTTP proxy connection handlers use `stream.write_all(...)` and `stream.read_exact(...)` extensively. While these handlers are spawned as independent tasks (not inside select! loops themselves), the spawn is done inside a `select!` loop at `listener/socks5.rs:40-58` and `listener/http.rs:41-59`. If the `shutdown_rx.changed()` branch fires while a `select!` iteration is in progress, the `accept()` future is dropped (cancel-safe), but any in-flight spawned task from a prior iteration continues without supervision. There is no `JoinSet` or task tracker — spawned tasks are fire-and-forget, meaning `write_all` partial writes in handlers are never checked for completion on shutdown.

**Reference**: Tokio graceful shutdown guide recommends using `JoinSet` with `AbortHandle` for supervised shutdown. The current code at `handlers.rs:51-61` does use `abort_handle`, but only for the background loop handlers, not for connection handler tasks spawned by the listeners.

### D-CSAFE-003: `Mutex::lock().await` inside select! branch — lost queue position

**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:745` and `run.rs:901`
**Severity**: MEDIUM

The `spawn_handler!` macro (line 735-760) acquires `h.lock().await` inside a `select!` branch. When `biased;` is used and the shutdown branch fires while a handler is waiting to acquire the lock, the `lock()` future is dropped. Per Tokio docs, `Mutex::lock` is NOT cancel-safe — dropping it loses the task's place in the fairness queue. Under high contention (multiple handlers competing for the same `BackgroundLoopHandle` lock), this means shutdown can cause handlers to starve indefinitely as they repeatedly lose their queue position.

**Reference**: Tokio docs explicitly list `tokio::sync::Mutex::lock` as NOT cancel-safe because "cancellation makes you lose your place in the queue."

### D-CSAFE-004: `tokio::io::copy_bidirectional` without cancellation guard — half-duplex state corruption

**Files**: `listener/socks5.rs:160`, `listener/http.rs:151`, `nt_shield_traffic/mitm.rs:270,329`, `nt_shield_stealth_net/local_proxy.rs:205-206`
**Severity**: MEDIUM

Multiple locations use `tokio::io::copy_bidirectional` as a standalone `.await` without any timeout, cancellation token, or select! guard. This function runs until one side closes the connection. If the task is aborted (via `JoinHandle::abort` or parent task drop), the TCP streams are dropped without proper half-close (FIN). The remote peer sees a connection reset instead of a clean EOF. The SOCKS5 handler at `socks5.rs:160` is particularly problematic because it's inside a spawned task with no shutdown coordination — if the kernel shuts down, these relay tasks are aborted at arbitrary points in the bidirectional copy.

**Reference**: Oxide RFD 400 §task_aborts documents that aborting tasks mid-operation leads to invariant violations. The recommended pattern is cooperative cancellation via `CancellationToken` in a `select!` loop.

### D-CSAFE-005: `tokio::io::copy` in `local_proxy.rs` relay — no shutdown coordination

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/local_proxy.rs:201-208`
**Severity**: MEDIUM

The `relay` function uses `tokio::join!` with two `tokio::io::copy` calls. `tokio::join!` runs both futures concurrently but does NOT provide cancellation semantics — when one copy completes (or fails), the other continues running until the function returns. There is no mechanism to cancel the relay on shutdown. If the parent task is dropped, both copy operations are abandoned mid-flight, potentially leaving sockets in a half-open state with unread data.

**Reference**: The `cancel-safe-futures` crate's `coop_cancel` module specifically addresses this pattern — cooperative cancellation should be used instead of relying on task drop.

### D-CSAFE-006: `shutdown_rx.changed()` inside select! — watch::Receiver is cancel-safe but borrow semantics may mask races

**Files**: `kernel.rs:548`, `dns_intercept.rs:63`, `listener/socks5.rs:51`, `listener/http.rs:52`
**Severity**: LOW

All four `select!` loops use `shutdown_rx.changed()` as the cancellation branch. While `watch::Receiver::changed()` IS cancel-safe per Tokio docs, the `*shutdown_rx.borrow()` pattern (e.g., `kernel.rs:228,548`) reads the latest value after `changed()` resolves. The race: between `changed()` returning `Ready` and the `borrow()` call, another task could theoretically update the watch channel again. This is a very narrow window but could theoretically cause a missed shutdown signal if the value is toggled false→true→false rapidly.

**Reference**: Tokio watch channel docs warn that `borrow()` returns the latest value, not the value that triggered `changed()`. The idiomatic pattern is to use the value returned by `changed()` directly when possible.

### D-CSAFE-007: No `JoinSet` for connection handler tasks — uncontrolled task proliferation

**Files**: `listener/socks5.rs:45-49`, `listener/http.rs:46-50`
**Severity**: MEDIUM

Both listener loops spawn connection handlers with `tokio::spawn(async move { ... })` and discard the `JoinHandle`. This means:
1. No backpressure — an attacker can open thousands of connections and each spawns a task
2. No graceful shutdown — spawned tasks are not tracked, so shutdown cannot wait for them to complete
3. No limit on concurrent connections — the `active_connections` counter is advisory, not enforced

The `handlers.rs:51-61` code shows awareness of this pattern for background loop handlers (using `abort_handle`), but the connection handler tasks spawned by listeners lack this supervision.

**Reference**: Tokio graceful shutdown guide recommends `JoinSet` + cooperative `CancellationToken` for tracked, graceful shutdown of spawned tasks.

### D-CSAFE-008: `biased;` select! with lock acquisition — starvation risk under contention

**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:742-757`
**Severity**: LOW

The `spawn_handler!` macro uses `biased;` in the `select!` macro, which polls branches top-to-bottom. The shutdown branch (`rx.changed()`) is listed second (line 753), so the tick branch is always polled first. This is intentional for normal operation, but during shutdown, the shutdown signal may be delayed if the tick branch's lock acquisition is slow. With multiple handlers competing for the same lock, this creates a worst-case scenario where shutdown must wait for ALL handlers to complete their current lock-held work before any can see the shutdown signal.

**Reference**: The Tokio `biased;` docs explain that this gives strict priority ordering. The Oxide RFD 400 recommends considering stream merging as an alternative to avoid these ordering issues.

## Key Insights

1. **The most critical defect (D-CSAFE-001) is a data-loss bug in the proxy kernel's bidirectional relay.** When `shutdown_rx.changed()` fires during `tokio::io::copy`, bytes read from the source but not yet written to the sink are silently lost. This violates the proxy's implicit contract of reliable forwarding.

2. **The absence of `CancellationToken` is a systemic gap.** NeoTrix uses `watch::Receiver<bool>` for shutdown signals, which works but is less ergonomic than `tokio_util::sync::CancellationToken`. The `cancel-safe-futures` crate (Oxide) specifically provides cooperative cancellation primitives that are designed for this use case.

3. **Fire-and-forget `tokio::spawn` for connection handlers is a shutdown reliability risk.** The background loop handlers (run.rs) have proper `JoinSet`-like tracking via `self.handles`, but connection handlers do not. This creates an asymmetric shutdown model where background tasks are gracefully stopped but connection tasks are forcibly aborted.

4. **The `Mutex::lock().await` inside select! (D-CSAFE-003) is a known anti-pattern.** Tokio's own docs warn against this. The fix is either to use `std::sync::Mutex` if the lock is not held across `.await` points, or to restructure the code to acquire the lock outside the `select!` loop.

5. **Multiple `write_all` and `read_exact` calls throughout the proxy stack are not cancel-safe.** While most are outside `select!` loops (called within spawned handler tasks), any future refactoring that wraps them in `select!` for timeout handling would introduce silent data loss. This is a latent risk that should be documented.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Sources consulted | 12 |
| Files with select! usage | 9 |
| Severity: HIGH | 1 |
| Severity: MEDIUM | 5 |
| Severity: LOW | 2 |
