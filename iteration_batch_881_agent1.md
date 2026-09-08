# Iteration Batch 881 — Agent 1: Rust Async IO Patterns

**Research Sources**: Rust Patterns Book (async-io-patterns), Tokio docs, ScyllaDB perf pitfall post, kindatechnical.com async pitfalls, tokio-rs/tokio#2716, stackoverflow AsyncRead implementations, deepwiki futures-rs.

**Research Query**: "rust async io patterns", "rust AsyncRead AsyncWrite", "rust BufReader async"

---

## Defect 1: Unbuffered AsyncRead in MITM Proxy — Syscall Storm

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:168`

**Evidence**: `handle_one()` reads directly from `TcpStream` with `stream.read(&mut buf)` (16384-byte stack buffer). The `PeekedStream<T>` wrapper implements `AsyncRead` by delegating to the raw inner stream with zero buffering. When wrapped by `tokio_rustls::TlsAcceptor`, the TLS layer performs small 5-16 byte record reads internally, each a separate syscall.

**Pattern**: ScyllaDB driver (issue #338) measured ~10x throughput regression from unbuffered reads. Every TLS record recv = 1 syscall. With `BufReader` wrapping, multiple TLS records batch into single `recv()`.

**Defect**: `PeekedStream` does not wrap its inner `T` in `BufReader<T>`. In the MITM path (`handle_connect_mitm`), the TLS acceptor reads ClientHello through `PeekedStream<TcpStream>` — unbuffered. This causes syscall-per-record overhead for the entire TLS session lifetime.

**Fix**: Wrap inner `T` in `tokio::io::BufReader` before TLS handshake:
```rust
let buffered = tokio::io::BufReader::new(stream);
let peeked = PeekedStream { peeked: client_hello, pos: 0, inner: buffered };
```

---

## Defect 2: No Backpressure in XtlsStream write_buf — Unbounded Memory Growth

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/connector/vless/mod.rs:308-320`

**Evidence**: `XtlsStream::poll_write` appends encrypted bytes to `self.write_buf` on partial-write or pending, with no capacity limit:
```rust
self.write_buf.extend_from_slice(&encrypted[n..]);
// or
self.write_buf.extend_from_slice(&encrypted);
```
If the inner stream stalls (network partition, slow peer), `write_buf` grows without bound. The `poll_flush` method reads from `write_buf` but never signals backpressure to the caller.

**Pattern**: Tokio backpressure strategies (kindatechnical.com) — bounded channels + semaphore for concurrency control. `BufWriter` has a configurable flush threshold. Unbounded buffers in async pipelines are the #1 OOM vector.

**Defect**: A slow or stalled upstream connection causes `write_buf` to grow proportionally to total bytes written. No flush-on-threshold or `try_write` fallback exists. In a proxy relay scenario (NT-SHIELD proxy kernel), this can accumulate gigabytes before OOM killer fires.

**Fix**: Add a `max_write_buf` field. When `write_buf.len() > max_write_buf`, `poll_write` returns `Poll::Pending` (applying backpressure) or flushes eagerly before buffering more:
```rust
if self.write_buf.len() > MAX_WRITE_BUF {
    // Flush existing before accepting more
    match Pin::new(&mut self.inner).poll_write(cx, &self.write_buf) { ... }
}
```

---

