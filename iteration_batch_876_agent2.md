# Agent 2: Async Task Scheduling (Batch 876)

## Sources
- https://www.youngju.dev/blog/culture/2026-04-15-rust-tokio-async-runtime-future-waker-work-stealing-deep-dive-guide-2025.en
- https://developers-heaven.net/blog/deep-dive-into-the-tokio-runtime-scheduler-and-work-stealing/
- https://tokio.rs/blog/2019-10-scheduler (Tokio official: Making the Tokio scheduler 10x faster)
- https://docs.rs/tokio/latest/tokio/task/coop/index.html (Tokio coop docs)
- https://github.com/tokio-rs/tokio/issues/3150 (Open feature request: task priority — still open since 2020)
- https://github.com/tokio-rs/tokio/issues/6049 (Scheduling fairness docs)
- https://github.com/tokio-rs/tokio/issues/7883 (Timer livelock from eager polling)
- https://www.toolsku.com/en/blog/rust-tokio-task-budget-2026 (5 core strategies for cooperative scheduling)
- https://medium.com/@SmokeAndStrive/the-dark-side-of-tokio-how-async-rust-can-starve-your-runtime-a33a04f6a258
- https://sesamedisk.com/tokio-async-rust-2026 (Real-world bugs and failure modes)
- https://deepwiki.com/tokio-rs/tokio/3.2-schedulers-and-task-execution
- https://deepwiki.com/tokio-rs/tokio/9.2-cooperative-scheduling-and-thread-parking

## Defects

**D-SCHED-001: No task priority system — 30+ background handlers compete at flat priority** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:734-829` | HIGH | Source: tokio-rs/tokio#3150 (still open since 2020), toolsku.com/task-budget-2026

The background loop spawns 30+ independent handler tasks via `spawn_handler!` macro (save, consolidate, goal, knowledge_chain, crystallization, exploration, game_training, healers, system_health_heal, etc.) all as flat-priority `tokio::spawn` tasks. Tokio has no native priority mechanism. Critical tasks like `system_health_heal` and `kb_guard` compete for worker threads with low-priority tasks like `game_training` and `exploration`. Under load, the work-stealing scheduler treats all tasks equally — a CPU-intensive exploration task can starve critical health-healing tasks for entire cooperative budget windows (128 IO ops). The `biased;` select inside each handler only prioritizes shutdown over tick, not cross-handler priority.

**D-SCHED-002: `std::thread::yield_now()` in async context blocks Tokio worker thread** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_skill_engine.rs:2574` | HIGH | Source: tokio.rs/spawn_blocking docs, daily.dev async-rust-deep-dive

`std::thread::yield_now()` yields the OS thread (scheduling hint to the OS kernel), NOT the Tokio cooperative task. This is a blocking operation inside async context — it wastes the Tokio worker thread's time slice doing nothing useful. The correct pattern is `tokio::task::yield_now().await` which yields the cooperative task back to the Tokio scheduler, allowing other async tasks to run. This is a classic anti-pattern: mixing OS-level thread yield with async cooperative scheduling. The code path is in the `SkillActivationHook` which runs during skill loading on every event bus tick.

**D-SCHED-003: Parallel executor awaits tasks sequentially — false parallelism** | `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/executor.rs:34-45` | MEDIUM | Source: youngju.dev Tokio deep dive, toolsku.com/task-budget

The `ParallelExecutor::execute()` in `ExecMode::Parallel` spawns tasks via `tokio::spawn` but then awaits each `JoinHandle` sequentially in a `for` loop (`if let Ok(res) = handle.await`). This means tasks run concurrently on the Tokio runtime but results are collected one-by-one — if task N blocks, tasks N+1..M complete but their results sit in JoinHandle buffers. The correct pattern is `tokio::join_all(handles)` or collecting all handles first, then awaiting them. As implemented, the "parallel" path has the latency of the slowest task plus sequential overhead.

