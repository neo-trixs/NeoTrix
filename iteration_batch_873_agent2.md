# Agent 2: Async Task Scheduling (Batch 873)

## Sources
1. **Tokio work-stealing scheduler internals** — tokio/src/runtime/scheduler/multi_thread/worker.rs (GitHub), DeepWiki: Schedulers and Task Execution, tokio.rs/blog/2019-10-scheduler
2. **Tokio cooperative scheduling** — tokio::task::coop docs (docs.rs), tokio.rs/blog/2020-04-preemption, kerkour.com async-rust-cooperative-scheduling
3. **Tokio task priority** — tokio-rs/tokio#3150 (Add priority to spawned tasks), tokio-rs/tokio#2702 (high tail latencies)
4. **Tokio task budget** — tokio::task::coop::Budget (128 ops/tick), poll_proceed, consume_budget, RestoreOnPending
5. **Tokio spawn bottlenecks** — tokio-rs/tokio#4606 (bottleneck when spawning and immediately awaiting), pranitha.dev "Tokio Gives Progress Not Ordering"
6. **Work-stealing starvation patterns** — dial9-tokio-telemetry TRACE_ANALYSIS_GUIDE.md, tokio-rs/tokio#6175 (fairness and starvation)

## Defects

### D-SCHED-001: `block_in_place` + `block_on` monopolizes worker threads during LLM calls
| Field | Value |
|-------|-------|
| **Defect** | Reasoning engine uses `tokio::task::block_in_place(\|\| Handle::current().block_on(...))` for LLM gateway calls (lines 469, 1649, 1705). LLM calls take 1-30 seconds. During this time the worker thread is fully blocked — all tasks on that worker starve. Tokio's work-stealing scheduler cannot redistribute work because the thread is pinned in `block_in_place`. With N workers and one long LLM call, throughput drops to (N-1)/N. Multiple concurrent reasoning tasks compound: each `block_in_place` locks a worker. |
| **File:line** | `neotrix-core/src/unified/layers/cognition/nt_mind/reason/reasoning_engine/engine_core.rs:469`, `:1649`, `:1705` |
| **Severity** | **Critical** |
| **Source** | Tokio docs: "block_in_place works by transitioning the current worker thread to a blocking thread, moving other tasks running on that thread to another worker" — but this only works for `current_thread` runtime; on `multi_thread` it still blocks the worker. Tokio blog 2020-04: "Tokio does not, and will not attempt to detect blocking tasks and automatically compensate by adding threads." |

### D-SCHED-002: `std::thread::yield_now()` bypasses cooperative scheduling budget
| Field | Value |
|-------|-------|
| **Defect** | Skill engine uses `std::thread::yield_now()` (OS-level thread yield) instead of `tokio::task::yield_now()` (cooperative task yield). The OS yield gives up the CPU time slice to the OS scheduler, not to Tokio's task scheduler. This means the Tokio worker thread may be preempted by the OS and re-scheduled on a different core, losing cache locality. The cooperative budget (128 ops/tick) is not consumed, so the task's Tokio-level yield semantics are lost. |
| **File:line** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_skill_engine.rs:2574` |
| **Severity** | **High** |
| **Source** | Tokio cooperative scheduling: "each Tokio task has an operation budget... once the task is out of budget, all Tokio resources will perpetually return 'not ready' until the task yields back to the scheduler." `std::thread::yield_now()` does not participate in this mechanism. |

### D-SCHED-003: EventBus `emit_from` holds `std::sync::Mutex` in async context blocking worker
| Field | Value |
|-------|-------|
| **Defect** | `EventBus::emit_from` acquires `std::sync::Mutex` on `log_file`, `hooks`, and `sync_handlers` (lines 134, 140, 157). These are synchronous mutexes held while doing file I/O (`writeln!` to log file). The method is called from async handler code (via `emit_event!` macro in background loop). When a subscriber is slow or the log file I/O stalls, the calling Tokio worker thread blocks on the mutex. With the EventBus being the central nervous system of NeoTrix (all modules emit), this is a contention hotspot. The `std::sync::Mutex` does not integrate with Tokio's cooperative scheduling — no budget is consumed, no yield occurs. |
| **File:line** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:132-169` |
| **Severity** | **Critical** |
| **Source** | Tokio docs: "Tasks should generally not perform system calls or other operations that could block a thread." Mutex contention + file I/O inside `emit_from` violates this. Tokio 1.40 scheduler: "per-worker local queues require no locking for pushes from the owning worker" — the EventBus's `std::sync::Mutex` design contradicts this principle. |

