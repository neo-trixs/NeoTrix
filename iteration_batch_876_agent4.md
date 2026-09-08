# Agent 4: Structured Concurrency (Batch 876)

## Sources
- https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html (Tokio JoinSet official docs)
- https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTracker.html (TaskTracker official docs)
- https://rust-lang.github.io/async-book/part-reference/structured.html (Rust Async Book — Structured Concurrency)
- https://medium.com/@adamszpilewicz/structured-concurrency-in-rust-with-tokio-beyond-tokio-spawn-78eefd1febb4 (Structured Concurrency in Rust with Tokio)
- https://www.rustfaq.org/en/what-is-tokio-task-joinset/ (What is tokio JoinSet)
- https://www.toolsku.com/en/blog/rust-tokio-graceful-shutdown-2026/ (Graceful Shutdown Patterns)
- https://microsoft.github.io/RustTraining/async-book/ch13-production-patterns.html (Production Patterns — JoinSet & TaskTracker)
- https://tokio.rs/tokio/topics/shutdown (Tokio Graceful Shutdown)
- https://github.com/tokio-rs/tokio/issues/6664 (JoinSet join_all discussion)
- https://github.com/tokio-rs/tokio/blob/5030b300/tokio/src/task/join_set.rs (JoinSet source)
- https://github.com/tokio-rs/tokio/blob/5030b300/tokio-util/src/task/task_tracker.rs (TaskTracker source)
- https://users.rust-lang.org/t/how-to-share-tokio-joinset-across-different-tasks-threads/117289 (JoinSet sharing patterns)
- https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/ (Cancellation patterns)

## Defects

### D-SCON-001: Zero JoinSet/TaskTracker usage — 92+ raw `tokio::spawn` without structured lifecycle
**Severity: HIGH**
**Source: Tokio docs + Async Book structured concurrency chapter + production patterns**
**File: codebase-wide (92+ occurrences across 40+ files)**

NeoTrix has **zero** uses of `tokio::task::JoinSet` or `tokio_util::task::TaskTracker` across the entire codebase. Every concurrent task is spawned via raw `tokio::spawn`, returning a `JoinHandle` that is either stored in a `Vec` or immediately discarded. The Tokio docs explicitly state: "When the JoinSet is dropped, all tasks in the JoinSet are immediately aborted." JoinSet is the recommended primitive for task groups. TaskTracker is recommended when tasks should exit and free memory immediately (vs JoinSet which holds results in memory). NeoTrix uses neither, meaning:
- No structured parent-child task relationship
- No automatic abort-all on drop
- No memory-efficient task tracking for long-lived background tasks
- No `close()`/`wait()` semantics for graceful shutdown

The Rust Async Book states: "The essential idea of structured concurrency is that tasks are organised into a tree. Child tasks start after their parents and always finish before them." NeoTrix's raw spawn pattern violates this invariant.

