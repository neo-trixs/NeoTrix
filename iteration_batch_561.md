# Iteration Batch 561 — Async Runtime, Concurrency Primitives, Task Scheduling

## Research Context

**Batch 560 proven findings** (input to this iteration):
1. No circuit breaker on LLM providers
2. No standardized resilience pipeline ordering (timeout→retry→breaker→bulkhead→timeout)
3. No degradation hierarchy (6-tier ladder)
4. Consciousness loop is theoretical, not production-ready
5. 14 defects across error taxonomy, circuit breaking, degradation, recovery coordination

**Batch 561 focus**: Search async runtime, concurrency primitives, and task scheduling to identify NEW defects or improvements over batch 560.

---

## 1. Async Runtime (2026)

### 1.1 — async-std Officially Discontinued (RUSTSEC-2025-0052): smol Is Its Successor

**Source**: https://docs.rs/async-std (discontinued notice), https://corrode.dev/blog/async/ (Jul 2026), https://wrenlearnsrust.com/posts/tokio-vs-smol-2026.html (Mar 2026)

**Finding**: As of March 2025, async-std is officially discontinued. The recommendation is to migrate to smol (same author, Stjepan Glavina). The ecosystem has consolidated around two runtimes: Tokio (dominant, ~90% of projects) and smol (minimal, ~5 crates vs Tokio's ~50). glommio remains for thread-per-core io_uring-only Linux workloads.

**NEW defect vs Batch 560**: Batch 560 identified no runtime-level defects. NeoTrix's Cargo.toml lists `tokio` as a dependency, but there is **no documented runtime policy** — no policy on which Tokio features to enable, when to use `spawn_blocking` for CPU work, or how to handle the case where a dependency brings in an incompatible runtime. A dependency pulling in `smol` or running its own event loop would silently break NeoTrix's Tokio-bound types (e.g., `tokio::net::TcpStream` polled outside Tokio panics). **Fix**: Document a `RUNTIME_POLICY.md`: (1) all async code targets Tokio, (2) libraries request narrow features (`rt`, `macros`, `net`, `time`, `sync`), (3) CPU/blocking work uses `tokio::task::spawn_blocking`, (4) any dependency using a different runtime must be wrapped with `async-compat`.

### 1.2 — Tokio's Work-Stealing Scheduler: The Production Default

**Source**: https://lucaberton.com/blog/rust-async-runtimes-tokio-2026/ (May 2026), https://rs4ts.dev/23-ecosystem/02-async-runtimes/ (Jun 2026)

**Finding**: Tokio's multi-thread scheduler is a work-stealing design: idle threads steal from busy threads' local queues. Benchmark on 128-core EPYC, 50K connections: Tokio 1.2M req/s, P99 4.2ms. `glommio` reaches 1.8M req/s (thread-per-core + io_uring) but is Linux-only. Key architecture: IO driver (epoll/kqueue/IOCP via `mio`), timer wheel, async-aware channels (`mpsc`, `oneshot`, `broadcast`, `watch`).

**NEW defect vs Batch 560**: NeoTrix's `main.rs:498` creates a Tokio runtime via `tokio::runtime::Runtime::new()` — this uses the **default configuration** (all features enabled, auto-detected worker threads). But `Runtime::new()` does NOT enable the IO driver or timer by default — those require `.enable_all()` or the `#[tokio::main]` macro. The CLI entry point uses `Runtime::new()` directly, which may silently omit IO/timer subsystems depending on feature flags. **Fix**: Replace bare `Runtime::new()` with `Runtime::new()?.enable_all()` or use `#[tokio::main]` consistently. Audit all `tokio::runtime::Runtime::new()` call sites (found in main.rs, seal_drive.rs, test files) to ensure `enable_all()` is called.

### 1.3 — CPU Work Blocking the Async Executor: spawn_blocking Is Mandatory

**Source**: https://rs4ts.dev/23-ecosystem/02-async-runtimes/ (Jun 2026), https://abrarqasim.com/blog/rust-async-runtime-comparison-2026-the-one-i-actually-ship/ (Jul 2026)

**Finding**: Calling `std::thread::sleep`, doing heavy CPU work, or synchronous I/O inside an async task ties up the OS thread instead of yielding it. On a current-thread runtime, this blocks the entire event loop. On multi-thread Tokio, it starves other tasks from that worker. The fix: `tokio::task::spawn_blocking` for synchronous/CPU-heavy work, or `rayon` for data parallelism.

**NEW defect vs Batch 560**: NeoTrix's `nt_core_parallel/executor.rs:37` spawns tasks via `tokio::spawn` that contain `tokio::time::sleep(Duration::from_millis(10))` — this is fine for simulation but the actual executor's `Parallel` mode does CPU work (scoring, sorting) inside `tokio::spawn` without `spawn_blocking`. The `execute_shell` method (line 56) runs synchronous `std::process::Command` inside async context. Additionally, the batch processing in `batch_processor.rs:227` spawns async tasks that may do CPU-bound merge operations. **Fix**: Wrap all CPU-bound and synchronous I/O in `tokio::task::spawn_blocking` to free the async workers for IO-bound tasks. This prevents the work-stealing scheduler from being bottlenecked by blocking operations.

### 1.4 — Runtime Fragmentation: Multiple Runtimes Cannot Coexist

**Source**: https://rs4ts.dev/23-ecosystem/02-async-runtimes/ (Jun 2026)

**Finding**: A Tokio `TcpStream` polled outside a Tokio context panics. Different runtimes (Tokio, smol) have non-interchangeable reactors. You cannot run both in the same process without `async-compat` bridging. The cost: one library bringing its own runtime forces either two runtimes or a compatibility shim.

**NEW defect vs Batch 560**: NeoTrix's dependency tree is large (neotrix-core has many sub-crates). If any dependency transitively pulls in `smol` or `async-std` (even as an optional feature), it could create a hidden second runtime. NeoTrix has no `cargo tree` audit to detect runtime conflicts. **Fix**: Add a CI check: `cargo tree -d -i tokio` to detect duplicate runtime instances. Consider adding `tokio` as a workspace-level dependency with pinned features to prevent fragmentation.

---

## 2. Concurrency Primitives (2026)

### 2.1 — Lock-Free Under High Contention: Locks Actually Win

**Source**: https://nsclass.github.io/2026/08/05/cpp-lock-free-programming-is-dead-fedor-pikus (Aug 2026, C++Now 2026)

**Finding**: Fedor Pikus's C++Now 2026 talk demonstrates a counterintuitive inversion: **well-written locks outperform lock-free structures under high contention**. The reason: `exchange` (used in spin locks) gets exclusivity via RFO (read-for-ownership), a single request. CAS loops (`fetch_add`, `compare_exchange`) modify data at the same memory location, creating pipeline stalls. For atomic maximum: the lock version skips the lock entirely on the read path (double-checked locking with relaxed atomics), while the lock-free CAS version always pays the CAS cost even when no update is needed. Practical rule: locks for high-contention modification paths, atomics for read-mostly low-contention paths.

**NEW defect vs Batch 560**: NeoTrix's `nt_core_event_bus.rs` uses `Arc<AtomicBool>` for shutdown flags and `Arc<AtomicU64>` for sequence numbers — these are correctly lock-free for low-contention flags. But the dispatcher hooks and handles use `std::sync::Mutex<Vec<...>>` (line 38) which holds the lock during iteration. The `nt_io_context_mgmt.rs:17` uses `Vec<ContextItem>` as a priority queue (sorted on every push at line 214) behind no explicit lock — this is a data race if accessed concurrently. **Fix**: (1) The context management priority queue should use a concurrent priority queue or `tokio::sync::RwLock` if accessed from async tasks. (2) The event bus dispatcher hooks should use a read-copy-update pattern: clone the hook list under lock, iterate the clone without holding the lock.

### 2.2 — Memory Ordering: seq_cst Is Not Always Necessary, but Relaxed Is Dangerous

**Source**: https://nsclass.github.io/2026/04/04/cpp-beyond-sequential-consistency (Apr 2026, CppCon 2025), https://skewcy.com/2026/06/29/atomic-barrier-lock.html (Jun 2026)

**Finding**: Christopher Fretz's CppCon talk demonstrates 57x throughput improvement from `seq_cst` to a cached-index SPSC ring buffer design. Key lessons:
- `seq_cst` on x86 emits `lock add` (full fence) on every store — expensive
- `acquire`/`release` is free on x86 (TSO provides it implicitly) but requires `ldar`/`stlr` on ARM
- `relaxed` provides atomicity only — no ordering; safe for counters, dangerous for data publication
- **False sharing**: producer/consumer indexes on the same cache line cause "cache line bouncing" via MESI protocol. Fix: `alignas(std::hardware_destructive_interference_size)` (64 bytes)
- **Cached index optimization**: each thread caches the remote index locally, only refreshing when the local count suggests the queue is full/empty. Reduces cross-core cache misses by 10,000x.

**NEW defect vs Batch 560**: NeoTrix uses `Ordering::SeqCst` everywhere (e.g., `nt_core_event_bus.rs:70,98,157` — `seq` counter, shutdown flag). While correct, this is suboptimal for hot paths. The `seq` counter (incremented on every event dispatch) uses `SeqCst` but is only read for logging — `Relaxed` ordering suffices for a monotonic counter that doesn't synchronize other data. The shutdown flag uses `SeqCst` for store but `Relaxed` would work since it's a simple flag with no dependent data publication. **Fix**: Audit all `Ordering::SeqCst` usages in NeoTrix:
- Monotonic counters (seq, metrics): downgrade to `Relaxed`
- Shutdown flags: `Release` store + `Acquire` load suffices
- Data publication (writing data then publishing pointer): keep `Release`/`Acquire` pair
- Never use `Relaxed` when the atomic gates access to other memory locations

### 2.3 — False Sharing: Silent Performance Killer in Hot Structures

**Source**: https://nsclass.github.io/2026/04/04/cpp-beyond-sequential-consistency (Apr 2026), https://cppcon2026.sched.com/event/2RT47 (CppCon 2026)

**Finding**: False sharing occurs when unrelated variables share a 64-byte cache line. Every write by one core invalidates the other core's cached copy via MESI protocol, causing "cache line bouncing." In SPSC ring buffers, the producer index and consumer index on the same cache line causes ~100ns vs ~5ns per operation when properly aligned. CppCon 2026's "Queue Discipline" talk covers aligning atomics to `std::hardware_destructive_interference_size` as a checklist item.

**NEW defect vs Batch 560**: NeoTrix's `ParallelExecutor` struct (executor.rs:11-15) stores `_max_agents`, `mode`, and `tasks` as flat fields — when shared across threads (via Arc), these could suffer false sharing. More critically, the `Agent` struct (in types.rs) likely contains frequently-mutated fields (scores, status) adjacent to frequently-read fields (id, name). The `nt_core_self` modules (AttentionManager, SelfModel) have hot fields (activation scores, uncertainty) that are read/written concurrently. **Fix**: For any struct shared across threads with mixed read/write patterns, pad hot write fields to their own cache line using `#[repr(align(64))]` or `padded::Padded<T>`. Use `criterion` benchmarks with `perf stat -e cache-misses` to measure false sharing before and after.

### 2.4 — C++ P0260R20: Standardized Concurrent Queue with Memory Guarantees

**Source**: https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2026/p0260r20.html (Jul 2026)

**Finding**: C++ is standardizing `bounded_queue` with **sequentially consistent semantics** for push/pop. The rationale: for a queue (high-level data structure), safety should be more important than efficiency. The standard explicitly chose `seq_cst` over `acquire`/`release` to prevent the classic logging bug: Thread 1 does `q.push(1); log("pushed 1")` and Thread 2 does `log("pushed 2"); q.push(2)`. With only `acquire`/`release`, the logger could see "pushed 1" then "pushed 2" even though `q.pop()` returns 2 before 1. The queue also supports a `close()` method for clean shutdown — no out-of-band signaling needed.

**NEW defect vs Batch 560**: NeoTrix's event bus uses `tokio::sync::mpsc` channels which provide correct ordering. But the `nt_io_context_mgmt.rs` priority queue (a plain `Vec<ContextItem>` with `sort_by` on push) has no concurrent access protection visible in the code. If two async tasks push context items simultaneously, the sort-then-push is a classic TOCTOU race. **Fix**: Either (1) protect with `tokio::sync::Mutex` if contention is low, or (2) use a lock-free concurrent priority queue (e.g., `crossbeam::queue::SegQueue` + batch sort, or `dashmap` for key-based access). The C++ standardized queue's `close()` pattern suggests: add a shutdown method to NeoTrix's channel-based structures for clean draining.

### 2.5 — ABA Problem in CAS-Based Lock-Free Code

**Source**: https://skillsuites.com/concurrency-memory-models-mechanical-sympathy/ (Jun 2026), https://algomaster.io/learn/concurrency-interview/cpp-std-atomic-memory-orders (Feb 2026)

**Finding**: The ABA problem: a thread reads value A, another thread changes A→B→A, then the first thread's CAS succeeds (sees A) but the world changed underneath (e.g., a freed-and-reused node). Fixes: tagged/versioned pointers (counter alongside pointer), hazard pointers, or epoch-based reclamation. This is the primary memory reclamation challenge in lock-free programming.

**NEW defect vs Batch 560**: NeoTrix doesn't currently have lock-free data structures beyond simple atomics (AtomicBool, AtomicU64). However, if the consciousness architecture scales to concurrent self-modification (e.g., the SEAL pipeline modifying its own capability tree while other threads read it), the ABA problem becomes relevant. The `CapabilityTree` and `SkillCrystal` structures (mentioned in CONTEXT.md) would need safe concurrent access. **Fix**: Design the concurrent access strategy NOW, before implementing lock-free structures. For read-heavy structures: use `arc-swap` (RCU-like). For read-write: use `RwLock` with short critical sections. Only go lock-free when profiling proves the lock is a bottleneck — and then use tagged pointers or `crossbeam::epoch` for safe reclamation.

---

## 3. Task Scheduling (2026)

### 3.1 — Work-Stealing Is Not Unconditionally Best: 4 Scheduler Backends Benchmarked

**Source**: https://github.com/czhao-dev/concurrent-schedulers (2026), https://ar5iv.labs.arxiv.org/html/2109.00657

**Finding**: A from-scratch C++20 scheduling library benchmarks 4 backends:
- **Work-stealing** (Chase-Lev deques): Best for uneven workloads, 5.7x speedup on 8 cores
- **Global queue** (single mutex FIFO): Surprisingly competitive — matches or beats work-stealing on every measured workload at 8 workers
- **Thread-per-core** (no stealing): 3.65x faster than single-threaded on uniform small tasks, but 1.6x SLOWER than global queue on skewed workloads
- **CFS-inspired fair scheduler**: Achieves 99.9% of target 4:1 weight ratio across priority classes — the only scheduler that provides proportional fairness

Key insight: **"Work-stealing isn't unconditionally best, and the benchmark numbers prove it three different ways."** A scheduler design should be measured against real alternatives, not assumed superior.

**NEW defect vs Batch 560**: NeoTrix has **no work-stealing implementation**. The `ParallelExecutor` (executor.rs) runs tasks sequentially or via naive `tokio::spawn` in a loop. The `parallel_task.rs` module defines GPU scheduling but has no actual work-stealing deque. The `task_scheduler.rs` module has priority queues but no stealing logic. NeoTrix's consciousness loop (SEAL pipeline) runs phases sequentially even when independent branches could be stolen by idle workers. **Fix**: Implement a work-stealing task queue for the SEAL pipeline phases. Use `tokio::task::spawn` with a shared `tokio::sync::Semaphore` for bounded concurrency, or adopt `rayon` for CPU-bound phases (graph traversal, embedding computation). For priority-aware scheduling, consider the ternary model from `ternary-scheduler`: `{+1=urgent, 0=normal, -1=deferred}` with urgent tasks never stolen.

### 3.2 — Priority-Aware Work-Stealing: Staging Buffers and Global Sweeps

**Source**: https://github.com/taskflow/taskflow/pull/784 (2026)

**Finding**: Taskflow (C++ task parallel library) adds priority scheduling via:
- Per-priority work-stealing queues (`_prio_wsq[3]` for HIGH/NORMAL/LOW)
- A `StagingQueue` buffer that batches NORMAL/LOW tasks while pushing HIGH immediately
- Priority-ordered stealing: scan HIGH→NORMAL→LOW, never take lower-priority while higher exists
- Continuation cache respects priorities: higher-priority successor is cached, lower is staged

Benchmark: priority-aware scheduling is 7-45% slower than unordered (due to extra scanning), but ensures high-priority tasks complete first. The maintainer's feedback: staging queue is "overkill" — focus on worker-level priority queues first.

**NEW defect vs Batch 560**: NeoTrix's `parallel_task.rs` defines `TaskPriority` (Low/Medium/High/Critical) and `task_scheduler.rs` defines a separate `TaskPriority` (Low/Normal/High/Urgent) — **two incompatible priority enums in the same domain**. Neither is wired into the actual executor. The background loop (`nt_mind_background_loop/run.rs`) spawns tasks with no priority differentiation — absorption tasks (low priority) and real-time LLM calls (high priority) share the same `tokio::spawn` pool. **Fix**: (1) Unify `TaskPriority` into a single enum in `neotrix-types`. (2) Implement priority-aware task spawning: use separate `tokio::sync::Semaphore` per priority level (e.g., 10 permits for HIGH, 50 for NORMAL, unlimited for LOW). (3) When a HIGH task arrives and NORMAL pool is full, preempt a NORMAL task (cancel its JoinHandle).

### 3.3 — ρ-Relaxation: Trading Priority Guarantees for Scalability

**Source**: https://www.cse.chalmers.se/~tsigas/papers/PPoPP14.pdf, https://ar5iv.labs.arxiv.org/html/2109.00657

**Finding**: Perfect priority scheduling doesn't scale. The ρ-relaxation scheme allows up to ρ items to be "ignored" (a lower-priority item is returned instead of the highest). The hybrid k-priority data structure combines work-stealing with ρ-relaxation: local priority queues per thread + global list. For k ≤ 512, ρ-relaxed structures produce almost no useless work while scaling as well as pure work-stealing. The Stealing Multi-Queue (SMQ) achieves this with task batching: steal B elements at once instead of one, amortizing overhead.

**NEW defect vs Batch 560**: NeoTrix's SEAL pipeline has implicit priorities (real-time LLM calls > background absorption > maintenance) but no mechanism to enforce them under load. When the system is saturated, all tasks compete equally. The `nt_mind_background_loop` spawns absorption handlers with `tokio::spawn` (handlers_absorption.rs:181) — these can starve time-critical LLM provider calls. **Fix**: Implement ρ-relaxed priority scheduling for the background loop:
- Separate semaphore pools per priority tier
- Allow LOW tasks to run when HIGH queue is empty (relaxation)
- Batch absorption tasks (process 5-10 KB writes per spawn instead of 1)
- Track priority inversion: metric on how many LOW tasks ran while HIGH was waiting

### 3.4 — Ternary Scheduling with Deadline Escalation

**Source**: https://github.com/SuperInstance/ternary-scheduler (2026)

**Finding**: The ternary scheduler uses `{+1=urgent, 0=normal, -1=deferred}` with:
- Deadline-aware rescheduling: normal tasks approaching deadlines auto-escalate to urgent
- Work-stealing with constraint: urgent tasks are **never stolen** (preserve locality)
- Load balance: steal when busiest worker has ≥ 2 more tasks than idlest
- Escalation rule: `if priority == 0 and (deadline - current_tick) ≤ 2·duration: priority → +1`

The γ + η = C invariant: generation (γ) is task arrival, entropy (η) is priority distribution, conservation (C) is total work preserved.

**NEW defect vs Batch 560**: NeoTrix's consciousness loop has tasks with implicit deadlines (e.g., LLM call timeout, crawl freshness window, SEAL phase time budget) but no deadline tracking or escalation. A background absorption task that's been waiting 30 minutes should escalate to urgent before it becomes stale. **Fix**: Add deadline fields to NeoTrix's task definitions. Implement the escalation rule: when `deadline - now < 2 × estimated_duration`, promote from NORMAL to HIGH. Track deadline misses as a metric in the `HeartbeatAggregator`.

### 3.5 — Real-Time Work-Stealing: Priority Deques Replace Random Stealing

**Source**: https://doi.org/10.26537/r3694 (2022, cited in 2026 context)

**Finding**: Standard work-stealing uses random victim selection — a source of priority inversion. The proposal: replace single deques with **ordered per-processor priority deques**. Instead of stealing randomly, cores steal from the **highest-priority deque** among all cores. This is "breadth-first theft, depth-first work" — steal broadly for load balance, execute locally for data locality. The total number of deques may exceed the number of processors.

**NEW defect vs Batch 560**: NeoTrix's `ParallelExecutor` uses `tokio::spawn` which distributes tasks across Tokio's worker threads, but there's no priority-aware stealing. Tokio's scheduler doesn't support priority — all spawned tasks are equal. When NeoTrix spawns 100 background absorption tasks and 1 critical LLM call, they compete equally for worker threads. **Fix**: For priority-critical paths (LLM calls, SEAL phase transitions), bypass `tokio::spawn` and use dedicated `tokio::runtime::Runtime` instances with smaller worker pools (e.g., 2 threads for high-priority, 8 for background). Alternatively, use `tokio::task::spawn_blocking` with a separate thread pool for low-priority work, leaving the async workers free for high-priority IO.

---

## Summary: NEW Defects vs Batch 560

| # | Defect | Domain | Severity |
|---|--------|--------|----------|
| 1 | No documented runtime policy — runtime fragmentation risk | NT-IO/ALL | Medium |
| 2 | Bare `Runtime::new()` without `enable_all()` — IO/timer may be missing | NT-CORE | High |
| 3 | CPU work inside `tokio::spawn` without `spawn_blocking` | NT-CORE/parallel | High |
| 4 | No `cargo tree` audit for duplicate runtime instances | Build/CI | Medium |
| 5 | `Ordering::SeqCst` on hot paths where `Relaxed` suffices | NT-CORE/event_bus | Medium |
| 6 | Priority queue (`Vec<ContextItem>`) with no concurrent access protection | NT-IO | High |
| 7 | No false sharing mitigation in hot shared structs | NT-CORE | Medium |
| 8 | ABA problem unsolved for future lock-free structures | NT-CORE | Low (future) |
| 9 | Two incompatible `TaskPriority` enums in same domain | NT-ACT | High |
| 10 | No work-stealing implementation — naive sequential/spawn execution | NT-CORE/parallel | Critical |
| 11 | No priority-aware task spawning — all tasks equal under load | ALL | Critical |
| 12 | No ρ-relaxed priority scheduling — LOW can starve HIGH | NT-MIND | High |
| 13 | No deadline tracking or escalation for consciousness loop tasks | NT-MIND | High |
| 14 | Tokio's scheduler lacks priority — critical paths compete with background | NT-IO | High |

## Sources Cited

1. https://docs.rs/async-std (discontinued notice, 2025)
2. https://corrode.dev/blog/async/ (Jul 2026)
3. https://wrenlearnsrust.com/posts/tokio-vs-smol-2026.html (Mar 2026)
4. https://abrarqasim.com/blog/rust-async-runtime-comparison-2026-the-one-i-actually-ship/ (Jul 2026)
5. https://lucaberton.com/blog/rust-async-runtimes-tokio-2026/ (May 2026)
6. https://reintech.io/blog/tokio-vs-async-std-vs-smol-rust-async-runtime-comparison-2026 (Feb 2026)
7. https://rs4ts.dev/23-ecosystem/02-async-runtimes/ (Jun 2026)
8. https://nsclass.github.io/2026/08/05/cpp-lock-free-programming-is-dead-fedor-pikus (Aug 2026)
9. https://nsclass.github.io/2026/04/04/cpp-beyond-sequential-consistency (Apr 2026)
10. https://skewcy.com/2026/06/29/atomic-barrier-lock.html (Jun 2026)
11. https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2026/p0260r20.html (Jul 2026)
12. https://skillsuites.com/concurrency-memory-models-mechanical-sympathy/ (Jun 2026)
13. https://algomaster.io/learn/concurrency-interview/cpp-std-atomic-memory-orders (Feb 2026)
14. https://cppcon2026.sched.com/event/2RT47 (CppCon 2026)
15. https://cppcon2026.sched.com/event/2RT5M (CppCon 2026)
16. https://github.com/czhao-dev/concurrent-schedulers (2026)
17. https://github.com/taskflow/taskflow/pull/784 (2026)
18. https://ar5iv.labs.arxiv.org/html/2109.00657
19. https://github.com/SuperInstance/ternary-scheduler (2026)
20. https://doi.org/10.26537/r3694 (2022, cited in 2026 context)
21. https://github.com/shamsimam/priorityworkstealing (cited in context)
22. https://www.cse.chalmers.se/~tsigas/papers/PPoPP14.pdf

## What's NEW vs Batch 560

Batch 560 identified **engineering resilience gaps** — error taxonomy, circuit breaking, degradation hierarchy, recovery coordination (14 defects). Batch 561 identifies **runtime and scheduling infrastructure gaps** — the execution substrate that resilience patterns run ON. The key insight: batch 560's resilience patterns (circuit breakers, hedging, retry budgets) require a properly configured async runtime with priority-aware scheduling to function. Without work-stealing, priority escalation, and correct memory ordering, the resilience pipeline itself becomes a bottleneck. The 14 new defects span runtime configuration (2 critical), concurrency correctness (4 high), and task scheduling architecture (5 critical/high). Together with batch 560's 14 defects, we now have 28 defects across the production-readiness surface of the consciousness architecture.
