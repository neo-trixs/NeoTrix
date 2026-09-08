# Agent 1: Async IO Patterns (Batch 877)

## Sources
1. Tokio docs — `AsyncRead`, `AsyncWrite`, `BufReader`, `BufWriter` (docs.rs/tokio)
2. Tokio tutorial — I/O patterns, EOF handling, buffer allocation (tokio.rs/tokio/tutorial/io)
3. Biriukov — "Tokio I/O Patterns: split, bidirectional driver, backpressure, framed I/O" (biriukov.dev, Oct 2025)
4. Biriukov — "Async Rust with Tokio I/O Streams: Backpressure, Concurrency" (biriukov.dev, Oct 2025)
5. EliteDev — "8 Tokio Patterns That Transform Rust Network Services" (elitedev.in, Jul 2026)
6. Rust Patterns Book — "Async I/O Patterns" (rust-patterns.com)
7. DeepWiki — "Async I/O Traits and Readiness Model" (deepwiki.com, Jul 2026)
8. Rust Performance Book — "I/O" (nnethercote.github.io)
9. rust-skills — "rules/perf-io-buffering.md" (GitHub)
10. OneUptime — "How to Handle File I/O Efficiently in Rust" (oneuptime.com, Jan 2026)
11. Rust Users Forum — "Buffer pooling for async IO?" (users.rust-lang.org, May 2022)
12. Yildirim — "Async I/O with Tokio & Rust" (mete.software, Apr 2023)

## Defects

**D-IO-001: `io::split()` on `Box<dyn ProxyStream>` introduces Arc+Mutex overhead for every proxy relay** | `kernel.rs:539` | HIGH | Sources 3,4,6

`tokio::io::split()` on a `Box<dyn ProxyStream>` (trait object) uses `Arc<Mutex<>>` internally. For every TCP relay through the proxy kernel, this adds unnecessary contention. The code already has `client_stream.into_split()` (zero-cost for `TcpStream`), but the upstream side pays the full Mutex cost. For a proxy handling hundreds of concurrent connections, this compounds. Fix: Use a dedicated connection-driver pattern (like h2/hyper) for the `ProxyStream` trait object path, or add a `split()` method to the `ProxyStream` trait that returns typed halves without the generic `io::split`.

**D-IO-002: XtlsStream `poll_read` silently drops unprocessed encrypted bytes on partial decrypt** | `connector/vless/mod.rs:287-293` | CRITICAL | Sources 2,6

When `decrypt()` returns plaintext longer than `buf.remaining()`, excess bytes go into `pending_plaintext`. But if `poll_read` is called again and the inner stream has already advanced, the `read_buf` is resized to `buf.remaining()` (line 277), potentially losing the boundary context for the next decrypt call. The `read_buf` is a `BytesMut` that gets mutated in-place — if `fill_buf` in a subsequent call overwrites bytes that were part of a partially-consumed encrypted frame, decrypt will fail or produce garbage. Fix: Track consumed byte count within the encrypted frame boundary, and only advance `read_buf` after a complete frame is decrypted.

**D-IO-003: `copy_bidirectional` error results silently discarded in proxy relay paths** | `listener/socks5.rs:160`, `listener/http.rs:151`, `mitm.rs:270`, `local_proxy.rs:205-206,355` | MEDIUM | Sources 2,5,9

Multiple relay paths use `tokio::io::copy_bidirectional(...).await.ok()` or `tokio::io::copy(...).await` with discarded results. When one direction fails (connection reset, timeout), the other direction is never explicitly shut down. This leaks half-open TCP connections. The `local_proxy.rs:205` path uses `tokio::join!` on two unidirectional copies — if one finishes (EOF or error), the other keeps running indefinitely until its own timeout or the task is dropped. Fix: Use `tokio::select!` with explicit shutdown of both halves on any error, or wrap in `CancellationToken`.

**D-IO-004: `ObfuscatedStream::poll_write` never signals backpressure — always returns `Poll::Ready(Ok(buf.len()))`** | `security.rs:473-479` | HIGH | Sources 3,4,8

`poll_write` unconditionally accepts all data into `write_buf` and returns `Ready(Ok(buf.len()))` regardless of how much is buffered. The actual flushing happens only in `poll_flush`. This means the writer appears to accept unlimited data with zero backpressure. If the inner stream is slow or blocked, `write_buf` grows unbounded. The frame-level flush in `flush_write_buf` does handle partial writes correctly, but by that point, the caller has already been told the write succeeded. For a proxy handling many concurrent streams, this can cause OOM. Fix: Either limit `write_buf` capacity and return `Poll::Pending` when exceeded, or flush synchronously in `poll_write` before returning.

**D-IO-005: `knowledge_storage.rs` uses synchronous `std::io::BufReader` in code that may be called from async context** | `knowledge_storage.rs:76` | MEDIUM | Sources 2,8,9

`BufReader::new(file).lines()` on line 76 is a blocking I/O operation (uses `std::fs::File`). If this code is ever called from a Tokio task (even via `spawn_blocking`), it blocks the worker thread for the entire duration of journal replay. The `compact()` method at line 182 also uses synchronous `File::create` + `BufWriter` + `serde_json::to_writer`. For large knowledge bases, this blocks the runtime. Fix: Either ensure all calls are wrapped in `spawn_blocking`, or migrate to `tokio::fs` + async buffered IO for the replay and compaction paths.

