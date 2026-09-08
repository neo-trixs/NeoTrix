# Agent 1: Async Runtime Optimization (Batch 872)

## Sources

1. **Tokio Runtime Tuning for Production (2026)** — rustz2h.com/chapter_07_mastering_async_rust_and_tokio/series_01_tokio_runtime_internals_and_tasks/tokio_runtime_tuning_production
2. **Tokio Performance Tuning: Fix Bottlenecks in Async Rust** — krun.pro/tokio-performance-tuning/
3. **Optimize Rust Async/Await Performance with Tokio (Runbook)** — blogs.abhipanseriya.dev/guides/optimize-rust-asyncawait-performance-a-tokio-runbook
4. **Configure Tokio Worker Threads: Step-by-Step** — rustz2h.com/chapter_07_mastering_async_rust_and_tokio/series_01_tokio_runtime_internals_and_tasks/configure_worker_threads
5. **Blocking Operations | tokio-rs/tokio** — deepwiki.com/tokio-rs/tokio/3.4-blocking-operations
6. **Tokio Runtime Documentation** — docs.rs/tokio/latest/tokio/runtime/
7. **Rust Async Runtime Deep Dive: Tokio Architecture** — andrewodendaal.com/rust-async-runtime-tokio-architecture/
8. **I Spent 3 Months Tuning a Tokio Runtime for My Robot** — dev.to/motedb (2026-04-17)

## Defects

### D-RUN-001: Pervasive `Runtime::new().block_on()` anti-pattern creates orphan runtimes

The codebase creates fresh `tokio::runtime::Runtime` instances inside synchronous/async bridging code via `tokio::runtime::Runtime::new().unwrap().block_on(...)`. Each call spawns a full thread pool (default: num_cpus workers + up to 512 blocking threads) that is immediately discarded after the single `.block_on()` call. This wastes OS threads, prevents work-stealing with the parent runtime, and creates thread explosion under concurrent load.

Per Tokio docs: "One runtime per application is standard. Multiple runtimes have overhead and prevent work-stealing between them."

| File | Line(s) |
|------|---------|
| `nt_core_consciousness_core.rs` | 1868-1877 |
| `nt_core_llm.rs` | 1022, 1032, 1043, 1058, 1101 |
| `nt_core_forecast.rs` | 373 |
| `nt_shield_sandbox/mod.rs` | 585, 876, 895, 979, 992 |
| `nt_shield_sandbox/remote.rs` | 246 |
| `benches/_disabled/performance_benchmark.rs` | 25, 50, 51, 65, 73, 91, 99, 112, 120, 133, 142, 155, 169, 176, 190, 191, 204, 216, 223, 235, 236, 283, 286 |

**Severity**: high
**Source**: krun.pro — "One runtime per application is standard. Multiple runtimes have overhead and prevent work-stealing between them." | rustz2h.com — "Worker count is fixed at runtime creation. Create a new runtime if you need to change it."

---

### D-RUN-002: `futures::executor::block_on()` inside async context — deadlock risk

`nt_core_resource_pool/discovery.rs` calls `futures::executor::block_on()` inside an `async fn discover_all()`. The `futures::executor::block_on` creates its own single-threaded event loop, which blocks the Tokio worker thread entirely. If the inner future needs to be polled by Tokio (e.g., it spawns tasks or uses Tokio timers), this deadlocks. Even if it doesn't deadlock, it freezes the worker thread for every resource discovery, defeating async concurrency.

| File | Line(s) |
|------|---------|
| `nt_core_resource_pool/discovery.rs` | 115, 134 |

**Severity**: critical
**Source**: Tokio docs — "Issuing a blocking call or performing a lot of compute in a future without yielding is problematic, as it may prevent the executor from driving other futures forward." | krun.pro — "Calling std::thread::sleep inside an async fn blocks the OS thread entirely. Every task queued behind yours on that worker sits frozen."

---

### D-RUN-003: WasmSandbox holds unconfigured dedicated Runtime — thread explosion

`WasmSandbox` creates a dedicated `tokio::runtime::Runtime` without any Builder configuration (no `worker_threads`, `max_blocking_threads`, `thread_keep_alive`, or `enable_all()`). Each sandbox instance spawns default worker threads. When multiple sandbox instances are created concurrently (e.g., parallel code execution), this creates OS thread explosion — each sandbox gets its own full thread pool that cannot share work.

| File | Line(s) |
|------|---------|
| `nt_shield_sandbox_entry.rs` | 213, 228 |

**Severity**: high
**Source**: DPTCloud — "Restricting this pool prevents thread explosion on low-memory VPS instances." | rustz2h.com — "Defaults are sensible: one worker per core, up to 512 blocking threads. Change them with evidence."

---

### D-RUN-004: `block_in_place` used without runtime flavor guard — panics on current_thread

`engine_core.rs:469` uses `tokio::task::block_in_place(|| Handle::current().block_on(cot_future))`. The `block_in_place` function panics if called from a `current_thread` runtime. The code does not verify it is running on a multi-thread runtime before calling. If the reasoning engine is ever tested or invoked from a single-threaded context, this causes a runtime panic.

| File | Line(s) |
|------|---------|
| `engine_core.rs` | 469 |

**Severity**: medium
**Source**: Tokio docs — "block_in_place panics on a current_thread runtime." | krun.pro — "block_in_place runs on the current worker thread but tells the scheduler to off-load other tasks first, and it panics on a current_thread runtime."

