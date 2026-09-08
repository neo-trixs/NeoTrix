# Agent 2: Async Task Monitoring (Batch 871)

## Sources
1. https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view — Tokio runtime metrics with OpenTelemetry: `RuntimeMonitor`, `RuntimeMetrics` fields (global_queue_depth, total_busy_duration, mean_poll_duration, budget_forced_yield_count), worker thread utilization, task starvation detection, budget monitoring.
2. https://github.com/tokio-rs/tokio-metrics — Official `tokio-metrics` crate: `TaskMonitor`, `RuntimeMonitor`, per-task and per-runtime metrics (instrumented_count, mean_poll_duration, max_poll_duration, total_polls_count, budget_forced_yield_count), `RuntimeMetricsReporterBuilder`.
3. https://github.com/tokio-rs/console — Tokio Console: real-time async debugging, task lifecycle states (RUNNING/IDLE/SCHED/DONE), poll duration histograms, self-wake detection, resource tracking.
4. https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html — `tokio::runtime::RuntimeMetrics`: num_alive_tasks, worker_park_count, worker_total_busy_duration, global_queue_depth — the raw API NeoTrix does not use.
5. https://sesamedisk.com/tokio-async-rust-2026 — Production anti-patterns: blocking in async, uncontrolled task spawning, resource leaks (forgotten JoinHandles, unbounded channels), scheduler starvation.
6. https://rustify.rs/articles/rust-async-runtimes-tokio-vs-async-std-2026 — Common mistakes: calling blocking functions inside async tasks, task lifecycle understanding, `Pin<Box<dyn Future>>` machinery.

## Defects

**D-TMON-001: No tokio-metrics integration — zero runtime observability** | `neotrix-core/Cargo.toml:75-194` | high | Source: tokio-metrics, OpenTelemetry monitoring guide
NeoTrix has zero dependency on `tokio-metrics` (v0.5) or `console-subscriber`. The `HeartbeatAggregator` (nt_core_heartbeat.rs:32) is a pure in-memory HashMap with no tokio runtime instrumentation. There is no `RuntimeMonitor`, no `TaskMonitor`, no `RuntimeMetrics` collection. The system cannot observe: task spawning rates, worker thread utilization, queue depths, poll durations, or budget exhaustion at the runtime level. All 30+ `spawn_handler!` tasks in `run.rs:734-829` are fire-and-forget with no metrics.

