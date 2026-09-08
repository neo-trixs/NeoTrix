# Iteration Batch 879 — Agent 3: Rust Async Synchronization Deep Research

**Date**: 2026-09-07
**Topic**: `std::sync::Mutex`, `tokio::sync::Mutex`, `tokio::sync::RwLock`, `tokio::sync::Semaphore` — defects & anti-patterns in NeoTrix

---

## 1. Key Findings: Async Sync Primitives Landscape (2026)

### Mutex Decision Matrix

| Primitive | Guard is `Send`? | Blocks thread? | Use across `.await`? | Relative Cost |
|-----------|-----------------|---------------|----------------------|---------------|
| `std::sync::Mutex` | **No** (`!Send`) | Yes — OS futex | **Never** | Lowest |
| `tokio::sync::Mutex` | **Yes** | No — cooperative yield | Yes (intended use) | ~2-3x higher |
| `parking_lot::Mutex` | **No** | Yes — OS futex | **Never** | Lower than std |

**Tokio docs are explicit**: "it is ok and often preferred to use the ordinary Mutex from the standard library in asynchronous code" — **if the guard does not cross `.await`**. The async mutex exists solely for the case where you *must* hold the lock across an await point (transactional semantics, TOCTOU avoidance).

### Semaphore Semantics

- `tokio::sync::Semaphore` is FIFO-fair (built on `batch_semaphore`).
- **Head-of-line blocking**: if `acquire_many(5)` is at the front of the queue and only 3 permits are available, a subsequent `acquire(1)` will block even though it could succeed.
- Permits can be moved between tasks via `acquire_owned()` — essential for `tokio::spawn` patterns.
- `Semaphore::close()` causes all pending `acquire` to fail with `TryAcquireError::Closed`.

### RwLock Anti-Patterns

- `std::sync::RwLock` across `.await` blocks the OS thread — **use `tokio::sync::RwLock`**.
- `tokio::sync::RwLock` writer starvation avoidance: writers are prioritized once a write is pending, but **readers holding the lock across `.await` can block writers indefinitely**.
- `std::thread::yield_now()` inside async context (on `try_write()` failure) is **wrong** — it blocks the OS thread. Use `tokio::task::yield_now().await` or `write().await`.

---

## 2. Defects Found in NeoTrix

### Defect 1: `std::sync::Mutex` Held Across `.await` in Background Handlers

**Severity**: HIGH
**Location**: `nt_mind_background_loop/run.rs:745` — `BackgroundLoopHandle` wrapped in `Arc<tokio::sync::Mutex>`, but inner `avatar_engine: Option<std::sync::Mutex<DistillationEngine>>` (run.rs:938, mod.rs:94)

**Problem**: `BackgroundLoopHandle` is behind `Arc<tokio::sync::Mutex>`. Every handler acquires `h.lock().await` to access the entire handle, including `avatar_engine` which is a `std::sync::Mutex`. If any handler does KB/LLM/network work *while holding the tokio lock* (which they do — each handler body does heavy I/O), the entire lock is held across multiple `.await` points. This serializes ALL 30+ background handlers into a single queue, defeating the concurrent spawn design.

**Evidence from prior batch** (iteration 870): "NeoTrix handlers acquire `tokio::sync::Mutex` locks (run.rs:745) and do KB/LLM/network work inside the lock. If any handler's work takes >500ms, it gets force-yielded while still holding the lock."

**Fix**: Reduce lock scope — extract the needed data, drop the guard, then do async work. Or shard the handle into independent `Arc<Mutex<>>` sub-structures so handlers don't contend on the same lock.

---

### Defect 2: `std::thread::yield_now()` Inside Async Context (Sync Yield in Async)

**Severity**: HIGH
**Location**: `CsgnWakeHook` handler — `nt_mind_skill_engine` skill activation path

**Problem**: Inside `CsgnWakeHook` (called from skill activation which runs in async context via hook registry), `std::thread::yield_now()` is used as backoff when `try_write()` fails on a `tokio::sync::RwLock`. This is a **sync thread yield**, not a cooperative future yield. It blocks the current Tokio worker thread for the OS scheduling quantum (~1-10ms on macOS). Under contention this degrades into thread starvation.

