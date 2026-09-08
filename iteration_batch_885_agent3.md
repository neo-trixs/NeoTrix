# Iteration Batch 885 — Agent 3: Rust Async Synchronization Defects

**Research Topic**: Mutex, RwLock, Semaphore in Rust async (tokio)
**Sources**: Tokio docs, Rust Clippy, tokio-rs GitHub issues (#2599, #5176, #6450, #149359), e6data deadlock case study, Oxide RFD-0400, async-lock crate docs, multiple blog/tutorial analyses
**Date**: 2026-09-07

---

## Defect D-885-1: `std::sync::Mutex` Held Across `.await` — Thread Starvation Deadlock

**Severity**: Critical
**Category**: Deadlock / Runtime Starvation

**Description**: When `std::sync::Mutex` guard is held across an `.await` point, the OS-level mutex blocks the entire Tokio worker thread. Other tasks scheduled on that thread cannot run, including the future that would eventually release the lock. This creates a deadlock that is invisible to standard deadlock detectors because only one thread is blocked, not the OS-level mutex contention pattern.

**Evidence from Tokio Issue #2599**: `tokio::sync::Mutex` showed ~35% lower throughput vs `async_std::sync::Mutex` under 5000 connections because Tokio's async mutex uses an internal `std::sync::Mutex` to protect its waker list. But the real danger is the *opposite* direction: using `std::sync::Mutex` in async code at all.

**Evidence from Stack Overflow (5k views)**: Classic repro — two tasks join on same `std::sync::Mutex`, one holds lock across `delay_for().await`, second blocks the thread permanently. Output: `lock, locked, lock` then hangs forever.

**NeoTrix Exposure**: The codebase has **100+ `std::sync::Mutex`** and **100+ `std::sync::RwLock`** usages across:
- `nt_core_event_bus.rs` — 8+ `std::sync::Mutex` instances (handles, hooks, sync_handlers, log_file)
- `nt_memory_kb/mod.rs` — SQLite `Connection` wrapped in `std::sync::Mutex`
- `nt_core_bank/` — multiple `std::sync::RwLock` for bank state
- `nt_shield/guard.rs`, `safety_kernel.rs`, `permissions.rs` — security-critical paths

Any of these held across an `.await` in async context will deadlock. Clippy lint `await_holding_lock` should be enforced as deny-level.

**Recommendation**:
1. Run `cargo clippy -- -W clippy::await_holding_lock` across entire workspace
2. Audit every `std::sync::Mutex` in async modules — convert to `tokio::sync::Mutex` if guard crosses `.await`, or scope the guard to drop before `.await`
3. Add CI gate: `clippy::await_holding_lock` must pass with zero warnings

---

## Defect D-885-2: `tokio::sync::Semaphore` Cancel-Unsafe — FIFO Queue Position Lost

**Severity**: High
**Category**: Cancellation Safety / Fairness Violation

**Description**: `Semaphore::acquire()` uses a FIFO queue for fairness. However, cancelling a call to `acquire` (e.g., via `tokio::select!` timeout or task drop) silently removes the task from the queue. The task loses its position entirely. Under sustained load, a repeatedly-cancelled task can starve indefinitely — it re-enters the back of the queue each time, never acquiring a permit.

**Evidence from Tokio Docs**: "Cancelling a call to `acquire` makes you lose your place in the queue." This is explicitly documented but counter-intuitive.

**Evidence from Tokio Issue #6450**: When `tokio::select!` creates multiple waiters for the same semaphore from one task, the wake order becomes undefined. A permit released to the first waiter may be shadowed by the second waiter's registration, causing deadlock even though the semaphore has available permits.

**NeoTrix Exposure**:
- `nt_world_osint/person.rs:105` — `Semaphore::new(config.concurrency.max(5))` used for crawl concurrency. If OSINT tasks use `select!` with timeouts (common for network requests), cancelled tasks lose queue position, causing effective concurrency to drop below configured limit.
- `nt_mind_eval_harness.rs:18` — Semaphore for evaluation harness parallelism. Eval tasks with timeouts will lose position on cancellation.
- `nt_io_provider/free_catalog.rs:80` — Semaphore for free provider rate limiting.

**Recommendation**:
1. For timeout-protected semaphore acquisitions, use `tokio::time::timeout` *outside* the semaphore acquire, not wrapping it with `select!`
2. Consider `try_acquire_owned()` for load-shedding instead of `select!`-based cancellation
3. Add metric tracking: count cancelled acquire attempts vs successful acquisitions
4. Document the FIFO loss behavior in NeoTrix sync primitives guide

---

## Defect D-885-3: `tokio::sync::RwLock` Write-Preferring Policy Causes Reader Starvation Under Write Surge

**Severity**: High
**Category**: Performance Degradation / Liveness

**Description**: Tokio's `RwLock` is intentionally write-preferring (fair) to prevent writer starvation. However, this means a sustained burst of write requests blocks ALL new read locks — even when the lock is currently held by readers. Any new `read()` call queued behind a pending `write()` will block until that write completes, even if multiple readers could safely proceed concurrently.

**Evidence from Tokio Docs**: "A read lock will not be given out until all write lock requests that were queued before it have been acquired and released." This is FIFO-fair but can cause throughput collapse in read-heavy workloads with periodic writes.

**Evidence from Tokio Source**: `rwlock.rs` uses `MAX_READS: u32 = u32::MAX >> 3` as the limit. Write-preferring means: once a writer is in the queue, ALL subsequent readers block, even if current readers are active.

**NeoTrix Exposure**: NeoTrix has **35+ `tokio::sync::RwLock`** instances across:
- `nt_world_crawl/ordered_backend_router/mod.rs` — search backend routing, read-heavy
- `nt_shield_stealth_net/` — 12+ RwLock instances (proxy_control, proxy_heartbeat, network_pool, ip_rotator, rule_api, etc.)
- `nt_shield_proxy_kernel/` — router.rs, security.rs, fakeip.rs, dns_intercept.rs
- `nt_core_gwt/expert_store.rs`, `lease_manager.rs` — GWT attention routing
- `nt_mind_skill_engine.rs` — skill engine state

Under crawl-heavy workloads, periodic writes to proxy rotation state or rule engine updates can block all read-path routing, causing cascading latency spikes.

**Recommendation**:
1. Profile RwLock contention under load for `ordered_backend_router` and `proxy_control`
2. For read-dominant paths (proxy_heartbeat, network_pool reads), consider lock-free alternatives or `std::sync::RwLock` if no `.await` crossing
3. Implement write batching: accumulate multiple small writes, flush periodically under single write lock
4. Add contention metrics: track `read()` and `write()` wait times via instrumented wrappers

---

## Defect D-885-4: Tokio Mutex Internal `std::sync::Mutex` — Double-Lock Overhead

**Severity**: Medium
**Category**: Performance / Architecture

**Description**: `tokio::sync::Mutex` internally uses a `std::sync::Mutex` to protect its waker list. This means every `lock().await` call involves TWO lock acquisitions: the async-aware lock for the permit, and the internal sync mutex for waker management. Under high contention, this double-locking creates measurable overhead.

**Evidence from Tokio Discussion #7627**: "The async Mutex uses the sync Mutex internally to protect the waker list. So if the lock contention is not significant, the sync Mutex is usually faster."

**Evidence from Tokio Issue #2599**: Benchmark showed `tokio::sync::Mutex` at 51k req/s vs `async_std` at 84k req/s under 5000 connections. The internal sync mutex is the bottleneck.

**NeoTrix Exposure**: 66 `tokio::sync::Mutex` instances across the codebase. Many are used for data-only state that never crosses `.await`:
- `nt_shield_traffic/mod.rs`, `api_proxy.rs`, `mitm.rs` — traffic state
- `nt_io_neocodex.rs` — codex state
- `nt_io_mail/self_heal.rs:1064` — call counter
- `nt_core_self_test.rs` — test environment lock

Every unnecessary `tokio::sync::Mutex` pays double-lock overhead. The Tokio docs explicitly state: "If the value behind the mutex is just data, it's usually appropriate to use a blocking mutex such as the one in the standard library or parking_lot."

**Recommendation**:
1. Audit all 66 `tokio::sync::Mutex` usages — convert to `std::sync::Mutex` (or `parking_lot::Mutex`) where guard never crosses `.await`
2. Keep `tokio::sync::Mutex` only for: shared IO resources (DB connections), locks genuinely held across `.await`
3. Estimated performance gain: 20-35% for uncontended lock paths

---

## Defect D-885-5: PausableFuture Pattern Creates Phantom Deadlock — Mutex Appears Unlocked But Future Hangs

**Severity**: Critical
**Category**: Deadlock / Cancellation Safety

**Description**: When a custom wrapper (like `PausableFuture`) stops polling a future that holds a Tokio semaphore permit, the permit is never released. The mutex/semaphore appears unlocked from the outside (the permit was issued), but the future holding it is paused and never polled again. Other futures waiting for the permit deadlock permanently.

**Evidence from e6data Case Study (March 2026)**: Production DataFusion query engine — 4 futures compete for one `tokio::sync::Mutex`. Three complete, fourth hangs forever. Logs showed lock released, but no future acquired it. Root cause: `PausableFuture` wrapper stopped polling the inner future, which held a semaphore permit (the mutex is built on a semaphore internally). The permit was never returned.

**Coffman Conditions Analysis**: Standard deadlock analysis fails here because no thread holds the lock — the future simply stopped being polled. This is a "phantom deadlock" invisible to traditional tools.

**NeoTrix Exposure**: NeoTrix uses `select!` extensively across its async infrastructure. Any pattern that wraps futures in cancellation-aware combinators (timeout, select, biased) while holding Tokio sync primitives is at risk. Specific risk areas:
- `nt_shield_stealth_net/` — proxy management with timeouts
- `nt_world_crawl/` — crawl operations with per-request timeouts
- `nt_io_provider/` — LLM provider calls with timeout fallback
- `nt_mind_background_loop/` — background task orchestration

If any NeoTrix future pattern pauses polling (via `select!` branches or custom combinators) while holding a Tokio Mutex/Semaphore permit, the same phantom deadlock occurs.

**Recommendation**:
1. Ensure all futures holding Tokio sync permits are never paused without being dropped (dropping returns the permit)
2. Use `tokio::select!` only with cancel-safe branches, or ensure the held-permit branch is always the one that completes
3. Add lint: flag any `select!` that has a branch acquiring a Tokio sync primitive
4. Document the "phantom deadlock" pattern in NeoTrix async guidelines
5. Consider adding a runtime debug assertion that tracks permit holders and detects long-held permits

---

## Defect D-885-6: Semaphore `forget_permits` / `add_permits` Race with Waiter Queue

**Severity**: Medium
**Category**: Race Condition / State Inconsistency

**Description**: `Semaphore::forget_permits(n)` permanently reduces the semaphore's capacity, while `add_permits(n)` increases it. When called concurrently with active waiters, the permit accounting can become inconsistent: a waiter may be woken for a permit that was simultaneously forgotten by another thread, causing the semaphore to have fewer available permits than expected.

**Evidence from Tokio Semaphore Semantics**: `forget_permits` returns the actual number of permits reduced (may be less than `n` if insufficient). But between the atomic decrement and the waiter notification, another thread can `add_permits`, creating a window where the permit count is transiently incorrect.

**NeoTrix Exposure**: If NeoTrix uses dynamic semaphore sizing (adjusting crawl concurrency, API rate limits, or connection pool sizes at runtime), concurrent `add_permits`/`forget_permits` calls could cause:
- Temporary over-admission (more concurrent tasks than intended)
- Temporary under-admission (fewer permits than configured)

This is particularly dangerous for NT-SHIELD rate limiting and NT-WORLD crawl throttling.

**Recommendation**:
1. Guard `add_permits`/`forget_permits` calls behind a Mutex to serialize capacity changes
2. Use `available_permits()` as advisory only, never for critical admission decisions
3. For dynamic capacity management, use `Semaphore::close()` + create new Semaphore pattern
4. Add metrics: track `available_permits()` over time to detect transient inconsistencies

---

## Defect D-885-7: `std::sync::RwLock` Poisoning in Multi-Threaded Test Infrastructure

**Severity**: Medium
**Category**: Error Handling / Test Reliability

**Description**: `std::sync::RwLock` (and `Mutex`) become "poisoned" when a thread panics while holding the lock. All subsequent `lock()` calls return `Err(PoisonError)`. In NeoTrix's test infrastructure, a single panicking test can poison shared lock state, causing all subsequent tests to fail with opaque `PoisonError` messages.

**Evidence from Rust Issue #149359 (Nov 2025)**: Active discussion on replacing poisoning locks with non-poisoning versions in Rust Edition 2027. "Without poisoning you are way too likely to keep limping around without making any forward progress ever again when an essential background thread panicked." — bjorn3. Counter: "Having posioning locks tends to be a safer default" — davepacheco.

**NeoTrix Exposure**:
- `nt_core_self_test.rs:11` — `pub static TEST_ENV_LOCK: std::sync::Mutex<()>` — if any test panics while holding this, all subsequent tests deadlock on poisoned lock
- `nt_core_consciousness_core.rs:1935` — `static LOCK: std::sync::Mutex<()>` — same pattern
- `nt_core_bank/` — multiple `std::sync::RwLock` for persistent state; panic during bank operation poisons the lock, preventing recovery

**Recommendation**:
1. Use `lock().unwrap_or_else(|e| e.into_inner())` pattern to recover from poisoning in non-critical paths
2. For test infrastructure, use `std::sync::Mutex::new(()).into()` pattern or `parking_lot` (no poisoning)
3. Consider `parking_lot::Mutex`/`RwLock` globally — zero-cost, no poisoning, faster
4. Add panic guard in `drop()` for critical locks to avoid recursive poisoning

---

## Defect D-885-8: Mixed `std::sync` and `tokio::sync` in Same Data Structure

**Severity**: Medium
**Category**: Architecture / Maintainability

**Description**: NeoTrix mixes `std::sync::Mutex` and `tokio::sync::Mutex` protecting the same logical subsystem. This creates confusion about lock semantics: developers must track which locks block threads vs yield, which are `Send` vs `!Send`, and which can cross `.await` points. The risk of accidentally holding a `std::sync` guard across `.await` increases when both types are present in the same module.

**NeoTrix Exposure**:
- `nt_core_event_bus.rs` — uses `std::sync::Mutex` for handles/hooks but is called from async contexts
- `nt_memory_kb/` — `std::sync::Mutex<Connection>` + `std::sync::RwLock` for BM25/embedding alongside `tokio::sync` elsewhere
- `nt_shield/` — mix of `std::sync::Mutex` (guard, permissions) and `tokio::sync::RwLock` (proxy control)

**Recommendation**:
1. Establish clear policy: async modules use `tokio::sync`, sync-only modules use `std::sync` (or `parking_lot`)
2. Never mix both in the same struct — if struct has async methods, all its locks should be `tokio::sync`
3. Document the policy in `dev-rules.md`
4. Add clippy configuration: `disallowed_types = ["std::sync::Mutex"]` for async modules

---

## Summary Table

| ID | Defect | Severity | Root Cause | NeoTrix Impact |
|----|--------|----------|------------|----------------|
| D-885-1 | std Mutex held across `.await` | Critical | Thread starvation deadlock | 100+ std Mutex instances in async code |
| D-885-2 | Semaphore cancel-unsafe FIFO loss | High | `select!` drops queue position | OSINT crawl, eval harness, provider rate limiting |
| D-885-3 | RwLock write-preferring reader starvation | High | FIFO fairness blocks reads on write surge | 35+ RwLock instances, crawl/proxy heavy |
| D-885-4 | Tokio Mutex double-lock overhead | Medium | Internal std Mutex for waker list | 66 tokio::sync::Mutex, many unnecessary |
| D-885-5 | PausableFuture phantom deadlock | Critical | Permit held but future not polled | select!-based timeout patterns across codebase |
| D-885-6 | Semaphore add/forget race | Medium | Concurrent capacity mutation | Dynamic crawl/rate-limit sizing |
| D-885-7 | std RwLock poisoning in tests | Medium | Panic poisons shared test lock | TEST_ENV_LOCK, bank state |
| D-885-8 | Mixed sync primitives | Medium | Inconsistent lock semantics | Event bus, KB, shield modules |

---

## Sources

1. Tokio docs: `tokio::sync::Mutex`, `RwLock`, `Semaphore` — tokio.rs
2. Tokio Issue #2599: "Why tokio::sync::Mutex has poor performance" (2020)
3. Tokio Issue #5176: "Possible reasons for deadlock of tokio::sync::Mutex" (2022)
4. Tokio Issue #6450: "Semaphore deadlock with select!" (2024)
5. Tokio Discussion #7627: "Why std::sync::Mutex preferred over tokio::sync::Mutex"
6. Rust Issue #149359: "Edition 2027: Consider replacing poisoning locks" (2025)
7. e6data Blog: "The Lock Nobody Held: Deadlocking a Tokio Mutex Without Holding a Lock" (March 2026)
8. Oxide RFD-0400: "Dealing with cancel safety in async Rust"
9. async-lock crate docs: Servo/async-lock relationship with std::sync
10. Clippy lint: `await_holding_lock`, `await_holding_invalid`
11. TheLinuxCode: "Mutex vs Semaphore: How I Choose in 2026"
12. Stanza courses: Async vs Std Mutex patterns
13. NeoTrix codebase grep: 66 tokio::sync + 100 std::sync instances
