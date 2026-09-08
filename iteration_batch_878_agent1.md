# Batch 878 Agent 1 — Rust Async Runtime Configuration, Tokio Builder, Thread Pool Sizing

**Research Topic**: Tokio runtime configuration best practices, Builder API, thread pool sizing, and common async defects
**Date**: 2026-09-07
**Sources**: Tokio docs.rs, tokio-rs/tokio GitHub issues, production runbooks (techbuddies.io, rustz2h.com, johal.in, learnrust.net, microsoft RustTraining), DeepWiki, mintlify wiki, Rust Users Forum

---

## 1. Executive Summary

Tokio's runtime is both executor and scheduler. Misconfiguration rarely crashes the process — it silently degrades throughput, inflates tail latency, and creates starvation under load. The 5 most damaging defect categories are: blocking calls on worker threads, mis-sized runtime for environment, over-spawning micro-tasks, nested `block_on` deadlock risks, and missing backpressure/cancellation. NeoTrix exhibits all 5 categories across its codebase.

---

## 2. Research Findings

### 2.1 Tokio Runtime Architecture

- **Multi-thread scheduler**: Work-stealing thread pool. Default: 1 worker per CPU core. Fixed at startup — never changes.
- **Current-thread scheduler**: Single-threaded. Tasks execute only when `Runtime::block_on` is called.
- **Blocking pool**: Separate thread pool for `spawn_blocking`. Default max: 512 threads. Idle threads die after 10s (`thread_keep_alive`).
- **LIFO slot**: Each worker has a non-stealable slot for the most recently woken task. Can be disabled via `disable_lifo_slot` (unstable).
- **Local queue**: 256 tasks per worker. Overflow moves half to global queue.
- **Global queue interval**: Dynamic heuristic targeting ~10ms between global queue checks. Configurable via `global_queue_interval`.
- **Event interval**: Default 61 ticks between I/O/timer checks. Configurable via `event_interval`.

### 2.2 Builder API Configuration Knobs

| Knob | Default | Recommendation |
|------|---------|----------------|
| `worker_threads(n)` | num CPU cores | Set explicitly for containers; `min(cores, container_quota)` |
| `max_blocking_threads(n)` | 512 | Lower for memory-constrained; raise for heavy blocking workloads |
| `thread_keep_alive(duration)` | 10s | Increase if blocking tasks are bursty to avoid re-spawn overhead |
| `thread_stack_size(n)` | 2 MiB | Reduce for lightweight tasks; increase for deep async recursion |
| `global_queue_interval(n)` | dynamic (~10ms) | Lower for fairness; higher for throughput |
| `event_interval(n)` | 61 | Lower for I/O-sensitive; higher for batch throughput |
| `enable_io()` / `enable_time()` | off (if manual) | Always use `enable_all()` or enable explicitly |
| `disable_lifo_slot()` | enabled (unstable) | Disable if tasks have long poll times but runtime underutilized |

### 2.3 Container / NUMA Awareness

