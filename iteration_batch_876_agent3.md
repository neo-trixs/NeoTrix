# Agent 3: Async Cancellation Safety (Batch 876)

## Sources

1. **Tokio select! docs** — `docs.rs/tokio/latest/tokio/macro.select.html` — Official cancellation safety documentation, lists safe/unsafe methods
2. **Oxide RFD 400** — `rfd.shared.oxide.computer/rfd/0400` — Comprehensive cancel safety guide for async Rust APIs; reserve pattern, FutureLock bug, cooperative cancellation
3. **Tokio tutorial select** — `tokio.rs/tokio/tutorial/select` — Tutorial on select! patterns, fairness, pinning futures
4. **Sunshowers "Cancelling async Rust"** — `sunshowers.io/posts/cancelling-async-rust/` — Cancel safety vs cancel correctness framework; pin-future-resume pattern
5. **RustConf 2025 "Cancelling Async Rust"** — `youtube.com/watch?v=zrv5Cy1R7r4` — Talk covering cancel correctness three-prong framework, reserve pattern, task-based isolation
6. **Stanza cancellation safety course** — `stanza.dev/courses/rust-async/async-control-flow/rust-async-cancellation` — Patterns: pre-read in select, atomic sub-operations, move state outside
7. **Rust forum: cancellation safety of locks in select** — `users.rust-lang.org/t/cancellation-safety-of-locks-in-select/137188` — FutureLock bug: pinning mutex lock doesn't fix `.await` in handler
8. **Barafael "Stop Worrying and Learn to Loop-Select"** — `barafael.github.io/posts/stop-worrying-and-learn-to-loop-select/` — 4 issues: cancel safety, hidden panic, busy loop, tooling
9. **Biriukov "Async Rust gotcha: evolving tokio::select!"** — `biriukov.dev/posts/async-rust-gocha-tokio-cancelation-select-future-then/` — FutureExt::then() cancel-unsafe, StreamExt::then() safer, reserve() pattern for backpressure
10. **Rust forum "Is tokio's select! cancel safe?"** — `users.rust-lang.org/t/is-tokios-select-cancel-safe/138050` — Nested select! cancel safety; invisible implementation details
11. **cancel-safe-futures crate** — `docs.rs/cancel-safe-futures/latest/cancel_safe_futures/coop_cancel/` — Cooperative cancellation channel pattern
12. **Rust skills: async-cancel-safety rules** — `github.com/leonardomso/rust-skills/rules/async-cancel-safety.md` — Summary table of safe/unsafe ops, pin-future pattern
13. **OpenAI codex patterns: biased-select-for-cancellation** — `github.com/pproenca/dot-skills` — biased; when cancel must win ties; active unwind pattern

## Defects

**D-CSAFE-001**: Background loop `spawn_handler!` macro locks `tokio::sync::Mutex` inside `select!` branch handler body (line 745: `h.lock().await`). If the `rx.changed()` shutdown branch wins while the handler is awaiting the lock, the handler future is dropped mid-lock-acquire, losing its place in the Mutex queue. The handler's body (e.g., `handle_save`, `handle_consolidate`) may hold state mutations that are silently abandoned. | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:742-757` | HIGH | Sources 1, 6, 7

**D-CSAFE-002**: EventBus behavioral consumer (line 896-914) uses `biased;` select with `event_rx.recv()` followed by `h.lock().await` inside the handler. `tokio::sync::Mutex::lock` is explicitly documented as NOT cancellation-safe (queue position lost on drop). If the shutdown branch wins while the consumer holds the mutex guard, the lock is dropped but the event has already been consumed from the broadcast channel — the event is silently lost. | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:896-914` | HIGH | Sources 1, 6, 7

**D-CSAFE-003**: DNS intercept loop (line 127-148) acquires `tokio::sync::Mutex` (`rate_limiter.lock().await`) inside a `select!` branch. If the `shutdown_rx.changed()` branch fires while the rate limiter lock is being acquired, the lock future is dropped, losing queue position. Under high DNS query load, repeated cancellation of the lock acquire can starve the rate limiter indefinitely. | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/dns_intercept.rs:127-148` | MEDIUM | Sources 1, 6, 7

**D-CSAFE-004**: Proxy kernel bidirectional copy (line 546-565) uses `select!` over `tokio::io::copy` in both directions plus shutdown. `tokio::io::copy` internally calls `read` then `write` — while individual `read`/`write` are cancel-safe, `copy` as a composite operation is NOT cancellation-safe because a partial copy may have read bytes into an internal buffer that are lost when the future is dropped. On shutdown signal, in-flight copies are dropped, potentially losing buffered bytes. | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:546-565` | MEDIUM | Sources 1, 4, 12

**D-CSAFE-005**: The `spawn_handler!` macro pattern creates new `tokio::time::interval` tickers inside each spawned task but uses `biased;` select with shutdown as the second branch. When the ticker branch wins repeatedly (high-frequency handlers like `always_on`), the shutdown branch may be starved due to biased polling always checking ticker first. This violates the biased select caveat documented by Tokio: "it becomes your responsibility to ensure that the polling order of your futures is fair." The shutdown signal delay under load can cause handler to run one extra iteration after shutdown is requested. | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:742-757` | LOW | Sources 1, 3, 13

**D-CSAFE-006**: No cancellation safety annotations exist anywhere in the NeoTrix codebase. None of the 11 `select!` usages have comments documenting whether the selected futures are cancel-safe. The Oxide RFD 400 recommends: "each method have a 'cancel safety' section associated with it." Without these annotations, future developers cannot determine whether refactoring introduces cancel-safety bugs. The Tokio docs themselves were only updated to document `Sender::send` as cancel-unsafe in version 1.33 — NeoTrix has no equivalent documentation burden. | Codebase-wide (11 select! sites) | MEDIUM | Sources 2, 4, 12

