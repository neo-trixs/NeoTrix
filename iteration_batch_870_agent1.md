# Agent 1: Async Runtime Metrics (Batch 870)

## Sources

1. **tokio-console docs.rs** — https://docs.rs/tokio-console/latest/tokio_console/ — Task states (RUNNING/IDLE/SCHED/DONE), resource tracking, async ops, warnings system (self-wakes, lost-waker, never-yielded, auto-boxed-future, large-future)
2. **tokio-rs/console GitHub** — https://github.com/tokio-rs/console — gRPC wire protocol, console-subscriber Layer, tracing instrumentation, `tokio=trace,runtime=trace` filter requirements
3. **tokio-rs/tokio-metrics GitHub** — https://github.com/tokio-rs/tokio-metrics — RuntimeMonitor (workers_count, total_busy_duration, global_queue_depth, live_tasks_count, mean_poll_duration, budget_forced_yield_count), TaskMonitor (mean_first_poll_delay, mean_scheduled_duration, long_delay_ratio), metrics-rs integration, Prometheus export
4. **docs.rs/tokio-metrics** — https://docs.rs/tokio-metrics/latest/tokio_metrics/ — RuntimeMetrics (41 fields), TaskMetrics, PollTimeHistogram, HistogramBucket, RuntimeIntervals, TaskIntervals, TaskMonitorCore, metrics-rs-integration feature
5. **docs.rs/tokio RuntimeMetrics** — https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html — worker_park_count, worker_steal_count, worker_overflow_count, budget_forced_yield_count, poll_time_histogram, schedule_latency_histogram, io_driver_ready_count
6. **OneUptime blog** — https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view — TokioMetricsCollector pattern, StarvationDetector (max_poll_duration 50ms, max_queue_depth 1000), BudgetMetricsCollector, WorkerMetricsCollector, OpenTelemetry integration
7. **Tokio blog announcing tokio-metrics** — https://tokio.rs/blog/2022-02-announcing-tokio-metrics — TaskMonitor per-endpoint instrumentation, `tokio_unstable` flag requirement, metrics-rs export to Prometheus/Grafana
8. **Medium: Deep Dive into Tokio Console** — https://medium.com/rustaceans/a-deep-dive-into-tokio-console-6af9bfa870ee — poll count/busy/idle ratio analysis, high scheduled time diagnosis, resource contention detection, task leak identification, baseline establishment

## Defects

**D-METRIC-001: Zero async runtime metrics instrumentation** | `neotrix-core/Cargo.toml:84-85` | HIGH | tokio-console/tokio-metrics not in dependencies
NeoTrix has `tracing` and `tracing-subscriber` but no `tokio-metrics`, `console-subscriber`, or `tokio-console` crate. The `tokio_unstable` cfg flag is never set. The system runs 30+ background handlers (run.rs:779-819) with zero visibility into worker thread utilization, queue depths, poll durations, or task scheduling latencies. The HeartbeatAggregator (nt_core_heartbeat.rs:32-79) is a trivial HashMap with no Tokio runtime metric integration — it tracks component status strings but cannot detect executor starvation, queue buildup, or blocking thread pool exhaustion.

**D-METRIC-002: No task naming — background loop tasks invisible to diagnostics** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:734-767` | HIGH | tokio::task::Builder::name() never used
The `spawn_handler!` macro at run.rs:738 calls `tokio::spawn(async move { ... })` without setting task names via `tokio::task::Builder::name()`. All 30+ background handlers (save, consolidate, goal, crystallization, consciousness_tick, etc.) appear as unnamed tasks in tokio-console, making it impossible to identify which handler is causing starvation, self-waking, or long scheduling delays. Tokio Console's task list view depends on names for meaningful diagnosis.

**D-METRIC-003: No task-level metrics (TaskMonitor) for background handlers** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:734-819` | HIGH | No TaskMonitor wrapping any spawned handler
Each `spawn_handler!` invocation spawns a bare `tokio::spawn()`. None are wrapped with `tokio_metrics::TaskMonitor::instrument()`. This means: (1) no per-handler poll count, busy time, idle time, or scheduling delay metrics; (2) no detection of handlers that never yield (never-yielded warning); (3) no detection of handlers that self-wake excessively (self-wakes warning); (4) no detection of handlers dropped without being woken (lost-waker warning). The OneUptime starvation detector pattern (max_poll_duration 50ms, max_queue_depth 1000) is completely absent.

**D-METRIC-004: HeartbeatAggregator lacks runtime health signal integration** | `neotrix-core/src/unified/core/nt_core_heartbeat.rs:32-79` | MEDIUM | HeartbeatAggregator is pure application-level, no runtime metrics
The HeartbeatAggregator records component health as status strings (Healthy/Degraded/Unhealthy/Unknown) but has no fields for: `global_queue_depth`, `busy_ratio`, `mean_poll_duration`, `budget_forced_yield_count`, `io_driver_ready_count`, or `blocking_queue_depth`. The consciousness tree's health assessment cannot detect that the Tokio executor is overloaded, that worker threads are 100% busy, or that the blocking thread pool is saturated. The aggregate report (line 56-72) is a static snapshot with no time-decay, no trend detection, and no correlation with runtime metrics.

