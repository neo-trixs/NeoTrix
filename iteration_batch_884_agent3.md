# Iteration Batch 884 — Agent 3: Rust Cancel-Safe Futures & select! Patterns

**Research Depth**: Deep (2024-2026 sources, Oxide RFD 609, tokio docs, Cliffle blog, 13+ select! sites audited)
**NeoTrix Files Audited**: 8 files, 13 `select!` sites across NT-SHIELD, NT-MIND

---

## 1. Research Summary: Cancel Safety in Rust Futures

### Core Concept

A future is **cancel-safe** if dropping it before completion is equivalent to never having called it. `tokio::select!` drops every branch that didn't win the race — if any dropped future owned state, that state is lost.

**Key insight** (from Oxide RFD 609, 2025-11): Cancel safety is a *local* property of a future; cancel *correctness* is a *global* property of a system. A system can have individually cancel-safe futures that compose into a cancel-incorrect system.

### The Three Killers

| Killer | Description | Difficulty |
|--------|-------------|------------|
| **Silent Data Loss** | Future cancelled mid-operation, no error returned, DB write never committed | Invisible — no panic, no log |
| **FutureLock** (Oxide, 2025) | Future A holds a Mutex; select! switches to Future B which needs same Mutex; Task can't poll A to release it — permanent deadlock | Requires cross-future dependency analysis |
| **Timer Reset** | `sleep()` inside `loop { select! {} }` without `pin!` — each cancelled sleep recreates a fresh timer, resetting the countdown | Subtle timing bug |

### Cancel-Safe vs Unsafe Reference (from tokio docs)

| Safe | Unsafe |
|------|--------|
| `mpsc::Receiver::recv()` | `Mutex::lock()` / `RwLock::read()/write()` |
| `broadcast::Receiver::recv()` | `Semaphore::acquire()` |
| `watch::Receiver::changed()` | `AsyncReadExt::read_to_end()` / `read_to_string()` |
| `TcpListener::accept()` | `AsyncWriteExt::write_all()` |
| `signal::recv()` | `BufReader::read_line()` |
| `AsyncReadExt::read()` / `read_buf()` | `mpsc::Sender::send()` (use `reserve()` instead) |
| `StreamExt::next()` | |

### The `reserve` Pattern (Oxide RFD 400)

Instead of `sender.send(value)` (not cancel-safe — value lost if cancelled), use:
```rust
let permit = sender.reserve().await?;  // cancel-safe: no ownership transferred yet
permit.send(value);                     // synchronous: no await point, cannot be cancelled
```

### FutureLock (Oxide RFD 609, 2025-11)

```rust
tokio::select! {
    _ = &mut future1 => { /* future1 holds a Mutex */ }
    _ = sleep(500ms) => {
        // This branch needs the SAME Mutex → deadlock
        // future1 is not polled (cancelled), can't release Mutex
        // future2 waits for Mutex forever
    }
}
```

**Mitigation**: Use `tokio::spawn()` to isolate futures — JoinHandles don't hold shared resources and can't cause FutureLock.

---

## 2. Defects Extracted for NeoTrix

### Defect 1: Timer Reset — `sleep()` Without `pin!` in Loops

**Files**: `kernel.rs:399-408`, `kernel.rs:416-424`
**Severity**: Medium (logic bug, not data loss)
**Category**: Timer Reset Anti-Pattern

**Code**:
```rust
// kernel.rs:399 — failure_counts GC loop
loop {
    tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
            security_for_gc.cleanup_stale_failures().await;
            kernel_for_gc.cleanup_stale_clients().await;
        }
        _ = shutdown_rx_gc.changed() => {
            if *shutdown_rx_gc.borrow() { break; }
        }
    }
}
```

**Problem**: `tokio::time::sleep(60s)` creates a new future each loop iteration. When `shutdown_rx_gc.changed()` wins the select!, the sleep future is dropped. On the next loop iteration, a **new** `sleep(60s)` starts from zero. If the GC branch keeps losing (e.g., rapid shutdown signals or busy event loop), the GC never runs.

**Impact**: Stale failure counts and client connections accumulate if the GC loop is starved. Not a data loss bug, but a resource leak.

**Fix**:
```rust
let mut gc_timer = tokio::time::interval(std::time::Duration::from_secs(60));
loop {
    tokio::select! {
        _ = gc_timer.tick() => { /* GC work */ }
        _ = shutdown_rx_gc.changed() => { break; }
    }
}
```
`interval.tick()` is cancel-safe and preserves timing across cancellations.

**Same defect at**: `kernel.rs:416-424` (connection pool cleanup loop)

---

### Defect 2: Mutex::lock() Inside select! Branch — FutureLock Risk

**File**: `dns_intercept.rs:127-154`
**Severity**: High (potential deadlock under load)
**Category**: FutureLock / Vacuously Cancel-Safe

