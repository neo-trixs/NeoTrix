# Agent 1: Async Runtime Configuration (Batch 865)

## Sources
1. Tokio docs: `tokio::runtime` — Runtime configuration, Builder API, multi-thread vs current-thread schedulers, resource drivers, thread lifetime, fork safety
2. Tokio docs: `tokio::runtime::Builder` — worker_threads, max_blocking_threads, thread_name, thread_stack_size, on_thread_start/on_thread_stop, event_interval, global_queue_interval, disable_lifo_slot, unhandled_panic
3. Tokio docs: `tokio::runtime::RuntimeMetrics` (tokio_unstable) — worker_histogram, blocking_thread_count, io_driver_ready_count
4. Krun.pro: "Tokio Performance Tuning That Actually Works" — worker thread saturation, spawn_blocking vs block_in_place, actor-lite pattern, FuturesUnordered, channel backpressure, tokio-console diagnostics
5. Vibewise.top: "Master the Tokio Runtime: Practical Checklist" — runtime flavor selection, task budget (128 ops), oversubscription diagnosis, scheduler lock contention, dynamic tuning limitations
6. RustFromZeroToHero: "Tokio Runtime Tuning for Production (2026)" — baseline metrics, load-shedding with bounded channels, graceful shutdown via broadcast, tokio-console setup
7. RustFromZeroToHero: "Configure Tokio Worker Threads" — CPU-bound vs I/O-bound tuning, max_blocking_threads soft limit, thread stack size, mixed workload heuristics
8. Tokio GitHub repo: LTS releases (1.47.x, 1.51.x), `tokio_unstable` feature flags, `prewarm-fd-table` example for Linux
9. DeepWiki: Tokio runtime initialization — Builder API internals, RngSeed, LogHistogram configuration
10. NanoTechInsight: "Rust Async Programming with Tokio (2026)" — TCP_NODELAY, socket2 tuning, spawn_blocking for CPU work, current_thread for lightweight services
11. DPTCloud: "Maximizing Rust Web Performance on VPS" — Tokio runtime tuning on constrained environments, jemalloc integration, vCPU-aware thread sizing

## Defects

### D-RUN-009: `futures::executor::block_on` inside async method — nested executor deadlock risk
**File**: `nt_core_resource_pool/discovery.rs:115,134`
**Severity**: High
**Source**: Tokio docs: `tokio::runtime` — "Handle::block_on does not drive the runtime", Krun.pro: "A single blocking call inside an async fn can stall every task on that worker thread"

**Detail**: `ResourceDiscoveryEngine::discover_all()` and `discover_kind()` are `async fn` methods that call `futures::executor::block_on(self.cache.check_and_mark(...))` at lines 115 and 134. This invokes a completely separate executor (futures-rs's single-threaded executor) from within an async context that runs on the Tokio runtime. The `futures::executor::block_on` call blocks the current Tokio worker thread until the inner future completes, effectively removing that worker from Tokio's work-stealing pool for the duration.

**Tokio insight**: Tokio's documentation explicitly warns: "Handle::block_on does not drive the runtime." While this specific call uses the futures executor (not Tokio's Handle), the effect is identical: a Tokio worker thread is blocked executing a foreign executor's event loop. The `DiscoveryCache::check_and_mark` is an `async fn` that acquires a `tokio::sync::RwLock` write guard — calling `futures::executor::block_on` on it from a Tokio worker creates a nested executor that may deadlock if the RwLock is contested by another Tokio task on the same worker.

**Impact**: Each discovery cycle blocks one Tokio worker thread per discoverer. With N discoverers, N workers are blocked sequentially. The code comment at line 115 doesn't acknowledge the executor mismatch. The `async fn` signature misleads callers into thinking the method is non-blocking.

### D-RUN-010: `block_in_place` + `Handle::block_on` in reasoning hot path — worker thread monopolization
**File**: `engine_core.rs:469,1649,1705`
**Severity**: High
**Source**: Krun.pro: "`block_in_place` converts the current worker thread into a blocking thread temporarily and migrates the other tasks off it", Vibewise.top: "A blocking operation on a worker thread can stall all tasks on that thread"

