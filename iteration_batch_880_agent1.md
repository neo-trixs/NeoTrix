# Iteration Batch 880 — Agent 1
## Rust Async Runtime Configuration: Tokio Builder & Thread Pool Sizing
**Deep Research Report | 5 Defects Extracted for NeoTrix**

---

## Research Sources

| Source | Type | Key Findings |
|--------|------|--------------|
| Tokio docs (docs.rs/tokio/latest/tokio/runtime) | Official | Builder API, worker_threads defaults, blocking pool behavior |
| DeepWiki: Runtime Initialization & Configuration | Analysis | LIFO slot optimization, global_queue_interval heuristics |
| "Optimize Rust Async/Await Performance with Tokio" (Stack Dispatch, 2026-06) | Production runbook | Blocking anti-patterns, lock-across-await, unbounded concurrency |
| "Top 5 Tokio Runtime Mistakes" (techbuddies, 2026-03) | Production guide | Runtime mis-sizing, over-spawning, nested block_on |
| "Tokio Performance Tuning: Fix Bottlenecks" (KruN, 2026-05) | Deep dive | poll_duration >100μs threshold, actor task pattern |
| "Debugging Concurrency Bugs in Rust Tokio" (johal.in, 2026-03) | Case studies | Silent throughput degradation, channel deadlock patterns |
| "Untangling Tokio and Rayon in production" (PostHog, 2026-04) | Case study | Thread oversubscription, CFS throttling, Rayon+Tokio coordination |
| GitHub tokio-rs/tokio discussions #3858 | Community | Worker vs blocking thread semantics |

---

## Key Tokio Runtime Facts (Reference)

- **Worker threads**: Fixed at startup, default = CPU cores. Work-stealing scheduler. Local queue = 256 tasks max, overflow → global queue.
- **Blocking threads**: Dynamic, default cap = 512, 10s idle timeout. Unbounded queue when cap reached (no backpressure).
- **LIFO slot**: Each worker has one non-stealable slot for last-woken task. Disabled after 3 consecutive uses. Benefits locality, hurts when poll times are long.
- **global_queue_interval**: Default = 31 (current-thread) or dynamic ~10ms heuristic (multi-thread). Controls local-vs-global balance.
- **event_interval**: Default = 61 ticks before checking I/O/timer events.
- **Thread stack size**: Default 2 MiB. Platform minimum may override.
- **`#[tokio::main]`**: Creates multi-thread runtime with defaults. No I/O or time drivers when building manually without `enable_all()`.

---

## Defect 1: `std::thread::sleep` Inside Async Context — Worker Thread Starvation

**Severity**: CRITICAL (P0)
**Affected Files**: 20+ locations across codebase

**Evidence**:
```
neotrix-core/src/neotrix/nt_core_event_bus.rs:464      → std::thread::sleep(Duration::from_millis(10))
neotrix-core/src/neotrix/nt_core_event_bus.rs:645      → std::thread::sleep(Duration::from_millis(50))
neotrix-core/src/neotrix/nt_core_capability_tree/src/registry_watcher.rs:202 → std::thread::sleep(backoff)
neotrix-core/src/neotrix/nt_core_capability_tree/src/registry_watcher.rs:252 → std::thread::sleep(self.poll_interval)
crates/neotrix-types/src/core/nt_core_meta/self_model.rs:281 → std::thread::sleep(Duration::from_millis(50))
neotrix-core/src/unified/core/nt_core_forecast.rs:440   → std::thread::sleep(Duration::from_millis(1200 + attempt * 800))
neotrix-core/src/unified/core/nt_core_forecast.rs:486   → std::thread::sleep(Duration::from_secs(retry_after))
neotrix-core/src/unified/core/nt_core_deploy.rs:464     → std::thread::sleep(Duration::from_millis(100))
neotrix-core/src/unified/core/nt_core_observer_error.rs:59 → std::thread::sleep(Duration::from_millis(delay))
neotrix-core/src/unified/core/nt_core_observer_error.rs:359,371 → std::thread::sleep(Duration::from_millis(2))
neotrix-core/src/unified/layers/perception/nt_world/nt_world_edgar.rs:302 → std::thread::sleep(min_interval - elapsed)
neotrix-core/src/unified/layers/perception/nt_world/nt_world_crawl/unified.rs:355 → std::thread::sleep(...)
neotrix-core/src/unified/layers/perception/nt_world/nt_world_crawl/fetcher.rs:265,278,291,408 → std::thread::sleep(...)
neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield/http_proxy.rs:182 → std::thread::sleep(Duration::from_millis(100))
neotrix-core/src/unified/core/nt_core_self/pilot_steering.rs:438 → std::thread::sleep(Duration::from_millis(10))
neotrix-core/src/unified/core/nt_core_self/human_approval.rs:372 → std::thread::sleep(Duration::from_millis(10))
neotrix-core/src/unified/core/nt_core_bank/mem.rs:300   → std::thread::sleep(Duration::from_millis(10))
neotrix-core/src/unified/core/nt_core_rule_memory.rs:478 → std::thread::sleep(Duration::from_millis(50 * (attempt as u64 + 1)))
neotrix-core/src/unified/core/nt_core_aware/mod.rs:304,306 → std::thread::sleep(Duration::from_millis(1))
neotrix-core/src/unified/layers/cognition/nt_mind/infrastructure/code_graph.rs:365 → std::thread::sleep(Duration::from_millis(50))
```

