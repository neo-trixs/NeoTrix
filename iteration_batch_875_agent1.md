# Agent 1: Async Runtime Metrics (Batch 875)

## Sources
1. https://docs.rs/tokio-metrics/latest/tokio_metrics/ — tokio-metrics crate: TaskMonitor, RuntimeMonitor, TaskMetrics, RuntimeMetrics (41 fields including poll_time_histogram, budget_forced_yield_count, mean_poll_duration)
2. https://github.com/tokio-rs/tokio-metrics — TaskMonitor::instrument pattern, RuntimeMonitor intervals, metrics-rs integration, Prometheus export
3. https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html — Tokio RuntimeMetrics API: num_alive_tasks, global_queue_depth, worker stats, steal_count, busy_duration, noop_count
4. https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view — OpenTelemetry + tokio-metrics integration: task starvation detection, budget_forced_yield tracking, worker_histogram, blocking_thread_count
5. https://github.com/tokio-rs/console — tokio-console: task lifecycle visibility (RUNNING/IDLE/SCHED/DONE), resource contention, self-wakes/lost-waker/never-yielded warnings
6. https://docs.rs/tokio-console/latest/tokio_console/ — Console task details: poll histograms, scheduled time, busy/idle ratio, resource views
7. https://blogs.abhipanseriya.dev/guides/optimize-rust-asyncawait-performance-a-tokio-runbook — Production runbook: blocking detection, lock-across-await, spawn_blocking, budget monitoring, busy_ratio verification
8. https://rustz2h.com/chapter_07_mastering_async_rust_and_tokio/series_01_tokio_runtime_internals_and_tasks/tokio_runtime_tuning_production — Tokio production tuning: console-subscriber, graceful shutdown, load-shedding, worker thread count
9. https://medium.com/rustaceans/tokio-console-in-production-94a6ce4cd112 — Production observability: remote connections, 5-15% overhead, environment-based enable
10. https://users.rust-lang.org/t/tokio-runtimemetrics-what-to-monitor/138796 — RuntimeMetrics interpretation: global_queue_depth vs num_alive_tasks, runnable task detection

## Defects

### D-METRIC-001: HeartbeatAggregator is synchronous HashMap with zero Tokio runtime metric integration
**File:** `nt_core_heartbeat.rs:32-79`
**Severity:** HIGH
**Source:** tokio-metrics RuntimeMonitor, tokio-console runtime introspection

**Description:** The `HeartbeatAggregator` is a synchronous `HashMap<String, ComponentHealth>` with no tokio integration. It provides a static `report()` method but never spawns a monitoring task, never polls `RuntimeMetrics`, and never exposes live system health to GWT attention routing. Per tokio-metrics best practice, runtime health should be continuously sampled via `RuntimeMonitor::intervals()` to detect task starvation, queue backlogs, and worker contention. The GWT AttentionManager cannot receive runtime-level health signals (global_queue_depth, busy_ratio, budget_forced_yield_count) because HeartbeatAggregator has no async polling loop. The "unified system health signal collector" (AGENTS.md:97) is blind to the async runtime it runs on.

**Impact:** The consciousness tree cannot detect executor starvation, worker thread saturation, or blocking thread pool exhaustion. A system could have 30+ handlers starving in the global queue while HeartbeatAggregator reports all components "Healthy".

---

