# Agent 4: Async Runtime Metrics (Batch 868)

## Sources

1. **Tokio Console GitHub** — `tokio-rs/console`: real-time async task debugger, gRPC wire protocol, poll durations, busy/idle time, waker ops. Requires `--cfg tokio_unstable`. Source: https://github.com/tokio-rs/console
2. **tokio-metrics GitHub** — `tokio-rs/tokio-metrics`: `RuntimeMonitor` (worker park/noop/steal counts, poll time histograms, budget exhaustion) + `TaskMonitor` (per-task poll duration, mean busy time). Source: https://github.com/tokio-rs/tokio-metrics
3. **OneUptime Blog** — "How to Monitor Tokio Runtime Metrics with OpenTelemetry in Rust" (2026-02-06): `RuntimeMetrics` integration with OTLP, task starvation detection, budget_forced_yield tracking, worker_histogram, blocking_thread_count. Source: https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view
4. **Rustify** — "Rust Async Runtimes: Tokio vs async-std vs smol 2026": runtime configuration best practices, work-stealing tuning, ecosystem lock-in (150M+ downloads/month). Source: https://rustify.rs/articles/rust-async-runtimes-tokio-vs-async-std-2026
5. **Reintech** — "Tokio Tutorial 2026": monitoring/debugging section, `console-subscriber` setup, `std::thread::sleep` as blocking antipattern, lock-hold-across-await deadlock pattern. Source: https://reintech.io/blog/tokio-tutorial-2026-building-async-applications-rust
6. **Chandan Bhagat** — "Async Rust with Tokio Part 9: Observability and Debugging" (2026-05-03): runtime starvation diagnosis, slow poll detection (>10µs flagged), structured tracing, `#[instrument]` spans, waker registration. Source: https://chandanbhagat.com.np/async-rust-with-tokio-part-9-observability-and-deb
7. **DeepWiki** — Tokio runtime metrics and observability: `tokio::runtime::dump`, task backtraces, `RuntimeMetrics` worker_histogram, io_driver_ready_count, blocking_thread_count. Source: https://deepwiki.com/tokio-rs/tokio/9.3-runtime-metrics-and-observability
8. **Tokio Blog** — "Announcing Tokio Console 0.1" (2021-12): warnings system (clippy for async), long-running-task detection, self-woken detection. Source: https://tokio.rs/blog/2021-12-announcing-tokio-console
9. **Andrew Odendaal** — "Rust Async Runtime Deep Dive: Tokio Architecture" (2026-06): `tokio-console` for live task states/poll times/waker counts, `FuturesUnordered` vs spawn tradeoffs, worker thread count tuning. Source: https://andrewodendaal.com/rust-async-runtime-tokio-architecture/
10. **CodeZup** — "Tokio Task Spawning and Channels" (2026-07): spawn_blocking for CPU work, max_blocking_threads config, tokio-console for task metrics, `JoinSet` pattern. Source: https://codezup.com/mastering-async-rust-concurrency-patterns

## Defects

**D-METRIC-001: Zero Tokio console-subscriber integration — blind async task lifecycle** | `neotrix-core/Cargo.toml` | **High** | Sources 1, 2, 8, 9

NeoTrix has zero `console-subscriber` dependency in any `Cargo.toml` and zero `tokio-console` usage in any production or dev path. Tokio Console provides real-time per-task poll durations, busy/idle time, waker counts, and a warnings system that detects tasks running for very long time without yielding, tasks that have woken themselves more times than woken by others, and resource leaks. Without this, NeoTrix cannot identify which async tasks are starved, which are blocking worker threads, or which are leaking wakers. The GWT attention routing, SEAL pipeline stages, and EventBus dispatch all execute as Tokio tasks but have zero runtime-level visibility.

**D-METRIC-002: Zero RuntimeMetrics / tokio-metrics collection — no worker thread utilization data** | `neotrix-core/src/unified/core/nt_core_heartbeat.rs` | **High** | Sources 2, 3, 7

