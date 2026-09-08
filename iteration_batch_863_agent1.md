# Agent 1: Async Stream Patterns (Batch 863)

## Sources
1. Rust Book Ch17-04 Streams (doc.rust-lang.org/stable/book/ch17-04-streams.html) — Stream/StreamExt trait definitions, `next()` requiring `Unpin`, `stream_from_iter` pattern
2. Tokio Tutorial Streams (tokio.rs/tokio/tutorial/streams) — `pin!` requirement, `async-stream` crate for generator pattern, `buffer_unordered` for concurrency
3. futures::StreamExt docs (docs.rs/futures/latest/futures/stream/trait.StreamExt.html) — 46 methods: `next`, `filter`, `then`, `fold`, `fuse`, `boxed`, `for_each_concurrent`, `buffered/buffer_unordered`
4. tokio_stream::StreamExt docs (docs.rs/tokio-stream/latest/tokio_stream/trait.StreamExt.html) — 23 methods, `merge`, `timeout`, `throttle`, `chunks_timeout`; two competing StreamExt traits cause ambiguity errors
5. async-stream crate docs (docs.rs/async-stream/latest/) — `stream!`/`try_stream!` macros using proc macros, `yield` transformed to `sender.send().await`, thread-local pointer for poll state
6. async_stream_lite docs — `async_stream(|yielder| async move { yielder.y().await })` alternative pattern
7. Rust Generators Unstable Book (doc.rust-lang.org/unstable-book/language-features/generators) — `#![feature(coroutines)]` nightly-only, `Generator` trait, state machine compilation
8. Microsoft RustTraining Ch11 Streams — `buffer_unordered(N)` for concurrency, `stream::unfold` for stateful generation, comparison with C# `IAsyncEnumerable`
9. Rust for TS/JS Developers (rs4ts.dev/11-async/06-streams) — Import one `StreamExt` per module, `while let Some(x) = s.next().await` not `for await`, `tokio::pin!` for self-referential producers
10. Backpressure patterns (biriukov.dev, dev.to, palakorn.com) — Bounded `mpsc::channel(n)` for backpressure, `Semaphore` for concurrency limiting, `buffered(n)` preserves order
11. Async Pipeline Pattern (github.com/alexpusch/rust-magic-patterns) — Spawned tasks + channels for eager execution vs lazy Stream; `buffered` stream stalls upstream polling
12. FusedStream docs (docs.rs/futures-util/latest/futures_util/stream/trait.FusedStream.html) — `is_terminated()` tracks completion, prevents post-None polling
13. Swatinem: Complexities of Rust Async Streams (swatinem.de) — Two competing `AsyncRead` traits (futures vs tokio), constant error type conversion with `TryStreamExt::map_err`
14. Three Problems of Pinning (without.boats/blog/three-problems-of-pinning) — `for await` loop needed to eliminate pin-at-callsite, `merge!` macro for select-in-loop

## Defects

**D-STREAM-001: ConsciousnessStream uses synchronous VecDeque, not async Stream trait** | `neotrix-core/src/unified/core/nt_core_consciousness/stream_buffer.rs:10` | HIGH | Source 1, 3

`ConsciousnessStream` wraps a `VecDeque<VsaTagged>` with synchronous `push()`/`recent()` methods. It does not implement `futures::Stream` trait, meaning it cannot participate in async stream pipelines (map/filter/merge/fold). When the consciousness runtime needs to process VSA vectors asynchronously (e.g., from GWT attention routing or EventBus), there is no backpressure signal — the buffer silently evicts old data via `pop_front()` without notifying producers. A proper `Stream` impl with `poll_next` would allow integration with `buffer_unordered`, `for_each_concurrent`, and other async combinators.

**D-STREAM-002: VibrationalResonanceFramework ProcessingStream is a plain struct, not a Rust async Stream** | `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_vibrational_resonance.rs:45` | MEDIUM | Source 1, 8

`ProcessingStream` is a data struct holding `frequency_hz`, `amplitude`, `phase` — it models a frequency-domain concept but has no `Stream` trait implementation. The `VibrationalResonanceFramework` evaluates gamma sync by iterating `processing_streams` synchronously with `filter`/`map`. This blocks the consciousness tick loop from concurrently evaluating cross-stream coherence when N streams grow large. Converting `ProcessingStream` to an actual `Stream<Item = ProcessingFrame>` with `buffer_unordered` would enable concurrent gamma evaluation across sensory/cognitive/emotional streams.