**Code**:
```rust
loop {
    tokio::select! {
        result = socket.recv_from(&mut buf) => {
            match result {
                Ok((len, peer)) => {
                    {
                        let mut limiter = rate_limiter.lock().await;  // ← Mutex lock in select branch
                        if !limiter.check(peer.ip()) { continue; }
                    }
                    // ... async DNS processing follows
                }
            }
        }
        _ = shutdown_rx.changed() => { break; }
    }
}
```

**Problem**: While the `recv_from` future itself is cancel-safe, the handler body acquires a `Mutex::lock().await`. If a futureock scenario arises (e.g., another task holds the same Mutex and is blocked on something this task owns), the shutdown branch could win while the lock is pending, creating a deadlock window. The immediate code is safe (lock is held briefly, no `.await` under the lock), but the pattern is a **FutureLock generator** — any future modification adding an `.await` under the lock guard silently introduces a deadlock.

**Impact**: Latent deadlock risk. Currently safe by accident (lock held only for synchronous `check()`), but the pattern invites FutureLock on any code evolution.

**Fix**: Move Mutex acquisition to a `tokio::spawn()` or use a lock-free rate limiter (e.g., `dashmap` or atomic counters). Alternatively, follow lilos pattern: `perform(|limiter| limiter.check(peer.ip()))` — sync closure, no `.await` possible.

---

### Defect 3: Bidirectional `io::copy` in select! — Silent Half-Connection Drop

**File**: `kernel.rs:793-816`
**Severity**: High (data loss on proxy connections)
**Category**: Cancel-Unsafe I/O Composition

**Code**:
```rust
tokio::select! {
    result = tokio::io::copy(&mut client_read, &mut upstream_write) => {
        client_to_upstream = result.unwrap_or(0);
        upstream_to_client = 0;
    }
    result = tokio::io::copy(&mut upstream_read, &mut client_write) => {
        upstream_to_client = result.unwrap_or(0);
        client_to_upstream = 0;
    }
    _ = shutdown_rx.changed() => {
        client_to_upstream = 0;
        upstream_to_client = 0;
    }
}
```

**Problem**: `tokio::io::copy` is cancel-safe in isolation, but the **composition** here is not. When one direction completes (e.g., client closes send), the other direction is cancelled mid-transfer. On shutdown, both directions are cancelled simultaneously — any data buffered in kernel TCP buffers is lost without error.

**Impact**: Users on NT-SHIELD proxy connections experience silent data truncation when one side closes. The `unwrap_or(0)` silently masks partial transfer counts. On shutdown, in-flight data is dropped with no drain period.

**Fix**: Spawn each direction as an independent task:
```rust
let h1 = tokio::spawn(async { tokio::io::copy(&mut client_read, &mut upstream_write).await });
let h2 = tokio::spawn(async { tokio::io::copy(&mut upstream_read, &mut client_write).await });
tokio::select! {
    _ = h1 => { h2.abort(); }
    _ = h2 => { h1.abort(); }
    _ = shutdown_rx.changed() => { h1.abort(); h2.abort(); }
}
```
Or use `tokio::io::copy_bidirectional()` which handles this correctly.

---

### Defect 4: `biased;` Select Masking Starvation in Handler Loops

**File**: `run.rs:742-757` (spawn_handler! macro)
**Severity**: Medium (handler starvation under load)
**Category**: Priority Inversion / Fairness

**Code**:
```rust
tokio::select! {
    biased;
    _ = ticker.tick() => {
        let mut $lock = h.lock().await;
        // ... handler body
    }
    _ = rx.changed() => {
        break;
    }
}
```

**Problem**: `biased;` polls branches top-to-bottom deterministically. The `ticker.tick()` branch is always polled first. This means:
1. If a handler is slow (holds the `h` Mutex for a long time), the shutdown signal is delayed until the handler finishes.
2. If multiple handlers share a resource and one is slow, others are starved because `biased` doesn't give the shutdown branch a fair chance.

**Impact**: Graceful shutdown can be delayed indefinitely if any handler hangs on its Mutex. The 5-second deadline in `handlers.rs:48-61` partially mitigates this, but the select! itself introduces a window where shutdown is invisible.

**Fix**: For shutdown branches, consider using `tokio::sync::CancellationToken` with `token.cancelled()` which is cancel-safe and can be checked independently. Or reorder: put shutdown first in biased select.

---

### Defect 5: EventBus Consumer Holding Mutex Across Handler Execution

**File**: `run.rs:896-914`
**Severity**: Medium (event processing stall)
**Category**: Lock Contention in select! Loop

**Code**:
```rust
tokio::select! {
    biased;
    result = event_rx.recv() => {
        match result {
            Ok(event) => {
                let mut handle = h.lock().await;       // ← acquires shared Mutex
                handle.handle_event_bus_event(event).await;  // ← .await while holding lock
            }
            // ...
        }
    }
    _ = rx.changed() => { break; }
}
```

