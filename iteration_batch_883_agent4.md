# Iteration Batch 883 — Agent 4: Rust Async Timer Patterns Deep Research

**Date**: 2026-09-07
**Domain**: async timer patterns — `interval`, `timeout`, `sleep`
**Sources**: Tokio docs, Microsoft RustTraining, Qovery, reintech, markaicode, tokio-rs GitHub, Rust community

---

## 1. Core Patterns Summary

### `tokio::time::sleep` vs `std::thread::sleep`

| Property | `tokio::time::sleep` | `std::thread::sleep` |
|----------|---------------------|----------------------|
| Context | Async (Tokio runtime) | Synchronous |
| Blocks OS thread | No | Yes |
| Blocks executor | No | Yes (if called from async ctx) |
| Cancellation | Safe — drop cancels | N/A — blocks until done |

**Critical rule**: `std::thread::sleep` inside an async function silently starves every other future on that executor thread. The compiler emits no warning. It is the #1 async mistake.

### `tokio::time::interval`

- Creates a periodic ticker; first tick completes **immediately**
- Measures time since last tick (not since creation), so late ticks don't accumulate
- **`MissedTickBehavior`** controls catch-up strategy when tick is delayed:
  - `Burst` (default): fires ticks as fast as possible to catch up
  - `Delay`: resets the tick window from current time
  - `Skip`: skips missed ticks, fires at next `period` multiple from `start`
- **Note**: strategies only activate when delay > 5ms (Tokio precision floor)

### `tokio::time::timeout`

- Wraps a `Future` with a deadline; returns `Result<T, Elapsed>`
- On timeout: **drops the future** (cancellation), runs destructors
- **Does NOT cancel spawned tasks** — only the local future is dropped
- Panics if called outside Tokio runtime or without `time` feature

### `tokio::select!` + Timers

- Selects the first-ready branch; **drops all other branches** (cancellation)
- `Interval::tick()` is cancel-safe (dropping mid-select loses no tick)
- `sleep` is cancel-safe (dropping just cancels the timer)
- Non-cancel-safe futures inside `select!` get silently dropped mid-operation

---

## 2. Extracted Defects for NeoTrix

### Defect 1: `std::thread::sleep` in Async Context — Executor Starvation (48 occurrences)

**Severity**: HIGH
**Files affected**: `nt_core_event_bus.rs:464`, `registry_watcher.rs:202`, `self_model.rs:281`, `nt_world_scrape.rs:287/348`, `nt_world_crawl/fetcher.rs:265/278/291`, `nt_memory_kb/nt_http.rs:242/374`, `nt_core_observer_error.rs:59/359/371`, and 30+ more locations.

**Root cause**: `std::thread::sleep` is called from within Tokio async contexts (spawned tasks, async functions). Each call blocks the executor thread for the sleep duration, starving all other tasks.

**Evidence** (nt_core_event_bus.rs:464):
```rust
// Inside tokio::spawn async block — BLOCKS THE EXECUTOR
Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
    std::thread::sleep(std::time::Duration::from_millis(10));
}
```
This is an event-bus poll loop. Every 10ms of idle blocks the Tokio thread. With multiple subscribers (the code uses `.collect()` to spawn N handlers), N threads are simultaneously blocked on sleep.

**Evidence** (registry_watcher.rs:202):
```rust
// Backoff in retry loop — blocks executor thread
std::thread::sleep(backoff); // Duration::from_millis(100 * 2^attempt)
```
Exponential backoff at 100ms, 200ms, 400ms — each blocks the executor.

**Evidence** (nt_world_crawl/fetcher.rs:265):
```rust
std::thread::sleep(Duration::from_millis(self.strategy.delay_ms() - duration_ms));
```
Rate-limiting delay inside crawl pipeline — blocks the executor thread.

**Impact**: Under concurrent load (e.g., crawling + KB writes + consciousness loop), every `std::thread::sleep` in an async context causes multi-second stalls across all tasks sharing that thread.

**Fix**: Replace with `tokio::time::sleep(...).await` for async contexts. Use `tokio::task::spawn_blocking` only for genuinely CPU-bound work. The crawl/fetcher sleeps are the most impactful — they are in the hot path for NT-WORLD data acquisition.

---

### Defect 2: Default `MissedTickBehavior::Burst` in Background Loop — Thundering Herd on Recovery

**Severity**: MEDIUM
**File**: `nt_mind_background_loop/run.rs:739-758`

