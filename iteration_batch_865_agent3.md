# Agent 3: Async I/O Patterns (Batch 865)

## Sources

1. **Tokio docs: AsyncRead/AsyncWrite traits** — `tokio::io` trait definitions, `ReadBuf`, `poll_read` semantics, `BufReader`/`BufWriter` wrappers
2. **Rust Patterns Book: Async I/O Patterns** — Buffered streams, backpressure, connection pooling, codec framing, `select!` fairness
3. **Microsoft Async Rust: Common Pitfalls** — Executor blocking, `MutexGuard` across `.await`, cancellation hazards, no async drop, `select!` starvation
4. **Rust FAQ: BufReader/BufWriter performance** — Buffer size myths, silent flush failures, borrowing conflicts, `into_inner` data loss
5. **Tokio tutorial: I/O** — `AsyncReadExt::read` EOF semantics, `write_all` not cancellation-safe, `tokio::fs` uses `spawn_blocking` internally
6. **Tokio fs docs** — `set_max_buf_size`, batching `spawn_blocking` calls, `BufWriter` critical for perf
7. **DeepWiki: futures-rs AsyncRead/AsyncWrite** — `futures` vs `tokio` trait divergence, `ReadBuf` design rationale
8. **Internal Rust: Going from AsyncWrite to AsyncRead** — `!Send` constraints, borrow conflicts between read/write halves
9. **GitHub tokio #2716** — New `AsyncRead`/`AsyncWrite` proposals, `io_uring` future-proofing
10. **NeoTrix codebase** — `mitm.rs`, `security.rs`, `vless/mod.rs`, `kernel.rs`, `local_proxy.rs`, `knowledge_storage.rs`, `traits.rs`, `handlers_absorption.rs`, `tables.rs`

## Defects

### D-IO-001: PeekedStream lacks pin-project safety — fragile manual pin projection
**File:** `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:23-55`
**Severity:** MEDIUM

PeekedStream manually implements `AsyncRead`/`AsyncWrite` using `self.get_mut()` without `pin-project` or `pin-project-lite`. For `Unpin` types this is safe today, but if `T` is ever changed to `!Unpin`, the code silently becomes unsound. The Microsoft Async Rust pitfall #9 warns: "Pin management only becomes necessary when implementing custom Future types or working with async_stream patterns." The manual `Pin::new(&mut self.get_mut().inner)` bypasses compiler pin-safety checks.

**Fix:** Add `pin-project-lite` dependency and use `#[pin_project]` on PeekedStream, or add explicit `where T: Unpin` bounds with a compile-time check comment.

---

### D-IO-002: ObfuscatedStream::poll_write buffers data without backpressure — unbounded memory growth
**File:** `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/security.rs:434-448`
**Severity:** HIGH

`poll_write` for `ObfuscatedStream` unconditionally appends to `write_buf` (`self.write_buf.extend_from_slice(buf)`) and always returns `Poll::Ready(Ok(buf.len()))`. This means the caller thinks the write succeeded immediately, but data only enters the internal buffer — it is not flushed until `poll_flush` is called. If the upstream is slow or the flush loop stalls, `write_buf` grows without bound. The Rust Patterns Book Pattern 3 warns: "Fast producer overwhelms slow consumer causing unbounded memory growth." The Tokio docs explicitly state: "When `write()` returns, data may only be in the internal buffer — flush forces it to the wire."

**Fix:** Return `Poll::Pending` when `write_buf` exceeds a threshold (e.g., 256KB), or implement a maximum buffer capacity check in `poll_write` that triggers `poll_flush` internally.

---

### D-IO-003: XtlsStream::poll_read allocates per-read — defeats buffered I/O purpose
**File:** `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/connector/vless/mod.rs:266-297`
**Severity:** MEDIUM

Every `poll_read` call does `self.read_buf.resize(buf.remaining(), 0)` + `ReadBuf::new(&mut self.read_buf)` + allocates a new `Vec`. For high-throughput proxied traffic, this means thousands of small allocations per second. The Rust Performance Book states: "If you have many small and repeated read or write calls, use BufReader or BufWriter. They maintain an in-memory buffer." The existing `read_buf: BytesMut` is allocated once but resized per call, which for `BytesMut` may trigger reallocation. Meanwhile, the `decrypt()` call also allocates a new `Vec<u8>` for plaintext.

