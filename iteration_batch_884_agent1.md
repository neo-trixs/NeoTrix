# Iteration Batch 884 — Agent 1: Rust Async Runtime Configuration

**Topic**: Tokio Builder, Thread Pool Sizing, Runtime Optimization
**Date**: 2026-09-07
**Sources**: Tokio docs.rs, tokio-rs/tokio GitHub, Tokio blog, multiple production guides (2025-2026)

---

## Executive Summary

Tokio's runtime configuration is a high-leverage but under-documented area. NeoTrix currently uses `tokio::runtime::Runtime::new()` (default config) in 50+ locations across the codebase, with only 2 places using explicit `Builder` configuration. This creates a systemic risk: the default worker thread count equals `num_cpus()`, the blocking pool defaults to 512 threads, and no thread naming/stack/keep_alive is configured. Combined with widespread `std::fs` and `std::thread::sleep` usage in async contexts, NeoTrix has significant runtime starvation risk.

---

## Tokio Runtime Architecture (Key Facts)

| Parameter | Default | Notes |
|-----------|---------|-------|
| `worker_threads` | `num_cpus()` | One per core; set via `Builder::new_multi_thread()` |
| `max_blocking_threads` | 512 | For `spawn_blocking` pool |
| `thread_stack_size` | 2 MiB | Configurable via `Builder::thread_stack_size()` |
| `global_queue_interval` | Dynamic (multi-thread), 31 (current-thread) | Targets 10ms between global queue checks |
| `event_interval` | 61 | Ticks between I/O/timer event checks |
| `thread_keep_alive` | 10 seconds | Blocking thread idle timeout |
| Local queue capacity | 256 tasks | Overflow moves half to global queue |
| LIFO slot | Enabled (multi-thread) | Can be disabled via `disable_lifo_slot()` |

### Scheduler Types
- **Multi-thread**: Work-stealing across N worker threads; recommended for most apps
- **Current-thread**: Single-threaded; requires `block_on` to drive; panics on `block_in_place`
- Local queue = 256 max; overflow spills to global queue
- Work-stealing moves half of tasks from victim's local queue
- LIFO slot optimization: wake-to-same-thread pattern avoids queue contention

---

## Defect Analysis for NeoTrix

### DEFECT-1: Pervasive Default Runtime Construction — No Tuning

**Severity**: HIGH
**File references**: 50+ locations across `neotrix-core/src/unified/`, `neotrix-core/tests/`

NeoTrix creates `tokio::runtime::Runtime::new()` (default builder) in 50+ locations, including:
- `nt_core_forecast.rs:373`
- `nt_shield_sandbox/mod.rs:585,876`
- `nt_shield_sandbox_entry.rs:228`
- `nt_core_llm.rs:1022,1032,1043,1056,1099,1110`
- `nt_core_consciousness_core.rs:1868`
- `seal_drive.rs:9`
- `test_stealth_net_e2e.rs:24,35,45,67,81,96`

**Impact**: Each `Runtime::new()` creates a fresh runtime with `num_cpus()` worker threads. In benchmarks (`_disabled/performance_benchmark.rs`), a new runtime is created **per benchmark iteration** — this allocates N threads, N parking structures, and one blocking pool per call. Under concurrent load, this causes thread proliferation and memory bloat.

**Recommendation**: Create a single shared runtime (via `lazy_static` or `OnceCell`) or pass a `Handle` through the call chain. Never create `Runtime::new()` inside hot paths or per-request handlers.

---

### DEFECT-2: Missing `enable_all()` on Manual Builder

**Severity**: HIGH
**File references**: `nt_core_forecast.rs:370-377`, `nt_core_consciousness_core.rs:1867-1877`

When using `Builder::new_multi_thread()` or `Builder::new_current_thread()`, `enable_all()` is NOT called by default. Without it:
- **I/O driver is disabled** — `TcpStream`, `UdpSocket`, `UnixSocket` will panic
- **Time driver is disabled** — `tokio::time::sleep`, `Interval` will not work

Only 2 explicit Builder usages exist in NeoTrix:
1. `reasoning_engine/engine_core.rs:2571` — `Builder::new_multi_thread().enable_all().build()` ✓
2. `nt_memory_api.rs:600` — `Builder::new_current_thread().enable_all()` ✓

But the 50+ `Runtime::new()` calls DO enable both drivers (default behavior), creating an inconsistency: code migrated to explicit Builder may silently lose I/O/time support.

**Recommendation**: Always call `.enable_all()` when using `Builder`. Add a project-level lint or clippy check.

---

### DEFECT-3: Blocking Calls on Async Worker Threads (Starvation Risk)

