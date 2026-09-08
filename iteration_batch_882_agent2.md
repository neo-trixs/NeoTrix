# Iteration Batch 882 — Agent 2: Rust Async Task Monitoring Deep Research

## Research Summary

**Crates analyzed**: `tokio-metrics` (v0.5.1), `console-subscriber` (v0.4.1/0.5.0), `tokio-console` (v0.1.14)
**NeoTrix baseline**: 80+ `tokio::spawn` / `spawn_blocking` call sites across 30+ files. Zero async task monitoring instrumentation. Tokio `tracing` feature **not enabled** in workspace `Cargo.toml`. No `tokio_unstable` cfg flag anywhere.

---

## Defect 1: Zero Async Task Visibility — Silent Task Starvation Risk

**Severity**: CRITICAL | **Type**: Structural Gap

NeoTrix has 80+ `tokio::spawn` sites across `nt_core_llm`, `nt_shield_proxy_kernel`, `nt_io_provider`, `nt_mind_background_loop`, `nt_world_osint`, `nt_shield_stealth_net`, and others — with **zero instrumentation**. There is no `TaskMonitor`, no `RuntimeMonitor`, no task-level metrics collection.

**Why this matters**: Tokio tasks that block worker threads (e.g., `reqwest::blocking` calls at `nt_io_provider/gateway/selection.rs:365`, `nt_io_provider/factory.rs:1295`) or starve other tasks of execution time are **completely invisible**. NeoTrix already has `nt_meta_async_safety.rs` that detects blocking patterns, but it only generates recommendations — it cannot measure actual impact because no runtime metrics are collected.

**Upstream capability**: `tokio-metrics::RuntimeMonitor` exposes `global_queue_depth`, `local_queue_depth`, `total_busy_duration`, `min/max_park_count` per-worker, `budget_forced_yield_count`, and `poll_time_histogram` — all absent from NeoTrix.

**Evidence**: `neotrix-core/src/unified/layers/meta/nt_meta/nt_meta_async_safety.rs:5` documents that `tokio::task::spawn_blocking` wrapping is mandatory but has no runtime enforcement.

---

## Defect 2: `tokio_unstable` CFG Dependency — Build Fragility

**Severity**: HIGH | **Type**: Build Configuration

Both `tokio-metrics` (runtime metrics) and `console-subscriber` require `RUSTFLAGS="--cfg tokio_unstable"` for their full feature set. NeoTrix does not set this anywhere:

- No `tokio_unstable` in any `.cargo/config.toml`
- No `RUSTFLAGS` in workspace `Cargo.toml`
- The `tokio` dependency uses `features = ["full"]` but does **not** enable `tracing`

**Consequence**: Even if NeoTrix adds `console-subscriber` today, it would silently produce empty metrics without the cfg flag. The docs warn that "alternating between building with and without this configuration file will cause full rebuilds of your project" — meaning NeoTrix's CI/CD would suffer unpredictable rebuild thrash.

**Upstream trap**: `console-subscriber` checks for `tokio_unstable` at compile time but falls back silently rather than failing — a classic "works but gives you nothing" scenario.

---

## Defect 3: u64 Overflow in Duration Counters — Silent Data Corruption

**Severity**: HIGH | **Type**: Correctness

`tokio-metrics::TaskMetrics` uses `u64` nanosecond counters for durations. The documentation explicitly warns:

> "If a cumulative metric overflows *more than once* in the midst of an interval, its interval-sampled counterpart will also overflow."

NeoTrix spawns long-lived background tasks (`nt_mind_background_loop` runs absorption cycles with 60s ticks, `nt_shield_proxy_kernel` maintains persistent connection pools). For these workloads:

- `total_first_poll_delay` overflows at ~184 seconds cumulative (with 256K tasks)
- `total_poll_duration` overflows at ~584 years (safe for individual tasks but not for long-running aggregated monitors)
- **Critical**: NeoTrix's `HeartbeatAggregator` already tracks system health — adding `tokio-metrics` without overflow-awareness would produce corrupted health signals

**Mitigation needed**: Interval-sampled metrics (via `intervals()`) survive single overflow events but not double-overflow within one interval. NeoTrix must implement overflow detection or use 128-bit accumulators.

---

## Defect 4: console-subscriber gRPC Overhead — Incompatible with NT-SHIELD Stealth Requirements

**Severity**: HIGH | **Type**: Architectural Conflict

`console-subscriber` spawns a **tonic gRPC server** (default port 6669) that streams all task telemetry. This directly conflicts with NeoTrix's NT-SHIELD domain:

- `nt_shield_sandbox` enforces egress privacy guard and network trust boundaries
- `nt_shield_stealth_net` manages fingerprint management and Tor client
- `nt_shield_proxy_kernel` controls all outbound connections

A persistent gRPC listener on a well-known port would:
1. Bypass NT-SHIELD's egress policy (outbound telemetry to `tokio-console` client)
2. Leak runtime state (task names, poll durations, waker counts) to any local observer
3. Conflict with `nt_core_llm::egress_privacy_guard` — which redacts internal fingerprints from external models

**Production advice from upstream**: "Should I use tokio-console in production? No. tokio-console has overhead; use it in staging for debugging." (rustz2h.com, 2026). This confirms the architectural incompatibility.

---