**Fix:** Pre-allocate a fixed-size read buffer (e.g., 8KB or 16KB) and use `BufReader` wrapping on the inner stream. Use `BytesMut::reserve` + `unsafe` uninitialized fill pattern to avoid re-zeroing, or use `read_buf` reuse across calls.

---

### D-IO-004: XtlsStream::poll_write returns buf.len() even if inner poll_write wrote less — data integrity violation
**File:** `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/connector/vless/mod.rs:301-320`
**Severity:** HIGH

When `encrypt` succeeds and `inner.poll_write(cx, &encrypted)` returns `Poll::Ready(Ok(n))` where `n < encrypted.len()`, the method still returns `Poll::Ready(Ok(buf.len()))`. This tells the caller that all plaintext bytes were consumed, but only a partial write of the encrypted form occurred. The Tokio docs warn: "`write` may return fewer bytes than requested — callers MUST handle partial writes." For an encryption layer, this means plaintext is acknowledged as sent but its encrypted form was partially dropped — silent data corruption on the wire.

**Fix:** Track how many bytes of `encrypted` were actually written, map that back to plaintext bytes consumed, and return the correct count. If `n < encrypted.len()`, buffer the remaining encrypted bytes for the next `poll_write` call.

---

### D-IO-005: MitmProxy `handle_http` reads entire response into memory — OOM on large responses
**File:** `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:238-260`
**Severity:** HIGH

The `handle_http` function reads the upstream response into `let mut resp = Vec::new()` in a loop, extending it without any size limit. For large HTTP responses (video streams, file downloads), this grows without bound until OOM. The Tokio tutorial explicitly warns: "For large files, consider memory-mapped I/O or streaming reads." The Rust Patterns Book Pattern 1 states: "Use streaming to avoid loading everything into memory." The `max_body_capture` config is only used for *logging*, not for limiting the actual `resp` buffer.

**Fix:** Stream the response directly to the client using `tokio::io::copy` with a `BufReader`/`BufWriter` pair, capturing only the first N bytes for analysis. Use `tokio::io::copy_bidirectional` like the CONNECT handler does.

---

### D-IO-006: LocalProxy `relay()` uses `split()` without BufReader/BufWriter — syscall overhead
**File:** `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/local_proxy.rs:205-208`
**Severity:** MEDIUM

The `relay` function splits the TCP streams and uses raw `tokio::io::copy` without `BufReader`/`BufWriter` wrappers. Every `copy` iteration issues individual `poll_read`/`poll_write` syscalls. The Rust FAQ warns: "Every single byte you read triggers a system call, crossing the boundary between user space and kernel space. That context switch costs cycles. Doing it millions of times turns a fast program into a slow one." The `BufReader`/`BufWriter` documentation explicitly states: "Improves speed of programs that make small and repeated read/write calls."

**Fix:** Wrap both read and write halves in `BufReader`/`BufWriter` before `tokio::io::copy`. Alternatively, use `tokio::io::copy_buf` with buffered streams.

---

### D-IO-007: KnowledgeStorage uses sync `std::io::BufReader` in async context — executor blocking risk
**File:** `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/knowledge_storage.rs:76,182`
**Severity:** LOW (file I/O, not network, but still blocks if called from async context)

`KnowledgeStorage` uses `std::io::BufReader::new(file).lines()` (sync) and `BufWriter::new(f)` (sync) for journal replay and compaction. These are synchronous file operations. If `load()` or `compact()` is called from an async context (e.g., from a background handler), they block the executor thread. The Microsoft Async Rust pitfall warns: "The #1 mistake: running blocking code on the async executor thread. This starves other tasks." The Tokio docs state: "Tokio will use ordinary blocking file operations behind the scenes using `spawn_blocking`."

**Fix:** Since `KnowledgeStorage` is currently used synchronously in the KB module, this is acceptable. Add a comment documenting that `open()`/`compact()`/`flush()` must NOT be called from async contexts, or wrap them in `spawn_blocking` if async usage is planned.

---

### D-IO-008: ProxyKernel `run_bidirectional` — dual `tokio::io::copy` in `select!` without half-close
**File:** `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:525-548`
**Severity:** MEDIUM