**D-IO-006: `neotrix_dl.rs` download writes raw TCP chunks without BufWriter — one syscall per HTTP chunk** | `neotrix_dl.rs:122` | LOW | Sources 1,9,10,12

`file.write_all(&chunk).await?` writes each HTTP response chunk directly to the `tokio::fs::File` without a `BufWriter`. While `tokio::fs::File` internally uses a blocking thread pool, each chunk still triggers a separate `write` syscall. For large GGUF model downloads (multi-GB), adding a `BufWriter` with 64KB-256KB capacity would batch small chunks into fewer syscalls. The explicit `file.flush().await?` at line 134 is correct but the intermediate writes are suboptimal.

**D-IO-007: `local_proxy.rs:relay()` uses `TcpStream::split()` (borrow-based) but relay runs across await points** | `local_proxy.rs:201-207` | MEDIUM | Sources 3,4

`client.split()` and `remote.split()` return `ReadHalf`/`WriteHalf` that borrow the stream (must stay on same task). While `tokio::join!` on two `io::copy` calls is fine within one task, the function signature takes owned `TcpStream` and splits immediately — this is correct. However, if this function is ever refactored to spawn separate tasks for read/write (which the pattern suggests it might be), it would need `into_split()` instead. The code comment `_cr`, `_rr` (underscore prefix) suggests dead read halves, but they are used — naming is misleading and masks the architectural decision.

**D-IO-008: XtlsStream `poll_write` does not register waker for write_buf drain, causing missed wakeups** | `connector/vless/mod.rs:316-326` | HIGH | Sources 2,4,6

When `poll_write` encounters a partial write of `write_buf` (line 320: `return Poll::Pending`), it returns `Pending` but does not ensure the waker is registered for when the inner stream becomes writable again. The `cx.waker()` from the outer `poll_write` call is passed to the inner `poll_write` at line 316, but if the inner write succeeded partially (line 317-319) and then we return `Pending` at line 320, the waker from the *next* call to `poll_write` is the one that will be registered — not the one from the partial write. This can cause a one-cycle delay in wakeup. Fix: After draining partial `write_buf`, re-poll the inner stream to ensure the waker is properly registered, or use a flag to track pending drain state.

**D-IO-009: `mitm.rs:PeekedStream` drops buffered peek data on seek/drop, but no mechanism to restore** | `mitm.rs:17-37` | LOW | Sources 1,6

`PeekedStream` stores pre-read bytes in `peeked: Vec<u8>` with a `pos` cursor. If the inner stream is a TLS `ClientStream` that gets replaced (e.g., after TLS handshake), the peeked bytes are consumed once and cannot be replayed. This is by design for MITM, but the lack of a `buffered()` method or `AsyncBufRead` implementation means callers cannot inspect how many bytes remain buffered without consuming them. For debugging MITM issues, this makes it harder to verify that the ClientHello was fully captured.

**D-IO-010: No buffered IO wrapper for NT-SHIELD proxy TCP streams — raw `write_all` per message** | `listener/http.rs:142,147,155`, `listener/socks5.rs:145,156,165,178` | MEDIUM | Sources 1,9,10

HTTP and SOCKS5 listeners perform multiple `stream.write_all(...)` calls per connection (response headers, body, error responses) without wrapping the `TcpStream` in a `BufWriter`. Each `write_all` becomes a separate syscall. For the SOCKS5 BIND command path (`socks5.rs:145,156`), two `write_all` calls happen in sequence (initial response + bound address notification). Wrapping in `BufWriter` and flushing once would halve syscalls. Fix: Wrap `TcpStream` in `BufWriter` at the start of connection handling, flush before returning.

## Key Insights

1. **Backpressure gap in `ObfuscatedStream`**: The most critical finding. The obfuscation layer accepts unlimited writes without signaling congestion. In a proxy architecture where NT-SHIELD relays traffic, a slow downstream peer can cause memory exhaustion in the obfuscation layer's `write_buf`.

2. **`io::split` vs `into_split` inconsistency**: NeoTrix correctly uses `into_split()` for `TcpStream` in some places (kernel.rs:538, tor_client.rs:311) but falls back to `tokio::io::split()` for trait objects (kernel.rs:539). The generic path adds Arc+Mutex overhead that compounds at scale.

3. **Silent error swallowing in relay paths**: Five separate locations discard `copy_bidirectional` / `copy` results. This is a systemic pattern — not a one-off mistake. It means half-open connections accumulate under error conditions.

4. **Sync IO in async-adjacent code**: `knowledge_storage.rs` uses blocking IO that could block Tokio worker threads. The journal replay path (line 76) reads potentially large files synchronously.

5. **Custom `AsyncRead`/`AsyncWrite` implementations are correct but fragile**: `XtlsStream`, `ObfuscatedStream`, and `PeekedStream` all implement the traits correctly, but the partial-write buffer management in `XtlsStream` has a subtle waker registration issue (D-IO-008) that could cause latency spikes under load.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| CRITICAL | 1 |
| HIGH | 3 |
| MEDIUM | 4 |
| LOW | 2 |
| Sources analyzed | 12 |
| Files with async IO patterns found | 22+ |
| Custom AsyncRead/AsyncWrite impls found | 3 (XtlsStream, ObfuscatedStream, PeekedStream) |
| Unbuffered raw write paths found | 5+ locations |
| Silent error discard sites found | 5 |
