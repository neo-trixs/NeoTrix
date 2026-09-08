# Agent 4: Structured Concurrency (Batch 869)

## Sources
1. Tokio JoinSet docs — `docs.rs/tokio/latest/tokio/task/struct.JoinSet.html`
2. Tokio TaskTracker source — `github.com/tokio-rs/tokio/blob/master/tokio-util/src/task/task_tracker.rs`
3. Rust Async Book: Structured Concurrency — `rust-lang.github.io/async-book/part-reference/structured.html`
4. RustFAQ: How to Implement Structured Concurrency in Rust — `rustfaq.org/en/how-to-implement-structured-concurrency-in-rust`
5. Rust From Zero to Hero: Structured Concurrency JoinSet — `rustz2h.com/chapter_07_mastering_async_rust_and_tokio/`
6. Tokio Issue #6664: JoinSet join_all/try_join_all — `github.com/tokio-rs/tokio/issues/6664`
7. Tokio Forum: TaskTracker not awaiting spawned futures — `users.rust-lang.org/t/tokio-tasktracker-not-awaiting-for-spawned-futures-to-complete/110139`
8. SharpSkill: Async/Await in Rust: Tokio, Futures and Concurrency Guide 2026 — `sharpskill.dev/en/blog/rust/rust-async-await-tokio-futures-concurrency`
9. Tokio Forum: how to abort one task and all its subtasks — `users.rust-lang.org/t/tokio-how-to-abort-one-task-and-all-its-subtasks/121153`
10. pedram-rust.com: A tokio JoinSet and the task I forgot to await — `pedram-rust.com/posts/tokio-join-set-and-the-task-i-forgot-to-await.html`

## Defects

D-SCON-001: Zero JoinSet usage across 50+ tokio::spawn sites — entire codebase uses bare Vec<JoinHandle<()>> with manual abort-on-deadline instead of structured concurrency. Every spawned task is fire-and-forget with no parent-child lifecycle guarantee. | neotrix-core/src/ (50 occurrences across 30+ files) | HIGH | Source: async-book structured concurrency section + JoinSet docs ("When the JoinSet is dropped, all tasks in the JoinSet are immediately aborted")

D-SCON-002: Background loop uses manual Vec<JoinHandle> with 5-second hard abort deadline — handlers that take >5s to complete are forcibly aborted mid-operation (possibly mid-KB-write), with no structured cancellation propagation or cleanup guarantee. The shutdown races handler completion in a loop, meaning early handlers consume budget from later ones. | neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:23-63 | HIGH | Source: async-book ("cancellation of parents is always propagated to child tasks") + TaskTracker docs ("note that unlike JoinSet, dropping a TaskTracker does not abort the tasks")

D-SCON-003: Zero CancellationToken usage across entire codebase — 30+ fire-and-forget tokio::spawn sites have no cooperative cancellation mechanism. Tasks spawned in proxy kernel, stealth net, provider gateway, web API, and batch processor cannot be signaled to stop cleanly. | neotrix-core/src/ (all tokio::spawn sites) | HIGH | Source: TaskTracker docs ("TaskTracker is usually used together with CancellationToken to implement graceful shutdown") + RustFAQ ("tasks must be responsive to cancellation. If your task ignores cancellation, you break the contract of structured concurrency")

D-SCON-004: Batch processor silently swallows task panics as parse errors — `handle.await` failure on line 238-241 maps JoinError (which could be panic or abort) to `AdapterError::Parse("任务失败")`, losing panic information and making failures invisible to callers. No structured error propagation from child to parent task. | neotrix-core/src/neotrix/nt_file_ability/batch_processor.rs:236-243 | MEDIUM | Source: async-book ("errors are always propagated to the parent task. In Rust, this should apply to both returning Result::Err and to panicking")

D-SCON-005: Batch processor and parallel coordinator use Vec<JoinHandle> + sequential join instead of JoinSet — no memory safety guarantee for task return values accumulation. If callers keep inserting tasks without calling join_next, return values build up consuming memory (JoinSet keeps return values; Vec<JoinHandle> does not even track which have completed). | neotrix-core/src/neotrix/nt_file_ability/batch_processor.rs:221-245 + neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/coordinator.rs:81-124 | MEDIUM | Source: TaskTracker docs ("A JoinSet keeps track of the return value of every inserted task. This means that if the caller keeps inserting tasks and never calls join_next, then their return values will keep building up and consuming memory")

