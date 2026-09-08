# Iteration Batch 880 — Agent 3: Rust Cancel-Safe Futures & `select!` Patterns

## Deep Research Summary

### Core Concepts

**Cancellation Safety Definition** (Tokio): A future is cancel-safe if dropping it before completion and recreating it produces correct results — no data loss or invariant violation. Cancellation happens at any `.await` point when a future is dropped.

**Cancel Correctness** (Oxide RFD 400): A *global* property — the system behaves correctly even when futures are cancelled. Violated when: (1) cancel-unsafe future exists, (2) it is cancelled, (3) cancellation breaks a system invariant.

**Key Source of Bugs**: `tokio::select!` drops all losing branches when one wins. If a losing branch had partial state (buffered reads, in-progress writes, held locks), that state is silently destroyed.

---

## `select!` Cancellation Safety Reference

### Cancellation-Safe (safe to use in `select!` loops)
| Operation | Why Safe |
|-----------|----------|
| `mpsc::Receiver::recv()` | Message stays in channel if cancelled |
| `broadcast::Receiver::recv()` | Position tracked in receiver |
| `watch::Receiver::changed()` | No data consumed on cancel |
| `TcpListener::accept()` | Connections stay in OS queue |
| `AsyncReadExt::read()` | Partial reads surfaced to caller |
| `StreamExt::next()` | Stream continues on next poll |
| `tokio::time::sleep()` | Timer resets cleanly |

### NOT Cancellation-Safe (data loss on cancel)
| Operation | Why Unsafe |
|-----------|-----------|
| `AsyncReadExt::read_exact()` | Partially filled buffer lost |
| `AsyncReadExt::read_to_end()` | Accumulation inside future |
| `AsyncReadExt::read_to_string()` | Partial data lost |
| `AsyncWriteExt::write_all()` | Partial write state lost |
| `mpsc::Sender::send()` | Value dropped if not sent |
| `Mutex::lock()` | Lose place in fairness queue |
| `RwLock::read/write()` | Lose place in fairness queue |
| `Semaphore::acquire()` | Lose place in fairness queue |

### Common Mitigation Patterns

1. **Resume, don't recreate**: Pin futures outside `select!` loop, poll `&mut` reference
2. **Reserve-permit pattern**: Use `sender.reserve()` (cancel-safe) instead of `sender.send()` (unsafe)
3. **Spawn for cancel-unsafe**: `tokio::spawn` detaches task from `select!` cancellation
4. **readable() + try_read()**: Use `readable()` (safe) then do blocking read outside `select!`
5. **Cooperative cancellation**: `cancel_safe_futures::coop_cancel` for explicit cancellation channels
6. **Stream merging**: `tokio_stream::StreamExt::merge()` inherently handles cancellation safety

---

## Defects Identified in NeoTrix

### Defect 1: `proxy_kernel/kernel.rs` — `shutdown_tx.send()` Inside `select!` Branch (Lines 260-265)

```rust
tokio::select! {
    Some(err_msg) = error_rx.recv() => {
        let mut state = kernel.state.write().await;
        *state = KernelState::Failed(err_msg);
        let _ = kernel.shutdown_tx.send(true);  // ← UNSAFE: send() is not cancel-safe
        break;
    }
    _ = &mut socks5_handle => { ... }
    _ = &mut http_handle => { ... }
}
```

**Impact**: `shutdown_tx.send(true)` is a `watch::Sender::send` which IS cancel-safe. However, the `.write().await` on `KernelState` is a `RwLock::write()` which is NOT cancel-safe — if the `socks5_handle` or `http_handle` branch wins between the lock acquisition and the state update, the lock is dropped without updating state, leaving the kernel in an inconsistent `Running` state while a listener has already exited.

**Fix**: Move state update outside the select branch, or use `std::sync::Mutex` for short critical sections.

---

### Defect 2: `proxy_kernel/kernel.rs` — `RwLock::write().await` in GC Task (Lines 224-234)

```rust
tokio::select! {
    _ = tokio::time::sleep(Duration::from_secs(60)) => {
        security_for_gc.cleanup_stale_failures().await;
        kernel_for_gc.cleanup_stale_clients().await;
    }
    _ = shutdown_rx_gc.changed() => {
        if *shutdown_rx_gc.borrow() { break; }
    }
}
```

**Impact**: If `cleanup_stale_failures()` or `cleanup_stale_clients()` internally acquires a `tokio::sync::Mutex` or `RwLock`, and the shutdown branch fires during the await, the lock is dropped without completing cleanup. This can leave stale failure records or client connections in an inconsistent state.

**Fix**: Use `std::sync::Mutex` for short critical sections, or restructure to ensure cleanup completes atomically before checking shutdown.

---

### Defect 3: `background_loop/run.rs` — Replaced Single `select!` with Spawns (Line 381)

```rust
/// Replaces single `tokio::select!` which blocked when any handler was slow.
pub async fn start(&mut self) {
    // Spawns all background handlers as independent tokio tasks.
```

**Impact**: The comment acknowledges the original design flaw — a single `select!` loop where one slow handler blocks all others. The fix (independent spawns) is correct, but creates a new concern: `tokio::spawn` tasks are cancel-unsafe if `JoinHandle::abort()` is called during a non-cancel-safe operation. The codebase uses `abort_handle()` in `handlers.rs:56` which can abort tasks mid-operation.

