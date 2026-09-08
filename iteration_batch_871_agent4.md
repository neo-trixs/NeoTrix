# Agent 4: Structured Concurrency (Batch 871)

## Sources

1. Tokio JoinSet documentation — `docs.rs/tokio/latest/tokio/task/join_set/struct.JoinSet.html`
2. Rust Async Book — Structured Concurrency — `rust-lang.github.io/async-book/part-reference/structured.html`
3. Tokio TaskTracker documentation — `docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html`
4. Rust Cookbook — Join Sets — `rust-lang-nursery.github.io/rust-cookbook/asynchronous/join.html`
5. Tokio source: `join_set.rs` — `github.com/tokio-rs/tokio/blob/5030b300/tokio/src/task/join_set.rs`
6. Structured Concurrency in Rust with Tokio — Medium article by Adam Szpilewicz (2025-04-22)
7. Structured Concurrency in Tokio: Safe Task Management — `rustz2h.com`
8. Rust FAQ — How to implement structured concurrency in Rust — `rustfaq.org`
9. Production Patterns (Microsoft Rust Training) — `microsoft.github.io/RustTraining/async-book/ch13-production-patterns.html`
10. Tokio Graceful Shutdown — `tokio.rs/tokio/topics/shutdown`
11. Async Runtime Patterns — `rust-patterns.com/book/15-async-runtime-patterns.html`
12. `structured_spawn` crate — `docs.rs/structured-spawn`

## Defects

D-SCON-001: **Zero JoinSet usage across 62+ tokio::spawn sites** — The entire NeoTrix codebase has zero `JoinSet` instances. All 62+ `tokio::spawn` calls use fire-and-forget or `Vec<JoinHandle>` collection, which is the unstructured concurrency anti-pattern. Structured concurrency requires parent scope ownership of child lifetimes. | `neotrix-core/src/` (62 sites) | high | Source 1, 2, 5

D-SCON-002: **BackgroundLoop stores handles as `Vec<JoinHandle<()>>` instead of `JoinSet<()>`** — `mod.rs:96` uses `pub handles: Vec<JoinHandle<()>>` to track spawned handler tasks. A `Vec<JoinHandle>` is not a structured concurrency primitive: dropping it does NOT cancel tasks (unlike `JoinSet` which aborts on drop). The 5-second manual abort loop in `handlers.rs:48-61` is a workaround for this missing semantic. | `nt_mind_background_loop/mod.rs:96` | high | Source 1, 6

D-SCON-003: **40+ fire-and-forget `tokio::spawn` without JoinHandle capture** — At least 40 spawn sites drop the returned `JoinHandle` immediately, creating orphan tasks that can never be awaited, cancelled, or monitored. Examples: `nt_core_event_bus.rs:362`, `nt_io_provider/openai.rs:234`, `nt_io_provider/anthropic.rs:257`, `nt_io_provider/gemini.rs:155`, `nt_io_provider/ollama.rs:135`, `nt_io_mail/self_heal.rs:904`, `nt_shield_sandbox/remote.rs:170`, `nt_shield_traffic/api_proxy.rs:353`, `nt_shield_traffic/mitm.rs:123`, `nt_shield_stealth_net/stealth_browser.rs:113`, `nt_shield_stealth_net/proxy_control.rs:130`, `nt_shield_proxy_kernel/kernel.rs:220`, `nt_io_web/api.rs:465,1098`, `nt_io_hotreload/mod.rs:168`, `reasoning_engine/engine_core.rs:1909`, `handlers_absorption.rs:181`, `nt_mind_background_loop/run.rs:894`. Per the Async Book, "spawning tasks without awaiting their completion via a join handle, or dropping those join handles" is a structured concurrency violation. | 40+ sites | high | Source 2

