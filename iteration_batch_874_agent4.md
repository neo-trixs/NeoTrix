# Agent 4: Structured Concurrency (Batch 874)

## Sources

1. Tokio JoinSet documentation — `docs.rs/tokio/latest/tokio/task/struct.JoinSet.html`
2. Tokio TaskTracker documentation — `docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html`
3. Rust async-book structured concurrency section — `rust-lang.github.io/async-book/part-reference/structured.html`
4. Tokio TaskTracker source — `github.com/tokio-rs/tokio/blob/master/tokio-util/src/task/task_tracker.rs`
5. Rust From Zero to Hero: Structured Concurrency — `rustz2h.com/chapter_07_mastering_async_rust_and_tokio/`
6. Tokio Issue #6664: JoinSet join_all/try_join_all — `github.com/tokio-rs/tokio/issues/6664`
7. Tokio Forum: TaskTracker not awaiting spawned futures — `users.rust-lang.org/t/tokio-tasktracker-not-awaiting-for-spawned-futures-to-complete/110139`
8. Rust FAQ: Structured Concurrency — `rustfaq.org/en/how-to-implement-structured-concurrency-in-rust/`
9. Tokio Graceful Shutdown tutorial — `tokio.rs/tokio/topics/shutdown`
10. pedram-rust.com: A tokio JoinSet and the task I forgot to await — `pedram-rust.com/posts/tokio-join-set-and-the-task-i-forgot-to-await.html`

## Defects

D-SCON-001: Zero JoinSet usage across 50+ tokio::spawn sites — entire codebase uses bare `Vec<JoinHandle<()>>` with manual abort-on-deadline instead of structured concurrency. Every spawned task is fire-and-forget with no parent-child lifecycle guarantee. | `neotrix-core/src/` (50 occurrences across 30+ files) | high | Source 1, 2, 5

D-SCON-002: BackgroundLoop stores handles as `Vec<JoinHandle<()>>` instead of `JoinSet<()>` — `mod.rs:96` uses `pub handles: Vec<JoinHandle<()>>` to track spawned handler tasks. A `Vec<JoinHandle>` is not a structured concurrency primitive: dropping it does NOT cancel tasks (unlike `JoinSet` which aborts on drop). The 5-second manual abort loop in `handlers.rs:48-61` is a workaround for this missing semantic. | `nt_mind_background_loop/mod.rs:96` | high | Source 1, 6

D-SCON-003: Zero CancellationToken usage across entire codebase — 30+ fire-and-forget tokio::spawn sites have no cooperative cancellation mechanism. Tasks spawned in proxy kernel, stealth net, provider gateway, web API, and batch processor cannot be signaled to stop cleanly. | `neotrix-core/src/` (all tokio::spawn sites) | high | Source 3, 9

D-SCON-004: Parallel executor sequentially awaits each spawned task — `nt_core_parallel/executor.rs:37-43` spawns a task then immediately `.await`s the handle before spawning the next. This means tasks execute sequentially despite being on separate threads. The entire point of `JoinSet` (or even `futures::join_all`) is to poll multiple futures concurrently. This code defeats parallelism entirely. | `nt_core_parallel/executor.rs:37-43` | high | Source 1, 5, 10

D-SCON-005: Proxy kernel spawns 3-4 independent server tasks (SOCKS5, HTTP, DNS, cleanup) with bare tokio::spawn and no structured parent scope — if kernel.start() returns early on error, spawned servers continue running orphaned with no lifecycle management. Each server has its own ad-hoc shutdown via broadcast channel, not a unified JoinSet. | `nt_shield_proxy_kernel/kernel.rs:173-232` | high | Source 1, 2, 5

D-SCON-006: BackgroundLoop shutdown has 5-second hard deadline with no graceful drain — `handlers.rs:48-61` implements shutdown with a 5-second `tokio::time::sleep` deadline. After the deadline, remaining tasks are forcibly aborted via `AbortHandle`. This is a "crash-only" shutdown rather than a graceful drain. A `TaskTracker` with `close()` + `wait()` pattern would allow tasks to complete their current work before exiting. | `nt_mind_background_loop/handlers.rs:48-61` | high | Source 2, 3, 7

D-SCON-007: Shutdown drain uses sequential select per handler instead of concurrent JoinSet — shared 5-second budget creates starvation cascade. The drain loop (handlers.rs:51-61) iterates handlers sequentially: `for handle in self.handles.drain(..) { tokio::select! { ... } }`. With 35+ handlers and a single 5-second deadline, the budget is consumed sequentially. If the first handler takes 4 seconds, the second handler gets 1 second, the third gets the remainder, and all remaining handlers are instantly aborted. | `nt_mind_background_loop/handlers.rs:51-61` | high | Source 1, 2

