# Batch 878 — Agent 4: Rust Structured Concurrency (JoinSet / TaskTracker)

**Topic**: Deep research on `tokio::task::JoinSet`, `tokio_util::task::TaskTracker`, structured concurrency patterns, and defect extraction for NeoTrix.

---

## 1. Research Summary

### 1.1 JoinSet (`tokio::task::JoinSet`)

**What it is**: A collection of tasks spawned on a Tokio runtime. The canonical structured concurrency primitive in Tokio. When dropped, all tasks are immediately aborted.

**Key API surface**:
- `spawn()` / `spawn_local()` / `spawn_blocking()` — add tasks
- `join_next()` — await one task completion (cancel-safe, usable in `select!`)
- `try_join_next()` — non-blocking poll for a completed task
- `join_all(self)` — await all, collect results; **panics on any `JoinError`**
- `abort_all()` — abort all tasks (does NOT remove from set; must `join_next` loop to drain)
- `detach_all()` — remove tasks without aborting (tasks continue in background)
- `shutdown()` — abort + drain loop, ignores panics
- `len()` / `is_empty()` / `contains()` — introspection

**Critical behavioral properties**:
1. **Drop = abort**: Dropping a `JoinSet` aborts all tracked tasks immediately. No grace period.
2. **Completed results accumulate**: If you never call `join_next()`, completed task return values pile up in memory until OOM.
3. **Not shareable**: `JoinSet` is `!Sync` and has no `clone()`. Cannot be shared across tasks without actor pattern or wrapping in `Arc<Mutex<>>`.
4. **Single waker**: Only the most recent `poll_join_next` caller gets woken. Earlier pollers are silently dropped.
5. **join_all panics**: `join_all()` panics on first `JoinError` (panic or abort), leaving remaining tasks abandoned.

### 1.2 TaskTracker (`tokio_util::task::TaskTracker`)

**What it is**: A lightweight task tracker from `tokio-util`. Unlike `JoinSet`, dropping a `TaskTracker` does NOT abort tasks. Designed for graceful shutdown with `CancellationToken`.

**Key differences from JoinSet**:
| Feature | JoinSet | TaskTracker |
|---------|---------|-------------|
| Drop behavior | Aborts all tasks | Does nothing to tasks |
| Return values | Stores all (memory grows) | Discards immediately (OOM-safe) |
| Sharing | Not shareable | `Clone` — can share across tasks |
| Result collection | `join_next()` | None — must use separate mechanism |
| Close semantics | N/A | `close()` + `wait()` — await both closed AND empty |
| Graceful shutdown | Manual `abort_all` + loop | Built-in `close()` + `wait()` |

**ABA hazard**: `TaskTracker::wait()` loses ABA resistance when used inside `tokio::select!` loops.

### 1.3 Structured Concurrency in Rust (2025-2026 state)

- Rust does NOT enforce structured concurrency at language level. The compiler won't stop you from spawning detached tasks.
- Tokio's `JoinSet` is the de facto standard, but has significant ergonomic gaps.
- `tokio-graceful-shutdown` crate (Finomnis) abstracts the boilerplate of error propagation + subsystem restart.
- Microsoft's async Rust training materials (2026) recommend `JoinSet` + `TaskTracker` + `CancellationToken` as the production triad.
- Key community insight: "Error propagation and error-based shutdown initiation are the real hard points that make boilerplate very complicated" (Finomnis, 2026-04-29).

---

## 2. Defects Extracted for NeoTrix

### DEFECT 1: Fire-and-Forget `tokio::spawn` Everywhere (No Structured Concurrency)

**Severity**: HIGH | **Category**: Memory / Lifecycle | **CW-**: SC-001

**Evidence**: 80+ instances of bare `tokio::spawn()` across NeoTrix codebase with no `JoinSet` or `TaskTracker`. Key offenders:

- `nt_mind_background_loop/run.rs:738` — `spawn_handler!` macro pushes to `Vec<JoinHandle>` but never drains completed handles
- `nt_shield_proxy_kernel/kernel.rs:176,190,203,223` — 4 separate `tokio::spawn` for proxy servers, DNS, cleanup; none tracked in a `JoinSet`
- `nt_io_provider/free_providers.rs:153,342,511,678` — 4 fire-and-forget spawns per provider
- `nt_shield_traffic/api_proxy.rs:353` — streaming response spawn with no lifecycle tracking
- `nt_core_event_bus.rs:362` — event handler spawn with no tracking

**Impact**: 
1. **Memory leak**: `JoinHandle`s in `Vec` accumulate completed task metadata. The `spawn_handler!` macro at `run.rs:738` pushes to `self.handles: Vec<JoinHandle<()>>` but the drain loop at shutdown (`handlers.rs:51`) only runs once — between shutdowns, completed handles sit in memory.
2. **No panic propagation**: If a spawned task panics, the panic is silently swallowed. No `JoinError` inspection.
3. **No graceful shutdown**: Proxies, listeners, and background tasks are aborted abruptly on process exit. No drain period for in-flight requests.

**Fix**: Replace `Vec<JoinHandle>` with `JoinSet<()>` in `BackgroundLoop`. Replace bare `tokio::spawn` in proxy kernel with `TaskTracker` + `CancellationToken`. Add `join_next()` draining in the main tick loop or use `try_join_next()` periodically.

---

### DEFECT 2: `spawn_handler!` Macro Never Drains Completed Handles

**Severity**: HIGH | **Category**: Memory Leak | **CW-**: SC-002

**Evidence**: `nt_mind_background_loop/run.rs:734-760`

```rust
macro_rules! spawn_handler {
    ($interval:expr, $name:literal, |$lock:ident| $body:expr) => {{
        let h = this.clone();
        let mut rx = shutdown_rx.clone();
        self.handles.push(tokio::spawn(async move { ... }));
    }};
}
```

30+ handlers are spawned and pushed to `self.handles`. The `Vec<JoinHandle<()>>` grows monotonically. Completed `JoinHandle`s are never removed between shutdowns. At `handlers.rs:51`, the drain only happens during `shutdown()`.

**Impact**: Each completed background handler retains its `JoinHandle` metadata (task ID, waker, join output slot). With 30+ handlers running every 60-3600s, this is a slow memory leak proportional to uptime.

**Fix**: Use `JoinSet<()>` and periodically drain with `try_join_next()`, or add a `drain_completed()` call in the tick loop.

---

### DEFECT 3: Proxy Kernel Spawns Are Unstructured (4 Independent Fire-and-Forget Tasks)

**Severity**: MEDIUM | **Category**: Lifecycle / Reliability | **CW-**: SC-003

**Evidence**: `nt_shield_proxy_kernel/kernel.rs:176-235`

```rust
let socks5_handle = tokio::spawn(async move { ... });  // line 176
let http_handle = tokio::spawn(async move { ... });     // line 190
let dns_handle = Some(tokio::spawn(async move { ... })); // line 203
tokio::spawn(async move { ... });  // cleanup loop, line 223
```

Four independent spawns with no shared `JoinSet`. If SOCKS5 panics, HTTP proxy keeps running with no awareness. If DNS interceptor crashes, no restart. The cleanup task at line 223 is completely fire-and-forget — no handle stored.

**Impact**: Partial failure is invisible. The kernel cannot report which sub-service failed. No coordinated shutdown — SOCKS5 might still be accepting connections when HTTP proxy is already dead.

**Fix**: Use a `JoinSet` for all four sub-services. On any panic/error, decide whether to restart or shut down the entire kernel. Use `CancellationToken` for coordinated shutdown.

---

### DEFECT 4: `join_all()` Panics on First Task Error (Untested Edge Case)

**Severity**: MEDIUM | **Category**: Error Handling | **CW-**: SC-004

**Evidence**: Tokio docs — `JoinSet::join_all()` panics if any task returns `JoinError`. This is documented behavior but NeoTrix may use it implicitly.

**Impact**: If any one task in a batch panics or is aborted, `join_all()` re-panics the calling task, potentially cascading failures. In a multi-agent coordinator scenario (`coordinator.rs:89`), one agent panic kills the entire batch.

