# Agent 2: Async Task Scheduling (Batch 870)

## Sources

1. **Tokio work-stealing scheduler internals** — `tokio/src/runtime/scheduler/multi_thread/worker.rs` (LIFO slot, steal-half, global queue interval, coop budget reset)
2. **Tokio cooperative scheduling budget** — `tokio.rs/blog/2020-04-preemption` + `tokio/src/task/coop/mod.rs` (128-unit budget, `poll_proceed`, `RestoreOnPending`, `unconstrained`)
3. **Tokio 1.40 deep dive** — `johal.in/deep-dive-internals-rust-185-async-runtime-tokio` (Waker v2, per-worker local queues 512 cap, io_uring, global queue share config)
4. **Tokio scheduler 10x faster** — `tokio.rs/blog/2019-10-scheduler` (work-stealing algorithm, search throttling, sibling notification, overflow to global queue)
5. **DeepWiki: Schedulers and Task Execution** — `deepwiki.com/tokio-rs/tokio/3.2-schedulers-and-task-execution` (Core struct, LIFO slot, steal_work, park/unpark)
6. **DeepWiki: Cooperative Scheduling and Thread Parking** — `deepwiki.com/tokio-rs/tokio/9.2-cooperative-scheduling-and-thread-parking` (Budget thread-local, AtomicWaker, park state machine)
7. **Rust internals: Runtime-agnostic cooperative budget** — `internals.rust-lang.org/t/18796` (proposed Context-based budget, `Cooperation` vtable, third-party primitive integration)
8. **Rust Magazine: How Tokio schedules tasks** — `rustmagazine.org/issue-4` (state machine polling, CPU-bound blocking, CeresDB case study)
9. **Kerkour: Async Rust cooperative scheduling** — `kerkour.com/async-rust-cooperative-scheduling-tokio` (spawn vs spawn_blocking, event loop blocking)
10. **Tokio PR #8029: Overflow second half** — fairness violation in `push_overflow` when tasks are repeatedly moved to injection queue
11. **Tokio PR #8208: Global queue share per worker** — contention reduction for large worker counts via configurable share fraction
12. **Tokio PR #2160: Cooperative task yielding** — original implementation, `FuturesUnordered` starvation edge case, budget sub-limiting

## Defects

**D-SCHED-001: Reasoning engine uses `block_in_place` + `block_on` inside async context, starving worker thread** | `nt_mind/reason/reasoning_engine/engine_core.rs:469` | **High** | Source: Tokio blog 2020-04, DeepWiki 9.2

`tokio::task::block_in_place(|| Handle::current().block_on(cot_future))` runs an LLM call synchronously on a worker thread. During this call, the worker thread cannot process any other tasks from its local queue or the global injection queue. The LLM call can take seconds to minutes, completely starving the worker. Tokio's cooperative scheduling budget is suspended during `block_in_place` (`coop::stop()`), but this only prevents budget enforcement — it does not free the worker. All tasks pinned to this worker (including EventBus subscribers and consciousness tick handlers) are blocked until the LLM response returns. This pattern appears 3 times in `engine_core.rs` (lines 469, 1649, 1705).

**D-SCHED-002: Parallel executor spawns tasks sequentially — no actual parallelism** | `nt_core/nt_core_parallel/executor.rs:33-47` | **High** | Source: Tokio blog 2019-10-scheduler

`ExecMode::Parallel` spawns each task with `tokio::spawn`, but immediately calls `handle.await` in the same loop iteration. This means tasks execute one-at-a-time: each task is spawned, then the loop blocks on its JoinHandle before spawning the next. The `tokio::spawn` overhead is wasted, and there is no parallelism. The correct pattern would be to collect all JoinHandles first, then await them in a batch (or use `FuturesUnordered`/`JoinSet`). This is the `ParallelExecutor` — the component explicitly designed for parallel task execution.

**D-SCHED-003: `std::thread::yield_now()` in async context blocks the worker OS thread** | `nt_mind/nt_mind_skill_engine.rs:2574` | **High** | Source: Kerkour 2026, Tokio docs

