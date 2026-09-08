# Iteration Batch 882 — Agent 4: Rust Structured Concurrency Defects

**Topic**: JoinSet, TaskTracker, Structured Concurrency patterns in Tokio  
**Research Date**: 2026-09-07  
**Sources**: Tokio docs, Tokio GitHub discussions, Rust async guides (2025-2026), Microsoft RustTraining, community forums

---

## Defect 1: OOM via JoinSet Return Value Accumulation

**Severity**: P1 — Production Memory Leak  
**Root Cause**: `JoinSet` retains the `Result<T, JoinError>` of every spawned task. If tasks are spawned in a loop and `join_next()` is never called (or called too slowly), completed task return values accumulate unboundedly. Tokio docs explicitly warn: "if the caller keeps inserting tasks and never calls join_next, their return values will keep building up and consuming memory, even if most of the tasks have already exited. This can cause the process to run out of memory." (tokio-util TaskTracker docs)

**NeoTrix Exposure**:
- `nt_mind_background_loop/handlers.rs:11` — `spawn()` pushes `JoinHandle` into `self.handles: Vec<JoinHandle>` without draining completed handles. Ad-hoc tasks accumulate.
- `nt_mind_background_loop/run.rs:738` — `spawn_handler!` macro pushes handles into `self.handles` identically.
- `nt_core_parallel/executor.rs:37` — spawns into a loop with sequential `handle.await` (no OOM here, but pattern is fragile).
- `nt_world_osint/person.rs:111`, `nt_shield_stealth_net/proxy_pool.rs:616`, `nt_file_ability/batch_processor.rs:227` — all push to `Vec<JoinHandle>` without bound.

**Fix**: Use `TaskTracker` from `tokio-util` for fire-and-forget work (immediate memory reclamation on task exit), or drain `JoinSet` with `join_next()` in a loop. For `Vec<JoinHandle>`, periodically call `handle.await` or use `JoinSet` with bounded cardinality.

