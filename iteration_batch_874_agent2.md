# Agent 2: Async Task Monitoring (Batch 874)

## Sources

1. https://docs.rs/tokio-metrics/latest/tokio_metrics/ — tokio-metrics crate: TaskMonitor, RuntimeMonitor, TaskMetrics, RuntimeMetrics (41 fields including poll_time_histogram, budget_forced_yield_count, mean_poll_duration)
2. https://github.com/tokio-rs/tokio-metrics — TaskMonitor::instrument pattern, RuntimeMonitor intervals, metrics-rs integration, Prometheus export
3. https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html — Tokio RuntimeMetrics API: num_alive_tasks, global_queue_depth, worker stats, steal_count, busy_duration, noop_count
4. https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view — OpenTelemetry + tokio-metrics integration: task starvation detection, budget_forced_yield tracking, worker_histogram, blocking_thread_count
5. https://github.com/tokio-rs/console — tokio-console: task lifecycle visibility (RUNNING/IDLE/SCHED/DONE), resource contention, self-wakes/lost-waker/never-yielded warnings
6. https://docs.rs/tokio-console/latest/tokio_console/ — Console task details: poll histograms, scheduled time, busy/idle ratio, resource views
7. https://hotpath.rs/tokio_runtime — hotpath vs RuntimeMetrics: runtime tells you THAT it's under load; hotpath correlates with WHICH code causes saturation
8. https://sesamedisk.com/tokio-async-rust-2026 — Production anti-patterns: blocking in async, uncontrolled task spawning, resource leaks from forgotten JoinHandles
9. https://medium.com/rustaceans/a-deep-dive-into-tokio-console-6af9bfa870ee — Production debugging: poll count/busy/idle ratio, high scheduled time diagnosis, task leak identification
10. https://rustify.rs/articles/rust-async-runtimes-tokio-vs-async-std-2026 — 2026 runtime landscape: Tokio dominates 42%+ of new Rust backend projects

## Defects

### D-TMON-001: HeartbeatAggregator has zero Tokio runtime metrics integration — health collector is blind to executor state

**File:** `neotrix-core/src/unified/core/nt_core_heartbeat.rs:32-79`

**Severity:** HIGH

**Description:** The `HeartbeatAggregator` is a synchronous `HashMap<String, ComponentHealth>` with no tokio integration. It provides a static `report()` method but never spawns a monitoring task, never polls `RuntimeMetrics`, and never exposes live system health to GWT attention routing. Per tokio-metrics best practice, runtime health should be continuously sampled via `RuntimeMonitor::intervals()` to detect task starvation, queue backlogs, and worker contention. The GWT AttentionManager cannot receive runtime-level health signals (global_queue_depth, busy_ratio, budget_forced_yield_count) because HeartbeatAggregator has no async polling loop. The "unified system health signal collector" (AGENTS.md:97) is blind to the async runtime it runs on.

**Source:** tokio-metrics RuntimeMonitor, tokio::runtime::RuntimeMetrics docs

---

### D-TMON-002: 30+ background handlers spawned without TaskMonitor::instrument() — no per-handler poll/scheduling metrics