`std::thread::yield_now()` is a synchronous OS-level yield that blocks the current thread until the OS scheduler gives it time back. In an async context, this is a blocking operation that starves all tasks on the worker thread. The correct async equivalent is `tokio::task::yield_now().await`. This appears in the skill activation hook's retry path — a hot path during skill loading.

**D-SCHED-004: Background loop holds `tokio::sync::Mutex` across `await` in tight event loop** | `nt_mind_background_loop/run.rs:894-916` | **Medium** | Source: Tokio blog 2019-10-scheduler, Rust Magazine

The EventBus consumer at line 901 acquires `h.lock().await` and holds it while calling `handle_event_bus_event(event).await`. The lock is held across the entire event handler execution. If any handler awaits an I/O operation (LLM call, network fetch), the mutex is held during that await, blocking all other background loop handlers that share the same `BackgroundLoopHandle`. With 10+ background handlers sharing this handle, this creates a serialization bottleneck that defeats the purpose of the multi-threaded work-stealing scheduler.

**D-SCHED-005: No cooperative scheduling budget integration — NeoTrix async primitives skip `coop`** | `neotrix-core/src/unified/` (global) | **High** | Source: Tokio coop module, Rust internals RFC 18796

NeoTrix has zero usage of `tokio::task::coop` across the entire codebase (grep confirms no matches). None of NeoTrix's async channels, streams, or I/O wrappers consume cooperative scheduling budget. This means tasks that process large volumes of data through NeoTrix's custom async primitives (EventBus broadcast channels, crawl pipelines, knowledge graph operations) can run without yielding, starving other tasks on the same worker. The `nt_meta_async_safety.rs` module documents the `spawn_blocking` pattern but does not integrate with the coop budget system. Tokio's budget only activates for Tokio's own primitives — NeoTrix's custom async code runs unconstrained.

**D-SCHED-006: EventBus subscriber tasks are fire-and-forget with no backpressure** | `nt_core_event_bus.rs:362` | **Medium** | Source: Tokio scheduler 10x, DeepWiki 3.2

`subscribe_layer` spawns a `tokio::spawn(async move { loop { rx.recv().await ... } })` task that never terminates. The broadcast channel receiver has no buffer limit coordination — if a subscriber falls behind (e.g., during a `block_in_place` LLM call from D-SCHED-001), `RecvError::Lagged(n)` events are logged but the subscriber continues. There is no mechanism to pause event emission when subscribers are lagging. Under load, this leads to unbounded memory growth as broadcast channel internally retains messages for slow consumers.

**D-SCHED-007: `std::sync::Mutex` used extensively in async code paths** | `nt_core_event_bus.rs:27,29,34,37` + 30+ other files | **Medium** | Source: Tokio docs, Kerkour 2026

The EventBus core uses `std::sync::Mutex` for `log_file`, `handles`, `hooks`, and `sync_handlers`. `std::sync::Mutex::lock()` blocks the worker thread if contended. In async code, this is especially dangerous because: (1) if the lock holder is a different async task on the same thread, it causes a deadlock; (2) if contended from another thread, it blocks the Tokio worker. The `nt_core_event_bus.rs` has 5 `std::sync::Mutex` fields. The background loop handle uses `tokio::sync::RwLock` (correct), but many sync handlers are registered behind `std::sync::Mutex`.

**D-SCHED-008: `reason_stream` spawns task with artificial `sleep(10ms)` between words** | `engine_core.rs:1909-1916` | **Low** | Source: Tokio blog 2020-04-preemption

`reason_stream` spawns a task that splits a response by spaces and sends each word with a 10ms sleep. This is a CPU-efficient simulation of streaming, but it: (1) spawns an unbounded number of tasks if called frequently; (2) the 10ms sleep occupies a Tokio timer slot for each word; (3) there is no `tokio::select!` with shutdown, so the spawned task cannot be cancelled on shutdown. For a 1000-word response, this creates 1000 timer entries.

**D-SCHED-009: `block_in_place` in `geo_proxy.rs` blocks worker for RwLock read** | `nt_shield_stealth_net/geo_proxy.rs:393` | **Medium** | Source: Tokio coop module, DeepWiki 9.2