**Reference**: [tokio-rs/tokio#5910](https://github.com/tokio-rs/tokio/discussions/5910) — "Is there something like JoinSet, but will drop completed task automatically?"

---

## Defect 2: Detached `tokio::spawn` — Zombie Task Leaks

**Severity**: P1 — Resource Leak / Unclean Shutdown  
**Root Cause**: Rust does not enforce structured concurrency at the language level. Every `tokio::spawn` without a corresponding `.await` on the handle creates a detached "zombie task" that outlives its creator. The task holds its captured state (Arc, channels, DB connections) until completion. On shutdown, these tasks are aborted abruptly — no cleanup, no rollback.

**NeoTrix Exposure**: **64 instances** of `tokio::spawn` across the codebase (grep results). Critical offenders:
- `nt_core_event_bus.rs:362` — event bus dispatch task, detached
- `nt_io_web/api.rs:465,1098` — HTTP handler spawns detached tasks
- `nt_io_mail/self_heal.rs:889` — mail self-heal detached
- `nt_io_provider/openai.rs:234`, `anthropic.rs:257`, `gemini.rs:155`, `ollama.rs:135` — provider health-check spawns, all detached
- `nt_shield_traffic/api_proxy.rs:353`, `mitm.rs:123` — network proxy tasks
- `nt_shield_proxy_kernel/kernel.rs:176,190,203,223` — SOCKS5/HTTP listener spawns
- `nt_shield_sandbox/remote.rs:170` — sandbox remote task

**Fix**: Replace detached `tokio::spawn` with `JoinSet` or `TaskTracker` for all tasks that hold resources. At minimum, store handles and drain on shutdown. The `CancellationToken` + `TaskTracker` pattern from `tokio-util` is the idiomatic solution.

**Reference**: [rustfaq.org — "How to Implement Structured Concurrency in Rust"](https://www.rustfaq.org/en/how-to-implement-structured-concurrency-in-rust) — "Detached tasks are a leak waiting to happen. Stick to scopes unless you have a reason not to."

---

## Defect 3: Cooperative Cancellation Gap in CPU-Bound Tasks

**Severity**: P2 — Graceful Shutdown Failure  
**Root Cause**: Tokio cancellation is cooperative — a task only checks the cancel flag at `.await` points. A CPU-bound loop without `.await` will never terminate, even after its `JoinSet` is dropped. The task blocks the executor thread until it yields.

**NeoTrix Exposure**:
- `nt_core_parallel/executor.rs:37-40` — spawns a task that does `tokio::time::sleep(10ms).await` then returns input. This yields, but the pattern is fragile — any future modification that replaces the sleep with pure CPU work creates a non-cancellable task.
- `nt_mind_background_loop/run.rs:744` — handler tick body acquires `MutexGuard` (`h.lock().await`) then runs `$body` which may contain CPU work (denylist checks, capability evolution). If `$body` blocks, the handler won't respond to shutdown signal.
- No `tokio::task::yield_now()` or chunked processing pattern exists in any handler.

**Fix**: Insert `tokio::task::yield_now().await` or `tokio::time::sleep(Duration::ZERO).await` in long-running handler bodies. Use `tokio::select!` with a timeout branch to force periodic yielding. Document the cancellation-point contract for all handler implementations.

**Reference**: [Microsoft RustTraining Ch13](https://microsoft.github.io/RustTraining/async-book/src/ch13-production-patterns.md) — "Cancellation drops the future instantly — use 'cancel-safe' patterns for partial operations"

---

## Defect 4: Missing `JoinSet::abort_all` / No Hierarchical Shutdown

**Severity**: P2 — Cascading Shutdown Failure  
**Root Cause**: NeoTrix uses a flat `Vec<JoinHandle>` with a `watch` channel for shutdown, but there is no hierarchical task tree. When a parent task is cancelled, its children are not automatically cancelled — they must independently observe the shutdown signal. If a child task blocks (e.g., on DB I/O), the parent cannot force-terminate it.

**NeoTrix Exposure**:
- `nt_mind_background_loop/handlers.rs:23-40` — `shutdown()` broadcasts via `watch` channel and waits 5s, then aborts remaining. But child tasks spawned by handlers (e.g., `handlers_absorption.rs:181`) use a separate `tokio::spawn` that doesn't participate in the `watch` channel.
- `nt_shield_proxy_kernel/kernel.rs:176-223` — SOCKS5, HTTP, and fakeIP listeners are spawned independently. No parent `JoinSet` owns them. On shutdown, each must independently notice the signal.
- `nt_io_provider/gateway/mod.rs:547,1200` — gateway dispatch spawns detached tasks per-request.

**Fix**: Replace `Vec<JoinHandle>` with `JoinSet` (which auto-aborts on drop) or `TaskTracker` + `CancellationToken` for graceful shutdown. Implement a task tree: parent `JoinSet` owns child `JoinSet`s. Use `JoinSet::abort_all()` for hard shutdown.

**Reference**: [tokio-rs/tokio#6664](https://github.com/tokio-rs/tokio/issues/6664) — "JoinSet is the main tokio primitive for structured concurrency: it automatically aborts all its tasks on drop."

---

## Defect 5: `ParallelExecutor` Sequential Execution in Parallel Mode

**Severity**: P3 — Logic Bug / False Concurrency  
**Root Cause**: `nt_core_parallel/executor.rs:33-45` — The `ExecMode::Parallel` branch spawns tasks but `await`s each handle sequentially in a `for` loop. This means tasks execute **one at a time**, not concurrently. The `tokio::spawn` is pointless here — the future completes before the next one is polled.

```rust
// Current: sequential await defeats parallel spawn
for (_, input, _) in &self.tasks {
    let handle = tokio::spawn(async move { ... });
    if let Ok(res) = handle.await {  // blocks until this task finishes
        results.push(res);
    }
}
```

**Fix**: Use `JoinSet::join_next()` in a `while let` loop (true concurrent execution), or `futures::future::join_all()` for batch await. The current code is functionally equivalent to sequential execution with extra overhead.

**Reference**: [Tokio JoinSet docs](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html) — "A collection of tasks spawned on a Tokio runtime... allows asynchronously awaiting the output of those tasks as they complete."

---

## Defect 6: `TaskTracker` vs `JoinSet` Misuse — Drop Doesn't Abort

**Severity**: P2 — Silent Task Continuation  
**Root Cause**: If NeoTrix adopts `TaskTracker` (from `tokio-util`) thinking it behaves like `JoinSet`, tasks will continue running after the tracker is dropped. Unlike `JoinSet`, **dropping a `TaskTracker` does not abort the tasks**. This is a deliberate design difference documented in the `TaskTracker` comparison table.

**NeoTrix Exposure**: Not yet present in code, but this is a trap for the planned migration. The `BackgroundLoop::handles` pattern is a natural candidate for `TaskTracker` (memory reclamation), but the shutdown semantics differ fundamentally.

**Fix**: Use `JoinSet` for tasks that must be cancelled on drop. Use `TaskTracker` only for fire-and-forget tasks where graceful completion is acceptable. If using `TaskTracker`, explicitly call `tracker.close()` + `tracker.wait().await` before dropping. Document the semantic difference.

**Reference**: [Tokio TaskTracker docs](https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTracker.html) — "Note that unlike JoinSet, dropping a TaskTracker does not abort the tasks."

---

## Defect 7: No Backpressure on Task Spawning

**Severity**: P2 — Resource Exhaustion Under Load  
**Root Cause**: Multiple NeoTrix modules spawn tasks in unbounded loops without cardinality limits:
- `nt_shield_stealth_net/proxy_pool.rs:616` — spawns a task per proxy health check
- `nt_file_ability/batch_processor.rs:227` — spawns a task per file in batch
- `nt_world_osint/person.rs:111` — spawns a task per OSINT query

No semaphore or bounded concurrency limiter is used. Under high load (many proxies, many files, many queries), task count can spike to thousands, exhausting memory and scheduler capacity.

**Fix**: Use `tokio::sync::Semaphore` to limit concurrent task spawning, or use `JoinSet` with `capacity` (if available) or a bounded `mpsc` channel with worker tasks. The `ParallelExecutor` has `_max_agents` but doesn't enforce it.

**Reference**: [Microsoft RustTraining Ch13](https://microsoft.github.io/RustTraining/async-book/src/ch13-production-patterns.md) — "Bounded channels (mpsc::channel(N)) provide backpressure — senders block when the buffer is full"

---

## Defect 8: `MutexGuard` Across `.await` — Potential Deadlock

**Severity**: P2 — Async Deadlock  
**Root Cause**: The `spawn_handler!` macro acquires `h.lock().await` (a `tokio::sync::Mutex`) and then executes the handler body. If the handler body performs any operation that tries to acquire the same mutex (or a lock that transitively depends on it), a deadlock occurs.

**NeoTrix Exposure**:
- `nt_mind_background_loop/run.rs:745` — `let mut $lock = h.lock().await;` then `$body` runs. If `$body` calls any method on `BackgroundLoop` that re-acquires the lock, deadlock.
- `nt_core_llm.rs:60` — spawns a detached task that may interact with shared state.
- The `BackgroundLoop` struct holds `Arc<Mutex<...>>` and `handles: Vec<JoinHandle>`. Any handler that pushes to `handles` while holding the lock is safe (same lock), but cross-module calls could deadlock.

**Fix**: Scope locks tightly — drop the lock before calling async methods that may re-acquire. Use `tokio::sync::RwLock` for read-heavy handlers. Add `tracing::instrument` to detect lock hold duration in production.

**Reference**: [reintech.io — "How to Avoid Common Async Rust Pitfalls and Deadlocks"](https://reintech.io/blog/avoid-common-async-rust-pitfalls-deadlocks) — "Never hold a MutexGuard across .await — scope locks tightly or use tokio::sync::Mutex"

---

## Summary Table

| # | Defect | Severity | NeoTrix Files | Pattern |
|---|--------|----------|---------------|---------|
| 1 | JoinSet OOM accumulation | P1 | handlers.rs, run.rs, executor.rs, batch_processor.rs, proxy_pool.rs | `Vec<JoinHandle>` never drained |
| 2 | Detached tokio::spawn zombies | P1 | 64 sites across 20+ files | Fire-and-forget spawn |
| 3 | Cooperative cancellation gap | P2 | executor.rs, run.rs | CPU loop without yield |
| 4 | No hierarchical shutdown | P2 | handlers.rs, kernel.rs, gateway | Flat handle vec, no JoinSet |
| 5 | ParallelExecutor is sequential | P3 | executor.rs | Sequential await in parallel mode |
| 6 | TaskTracker drop semantics | P2 | (future risk) | TaskTracker ≠ JoinSet on drop |
| 7 | No spawn backpressure | P2 | proxy_pool.rs, batch_processor.rs, person.rs | Unbounded spawn loops |
| 8 | MutexGuard across .await | P2 | run.rs | Lock held through handler body |

---

## Recommended Migration Path

1. **Phase 1 (Immediate)**: Add `tokio_util::task::TaskTracker` for fire-and-forget spawns (defects 1, 2). Replace `Vec<JoinHandle>` in `BackgroundLoop`.
2. **Phase 2 (Structural)**: Introduce `JoinSet` as parent scope for each handler. Implement hierarchical shutdown with `CancellationToken` tree.
3. **Phase 3 (Correctness)**: Fix `ParallelExecutor` to use `JoinSet::join_next()` loop. Add `Semaphore`-based backpressure to all spawn loops.
4. **Phase 4 (Robustness)**: Audit all `MutexGuard` hold durations. Add `tokio-console` integration for production monitoring.

---

*Generated by Agent 4, Batch 882 — Rust Structured Concurrency Research*
