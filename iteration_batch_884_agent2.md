# Iteration Batch 884 — Agent 2: Rust Async Task Monitoring (Deep Research)

**Date**: 2026-09-07  
**Focus**: `tokio-metrics`, `console-subscriber`, tokio task metrics  
**Sources**: crates.io, docs.rs, GitHub issues/discussions, tokio-rs/tokio-metrics, tokio-rs/console, Medium articles, Rust users forum

---

## 1. Crate Landscape

### tokio-metrics (v0.5.1)
- **Purpose**: Production-ready metrics for Tokio task and runtime monitoring
- **Key APIs**: `TaskMonitor`, `RuntimeMonitor`, `TaskMetrics` (19 fields), `RuntimeMetrics` (41 fields)
- **Integration**: Optional `metrics-rs-integration` feature → Prometheus/Grafana export
- **Stability**: Task metrics stable; runtime metrics partially stable (some require `tokio_unstable`)
- **Downloads**: 44M+ total, 6.2M recent

### console-subscriber (v0.5.0)
- **Purpose**: Debugging/profiling via `tracing-subscriber::Layer` → gRPC → tokio-console TUI
- **Key APIs**: `ConsoleLayer`, `Builder`, `init()`, `spawn()`
- **Requires**: `tokio_unstable` cfg flag + Tokio `tracing` feature + TRACE-level targets
- **Downloads**: 42M+ total, 7.7M recent

---

## 2. Critical Defects for NeoTrix Integration

### DEFECT-1: `tokio_unstable` CFG Flag Breaks Build Consistency
**Severity**: HIGH  
**Component**: Build system / CI

Both crates require `RUSTFLAGS="--cfg tokio_unstable"` at compile time. Missing this flag causes:
- `console-subscriber`: silently produces no task data (app compiles but console shows zero tasks)
- `tokio-metrics` runtime metrics: compilation failure for unstable metric fields
- Toggling this flag causes **full rebuilds** of the entire dependency tree

**NeoTrix Impact**: NeoTrix already has a complex build with `#![forbid(unsafe_code)]` (R-P1) and multi-crate workspace. Adding `tokio_unstable` to `.cargo/config.toml` means:
- Every developer and CI must have the flag; forgetting it = silent failure
- rust-analyzer may not pick up `.cargo/config.toml` from subdirectories
- Conditional compilation (`cfg(tokio_unstable)`) fragments code paths

**Recommendation**: Gate behind feature flag with explicit opt-in; never enable by default.

### DEFECT-2: Performance Overhead 5-15% on High-Frequency Task Workloads
**Severity**: HIGH  
**Component**: Runtime metrics collection

**Evidence from tokio-rs/tokio#7571**:
- Tokio 1.47+ added `Instant::now()` calls on every task poll for time-based metrics
- Real-world reports: **8-15% overhead** in production services using `yield_now()` loops or `consume_budget()`
- Flamegraph shows `Timespec::now` + `Instant::elapsed` consuming 63% of total CPU in pathological cases
- Production report: 0.09% overhead for normal workloads, but **10% for busy-polling loops**

**console-subscriber overhead**: Additional 5-15% beyond base Tokio instrumentation (intercepts every spawn, poll, wake, resource acquisition)

**NeoTrix Impact**: NeoTrix's `HeartbeatAggregator`, `ConsciousnessTree` growth cycles, and `SEAL pipeline` all spawn high-frequency async tasks. Combined overhead could exceed 20% in hot paths.

**Recommendation**: 
- Use `tokio-metrics` TaskMonitor only on critical-path tasks (not every spawn)
- `console-subscriber` must be **compile-time gated** with `#[cfg(debug_assertions)]`
- Never ship `console-subscriber` to production

### DEFECT-3: Memory Leak in console-subscriber Aggregator
**Severity**: HIGH  
**Component**: `console-subscriber::Aggregator`

