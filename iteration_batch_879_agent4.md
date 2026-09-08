# Iteration Batch 879 — Agent 4: Rust Async Timer Patterns DEEP Research

## Scope

Deep research on `tokio::time::interval`, `tokio::time::timeout`, `tokio::time::sleep`, `MissedTickBehavior`, timer wheel internals, and production failure modes. Extracted 5+ defects applicable to NeoTrix.

---

## 1. Research Sources

| Source | URL | Key Finding |
|--------|-----|-------------|
| tokio::time docs | docs.rs/tokio/latest/tokio/time/ | `interval()` default `MissedTickBehavior::Burst` fires all missed ticks immediately — a thundering-herd trap |
| tokio #5987 | github.com/tokio-rs/tokio/issues/5987 | Polling a timer costs 4–40µs per call; in a `select!` loop this adds massive tax to IO branches |
| tokio #6545 | github.com/tokio-rs/tokio/issues/6545 | `interval` silently freezes when task blocks the worker thread without yielding; `biased` select + long `do_work` starves the time driver |
| tokio #8334 (PR) | github.com/tokio-rs/tokio/pull/8334 | Timer wheel corruption after ~12 days uptime: long `Sleep` + short intervals → top-level slot 0 hijacked → silent hang |
| tokio #5119 | github.com/tokio-rs/tokio/issues/5119 | `timeout` does NOT fire if the inner future completes without yielding — sync blocking defeats timeout entirely |
| tokio #7432 | github.com/tokio-rs/tokio/discussions/7432 | `timeout` polls inner future BEFORE checking deadline; CPU-bound futures can exceed timeout by arbitrarily long amounts |
| rust-users #122420 | users.rust-lang.org/t/possible-issue-with-tokio-interval/122420 | ~10% of devices lose interval ticks after 2 weeks; root cause: blocking in select loop starves timer driver |
| Markaicode 2026 guide | markaicode.com/errors/rust-timeout-fix/ | `spawn_blocking` tasks cannot be cancelled; timeout on handle only cancels the wait, not the thread |
| Tony Finch 2026 | dotat.at/@/2026-02-16-async.html | Custom `Sleep`/`Yield` implementations require careful Waker management; `Waker::noop()` misused leads to silent stalls |
| Microsoft async-book ch12 | microsoft.github.io/RustTraining/async-book/ch12-common-pitfalls.html | MutexGuard held across `.await` + timeout → deadlock; `std::thread::sleep` in async fn → executor starvation |
| hackernoon timers | hackernoon.com/8-timer-implementation-patterns | Timer wheels: O(1) insert/delete but corruption risk at wheel boundary; min-heap: O(log n) but predictable |

---

## 2. NeoTrix Codebase Audit — Timer Usage

### 2.1 Interval Usage (1 site)

**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:739`
```rust
let mut ticker = tokio::time::interval(
    tokio::time::Duration::from_secs($interval));
```
- **No `set_missed_tick_behavior`** → defaults to `Burst`
- Used inside `spawn_handler!` macro with `biased` select
- Handler body acquires `h.lock().await` before work → lock contention can cause tick starvation

### 2.2 Timeout Usage (50+ sites)

Heavy usage across NT-SHIELD (proxy, SOCKS5, network), NT-IO (mail, provider gateway, download), NT-WORLD (search, crawl), NT-MIND (absorption). Patterns observed:

1. **SOCKS5 handshake** (`local_proxy.rs`): 7 sequential `timeout` calls per connection — 5s/10s each, no cumulative timeout
2. **Network pool** (`network_pool.rs`): 2s–30s timeouts on reads/writes — individual operation timeouts, no deadline propagation
3. **Provider gateway** (`execution.rs`): `sleep(20ms)` inside retry loops — fixed backoff, no exponential
4. **Absorption** (`handlers_absorption.rs`): Nested `timeout` — outer 30s, inner 15s — good pattern but no logging on timeout
5. **MITM proxy** (`mitm.rs`): 10s–30s timeouts on stream reads — hardcoded, not configurable

### 2.3 Sleep Usage (40+ sites)

Used for retry backoff, startup delays, polling loops. Key anti-patterns:
- `sleep(1000)` in DOM extractor polling loop — fixed 1s delay regardless of DOM state
- `sleep(10ms)` in reasoning engine word-streaming — artificial latency
- `sleep(Duration::from_secs(2))` in llama process — hardcoded startup wait

---

## 3. Extracted Defects (5+)

### DEFECT-1: Silent Interval Freezing from Worker Thread Starvation

**Severity**: Critical  
**Domain**: NT-MIND (background loop), NT-SHIELD (proxy kernel)

**Evidence**: tokio #6545, #6544, rust-users #122420. When a `select!` loop with an `interval` branch also runs blocking or long-running work in another branch without yielding, the time driver never gets polled. The interval silently stops ticking. ~10% of production devices affected after 2 weeks uptime.

**NeoTrix exposure**: `spawn_handler!` macro at `run.rs:739` uses `biased` select with `ticker.tick()` vs handler body that acquires `h.lock().await` and runs arbitrary logic. If handler logic is slow (KB write, absorption cycle, denylist check), the interval starves. The `biased` priority actually makes this worse — if the handler branch is ready, the ticker branch is never polled.

**Recommendation**:
```rust
// BEFORE (silent freeze risk):
tokio::select! {
    biased;
    _ = ticker.tick() => { /* long work */ }
    _ = rx.changed() => { break; }
}

