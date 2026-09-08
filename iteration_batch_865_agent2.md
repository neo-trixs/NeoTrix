# Agent 2: Concurrent Data Structures (Batch 865)

## Sources
1. DashMap 6.2.1 docs — sharded concurrent HashMap, per-shard RwLock, 7.0.0-rc2 stalled
2. arc-swap 1.9.2 docs — atomic Arc pointer swap, SeqCst ordering, Guard-based lifetime
3. parking_lot 0.12.5 benchmarks — 1-byte Mutex, fairness timer, std starvation under hog scenario
4. crossbeam-epoch 0.9.18 — epoch-based GC for lock-free structures, pin/unpin lifecycle
5. CAS pattern guide (stanza.dev) — ABA problem, compare_exchange_weak, tagged pointers
6. minikin.me ABA series — tagged pointers, epoch-based reclamation, hazard pointers
7. mutex-benches (cuongleqq) — std vs parking_lot: 95% starvation variance under contention
8. arc-swap patterns docs — Cache for delayed reclamation, DynAccess projections
9. clashmap fork — DashMap fork removing unsafe, improving API safety

## Defects

D-CONC-001: **GWT Router broadcast_history Vec grows unbounded under write lock** | `gwt_router.rs:42-45` | High | arc-swap patterns docs (consistency snapshots)

The `route_wisdom()` method acquires a write lock on `broadcast_history` (a `Vec<Wisdom>`), pushes a clone, and truncates to 1000 entries via `remove(0)`. `Vec::remove(0)` is O(n) — shifting all elements left on every insert. Under high broadcast frequency this creates O(n²) total work. Additionally, the write lock is held during the entire truncation, blocking all concurrent readers (`get_broadcast_history`, `resonance_strength`). The arc-swap docs emphasize that read-heavy structures should use lock-free RCU-like patterns; here, a `VecDeque` with `ArcSwap<VecDeque<Wisdom>>` would give O(1) append, O(1) pop-front, and lock-free reads.

**Fix**: Replace `Vec` with `VecDeque` for O(1) front removal. Consider `ArcSwap<VecDeque<Wisdom>>` for lock-free read path (the history is read-heavy, write-light). Alternatively, use `crossbeam::queue::ArrayQueue<Wisdom>` with bounded capacity for automatic eviction.

D-CONC-002: **SkillRetriever double-lock on embeddings in retrieve()** | `skill_retrieval.rs:189,196` | High | DashMap cross-shard deadlock docs, parking_lot fairness benchmarks

The `retrieve()` method locks `self.embeddings` at line 189 (`self.embeddings.lock().unwrap()`) and again at line 196 (`self.embeddings.lock().unwrap()`). Since `Mutex` is not reentrant in Rust, the second lock will **deadlock** if somehow reached on the same thread (e.g., via recursive call or async task migration). Even if it doesn't deadlock in current code, the pattern of acquiring the same Mutex twice within a single function is a maintenance hazard — any refactor that moves code between locks risks deadlock. The parking_lot benchmarks show that even std Mutex can cause thread starvation under contention; double-locking amplifies this.

**Fix**: Restructure `retrieve()` to acquire the lock once, extract all needed data into owned types, then drop the guard before the reranker call. The current code already partially does this (lines 218-223) but the initial double-acquire at 189+196 is a latent deadlock.

D-CONC-003: **AccountPool uses std::sync::RwLock in sync context — blocks tokio runtime** | `account_pool.rs:93,115,420` | Medium | parking_lot mutex benchmarks, tokio Mutex overhead docs

`AccountPool.accounts` is `Arc<std::sync::RwLock<HashMap<String, AccountState>>>`. The `select()`, `acquire()`, `register()`, `quarantine()`, `record_success()`, `record_failure()`, `reconcile()`, and `evict_unhealthy()` methods all call `.write().unwrap_or_else(|e| e.into_inner())` — the `into_inner()` fallback silently recovers from poisoned locks but **does not restore consistency** of the inner HashMap. If a panic occurred while holding the write lock, the HashMap may be in an inconsistent state (e.g., partially updated entry). The `unwrap_or_else(|e| e.into_inner())` pattern trades data corruption for crash prevention. Additionally, std::sync::RwLock in async contexts blocks the tokio worker thread during contention.

**Fix**: Replace `std::sync::RwLock` with `tokio::sync::RwLock` if called from async, or `parking_lot::RwLock` if sync-only. Remove the `unwrap_or_else(|e| e.into_inner())` pattern — poisoned locks should be propagated as errors, not silently recovered. AccountPool is the NT-IO provider selection layer; silent corruption here means wrong provider selection under load.

D-CONC-004: **ConsciousnessTree node_count/total_strength/layer_distribution each acquire separate read lock** | `consciousness_tree.rs:88-112` | Medium | arc-swap consistency snapshots

Three independent methods (`node_count`, `total_strength`, `layer_distribution`) each acquire their own `read().await` on the same `Arc<RwLock<HashMap>>`. Any caller composing these (e.g., a health dashboard that wants node count + total strength + distribution) will hold three sequential read locks, during which a write lock can interleave and change the data between measurements. The arc-swap docs warn: "a query should be answered by a consistent version of data." A snapshot approach (load once, derive multiple metrics) would ensure consistency.

