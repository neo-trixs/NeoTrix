# Agent 2: Async Task Scheduling (Batch 875)

## Sources

1. Tokio scheduler deep dive — https://tokio.rs/blog/2019-10-scheduler
2. tokio::task::coop docs — https://docs.rs/tokio/latest/tokio/task/coop/index.html
3. DeepWiki: Cooperative Scheduling and Thread Parking — https://deepwiki.com/tokio-rs/tokio/9.2-cooperative-scheduling-and-thread-parking
4. Tokio task priority issue #3150 — https://github.com/tokio-rs/tokio/issues/3150
5. Rust Tokio Task Budget strategies — https://www.toolsku.com/en/blog/rust-tokio-task-budget-2026
6. DeepWiki: Task Management — https://deepwiki.com/tokio-rs/tokio/4-task-management
7. Tokio runtime docs — https://docs.rs/tokio/latest/tokio/runtime
8. Rust internals: runtime-agnostic cooperative budget — https://internals.rust-lang.org/t/runtime-agnostic-cooperative-task-scheduling-budget/18796
9. NeoTrix background loop run.rs (lines 734-880)
10. NeoTrix parallel executor.rs (lines 28-48)
11. NeoTrix event_bus.rs (lines 259-397)
12. NeoTrix nt_meta_async_safety.rs (full file)
13. NeoTrix heartbeat.rs (full file)
14. NeoTrix parallel coordinator.rs (lines 60-128)

## Defects

D-SCHED-001: **BackgroundLoop spawns 30+ independent timer-driven tasks with no cooperative budget accounting** | `nt_mind_background_loop/run.rs:734-880` | HIGH | Tokio cooperative budget (default 128 ops/task) is a thread-local counter; 30+ tasks sharing a worker thread each consume budget independently, but none of the spawned handlers check `has_budget_remaining()` or call `consume_budget()`. A single handler tick that does heavy KB I/O (e.g., `handle_consolidate`, `handle_knowledge_aging`) can exhaust the worker thread's budget for all co-located tasks. Tokio's budget is per-task, but the starvation risk here is at the worker-thread level: if 30 handlers fire on the same worker within one `global_queue_interval`, the combined budget consumption degrades fairness across all handlers. No yield points (`yield_now`) are inserted in any handler body.

D-SCHED-002: **ParallelExecutor spawns tasks sequentially in a loop, defeating work-stealing parallelism** | `nt_core_parallel/executor.rs:33-44` | HIGH | The `ExecMode::Parallel` branch spawns tasks one at a time via `tokio::spawn` and immediately `handle.await`s each result before spawning the next. This serializes execution completely — each task runs and completes before the next is spawned. The work-stealing scheduler has nothing to steal because only one task exists at a time. The `priority` field in `tasks: Vec<(String, Vec<f64>, i32)>` is accepted but completely ignored during execution — no priority-based ordering occurs.

D-SCHED-003: **EventBus subscriber tasks run infinite loops with no cooperative yield or budget check** | `nt_core_event_bus.rs:359-396` | MEDIUM | `subscribe_layer` spawns a tokio task with an infinite `loop { rx.recv().await ... }`. While `recv().await` does yield to the scheduler when no event is available, the event-processing branch (lines 369-384) does synchronous work including `filter_event_for_layer` and `log::` calls with no `yield_now` or budget check. Under high event throughput (flood-guard window 500ms allows bursts), a subscriber could process many events consecutively without yielding. The `subscribe_all_layers_sync` variant (line 418) spawns 9 std threads — one per layer — with busy-polling `rx.recv()` + `shutdown_flag` spin-check at 1ms intervals, burning CPU even when idle.

D-SCHED-004: **HeartbeatAggregator is synchronous with no async integration — cannot participate in cooperative scheduling** | `nt_core_heartbeat.rs:32-79` | MEDIUM | The `HeartbeatAggregator` uses `HashMap<String, ComponentHealth>` with no `Arc` or `Send + Sync` bounds. It has no async interface — callers must call `record()` and `report()` synchronously. In the consciousness architecture, health aggregation feeds GWT attention modulation (per `CONTEXT.md`: "Single fact source for GWT attention modulation"). If health checks are called from an async context (e.g., a background handler tick), they must be either in a `spawn_blocking` wrapper or on a blocking thread. Currently there is no evidence any handler wraps health aggregation in `spawn_blocking`, meaning synchronous health checks block the Tokio worker thread.

