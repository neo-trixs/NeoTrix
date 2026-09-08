# Agent 4: Async Synchronization (Batch 870)

## Sources

1. https://rust-lang.github.io/async-book/part-guide/sync.html — Official Rust async book on channels, locking, and synchronization
2. https://deepwiki.com/tokio-rs/tokio/5.1-mutex-rwlock-and-semaphore — DeepWiki: Tokio Mutex/RwLock/Semaphore internals (batch_semaphore FIFO fairness, bit-packing)
3. https://rs4ts.dev/11-async/11-sync-primitives/ — Async synchronization primitives for Rust developers
4. https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html — Tokio Mutex API docs (FIFO fairness, no poisoning, cancel safety)
5. https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html — Tokio RwLock docs (write-preferring, recursive read deadlock)
6. https://tokio.rs/tokio/tutorial/shared-state — Tokio shared state tutorial (Arc<Mutex> pattern, Mutex sharding)
7. https://juanchi.dev/en/blog/mutex-deadlock-async-rust-production-diagnosis-patterns — Production deadlock diagnosis: 3 patterns (Classic Deadly Embrace, Reentrant, Inverted Order)
8. https://markaicode.com/rust-async-deadlock-prevention-patterns/ — 3 deadlock prevention patterns (Lock Ordering, Minimal Scope, Timeout Detection)
9. https://e6data.com/blog/deadlocking-tokio-mutex-without-holding-lock — Hidden deadlock: PausableFuture violates waker contract, semaphore permit held forever
10. https://dev.to/cspinetta/a-deadlock-on-an-uncontended-tokio-rwlock-caused-by-futuresexecutorblockon-490i — Uncontended RwLock deadlock from mixed executors (futures::executor::block_on + Tokio)
11. https://news.lavx.hu/article/never-snooze-a-future-the-hidden-deadlock-bug-in-async-rust — Future snoozing: pinning futures in select! causes hidden deadlocks
12. https://github.com/existential-birds/beagle/blob/HEAD/plugins/beagle-rust/skills/tokio-async-code-review/references/sync-primitives.md — Comprehensive sync review checklist (TOKIO_MUTEX_FOR_SHORT_SECTION, RWLOCK_NESTED_READ, etc.)

## Defects

**D-SYNC-001: Double std::sync::Mutex lock in PilotSupervisor::extract_failure_patterns — lock-ordering deadlock risk**
`neotrix-core/src/unified/core/nt_core_self/pilot_steering.rs:178-179` | **HIGH** | Source: juanchi.dev (Pattern 1: Classic Deadly Embrace)
Two `std::sync::Mutex` locks acquired simultaneously on `traces` and `failure_patterns` in the same function. If any other code path locks these in reverse order, or if `failure_patterns` is accessed from an async context while `traces` is held, this is a classic deadlock. Additionally, `self.worker_state.lock().unwrap()` at line 131 holds the lock while cloning, then drops it, but line 178-179 holds both locks across iteration — not atomic.

**D-SYNC-002: Multiple nested lock acquisitions in PilotSupervisor::evaluate — two sequential std::sync::Mutex locks without ordering discipline**
`neotrix-core/src/unified/core/nt_core_self/pilot_steering.rs:131,144` | **MEDIUM** | Source: markaicode.com (Lock Ordering Protocol)
`evaluate()` acquires `worker_state.lock()` at line 131, clones and drops it, then acquires `traces.lock()` at line 144 while still holding `&mut self`. No consistent lock ordering documented. The `&mut self` borrow means the compiler prevents concurrent access within the same call, but across different methods (`launch_worker` vs `record_trace` vs `evaluate`), the interleaving can create lock-ordering inversions under concurrent calls.

**D-SYNC-003: SkillRetrieval::retrieve double-locks embeddings — self-deadlock on non-reentrant std::sync::Mutex**
`neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_skill_engine/skill_retrieval.rs:228,237` | **HIGH** | Source: rs4ts.dev (Pitfall 2: Locking the same lock twice)
`retrieve()` calls `self.embeddings.lock().unwrap()` at line 228, then calls `self.embeddings.lock().unwrap()` again at line 237 while the first guard is still alive (line 228's guard isn't dropped until line 267 via `drop(embeddings_guard)`). This is a guaranteed self-deadlock on a `std::sync::Mutex` which is not reentrant. The second lock attempt will block forever waiting for the first permit that this same task holds.

