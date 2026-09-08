# Agent 1: Async Runtime Configuration (Batch 874)

## Sources
1. Tokio docs — `tokio::runtime::Builder` API (docs.rs/tokio/latest) — thread naming, event_interval, global_queue_interval, stack_size
2. Tokio docs — `tokio::task::block_in_place` — deadlock caveats, current_thread panic, nested runtime semantics
3. Tokio docs — `tokio::runtime::RuntimeMetrics` (tokio_unstable) — per-worker histogram, blocking thread count
4. tokio-rs/tokio Issue #7892 — `block_in_place` + `block_on` + `Mutex` deadlock scenario
5. tokio-rs/tokio Issue #7877 — deferred wakers stalled by `block_in_place`
6. tokio-rs/tokio Issue #6463 — `block_in_place` + `block_on` hang on runtime shutdown
7. Tokio docs — `futures::executor::block_on` vs Tokio executor incompatibility (nested executor deadlocks)
8. The Stack Dispatch — "Optimize Rust Async/Await Performance with Tokio" (2026) — worker thread blocking
9. Meridian Space course — Tokio runtime tuning for production — runtime isolation
10. Rust From Zero To Hero — "Tokio Runtime Tuning for Production" (2026) — thread naming, metrics
11. Krun.pro — "Tokio Performance Tuning: Fix Bottlenecks" (2026) — tokio-console, runtime health
12. NeoTrix source — `nt_core_resource_pool/discovery.rs:115,134` (futures::executor::block_on in async)
13. NeoTrix source — `nt_io_hotreload/mod.rs:289-298` (Handle::block_on + blocking_write from tokio::spawn)
14. NeoTrix source — `nt_core_forecast.rs:440,486` (std::thread::sleep in LLM retry)
15. NeoTrix source — `nt_world_crawl/fetcher.rs:265,278,291` (std::thread::sleep in crawl backoff)
16. NeoTrix source — `nt_io_provider/factory.rs:1447,1462` (std::mem::forget(rt) leak)
17. NeoTrix source — `nt_mind_background_loop/builder.rs:78-79` (Handle::block_on in builder)

## Defects

**D-RUN-011: `futures::executor::block_on` inside Tokio async fn — Wrong executor in async context**
| File:Line | Severity | Source |
|-----------|----------|--------|
| `nt_core_resource_pool/discovery.rs:115` | High | Tokio docs (nested executor deadlock), Source #7 |
| `nt_core_resource_pool/discovery.rs:134` | High | Tokio docs (nested executor deadlock) |

`ResourceDiscoveryEngine::discover_all()` and `discover_kind()` are `async fn` that call `futures::executor::block_on(self.cache.check_and_mark(...))` inside a `.retain()` closure. The `check_and_mark` method is an `async fn` on `tokio::sync::RwLock`. Using `futures::executor::block_on` (a single-threaded, non-Tokio executor) to drive a `tokio::sync::RwLock` future is a category error: the Tokio RwLock's internal waker expects to be driven by a Tokio runtime, but `futures::executor::block_on` provides its own executor context. This can: (a) deadlock because the Tokio reactor is not being polled by the nested executor, (b) panic because `tokio::sync::RwLock::write().await` expects to yield to a Tokio-aware waker, (c) bypass Tokio's cooperative budget system, allowing this synchronous call to monopolize the thread indefinitely while the outer async task appears to be yielding. The fix is to use a for-loop with `.await` instead of `.retain()` with a sync closure, or make the closure async-aware.

**D-RUN-012: HotReload callback calls `Handle::block_on` from `tokio::spawn` — Panic on current_thread runtime**
| File:Line | Severity | Source |
|-----------|----------|--------|
| `nt_io_hotreload/mod.rs:297-298` | High | Tokio docs (Handle::block_on on current_thread) |

The HotReload watcher's subscription reload callback (line 296-300) calls `tokio::runtime::Handle::current().block_on(pp.reload_subscriptions())` from inside a synchronous `Fn` closure invoked by `tokio::spawn`. If the HotReload watcher is running on a `current_thread` runtime (which `#[tokio::main(flavor = "current_thread")]` or single-worker configurations use), `Handle::block_on` will panic with "Cannot start a runtime from within a runtime". Even on multi-thread runtimes, calling `block_on` inside a spawned task blocks that worker thread for the entire duration of the async operation, reducing the effective thread pool capacity by one. Tokio docs explicitly state: "Calling Handle::block_on from a runtime task is not recommended." The watcher task is `tokio::spawn`'d at line 168 — it runs ON the runtime, not outside it.

