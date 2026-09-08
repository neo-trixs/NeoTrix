# Iteration Batch 880 — Agent 2: Rust Async Task Monitoring

**Date**: 2026-09-07
**Scope**: tokio-metrics, console-subscriber, tokio-console, tokio-metrics-collector
**Goal**: Extract defects/improvements for NeoTrix async task monitoring capabilities

---

## 1. Research Summary

### tokio-metrics (v0.5)

- **Purpose**: Production-grade task & runtime metrics for Tokio. Two monitoring tiers:
  - **Task-level** (`TaskMonitor`/`TaskMonitorCore`): per-task polling, scheduling, idle/slow metrics
  - **Runtime-level** (`RuntimeMonitor`): worker thread utilization, queue depths, poll counts
- **Key metrics**: `instrumented_count`, `dropped_count`, `total_poll_count`, `total_poll_duration`, `total_slow_poll_count/duration`, `total_long_delay_count/duration`, `total_budget_exhausted_yields`, `workers_count`, `total_park_count`, `global_queue_depth`, `total_local_queue_depth`, `injection_queue_depth`
- **Integration**: Works with `metrics-rs` via feature flag; Prometheus exporter available via `tokio-metrics-collector`
- **Limitation**: Requires `tokio_unstable` cfg flag for runtime metrics; task metrics need `rt` feature (default on)
- **Overflow risk**: `u64::MAX` nanosecond overflow on cumulative counters — documented but no saturation/clamping

### console-subscriber (v0.1.6)

- **Purpose**: Tracing-based instrumentation layer for `tokio-console` — a TUI "top for async tasks"
- **Architecture**: `ConsoleLayer` (tracing `Layer`) → aggregator → gRPC server → `tokio-console` TUI client
- **Metrics collected**: Task states (running/waiting/idle/terminated), resource spans, async operations, poll scheduling
- **Features**: Configurable retention, server address (TCP/Unix socket/vsock), `parking_lot` locks, gRPC-Web support
- **Limitation**: Requires `tokio_unstable` cfg, `tokio` + `runtime` tracing targets at TRACE level
- **Debugging focus**: Primarily local debugging tool; not designed for production telemetry export

### tokio-metrics-collector (v0.3.1)

- **Purpose**: Prometheus-compatible bridge for tokio-metrics
- **Limitation**: Duplicate labels across multiple monitors cause incorrect data aggregation (documented caveat)
- **Runtime metrics**: Requires `tokio_unstable`

### Related: OpenTelemetry Integration (2026)

- Emerging pattern: `tokio-metrics` → OpenTelemetry SDK → OTLP export → Prometheus/Grafana
- Starvation detection: monitors `global_queue_depth + total_local_queue_depth` against threshold
- Budget exhaustion tracking: `budget_forced_yields` counter for compute-heavy tasks
- Worker thread utilization + blocking thread count monitoring

---

## 2. Extracted Defects for NeoTrix

### DEFECT-1: No Async Task-Level Observability

**Evidence**: NeoTrix codebase contains zero references to `tokio_metrics`, `TaskMonitor`, or any task instrumentation wrapper. The `HeartbeatAggregator` (mentioned in AGENTS.md) only collects compilation/test/KB/module health — it does not observe async task scheduling, polling, or resource contention.

**Impact**: Cannot detect slow polls, task starvation, or scheduling delays in NeoTrix's own async tasks (crawl pipelines, SEAL loop, GWT attention routing).

**Recommendation**: Wrap critical NeoTrix async tasks with `tokio_metrics::TaskMonitor` per subsystem. Aggregate per-domain metrics (NT-WORLD crawls, NT-MIND evolution, NT-ACT tool invocations). Feed into `HeartbeatAggregator` as a new signal source.

---

### DEFECT-2: No Runtime Worker Thread Health Monitoring

**Evidence**: `RuntimeMonitor` exposes `workers_count`, `total_park_count`, `injection_queue_depth`, `total_local_queue_depth`, `global_queue_depth`, `total_budget_exhausted_yields`, `total_blocking_thread_count` — none of these are collected by NeoTrix.

**Impact**: Cannot detect:
- Worker thread starvation (all workers blocked on sync operations)
- Queue depth spikes indicating backpressure
- Budget exhaustion from CPU-bound tasks in the async runtime
- Blocking thread pool exhaustion

**Recommendation**: Add `RuntimeMonitor` to NeoTrix's startup. Feed `injection_queue_depth` and `global_queue_depth` into `HeartbeatAggregator`. Set alerting thresholds on `total_budget_exhausted_yields` (compute tasks should use `spawn_blocking`).

---

### DEFECT-3: No Task Slow-Poll / Long-Delay Detection

**Evidence**: `tokio-metrics` provides `total_slow_poll_count/duration` and `total_long_delay_count/duration` with configurable thresholds (`slow_poll_threshold`, `long_delay_threshold`). NeoTrix has no equivalent detection.

**Impact**: Silent performance degradation — tasks may be starved by long-running polls or delayed scheduling without any diagnostic signal. The E8 Hexagram reasoning engine and GWT attention routing could be silently degraded by competing tasks.