### D-SCON-002: BackgroundLoop `Vec<JoinHandle<()>>` — no abort-all-on-drop, no completion ordering
**Severity: HIGH**
**Source: Tokio JoinSet docs + TaskTracker docs (memory leak warning)**
**File: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/mod.rs:96`**

`BackgroundLoop` stores spawned tasks as `pub handles: Vec<JoinHandle<()>>` (line 96). This is structurally equivalent to a JoinSet but without:
1. **Abort-all-on-drop**: When `BackgroundLoop` is dropped without calling `shutdown()`, all tasks are detached (continue running as orphans) rather than aborted. JoinSet would abort them automatically.
2. **Completion-order yielding**: `Vec<JoinHandle>` requires sequential `.await` on each handle. JoinSet yields results as they complete.
3. **Memory efficiency**: TaskTracker docs warn: "A JoinSet keeps track of the return value of every inserted task. This means that if the caller keeps inserting tasks and never calls join_next, then their return values will keep building up and consuming memory." The `Vec<JoinHandle>` has the same issue — if tasks complete but aren't drained, their results accumulate.
4. **No task count visibility**: No `len()` or `is_empty()` to monitor active background tasks.

### D-SCON-003: ParallelExecutor spawns tasks sequentially, defeating parallelism
**Severity: MEDIUM**
**Source: Tokio docs on JoinSet vs futures::join_all performance**
**File: `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/executor.rs:34-46`**

```rust
ExecMode::Parallel => {
    let mut results = Vec::new();
    for (_, input, _) in &self.tasks {
        let input = input.clone();
        let handle = tokio::spawn(async move { ... });
        if let Ok(res) = handle.await {  // AWAIT INSIDE THE LOOP
            results.push(res);
        }
    }
    results
}
```

The `ParallelExecutor` spawns tasks in a loop and **awaits each handle inside the same loop** (line 41). This serializes execution — task N+1 doesn't complete until task N finishes, even though all tasks are spawned as independent Tokio tasks. This is equivalent to `futures::future::join_all` polled sequentially, which the research identifies as: "Lacks true CPU parallelism; higher deadlock potential; polled on a single thread." The correct pattern is to spawn all tasks into a JoinSet, then drain with `while let Some(res) = set.join_next().await`.

### D-SCON-004: EventBus `subscribe_layer` spawns fire-and-forget tasks with no structured shutdown
**Severity: HIGH**
**Source: Tokio graceful shutdown docs + production patterns**
**File: `neotrix-core/src/neotrix/nt_core_event_bus.rs:359-397`**

`subscribe_layer()` returns a raw `tokio::task::JoinHandle<()>` for each layer subscriber (9 layers = 9 spawned tasks). The caller (`subscribe_all_layers`) collects them into a `Vec<JoinHandle>` but there is no structured coordination:
- No JoinSet to track all 9 subscribers
- No CancellationToken to signal graceful shutdown
- The `EventBus::shutdown()` method (line 228) only handles `std::thread::JoinHandle` (sync mode), not the tokio tasks
- If the `broadcast::Sender` is dropped, subscribers receive `RecvError::Closed` and break, but there's no guarantee they've exited before the process terminates

The Tokio production patterns doc states: "Always have a timeout on the shutdown wait — a hung task shouldn't prevent process exit." NeoTrix has no such mechanism for the async subscriber tasks.

### D-SCON-005: EventBus clone silently discards all handles — cloned bus has orphaned subscribers
**Severity: MEDIUM**
**Source: JoinSet sharing patterns research + Tokio docs on handle ownership**
**File: `neotrix-core/src/neotrix/nt_core_event_bus.rs:40-52`**

```rust
impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            handles: std::sync::Mutex::new(Vec::new()), // EMPTY!
            ...
        }
    }
}
```

When `EventBus` is cloned (line 46), the `handles` field is initialized as an **empty Vec**, discarding all tracked subscriber thread handles from the original. This means:
- Cloned EventBus loses visibility into subscriber threads
- Shutdown on a cloned bus won't wait for subscribers
- Original subscriber threads continue running but are unreachable from the clone

A JoinSet-based design would use `Arc`-shared tracking or a dedicated task manager that all clones reference.

### D-SCON-006: BackgroundLoop shutdown uses shared 5-second deadline — one slow task aborts all
**Severity: HIGH**
**Source: Tokio production patterns (per-task timeout) + JoinSet shutdown semantics**
**File: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:48-61`**

```rust
let deadline = tokio::time::sleep(Duration::from_secs(5));
tokio::pin!(deadline);
for handle in self.handles.drain(..) {
    let abort = handle.abort_handle();
    tokio::select! {
        biased;
        _ = &mut deadline => {
            abort.abort();
        }
        _ = handle => {}
    }
}
```

The shutdown handler uses a **single shared deadline** across all background tasks. If task 1 takes 4.9 seconds to finish, task 2 only gets 0.1 seconds before being aborted. The correct pattern (per the production patterns source) is:
- Per-task timeout: each task gets its own deadline
- Or: `JoinSet::shutdown()` which "aborts all tasks and waits for them to finish shutting down"
- Or: CancellationToken + TaskTracker with per-task `tokio::time::timeout`