**Root cause**: The background loop `spawn_handler!` macro creates intervals with default `MissedTickBehavior::Burst`:
```rust
let mut ticker = tokio::time::interval(
    tokio::time::Duration::from_secs($interval));
loop {
    tokio::select! {
        biased;
        _ = ticker.tick() => { ... }
        _ = rx.changed() => { break; }
    }
}
```

When the handler takes longer than `$interval` (e.g., absorption handler takes 60s, interval is 30s), `Burst` fires ticks as fast as possible to "catch up." This creates a burst of rapid-fire executions that can:
1. Trigger concurrent denylist checks, lock contention on `h.lock().await`
2. Execute the handler body repeatedly in tight succession before normal timing resumes
3. Amplify load during recovery periods (e.g., after GC pause, system sleep, or high CPU load)

**Impact**: After any period where the handler lags, it fires N catch-up ticks immediately, causing N× concurrent handler invocations competing for the same `Arc<Mutex>` lock.

**Fix**: Set `set_missed_tick_behavior(MissedTickBehavior::Skip)` or `MissedTickBehavior::Delay` after creating the interval. `Skip` is better for background health checks (no redundant work); `Delay` is better if wall-clock cadence matters.

---

### Defect 3: `timeout` on Spawned Task Without Abort — Zombie Tasks

**Severity**: HIGH
**Files affected**: Multiple locations in `nt_shield_proxy_kernel`, `nt_shield_stealth_net`, `nt_io_mail`, `nt_io_download`

**Root cause**: Several places wrap `tokio::spawn` handles in `timeout` but do NOT call `abort()` on timeout:
```rust
// From tokio docs discussion #7213:
let task = tokio::spawn(...);
timeout(&mut task, duration).await; // drops JoinHandle on timeout
// task is NOT aborted — still runs in background
```

When `tokio::time::timeout` expires on a `JoinHandle`, it drops the handle. Dropping the handle does NOT cancel the spawned task. The task continues running, consuming resources, with no one waiting for its result.

**Impact**: Memory leak, CPU waste, and potential resource contention. In NT-SHIELD's proxy kernel and stealth network layers, zombie connection handlers could accumulate during timeout storms.

**Fix**: After timeout on a spawned task, always call `handle.abort()`:
```rust
let handle = tokio::spawn(async { ... });
match timeout(Duration::from_secs(5), handle).await {
    Ok(result) => { /* use result */ }
    Err(_) => { /* handle.abort() is already called when JoinHandle is dropped by timeout... */ }
}
```
Actually, timeout drops the JoinHandle but `abort` must be called explicitly. The correct pattern is:
```rust
let handle = tokio::spawn(async { ... });
match tokio::time::pin!(handle).as_mut().poll(...) { ... }
// Or: save handle, timeout, then abort on Err
```

---

### Defect 4: `timeout` Double-Nested `Result` — Unhandled Inner Error Paths

**Severity**: MEDIUM
**Files affected**: `nt_memory_kb/nt_http.rs:268-275`, `nt_io_mail/imap.rs:402`, `nt_io_mail/self_heal.rs:803`, `nt_shield_proxy_kernel/kernel.rs:488-494`

**Root cause**: `timeout(duration, async_fn_returning_result)` produces `Result<Result<T, E>, Elapsed>`. Many call sites use `?` or `.await` without matching the inner error:
```rust
let resp = tokio::time::timeout(TIMEOUT, pin_client.get(url).send()).await;
// If timeout succeeds but the inner Result is Err, the error is silently swallowed
```

In `nt_memory_kb/nt_http.rs:268-275`, two sequential timeouts on request + text body mean the inner `reqwest::Error` is wrapped in `Result<Result<...>, Elapsed>` and the double-match is easy to get wrong.

**Impact**: Silent error swallowing. Network failures appear as timeout errors instead of actual HTTP errors, misleading diagnostics and retry logic.

**Fix**: Flatten the double result with explicit matching or a helper:
```rust
let result = timeout(TIMEOUT, async { resp.text().await }).await;
match result {
    Ok(Ok(body)) => { /* success */ }
    Ok(Err(e)) => { log::warn!("HTTP error: {e}"); }
    Err(_) => { log::warn!("timeout after {TIMEOUT:?}"); }
}
```

---

### Defect 5: `sleep` Inside Retry Loops Without Drift Correction

