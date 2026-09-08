# Agent 3: Async Synchronization (Batch 877)

## Sources

1. **Tokio Mutex docs** — docs.rs/tokio/latest/tokio/sync/struct.Mutex.html — FIFO fairness, no poisoning, cancel safety semantics, `blocking_lock` panic on async context
2. **Async Synchronization Primitives (rs4ts.dev)** — The "one rule": do you hold the guard across `.await`? std vs tokio lock selection; scope-based guard release; Semaphore for backpressure; RwLock writer starvation; `await_holding_lock` lint
3. **Mutex deadlock async Rust production (juanchi.dev)** — Three deadlock patterns: Classic Deadly Embrace, Reentrant Lock, Inverted Order Under Pressure; tokio-console diagnosis; timeout-based deadlock breaking
4. **The Lock Nobody Held (e6data.com)** — PausableFuture + tokio mutex deadlock: permit eagerly assigned to unpolled future, waker contract violation; fix: drop inner future or avoid pausing
5. **Semaphore deadlock Issue #6450 (tokio-rs/tokio)** — `tokio::select!` with RwLock creates shadow waiters; FIFO queue can deadlock when a single task has multiple waiter entries and the wrong one is woken
6. **DeepWiki: Mutex, RwLock, Semaphore (tokio)** — batch_semaphore internals: bit-packed permits, intrusive LinkedList, FIFO fairness, write-preferring RwLock, SemaphorePermit RAII
7. **Tokio Semaphore docs** — Cancel safety: cancelling `acquire` loses queue place; `acquire_many` blocks head-of-line; `close()` wakes all waiters with error
8. **Turso blog: Single Mutex deadlock** — std::sync::Mutex + spawn_blocking + block_on inside lock → deadlock; tokio::sync::Mutex fixes it
9. **Mutex vs tokio::sync::Mutex (atharvapandey.com)** — Performance: std is nanoseconds, tokio is slower; never use tokio unless `.await` inside lock; RwLock starvation warnings
10. **async-lock crate docs** — async-lock crate: use std when possible, async-lock only for `no_std` or across-await; heavier than std::sync

## Defects

D-SYNC-001: **Dual RwLock ordering inversion in FakeIpTable** — `get_or_insert` (fakeip.rs:63-64) acquires `domain_to_fake.write()` then `fake_to_domain.write()`. `cleanup_expired` (fakeip.rs:117-118) acquires `fake_to_domain.write()` then `domain_to_fake.write()`. Opposite ordering = guaranteed deadlock under concurrent get+cleanup. | `fakeip.rs:63-64,117-118` | **critical** | Source 3 (Inverted Order Under Pressure)

D-SYNC-002: **Triple RwLock in GWT router without lock ordering discipline** — `GWTRouterImpl` (gwt_router.rs:13-17) holds three separate `Arc<RwLock<...>>` fields: `subscribers`, `broadcast_history`, `attention_weights`. `route_wisdom` (gwt_router.rs:43-66) acquires `broadcast_history.write()` → `subscribers.read()` → `attention_weights.read()` sequentially. `subscribe` (gwt_router.rs:71) acquires `subscribers.write()`. No documented lock ordering invariant; any path acquiring two locks in different order deadlocks. | `gwt_router.rs:13-17,38-67` | **high** | Source 3 (Classic Deadly Embrace)

D-SYNC-003: **BackgroundLoop macro holds tokio::sync::Mutex for entire handler body** — The `spawn_handler!` macro (run.rs:745) acquires `h.lock().await` and holds the `BackgroundLoopHandle` lock for the entire duration of each handler (`$body`). With 20+ handlers sharing one `Arc<Mutex<BackgroundLoopHandle>>`, this serializes all background work. A slow handler (e.g., `handle_consolidate` which writes to brain + KB) blocks every other handler tick for its entire duration. | `run.rs:745,779-814` | **high** | Source 2 (Keep critical sections tiny)

D-SYNC-004: **std::sync::RwLock in ExpertStore and LeaseManager can panic under contention** — `ExpertStore` (expert_store.rs:12) and `LeaseManager` (lease_manager.rs:11) use `std::sync::RwLock` with `.unwrap()` on lock acquisition (lease_manager.rs:78,84,104,113,130). If any thread panics while holding the write lock, the lock poisons and all subsequent `.unwrap()` calls panic, cascading failure across the entire GWT attention routing system. No timeout or graceful degradation. | `expert_store.rs:12`, `lease_manager.rs:78` | **high** | Source 1 (No poisoning in tokio but std does poison), Source 4 (timeout-based deadlock breaking)

