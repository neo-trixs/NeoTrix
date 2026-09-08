# Batch 885 — Agent 4: Rust Async Timer Patterns Deep Research

## 1. Core Primitives Summary

### `tokio::time::sleep` / `sleep_until`
- Future that completes at a specific `Instant`. Does no work while waiting.
- At millisecond granularity, requires runtime with time driver enabled.
- First tick of `interval` completes immediately; `sleep` waits full duration.

### `tokio::time::interval` / `interval_at`
- Stream-like utility yielding values at a fixed period.
- Unlike repeated `sleep` in a loop, `Interval` measures time since last tick to prevent drift.
- First tick fires immediately. Period of zero panics.
- Cancellation-safe: if `tick()` is used in `select!` and another branch wins, no tick is consumed.

### `tokio::time::timeout` / `timeout_at`
- Wraps a future with a time bound. Returns `Result<T, Elapsed>`.
- **Critical**: The inner future is polled *before* the timeout is checked. If the future does not yield, it can complete and exceed the timeout without returning an error.
- Cooperative budget workaround (tokio 1.38.1+): if the inner future exhausts the coop budget, `timeout` polls the delay with unconstrained budget to ensure the deadline is evaluated.

### `MissedTickBehavior` (3 strategies)
| Strategy | Behavior |
|----------|----------|
| **Burst** (default) | Fires all missed ticks immediately until caught up |
| **Delay** | Schedules next tick `period` from when `tick()` was called, resetting the cadence |
| **Skip** | Skips to next multiple of `period` from `start` time |

---

## 2. Common Pitfalls (from tokio issues + forums)