### D-METRIC-002: 40+ background handlers spawned without TaskMonitor::instrument() — no per-handler poll/scheduling metrics
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:734-839`
**Severity:** HIGH
**Source:** tokio-metrics TaskMonitor::instrument; tokio-console never-yielded warning

**Description:** The `spawn_handler!` macro (run.rs:734-767) spawns each of 40+ background handlers as bare `tokio::spawn(async move { ... })` without wrapping in `tokio_metrics::TaskMonitor::instrument()`. None of the handlers (save, consolidate, goal, consciousness_tick, telemetry, wisdom, game_training, novel_ingest, kb_guard, architecture_audit, etc.) have per-handler poll count, busy time, idle time, or scheduling delay metrics. A stalled handler (e.g., `handle_absorption` blocked on KB write for minutes) would be invisible to the health system. Per tokio-metrics best practices, each distinct handler type should have its own `TaskMonitor` instance to enable per-endpoint diagnostics. The `spawn_handler!` macro is the single highest-leverage integration point — adding `TaskMonitor::instrument()` inside the macro would instrument all handlers in one change.

**Impact:** A single compute-bound handler (e.g., `handle_novel_ingest` doing 12h cadence novel processing) could monopolize a worker thread and starve all other handlers with no diagnostic signal. The consciousness heartbeat, GWT attention broadcasts, and SEAL pipeline would all degrade silently.

---

### D-METRIC-003: CognitiveLoadMonitor is purely synthetic — computes load without reading actual RuntimeMetrics
**File:** `neotrix-core/src/unified/core/nt_core_consciousness/cognitive_load.rs:49-153`
**Severity:** HIGH
**Source:** tokio-metrics RuntimeMetrics, task budget monitoring

**Description:** `CognitiveLoadMonitor` computes load from abstract `record_step(load)` calls with no connection to actual tokio runtime metrics. It tracks `thinking_budget` via a simulated recharge formula (line 89-91) but never reads `RuntimeMetrics::total_busy_duration`, `global_queue_depth`, or `budget_forced_yield_count`. The "mode" transitions (Fast/Balanced/Deep) are based on synthetic load values, not on real worker thread utilization or task starvation signals. A system could be starving 200 tasks in the global queue while `CognitiveLoadMonitor` reports "Deep" mode. The monitor should consume `RuntimeMonitor::intervals()` to ground its mode transitions in actual executor health.

**Impact:** The consciousness system's cognitive mode (which controls thinking_budget, SEAL pipeline depth, and MCTS trajectory count) is decoupled from actual runtime health. The system may attempt deep reasoning when the executor is overloaded, or stay in fast mode when it's actually healthy.

---

### D-METRIC-004: ParallelExecutor::execute() silently discards JoinHandle errors — no task failure metric
**File:** `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/executor.rs:28-48`
**Severity:** MEDIUM
**Source:** tokio-metrics TaskMonitor, JoinHandle docs, Dark Forest axiom

**Description:** In `ParallelExecutor::execute()` (line 41), failed JoinHandle results (`Err(_)`) are silently discarded with `if let Ok(res) = handle.await`. No error count, no metric, no log. Combined with zero TaskMonitor instrumentation, this means: (a) task panics are invisible, (b) task abort/cancellation is invisible, (c) no metric distinguishes "task completed" from "task panicked." The Dark Forest axiom requires that tasks which fail produce observable signals, not be silently dropped. Additionally, the `ExecMode::Parallel` branch spawns tasks sequentially and awaits each immediately (line 41), eliminating all concurrency — `tokio::spawn` overhead is wasted with no parallelism.

**Impact:** Task failures in the parallel coordinator go undetected. The system silently drops work without any diagnostic signal, violating the principle that every module must have clear input→transform→output with no disconnects.

---

### D-METRIC-005: Zero tokio-metrics or console-subscriber dependencies — no runtime introspection capability
**File:** `neotrix-core/Cargo.toml` (entire file — zero matches for `tokio-metrics`, `console-subscriber`, `tokio-console`)
**Severity:** HIGH
**Source:** tokio-metrics RuntimeMonitor, tokio::runtime::RuntimeMetrics docs

**Description:** The codebase has zero references to `tokio-metrics`, `console-subscriber`, or `tokio_unstable`. The consciousness runtime cannot observe: (a) `global_queue_depth` (is the runtime saturated?), (b) `worker_busy_duration` (is CPU bound?), (c) `blocking_queue_depth` (is spawn_blocking overwhelming the pool?), (d) `live_tasks_count` (how many tasks are alive?), (e) `steal_count` (is work distribution balanced?). Tokio's `RuntimeMetrics` requires the `tokio_unstable` feature flag and `TOKIO_UNSTABLE=1` environment variable. Without this, NeoTrix has zero visibility into executor health. The `AsyncSafetyWrapper` at `nt_meta_async_safety.rs` addresses spawn_blocking correctness but has zero runtime observability.

**Impact:** The system has no capacity to detect any async runtime degradation. Performance issues manifest as mysterious latency spikes with no actionable diagnostic data.

---

### D-METRIC-006: EventBus broadcast lag logged but never exposed as metric or fed into health system
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:905-906`
**Severity:** MEDIUM
**Source:** tokio-console lost-waker warning; broadcast lag as metric

