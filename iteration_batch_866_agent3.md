# Agent 3: Async Synchronization (Batch 866)

## Sources
1. Tokio Mutex docs (docs.rs/tokio) — async vs std mutex decision rules, FIFO fairness, no poisoning
2. DeepWiki: tokio-rs/tokio Mutex/RwLock/Semaphore — batch_semaphore internals, FIFO wait queues, write-preferring RwLock
3. rs4ts.dev Async Synchronization Primitives — one-rule decision (hold guard across `.await`?), std::sync::Mutex preferred for short critical sections
4. Tokio tutorial: Shared State — std::sync::Mutex for data, tokio::sync for IO resources, message-passing preferred
5. Tokio discussion #7627 — std::sync::Mutex faster for non-contention paths, tokio Mutex ~3x overhead
6. Stanza Async Sync — blocking vs yielding semantics, RwLock for read-heavy, Semaphore for rate limiting
7. beagle-rust sync-primitives.md — ARC_MUTEX_FOR_COPY_TYPE, TOKIO_MUTEX_FOR_SHORT_SECTION, Notify lost-wakeup hazard
8. sota-async-concurrency rules/03-primitives.md — audit checklist: unbounded anything, RwLock upgrade deadlocks, Semaphore RAII release
9. tokio::sync::Notify docs — permit coalescing, enable() pattern for lost-wakeup prevention
10. Semaphore docs — fairness with acquire_many blocking single permits, close semantics
11. DeepWiki: Oneshot/Notify/Barrier — enable pattern for multi-consumer Notify

## Defects

**D-SYNC-001: tokio::sync::Mutex on data-only TrafficAnalyzer in hot request path**
`neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/api_proxy.rs:41` | MEDIUM | Source: beagle-rust TOKIO_MUTEX_FOR_SHORT_SECTION + Tokio docs "if the value behind the mutex is just data, use a blocking mutex"
`TrafficAnalyzer` is a pure-data statistics tracker (capture_request/capture_response are synchronous, no IO). The `tokio::sync::Mutex` is acquired on every HTTP request (lines 172, 297, 319, 327, 399) — up to 3 times per request. Tokio's async Mutex carries ~3x overhead over std::sync::Mutex due to task scheduling machinery. The critical section never crosses an `.await` boundary. Should use `std::sync::Mutex` or atomics for counters.

**D-SYNC-002: tokio::sync::Mutex on data-only TrafficAnalyzer in MITM proxy**
`neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:250` | MEDIUM | Source: Tokio tutorial "use std::sync::Mutex for just data"
Same pattern as D-SYNC-001. The `TrafficAnalyzer` is wrapped in `Arc<Mutex<TrafficAnalyzer>>` and locked inside `handle_connect`, `handle_connect_mitm`, and `handle_http` — all synchronous critical sections with no `.await` while guard is held. The MITM proxy handles high-throughput tunnel traffic; the async mutex overhead is unnecessary.

**D-SYNC-003: ProxyPool holds 8 independent RwLock fields with no lock ordering**
`neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/proxy_pool.rs:183-198` | HIGH | Source: sota-async-concurrency "Pool-within-pool is a classic production deadlock"
`ProxyPool` has 8 RwLock-wrapped fields (nodes, subscriptions, strategy, rr_idx, learner, broker_scores, relay_descriptors, lease_tokens). Methods like `select_node_for_host` acquire `strategy.read()` then `learner.read()` sequentially. `heal_if_needed` acquires `nodes.read()` then `subscriptions.read()`. There is no documented lock acquisition order. Under concurrent access, task A holding nodes→subscriptions and task B holding subscriptions→nodes will deadlock. The FIFO fairness of tokio::RwLock does not prevent cross-lock deadlocks.

**D-SYNC-004: TorCrawler dequeue() holds write lock during O(n log n) sort**
`neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/tor_crawler.rs:97-108` | MEDIUM | Source: beagle-rust "keep critical sections tiny"
`dequeue()` acquires a write lock on the entire queue, then drains all items into a Vec, sorts them by priority, and pushes them back. With a large queue (MAX_QUEUE_SIZE is likely hundreds/thousands), this O(n log n) sort under write lock blocks all other tasks (enqueue, queue_len, save_queue) from accessing the queue. Should sort only when popping (use a priority queue data structure) or extract-and-sort in a tighter scope.

**D-SYNC-005: TorCrawler load_state() silently ignores try_read/try_write failures**
`neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/tor_crawler.rs:270-298` | HIGH | Source: sota-async-concurrency audit checklist "released on all paths?"
`load_state()` uses `try_read()`/`try_write()` for all 4 collections (visited, queue, index, stats) and silently ignores lock acquisition failures via `unwrap_or(0)`. If any lock is contested during initialization, the crawler starts with a partially-loaded state. Since this runs in `new()` (synchronous constructor), async locks may not be available. Should use `std::sync::Mutex` for initialization data or initialize from file without locks.