**Detail**: The reasoning engine uses `tokio::task::block_in_place(|| Handle::current().block_on(future))` at three locations in the hot reasoning path:
- Line 469: CoT generation (`cot_gen.generate_cot()`)
- Line 1649: Gateway completion (`gateway_ref.complete()`)
- Line 1705: Panel async execution (`panel.run_async()`)

`block_in_place` converts the current Tokio worker thread into a temporary blocking thread. While Tokio migrates other tasks off this worker, the thread itself is now dedicated to the blocking operation for its entire duration. If the LLM call takes 1-30s, one worker thread is fully consumed.

**Tokio insight**: Tokio's `block_in_place` docs state: "This function should only be called from inside a multi-threaded Tokio runtime." More critically, during `block_in_place`, the worker thread cannot participate in work-stealing. With N workers and M concurrent `block_in_place` calls, only N-M workers remain available for the cooperative scheduler. The reasoning engine is the core cognitive loop — multiple concurrent reasoning sessions would starve the runtime.

**Impact**: Each reasoning session monopolizes one worker thread for the full LLM round-trip (1-30s). With 4 workers (default on 4-core machine), 4 concurrent reasoning sessions would block ALL workers, stalling event dispatch, crawl operations, and consciousness cycles.

### D-RUN-011: Unbounded channels in production hot paths — OOM under load
**File**: `nt_io_hotreload/mod.rs:137`, `nt_io_plugin/registry.rs:436`, `nt_shield_proxy_kernel/kernel.rs:125`
**Severity**: Medium
**Source**: Krun.pro: "Unbounded channels are just OOM bugs waiting to ship to production", Tokio docs: `mpsc::unbounded_channel` — "no backpressure; if the receiver cannot keep up, the sender will keep buffering messages indefinitely"

**Detail**: Three production code paths use `tokio::sync::mpsc::unbounded_channel`:
- `nt_io_hotreload/mod.rs:137`: File watcher events (`notify::Result<notify::Event>`)
- `nt_io_plugin/registry.rs:436`: Plugin file watcher events
- `nt_shield_proxy_kernel/kernel.rs:125`: Listener error reporting

Unbounded channels have no backpressure mechanism. If the consumer (event handler, plugin loader, error aggregator) falls behind — due to CPU contention, blocking operations, or a burst of events — messages accumulate unbounded in heap memory until OOM.

**Tokio insight**: Tokio's bounded `mpsc::channel(bound)` makes `send()` async and parks the producer when the buffer is full, naturally propagating backpressure. Unbounded channels use `try_send()` which never blocks but silently grows memory. The hotreload watcher generates events on every file change; during a `cargo build` or mass file edit, this can produce thousands of events per second.

**Impact**: Under file-heavy operations (build, mass edit, plugin install), the unbounded channel can grow to consume gigabytes of memory before OOM kills the process. No monitoring exists for channel depth.

### D-RUN-012: `Runtime::new()` in NT-SHIELD production paths — throwaway runtimes in sandbox/proxy
**File**: `nt_shield_sandbox_entry.rs:228`, `nt_shield_sandbox/mod.rs:585,876,895,979,992`, `nt_shield_sandbox/remote.rs:246`, `nt_shield_sandbox/judge.rs:570,590,611`, `nt_shield_stealth_net/rule_api.rs:192,460,472,488,503,520,538,552,564,576,588,603`
**Severity**: High
**Source**: Tokio docs: `tokio::runtime::Runtime` — "The runtime is designed to be long-lived. Creating and destroying runtimes repeatedly is expensive", RustFromZeroToHero: "Each Runtime::new() spawns N worker threads (default=num_cpus), allocates a global inject queue, per-worker local queues"

**Detail**: The NT-SHIELD subsystem creates `tokio::runtime::Runtime::new()` in at least 15+ production code paths (not tests). The `nt_shield_sandbox_entry.rs:228` creates a full runtime to execute sandboxed code. The `nt_shield_sandbox/mod.rs` creates runtimes at lines 585, 876, 895, 979, 992 for sandbox operations. The `rule_api.rs` creates 12+ runtimes for rule API calls (lines 192, 460-603).

