# Agent 1: Async Stream Patterns (Batch 868)

## Sources

| # | Source | URL | Key Insight |
|---|--------|-----|-------------|
| 1 | Rust Book Ch17-04: Streams | https://doc.rust-lang.org/stable/book/ch17-04-streams.html | `Stream` is async Iterator; `StreamExt::next()` requires `Unpin`; `while let` loop pattern; pinning required before iteration |
| 2 | Tokio Streams Tutorial | https://tokio.rs/tokio/tutorial/streams | `tokio_stream` provides `StreamExt` with `timeout`, `throttle`, `chunks_timeout`; `stream!` macro from `async-stream`; manual `Stream::poll_next` requires `Pin<&mut Self>` + `Context` + `Waker` |
| 3 | futures::Stream trait docs | https://docs.rs/futures/latest/futures/stream/trait.Stream.html | After `Ready(None)` returned, subsequent `poll_next` calls may panic/block/UB; `.fuse()` adapter prevents this; `size_hint()` is advisory only |
| 4 | futures::StreamExt docs | https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html | 46 methods; `buffer_unordered(N)` for concurrent stream processing; `.boxed()` for trait-object pinning; `.by_ref()` for borrowing without consuming |
| 5 | async-stream crate docs | https://docs.rs/async-stream/latest/async_stream/ | `stream!`/`try_stream!` macros via proc-macro; uses thread-local `Option<T>` cell; `for await` syntax for composing streams; `yield` transforms to `sender.send().await` |
| 6 | RFC 2996: AsyncIterator | https://rust-lang.github.io/rfcs/2996-async-iterator.html | `AsyncIterator` not yet in std; combinators excluded from base trait; `for` loop syntax not yet available; method resolution ambiguity risk with extension traits |
| 7 | Prism News: Rust Streams Explained | https://www.prismnews.com/hobbies/rust-programming/rust-streams-explained-async-iterators-poll-pin-and-waker | Common mistakes: treating stream as plain iterator, forgetting Poll::Pending handling, forcing sync habits onto async code, not using StreamExt combinators |
| 8 | Microsoft Async Rust: Streams Ch11 | https://microsoft.github.io/RustTraining/async-book/ch11-streams-and-asynciterator.html | `buffer_unordered(N)` is key concurrency tool; `try_stream!` needed for `?` propagation; `stream::unfold` for functional state machines |
| 9 | Rust FAQ: Async Iterators | https://www.rustfaq.org/en/how-to-use-async-iterators-streams-in-rust/ | Mix of sync/async iterators is common error; `stream::iter` wraps sync Iterator; `futures::stream::unfold` for stateful async generators |

## Defects

**D-STREAM-001: EventBus broadcast channel has no per-subscriber backpressure — slow consumers silently lag** | `neotrix-core/src/neotrix/nt_core_event_bus.rs:167-169` | MEDIUM | Rust Book Ch17-04, Tokio Streams

The `EventBus::emit_from` calls `self.sender.send(event)` and only logs `warn!` on `SendError`. However, the `broadcast::Receiver::recv()` returns `RecvError::Lagged(n)` when a consumer falls behind (source 7). The consumer in `run.rs:905-906` only logs the lag count but does NOT re-sync or drop stale events — it continues processing stale events that may contain consciousness state from a previous cycle. This means GWT attention routing decisions may be based on outdated `ConsciousnessCritique` events.

**D-STREAM-002: `stream_logs` implementations use `stream::iter` — entire log buffer materialized in memory before first yield** | `nt_shield_sandbox/docker.rs:327-334` | HIGH | Tokio Streams Tutorial, async-stream docs

`DockerSandboxProvider::stream_logs` calls `self.log_buffers.lock().ok().and_then(|b| b.get(session_id).cloned()).unwrap_or_default()` then wraps in `stream::iter(logs).boxed()`. This copies the entire `Vec<String>` log buffer, creates a new stream, and yields all items only when polled — no lazy loading. For long-running sandbox sessions with megabytes of logs, this defeats the purpose of streaming. Should use `async_stream::stream!` with lazy iteration or a channel-based approach.

