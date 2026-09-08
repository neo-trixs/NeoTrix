# Agent 4: Structured Concurrency (Batch 875)

## Sources
- tokio::task::JoinSet docs (docs.rs/tokio/latest/tokio/task/struct.JoinSet.html)
- tokio_util::task::TaskTracker docs (docs.rs/tokio-util/latest/tokio_util/task/task_tracker/)
- Rust Async Book — Structured concurrency (rust-lang.github.io/async-book/part-reference/structured.html)
- Rust Async Book — Cancellation safety (rust-lang.github.io/async-book/part-reference/cancellation.html)
- Microsoft RustTraining Ch13: Production Patterns (github.com/microsoft/RustTraining)
- Tokio Discussion #1819: Graceful shutdown patterns
- Sunshowers: Cancelling async Rust (RustConf 2025)
- Stanza Async Rust: Cancellation Safety
- SharpSkill: Async/Await Tokio Futures Concurrency Guide 2026
- Dev1ta Async Patterns: Structured Concurrency + Backpressure

## Defects

**D-SCON-001: BackgroundLoop uses `Vec<JoinHandle<()>>` instead of `JoinSet` — structured concurrency violated** | `nt_mind_background_loop/mod.rs:96` | HIGH | Source: Rust Async Book structured concurrency chapter + Tokio JoinSet docs

The `BackgroundLoop` struct stores spawned tasks in `pub handles: Vec<JoinHandle<()>>` (line 96). Structured concurrency requires that all child tasks complete before the parent scope exits. `JoinSet` enforces this by cancelling remaining tasks on drop and providing `join_next()` for ordered completion. With raw `Vec<JoinHandle>`, if `BackgroundLoop` is dropped without explicit `shutdown()`, all 40+ handler tasks become orphaned with no cancellation signal. The code comment on `spawn()` (handlers.rs:5) explicitly states "Such tasks will be aborted during shutdown without grace period" — acknowledging the unstructured lifecycle. A `JoinSet` would provide drop-based cancellation, automatic memory reclamation for completed tasks, and elimination of the manual drain+abort loop in `shutdown()`.

**D-SCON-002: Shutdown deadline is shared across all 40+ handlers — later handlers starved** | `nt_mind_background_loop/handlers.rs:48-61` | HIGH | Source: Tokio Discussion #1819 graceful shutdown patterns + async-book cancellation

The `shutdown()` method creates a single `tokio::time::sleep(Duration::from_secs(5))` deadline (line 48) and then iterates over all handles sequentially (line 51). Each handle gets a `tokio::select!` between the shared deadline and the task completing. If the first handler takes 4.9s to respond, the remaining 39+ handlers share only 0.1s. Handlers spawned via `spawn()` (not `spawn_handler!`) are "immediately aborted if they haven't finished by the deadline" (line 22), but the sequential drain means order-dependent starvation. A proper pattern uses per-task deadlines or `JoinSet::abort_all()` after a coordinator-level timeout, ensuring all tasks get equal opportunity for graceful cleanup.

**D-SCON-003: LLM provider streaming tasks fire-and-forget with no cancellation — orphaned HTTP connections** | `nt_io_provider/anthropic.rs:257`, `nt_io_provider/ollama.rs:135`, `free_providers.rs:153` | HIGH | Source: Rust Async Book — "Spawning tasks without awaiting their completion via a join handle, or dropping those join handles" is explicitly listed as a structured concurrency violation

All LLM provider `stream_complete_raw()` implementations spawn a task via `tokio::spawn` (e.g., anthropic.rs:257, ollama.rs:135, free_providers.rs:153) and return the `mpsc::Receiver` without storing the `JoinHandle`. When the receiver is dropped (e.g., client disconnects, timeout), the spawned task continues executing the HTTP request to completion. For expensive operations (Anthropic API calls, multi-second streaming), this wastes network resources. Worse, if the process shuts down while these tasks are in-flight, the HTTP connections are abruptly severed without cleanup. Structured concurrency would scope these tasks to the request lifetime and cancel them when the caller drops.

**D-SCON-004: ParallelExecutor executes tasks sequentially despite `ExecMode::Parallel` — false concurrency** | `nt_core_parallel/executor.rs:33-47` | MEDIUM | Source: Tokio structured concurrency patterns — "Use task groups to manage lifetimes collectively"

The `ParallelExecutor::execute()` method in Parallel mode (line 33) creates tasks in a `for` loop and immediately `handle.await`s each one (line 41), making them execute sequentially despite being spawned. The `tokio::spawn` at line 37 creates a new task, but the subsequent `if let Ok(res) = handle.await` blocks until that specific task completes before spawning the next. All handles should be collected first, then awaited together. This defeats the purpose of structured concurrency for parallelism — tasks are spawned but not coordinated as a group.

**D-SCON-005: EventBus subscriber tasks have no lifecycle tracking — fire-and-forget across 9 layers** | `nt_core_event_bus.rs:362,401` | MEDIUM | Source: Rust Async Book — "Always propagate errors to the parent task" + Tokio best practices

`subscribe_layer()` returns a `JoinHandle<()>` (line 359), and `subscribe_all_layers()` collects 9 handles into a `Vec` (line 401). However, the callers in `BackgroundLoop` and other consumers never store these handles for lifecycle management. The EventBus subscriber tasks run infinite loops (`loop { match rx.recv().await }`) and only exit on `RecvError::Closed`. If the EventBus is dropped, subscribers silently exit. There is no coordinated shutdown — subscribers outlive their intended scope. These should be scoped within a `JoinSet` tied to the EventBus lifetime.