**GitHub issue tokio-rs/console#184**:
- Without connected console clients, `poll_ops` grows forever — no cleanup occurs
- Default retention of 1 hour for completed tasks compounds the issue
- Observed: **250+ MB growth over several hours** at idle
- Memory only freed when console client connects (pulls data then drops)
- Partial fix in PR #501 but regression reports persist

**NeoTrix Impact**: NeoTrix long-running processes (background loop, proxy heartbeat, KB maintenance) would accumulate unbounded memory. The `HeartbeatAggregator` already collects health signals — adding a leaky subscriber multiplies risk.

**Recommendation**: 
- Set aggressive `retention(Duration::from_secs(60))` if used
- Monitor subscriber memory separately
- Consider `tokio-metrics` (no gRPC/aggregator overhead) for production

### DEFECT-4: u64 Overflow in TaskMetrics Counters
**Severity**: MEDIUM  
**Component**: `tokio_metrics::TaskMetrics`

**Documentation warning**: All counters and durations use `u64`. Counters overflow at 2^64 events; durations overflow at ~584 years of nanoseconds. However:
- `TaskMonitor::intervals()` computes **delta between successive `cumulative()` calls**
- If >2^64 events occur within a single monitoring interval, the delta overflows silently
- Duration overflow possible in cumulative mode for very long-running processes

**NeoTrix Impact**: NeoTrix's `ConsciousnessTree` runs 6-stage growth cycles continuously. The `total_poll_count` and `total_poll_duration` fields could overflow in long-running sessions (days/weeks of continuous operation).

**Recommendation**: 
- Use short monitoring intervals (500ms-1s) to prevent delta overflow
- Add overflow detection: check if `intervals()` delta > previous cumulative
- Consider wrapping with saturation arithmetic for safety

### DEFECT-5: No Cross-Task Dependency / Causal Tracing
**Severity**: MEDIUM  
**Component**: `tokio-metrics` / `console-subscriber`

Both crates provide **aggregate metrics per TaskMonitor** or **per-task lifecycle** but lack:
- No causal link between tasks (e.g., "task A spawned task B which blocked task C")
- No resource contention tracking across task boundaries
- No end-to-end latency decomposition (network vs. compute vs. scheduling)

**NeoTrix Impact**: NeoTrix's `nt_core_parallel::coordinator` manages multi-agent task execution with dependency graphs. Current tools show *that* tasks are slow, not *why* in the context of cross-domain task chains (e.g., NT-WORLD crawl → NT-MEMORY KB write → NT-MIND distillation).

**Recommendation**: 
- Augment with custom `tracing::Span` fields linking parent→child tasks
- Use `console-subscriber` resource view for lock/semaphore contention
- Build custom dashboard combining `tokio-metrics` aggregates with NeoTrix domain labels

### DEFECT-6: No Per-Task Naming / Labeling in TaskMetrics
**Severity**: MEDIUM  
**Component**: `tokio_metrics::TaskMonitor`

`TaskMonitor` aggregates ALL instrumented tasks together. To split metrics:
- Must create separate `TaskMonitor` instances per task category
- No built-in naming — you must manually track which monitor = which task type
- `TaskMetricsReporterBuilder` accepts a name transformer, but it's a static string, not dynamic

**NeoTrix Impact**: NeoTrix has 7 domains (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD) each spawning tasks. Without per-domain task monitors, metrics are meaningless noise.

**Recommendation**: 
- Create one `TaskMonitor` per domain (7 monitors minimum)
- Use `TaskMonitorCore` for static/const monitors to avoid allocation overhead
- Map monitor → domain in a registry

### DEFECT-7: `tokio_unstable` API Stability Risk
**Severity**: MEDIUM  
**Component**: Both crates

`console-subscriber` depends on Tokio's **experimental tracing instrumentation** which:
- Can break between Tokio minor versions (e.g., v1.41.0 added future size tracking)
- No stability guarantee — APIs may change or be removed
- Minimum Tokio version requirements escalate with each feature (v1.7 → v1.12 → v1.13 → v1.15 → v1.21 → v1.41)