**D-STREAM-003: EventStream in nt_core_forecast uses Vec with manual front-removal, no async Stream composition** | `neotrix-core/src/unified/core/nt_core_forecast.rs:76` | MEDIUM | Source 3, 10

`EventStream` stores `Vec<StructuredEvent>` and enforces a hard 256 limit with `self.events.remove(0)` — O(n) front removal on every push. It does not implement `Stream`, so it cannot be merged with other async event sources (e.g., EventBus broadcasts, crawl results). The `aggregate_impact()` method iterates all events synchronously. With a proper `Stream<Item = StructuredEvent>` implementation, events could be piped through `filter`, `scan` (for running aggregate), and `take_while` for temporal decay without manual iteration.

**D-STREAM-004: No Fuse wrapper on ConsciousnessStream — post-completion polling undefined behavior** | `neotrix-core/src/unified/core/nt_core_consciousness/stream_buffer.rs:23` | HIGH | Source 4, 12

`ConsciousnessStream::current()` returns the last pushed item but provides no `is_terminated()` / fused semantics. If a stream consumer polls after the consciousness runtime shuts down (e.g., SIGTERM during SEAL pipeline), it could read stale data or undefined buffer state. The `fuse()` combinator from `StreamExt` or implementing `FusedStream` would guarantee that `poll_next` returns `Ready(None)` permanently after shutdown, preventing downstream operators from processing garbage data.

**D-STREAM-005: No backpressure propagation in ConsciousnessStream push path** | `neotrix-core/src/unified/core/nt_core_consciousness/stream_buffer.rs:30` | HIGH | Source 6, 10

`push()` unconditionally evicts the oldest item when at capacity (`pop_front()` + increment `total_evicted`). This is a silent-drop backpressure strategy with no signal to producers. When NT-WORLD crawlers or NT-ACT social media scrapers produce VSA vectors faster than the consciousness loop consumes them, the eviction counter grows but producers have no way to know they're outrunning the consumer. A bounded `mpsc::channel` or `tokio::sync::Semaphore`-gated push would propagate backpressure to crawl pipelines, preventing OOM under burst load.

**D-STREAM-006: ConsciousnessStream::bundled_self/world allocates Vec on every call** | `neotrix-core/src/unified/core/nt_core_consciousness/stream_buffer.rs:63` | LOW | Source 2, 9

`bundled_self()` and `bundled_world()` call `self.buffer.iter().rev().take(n).filter(...).map(...).collect::<Vec<&[u8]>>()` then `QuantizedVSA::bundle()`. Each call allocates a new `Vec` on the heap. During high-frequency consciousness ticks (64Hz attention span), this creates allocation pressure. A `Stream`-based approach with `fold` over the buffer — accumulating the bundle in-place — would eliminate per-call allocation. Alternatively, a reusable `Vec<&[u8]>` scratch buffer could be retained across calls.

**D-STREAM-007: Missing StreamExt::fuse() on raw Stream consumers in consciousness runtime** | `neotrix-core/src/unified/core/nt_core_consciousness/consciousness_runtime.rs:65` | MEDIUM | Source 4, 12

The `ConsciousnessRuntime` holds a `stream: ConsciousnessStream` field but never wraps it with `.fuse()`. If the runtime's main loop accidentally calls `poll_next` after the stream signals completion (via buffer empty + shutdown), the behavior is undefined per the `Stream` trait contract ("calling `poll_next` again may panic, block forever, or cause other problems"). Adding `.fuse()` guarantees safe post-completion behavior and enables `select_next_some()` usage with the `select!` macro.

**D-STREAM-008: Two competing StreamExt traits create ambiguity risk across NT modules** | Architecture-wide | MEDIUM | Source 4, 9, 13

Rust ecosystem has two `StreamExt` traits: `futures::stream::StreamExt` (46 methods) and `tokio_stream::StreamExt` (23 methods). Importing both in the same module causes `error[E0034]: multiple applicable items in scope` on shared methods like `.next()`, `.map()`, `.filter()`. NeoTrix modules that mix `tokio_stream` (for `ReceiverStream` wrappers) with `futures` (for `buffer_unordered`) risk compile-time ambiguity. The architecture needs a module-level convention: one `StreamExt` per module, with the other accessed via fully-qualified syntax.

