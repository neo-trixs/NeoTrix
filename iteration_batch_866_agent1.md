# Agent 1: Async Task Scheduling (Batch 866)

## Sources
- Tokio docs: `tokio::task::JoinSet`, `tokio::task::coop`, `tokio::task::yield_now` (docs.rs)
- DeepWiki: "Cooperative Scheduling and Thread Parking | tokio-rs/tokio" (2026-09-03)
- ToolsKu: "Rust Tokio Task Budget: 5 Core Strategies for Cooperative Scheduling & Starvation Prevention" (2026-05-08)
- Kerkour: "Async Rust: deep dive into cooperative scheduling and Tokio's architecture" (2026-06-03)
- Rust Internals: "Runtime-agnostic cooperative task scheduling budget" (2023-05-07, withoutboats/jonhoo)
- Stanza: "Spawning Tasks - Asynchronous Rust" (tokio::spawn, JoinSet, spawn_blocking)
- GitHub: `tokio-rs/tokio/src/task/join_set.rs` (source)
- Sharpskill: "Async/Await in Rust: Tokio, Futures and Concurrency Guide 2026"

## Defects

**D-TASK-001: No JoinSet usage — 63 raw `tokio::spawn` calls with `Vec<JoinHandle>` lose structured concurrency** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738` | HIGH | Source: JoinSet docs (drop-cancellation guarantee), DeepWiki (structured concurrency section)
- NeoTrix spawns 30+ background handlers via `self.handles.push(tokio::spawn(...))` using a raw `Vec`. `JoinSet` provides structured concurrency: drop cancels all tasks, `join_next()` processes results as they complete, `abort_all()` provides clean shutdown. The manual `shutdown()` method (handlers.rs:23-64) reimplements what JoinSet gives for free with worse correctness guarantees — abort-after-5s is a guess, not structured lifecycle management.

**D-TASK-002: 50+ instances of `std::thread::sleep` in async-adjacent code paths block Tokio worker threads** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:464` | CRITICAL | Source: Tokio docs (cooperative scheduling), Kerkour (avoid blocking event loops), ToolsKu (Pitfall 2)
- The EventBus polling loop uses `std::thread::sleep(Duration::from_millis(10))` in its `TryRecvError::Empty` branch. This blocks the entire Tokio worker thread for 10ms, starving ALL other tasks on that thread. Additional blocking sleeps found in: `nt_world_crawl/fetcher.rs:265,278,291`, `nt_shield_stealth_net` multiple locations, `nt_core_rule_memory.rs:478`, `nt_mind_skill_engine.rs:2574` (`std::thread::yield_now`), `nt_core_observer_error.rs:59,359,371`, `nt_memory_kb/nt_memory_geo.rs:804,992`, `nt_core_forecast.rs:440,486`. All should use `tokio::time::sleep` or `spawn_blocking`.

**D-TASK-003: Zero cooperative scheduling awareness — no `yield_now`, no task budget, no coop module usage** | `neotrix-core/src/unified/layers/meta/nt_meta/nt_meta_async_safety.rs:1-173` | HIGH | Source: Tokio coop docs (budget system), DeepWiki (budget implementation), ToolsKu (Strategy 1)
- The `AsyncSafetyWrapper` module documents that `spawn_blocking` must be used and checks for blocking patterns at analysis time — but it is purely advisory. No runtime enforcement exists. There is zero usage of `tokio::task::yield_now`, `tokio::task::coop::poll_proceed`, or any budget-aware patterns anywhere in the codebase. CPU-intensive loops in background handlers (JSON serialization, KB operations, graph traversal) have no yield points, risking task starvation under load.