### 2.1 `sleep` in `select!` resets every loop iteration
```rust
// BUG: sleep is recreated each iteration, never fires if messages arrive fast
loop {
    let sleep = sleep(Duration::from_millis(100));
    tokio::pin!(sleep);
    tokio::select! {
        _ = &mut sleep => { /* batch process */ }
        value = rx.recv() => { values.push(value); }
    }
}
// FIX: use `interval` instead, which survives across iterations
```
Source: [tokio users forum](https://users.rust-lang.org/t/accumulating-and-handling-values-at-a-fixed-time-period-with-async-rust/99057)

### 2.2 `timeout` does not fire for non-yielding futures
If the inner future never returns `Pending` (e.g., contains `std::thread::sleep` or CPU-bound loops), `timeout` will never check the deadline. The future completes with `Ok()` even if it took 10x longer than the timeout.
Source: [tokio#5119](https://github.com/tokio-rs/tokio/issues/5119), [tokio#6726](https://github.com/tokio-rs/tokio/discussions/6726)

### 2.3 `select!` branch with async block that immediately completes
```rust
tokio::select! {
    _ = timer.tick() => { /* never reached if other arm always ready */ }
    _ = async { if s { s = false; long_sleep().await; } } => {}
}
```
If the async block in the second arm always completes (either immediately or after one execution), it wins every `select!` iteration. Use `if` preconditions outside the async block, or use `std::future::pending()`.
Source: [tokio#6544](https://github.com/tokio-rs/tokio/issues/6544)

### 2.4 Waker clone causes memory leak when futures are canceled
When `select!` cancels a branch, if that branch captured a waker clone (e.g., by awaiting on a channel), the task memory is not freed until all waker clones are dropped. The `Stage` enum in the task retains the size of the largest future even after cancellation.
Source: [tokio#5861](https://github.com/tokio-rs/tokio/issues/5861)

### 2.5 Timer wheel race condition (tokio 1.38.0)
The sharded timer wheel PR (#6534) introduced a race where `timeout()` could hang indefinitely. `Driver::park_internal()` calculated `expiration_time` as `None` right after creating a new `Sleep`, so the timeout never fired. Fixed in 1.38.1 via #6683.
Source: [tokio#6682](https://github.com/tokio-rs/tokio/issues/6682)

### 2.6 Cancelled timer entry leaks into wheel
A `Sleep` created then dropped (e.g., losing branch of `select!`) before the worker parks can leave entries permanently stuck in the timer wheel. The `cancel()` flips the `cancelled` bit but has no `cancel_tx` to push into the queue. Entry stays until natural expiration (~2 years) or runtime shutdown. Fixed in tokio 1.53.1 (#8252).
Source: [tokio#8252](https://github.com/tokio-rs/tokio/pull/8252)

### 2.7 Cooperative scheduling budget exhaustion
Tokio's `coop` module gives each task a budget of 128 operations per tick. If a task exhausts the budget without yielding, all Tokio resources return `Poll::Pending` until the task yields. This can cause `timeout` to never fire if the inner future is a budget-depleting loop. The workaround (`coop::with_unconstrained`) was added in tokio 1.38.1.
Source: [tokio#4314](https://github.com/tokio-rs/tokio/pull/4314), [coop docs](https://docs.rs/tokio/latest/tokio/task/coop/index.html)

---

## 3. Defects for NeoTrix (5+ extracted)

### DEFECT-1: `MissedTickBehavior::Burst` causes thundering-herd tick storms
**Impact**: HIGH — NeoTrix heartbeat, SEAL pipeline, and any periodic health check use `interval`.
**Root cause**: Default `Burst` behavior fires all missed ticks immediately when the consumer catches up. If a SEAL phase takes longer than the interval period (common under load), the next poll triggers a burst of N ticks in rapid succession, each invoking the full tick handler. This can cause exponential work amplification.
**Fix**: Explicitly set `set_missed_tick_behavior(MissedTickBehavior::Skip)` or `::Delay` on all NeoTrix intervals. Document the choice per-module.
**Tokio ref**: [MissedTickBehavior docs](https://docs.rs/tokio/latest/tokio/time/enum.MissedTickBehavior.html)

### DEFECT-2: `timeout()` silently succeeds for non-yielding futures in NT-MIND distillation
**Impact**: HIGH — SEAL pipeline distillation phases do heavy computation.
**Root cause**: If a distillation task or `experience-tree` absorption phase contains CPU-bound work without `.await` yield points, `tokio::time::timeout()` will never fire the deadline. The task completes with `Ok()` regardless of elapsed time, defeating timeout protection.
**Fix**: Either (a) add `tokio::task::yield_now().await` periodically in compute-heavy loops, (b) use `tokio::task::spawn_blocking` for CPU-bound work, or (c) implement manual deadline checking with `Instant::now() > deadline` inside the compute loop.
**Tokio ref**: [tokio#5119](https://github.com/tokio-rs/tokio/issues/5119)

### DEFECT-3: `sleep` in `select!` loop causes timer reset storm in NT-ACT orchestration
**Impact**: MEDIUM — NT-ACT task orchestration loops.
**Root cause**: Common pattern of creating `sleep` inside a `select!` loop (e.g., batch accumulation with timeout). Each loop iteration recreates the sleep future, resetting the timer. If messages arrive faster than the timeout, the timeout never fires — all messages are processed one-by-one instead of batched.
**Fix**: Use `interval` (which persists across loop iterations) instead of `sleep` for periodic triggers in `select!` loops. `interval.tick()` is cancellation-safe and does not reset on `select!` completion.
**Tokio ref**: [Users forum](https://users.rust-lang.org/t/accumulating-and-handling-values-at-a-fixed-time-period-with-async-rust/99057)

### DEFECT-4: Timer wheel entry leak on `select!` branch cancellation
**Impact**: MEDIUM — Any NeoTrix task using `tokio::select!` with `timeout` or `sleep` branches.
**Root cause**: When a `Sleep` is created (e.g., as part of `timeout`) and dropped (losing `select!` branch) before the worker thread parks, the timer entry's `cancelled` bit is set but no `cancel_tx` exists yet. The entry is then inserted into the wheel with no way to remove it except natural expiration. Under high-frequency select! loops (e.g., event bus polling), this accumulates leaked timer entries.
**Fix**: Upgrade to tokio ≥ 1.53.1 (fix in #8252). Alternatively, minimize timer creation inside select! branches — prefer pre-created `Interval` or `Sleep` values pinned outside the loop.
**Tokio ref**: [tokio#8252](https://github.com/tokio-rs/tokio/pull/8252)

### DEFECT-5: Cooperative budget starvation in NT-WORLD crawl pipelines
**Impact**: HIGH — NT-WORLD UnifiedCrawler and fetcher pipelines.
**Root cause**: Tokio's cooperative scheduling gives each task 128 operations per tick. Crawl pipelines that process many items in a tight loop (parsing, classification, embedding) without hitting Tokio I/O resources can exhaust the budget. Other tasks (heartbeat, EventBus consumers) starve. The crawl task itself doesn't yield because non-Tokio operations (regex, serde, vector math) don't participate in `coop`.
**Fix**: (a) Add explicit `tokio::task::consume_budget().await` or `yield_now().await` in crawl processing loops. (b) Use `spawn_blocking` for CPU-heavy parsing/classification. (c) Consider `tokio::task::coop::cooperative()` wrapper for non-Tokio streams.
**Tokio ref**: [coop module](https://docs.rs/tokio/latest/tokio/task/coop/index.html), [tokio#2542](https://github.com/tokio-rs/tokio/issues/2542)

### DEFECT-6: Waker-clone memory leak in long-running NT-CORE GWT broadcast tasks
**Impact**: LOW-MEDIUM — GWT attention broadcast tasks with select! on multiple channels.
**Root cause**: When `select!` cancels a branch that captured a waker clone (e.g., from `channel.recv()`), the task's `Stage` enum retains the full future size even after the future is dropped. The waker holds a strong reference to the task's allocation. Under sustained operation, cancelled branches with large captured futures leak memory proportional to `(sizeof.cancelled_future × num_waker_clones)`.
**Fix**: (a) Keep captured state minimal in select! branches. (b) Use `CancellationToken` with `tokio::select!` for clean shutdown. (c) Box large futures inside select! branches to cap leaked allocation at `sizeof(Box)`. (d) Monitor with `tokio-console` for task memory growth.
**Tokio ref**: [tokio#5861](https://github.com/tokio-rs/tokio/issues/5861)

### DEFECT-7: `timeout` poll ordering allows deadline bypass in NT-MEMORY KB operations
**Impact**: MEDIUM — NT-MEMORY SQLite/embedding operations wrapped in timeout.
**Root cause**: `Timeout::poll` polls the inner future first, then checks the deadline. If the inner future returns `Poll::Ready` on the same poll that the deadline expired, `Ok(result)` is returned instead of `Err(Elapsed)`. For KB operations that complete synchronously (e.g., in-memory FTS5 queries), this means the timeout provides no protection — the query always "succeeds" even if it took longer than expected.
**Fix**: Use `tokio::time::timeout_at(deadline, ...)` with `deadline = Instant::now()` to get immediate expiry, or implement manual deadline checking: record `Instant::now()` before the operation and check after. For critical KB paths, use `spawn_blocking` with a separate timeout on the JoinHandle.
**Tokio ref**: [timeout.rs](https://github.com/tokio-rs/tokio/blob/c637f6e7/tokio/src/time/timeout.rs)

---

## 4. Recommended NeoTrix Timer Patterns

### Pattern 1: Pinned Interval for Periodic Tasks
```rust
let mut interval = tokio::time::interval(Duration::from_secs(60));
interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
loop {
    interval.tick().await;
    do_periodic_work().await;
}
```

### Pattern 2: Timeout with Yield Guarantee
```rust
tokio::time::timeout(Duration::from_secs(5), async {
    // Must yield periodically or use spawn_blocking
    for chunk in data.chunks(1000) {
        process(chunk);
        tokio::task::yield_now().await; // ensure timeout can fire
    }
}).await
```

### Pattern 3: Select with CancellationToken (not sleep)
```rust
let cancel = CancellationToken::new();
let child = cancel.clone();
tokio::select! {
    _ = long_task() => {}
    _ = child.cancelled() => { /* graceful cleanup */ }
}
```

### Pattern 4: Budget-Aware Compute
```rust
let mut budget = 128u32;
for item in items {
    process(item);
    budget -= 1;
    if budget == 0 {
        tokio::task::yield_now().await;
        budget = 128;
    }
}
```

---

## 5. Tokio Version Requirements

| Feature | Minimum Version |
|---------|----------------|
| `MissedTickBehavior` | 1.10+ |
| Cooperative budget timeout fix | 1.38.1 |
| Timer wheel entry leak fix | 1.53.1 |
| `interval.reset_immediately()` | 1.23+ |

**Recommendation**: NeoTrix should pin `tokio >= 1.53.1` to get all timer fixes.

---

## References

- [tokio::time docs](https://docs.rs/tokio/latest/tokio/time/index.html)
- [MissedTickBehavior](https://docs.rs/tokio/latest/tokio/time/enum.MissedTickBehavior.html)
- [tokio cooperative scheduling](https://tokio.rs/blog/2020-04-preemption)
- [tokio#5119 — timeout doesn't fire for non-yielding futures](https://github.com/tokio-rs/tokio/issues/5119)
- [tokio#5861 — memory leak from cancelled futures](https://github.com/tokio-rs/tokio/issues/5861)
- [tokio#6544 — interval not working in select](https://github.com/tokio-rs/tokio/issues/6544)
- [tokio#6682 — timeout hangs (sharded timer wheel race)](https://github.com/tokio-rs/tokio/issues/6682)
- [tokio#8252 — timer entry leak fix](https://github.com/tokio-rs/tokio/pull/8252)
- [tokio#4314 — budget-depleting timeout fix](https://github.com/tokio-rs/tokio/pull/4314)
- [DeepWiki: tokio time](https://deepwiki.com/tokio-rs/tokio/10-time)