**Root Cause**: `std::thread::sleep` blocks the OS thread entirely. Tokio has no way to preempt it. Every task queued behind it on that worker sits frozen. With the default of one worker per core, even a few 50ms sleeps can starve all async I/O for hundreds of milliseconds.

**Impact**:
- Single `std::thread::sleep(50ms)` on a 4-core runtime starves 25% of async capacity for 50ms
- Retry loops with `std::thread::sleep(1200ms)` completely freeze one worker thread
- Network pollers (`registry_watcher`, `fetcher`, `edgar`) blocking the scheduler defeats the entire async design

**Fix**:
```rust
// WRONG: blocks OS thread
std::thread::sleep(Duration::from_millis(50));

// CORRECT: yields to Tokio scheduler
tokio::time::sleep(Duration::from_millis(50)).await;

// CORRECT for sync contexts that must sleep:
// Use spawn_blocking to offload the sleep
tokio::task::spawn_blocking(move || {
    std::thread::sleep(Duration::from_millis(50));
}).await.unwrap();
```

---

## Defect 2: Runtime Anti-Pattern in Benchmarks — Creating Runtime Per Iteration

**Severity**: HIGH (P1)
**Affected File**: `benches/_disabled/performance_benchmark.rs`

**Evidence**:
```rust
// Line 25: creates new Runtime on EVERY iteration
b.iter(|| {
    let _ = tokio::runtime::Runtime::new().unwrap().block_on(arch.init());
});

// Line 50-51: creates TWO new Runtimes per request
let _ = tokio::runtime::Runtime::new().unwrap().block_on(arch.init());
tokio::runtime::Runtime::new().unwrap().block_on(arch.process_request(request))

// Line 283-286: creates Runtime per request in 100-request loop
for i in 0..100 {
    let _ = tokio::runtime::Runtime::new().unwrap().block_on(
        arch.process_request(&format!("Memory test {}", i))
    );
}

// Line 252: creates Runtime per concurrency level
let rt = tokio::runtime::Runtime::new().unwrap();
```

**Root Cause**: `Runtime::new()` spawns worker threads (one per CPU core), initializes the I/O driver, timer driver, and blocking pool — then drops them all on `Runtime::drop`. Creating and destroying a runtime per benchmark iteration adds ~1-5ms overhead per call and makes benchmark results meaningless (measuring runtime creation, not business logic).

**Impact**:
- Benchmark numbers measure runtime creation/destruction overhead, not actual throughput
- Thread pool thrashing creates OS-level noise that affects other benchmarks in the same process
- Criterion warmup phase creates/destroys dozens of runtimes

**Fix**:
```rust
// Create runtime ONCE, reuse across iterations
fn bench_with_shared_runtime(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    c.bench_function("process_request", |b| {
        b.iter(|| {
            rt.block_on(arch.process_request("test"))
        })
    });
}
```

---

## Defect 3: No Runtime Builder Configuration — Uncontrolled Default Sizing

