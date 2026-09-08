# Agent 3: CancellationToken Patterns (Batch 872)

## Sources

1. **tokio-util CancellationToken docs** — `docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html` — Official API: `child_token()` for hierarchical shutdown, `cancelled()` future, `is_cancelled()` check, `DropGuard` for auto-cancel
2. **Tokio Graceful Shutdown tutorial** — `tokio.rs/tokio/topics/shutdown` — Recommended pattern: CancellationToken + JoinSet + signal handler
3. **tokio-graceful crate** — `github.com/plabayo/tokio-graceful` (145★) — Production shutdown with guard-based task tracking, delay drain, signal override
4. **tokio-graceful-shutdown crate** — `docs.rs/tokio-graceful-shutdown` — Subsystem-based hierarchical shutdown with error propagation
5. **cancellation-patterns repo** — `github.com/milosgajdos/cancellation-patterns` — Comparison: abort, oneshot, broadcast, watch, CancellationToken
6. **tokio-patterns skill** — `agentskills.so/skills/geoffjay-claude-plugins-tokio-patterns` — Graceful shutdown via broadcast coordinator, worker pattern, CancellationToken hierarchy
7. **Rust From Zero CancellationToken guide** — `rustz2h.com/chapter_07_mastering_async_rust_and_tokio/.../cancellation_token_guide` — Child token hierarchy, service shutdown pattern, one-way signal semantics
8. **Toolsku Graceful Shutdown article** — `toolsku.com/en/blog/rust-tokio-graceful-shutdown-2026` — 5 patterns: signal capture, JoinSet, connection draining, multi-service, ShutdownManager
9. **openclaw async-concurrency reference** — `github.com/openclaw/skills/.../async-concurrency.md` — Anti-patterns: blocking in async, select! cancel-safety, Send/Sync violations, CancellationToken best practices
10. **Tokio Discussion #1819** — `github.com/tokio-rs/tokio/discussions/1819` — Community pain points: task cancellation, JoinHandle ownership, error propagation during shutdown

## Defects

### D-CT-001: Zero CancellationToken usage across entire codebase
**File**: `neotrix-core/src/` (all modules) | **Severity**: HIGH | **Source**: Sources 1,2,5,7

The entire NeoTrix codebase uses zero `tokio_util::sync::CancellationToken`. All shutdown signaling uses either `Arc<AtomicBool>` (EventBus), `watch::channel<bool>` (BackgroundLoop, ProxyKernel), or ad-hoc `process::exit()`. CancellationToken provides: (a) hierarchical parent-child shutdown via `child_token()`, (b) `DropGuard` for RAII auto-cancel, (c) zero-cost cloning, (d) `is_cancelled()` without await. None of these benefits are available with the current patterns.

### D-CT-002: BackgroundLoop uses abort-based shutdown instead of cooperative cancellation
**File**: `nt_mind_background_loop/handlers.rs:52-57` | **Severity**: HIGH | **Source**: Sources 1,2,4,8

`BackgroundLoop::shutdown()` uses `handle.abort_handle().abort()` after a 5s deadline. This is ungraceful: tasks performing KB writes, file I/O, or network calls are killed mid-operation with no cleanup. The 35+ handler tasks spawned via `spawn_handler!` (run.rs:738-887) have no mechanism to observe a cancellation signal, save state, or complete in-flight operations. Production shutdown should use CancellationToken so handlers can observe `cancelled()`, flush pending writes, and exit cleanly.

### D-CT-003: EventBus Clone loses shutdown coordination state
**File**: `nt_core_event_bus.rs:40-51` | **Severity**: HIGH | **Source**: Sources 1,5,7

`EventBus::Clone` creates fresh empty `handles` and `sync_handlers` vectors: `handles: std::sync::Mutex::new(Vec::new())` and `sync_handlers: std::sync::Mutex::new(Vec::new())`. Cloned EventBus instances cannot be shut down cleanly — their threads are never joined. The `subscribe_all_layers_sync` threads registered on the original are invisible to clones. A CancellationToken tree would solve this: each clone shares the same token via Arc, and `child_token()` per subscriber ensures clean propagation.

