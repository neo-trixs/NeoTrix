# Iteration Batch 882 — Agent 1

## Deep Research: Rust Async Runtime Configuration, Tokio Builder, Thread Pool Sizing

**Research Date**: 2026-09-07
**Sources**: Tokio docs.rs, Tokio GitHub, DeepWiki, production tuning guides, async pitfall analyses

---

## Defect 1: No Centralized Runtime Builder — Scattered `Runtime::new()` Allocations

**Severity**: High
**Files**: `nt_core_llm.rs:1022`, `reasoning_engine/engine_core.rs:2571`, `nt_shield_sandbox_entry.rs` (239-310), `nt_meta_async_safety.rs`, `neotrix_dl.rs:4`, `todo_parallel.rs:7`, `proxy.rs:21`, `guji_llm_pool_test.rs:84`
**Count**: 100+ call sites

**Problem**: NeoTrix creates ad-hoc `tokio::runtime::Runtime::new()` or `Builder::new_multi_thread().enable_all().build()` in at least 6 independent locations with no shared configuration. Each creates its own thread pool, worker count, and blocking thread limit with no tuning for containerized environments (K8s CPU quotas, Docker cgroups). Tokio docs explicitly state: *"The tokio runtime is not NUMA aware. You may want to start multiple runtimes instead of a single runtime for better performance on NUMA systems."* — but NeoTrix does the opposite: many uncoordinated runtimes with default settings.

**Impact**: On a 4-core container with 2 CPU quota, Tokio will create 4 worker threads (detected cores) instead of 2, causing CPU throttling and latency spikes. Each separate `Runtime::new()` allocates 4 workers + 512 blocking thread cap = potential 2048+ OS threads across 4 runtimes.

**Evidence**:
```
neotrix-core/src/bin/neotrix_dl.rs:4:        #[tokio::main]
neotrix-core/examples/todo_parallel.rs:7:    #[tokio::main]
neotrix-core/examples/proxy.rs:21:           #[tokio::main]
neotrix-core/examples/guji_llm_pool_test.rs:84: #[tokio::main]
reasoning_engine/engine_core.rs:2571:        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("tokio rt");
nt_memory_api.rs:600:                       tokio::runtime::Builder::new_current_thread()
```

**Fix**: Introduce a shared `NeotrixRuntime` builder module (e.g., `nt_core_runtime`) that:
1. Reads CPU quota from `std::thread::available_parallelism()` or cgroup v2 `/sys/fs/cgroup/cpu.max`
2. Provides a single `build_worker()` and `build_blocking()` with environment-aware defaults
3. All subsystems call `NeotrixRuntime::worker_handle()` instead of `Runtime::new()`
4. Deprecate raw `#[tokio::main]` in favor of the centralized builder

---

## Defect 2: `block_on` Called From Tokio Worker Threads — Deadlock Risk

**Severity**: Critical
**Files**: `nt_core_forecast.rs:371`, `engine_core.rs:467-469,1649-1651,1705-1707`, `nt_shield_sandbox/mod.rs:586,877,896,981,994`, `nt_shield_sandbox/remote.rs:248`, `nt_shield_sandbox_entry.rs:239-310`, `nt_core_resource_pool/discovery.rs:115,134`, `nt_shield_stealth_net/geo_proxy.rs:392-393`

**Problem**: At least 30 call sites use `Handle::block_on()` or `Runtime::block_on()` from within Tokio async contexts. Tokio docs explicitly state: *"Handle::block_on does not drive the runtime. There must be at least one call to Runtime::block_on when using the current thread runtime. Handle::block_on is not enough."* and *"This function panics if called within an asynchronous execution context."* When called from a multi-thread worker, it can deadlock: the thread blocks waiting for a future that needs that same thread to poll.

**Impact**: Intermittent production deadlocks under load. The `nt_shield_sandbox_entry.rs` is particularly dangerous — it calls `.block_on()` 8 times (lines 239-310) inside what appears to be sync API surface exposed to consumers, meaning every call into the sandbox from a Tokio context risks deadlock.

**Evidence**:
```rust
// nt_core_forecast.rs:371 — called from Handle context
Ok(handle) => handle.block_on(self.0.complete_single(provider_name, request)),

// engine_core.rs:469 — block_in_place wrapping Handle::block_on (safe pattern, but fragile)
if let Ok(cot_output) = tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(cot_future)) {

// nt_shield_sandbox_entry.rs:239 — sync API calling block_on inside async runtime
.block_on(self.inner.write_file("/input", input))
```

