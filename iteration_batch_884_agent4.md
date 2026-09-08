# Iteration Batch 884 — Agent 4: Deep Research: Rust Structured Concurrency

## Research Topic

Rust structured concurrency: `tokio::task::JoinSet`, `tokio_util::task::TaskTracker`, and their implications for NeoTrix's async architecture.

## Key Findings

### 1. `JoinSet` — Lifecycle-Guarded Task Collection

`JoinSet<T>` is Tokio's structured concurrency primitive. Core contract:

- **Drop = abort all tasks**. When a `JoinSet` is dropped, every tracked task is immediately aborted.
- **`join_next().await`** yields results in **completion order**, not spawn order.
- **`shutdown().await`** = `abort_all()` + drain loop. Ignores panics during shutdown.
- **`join_all(self)`** panics on the first `JoinError` — a trap for production code.
- **`detach_all()`** removes tasks without aborting them, breaking structured concurrency guarantees.
- **Cooperative cancellation**: Tasks must hit an `await` point to observe cancellation. CPU-bound loops without yield points will not cancel.
- **Heterogeneous results**: Requires `JoinSet<Box<dyn Future>>` or a custom enum since all tasks must return the same type `T`.
- **No `Send`-free borrowing**: Tasks must be `Send + 'static` (for `spawn`). Use `spawn_local` for `!Send` but this breaks cross-thread structured concurrency.
- **Performance overhead**: ~5–18% slower than `FuturesUnordered` for spawning due to preemptive waker setup via `set_join_waker`.

### 2. `TaskTracker` — Memory-Efficient Graceful Shutdown

`TaskTracker` (from `tokio-util`) is a lighter alternative for shutdown coordination:

- **Immediate memory reclamation**: Unlike `JoinSet`, completed tasks are freed immediately — `JoinSet` retains return values until `join_next()` is called, which can OOM under high task churn.
- **Clonable**: Can be shared across tasks (unlike `JoinSet` which requires `&mut self`).
- **Non-abort on drop**: Dropping a `TaskTracker` does **not** cancel tasks — the opposite of `JoinSet`.
- **`wait()` is gated by `close()`**: `wait()` blocks until both closed AND empty, preventing premature shutdown.
- **No result collection**: Cannot retrieve task return values — designed purely for lifecycle tracking.

### 3. Structured Concurrency Anti-Patterns

| Anti-Pattern | Risk | Mitigation |
|---|---|---|
| `tokio::spawn` without handle tracking | Task leaks, resource exhaustion | Use `JoinSet` or `TaskTracker` |
| `join_next()` in `select!` with external branch | Missed completions | Use `join_next_with_id()` or accept cancel-safety |
| `join_all()` on fallible tasks | Panics on first error, cancels rest | Manual `join_next` loop with error handling |
| `detach_all()` | Breaks lifecycle guarantees | Never call unless explicitly needed |
| `AbortHandle` without drain | Zombie tasks until next `join_next` | Always drain after `abort_all` |
| `select!` with `tokio::spawn` | Unstructured cancellation drops child | Use `JoinSet` + `CancellationToken` |

### 4. `CancellationToken` — Hierarchical Graceful Shutdown

- Parent/child token hierarchy: cancelling parent cascades to children, but not vice versa.
- Must be polled in a `select!` loop inside each task — not automatic.
- Complements `TaskTracker` for production shutdown patterns.

## Defects Extracted for NeoTrix

### Defect S-1: Pervasive Untracked `tokio::spawn` — 75+ Fire-and-Forget Tasks

**Severity: HIGH**

NeoTrix contains **75+ `tokio::spawn` calls** across production code, most of which discard the returned `JoinHandle`. This means:

- Tasks run as detached children of the Tokio runtime with no parent lifecycle.
- If a subsystem (e.g., `nt_shield_proxy_kernel`) is dropped, spawned tasks continue running indefinitely.
- No structured error propagation: a panic in a spawned provider task (e.g., `nt_io_provider/openai.rs:234`) is silently lost.
- Resource leaks under load: spawned OSINT fetchers (`nt_world_osint/person.rs:111`), proxy handlers, and crawl tasks accumulate without bound.

