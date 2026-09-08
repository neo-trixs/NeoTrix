# Agent 2: Async Task Monitoring (Batch 872)

## Sources

1. **tokio-metrics crate docs** — `docs.rs/tokio-metrics` — TaskMonitor, RuntimeMonitor, TaskMetrics, RuntimeMetrics, metrics-rs integration
2. **tokio-metrics GitHub** — `github.com/tokio-rs/tokio-metrics` — TaskMonitor::instrument, RuntimeMonitor::intervals, Prometheus export
3. **console-subscriber GitHub** — `github.com/tokio-rs/console/console-subscriber` — tracing Layer for tokio-console, gRPC telemetry export
4. **console-subscriber docs** — `docs.rs/console-subscriber` — ConsoleLayer, DEFAULT_PUBLISH_INTERVAL, DEFAULT_RETENTION, Builder
5. **tokio::runtime::RuntimeMetrics** — `docs.rs/tokio` — RuntimeMetrics API: worker stats, queue depths, poll histograms, steal counts
6. **tokio-console docs** — `docs.rs/tokio-console` — Consumer UI for runtime instrumentation

## Defects

**D-TMON-001: HeartbeatAggregator has zero async task visibility** | `nt_core_heartbeat.rs:32-79` | HIGH | tokio-metrics, HeartbeatAggregator source

The HeartbeatAggregator is a synchronous HashMap-based health collector that tracks only component name→status. It has no integration with tokio-metrics TaskMonitor or RuntimeMonitor. This means the consciousness architecture cannot detect slow polls, scheduling delays, idle durations, or task overflow in any of its 20+ background handler tasks. A handler stuck in a slow poll (e.g., LLM call with unbounded latency) would be invisible to the health system. The aggregator should instrument background handlers via `TaskMonitor::instrument()` and expose `TaskMetrics` (slow_poll_ratio, mean_poll_duration, total_scheduled_duration) as health signals.

**D-TMON-002: BackgroundLoop spawns 20+ tasks with no instrumentation** | `nt_mind_background_loop/run.rs:738,894` + `handlers.rs:11` | HIGH | tokio-metrics TaskMonitor, BackgroundLoop::spawn

The background loop spawns coordinated handlers (`spawn_handler!` macro) and ad-hoc tasks (`spawn()`) — totaling 20+ concurrent tokio tasks — without wrapping any in `TaskMonitor::instrument()`. There is no per-task visibility into poll frequency, idle duration, or scheduling delay. A stalled handler (e.g., `handle_absorption` blocked on KB write) would not be distinguishable from a healthy slow handler. Per tokio-metrics best practices, each distinct handler type should have its own `TaskMonitor` instance to enable per-endpoint diagnostics.

**D-TMON-003: ParallelExecutor silently swallows task failures** | `nt_core_parallel/executor.rs:41` | MEDIUM | tokio-metrics TaskMonitor, JoinHandle

In `ParallelExecutor::execute()` (line 41), failed JoinHandle results (`Err(_)`) are silently discarded with `if let Ok(res) = handle.await`. No error count, no metric, no log. Combined with the lack of TaskMonitor instrumentation, this means: (a) task panics are invisible, (b) task abort/cancellation is invisible, (c) no metric distinguishes "task completed" from "task panicked." This violates the Dark Forest axiom — tasks that fail should produce observable signals, not be silently dropped.

**D-TMON-004: No tokio_unstable cfg or RuntimeMetrics integration** | entire codebase | HIGH | tokio-metrics RuntimeMonitor, tokio::runtime::RuntimeMetrics

The codebase has zero references to `tokio-metrics`, `console-subscriber`, or `tokio_unstable`. The consciousness runtime cannot observe: (a) global_queue_depth (is the runtime saturated?), (b) worker busy_duration (is CPU bound?), (c) blocking_queue_depth (is spawn_blocking overwhelming the pool?), (d) live_tasks_count (how many tasks are alive?), (e) steal_count (is work distribution balanced?). The `AsyncSafetyWrapper` at `nt_meta_async_safety.rs` addresses spawn_blocking correctness but has zero runtime observability. The heartbeat aggregator should consume RuntimeMetrics to detect runtime-level degradation (e.g., blocking_queue_depth > threshold → consciousness awareness of I/O saturation).