With 15+ background handlers (backup, discovery, nexus weaver, absorption, daily intel, always-on, skill scan, healer scan, avatar distill, KB absorb, seed crawl, session recovery, crawl queue, architecture audit, novel ingest, constitution reload, second brain, loop readiness, market re-eval, telemetry, system health, game training — all defined in `run.rs`), a single slow handler (e.g., KB write or network crawl) can cascade-abort all others.

### D-SCON-007: No CancellationToken anywhere in the codebase — no cooperative cancellation
**Severity: HIGH**
**Source: Tokio docs on CancellationToken + graceful shutdown patterns**
**File: codebase-wide (zero matches for `CancellationToken`)**

NeoTrix has **zero** uses of `tokio_util::sync::CancellationToken` across the entire codebase. The Tokio graceful shutdown docs explicitly recommend: "CancellationToken is the broadcaster, JoinSet is the collector, timeout is the safety net." NeoTrix uses `watch::channel` in `BackgroundLoop` for shutdown signaling (mod.rs:52-61), but this is not the same:
- `watch::channel` is single-producer, requires `&mut self` access to send
- `CancellationToken` is cloneable, can be distributed to any number of tasks
- `CancellationToken` has `child_token()` for hierarchical cancellation (parent→child propagation)
- `CancellationToken` integrates with `tokio::select!` via `.cancelled()` future

The absence of CancellationToken means:
- No hierarchical shutdown (cancel parent → children auto-cancel)
- No way to cancel a specific task group without aborting all tasks
- No cooperative cancellation — tasks must poll a watch receiver, which requires `&mut` access to the receiver (hard to share)

### D-SCON-008: MultiAgentCoordinator parallel execution has no cancellation or timeout
**Severity: MEDIUM**
**Source: Tokio production patterns (timeout + cancellation)**
**File: `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/coordinator.rs:80-128`**

```rust
let mut handles = Vec::new();
for (agent_id, task_indices) in &allocation {
    for &ti in task_indices {
        handles.push(tokio::spawn(async move { ... }));
    }
}
for handle in handles {
    if let Ok(result) = handle.await {
        results.push(result);
    }
}
```

The `MultiAgentCoordinator::execute_tasks` method:
1. Spawns tasks into a `Vec<JoinHandle>` (not JoinSet)
2. Awaits them **sequentially** (same bug as D-SCON-003)
3. Has **no timeout** — a stuck reasoning provider blocks forever
4. Has **no cancellation** — if one agent fails, others continue unmonitored
5. Silently drops `Err` results (line 123: `if let Ok(result)`)

The correct pattern: spawn into JoinSet, wrap each task with `tokio::time::timeout`, use `join_next()` for completion-order processing, and handle `JoinError` explicitly.

### D-SCON-009: EventBus sync subscriber uses polling `try_recv` with `thread::sleep(10ms)` — busy-wait anti-pattern
**Severity: MEDIUM**
**Source: Tokio docs on async task efficiency**
**File: `neotrix-core/src/neotrix/nt_core_event_bus.rs:434-474`**

```rust
std::thread::spawn(move || {
    loop {
        if shutdown.load(Ordering::SeqCst) { break; }
        match rx.try_recv() {
            Ok(event) => { ... }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(std::time::Duration::from_millis(10)); // BUSY-WAIT
            }
            ...
        }
    }
})
```

The sync subscriber thread uses `try_recv()` + `thread::sleep(10ms)` polling — a busy-wait pattern that:
- Wastes CPU cycles even when no events arrive
- Has 10ms latency on event delivery
- Cannot be combined with other work on the same thread
- Violates the async runtime's cooperative scheduling model

This should be replaced with `tokio::spawn` + `rx.recv().await` (the async version already exists at line 362), or at minimum use `tokio::sync::mpsc` with blocking recv.

### D-SCON-010: HotReloadWatcher spawned task has no cancellation mechanism
**Severity: LOW**
**Source: Tokio production patterns (graceful shutdown)**
**File: `neotrix-core/src/unified/layers/action/nt_io/nt_io_hotreload/mod.rs:136-168`**

