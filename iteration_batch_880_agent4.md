# Iteration Batch 880 — Agent 4: Rust Structured Concurrency (JoinSet / TaskTracker)

## 1. Executive Summary

Rust's async concurrency model is **unstructured by default**: `tokio::spawn` creates detached tasks that outlive their parent scope, causing zombie tasks, resource leaks, and silent panics. Two key primitives — **`JoinSet`** (tokio 1.21+) and **`TaskTracker`** (tokio-util 0.7) — solve this by enforcing lifecycle ownership. NeoTrix currently uses **zero** instances of either; all 50+ `tokio::spawn` call sites are unstructured fire-and-forget patterns. This is a critical structural defect.

---

## 2. Core Concepts

### 2.1 Structured Concurrency

> "Every task you spawn must be awaited before its parent scope exits."

Structured concurrency ties every spawned task to a parent scope. When the parent ends, all children are cancelled. No zombies, no leaks. The code structure matches the lifecycle of the work.

**Rust does NOT enforce this at the language level.** The compiler won't stop you from spawning a detached task. You must choose structured primitives explicitly.

### 2.2 `JoinSet<T>` — Tokio's Structured Primitive

| Property | Detail |
|---|---|
| **Location** | `tokio::task::JoinSet` (requires `rt` feature) |
| **Lifecycle** | Tasks are **immediately aborted** on drop |
| **Ordering** | Results returned in **completion order** (NOT spawn order) |
| **Homogeneity** | All tasks must return the same type `T` |
| **Cancellation** | `join_next()` is **cancel safe** (safe in `select!`) |
| **Memory** | Retains return values until `join_next()` is called — **unbounded accumulation if not polled** |

**Key API:**
```rust
let mut set = JoinSet::new();
set.spawn(async { /* work */ });          // returns AbortHandle
set.spawn_blocking(|| { /* blocking */ }); // blocking tasks in same set
set.join_next().await;                    // next completed task
set.join_all().await;                     // all results (panics on JoinError)
set.abort_all();                          // cancel everything
set.shutdown();                           // abort_all + wait for shutdown
```

### 2.3 `TaskTracker` — Lightweight Alternative

| Property | Detail |
|---|---|
| **Location** | `tokio_util::task::TaskTracker` |
| **Memory** | Tasks are **immediately freed** when they exit (no return value retention) |
| **Cloneable** | Can be shared across many tasks (`Arc`-like) |
| **No mutable insert** | `tracker.spawn()` takes `&self` |
| **No abort on drop** | **Critical difference**: dropping a `TaskTracker` does NOT cancel tasks |
| **Wait semantics** | `tracker.close()` then `tracker.wait().await` — `wait` blocks until all tracked tasks exit |

**When to use TaskTracker over JoinSet:**
1. Fire-and-forget tasks where return values don't matter
2. Long-lived background tasks where memory accumulation would cause OOM
3. Need to share tracker across threads/tasks (cloneable)
4. Need `wait()` to block even if tracker is temporarily empty (close semantics)

### 2.4 JoinSet vs TaskTracker Decision Matrix

| Criteria | `JoinSet` | `TaskTracker` |
|---|---|---|
| Need return values | ✅ | ❌ |
| Auto-cancel on drop | ✅ (abort) | ❌ (tasks continue) |
| Memory accumulation risk | ⚠️ if not polled | ✅ freed immediately |
| Cloneable | ❌ | ✅ |
| `&self` insert | ❌ (`&mut self`) | ✅ |
| Mixing async + blocking | ✅ (`spawn_blocking`) | ❌ (only `spawn`) |
| Structured lifecycle | ✅ | ⚠️ (must `close()` + `wait()`) |

---

## 3. Defect Analysis for NeoTrix

### DEFECT-1: Zero Structured Concurrency Adoption (Severity: CRITICAL)

**Evidence:** Grep across `neotrix-core/src/` finds **0 uses** of `JoinSet` or `TaskTracker`. All 50+ concurrent spawn sites use raw `tokio::spawn`:

```
neotrix-core/src/neotrix/nt_file_ability/batch_processor.rs:227
neotrix-core/src/neotrix/nt_core_event_bus.rs:362
neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/executor.rs:37
neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738,894
neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:176,190,203,223
... (50+ total)
```