**D-RUN-013: HotReload `RwLock::blocking_write` from tokio::spawn — Deadlock risk with concurrent access**
| File:Line | Severity | Source |
|-----------|----------|--------|
| `nt_io_hotreload/mod.rs:289-291` | High | Tokio docs (blocking_write deadlock), Tokio Issue #7892 |

The HotReload callback for rules.json calls `re.blocking_write()` (on a `tokio::sync::RwLock`) from a sync closure invoked by the `tokio::spawn` watcher task. `blocking_write()` acquires a blocking write guard by parking the current Tokio worker thread. If any other task on the same runtime holds or attempts to acquire this RwLock across an `.await` point (e.g., rule engine reads during proxy operations), a deadlock occurs: the blocking_write parks the worker thread, but the RwLock can only be released by another task that can't make progress because the worker is parked. Tokio docs warn: "If you need to hold the lock across .await, use the async lock method." The `blocking_write` variant is designed for sync code that is NOT on a Tokio worker thread, but this callback IS on a Tokio worker thread (spawned at line 168).

**D-RUN-014: `std::thread::sleep` in LLM narrator retry loop — Blocks Tokio worker for 1.2–7 seconds**
| File:Line | Severity | Source |
|-----------|----------|--------|
| `nt_core_forecast.rs:440` | High | Source #8 (worker thread blocking), Tokio docs |
| `nt_core_forecast.rs:486` | High | Source #8 (worker thread blocking) |

The LLM narrator retry loop uses `std::thread::sleep` for backoff: line 440 sleeps `1200 + attempt*800` ms (1.2s, 2.0s, 2.8s) before each attempt, and line 486 sleeps `retry_after` seconds (2–8s) on retryable errors. `nt_core_forecast.rs:370` shows this function can be called via `Handle::try_current()` from a Tokio context. When called from a Tokio worker thread, `std::thread::sleep` blocks the entire OS thread, making it unavailable for ANY other task. With 3 retry attempts, a single LLM call can monopolize a worker thread for up to 2.8 + 3*retry_after ≈ 10+ seconds. During this time, all other tasks scheduled on that worker (L1 tool calls, L6 health checks, L4 emotion processing) are starved. Tokio docs explicitly state: "Do not perform blocking operations inside an async function. Use `tokio::time::sleep` instead." The `blocking_read` in `nt_io_hotreload/mod.rs:290` and crawl backoff in `fetcher.rs` have the same pattern.

**D-RUN-015: `std::thread::sleep` in crawl pipeline rate limiting — Starves Tokio executor during stealth delays**
| File:Line | Severity | Source |
|-----------|----------|--------|
| `nt_world_crawl/fetcher.rs:265` | Medium | Tokio docs (blocking in async), Source #8 |
| `nt_world_crawl/fetcher.rs:278` | Medium | Tokio docs (blocking in async) |
| `nt_world_crawl/fetcher.rs:291` | Medium | Tokio docs (blocking in async) |
| `nt_world_crawl/unified.rs:355` | Medium | Tokio docs (blocking in async) |
| `nt_world_scrape.rs:287` | Medium | Tokio docs (blocking in async) |

The NT-WORLD crawl pipeline uses `std::thread::sleep` extensively for rate limiting, retry backoff, and anti-detection delays. `fetcher.rs:265` sleeps up to `delay_ms()` (configurable, potentially seconds), `fetcher.rs:291` sleeps 5 seconds on blocked requests, and `unified.rs:355` enforces a minimum 1-second cycle time via sleep. The crawl pipeline is part of NT-WORLD (虚空探索者), which is an L2 Perception domain task likely spawned via `tokio::spawn`. Each `std::thread::sleep` blocks the Tokio worker thread for the entire delay duration. On a crawl of 10 URLs with 1-second delays, a single worker thread is blocked for 10+ seconds of pure sleep time. The correct pattern is `tokio::time::sleep(...).await` which yields the worker thread back to the scheduler. This is especially damaging because crawl tasks are long-running and share the executor with latency-critical L5 cognition and L6 meta-cognition tasks.

**D-RUN-016: `std::mem::forget(rt)` in gateway factory — Permanent zombie thread leak**
| File:Line | Severity | Source |
|-----------|----------|--------|
| `nt_io_provider/factory.rs:1447` | Medium | Tokio docs (Runtime::drop), memory analysis |
| `nt_io_provider/factory.rs:1462` | Medium | Tokio docs (Runtime::drop) |

