# Agent 3 — Batch 882: Rust Cancel-Safe Futures & `select!` Patterns

## Research Summary

Deep investigation into Rust async cancellation safety, `tokio::select!` semantics, and production failure patterns. Sources: Tokio docs, Oxide RFD 400, `cancel-safe-futures` crate (Oxide), RustConf 2025 talk by Rain Paharia, multiple production war stories (2025-2026).

---

## Core Concepts

**Cancel safety** = a future can be dropped at any `.await` point with no side effects (no data loss, no invariant violation).

**Cancel correctness** = global property: no cancel-unsafe future is ever cancelled in a way that violates a system invariant. Three prongs required:
1. A cancel-unsafe future exists
2. It is actually cancelled
3. The cancellation violates a system property

**`tokio::select!`** drops all non-winning branches immediately. Any `.await` inside a branch is a cancellation point. The compiler cannot detect cancel-unsafety — it's a semantic property.

---

## 5+ Defects Identified for NeoTrix

### DEFECT-1: `read_exact` / `read_to_end` inside `select!` branches — Silent Data Loss

**Severity**: High (data loss)  
**Files**: `nt_shield_proxy_kernel/listener/socks5.rs:131-168`, `nt_shield_stealth_net/local_proxy.rs:152-194`, `nt_shield_stealth_net/tor_client.rs:159-197`  
**Pattern**: `stream.read_exact(&mut buf).await` used in code paths that can be driven by `select!`-adjacent timeouts.

**Analysis**: `AsyncReadExt::read_exact` is explicitly documented as **NOT** cancel-safe. If cancelled mid-read, partially-filled buffer contents are lost silently. In `socks5.rs:131`:
```rust
stream.read_exact(&mut buf).await?;  // NOT cancel-safe
```
If this future is ever placed inside a `select!` branch (or wrapped in `tokio::time::timeout` which has identical drop semantics), partial reads are silently discarded. The SOCKS5 handshake would then fail with corrupted state.

**Fix**: Use `AsyncReadExt::read` (cancel-safe, returns partial read count) and accumulate into an external buffer, OR ensure these code paths are never inside `select!`/`timeout` boundaries.

---

### DEFECT-2: `write_all` in `select!`-adjacent contexts — Partial Write + Silent Loss

**Severity**: High (data corruption)  
**Files**: `nt_shield_proxy_kernel/listener/socks5.rs:120-232`, `nt_shield_traffic/mitm.rs:262-383`, `nt_shield_stealth_net/local_proxy.rs:331-351`  
**Pattern**: `stream.write_all(&data).await` used pervasively.

**Analysis**: `AsyncWriteExt::write_all` is documented as **NOT** cancel-safe. If dropped, you have no idea how many bytes were written. NeoTrix uses `write_all` extensively in SOCKS5/HTTP proxy code paths. While most current usage is in spawned tasks (not directly in `select!`), any future refactoring that adds shutdown tokens or timeouts to these paths will introduce silent corruption.

**Fix**: Migrate to `AsyncWriteExt::write_buf` or `tokio_util::io::poll_write_buf` which track partial progress via cursor advancement. Alternatively, ensure all `write_all` calls are in spawned tasks that run to completion.

---

### DEFECT-3: `tokio::sync::Mutex` held across `.await` — Cancellation Blast Radius

**Severity**: Medium-High (deadlock + invariant violation)  
**Files**: `nt_shield_proxy_kernel/dns_intercept.rs:14`, `nt_shield_traffic/api_proxy.rs:12`, `nt_shield_traffic/mitm.rs:8`, `nt_io_provider/llama_process.rs:10`, `nt_mind_background_loop/run.rs:7`  
**Pattern**: 12 files import `tokio::sync::Mutex`; several hold the guard across `.await` points.

**Analysis**: Oxide RFD 400 and RustConf 2025 explicitly recommend **avoiding `tokio::sync::Mutex`** entirely. The problem: when a future holding a `tokio::sync::Mutex` guard is cancelled (via `select!`, `timeout`, or task abort), the mutex is dropped but the state it protected may be in an inconsistent mid-variant state. Unlike `std::sync::Mutex`, there's no poisoning mechanism. The "cancellation blast radius" extends to all other futures waiting on that mutex.