The `HeartbeatAggregator` (nt_core_heartbeat.rs:32-79) is a plain `HashMap<String, ComponentHealth>` with no Tokio runtime metric integration. `tokio-metrics` v0.5 provides `RuntimeMonitor` which tracks: worker park count, noop count (false-positive wakes), steal count, poll time histograms, `budget_forced_yield_count` (compute-heavy tasks exceeding poll budget), and `idle_blocking_threads_count`. None of these metrics are collected. NeoTrix's health reporting is purely application-level component checks (Healthy/Degraded/Unhealthy) with zero knowledge of whether the underlying Tokio executor is saturated, whether worker threads are starving, or whether tasks are being budget-forced-yielded.

**D-METRIC-003: std::thread::sleep inside async spawn loops — worker thread starvation risk** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:464` | **High** | Sources 5, 6, 9

The EventBus fire-and-forget handler at line 464 uses `std::thread::sleep(Duration::from_millis(10))` inside an async context. This blocks the Tokio worker thread for the entire sleep duration, starving all other tasks on that worker. The Tokio Console warnings system specifically detects this pattern: "tasks that have run for a very long time without yielding." Similarly, `nt_world_crawl/stealth.rs:264`, `nt_world_browse/session.rs:64`, `nt_memory_kb/nt_http.rs:242`, `nt_shield_http_proxy.rs:182`, `nt_world_scrape.rs:287`, and `nt_memory_kb/nt_memory_crawl.rs:346` all use `std::thread::sleep` in async codepaths. These should use `tokio::time::sleep` instead. Every `std::thread::sleep` in an async context blocks the worker thread entirely.

**D-METRIC-004: Nested Runtime::new() spawning from async context — thread pool fragmentation** | `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/factory.rs:1435-1460` | **Medium** | Sources 3, 7, 10

When `Handle::try_current().is_ok()` (inside a Tokio runtime), NeoTrix spawns a new OS thread and creates a brand-new `tokio::runtime::Runtime` within it (factory.rs:1438). This pattern repeats across the codebase: `nt_core_forecast.rs:373`, `nt_shield_sandbox_entry.rs:228`, `nt_shield_sandbox/mod.rs:585`, `nt_shield_stealth_net/rule_api.rs:192`, and many test files. Each `Runtime::new()` creates a separate thread pool (default: num_cpus workers + blocking pool). This means NeoTrix may run 3-5 independent Tokio runtimes simultaneously, each with their own thread pool, causing: (1) CPU oversubscription, (2) no cross-runtime task visibility, (3) no unified metric collection, (4) memory overhead from duplicated driver state. The factory.rs code has an explicit comment acknowledging the panic risk but the solution leaks the runtime via `std::mem::forget(rt)` (line 1447).

**D-METRIC-005: No task poll duration instrumentation — cannot detect slow futures** | Global (no `#[instrument]` + no `TaskMonitor`) | **Medium** | Sources 2, 6, 8

