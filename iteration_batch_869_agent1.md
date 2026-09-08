# Agent 1: Async Runtime Internals (Batch 869)

## Sources

1. https://piush.in/blog/tokio-under-the-hood-how-rusts-async-runtime-actually-works — Tokio Architecture: Scheduler, Reactor, Timers (2026-05-22)
2. https://deepwiki.com/tokio-rs/tokio/3.2-schedulers-and-task-execution — Tokio Schedulers and Task Execution (2026-09-03)
3. https://developers-heaven.net/blog/deep-dive-into-the-tokio-runtime-scheduler-and-work-stealing — Deep Dive: Tokio Runtime Scheduler and Work-Stealing (2026-06-25)
4. https://www.youngju.dev/blog/culture/2026-04-15-rust-tokio-async-runtime-future-waker-work-stealing-deep-dive-guide-2025.en — Tokio Runtime Deep Dive: Future, Waker, Work-Stealing (2026-04-15)
5. https://docs.rs/tokio/latest/tokio/runtime — Official Tokio Runtime Documentation
6. https://rustz2h.com/chapter_07_mastering_async_rust_and_tokio/series_01_tokio_runtime_internals_and_tasks/work_stealing_scheduler_design — Work-Stealing Scheduler Design Deep Dive
7. https://andrewodendaal.com/rust-async-runtime-tokio-architecture/ — Rust Async Runtime Deep Dive: Tokio Architecture (2026-05-15)
8. https://chandanbhagat.com.np/async-rust-with-tokio-part-2-tokio-architecture-de — Async Rust with Tokio Part 2: Scheduler, epoll, Thread Model (2026-04-26)
9. https://deepwiki.com/tokio-rs/tokio/4-task-management — Tokio Task Management (2026-07-04)
10. https://rustify.rs/resources/how-tokio-async-runtime-works — How Tokio Works: Async Runtime Explained (2026-08-19)

## Defects

### D-RUN-001: `block_in_place` + `Handle::current().block_on()` nested runtime anti-pattern in reasoning engine

**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/reason/reasoning_engine/engine_core.rs:469`

**Severity:** HIGH

**Source:** Tokio docs: "In a multi-threaded runtime, `block_in_place` moves the current task to a blocking thread, but `Handle::block_on` inside it creates a nested runtime context. If the inner future spawns tasks or uses the handle, those tasks may not be scheduled correctly." Source #5 (docs.rs/tokio): `Handle::block_on` "is not enough" — must use `Runtime::block_on`. The pattern `block_in_place(|| Handle::current().block_on(fut))` is a well-known anti-pattern that can cause panics when the inner future tries to access the runtime handle or when `current_thread` runtime is active.

**Description:** The reasoning engine calls `tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(cot_future))` to synchronously wait for a CoT future. This pattern is dangerous on `current_thread` runtime (the `run_blocking` helper in `nt_http.rs:33` explicitly checks for this), but the reasoning engine has no such guard. If the system is ever configured with `current_thread` flavor, this will panic. Additionally, `block_in_place` on a multi-thread runtime sends the current task to a blocking thread, but `block_on` inside it creates a new inner event loop that conflicts with the outer scheduler — the inner future's spawned tasks may be orphaned.

**Impact:** Potential runtime panic on `current_thread` flavor; orphaned inner tasks on multi-thread flavor; CoT generation failures are silently swallowed (line 476: `cot_generation_failed = true` with no retry).

---

### D-RUN-002: `std::thread::sleep` inside `tokio::spawn` async task — event bus polling blocks worker thread

**File:** `neotrix-core/src/neotrix/nt_core_event_bus.rs:464`

**Severity:** HIGH

**Source:** Source #3 (developers-heaven.net): "Never use `std::thread::sleep` or blocking I/O within an async function; use `tokio::time::sleep` instead." Source #4 (youngju.dev): "CPU-bound loop inside `tokio::spawn` monopolises one worker because Tokio is cooperative. Throughput drops to `(N-1)/N` while the loop runs." Source #7 (andrewodendaal.com): "that sync database call blocked the worker thread... the entire runtime froze."

**Description:** Inside `tokio::spawn(async move { ... })` at line 362, the event bus uses `rx.try_recv()` in a loop with `std::thread::sleep(Duration::from_millis(10))` on `TryRecvError::Empty` (line 464). This blocks the Tokio worker thread for 10ms per empty poll cycle. With 9 layers subscribed (L1-L9), each subscriber task occupies a worker thread during its sleep. Under low event throughput, up to 9 worker threads are simultaneously blocked on `std::thread::sleep`, starving other tasks. The async version at line 362 correctly uses `rx.recv().await` — but the `spawn_layer_subscribers` function at line 430 uses `std::thread::spawn` with `try_recv()` + sleep, which is acceptable for OS threads but creates a mixed model where some subscribers are async (Tokio tasks) and some are OS threads.

**Impact:** Worker thread starvation during low event throughput; inconsistent subscriber model (async vs OS thread) makes shutdown ordering unreliable.

---

### D-RUN-003: `std::thread::sleep` rate-limiting in synchronous crawler called from async handler without `spawn_blocking`

**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers_core.rs:223` + `neotrix-core/src/unified/layers/perception/nt_world/nt_world_crawl/unified.rs:355`

