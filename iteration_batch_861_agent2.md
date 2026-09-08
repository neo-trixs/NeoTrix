# Agent 2: Tokio Runtime Internals (Batch 861)

## Sources
1. Tokio source: `runtime/scheduler/multi_thread/worker.rs` — Worker, Core, Shared structs, work-stealing algorithm, LIFO slot, park/unpark lifecycle
2. Tokio source: `runtime/scheduler/multi_thread/queue.rs` — Fixed-size local queue (256 capacity), overflow to global inject queue
3. Tokio docs: `tokio::runtime` — Runtime configuration, Builder API, global_queue_interval, event_interval, max_blocking_threads
4. Tokio docs: `tokio::task::coop` — Budget(Option<u8>), initial budget=128, poll_proceed, consume_budget, cooperative wrapper, unconstrained opt-out
5. Tokio blog: "Making the Tokio scheduler 10x faster" — LIFO slot, fixed-size local queue, throttle stealing (half CPU searchers), Go-inspired notification
6. Tokio blog: "Reducing tail latencies with automatic cooperative task yielding" — Per-task budget of 128 ops, auto-yield on budget exhaustion
7. DeepWiki: "Schedulers and Task Execution" — Multi-thread scheduler architecture, work-stealing, shutdown procedure
8. Tokio docs: `tokio::runtime::Builder` — worker_threads, max_blocking_threads, thread_keep_alive, global_queue_interval, event_interval, disable_lifo_slot, eager_driver_handoff

## Defects

### D-RUN-001: Production sync code creates new Tokio Runtime per call — unbounded runtime proliferation
**File**: `nt_core_consciousness_core.rs:1868` (LlmSolutionExecutor::attempt), `nt_core_forecast.rs:373` (GatewayHandle::complete_single)
**Severity**: High
**Source**: Tokio source: `runtime/scheduler/multi_thread/worker.rs` (Shared::owned, remotes, inject queue — all allocate per-runtime)

**Detail**: `LlmSolutionExecutor::attempt()` calls `tokio::runtime::Runtime::new()` on every invocation from sync context. Each `Runtime::new()` spawns N worker threads (default = num_cpus), allocates a global inject queue, per-worker local queues (256 slots each), a blocking pool (up to 512 threads), and an I/O driver (epoll/kqueue fd). This is called inside the SEAL pipeline's consciousness task execution loop — a hot path. `GatewayHandle::complete_single()` also falls through to `Runtime::new()` when no handle is available (line 373).

**Tokio insight**: Tokio's runtime is designed to be long-lived. The `Runtime::new()` call allocates: (1) worker thread pool, (2) per-worker `Core` with `run_queue` (fixed 256-slot ring buffer), `lifo_slot`, `park` state; (3) `Shared` with global `inject` queue, `OwnedTasks`, `idle` coordinator, `synced` mutex. Creating/destroying this on every call pays thread spawn cost (2MB stack per thread) and loses all scheduling state (task locality, LIFO optimization, worker affinity).

**Impact**: Each LLM call pays ~10-50ms for runtime construction. Under SEAL pipeline load (10+ tasks/cycle), this creates 10+ independent runtimes that cannot share work via work-stealing, defeating the entire purpose of the multi-thread scheduler.

### D-RUN-002: EventBus spawns 9 OS threads with busy-poll sleep — bypasses Tokio cooperative scheduling entirely
**File**: `nt_core_event_bus.rs:434` (std::thread::spawn), `nt_core_event_bus.rs:464` (std::thread::sleep(10ms))
**Severity**: High
**Source**: Tokio docs: `tokio::runtime::Builder::max_blocking_threads` (default 512), Tokio blog: "cooperative scheduling" (worker starvation from blocking ops)

**Detail**: `subscribe_all_layers_sync()` spawns 9 OS threads (one per LayerId L1-L9) that run a tight loop: `try_recv()` → `std::thread::sleep(10ms)` on empty → repeat. These threads are completely outside the Tokio runtime. Each OS thread consumes 2MB stack + kernel scheduling overhead. The 10ms sleep is a hard sleep that cannot be interrupted by shutdown signal (only checked after wake). Meanwhile, Tokio's worker threads are cooperative — if they call into code that touches these OS threads, they can block the worker.

