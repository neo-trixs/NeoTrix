# Agent 1: Async Runtime Metrics (Batch 873)

## Sources
1. https://github.com/tokio-rs/tokio-metrics — tokio-metrics crate: RuntimeMonitor, TaskMonitor, RuntimeMetrics (worker counts, queue depth, poll counts, steal counts, busy duration, noop counts, budget forced yields, poll time histograms)
2. https://github.com/tokio-rs/console — tokio-console: live async debugging TUI, console-subscriber instrumentation, task warnings system (long-running tasks, self-waking tasks)
3. https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html — Tokio RuntimeMetrics API: num_workers, num_alive_tasks, global_queue_depth, worker_poll_count, worker_noop_count, worker_steal_count, busy_duration, poll_time_histogram, io_driver_fd counts
4. https://deepwiki.com/tokio-rs/tokio-metrics/4-runtime-monitoring — Runtime monitoring deep dive: worker thread utilization, task scheduling, queue depths, performance characteristics
5. https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view — OpenTelemetry + Tokio metrics integration: task spawning rates, worker thread utilization, queue depths, polling times, budget exhaustion detection
6. https://hotpath.rs/tokio_runtime — hotpath vs RuntimeMetrics comparison: RuntimeMetrics tells you *that* runtime is under load, not *which code* is responsible; correlates with function timings and future poll counts
7. https://reintech.io/blog/tokio-tutorial-2026-building-async-applications-rust — Tokio tutorial: common pitfalls (holding locks across await, forgetting to await), tokio-console setup
8. https://github.com/ibrahimcesar/async-inspect — async-inspect: state machine visibility, deadlock detection, variable inspection (complementary to tokio-console); shows what tokio-console DOESN'T show (which .await is blocked, variable values, deadlock detection)
9. https://tokio.rs/blog/2021-12-announcing-tokio-console — Tokio Console 0.1 announcement: warnings system (clippy for async), task self-wake detection, long-running-without-yield detection
10. https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html — RuntimeMetrics field reference: all stable and unstable metrics with definitions

## Defects

**D-METRIC-001: HeartbeatAggregator ignores Tokio runtime metrics entirely** | `neotrix-core/src/unified/core/nt_core_heartbeat.rs:32-79` | HIGH | Source: tokio-metrics, hotpath

The HeartbeatAggregator tracks component health via string-keyed `ComponentHealth` entries but has zero integration with `tokio::runtime::RuntimeMetrics` or `tokio_metrics::RuntimeMonitor`. It cannot detect worker thread starvation, queue depth saturation, noop wake storms, or budget exhaustion. The "unified system health signal collector" (AGENTS.md:97) is blind to the async runtime it runs on. Per tokio-metrics research, critical signals like `total_noop_count` (false-positive wakeups), `budget_forced_yield_count` (compute-heavy task starvation), and `global_queue_depth` (backpressure) are never collected. The aggregator should be the single fact source for runtime health but instead only sees application-level heartbeats, missing the executor layer entirely.

**D-METRIC-002: Leaked runtimes via std::mem::forget(rt) bypass metrics collection** | `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/factory.rs:1447,1462` | HIGH | Source: tokio-metrics, tokio-console

Factory creates independent Tokio runtimes and leaks them with `std::mem::forget(rt)` to avoid `BlockingPool::shutdown` deadlocks. These leaked runtimes have no `RuntimeMetrics` handle attached, no `RuntimeMonitor` polling them, and no console-subscriber instrumentation. Any task starvation, worker thread exhaustion, or queue saturation on these runtimes is invisible. The comment at line 1446 acknowledges the leak but doesn't address observability. With multiple leaked runtimes (sandbox, LLM gateway, consciousness bridge), NeoTrix has zero visibility into their executor health.

**D-METRIC-003: Multiple independent runtimes with no aggregate metrics** | Multiple files (sandbox/mod.rs:585, factory.rs:1438/1453, consciousness_core.rs:1868, llm.rs:1022-1110) | MEDIUM | Source: tokio-metrics, oneuptime OpenTelemetry guide

NeoTrix creates 5+ independent `tokio::runtime::Runtime::new()` instances across sandbox, LLM provider factory, consciousness core, and test harnesses. Each runtime spawns its own worker thread pool with no coordinated metrics. The OpenTelemetry integration guide explicitly warns: "If your application uses multiple Tokio runtimes, tag metrics with runtime identifiers to distinguish them." NeoTrix has no runtime tagging, no aggregate worker thread count, and no cross-runtime queue depth visibility. During peak load, multiple runtimes may independently saturate their worker pools with no global backpressure signal.

**D-METRIC-004: No budget_forced_yield monitoring — compute-heavy tasks silently starve executor** | `neotrix-core/src/unified/core/nt_core_heartbeat.rs` (absent) | HIGH | Source: tokio-metrics, oneuptime guide

Tokio's budget system forces tasks to yield after exhausting their poll budget (`budget_forced_yield_count` in RuntimeMetrics). This metric directly indicates compute-heavy tasks that are starving the executor. NeoTrix's HeartbeatAggregator has no tracking of this metric. The consciousness tick loop, SEAL pipeline phases, and KB operations could all be budget-exhausting without any detection. Per tokio-metrics research: "Budget exhaustion indicating compute-heavy tasks" is a critical runtime signal. Without it, the system cannot distinguish between I/O-bound latency and compute-bound starvation.