// AFTER (explicit yield + timeout guard):
tokio::select! {
    _ = ticker.tick() => {
        let result = tokio::time::timeout(
            Duration::from_secs(30),
            async { /* handler work */ }
        ).await;
        if result.is_err() {
            log::error!("[bg-loop] handler '{}' exceeded 30s deadline", $name);
        }
    }
    _ = rx.changed() => { break; }
}
```
Also: replace `biased` with default (random) select to give ticker branch fair scheduling.

---

### DEFECT-2: Burst MissedTickBehavior Causing Thundering-Herd Recovery

**Severity**: High  
**Domain**: NT-MIND, all periodic tasks

**Evidence**: tokio docs, rust-users #97741, #122420. Default `MissedTickBehavior::Burst` fires ALL missed ticks immediately when an interval catches up. If a handler is blocked for 5s with a 1s interval, the next poll fires 5 ticks in rapid succession — causing resource spikes, duplicate work, and potential panics.

**NeoTrix exposure**: `run.rs:739` never calls `set_missed_tick_behavior()`. After any handler stall, the background loop will burst-fire catch-up ticks. With the handler acquiring `h.lock().await`, consecutive burst ticks will serialize on the lock but still cause unnecessary load.

**Recommendation**:
```rust
let mut ticker = tokio::time::interval(Duration::from_secs($interval));
ticker.set_missed_tick_behavior(Missio::Skip); // or Delay
```
For background monitoring loops, `Skip` is almost always correct — you want the NEXT tick, not a replay of history.

---

### DEFECT-3: Timeout Not Firing on Sync-Blocking Futures

**Severity**: High  
**Domain**: NT-SHIELD (SOCKS5, network), NT-IO (provider gateway, mail)

**Evidence**: tokio #5119, #7432, Markaicode guide. `tokio::time::timeout` polls the inner future first; if the inner future completes synchronously (no `.await` yield point), timeout returns `Ok` even if wall-clock time exceeded the deadline. Sync blocking inside async fn completely defeats timeout.

**NeoTrix exposure**: 
- `local_proxy.rs:140-196`: 7 sequential `timeout` calls on SOCKS5 handshake. If `TcpStream::connect` resolves from a cached DNS or local socket, the timeout is bypassed.
- `execution.rs:137`: `sleep(20ms)` in retry loop — if the loop body doesn't yield, timeout is ineffective.
- `nt_io_download/engine.rs:101`: `sleep(Duration::from_secs(backoff))` — backoff uses `tokio::time::sleep` (good), but the outer timeout may not fire if download chunk processing is sync-heavy.

**Recommendation**:
```rust
// Ensure all IO paths yield — use tokio::net::TcpStream, never std::net
// Wrap any sync FFI or CPU-bound code:
let result = tokio::time::timeout(
    Duration::from_secs(10),
    tokio::task::spawn_blocking(move || expensive_sync_call())
).await;
```
Audit all `timeout` call sites to verify the inner future has `.await` yield points. Add `#[tracing::instrument]` to time critical paths.

---

### DEFECT-4: No Cumulative Deadline on Multi-Operation Sequences

**Severity**: Medium-High  
**Domain**: NT-SHIELD (SOCKS5, proxy chain), NT-IO (mail IMAP)

**Evidence**: Stack Overflow #76631005, tokio #7432. When multiple async operations are wrapped in individual timeouts, the total wall-clock time can far exceed any single timeout. A SOCKS5 connection with 7 sequential 5s timeouts can take 35s before failing.

**NeoTrix exposure**: `local_proxy.rs:140-196` performs 7 sequential timeout-wrapped operations (connect, greet write, greet read, connect write, connect read, bound addr read, bound port read). Total worst-case: 10+5+5+10+10+5+5 = 50s for a single SOCKS5 connection. `proxy_chain/chain.rs:334` adds another timeout layer on top.

**Recommendation**:
```rust
// BEFORE: individual timeouts
let stream = tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(addr)).await?;
tokio::time::timeout(Duration::from_secs(5), stream.write_all(&greet)).await?;

// AFTER: deadline-based approach
let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
let remaining = || deadline.saturating_duration_since(tokio::time::Instant::now());
let stream = tokio::time::timeout(remaining(), TcpStream::connect(addr)).await??;
tokio::time::timeout(remaining(), stream.write_all(&greet)).await??;
```
Or use `tokio::time::timeout_at(deadline, ...)` throughout.

---

### DEFECT-5: Timer Wheel Corruption at 12-Day Uptime Boundary