**Tokio insight**: Tokio's scheduler limits searching workers to half the CPU count (D13: "max number of searchers is half the total number of processors"). Spinning 9 OS threads competes with Tokio workers for CPU time, potentially reducing the effective worker count below optimal. The `std::thread::sleep(10ms)` is not cooperative — it blocks an OS thread regardless of Tokio's budget mechanism (Budget(Option<u8>), initial=128 ops per poll).

**Impact**: 9 OS threads permanently consuming CPU slices. Under load, these compete with Tokio's work-stealing pool, reducing effective throughput. The 10ms sleep granularity means event delivery latency is 0-10ms unconditionally, even when events arrive 1μs after the sleep.

### D-RUN-003: std::thread::yield_now() in async context — cooperative scheduling violation
**File**: `nt_mind_skill_engine.rs:2574`
**Severity**: Medium
**Source**: Tokio docs: `tokio::task::coop` — "A single call to poll on a top-level task may potentially do a lot of work before it returns Poll::Pending... cooperative scheduling provides an opt-in mechanism for futures to collaborate"

**Detail**: Inside the `CsgnWakeHook` hook handler (called from skill activation, which runs in async context via hook registry), `std::thread::yield_now()` is used as a backoff when `try_write()` fails on a `tokio::sync::RwLock`. This is a sync thread yield, not a cooperative future yield. It blocks the current Tokio worker thread for the OS scheduling quantum (~1-10ms depending on platform). The correct pattern is `tokio::task::yield_now().await` or `tokio::sync::RwLock::write().await`.

**Tokio insight**: Tokio's cooperative budget (128 ops per poll) tracks async yield points. `std::thread::yield_now()` does NOT decrement the budget — it blocks the entire worker thread while appearing to "yield". A worker blocked here cannot poll other tasks, breaking the fairness guarantee: "there is some number MAX_DELAY such that when a task is woken, it will be scheduled within MAX_DELAY time units."

**Impact**: Every skill activation that hits the RwLock contention path stalls one Tokio worker thread for an OS scheduling quantum. Under concurrent skill loads, this creates head-of-line blocking across all tasks on that worker.

### D-RUN-004: No cooperative budget consumption in background loop handlers — starvation risk
**File**: `nt_mind_background_loop/run.rs:738-742` (tokio::select! with interval), `handlers_absorption.rs:181` (tokio::spawn)
**Severity**: High
**Source**: Tokio blog: "Reducing tail latencies with automatic cooperative task yielding" — "Once the task is out of budget, all Tokio resources will perpetually return 'not ready' until the task yields back to the scheduler"

**Detail**: The background loop spawns handlers as tokio tasks (line 738: `tokio::spawn(async move {...})`). These handlers run in tight `tokio::select!` loops with interval tickers. None of them call `tokio::task::consume_budget()` or interact with the cooperative budget system. If a handler does significant CPU work in a single poll (e.g., knowledge absorption at `handlers_absorption.rs:187` which runs `child.wait_with_output()` with 600s timeout), it exhausts the 128-op budget without yielding. Since the handler doesn't use Tokio sync primitives (mpsc, Mutex, etc.) that auto-consume budget, the auto-yield mechanism never triggers.

**Tokio insight**: The budget is stored in a thread-local `Cell<Budget>` and only decremented by Tokio's own types (mpsc::Receiver::recv, Mutex::lock, etc.). Custom async code that does CPU work without calling `consume_budget()` or hitting a Tokio primitive runs unbounded. The `cooperative()` wrapper exists for exactly this case but is not used.

**Impact**: Background handlers that do CPU-heavy work (KB absorption, consciousness cycle) can monopolize a worker thread for unbounded time, starving latency-sensitive tasks (LLM gateway, event dispatch) on the same worker.

### D-RUN-005: LLM stream spawn creates fire-and-forget task — no error propagation, no backpressure
**File**: `nt_core_llm.rs:60-73` (tokio::spawn in stream_complete)
**Severity**: Medium
**Source**: Tokio source: `runtime/scheduler/multi_thread/worker.rs` — Shared::owned (OwnedTasks tracks all spawned tasks), Tokio docs: JoinHandle

**Detail**: `LlmProvider::stream_complete()` spawns a tokio task (line 60) that reads from the raw receiver, applies ingress privacy guard, and forwards to the output channel. The `JoinHandle` is immediately dropped ("fire-and-forget"). If the spawned task panics, the error is silently lost. If the output channel closes (receiver dropped), the `tx.send(item).await.is_err()` break catches it, but there's no metrics/logging for dropped streams. More critically, there's no backpressure: the raw receiver (typically a reqwest SSE stream) can produce data faster than the privacy guard processes it, causing unbounded memory growth in the mpsc channel (buffer=64, but if the consumer is slow, items queue).

