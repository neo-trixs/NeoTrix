# Agent 1: Async Runtime Metrics (Batch 876)

## Sources
- [tokio-metrics crate (docs.rs)](https://docs.rs/tokio-metrics/latest/tokio_metrics/) - RuntimeMonitor, TaskMonitor, RuntimeMetrics API
- [tokio-console (GitHub)](https://github.com/tokio-rs/console) - Async runtime debugger with lint warnings (self-wakes, lost-waker, never-yielded, auto-boxed-future, large-future)
- [OneUptime: Monitor Tokio Runtime Metrics with OpenTelemetry](https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view) - Complete OTel integration guide
- [DeepWiki: Runtime Monitoring (tokio-metrics)](https://deepwiki.com/tokio-rs/tokio-metrics/4-runtime-monitoring) - Architecture, stable vs unstable metrics, worker state tracking
- [DeepWiki: Task Monitoring (tokio-metrics)](https://deepwiki.com/tokio-rs/tokio-metrics/3-task-monitoring) - TaskMonitor instrumentation, derived metrics, threshold config
- [Rust Zero to Hero: Tokio Runtime Tuning for Production](https://rustz2h.com/chapter_07_mastering_async_rust_and_tokio/series_01_tokio_runtime_internals_and_tasks/tokio_runtime_tuning_production) - Production tuning, worker thread count, load shedding
- [hotpath.rs: Tokio Runtime Metrics](https://hotpath.rs/tokio_runtime) - Per-worker utilization, queue depth, hotpath TUI integration
- [opentelemetry-instrumentation-tokio (GitHub)](https://github.com/sandhose/opentelemetry-instrumentation-tokio) - Auto-observe runtime with OTel metrics (23 metrics: 4 always-on + 19 with tokio_unstable)
- [Leapcell: Tokio, Futures, and Beyond](https://de.leapcell.io/blog/en/tokio-futures-async-rust) - Runtime configuration, spawn_blocking patterns, GC impact
- [Medium: More Async Debugging Scenarios with Tokio Console](https://medium.com/rustaceans/more-async-debugging-scenarios-with-tokio-console-3bcebf39c981) - Waker storm, exotic failure modes

## Defects

### D-METRIC-001: HeartbeatAggregator has zero tokio-metrics or tokio-console integration — blind to runtime health
**File:** `neotrix-core/src/unified/core/nt_core_heartbeat.rs:32-79`
**Severity:** HIGH
**Source:** tokio-metrics docs.rs (RuntimeMonitor::intervals), DeepWiki runtime monitoring, OneUptime OTel guide

**Description:** The `HeartbeatAggregator` is a plain `HashMap<String, ComponentHealth>` with no Tokio integration. It provides `record()` and `report()` but never spawns a monitoring task, never polls `RuntimeMetrics`, and never exposes live async runtime health to GWT attention routing. Per tokio-metrics best practice, runtime health should be continuously sampled via `RuntimeMonitor::intervals()` to detect task starvation, queue backlogs, and worker contention. The GWT AttentionManager cannot receive runtime-level health signals (`global_queue_depth`, `busy_ratio`, `budget_forced_yield_count`, `blocking_queue_depth`) because HeartbeatAggregator has no async polling loop. The "unified system health signal collector" (AGENTS.md:97) is blind to the async runtime it runs on.

**Impact:** The consciousness tree cannot detect executor starvation, worker thread saturation, or blocking thread pool exhaustion. A system could have 30+ handlers starving in the global queue while HeartbeatAggregator reports all components "Healthy". The `busy_ratio` (ratio of busy duration to elapsed time) — the single most important runtime health signal — is never computed.

### D-METRIC-002: No `tokio_unstable` cfg flag anywhere in the codebase — 19 critical metrics permanently unavailable
**File:** N/A (build system level — `.cargo/config.toml` missing)
**Severity:** HIGH
**Source:** tokio-console README, opentelemetry-instrumentation-tokio docs, Rust Zero to Hero guide

**Description:** The `tokio_unstable` cfg flag is never set in the NeoTrix codebase. This flag enables 19 additional metrics including `total_noop_count`, `total_steal_count`, `total_steal_operations`, `poll_time_histogram`, `budget_forced_yield_count`, `io_driver_ready_count`, per-worker `mean_poll_duration`, and the `blocking_queue_depth`. Without this flag, tokio-console cannot detect self-wakes, lost-wakers, never-yielded tasks, auto-boxed-futures, or large-futures — all five of its built-in lint warnings. The entire diagnostic capability of tokio-console is disabled at build time.

**Impact:** NeoTrix's 30+ background handlers (run.rs:738-894) run with zero compile-time diagnostic coverage for the most common async antipatterns: tasks that wake themselves in hot loops (starvation), tasks dropped without being woken (lost resources), and tasks that never yield (blocking the executor).

### D-METRIC-003: EventBus broadcast lag events silently discarded — no lag counter, no alert threshold, no metric feed
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:905`
**Severity:** HIGH
**Source:** tokio-console lint system, Medium: More Async Debugging (waker storm), OneUptime starvation detection

**Description:** When `RecvError::Lagged(n)` occurs, the system logs `"[bg] event_bus consumer lagged {} events"` but: (1) no counter tracks total lagged events over time, (2) no alert threshold triggers when lag exceeds a critical level, (3) the lagged event count `n` is not fed into HeartbeatAggregator or any metric. Per tokio-console documentation, a broadcast channel lag indicates the producer is outpacing consumers — this is a direct signal of executor overload or handler starvation that NeoTrix silently discards. The EventBus subscriber task (line 894: `tokio::spawn(async move { loop { event_rx.recv().await ... } })`) could itself be a victim of worker starvation, but without `RuntimeMetrics::worker_noop_count` or `busy_ratio`, there is no way to distinguish subscriber starvation from publisher backpressure.

**Impact:** During high event throughput (SEAL pipeline exploration, crawl queue bursts, knowledge ingestion), events are dropped without any feedback to the consciousness tree. The "waker storm" pattern (Medium) — where tasks wake themselves excessively, consuming all worker time — would be invisible to NeoTrix's monitoring.

### D-METRIC-004: 30+ background handlers spawned with zero per-task metrics — cannot detect starvation or slow handlers
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738-894`
**Severity:** MEDIUM
**Source:** tokio-metrics TaskMonitor, DeepWiki task monitoring, OneUptime individual task tracking

**Description:** The `spawn_handler!` macro (line 738) spawns 30+ independent `tokio::spawn` tasks with `tokio::time::interval` tickers, but none are instrumented with `tokio_metrics::TaskMonitor`. Per tokio-metrics best practice, each spawned task should be wrapped with `monitor.instrument(task)` to track: `total_poll_count`, `total_slow_poll_count`, `total_idle_duration`, `total_scheduled_duration`, `mean_short_delay_duration`, `mean_long_delay_duration`. The `slow_poll_threshold` default (50μs) and `long_delay_threshold` (50μs) would immediately surface handlers that are blocking the executor or experiencing scheduling starvation. Currently there is zero visibility into which of the 30+ handlers is the slowest, which has the longest scheduling delay, or which is consuming the most poll time.

**Impact:** When a handler like `handle_crawl_queue` or `handle_novel_ingest` takes 30+ seconds, there is no metric to correlate with heartbeat degradation. The consciousness tree's `LoopReadyScore` (run.rs:465) computes readiness from boolean signals but cannot attribute slowness to a specific handler.

### D-METRIC-005: HeartbeatAggregator uses synchronous HashMap — cannot participate in cooperative scheduling without blocking
**File:** `neotrix-core/src/unified/core/nt_core_heartbeat.rs:32-79`
**Severity:** MEDIUM
**Source:** Rust Zero to Hero: spawn_blocking pattern, Leapcell: Async/Sync boundary, OneUptime production best practices

**Description:** The `HeartbeatAggregator` is a plain `HashMap<String, ComponentHealth>` with no `Arc`, no `Send + Sync` bounds, and no async interface. Callers must call `record()` and `report()` synchronously. In the consciousness architecture, health aggregation feeds GWT attention modulation (per `CONTEXT.md`: "Single fact source for GWT attention modulation"). If health checks are called from an async context (e.g., a background handler tick), they must be either in a `spawn_blocking` wrapper or on a blocking thread. Currently there is no evidence any handler wraps health aggregation in `spawn_blocking`, meaning synchronous health checks block the Tokio worker thread. Per the Leapcell guide: "Forcibly blocking an Async task in a Sync function will occupy Tokio worker threads, causing starvation for other tasks."

**Impact:** A slow health report operation (e.g., cloning 11+ ComponentHealth entries per tick) holds a Tokio worker thread during the entire operation, reducing available parallelism for all other async tasks on that thread.

### D-METRIC-006: No runtime metrics dashboard or Prometheus/OpenTelemetry export — observability gap is architectural
**File:** N/A (missing integration layer)
**Severity:** MEDIUM
**Source:** OneUptime OTel integration, opentelemetry-instrumentation-tokio, hotpath.rs

**Description:** NeoTrix has `tracing` and `tracing-subscriber` but no `tokio-metrics`, `console-subscriber`, `opentelemetry-instrumentation-tokio`, or Prometheus exporter. The `tokio_unstable` cfg flag is never set. The system runs 30+ background handlers (run.rs:738-894) with zero visibility into worker thread utilization, queue depths, poll durations, or task scheduling latencies. Per the OneUptime guide, production Tokio monitoring requires: (1) runtime metrics collection via `RuntimeMonitor::intervals()`, (2) export via OTLP/Prometheus, (3) dashboard with task throughput, active tasks, task duration, worker utilization, and scheduling delay panels. NeoTrix has none of these. The `opentelemetry-instrumentation-tokio` crate provides 23 metrics (4 always-on + 19 with tokio_unstable) with zero configuration — it could be added with 3 lines of code.

**Impact:** Production debugging requires manual log analysis rather than systematic metric-driven observability. Performance regressions (e.g., handler starvation after a code change) cannot be detected until they cause user-visible failures.

### D-METRIC-007: No task naming or labeling — tokio-console and metrics cannot distinguish handler tasks
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738-894`
**Severity:** LOW
**Source:** tokio-console task naming, OneUptime cardinality control, tokio-metrics TaskMetricsReporterBuilder

**Description:** The `spawn_handler!` macro spawns tasks without calling `tokio::task::Builder::new().name(...)`. Tokio-console and tokio-metrics both support task naming for identification — without names, all tasks appear as anonymous futures, making it impossible to correlate metric signals with specific handlers. The `TaskMetricsReporterBuilder::new(|name| ...)` in tokio-metrics expects a naming function, but since tasks have no names, all metrics collapse into a single anonymous bucket. Per OneUptime best practice: "Use task types or modules instead of task IDs" for cardinality control — but NeoTrix uses neither.

**Impact:** When tokio-console is eventually enabled, all 30+ handlers will appear as unnamed tasks, making it impossible to identify which handler is slow, which is starved, or which has lost its waker.

## Key Insights

1. **The observability gap is architectural, not incidental.** NeoTrix has zero `tokio-metrics` or `tokio-console` integration despite spawning 30+ long-lived background tasks. The `HeartbeatAggregator` provides component-level health but has zero visibility into the async runtime substrate that drives all components. This is like having a health monitor that checks blood pressure but never looks at the heart rate.

2. **The fix is minimal-effort, maximum-signal.** Adding `console-subscriber = "0.2"` to Cargo.toml and `console_subscriber::init()` to main would immediately enable tokio-console's five built-in lint warnings. Adding `opentelemetry-instrumentation-tokio` would export 23 runtime metrics with zero configuration. Both are additive, non-breaking changes.

3. **The HeartbeatAggregator should be extended, not replaced.** The existing `HeartbeatAggregator` (nt_core_heartbeat.rs:32-79) already provides the aggregation pattern. Extending it with `RuntimeMetrics` fields (`global_queue_depth`, `busy_ratio`, `mean_poll_duration`, `budget_forced_yield_count`, `blocking_queue_depth`) and a `RuntimeMonitor::intervals()` polling loop would provide runtime health signals to the consciousness tree with minimal architectural change.

4. **The broadcast lag silent discard is a production hazard.** The `RecvError::Lagged(n)` at run.rs:905 is a direct signal of executor overload or handler starvation. Silently discarding it means the consciousness tree cannot detect that events are being lost — a critical failure mode for a system that relies on event-driven coordination across 9 domains.

5. **tokio-console's lint system is purpose-built for NeoTrix's failure modes.** The five warnings (self-wakes, lost-waker, never-yielded, auto-boxed-future, large-future) directly map to common async antipatterns in complex systems like NeoTrix: tasks that wake themselves in hot loops, tasks dropped without cleanup, tasks that never yield control, and tasks with excessive stack usage. These are precisely the failure modes that would emerge in a system with 30+ concurrent handlers and no runtime monitoring.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 7 |
| Sources consulted | 10 |
| Severity breakdown | 3 HIGH / 3 MEDIUM / 1 LOW |