**Severity**: Medium (long-running daemon risk)  
**Domain**: NT-MIND (background loop), any long-running task

**Evidence**: tokio #8334. After ~12 days (2^30 ms), the hierarchical timer wheel's top level gets corrupted. A long `Sleep` (e.g., `i64::MAX` ms) occupies slot 0, and new short intervals land on the same top-level slot, causing silent hangs. Tokio has a fix in PR #8334 but it may not be in all released versions.

**NeoTrix exposure**: NeoTrix is a long-running daemon (desktop + server modes). The `proxy.rs` example uses `sleep(Duration::from_secs(3600))` which is safe (1hr < 12 days), but any `sleep(Duration::from_secs(u64::MAX))` or similar long-duration timer could trigger this. The background loop's `ticker` is fine (short intervals), but if a future holds a very long sleep while other intervals are registered, corruption is possible.

**Recommendation**:
1. Audit all `sleep`/`timeout` durations — cap any "infinite wait" at < 2^30 ms (~12 days) with periodic renewal
2. Ensure tokio version includes the #8334 fix (≥1.41.x or equivalent)
3. Add a watchdog that restarts the runtime if heartbeat fails for > 60s
4. Never use `Duration::from_secs(u64::MAX)` — use `sleep_until(Instant::MAX)` or periodic re-arm

---

### DEFECT-6: Sleep-in-Loop Polling Without State-Change Detection

**Severity**: Medium  
**Domain**: NT-WORLD (DOM extractor)

**Evidence**: Tony Finch 2026, tokio docs. Using `sleep(Duration::from_millis(1000))` in a polling loop is wasteful — it always waits the full duration even if the DOM is ready sooner. It also has no backoff or jitter, causing thundering-herd when multiple extractors run.

**NeoTrix exposure**: `dom_extractor/mod.rs:173`:
```rust
tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
```
This is inside a loop that polls for DOM stability. It always sleeps 1s regardless of whether items are still arriving.

**Recommendation**:
```rust
// Use notify-based approach or exponential backoff:
let mut backoff = Duration::from_millis(100);
loop {
    // ... extraction logic ...
    if no_new_items {
        break;
    }
    tokio::time::sleep(backoff).await;
    backoff = (backoff * 2).min(Duration::from_secs(5));
}
```
Or better: use `tokio::sync::Notify` / `watch` channel to signal DOM changes instead of polling.

---

### DEFECT-7: Artificial Latency in Streaming (sleep per word)

**Severity**: Low  
**Domain**: NT-MIND (reasoning engine)

**Evidence**: tokio best practices — avoid `sleep` for rate limiting when channel backpressure is sufficient.

**NeoTrix exposure**: `engine_core.rs:1914`:
```rust
tokio::time::sleep(std::time::Duration::from_millis(10)).await;
```
This adds 10ms artificial delay between each word in a streaming response. If the response has 500 words, that's 5s of pure sleep. The `mpsc::channel(64)` already provides backpressure.

**Recommendation**: Remove the `sleep` — let channel backpressure handle rate limiting. If UI rendering needs pacing, use `tokio::time::interval(Duration::from_millis(16))` (60fps) instead of per-word sleep.

---

## 4. Summary Table

| # | Defect | Severity | NeoTrix Files | Root Cause |
|---|--------|----------|---------------|------------|
| 1 | Silent interval freeze from worker starvation | Critical | `run.rs:739` | `biased` select + blocking handler |
| 2 | Burst catch-up thundering herd | High | `run.rs:739` | Default `MissedTickBehavior::Burst` |
| 3 | Timeout bypassed by sync-blocking futures | High | `local_proxy.rs`, `execution.rs` | No yield points in inner future |
| 4 | No cumulative deadline on multi-op sequences | Medium-High | `local_proxy.rs:140-196` | Individual timeouts, no deadline |
| 5 | Timer wheel corruption at 12-day uptime | Medium | Any long-running task | Tokio timer wheel boundary |
| 6 | Sleep-in-loop polling without state detection | Medium | `dom_extractor/mod.rs:173` | Fixed sleep instead of notify |
| 7 | Artificial latency via per-word sleep | Low | `engine_core.rs:1914` | sleep instead of channel backpressure |

---

## 5. Recommended Actions

1. **Immediate**: Add `set_missed_tick_behavior(MissedTickBehavior::Skip)` to `run.rs:739`
2. **Immediate**: Replace `biased` with default select in `spawn_handler!` macro
3. **Short-term**: Add cumulative deadline pattern to SOCKS5 handshake (`local_proxy.rs`)
4. **Short-term**: Audit all `timeout` call sites for sync-blocking inner futures
5. **Medium-term**: Add tokio version pinning to ensure #8334 fix is included
6. **Medium-term**: Replace sleep-polling in `dom_extractor` with notify-based approach
7. **Low priority**: Remove artificial `sleep(10ms)` in reasoning engine streaming

---

*Research completed: 2026-09-07 | Sources: 11 web pages + 100+ codebase grep hits*