**Source**: iteration 861 agent2 found this: "std::thread::yield_now() is used as a backoff when try_write() fails on a tokio::sync::RwLock. This is a sync thread yield, not a cooperative future yield."

**Fix**: Replace with `tokio::task::yield_now().await` or simply call `write().await` (which yields cooperatively).

---

### Defect 3: Mixed `std::sync::Mutex` and `tokio::sync::RwLock` in Shared State Without Guarantees

**Severity**: MEDIUM
**Location**: Multiple files — `energy_core/core.rs`, `energy_field.rs`, `gwt_router.rs`, `resource_registry.rs`, `nt_mind_skill_engine.rs`, `confirmation_gate/mod.rs` all use `tokio::sync::RwLock`; while `nt_core_telemetry.rs`, `nt_core_span.rs`, `native_bus.rs`, `provider_pool.rs` use `std::sync::Mutex`.

**Problem**: No architectural rule enforces which primitive to use where. The codebase mixes both freely. `nt_core_router.rs:4` uses `std::sync::Mutex` for routing state — if any async code path touches this without dropping the guard first, it blocks the thread. `provider_pool.rs:251-252` uses `OnceLock<std::sync::Mutex<ProviderPool>>` as a global — if any async handler calls `global_provider_pool().lock().unwrap()` and does I/O, it blocks.

**Fix**: Establish a rule: all shared state accessed from async code must use `tokio::sync::Mutex` or `tokio::sync::RwLock`. `std::sync::Mutex` only for truly sync-only paths (e.g., static test locks, one-time init).

---

### Defect 4: Semaphore Head-of-Line Blocking in `free_catalog.rs`

**Severity**: LOW-MEDIUM
**Location**: `nt_io_provider/free_catalog.rs:80` — `reachable_subset()` uses `Semaphore::new(8)` with `acquire_owned()`.

**Problem**: The semaphore is acquired *before* spawning the task (`let permit = sem.clone().acquire_owned().await.ok()`), then the permit is moved into the task. This is correct but has a subtle issue: if `acquire_owned().await` fails (semaphore closed), `permit` is `None` and the task runs without a permit — **bypassing the concurrency limit silently**. The `.ok()` discards the `Closed` error.

**Current code**:
```rust
let permit = sem.clone().acquire_owned().await.ok(); // None on close
tasks.push(tokio::spawn(async move {
    let _permit = permit; // None = no limit!
```

**Fix**: Either propagate the error or add an explicit check: `if permit.is_none() { return (e, false); }` before the network probe.

---

### Defect 5: `std::sync::Mutex` Poisoning Not Handled in Static Locks

**Severity**: MEDIUM
**Location**: Multiple `static` mutex declarations:
- `nt_core_self_test.rs:11`: `pub static TEST_ENV_LOCK: std::sync::Mutex<()>`
- `nt_core_consciousness_core.rs:1935`: `static LOCK: std::sync::Mutex<()>`
- `nt_core_e8_predictor.rs:242`: `static LOCK: std::sync::Mutex<()>`
- `run.rs:1097`: `static LAST_ALERTS: std::sync::Mutex<Vec<TelemetryAlert>>`
- `run.rs:1114`: `static WARMUP: std::sync::Mutex<u32>`
- `cipher.rs:193`: `static HOME_LOCK: std::sync::Mutex<()>`

**Problem**: All use `.lock().unwrap()`. If any thread panics while holding one of these static locks, the mutex becomes poisoned and **all subsequent `.lock().unwrap()` calls panic** — cascading failures across the entire process. This is especially dangerous for `LAST_ALERTS` and `WARMUP` which are hit on every telemetry tick.

**Fix**: Use `.lock().unwrap_or_else(|e| e.into_inner())` for non-critical state, or `.lock().expect("mutex poisoned")` with recovery logic, or switch to `parking_lot::Mutex` which doesn't poison.

---

### Defect 6: `avatar_engine` Field Uses `std::sync::Mutex` in Async Context

**Severity**: MEDIUM
**Location**: `mod.rs:94`, `run.rs:938`: `avatar_engine: Option<std::sync::Mutex<DistillationEngine>>`