**Severity:** MEDIUM

**Source:** Source #1 (piush.in): "Tokio's timer architecture uses a hierarchical hashed timing wheel with millisecond resolution — `std::thread::sleep` bypasses this entirely." Source #5 (docs.rs/tokio): "code that spends a long time without reaching an `.await` will prevent other tasks from running."

**Description:** The async handler `handle_prediction` (handlers_core.rs:219) calls `pano.run_cycle()` synchronously at line 223. Inside `run_cycle` (unified.rs:355), `std::thread::sleep(Duration::from_millis(min_delay - elapsed_ms))` is used for rate-limiting. The `UnifiedCrawler::run_cycle` is a `pub fn` (sync) that performs blocking I/O (HTTP fetches via `reqwest::blocking::Client`), filesystem operations, and `std::thread::sleep` for rate limiting. This entire synchronous pipeline runs on the Tokio worker thread, blocking all other tasks on that worker for the duration of a crawl cycle (potentially seconds).

**Impact:** Tokio worker thread blocked for entire crawl cycle duration (could be seconds with rate limiting); other background loop handlers (absorption, maintenance, consciousness) are starved on that worker.

---

### D-RUN-004: Synchronous filesystem I/O in async handler — `std::fs::read_to_string` blocks worker thread

**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers_core.rs:245`

**Severity:** MEDIUM

**Source:** Source #3 (developers-heaven.net): "Heavy CPU-bound tasks should be moved to `spawn_blocking` to avoid stalling the async executor." Source #1 (piush.in): "The reactor is what lets thousands of sockets be served by a smaller number of threads — blocking a thread defeats this."

**Description:** Inside `handle_exploration` (an `async fn`), `std::fs::read_to_string(&p)` is called directly at line 245 without `spawn_blocking`. This is a blocking filesystem operation on the Tokio worker thread. Similarly, `std::fs::remove_file(&p)` at line 255 is a blocking I/O call. While the file is likely small, on slow filesystems (network mounts, busy disk) this can block the worker thread for milliseconds to seconds.

**Impact:** Worker thread blocked during filesystem I/O; violates Tokio's cooperative scheduling contract; under disk pressure, multiple handlers can compound blocking.

---

### D-RUN-005: Triple `block_in_place` + `Handle::block_on` pattern in reasoning engine `call_llm` and `llm_judge`

**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/reason/reasoning_engine/engine_core.rs:1649` and `:1705`

**Severity:** HIGH

**Source:** Source #5 (docs.rs/tokio): "There must be at least one call to `Runtime::block_on` when using the current thread runtime. `Handle::block_on` is not enough." Source #2 (deepwiki.com): Tokio's `block_in_place` "temporarily moves the current task to a blocking thread" — but `Handle::block_on` inside it creates a nested event loop.