**D-CSAFE-007**: `nt_memory_resource_ingest.rs` uses synchronous `std::io::Read::read_exact` (line 588) and `std::fs::File::write_all` (line 589) in a copy loop. While currently synchronous (not inside async select), if this code is ever refactored to be async or called from an async context within a select branch, `read_exact` and `write_all` are both documented as NOT cancellation-safe — partially-filled buffers are lost on cancellation. The pattern is a latent trap. | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_resource_ingest.rs:584-594` | LOW | Sources 1, 12

**D-CSAFE-008**: Background loop handlers acquire `tokio::sync::Mutex` (BackgroundLoopHandle) in every handler tick (line 745). The handle mutex is shared across ~20+ spawned tasks. With `biased;` select, the handler always polls the ticker first, but if multiple handlers are trying to acquire the same mutex concurrently, they form a queue. Any cancellation of a handler task (via `abort_handle()` in shutdown, line 52-60) drops the handler from the queue, but if the handler had already begun processing state mutations before the abort, those mutations may be partially applied. The shutdown path (handlers.rs:53-60) uses `biased;` select with deadline, meaning after 5s all remaining handlers are force-aborted via `AbortHandle`, which is a forceful (not cooperative) cancellation. | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:51-61` | HIGH | Sources 2, 4, 5

**D-CSAFE-009**: `stdin.write_all(payload.as_bytes()).await` in handlers_absorption.rs (line 183) is called inside a spawned task but the write is to a child process stdin. `AsyncWriteExt::write_all` is documented as NOT cancellation-safe by Tokio. If the child process is killed or the stdin pipe closes mid-write, the `write_all` future is dropped with potentially partial data written. The payload (experience-tree absorption JSON) could be truncated, causing the absorption pipeline to receive malformed input. | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers_absorption.rs:181-184` | MEDIUM | Sources 1, 4

**D-CSAFE-010**: `neotrix_dl.rs` download engine (line 122) calls `file.write_all(&chunk).await?` inside a `while let Some(chunk) = stream.next().await` loop. While this isn't inside a select!, the download is not wrapped in a cooperative cancellation mechanism. If the download task is aborted (e.g., via Ctrl+C or JoinHandle::abort), `write_all` may have partially written the chunk, leaving the file in an inconsistent state with a corrupted partial write. No checksum or atomic write mechanism exists to detect/recover from this. | `neotrix-core/src/bin/neotrix_dl.rs:120-132` | LOW | Sources 1, 4, 5

**D-CSAFE-011**: Proxy kernel shutdown handler (kernel.rs:259-265) uses `select!` over `error_rx.recv()`, `socks5_handle`, and `http_handle`. The `socks5_handle` and `http_handle` are `JoinHandle`s — when a JoinHandle is dropped (not awaited), the underlying task continues running (this is correct). However, the handler only breaks out of the select without awaiting the remaining handles, meaning background connection handling tasks may outlive the kernel's state being set to `Failed`. This is a cancellation-ordering issue: the state transition to `Failed` happens before the listener tasks are fully drained. | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:259-265` | LOW | Sources 2, 4

**D-CSAFE-012**: Traffic analyzer Mutex (`tokio::sync::Mutex<TrafficAnalyzer>`) is locked inside HTTP request handlers (api_proxy.rs:172, mitm.rs:266,325,355,379) which are called from within spawned tasks. If a client disconnects mid-request while the analyzer lock is held, the handler future is dropped, releasing the lock. However, the analyzer may have already partially recorded the request (e.g., `capture_request` at mitm.rs:267) but not the corresponding response, leaving the traffic analysis in an inconsistent state with orphaned request records. | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:265-268` | MEDIUM | Sources 2, 4, 6

## Key Insights

1. **The `spawn_handler!` macro is the single highest-risk pattern**: It uses `biased;` select with `tokio::sync::Mutex` acquisition inside the handler body. This is the exact anti-pattern documented by Tokio as not cancellation-safe. With 20+ handlers sharing a mutex, queue starvation under shutdown is almost certain.

2. **Cooperative cancellation is missing**: NeoTrix uses `JoinHandle::abort()` (forceful) for shutdown instead of cooperative cancellation channels (`CancellationToken` + explicit break). The Oxide RFD 400 and RustConf 2025 talk both recommend cooperative cancellation to avoid invariant violations.

3. **No cancel-safety audit has been performed**: None of the 11 `select!` sites have documentation about the cancel-safety properties of their selected futures. This is a systemic gap that makes every future refactor a potential data-loss bug.

4. **The `biased;` usage is inconsistent**: Some select! blocks use `biased;` (run.rs:743,897, handlers.rs:54) while others don't (kernel.rs:225,259,546, socks5.rs:40, http.rs:41, dns_intercept.rs:127, system_proxy.rs:122). The choice appears arbitrary rather than deliberate.

5. **Latent async traps in synchronous code**: `read_exact`, `write_all`, and `read_to_end` are used extensively in synchronous contexts throughout the codebase. If any of these are ever lifted into async contexts or called from select branches, they become cancellation-unsafe. The codebase has no guardrails against this evolution.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Sources consulted | 13 |
| select! sites analyzed | 11 |
| tokio::sync::Mutex import sites | 12 |
| High-severity defects | 3 |
| Medium-severity defects | 4 |
| Low-severity defects | 5 |
