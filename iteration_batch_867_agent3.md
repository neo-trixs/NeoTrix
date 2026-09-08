# Agent 3: Async State Machine (Batch 867)

## Sources
1. https://sharpskill.dev/en/blog/rust/rust-async-await-tokio-futures-concurrency — Rust Async/Await deep dive: lazy state machine, Pin/Unpin, zero-cost compilation
2. https://doc.rust-lang.org/stable/book/ch17-01-futures-and-syntax.html — Official Rust Book: async desugaring to state machine enums, await points as variants
3. https://www.stanza.dev/courses/rust-async/future-trait/rust-async-state-machines — Async state machines: enum size = max variant, drop-before-await optimization
4. https://microsoft.github.io/RustTraining/async-book/ch05-the-state-machine-reveal.html — State machine reveal: conceptual desugaring, self-referential Pin requirement
5. https://microsoft.github.io/RustTraining/async-book/ch06-building-futures-by-hand.html — Manual Future: state + poll() + waker registration, join/select combinators
6. https://rustz2h.com/chapter_07_mastering_async_rust_and_tokio/series_02_futures_streams_and_async_combinators/manual_future_implementation — Manual Future impl for specialized I/O, performance-critical code
7. https://reintech.io/blog/understanding-implementing-futures-rust — Future trait: Poll::Pending requires waker registration, executor never busy-waits
8. https://microsoft.github.io/RustTraining/rust-patterns-book/ch16-asyncawait-essentials.html — Common async pitfalls: blocking the runtime, forgetting waker registration
9. https://goals.rust-lang.org/2026/async-statemachine-optimisation.html — Async bloat: state machines can 2x binary size, no MIR optimization for async yet
10. https://users.rust-lang.org/t/on-state-machines/114910 — Type-state pattern vs runtime state machines, compile-time transition checks

## Defects

D-FUTURE-001: `ObfuscatedStream::poll_read` inner read loop does not register Waker when read_buf is empty but inner poll returns `Poll::Pending` — the `read_inner` method at `security.rs:418` returns `Poll::Pending` which propagates correctly, BUT the outer `poll_read` loop at line 498 calls `self.read_inner(cx)` then checks `read_buf.is_empty()` at line 500 — if inner returns `Ready(Ok(()))` with zero bytes AND read_buf has partial frame data, the loop re-enters without yielding Pending, risking busy-spin on empty reads | `security.rs:498-506` | HIGH | Source: Future trait requires `Poll::Pending` only when blocked on external event; busy-loop on empty inner read wastes CPU (Microsoft RustTraining Ch16)

D-FUTURE-002: `ObfuscatedStream::poll_write` silently buffers all data in `write_buf` (line 440) without any backpressure — `poll_write` returns `Poll::Ready(Ok(buf.len()))` unconditionally, meaning an infinite stream of writes will grow `write_buf` without bound until OOM | `security.rs:435-441` | HIGH | Source: State machine pattern requires bounded buffers; async state machines with unbounded buffering violate flow control (Stanza course: "large stack allocations blow up future size")

D-FUTURE-003: BackgroundLoop spawns 30+ independent `tokio::spawn` tasks via `spawn_handler!` macro (lines 779-839) without any Semaphore or JoinSet limiting concurrency — each handler acquires a `Mutex<BackgroundLoopInner>` lock, creating a convoy effect where slow handlers block all other handlers from acquiring the same lock | `run.rs:738-839` | HIGH | Source: "Tokio spawn + Semaphore suits dynamic collections; picking wrong primitive avoids under-utilization" (SharpSkill: async/await concurrency guide)

D-FUTURE-004: `XtlsStream<T>` manual `AsyncRead` implementation (lines 266-298) allocates a temporary `ReadBuf` on each `poll_read` call (`tokio::io::ReadBuf::new(&mut self.read_buf)`) but never clears `self.read_buf` between calls — stale encrypted data from a prior poll persists in the buffer, causing potential frame misalignment and decryption failure on subsequent reads | `vless/mod.rs:273-297` | HIGH | Source: Manual Future implementations must manage state machine transitions explicitly; the desugared state machine must drop values no longer needed at state transitions (Microsoft Ch5: "compiler inserts drops at state transitions")

D-FUTURE-005: `BackgroundLoop::shutdown` (handlers.rs:23-64) iterates `self.handles.drain(..)` with a shared `deadline` future — but `tokio::select!` with `biased` means the FIRST unhandled task gets the deadline, not all tasks. After the first abort, the deadline future is consumed and remaining tasks get NO timeout enforcement | `handlers.rs:51-61` | MEDIUM | Source: `select!` polls the first ready branch; the deadline future is not reusable across multiple `select!` invocations (Rust async book Ch17: select takes first future to finish)