**Impact:**
- **Zombie tasks**: Background loop handlers (`run.rs:738`) push `JoinHandle` into `self.handles` but if the parent `BackgroundLoop` is dropped without draining, orphaned tasks continue consuming resources
- **No structured cleanup**: `kernel.rs` spawns SOCKS5, HTTP proxy, DNS interceptor, and GC loop as 4 separate detached tasks — no parent scope ensures they all stop together
- **Silent panics**: `tokio::spawn` panics in child tasks are silently swallowed if the `JoinHandle` is dropped without `.await`

**Fix:** Audit all 50+ `tokio::spawn` sites. Group related tasks into `JoinSet` or `TaskTracker`. At minimum, the background loop and proxy kernel must use structured concurrency.

---

### DEFECT-2: Background Loop Handle Accumulation (Severity: HIGH)

**File:** `nt_mind_background_loop/run.rs:738`

```rust
self.handles.push(tokio::spawn(async move {
    // ... handler loop
}));
```

The `spawn_handler!` macro pushes 15+ `JoinHandle`s into a `Vec<JoinHandle>`. These handles are **never drained** in normal operation — they accumulate indefinitely. If `self.handles` is not explicitly awaited before shutdown:

1. Completed task return values are retained in the handle (memory)
2. The `JoinHandle` itself holds a reference count to the runtime task
3. On process shutdown, the runtime drops all tasks — but without structured cleanup, in-flight operations may be corrupted

**Fix:** Replace `Vec<JoinHandle>` + `spawn_handler!` with a `JoinSet<()>`:
```rust
let mut handler_set = JoinSet::new();
// In macro:
handler_set.spawn(async move { /* ... */ });
// On shutdown:
handler_set.shutdown().await;
```

---

### DEFECT-3: JoinSet Memory Accumulation — Return Value Retention (Severity: MEDIUM)

**Mechanism:** `JoinSet` retains the `Ok(T)` return value of every completed task until `join_next()` is called. If a producer spawns tasks faster than the consumer polls:

```rust
// DANGER: if join_next() is never called, completed results accumulate
let mut set = JoinSet::new();
for item in infinite_stream {
    set.spawn(process(item)); // return values pile up
}
```

**NeoTrix exposure:** `nt_core_parallel/executor.rs` spawns tasks in a loop and awaits each immediately (safe), but the pattern is fragile — any refactor that delays polling could cause OOM.

**Fix for fire-and-forget patterns:** Use `TaskTracker` instead, which immediately frees completed tasks:
```rust
let tracker = TaskTracker::new();
for item in work_items {
    tracker.spawn(process(item));
}
tracker.close();
tracker.wait().await;
```

---

### DEFECT-4: Cooperative Cancellation Gaps — CPU-Bound Tasks (Severity: HIGH)

**Problem:** When a `JoinSet` is dropped, tasks are cancelled at the **next await point**. CPU-bound loops without `.await` cannot be cancelled:

```rust
set.spawn(async {
    // This loop has no await — CANNOT be cancelled
    for i in 0..1_000_000 {
        expensive_computation(i); // runs to completion even after drop
    }
});
```

**NeoTrix exposure:** `nt_core_parallel/executor.rs:37-39` spawns tasks with `tokio::time::sleep` (cancellation point), but the actual computation pattern is unclear. The `batch_processor.rs:227` and `proxy_pool.rs:616` patterns are more concerning — batch operations may contain CPU-intensive work.

**Fix:**
1. Insert `tokio::task::yield_now().await` in tight loops
2. Use `tokio::task::spawn_blocking` for CPU-bound work
3. Add `CancellationToken` for explicit shutdown signaling
4. Audit all spawn sites for non-cancellable loops

---

### DEFECT-5: TaskTracker Drop Does NOT Cancel Tasks (Severity: MEDIUM)

**Critical gotcha:** Unlike `JoinSet`, dropping a `TaskTracker` does **not** abort tracked tasks. Tasks continue running in the background.

```rust
{
    let tracker = TaskTracker::new();
    tracker.spawn(async { /* long work */ });
    // tracker dropped here — but task CONTINUES RUNNING
}
// task is still alive!
```

**NeoTrix risk:** If any future adoption of `TaskTracker` follows the `JoinSet` mental model (drop = cancel), tasks will silently leak.

**Fix:** Always call `tracker.close(); tracker.wait().await;` explicitly. Never rely on drop for cancellation with `TaskTracker`.

---

### DEFECT-6: Completion Order Non-Determinism (Severity: MEDIUM)

**Problem:** `JoinSet::join_next()` returns results in **completion order**, not spawn order. If ordering matters (e.g., processing pipeline stages), this is a footgun.