**Problem**: `avatar_engine` is a `std::sync::Mutex` stored inside `BackgroundLoopHandle` (which is behind `Arc<tokio::sync::Mutex>`). Any handler that accesses `h.avatar_engine` while holding the tokio lock could call `.lock().unwrap()` on the std mutex — and since the tokio lock is already held (across `.await`), the std mutex guard would also be held across `.await`, blocking the OS thread.

**Fix**: Change to `tokio::sync::Mutex<DistillationEngine>` or restructure to avoid needing the mutex at all (e.g., move the engine into its own task with message passing).

---

### Defect 7: No Lock Ordering Enforcement — Potential Deadlock

**Severity**: HIGH
**Location**: Cross-module — multiple handlers access `brain` (`Arc<RwLock<SelfIteratingBrain>>`), `bbrain` (`Arc<RwLock<BMonitor>>`), and various `std::sync::Mutex` fields.

**Problem**: There is no documented or enforced lock ordering. If handler A locks `brain` then `bbrain`, while handler B locks `bbrain` then `brain`, a deadlock occurs. Tokio's async mutex deadlocks are harder to diagnose than sync ones because the Coffman conditions don't fully apply — futures can be dropped/paused without releasing locks (e6data 2026 research).

**Evidence**: Tokio mutex can deadlock **without holding a lock** — if a future is paused (e.g., by `select!` cancellation or task suspension), its waker is never polled again, so it never releases the lock. This is a known Tokio issue (e6data, March 2026).

**Fix**: Document a global lock ordering. Use `try_lock()` with timeout and retry, or restructure to eliminate nested lock acquisition.

---

### Defect 8: `blocking_lock()` Called Inside Async Context

**Severity**: LOW
**Location**: Tokio docs warn: "This function panics if called within an asynchronous execution context."

**Problem**: `tokio::sync::Mutex::blocking_lock()` is designed for sync code that needs to use an async mutex. If accidentally called inside an async function, it panics. While no direct evidence of this in NeoTrix yet, the `AvatarEngine` pattern (std::sync::Mutex in async context) suggests this trap is one refactor away.

**Fix**: Add clippy lint `clippy::await_holding_lock` or use `tokio::task::spawn_blocking` if sync lock acquisition is needed.

---

## 3. Recommended Fixes Summary

| # | Defect | Severity | Fix Complexity | Action |
|---|--------|----------|---------------|--------|
| 1 | Background handler lock scope too wide | HIGH | High | Shard `BackgroundLoopHandle` into independent sub-locks |
| 2 | `std::thread::yield_now()` in async | HIGH | Low | Replace with `tokio::task::yield_now().await` |
| 3 | Mixed std/tokio primitives, no rule | MEDIUM | Medium | Establish architectural rule, grep audit |
| 4 | Semaphore `.ok()` silently bypasses limit | LOW-MED | Low | Propagate error or guard with `if permit.is_none()` |
| 5 | Static `std::sync::Mutex` poisoning | MEDIUM | Low | Switch to `parking_lot` or `unwrap_or_else` |
| 6 | `avatar_engine` std::sync::Mutex in async | MEDIUM | Medium | Convert to `tokio::sync::Mutex` or message passing |
| 7 | No lock ordering enforcement | HIGH | High | Document lock hierarchy, add `try_lock` timeouts |
| 8 | Potential `blocking_lock()` in async | LOW | Low | Add clippy lint guard |

---

## 4. References

- Tokio docs: `tokio::sync::Mutex` — "it is ok and often preferred to use the ordinary Mutex" (2026)
- e6data: "Deadlocking a Tokio Mutex without Holding a Lock" (March 2026)
- Microsoft RustTraining: "Common Pitfalls — Holding MutexGuard Across .await"
- TheLinuxCode: "Mutex vs Semaphore: How I Choose in 2026"
- Qovery: "Common Mistakes with Rust Async" (2025)
- iteration_batch_861_agent2: `std::thread::yield_now()` in CsgnWakeHook
- iteration_batch_870_agent1: Budget forced yields + tokio::sync::Mutex lock scope