**Severity**: CRITICAL
**File references**: 100+ `std::fs` calls, 15+ `std::thread::sleep` calls in async contexts

NeoTrix has **extensive** synchronous I/O in async code:

| Pattern | Count | Example locations |
|---------|-------|-------------------|
| `std::fs::read_to_string` | 40+ | `config.rs:35`, `agent.rs:327`, `capability_bridge.rs:347,666` |
| `std::fs::read` | 30+ | `tables.rs:221`, `structured.rs:19,122`, `core.rs:92,179` |
| `std::fs::read_dir` | 20+ | `agent.rs:316,375`, `merge.rs:370,715` |
| `std::thread::sleep` | 15+ | `nt_core_forecast.rs:440,486`, `event_bus.rs:464,645` |

Each `std::fs` call blocks the Tokio worker thread for disk I/O latency (1-100ms on HDD, 0.1-1ms on SSD). On a 4-core machine, 4 concurrent blocking calls stall the entire runtime. The `std::thread::sleep` calls are even worse — they pin the thread for the full duration.

**Key offenders**:
- `nt_core_forecast.rs:440`: `std::thread::sleep(Duration::from_millis(1200 + attempt * 800))` — blocks for **2+ seconds** in retry loop
- `nt_core_forecast.rs:486`: `std::thread::sleep(Duration::from_secs(retry_after))` — blocks for arbitrary duration
- `event_bus.rs:464,645`: `std::thread::sleep(Duration::from_millis(10-50))` — in event processing
- `registry_watcher.rs:202,252`: `std::thread::sleep(backoff)` — in file watcher loop

**Recommendation**:
1. Replace `std::fs` with `tokio::fs` or `spawn_blocking`
2. Replace `std::thread::sleep` with `tokio::time::sleep`
3. For file operations in `nt_file_ability`, wrap in `spawn_blocking` or use `tokio::fs`
4. For retry loops, use `tokio::time::sleep` with exponential backoff

---

### DEFECT-4: No Runtime Metrics or Monitoring Integration

**Severity**: MEDIUM
**File references**: None (gap in observability)

NeoTrix has no `tokio-console` integration, no `tokio-metrics` collection, and no runtime health monitoring. The only health signal is `HeartbeatAggregator` which tracks compilation/test/KB health — not runtime scheduler health.

Key metrics that should be monitored:
- `worker_busy_ratio` — fraction of time workers are busy
- `worker_mean_poll_time` — detects blocking in poll
- `blocking_thread_count` — detects pool saturation
- `scheduled_delay` — tasks waiting to be polled (saturation indicator)
- `poll_duration` — per-task blocking detection (>100μs = problem)

**Recommendation**:
1. Add `tokio-metrics` crate for production metrics
2. Wire `worker_busy_ratio` into `HeartbeatAggregator`
3. Add `tokio-console` support (behind `tokio_unstable` feature flag) for staging
4. Alert when `blocking_thread_count` approaches `max_blocking_threads`

---

### DEFECT-5: Thread Pool Sizing Not Adapted to Container/NUMA Environments

**Severity**: MEDIUM
**File references**: `reasoning_engine/engine_core.rs:2571`, `nt_memory_api.rs:600`

Tokio's default `worker_threads = num_cpus()` reads the host's CPU count, not the container's cgroup limits. In Docker/Kubernetes with CPU quotas (e.g., 2 cores on a 64-core host), Tokio spawns 64 workers but only 2 can run — causing massive context switching overhead.

Additionally, Tokio is not NUMA-aware. The official docs state: *"The tokio runtime is not NUMA (Non-Uniform Memory Access) aware. You may want to start multiple runtimes instead of a single runtime for better performance on NUMA systems."*

NeoTrix currently has no NUMA or cgroup detection.

**Recommendation**:
1. Use `num_cpus::get()` explicitly in Builder (it respects cgroups on Linux)
2. Consider `std::thread::available_parallelism()` (Rust 1.59+) which also respects cgroups
3. For NUMA-aware deployment, document that multiple runtimes may be needed
4. Add environment variable override: `NEOTRIX_WORKER_THREADS=N`

---

### DEFECT-6: Blocking Thread Pool Not Sized for Workload

**Severity**: MEDIUM
**File references**: All `Runtime::new()` calls

The default `max_blocking_threads = 512` may be too high or too low depending on workload:
- **Too high**: For NeoTrix's typical workload (LLM calls, file ops, KB queries), 512 blocking threads can consume significant memory (each thread ~2MB stack = 1GB total)
- **Too low**: If NeoTrix spawns many concurrent file operations or external API calls via `spawn_blocking`, tasks queue up

