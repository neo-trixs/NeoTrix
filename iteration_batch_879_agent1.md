# Iteration Batch 879 — Agent 1: Deep Research: Rust Async IO Patterns (AsyncRead, AsyncWrite, BufReader)

**Date**: 2026-09-07
**Research Domain**: Rust async I/O — AsyncRead/AsyncWrite traits, BufReader/BufWriter, buffered streams, cancellation safety, backpressure
**Sources**: 12 (Tokio docs, Biriukov I/O patterns series, Rust Patterns Book, Microsoft Async Rust training, xfbs AsyncRead design critique, Rust wg-async-foundations, Rust Networking Part 4 framing, cong-or blocking-async-rust, Boardor async trait perf traps, Stack Overflow BufReader chunks, learnrust.net async pitfalls)

---

## Source Index

| # | Source | Key Insight |
|---|--------|-------------|
| 1 | Tokio docs — `tokio::io` (docs.rs/tokio) | AsyncRead/AsyncWrite readiness model, BufReader default 8KB, BufWriter must flush |
| 2 | Biriukov — Tokio I/O patterns (biriukov.dev) | `io::split()` uses internal mutex, TLS split causes deadlocks, connection-driver pattern preferred |
| 3 | Biriukov — I/O loop backpressure | `select!` loop-select blocks reads when writes stall, read starvation under backpressure |
| 4 | Biriukov — Async Rust with Tokio IO streams | Cancellation: `write_all` in `select!` branch is not cancel-safe, nested `select!` for cancellation scales poorly |
| 5 | Rust Patterns Book — Async I/O patterns | BufReader reduces syscalls 100-1000x, BufWriter final flush critical, codec pattern for framed I/O |
| 6 | Microsoft Async Rust training — Common pitfalls | #1 mistake: blocking executor, MutexGuard across .await, cancellation drops future at await point, no async Drop |
| 7 | xfbs — AsyncRead design critique | Tokio file writes return before data hits disk (`poll_write` returns `Ready` before kernel flush), `flush()` required |
| 8 | Rust Networking Part 4 — Framing | `BytesMut::split_to` zero-copy framing, length-prefix guard against 16MB frames, `read_exact` for partial reads |
| 9 | cong-or — Blocking in async Rust | `std::fs` in async blocks workers, hud eBPF profiler detects scheduling latency, `spawn_blocking` for sync I/O |
| 10 | Boardor — Async trait perf traps | `dyn Trait` async methods box futures: 340% memory spike, 89% throughput loss at 10K req/s; generics eliminate boxing |
| 11 | learnrust.net — Async pitfalls | Cancellation at await points: cleanup must be in `Drop`, not after await; `!Send` across .await breaks `tokio::spawn` |
| 12 | Rust wg-async-foundations — Read/write traits | Simultaneous read/write requires split; SSL read may trigger write; future may need `async fn split()` trait method |

---

## Extracted Defects (7 Total)

### D-IO-009: `XtlsStream::poll_write` returns `buf.len()` regardless of actual bytes written — violates AsyncWrite partial-write contract

**File**: `nt_shield_proxy_kernel/connector/vless/mod.rs:308-338`
**Severity**: HIGH
**Sources**: 1, 6, 10, 12

**Description**: The `poll_write` implementation returns `Poll::Ready(Ok(buf.len()))` even when the inner stream accepts fewer bytes than requested. Tokio's `AsyncWrite` contract explicitly states: "If the writer is not ready to accept bytes, it returns `Poll::Pending`. If the writer is ready, it returns the number of bytes accepted." Returning the full `buf.len()` on a partial write causes the caller to believe all data was consumed. The buffered remainder in `self.write_buf` is silently re-appended on the next flush, but the *caller* has already advanced past those bytes — creating an invisible divergence between the reported write position and the actual wire position.

**Evidence**:
```rust
// vless/mod.rs:332-334
std::task::Poll::Ready(Ok(n)) => {
    if n < encrypted.len() {
        self.write_buf.extend_from_slice(&encrypted[n..]);
    }
    std::task::Poll::Ready(Ok(buf.len())) // BUG: returns original buf.len(), not bytes consumed
}
```