**Evidence**: `grep` found 75+ bare `tokio::spawn` calls across NT-SHIELD, NT-IO, NT-WORLD, NT-MIND, NT-CORE domains. Only `nt_mind_background_loop` collects handles into a `Vec<JoinHandle<()>>`, and even there they are never joined on shutdown.

**Recommendation**: Migrate critical paths to `JoinSet` or `TaskTracker + CancellationToken`. At minimum, track handles and drain on subsystem teardown.

---

### Defect S-2: No Cooperative Cancellation in Long-Running Tasks

**Severity: HIGH**

Several spawned tasks perform long-running loops without `await` yield points or `CancellationToken` polling:

- `nt_core_llm.rs:60` — background loop with no cancellation check
- `nt_io_hotreload/mod.rs:168` — file watcher spawned without shutdown signal
- `nt_shield_traffic/mitm.rs:123` — MITM proxy handler with no cancellation
- `nt_shield_stealth_net/system_proxy.rs:121` — system proxy installer with no abort mechanism
- `nt_io_provider/free_providers.rs:153,342,511,678` — four separate fire-and-forget provider tasks

When a subsystem needs to shut down, these tasks will keep running until the Tokio runtime itself is dropped. This is the classic "dark forest" anti-pattern: tasks that survive their parent.

**Recommendation**: Adopt `CancellationToken` propagated to all long-running loops via `select!`. Wire cancellation into the existing `watch::channel` shutdown patterns already used in `nt_shield_proxy_kernel`.

---

### Defect S-3: EventBus Uses OS Threads Instead of Tokio Tasks

**Severity: MEDIUM**

`nt_core_event_bus.rs:29` stores `Vec<std::thread::JoinHandle<()>>` for event subscribers. The `subscribe_layer` function (line 359) spawns a Tokio task but the bus itself manages OS thread handles.

This creates a hybrid concurrency model:
- OS threads for event dispatch (blocking, heavyweight)
- Tokio tasks for async consumers
- Shutdown uses `AtomicBool` polling (line 436) instead of `CancellationToken`

The `shutdown()` method (line 228) sets an atomic flag and joins OS threads with a timeout. Tokio-spawned tasks (line 362) are never joined or cancelled.

**Recommendation**: Unify on Tokio tasks with `JoinSet` for subscriber management. Replace `AtomicBool` polling with `CancellationToken` for clean shutdown.

---

### Defect S-4: `nt_mind_background_loop` Collects Handles But Never Drains

**Severity: MEDIUM**

`nt_mind_background_loop/mod.rs:96` stores `pub handles: Vec<JoinHandle<()>>`, and `run.rs:738,894` pushes into it. However:

- No `shutdown()` or `abort_all()` call exists in the background loop module.
- When the `BackgroundLoop` struct is dropped, these handles are dropped, which **detaches** the tasks (Tokio `JoinHandle` drop = detach, not cancel).
- Tasks spawned in `handlers_absorption.rs:181` and `handlers.rs:11` are fire-and-forget.

This means the background loop's tasks will continue running after the loop is "stopped", consuming resources and potentially operating on stale state.

**Recommendation**: Add a `shutdown()` method that calls `abort_all()` + drain loop. Use `JoinSet` instead of `Vec<JoinHandle>`.

---

### Defect S-5: No Bounded Concurrency for Provider/Crawler Tasks

**Severity: MEDIUM**

Multiple subsystems spawn tasks in unbounded loops:

- `nt_io_provider/free_catalog.rs:86` — spawns a task per provider in a catalog refresh
- `nt_shield_stealth_net/proxy_pool.rs:616` — spawns health check tasks per proxy
- `nt_world_osint/person.rs:111` — spawns parallel OSINT fetch tasks
- `nt_file_ability/batch_processor.rs:227` — spawns file processing tasks