**NeoTrix Impact**: NeoTrix pins Tokio versions carefully. Adding `tokio_unstable` dependency means:
- Every Tokio upgrade requires testing console-subscriber compatibility
- Potential breakage in production if Tokio changes internal instrumentation format

**Recommendation**: 
- Use `console-subscriber` only in development builds
- Pin `tokio-metrics` to stable task metrics (no `tokio_unstable` needed for task-level)
- Abstract behind a trait so runtime can be swapped

---

## 3. NeoTrix-Specific Integration Gaps

### GAP-1: HeartbeatAggregator Has No Task-Level Metrics
NeoTrix's `HeartbeatAggregator` (core/nt_core_heartbeat.rs:32) collects system health signals but has zero visibility into **async task health**:
- No poll duration tracking per domain task
- No scheduling delay detection
- No slow-poll ratio monitoring

**Fix**: Instrument HeartbeatAggregator's internal tasks with `TaskMonitor` per domain.

### GAP-2: ConsciousnessTree Growth Cycles Are Unobservable
The 6-stage growth cycle (Soil→Roots→Trunk→Branches→Fruits→Core) runs as async tasks but:
- No metrics on cycle stage durations
- No detection of stalls in specific stages
- No correlation between cycle stage and task scheduling delays

**Fix**: Wrap each growth stage in a `TaskMonitor`-instrumented future with stage labels.

### GAP-3: No Prometheus/Metrics-RS Integration
NeoTrix has no `metrics-rs` exporter configured. `tokio-metrics` supports it via `metrics-rs-integration` feature, but:
- No `metrics_exporter_prometheus` in dependencies
- No OpenTelemetry integration for async runtime metrics
- HeartbeatAggregator health data is in-memory only

**Fix**: Add `metrics_exporter_prometheus` to neotrix-core, export to local socket or HTTP endpoint.

---

## 4. Recommended Integration Strategy

| Layer | Tool | Gate | Purpose |
|-------|------|------|---------|
| L5 Cognition | `tokio-metrics` TaskMonitor | Feature flag `async-metrics` | Per-domain task health |
| L5 Cognition | `tokio-metrics` RuntimeMonitor | `tokio_unstable` + feature flag | Worker thread utilization |
| L6 Meta | `console-subscriber` | `#[cfg(debug_assertions)]` only | Interactive debugging |
| L1 Action | `metrics-rs` + Prometheus | Always-on (low overhead) | Production dashboards |

**Priority Order**:
1. `tokio-metrics` TaskMonitor (stable, no `tokio_unstable`, low overhead)
2. `metrics-rs` integration (production visibility)
3. `console-subscriber` (dev-only, gated behind debug builds)
4. `tokio-metrics` RuntimeMonitor (requires `tokio_unstable`, highest overhead)

---

## 5. References

| Source | URL | Key Finding |
|--------|-----|-------------|
| tokio-metrics crate | https://crates.io/crates/tokio-metrics | v0.5.1, 44M downloads |
| console-subscriber crate | https://crates.io/crates/console-subscriber | v0.5.0, gRPC-based |
| tokio#7571 (overhead) | https://github.com/tokio-rs/tokio/issues/7571 | 8-15% overhead from `Instant::now()` |
| console#184 (memory leak) | https://github.com/tokio-rs/console/issues/184 | 250MB+ unbounded growth |
| tokio-metrics#86 (stability) | https://github.com/tokio-rs/tokio-metrics/issues/86 | Partial stabilization in progress |
| tokio#4073 (stabilization) | https://github.com/tokio-rs/tokio/issues/4073 | Runtime metrics stabilization tracker |
| DeepWiki runtime metrics | https://deepwiki.com/tokio-rs/tokio/9.3 | MetricsBatch architecture |
| Medium production guide | https://medium.com/rustaceans/tokio-console-in-production-94a6ce4cd112 | 5-15% overhead in prod |
| Hotpath.rs comparison | https://hotpath.rs/tokio_runtime | RuntimeMetrics vs application profiling |