**D-SCHED-004: Fire-and-forget `tokio::spawn` without JoinHandle tracking — silent task loss** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:362`, `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/remote.rs:170`, `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/free_providers.rs:153,342,511,678` | MEDIUM | Source: sesamedisk.com/tokio-async-rust-2026 (Resource Leaks section), vinhnx/VTCode async-architecture guide

Multiple locations spawn tasks via `tokio::spawn(async { ... })` without storing the `JoinHandle`. This creates fire-and-forget tasks that: (1) cannot be cancelled during shutdown, (2) panic errors are silently swallowed, (3) if the runtime shuts down before the task completes, it is silently dropped. The EventBus subscriber at `nt_core_event_bus.rs:362` spawns a loop task that never gets tracked. The remote sandbox log streaming at `remote.rs:170` has the same pattern. At least 8 occurrences in `free_providers.rs` alone. The background loop's `handlers.rs:11` correctly stores handles in `self.handles`, but these external spawn points do not.

**D-SCHED-005: `child.wait_with_output()` in async context — blocking process wait** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers_absorption.rs:189` | HIGH | Source: Tokio spawn_blocking docs ("blocking calls in async context freeze the runtime thread")

`child.wait_with_output()` is a synchronous blocking call (spawns a thread internally via `std::process::Command`). While wrapped in `tokio::time::timeout`, the inner call still blocks the Tokio worker thread for up to 600 seconds (the timeout). During this time, the worker thread cannot service any other async tasks. The correct pattern is `tokio::process::Command::new(...).output().await` which uses async I/O, or at minimum wrapping in `tokio::task::spawn_blocking`. This runs inside the absorption handler which ticks every `PENDING_ABSORPTION_INTERVAL_SECS` — if multiple absorptions queue up, multiple worker threads can be blocked simultaneously.

**D-SCHED-006: No cooperative yield budget in long-running background handlers** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:779-828` | HIGH | Source: Tokio coop module docs, toolsku.com/task-budget-2026 (IO budget exhaustion section)

Each background handler (handle_consolidate, handle_kb_guard, handle_crawl_queue, etc.) runs under the cooperative scheduling model with a default budget of 128 IO operations. Handlers like `handle_consolidate` (KB consolidation), `handle_crawl_queue` (web crawling), and `handle_seed_crawl_queue` perform substantial work per tick — database reads/writes, network fetches, embedding calculations — but never call `tokio::task::yield_now().await` internally. When a handler's work exceeds the 128-op budget, Tokio forcibly yields it, but if the handler performs CPU-heavy work (serialization, parsing) between IO ops, the cooperative budget doesn't trigger. There is no `tokio::task::coop::consume_budget()` usage anywhere in the background loop code. Under sustained load, a single handler tick can monopolize a worker thread for the full duration of its work.

**D-SCHED-007: Nested Tokio runtime in test — potential deadlock and resource waste** | `neotrix-core/src/unified/layers/cognition/nt_mind/reason/reasoning_engine/engine_core.rs:2571` | LOW | Source: sesamedisk.com/tokio-async-rust-2026 (Runtime Mixing Problems section)

Test code creates a new `tokio::runtime::Builder::new_multi_thread()` runtime inside what may already be a `#[tokio::test]` context. Nested runtimes cause: (1) double the thread pool (wasted OS resources), (2) potential deadlocks if the inner runtime blocks on work that requires the outer runtime, (3) confusing behavior when `tokio::spawn` from the outer runtime runs on different threads than expected. The Tokio docs explicitly warn: "Creating nested runtime — this will panic at runtime." While this is test code, it establishes a pattern that could propagate to production code paths.

**D-SCHED-008: `std::sync::Mutex` used extensively across async boundaries — priority inversion risk** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:27-94`, `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_search.rs:1702`, `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/provider_pool.rs:249-252` | MEDIUM | Source: toolsku.com/task-budget-2026 (Priority Inversion section), Tokio anti-patterns

Over 100 instances of `std::sync::Mutex` across the codebase. While many are used correctly (short critical sections, no await while held), several high-contention cases create priority inversion risk: `provider_pool.rs:249` uses a global `static Mutex<ProviderPool>` that every LLM provider selection must lock — under load, low-priority background tasks and high-priority reasoning tasks contend on this same lock. `nt_core_event_bus.rs:27-94` wraps handles, hooks, and sync_handlers in `std::sync::Mutex`, meaning the event bus emit path (called from every domain) contends on these locks. Tokio's cooperative scheduling makes this worse: a task holding a `std::sync::Mutex` blocks the entire worker thread, and other tasks waiting for the mutex cannot proceed even if they're on different workers.

**D-SCHED-009: Work-stealing scheduler LIFO slot bypass under batch spawn** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738`, `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/proxy_pool.rs:616` | LOW | Source: tokio.rs/blog/2019-10-scheduler (LIFO slot section), youngju.dev Tokio deep dive