### D-CT-004: EventBus synchronous subscriber uses busy-wait polling
**File**: `nt_core_event_bus.rs:464` | **Severity**: MEDIUM | **Source**: Sources 1,2,5

`subscribe_all_layers_sync()` spawns std::threads that poll with `std::thread::sleep(Duration::from_millis(10))` between `try_recv()` calls (line 464). This wastes CPU cycles and introduces up to 10ms latency per event. The tokio async subscriber (`subscribe_layer`) uses `rx.recv().await` which is event-driven. The sync path should use `tokio::sync::Notify` or a Condvar to block until events arrive, or migrate entirely to the async path.

### D-CT-005: EventBus shutdown uses hardcoded 2s deadline
**File**: `nt_core_event_bus.rs:232` | **Severity**: MEDIUM | **Source**: Sources 1,4,8

`EventBus::shutdown()` has a hardcoded `Duration::from_secs(2)` deadline for joining all threads. After 2s, remaining threads are detached (line 235-236). Production systems should use configurable drain timeouts with CancellationToken + timeout per thread, allowing each subsystem to define its own grace period. The current approach may orphan threads that are mid-write to the JSONL log file, causing data corruption.

### D-CT-006: ProxyKernel does not await DNS handler task
**File**: `nt_shield_proxy_kernel/kernel.rs:195-213,254-273` | **Severity**: MEDIUM | **Source**: Sources 2,3,8

The ProxyKernel spawns a DNS handler (`dns_handle`) at line 200 as `Option<JoinHandle>`, but the main select loop (lines 254-273) only monitors `socks5_handle` and `http_handle`. If the DNS handler crashes or hangs, it is never detected. During shutdown, the DNS handler may continue processing UDP packets after SOCKS5/HTTP have stopped. All three listener handles should be monitored in the main loop with proper CancellationToken propagation.

### D-CT-007: ProxyKernel drain loop uses sleep-polling instead of event notification
**File**: `nt_shield_proxy_kernel/kernel.rs:279-292` | **Severity**: MEDIUM | **Source**: Sources 1,3,8

The connection drain loop polls `active_connections` with `tokio::time::sleep(Duration::from_millis(100))` (line 291). This is reactive but wastes cycles. A CancellationToken pattern with `tokio::sync::Notify` would allow the drain to complete immediately when the last connection closes, without polling. The `ConnectionCountGuard` (line 600) should signal a Notify on decrement.

### D-CT-008: system_proxy.rs calls process::exit(0) bypassing all Drop handlers
**File**: `nt_shield_stealth_net/system_proxy.rs:128` | **Severity**: CRITICAL | **Source**: Sources 2,4,8,9

The system proxy signal handler calls `std::process::exit(0)` after restoring proxy settings. This kills the entire process immediately, skipping: (a) EventBus shutdown (JSONL flush), (b) BackgroundLoop shutdown (35+ handler cleanup), (c) ProxyKernel connection drain, (d) KB WAL checkpoint, (e) all Drop impls. This is the single most dangerous shutdown defect — any prior signal handler registration or ShutdownManager is completely bypassed. The self-review already flags this (PA009: `nt_core_self_review/mod.rs:652`), yet 17 `process::exit()` calls remain in production code paths.

### D-CT-009: 30+ fire-and-forget tokio::spawn without JoinHandle tracking
**File**: Multiple files (see grep results above) | **Severity**: MEDIUM | **Source**: Sources 2,3,9

At least 30 `tokio::spawn` calls across the codebase do not store the returned JoinHandle. Examples: `system_proxy.rs:121`, `http_client/mod.rs:295,302`, `tor_crawler.rs:316,345`, `nt_io_web/api.rs:465,1098`, `nt_io_provider/free_providers.rs:153,342,511,678`. These tasks run unmonitored — if they panic, the error is silently swallowed by tokio's default panic handler. During shutdown, these tasks cannot be waited on or aborted cleanly. Production code should use `JoinSet` or a task registry with CancellationToken.