**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738-888`

**Severity:** HIGH

**Description:** The `spawn_handler!` macro (run.rs:734-767) spawns each of 30+ background handlers as bare `tokio::spawn(async move { ... })` without wrapping in `tokio_metrics::TaskMonitor::instrument()`. None of the handlers (save, consolidate, goal, consciousness_tick, telemetry, wisdom, game_training, etc.) have per-handler poll count, busy time, idle time, or scheduling delay metrics. A stalled handler (e.g., `handle_absorption` blocked on KB write for minutes) would be invisible to the health system. Per tokio-metrics best practices, each distinct handler type should have its own `TaskMonitor` instance to enable per-endpoint diagnostics. The `spawn_handler!` macro is the single highest-leverage integration point — adding `TaskMonitor::instrument()` inside the macro would instrument all handlers in one change.

**Source:** tokio-metrics TaskMonitor::instrument; tokio-console never-yielded warning

---

### D-TMON-003: CognitiveLoadMonitor is purely synthetic — computes load without reading actual RuntimeMetrics

**File:** `neotrix-core/src/unified/core/nt_core_consciousness/cognitive_load.rs:49-153`

**Severity:** HIGH

**Description:** `CognitiveLoadMonitor` computes load from abstract `record_step(load)` calls with no connection to actual tokio runtime metrics. It tracks `thinking_budget` via a simulated recharge formula (line 89-91) but never reads `RuntimeMetrics::total_busy_duration`, `global_queue_depth`, or `budget_forced_yield_count`. The "mode" transitions (Fast/Balanced/Deep) are based on synthetic load values, not on real worker thread utilization or task starvation signals. A system could be starving 200 tasks in the global queue while `CognitiveLoadMonitor` reports "Deep" mode. The monitor should consume `RuntimeMonitor::intervals()` to ground its mode transitions in actual executor health.

**Source:** tokio-metrics RuntimeMetrics, task budget monitoring

---

### D-TMON-004: ParallelExecutor silently swallows JoinHandle errors — no task failure tracking or metrics

**File:** `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/executor.rs:37-44`

**Severity:** MEDIUM

**Description:** In `ParallelExecutor::execute()` (line 41), failed JoinHandle results (`Err(_)`) are silently discarded with `if let Ok(res) = handle.await`. No error count, no metric, no log. Combined with zero TaskMonitor instrumentation, this means: (a) task panics are invisible, (b) task abort/cancellation is invisible, (c) no metric distinguishes "task completed" from "task panicked." The Dark Forest axiom requires that tasks which fail produce observable signals, not be silently dropped. Additionally, the `ExecMode::Parallel` branch spawns tasks sequentially and awaits each immediately (line 41), eliminating all concurrency — `tokio::spawn` overhead is wasted with no parallelism.

**Source:** tokio-metrics TaskMonitor, JoinHandle docs, Dark Forest axiom

---

### D-TMON-005: No tokio_unstable cfg or RuntimeMetrics enablement anywhere in codebase

**File:** entire codebase (zero matches for `tokio_unstable`, `RuntimeMetrics`, `tokio_metrics`)

**Severity:** HIGH

**Description:** The codebase has zero references to `tokio-metrics`, `console-subscriber`, or `tokio_unstable`. The consciousness runtime cannot observe: (a) `global_queue_depth` (is the runtime saturated?), (b) `worker_busy_duration` (is CPU bound?), (c) `blocking_queue_depth` (is spawn_blocking overwhelming the pool?), (d) `live_tasks_count` (how many tasks are alive?), (e) `steal_count` (is work distribution balanced?). Tokio's `RuntimeMetrics` requires the `tokio_unstable` feature flag and `TOKIO_UNSTABLE=1` environment variable. Without this, NeoTrix has zero visibility into executor health. The `AsyncSafetyWrapper` at `nt_meta_async_safety.rs` addresses spawn_blocking correctness but has zero runtime observability.

**Source:** tokio-metrics RuntimeMonitor, tokio::runtime::RuntimeMetrics docs

---

### D-TMON-006: No spawn_blocking pool monitoring — blocking thread exhaustion undetected

**File:** `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/factory.rs:1292-1295`, `selection.rs:362-365`, `nt_memory_api.rs:369-403`

**Severity:** MEDIUM

**Description:** Multiple `tokio::task::spawn_blocking` calls exist (factory.rs:1295, selection.rs:365, nt_io_web/tiles.rs:79, nt_memory_api.rs:369,403, free_catalog.rs:89) but there is no monitoring of `RuntimeMetrics::num_blocking_threads`, `worker_blocking_queue_depth`, or idle blocking thread count. Tokio's default blocking pool is 512 threads. If all are occupied (e.g., by slow `reqwest::blocking` calls in catalog refresh), subsequent `spawn_blocking` calls queue silently and block. No alert or metric tracks this. Production incidents from the Tokio sharded queue issue (PR #7757) show exactly this: "queue_wait_us=7441829" (7.4 seconds of queue wait for a blocking task).

**Source:** OpenTelemetry blocking thread monitoring, tokio RuntimeMetrics

---

### D-TMON-007: EventBus broadcast lag detected but not diagnosed — lag events silently discarded

**File:** `neotrix-core/src/neotrix/nt_core_event_bus.rs:387-389`

**Severity:** MEDIUM

**Description:** When `RecvError::Lagged(n)` occurs (event_bus.rs:387), the system logs `"[event-bus:{}] lagged {} events"` but: (1) no counter tracks total lagged events over time, (2) no alert threshold triggers when lag exceeds a critical level, (3) the lagged event count `n` is not fed into HeartbeatAggregator or any metric. Per tokio-console documentation, a broadcast channel lag indicates the producer is outpacing consumers — this is a direct signal of executor overload or handler starvation that NeoTrix silently discards. The EventBus subscriber tasks (line 362: `tokio::spawn(async move { loop { rx.recv().await ... } })`) could themselves be victims of worker starvation, but without `RuntimeMetrics::worker_noop_count` or `busy_ratio`, there is no way to distinguish subscriber starvation from publisher backpressure.

**Source:** tokio-console, hotpath runtime diagnostics

---

### D-TMON-008: Shutdown race — 5s hard abort deadline with no per-task progress reporting

**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:23-64`

