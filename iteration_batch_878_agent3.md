# Batch 878 — Agent 3: Rust Cancel-Safe Futures & `select!` Patterns

## Research Sources

1. **Tokio docs** — `tokio::select!` cancellation safety: cancel-safe vs non-cancel-safe method lists, `biased;` semantics
2. **Oxide RFD 400** — "Dealing with cancel safety in async Rust" (Sep 2025): `Sender::send` not cancel-safe, `Mutex` trap, `try_join!` early-cancellation, cooperative cancellation
3. **Biriukov (Feb 2026)** — "Async Rust gotcha: evolving tokio::select! code has sharp edges": `FutureExt::then()` cancel-unsafe, `StreamExt::then()` safer, `reserve()` pattern, test `Poll::Pending` paths
4. **cancel-safe-futures crate** (docs.rs) — `coop_cancel` cooperative cancellation, `then_try` alternatives to `try_join!`
5. **Rust async book** — Cancellation safety definition: dropping and restarting must be a no-op
6. **leonardomso/rust-skills** — `async-cancel-safety.md`: pinned futures, buffer-outside-select pattern
7. **Google Comprehensive Rust** — `read_line` / `BufReader` not cancel-safe, `interval::tick` is cancel-safe
8. **Stack Overflow #74547855** — Assessing cancel safety: verify all `.await` points
9. **stanza.dev course** — Common pitfalls: multi-step mutations inside `select!` branches
10. **Rust Users Forum #92381** — Borrowing future cancellation, channel recv cancel-safety
11. **Medium "tokio::select! Is a Loaded Gun"** — `reserve` pattern fix, Mutex trap, partial DB writes
12. **previous batch reports** — 873, 874, 875, 876, 877 all flagging NeoTrix `select!` defects

---

## Defects

### D-CSAFE-001: `spawn_handler!` macro acquires Mutex inside `select!` tick branch — lock drop on shutdown leaves inconsistent state

**File**: `nt_mind_background_loop/run.rs:734-760`
**Severity**: HIGH

The `spawn_handler!` macro expands to:
```rust
tokio::select! {
    biased;
    _ = ticker.tick() => {
        let mut $lock = h.lock().await;  // <-- lock acquired INSIDE select
        $body;                           // <-- handler body may have multiple .await
    }
    _ = rx.changed() => { break; }
}
```

When the shutdown branch wins (or the handler body panics), the `MutexGuard` is dropped mid-operation. Since `BackgroundLoopHandle` contains 20+ fields mutated by handlers (save, consolidate, goal, crystallization), dropping the guard mid-mutation leaves shared state in an inconsistent state.

**Reference**: Oxide RFD 400: "if a future holding a mutex is cancelled, the state guarded by the mutex is likely invalid." Tokio docs: `Mutex::lock` is "not cancellation safe — uses a queue for fairness; cancellation makes you lose your place in the queue."

**Impact**: 35+ handlers all share `Arc<Mutex<BackgroundLoop>>`. A cancelled lock acquisition wastes a queue slot; under concurrent load, FIFO ordering breaks down.

**Fix**: Move lock acquisition outside the select branch, or use `std::sync::Mutex` (synchronous, no queue loss), or restructure so handler body does not hold the lock across `.await` points.

---

### D-CSAFE-002: Shutdown drain shares a single 5-second deadline across all 35+ handlers — sequential starvation cascade

**File**: `nt_mind_background_loop/handlers.rs:48-61`
**Severity**: HIGH

```rust
let deadline = tokio::time::sleep(Duration::from_secs(5));
tokio::pin!(deadline);
for handle in self.handles.drain(..) {
    let abort = handle.abort_handle();
    tokio::select! {
        biased;
        _ = &mut deadline => { abort.abort(); }
        _ = handle => {}
    }
}
```

The single pinned `deadline` future is shared across all iterations. If handler A takes 4.9s, handler B gets 0.1s, handler C and all remaining handlers get 0s (immediately aborted). This creates an unfair shutdown where handler ordering determines who gets grace time.

**Reference**: Oxide RFD 400 recommends per-task deadlines or `JoinSet::abort_all()` after a coordinator-level timeout. Tokio docs: "You should place the shutdown future earlier in the `select!` list" — but a shared deadline inherently starves later handlers.

**Fix**: Use `tokio::time::timeout(Duration::from_secs(5), handle)` per-handler, or `FuturesUnordered` with per-task timeouts, or `JoinSet::try_join_next()`.

---

### D-CSAFE-003: Bidirectional tunnel `select!` drops in-progress `io::copy` on shutdown without flushing or half-closing

**File**: `nt_shield_proxy_kernel/kernel.rs:544-566`
**Severity**: MEDIUM

```rust
tokio::select! {
    result = tokio::io::copy(&mut client_read, &mut upstream_write) => { ... }
    result = tokio::io::copy(&mut upstream_read, &mut client_write) => { ... }
    _ = shutdown_rx.changed() => { /* no flush, no half-close */ }
}
```