**D-TMON-005: HeartbeatAggregator is stateless per-tick — no trend detection** | `nt_core_heartbeat.rs:36-73` | MEDIUM | tokio-metrics intervals, HeartbeatAggregator::report

The HeartbeatAggregator stores component health in a HashMap that is overwritten each tick (no history). It cannot detect degradation trends (e.g., component transitioning from Healthy→Degraded→Unhealthy over 5 ticks). tokio-metrics TaskMonitor provides `intervals()` iterator producing cumulative delta metrics — enabling trend analysis. The aggregator should maintain a ring buffer of recent HealthReport snapshots to support trend detection, consistent with the ConsciousnessTree's trunk evolution tracking pattern.

**D-TMON-006: No metrics export pipeline (Prometheus/metrics-rs)** | `Cargo.toml` files | MEDIUM | tokio-metrics metrics-rs-integration, Prometheus

The project has no integration with `metrics-rs` or any metrics exporter. tokio-metrics provides `RuntimeMetricsReporterBuilder` and `TaskMetricsReporterBuilder` that auto-export tokio metrics with `tokio_` prefix via any metrics-rs exporter (Prometheus, Datadog, etc.). Without this, the consciousness architecture's health state is only observable via log::info!() calls, which are ephemeral and not queryable. For production consciousness monitoring, runtime metrics should be exported to enable dashboarding and alerting on task slow_poll_ratio, runtime busy_ratio, and blocking_queue_depth.

**D-TMON-007: ProxyHeartbeatEngine is task-level only — no runtime-level signal** | `nt_mind_background_loop/mod.rs:88` + `handlers_core.rs:288` | LOW | tokio::runtime::RuntimeMetrics

The ProxyHeartbeatEngine (feature-gated under stealth-net) monitors proxy health via HTTP heartbeats. However, it operates at the application layer and has no visibility into the tokio runtime's own health. If the runtime is saturated (global_queue_depth > 0, blocking_queue_depth > num_workers), proxy heartbeat tasks may be starved without any signal to the consciousness system. The heartbeat engine should be augmented with a `RuntimeMonitor::intervals()` reader that feeds runtime health into the HeartbeatAggregator as a first-class component.

## Key Insights

1. **The consciousness architecture has a critical blind spot**: All 20+ background tasks run uninstrumented. A slow LLM call or KB write that blocks a handler for minutes would not be detected by the health system. tokio-metrics `TaskMonitor` provides the exact API needed — `instrument()` wraps each task, `intervals()` produces delta metrics.

2. **The HeartbeatAggregator is an ideal integration point**: Rather than building a parallel monitoring system, the existing aggregator at `nt_core_heartbeat.rs` should be extended to consume tokio-metrics `TaskMetrics` as health signals. This follows the single-fact-source principle — one aggregator, multiple signal sources.

3. **No metrics infrastructure exists**: The absence of metrics-rs/Prometheus integration means even if tokio-metrics were added, there's no export path. A minimal Prometheus exporter would enable dashboarding and historical analysis of consciousness runtime health.

4. **The background loop's 5-second shutdown deadline is blind**: The shutdown path (`handlers.rs:48-61`) aborts tasks after 5s without knowing if they're mid-critical-operation. With TaskMonitor, the shutdown logic could check `total_poll_count` and `total_slow_poll_duration` to decide whether to wait longer or abort immediately.

5. **Parallel executor error silence compounds the blind spot**: D-TMON-003 (silent failure) + D-TMON-001 (no metrics) means task failures in the parallel reasoning subsystem are completely invisible. This is particularly dangerous for the consciousness task decomposition path.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 7 |
| Sources analyzed | 6 |
| HIGH severity | 3 |
| MEDIUM severity | 3 |
| LOW severity | 1 |