**Severity:** MEDIUM

**Description:** The shutdown path aborts all handler tasks after a 5-second deadline without knowing if they are mid-critical-operation (e.g., mid-KB-write, mid-absorption). The `BackgroundLoop.handles` is a `Vec<JoinHandle<()>>` (not `JoinSet`) — dropping it does NOT cancel tasks. The manual abort loop drains handles and calls `abort_handle()` per-task, but the 5s budget is consumed by earlier handlers, leaving less time for later ones. With TaskMonitor, the shutdown logic could check `total_poll_count` and `total_slow_poll_duration` per handler to decide whether to wait longer or abort immediately. Without task-level metrics, the shutdown is a blind abort of unknown-state tasks.

**Source:** tokio-console task details, graceful shutdown patterns

---

### D-TMON-009: Zero poll_time_histogram or schedule_latency_histogram enabled — cannot detect executor saturation

**File:** entire codebase (no `enable_metrics_poll_time_histogram` or `enable_metrics_schedule_latency_histogram` found)

**Severity:** HIGH

**Description:** Tokio's `RuntimeMetrics` provides `poll_time_histogram` and `schedule_latency_histogram` — both require explicit enabling via `Builder::enable_metrics_poll_time_histogram()` and `Builder::enable_metrics_schedule_latency_histogram()`. NeoTrix never calls either. Without poll time histograms, the consciousness tree cannot detect: (1) tasks with bimodal poll distributions (indicating mixed I/O and compute), (2) tasks with increasing P99 poll times (indicating resource contention), (3) schedule latency spikes (indicating executor overload). The schedule-latency histogram is the only way to detect the delay between task wakeup and first poll. The `hotpath` crate (source 7) fills the "which code" gap that RuntimeMetrics leaves — but NeoTrix uses neither.

**Source:** tokio-metrics, tokio RuntimeMetrics docs

---

## Key Insights

1. **Observability gap is architectural, not incidental**: NeoTrix has zero `tokio-metrics` or `tokio-console` integration despite spawning 30+ long-lived background tasks. The `HeartbeatAggregator` provides component-level health but has zero visibility into the async runtime substrate. This is like having a health monitor that checks blood pressure but never looks at the heart rate.

2. **The `spawn_handler!` macro is the single highest-leverage integration point**: Since all 30+ background handlers use the same macro (run.rs:734-767), adding `TaskMonitor::instrument()` and task naming inside this macro would instrument all handlers in one change — maximum coverage for minimum code churn.

3. **CognitiveLoadMonitor is a fiction without runtime grounding**: The monitor's Fast/Balanced/Deep mode transitions are driven by synthetic `record_step(load)` values, not actual executor metrics. A system with 200 queued tasks and 100% busy workers could still report "Deep" mode. This is the consciousness architecture's most dangerous blind spot — it makes load-based decisions on fabricated data.

4. **spawn_blocking pool is an invisible time bomb**: 6+ call sites use `spawn_blocking` across provider catalog refresh, KB operations, and web tiles, all sharing Tokio's default 512-thread blocking pool. No monitoring detects when the pool is saturated. A single slow `reqwest::blocking` call can cascade into system-wide async starvation with no diagnostic signal.

5. **Budget forced yields are the #1 missing diagnostic**: Tokio's `budget_forced_yield_count` (available via RuntimeMetrics with `tokio_unstable`) directly indicates compute-heavy tasks that should be offloaded to `spawn_blocking`. NeoTrix's background loop handlers and parallel executor tasks never check this metric. The cooperative scheduling budget is invisible.

6. **The HeartbeatAggregator should be extended, not replaced**: The existing `HeartbeatAggregator` (nt_core_heartbeat.rs:32-79) already provides the aggregation pattern. Extending it with `RuntimeMetrics` fields (`global_queue_depth`, `busy_ratio`, `budget_forced_yield_count`, `blocking_queue_depth`) and a `RuntimeMonitor::intervals()` polling loop would provide runtime health signals to the consciousness tree with minimal architectural change.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 9 |
| Sources consulted | 10 |
| Files examined | 4 key source files + 85+ grep matches |
