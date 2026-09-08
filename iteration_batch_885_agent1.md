# Iteration Batch 885 — Agent 1: Rust Async IO Patterns Deep Research

**Date**: 2026-09-07
**Topic**: AsyncRead, AsyncWrite, BufReader — Defect Analysis for NeoTrix

---

## 1. Research Summary

### Core Traits

| Trait | Purpose | Key Method |
|-------|---------|------------|
| `AsyncRead` | Async version of `std::io::Read` | `poll_read(Pin<&mut Self>, cx, buf: &mut ReadBuf) -> Poll<io::Result<()>>` |
| `AsyncWrite` | Async version of `std::io::Write` | `poll_write`, `poll_flush`, `poll_shutdown` |
| `AsyncBufRead` | Buffered async reading | `poll_fill_buf` / `consume` pair |

### Tokio IO Architecture

- **Readiness model**: Poll for readiness → attempt I/O → handle `WouldBlock`. Zero-cost when I/O is immediately ready.
- **Extension traits** (`AsyncReadExt`, `AsyncWriteExt`, `AsyncBufReadExt`) provide ergonomic `.await` methods wrapping the raw `poll_*` interface.
- **Splitting**: `tokio::io::split()` uses `Arc<Mutex<>>` internally. Zero-cost variants exist for `TcpStream` (`split`/`into_split`).
- **Buffered I/O**: `BufReader`/`BufWriter` reduce syscalls via in-memory buffers (default 8KB).

---

## 2. Extracted Defects for NeoTrix

### DEFECT-1: `poll_read` Pre-filled Buffer Footgun — Silent Double-Processing

**Severity**: High
**Component**: NT-WORLD (crawlers), NT-MEMORY (KB ingestion), any custom `AsyncRead` impl

**Description**: When implementing `AsyncRead::poll_read`, the `ReadBuf` may already be partially filled from a prior call. Code that reads `buf.filled()` without tracking the pre-call filled length will hash/process the same bytes twice.

**Buggy pattern** (from real production bug — Blake3 hashing reader):
```rust
fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
    let inner_poll = self.project().inner.poll_read(cx, buf);
    // BUG: buf.filled() includes bytes from BEFORE this call
    b3sum.lock().update(buf.filled());
    inner_poll
}
```

**Correct pattern**:
```rust
fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
    let already_filled = buf.filled().len();
    let inner_poll = self.project().inner.poll_read(cx, buf);
    if let Poll::Ready(Ok(_)) = inner_poll {
        b3sum.lock().update(&buf.filled()[already_filled..]);
    }
    inner_poll
}
```

**NeoTrix Risk**: Any custom `AsyncRead` wrapper in NT-WORLD (crawler stream processing), NT-MEMORY (KB embedding ingestion), or NT-ACT (tool output capture) that computes checksums, hashes, or metrics over `buf.filled()` will silently double-count data on every `poll_read` call.

**Fix**: Audit all `AsyncRead` implementations for `buf.filled()` usage. Always snapshot `buf.filled().len()` before delegating to inner `poll_read`.

---

### DEFECT-2: EOF Loop Infinite Spin — 100% CPU

**Severity**: High
**Component**: NT-WORLD (stream readers), NT-IO (LLM response streaming)

**Description**: When `AsyncRead::read()` returns `Ok(0)`, it signals EOF. Forgetting to break from the read loop causes the future to spin at 100% CPU because `read()` returns immediately with `Ok(0)` on every subsequent call.

**Buggy pattern**:
```rust
loop {
    let n = reader.read(&mut buf).await?;
    // BUG: no EOF check — infinite loop on closed connection
    process(&buf[..n]);
}
```

**Correct pattern**:
```rust
loop {
    let n = reader.read(&mut buf).await?;
    if n == 0 {
        break; // EOF — stream closed
    }
    process(&buf[..n]);
}
```