D-SCON-008: LLM stream relay JoinHandle not awaited — `nt_core_llm.rs:60-73` spawns a task to relay streaming chunks from provider to consumer, but the JoinHandle is immediately dropped. If the relay task panics, the error is silently swallowed by tokio's default panic handler. The caller receives a channel that may never complete. | `nt_core_llm.rs:60` | high | Source 5, 10

D-SCON-009: Background handlers lack cooperative cancellation — `spawn_handler!` macro (run.rs:738-839) spawns tasks that check `rx.changed()` for shutdown, but this is a one-shot signal. Handlers cannot observe cancellation mid-operation (e.g., during KB writes, file I/O). A CancellationToken pattern would allow handlers to check `is_cancelled()` at multiple points within their tick body. | `nt_mind_background_loop/run.rs:738-839` | medium | Source 3, 9

D-SCON-010: No structured error propagation from spawned tasks — proxy kernel spawns SOCKS5/HTTP/DNS listeners as independent tokio::spawn calls. If any listener panics (e.g., bind failure), the error is silently lost via `let _ = socks5_err_tx.send(...)`. There is no JoinSet to aggregate results or propagate failures to the parent. The proxy kernel continues running with partial functionality. | `nt_shield_proxy_kernel/kernel.rs:173-220` | medium | Source 1, 2, 5

D-SCON-011: Batch processor and parallel coordinator use Vec<JoinHandle> + sequential join instead of JoinSet — no memory safety guarantee for task return values accumulation. If callers keep inserting tasks without calling join_next, return values build up consuming memory. JoinSet's drop-abort semantics would prevent this. | `nt_file_ability/batch_processor.rs:221-245` + `nt_core_parallel/coordinator.rs:81-124` | medium | Source 2, 5

D-SCON-012: Firewall/guard task spawned without tracking — `kernel.rs:220` spawns a cleanup task (`security_for_gc`) with bare `tokio::spawn` and no handle stored. If this task panics or hangs, it is never detected. During shutdown, this task continues running after SOCKS5/HTTP have stopped. | `nt_shield_proxy_kernel/kernel.rs:220` | medium | Source 1, 5

## Key Insights

1. **Structural gap**: NeoTrix has 50+ `tokio::spawn` sites but zero structured concurrency primitives (JoinSet/TaskTracker/CancellationToken). The entire task lifecycle is managed ad-hoc via `Vec<JoinHandle>` + broadcast channels + manual abort. This is the single largest architectural deviation from Rust async best practices.

2. **JoinSet would eliminate manual shutdown**: The background loop already has `spawn_handler!` macro (run.rs:734) that wraps each handler in a structured loop with shutdown coordination. Converting `Vec<JoinHandle>` to `JoinSet<()>` and using `JoinSet::abort_all()` + `join_next()` drain in `shutdown()` would be a ~50-line change that brings the core loop to structured concurrency compliance.

3. **TaskTracker is the right primitive for long-running handlers**: Since NeoTrix needs daemon-style tasks (not result-collecting task groups), `TaskTracker` (which doesn't abort on drop, allows `close()` before `wait()`, and doesn't accumulate return values) is the correct primitive for the background loop.

4. **CancellationToken enables hierarchical shutdown**: The current `watch::channel<bool>` pattern cannot express "cancel this subsystem but not its parent" or "cancel all children when parent cancels." CancellationToken's `child_token()` solves this natively for the 6-layer architecture.

5. **The parallel executor defeats its own purpose**: The sequential await pattern in `executor.rs:37-43` means the "parallel" executor runs tasks one at a time. Adopting `JoinSet::join_next()` in a loop would fix this trivially.

6. **Memory leak risk from Vec<JoinHandle>**: The parallel coordinator and batch processor store `JoinHandle`s in a `Vec` and join them sequentially. If the Vec is dropped before all handles are joined (e.g., early return on error), the tasks become detached and leak. JoinSet's drop-abort semantics would prevent this.

7. **The 5-second abort deadline is a data loss risk**: Background handlers doing KB writes, file I/O, or network calls are aborted after 5s. Any in-flight SQLite transaction or file write will be corrupted. CancellationToken enables cooperative shutdown where handlers observe cancellation and flush pending work.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Sources consulted | 10 |
| Total tokio::spawn sites analyzed | 50+ |
| Key files with defects | run.rs, handlers.rs, kernel.rs, executor.rs, nt_core_llm.rs, batch_processor.rs, coordinator.rs |