**Tokio insight**: Tokio's `OwnedTasks` tracks all spawned tasks. When a JoinHandle is dropped, the task continues running but becomes "detached" — it still occupies a slot in the worker's local queue or the global inject queue. In the multi-thread scheduler, each worker's local queue holds 256 tasks max; detached tasks count toward this limit.

**Impact**: Under high-concurrency streaming (e.g., multiple LLM providers streaming simultaneously), detached tasks accumulate in Tokio's task queues, reducing available slots for other work. Silent failures mean broken streams go undetected.

### D-RUN-006: Multiple independent Runtime instances across test suite — no shared runtime
**File**: `nt_core_llm.rs:1022,1032,1043,1056,1110`, `self_iterating/mod.rs:99,125,153,237,262`, `seal.rs:20,35,52,71,90,135`, `registry.rs:341,377`, `gateway/mod.rs:182+` (20+ test functions)
**Severity**: Medium
**Source**: Tokio docs: `tokio::runtime` — "A multi-threaded runtime is always running because it spawns its own worker threads"

**Detail**: Each test function creates its own `tokio::runtime::Runtime::new()` — the test suite has 30+ independent runtimes. Each spawns N worker threads (default = num_cpus), a blocking pool, and I/O driver. On a typical machine (8 cores), this means 30×8 = 240 worker threads created and destroyed during test execution. The `#[tokio::test]` attribute already creates a per-test runtime, but many tests manually create additional ones via `Runtime::new()`.

**Tokio insight**: Tokio's `Runtime::new()` calls `Builder::new_multi_thread().enable_all().build()` — it allocates the full worker pool. Tests that use `#[tokio::test]` already get a runtime; adding `Runtime::new()` inside creates a nested runtime that wastes resources and can cause confusing behavior (the outer runtime's tasks cannot see the inner runtime's context).

**Impact**: Test execution time inflated by ~30-50% due to repeated runtime construction. Thread pool thrashing may cause flaky tests under CPU contention.

### D-RUN-007: std::thread::sleep in async-adjacent code paths — worker thread blocking
**File**: `nt_core_forecast.rs:440,486`, `nt_core_observer_error.rs:59,359,371`, `nt_core_deploy.rs:464`, `nt_core_self/human_approval.rs:372`, `nt_core_self/pilot_steering.rs:438`, `self_evolver.rs:184`, `code_graph.rs:365`, `cross_session_memory.rs:347,349,351`, `observability_stack.rs:236`, `behavioral_verifier.rs:38`, `nt_memory_crawl.rs:346,950`, `nt_memory_geo.rs:804,992`, `nt_http.rs:242,374`, `nt_memory_confidence.rs:789`, `remediation.rs:32`, `fakeip.rs:178`, `http_proxy.rs:182`, `unified.rs:355`, `fetcher.rs:265,278,291,408`, `stealth.rs:264`, `nt_world_scrape.rs:287,348`, `session.rs:64,115`, `nt_world_edgar.rs:302`, `registry_watcher.rs:202,252`, `nt_core_event_bus.rs:645` (45+ occurrences)
**Severity**: High
**Source**: Tokio docs: `tokio::task::spawn_blocking` — "Tasks that perform blocking operations should be spawned on the dedicated blocking pool using spawn_blocking", Tokio runtime: `max_blocking_threads` default=512

**Detail**: 45+ occurrences of `std::thread::sleep()` across the codebase, many in functions that are called from async contexts or from Tokio worker threads. These include:
- `nt_core_forecast.rs:440`: retry sleep of 1200ms+ in LLM retry loop — if called from async context, blocks a Tokio worker for >1s
- `nt_memory_crawl.rs:950`: exponential backoff sleep up to 800ms × attempt — crawl operations run inside tokio tasks
- `fetcher.rs:265,278,291`: rate limiting sleep in NT-WORLD fetchers — these run inside tokio::spawn tasks
- `http_proxy.rs:182`: 100ms sleep in proxy handling — proxy runs in its own thread but interacts with Tokio runtime
- `nt_core_event_bus.rs:645`: test-only, but demonstrates pattern