**D-SYNC-006: GWT Router broadcast_history uses Vec::remove(0) under write lock**
`neotrix-core/src/unified/core/energy_core/gwt_router.rs:43-49` | MEDIUM | Source: beagle-rust "keep critical sections tiny"
`route_wisdom()` pushes to `broadcast_history` then checks `len() > 1000` and calls `history.remove(0)`. `Vec::remove(0)` is O(n) — it shifts all elements left. With 1000+ wisdoms, this is a non-trivial operation under write lock. Every routing operation (which happens frequently as the GWT attention mechanism) pays this cost. Should use `VecDeque` with `pop_front()`, or a ring buffer.

**D-SYNC-007: ResourceRegistry register() is non-atomic across two locks**
`neotrix-core/src/unified/core/nt_core_resource_pool/resource_registry.rs:46-49` | HIGH | Source: sota-async-concurrency "consistency requires single lock or atomic compound operation"
`register()` acquires `pools.write()` then `name_to_kind.write()` sequentially. If the task is cancelled (Future dropped) between the two lock acquisitions, `pools` has the new entry but `name_to_kind` does not — an inconsistent state. Since these are two separate RwLock fields, there is no atomicity guarantee. Should acquire both locks in a single scope or use a single lock protecting both maps.

**D-SYNC-008: BackgroundLoopHandle single tokio::Mutex serializes all handlers**
`neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:745` | HIGH | Source: Tokio docs "if contention on a synchronous mutex becomes a problem, shard the mutex; let a dedicated task manage state"
The `spawn_handler!` macro (line 734-767) wraps `this: Arc<Mutex<BackgroundLoopHandle>>` and every handler tick does `h.lock().await` (line 745). With 20+ handlers (save, consolidate, goal, knowledge_chain, crystallization, voice, plugin, exploration, curiosity, prediction, awareness, cleanup, backup, kb_guard, etc.), all share one tokio Mutex. If any handler holds the lock for a long time (e.g., KB operations, network calls), all other handlers are blocked. This is a single-point serialization bottleneck for the entire background loop. Should shard per-domain or use message passing.

**D-SYNC-009: EnergyField sequential lock acquisitions per operation**
`neotrix-core/src/unified/core/energy_core/energy_field.rs:72-108` | MEDIUM | Source: Tokio docs "message passing is the most common form of synchronization"
`transform_energy()` calls `add_energy()` (locks `energy_state`), then `create_frequency()` (locks `frequency_set`), then `produce_vibration()` (locks `vibration_sequence`), then writes to `transformation_history` (locks `transformation_history`). Four separate lock acquisitions per transformation. Between locks, other tasks can observe partial state (energy added but frequency not yet created). Should either compound under a single lock or accept eventual consistency explicitly.

**D-SYNC-010: ConsciousnessTree prune() holds write lock while removing from HashMap**
`neotrix-core/src/unified/core/energy_core/consciousness_tree.rs:132-150` | MEDIUM | Source: beagle-rust "keep critical sections tiny"
`prune()` acquires a write lock on the entire nodes HashMap, collects IDs to remove, then iterates and removes them one by one. While the write lock is held, all readers (get_node, get_all_nodes, node_count, total_strength, layer_distribution) are blocked. For a tree with many nodes, this can cause a significant read stall. Should collect IDs, release lock, then do removal in a separate pass (or use a snapshot-swap pattern).

## Key Insights

1. **tokio::sync::Mutex is overused for pure-data structs**: The codebase wraps `TrafficAnalyzer` (a statistics struct with no IO) in tokio::sync::Mutex, incurring ~3x unnecessary overhead. Tokio's own docs state: "if the value behind the mutex is just data, it's usually appropriate to use a blocking mutex." All critical sections in api_proxy.rs and mitm.rs are synchronous.

2. **ProxyPool's 8-field RwLock architecture is a deadlock hazard**: With 8 independent RwLock fields and no documented lock ordering, the pool is a ticking deadlock bomb. The sota-async-concurrency audit checklist explicitly flags "Pool-within-pool" patterns.

3. **BackgroundLoopHandle single-Mutex design creates a head-of-line block**: 20+ handlers sharing one tokio Mutex means a slow KB write blocks consciousness tree updates, plugin ticks, and exploration. The architecture should either shard the state or use message passing (mpsc channels) for inter-handler communication.

4. **try_read()/try_write() in initialization code is fragile**: TorCrawler's `load_state()` silently drops lock failures, leading to partial initialization. The constructor runs synchronously but uses async primitives — a mismatch.

5. **Vec::remove(0) under write lock in GWT router**: A hot-path operation (every wisdom routing) pays O(n) cost for history trimming. VecDeque would be O(1).

6. **Non-atomic multi-lock registration**: ResourceRegistry's two-lock pattern creates a window for inconsistent state if the future is dropped between acquisitions.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| D-SYNC defects total | 10 |
| HIGH severity | 3 |
| MEDIUM severity | 7 |