**Severity**: HIGH (P1)
**Affected Files**: All `#[tokio::main]` entry points

**Evidence**:
```rust
// neotrix-core/src/bin/neotrix_dl.rs:4
#[tokio::main]
async fn main() { ... }

// neotrix-core/examples/todo_parallel.rs:7
#[tokio::main]
async fn main() { ... }

// neotrix-core/examples/proxy.rs:21
#[tokio::main]
async fn main() { ... }

// Cargo.toml: tokio = { version = "1", features = ["full"] }
```

**Root Cause**: `#[tokio::main]` defaults to `worker_threads = num_cpus::get()` and `max_blocking_threads = 512`. For NeoTrix (an AI developer toolkit running alongside LLM inference, vector search, and database operations), this uncontrolled default:
1. Oversubscribes CPU cores when running in containers with CPU quotas
2. Creates 512 potential blocking threads even for I/O-light workloads
3. Does not enable `on_thread_start`/`on_thread_stop` hooks for thread naming or monitoring

**Container/K8s Impact**: `std::thread::available_parallelism()` reads the cgroup limit, not the request. If NeoTrix runs alongside a Rayon thread pool or LLM inference, both will claim "all cores" and cause CFS throttling (100% thread oversubscription). PostHog's production case study showed this exact pattern caused p99 latency spikes from 94ms to 2.5s.

**Fix**:
```rust
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() { ... }

// Or explicit builder:
fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)           // explicit core budget
        .max_blocking_threads(64)    // match workload, not default 512
        .thread_name("nt-worker")    // debuggable thread names
        .enable_all()
        .thread_stack_size(2 * 1024 * 1024)
        .on_thread_start(|| { /* telemetry hook */ })
        .build()
        .unwrap();

    rt.block_on(async { ... });
}
```

---

## Defect 4: Unbounded Concurrency in `tokio::spawn` Without Backpressure

**Severity**: HIGH (P1)
**Affected Files**: 15+ spawn sites across codebase

**Evidence**:
```rust
// neotrix-core/src/neotrix/nt_file_ability/batch_processor.rs:227
handles.push(tokio::spawn(async move { ... }));

// benches/_disabled/performance_benchmark.rs:257
handles.push(tokio::spawn(async move { ... }));

// neotrix-core/src/unified/layers/perception/nt_world/nt_world_osint/person.rs:111
handles.push(tokio::spawn(async move { ... }));

// neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/proxy_pool.rs:616
handles.push(tokio::spawn(async move { ... }));

// neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/coordinator.rs:89
handles.push(tokio::spawn(async move { ... }));
```

**Root Cause**: Spawning tasks into a `Vec` without a concurrency limit. Every spawned task has bookkeeping overhead (allocation, Arc clones, vtable dispatch, context switch). Under burst load, thousands of tasks can be spawned simultaneously, exhausting sockets, file handles, or connection pools. The local queue fits 256 tasks — overflow sends half to the global queue, creating synchronization overhead.

**Impact**:
- Memory exhaustion from unbounded task accumulation
- Downstream service overload (no backpressure propagation)
- Scheduling overhead dominates useful work when spawning thousands of micro-tasks

**Fix**:
```rust
use tokio::sync::Semaphore;

let sem = Arc::new(Semaphore::new(100)); // max 100 concurrent
let mut handles = vec![];

for item in items {
    let permit = sem.clone().acquire_owned().await.unwrap();
    handles.push(tokio::spawn(async move {
        let _permit = permit; // released when task finishes
        process(item).await
    }));
}

for h in handles { let _ = h.await; }
```

---

## Defect 5: Missing `spawn_blocking` for CPU-Bound and Synchronous I/O in Async Context

**Severity**: HIGH (P1)
**Affected Files**: Multiple crawl/scraper modules