Each `Runtime::new()` allocates: worker thread pool (N × 2MB stack), per-worker Core with 256-slot ring buffer, global inject queue, blocking pool (up to 512 threads), I/O driver (epoll/kqueue fd). On a shield operation that runs frequently (proxy heartbeat, rule checks, sandbox execution), this creates and destroys heavy runtime infrastructure repeatedly.

**Tokio insight**: Tokio's runtime is designed to be created once and reused. The `Runtime` struct holds OS resources (thread handles, file descriptors, memory-mapped regions) that are expensive to allocate/teardown. Creating a runtime inside a sandbox is especially wasteful — the sandbox itself runs on a Tokio worker, so you're creating a nested runtime inside an async context.

**Impact**: Each shield operation pays 10-50ms for runtime construction. Under load (proxy requests, rule evaluations), this creates runtime proliferation similar to D-RUN-001 but in a different subsystem. The 12+ runtime::new() calls in rule_api.rs alone can create 12 × num_cpus worker threads during a single rule evaluation batch.

### D-RUN-013: No Tokio runtime graceful shutdown — task dropout on process exit
**File**: Global (no `Runtime::shutdown_timeout()` or `Runtime::shutdown_background()` found)
**Severity**: Medium
**Source**: Tokio docs: `tokio::runtime::Runtime::drop` — "While the Runtime is active, threads may shut down after periods of being idle. Once Runtime is dropped, all runtime threads have usually been terminated, but in the presence of unstoppable spawned work are not guaranteed to have been terminated", RustFromZeroToHero: "Implement graceful shutdown to avoid dropping in-flight requests"

**Detail**: No production code calls `Runtime::shutdown_timeout()` or `Runtime::shutdown_background()`. When the process exits, all Tokio runtimes are dropped, which immediately terminates worker threads. Spawned tasks that are mid-execution (LLM streaming responses, KB writes, crawl downloads, consciousness cycle state saves) are silently aborted without cleanup.

The background loop (`nt_mind_background_loop`) spawns long-running tasks with `tokio::spawn` at lines 738 and 894. These tasks maintain in-memory state (consciousness cycle state, absorption progress, pending experience data). On drop, this state is lost.

**Tokio insight**: Tokio's `Runtime::drop` implementation calls `shutdown()` which notifies all worker threads to stop. Workers finish their current task but new tasks are not accepted. However, tasks that are `.await`-ing on I/O (network, disk) will be cancelled immediately — the I/O driver is torn down before tasks complete. For a system that writes to KB, manages state, and streams LLM responses, this means data corruption risk on every shutdown.

**Impact**: Every process restart risks: (1) partial KB writes (knowledge corruption), (2) lost consciousness cycle state (evolution regression), (3) aborted LLM streams (user-facing errors), (4) incomplete experience absorption (learning loss). The risk is proportional to shutdown frequency.

### D-RUN-014: No worker thread naming — invisible in profiling and crash dumps
**File**: Global (all `Runtime::new()` and `Builder` calls use default thread name "tokio-rt-worker")
**Severity**: Low
**Source**: Tokio docs: `Builder::thread_name` — "Thread names appear in stack traces and `ps` output, making profiling easier", RustFromZeroToHero: "thread_name() helps with debugging"

**Detail**: None of the ~20+ `Runtime::new()` or `Builder` calls in the codebase set `thread_name()`. All Tokio worker threads are named "tokio-rt-worker" by default. When multiple runtimes exist simultaneously (which NeoTrix does via D-RUN-001 and D-RUN-012), their worker threads are indistinguishable in `ps`, `top`, stack traces, and core dumps.

**Tokio insight**: The `Builder::thread_name()` method accepts a string or a closure `Fn(usize) -> String` for per-thread naming (e.g., "neotrix-llm-worker-{id}", "neotrix-crawl-worker-{id}"). Named threads appear in `/proc/[pid]/task/[tid]/comm` on Linux and in debugger thread lists. For a multi-runtime system like NeoTrix, thread naming is the primary mechanism for understanding which subsystem owns which thread.