**Recommendation**: Instrument NT-CORE tasks with `TaskMonitor::with_slow_poll_threshold(Duration::from_millis(5))` and `with_long_delay_threshold(Duration::from_millis(10))`. Route slow-poll events to `NT-REPAIR` for self-healing trigger.

---

### DEFECT-4: No Task Lifecycle Tracking (Drop/Spawn Ratios)

**Evidence**: `TaskMetrics` tracks `instrumented_count` vs `dropped_count` — a high drop ratio indicates tasks being cancelled before completion (leaked work, wasted CPU). NeoTrix has no visibility into task completion rates.

**Impact**: Cannot detect:
- Task cancellation storms (e.g., crawl tasks aborted by timeout)
- Spawn-but-never-complete patterns (resource leaks)
- First-poll delays indicating global queue injection latency

**Recommendation**: Track `dropped_count / instrumented_count` ratio per domain. Alert when ratio exceeds 0.1 (10% task cancellation). Feed `mean_first_poll_delay` into system health snapshot.

---

### DEFECT-5: No Production Async Debugging Toolchain

**Evidence**: `console-subscriber` + `tokio-console` provides real-time TUI debugging of async tasks — task states, resource contention, poll scheduling visualization. NeoTrix has no equivalent for production debugging.

**Impact**: When async issues arise in production (deadlocks, resource starvation, unexpected task states), there is no diagnostic tool to introspect the runtime without restarting or adding ad-hoc logging.

**Recommendation**: Add `console-subscriber` as an optional dependency behind a feature flag (e.g., `debug-console`). When enabled, expose the gRPC endpoint for remote `tokio-console` connection. Gate behind `tokio_unstable` cfg to avoid production overhead.

---

### DEFECT-6: No Metrics Export Pipeline for Async Runtime

**Evidence**: `tokio-metrics-collector` and the OpenTelemetry integration pattern show how to export Tokio metrics to Prometheus/Grafana. NeoTrix has no metrics export for async runtime health.

**Impact**: No external observability into NeoTrix's async runtime behavior. Cannot build dashboards for task throughput, scheduling latency, or resource utilization over time.

**Recommendation**: Add `tokio-metrics` with `metrics-rs-integration` feature. Wire `RuntimeMetricsReporterBuilder` and `TaskMetricsReporterBuilder` into NeoTrix's startup. Expose via OpenTelemetry OTLP or Prometheus endpoint. Integrate with `NT-IO` domain.

---

### DEFECT-7: Overflow Risk on Cumulative Counters

**Evidence**: `tokio-metrics` documents that cumulative counters can overflow `u64::MAX` nanoseconds or counts. The crate does not clamp or saturate — it silently wraps. NeoTrix has no overflow protection for any long-running counters.

**Impact**: In long-running NeoTrix instances (days/weeks), metrics counters could silently wrap around, producing incorrect dashboard data and triggering false alerts.

**Recommendation**: Use `saturating_add` patterns for any long-running metric accumulation. Consider periodic reset of cumulative counters (per-interval deltas only). Add overflow detection as a `SelfTest` dimension.

---

## 3. Priority Matrix

| # | Defect | Severity | Effort | ROI |
|---|--------|----------|--------|-----|
| 1 | No task-level observability | **Critical** | Medium | High |
| 2 | No runtime worker health | **High** | Medium | High |
| 3 | No slow-poll detection | **High** | Low | High |
| 4 | No task lifecycle tracking | **Medium** | Low | Medium |
| 5 | No production debug toolchain | **Medium** | Low | Medium |
| 6 | No metrics export pipeline | **High** | Medium | High |
| 7 | Counter overflow risk | **Low** | Low | Low |

---

## 4. Recommended Absorption Plan

1. **Immediate** (this cycle): Add `tokio-metrics` dependency to `neotrix-core`. Wrap `HeartbeatAggregator` with `RuntimeMonitor` to collect worker health. Add `TaskMonitor` to 3 critical paths: crawl pipeline, SEAL loop, GWT attention broadcast.

2. **Short-term** (1-2 cycles): Wire `tokio-metrics` into `metrics-rs` ecosystem. Add Prometheus/OpenTelemetry export via `NT-IO`. Add slow-poll/long-delay detection with thresholds.

3. **Medium-term** (2-3 cycles): Add `console-subscriber` behind feature flag. Build async task debugging dashboard. Add task lifecycle ratio monitoring to `HeartbeatAggregator`.

4. **Long-term**: Integrate async metrics into `ConsciousnessTree` health chain. Use task starvation signals to modulate GWT attention routing priority.

---

## 5. Sources

- https://docs.rs/tokio-metrics/latest/tokio_metrics/
- https://github.com/tokio-rs/tokio-metrics
- https://docs.rs/console-subscriber/latest/console_subscriber/
- https://github.com/tokio-rs/console
- https://lib.rs/crates/tokio-metrics
- https://lib.rs/crates/console-subscriber
- https://crates.io/crates/tokio-metrics-collector
- https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view
- https://deepwiki.com/tokio-rs/tokio-metrics