## Defect 3: ObfuscatedStream poll_read Returns Incomplete Frames — Data Corruption Risk

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/security.rs:515-540`

**Evidence**: `ObfuscatedStream::poll_read` parses frames from `self.read_buf` (a `BytesMut`). The frame parser reads `payload_len` from bytes [0..2], then checks `self.read_buf.len() >= full_frame`. If insufficient data exists, it calls `self.read_inner(cx)`. But `read_inner` reads into a fixed 4096-byte `inner_read_buf` — if the frame is larger than 4096 bytes (payload > 4093), `read_inner` returns `Poll::Ready(Ok(()))` with partial data, and the outer loop re-enters. However, on the next iteration, the frame parser sees the same partial state and calls `read_inner` again. This works correctly for frames < 4096, but for large frames, the loop spins without yielding (`Poll::Pending`), potentially starving other tasks.

**Pattern**: Tokio cooperative scheduling (ScyllaDB post) — `FuturesUnordered` causes quadratic slowdown when tasks never yield. The `poll_read` loop can spin indefinitely reading small chunks without ever returning `Poll::Pending`.

**Defect**: For frames larger than `inner_read_buf` (4096 bytes), `poll_read` enters a busy-loop: `read_inner` returns `Ready(Ok(()))` → re-loop → check frame → insufficient → `read_inner` again. This blocks the executor thread cooperatively until the frame is complete. No `yield_now()` or cooperative budget check exists.

**Fix**: After `read_inner` returns data, if the frame is still incomplete and no `Poll::Pending` was encountered, force a yield:
```rust
// After read_inner succeeds but frame incomplete:
cx.waker().wake_by_ref(); // Re-schedule to yield
return Poll::Pending;
```

---

## Defect 4: PeekedStream poll_read Does Not Respect ReadBuf::remaining() Correctly

**File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:23-33`

**Evidence**: `PeekedStream::poll_read` drains from `self.peeked` using `buf.put_slice(...)`. The issue is subtle: when `peeked` bytes are consumed and the inner stream has data, the function falls through to `Pin::new(&mut this.inner).poll_read(cx, buf)`. But `buf` may have had its `filled` pointer advanced by the peeked bytes already. This is correct per Tokio's `ReadBuf` contract. However, the function does NOT call `buf.clear()` between the peeked-drain path and the inner-read path — if a caller passes a `ReadBuf` that already has filled data, the peeked bytes append to it. This violates the `AsyncRead` contract where `poll_read` should fill the unfilled portion.

**Pattern**: Tokio `ReadBuf` semantics (docs.rs/tokio/io/struct.ReadBuf) — `put_slice` advances `filled`, `remaining()` shrinks. A subsequent `poll_read` on the inner sees a partially-filled buffer. Standard `AsyncRead` implementations expect `poll_read` to start writing at `filled()`, not reset.

**Defect**: If the caller passes a `ReadBuf` with existing filled data (e.g., from a previous partial read), `PeekedStream` appends peeked bytes AFTER the existing data, then delegates to inner. This is technically correct but can cause confusion: the inner stream may fill data into the same buffer after the peeked portion, creating a mixed peeked+stream read. In practice, Tokio's `read()` extension creates fresh `ReadBuf`s, so this is low-risk. But custom callers using `poll_read` directly may trigger unexpected behavior.

**Severity**: Low (edge case with direct `poll_read` callers).

---

## Defect 5: Blocking `std::sync::Mutex` in Async Context — NT-MIND LSP Client