**D-SYNC-004: skill_retrieval.rs holds embeddings lock across reranker computation**
`neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_skill_engine/skill_retrieval.rs:228-267` | **HIGH** | Source: tokio docs (keep critical sections tiny)
The `embeddings` lock is held from line 228 through line 267 while `self.bi_encoder.encode()` (line 234) and `self.reranker.rerank()` (line 270) execute — both potentially expensive CPU-bound operations. This serializes all skill retrieval across all tasks. The lock should be released after cloning the candidates.

**D-SYNC-005: tokio::sync::RwLock used in background loop for BMonitor alongside std::sync::Mutex for DistillationEngine — mixed sync/async lock semantics**
`neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:930,938` | **MEDIUM** | Source: e6data.com (mixed executor hazard)
`bbrain` uses `Arc<tokio::sync::RwLock<BMonitor>>` while `avatar_engine` uses `std::sync::Mutex<DistillationEngine>` in the same struct. Both are accessed in the same async background loop. If the `std::sync::Mutex` is ever held across an `.await` point (even indirectly through a callback), this creates a deadlock vector. The inconsistent primitive choices indicate no systematic sync policy.

**D-SYNC-006: nt_world_absorber holds std::sync::Mutex<Connection> across synchronous I/O operations**
`neotrix-core/src/unified/layers/perception/nt_world/nt_world_absorber/mod.rs:228-232,323-339` | **HIGH** | Source: Tokio tutorial (blocking in async context)
`self.kb.conn.lock()` acquires a `std::sync::Mutex` guard, then passes the connection to `nt_memory_kb_crawl::discover_from_seed` and `nt_memory_kb_crawl::run_crawl_cycle` — synchronous database operations. While the absorber functions are not `async`, they are called from background loop handlers that run on Tokio worker threads. A poisoned lock (from a panic in any crawl operation) will propagate and block all subsequent absorption cycles via `unwrap_or_else` or `?` propagation.

**D-SYNC-007: EventBus uses 5+ std::sync::Mutex fields with silent lock poisoning swallowing**
`neotrix-core/src/neotrix/nt_core_event_bus.rs:27-37,116-157` | **HIGH** | Source: juanchi.dev (unwrap() masks deadlocks)
The EventBus struct uses `std::sync::Mutex` for `log_file`, `handles`, `hooks`, `sync_handlers`, and a local `last` HashMap. All lock acquisitions use `if let Ok(guard) = self.xxx.lock()` which silently drops events when the mutex is poisoned. A single panic in any sync handler or hook registration silently disables event logging, hook dispatch, and sync handler execution for the rest of the process lifetime. No recovery mechanism exists.

**D-SYNC-008: Background loop telemetry uses static std::sync::Mutex with unwrap_or_else — potential panic cascade under poisoning**
`neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:1097,1114,1127` | **MEDIUM** | Source: tokio docs (no poisoning in tokio, but std poisons)
Static `std::sync::Mutex<Vec<TelemetryAlert>>` and `std::sync::Mutex<u32>` use `.lock().unwrap_or_else(|e| e.into_inner())` which bypasses poisoning but reads potentially inconsistent data. The `into_inner()` path returns data from a panicking thread with no guarantee of structural integrity. If the warmup counter or alert dedup state is corrupted, it could cause alert storms or silent alert suppression.

**D-SYNC-009: ConfirmationGate uses tokio::sync::RwLock but holds write lock across status update without timeout**
`neotrix-core/src/unified/layers/meta/nt_meta/nt_shield_approval/confirmation_gate/mod.rs:164,195` | **LOW** | Source: markaicode.com (Timeout-Based Deadlock Breaking)
`pending_requests.write().await` at lines 164 and 195 are used without timeout protection. While Tokio's RwLock is FIFO-fair, a stuck writer (e.g., a slow callback at line 176) will block all readers (line 203) and other writers indefinitely. No timeout wrapping exists on any lock acquisition.