D-SYNC-005: **TorClient silently drops events via try_lock** — `push_event` (tor_client.rs:137) uses `self.event_sink.try_lock()` and silently drops the event if the lock is held. Under high event throughput (e.g., during circuit rotation or proxy failover), events are lost without any indication. This violates the event-sourcing contract for NT-SHIELD stealth monitoring. | `tor_client.rs:136-139` | **medium** | Source 2 (Semaphore for backpressure — should use channel or queue)

D-SYNC-006: **LeaseManager::clone() creates orphaned lease tracking** — `LeaseManager` implements `Clone` (lease_manager.rs:44-51) by copying the atomic counter but creating a fresh empty `HashMap`. The cloned instance has no knowledge of leases held by the original. If a cloned LeaseManager is used in a different task, `release()` on the clone silently returns `NotFound` while the original still tracks the lease, causing resource leak. | `lease_manager.rs:44-51` | **medium** | Source 3 (Reentrant Lock pattern — clones create implicit reentrancy)

D-SYNC-007: **Semaphore never used for NT-SHIELD resource bounding** — The codebase has zero `Semaphore::acquire` calls. NT-SHIELD modules (stealth_net, proxy_kernel, traffic) manage concurrent network connections, proxy pools, and DNS resolution without any concurrency bounding. Under load, unbounded concurrent connections can exhaust file descriptors or overwhelm proxy infrastructure. `tokio::sync::Semaphore` is available but unused. | (global) | **medium** | Source 7 (Semaphore for concurrency caps)

D-SYNC-008: **Shield audit regex false-flags atomics as race conditions** — The audit rule at `audit.rs:185` matches `AtomicU|std::sync::atomic|\.store\(|\.load\(` as "race-condition" with severity "high". Atomics are inherently race-free by design. This generates false positives on every safe atomic usage across the codebase (ExpertStore, LeaseManager, etc.), diluting real findings. | `audit.rs:183-188` | **low** | Source 10 (std::sync primitives are safe for non-await critical sections)

D-SYNC-009: **ConsciousnessTreeImpl write lock held across expensive computation** — `receive_wisdom` (consciousness_tree.rs:58-79) acquires `self.nodes.write().await` and then performs wisdom aggregation (iterating, summing, dividing) while holding the write lock. This blocks all reader tasks (e.g., GWT broadcast) for the duration of the computation. Should clone the node, release lock, compute, then re-acquire to write. | `consciousness_tree.rs:62-79` | **medium** | Source 2 (Compute outside the lock; lock only to read or commit)

D-SYNC-010: **No lock ordering registry across NT-SHIELD modules** — Multiple modules (fakeip, proxy_kernel, stealth_net, traffic) each independently use `tokio::sync::RwLock` or `tokio::sync::Mutex` with no global lock ordering policy. The codebase has no documented lock hierarchy, no `ordered_locks` utility, and no Clippy configuration for `deadlock_detector`. Cross-module lock acquisition (e.g., proxy_kernel calling into stealth_net) can deadlock unpredictably. | (cross-module: fakeip.rs, kernel.rs, security.rs) | **high** | Source 3 (Lock order review in code review), Source 10 (No runtime deadlock detection)

## Key Insights

1. **The fakeip.rs dual-lock inversion is the most critical finding**: Two maps protected by separate RwLocks are locked in opposite orders across two functions. This is a textbook deadlock that will manifest under concurrent DNS resolution + cache cleanup. Fix: always acquire `domain_to_fake` before `fake_to_domain`, or merge into a single RwLock-protected struct.

2. **BackgroundLoop serialization defeats async concurrency**: 20+ handlers sharing a single `Arc<Mutex<BackgroundLoopHandle>>` means the entire background system is effectively single-threaded per tick. A slow KB write or consolidation blocks plugin ticks, evolution ticks, and world sensing. Consider per-handler state or sharded locks.

3. **The audit tool overcounts atomics as race conditions**: The regex at `audit.rs:185` is too broad. Atomics (`AtomicU64::load/store`) are the *correct* lock-free pattern. Flagging them as "high severity race conditions" wastes review bandwidth. Should be scoped to `Arc<Mutex<` and `Arc<RwLock<` only.

4. **No semaphore-based backpressure anywhere**: NT-SHIELD manages network connections, proxy pools, and DNS resolution with zero concurrency bounds. Under adversarial conditions or high load, this can exhaust OS resources.

5. **std::sync::RwLock in hot paths (ExpertStore, LeaseManager)**: These use blocking locks with `.unwrap()` — no timeout, no poisoning recovery. A single panic poisons the lock and cascades failures through the GWT attention system. Tokio's async alternatives with `timeout()` would be more resilient.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Critical | 1 |
| High | 4 |
| Medium | 4 |
| Low | 1 |
| Sources consulted | 10 |