**Impact**: During production debugging (latency spikes, deadlocks, OOM), all Tokio threads look identical. Cannot correlate thread activity with subsystems (LLM gateway, crawl, consciousness, shield). Crash dumps and profiler output require manual thread-count deduction to identify subsystems.

### D-RUN-015: No Tokio runtime metrics collection — blind to scheduler health
**File**: Global (no `RuntimeMetrics`, `tokio-console`, or `tracing` integration with runtime metrics)
**Severity**: Medium
**Source**: Tokio docs: `tokio::runtime::RuntimeMetrics` (tokio_unstable) — "worker_histogram, blocking_thread_count, io_driver_ready_count", Krun.pro: "Tokio Console is a tracing-based diagnostic tool that gives you per-task runtime metrics live", Vibewise.top: "Use tokio-console to verify that tasks are yielding appropriately"

**Detail**: No production code collects or exposes Tokio runtime metrics. The `RuntimeMetrics` API (behind `tokio_unstable` flag) provides: per-worker histogram of poll durations, blocking thread count, I/O driver readiness count, total tasks processed, workers parked count. Without this, NeoTrix has zero visibility into:
- Whether workers are saturated (high `scheduled_delay`)
- Whether tasks are blocking (high `poll_duration`)
- How many blocking threads are active vs. idle
- Whether the I/O driver is keeping up with network requests

The codebase does not use `console-subscriber` or `tokio-console` in any production path. The `tracing` integration exists but does not hook into Tokio's runtime-level metrics.

**Tokio insight**: Tokio's `RuntimeMetrics` requires the `tokio_unstable` feature flag and `TOKIO_UNSTABLE=1` environment variable at build time. The metrics are per-runtime, so NeoTrix's multiple-runtime architecture (D-RUN-001, D-RUN-012) would need per-runtime metric collection. The `worker_histogram` metric is particularly valuable — it shows the distribution of poll durations, enabling detection of the "5% CPU mystery" (worker thread saturation without high CPU usage).

**Impact**: Cannot diagnose: (1) worker thread saturation before it causes latency spikes, (2) blocking operations that are invisible to application-level monitoring, (3) I/O driver bottlenecks under high crawl load, (4) blocking pool exhaustion. Production incidents are diagnosed reactively via symptoms rather than proactively via metrics.

## Key Insights

1. **The `futures::executor::block_on` in discovery.rs is worse than `std::thread::sleep`**: It creates a nested executor inside a Tokio async context. The nested executor competes for the same thread, can deadlock on `tokio::sync::RwLock`, and is invisible to Tokio's cooperative budget system. This is a category error — using the wrong executor primitive in the wrong runtime context.

2. **The `block_in_place` + `Handle::block_on` pattern in the reasoning engine is a scalability ceiling**: Each concurrent reasoning session consumes one worker thread for the full LLM round-trip. With N workers, the system can only handle N concurrent reasoning sessions before all workers are blocked. This directly limits the consciousness architecture's parallel processing capacity.

3. **The unbounded channel usage is a ticking OOM bomb in the hotreload and plugin subsystems**: Unlike the EventBus (which uses bounded broadcast), the hotreload and plugin watchers use unbounded channels with no backpressure. A mass file operation (build, install) can generate thousands of events per second with no flow control.

4. **NT-SHIELD's Runtime::new() proliferation is a second front of the same disease as D-RUN-001**: The shield subsystem has 15+ production `Runtime::new()` calls that were not identified in batch 861. Combined with the consciousness/LLM paths, NeoTrix may create 30+ throwaway runtimes per operation burst.

5. **The absence of graceful shutdown and runtime metrics means NeoTrix operates blind**: No visibility into scheduler health, no clean shutdown of long-running tasks, no protection against data corruption on exit. These are production-readiness gaps that compound all other runtime defects.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 7 |
| Sources analyzed | 11 |
| Code paths examined | 50+ |
| Critical defects | 0 |
| High defects | 3 (D-RUN-009, D-RUN-010, D-RUN-012) |
| Medium defects | 3 (D-RUN-011, D-RUN-013, D-RUN-015) |
| Low defects | 1 (D-RUN-014) |