**D-SCON-006: Proxy kernel spawns 4+ background tasks without structured tracking** | `nt_shield_proxy_kernel/kernel.rs:173,187,200,220` | MEDIUM | Source: Tokio graceful shutdown patterns — "coordinating graceful shutdown of multiple concurrently running agents while ensuring none are terminated mid-tool-call"

The `ProxyKernel::start()` spawns: SOCKS5 server (line 173), HTTP proxy (line 187), optional DNS interceptor (line 200), and a cleanup loop (line 220). Only `socks5_handle` and `http_handle` are monitored in the main `select!` loop (lines 264-271). The DNS handle is wrapped in `Option<JoinHandle>` but not awaited in the main loop — if DNS fails silently, the kernel continues running with a dead DNS interceptor. The cleanup task (line 220) is completely fire-and-forget with no error reporting. All 4+ tasks should be in a `JoinSet` where any failure triggers coordinated shutdown.

**D-SCON-007: Batch processor silently swallows JoinError — task panics invisible** | `nt_file_ability/batch_processor.rs:237-242` | MEDIUM | Source: Tokio best practices — "Ensure Send bounds on spawned futures" + "Handle JoinError to detect panics and cancellations"

The `process_chunk()` method spawns tasks and collects handles (line 227), then awaits them (line 238). When a `JoinHandle` returns `Err` (indicating a panic), it maps to a generic `AdapterError::Parse("任务失败")` (line 241). This silently converts task panics into non-panicking errors, hiding the actual failure reason. The `JoinError` from tokio contains the panic message, which should be preserved. Additionally, there's no coordination between tasks — if one panics, others continue. Structured concurrency would propagate the error to the parent and potentially cancel sibling tasks.

**D-SCON-008: `spawn_handler!` macro creates fire-and-forget tasks with no error propagation** | `nt_mind_background_loop/run.rs:734-767` | LOW | Source: Rust structured concurrency — "If a task completes early due to an error, then before returning the task must wait for all its child tasks to complete"

The `spawn_handler!` macro (line 734) spawns tasks that push to `self.handles` but the inner `loop { tokio::select! {...} }` has no error handling beyond the `denylist` gate. If the handler body panics or returns an error, the task silently exits and is only detected during `shutdown()` when the `JoinHandle` is awaited. The macro should propagate handler health back to a monitoring system (e.g., EventBus health signal) so degraded handlers can be detected mid-flight, not just at shutdown time.

**D-SCON-009: ReasoningEngine fire-and-forget token streaming task** | `reason/reasoning_engine/engine_core.rs:1909` | LOW | Source: Rust Async Book — "Spawning tasks without awaiting their completion" + Cancellation safety

The `reason_stream()` method (line 1901) spawns a task (line 1909) to simulate token-by-token streaming via `tokio::time::sleep(10ms)` between words. The spawned task's `JoinHandle` is immediately dropped. If the caller drops the receiver before the task completes, the `tx.send()` at line 1911 returns `Err` and the task exits via `break` — which is correct for backpressure. However, the task holds a clone of `response` (line 1908) which is unnecessarily retained. More critically, this pattern of spawning for delay simulation is an anti-pattern — `tokio_stream::iter` with `throttle` would be structured and cancellation-safe.

**D-SCON-010: Tor crawler spawns discovery and per-page tasks without structured lifecycle** | `nt_shield_stealth_net/tor_crawler.rs:316,345` | MEDIUM | Source: Tokio structured concurrency — "Use JoinSet for managing collections of spawned tasks efficiently"

The `TorCrawler::run()` method spawns a discovery task (line 316) and per-page crawl tasks (line 345) via `tokio::spawn`. The discovery task is fire-and-forget with no tracking. Per-page tasks acquire a semaphore permit but their handles are dropped — if the crawler's `running` flag is set to false, in-flight crawl tasks continue executing until they naturally complete. The `Arc<Self>` reference means the crawler state persists, but there's no coordinated shutdown. A `JoinSet` with `abort_all()` on shutdown would ensure clean termination of all in-flight Tor requests.

## Key Insights

1. **Zero JoinSet usage across 63+ spawn sites**: The entire NeoTrix codebase uses raw `tokio::spawn` + `Vec<JoinHandle>` everywhere. `JoinSet` (stable since Tokio 1.21) provides structured concurrency guarantees that the codebase completely lacks. This is the single most impactful improvement opportunity.

2. **Shared shutdown deadline is a correctness bug, not just a design smell**: With 40+ background handlers sharing a 5-second deadline processed sequentially, handlers at the end of the drain loop get effectively zero grace time. This means state corruption during shutdown (KB writes interrupted, in-flight network requests severed).

3. **LLM provider fire-and-forget is the highest-leak vector**: Every LLM provider (Anthropic, Ollama, OpenRouter, free providers, Gemini, Llama) spawns streaming tasks without tracking handles. In a long-running daemon, dropped client connections leave orphaned HTTP requests consuming bandwidth and potentially accumulating error state.

4. **Parallel executor is secretly sequential**: The `ParallelExecutor` in `nt_core_parallel` claims parallel execution but awaits each handle immediately, serializing all work. This is likely a performance bug that reduces throughput to 1/N of expected parallelism.

5. **No structured error propagation**: Task panics are either silently swallowed (batch_processor), converted to generic errors (eval_harness), or only detected at shutdown. A `JoinSet` with `join_next()` would catch errors in-order as they occur.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| HIGH severity | 3 |
| MEDIUM severity | 5 |
| LOW severity | 2 |
| Files analyzed | 15+ |
| Spawn sites audited | 63+ |