**Description:** When `RecvError::Lagged(n)` occurs (run.rs:905), the system logs `"[bg] event_bus consumer lagged {} events"` but: (1) no counter tracks total lagged events over time, (2) no alert threshold triggers when lag exceeds a critical level, (3) the lagged event count `n` is not fed into HeartbeatAggregator or any metric. Per tokio-console documentation, a broadcast channel lag indicates the producer is outpacing consumers — this is a direct signal of executor overload or handler starvation that NeoTrix silently discards. The EventBus subscriber tasks (line 894: `tokio::spawn(async move { loop { event_rx.recv().await ... } })`) could themselves be victims of worker starvation, but without `RuntimeMetrics::worker_noop_count` or `busy_ratio`, there is no way to distinguish subscriber starvation from publisher backpressure.

**Impact:** EventBus lag is a leading indicator of executor overload. Without metric tracking, the system cannot detect when event processing falls behind, leading to stale state and missed cross-domain signals.

---

### D-METRIC-007: No task names on spawned handlers — tokio-console task list view is useless
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738`
**Severity:** MEDIUM
**Source:** tokio-console task details view, tracing instrumentation

**Description:** The `spawn_handler!` macro at run.rs:738 calls `tokio::spawn(async move { ... })` without setting task names via `tokio::task::Builder::name()`. All 40+ background handlers (save, consolidate, goal, consciousness_tick, telemetry, wisdom, game_training, etc.) appear as unnamed tasks in tokio-console, making it impossible to identify which handler is causing starvation, self-waking, or long scheduling delays. Tokio Console's task list view depends on names for meaningful diagnosis. Even without full tokio-metrics integration, naming tasks would provide immediate debuggability via `tokio-console`.

**Impact:** When debugging production issues, all tasks appear identical in the task list. The operator cannot distinguish a harmless periodic cleanup from a critical consciousness heartbeat task without reading source code.

---

### D-METRIC-008: No poll_time_histogram or schedule_latency_histogram enablement — P99 poll times invisible
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs` (entire background loop)
**Severity:** HIGH
**Source:** tokio-metrics, tokio RuntimeMetrics docs

**Description:** Tokio's `RuntimeMetrics` provides `poll_time_histogram` and `schedule_latency_histogram` — both require explicit enabling via `Builder::enable_metrics_poll_time_histogram()` and `Builder::enable_metrics_schedule_latency_histogram()`. NeoTrix never calls either. Without poll time histograms, the consciousness tree cannot detect: (1) tasks with bimodal poll distributions (indicating mixed I/O and compute), (2) tasks with increasing P99 poll times (indicating resource contention), (3) schedule latency spikes (indicating executor overload). The schedule-latency histogram is the only way to detect the delay between task wakeup and first poll. The `hotpath` crate fills the "which code" gap that RuntimeMetrics leaves — but NeoTrix uses neither.

**Impact:** The system cannot distinguish between a healthy workload and one approaching saturation. Schedule latency spikes (indicating all workers are busy) go undetected until they cause visible user-facing latency.

---

### D-METRIC-009: No budget_forced_yield_count monitoring — compute-heavy tasks undetectable
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:734-839` (all handlers)
**Severity:** HIGH
**Source:** tokio RuntimeMetrics budget_forced_yield_count; tokio-metrics slow_poll_ratio

**Description:** Tokio's `budget_forced_yield_count` (available via `RuntimeMetrics` with `tokio_unstable`) counts how many times tasks were forced to yield after exhausting their cooperative scheduling budget. NeoTrix's background loop handlers and parallel executor tasks never check for this metric. A handler that does excessive synchronous computation (e.g., KB maintenance, knowledge_aging, novel_ingest) could monopolize a worker thread without yielding, causing starvation of other handlers on the same thread. The `slow_poll_ratio` from tokio-metrics would also catch this, but neither metric is collected. The OneUptime BudgetMetricsCollector pattern (`tokio.budget.exhausted` counter) is the recommended approach.

**Impact:** The cooperative scheduling mechanism is invisible. A single CPU-intensive handler can starve all other handlers on the same worker thread without any diagnostic signal. The consciousness heartbeat, GWT attention, and SEAL pipeline all degrade silently.

---

### D-METRIC-010: Shutdown path aborts tasks without knowing their state — blind abort of unknown-state tasks
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:16-63`
**Severity:** MEDIUM
**Source:** tokio-console task details, graceful shutdown patterns