### D-CT-010: BackgroundLoop spawn() creates untracked ad-hoc tasks
**File**: `nt_mind_background_loop/handlers.rs:5-12` | **Severity**: LOW | **Source**: Sources 1,2,4

`BackgroundLoop::spawn()` pushes JoinHandles into `self.handles` but the doc comment (line 6) explicitly states these tasks "will be aborted during shutdown without grace period." This is by design but still dangerous — any ad-hoc task performing KB writes or file operations will be killed mid-flight. The method should accept a CancellationToken parameter and pass it to the task, allowing cooperative shutdown within the 5s deadline.

### D-CT-011: ProxyKernel Drop sends shutdown signal but no drain coordination
**File**: `nt_shield_proxy_kernel/kernel.rs:594-597` | **Severity**: LOW | **Source**: Sources 1,3,7

`ProxyKernel::Drop` sends `true` on the watch channel but does not wait for active connections to drain. If the kernel is dropped (e.g., during process exit), the signal fires but the connection drain loop at line 279 is never reached. A proper RAII guard with CancellationToken would ensure drain completes before resources are released.

### D-CT-012: No hierarchical shutdown between BackgroundLoop and ProxyKernel
**File**: `nt_mind_background_loop/mod.rs` + `nt_shield_proxy_kernel/kernel.rs` | **Severity**: MEDIUM | **Source**: Sources 1,2,7

BackgroundLoop and ProxyKernel are completely independent subsystems with separate shutdown mechanisms (watch channel vs. watch channel). There is no parent CancellationToken coordinating their shutdown order. When the process exits, one may shut down while the other continues. The 6-layer architecture (L3 Embodiment for ProxyKernel, L5 Cognition for BackgroundLoop) demands hierarchical shutdown: a root token cancelled on SIGTERM propagates to child tokens for each layer's subsystem.

### D-CT-013: EventBus::subscribe_layer JoinHandle not tracked for shutdown
**File**: `nt_core_event_bus.rs:359-397` | **Severity**: LOW | **Source**: Sources 2,3,9

`subscribe_layer()` returns a `tokio::task::JoinHandle<()>` but the caller (`subscribe_all_layers` at line 401) collects them into a `Vec` that is immediately dropped (no binding). These 9 layer subscriber tasks run as detached fire-and-forget tasks with no shutdown coordination. The broadcast channel's `Closed` error (line 390) is the only way they stop — which only happens when the EventBus is dropped.

## Key Insights

1. **CancellationToken is the industry standard for Rust async shutdown**: Every production pattern (tokio official docs, tokio-graceful, tokio-graceful-shutdown) uses CancellationToken with hierarchical child tokens. NeoTrix uses none.

2. **watch::channel is flat, CancellationToken is hierarchical**: The current `watch::channel<bool>` pattern cannot express "cancel this subsystem but not its parent" or "cancel all children when parent cancels." CancellationToken's `child_token()` solves this natively.

3. **abort is the enemy of data integrity**: 35+ background handlers doing KB writes, file I/O, and network calls are aborted after 5s. Any in-flight SQLite transaction or file write will be corrupted. CancellationToken enables cooperative shutdown where handlers observe cancellation and flush pending work.

4. **process::exit() is a shutdown atom bomb**: 17 instances of `process::exit()` bypass all graceful shutdown logic. The system_proxy signal handler at line 128 is particularly dangerous because it registers first (via SIGTERM handler) and kills the process before other handlers can run.

5. **The 2-layer shutdown model is insufficient**: NeoTrix has L3 (ProxyKernel) and L5 (BackgroundLoop) as independent shutdown domains. A proper 6-layer architecture needs a root CancellationToken at L6 (Meta) that propagates down through all layers, matching the ConsciousnessTree hierarchy.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 13 |
| CRITICAL | 1 |
| HIGH | 3 |
| MEDIUM | 5 |
| LOW | 4 |
| Sources consulted | 10 |
| Key recommendation | Adopt `tokio_util::sync::CancellationToken` with hierarchical `child_token()` for all subsystem shutdown; eliminate `process::exit()` calls; replace abort-based shutdown with cooperative cancellation |