D-SCHED-005: **No task priority system exists — Tokio has no native priority and NeoTrix implements none** | `nt_core_parallel/executor.rs:24-25, run.rs:779-879` | HIGH | Tokio explicitly does not support task priority (issue #3150: "For the foreseeable future, Tokio will not have priorities"). NeoTrix's `ParallelExecutor.add_task()` accepts a `priority: i32` parameter but ignores it entirely during execution. The background loop uses hardcoded timer intervals (60s-86400s) with no priority differentiation — consciousness-critical tasks (e.g., `handle_consciousness_tick`, `handle_system_health_heal`) run at the same scheduling priority as cosmetic tasks (e.g., `handle_avatar_auto_distill`). The `biased;` keyword in `tokio::select!` (run.rs:743) biases toward the first branch (timer tick) over shutdown, but this is branch-level biasing, not task-level priority.

D-SCHED-006: **BackgroundHandler tasks hold `tokio::sync::Mutex` locks across entire handler tick duration — extended critical sections** | `nt_mind_background_loop/run.rs:744-751` | HIGH | Each handler tick acquires `h.lock().await` (line 745) at the start and holds it through the entire handler body. If a handler does slow I/O (KB writes, HTTP calls), the lock is held for the full duration, blocking all other handlers that share the same `BackgroundLoopHandle` (`Arc<RwLock<...>>`). With 30+ handlers sharing one lock, this creates convoy effects: if one handler takes 2s, all others wait 2s. The `biased;` select between timer tick and shutdown means the lock acquisition itself cannot be preempted.

D-SCHED-007: **Parallel coordinator spawns tasks but holds `Mutex<Box<dyn ReasoningProvider>>` across all parallel tasks** | `nt_core_parallel/coordinator.rs:89-117` | HIGH | The coordinator spawns multiple `tokio::spawn` tasks (line 89) but each spawned task acquires `engine_arc.lock()` (line 91) on a shared `Arc<Mutex<Box<dyn ReasoningProvider>>>`. Since `Mutex` is not async-aware (it's `std::sync::Mutex`), if the lock is contended, the task blocks the Tokio worker thread. This completely defeats the purpose of parallel execution — tasks serialize on the mutex. The `Err(poisoned) => poisoned.into_inner()` path (line 93) silently ignores poison, which could propagate corrupted state.

D-SCHED-008: **AsyncSafetyWrapper is a static analysis tool, not a runtime guard — no enforcement of cooperative scheduling at runtime** | `nt_meta_async_safety.rs:1-173` | MEDIUM | The `AsyncSafetyWrapper` does string-matching on operation names (`operation.contains("blocking")`) to detect violations, but it has no runtime hooks, no Tokio task-local budget tracking, and no actual enforcement mechanism. It's a lint-on-demand tool, not a runtime safety net. The comment at line 5 says "必须 tokio::task::spawn_blocking 包裹并 .await" but the code cannot verify this constraint is met at runtime. A developer could call `reqwest::blocking::get` from async code and the wrapper would only flag it if explicitly asked via `check_safety()`.

D-SCHED-009: **Spawned ad-hoc tasks (via `BackgroundLoop::spawn()`) bypass the coordinated shutdown — abort on deadline without cleanup** | `nt_mind_background_loop/handlers.rs:7-12, 51-61` | MEDIUM | `spawn()` (line 7-12) pushes `tokio::spawn(task)` handles into `self.handles`. During shutdown (line 51-61), all handles are drained and aborted after a 5-second deadline. Tasks spawned via `spawn()` get no cooperative shutdown signal — they are simply `abort()`-ed. This is documented ("aborted during shutdown without grace period") but creates a hazard: if an ad-hoc task holds KB write locks or file handles at abort time, those resources are leaked. The `watch::Receiver` shutdown signal is only available to `spawn_handler!` tasks, not ad-hoc tasks.

D-SCHED-010: **Work-stealing starvation risk: 30+ independent timer tasks with different intervals can synchronize and flood one worker** | `nt_mind_background_loop/run.rs:779-879` | LOW | Tokio's work-stealing scheduler distributes tasks from the global queue across workers. When multiple timers fire simultaneously (e.g., at the 60s boundary, `AGENT_DISCOVERY`, `PENDING_ABSORPTION`, `TELEMETRY`, `ALWAYS_ON` all tick), all tasks are pushed to the global queue and stolen by idle workers. If all 30+ timers happen to tick within the same `global_queue_interval` (31 tasks), one worker could pick up a disproportionate share. Tokio's LIFO slot optimization (per runtime docs: "if a worker thread uses the lifo slot three times in a row, it is temporarily disabled") partially mitigates this, but the scenario where 5+ timers fire on the same second is realistic for NeoTrix's interval design.

## Key Insights

1. **Tokio's cooperative scheduling is opt-in and per-task, not per-worker**: The budget system (`tokio::task::coop`) tracks per-task IO operation counts, but starvation at the worker-thread level is only prevented if tasks voluntarily yield. NeoTrix's 30+ background handlers never check budget or yield voluntarily.

2. **Work-stealing requires concurrent tasks to be effective**: NeoTrix's `ParallelExecutor` serializes task execution via immediate `await`, completely negating the work-stealing scheduler's benefits. Tasks should be spawned in batch and then joined.

3. **Tokio has no native task priority**: This is a deliberate design choice (GitHub #3150). NeoTrix cannot rely on the runtime for priority-based scheduling. The recommended workaround (separate runtimes per priority level) is not implemented.

4. **`std::sync::Mutex` in async contexts is a correctness hazard**: NeoTrix uses `std::sync::Mutex` (not `tokio::sync::Mutex`) in the parallel coordinator, which blocks the Tokio worker thread on contention. This is explicitly warned against in Tokio documentation.

5. **The HeartbeatAggregator lacks async integration**: As the "single fact source for GWT attention modulation" (per CONTEXT.md), it must be callable from async contexts without blocking. It needs `Arc<RwLock<...>>` or channel-based integration.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| HIGH severity | 5 |
| MEDIUM severity | 4 |
| LOW severity | 1 |
| Sources consulted | 14 |