**D-METRIC-005: Event bus broadcast lag is detected but not diagnosed** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:387-389` | MEDIUM | Source: tokio-console, hotpath

The event bus catches `RecvError::Lagged(n)` and logs a warning, but has no mechanism to diagnose *why* lag occurred. Tokio-console's warning system detects patterns like tasks running without yielding, self-waking tasks, and long poll durations. The event bus subscriber at line 362 runs in a `tokio::spawn` loop that itself could be the victim of worker starvation — but without `RuntimeMetrics::worker_noop_count` or `busy_ratio`, there's no way to distinguish subscriber starvation from publisher backpressure. The lag event is a symptom; the cause is invisible.

**D-METRIC-006: AsyncSafetyWrapper uses string matching instead of runtime metrics** | `neotrix-core/src/unified/layers/meta/nt_meta/nt_meta_async_safety.rs:127-156` | MEDIUM | Source: tokio-metrics, tokio-console

The AsyncSafetyWrapper detects blocking-in-async patterns by checking if operation strings contain "blocking" or "reqwest::blocking" (line 132). This is a static string-match approach that cannot detect actual runtime violations. Per tokio-console research, runtime-level detection (monitoring `worker_noop_count`, `busy_duration`, poll time histograms) is the only reliable way to detect blocking-in-async at runtime. The wrapper's `check_safety()` is purely advisory and never triggered by actual metrics. Real violations (e.g., a long CPU-bound future that blocks a worker) are invisible.

**D-METRIC-007: No poll_time_histogram or schedule_latency_histogram enabled** | Entire codebase (no `enable_metrics_poll_time_histogram` found) | MEDIUM | Source: tokio-metrics, tokio RuntimeMetrics docs

Tokio provides `poll_time_histogram` (distribution of task poll durations) and `schedule_latency_histogram` (time between task notification and actual poll) — both require explicit opt-in via `enable_metrics_poll_time_histogram()` and `enable_metrics_schedule_latency_histogram()` on the runtime builder. Neither is enabled anywhere in NeoTrix. These histograms are the primary tool for identifying tasks with excessive poll times (hot loops, inefficient futures) and scheduling delays (worker starvation). Without them, the system cannot answer: "Which future is taking too long to poll?" or "Are tasks waiting too long before being scheduled?"

**D-METRIC-008: Scheduler heartbeat monitoring is decoupled from executor metrics** | `neotrix-core/src/unified/core/nt_core_scheduler/engine.rs:20-27,127-163` | LOW | Source: tokio-metrics, tokio-console

The scheduler engine has its own heartbeat monitoring (KiroCrew pattern) that tracks job liveness via `heartbeat_secs`/`last_heartbeat`. However, this is purely application-level — it detects "job X hasn't reported in 30s" but cannot detect "the executor worker thread running job X is saturated/starved/deadlocked." Per tokio-console research, the warnings system detects "tasks that have run for a very long time without yielding" and "tasks that have woken themselves more times than they've been woken by other tasks." The scheduler's heartbeat check would fire *after* the executor problem has already caused cascading delays, with no pre-failure signal.

## Key Insights

1. **Tokio runtime metrics are the missing observability layer**: NeoTrix's HeartbeatAggregator tracks application-level component health but is blind to the async executor it runs on. The tokio-metrics crate provides 30+ metrics (worker counts, queue depths, poll counts, steal counts, busy duration, noop counts, budget exhaustion, histograms) that would fill this gap with minimal code changes.

2. **Runtime leaks are a silent observability killer**: The `std::mem::forget(rt)` pattern in factory.rs creates invisible runtime instances. Each leaked runtime's worker threads, task queues, and I/O driver are completely unmonitored. This affects the LLM gateway and sandbox — two of the most latency-sensitive paths.

3. **Multiple runtimes need aggregate visibility**: With 5+ independent runtimes, NeoTrix needs either a single shared runtime (ideal) or a metrics aggregation layer that tags and combines metrics from all runtimes. The current approach creates blind spots where one runtime's saturation is invisible to others.

4. **Budget exhaustion is the #1 undetected async antipattern**: Tokio's cooperative scheduling budget forces yields to prevent starvation, but NeoTrix never tracks `budget_forced_yield_count`. This metric directly answers "are my compute-heavy tasks starving the executor?" — the most common production async performance issue.

5. **String-based safety checks are insufficient**: The AsyncSafetyWrapper's string matching for blocking patterns is a compile-time/lint-time heuristic, not a runtime detector. Real async safety requires metrics-based detection (noop counts, busy ratios, poll time histograms) that can catch violations at runtime.

6. **Console-subscriber integration is zero-cost in production**: `console-subscriber` adds tracing instrumentation that is only active when a console client connects. Enabling it in NeoTrix would provide live task monitoring without production overhead, but it requires `RUSTFLAGS="--cfg tokio_unstable"` at compile time.

7. **hotpath填补了RuntimeMetrics的"which code"空白**: tokio-metrics tells you *that* the runtime is under load; hotpath correlates it with *which futures, channels, locks, or functions* cause worker saturation. For NeoTrix's multi-domain architecture, this correlation is essential to attribute performance issues to specific domains (NT-CORE, NT-MIND, etc.).

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Sources consulted | 10 |
| Codebase files analyzed | 12+ |