`HotReloadWatcher::spawn()` returns a `JoinHandle<()>` but:
- No CancellationToken is passed into the spawned task
- The task runs an infinite `loop` on `rx.recv().await` with no shutdown signal
- The only way to stop it is `handle.abort()`, which doesn't allow cleanup
- The `notify::Watcher` is held inside the task (`let _watcher = watcher;`), so aborting the task drops the watcher abruptly

The correct pattern: pass a CancellationToken into the task, use `tokio::select!` between `rx.recv()` and `token.cancelled()`, and ensure the watcher is dropped in a cleanup block.

### D-SCON-011: BackgroundLoop::spawn() pushes to Vec without tracking task identity — no per-task abort
**Severity: MEDIUM**
**Source: JoinSet AbortHandle pattern + TaskTracker token pattern**
**File: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:7-12`**

```rust
pub fn spawn<F>(&mut self, task: F) {
    self.handles.push(tokio::spawn(task));
}
```

Ad-hoc tasks spawned via `BackgroundLoop::spawn()` are pushed into the shared `Vec<JoinHandle>` with no identity tracking. This means:
- Cannot abort a specific task by ID
- Cannot check if a specific task is still running
- Cannot get the result of a specific task
- All ad-hoc tasks share the same 5-second shutdown deadline as coordinated handlers

A JoinSet would provide `AbortHandle` per task, enabling selective cancellation. A TaskTracker would provide `TaskTrackerToken` for fine-grained lifecycle tracking.

### D-SCON-012: Proxy/kernel modules spawn background tasks with no handle tracking — fire-and-forget leaks
**Severity: HIGH**
**Source: Tokio docs ("Spawning tasks without awaiting their completion via a join handle, or dropping those join handles" is unstructured)**
**File: Multiple — `nt_shield_proxy_kernel/kernel.rs:176-223`, `nt_shield_traffic/api_proxy.rs:353`, `nt_shield_traffic/mitm.rs:123`, `nt_shield_stealth_net/tor_crawler.rs:316-345`, `nt_shield_stealth_net/proxy_pool.rs:616`, `nt_io_web/api.rs:465,1098`, `nt_io_provider/gateway/mod.rs:547,1200`**

Multiple proxy and networking modules spawn tasks via `tokio::spawn` without storing the returned `JoinHandle`. Examples:
- `kernel.rs:176`: SOCKS5 listener spawned with no handle
- `kernel.rs:190`: HTTP listener spawned with no handle
- `api_proxy.rs:353`: Proxy accept loop spawned with no handle
- `mitm.rs:123`: MITM handler spawned with no handle
- `tor_crawler.rs:316`: Tor crawler tasks spawned with no handle
- `proxy_pool.rs:616`: Health check tasks pushed into a local Vec that is never drained

These fire-and-forget tasks:
- Cannot be gracefully shut down
- Cannot be monitored for completion
- May continue running after the parent module is dropped
- Can leak resources (open sockets, file descriptors) if not properly cleaned up

The Rust Async Book explicitly lists "Spawning tasks without awaiting their completion via a join handle, or dropping those join handles" as a violation of structured concurrency.

### D-SCON-013: BackgroundLoop shutdown drains handles without awaiting completion — race condition
**Severity: MEDIUM**
**Source: Tokio JoinSet docs ("abort_all does not remove tasks from the JoinSet")**
**File: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:51-61`**

```rust
for handle in self.handles.drain(..) {
    let abort = handle.abort_handle();
    tokio::select! {
        biased;
        _ = &mut deadline => { abort.abort(); }
        _ = handle => {}
    }
}
```

After `drain(..)`, the handles are moved out of the Vec. If the shutdown loop is interrupted (e.g., by a panic in one of the `handle.await` results), the remaining handles are dropped without being awaited. Dropped JoinHandles **detach** the tasks (they continue running as orphans), unlike JoinSet which aborts on drop. The correct pattern:
1. Call `abort_all()` first (sets abort flag on all tasks)
2. Then drain with `join_next()` in a loop (ensures all tasks actually finish)
3. JoinSet's `shutdown()` method does exactly this: "aborts all tasks and waits for them to finish shutting down"