NeoTrix uses `spawn_blocking` nowhere in production code — all blocking is done inline on worker threads. This means the blocking pool is wasted capacity while the worker threads are starved.

**Recommendation**:
1. Set `max_blocking_threads` to a workload-appropriate value (64-128 for typical NeoTrix workload)
2. Migrate inline blocking calls to `spawn_blocking` with a `Semaphore` for backpressure
3. Consider dedicated thread pools for CPU-bound work via `rayon` (wrapped in `spawn_blocking`)

---

### DEFECT-7: Multiple Runtimes Created Without Handle Sharing

**Severity**: HIGH
**File references**: `nt_core_forecast.rs:370-377`, `nt_core_consciousness_core.rs:1867-1877`

NeoTrix creates independent runtimes in several sync-async bridge points:

```rust
// nt_core_forecast.rs:370
let rt = match tokio::runtime::Handle::try_current() {
    Ok(handle) => handle.block_on(...),
    Err(_) => {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(...)
    }
};
```

This creates a new runtime when no existing runtime is available, which:
1. Allocates a full thread pool (N workers + blocking pool)
2. Cannot be shared with other async tasks
3. May conflict with existing runtimes on the same threads
4. Each runtime has its own I/O driver — file descriptors are duplicated

**Recommendation**:
1. Use a global runtime (via `OnceCell<Runtime>`) for sync-to-async bridges
2. Pass `Handle` through the call chain where possible
3. Never create `Runtime::new()` inside a loop or per-request handler

---

### DEFECT-8: `global_queue_interval` and `event_interval` Not Tuned

**Severity**: LOW
**File references**: None (default values used everywhere)

NeoTrix uses default scheduler intervals:
- `global_queue_interval = 31` (current-thread) or dynamic ~10ms (multi-thread)
- `event_interval = 61`

For NeoTrix's workload (heavy LLM I/O, file operations, KB queries):
- **Lower `global_queue_interval`** (e.g., 7-15) would improve fairness for latency-sensitive tasks (LLM streaming)
- **Lower `event_interval`** (e.g., 31) would improve I/O responsiveness for crawl operations

**Recommendation**: Profile with `tokio-console` to determine if tuning is needed. For NeoTrix's mixed I/O + compute workload, `global_queue_interval(15)` and `event_interval(31)` are reasonable starting points.

---

## Recommendations Summary

| Priority | Action | Impact |
|----------|--------|--------|
| P0 | Replace `std::fs` with `tokio::fs` or `spawn_blocking` in async contexts | Eliminates worker starvation |
| P0 | Replace `std::thread::sleep` with `tokio::time::sleep` in async contexts | Eliminates thread pinning |
| P1 | Consolidate to a single shared runtime with `Handle` | Eliminates thread proliferation |
| P1 | Always call `enable_all()` on Builder | Prevents silent I/O/time driver loss |
| P2 | Add `tokio-metrics` / `tokio-console` integration | Enables runtime health monitoring |
| P2 | Set `worker_threads` explicitly with cgroup awareness | Container-safe deployment |
| P3 | Tune `global_queue_interval` and `event_interval` | Tail latency optimization |
| P3 | Size `max_blocking_threads` to workload | Memory optimization |

---

## Key Sources

1. **Tokio Builder docs**: https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html
2. **Tokio Runtime docs**: https://docs.rs/tokio/latest/tokio/runtime/index.html
3. **Tokio scheduler rewrite (2019)**: https://tokio.rs/blog/2019-10-scheduler
4. **Optimize Rust Async/Await with Tokio (2026)**: https://blogs.abhipanseriya.dev/guides/optimize-rust-asyncawait-performance-a-tokio-runbook
5. **Tokio Performance Tuning (2026)**: https://krun.pro/tokio-performance-tuning/
6. **Tokio Runtime Tuning Production (2026)**: https://rustz2h.com/chapter_07_mastering_async_rust_and_tokio/series_01_tokio_runtime_internals_and_tasks/tokio_runtime_tuning_production
7. **Common Pitfalls (Microsoft RustTraining)**: https://microsoft.github.io/RustTraining/async-book/ch12-common-pitfalls.html
8. **Top 5 Tokio Runtime Mistakes (2026)**: https://www.techbuddies.io/2026/03/21/top-5-tokio-runtime-mistakes-that-quietly-kill-your-async-rust/
9. **Tokio spawn_blocking clarification PR**: https://github.com/tokio-rs/tokio/pull/7923
10. **Tokio GitHub discussions**: https://github.com/tokio-rs/tokio/discussions/3858