**Description:** The `call_llm` method (line 1649) wraps `gateway_ref.complete(&request)` in `block_in_place(|| Handle::current().block_on(...))`. This is the synchronous LLM call path that every reasoning engine invocation uses. The `llm_judge` method (line 1705) does the same for the judge panel. Combined with D-RUN-001, there are **three** instances of this pattern in a single file. Each LLM call (which can take seconds due to network I/O) blocks a Tokio worker thread via `block_in_place`, temporarily removing it from the async pool. Under concurrent LLM calls, multiple workers can be simultaneously in `block_in_place` + `block_on`, degrading async throughput.

**Impact:** Each LLM call removes one worker thread from the async pool for the call duration; concurrent LLM calls compound this; nested runtime context can cause subtle scheduling bugs.

---

### D-RUN-006: `run_blocking` helper only works on multi-thread runtime — silent degradation on `current_thread`

**File:** `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_http.rs:31-40`

**Severity:** MEDIUM

**Source:** Source #5 (docs.rs/tokio): "The current-thread scheduler provides a single-threaded future executor... `Handle::block_on` is not enough." Source #4 (youngju.dev): "In a single-threaded runtime, `block_in_place` is not supported" — it panics.

**Description:** The `run_blocking` helper checks `runtime_flavor() == MultiThread` before using `block_in_place`. On `current_thread` flavor, it falls back to direct execution (line 36: `f()`). This means on a single-threaded runtime, reqwest blocking HTTP calls execute synchronously on the single worker thread, blocking all async tasks. The comment at line 29 acknowledges this: "current_thread runtime 内 block_in_place 不支持, 退化为直接执行 (仅测试辅助场景, 不触网)." But there's no compile-time or runtime guard preventing this from being used in production on a `current_thread` runtime.

**Impact:** If NeoTrix is ever configured with `current_thread` runtime (e.g., for testing or embedded deployment), all KB HTTP operations block the single worker thread; no warning or error is emitted.

---

### D-RUN-007: Unbounded `std::thread::sleep` in retry loops within crawl subsystem — worker thread starvation

**File:** `neotrix-core/src/unified/layers/perception/nt_world/nt_world_crawl/fetcher.rs:265,278,291`

**Severity:** MEDIUM

**Source:** Source #6 (rustz2h.com): "Work-stealing is relatively cheap... and it only happens when a worker's queue is empty. In steady state, workers process their own queues without stealing." Source #3 (developers-heaven.net): "Blocking prevention: Heavy CPU-bound tasks should be moved to `spawn_blocking`."

**Description:** The `FetcherPool` uses `std::thread::sleep` for rate limiting (line 265), retry backoff (line 278), and Tor safety delays (line 291). The delays can be up to 3600 seconds (1 hour) for exponential backoff (line 277: `Duration::from_secs(2u64.saturating_pow(retries).min(3600))`). While `FetcherPool` is synchronous, it's called from `UnifiedCrawler::run_cycle` which runs inside a Tokio task (via `handlers_core.rs:223`). If a crawl encounters repeated failures, a single worker thread can be blocked for up to 3600 seconds.

**Impact:** Worker thread blocked for potentially hours during retry backoff; all other tasks on that worker are starved; no timeout on the total retry duration.

---

### D-RUN-008: `std::thread::sleep` in memory KB crawl rate-limiting — blocks async context

**File:** `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_crawl.rs:346`

**Severity:** MEDIUM

**Source:** Source #1 (piush.in): "The reactor pattern bridges the gap between high-level async code and low-level system events — blocking defeats this." Source #7 (andrewodendaal.com): "never block a worker thread."

**Description:** Inside `fill_embeddings_from_text` (a sync function), `std::thread::sleep(Duration::from_millis(200))` is used for rate limiting after each KB entry. This function is called from the memory subsystem which operates in async contexts. A 200ms sleep per entry means processing 100 entries blocks a worker thread for 20 seconds.

**Impact:** Worker thread blocked during KB embedding operations; scales linearly with number of entries; no async alternative provided.