**Evidence from codebase**: `nt_shield_traffic/api_proxy.rs` and `nt_shield_traffic/mitm.rs` both use `tokio::sync::Mutex` for HTTP connection state. If a request handler is cancelled mid-processing (e.g., client disconnect), the mutex-guarded state is left invalid, corrupting subsequent requests.

**Fix**: Replace with actor-model message passing (`mpsc` channels) or `std::sync::Mutex` if locks aren't held across `.await`. Use the `cancel-safe-futures` crate's `sync::Mutex` if shared mutable state is unavoidable.

---

### DEFECT-4: `select!` loop with `shutdown_rx.changed()` — Fairness Starvation Risk

**Severity**: Medium (liveness)  
**Files**: `nt_shield_proxy_kernel/kernel.rs:225-233`  
**Pattern**:
```rust
loop {
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(60)) => {
            security_for_gc.cleanup_stale_failures().await;
            kernel_for_gc.cleanup_stale_clients().await;
        }
        _ = shutdown_rx_gc.changed() => {
            if *shutdown_rx_gc.borrow() { break; }
        }
    }
}
```

**Analysis**: Without `biased;`, `select!` uses pseudo-random polling. If the `sleep` branch is consistently ready (e.g., fast GC cycles), the `shutdown_rx` branch may be starved. While `sleep` resets after firing (so not permanently ready), under high GC load the shutdown signal can be delayed significantly. More critically, the `changed()` method on `watch::Receiver` is cancel-safe but has a subtlety: if the receiver misses a change notification while it was not being polled, the `changed()` future may never resolve until the next change.

**Fix**: Add `biased;` with shutdown branch first, ensuring shutdown is always polled first:
```rust
tokio::select! {
    biased;
    _ = shutdown_rx_gc.changed() => { ... }
    _ = tokio::time::sleep(...) => { ... }
}
```

---

### DEFECT-5: No `reserve()` Pattern for Channel Sends — Data Loss on Timeout/Cancel

**Severity**: Medium (data loss)  
**Files**: `nt_core_event_bus.rs:167,215,525,532`, `nt_mind_background_loop/handlers.rs:40`, `nt_io_web/api.rs:469-494`, `nt_core_llm.rs:69,972-1084`  
**Pattern**: `tx.send(value).await` without using `reserve()` first.

**Analysis**: `mpsc::Sender::send` is documented as **NOT** cancel-safe. If the `send` future is dropped (e.g., by `select!` timeout), the value is lost. NeoTrix has ~100 `.send()` calls. While most are in non-select contexts today, several are in streaming response handlers (`nt_io_web/api.rs:1102-1161`) where SSE/WS connections can drop mid-stream, and in the event bus (`nt_core_event_bus.rs`) where shutdown can race with event emission.

**Concrete risk**: In `nt_io_web/api.rs:469`:
```rust
let _ = tx.send(serde_json::json!({"error": e}).to_string());
```
If the SSE client disconnects while this send is in-flight, the error message is silently lost.

**Fix**: Use `reserve()` to acquire a permit before consuming the value, then `permit.send(value)` which is atomic and doesn't need `.await`:
```rust
let permit = tx.reserve().await.map_err(|_| "channel closed")?;
// ... produce value ...
permit.send(value);  // no .await, cannot be cancelled
```

---

### DEFECT-6: `try_join!` Early Cancellation of Side-Effectful Futures

**Severity**: Medium (incomplete cleanup)  
**Files**: Potential risk in any code using `tokio::try_join!` or `tokio::try_join`  
**Pattern**: Using `try_join!` for operations with side effects (e.g., flushing multiple connections, committing multiple DB transactions).

**Analysis**: `tokio::try_join!` cancels all remaining futures as soon as one returns `Err`. If you're flushing two proxy connections and one fails, the other is silently cancelled — potentially leaving partial data on the wire. The `cancel-safe-futures` crate (by Oxide) was created specifically to address this, providing `join_then_try!` which runs all futures to completion.