D-SCON-006: Proxy kernel spawns 3-4 independent server tasks (SOCKS5, HTTP, DNS, cleanup) with bare tokio::spawn and no structured parent scope — if kernel.start() returns early on error, spawned servers continue running orphaned with no lifecycle management. Each server has its own ad-hoc shutdown via broadcast channel, not a unified JoinSet. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:173-232 | MEDIUM | Source: async-book ("every task you spawn must be awaited before its parent scope exits")

D-SCON-007: Web API agent_reason_stream handler spawns fire-and-forget task on line 465 — if the SSE connection drops, the spawned LLM provider task continues running with no cancellation. The oneshot channel sender is dropped, causing the spawned task's tx.send to silently fail (already handled), but the task itself has no cleanup path. | neotrix-core/src/unified/layers/action/nt_io/nt_io_web/api.rs:465-497 | LOW | Source: async-book ("Spawning tasks without awaiting their completion via a join handle, or dropping those join handles")

D-SCON-008: No task count monitoring or backpressure for spawned tasks — background loop spawns ~30 handlers plus ad-hoc tasks via spawn(), with no concurrent task count tracking, no semaphore-based backpressure, and no way to observe task health at runtime. Proxy health check uses Semaphore for rate limiting but no global task count. | neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:734-824 | MEDIUM | Source: Structured concurrency principle ("the key new fact is that if a task is live, then all of its ancestor tasks must also be live")

D-SCON-009: Shutdown coordinator uses broadcast channel without guaranteed delivery — handlers that are not yet polled when the broadcast fires will miss the shutdown signal and run until abort. broadcast::Sender::send does not guarantee all receivers see the message if they are not actively awaiting. | neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:39-41 | MEDIUM | Source: async-book ("cancellation is cooperative. The task must check the signal and stop")

D-SCON-010: Stealth net modules (proxy_pool, http_client, local_proxy, system_proxy) spawn tasks with no error propagation — spawned tasks log warnings but return errors into void. If a stealth net health check panics, the error is lost. No parent task observes child failure. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/proxy_pool.rs:613-631 + http_client/mod.rs:295-302 + local_proxy.rs:257 | MEDIUM | Source: async-book ("always propagate errors to the parent task. Just like regular error handling, the best thing to do might be to ignore the error, but this should be explicit in the code of the parent task")

## Key Insights

1. **Structural gap**: NeoTrix has 50+ `tokio::spawn` sites but zero structured concurrency primitives (JoinSet/TaskTracker/CancellationToken). The entire task lifecycle is managed ad-hoc via `Vec<JoinHandle>` + broadcast channels + manual abort. This is the single largest architectural deviation from Rust async best practices.

2. **The background loop is the critical path**: It spawns ~30 handler tasks that form the "nervous system" of NeoTrix (save, consolidate, explore, crystallize, etc.). These use a manual shutdown coordinator with a hard 5s abort deadline. A slow KB write or crystallization could be forcibly aborted mid-operation, potentially corrupting state.

3. **No cancellation propagation**: The Rust async book explicitly states that structured concurrency requires "cancellation of parents is always propagated to child tasks." NeoTrix has no mechanism for this — each task independently checks its own broadcast receiver, and fire-and-forget tasks have no check at all.

4. **TaskTracker would be a better fit than JoinSet for the background loop**: Since NeoTrix needs long-running daemon-style tasks (not result-collecting task groups), `TaskTracker` (which doesn't abort on drop, allows `close()` before `wait()`, and doesn't accumulate return values) is the correct primitive. The background loop already has `close()` semantics via broadcast — TaskTracker would formalize this.

5. **Memory leak risk from Vec<JoinHandle>**: The parallel coordinator and batch processor store `JoinHandle`s in a `Vec` and join them sequentially. If the Vec is dropped before all handles are joined (e.g., early return on error), the tasks become detached and leak. JoinSet's drop-abort semantics would prevent this.

6. **Error opacity**: Multiple spawn sites (batch_processor, proxy_pool, web API) silently swallow JoinError into generic error strings, losing panic messages and stack traces. Structured error propagation from child to parent is missing.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| D-SCON-001 through D-SCON-010 | 10 |
| HIGH severity | 3 |
| MEDIUM severity | 6 |
| LOW severity | 1 |
| Sources consulted | 10 |