**D-TASK-004: Background loop handlers lack task monitoring — no starvation detection, no in-flight metrics** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738-919` | MEDIUM | Source: ToolsKu (Strategy 5 — Runtime Monitoring), DeepWiki (Runtime Metrics section)
- 30+ handlers are spawned without any runtime metrics: no `tasks_spawned`/`tasks_completed` counters, no `in-flight` tracking, no yield count monitoring. Tokio's cooperative scheduling requires the application to observe its own health. Without metrics, starvation is a "silent failure — no panics, no error logs, just a slow rise in latency curves" (ToolsKu). NeoTrix's `LoopReadyScore` (G9) checks handler existence but not scheduling health.

**D-TASK-005: `std::thread::yield_now()` used inside synchronous Hook trait implementation reachable from async context** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_skill_engine.rs:2574` | MEDIUM | Source: Kerkour (cooperative scheduling), Tokio docs (spawn_blocking vs thread::yield)
- `SkillActivationHook::execute()` uses `std::thread::yield_now()` on lock contention. This is a synchronous trait method called from async code. If called on a Tokio worker thread, `std::thread::yield_now()` yields to the OS scheduler (not Tokio's), causing unpredictable delays. Should either be async with `tokio::sync::RwLock` or use `spawn_blocking` for the critical section.

**D-TASK-006: 50+ `tokio::spawn` calls with no JoinError handling — panics silently lost** | `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/gateway/mod.rs:547` | MEDIUM | Source: ToolsKu (Pitfall 4 — Ignoring JoinError), Tokio docs (JoinError::is_panic)
- Most `tokio::spawn` calls store handles in a Vec but never `.await` them or handle JoinError. If a spawned task panics, the error is silently dropped. In the background loop, the behavioral consumer (line 894) and EventBus subscription tasks are fire-and-forget with no error propagation. A panicking handler would silently disappear.

**D-TASK-007: EventBus poll loop uses spin-sleep pattern that blocks Tokio worker — should use async channel recv** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:462-465` | HIGH | Source: Tokio docs (cooperative scheduling), Kerkour (event loop blocking)
- The EventBus consumer loop pattern is: `try_recv()` → if Empty, `std::thread::sleep(10ms)` → repeat. This is a synchronous polling loop running inside `tokio::spawn`. The correct pattern is `tokio::sync::broadcast::Receiver::recv().await` which parks the task without blocking the thread. The current pattern monopolizes a worker thread for up to 10ms per empty poll cycle.

**D-TASK-008: No priority-aware task scheduling — all 30+ background handlers have equal priority** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:779-819` | MEDIUM | Source: ToolsKu (Strategy 3 — Task Priority), Tokio docs (biased select)
- All handlers use the same `spawn_handler!` macro with `tokio::select! { biased; }`. Critical handlers (consciousness_tick, system_health_heal) share scheduling priority with low-priority ones (novel_ingest, game_training). Under contention, critical paths may be starved by low-priority work. The `biased` select only helps within a single task, not across tasks. Should use channel-based priority routing or separate runtimes for critical vs. background work.

## Key Insights
1. **Structured concurrency gap**: NeoTrix's manual Vec<JoinHandle> + timeout-based shutdown is fragile. JoinSet provides drop-cancellation, structured lifecycle, and `join_next()` ordering guarantees for free.
2. **Blocking in async is pervasive**: 50+ `std::thread::sleep` calls across the codebase, many in code reachable from Tokio worker threads. Each blocks the entire worker for the sleep duration, starving all co-located tasks. This is the #1 starvation risk.
3. **No runtime health observability**: Tokio's cooperative scheduling is a two-way contract — tasks must yield, and the runtime must be monitored. NeoTrix has no task metrics, no starvation detection, no yield counting. Silent latency degradation is invisible.
4. **AsyncSafetyWrapper is advisory-only**: The module documents rules but has no enforcement. A static analysis lint or runtime assertion would catch violations at compile/test time rather than production.
5. **Priority inversion risk**: With 30+ equal-priority handlers competing for the same worker threads, critical consciousness evolution work can be delayed by low-priority crawls or novel ingestion.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Sources consulted | 8 |