D-SCON-004: **Parallel executor sequentially awaits each spawned task** — `nt_core_parallel/executor.rs:37-43` spawns a task then immediately `.await`s the handle before spawning the next. This means tasks execute sequentially despite being on separate threads. The entire point of `JoinSet` (or even `futures::join_all`) is to poll multiple futures concurrently. This code defeats parallelism entirely. | `nt_core_parallel/executor.rs:37-43` | high | Source 1, 11

D-SCON-005: **MultiAgentCoordinator silently swallows JoinError from panicked tasks** — `nt_core_parallel/coordinator.rs:122-126` does `if let Ok(result) = handle.await { results.push(result); }`. When a task panics, `handle.await` returns `Err(JoinError)` which is silently dropped. A panicking reasoning agent would produce zero results with no error signal. Structured concurrency requires explicit error propagation to parent. | `nt_core_parallel/coordinator.rs:122-126` | high | Source 1, 2

D-SCON-006: **No CancellationToken integration for graceful shutdown** — The `BackgroundLoop` uses a `watch::channel<bool>` (`ShutdownCoordinator`) for shutdown signaling. While functional, this is not the idiomatic structured concurrency pattern. `tokio_util::sync::CancellationToken` (Source 10) provides hierarchical cancellation propagation, cloneable tokens for child tasks, and `select!`-compatible `cancelled()` future. The current watch channel approach requires each handler to clone a `watch::Receiver` and manually check `rx.changed()`, which is fragile and doesn't compose with `JoinSet`'s abort-on-drop semantics. | `nt_mind_background_loop/mod.rs:52-61` | medium | Source 7, 10

D-SCON-007: **`select!` macro in handler loops creates unstructured cancellation** — `run.rs:742` uses `tokio::select! { biased; _ = ticker.tick() => { ... }, _ = rx.changed() => { break; } }`. Per the Async Book: "Select or race macros/functions are not inherently structured, but since they abruptly cancel futures, it's a common source of unstructured cancellation." When the shutdown branch wins, the tick branch future is dropped mid-execution, potentially leaving shared state inconsistent. There is no cancellation token propagation to sub-tasks spawned within the tick handler. | `nt_mind_background_loop/run.rs:742` | medium | Source 2

D-SCON-008: **EventBus layer subscriber handles not tracked by BackgroundLoop** — `nt_core_event_bus.rs:359-396` returns `JoinHandle<()>` from `subscribe_layer()`, and `subscribe_all_layers()` returns `Vec<JoinHandle<()>>` (line 401-413). However, these handles are never stored in the `BackgroundLoop.handles` vector or any structured container. The layer subscribers run as detached tasks that are not cancelled during `shutdown()`. | `nt_core_event_bus.rs:359-413` | medium | Source 1, 2

D-SCON-009: **Download engine lacks fail-fast task cancellation** — `nt_io_download/engine.rs:135-156` spawns chunk download tasks into a `Vec<JoinHandle>` and awaits them sequentially. If one chunk fails, the remaining chunks continue running to completion. A `JoinSet` with `abort_all()` on first error (Source 6 pattern) would cancel redundant network I/O and reduce resource waste. | `nt_io_download/engine.rs:135-156` | medium | Source 1, 6

D-SCON-010: **stdio writer tasks in absorption pipeline are fire-and-forget** — `handlers_absorption.rs:181-185` spawns `tokio::spawn(async move { stdin.write_all(...).await; stdin.shutdown().await; })` without capturing the handle. If the parent process exits or the absorb handler is cancelled, these writers may be left writing to a closed stdin, causing silent errors. Structured concurrency requires the writer to be a child of the absorption scope. | `handlers_absorption.rs:181-185` | medium | Source 1, 2

D-SCON-011: **Reason streaming spawns orphaned word-emitter task** — `engine_core.rs:1909-1916` spawns `tokio::spawn(async move { for word in ... { tx.send(...).await; } })` without tracking the handle. If the receiver is dropped (consumer cancels), the emitter task continues running until the channel send fails, wasting CPU cycles on word splitting and sleeping. No cancellation propagation. | `engine_core.rs:1909-1916` | medium | Source 1, 2