When the background loop spawns 30+ tasks in rapid succession (lines 738-828), Tokio's LIFO slot optimization only keeps the most recently spawned task in the slot for cache locality. All other tasks go to the local queue or global queue. This means tasks spawned early in the batch (save, consolidate) may be stolen by idle workers before the LIFO slot can serve them, while the last-spawned task (novel_queue) gets LIFO priority. The scheduler periodically bypasses the LIFO slot in favor of FIFO, but during the initial burst spawn, task ordering is unpredictable. This is a minor scheduling inefficiency but contributes to non-deterministic startup behavior.

**D-SCHED-010: No JoinSet usage — unstructured concurrency throughout** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738-828`, `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/coordinator.rs:89-117` | MEDIUM | Source: Tokio docs (JoinSet), sesamedisk.com/tokio-async-rust-2026

The entire background loop and parallel coordinator use `Vec<JoinHandle<()>>` for task management. Tokio 1.x provides `JoinSet` which offers: (1) structured concurrency — all tasks in the set are joined/cancelled together, (2) automatic abort on drop, (3) `join_next()` for polling one task at a time (reduces thundering herd), (4) task ID tracking. The current `Vec<JoinHandle>` pattern requires manual iteration for shutdown (as seen in `handlers.rs:51-61`), cannot join-all efficiently, and has no built-in support for bounded concurrency. The background loop's 5-second shutdown deadline with abort is a workaround for the lack of structured concurrency.

## Key Insights

1. **Tokio has no native task priority** — the feature request (tokio-rs/tokio#3150) has been open since 2020 with explicit maintainer rejection: "For the foreseeable future, Tokio will not have priorities." NeoTrix's 30+ flat-priority background handlers have no mechanism to ensure critical tasks (system_health_heal, kb_guard) run before non-critical ones (game_training, exploration). The recommended workaround is multiple runtimes or channel-based priority queues with `biased;` select.

2. **Cooperative scheduling is a double-edged sword for NeoTrix's architecture** — the cooperative model (128-op budget) works well for IO-bound tasks but NeoTrix's background handlers perform mixed IO+CPU work (KB consolidation, embedding backfill, crawl queue processing). Without explicit `yield_now()` or `consume_budget()` calls, these handlers can monopolize worker threads. The `std::thread::yield_now()` in skill_engine.rs is a symptom of misunderstanding the cooperative model.

3. **The work-stealing scheduler's strengths are undermined by fire-and-forget spawning** — Tokio's work-stealing excels when tasks are well-structured (spawned, awaited, joined). NeoTrix's many untracked `tokio::spawn` calls (event bus subscribers, sandbox log streams, provider pool refreshes) create zombie tasks that the scheduler cannot efficiently manage. These tasks consume memory and scheduling overhead without providing join/cancel semantics.

4. **`std::sync::Mutex` across async boundaries is the silent killer** — with 100+ instances, several in hot paths (provider_pool, event_bus), NeoTrix risks priority inversion where a background task holds a mutex and blocks a high-priority reasoning task. Tokio's cooperative model amplifies this: the mutex-holding thread cannot yield, so all other tasks on that worker are blocked.

5. **The parallel executor's sequential await pattern is a latent performance bug** — tasks are spawned concurrently but results are collected one-by-one, meaning the effective latency is `sum(task_latencies)` not `max(task_latencies)`. This negates the benefit of parallel execution for multi-agent reasoning.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| HIGH severity | 4 |
| MEDIUM severity | 4 |
| LOW severity | 2 |
| Sources consulted | 12 |
