# Agent 3: Async Cancellation Safety (Batch 864)

## Sources
1. Oxide RFD 400 — Dealing with cancel safety in async Rust (rfd.shared.oxide.computer)
2. Tokio Graceful Shutdown docs (tokio.rs/tokio/topics/shutdown)
3. Rain — "Cancelling Async Rust" RustConf 2025 (sunshowers.io)
4. CancellationToken API docs (docs.rs/tokio-util)
5. JoinSet API docs (docs.rs/tokio)
6. "Async & Observability: Cancellation Patterns That Actually Work" (medium.com/rustaceans, 2026-02)
7. "Cancellation Safety in different Rust AsyncRuntimes" (bshn.rs, 2026-04)
8. "8 Tokio Patterns" (elitedev.in, 2026-07)

## Defects

**D-CANCEL-001: No CancellationToken adoption — hierarchical shutdown missing** | `nt_mind_background_loop/run.rs:388` | HIGH | Sources 1,2,4,6
The BackgroundLoop uses a custom `ShutdownCoordinator` wrapping `tokio::sync::watch::Receiver<bool>` for shutdown signaling. This misses `tokio_util::sync::CancellationToken`'s hierarchical `child_token()` support, which would let subsystems (proxy kernel, crawl pipelines, LLM providers) independently cancel their sub-tasks when a parent is cancelled. The current flat broadcast model means a single `send(true)` cannot propagate cancellation to nested task trees — each subsystem must manually subscribe to the same watch channel.

**D-CANCEL-002: JoinSet not used — raw Vec\<JoinHandle\> loses structured concurrency** | `nt_mind_background_loop/run.rs:738` | HIGH | Sources 2,5,6,8
The `spawn_handler!` macro pushes into `self.handles: Vec<JoinHandle<()>>`. Tokio's `JoinSet` provides `abort_all()`, `shutdown()`, and `detach_all()` — all missing here. The manual `for handle in self.handles.drain(..)` loop with `abort_handle()` per-task in `handlers.rs:51-61` is fragile: if the loop panics mid-iteration, remaining handles are never aborted. JoinSet's Drop automatically aborts all contained tasks, providing a safety net the current code lacks.

**D-CANCEL-003: EventBus sync threads use try_recv polling loop — not cancellation-safe** | `nt_core_event_bus.rs:440-474` | MEDIUM | Sources 1,6
`subscribe_all_layers_sync` spawns `std::thread`s that poll via `rx.try_recv()` with a 10ms sleep on `Empty`. This is a synchronous busy-wait that cannot participate in async cancellation. The `shutdown_flag: Arc<AtomicBool>` is the only shutdown mechanism — there's no drop guard or RAII pattern to ensure in-flight event processing completes before thread exit. If a sync handler is mid-processing when `shutdown()` fires, its work is silently abandoned.

**D-CANCEL-004: Dropped JoinHandles throughout codebase — fire-and-forget tasks untrackable** | Multiple files (63 tokio::spawn sites, ~40 drop JoinHandle) | HIGH | Sources 1,2,5
Approximately 40 `tokio::spawn` calls across the codebase (e.g., `nt_core_llm.rs:60`, `nt_io_download/engine.rs:135`, `nt_io_web/api.rs:465`, `nt_io_provider/gateway/mod.rs:547`) drop the returned `JoinHandle` immediately. These tasks become detached — they cannot be aborted, joined, or monitored during shutdown. A crash or hang in any of these tasks is invisible. Per Oxide RFD 400: "dropping a handle does not cancel the task" — these tasks will run to completion even after the system intends to shut down.

**D-CANCEL-005: SOCKS5 handler uses read_exact — not cancel-safe, can corrupt protocol state** | `nt_shield_proxy_kernel/listener/socks5.rs:76-80` | MEDIUM | Sources 1,3,7
`handle_socks5_connection` performs multiple sequential `stream.read_exact(&mut buf).await` calls. Per Tokio docs, `read_exact` is explicitly **not** cancel-safe — if dropped mid-read, the partial bytes consumed from the stream are lost, leaving the SOCKS5 protocol parser in an inconsistent state. A timeout or cancellation during the method negotiation phase (line 76-80) would corrupt the stream for any subsequent reconnection attempt on the same socket.

