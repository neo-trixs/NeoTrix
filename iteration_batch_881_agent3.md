# Iteration Batch 881 — Agent 3: Deep Research on Rust Async Synchronization

**Date:** 2026-09-07
**Research Scope:** `tokio::sync::Mutex`, `tokio::sync::RwLock`, `tokio::sync::Semaphore` — defects, anti-patterns, and NeoTrix applicability
**Sources:** Tokio docs.rs, GitHub issues (#2479, #4246, valkey-glide#5450), async-concurrency code review references, production Rust patterns (StoneMQ, Qdrant)

---

## 1. Defect: `std::sync::Mutex` Blocking Tokio Runtime in Async Hot Paths

**Severity:** Critical
**Source:** valkey-io/valkey-glide#5450 (2026-02-26), openclaw/async-concurrency.md
**Pattern:** `std::sync::Mutex` or `std::sync::RwLock` used inside async functions that are called from Tokio worker threads. The OS-level lock blocks the entire worker thread, starving the event loop. Under high concurrency, operations get stuck for seconds.

**Root Cause:** `std::sync::Mutex::lock()` parks the OS thread. If the lock is contended, the Tokio worker thread cannot service other tasks on that thread. This is distinct from `tokio::sync::Mutex` which yields to the runtime.

**NeoTrix Exposure:**
- `nt_shield_approval/confirmation_gate/mod.rs` uses `tokio::sync::RwLock` — correct, but audit should verify no `std::sync` locks appear in async paths across the 65+ files importing tokio sync primitives.
- The `audit.rs` regex at `nt_shield/audit.rs:185` already scans for `Arc<Mutex<` and `Arc<RwLock<` patterns — good defense, but should be extended to detect `std::sync::Mutex` vs `tokio::sync::Mutex` disambiguation.

**Recommendation:**
- Add a Clippy lint `clippy::disallowed_types` to ban `std::sync::Mutex` and `std::sync::RwLock` in all `async fn` paths.
- Exception: `std::sync::Mutex` is acceptable for very short critical sections (field reads, no `.await` inside guard scope) — Tokio docs explicitly endorse this. Enforce via review rule, not blanket ban.

---

## 2. Defect: Holding `MutexGuard` Across `.await` Points

**Severity:** High
**Source:** openclaw/async-concurrency.md §2, Tokio docs
**Pattern:** A `MutexGuard` (whether `std::sync` or `tokio::sync`) is held across an `.await` point. With `tokio::sync::Mutex`, this is allowed but dangerous — it prevents the lock from being released during suspension, enabling deadlocks if the suspended task holds the only token needed by another task.

```rust
// DEADLOCK-PRONE:
let guard = state.lock().await;
let data = fetch_from_db().await; // guard still held during suspension
guard.update(data);
```

**NeoTrix Exposure:**
- NeoTrix has 65+ files importing `tokio::sync::{Mutex,RwLock,Semaphore}`. Any guard held across `.await` in the `nt_core_data_pipeline`, `energy_core`, or `nt_shield_stealth_net` modules could deadlock the runtime.
- `resource_registry.rs` uses `RwLock::write().await` in `register()` and `RwLock::read().await` in `get()` — these are short-lived and safe, but the pattern should be verified across all 65+ usages.

**Recommendation:**
- Enforce: guards must be scoped to the minimum block. Use `{ let guard = lock.lock().await; ... }` to ensure drop before any subsequent `.await`.
- Add a `tokio-console` integration or a custom tracing subscriber that detects when a `MutexGuard` outlives an `.await` (Tokio's `tokio_unstable` feature can help).
- Consider `Mutex::lock_owned()` when the guard must cross `.await` boundaries (it returns an owned guard that can be moved into spawned tasks).

---

## 3. Defect: Semaphore Permit Lifetime / `Send` Bound Violations with `acquire()` vs `acquire_owned()`

**Severity:** Medium
**Source:** amritsingh183 Semaphore guide, Tokio docs
**Pattern:** Using `sem.acquire().await` (borrowed permit) in code that spawns tasks via `tokio::spawn`. The permit borrows from the semaphore — if the semaphore is stack-local, the permit cannot cross `.await` into a spawned task.

```rust
let sem = Semaphore::new(5);
let permit = sem.acquire().await.unwrap();
tokio::spawn(async move {
    // ERROR: permit borrows from `sem`, not 'static
    drop(permit);
});
```

**NeoTrix Exposure:**
- `free_catalog.rs:85`: `sem.clone().acquire_owned().await.ok()` — **correctly uses `acquire_owned`** with `Arc<Semaphore>`. However, the permit is stored as `Option<OwnedSemaphorePermit>` and moved into the spawned task. If the `.ok()` converts `None` to `None`, the task runs without a permit — defeating concurrency limiting.
- `nt_mind_eval_harness.rs` imports `Semaphore` but the actual usage pattern needs verification — ensure `acquire_owned()` is used when permits cross task boundaries.

**Recommendation:**
- Mandate: if a semaphore permit crosses a `.await` or enters a `tokio::spawn`, use `acquire_owned()` with `Arc<Semaphore>`.
- For `try_acquire()` / `try_acquire_owned()`: handle the `NoPermits` case explicitly (load shedding), don't silently discard.
- Add `Semaphore::new(N)` initialization lint to ensure N matches actual bottleneck (DB pool size, connection limit, GPU slot count).

---

## 4. Defect: Tokio Async Mutex/RwLock Performance Overhead Under High Contention

**Severity:** Medium
**Source:** tokio-rs/tokio#2479, StoneMQ design lessons, RustFromZeroToHero RwLock comparison
**Pattern:** `tokio::sync::Mutex` and `tokio::sync::RwLock` use a **fair FIFO queue** internally. This fairness guarantee comes at significant cost — up to 3.6x slower than `parking_lot::Mutex` in microbenchmarks with tiny critical sections. Tokio's maintainer explicitly states: "if you have such a small critical section and no async work with the lock held, you will be best served with `parking_lot::RwLock`."

**NeoTrix Exposure:**
- NeoTrix uses `tokio::sync::RwLock` in **30+ modules** (energy_core, nt_shield_stealth_net, nt_world_osint, etc.). Many of these are read-heavy caches (e.g., `ResourceRegistry.pools`, `firewall.rs` rules, `geo_proxy.rs` state).
- For read-heavy workloads (>90% reads), `tokio::sync::RwLock` is justified. For write-heavy or tiny critical sections, it introduces unnecessary overhead.
- The `resource_registry.rs` registers pools rarely but reads frequently — `RwLock` is appropriate here.

**Recommendation:**
- Audit lock contention: identify critical sections that are < 1μs and use no `.await` inside the guard. Replace with `parking_lot::RwLock` or `parking_lot::Mutex` for those paths.
- For NeoTrix specifically: the `nt_core_data_pipeline`, `energy_field`, `consciousness_tree` modules have hot-path reads. Consider `arc-swap` (lock-free read, atomic swap) for these read-heavy caches.
- Document the decision matrix: `tokio::sync::Mutex` when guard crosses `.await`, `parking_lot::Mutex` for short non-async critical sections, `tokio::sync::RwLock` for read-heavy async, `parking_lot::RwLock` for read-heavy non-async.

---

## 5. Defect: `RwLock` Upgrade-Downgrade Deadlock (Read → Write Upgrade)

**Severity:** High
**Source:** Tokio RwLock docs, RustFromZeroToHero
**Pattern:** Holding a read guard and attempting to acquire a write guard on the same `RwLock` without releasing the read guard first. Tokio's `RwLock` is **write-preferring** — a pending write blocks new reads. This creates a deadlock:

```
Task A: holds read_lock → attempts write_lock (blocks, waiting for existing reads to finish)
Task B: holds read_lock → attempts write_lock (blocks, same reason)
→ Both wait for each other's read locks to release → deadlock
```

**NeoTrix Exposure:**
- Any module that does `let r = lock.read().await; ... r.some_check() ... let mut w = lock.write().await;` without dropping `r` first is vulnerable.
- `energy_core/consciousness_tree.rs`, `energy_core/core.rs`, `gwt_router.rs` all use `RwLock` and have complex state mutation patterns.
- `nt_shield_stealth_net/` modules (12+ files) use `RwLock` for proxy/firewall state — if any reads a rule then tries to update, this pattern is at risk.

**Recommendation:**
- Ban in-memory read-to-write upgrades. Use `try_write()` after dropping read guard, or restructure to acquire write lock first.
- Add a lint/audit check: scan for `.read().await` followed by `.write().await` in the same function without an intervening drop.
- Use `RwLock::downgrade()` (if available) or explicit scope blocks to ensure read guards are dropped before write acquisition.

---

## 6. Defect: Semaphore Never Closed → Infinite Wait / Resource Leak

**Severity:** Medium
**Source:** Tokio Semaphore docs, amritsingh183 Semaphore guide
**Pattern:** `Semaphore` is created but `close()` is never called. If all permits are acquired by tasks that panic or are cancelled without dropping permits, remaining tasks wait forever.

```rust
let sem = Arc::new(Semaphore::new(5));
// Spawn 10 tasks, each acquires a permit
// If 5 tasks panic with permits held, the other 5 wait forever
```

Tokio's semaphore is fair and RAII-based — permits are released on drop. But if a task panics while holding a permit (and the panic isn't caught), the permit is still released. The real risk is **logical leaks**: a task holds a permit while doing unbounded work (e.g., retrying a network call forever).

**NeoTrix Exposure:**
- `free_catalog.rs:82-87`: semaphore with 8 permits for reachability checks. Tasks use `spawn_blocking` for `reqwest::blocking` — if a request hangs beyond the 5s timeout, the permit is held for the full duration. With 8 concurrent tasks, this could block the semaphore for 5s × 8 = 40s worst case.
- `nt_mind_eval_harness.rs`: semaphore for LLM evaluation concurrency — if a provider call hangs, the permit is held indefinitely.

**Recommendation:**
- Always wrap `acquire_owned()` with `tokio::time::timeout()` to set an upper bound:
  ```rust
  match timeout(Duration::from_secs(10), sem.acquire_owned()).await {
      Ok(Ok(permit)) => { /* got permit */ }
      Ok(Err(_)) => { /* semaphore closed */ }
      Err(_) => { /* timeout — permit acquisition too slow */ }
  }
  ```
- Ensure spawned tasks that hold permits have their own timeout on the inner work.
- Call `sem.close()` during shutdown to wake all waiters with `AcquireError::Closed` instead of hanging.

---

## 7. Defect: Unbounded Task Spawning Without Backpressure

**Severity:** Medium
**Source:** rustify.rs async guide 2026, openclaw/async-concurrency.md
**Pattern:** `tokio::spawn()` called in a loop without any concurrency limit. Each task allocates ~few hundred bytes of stack + heap. At 100K+ concurrent tasks, memory pressure becomes significant.

**NeoTrix Exposure:**
- `free_catalog.rs:84-86`: spawns a task per entry, but uses semaphore to limit to 8 concurrent — **correct**.
- Other modules (crawl pipelines, social intel, background loops) may spawn tasks without limits. The `nt_mind_background_loop` runs on a 60s tick and spawns handlers — if a handler itself spawns sub-tasks without limits, this could cascade.

**Recommendation:**
- Enforce: every `tokio::spawn()` in a loop must be paired with either a semaphore or a bounded channel for backpressure.
- Add `JoinSet` (Tokio 1.x) or `FuturesUnordered` for bounded concurrency instead of raw `tokio::spawn`.
- Monitor: use `tokio::runtime::Handle::runtime_metrics()` (with `tokio_unstable`) to track spawned task count.

---

## Summary: 7 Defects Extracted for NeoTrix

| # | Defect | Severity | Primary Module(s) |
|---|--------|----------|-------------------|
| 1 | `std::sync::Mutex` blocking Tokio runtime | Critical | All async hot paths |
| 2 | Guard held across `.await` → deadlock | High | energy_core, nt_shield, data_pipeline |
| 3 | Semaphore `acquire()` vs `acquire_owned()` `Send` violation | Medium | free_catalog, eval_harness |
| 4 | Tokio async lock overhead under contention | Medium | 30+ RwLock-using modules |
| 5 | RwLock read→write upgrade deadlock | High | consciousness_tree, stealth_net |
| 6 | Semaphore never closed / no timeout → infinite wait | Medium | free_catalog, eval_harness |
| 7 | Unbounded task spawning without backpressure | Medium | background_loop, crawl, social_intel |

## Recommended Actions

1. **Add Clippy lint** `clippy::disallowed_types` for `std::sync::Mutex` in async crates.
2. **Audit all 65+ files** importing `tokio::sync` — verify no guard-across-await patterns.
3. **Enforce timeout on all semaphore acquisitions** — `tokio::time::timeout` wrapper.
4. **Benchmark critical paths** — identify < 1μs critical sections using `parking_lot` instead of tokio async locks.
5. **Add `RwLock` upgrade detection** to the existing `audit.rs` regex scanner.
6. **Mandate `JoinSet` or semaphore** for all loop-based `tokio::spawn`.
7. **Document the lock decision matrix** in `dev-rules.md` for all contributors.