**Fix**: Use cooperative cancellation (`CancellationToken`) instead of `abort()`, or ensure all spawned handlers are themselves cancel-safe.

---

### Defect 4: `background_loop/handlers.rs` — `abort()` on `JoinHandle` in Shutdown (Lines 51-61)

```rust
for handle in self.handles.drain(..) {
    let abort = handle.abort_handle();
    tokio::select! {
        biased;
        _ = &mut deadline => {
            abort.abort();  // ← CAN CANCEL mid-write/mid-KB-operation
            log::warn!("[bg] shutdown deadline reached — aborting task");
        }
        _ = handle => {}
    }
}
```

**Impact**: `abort()` cancels the task at the next `.await` point. If the task is in the middle of a KB write (`nt_memory_kb`), EventBus broadcast, or experience absorption, the operation is cancelled mid-flight. Per Oxide RFD 400: "aborting Tokio tasks is problematic because a task could potentially be cancelled in the middle of a cancel-unsafe operation." This can corrupt KB state, lose experience data, or leave EventBus in an inconsistent state.

**Fix**: 
1. Use cooperative cancellation via `CancellationToken` — handlers check token and exit gracefully
2. Or: Ensure all handler loops are cancel-safe by design (no multi-step state mutations across `.await` points)

---

### Defect 5: `background_loop/handlers.rs` — Deadline + Abort Race Condition (Lines 48-61)

```rust
let deadline = tokio::time::sleep(Duration::from_secs(5));
tokio::pin!(deadline);

for handle in self.handles.drain(..) {
    let abort = handle.abort_handle();
    tokio::select! {
        biased;
        _ = &mut deadline => {
            abort.abort();
        }
        _ = handle => {}
    }
}
```

**Impact**: The `biased;` ensures deadline is checked first, but there's a TOCTOU race: between checking the deadline and polling the handle, the task could complete normally. The `abort()` call is idempotent per Tokio docs, but if the task completed and the handle is consumed, the abort is a no-op. However, the real issue is that the `deadline` pin is shared across all handle iterations — after the first abort, the deadline is already elapsed, so ALL subsequent tasks are immediately aborted regardless of their progress.

**Fix**: Reset or create a new deadline for each handle, or use `tokio::time::timeout` per-task.

---

### Defect 6: `proxy_kernel/kernel.rs` — `state.write().await` in Select Loop (Line 239)

```rust
{
    let mut state = kernel.state.write().await;
    *state = KernelState::Running;
}
```

**Impact**: This `RwLock::write()` is called BEFORE the select loop, so it's not directly inside `select!`. However, the pattern of holding a `tokio::sync::RwLock` guard across `.await` points (which happens in the GC task at line 227-228) is explicitly flagged as problematic in Oxide RFD 400: "if a future that is currently holding on to a mutex is cancelled, the state guarded by the mutex is likely invalid."

**Fix**: Use `std::sync::Mutex` for short critical sections that don't span `.await` points, or restructure to avoid holding locks across awaits.

---

### Defect 7: `background_loop/run.rs` — EventBus Sync Handler Without Cancellation Protection (Lines 404-412)

```rust
event_bus.register_sync_handler(move |event: &CoreEvent| {
    let kind = format!("{:?}", event);
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let key = format!("ev_{}", ts);
    let _ = kb_clone.field_stage("field_events", &key, &kind, "event_bus");
});
```

**Impact**: The sync handler performs KB writes (`field_stage`) inside the EventBus broadcast path. If the EventBus broadcast is cancelled (e.g., by task abort during shutdown), the KB write may be partially committed. Since this is a sync handler (not async), it doesn't have `.await` points, but the caller (broadcast path) could be cancelled.

**Fix**: Ensure KB writes are idempotent, or use a two-phase commit pattern where the sync handler only stages data and a separate async task commits it.

---

## Recommendations for NeoTrix

1. **Audit all `select!` loops** for cancel-unsafe operations (11 `tokio::select!` sites identified)
2. **Replace `abort()` with cooperative cancellation** — use `CancellationToken` from `tokio_util`
3. **Replace `tokio::sync::Mutex/RwLock` with `std::sync::Mutex`** where locks don't span `.await`
4. **Add `cancel-safe-futures` crate** for `then_try` adapters and `RobustMutex`
5. **Document cancel safety** on all async public APIs (per Oxide recommendation)
6. **Test with Poll::Pending paths** — use tiny duplex buffers to force cancellation races
7. **Consider `StreamExt::merge()`** as alternative to `loop { select! { ... } }` patterns

## References

- Tokio `select!` docs: https://docs.rs/tokio/latest/tokio/macro.select.html
- Oxide RFD 400: https://rfd.shared.oxide.computer/rfd/0400
- `cancel-safe-futures` crate: https://docs.rs/cancel-safe-futures/latest/cancel_safe_futures/
- Rust async book cancellation: https://rust-lang.github.io/async-book/part-reference/cancellation.html
- Comprehensive Rust cancellation: https://google.github.io/comprehensive-rust/concurrency/async-pitfalls/cancellation.html