None of these use `JoinSet` with bounded concurrency or a semaphore. Under load, unbounded spawning can exhaust memory and file descriptors. The `bounded_join_set` crate (73K downloads) exists specifically for this pattern.

**Recommendation**: Replace `Vec<JoinHandle>` + `tokio::spawn` with `bounded_join_set::JoinSet` or `JoinSet` + `tokio::sync::Semaphore` for rate limiting.

---

### Defect S-6: Inconsistent Shutdown Patterns Across Domains

**Severity: MEDIUM**

NeoTrix uses at least 4 different shutdown mechanisms:

| Pattern | Used In | Problem |
|---|---|---|
| `AtomicBool` polling | `nt_core_event_bus`, `event_bus subscribers` | CPU-wasting spin, delayed shutdown |
| `watch::channel<bool>` | `nt_shield_proxy_kernel` | Correct but ad-hoc, not composable |
| `JoinHandle` drop (detach) | `nt_mind_background_loop`, `nt_io_hotreload` | Tasks leak on shutdown |
| `std::thread::JoinHandle` | `nt_core_event_bus`, `nt_shield/http_proxy` | Blocking thread join, not async-aware |

There is no unified `ShutdownCoordinator` or `CancellationToken` tree. Each subsystem reinvents shutdown, making it impossible to guarantee clean teardown order.

**Recommendation**: Adopt a single `CancellationToken` hierarchy rooted at the application level. Use `TaskTracker` for subsystems that need to wait for tasks to exit without collecting results.

---

### Defect S-7: `join_all()` Trap in Potential Future Code

**Severity: LOW (latent)**

While NeoTrix doesn't currently call `JoinSet::join_all()`, the API is available and could be adopted in future refactoring. The trap: `join_all()` panics on the first `JoinError` and cancels all remaining tasks. This is a production footgun for any code that spawns fallible tasks (which is most of NeoTrix).

**Recommendation**: If migrating to `JoinSet`, document in dev-rules that `join_all()` is forbidden for fallible workloads. Use manual `join_next` loops with explicit error handling.

---

## References

| Source | URL |
|---|---|
| Tokio JoinSet docs | https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html |
| Tokio TaskTracker docs | https://docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html |
| Structured concurrency (async-book) | https://rust-lang.github.io/async-book/part-reference/structured.html |
| Tree-structured concurrency (Wuyts) | https://blog.yoshuawuyts.com/tree-structured-concurrency/ |
| JoinSet vs FuturesUnordered perf | https://github.com/tokio-rs/tokio/issues/5564 |
| bounded_join_set crate | https://crates.io/crates/bounded_join_set |
| Structured concurrency cookbook | https://rust-lang-nursery.github.io/rust-cookbook/asynchronous/join.html |
| Tokio #1879 Structured concurrency RFC | https://github.com/tokio-rs/tokio/issues/1879 |
| Tokio #4535 JoinSet stabilization | https://github.com/tokio-rs/tokio/issues/4535 |
| Tokio #5924 JoinSet insert JoinHandle | https://github.com/tokio-rs/tokio/issues/5924 |

## Summary

| # | Defect | Severity | Domain |
|---|---|---|---|
| S-1 | 75+ untracked `tokio::spawn` (fire-and-forget) | HIGH | Cross-cutting |
| S-2 | No cooperative cancellation in long-running tasks | HIGH | NT-IO, NT-SHIELD, NT-WORLD |
| S-3 | EventBus uses OS threads instead of Tokio tasks | MEDIUM | NT-CORE |
| S-4 | Background loop collects handles but never drains | MEDIUM | NT-MIND |
| S-5 | No bounded concurrency for provider/crawler tasks | MEDIUM | NT-IO, NT-WORLD, NT-SHIELD |
| S-6 | Inconsistent shutdown patterns (4+ mechanisms) | MEDIUM | Cross-cutting |
| S-7 | `join_all()` panic trap (latent) | LOW | Future risk |