**D-CANCEL-006: EventBus shutdown race — deadline checked after lock acquisition** | `nt_core_event_bus.rs:232-242` | MEDIUM | Sources 1,6
The `EventBus::shutdown()` acquires `self.handles.lock()`, then iterates with a deadline check. If the lock is contended (e.g., a sync handler is registering a new subscriber), the 2-second deadline may already be partially consumed before iteration begins. More critically, the `break` on deadline expiry detaches remaining thread handles — those threads continue running with no notification, potentially holding locks or writing to shared state after the EventBus is considered shut down.

**D-CANCEL-007: std::process::exit(0) in signal handler bypasses all Drop/async cleanup** | `nt_shield_stealth_net/system_proxy.rs:128` | HIGH | Sources 1,6
The SIGTERM/SIGINT handler calls `std::process::exit(0)` immediately after `proxy.disable().await`. This kills the entire process without running Drop impls on any remaining values — including open database connections, KB WAL files, EventBus state, and in-flight background handler writes. Per Oxide RFD 400 and Tokio docs, graceful shutdown requires draining in-flight work before exit. The `exit()` call defeats the entire cooperative cancellation model.

**D-CANCEL-008: Background loop shared deadline — early handlers consume budget** | `nt_mind_background_loop/handlers.rs:48-61` | LOW | Sources 1,6,8
The shutdown loop creates a single `deadline` future and shares it across all handler joins via `tokio::pin!(deadline)`. Each `tokio::select!` races the handle against `&mut deadline`. If handler A takes 4.9s to stop, the deadline fires during handler B's select, giving B only ~100ms. This creates an unfair shutdown where handler ordering determines who gets grace time. A per-handler deadline or sequential drain with total timeout would be more predictable.

**D-CANCEL-009: Absorption stdin write spawned task not cancellation-tracked** | `nt_mind_background_loop/handlers_absorption.rs:181-185` | LOW | Sources 1,4
A `tokio::spawn` writes to child stdin and calls `stdin.shutdown().await`, but the JoinHandle is dropped. If the absorption process is killed mid-write (e.g., the 600s timeout at line 187 fires), this spawned task continues writing to a dead pipe — `write_all` will return an error but the task has no way to know the parent already timed out. The task is effectively orphaned.

**D-CANCEL-010: ParallelExecutor sequential mode skips cancellation entirely** | `nt_core_parallel/executor.rs:30-31` | LOW | Sources 1,5
In `ExecMode::Sequential`, tasks are iterated synchronously with `.map()` — no async runtime involvement. This means sequential execution cannot be cancelled at all: the iterator runs to completion with no await points where a shutdown signal could be observed. A long-running sequential batch will block shutdown indefinitely until the process is force-killed.

## Key Insights

1. **The codebase has zero `CancellationToken` usage** — it relies entirely on `watch::Receiver<bool>` and `AtomicBool` for shutdown signaling. This is the single largest gap: `CancellationToken` provides hierarchical cancellation, drop guards, and `run_until_cancelled` — none of which are available with the current approach.

2. **Fire-and-forget spawns are pervasive** — ~40 of ~63 `tokio::spawn` calls drop the JoinHandle. This creates a large surface area of untrackable background work that cannot be participating in structured shutdown.

3. **The EventBus has two separate shutdown paths** (async via `shutdown_flag` + thread join, and Drop-triggered) that race against each other. Neither path ensures in-flight handlers complete their work.

4. **`std::process::exit(0)` in the signal handler is the most dangerous pattern** — it makes all other shutdown coordination irrelevant by killing the process before Drop runs.

5. **No drop guards are used anywhere** for async cleanup. Per best practices (Sources 1,6), every async operation that modifies state should have a drop guard that compensates if cancelled. NeoTrix has none.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| HIGH severity | 4 |
| MEDIUM severity | 3 |
| LOW severity | 3 |
| Sources consulted | 8 |