**Fix**:
1. Replace `Handle::block_on()` with `.await` wherever the caller is async
2. For sync API surfaces, use `spawn_blocking` + `block_on` inside the blocking thread (not the worker thread)
3. Add `#[tokio::test]` with `#[should_panic]` tests to catch `block_on` in async contexts
4. Audit `nt_shield_sandbox_entry.rs` — convert all 8 `block_on` calls to async methods or explicit `spawn_blocking`

---

## Defect 3: `std::thread::sleep` Inside Async Task — Executor Starvation

**Severity**: High
**File**: `crates/neotrix-types/src/core/nt_core_meta/self_model.rs:281`

**Problem**: A synchronous `std::thread::sleep(Duration::from_millis(50))` is called inside what appears to be async-aware code. Tokio docs: *"A blocking operation, one that makes the thread wait without yielding, never gives the thread back, so it stalls every other task scheduled there."* Even 50ms of blocking on a 4-worker runtime can stall 4 concurrent tasks.

**Impact**: Under load, this single call can cause 50ms of latency for all tasks scheduled on that worker thread. With 4 workers, 4 concurrent tasks are affected simultaneously.

**Evidence**:
```rust
// self_model.rs:281
std::thread::sleep(std::time::Duration::from_millis(50));
```

**Fix**: Replace with `tokio::time::sleep(Duration::from_millis(50)).await` — requires making the containing function async. If the function cannot be async, use `tokio::task::spawn_blocking` to offload the sleep to the blocking pool.

---

## Defect 4: Unbounded Blocking Thread Pool — No Backpressure on `spawn_blocking`

**Severity**: Medium
**Files**: `nt_memory_api.rs:369,403`, `nt_meta_async_safety.rs:23,138`, `geo_proxy.rs:414`

**Problem**: NeoTrix uses `spawn_blocking` in several locations but never configures `max_blocking_threads`. The default is 512 — Tokio docs warn: *"since the queue does not apply any backpressure, it could potentially grow unbounded."* Under sustained load, 512 blocking threads consume 512 * 2MiB stack = 1GiB of stack memory alone, plus OS thread overhead.

**Impact**: Memory bloat under load. On a memory-constrained container, this can trigger OOM kills. The blocking thread pool also has a 10-second idle timeout (default), meaning thread churn under variable load, adding allocation pressure.

**Evidence**:
```rust
// nt_memory_api.rs:369 — no semaphore limiting concurrency
let reachable = tokio::task::spawn_blocking({
// nt_memory_api.rs:403 — same pattern
let processed = tokio::task::spawn_blocking(move || {
// geo_proxy.rs:414 — same pattern
let task = tokio::task::spawn_blocking(move || {
```

**Fix**:
1. Configure `max_blocking_threads` explicitly based on environment (e.g., 64 for containers, 128 for bare metal)
2. Wrap `spawn_blocking` calls with a `Semaphore` to enforce concurrency limits:
   ```rust
   static BLOCKING_SEM: Semaphore = Semaphore::const_new(32);
   let permit = BLOCKING_SEM.acquire().await.unwrap();
   spawn_blocking(move || { /* work */ }).await
   ```
3. Monitor blocking thread utilization via `RuntimeMetrics::blocking_thread_count()`

---

## Defect 5: Missing `enable_all()` on Manual Builder — Silent IO/Time Driver Omission

**Severity**: Medium
**File**: `nt_memory_api.rs:600`

**Problem**: `Builder::new_current_thread()` is used without `.enable_all()`. Tokio docs: *"When configuring a runtime by hand, no resource drivers are enabled by default. In this case, attempting to use networking types or time types will fail."* This means any attempt to use `tokio::time::sleep`, `TcpStream`, or `tokio::net` from within this runtime will panic at runtime.

**Impact**: Silent runtime failures when code evolves to require IO/timers. This is a ticking time bomb — the current-thread runtime is used for KB operations, and any future addition of async networking (e.g., HTTP health check, metrics export) will fail without compile-time warning.

**Evidence**:
```rust
// nt_memory_api.rs:600
tokio::runtime::Builder::new_current_thread()
    // Missing .enable_all() or .enable_io().enable_time()
```

**Fix**: Add `.enable_all()` or explicitly `.enable_io().enable_time()` to all manual `Builder` configurations. Consider adding a lint or custom `NeotrixRuntimeBuilder` wrapper that enforces `enable_all()` by default.

---

## Defect 6: `futures::executor::block_on` Mixed With Tokio — Cross-Runtime Hazard

**Severity**: High
**File**: `nt_core_resource_pool/discovery.rs:115,134`