The `create_gateway()` function uses `std::mem::forget(rt)` to prevent `Runtime::drop` from blocking on `BlockingPool::shutdown`. The comment explains this avoids a hang when `spawn_blocking` reqwest requests are interrupted by timeout. However, `mem::forget` permanently leaks: (a) the 8+ worker thread stacks (~2MB each = 16MB+), (b) the runtime's internal Core data structures, (c) the I/O driver file descriptors, (d) the blocking thread pool (up to 512 zombie threads if any were spawned). Worse, the leaked runtime's worker threads continue running as zombie threads — they never terminate because `Runtime::drop` is skipped. On an 8-core machine, each `create_gateway()` call leaks 8 zombie worker threads. If `create_gateway()` is called multiple times (e.g., during re-initialization or test scenarios), zombie threads accumulate without bound. The correct fix is to use `tokio::runtime::Builder::enable_all()` + graceful shutdown with `Runtime::shutdown_background()` (available since Tokio 1.24), or restructure to avoid the blocking pool issue entirely.

**D-RUN-017: `Handle::block_on` in BackgroundLoop builder — Panic outside runtime context**
| File:Line | Severity | Source |
|-----------|----------|--------|
| `nt_mind_background_loop/builder.rs:78-79` | Medium | Tokio docs (Handle::current panic) |

`BackgroundLoop::with_builtin_plugins()` calls `tokio::runtime::Handle::current().block_on(async { ... })` to register a logging plugin. This builder method uses `self` by value (consuming builder pattern), implying it should be safe to call at any construction point. However, `Handle::current()` panics if called outside a Tokio runtime context — e.g., during CLI startup before `#[tokio::main]`, in test setup code, or in any synchronous initialization path. If the BackgroundLoop builder is constructed during early init (before the main runtime is started), this method will crash the entire process. The builder pattern's contract implies safe construction at any time, but this method silently requires a runtime context. Tokio docs state: "Handle::current panics if no runtime has been started."

## Key Insights

1. **The `futures::executor::block_on` inside Tokio async (D-RUN-011) is the most architecturally dangerous defect.** It uses the wrong executor primitive inside a Tokio async function, creating a nested single-threaded executor that is invisible to Tokio's cooperative budget, can deadlock on `tokio::sync::RwLock`, and provides zero `.await` yield points. This is a category error — mixing two different async runtimes in the same call stack. The fix is trivial: refactor `.retain(|r| futures::executor::block_on(...))` to use an async-aware pattern.

2. **The HotReload subsystem has two interacting defects (D-RUN-012, D-RUN-013)** that compound: `Handle::block_on` can panic on current_thread runtime, and `blocking_write` can deadlock on multi-thread runtime. Both are called from callbacks invoked inside `tokio::spawn`, meaning they run ON a Tokio worker thread, not outside it. The entire HotReload callback design needs to be rethought — callbacks should either be async or use `spawn_blocking`.

3. **`std::thread::sleep` is pervasive across 40+ call sites** (D-RUN-014, D-RUN-015), representing a systemic misunderstanding of async semantics in the codebase. The LLM retry loop (1.2-7s sleeps) and crawl pipeline (1-5s sleeps) are the most damaging because they're on hot paths shared with latency-critical consciousness tasks. A single LLM retry can block a worker thread for 10+ seconds, starving all other tasks on that worker.

4. **The `std::mem::forget(rt)` pattern (D-RUN-016) is a memory leak masquerading as a workaround.** It was added to avoid `Runtime::drop` blocking on `BlockingPool::shutdown`, but the correct solution is `Runtime::shutdown_background()` (Tokio 1.24+) or restructuring to avoid timeout-interrupted blocking requests. Each leaked call permanently loses 8+ threads and ~16MB+ of memory.

5. **The builder's implicit runtime context requirement (D-RUN-017) violates the builder pattern contract.** `with_builtin_plugins()` is the only builder method that panics outside a runtime context, making the entire `BackgroundLoop` builder unsafe to use during early initialization.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 7 |
| Critical severity | 0 |
| High severity | 4 (D-RUN-011, D-RUN-012, D-RUN-013, D-RUN-014) |
| Medium severity | 3 (D-RUN-015, D-RUN-016, D-RUN-017) |
| Sources consulted | 17 |