**Fix**: Never use `join_all()` for production paths. Use `join_next()` loop with explicit error handling. Or use `try_join_next()` for non-blocking drain.

---

### DEFECT 5: ParallelExecutor Uses Sequential Spawn-then-Await Pattern

**Severity**: MEDIUM | **Category**: Performance | **CW-**: SC-005

**Evidence**: `nt_core_parallel/executor.rs:33-44`

```rust
ExecMode::Parallel => {
    let mut results = Vec::new();
    for (_, input, _) in &self.tasks {
        let input = input.clone();
        let handle = tokio::spawn(async move { ... });
        if let Ok(res) = handle.await {
            results.push(res);
        }
    }
    results
}
```

Tasks are spawned one-by-one and immediately awaited. This is **sequential**, not parallel. Each `handle.await` blocks until that task completes before spawning the next. The `JoinSet` pattern (`spawn` all, then `join_next` loop) would actually achieve parallelism.

**Impact**: The "Parallel" execution mode is effectively sequential. Users expecting concurrent execution get no benefit.

**Fix**: Spawn all tasks into a `JoinSet`, then drain with `join_next()` loop. This is exactly what `JoinSet` is designed for.

---

### DEFECT 6: No CancellationToken Integration for Graceful Shutdown

**Severity**: MEDIUM | **Category**: Reliability | **CW-**: SC-006

**Evidence**: `BackgroundLoop::shutdown()` at `handlers.rs:23-60` uses a `watch` channel + 5s deadline + `abort()`. This is a manual reimplementation of what `CancellationToken` + `TaskTracker` provides out of the box.

The 5s deadline is hardcoded. If a handler needs more time (e.g., flushing KB writes, closing network connections), it's forcibly aborted. The `abort()` call at `handlers.rs:56` is ungraceful.

**Impact**: 
- KB writes may be interrupted mid-transaction
- Network connections are closed without FIN handshake
- In-flight LLM API responses are truncated
- No mechanism for handlers to request extended shutdown time

**Fix**: Replace `watch` channel + deadline with `CancellationToken` + `TaskTracker`. Handlers check `token.is_cancelled()` and can request extended shutdown via a protocol.

---

### DEFECT 7: TaskTracker Not Used Anywhere (Missing OOM-Safe Pattern)

**Severity**: LOW | **Category**: Architecture | **CW-**: SC-007

**Evidence**: Zero `TaskTracker` usage in the entire NeoTrix codebase (grep confirms). The `tokio-util` crate with `TaskTracker` is not in dependencies.

**Impact**: NeoTrix has no OOM-safe task tracking. Any path that spawns tasks in a loop without draining will eventually OOM. The `JoinSet` pattern is used in `coordinator.rs` and `executor.rs`, but those are small bounded batches — the real risk is in long-running background loops.

**Fix**: Add `tokio-util` dependency. Use `TaskTracker` for long-lived task pools (proxy connections, crawl workers, streaming responses) where return values are discarded. Use `JoinSet` only when results are needed.

---

### DEFECT 8: Single-Waker Race in `poll_join_next`

**Severity**: LOW | **Category**: Concurrency Bug | **CW-**: SC-008

**Evidence**: Tokio docs — "on multiple calls to `poll_join_next`, only the `Waker` from the `Context` passed to the most recent call is scheduled to receive a wakeup." Tokio issue #5835 (fixed in later versions) — `join_next` could hang if initial poll occurs on an empty set.

**Impact**: If two components both call `join_next()` on the same `JoinSet` (unlikely but possible with shared state), only one gets notified of completions. The other silently starves.

**Fix**: Enforce single-owner pattern for `JoinSet`. Use the actor pattern for shared task management. Document that `JoinSet` must not be polled from multiple async contexts.

---

### DEFECT 9: `detach_all()` Used Without Awareness of Consequences

**Severity**: LOW | **Category**: Resource Leak | **CW-**: SC-009

**Evidence**: `JoinSet::detach_all()` removes tasks without aborting. The NeoTrix codebase doesn't use it, but the `spawn()` method in `handlers.rs:7-12` effectively creates detached tasks — spawned tasks have no owner tracking their completion.