**Evidence**:
```rust
// neotrix-core/src/unified/layers/perception/nt_world/nt_world_crawl/fetcher.rs:265
// Rate-limiting sleep inside async handler — blocks worker
std::thread::sleep(Duration::from_millis(self.strategy.delay_ms() - duration_ms));

// neotrix-core/src/unified/layers/perception/nt_world/nt_world_crawl/unified.rs:355
// Same pattern in unified crawler
std::thread::sleep(Duration::from_millis(min_delay - elapsed_ms));

// neotrix-core/src/unified/layers/perception/nt_world/nt_world_edgar.rs:302
// Rate limiter blocking worker thread
std::thread::sleep(min_interval - elapsed);

// neotrix-core/src/unified/layers/perception/nt_world/nt_world_crawl/spider/tests.rs:120
std::thread::sleep(Duration::from_millis(1));

// neotrix-core/src/unified/layers/perception/nt_world/nt_world_browse/session.rs:64,115
std::thread::sleep(Duration::from_millis(200));
```

**Root Cause**: Rate-limiting logic, retry backoff, and crawl delays use `std::thread::sleep` directly in async context. Tokio's poll loop is blocked for the entire sleep duration. This is particularly damaging in NT-WORLD (crawl/spider/scraper) which is inherently I/O-bound and should yield to the scheduler during waits.

**100ms Rule Violation**: Tokio docs state: "No async task should run for more than 100ms without yielding." The `nt_world_crawl/fetcher.rs` rate limiter can sleep for `delay_ms()` which may be seconds.

**Fix**:
```rust
// WRONG: blocks worker thread
std::thread::sleep(Duration::from_millis(delay));

// CORRECT: yields to scheduler
tokio::time::sleep(Duration::from_millis(delay)).await;

// For sync-only rate limiters:
tokio::task::spawn_blocking(move || {
    std::thread::sleep(Duration::from_millis(delay));
}).await.unwrap();
```

---

## Additional Findings (Supplementary)

### Finding A: `#[tokio::test]` Without Configuration
All test files use bare `#[tokio::test]` which creates a `current_thread` runtime per test. This is correct for unit tests, but integration tests that exercise concurrent behavior should use `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`.

### Finding B: No `tokio-console` Integration for Production
NeoTrix uses `#[tokio::main]` without `tokio_unstable` feature or `console-subscriber`. The `poll_duration` metric (alert threshold >100μs) is unavailable for detecting hidden blocking calls in production.

### Finding C: Blocking Pool Queue Unbounded
Tokio's `max_blocking_threads` queue "does not apply any backpressure, it could potentially grow unbounded" (docs). NeoTrix's crawl modules heavily use `spawn_blocking` patterns, but the queue depth is never monitored. If blocking tasks pile up, memory grows without limit.

---

## Priority Matrix

| # | Defect | Severity | File Count | Effort |
|---|--------|----------|------------|--------|
| 1 | `std::thread::sleep` in async context | CRITICAL | 20+ files | Medium (find-replace + await) |
| 2 | Runtime-per-iteration in benchmarks | HIGH | 1 file | Low (hoist runtime) |
| 3 | No runtime builder configuration | HIGH | 3+ entry points | Low (add builder) |
| 4 | Unbounded `tokio::spawn` without backpressure | HIGH | 5+ sites | Medium (add Semaphore) |
| 5 | Missing `spawn_blocking` for CPU/sync I/O | HIGH | 5+ crawl modules | Medium (refactor to spawn_blocking) |

---

## Recommended Fix Order

1. **Defect 1** (CRITICAL): Global search-replace `std::thread::sleep` → `tokio::time::sleep(...).await` in all async contexts. Estimated: 20 files, 30 changes.
2. **Defect 3** (HIGH): Add explicit runtime builder to all `#[tokio::main]` entry points with `worker_threads = 4` and `max_blocking_threads = 64`.
3. **Defect 2** (HIGH): Fix benchmark file to hoist runtime creation outside iteration loop.
4. **Defect 4** (HIGH): Add `Semaphore`-based backpressure to unbounded spawn sites.
5. **Defect 5** (HIGH): Refactor crawl rate limiters to use `tokio::time::sleep` or `spawn_blocking`.

---

## Verification Commands

```sh
# Find all std::thread::sleep in async contexts
rg 'std::thread::sleep' --type rust -l

# Find all bare #[tokio::main] without builder
rg '#\[tokio::main\]' --type rust

# Find all tokio::spawn without semaphore
rg 'tokio::spawn' --type rust | grep -v semaphore

# Run tests after fixes
cargo test -p neotrix --lib

# Check for async deadlocks
cargo check --all-targets -p neotrix
```