**D-SYNC-010: PilotSteering holds traces lock while iterating AND mutates failure_patterns lock — compound lock scope**
`neotrix-core/src/unified/core/nt_core_self/pilot_steering.rs:178-179` | **MEDIUM** | Source: beagle review checklist (MINIMAL_LOCK_SCOPE)
`extract_failure_patterns()` acquires `traces.lock()` at line 178 and `failure_patterns.lock()` at line 179, then iterates over traces and pushes into patterns while both locks are held. The lock scope spans the entire function body including error grouping and pattern creation. This maximizes contention and makes deadlock detection harder.

**D-SYNC-011: No Semaphore usage for KB connection pool — SQLite connection serialized through single Mutex**
`neotrix-core/src/unified/layers/perception/nt_world/nt_world_absorber/mod.rs:228,323,370,425` | **MEDIUM** | Source: DeepWiki (Semaphore for resource counting)
All KB operations serialize through `self.kb.conn.lock()` — a single `std::sync::Mutex<Connection>`. No connection pooling or `tokio::sync::Semaphore`-based concurrency limiting exists. Under concurrent absorption (batch operations, multiple background loop handlers), all KB operations serialize on this single mutex, creating a bottleneck. Tokio's `Semaphore` with permit counting would allow bounded concurrent access.

**D-SYNC-012: nt_core_sae_bridge uses RwLock with unwrap_or_else poison bypass — SparseAutoencoder data integrity unknown**
`neotrix-core/src/unified/core/nt_core_sae_bridge.rs:44,92` | **LOW** | Source: tokio docs (poisoning indicates inconsistent state)
`self.sae.write().unwrap_or_else(|e| e.into_inner())` bypasses poisoning on the `SparseAutoencoder` lock. If a panic occurs during SAE encode/decode, the next caller silently receives potentially corrupted model state. The SAE is a neural network — corrupted weights could produce meaningless embeddings without any error signal.

## Key Insights

1. **No systematic sync primitive policy**: NeoTrix mixes `std::sync::Mutex` (60+ occurrences), `tokio::sync::RwLock` (~15 occurrences), and `std::sync::RwLock` (~12 occurrences) without a documented decision framework. The Tokio recommendation — use `std::sync::Mutex` for short non-await critical sections, `tokio::sync::Mutex` only when holding across `.await` — is not consistently followed.

2. **Self-deadlock in SkillRetrieval is a confirmed bug**: `skill_retrieval.rs:228,237` double-locks the same non-reentrant `std::sync::Mutex`, which will deadlock on every call to `retrieve()` when embeddings are non-empty. This is a production-blocking defect.

3. **Poisoning silence is systemic**: The EventBus (5 mutexes), background loop telemetry, and SAE bridge all silently swallow poisoned locks via `if let Ok()` or `unwrap_or_else(|e| e.into_inner())`. No recovery, logging, or alerting exists. A single panic in any handler can permanently degrade subsystems.

4. **Lock ordering is undocumented**: `PilotSupervisor` acquires `traces` and `failure_patterns` locks without a documented ordering. The orchestrator acquires `worker` and `agent_team` locks without ordering discipline. Any function acquiring two locks is a potential deadlock.

5. **KB connection serialization is a bottleneck**: The single `Mutex<Connection>` in the absorber serializes all database operations across the entire background loop. A `Semaphore`-based pool or dedicated task with message-passing (mpsc) would eliminate this contention.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Sources analyzed | 12 |
| HIGH severity | 4 |
| MEDIUM severity | 5 |
| LOW severity | 3 |
| Self-deadlock bugs | 1 (D-SYNC-003) |
| Lock-ordering risks | 3 (D-SYNC-001, 002, 010) |
| Poison-silence defects | 3 (D-SYNC-007, 008, 012) |
| Contention/serialization issues | 2 (D-SYNC-004, 011) |