### D-SCON-014: EventBus::shutdown() has 2-second hard deadline with thread detach — no structured completion
**Severity: MEDIUM**
**Source: Tokio production patterns (timeout on shutdown)**
**File: `neotrix-core/src/neotrix/nt_core_event_bus.rs:228-244`**

```rust
pub fn shutdown(&self) {
    self.shutdown_flag.store(true, Ordering::SeqCst);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    for h in handles.drain(..) {
        if std::time::Instant::now() > deadline {
            log::warn!("[event-bus] shutdown timeout reached, detaching remaining threads");
            break;
        }
        ...
    }
}
```

The EventBus shutdown:
1. Sets a flag and gives threads 2 seconds total (not per-thread)
2. On timeout, **detaches** remaining threads (they continue running as orphans)
3. No mechanism to ensure threads have actually processed their last event
4. The 2-second deadline is hardcoded, not configurable

With 9 subscriber threads (L1-L9) plus sync threads, a slow thread early in the drain loop can cause all subsequent threads to be detached.

### D-SCON-015: No JoinSet-based task group for consciousness tick handlers — consciousness loop has unstructured task spawning
**Severity: HIGH**
**Source: Structured concurrency chapter ("tasks organised into a tree") + JoinSet docs**
**File: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738,894`**

The background loop's consciousness tick handlers spawn tasks via `tokio::spawn` into the shared `handles: Vec<JoinHandle>`:
- Line 738: `self.handles.push(tokio::spawn(async move { ... }))`
- Line 894: `self.handles.push(tokio::spawn(async move { ... }))`

These consciousness-related tasks (GWT resonance, consciousness tick, FEP-IIT bridge, emotion restore) are spawned ad-hoc without structured lifecycle. If the consciousness tick handler panics, the spawned task becomes orphaned. If the background loop shuts down during a consciousness tick, the tick task may be mid-execution when aborted, leaving the consciousness state inconsistent.

A structured approach: use a dedicated `JoinSet` for consciousness tick tasks, with a CancellationToken that propagates to all tick subtasks, ensuring clean shutdown of the entire consciousness tree.

## Key Insights

1. **Complete absence of structured concurrency primitives**: NeoTrix has 92+ `tokio::spawn` calls and zero uses of `JoinSet` or `TaskTracker`. This is the single most impactful finding — the entire concurrent task management layer is unstructured.

2. **The Rust Async Book's structured concurrency checklist is violated at every level**:
   - "Child tasks start after their parents and always finish before them" — violated by fire-and-forget spawns
   - "Cancellation of parents is always propagated to child tasks" — no CancellationToken propagation
   - "Task should not outlive the function or block where it is created" — detached tasks in proxy/kernel modules

3. **TaskTracker is the correct primitive for NeoTrix's background handlers** — the BackgroundLoop spawns long-lived tasks whose results are never consumed (`JoinHandle<()>`). TaskTracker would:
   - Immediately free memory when tasks complete (vs JoinSet which holds results)
   - Allow `close()` + `wait()` for graceful shutdown
   - Be shareable via `clone()` across BackgroundLoop and BackgroundLoopHandle

4. **JoinSet is the correct primitive for batch operations** — ParallelExecutor, MultiAgentCoordinator, and proxy health checks should use JoinSet for:
   - Completion-order result processing
   - `abort_all()` for emergency shutdown
   - `shutdown()` for clean abort-and-wait

5. **CancellationToken is critical for hierarchical shutdown** — NeoTrix's 6-layer architecture (L1-L6) should have a CancellationToken tree where cancelling the root propagates to all layers, which propagate to their child tasks. The current `watch::channel` approach doesn't support this hierarchy.

6. **The shutdown deadline bug (D-SCON-006) is a production risk** — with 20+ background handlers sharing a 5-second deadline, a single slow KB write or network crawl can cascade-abort all handlers, potentially leaving the system in an inconsistent state.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 15 |
| Sources consulted | 13 |
| Codebase files analyzed | 12+ |
| tokio::spawn occurrences found | 92+ |
| JoinSet/TaskTracker uses found | 0 |
| CancellationToken uses found | 0 |