---

### D-RUN-005: No blocking thread pool utilization monitoring

The codebase uses `tokio::task::spawn_blocking` in 5+ locations (tiles.rs, selection.rs, free_catalog.rs, nt_memory_api.rs) but has zero monitoring of blocking pool utilization. Per production guidance, if the blocking pool is consistently saturated (all 512 threads busy), new `spawn_blocking` calls queue and block. Without metrics, this saturation is invisible until p99 latency spikes.

| File | Line(s) |
|------|---------|
| `nt_io_web/tiles.rs` | 79 |
| `nt_io_provider/gateway/selection.rs` | 365 |
| `nt_io_provider/free_catalog.rs` | 89 |
| `nt_memory_kb/nt_memory_api.rs` | 369, 403 |

**Severity**: medium
**Source**: rustz2h.com — "Monitor blocking thread pool utilization. If it's consistently full, add a dedicated CPU thread pool with rayon." | krun.pro — "If poll_duration exceeds 100μs consistently, it's a blocking problem."

---

### D-RUN-006: No `tokio-console` or `tokio-metrics` integration — blind to saturation

The codebase has no `console-subscriber` dependency, no `tokio-metrics` RuntimeMonitor, and no `busy_ratio`/`scheduled_delay` tracking. The "5% CPU mystery" — where a service is slow but CPU appears idle — is a documented failure mode caused by worker thread saturation. Without these diagnostics, the team cannot distinguish between a blocking problem (high `poll_duration`) and a saturation problem (high `scheduled_delay`).

**Severity**: high
**Source**: krun.pro — "poll_duration measures how long a single poll held the worker thread. scheduled_delay is the time a task spent waiting to be polled after being woken. High values here mean your worker threads are saturated." | rustz2h.com — "Use tokio-console or tracing to monitor executor health in production."

---

### D-RUN-007: Benchmark file creates 20+ orphan Runtimes — skews results and wastes threads

`benches/_disabled/performance_benchmark.rs` creates `tokio::runtime::Runtime::new()` approximately 20+ times across individual benchmark functions. Each call spawns a full thread pool that is dropped after a single `block_on()`. This wastes hundreds of OS threads during benchmarking and skews timing results due to thread creation overhead. The file is disabled but represents a pattern that may be copied.

| File | Line(s) |
|------|---------|
| `benches/_disabled/performance_benchmark.rs` | 25, 50, 51, 65, 73, 91, 99, 112, 120, 133, 142, 155, 169, 176, 190, 191, 204, 216, 223, 235, 236, 283, 286 |

**Severity**: low (disabled file)
**Source**: rustz2h.com — "One big runtime is better than many small ones. Multiple runtimes have overhead and prevent work-stealing."

---

### D-RUN-008: `#[tokio::main]` default runtime used without Builder — no production tuning knobs

Several test files and the benchmark file use `Runtime::new()` or `#[tokio::main]` defaults without `Builder`, making runtime configuration impossible to version-control or tune. Per production guidance, the runtime should be built explicitly with `Builder::new_multi_thread()` so that worker count, blocking pool size, and thread names are visible and configurable.

| File | Line(s) |
|------|---------|
| `seal_drive.rs` | 9 |
| `test_stealth_net_e2e.rs` | 24, 35, 45, 67, 81, 96 |
| `mail_integration_test.rs` | 1191 |
| `seal_core/self_iterating/mod.rs` | 99, 125, 153, 237, 262 |
| `consciousness/element/registry.rs` | 341, 377 |
| `nt_shield_sandbox/judge.rs` | 570, 590, 611 |
| `nt_mind/infrastructure/tests/seal.rs` | 20, 35, 52, 71, 90, 135 |

**Severity**: low (test-only)
**Source**: rustz2h.com — "Build it explicitly so the knobs are visible and version-controlled." | Tokio docs — "Most users will use the #[tokio::main] annotation on their entry point instead."

---

## Key Insights

1. **The `Runtime::new().block_on()` anti-pattern is systemic**: NeoTrix uses this pattern in production code (consciousness core, LLM provider, forecast, sandbox) not just tests. Each call creates a full thread pool. The fix is to propagate a shared `Handle` from the parent runtime or use `Handle::current().block_on()` instead of creating new runtimes.

2. **`futures::executor::block_on()` is a deadlock bomb**: This is the most critical finding. Using `futures::executor::block_on()` inside an async Tokio context can deadlock if the inner future touches Tokio internals. It should be replaced with `tokio::task::spawn_blocking` or `Handle::current().block_on()`.

3. **WasmSandbox thread explosion is real**: Each sandbox creates its own Runtime with default config. In scenarios where multiple sandboxes run concurrently (e.g., batch code execution), this creates N × num_cpus threads competing for CPU, causing severe context-switch overhead.

4. **No observability = no diagnosis path**: Without `tokio-console` or `tokio-metrics`, the team cannot distinguish worker saturation from genuine I/O slowness. This is the "5% CPU mystery" that krun.pro documents as the most misdiagnosed failure mode in async Rust.

5. **Production runtime needs explicit Builder**: The codebase pattern of `Runtime::new()` everywhere makes it impossible to tune `worker_threads`, `max_blocking_threads`, or `thread_keep_alive` without modifying every call site. A central runtime factory with Builder configuration is needed.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Critical severity | 1 |
| High severity | 3 |
| Medium severity | 2 |
| Low severity | 2 |
| Files affected | 15+ |