**Problem**: The `handle` Mutex is held across `handle_event_bus_event(event).await`. If that handler is slow or blocks on I/O, the entire EventBus consumer is stalled — no other events can be processed. Combined with `biased;`, this creates a window where the shutdown branch is invisible while the handler runs.

**Impact**: EventBus event processing becomes single-threaded and blocking. Under load, events queue up (broadcast channel lag), and the `Lagged(n)` warning fires.

**Fix**: Spawn event handling as a separate task:
```rust
Ok(event) => {
    let h2 = h.clone();
    tokio::spawn(async move {
        let mut handle = h2.lock().await;
        handle.handle_event_bus_event(event).await;
    });
}
```

---

### Defect 6: Rate Limiter Mutex in DNS Select Loop — No Poison Protection

**File**: `dns_intercept.rs:135`
**Severity**: Low (currently safe, but no guard)
**Category**: Missing Poison Handling

**Code**:
```rust
let mut limiter = rate_limiter.lock().await;
if !limiter.check(peer.ip()) { continue; }
```

**Problem**: `tokio::sync::Mutex` does not implement poisoning (unlike `std::sync::Mutex`). If any code panics while holding this lock, the Mutex silently unlocks in an inconsistent state. The rate limiter's internal counters could be corrupted, allowing unlimited requests (bypass) or blocking all requests (DoS).

**Impact**: Low probability, but if a panic occurs inside the critical section, the rate limiter becomes unreliable — either allowing burst traffic or blocking all clients.

**Fix**: Use `std::panic::catch_unwind` around critical sections, or restructure to use atomic counters for rate limiting (no Mutex needed).

---

### Defect 7: Shutdown Deadline Loop Consuming Aborted Handles

**File**: `handlers.rs:51-61`
**Severity**: Low (resource leak on forced abort)
**Category**: Abort-Without-Cleanup

**Code**:
```rust
for handle in self.handles.drain(..) {
    let abort = handle.abort_handle();
    tokio::select! {
        biased;
        _ = &mut deadline => {
            abort.abort();
            log::warn!("[bg] shutdown deadline reached — aborting task");
        }
        _ = handle => {}
    }
}
```

**Problem**: When the deadline wins and `abort.abort()` is called, the JoinHandle is dropped without being `.await`ed. The task's Drop runs, but any `Drop` impl on resources held by the task may not complete cleanly. The `handle` variable (the JoinHandle) is consumed by the select! but its result is ignored on abort.

**Impact**: Resources held by aborted tasks (file handles, database connections, network sockets) may not be cleaned up properly. The `for` loop moves to the next handle immediately without waiting for the aborted task to actually stop.

**Fix**: After abort, add a brief `tokio::time::sleep(Duration::from_millis(100))` to allow Drop impls to run, or collect aborted handles and join them with a short timeout.

---

## 3. Recommendations Summary

| # | Defect | File | Fix Complexity | Priority |
|---|--------|------|---------------|----------|
| 1 | Timer reset (sleep without pin/interval) | kernel.rs:399,416 | Low | P2 |
| 2 | Mutex in select! branch (FutureLock risk) | dns_intercept.rs:135 | Medium | P1 |
| 3 | Bidirectional io::copy in select! | kernel.rs:793 | Medium | P1 |
| 4 | biased; masking shutdown starvation | run.rs:742 | Low | P2 |
| 5 | Mutex held across event handler .await | run.rs:896 | Low | P2 |
| 6 | Rate limiter Mutex no poison guard | dns_intercept.rs:135 | Low | P3 |
| 7 | Abort without cleanup wait | handlers.rs:51 | Low | P3 |

**Total Defects**: 7
**Critical (P1)**: 2 (FutureLock risk, data loss on proxy connections)
**Important (P2)**: 3 (timer reset, starvation, event stall)
**Low (P3)**: 2 (poison guard, abort cleanup)

---

## 4. Sources

- [Tokio select! docs](https://docs.rs/tokio/latest/tokio/macro.select.html) — cancel safety reference
- [Oxide RFD 609: Futurelock](https://rfd.shared.oxide.computer/rfd/0609) — FutureLock definition and mitigations (2025-11)
- [Oxide RFD 400: Cancel Safety](https://rfd.shared.oxide.computer/rfd/0400) — reserve pattern, cancel correctness
- [Cliffle: Cancel Safety in lilos](https://cliffle.com/blog/lilos-cancel-safety) — levels of cancel safety, vacuously cancel-safe
- [Tokio Issue #8175](https://github.com/tokio-rs/tokio/issues/8175) — inconsistent cancel safety docs
- [Rust Forum: FutureLock](https://users.rust-lang.org/t/an-approach-to-solving-futurelock/137257) — community discussion
- [Biriukov: Async Rust Gotcha](https://biriukov.dev/posts/async-rust-gocha-tokio-cancelation-select-future-then/) — backpressure + select! bugs
- [Medium: select! is a loaded gun](https://medium.com/@dhvani612/tokio-select-is-a-loaded-gun-...) — DB write silent drop