D-FUTURE-006: `handlers_absorption.rs:181` spawns a `tokio::spawn` for stdin write with a fire-and-forget JoinHandle (`let _ = stdin.write_all(...)`) — if the child process crashes before consuming stdin, the spawned task hangs indefinitely holding the Tokio runtime task slot | `handlers_absorption.rs:181-185` | MEDIUM | Source: "A future must register a Waker before returning Pending; drop = cancel" (Microsoft Ch2: futures); spawned tasks that block forever leak runtime capacity

D-FUTURE-007: `ObfuscatedStream` `AsyncRead::poll_read` (line 484) uses `self.read_buf.split_to()` which returns `BytesMut` values that are immediately discarded (`let _ = ...`) — these allocations are not pooled or reused, causing per-frame heap churn in the hot path of the encrypted proxy kernel | `security.rs:484-486` | MEDIUM | Source: "async state machines compile to stack-allocated enums with no heap allocation per future" (Stanza course); manual implementations must minimize allocations in poll methods

D-FUTURE-008: The `spawn_handler!` macro (line 734-767) spawns tasks that lock `BackgroundLoopInner` via `tokio::sync::Mutex` for the duration of each handler tick — if a handler takes longer than its interval (e.g., `handle_save` with heavy KB writes), the next tick still fires but blocks on lock acquisition, creating cascading delays across ALL handlers sharing the same lock | `run.rs:745-751` | MEDIUM | Source: State machine transitions must be non-blocking; holding a lock across an await point creates implicit state coupling between state machine variants (Rust patterns: async/await pitfalls)

D-FUTURE-009: `ObfuscatedStream` does not implement `FusedFuture` or any mechanism to detect if the inner stream has already returned EOF — on a second `poll_read` after the inner stream closed, `read_inner` returns `Ready(Ok(()))` with n=0 (line 500-501), which is treated as "no data yet" rather than "stream terminated", potentially causing infinite re-polling of a dead connection | `security.rs:498-506` | MEDIUM | Source: "FusedFuture is required for select! to avoid polling completed futures" (futures-rs docs); manual Stream/Future implementations should track terminal state

D-FUTURE-010: `ProxyKernel::start` (kernel.rs:152) consumes `self` by value via `pub async fn start(self)` then immediately wraps in `Arc::new(self)` — this means the kernel's fields (config, router, pool, security, state, shutdown_tx) are immutably shared across all spawned listener tasks, but `state` is behind `Arc<RwLock>` requiring async lock on every state check, creating a hot contention point during startup and shutdown | `kernel.rs:152-163` | LOW | Source: "Pin<&mut Self> guarantees future won't be moved; async state machines may contain self-referential fields" (Microsoft Ch5); the Arc<RwLock> pattern works but adds overhead to every poll cycle

## Key Insights

1. **State machine bloat is the hidden cost**: NeoTrix's 30+ background handlers each compile to independent state machines. With the `Mutex<BackgroundLoopInner>` shared across all of them, each state machine variant stores the lock guard, inflating every future's size by the guard's size. Dropping the lock before `.await` points would shrink each state machine.

2. **Missing backpressure in the write path**: The `ObfuscatedStream::poll_write` pattern of "buffer unconditionally, flush later" is a classic async antipattern. The Rust async model is pull-based (poll-driven), but this write path is push-based with no flow control, violating the core contract.

3. **Waker correctness in manual implementations**: Both `ObfuscatedStream` and `XtlsStream` correctly propagate `cx` to inner poll calls (so wakers are registered), but the outer `poll_read` loop in `ObfuscatedStream` can spin if the inner stream returns `Ready(Ok(()))` with zero bytes repeatedly — this is a subtle busy-wait that wastes CPU without yielding.

4. **Select deadline sharing bug**: The shutdown handler's use of a pinned deadline across multiple `select!` invocations is incorrect — `tokio::pin!` creates a single future that can only be polled to completion once. After the first task is aborted against the deadline, the remaining tasks get no timeout.

5. **No future size monitoring**: The codebase uses `Box::pin` in only 5 places (backend_router and proxy_pool), suggesting most futures are stack-allocated. Given the 30+ handlers with heavy state (KB handles, consciousness tree, event bus), some futures may be dangerously large for Tokio's default stack (2MB). The async state machine bloat research shows 2x binary size impact without MIR optimization.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| High severity | 4 |
| Medium severity | 5 |
| Low severity | 1 |