**Fix**: Add a `snapshot()` method that acquires one read lock and returns a `ConsciousnessSnapshot` struct containing `node_count`, `total_strength`, `layer_distribution`, and `nodes` — all from a single consistent point-in-time read. Consumers call `snapshot()` once instead of multiple individual methods.

D-CONC-005: **GWT Router attention_weights RwLock acquired per-call in resonance_strength** | `gwt_router.rs:128-134` | Medium | arc-swap Cache pattern, DashMap performance

`resonance_strength()` acquires a read lock on `attention_weights` to compute a product of weights. This is called in the hot attention-routing path. The arc-swap Cache pattern shows that for read-mostly data with rare writes (weights update via `set_attention_weight`), an `ArcSwap` with `Cache` handle gives ~5ns reads vs RwLock read which still involves atomic refcount bumps and potential contention on the write-biased side.

**Fix**: Replace `Arc<RwLock<HashMap<Layer, f64>>>` with `ArcSwap<HashMap<Layer, f64>>` for attention_weights. Reads become a single atomic load (no lock). Writes use `ArcSwap::store()`. This eliminates lock contention on the hot attention path entirely.

D-CONC-006: **CheckRegistry call_counter Mutex held across HashMap operations** | `check_registry.rs:234` | Low-Medium | parking_lot Mutex 1-byte vs std 40-byte

`CheckRegistry.call_counter` is `Arc<Mutex<HashMap<String, usize>>>`. The Mutex is 40+ bytes (std) vs 1 byte (parking_lot). Every security check invocation acquires this lock to increment a counter. Under high tool-call throughput (NT-ACT orchestration), this Mutex becomes a serialization point for all security checks. The parking_lot benchmarks show std Mutex has 95% starvation variance under contention.

**Fix**: Replace with `DashMap<String, AtomicUsize>` for per-key lock-free increments, or `parking_lot::Mutex` for 40× smaller footprint. For a simple counter, `AtomicUsize` with `fetch_add` in a `DashMap` eliminates the lock entirely.

D-CONC-007: **OrderedBackendRouter health_cache uses std RwLock — same poisoned-lock pattern** | `ordered_backend_router/mod.rs:71` | Low-Medium | parking_lot no-poisoning, account_pool pattern

`health_cache: Arc<RwLock<HashMap<BackendType, BackendHealth>>>` uses std `RwLock`. Backend health checks are async I/O operations. If a health-check task panics while holding the write lock, the lock is poisoned. Subsequent health reads will fail, causing the router to treat all backends as unhealthy (cascading failure). The ordered-backend router is the NT-WORLD search fallback chain; poisoning it blocks all external information retrieval.

**Fix**: Use `parking_lot::RwLock` (no poisoning) or `ArcSwap<HashMap<BackendType, BackendHealth>>` (lock-free reads for the hot path where most callers just read health to decide routing order).

D-CONC-008: **ConfirmationGate pending_requests has no TTL/eviction — unbounded memory growth** | `confirmation_gate/mod.rs:64` | Medium | crossbeam-epoch deferred reclamation, arc-swap patterns

`pending_requests: Arc<RwLock<HashMap<String, ConfirmationRequest>>>` stores requests that are never evicted. If a confirmation callback never responds (e.g., human operator disconnects), the request stays forever. The crossbeam-epoch pattern shows that deferred reclamation must eventually complete; indefinitely deferred resources are leaks. The HashMap grows monotonically with each unhandled confirmation.

**Fix**: Add TTL-based eviction: store `Instant` alongside each `ConfirmationRequest`. Run periodic `reconcile()` (like AccountPool) to remove stale entries. Alternatively, use `ArcSwap` for the read-heavy request lookup path and a background task for cleanup.

## Key Insights

1. **DashMap is underutilized**: NeoTrix has 25+ `Arc<RwLock<HashMap>>` locations that would benefit from DashMap's per-shard locking (10-40× contention reduction). DashMap 7.0.0-rc2 is stalled 14+ months; pin to 6.2.1 for production stability.

2. **arc-swap fits the read-heavy GWT pattern perfectly**: The attention routing system (GWT Router, ConsciousnessTree) is overwhelmingly read-heavy. `ArcSwap` provides ~5ns lock-free reads vs RwLock's atomic refcount + mutex overhead. The Cache pattern adds delayed reclamation for consistency.

3. **parking_lot starvation prevention is critical**: The mutex-benchmarks show std::Mutex causes 95% starvation variance under contention. NT-IO AccountPool and NT-SHIELD CheckRegistry are hot paths where fairness matters. parking_lot's 0.5ms fairness timer prevents thread monopolization.

4. **Poisoned-lock recovery (`into_inner()`) is a silent corruption vector**: AccountPool's `unwrap_or_else(|e| e.into_inner())` pattern silently accepts potentially inconsistent state after a panic. This is worse than crashing — it continues operating with corrupted data.

5. **Lock-free structures need memory reclamation**: crossbeam-epoch is already in Cargo.lock (transitive dep) but not directly used. For future lock-free data structures (CapabilityTree concurrent access), epoch-based reclamation or tagged pointers are necessary to prevent ABA problems.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Sources consulted | 9 |
| Unique files implicated | 7 |
| High severity | 2 |
| Medium severity | 4 |
| Low-Medium severity | 2 |