### D-SCHED-004: 30+ background handlers spawned with no concurrency bound or priority differentiation
| Field | Value |
|-------|-------|
| **Defect** | The background loop spawns 30+ independent `tokio::spawn` tasks (lines 779-888 in run.rs), each with its own `tokio::time::interval`. All tasks compete equally on Tokio's work-stealing queue with no priority differentiation. Tokio does not support task priority (tokio-rs/tokio#3150: "For the foreseeable future, Tokio will not have priorities"). Critical consciousness tick handlers (600s cadence) share the same FIFO queue as low-priority telemetry and novelty ingest tasks. Under load, a burst of telemetry events can delay consciousness ticks, causing the SEAL pipeline to miss evolution windows. |
| **File:line** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:779-888` |
| **Severity** | **High** |
| **Source** | Tokio issue #3150: "Neither a binary heap or rb tree work well with work stealing... an absolute priority is unlikely to be what you want. It allows a high priority task to completely prevent a lower priority task from running." Tokio provides no built-in priority — NeoTrix must implement priority via separate runtimes or semaphore-based gating. |

### D-SCHED-005: 48 `std::thread::sleep` call sites block Tokio worker threads
| Field | Value |
|-------|-------|
| **Defect** | 48 instances of `std::thread::sleep()` found across the codebase (crawl rate limiting, retry backoff, test delays, stealth proxy, KB operations). When called from a Tokio worker thread (not inside `spawn_blocking`), the entire worker thread sleeps, blocking all other tasks on that worker. Rate limiting delays of 200ms-2100ms (e.g., `nt_memory_crawl.rs:346`, `nt_memory_geo.rs:804`, `nt_world_scrape.rs:287`) completely stall a worker. The `global_queue_interval` in Tokio is ~61 ticks; a 2-second sleep causes ~120 missed global queue checks. |
| **File:line** | Multiple: `nt_memory_kb/nt_memory_crawl.rs:346`, `nt_world_scrape.rs:287`, `nt_world_crawl/fetcher.rs:265`, `nt_core_forecast.rs:440`, and 44 more locations |
| **Severity** | **Critical** |
| **Source** | Tokio blog 2019-10: "When a processor becomes idle, it checks sibling processor run queues and attempts to steal from them." A sleeping worker is not idle — it's occupied doing nothing. Tokio docs: "Tasks should generally not perform system calls or other operations that could block a thread, as this would prevent other tasks running on the same thread from executing as well." |

### D-SCHED-006: EventBus broadcast channel lag causes silent event loss under load
| Field | Value |
|-------|-------|
| **Defect** | The EventBus uses a `tokio::sync::broadcast` channel with capacity 1024 (line 62, 86). When the behavioral consumer (D30 handler, run.rs:894-916) is slow — e.g., blocked by KB writes or consciousness_tick processing — the broadcast receiver lags. The consumer only logs a warning (`RecvError::Lagged(n)`) and drops the missed events (run.rs:905-906). In a burst scenario, critical events (SystemError, GlobalHalt, ConsciousnessCritique) can be silently lost. Tokio's broadcast channel is designed for "fan-out" but not for reliable delivery — it sacrifices reliability for latency. |
| **File:line** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:62,86,167`, `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:891-916` |
| **Severity** | **High** |
| **Source** | Tokio broadcast docs: "If the receiver is too slow, it will receive `Lagged(n)` on next recv." Tokio issue #2702: "high tail latencies with threaded scheduler when under load... tasks spawned from the main function land on the global injection queue... under load, the scheduler heavily prioritizes its local queue." |

### D-SCHED-007: `block_in_place` inside `handle_consciousness_tick` locks background loop for seconds
| Field | Value |
|-------|-------|
| **Defect** | The background loop acquires an `Arc<Mutex<BackgroundLoopHandle>>` lock for each handler tick (run.rs:745). The `handle_consciousness_tick` handler (handlers_consciousness.rs:301) runs the full ConsciousnessTree growth cycle including KB stats queries, GWT resonance broadcast, SEAL pipeline consumption, and self-test evaluation. During this time, all other handlers (save, consolidate, goal, telemetry, etc.) are blocked waiting for the lock. If the growth cycle takes >1s (KB queries + tree computation), the entire background loop stalls. Combined with D-SCHED-001 (LLM calls via block_in_place), the lock can be held for 10+ seconds. |
| **File:line** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:745`, `handlers_consciousness.rs:301-420` |
| **Severity** | **Critical** |
| **Source** | Tokio work-stealing: "a task runs for a long period of time without yielding back to the executor, it can starve other tasks." The background loop's single-lock architecture serializes all handlers, negating the benefit of spawning them as separate Tokio tasks. |

### D-SCHED-008: Unbounded `tokio::spawn` in absorption handler creates uncontrolled fan-out
| Field | Value |
|-------|-------|
| **Defect** | The absorption handler spawns child processes via `tokio::process::Command` with unbounded concurrency (handlers_absorption.rs:181). Each spawned task writes to stdin and waits for completion with a 600s timeout. Under heavy absorption load (many pending cycles), this can create dozens of concurrent child processes, each competing for worker threads. Tokio's default `max_blocking_threads` is 512; unbounded spawning can exhaust this pool. No semaphore or JoinSet bounds the concurrency. |
| **File:line** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers_absorption.rs:181-220` |
| **Severity** | **High** |
| **Source** | Tokio docs: "spawn_blocking uses a separate thread pool (default cap 512)." Tokio blog 2019-10: "The downside is this approach is far more complicated... If not done correctly, the overhead to implement the work-stealing model can be greater than the benefits gained." Unbounded fan-out defeats work-stealing load balancing. |