## Defect 5: No Task-Level Scheduling Delay Measurement — Blind Spot for NT-CORE's GWT

**Severity**: MEDIUM | **Type**: Observability Gap

NT-CORE's GWT (Global Workspace Theory) routes attention based on system health signals. `HeartbeatAggregator` currently collects compilation/test/KB/eventbus/module health — but **zero scheduling latency data**.

`tokio-metrics` provides exactly what GWT needs:
- `total_scheduled_duration` / `total_scheduled_count` → mean scheduling delay
- `total_long_delay_count` / `total_long_delay_duration` → long-delay detection
- `total_idled_count` / `total_idle_duration` → task starvation detection

Without these, GWT's attention modulation is operating blind on async execution health. A task that takes 200ms to schedule (vs. 2ms normal) would not trigger any attention shift.

---

## Defect 6: No Per-Domain Task Metrics — Cannot Attribute Latency

**Severity**: MEDIUM | **Type**: Operational Blind Spot

NeoTrix's 7-faction architecture (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-SHIELD, NT-IO) spawns tasks across all domains. `tokio-metrics::TaskMonitor` supports per-group instrumentation via separate `TaskMonitor` instances, but NeoTrix has no mechanism to:

1. Tag spawned tasks with their originating domain
2. Collect per-domain poll duration / scheduling delay
3. Attribute latency regressions to specific domains

The `tokio-metrics-collector` crate (v0.3.1) provides Prometheus-compatible metrics with label support, but requires `tokio_unstable` and `tokio features = ["tracing"]` — neither currently enabled.

---

## Defect 7: Blocking Task Pool Exhaustion Undetected — NT-ACT Blind Spot

**Severity**: MEDIUM | **Type**: Resource Exhaustion Risk

NeoTrix uses `tokio::task::spawn_blocking` in at least 5 locations:
- `nt_io_provider/gateway/selection.rs:365` — catalog refresh
- `nt_io_provider/factory.rs:1295` — catalog refresh
- `nt_io_web/tiles.rs:79` — tile processing
- `nt_memory_kb/nt_memory_api.rs:369` — KB reachability check
- `nt_shield_stealth_net/geo_proxy.rs:414` — geo proxy computation

Tokio's default blocking pool is 512 threads. `RuntimeMetrics` exposes `blocking_queue_depth` — the number of tasks waiting for a blocking thread. NeoTrix has **no monitoring** of this metric. Under load (e.g., parallel crawl + LLM calls + KB operations), the blocking pool can saturate, causing cascading latency.

---

## Defect 8: HeartbeatAggregator Lacks Async Executor Metrics — Incomplete Health Picture

**Severity**: LOW | **Type**: Health Model Gap

`HeartbeatAggregator` (documented in AGENTS.md) produces a unified `SystemHealthSnapshot` with time-decay. However, it only collects:
- Compilation status
- Test pass rates
- KB health
- EventBus health
- Module health

Missing from the health snapshot:
- `workers_count` (runtime thread count)
- `total_busy_duration` / `elapsed` ratio (executor utilization)
- `budget_forced_yield_count` (compute-heavy task detection)
- `total_steal_count` (work-stealing imbalance)

These are exactly what `tokio-metrics::RuntimeMonitor` provides. Adding them to `HeartbeatAggregator` would give GWT a complete picture of async runtime health.

---

## Recommended Integration Path

1. **Immediate**: Add `tokio = { features = ["full", "tracing"] }` to workspace `Cargo.toml`
2. **Phase 1**: Add `tokio-metrics` dependency, create `NtAsyncMonitor` module under NT-META with domain-tagged `TaskMonitor` instances per faction
3. **Phase 2**: Wire `RuntimeMonitor` metrics into `HeartbeatAggregator` as a new health dimension
4. **Phase 3**: Add overflow detection for u64 duration counters; implement interval-sampled metrics only
5. **Phase 4**: Consider `console-subscriber` behind a `dev-tools` feature flag (never in production, never with NT-SHIELD active)
6. **Guard**: All console-subscriber traffic must route through NT-SHIELD's egress privacy guard if ever enabled outside localhost

---

## Sources

| Source | URL | Date |
|--------|-----|------|
| tokio-metrics docs | https://docs.rs/tokio-metrics/latest/tokio_metrics/ | 2026-09-07 |
| tokio-metrics GitHub | https://github.com/tokio-rs/tokio-metrics | 2026-09-07 |
| console-subscriber docs | https://docs.rs/console-subscriber/latest/console_subscriber/ | 2026-09-07 |
| console-subscriber README | https://github.com/tokio-rs/console/blob/main/console-subscriber/README.md | 2026-09-07 |
| tokio-console GitHub | https://github.com/tokio-rs/console | 2026-09-07 |
| OpenTelemetry Tokio guide | https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view | 2026-09-07 |
| Production tokio-console | https://medium.com/rustaceans/tokio-console-in-production-94a6ce4cd112 | 2026-09-07 |
| Tokio runtime tuning | https://rustz2h.com/chapter_07_mastering_async_rust_and_tokio/series_01_tokio_runtime_internals_and_tasks/tokio_runtime_tuning_production | 2026-09-07 |
| tokio-metrics-collector | https://crates.io/crates/tokio-metrics-collector | 2026-09-07 |