**Fix**: Return `Ok(n)` where `n` is the number of plaintext bytes whose encrypted form was fully committed. For a partial write of encrypted bytes, return 0 (none of the plaintext was fully written) or buffer the plaintext and return `Pending`. The cleanest approach: encrypt into `write_buf`, attempt `poll_write` on inner, on partial return 0 and keep remainder, on full drain return `buf.len()`.

**Impact**: Data corruption in XTLS tunnel under backpressure. The caller advances its read pointer by `buf.len()` bytes, but some bytes are buffered in `write_buf`. On the next call, `write_buf` flush first, then new plaintext encrypts — resulting in the same plaintext being sent twice or plaintext ordering corruption.

---

### D-IO-010: `ObfuscatedStream::poll_write` always returns `Poll::Ready(Ok(buf.len()))` — zero backpressure, unbounded memory growth

**File**: `nt_shield_proxy_kernel/security.rs:492-501`
**Severity**: HIGH
**Sources**: 1, 3, 5, 9

**Description**: `ObfuscatedStream::poll_write` unconditionally appends `buf` to `self.write_buf` and returns `Ready(Ok(buf.len()))` without ever attempting to flush to the inner stream. The actual flush only happens in `poll_flush` (called by `BufWriter` or explicit `.flush().await`). If the consumer writes rapidly without flushing, `write_buf` grows without bound — this is the classic "write-back pressure bypass" that Tokio's bounded-channel pattern exists to prevent.

**Evidence**:
```rust
// security.rs:494-501
fn poll_write(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
    self.write_buf.extend_from_slice(buf);
    Poll::Ready(Ok(buf.len())) // Never checks inner stream readiness
}
```

**Fix**: `poll_write` should attempt to flush `write_buf` to the inner stream first (calling `flush_write_buf`), then append the new data, then attempt another flush. If the inner stream is not ready, the new data should be buffered but the method should return the number of bytes accepted (bounded by a configurable high-water mark, e.g., 256KB). The current design relies entirely on the caller calling `poll_flush` — but `copy_bidirectional` does NOT guarantee flush calls between write batches.

**Impact**: Under high-throughput proxied traffic (e.g., large file downloads), `write_buf` can grow to gigabytes before `poll_flush` is called, causing OOM kills.

---

### D-IO-011: `ObfuscatedStream::flush_write_buf` re-appends already-sent bytes on partial write — duplicate frame transmission

**File**: `nt_shield_proxy_kernel/security.rs:455-489`
**Severity**: MEDIUM
**Sources**: 1, 6, 12

**Description**: When `flush_write_buf` encounters a partial write (`n < self.frame_buf.len()`), it does `self.write_buf.extend_from_slice(&self.frame_buf[written..])`. However, `self.write_buf` was already cleared earlier at line 471 (`self.write_buf.clear()`). The problem is that `self.write_buf` now contains the *encrypted frame bytes*, not the *original plaintext*. On the next flush cycle, these frame bytes get re-wrapped with a new length prefix, padding, and HMAC — resulting in a double-framed message on the wire. The remote peer's decoder will read the outer frame, extract the inner frame bytes as "payload", and attempt to decode them — causing a decode error or protocol desync.

**Evidence**:
```rust
// security.rs:471
self.write_buf.clear(); // Clears original plaintext
// security.rs:486-488
Poll::Pending => {
    self.write_buf.extend_from_slice(&self.frame_buf[written..]); // Re-appends partially-sent frame bytes as if they were plaintext
    return Poll::Pending;
}
```

**Fix**: After `self.write_buf.clear()`, the partially-sent frame bytes should either be (a) stored in a separate `pending_frame` field and flushed on the next `poll_flush` without re-framing, or (b) the entire `frame_buf` should be kept as-is and retried without re-adding padding/HMAC. The current approach fundamentally corrupts the framing protocol.