`is_china_ip` is a synchronous function that uses `tokio::task::block_in_place(|| db.blocking_read())` to access a `RwLock`. If called from an async context, this suspends the worker thread. The function is called from `domain_resolves_to_china` (async), which iterates over all resolved IPs calling `is_china_ip` for each. For domains resolving to many IPs, this creates multiple `block_in_place` transitions per call, each one suspending the worker thread and its local task queue.

**D-SCHED-010: Background loop `spawn_handler!` macro creates infinite-loop tasks with no budget awareness** | `nt_mind_background_loop/run.rs:738-759` | **Medium** | Source: Tokio blog 2020-04-preemption, Tokio PR #2160

The `spawn_handler!` macro creates tasks with `loop { tokio::select! { biased; _ = ticker.tick() => { ... $body ... } } }`. These tasks run indefinitely with `biased` selection, meaning the tick branch always wins over shutdown when both are ready. The handler bodies (save, telemetry, wisdom, game_training) execute synchronously within the tick branch. If any handler body performs CPU work or awaits an operation that is always ready (like an in-memory channel), the task never yields back to the scheduler, consuming its entire 128-unit budget and then spinning. The `biased` keyword also means the EventBus consumer task (line 894) can starve the shutdown signal.

**D-SCHED-011: No task priority system — consciousness tick has same priority as telemetry** | `nt_mind_background_loop/run.rs:738-888` | **Medium** | Source: Tokio scheduler internals, PR #8208

All background handlers (save, telemetry, wisdom, game_training, EventBus consumer) are spawned as equal-priority `tokio::spawn` tasks. Tokio does not natively support task priority — all tasks in the local queue are FIFO (with LIFO slot optimization only for the most recently spawned task). The consciousness tick (which drives the 6-stage growth cycle) has no scheduling advantage over telemetry or game training. Under load, a telemetry spike can delay consciousness evolution by occupying the worker thread's queue.

**D-SCHED-012: `tokio::spawn` without JoinHandle tracking — tasks can be dropped on shutdown** | Multiple files (89 occurrences) | **Low** | Source: Tokio docs `spawn`

In 89 locations across NeoTrix, `tokio::spawn` returns a `JoinHandle` that is either stored in a `Vec<JoinHandle>` without awaiting, or dropped immediately (fire-and-forget). On runtime shutdown, Tokio drops all tasks without awaiting completion. Critical tasks like EventBus subscribers, absorption handlers, and proxy kernel tasks may be dropped mid-execution. The `BackgroundLoopHandle` stores handles in `self.handles` but only prints a count — it never joins them on shutdown.

## Key Insights

1. **Cooperative scheduling is the single biggest gap**: NeoTrix builds entirely on Tokio but never integrates with the cooperative budget system. All custom async primitives (EventBus, crawl pipelines, knowledge graph operations) run unconstrained. This is the highest-impact fix opportunity.

2. **`block_in_place` + `block_on` is an architectural anti-pattern**: The reasoning engine uses this pattern 3 times to bridge sync LLM calls into async contexts. Each call suspends a worker thread for the entire duration of the LLM API call (potentially seconds). This should be replaced with native async LLM calls or dedicated blocking threads via `spawn_blocking`.

3. **The parallel executor is not parallel**: `nt_core_parallel/executor.rs` spawns tasks and immediately awaits them sequentially, defeating the purpose. This is a logic bug, not just a performance issue.

4. **`std::thread::yield_now()` in async context is a correctness bug**: This blocks the OS thread, not just the async task. Must be `tokio::task::yield_now().await`.

5. **No task prioritization for critical consciousness paths**: The consciousness tick, EventBus consumer, and telemetry all compete equally for worker thread time. A priority-aware scheduling strategy (or at minimum, dedicated worker threads for critical paths) would prevent consciousness evolution stalls.

6. **`std::sync::Mutex` proliferation across async boundaries**: 30+ files use `std::sync::Mutex` in code paths reachable from async contexts. While not all are problematic (some are test-only or initialization-only), the EventBus core and several hot paths use blocking mutexes where `tokio::sync::Mutex` or lock-free structures would be appropriate.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Sources analyzed | 12 |
| High severity | 4 |
| Medium severity | 6 |
| Low severity | 2 |