**NeoTrix Risk**: NT-WORLD crawlers reading HTTP response streams, NT-IO reading LLM streaming responses, NT-MEMORY reading KB export streams. If a remote endpoint closes the connection and the EOF check is missing, the task spins at 100% CPU indefinitely, starving other tasks on the Tokio runtime.

**Fix**: Grep all `loop` + `.read()` combinations across the codebase. Ensure every read loop has an `if n == 0 { break }` guard.

---

### DEFECT-3: BufReader Data Loss on Drop / into_inner / Multiple Wrapping

**Severity**: Medium-High
**Component**: NT-WORLD (file parsing), NT-IO (stream handling), NT-MEMORY (KB import)

**Description**: Three data-loss scenarios with `BufReader`:

1. **Drop without consume**: When a `BufReader` is dropped, its internal buffer contents are silently discarded.
2. **`into_inner()` discards leftovers**: Calling `into_inner()` returns the underlying reader but loses any buffered-but-not-yet-read bytes.
3. **Multiple `BufReader` wrapping**: Creating multiple `BufReader` instances on the same stream causes data loss — the second `BufReader` reads from the underlying reader (bypassing the first's buffer), skipping bytes the first had already buffered.

**NeoTrix Risk**: NT-WORLD crawling pipelines that wrap file/network streams in `BufReader`, then re-wrap or drop them during error handling or pipeline teardown, will silently lose data. KB import/export routines using `BufReader::into_inner()` mid-parse will corrupt data streams.

**Fix**:
- Never call `into_inner()` on a `BufReader` that still has unconsumed buffered data.
- Never create a second `BufReader` on the same underlying reader.
- In error paths, ensure `BufReader` is fully consumed before dropping, or explicitly `consume()` remaining bytes.

---

### DEFECT-4: Cancellation Safety Violation in `select!` — Data Corruption

**Severity**: High
**Component**: NT-ACT (tool orchestration), NT-IO (LLM streaming), NT-WORLD (concurrent fetchers)

**Description**: `read()`, `read_exact()`, and `read_line()` are **not** cancellation-safe. If used as events in `tokio::select!` and another branch completes first, partially-read data is lost with no recovery mechanism.

**Critical detail from Tokio docs**:
> This method is not cancellation safe. If the method is used as the event in a `tokio::select!` statement and some other branch completes first, then some data may have been partially read, and this data is lost.

**Buggy pattern**:
```rust
tokio::select! {
    result = reader.read_line(&mut line) => { /* data may be partially lost */ }
    _ = cancel.cancelled() => { /* line is in undefined state */ }
}
```

**Safe alternatives**:
- Use `fill_buf()` + `consume()` (both cancel-safe at the primitive level).
- Use `tokio_util::codec::LinesCodec` for framed line reading.
- Use the `lines()` stream whose `next_line()` is cancel-safe.

**NeoTrix Risk**: NT-ACT tool orchestration that multiplexes tool output reading with timeout/cancellation. NT-IO LLM streaming that reads response chunks with cancellation. NT-WORLD crawlers that fetch multiple URLs with timeout. Any `select!` involving `read`/`read_line`/`read_exact` risks silent data corruption.

**Fix**: Audit all `tokio::select!` blocks. Replace non-cancel-safe read methods with cancel-safe alternatives (`fill_buf`+`consume`, codec-based, or `lines()` stream).

---

### DEFECT-5: `io::split()` Mutex Overhead and TLS Deadlock Risk

**Severity**: Medium
**Component**: NT-IO (bidirectional LLM connections), NT-WORLD (TLS crawlers), NT-SHIELD (proxy connections)

**Description**: `tokio::io::split()` uses `Arc<Mutex<>>` internally for any `AsyncRead + AsyncWrite` type. Two problems:

1. **Mutex overhead**: Even within a single task, the mutex is acquired on every read/write. For high-throughput scenarios (crawler connections, LLM streaming), this is unnecessary overhead.
2. **TLS deadlock**: TLS transports couple reads and writes (a read may trigger a write for alerts/key updates). Splitting such streams into separate tasks can cause deadlocks, stuck futures, or missed wakeups — the read half blocks waiting on a write event that never fires.

**Reference**: Tokio GitHub issue on TLS split safety. Hyper and h2 avoid `io::split()` entirely, using a connection-driver pattern instead.

**NeoTrix Risk**: NT-IO bidirectional LLM connections using TLS. NT-WORLD HTTPS crawlers. NT-SHIELD Tor/proxy connections. All may deadlock or exhibit missed wakeups when split across tasks.

**Fix**:
- For non-TLS streams within a single task: use `TcpStream::split()` (zero-cost, no mutex).
- For TLS or cross-task splitting: implement a connection-driver pattern (single future driving the protocol state machine, channel-based actor interface for read/write).
- Document which streams are safe to split and which require the driver pattern.

---

### DEFECT-6: `poll_write` Buffer Contract Violation on Retry

**Severity**: Medium
**Component**: NT-ACT (tool output), NT-IO (LLM request writing), NT-MEMORY (KB writes)

**Description**: When `poll_write` returns `Poll::Pending` (or a partial write), the buffer passed to the next `poll_write` call may differ from the original. The caller must track the original buffer slice and only advance on successful partial writes.

**Key insight from Tokio docs**:
> There is no guarantee that the same buffer will be passed on subsequent calls to `poll_write`. The implementation may buffer internally, or may receive a different (but equivalent) buffer.

**Buggy pattern** (assumes same buffer):
```rust
fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
    // If pending, assumes same buf is passed next call — WRONG
    let n = ready!(inner.poll_write(cx, buf))?;
    self.pos += n;
    Poll::Ready(Ok(n))
}
```

**Correct pattern** (track position independently):
```rust
fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
    let start = self.pos;
    let remaining = &buf[start..];
    let n = ready!(inner.poll_write(cx, remaining))?;
    self.pos += n;
    Poll::Ready(Ok(n))
}
```

**NeoTrix Risk**: Custom `AsyncWrite` wrappers in NT-ACT (tool output buffering), NT-IO (LLM request serialization), or NT-MEMORY (KB batch writes) that assume buffer identity across `poll_write` calls will silently corrupt data or write partial messages.

**Fix**: All custom `AsyncWrite` implementations must track write position in `self`, never relying on the buffer pointer being the same across calls.

---

### DEFECT-7: `BufWriter` Flush-on-Drop Data Loss

**Severity**: Medium
**Component**: NT-MEMORY (KB persistence), NT-IO (LLM response buffering), NT-ACT (tool output)

**Description**: `BufWriter` does not flush on drop — buffered data is silently discarded. This is a common source of data loss in error paths and early returns.

```rust
// If this function returns early or panics, buffered writes are LOST
async fn write_kB_entry(writer: &mut BufWriter<File>, entry: &Entry) -> io::Result<()> {
    writer.write_all(&entry.serialize()).await?;
    writer.flush().await?; // Must explicitly flush!
    Ok(())
}
```

**Tokio docs**:
> `BufWriter` doesn't add any new ways of writing; it just buffers every call to `write`. However, you must flush `BufWriter` to ensure that any buffered data is written.

**NeoTrix Risk**: NT-MEMORY KB write operations using `BufWriter` that don't flush before returning. NT-IO LLM response buffering. NT-ACT tool output capture. Any early return or error propagation path that skips `flush()` will lose data silently.

**Fix**: Audit all `BufWriter` usage sites. Ensure `flush()` is called in all code paths including error handling. Consider using `tokio::pin!` + `scopeguard` or a wrapper that flushes on drop.

---

### DEFECT-8: Fake Async File I/O — Thread Pool Starvation

**Severity**: Low-Medium
**Component**: NT-WORLD (file-based crawlers), NT-MEMORY (KB file operations)

**Description**: On most platforms (Linux, macOS), Tokio's file I/O is not truly async — it dispatches to `spawn_blocking` (a thread pool). This means:

1. File I/O operations consume blocking thread pool slots (default: 512 threads).
2. Under heavy file I/O load, the blocking pool can saturate, stalling all `spawn_blocking` operations including non-file work.
3. `BufReader` wrapping a `tokio::fs::File` still blocks threads because the underlying file reads go through the thread pool.

**NeoTrix Risk**: NT-WORLD crawlers processing many files simultaneously. NT-MEMORY KB operations. If file I/O saturates the blocking pool, other blocking operations (DNS resolution, CPU-intensive tasks) will stall.

**Fix**:
- Use `tokio::task::spawn_blocking` with explicit pool sizing for heavy file I/O.
- For high-throughput file reading, consider `io_uring` (Linux 5.6+) via `tokio-uring` crate.
- Document that file I/O is not truly async and set appropriate concurrency limits.

---

## 3. Defect Summary Matrix

| # | Defect | Severity | Component | Fix Difficulty |
|---|--------|----------|-----------|----------------|
| 1 | `poll_read` pre-filled buffer double-processing | High | NT-WORLD/NT-MEMORY | Low (audit) |
| 2 | EOF loop infinite spin | High | NT-WORLD/NT-IO | Low (grep+fix) |
| 3 | BufReader data loss on drop/into_inner | Medium-High | NT-WORLD/NT-IO/NT-MEMORY | Medium |
| 4 | Cancellation safety violation in `select!` | High | NT-ACT/NT-IO/NT-WORLD | Medium |
| 5 | `io::split()` mutex + TLS deadlock | Medium | NT-IO/NT-SHIELD | High (redesign) |
| 6 | `poll_write` buffer contract violation | Medium | NT-ACT/NT-IO/NT-MEMORY | Low (audit) |
| 7 | `BufWriter` flush-on-drop data loss | Medium | NT-MEMORY/NT-IO/NT-ACT | Low (grep+fix) |
| 8 | Fake async file I/O thread pool starvation | Low-Medium | NT-WORLD/NT-MEMORY | Medium |

---

## 4. Recommended Actions

### Immediate (This Sprint)
1. **Grep for `buf.filled()` in all `AsyncRead` impls** — fix pre-filled buffer footgun (DEFECT-1)
2. **Grep for all `loop` + `.read()` patterns** — add EOF guards (DEFECT-2)
3. **Grep for all `BufWriter` usage** — ensure `flush()` in all code paths (DEFECT-7)
4. **Audit all `tokio::select!` blocks** — replace non-cancel-safe read methods (DEFECT-4)

### Short-Term (Next Sprint)
5. **Audit `BufReader` wrapping patterns** — eliminate double-wrapping and `into_inner()` mid-stream (DEFECT-3)
6. **Audit all custom `AsyncWrite` impls** — verify buffer position tracking (DEFECT-6)

### Long-Term
7. **Document connection-driver pattern** for TLS/bidirectional streams (DEFECT-5)
8. **Set explicit concurrency limits** for file I/O operations (DEFECT-8)

---

## 5. References

- [Tokio IO Patterns — Biriukov](https://biriukov.dev/docs/async-rust-tokio-io/3-tokio-io-patterns/)
- [Tokio AsyncRead docs](https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html)
- [Tokio BufReader docs](https://docs.rs/tokio/latest/tokio/io/struct.BufReader.html)
- [Async I/O Patterns — Rust Patterns Book](https://www.rust-patterns.com/book/21-async-io-patterns.html)
- [poll_read footgun — Ivan](https://gist.github.com/ivan/3c8a7dd325e7ffd3fb7bc3e46975b58a)
- [wg-async: Read/write traits roadmap](https://rust-lang.github.io/wg-async/vision/roadmap/portable/read_write.html)
- [Tokio BufReader lines hang — users.rust-lang.org](https://users.rust-lang.org/t/tokio-bufreader-lines-next-line-await-never-resolves-execution-paused-forever/109829)