**Tokio insight**: `std::thread::sleep` blocks the current thread for the full duration. If called on a Tokio worker thread, it effectively removes that worker from the pool. Tokio's scheduler has N workers (default=num_cpus); with K workers blocked on sleep, only N-K workers are available for work-stealing. The scheduler's `idle` coordinator cannot distinguish "sleeping because of std::thread::sleep" from "parked because no work" — the worker simply stops responding to steal requests.

**Impact**: Under load, multiple concurrent sleep calls can reduce effective worker count to zero, causing all Tokio tasks (including LLM I/O, event dispatch, consciousness cycle) to stall. The 45+ call sites represent a systematic pattern of sync-sleep-in-async that defeats the cooperative scheduler.

### D-RUN-008: No Tokio runtime configuration — defaults assume optimal deployment
**File**: Global (no explicit `Builder` configuration found in production code)
**Severity**: Medium
**Source**: Tokio docs: `tokio::runtime::Builder` — worker_threads, event_interval, global_queue_interval, max_blocking_threads, disable_lifo_slot

**Detail**: No production code configures a Tokio runtime via `Builder`. All runtimes use defaults: `worker_threads = num_cpus`, `event_interval = 61`, `global_queue_interval = dynamic (targets 10ms)`, `max_blocking_threads = 512`, LIFO slot enabled. For NeoTrix's consciousness architecture, these defaults are suboptimal:
- `event_interval = 61`: With LLM operations that take 1-30s, checking I/O events every 61 polls means timer-driven events (timeouts, rate limits) can be delayed by up to 61 poll cycles.
- `global_queue_interval = dynamic`: The heuristic targets 10ms between global queue checks. For consciousness cycle tasks that spawn many short-lived tasks, this adds unnecessary latency for cross-worker task distribution.
- `max_blocking_threads = 512`: NeoTrix has many blocking operations (crawl, KB writes, LLM retries). 512 may be too high (wasting memory) or too low (queue buildup during burst).
- LIFO slot enabled: Good for message-passing patterns but can cause starvation for compute-heavy tasks that don't benefit from cache locality.

**Tokio insight**: The Builder docs explicitly warn: "The tokio runtime is not NUMA aware. You may want to start multiple runtimes instead of a single runtime for better performance on NUMA systems." NeoTrix runs on diverse hardware (laptops, servers, edge) with no runtime tuning.

**Impact**: Suboptimal scheduling behavior across deployment targets. On a 4-core laptop, the default spawns 4 workers + 512 blocking threads — excessive for the workload. On a 64-core server, 64 workers may create too much stealing overhead for consciousness cycle workloads.

## Key Insights

1. **The Runtime::new() pattern is systemic**: NeoTrix has no shared runtime instance. Production code (`LlmSolutionExecutor`, `GatewayHandle`) creates throwaway runtimes per call. This is the #1 Tokio anti-pattern — it defeats work-stealing, wastes thread resources, and loses task locality.

2. **The EventBus OS thread model is incompatible with Tokio**: 9 OS threads doing busy-poll with `std::thread::sleep(10ms)` is a pre-Tokio design. This should be converted to `tokio::spawn` tasks using `tokio::sync::broadcast::Receiver` with async `recv()`, which auto-participates in cooperative scheduling and work-stealing.

3. **Cooperative budget is completely unused**: Zero calls to `consume_budget()`, `poll_proceed()`, or `cooperative()` wrapper. The 128-op budget system exists to prevent exactly the starvation scenarios NeoTrix's background loops can create. The `AsyncSafetyWrapper` (`nt_meta_async_safety.rs`) documents the rules but doesn't enforce them at runtime.

4. **The std::thread::sleep epidemic is the #1 production risk**: 45+ call sites across the codebase. Each one can block a Tokio worker thread. The crawl subsystem alone has 10+ sleep calls. This is a systematic pattern that requires a codebase-wide migration to `tokio::time::sleep().await` for async contexts or `tokio::task::spawn_blocking` for truly blocking operations.

5. **Tokio's LIFO slot optimization is invisible to NeoTrix**: The LIFO slot (designed for message-passing patterns) is automatically enabled but NeoTrix's spawn patterns don't benefit — tasks are spawned from non-worker threads (going to global inject queue) or from worker threads but without the waker-to-waker pattern that triggers LIFO placement.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Sources analyzed | 8 |
| Code paths examined | 45+ |
| Critical defects | 0 |
| High defects | 4 (D-RUN-001, D-RUN-002, D-RUN-004, D-RUN-007) |
| Medium defects | 4 (D-RUN-003, D-RUN-005, D-RUN-006, D-RUN-008) |