**NeoTrix exposure:** `nt_core_parallel/executor.rs:34-44` spawns tasks and collects results sequentially — but with `JoinSet`, the order would be lost. Currently the code awaits each handle in spawn order (blocking), which is correct but defeats concurrency.

**Fix for ordered results:**
1. Use `join_all()` for all-at-once ordered collection
2. Use `VecDeque<JoinHandle>` + sequential await for strict FIFO ordering
3. Use `TaskTracker` + separate channel for ordered result collection
4. Consider `JoinDeque` (proposed in tokio-rs/tokio#7576, not yet merged)

---

### DEFECT-7: `join_all()` Panics on JoinError (Severity: MEDIUM)

**Problem:** `JoinSet::join_all()` panics if any task returns a `JoinError` (panic or abort). This is hostile to production error handling.

**NeoTrix risk:** The background loop has 15+ handlers — if one panics, `join_all()` would crash the entire loop, taking down all other healthy handlers.

**Fix:** Use `join_next()` in a loop with explicit error handling:
```rust
while let Some(result) = set.join_next().await {
    match result {
        Ok(()) => { /* task completed */ }
        Err(e) if e.is_cancelled() => { /* expected during shutdown */ }
        Err(e) => { log::error!("Handler panicked: {:?}", e); }
    }
}
```

---

## 4. Recommended Migration Plan

### Phase 1: Background Loop (highest impact)
- Replace `Vec<JoinHandle>` in `run.rs` with `JoinSet<()>`
- Replace `spawn_handler!` macro to use `handler_set.spawn()`
- Add `handler_set.shutdown().await` on shutdown path

### Phase 2: Proxy Kernel
- Wrap SOCKS5 + HTTP + DNS + GC tasks in `JoinSet<()>`
- Ensure `kernel.start()` awaits all child tasks before returning

### Phase 3: Batch Operations
- Convert `batch_processor.rs` and `proxy_pool.rs` batch spawns to `TaskTracker` (fire-and-forget with memory safety)
- Add `CancellationToken` for graceful batch cancellation

### Phase 4: All Remaining Spawn Sites
- Audit remaining 40+ `tokio::spawn` sites
- Classify: structured (JoinSet), fire-and-forget (TaskTracker), or truly detached (keep tokio::spawn)
- Document decision for each site

---

## 5. Key References

| Source | URL | Key Insight |
|---|---|---|
| Tokio JoinSet docs | docs.rs/tokio/latest/tokio/task/struct.JoinSet.html | Drop aborts all tasks; completion order; cancel-safe join_next |
| TaskTracker docs | docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTracker.html | Memory-efficient; no abort on drop; cloneable |
| Tokio #6664 | github.com/tokio-rs/tokio/issues/6664 | Request for join_all/try_join_all on JoinSet |
| Tokio #7576 | github.com/tokio-rs/tokio/issues/7576 | JoinDeque proposal for FIFO-ordered completion |
| Tokio #7222 | github.com/tokio-rs/tokio/issues/7222 | TaskTracker drop ≠ abort documentation gap |
| Sunshowers (RustConf 2025) | sunshowers.io/posts/cancelling-async-rust/ | Comprehensive cancellation safety analysis |
| Oxide RFD 400 | rfd.shared.oxide.computer/rfd/400 | Cancel safety patterns in production async Rust |
| Microsoft RustTraining | github.com/microsoft/RustTraining | Structured concurrency with JoinSet + TaskTracker patterns |
| RustFAQ | rustfaq.org/en/how-to-implement-structured-concurrency-in-rust/ | Practical guide: zombie tasks, JoinSet, cancellation |

---

## 6. Summary of Defects

| # | Defect | Severity | NeoTrix Exposure |
|---|---|---|---|
| 1 | Zero structured concurrency adoption | CRITICAL | 50+ unstructured spawn sites |
| 2 | Background loop handle accumulation | HIGH | `run.rs` Vec<JoinHandle> never drained |
| 3 | JoinSet return value memory accumulation | MEDIUM | Fragile polling patterns |
| 4 | CPU-bound tasks cannot be cancelled | HIGH | batch_processor, proxy_pool |
| 5 | TaskTracker drop ≠ cancel | MEDIUM | Future adoption footgun |
| 6 | Completion order non-determinism | MEDIUM | parallel executor ordering |
| 7 | join_all() panics on JoinError | MEDIUM | background loop crash risk |