**Problem**: Uses `futures::executor::block_on` (a non-Tokio executor) inside code that may already be running on a Tokio runtime. The `futures` executor does not integrate with Tokio's IO driver, timers, or work-stealing scheduler. This creates two independent executor worlds: tasks scheduled on the `futures` executor cannot interact with Tokio primitives, and blocking inside a Tokio worker thread freezes the scheduler.

**Impact**: If `discovery.rs` is called from a Tokio worker thread, it blocks that thread entirely. The `futures` executor also lacks Tokio's fairness guarantees and cooperative scheduling.

**Evidence**:
```rust
// discovery.rs:115
futures::executor::block_on(self.cache.check_and_mark(&r.kind, &r.resource_id));
// discovery.rs:134
futures::executor::block_on(self.cache.check_and_mark(&r.kind, &r.resource_id));
```

**Fix**: Replace `futures::executor::block_on` with either:
1. `tokio::task::spawn_blocking` if the cache check is blocking, or
2. Make the containing function async and use `.await` if the cache supports async
3. If the cache is inherently sync, wrap in `tokio::task::spawn_blocking` to isolate from the worker thread

---

## Defect 7: Per-Benchmark `Runtime::new()` — Bench Unreliability

**Severity**: Low
**File**: `benches/_disabled/performance_benchmark.rs` (20+ occurrences)

**Problem**: Every benchmark iteration creates a fresh `Runtime::new()` with default settings. This measures cold-start overhead, not steady-state performance. Tokio docs: *"A multi-threaded runtime is always running because it spawns its own worker threads."* Each `Runtime::new()` spawns N worker threads + thread pool initialization — repeated 20+ times in benchmarks, this inflates latency numbers and masks actual bottleneck patterns.

**Impact**: Benchmark results are unreliable for production tuning decisions. The cold-start cost (thread spawning, FD table pre-warming) dominates measurements.

**Evidence**:
```rust
// performance_benchmark.rs:25
let _ = tokio::runtime::Runtime::new().unwrap().block_on(arch.init());
// ... repeated 20+ times
```

**Fix**: Create a single `Runtime` at benchmark setup and reuse it across iterations. Use `criterion::Benchmark::iter_custom` to separate warm-up from measurement.

---

## Summary Table

| # | Defect | Severity | Category | Fix Effort |
|---|--------|----------|----------|------------|
| 1 | No centralized runtime builder | High | Architecture | Medium |
| 2 | `block_on` from worker threads | Critical | Deadlock | High |
| 3 | `std::thread::sleep` in async | High | Executor starvation | Low |
| 4 | No backpressure on `spawn_blocking` | Medium | Resource exhaustion | Low |
| 5 | Missing `enable_all()` on manual builder | Medium | Runtime correctness | Low |
| 6 | `futures::executor::block_on` mixed with Tokio | High | Cross-runtime hazard | Medium |
| 7 | Per-benchmark `Runtime::new()` | Low | Benchmark validity | Low |

---

## Tokio Runtime Configuration Reference (from research)

### Optimal Defaults for NeoTrix (Production)
```rust
use tokio::runtime::Builder;
use std::time::Duration;

let rt = Builder::new_multi_thread()
    .worker_threads(num_cpus::get().min(4))  // respect container limits
    .max_blocking_threads(64)                // bounded, not 512
    .thread_keep_alive(Duration::from_secs(10))
    .thread_name("neotrix-worker")
    .enable_all()
    .build()
    .unwrap();
```

### Key Knobs
| Setting | Default | Recommendation |
|---------|---------|----------------|
| `worker_threads` | num_cpus | Match container CPU quota |
| `max_blocking_threads` | 512 | Cap at 64-128 for most workloads |
| `global_queue_interval` | 31 (dynamic) | Lower (1) for latency-sensitive; higher for throughput |
| `event_interval` | 61 | Lower for timer-heavy workloads |
| `thread_keep_alive` | 10s | Keep default; reduce for bursty workloads |
| `disable_lifo_slot` | false | Set true if task "scheduled" time high but runtime underutilized |

### Critical Rules
1. **Never call `block_on` from a Tokio worker thread** — use `.await` or `spawn_blocking`
2. **Never use `std::thread::sleep` in async tasks** — use `tokio::time::sleep`
3. **Always `enable_all()` on manual builders** — missing IO/time driver = runtime panics
4. **Bound `spawn_blocking` with a semaphore** — default 512 unbounded can OOM
5. **Use `tokio::sync::Mutex` only when holding lock across `.await`** — `std::sync::Mutex` is preferred for short critical sections