**Severity**: LOW-MEDIUM
**Files affected**: `nt_io_download/engine.rs:101`, `nt_io_provider/gateway/keyless.rs:54`, `nt_shield_proxy_kernel/security.rs:388`, `nt_shield_stealth_net/http_client/request.rs:185`

**Root cause**: Retry loops use `sleep(Duration::from_secs(backoff))` for exponential backoff, but they don't account for the time already elapsed in the failed operation:
```rust
// nt_io_download/engine.rs
tokio::time::sleep(Duration::from_secs(backoff)).await;
```
If the operation took 3s and backoff is 2s, the total retry interval is 5s, not 2s. Over multiple retries, drift accumulates. For rate-limited APIs (e.g., crawling), this can cause thundering-herd re-entries.

**Impact**: Retry timing is unreliable. For rate-limited crawl endpoints, accumulated drift can cause requests to fire too early (getting rate-limited again) or too late (wasting time).

**Fix**: Use `Instant::now()` to measure elapsed time and sleep for the remainder:
```rust
let start = Instant::now();
operation().await;
let elapsed = start.elapsed();
if elapsed < backoff {
    sleep(backoff - elapsed).await;
}
```
Or use `tokio::time::sleep_until(start + backoff)` for drift-free scheduling.

---

### Defect 6: Missing `tokio::time::pause()` in Timer Tests

**Severity**: LOW
**Files affected**: Test files across the codebase

**Root cause**: Tokio's `time::pause()` is required for deterministic timer testing. Without it, tests relying on `interval` or `timeout` are flaky — they depend on wall-clock timing and can fail on slow CI. Tokio docs recommend `#[tokio::test(start_paused = true)]` for any test touching timers.

**Impact**: Flaky tests, especially on resource-constrained CI environments. Timer-dependent tests may pass locally but fail in CI.

**Fix**: Add `#[tokio::test(start_paused = true)]` or call `tokio::time::pause()` at the start of any test that uses `sleep`, `interval`, or `timeout`.

---

## 3. Cross-Cutting Patterns

### Pattern: `select!` + `biased` + `interval.tick()` (Good)

The `spawn_handler!` macro in `run.rs:742-757` uses `biased;` selection, which is correct — it ensures shutdown signal is checked first, preventing missed graceful shutdown. However, the interval catch-up behavior (Defect 2) undermines this.

### Pattern: `timeout` for I/O Boundaries (Good, but needs discipline)

NT-SHIELD's proxy kernel, MITM layer, and network modules use `timeout` extensively for network I/O — this is correct. The issue is that some of these timeout wrappers on `JoinHandle` don't abort on expiry (Defect 3).

### Pattern: `std::thread::sleep` in Sync-Only Code (Acceptable)

Some uses of `std::thread::sleep` are in genuinely synchronous contexts (e.g., test helper functions, non-async utility functions). These are acceptable. The problem is distinguishing them from async-context uses — the codebase has no guard against the latter.

---

## 4. Priority Matrix

| # | Defect | Severity | Effort | Impact |
|---|--------|----------|--------|--------|
| 1 | `std::thread::sleep` in async (48 loc) | HIGH | Medium | Executor starvation, latency spikes |
| 2 | Default Burst in background interval | MEDIUM | Low | Thundering herd on recovery |
| 3 | timeout without abort on JoinHandle | HIGH | Low | Zombie tasks, resource leak |
| 4 | Double-nested Result unhandled | MEDIUM | Low | Silent error swallowing |
| 5 | sleep retry loops without drift correction | LOW-MED | Low | Rate limit violations |
| 6 | Missing `time::pause()` in tests | LOW | Low | Flaky CI |

---

## 5. Recommended Fixes (Ordered by Impact)

1. **Audit all `std::thread::sleep` calls** in async contexts → replace with `tokio::time::sleep(...).await` (48 locations, highest impact)
2. **Add `set_missed_tick_behavior(MissedTickBehavior::Skip)`** to all interval users in background loops
3. **Add `handle.abort()` after timeout expiry** on all `JoinHandle` timeout patterns
4. **Flatten double-Result** from `timeout` with explicit three-way match
5. **Use `Instant`-based drift correction** in retry backoff loops
6. **Add `start_paused = true`** to timer-dependent tests

---

*Research complete. 6 defects extracted. 48+ `std::thread::sleep` in async contexts identified as the highest-priority fix.*
