# Iteration Batch 881 — Agent 4: Rust Async Timer Patterns

**Topic**: `tokio::time` — interval, timeout, sleep
**Date**: 2026-09-07
**Sources**: tokio docs (docs.rs/tokio), Tokio GitHub issues (#2565, #5119, #6682, #6726, #7432), DeepWiki, Stack Overflow, Rust users forum

---

## 1. Research Summary

### 1.1 Core Primitives

| Primitive | Behavior | Cancellation Safe |
|-----------|----------|-------------------|
| `sleep(duration)` | Completes at `Instant::now() + duration`. Millisecond granularity. | Yes (dropping cancels) |
| `interval(period)` | Yields at fixed period. First tick fires **immediately**. Measures time since last tick (catch-up semantics). | `tick()` is cancellation safe |
| `timeout(dur, fut)` | Wraps a future; returns `Err(Elapsed)` if inner future doesn't complete in time. | Cancellation of the wrapper is safe; inner future cancellation depends on its impl |

### 1.2 MissedTickBehavior (Interval)

| Strategy | When tick missed | Use case |
|----------|-----------------|----------|
| `Burst` (default) | Fires all missed ticks as fast as possible until caught up | Telemetry, metrics — every sample matters |
| `Delay` | Schedules next tick at `now + period` from when tick was called | Rate limiting — maintain spacing |
| `Skip` | Skips missed ticks, aligns to next multiple of `period` from `start` | UI refresh — don't flood with stale frames |

### 1.3 Critical Mechanics

1. **`timeout` polls inner future FIRST**: The future gets a "last chance" to complete before timeout is checked. If the future never yields (`Poll::Pending`), the timeout never fires. This is by design for cancellation safety.
2. **`interval` first tick is immediate**: `interval(Duration::from_secs(5))` fires at t=0, t=5, t=10... Not t=5, t=10.
3. **`timeout` + blocking = no timeout**: `std::thread::sleep` inside an `async` block inside `timeout` means the timeout never fires. The future returns `Ok(...)` after the blocking call completes regardless of elapsed time.
4. **`spawn_blocking` tasks cannot be cancelled**: `timeout` on a `spawn_blocking` join handle only cancels the *wait*, not the underlying OS thread.
5. **Tokio timer is hierarchical wheel**: Six levels, millisecond granularity. Sub-ms precision is not guaranteed.
6. **`sleep` in a loop ≠ `interval`**: Using `sleep` in a loop accumulates drift. `Interval` tracks cumulative time.

---

## 2. Defects Found in NeoTrix

### DEFECT-001: Background Loop Interval Without MissedTickBehavior Configuration
**Severity**: Medium | **Location**: `nt_mind_background_loop/run.rs:739-758`
**Pattern**: `spawn_handler!` macro creates `tokio::time::interval(Duration::from_secs($interval))` but **never calls `set_missed_tick_behavior()`**.
**Problem**: Default `Burst` strategy means if a handler's body takes longer than the interval, multiple rapid-fire ticks execute back-to-back. For background loops doing I/O (absorption, heartbeat, crawl), this can cause resource exhaustion — the handler will execute N times in rapid succession to "catch up" after a slow operation.
**Impact**: Under load, background handlers can spike CPU/disk/network as they burst through missed ticks. The `denylist.check()` gate partially mitigates this, but the burst pattern is still wasteful.
**Fix**:
```rust
let mut ticker = tokio::time::interval(Duration::from_secs($interval));
ticker.set_missed_tick_behavior(MissedTickBehavior::Skip); // or Delay
```
**Relevant code**: `run.rs:739-740` — `interval` created, `MissedTickBehavior` never set.

---

### DEFECT-002: Spin-Wait Loop in Provider Gateway Gate
**Severity**: High | **Location**: `nt_io_provider/gateway/execution.rs:128-138`
**Pattern**:
```rust
loop {
    let acquired = self.tiered_semaphore.lock().unwrap_or_else(|e| e.into_inner()).try_acquire(tier);
    if acquired { break; }
    tokio::time::sleep(Duration::from_millis(20)).await;
}
```
**Problem**: This is an async spin-wait polling at 20ms intervals. Under high concurrency, this creates O(N) wake-ups per second where N is the number of waiting tasks. Each wake-up allocates a timer entry in the hierarchical wheel. This is the classic "async polling with sleep" anti-pattern that the Tokio docs explicitly warn against.
**Impact**: Wasted CPU cycles, timer wheel pressure, potential latency spikes when many providers are contended. The 20ms granularity also means worst-case additional latency of 20ms even when a slot opens immediately.
**Fix**: Use `tokio::sync::Semaphore` (async-aware, zero-cost when no contention) instead of `std::sync::Mutex` + sleep polling. Or at minimum use `tokio::sync::Notify` to wake exactly one waiter.
**Relevant code**: `execution.rs:126-138` — spin-wait with `sleep(20ms)`.

---

### DEFECT-003: FakeIpTable Cleanup Task Never Terminates
**Severity**: Medium | **Location**: `nt_shield_proxy_kernel/fakeip.rs:139-146`
**Pattern**:
```rust
pub fn start_cleanup_task(self: &Arc<Self>, interval: Duration) {
    let table = Arc::clone(self);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            table.cleanup_expired().await;
        }
    });
}
```
**Problem**: The spawned task has **no shutdown mechanism**. It loops forever with no cancellation token, no `select!` on a shutdown signal, and the `JoinHandle` is dropped immediately (fire-and-forget). When the `FakeIpTable` is dropped, the cleanup task continues running as a leaked task holding an `Arc` reference, preventing deallocation.
**Impact**: Task leak on `FakeIpTable` recreation/restart. Accumulated leaked tasks over long uptimes. The `Arc` reference prevents memory cleanup.
**Fix**:
```rust
pub fn start_cleanup_task(self: &Arc<Self>, interval: Duration, mut shutdown: broadcast::Receiver<()>) {
    let table = Arc::clone(self);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(interval) => { table.cleanup_expired().await; }
                _ = shutdown.recv() => { break; }
            }
        }
    });
}
```
**Relevant code**: `fakeip.rs:139-146`.

---

### DEFECT-004: Fixed 10ms Sleep in Parallel Executor Instead of Yield
**Severity**: Low | **Location**: `nt_core_parallel/executor.rs:37-39`
**Pattern**:
```rust
let handle = tokio::spawn(async move {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    input
});
```
**Problem**: Each spawned task sleeps for exactly 10ms before returning its input. This is a synthetic delay that wastes 10ms per task. If the parallel executor runs N tasks sequentially (as the code suggests — the `for` loop is not concurrent), this adds `N × 10ms` of pure dead time.
**Impact**: Unnecessary latency in the parallel execution path. 10 tasks = 100ms wasted.
**Fix**: Replace with `tokio::task::yield_now().await` if the intent is cooperative yielding, or remove entirely if the delay serves no purpose. If the intent is rate-limiting, use an `Interval`.
**Relevant code**: `executor.rs:38`.

---

### DEFECT-005: sleep-in-loop Drain Pattern Without Interval
**Severity**: Low | **Location**: `nt_shield_proxy_kernel/kernel.rs:283-297`
**Pattern**:
```rust
loop {
    let conns = kernel.active_connections.load(Ordering::Relaxed);
    if conns == 0 { break; }
    if drain_start.elapsed() >= drain_deadline { break; }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```
**Problem**: This is a "sleep-poll" loop checking connection count every 100ms. It's a classic polling anti-pattern — the task wakes every 100ms, loads an atomic, and either breaks or sleeps again. Under ideal conditions, this adds up to 100ms of extra latency before detecting zero connections.
**Impact**: Minor latency on shutdown. Not critical, but architecturally impure — an `AtomicUsize` combined with a `Notify` or `watch` channel would be zero-latency.
**Fix**: Use `tokio::sync::watch` or `tokio::sync::Notify` to signal when `active_connections` reaches zero. Only fall back to sleep-poll as a timeout watchdog.
**Relevant code**: `kernel.rs:283-297`.

---

### DEFECT-006: Exponential Backoff Without Cap in Download Engine
**Severity**: Medium | **Location**: `nt_io_download/engine.rs:99-101`
**Pattern**:
```rust
let backoff = self.config.retry_base_secs * 2_u64.pow(attempt - 1);
tokio::time::sleep(Duration::from_secs(backoff)).await;
```
**Problem**: Unbounded exponential backoff. If `retry_base_secs = 5` and `max_retries = 10`, the last retry waits `5 × 2^9 = 2560 seconds` (42+ minutes). There is no `min(cap, calculated)` logic.
**Impact**: The download engine can block for arbitrarily long periods on retries, potentially tying up a tokio task for minutes/hours.
**Fix**:
```rust
let backoff = self.config.retry_base_secs.saturating_mul(2_u64.pow(attempt - 1));
let capped = backoff.min(self.config.retry_max_secs.unwrap_or(60));
tokio::time::sleep(Duration::from_secs(capped)).await;
```
**Relevant code**: `engine.rs:99-101`.

---

### DEFECT-007: Retry Sleep Without Cancellation Token in SelfHeal
**Severity**: Low | **Location**: `nt_file_ability/self_heal.rs:67-68`
**Pattern**:
```rust
for attempt in 0..=self.config.max_retries {
    match self.try_process_file(path).await {
        Ok(model) => return Ok(model),
        Err(e) => {
            if attempt < self.config.max_retries {
                sleep(self.config.retry_delay).await;
                // fallback attempt...
            }
        }
    }
}
```
**Problem**: The retry loop uses a fixed delay with no jitter and no cancellation mechanism. If the system is shutting down, the retry loop will sleep through the shutdown signal (no `select!` on shutdown). Combined with a fixed 100ms delay, retries are predictable and can cause thundering-herd effects when many files fail simultaneously.
**Impact**: Shutdown delay (up to `max_retries × retry_delay`), no jitter for thundering-herd mitigation.
**Fix**: Add jitter (`retry_delay ± 20%`) and `tokio::select!` on shutdown signal.

---

## 3. Summary Table

| ID | Severity | File | Defect | Pattern |
|----|----------|------|--------|---------|
| 001 | Medium | `run.rs:739` | Interval uses default `Burst` in background loop | Missing `set_missed_tick_behavior` |
| 002 | High | `execution.rs:128` | Spin-wait with `sleep(20ms)` for semaphore | Async polling anti-pattern |
| 003 | Medium | `fakeip.rs:139` | Cleanup task never terminates | No shutdown token, fire-and-forget |
| 004 | Low | `executor.rs:38` | Fixed 10ms sleep per task | Synthetic delay, should be `yield_now` or removed |
| 005 | Low | `kernel.rs:283` | Sleep-poll loop for connection drain | Should use `Notify`/`watch` |
| 006 | Medium | `engine.rs:99` | Unbounded exponential backoff | Missing cap on retry delay |
| 007 | Low | `self_heal.rs:67` | Fixed retry delay, no jitter, no shutdown | Add jitter + cancellation |

---

## 4. Tokio Timer Anti-Patterns to Audit

| Anti-Pattern | Occurrences in NeoTrix | Risk |
|-------------|----------------------|------|
| `sleep` in `loop` (polling) | `kernel.rs:283`, `execution.rs:128`, `fakeip.rs:142` | Timer wheel pressure, wasted wake-ups |
| Interval without `MissedTickBehavior` | `run.rs:739` (macro-generated) | Burst storms under load |
| `timeout` wrapping blocking work | `mitm.rs` (TLS operations), `network_pool.rs` (TCP connect) | Timeout may not fire if future doesn't yield |
| Fire-and-forget spawned tasks | `fakeip.rs:141`, `engine_core.rs:1909` | Task leak, no cleanup |
| Exponential backoff without cap | `engine.rs:99`, `llama_process.rs:367` | Unbounded sleep duration |