- Tokio is **not NUMA-aware**. On NUMA systems, use multiple runtimes per NUMA node.
- In containers with CPU quotas, `num_available_cores()` may report host cores, not container limits. Use `std::thread::available_parallelism()` or env var `TOKIO_WORKER_THREADS`.
- On high-core-count systems (e.g., AMD Threadripper 96-core), spawning all worker threads at once causes massive address space reservation (~64MB/thread with jemalloc = 6GB), which can crash JIT compilers (Cranelift) due to 2GB displacement limits (tokio-rs/tokio#7909).

---

## 3. NeoTrix Defects Identified

### DEFECT 1: No Centralized Tokio Runtime Configuration — All `#[tokio::main]` Use Defaults

**Severity**: HIGH | **Category**: Runtime Misconfiguration

**Evidence**: 5 files use `#[tokio::main]` with zero explicit configuration:
- `neotrix-core/src/bin/neotrix_dl.rs:4`
- `neotrix-core/examples/todo_parallel.rs:7`
- `neotrix-core/examples/proxy.rs:21`
- `neotrix-core/examples/guji_llm_pool_test.rs:84`
- 2 production Builder usages (engine_core.rs:2571, nt_memory_api.rs:600) both use defaults

**Impact**: 
- Worker thread count = host CPU cores, not container quota → thread thrash in k8s/Docker
- `max_blocking_threads` = 512 (too high for memory-constrained; too low for heavy crawling)
- `thread_keep_alive` = 10s (may cause blocking thread churn under bursty workloads)
- `thread_stack_size` = 2MiB (may cause stack overflow in deep async recursion chains)
- No `thread_name` set → impossible to distinguish Tokio workers in debugger/`top`

**Fix**:
```rust
// Create a shared runtime factory for NeoTrix
pub fn create_neotrix_runtime() -> tokio::runtime::Runtime {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(cores)
        .max_blocking_threads(cores * 16)  // proportional, not hardcoded 512
        .thread_keep_alive(std::time::Duration::from_secs(30))
        .thread_stack_size(2 * 1024 * 1024)
        .thread_name_fn(|| {
            static ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            format!("nt-worker-{}", ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
        })
        .enable_all()
        .build()
        .expect("failed to build neotrix runtime")
}
```

---

### DEFECT 2: Pervasive `std::thread::sleep` Inside Async Contexts (46 occurrences)

**Severity**: CRITICAL | **Category**: Blocking the Executor

**Evidence**: 46 `std::thread::sleep` calls found in production (non-test) code. Critical offenders:
- `nt_world_crawl/fetcher.rs:265,278,291,408` — sleep inside crawl loops
- `nt_world_browse/session.rs:64,115` — sleep during browser session
- `nt_core_forecast.rs:440` — 1200ms sleep in LLM retry loop
- `nt_memory_crawl.rs:346,950` — sleep in crawl pipeline
- `nt_http.rs:242,374` — sleep in HTTP retry logic
- `nt_world_scrape.rs:287,348` — sleep in scrape pipeline
- `nt_core_event_bus.rs:464,645` — sleep in event bus
- `nt_core_rule_memory.rs:478` — sleep in rule memory

**Impact**: Each `std::thread::sleep` blocks a Tokio worker thread for the duration. During that time, **zero other tasks** can be scheduled on that worker. Under load with 4 workers and 4 concurrent crawl requests, ALL workers can be blocked simultaneously → complete throughput collapse.

**Fix**: Replace all `std::thread::sleep(dur)` in async code with `tokio::time::sleep(dur).await`. For sync contexts that must delay, use `spawn_blocking` + `tokio::time::sleep`.

---

### DEFECT 3: Unbounded Channels Without Backpressure (3 production locations)

**Severity**: HIGH | **Category**: Missing Backpressure

**Evidence**:
- `nt_io_plugin/registry.rs:436` — `unbounded_channel` for file watch events
- `nt_shield_proxy_kernel/kernel.rs:128` — `unbounded_channel` for listener errors
- `nt_io_hotreload/mod.rs:137` — `unbounded_channel` for hot-reload events

**Impact**: Unbounded channels accept unlimited messages. If the producer outpaces the consumer (e.g., rapid file changes during hot-reload, error storm in proxy kernel), memory grows without bound → OOM kill. No backpressure signal reaches the producer.

**Fix**: Replace with bounded channels: `mpsc::channel(256)` or `mpsc::channel(1024)`. Use `try_send()` for non-blocking producers. Log/trace on backpressure events.

---

### DEFECT 4: Runtime Memory Leaks via `std::mem::forget(rt)`

**Severity**: MEDIUM | **Category**: Resource Leak / Architectural Debt

**Evidence**: `factory.rs:1447,1462` — Two explicit `std::mem::forget(rt)` calls to prevent `Runtime::drop` from blocking on `BlockingPool::shutdown`.

**Impact**: 
- Leaked Tokio runtimes + their blocking thread pools persist for process lifetime
- If `spawn_blocking` tasks were in-flight when the runtime was "forgotten", those threads and their stack allocations (2MiB each) are permanently leaked
- The comment acknowledges the issue: "spawn_blocking 的 reqwest 阻塞请求被 timeout 中断, 其后台线程仍可能存活"
- Each leaked runtime = 1 blocking pool with potential zombie threads

**Fix**: Use `Runtime::shutdown_timeout(Duration::from_secs(5))` instead of `mem::forget`. Or restructure to use the main application runtime instead of creating separate runtimes. The root cause is that `create_gateway()` creates a new runtime from within an existing runtime context — this architectural pattern should be eliminated.

---

### DEFECT 5: `Handle::block_on` Inside Sync Functions Called from Async Contexts

**Severity**: HIGH | **Category**: Nested Runtime / Deadlock Risk

**Evidence**:
- `engine_core.rs:469` — `block_in_place || Handle::current().block_on(cot_future)`
- `engine_core.rs:1651` — `block_in_place || handle.block_on(gateway_ref.complete(&request))`
- `engine_core.rs:1707` — `block_in_place || handle.block_on(panel.run_async(...))`
- `nt_io_messaging.rs:429` — `rt.block_on(async move { ... })` inside `send()` (sync trait)
- `nt_io_hotreload/mod.rs:298` — `rt.block_on(pp.reload_subscriptions())` inside a sync callback
- `nt_mind_background_loop/builder.rs:79` — `rt.block_on(async { ... })` in `with_builtin_plugins()` builder method
- `nt_core_forecast.rs:371` — `handle.block_on(self.0.complete_single(...))` in sync method
- `nt_shield_sandbox_entry.rs:239-311` — 8 instances of `self.rt.block_on(...)` in sync wrapper methods

**Impact**: `Handle::block_on` from within a Tokio worker thread blocks that thread while waiting for the future to complete. If the blocked future needs to be polled by the same worker (e.g., it spawns tasks on the same runtime), this is a **deadlock**. The `block_in_place` + `Handle::block_on` pattern in engine_core.rs is the "correct" workaround, but it still occupies a worker thread for the entire duration of the LLM call.

**Fix**:
- `nt_io_messaging.rs`: Make `send()` async or use `spawn_blocking`
- `nt_shield_sandbox_entry.rs`: Make methods async, expose async API
- `engine_core.rs`: The `block_in_place` pattern is acceptable but should be documented; consider moving to fully async engine core
- `nt_mind_background_loop/builder.rs:79`: Use `tokio::spawn` for plugin registration instead of `block_on`

---

### DEFECT 6: Pervasive `tokio::runtime::Runtime::new()` in Production Code

**Severity**: MEDIUM | **Category**: Runtime Proliferation / Performance

**Evidence**: 50 `Runtime::new()` calls found (many in tests, but several in production):
- `nt_core_forecast.rs:373` — creates new runtime per fallback LLM call
- `factory.rs:1438,1453` — creates new runtime for gateway initialization
- `nt_shield_sandbox_entry.rs:228` — creates runtime per sandbox instance
- `nt_shield_sandbox/mod.rs:585` — creates runtime per sandbox cancel
- `rule_api.rs:192` — creates runtime for port reading

**Impact**:
- Each `Runtime::new()` spawns default num-CPU-cores worker threads + blocking pool
- Multiple runtimes in same process compete for CPU, cause thread explosion
- Context switching overhead between runtimes
- `Runtime::new()` in hot paths (e.g., per LLM fallback call in forecast) is expensive

**Fix**: Pass the application's Tokio `Handle` or `Runtime` through dependency injection. Eliminate ad-hoc runtime creation. For sandbox, store the runtime in the struct (as done in `nt_shield_sandbox_entry.rs:228` — but this creates one per sandbox instance, still wasteful).

---

### DEFECT 7: Missing `enable_all()` / Incomplete Runtime Initialization

**Severity**: LOW-MEDIUM | **Category**: Runtime Misconfiguration

**Evidence**: When `Builder` is used manually, `enable_all()` is sometimes omitted:
- `nt_memory_api.rs:600` — Uses `Builder::new_current_thread().enable_all()` (correct)
- `engine_core.rs:2571` — Uses `Builder::new_multi_thread().enable_all()` (correct)
- But `factory.rs:1438` uses `Runtime::new()` which implicitly enables all drivers

**Impact**: If `enable_io()` or `enable_time()` is not called on a manually-built runtime, attempting to use `TcpStream`, `Sleep`, or any I/O/timer type will panic at runtime. Currently all manual Builder usages call `enable_all()`, but this is not enforced by convention — a new developer could easily omit it.

**Fix**: Add a lint/assertion in the shared runtime factory that `enable_all()` is always called. Consider a `NeoTrixRuntimeBuilder` wrapper that enforces correct configuration.

---

## 4. Architectural Recommendations

### 4.1 Centralized Runtime Management

```rust
// Proposed: neotrix-core/src/runtime.rs
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn global_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        Builder::new_multi_thread()
            .worker_threads(cores)
            .max_blocking_threads(cores * 16)
            .thread_keep_alive(Duration::from_secs(30))
            .thread_name("nt-worker")
            .enable_all()
            .build()
            .expect("neotrix runtime")
    })
}
```

### 4.2 Eliminate `std::thread::sleep` in Async Code

Replace all 46 occurrences. Priority order:
1. `nt_world_crawl/fetcher.rs` (4 occurrences, hot crawl path)
2. `nt_core_forecast.rs` (LLM retry backoff — use exponential backoff crate)
3. `nt_http.rs` (HTTP retry — use `tokio::time::sleep`)
4. All remaining crawl/scrape/browse files

### 4.3 Replace Unbounded Channels

Convert 3 unbounded channels to bounded with appropriate capacity:
- File watch events: `channel(256)` — file changes are bursty but bounded
- Proxy errors: `channel(64)` — errors should not flood
- Hot-reload events: `channel(32)` — config changes are rare

### 4.4 Eliminate Runtime Memory Leaks

Replace `std::mem::forget(rt)` with:
```rust
// Option A: timeout shutdown
rt.shutdown_timeout(Duration::from_secs(5));

// Option B: share the application runtime
// Pass Handle to create_gateway instead of creating new runtime
```

---

## 5. Performance Impact Estimates

| Defect | Impact Under Load | Estimated Throughput Loss |
|--------|-------------------|--------------------------|
| std::thread::sleep on workers | Worker starvation | 30-80% (depends on sleep duration) |
| Unbounded channels | OOM → process kill | 100% (fatal) |
| Runtime proliferation | Thread explosion, context switch overhead | 10-25% |
| Handle::block_on deadlock | Complete stall | 100% (deadlock) |
| No runtime sizing for containers | Oversized/undersized pool | 15-40% |

---

## 6. Verification Commands

After fixes, verify with:
```sh
cargo check --all-targets -p neotrix
cargo test -p neotrix --lib
# Search for remaining std::thread::sleep in async contexts:
rg 'std::thread::sleep' neotrix-core/src/ --include '*.rs' -l
# Search for unbounded_channel:
rg 'unbounded_channel' neotrix-core/src/ --include '*.rs'
# Search for mem::forget on runtime:
rg 'mem::forget' neotrix-core/src/ --include '*.rs'
```

---

## 7. References

1. Tokio Runtime Docs: https://docs.rs/tokio/latest/tokio/runtime/
2. Tokio Builder API: https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html
3. Tokio Best Practices Issue: https://github.com/tokio-rs/tokio/issues/7982
4. Tokio max_worker_threads Issue: https://github.com/tokio-rs/tokio/issues/7909
5. "Top 5 Tokio Runtime Mistakes" (techbuddies.io, 2026-03-21)
6. "Optimize Rust Async/Await with Tokio" (blogs.abhipanseriya.dev, 2026-06-26)
7. "Tokio Runtime Tuning for Production" (rustz2h.com, 2026)
8. "Debugging Concurrency Bugs in Tokio" (johal.in, 2026-03-18)
9. "Common Pitfalls" — Microsoft RustTraining async-book ch12
10. "Async Pitfalls" — learnrust.net ch23.4
11. DeepWiki: Tokio Runtime Initialization: https://deepwiki.com/tokio-rs/tokio/3.1
12. Mutex Deadlock in Production (juanchi.dev, 2026-05-07)