D-SCON-012: **Plugin directory watcher has no lifecycle management** — `nt_io_plugin/registry.rs:450` returns a `JoinHandle<()>` from `watch_dir()`, but callers must manually manage the handle. The watcher task owns the `RecommendedWatcher` (line 451: `let _watcher = watcher`), so if the handle is dropped, the watcher stops. However, there is no structured integration with the BackgroundLoop shutdown — if the watcher handle is not stored, the file system watcher leaks until process exit. | `nt_io_plugin/registry.rs:450-451` | medium | Source 1

D-SCON-013: **`subscribe_all_layers_sync` spawns 9 OS threads without tracking** — `nt_core_event_bus.rs:418` spawns a `std::thread` per layer subscriber (9 threads). These threads have no structured join mechanism — they check an `Arc<AtomicBool>` shutdown flag, but the threads themselves are never joined. A thread panic would be silently lost. The async version (`subscribe_all_layers`) returns handles but the sync version does not. | `nt_core_event_bus.rs:418+` | medium | Source 2

D-SCON-014: **Proxy kernel spawns SOCKS5/HTTP listener tasks without error propagation** — `nt_shield_proxy_kernel/kernel.rs:173-220` spawns `socks5_handle`, `http_handle`, and `dns_handle` as independent `tokio::spawn` calls. If any listener panics (e.g., bind failure), the error is silently lost. There is no `JoinSet` to aggregate results or propagate failures to the parent. The proxy kernel continues running with partial functionality. | `nt_shield_proxy_kernel/kernel.rs:173-220` | high | Source 1, 2

D-SCON-015: **BackgroundLoop shutdown has 5-second hard deadline with no graceful drain** — `handlers.rs:48-61` implements shutdown with a 5-second `tokio::time::sleep` deadline. After the deadline, remaining tasks are forcibly aborted via `AbortHandle`. This is a "crash-only" shutdown rather than a graceful drain. A `TaskTracker` (Source 3) with `close()` + `wait()` pattern would allow tasks to complete their current work before exiting, with optional timeout as a fallback. The current approach risks data loss in handlers that are mid-write (e.g., KB persistence). | `nt_mind_background_loop/handlers.rs:48-61` | high | Source 3, 7, 10

## Key Insights

1. **Systemic absence of structured concurrency**: NeoTrix has zero adoption of `JoinSet`, `TaskTracker`, or any structured concurrency primitive despite having 62+ spawn sites. The codebase relies entirely on raw `tokio::spawn` with either dropped handles or `Vec<JoinHandle>` collections. This is a C0-level architectural gap.

2. **Manual shutdown is fragile and incomplete**: The 5-second hard abort in `handlers.rs` is a workaround for the lack of structured lifecycle management. With 40+ fire-and-forget tasks, shutdown cannot guarantee all spawned work is complete or properly cancelled.

3. **Parallel executor defeats its own purpose**: The sequential await pattern in `executor.rs:37-43` means the "parallel" executor runs tasks one at a time. Adopting `JoinSet::join_next()` in a loop would fix this trivially.

4. **Silent error swallowing in coordinator**: The `coordinator.rs:122-126` pattern of `if let Ok(result) = handle.await` means panicking reasoning agents produce zero results with no error signal — a correctness hazard for the multi-agent reasoning pipeline.

5. **Opportunity cost**: The `BackgroundLoop` already has a `spawn_handler!` macro (line 734) that wraps each handler in a structured loop with shutdown coordination. Converting the `Vec<JoinHandle>` to `JoinSet<()>` and using `JoinSet::abort_all()` + `join_next()` drain in `shutdown()` would be a ~50-line change that brings the core loop to structured concurrency compliance.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 15 |
| Severity: high | 7 |
| Severity: medium | 8 |
| Sources consulted | 12 |