**D-STREAM-009: async-stream crate thread-local state is fragile for consciousness loop** | Used in `nt_core_forecast::LlmNarrator` retry loop | LOW | Source 5, 7

The `async-stream` crate stores a pointer to the yield cell in thread-local storage during `poll`. If the consciousness runtime moves a `stream!` block between Tokio worker threads (which happens with `spawn`), the thread-local becomes stale. The `stream!` macro's internal implementation uses `sender.send($expr).await` which is not cancellation-safe — if the stream is dropped mid-yield, the value is lost. For the SEAL pipeline's evolution stream, this means interrupted cycles could lose partially-processed experience data.

**D-STREAM-010: No Stream-based concurrent processing for GWT attention routing** | Architecture-level gap | HIGH | Source 3, 8, 10

GWT (Global Workspace Theory) attention routing broadcasts salient information across specialist modules. Currently there is no `Stream`-based fan-out pattern using `buffer_unordered(N)` for concurrent module notification. The architecture would benefit from:
```rust
let notifications = broadcast_rx.into_stream()
    .map(|msg| notify_module(msg))
    .buffer_unordered(MAX_CONCURRENT_MODULES);
```
This would limit concurrent module notifications to N (preventing thundering herd), process completions out-of-order (fast modules respond first), and provide natural backpressure via the bounded broadcast channel. Currently, module notification appears synchronous or ad-hoc.

**D-STREAM-011: ConsciousnessStream novelty() iterates buffer without short-circuit** | `neotrix-core/src/unified/core/nt_core_consciousness/stream_buffer.rs:121` | LOW | Source 3, 8

`novelty()` iterates the entire `lookback` window computing `QuantizedVSA::similarity` for each vector, taking the max. If the first vector is identical (similarity = 1.0), novelty = 0.0, but iteration continues. With `Stream`-based `take_while` or `scan` with early termination, once `max_sim >= 1.0 - epsilon`, remaining iterations could be skipped. For a 64-sample attention span, this saves up to 63 similarity computations per novelty check.

## Key Insights

1. **Stream not in std yet**: `AsyncIterator` (Rust's future `Stream`) is nightly-only and unstable as of 2026. All production code depends on `futures::Stream` re-exported by `tokio-stream`. NeoTrix must commit to one ecosystem.

2. **async-stream is a bridge, not a destination**: The `stream!` macro works via proc-macro codegen with thread-local state. It's explicitly labeled a "temporary solution" until native `async gen` blocks stabilize (RFC 3513 target: 2025 edition). NeoTrix should use `async-stream` for now but plan migration path to native generators.

3. **Two StreamExt traits = maintenance tax**: `futures::StreamExt` (46 methods) vs `tokio_stream::StreamExt` (23 methods) overlap heavily. The `enumerate` method only exists on futures; `timeout`/`throttle` only on tokio. NeoTrix needs a module-level import convention to avoid ambiguity errors.

4. **Backpressure is non-negotiable**: Every async stream pipeline must have bounded buffers. Unbounded channels + fire-and-forget producers = OOM under load. The `mpsc::channel(n)` pattern with `.send().await` blocking is the standard. `Semaphore` is for concurrency limiting, not backpressure.

5. **State machine size compounds multiplicatively**: A nested async function chain can produce state machine enums exceeding 400KB (Tweede golf 2024 measurement). `Box::pin()` trades one heap allocation for constant state machine size. For NeoTrix's consciousness loop with deeply nested SEAL pipelines, this trade-off should be profiled.

6. **`buffer_unordered` is the concurrency workhorse**: For I/O-bound stream processing (crawl fetches, LLM calls, KB writes), `buffer_unordered(N)` processes N items concurrently with results arriving out-of-order. This is superior to sequential `for_each` for throughput-sensitive paths.

7. **FusedStream prevents post-completion panics**: The `Stream` trait places no requirements on behavior after `poll_next` returns `Ready(None)`. Without `.fuse()`, downstream operators may panic, block forever, or corrupt state. Critical for consciousness runtime shutdown safety.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 11 |
| D-STREAM range | D-STREAM-001 through D-STREAM-011 |
| HIGH severity | 4 |
| MEDIUM severity | 4 |
| LOW severity | 3 |
| Sources consulted | 14 |
| Architecture areas affected | consciousness, forecast, vibrational resonance, GWT, SEAL pipeline |