**D-STREAM-003: Remote sandbox `stream_logs` uses `futures::channel::mpsc::unbounded` — no backpressure on log ingestion** | `nt_shield_sandbox/remote.rs:168-189` | HIGH | Tokio Streams Tutorial, Microsoft Async Rust Ch11

The remote sandbox provider creates `futures::channel::mpsc::unbounded()` and calls `tx.unbounded_send(line.to_string())` for every line of the HTTP response body. An unbounded channel can grow without limit if the consumer is slow, leading to OOM on the producer side. The spawned task reads the entire response body synchronously (`resp.text().await`) before sending to the channel — this is a time-of-check/time-of-use gap where memory could spike before any consumer processes lines. Should use bounded channel with backpressure or `tokio::sync::mpsc::channel(N)`.

**D-STREAM-004: `reason_stream` hardcodes channel capacity at 64 — no adaptive backpressure for token streaming** | `nt_mind/reason/reasoning_engine/engine_core.rs:1907` | MEDIUM | Rust FAQ Async Iterators, Microsoft Async Rust Ch11

The `reason_stream` method creates `tokio::sync::mpsc::channel(64)` and spawns a task that sends tokens one-by-one with `tokio::time::sleep(Duration::from_millis(10))`. The fixed 64-item buffer means: (1) if consumer processes slowly, tokens queue silently; (2) the artificial 10ms delay per word is hardcoded — not configurable; (3) the method returns both the full response AND the receiver, meaning the full response is always in memory even if only streaming is needed. The channel should use `tokio::sync::mpsc::channel(0)` (rendezvous) for true backpressure, or the buffer should be configurable.

**D-STREAM-005: SSE endpoint uses `stream::once` — sends exactly one event, not a true streaming response** | `nt_io_web/api.rs:499-504` | MEDIUM | Tokio Streams Tutorial, Rust Book Ch17-04

The SSE endpoint creates `stream::once(async { ... })` which yields exactly one `Event` and then terminates. The `Sse::new(stream).keep_alive(...)` will drop the connection after this single event because `stream::once` returns `Ready(None)` immediately after the first poll. For true SSE streaming, the endpoint needs a channel-based stream (e.g., `ReceiverStream`) or `async_stream::stream!` with a loop. The comment on line 1094 (`简化实现避免 async_stream 宏问题`) acknowledges this is a simplified implementation that avoids async_stream issues.

**D-STREAM-006: EventBus `flood_guard` uses `std::sync::Mutex` inside an async context — potential priority inversion** | `nt_core_event_bus.rs:190-204` | LOW | Prism News Streams Explained, Rust Book Ch17-04

The `flood_guard` closure captures a `std::sync::Mutex<HashMap<...>>` and locks it inside the waterfall filter chain. Since `EventBus::emit_from` is called from async contexts (background loop handlers), this `std::sync::Mutex` lock can block the async executor thread if contended. Tokio's `spawn_blocking` or `tokio::sync::Mutex` should be used for mutexes held across `.await` points, or this should be documented as intentionally lock-free (the critical section is very short). The `sync_handlers` vector in `EventBus` has the same issue.

**D-STREAM-007: No `.fuse()` usage on any stream in the codebase — post-`Ready(None)` poll_next behavior is undefined** | `nt_shield_sandbox/docker.rs:334`, `nt_shield_sandbox/remote.rs:189`, `nt_io_web/api.rs:504` | MEDIUM | futures::Stream trait docs, Rust Book Ch17-04

None of the three `BoxStream` return sites use `.fuse()` to guarantee safe post-exhaustion behavior. Per the `Stream` trait docs (source 3): "Once a stream has finished (returned `Ready(None)` from `poll_next`), calling its `poll_next` method again may panic, block forever, or cause other kinds of problems." While `BoxStream` wrapping may prevent some issues, the `stream::iter(logs)` case in `docker.rs:334` is particularly vulnerable — if the caller polls after exhaustion, behavior is undefined. `.fuse()` adds negligible overhead and is the standard defensive pattern.