### D-SCHED-009: Parallel executor lacks work-stealing integration — tasks are sequentially awaited
| Field | Value |
|-------|-------|
| **Defect** | The parallel executor (executor.rs:28-47) spawns tasks in a loop and awaits each sequentially (`for (_, input, _) in &self.tasks { ... handle.await }`). While tasks are spawned concurrently, the results are collected in order, meaning a slow task blocks the result collection. Tokio's work-stealing distributes the spawned tasks across workers, but the sequential `.await` chain means the spawning thread cannot proceed until each task completes. A `JoinSet` or `FuturesUnordered` would allow processing results as they arrive. Additionally, the task `priority` field (line 24) is accepted but completely ignored — there is no priority-based scheduling. |
| **File:line** | `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/executor.rs:28-47` |
| **Severity** | **Medium** |
| **Source** | Tokio issue #4606: "tasks doing the spawning aren't able to get distributed evenly across the threads... threads with the spawning tasks keep overflowing tasks into the global queue." Tokio: "There is no guarantee that a spawned task will execute to completion." |

### D-SCHED-010: 128-operation cooperative budget may be insufficient for consciousness pipeline
| Field | Value |
|-------|-------|
| **Defect** | Tokio's cooperative budget is 128 operations per poll tick. The consciousness pipeline performs KB reads (`stats()`, `embedding_count()`, `get_evolution_history(200)`, `kv_list("experience")`), GWT resonance broadcasts, tree growth cycles, SEAL consumption, and self-test evaluation — all in a single handler tick without explicit yield points. While each KB operation is likely a single budget unit, the aggregate may approach or exceed 128, causing forced yields mid-computation. More critically, the budget only applies to Tokio-native resources (sockets, channels, timers) — KB SQLite operations via `std::sync::Mutex` do not consume budget, creating a false sense of cooperative compliance. |
| **File:line** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:301-420` |
| **Severity** | **Medium** |
| **Source** | Tokio coop docs: "The value (128) needs to be high enough to amortize wakeup and scheduling costs, but low enough that we do not starve other tasks for too long." Tokio blog 2020: "This budget is reset when the scheduler switches to the task... each Tokio resource is aware of this budget." Non-Tokio resources (SQLite, std::sync::Mutex) bypass the budget entirely. |

## Key Insights

1. **Fundamental architecture mismatch**: NeoTrix's background loop serializes all 30+ handlers behind a single `Arc<Mutex<BackgroundLoopHandle>>`, defeating the purpose of Tokio's work-stealing scheduler. Spawning 30 tasks that all contend on one lock is worse than running them sequentially on a single thread.

2. **Blocking operations are pervasive**: 48 `std::thread::sleep` + 3 `block_in_place` + mutex-held file I/O in the EventBus create a "blocking minefield." Each blocking operation can stall a Tokio worker for 100ms-30s, and work-stealing cannot help because the thread is occupied (not idle).

3. **No priority mechanism exists**: Tokio explicitly refuses to implement task priorities (issue #3150). NeoTrix's consciousness tick (the most critical task) has the same scheduling priority as telemetry and novelty ingest. The recommended workaround — separate runtimes — is not implemented.

4. **EventBus reliability gap**: The broadcast channel is designed for speed, not reliability. Critical consciousness events can be silently dropped under load. A bounded mpsc channel with backpressure or a persistent queue would be more appropriate for the "central nervous system."

5. **Cooperative budget blindness**: The 128-operation cooperative budget only governs Tokio-native async resources. NeoTrix's heavy use of SQLite (via std::sync::Mutex), file I/O, and std::thread operations completely bypasses cooperative scheduling, meaning long-running operations can starve other tasks without any yield enforcement.

6. **Absorption fan-out risk**: The experience-tree absorption handler has no concurrency bound. A burst of pending cycles could spawn dozens of concurrent child processes, exhausting the blocking thread pool and starving async workers.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Critical severity | 4 |
| High severity | 4 |
| Medium severity | 2 |
| Sources consulted | 6 major + 48 code sites |