When shutdown fires, both `io::copy` futures are dropped. While `read` is cancel-safe, `write_all` (called internally by `copy`) is explicitly listed as **not cancel-safe** by Tokio. If one direction has buffered data pending write, that data is silently lost. No explicit `shutdown(Write)` or `flush()` is performed before dropping the split halves. The TCP connections are not half-closed — they rely on OS TCP stack cleanup.

**Reference**: Tokio docs: `AsyncWriteExt::write_all` is not cancellation safe. Biriukov: "`write_all()` is not message-atomic under cancellation — partial message on the wire."

**Fix**: Use `tokio::io::split` → after select, explicitly call `client_write.shutdown().await` and `upstream_write.shutdown().await`. Or use the `reserve()` permit pattern for the write side.

---

### D-CSAFE-004: SOCKS5 and HTTP listener loops lack `biased;` — new connections accepted after shutdown

**File**: `nt_shield_proxy_kernel/listener/socks5.rs:40-57`, `listener/http.rs:41`
**Severity**: MEDIUM

Both listener loops use `tokio::select!` **without** `biased;`:
```rust
tokio::select! {
    result = listener.accept() => {
        let (stream, addr) = result?;
        tokio::spawn(async move { handle_socks5_connection(stream, kernel).await });
    }
    _ = shutdown_rx.changed() => { break; }
}
```

Without `biased;`, Tokio randomly selects which ready branch to poll first. Per Tokio docs: "if you are selecting between a stream and a shutdown future, and the stream has a huge volume of messages... you should place the shutdown future earlier in the `select!` list to ensure that it is always polled." A new connection accepted after shutdown spawns a task that will never be tracked or drained — violating structured concurrency.

**Fix**: Add `biased;` with shutdown branch listed first. Or use `tokio_util::CancellationToken` propagated into spawned tasks.

---

### D-CSAFE-005: EventBus consumer holds MutexGuard across `handle_event_bus_event(event).await` — long lock duration blocks all 35+ handlers

**File**: `nt_mind_background_loop/run.rs:896-916`
**Severity**: MEDIUM

```rust
tokio::select! {
    biased;
    result = event_rx.recv() => {
        match result {
            Ok(event) => {
                let mut handle = h.lock().await;
                handle.handle_event_bus_event(event).await;  // <-- lock held during I/O
            }
            ...
        }
    }
    _ = rx.changed() => { break; }
}
```

If `handle_event_bus_event` performs I/O (KB writes, file operations), the MutexGuard is held for the entire I/O duration. During this time, ALL other 30+ handlers cannot acquire the lock. If shutdown fires while the event handler is mid-I/O, the lock is dropped but the I/O may be partially complete — leaving `BackgroundLoopHandle` in an inconsistent state.

**Reference**: Tokio Mutex docs: "If a task is cancelled while holding a lock, the lock is released, but other tasks that were waiting may see an inconsistent state." Oxide RFD 400: "cancellation blast radius of a future holding a mutex extends to other futures."

**Fix**: Extract event processing into a separate scope that drops the lock before any I/O, or use a channel-based event dispatch (actor pattern) where the handler does not hold the shared lock.

---

### D-CSAFE-006: DNS intercept `select!` lacks `biased;` — shutdown signal delayed by DNS query burst

**File**: `nt_shield_proxy_kernel/dns_intercept.rs:127`
**Severity**: LOW

```rust
tokio::select! {
    result = socket.recv_from(&mut buf) => { ... }
    _ = shutdown_rx.changed() => { ... }  // no biased; — non-deterministic
}
```

Without `biased;`, when a shutdown signal and a DNS query arrive simultaneously, the runtime picks non-deterministically. Under load, a burst of DNS queries may delay shutdown recognition by several poll cycles. This is inconsistent with the proxy kernel's `run_bidirectional` which uses `biased;` for the same shutdown pattern.

**Fix**: Add `biased;` with shutdown branch first. The proxy kernel already uses this pattern correctly — align DNS intercept for consistency.

---

### D-CSAFE-007: `read_exact` + `write_all` in SOCKS5 handshake not cancel-safe inside spawned tasks

**File**: `nt_shield_proxy_kernel/listener/socks5.rs:65-126`
**Severity**: LOW

The SOCKS5 handshake performs multiple `read_exact` and `write_all` calls:
```rust
stream.read_exact(&mut buf).await?;
stream.read_exact(&mut methods).await?;
stream.write_all(&[0x05, 0x00]).await?;
stream.read_exact(&mut request).await?;
```

While these are inside `tokio::spawn` (not directly in `select!`), the parent `start_with_shutdown` loop's `select!` can cause the spawned task to be aborted on shutdown. Tokio docs explicitly list `read_exact` and `write_all` as **not cancellation safe**: "partially filled buffer is lost."