---

### D-IO-012: `XtlsStream` and `ObfuscatedStream` use manual `Pin::new(&mut self.inner)` projection without `pin-project-lite` — unsound if `T` becomes `!Unpin`

**File**: `vless/mod.rs:266-340`, `security.rs:397-520`
**Severity**: MEDIUM
**Sources**: 1, 6, 11, 12

**Description**: Both `XtlsStream` and `ObfuscatedStream` implement `AsyncRead`/`AsyncWrite` using `Pin::new(&mut self.inner)` which is only safe when `T: Unpin`. The code does enforce `T: Unpin` via trait bounds today, but the manual `self.get_mut()` pattern in `PeekedStream` (mitm.rs:23-53) does not use pin-project at all — it calls `self.get_mut()` directly on a `Pin<&mut Self>`, which is only sound for `Unpin` types but has no compile-time enforcement. If any of these stream wrappers is ever composed with a `!Unpin` inner type (e.g., a future, a self-referential buffer), the pin projection silently becomes unsound, leading to undefined behavior.

**Evidence**:
```rust
// mitm.rs:24-28 — PeekedStream uses self.get_mut() without Unpin bound check
let this = self.get_mut(); // Only safe if Self: Unpin, not enforced
Pin::new(&mut this.inner).poll_read(cx, buf)
```

The `pin-project-lite` crate generates safe pin projections with zero overhead and compiler-verified `Unpin` propagation.

**Fix**: Add `pin-project-lite` as a dependency. Refactor all three stream wrappers:
```rust
use pin_project_lite::pin_project;
pin_project! {
    struct PeekedStream<T> {
        peeked: Vec<u8>,
        pos: usize,
        #[pin]
        inner: T,
    }
}
```
Then use `this.inner.poll_read(cx, buf)` (projected pin) instead of `Pin::new(&mut this.inner)`.

---

### D-IO-013: `XtlsStream` does not clear `read_buf` between `poll_read` calls — stale encrypted data persists

**File**: `vless/mod.rs:273-297`
**Severity**: MEDIUM
**Sources**: 1, 7, 10

**Description**: `XtlsStream::poll_read` calls `self.read_buf.resize(buf.remaining(), 0)` before reading, which resizes but does NOT clear the existing contents. `BytesMut::resize` preserves existing bytes when growing — only the new portion is zero-filled. If a previous `poll_read` filled `read_buf` with 100 bytes of encrypted data, and the next call resizes to 512, bytes 0-99 still contain the old data. The `ReadBuf::new(&mut self.read_buf)` then wraps the entire 512-byte buffer, and the inner `poll_read` fills from byte 0 — overwriting the stale prefix. This works by accident (the inner read overwrites the stale data), but creates a subtle invariant: `read_buf` must never be read between calls except via `ReadBuf::filled()`. If any future code path reads `self.read_buf[..n]` directly (as it does at line 283), it may process stale bytes from a prior call.

**Evidence**:
```rust
// vless/mod.rs:278-283
self.read_buf.resize(buf.remaining(), 0); // Does NOT clear existing bytes
let mut read_buf = tokio::io::ReadBuf::new(&mut self.read_buf);
match std::pin::Pin::new(&mut self.inner).poll_read(cx, &mut read_buf) {
    // ...
    let n = read_buf.filled().len();
    // self.read_buf[..n] contains old data overwritten by inner read — works by accident
    match self.encryptor.decrypt(&self.read_buf[..n]) { // Line 283
```

**Fix**: Add `self.read_buf.clear();` before `self.read_buf.resize(...)`, or use `self.read_buf.resize(0, 0); self.read_buf.resize(buf.remaining(), 0);` to reset before resize. Alternatively, use a fixed-size buffer and track fill length separately.

---

### D-IO-014: `knowledge_storage.rs` uses synchronous `std::io::BufReader` / `BufWriter` for journal replay and compaction — blocks Tokio executor

**File**: `nt_memory_kb/knowledge_storage.rs:76,182`
**Severity**: MEDIUM
**Sources**: 1, 6, 9, 11