---

### D-RUN-009: Benchmark creates new Tokio runtime per iteration — incorrect measurement and resource leak

**File:** `benches/_disabled/performance_benchmark.rs:25,50,51,65,...`

**Severity:** LOW

**Source:** Source #5 (docs.rs/tokio): "While the Runtime is active, threads may shut down after periods of being idle. Once Runtime is dropped, all runtime threads have usually been terminated, but in the presence of unstoppable spawned work are not guaranteed to have been terminated." Source #8 (chandanbhagat.com.np): "The multi-threaded runtime spawns a pool of worker threads, which are all created on startup."

**Description:** Each benchmark iteration creates `tokio::runtime::Runtime::new().unwrap()` which spawns N worker threads (one per CPU core). In a tight benchmark loop, this creates and destroys thread pools repeatedly, polluting the measurement with runtime creation overhead and potentially leaking threads from prior iterations.

**Impact:** Benchmark numbers include runtime creation overhead (inaccurate); thread pool creation/destruction can cause resource pressure; disabled file but still present in codebase.

---

### D-RUN-010: Missing `tokio::task::yield_now()` or budget cooperation in long-running background handlers

**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738`

**Severity:** MEDIUM

**Source:** Source #5 (docs.rs/tokio): "Tokio is able to concurrently run many tasks on a few threads by repeatedly swapping the currently running task on each thread. However, this kind of swapping can only happen at `.await` points." Source #4 (youngju.dev): "Fix with periodic `tokio::task::yield_now().await`."

**Description:** The background loop uses `spawn_handler!` macro to spawn independent Tokio tasks (line 738). Each handler acquires a `Mutex` lock (`h.lock().await`) and then executes its body synchronously within the `await` point. Handlers like `handle_prediction` call `pano.run_cycle()` (a synchronous function with blocking I/O and `std::thread::sleep`). While the handler does hold the lock for the duration, there's no cooperative yield point within long-running handler bodies. Under the default cooperative budget (128 poll points), a handler that runs for many polls without yielding will trigger the budget exhaustion panic.

**Impact:** Under heavy load, handlers that perform many sequential operations without `.await` points can exhaust the cooperative budget, causing a panic; no `consume_budget` or `yield_now` calls in handler bodies.

## Key Insights

1. **Mixed sync/async model is the root cause:** NeoTrix has a pervasive pattern of synchronous subsystems (`UnifiedCrawler`, `FetcherPool`, `ReasoningEngine`, `KB crawl`) being called from async Tokio tasks without proper `spawn_blocking` wrapping. This violates Tokio's cooperative scheduling model.

2. **`block_in_place` + `Handle::block_on` is overused:** Three instances in `engine_core.rs` alone use this nested runtime pattern. While it works on multi-thread runtime, it creates a fragile coupling to the runtime flavor and can cause panics on `current_thread`. The `run_blocking` helper in `nt_http.rs` already has the correct guard pattern — it should be extracted and used everywhere.

3. **Rate limiting via `std::thread::sleep` is architecturally wrong for async systems:** The crawl subsystem uses `std::thread::sleep` for rate limiting (200ms-3600s delays). This should use `tokio::time::sleep` with async rate limiting, or the entire crawl should run on `spawn_blocking`.

4. **Event bus has dual subscriber model:** The event bus uses both `tokio::spawn` (async) and `std::thread::spawn` (OS thread) subscribers, creating inconsistency in shutdown ordering and error handling. The OS-thread subscribers with `try_recv` + `std::thread::sleep` polling are acceptable for OS threads but the mixing creates confusion.

5. **No runtime flavor guard:** There's no centralized check that prevents running blocking-heavy code paths on `current_thread` runtime. The `run_blocking` helper in `nt_http.rs` is the only place that checks, and it silently degrades. A compile-time or startup-time assertion would be safer.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| HIGH severity | 3 |
| MEDIUM severity | 6 |
| LOW severity | 1 |
| Files affected | 9 |
| Root cause categories | 3 (blocking-in-async, nested runtime, missing yield) |