```rust
pub fn spawn<F>(&mut self, task: F) {
    self.handles.push(tokio::spawn(task));
}
```

The doc says "Such tasks will be aborted during shutdown without grace period" but the handles are never individually awaited — they're bulk-aborted at shutdown. This is functionally similar to `detach_all()` followed by `abort_all()`.

**Impact**: Ad-hoc spawned tasks (via `BackgroundLoop::spawn()`) cannot be individually monitored. If one crashes, no one knows until shutdown.

**Fix**: Route ad-hoc spawns through the same `JoinSet` as coordinated handlers. Add a `spawn_tracked()` method that returns an `AbortHandle` for individual monitoring.

---

### DEFECT 10: No Backpressure on Task Spawning

**Severity**: LOW | **Category**: Resource Management | **CW-**: SC-010

**Evidence**: `coordinator.rs:89` spawns one task per (agent, task_index) pair with no concurrency limit. If allocation produces 100 tasks, 100 `tokio::spawn` calls fire immediately.

**Impact**: Under high load, unbounded spawning can exhaust the Tokio task budget (default 512K tasks) or memory. No semaphore-based backpressure.

**Fix**: Use `tokio::sync::Semaphore` to limit concurrent task spawning. Or use `JoinSet` with `capacity()` (if available) or spawn-then-batch-await pattern with bounded batch size.

---

## 3. Recommended Architecture for NeoTrix

### Task Lifecycle Tiers

| Tier | Pattern | Use Case | NeoTrix Module |
|------|---------|----------|----------------|
| **T1: Ephemeral** | `tokio::spawn` + immediate `await` | One-shot operations | N/A (rare) |
| **T2: Batch** | `JoinSet` — spawn all, drain all | Parallel batch processing | `nt_core_parallel`, `nt_world_search` |
| **T3: Long-lived** | `TaskTracker` + `CancellationToken` | Background loops, proxies, listeners | `nt_mind_background_loop`, `nt_shield_proxy_kernel` |
| **T4: Dynamic** | `JoinSet` with `try_join_next()` drain | Dynamic task pools | `nt_io_provider` streaming, `nt_world_crawl` |

### Migration Path

1. **Immediate**: Add `tokio-util` to workspace dependencies
2. **Phase 1**: Replace `Vec<JoinHandle>` in `BackgroundLoop` with `JoinSet<()>` + periodic `try_join_next()` drain
3. **Phase 2**: Wrap proxy kernel sub-services in a `JoinSet` with `CancellationToken`
4. **Phase 3**: Add `TaskTracker` for long-lived connection handlers (API proxy, crawl workers)
5. **Phase 4**: Audit all `tokio::spawn` callsites — classify into T1-T4 and apply appropriate pattern

---

## 4. Sources

- Tokio `JoinSet` docs: https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html
- Tokio `TaskTracker` docs: https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTracker.html
- Tokio structured concurrency issue #1879: https://github.com/tokio-rs/tokio/issues/1879
- Tokio `join_next` hang bug #5835: https://github.com/tokio-rs/tokio/issues/5835
- Tokio memory "leak" #4406: https://github.com/tokio-rs/tokio/issues/4406
- Tokio `join_all` discussion #6664: https://github.com/tokio-rs/tokio/issues/6664
- `TaskTracker` drop behavior clarification #7222: https://github.com/tokio-rs/tokio/issues/7222
- `JoinSet` completed tasks retention #327: https://github.com/leynos/mxd/issues/327
- Microsoft RustTraining async patterns: https://github.com/microsoft/RustTraining/blob/main/async-book/src/ch13-production-patterns.md
- Tokio graceful shutdown discussion #1819: https://github.com/tokio-rs/tokio/discussions/1819
- Structured concurrency comparison (rustz2h): https://rustz2h.com/chapter_07_mastering_async_rust_and_tokio/series_04_async_error_handling_and_cancellation/structured_concurrency_join_set
- Rust community: TaskTracker vs JoinSet decision: https://users.rust-lang.org/t/handle-the-result-returned-by-tasks-in-tokio-util-s-tasktracker/126618