**D-METRIC-005: EventBus broadcast channel lag detected but not metered** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:387-388` | MEDIUM | Lag events logged but no counter/metric emitted
When `RecvError::Lagged(n)` occurs (event_bus.rs:387), the system logs `"[event-bus:{}] lagged {} events"` but: (1) no counter tracks total lagged events over time; (2) no alert threshold triggers when lag exceeds a critical level; (3) the lagged event count `n` is not fed into the HeartbeatAggregator or EventBus metrics. Per tokio-console documentation, a broadcast channel lag indicates the producer is outpacing consumers — this is a direct signal of executor overload or handler starvation that NeoTrix silently discards.

**D-METRIC-006: Runtime::new() creates isolated runtimes without metrics export** | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/mod.rs:585-992` | MEDIUM | Multiple `Runtime::new()` calls create unmonitored runtimes
At least 10+ locations create `tokio::runtime::Runtime::new()` (sandbox/mod.rs:585, 876, 895, 979, 992; sandbox/judge.rs:570, 590, 611; rule_api.rs:192; nt_memory_api.rs:600). Each creates an isolated Tokio runtime with default worker thread count, no metrics collection, and no runtime handle export. When sandbox execution blocks (e.g., cloud.run_code at sandbox/mod.rs:877), there is no way to detect the runtime is starved. The OpenTelemetry pattern (TokioMetricsCollector from OneUptime source) shows these runtimes should use `RuntimeMonitor::new(&handle)` to export metrics.

**D-METRIC-007: println! in async context — blocking stdout acquisition** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:919` | LOW | `println!("[bg] {} handlers spawned", self.handles.len())`
At run.rs:919, `println!()` is called from within an async context after spawning all handlers. Per tokio-console's warning system, `println!` acquires an exclusive lock on stdout and performs a synchronous write, which blocks the executor thread. While this is a one-shot call at startup, the pattern exists across 100+ locations in the codebase (config.rs:46-106, nt_file_ability.rs benchmarks, nt_core_capability_tree/cli.rs). Each `println!` in async context is a potential stall point that would be flagged by tokio-console's task-level monitoring.

**D-METRIC-008: No poll_time_histogram or schedule_latency_histogram enabled** | `neotrix-core/Cargo.toml` (missing config) | HIGH | No `enable_metrics_poll_time_histogram()` or `enable_metrics_schedule_latency_histogram()` call
Tokio's `RuntimeMetrics` (docs.rs/tokio) provides `poll_time_histogram` and `schedule_latency_histogram` — both require explicit enabling via `Builder::enable_metrics_poll_time_histogram()` and `Builder::enable_metrics_schedule_latency_histogram()`. NeoTrix never calls either. Without poll time histograms, the consciousness tree cannot detect: (1) tasks with bimodal poll distributions (indicating mixed I/O and compute); (2) tasks with increasing P99 poll times (indicating resource contention); (3) schedule latency spikes (indicating executor overload). The schedule-latency histogram (crate feature `schedule-latency`) is the only way to detect the delay between task wakeup and first poll.

**D-METRIC-009: No task budget monitoring — budget_forced_yield_count invisible** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:734-819` | HIGH | Budget exhaustion not tracked
Tokio's `budget_forced_yield_count` metric (RuntimeMetrics) counts how many times tasks were forced to yield after exhausting their cooperative scheduling budget. NeoTrix spawns 30+ long-running background handlers that each hold `h.lock().await` (a tokio::sync::Mutex) during their tick body (run.rs:745). If any handler's tick body takes too long, it will exhaust the budget and be force-yielded — but this event is never monitored. The OneUptime BudgetMetricsCollector pattern (`tokio.budget.exhausted` counter) is the recommended approach, and its absence means NeoTrix cannot detect tasks that are too CPU-intensive for cooperative scheduling.

**D-METRIC-010: No worker thread steal/overflow metrics — load imbalance undetected** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs` (all handlers) | MEDIUM | worker_steal_count, worker_overflow_count, worker_local_queue_depth not collected
Tokio's multi-threaded runtime uses work-stealing: when a worker's local queue overflows (`worker_overflow_count`), tasks spill to the global queue. High overflow counts indicate the local queue size is too small for the workload. High steal counts indicate uneven load distribution. NeoTrix spawns tasks from multiple sources (background handlers at run.rs:779-819, EventBus consumers at run.rs:894, proxy kernel at kernel.rs:173-256) without monitoring whether load is balanced across workers. The consciousness tree's health assessment cannot distinguish between "all workers busy" (healthy) and "one worker starved while others overflow" (unhealthy).

## Key Insights

1. **NeoTrix runs blind on async health**: With 30+ background handlers, 9 EventBus layer subscribers, and multiple proxy/crawl tasks, the system has zero visibility into Tokio runtime behavior. This is analogous to running a distributed system without Prometheus metrics — you only discover problems when they become catastrophic.

2. **HeartbeatAggregator is a missed integration point**: The existing HeartbeatAggregator (nt_core_heartbeat.rs) already provides the aggregation pattern but needs to be extended with RuntimeMetrics fields. A `RuntimeMonitor::intervals()` loop feeding into HeartbeatAggregator would provide runtime health signals to the consciousness tree.

3. **The `spawn_handler!` macro is the single integration point**: Since all 30+ background handlers use the same macro (run.rs:734-767), adding `TaskMonitor::instrument()` and task naming inside this macro would instrument all handlers in one change — maximum coverage for minimum code churn.

4. **Budget forced yields are the silent killer**: NeoTrix handlers acquire `tokio::sync::Mutex` locks (run.rs:745) and do KB/LLM/network work inside the lock. If any handler's work takes >500ms (Tokio's default budget), it gets force-yielded while still holding the lock. Without budget monitoring, this creates cascading starvation across all handlers competing for the same lock.

5. **The console-subscriber integration is trivially addable**: Adding `console_subscriber::init()` to the main entry point + enabling `tokio_unstable` in `.cargo/config.toml` would immediately expose all 30+ handlers to tokio-console's interactive TUI, providing real-time task state, resource contention, and warning diagnostics with minimal code change.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources consulted | 8 |
| Files analyzed | 25+ |
| Severity: HIGH | 5 |
| Severity: MEDIUM | 4 |
| Severity: LOW | 1 |