**Reference**: leonardomso/rust-skills: "Non-cancel-safe: `read_exact` owns an internal buffer inside the future. If select! drops this branch, the partially-read bytes are gone."

**Fix**: The SOCKS5 handler is spawned via `tokio::spawn` so it runs independently — this is actually correct if the task is allowed to complete. But if the runtime shuts down (e.g., `process::exit`), these tasks are killed mid-handshake. Document that SOCKS5 handlers are best-effort and may leave partial state.

---

### D-CSAFE-008: `tokio::sync::Mutex::acquire` inside `select!` tick branches loses queue position on cancellation

**File**: `nt_mind_background_loop/run.rs:745`
**Severity**: MEDIUM

The `h.lock().await` inside `spawn_handler!` uses `tokio::sync::Mutex` (as confirmed by the `Arc<Mutex<BackgroundLoopHandle>>` pattern). Tokio docs list `Mutex::lock` as **not cancellation safe**: "uses a queue for fairness; cancellation makes you lose your place in the queue."

When the shutdown branch fires while a handler is waiting to acquire the lock, the future is dropped and the handler loses its queue position. If multiple handlers are contending for the lock, the FIFO ordering breaks down — handlers that were waiting longer may end up behind newly-arrived handlers.

**Reference**: Tokio docs, Stack Overflow #74547855, Rust Users Forum #92381.

**Fix**: Use `std::sync::Mutex` if locks are not held across `.await` points (the recommended pattern). If async lock is needed, document the fairness loss and consider a `Semaphore`-based admission control.

---

### D-CSAFE-009: No `biased;` in non-biased `select!` for proxy listeners creates unfair shutdown under high connection throughput

**File**: `nt_shield_proxy_kernel/listener/socks5.rs:40`, `listener/http.rs:41`, `dns_intercept.rs:127`
**Severity**: LOW

Three `select!` sites across NT-SHIELD lack `biased;`. Without it, Tokio uses random branch polling. Under high throughput (many simultaneous connections + shutdown signal), the accept/recv branch can dominate polling, starving the shutdown branch. The Tokio source code comment warns: "if the stream has a huge volume of messages and zero or nearly zero time between them, you should place the shutdown future earlier in the `select!` list."

**Fix**: Add `biased;` with shutdown as the first branch in all three locations. This is a simple consistency fix that prevents shutdown starvation.

---

## Summary Table

| ID | Defect | Severity | Location |
|----|--------|----------|----------|
| D-CSAFE-001 | Mutex acquired inside `select!` tick — lock drop on shutdown | HIGH | `run.rs:734-760` |
| D-CSAFE-002 | Shared 5s deadline across 35+ handlers — sequential starvation | HIGH | `handlers.rs:48-61` |
| D-CSAFE-003 | Bidirectional tunnel drops `io::copy` without flush/half-close | MEDIUM | `kernel.rs:544-566` |
| D-CSAFE-004 | SOCKS5/HTTP listeners lack `biased;` — connections after shutdown | MEDIUM | `socks5.rs:40`, `http.rs:41` |
| D-CSAFE-005 | EventBus holds MutexGuard across async I/O — blocks all handlers | MEDIUM | `run.rs:896-916` |
| D-CSAFE-006 | DNS intercept lacks `biased;` — shutdown delayed | LOW | `dns_intercept.rs:127` |
| D-CSAFE-007 | `read_exact`/`write_all` in SOCKS5 not cancel-safe on task abort | LOW | `socks5.rs:65-126` |
| D-CSAFE-008 | `tokio::sync::Mutex` queue position lost on cancellation | MEDIUM | `run.rs:745` |
| D-CSAFE-009 | Three proxy `select!` sites lack `biased;` for consistent shutdown | LOW | `socks5.rs`, `http.rs`, `dns_intercept.rs` |

---

## Cross-References to Previous Batches

- Batch 873 agent3: Flagged `spawn_handler!` Mutex-in-select (D-CSAFE-001), EventBus consumer double-cancel-unsafe (D-CSAFE-005), `forward_traffic` non-biased select, inline sleep recreation, drain starvation cascade
- Batch 874 agent3: Flagged MutexGuard drop leaving BackgroundLoopHandle inconsistent (D-CSAFE-001), EventBus consumer lock duration (D-CSAFE-005), proxy signal handler `process::exit(0)` bypassing destructors
- Batch 874 agent4: Flagged sequential drain starvation (D-CSAFE-002), `select!` in handler loops unstructured cancellation
- Batch 875 agent3: Flagged `io::copy` bidirectional not cancel-safe on shutdown (D-CSAFE-003)
- Batch 877 agent3: Flagged `select!` with RwLock shadow waiters deadlock potential, drain sequential select per handler
- Batch 877 agent4: Flagged shared deadline across handlers (D-CSAFE-002), sequential drain with 5s budget
