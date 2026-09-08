# Agent 4: Async Timer Patterns (Batch 866)

## Sources
- Tokio docs: `tokio::time` — interval, sleep, timeout, MissedTickBehavior, cancel safety (docs.rs/tokio/latest/tokio/time/)
- Tokio Interval docs: cancel-safe `tick()`, `reset()`, `set_missed_tick_behavior()` (docs.rs/tokio/latest/tokio/time/struct.Interval.html)
- developerlife.com: "Rust async in practice tokio::select!, actor pattern & cancel safety" — sleep-in-select anti-pattern, interval vs sleep mental model, unpinned sleep future dropping state
- StudyRaid: "Timer and Interval Creation" — interval precision, drift, burst behavior, coordination patterns
- Rust Forum (users.rust-lang.org): "Possible issue with tokio::time::Interval" — intervals silently dying after 2 weeks uptime, `MissedTickBehavior::Skip` edge cases, replacing interval mid-loop
- reintech.io: "Tokio Tutorial 2026" — blocking runtime anti-pattern, select!/join!/spawn best practices
- NeoTrix codebase: 98+ `tokio::time` usages, 7 `tick().await` calls, 100+ `Duration` usages across NT-SHIELD, NT-MIND, NT-WORLD, NT-CORE, NT-IO

## Defects

**D-TIMER-001**: `std::thread::sleep()` inside async context blocks the tokio worker thread, starving all other tasks on that thread | `nt_core_event_bus.rs:464` | High | Tokio docs: "never block inside async functions"; code uses `std::thread::sleep(Duration::from_millis(10))` in broadcast receiver loop — a hot path for every event layer

**D-TIMER-002**: `std::thread::sleep()` in retry loop of synchronous `fetch()` and `fetch_with_retry()` — these block the calling thread during exponential backoff (up to 3600s) | `nt_world_crawl/fetcher.rs:265,278,291` | High | Crawler fetcher uses `std::thread::sleep` for rate limiting and retry backoff inside functions called from async context; upstream `reqwest::Client` is async-capable but used synchronously via `block_on` patterns

**D-TIMER-003**: Unpinned `sleep` in `select!` branch — `tokio::time::sleep(Duration::from_secs(60))` created inline in `select!` branch in GC task; if the `sleep` branch wins and other branch is slow, the sleep future is dropped and recreated on next iteration, causing non-deterministic GC timing | `nt_shield_proxy_kernel/kernel.rs:223` | Medium | Tokio cancel safety docs: unpinned sleep in select branch is not cancel-safe; interval should be used instead for periodic tasks

**D-TIMER-004**: `spawn_handler!` macro uses `tokio::time::interval` in `select!` with `biased` — biased select always polls ticker first, so `tick()` can accumulate missed ticks during long handler execution (default `MissedTickBehavior::Burst` means multiple ticks fire instantly after a slow handler) | `nt_mind_background_loop/run.rs:739-758` | Medium | Tokio docs: default Burst behavior causes tick storms; `MissedTickBehavior::Skip` or `Delay` should be configured for health-monitoring intervals to avoid cascading handler invocations

**D-TIMER-005**: Hardcoded `tokio::time::sleep(Duration::from_millis(1000))` in DOM extraction loop as "wait for DOM stability" — no backoff, no cancellation, no configurable timeout; spins indefinitely if DOM never stabilizes | `nt_world_crawl/dom_extractor/mod.rs:173` | Medium | Network timeout research: polling loops without timeout/escape hatch risk unbounded execution; should use `tokio::time::timeout` wrapping the entire loop

**D-TIMER-006**: `tokio::time::sleep(Duration::from_millis(10))` as a spin-wait for semaphore acquisition — busy-loops on a 10ms timer burning CPU; the `try_acquire` + sleep pattern creates a thundering herd if multiple tasks compete | `nt_io_provider/gateway/execution.rs:137` | Medium | Tokio docs: prefer `Semaphore::acquire()` (async native) over `try_acquire` + sleep spin-wait; the current pattern wastes a worker thread polling every 10ms

**D-TIMER-007**: Circuit breaker `allow_request()` uses `std::time::Instant::elapsed()` which is monotonic but the `CircuitBreaker` derives `Clone` and stores `last_tripped: Option<Instant>` — `Instant` is not `Clone`-safe across threads (it's a monotonic counter, not a wall clock), and the circuit breaker state can desync if cloned and used from different tokio tasks | `nt_core_observer_error.rs:136,159-163` | Low | Rust std docs: `Instant` measures monotonic time; cloning a `CircuitBreaker` with an `Instant` field copies the timestamp which is semantically correct but the `Clone` derive on a type containing `Instant` may mask thread-safety issues if shared via `Arc<Mutex>` vs plain `Clone`

**D-TIMER-008**: `tokio::time::sleep(Duration::from_secs(2))` as arbitrary retry delay in llama-server restart loop with no jitter — all concurrent restart attempts will thunder at exactly the same 2s interval | `nt_io_provider/llama_process.rs:367` | Low | Retry pattern research: fixed delays without jitter cause thundering herd; exponential backoff with jitter is standard practice

**D-TIMER-009**: Watchdog task in `spawn_watchdog` uses unbounded `loop` + `sleep(30s)` with no shutdown mechanism — if the parent `LlamaProcess` is dropped, the watchdog continues running forever, holding an `Arc<Mutex<Option<Child>>>` reference | `nt_io_provider/llama_process.rs:417-444` | High | Async task lifecycle: spawned tasks must have a shutdown signal (e.g., `CancellationToken` or `watch` channel); orphaned tasks leak resources and prevent clean process exit

**D-TIMER-010**: Shutdown handler in `handlers.rs:48-61` uses a single shared `deadline` sleep shared across all handles in a `for` loop — the 5-second deadline is shared, not per-task; if the first handle takes 4.9s, only 0.1s remains for all subsequent handles | `nt_mind_background_loop/handlers.rs:48-61` | Medium | Graceful shutdown research: each task should get its own deadline or use `tokio::time::timeout` per task; shared deadline races against task iteration

## Key Insights

1. **Pervasive blocking sleeps**: The most critical pattern is `std::thread::sleep()` used in async contexts (`nt_core_event_bus.rs:464`, `fetcher.rs:265,278,291`, `nt_core_observer_error.rs:59`). This blocks the tokio worker thread entirely, potentially starving dozens of concurrent tasks. Tokio's cooperative scheduling cannot preempt `std::thread::sleep`.

2. **Cancel safety anti-patterns**: Multiple `tokio::time::sleep()` calls appear inline in `select!` branches (kernel.rs:223, proxy_pool.rs:688) rather than using pinned or interval-based alternatives. Per Tokio docs, inline `sleep` in select creates a new future each iteration, losing timing state.

3. **Missing shutdown signals**: Watchdog tasks (llama_process.rs:417) and proxy cleanup tasks (kernel.rs:220) spawn unbounded loops without `CancellationToken` or `watch` channel integration, risking orphaned tasks.

4. **No MissedTickBehavior configuration**: The `spawn_handler!` macro (run.rs:739) uses default `Burst` behavior, which can cause tick storms after slow handler execution — a potential cascade failure in the background loop.

5. **Hardcoded magic intervals**: Timer intervals are scattered as inline `Duration::from_secs(N)` constants across 50+ files with no centralized configuration or documentation of why each value was chosen.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources analyzed | 7 web + 98 codebase matches |
| Files with timer patterns | 50+ |