**Description:** The shutdown path aborts all handler tasks after a 5-second deadline without knowing if they are mid-critical-operation (e.g., mid-KB-write, mid-absorption). The `BackgroundLoop.handles` is a `Vec<JoinHandle<()>>` (not `JoinSet`) — dropping it does NOT cancel tasks. The manual abort loop drains handles and calls `abort_handle()` per-task, but the 5s budget is consumed by earlier handlers, leaving less time for later ones. With TaskMonitor, the shutdown logic could check `total_poll_count` and `total_slow_poll_duration` per handler to decide whether to wait longer or abort immediately. Without task-level metrics, the shutdown is a blind abort of unknown-state tasks.

**Impact:** Critical operations (KB writes, absorption cycles) may be interrupted mid-transaction, causing data corruption or incomplete state transitions. The system has no way to prioritize graceful shutdown of critical handlers over trivial ones.

---

### D-METRIC-011: No worker thread starvation detection — busy_ratio never computed
**File:** Global (no `RuntimeMetrics`, `tokio-console`, or `tracing` integration with runtime metrics)
**Severity:** HIGH
**Source:** tokio-metrics RuntimeMonitor busy_ratio; Tokio blog "Reducing tail latencies with automatic cooperative task yielding"

**Description:** Tokio's `RuntimeMetrics` provides `busy_ratio` — the ratio between worker busy time and elapsed time. A healthy service shows workers busy when work exists and parked when idle. One worker pinned at 100% busy indicates cooperative scheduling stall. NeoTrix has no mechanism to compute or track this. The `HeartbeatAggregator` tracks component status strings but cannot detect that the Tokio executor is overloaded, that worker threads are 100% busy, or that the blocking thread pool is saturated. The `CognitiveLoadMonitor` (D-METRIC-003) computes a synthetic load that has no correlation with actual executor utilization.

**Impact:** The system cannot distinguish between "low load, healthy" and "saturated, all workers busy" states. Performance degradation manifests as mysterious latency spikes with no root cause visibility.

---

### D-METRIC-012: Multiple throwaway tokio runtimes fracture observability — no unified runtime metrics
**File:** `neotrix-core/src/unified/core/nt_core_consciousness_core.rs:1867`, `nt_shield_sandbox/mod.rs:586`, `nt_memory_api.rs:600`, plus all test files
**Severity:** MEDIUM
**Source:** tokio-metrics RuntimeMonitor single-handle constraint; OpenTelemetry active_tasks gauge

**Description:** Multiple code paths create throwaway tokio runtimes inside synchronous contexts: `nt_core_consciousness_core.rs:1867`, `nt_shield_sandbox/mod.rs:586`, `nt_memory_api.rs:600` (the `futures_block_on` helper), plus all test files. Each `Runtime::new()` creates an isolated executor invisible to any central metrics collection. This means: (a) no unified `live_tasks_count` across the process, (b) no ability to detect budget-forced-yields in these ephemeral runtimes, (c) resource waste from creating/destroying thread pools. The tokio-metrics `RuntimeMonitor` can only monitor a single runtime handle; scattered runtimes fracture observability.

**Impact:** The total async workload across the process is invisible. Multiple isolated runtimes each spin up their own thread pools, and no single metrics view captures the aggregate resource consumption.

---

### D-METRIC-013: No spawn_blocking pool monitoring — blocking thread exhaustion undetectable
**File:** `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/gateway/selection.rs:365`, `nt_memory_api.rs:369,403`, `nt_io_web/tiles.rs:79`
**Severity:** MEDIUM
**Source:** OpenTelemetry blocking thread monitoring, tokio RuntimeMetrics