Tokio Console and tokio-metrics both provide per-task poll duration histograms. NeoTrix has zero `#[instrument]` spans on async functions (confirmed in batch 754) and zero `TaskMonitor` usage. This means: (1) any single future poll taking >10ms is invisible — the "slow poll" detection that Tokio Console provides ("any poll that takes more than 10 microseconds is flagged") is completely absent; (2) budget exhaustion (compute-heavy tasks exceeding Tokio's cooperative poll budget) goes undetected; (3) the SEAL pipeline's 6-stage loop (Soil→Roots→Trunk→Branches→Fruits→Core) cannot be profiled at the future-poll level, making it impossible to identify which pipeline stage is the bottleneck.

**D-METRIC-006: WasmSandbox owns dedicated tokio::runtime::Runtime — no metric integration** | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox_entry.rs:213` | **Medium** | Sources 2, 3, 4

`WasmSandbox` (line 213) stores a dedicated `tokio::runtime::Runtime` as a struct field. This runtime is created with `Runtime::new()` (default config, line 228) — no worker thread count tuning, no metrics hook, no `RuntimeMetrics` collection. The sandbox is invoked via `rt.block_on(...)` for each operation (write_file, exec_js, exec, read_file, etc.), meaning each sandbox call synchronously blocks the calling thread waiting for the sandbox runtime to complete. If the sandbox future hangs, there is no timeout, no metrics trail, and no way to detect the hang from the parent runtime.

**D-METRIC-007: Missing budget_forced_yield tracking — undetected compute-heavy async tasks** | Global (no tokio-metrics integration) | **Medium** | Sources 2, 3

Tokio's cooperative scheduling uses a poll budget to prevent a single task from monopolizing a worker thread. When a task exceeds the budget, it is "budget forced yielded" — recorded in `RuntimeMetrics.budget_forced_yield_count`. This is the single most important metric for detecting compute-heavy async tasks that should be moved to `spawn_blocking`. NeoTrix has no `tokio-metrics` integration, so budget forced yields are invisible. The `nt_core_parallel/executor.rs` and `nt_core_parallel/coordinator.rs` spawn many concurrent tasks via `tokio::spawn`, but without budget yield tracking, there is no feedback loop to identify tasks that should be offloaded to the blocking pool.

**D-METRIC-008: No runtime-specific metric tagging — multi-runtime architecture unmeasurable** | `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/factory.rs` | **Low** | Sources 3, 4

NeoTrix creates multiple independent Tokio runtimes (main runtime, sandbox runtimes, factory fallback runtimes). The tokio-metrics `RuntimeMonitor` and OpenTelemetry best practices (Source 3) require per-runtime metric tagging to distinguish metrics from different runtime instances. NeoTrix has neither the tagging infrastructure nor the per-runtime monitor instances. Even if tokio-console were added, it would need per-runtime address configuration since each runtime exposes a separate gRPC endpoint.

## Key Insights

1. **Async observability is a critical gap**: NeoTrix is entirely async Tokio-based but has zero runtime-level observability. This was flagged in batches 755, 806, 809, 814, 818, 839, 862, and 865 but remains unfixed. The gap is now 8+ batches old.

2. **HeartbeatAggregator needs async-aware extension**: The current `HeartbeatAggregator` only tracks application-level component health (Healthy/Degraded/Unhealthy). It should be extended with Tokio runtime metrics: worker utilization, poll duration percentiles, budget yield count, blocking thread pool depth, and task count. This would give GWT attention routing actual runtime data to modulate attention based on executor health.

3. **std::thread::sleep is an async antipattern**: Found in 8+ locations across crawl, event bus, proxy, KB, and browse modules. Each one blocks a Tokio worker thread. The fix is straightforward: replace with `tokio::time::sleep` in async contexts. For the event bus fire-and-forget handler (nt_core_event_bus.rs:464), the 10ms sleep should be `tokio::time::sleep(Duration::from_millis(10)).await`.

4. **Runtime proliferation is a design concern**: The factory.rs pattern of spawning new runtimes in separate threads (with `std::mem::forget`) is a workaround for nested runtime panics but creates CPU oversubscription and metric fragmentation. A better approach would be a shared runtime registry that all NeoTrix subsystems use, with per-subsystem metric labels.

5. **tokio-console integration should be dev-dependency, gated behind tokio_unstable**: Per Tokio's own recommendation and batch 755's proposed fix, add `console-subscriber = "0.4"` as a dev-dependency gated behind `#[cfg(tokio_unstable)]`, with `RUSTFLAGS="--cfg tokio_unstable"` in `.cargo/config.toml` for development builds only.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Severity: High | 3 |
| Severity: Medium | 4 |
| Severity: Low | 1 |
| Sources analyzed | 10 |
| Unique files referenced | 15+ |