**D-TMON-002: Background loop handlers lack per-task lifecycle tracking** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:734-829` | high | Source: tokio-console task lifecycle, OpenTelemetry task tracking
The `spawn_handler!` macro (run.rs:734) spawns each handler as a bare `tokio::spawn` with no `TaskMonitor::instrument()`, no tracing spans, and no task naming. Tokio Console identifies tasks by name and tracks RUNNING/IDLE/SCHED/DONE states + poll histograms. NeoTrix's 30+ background handlers (save, consolidate, goal, curiosity, evolution, etc.) are invisible — a stalled handler (e.g., `handle_crawl_queue`) would only be detected by its heartbeat timeout, not by runtime state.

**D-TMON-003: HeartbeatAggregator is a static snapshot, not a time-series** | `neotrix-core/src/unified/core/nt_core_heartbeat.rs:32-79` | medium | Source: OpenTelemetry metrics architecture, tokio-metrics RuntimeIntervals
`HeartbeatAggregator::report()` returns a single `HealthReport` with no time-series history. `tokio-metrics` provides `RuntimeIntervals` — an iterator yielding delta metrics per interval with elapsed time, enabling trend analysis. NeoTrix's aggregator has no `elapsed` tracking, no delta computation, no `busy_ratio()`, and no `mean_polls_per_park()`. The ConsciousnessTree's `DataFoundation` only tracks `crawl_queue_depth` (types.rs:57) — not runtime queue depth.

**D-TMON-004: CognitiveLoadMonitor is synthetic, not grounded in Tokio runtime data** | `neotrix-core/src/unified/core/nt_core_consciousness/cognitive_load.rs:49-153` | high | Source: tokio-metrics RuntimeMetrics, task budget monitoring
`CognitiveLoadMonitor` computes load from abstract `record_step(load)` calls with no connection to actual tokio runtime metrics. It tracks `thinking_budget` via a simulated recharge formula (line 89-91) but never reads `RuntimeMetrics::total_busy_duration`, `global_queue_depth`, or `budget_forced_yield_count`. The "mode" transitions (Fast/Balanced/Deep) are based on synthetic load values, not on real worker thread utilization or task starvation signals. A system could be starving 200 tasks in the global queue while `CognitiveLoadMonitor` reports "Deep" mode.

**D-TMON-005: ParallelExecutor spawns tasks without abort/lifecycle management** | `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/executor.rs:37-44` | medium | Source: tokio-console resource tracking, SesameDisk resource leak patterns
`ParallelExecutor::execute()` spawns tasks via `tokio::spawn` (line 37) and silently drops the `JoinHandle` if `handle.await` fails (line 41-43: `if let Ok(res) = handle.await`). Per tokio-console, dropped JoinHandles without abort are a resource leak — the task continues running but is untracked. The executor also has no timeout, no max-concurrency guard, and no metric for spawned vs completed tasks. Under burst load this creates unbounded task growth.

**D-TMON-006: No spawn_blocking pool monitoring — blocking thread exhaustion undetected** | `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/factory.rs:1292-1295`, `selection.rs:362-365` | medium | Source: OpenTelemetry blocking thread monitoring, tokio RuntimeMetrics
Multiple `tokio::task::spawn_blocking` calls exist (factory.rs:1295, selection.rs:365, nt_io_web/tiles.rs:79, nt_memory_api.rs:369,403) but there is no monitoring of `RuntimeMetrics::num_blocking_threads`, `worker_blocking_queue_depth`, or idle blocking thread count. Tokio's default blocking pool is 512 threads. If all are occupied (e.g., by slow `reqwest::blocking` calls in catalog refresh), subsequent `spawn_blocking` calls queue silently and starve. No alert or metric tracks this.

**D-TMON-007: EventBus subscribe tasks have no health monitoring** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:359-396` | low | Source: tokio-console resource list, task state tracking
`subscribe_layer()` spawns a tokio task per layer (L1-L9) with an infinite loop receiving events. These tasks have no heartbeat, no watchdog, and no metric. If a subscriber task panics or gets stuck (e.g., on `RecvError::Lagged` with massive lag), the layer silently loses event monitoring. The task state (RUNNING/IDLE/DONE) is invisible. Tokio Console's resource list would show these as unmonitored resources.

**D-TMON-008: Shutdown race — 5s deadline with no per-task progress reporting** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:23-64` | medium | Source: tokio-console task details, graceful shutdown patterns
The shutdown path (handlers.rs:48-61) uses a single global 5-second deadline for all handlers. If any handler is mid-KB-write or mid-absorption when shutdown fires, it gets `abort()`ed. Tokio Console's `task_details` view provides per-task poll duration histograms and scheduled-time distributions — data that would inform a smarter shutdown (abort idle tasks first, give busy tasks more time). NeoTrix has no per-task busy/idle state to make this decision.

## Key Insights

1. **The fundamental gap**: NeoTrix runs 30+ persistent async tasks via `spawn_handler!` but has zero tokio runtime instrumentation. The entire system is opaque at the executor level — a task stall manifests only as a missed heartbeat, not as a detectable runtime anomaly.

2. **CognitiveLoadMonitor is decoupled from reality**: It models cognitive load through abstract `record_step()` calls but never grounds its assessments in actual tokio metrics (queue depth, busy duration, poll count). This means the consciousness tree's "mode switching" (Fast/Balanced/Deep) cannot respond to real async runtime pressure.

3. **HeartbeatAggregator is a point-in-time snapshot**: Unlike `tokio-metrics`'s `RuntimeIntervals` (delta-based time series with elapsed tracking), NeoTrix's health aggregator stores only the latest state with no history, trend analysis, or delta computation. This prevents detecting gradual degradation.

4. **No task-level metrics for the 30+ background handlers**: The `spawn_handler!` macro produces anonymous tasks. Without `TaskMonitor::instrument()` or `tokio::task::Builder::name()`, stalled handlers are indistinguishable from healthy ones in runtime diagnostics.

5. **Blocking pool is a blind spot**: Multiple `spawn_blocking` calls across the codebase (provider catalog refresh, KB operations, web tiles) share Tokio's default 512-thread blocking pool with no monitoring of utilization or queue depth. A single slow blocking operation can cascade into system-wide async starvation.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Sources consulted | 6 |
| Files analyzed | 15+ |