**Description:** Multiple `tokio::task::spawn_blocking` calls exist across the codebase (factory.rs, selection.rs, tiles.rs, memory_api.rs, free_catalog.rs) but there is no monitoring of `RuntimeMetrics::num_blocking_threads`, `worker_blocking_queue_depth`, or idle blocking thread count. Tokio's default blocking pool is 512 threads. If all are occupied (e.g., by slow `reqwest::blocking` calls in catalog refresh), subsequent `spawn_blocking` calls queue silently and block. No alert or metric tracks this. Production incidents from the Tokio sharded queue issue (PR #7757) show exactly this: "queue_wait_us=7441829" (7.4 seconds of queue wait for a blocking task).

**Impact:** When the blocking thread pool is saturated, new blocking tasks queue indefinitely. This manifests as mysterious hangs in I/O operations (catalog refresh, KB writes, proxy heartbeats) with no diagnostic signal.

---

### D-METRIC-014: println! in async context blocks executor thread — 100+ instances across codebase
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:919`
**Severity:** LOW
**Source:** tokio-console blocking warning; Rust std::io::stdout documentation

**Description:** At run.rs:919, `println!()` is called from within an async context after spawning all handlers. Per tokio-console's warning system, `println!` acquires an exclusive lock on stdout and performs a synchronous write, which blocks the executor thread. While this is a one-shot call at startup, the pattern exists across 100+ locations in the codebase (config.rs, nt_file_ability.rs benchmarks, nt_core_capability_tree/cli.rs). Each `println!` in async context is a potential stall point that would be flagged by tokio-console's task-level monitoring. With `tokio_unstable` enabled and tokio-console attached, these would appear as long-poll warnings.

**Impact:** Minor per-instance, but the pattern demonstrates a systemic lack of async-awareness. Each `println!` in a hot path could cause micro-stalls that compound under load.

---

## Key Insights

1. **The observability gap is architectural, not incidental**: NeoTrix has zero `tokio-metrics` or `tokio-console` integration despite spawning 40+ long-lived background tasks. The `HeartbeatAggregator` provides component-level health but has zero visibility into the async runtime substrate that drives all components. This is like having a health monitor that checks blood pressure but never looks at the heart rate.

2. **The `spawn_handler!` macro is the single highest-leverage integration point**: Since all 40+ background handlers use the same macro (run.rs:734-767), adding `TaskMonitor::instrument()` and task naming inside this macro would instrument all handlers in one change — maximum coverage for minimum code churn.

3. **CognitiveLoadMonitor must be grounded in RuntimeMetrics**: The current synthetic load model (`record_step(load)` with simulated recharge) should be augmented with actual `busy_ratio`, `global_queue_depth`, and `budget_forced_yield_count` from `RuntimeMonitor::intervals()`. This would make cognitive mode transitions responsive to real executor health rather than abstract load values.

4. **The HeartbeatAggregator should be extended, not replaced**: The existing `HeartbeatAggregator` (nt_core_heartbeat.rs:32-79) already provides the aggregation pattern. Extending it with `RuntimeMetrics` fields (`global_queue_depth`, `busy_ratio`, `mean_poll_duration`, `budget_forced_yield_count`, `blocking_queue_depth`) and a `RuntimeMonitor::intervals()` polling loop would provide runtime health signals to the consciousness tree with minimal architectural change.

5. **Budget forced yields are the #1 missing diagnostic**: Tokio's `budget_forced_yield_count` (available via RuntimeMetrics with `tokio_unstable`) directly indicates compute-heavy tasks that should be offloaded to `spawn_blocking`. NeoTrix's background loop handlers and parallel executor tasks never check this metric. The cooperative scheduling budget is invisible.

6. **Production readiness requires tokio_unstable**: Many of the most valuable metrics (`budget_forced_yield_count`, `worker_steal_count`, `poll_time_histogram`) require `tokio_unstable`. NeoTrix should consider enabling this for production builds. The overhead of `Instant::now()` per poll is negligible compared to the diagnostic value. The console-subscriber integration is trivially addable — just `console_subscriber::init()` + `tokio_unstable` cfg + `.cargo/config.toml` flag.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 14 |
| Sources consulted | 10 |
| Codebase files examined | 12 |