**Description**: `KnowledgeStorage::load()` calls `BufReader::new(file).lines()` (line 76) and `compact()` calls `BufWriter::new(f)` (line 182) using `std::io` (synchronous). These are blocking file operations. If `load()` or `compact()` is called from a Tokio task (even indirectly via the background loop or consciousness tick), they block the worker thread for the entire duration of journal replay. For a knowledge base with thousands of entries, this can block for hundreds of milliseconds — freezing all other tasks on that worker.

The Tokio docs explicitly state: "Tasks that perform blocking operations should be spawned on the dedicated blocking pool using spawn_blocking." The `tokio::fs` module internally uses `spawn_blocking` for all file operations.

**Evidence**:
```rust
// knowledge_storage.rs:14
use std::io::{BufRead, BufReader, BufWriter, Write};
// knowledge_storage.rs:76
for line in BufReader::new(file).lines() { // Blocking line-by-line read
// knowledge_storage.rs:182
let mut w = BufWriter::new(f); // Synchronous buffered writer
```

**Fix**: Wrap `load()` and `compact()` calls in `tokio::task::spawn_blocking`. Or migrate to `tokio::fs::File` + `tokio::io::BufReader` + `tokio::io::AsyncBufReadExt` for fully async I/O. The `spawn_blocking` approach is simpler and avoids rewriting the serialization logic.

---

### D-IO-015: `neotrix_dl.rs` writes HTTP download chunks to `tokio::fs::File` without `BufWriter` — syscall per chunk

**File**: `neotrix_dl.rs:122`
**Severity**: LOW
**Sources**: 1, 5, 8, 9

**Description**: `neotrix_dl.rs` downloads large GGUF model files (multi-GB) and writes each HTTP response chunk directly to a `tokio::fs::File` via `file.write_all(&chunk).await?`. While `tokio::fs::File` uses a blocking thread pool internally, each `write_all` still triggers a separate write syscall. For typical HTTP chunk sizes (16KB-64KB), downloading a 4GB model triggers 65,000-260,000 individual write calls. Adding a `BufWriter` with 256KB capacity would batch these into ~16,000-65,000 syscalls — a 4x reduction.

**Evidence**:
```rust
// neotrix_dl.rs:122
file.write_all(&chunk).await?;
```
The explicit `file.flush().await?` at line 134 is correct but the intermediate writes are suboptimal.

**Fix**: Wrap `file` in `tokio::io::BufWriter::with_capacity(256 * 1024, file)`. Flush before dropping.

---

## Pattern Summary

| Pattern | Occurrences | Severity Range |
|---------|-------------|----------------|
| AsyncWrite partial-write contract violation | 2 (XtlsStream, ObfuscatedStream) | HIGH, MEDIUM |
| Manual pin projection without pin-project | 3 (XtlsStream, ObfuscatedStream, PeekedStream) | MEDIUM |
| Synchronous IO in async context | 2 (knowledge_storage, neotrix_dl) | MEDIUM, LOW |
| Missing BufWriter for batched writes | 3 (neotrix_dl, local_proxy relay, proxy listeners) | LOW, MEDIUM |
| Unbounded buffer growth (no backpressure) | 1 (ObfuscatedStream) | HIGH |

## Recommendations

1. **Immediate (HIGH)**: Fix `XtlsStream::poll_write` return value and `ObfuscatedStream::flush_write_buf` re-frame bug — these are data corruption risks under load
2. **Short-term (MEDIUM)**: Adopt `pin-project-lite` for all custom AsyncRead/AsyncWrite impls; add `self.read_buf.clear()` to XtlsStream
3. **Medium-term**: Wrap `knowledge_storage` sync IO in `spawn_blocking`; add `BufWriter` to download and proxy paths
4. **Systemic**: Add a codebase-wide audit for `AsyncWrite::poll_write` implementations to verify partial-write contract compliance — every manual impl should return actual bytes written, not `buf.len()`