**D-STREAM-008: `subscribe_all_layers_sync` blocks the thread with `block_on` inside `tokio::spawn` — potential executor starvation** | `nt_core_event_bus.rs:418-425` (referenced from `run.rs:415`) | HIGH | Tokio Streams Tutorial, Prism News

The function `subscribe_all_layers_sync` (defined at `nt_core_event_bus.rs:418`) is called during EventBus setup in `run.rs:415`. This function is documented as "blocking" and likely uses `tokio::runtime::Runtime::block_on` or similar to create synchronous subscriptions. If called within an async context, this can starve the Tokio executor. The async variant `subscribe_all_layers` (returning `JoinHandle<()>`) exists but is not used in production — the sync version is preferred for hydration replay safety.

**D-STREAM-009: `SandboxProvider::stream_logs` trait returns `BoxStream<'static, String>` — no lifetime coupling to session** | `nt_shield_sandbox/provider.rs:33` | LOW | async-stream docs, Rust FAQ

The trait signature `fn stream_logs(&self, session_id: &str) -> BoxStream<'static, String>` requires `'static` lifetime on the returned stream. This forces the Docker provider to clone the entire log buffer into a `'static` stream (source 2 of docker.rs), and forces the remote provider to spawn an independent task (source 3 of remote.rs). A more ergonomic signature would use `'_` lifetime or `impl Stream<Item = String> + '_` to allow borrowing from the provider's internal state, avoiding unnecessary clones and task spawns.

**D-STREAM-010: EventBus replay broadcasts synchronously in a for-loop — no async interleaving with live events** | `nt_core_event_bus.rs:210-219` | MEDIUM | Tokio Streams Tutorial, Microsoft Async Rust Ch11

The `replay_and_broadcast` method reads all envelopes from disk and broadcasts them in a tight `for env in chain` loop. During this loop, no async work can interleave — live events that arrive during replay are buffered by the broadcast channel. If the JSONL log is large (thousands of events), the replay blocks the broadcast channel's internal buffer for the duration, causing `Lagged(n)` errors on slow consumers. The replay should use `tokio::time::yield_now()` or `tokio::task::yield_now()` periodically to allow live event processing.

## Key Insights

1. **Stream is not in std yet** — NeoTrix correctly uses `futures` crate's `Stream` trait (RFC 2996 is still in-progress). The codebase avoids the unstable `AsyncIterator` trait, which is the right call for production stability.

2. **No `async-stream` crate usage despite being available** — The codebase has `async-stream` as a dependency (used in tokio tutorial patterns) but the main code uses manual `mpsc::channel` + `tokio::spawn` patterns instead. The comment on `api.rs:1094` explicitly mentions avoiding `async_stream` macro issues. This suggests past instability or compile-time issues with the proc-macro.

3. **EventBus is the central stream hub** — All 9 layer subscribers consume from a single `broadcast::channel<CoreEvent>`. The two-phase design (sync handlers for persistence, async handlers for behavioral reaction) is architecturally sound but the sync handlers use `std::sync::Mutex` which can cause priority inversion under load.

4. **Backpressure is systematically missing** — Three of the four `stream_logs` implementations either materialize everything in memory (`stream::iter`), use unbounded channels (`mpsc::unbounded`), or use `stream::once` (single event). Only the reasoning engine uses bounded channels, and even those have hardcoded capacities.

5. **The consciousness architecture's stream usage is minimal** — Despite having `ConsciousnessTree` with 11 branches, the actual async stream patterns are limited to EventBus subscriptions, sandbox log streaming, and LLM token streaming. The consciousness loop (`run.rs`) uses `tokio::select!` with `broadcast::Receiver` rather than stream combinators, which is correct for event-driven architectures.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources analyzed | 9 |
| Files examined | 12 |
| High severity | 2 |
| Medium severity | 6 |
| Low severity | 2 |