When one direction of `tokio::io::copy` completes (e.g., client sends FIN), the `select!` drops the other direction's future. While `tokio::io::copy` is cancellation-safe at the byte level, the dropped direction does NOT send a TCP FIN/half-close to the remote peer. The Microsoft Async Rust pitfall warns: "Dropping a future cancels it — but this can leave things in an inconsistent state." The remote peer may continue writing data into a void, and the connection lingers until `idle_timeout` fires. The Oxide RFD 400 notes: "cancellation is an example of 'spooky action at a distance'."

**Fix:** After one direction completes, explicitly shut down the other direction's write half using `write_half.shutdown().await` before dropping. Use `tokio::io::split` + explicit shutdown instead of relying on `select!` drop semantics.

---

### D-IO-009: ObfuscatedStream `flush_write_buf` partial write recovery appends to write_buf — state corruption on error
**File:** `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/security.rs:385-410`
**Severity:** MEDIUM

When `poll_write` on the inner stream returns `Pending` during `flush_write_buf`, the code does `self.write_buf.extend_from_slice(&self.frame_buf[written..])`. But `write_buf` may already have pending data from previous failed flushes, and the next `flush_write_buf` call will attempt to flush the *entire* `write_buf` again (including already-flushed bytes). The frame structure (length prefix + payload + padding + MAC) is self-contained, so re-sending already-sent bytes would corrupt the protocol stream for the peer.

**Fix:** Track a separate `pending_flush_buf: Vec<u8>` for partially-sent frame data, or use `drain` to remove already-sent bytes from `write_buf` before re-appending unsent bytes.

---

### D-IO-010: handlers_absorption stdin write with fire-and-forget — silent data loss on pipe full
**File:** `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers_absorption.rs:182-190`
**Severity:** LOW

The absorption handler spawns a tokio task to write session JSON to the CLI's stdin with `let _ = stdin.write_all(payload.as_bytes()).await; let _ = stdin.shutdown().await;`. The `let _ =` silently discards both write and shutdown errors. If the child process exits before consuming stdin (e.g., OOM or crash), the pipe fills, `write_all` returns an error, and the absorption silently fails while the handler reports success (since `child.wait_with_output()` may still succeed with partial stdin).

**Fix:** Await the write result and propagate errors to set `all_ok = false`. Use `tokio::io::AsyncWriteExt::write_all` with proper error handling in the spawned task.

---

## Key Insights

1. **Custom AsyncRead/AsyncWrite implementations are the most defect-prone area.** Three separate stream wrappers (PeekedStream, ObfuscatedStream, XtlsStream) all have manual pin projection without safety guarantees. The Rust ecosystem strongly recommends `pin-project-lite` for any custom implementation.

2. **Buffered I/O is systematically underused.** The proxy kernel, local proxy, and MITM proxy all use raw `tokio::io::copy` or manual read/write loops without `BufReader`/`BufWriter`. For a high-throughput proxy system, this means thousands of unnecessary syscalls per second.

3. **Backpressure is missing from the obfuscation layer.** `ObfuscatedStream::poll_write` always returns success immediately, letting the caller believe data is sent when it is only buffered. This creates a potential OOM vector under load.

4. **Partial write handling is a cross-cutting deficiency.** Both `XtlsStream::poll_write` (returns `buf.len()` regardless of actual write) and `ObfuscatedStream::flush_write_buf` (re-appends already-sent bytes) violate the `AsyncWrite` contract that partial writes must be reported accurately.

5. **Cancellation semantics in bidirectional proxy forwarding need explicit half-close.** The current `select!`-based pattern silently orphans one direction without sending TCP FIN, leading to connection leaks until idle timeout fires.

6. **Sync file I/O in `KnowledgeStorage` is acceptable today** but becomes a defect if the module is ever called from async context. A `// SAFETY: must not be called from async context` comment would prevent future misuse.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources analyzed | 10 |
| Files examined | 9 |
| HIGH severity | 3 (D-IO-002, D-IO-004, D-IO-005) |
| MEDIUM severity | 5 (D-IO-001, D-IO-003, D-IO-006, D-IO-008, D-IO-009) |
| LOW severity | 2 (D-IO-007, D-IO-010) |