**Risk in NeoTrix**: The proxy kernel (`nt_shield_proxy_kernel`) manages multiple concurrent connections. Any use of `try_join!` for parallel cleanup/flush operations will exhibit this behavior.

**Fix**: Replace `try_join!` with `join_then_try!` from `cancel-safe-futures`, or use `tokio::join!` and collect errors manually.

---

### DEFECT-7: Missing Cancel-Safety Documentation on Public Async APIs

**Severity**: Medium (maintainability hazard)  
**Files**: All public async functions across NT-SHIELD, NT-WORLD, NT-ACT  
**Pattern**: No `/// # Cancel Safety` doc comments on any public async function.

**Analysis**: The Tokio project and Oxide both recommend documenting cancel safety on every public async API. NeoTrix has zero cancel-safety documentation. This means future developers (or AI agents) cannot determine whether a given async function is safe to use in `select!` without reading the full implementation.

**Fix**: Add `/// # Cancel Safety` sections to all public async functions, marking them as `Cancel-safe` or `NOT cancel-safe` with justification.

---

## Summary Matrix

| # | Defect | Severity | Root Cause | Fix Difficulty |
|---|--------|----------|------------|----------------|
| 1 | `read_exact` in select-adjacent code | High | Using cancel-unsafe I/O primitive | Medium |
| 2 | `write_all` pervasive usage | High | Using cancel-unsafe I/O primitive | Medium |
| 3 | `tokio::sync::Mutex` across `.await` | Medium-High | Shared mutable state in async | High (actor refactor) |
| 4 | `select!` loop fairness (shutdown) | Medium | Missing `biased;` ordering | Low |
| 5 | No `reserve()` for channel sends | Medium | Using `send()` directly | Medium |
| 6 | `try_join!` early cancellation | Medium | Default join semantics | Low (swap macro) |
| 7 | No cancel-safety documentation | Medium | Missing API contract | Low (add docs) |

---

## Recommendations for NeoTrix

1. **Immediate (low-effort)**: Add `biased;` with shutdown branch first in all `select!` loops (Defect 4). Document cancel safety on public APIs (Defect 7).

2. **Short-term**: Audit all `read_exact`/`write_all` usage to ensure they're never inside `select!` or `timeout` boundaries. Add `reserve()` pattern to critical channel sends (Defect 5). Swap `try_join!` → `join_then_try!` where side effects matter (Defect 6).

3. **Medium-term**: Systematic migration from `tokio::sync::Mutex` to actor-model message passing in NT-SHIELD traffic layer (Defect 3). This is the highest-risk defect due to concurrent proxy connection handling.

4. **Long-term**: Consider adopting `cancel-safe-futures` crate as a dependency for cooperative cancellation and cancel-safe primitives.

---

## Sources

- [Tokio `select!` docs](https://docs.rs/tokio/latest/tokio/macro.select.html) — cancel safety reference
- [Oxide RFD 400](https://rfd.shared.oxide.computer/rfd/0400) — comprehensive cancel safety guide
- [cancel-safe-futures crate](https://docs.rs/cancel-safe-futures/latest/cancel_safe_futures/) — Oxide's solution library
- [Biriukov (2026)](https://biriukov.dev/posts/async-rust-gocha-tokio-cancelation-select-future-then/) — `FutureExt::then()` + `StreamExt::then()` gotcha
- [RustConf 2025 — Rain Paharia](https://www.youtube.com/watch?v=zrv5Cy1R7r4) — "Cancelling Async Rust" talk
- [LWN.net (2025)](https://lwn.net/Articles/1036924/) — summary of RustConf talk
- [Sunshowers (2025)](https://sunshowers.io/posts/cancelling-async-rust/) — Oxide production war stories
- [rust-skills/async-cancel-safety](https://github.com/leonardomso/rust-skills/blob/HEAD/rules/async-cancel-safety.md) — pattern reference