**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/lsp_client/client.rs:3,120`

**Evidence**: LSP client uses `std::io::{BufRead, BufReader, Read, Write}` (synchronous) to wrap `ChildStdout`. This is the `lsp_client` which spawns an LSP server process. The `BufReader::new(stdout)` call at line 120 uses synchronous `BufRead`. If this code runs inside an async context (e.g., tokio::spawn), it blocks the executor thread on every `read_line()` call.

**Pattern**: kindatechnical.com async pitfalls — "The Number One Pitfall: Blocking the Executor." Using `std::fs`/`std::io` in async context blocks the thread. On a 4-thread runtime, blocking one thread = 25% throughput loss.

**Defect**: The LSP client's `BufReader` reads synchronously from `ChildStdout`. If the LSP server produces output slowly, `bufread.read_line()` blocks the tokio worker thread until data arrives. This is acceptable only if the LSP client is always run in `spawn_blocking`, but no such guard exists.

**Fix**: Either (a) wrap in `tokio::task::spawn_blocking`, or (b) use `tokio::process::Child` + `tokio::io::BufReader` for async I/O:
```rust
use tokio::process::Command;
use tokio::io::BufReader;
let mut child = Command::new("lsp-server").spawn()?;
let reader = BufReader::new(child.stdout.take().unwrap());
```

---

## Defect 6: Knowledge Storage Uses Synchronous BufReader in Async-Eligible Path

**File**: `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/knowledge_storage.rs:14,76,182`

**Evidence**: `knowledge_storage.rs` uses `std::io::{BufRead, BufReader, BufWriter, Write}` (all synchronous). Line 76: `BufReader::new(file).lines()` — synchronous line iteration. Line 182: `BufWriter::new(f)` — synchronous buffered write. This module handles KB persistence (knowledge storage/retrieval).

**Pattern**: Same as Defect 5. Synchronous I/O in an async-eligible codebase creates thread-blocking hazards when called from async contexts.

**Defect**: If `knowledge_storage` functions are called from async tasks (e.g., NT-MEMORY KB operations triggered by NT-MIND SEAL pipeline), the synchronous `BufReader`/`BufWriter` blocks the tokio worker. KB operations can involve large files (embeddings, experience logs).

**Fix**: Use `tokio::fs::File` + `tokio::io::BufReader`/`BufWriter` for async file I/O, or ensure callers always use `spawn_blocking`.

---

## Defect 7: Cooperative Scheduling Not Configured — FuturesUnordered Quadratic Risk

**File**: Project-wide (tokio runtime configuration)

**Evidence**: NeoTrix uses `tokio::spawn` extensively (MITM proxy, NT-SHIELD network pool, NT-MIND background loop). If any module uses `FuturesUnordered` (common in concurrent crawl/absorption pipelines), the lack of cooperative scheduling budget (`coop_budget`) means tasks that continuously return `Poll::Ready` can starve others.

**Pattern**: ScyllaDB post (2022) — `FuturesUnordered` caused quadratic slowdown because tasks never yielded. Tokio 1.22+ has cooperative scheduling by default (budget of 612 poll points), but custom `FuturesUnordered` usage or tight poll loops can still cause starvation.

**Defect**: No explicit `tokio::runtime::Builder::on_thread_park` or cooperative scheduling configuration exists in the codebase. While Tokio defaults are usually sufficient, NT-SHIELD's proxy kernel runs tight `copy_bidirectional` loops that may exhaust the cooperative budget.

**Fix**: Ensure all tight poll loops (especially in `copy_bidirectional`, `ObfuscatedStream::poll_read`, `XtlsStream::poll_read`) include `Poll::Pending` return paths. Audit `FuturesUnordered` usage in NT-WORLD crawl pipelines.

---

## Summary Table

| # | Defect | Severity | Domain | File |
|---|--------|----------|--------|------|
| 1 | Unbuffered AsyncRead in MITM — syscall storm | High | NT-SHIELD | mitm.rs:168 |
| 2 | Unbounded write_buf in XtlsStream — OOM | High | NT-SHIELD | vless/mod.rs:308 |
| 3 | ObfuscatedStream busy-loop on large frames | Medium | NT-SHIELD | security.rs:515 |
| 4 | PeekedStream ReadBuf semantics edge case | Low | NT-SHIELD | mitm.rs:23 |
| 5 | Blocking std::io in LSP client | Medium | NT-MIND | lsp_client/client.rs:120 |
| 6 | Synchronous BufReader in KB storage | Medium | NT-MEMORY | knowledge_storage.rs:76 |
| 7 | Cooperative scheduling gap in tight poll loops | Medium | Project-wide | runtime config |

**Common Root Cause**: NeoTrix's NT-SHIELD proxy kernel implements 4 custom `AsyncRead`/`AsyncWrite` types (`PeekedStream`, `ObfuscatedStream`, `XtlsStream`, `VlessStream`) without consistent buffering, backpressure, or cooperative yielding strategies. The NT-MIND and NT-MEMORY domains use synchronous I/O primitives in async-eligible code paths.
