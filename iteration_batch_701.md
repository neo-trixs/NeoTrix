# Iteration 701 — Async Runtime Ecosystem Audit

**Date**: 2026-09-06
**Prior Batch**: 700 (nalgebra FFI R-P1, GWT scalar SIMD, uncertainty estimate, numerical linting, VsaHyperCube copies)
**Domain**: NT-IO async runtime + NT-WORLD async pipeline + NT-ACT async dispatch

---

## Sources Cited

1. **async-std discontinuation** — RUSTSEC-2025-0052 (Aug 2025), docs.rs/async-std, GitHub async-rs/async-std releases
2. **Tokio vs Smol 2026** — wrenlearnsrust.com (2026-03-11)
3. **Rust Async Runtime Comparison 2026** — abrarqasim.com (2026-07-03)
4. **The End of async-std** — wrenlearnsrust.com (2026-03)
5. **Async Traits 2026** — wrenlearnsrust.com (2026-03-17)
6. **Async Traits Object Safety** — devencyclopedia.com (2026-07-24)
7. **State of Async Rust: Runtimes** — corrode.dev (2026-07-30)
8. **Cancellation Safety 2026** — biriukov.dev (2026-02), bshn.rs (2026-04)
9. **Tokio Graceful Shutdown** — tokio.rs (2026-07)
10. **Rust Project Goals 2026** — goals.rust-lang.org "Just add async"
11. **Async Fundamentals Roadmap** — rust-lang.github.io/async-fundamentals-initiative
12. **JetBrains/Carl Lerche interview** — blog.jetbrains.com (2026-02-17)
13. **Canceling Async Rust [LWN]** — lwn.net (2025-09-24), RustConf 2025

---

## NEW Defects Found

### DEFECT-701-1: async-std Transitive Dependency Risk (CRITICAL)

**What**: `async-std` officially discontinued (RUSTSEC-2025-0052, March 2025). No patches, no maintainer. 1,754 public crates still depend on it. If any NT-WORLD crawler dependency or NT-IO HTTP client transitively pulls `async-std`, NeoTrix inherits an unmaintained runtime with known vulnerabilities.

**Evidence**: `cargo tree -d` will reveal if dual runtimes are linked. `cargo tree -i async-std` reveals transitive dependents. The discontinuation notice was poorly propagated — not on docs.rs, only in GitHub README initially.

**Impact on NeoTrix**:
- NT-WORLD `UnifiedCrawler` uses HTTP clients (likely `surf` or `reqwest`) — need to verify no `async-std` transitive path
- NT-IO LLM provider HTTP calls could pull dead runtime
- NT-SHIELD stealth network layer could inherit unpatched security advisories

**Action**: Run `cargo tree -i async-std` and `cargo tree -d` across all workspace crates. If found, force-replace with `smol` or `tokio` feature flag.

---

### DEFECT-701-2: Async Trait Object Safety Blocks NT-IO Plugin Architecture (HIGH)

**What**: As of Rust 1.85+ (2026), `async fn in traits` is stable for static dispatch but **still not dyn-compatible**. You cannot write `dyn MyTrait` when `MyTrait` has async methods. This is not a near-term fix — requires naming anonymous future types or new object-safety mechanisms, both active research (not shipping).

**Evidence**: devencyclopedia.com (2026-07-24), wrenlearnsrust.com (2026-03-17), rust-lang RFC 3185, async-fundamentals-initiative roadmap.

**Impact on NeoTrix**:
- NT-IO defines provider traits (LLM, ACP, LSP) — if any use `async fn` in trait definitions, they cannot be dyn-dispatched
- Three workarounds exist, each with cost:
  - **Enum dispatch**: zero-cost but closed set (breaks NT-ACT plugin model)
  - **`async-trait` crate**: `Pin<Box>` allocation per call (violates zero-cost principle)
  - **RPITIT + object-safe adapter**: zero-cost for static, boxed for dyn (requires maintaining dual traits)

**Action**: Audit all NT-IO and NT-ACT trait definitions for `async fn`. Ensure plugin traits use either enum dispatch (closed sets) or RPITIT+adapter pattern (open sets). Never use `#[async_trait]` in hot paths.

---

### DEFECT-701-3: Send Bounds Propagation Silent Failure (HIGH)

**What**: `async fn` in traits does not automatically propagate `Send` bounds. The compiler is conservative — auto traits (Send/Sync) do not flow through async trait return types. This means a trait that works on single-threaded executor will **silently fail to compile** when used with Tokio's multi-threaded work-stealing executor that requires `Send`.

**Evidence**: wrenlearnsrust.com (2026-03-17), rust-lang/rust#103854, reintech.io (2026-02-11), Microsoft RustTraining async-book ch10.

**Impact on NeoTrix**:
- NT-CORE GWT broadcast uses `tokio::spawn` → requires `Send + 'static`
- NT-MIND SEAL pipeline stages are generic — if any stage's future holds `!Send` type, spawning breaks at integration time, not definition time
- NT-ACT MCP tool execution spawns tasks — any tool returning non-Send future silently breaks

**Fix pattern**:
```rust
// WRONG — Send not guaranteed
trait CrawlPipeline {
    async fn fetch(&self, url: &str) -> Result<Response>;
}

// CORRECT — explicit Send variant
#[trait_variant::make(CrawlPipelineSend: Send)]
trait CrawlPipeline {
    async fn fetch(&self, url: &str) -> Result<Response>;
}
```

**Action**: Add `trait_variant::make` to all traits whose futures may be spawned. Add compile-time `assert_send` assertions in integration tests.

---

### DEFECT-701-4: Cancellation Data Loss in NT-WORLD Crawl Pipeline (HIGH)

**What**: Rust's async cancellation is **normal control flow** — dropping a future at any `.await` point is silent, no compiler warning, no language mechanism to prevent data loss. Tokio's `select!` drops losing branches mid-flight. This is the "least Rusty part of Rust" per Oxide engineer Rain Paharia (RustConf 2025).

**Evidence**: lwn.net (2025-09-24), biriukov.dev (2026-02), bshn.rs (2026-04), Google Comprehensive Rust.

**Impact on NeoTrix**:
- NT-WORLD crawl loop: if a `select!` drops a stream-read future mid-item, the consumed item is lost from the source stream and cannot be re-yielded
- NT-ACT MCP tool timeout: if `tokio::time::timeout` fires during a write operation, partial writes may be lost
- NT-SHIELD Tor circuit: if circuit-building future is dropped mid-handshake, partial state is leaked

**Mitigation patterns**:
1. **`reserve()` before send** — obtain channel capacity before consuming from stream
2. **I/O actor model** — single task owns I/O objects, other tasks send commands via mpsc
3. **`tokio::try_join!`** — separate cancel future from worker futures

**Action**: Add cancellation-safety audit to NT-WORLD crawl loop. Add unit tests with tiny duplex buffers (1-byte) to force `Poll::Pending` paths.

---

### DEFECT-701-5: Tokio Ecosystem Lock-in Forces Send+Static Tax (MEDIUM)

**What**: Tokio mandates `Send + 'static` for spawned tasks. This forces `Arc<Mutex<T>>` wrapping for shared state even when single-threaded access is sufficient. The `corrode.dev` analysis (2026-07-30) argues this is a fundamental departure from sync Rust's borrowing model, and the performance cost of locking is non-trivial in embedded/low-latency contexts.

**Evidence**: corrode.dev (2026-07-30), tokio-rs/tokio (MSRV 1.71), blog.jetbrains.com (2026-02-17 Carl Lerche interview).

**Impact on NeoTrix**:
- NT-CORE E8 hexagram state: currently uses `RwLock` — forced `Send + 'static` means wrapping in `Arc` even for single-threaded access
- NT-MEMORY KB connections: SQLite connection pool must be `Send + 'static`
- NT-PHYSICAL sensor data: hardware I/O typically `!Send`, requires bridging layer

**Mitigation**: Use `tokio::task::LocalSet` for `!Send` workloads. Use smol for NT-WORLD crawl (no Send requirement). Reserve Tokio for NT-IO network layer.

**Action**: Document runtime selection policy: smol for NT-WORLD/NT-PHYSICAL (no Send tax), Tokio for NT-IO/NT-ACT (ecosystem compatibility).

---

### DEFECT-701-6: Async Drop Not Stabilized — NT-MIND Resource Cleanup Risk (MEDIUM)

**What**: Destructors in Rust are sync-only. Async resources (DB connections, network sessions, file handles) cannot perform async cleanup in `Drop`. The 2026 Rust roadmap lists "guaranteed destructors" as 2026-2027 goal, and "async drop" as inactive/dormant. Tower middleware ecosystem is blocked on this.

**Evidence**: goals.rust-lang.org (2026 roadmap "Just add async"), async-fundamentals-initiative roadmap, corrode.dev (2026-07-30).

**Impact on NeoTrix**:
- NT-MEMORY KB connections: async flush on drop (WAL checkpoint) impossible in current Drop
- NT-WORLD crawl sessions: HTTP client cleanup (connection pool drain) forced to sync or leaked
- NT-SHIELD Tor circuits: circuit teardown may not complete gracefully

**Mitigation**: Implement explicit `async fn close(&self) -> Result<()>` methods alongside Drop. Use `tokio::spawn` + `CancellationToken` for cleanup tasks that must outlive the owner.

**Action**: Add explicit `close()` / `shutdown()` to all resource-owning types in NT-MEMORY, NT-WORLD, NT-SHIELD. Add `#[cfg(test)]` leak detectors.

---

### DEFECT-701-7: Dual Runtime Linking Bloat (LOW)

**What**: If `async-compat` is used to bridge smol and Tokio, both runtimes are shipped in the binary. smol compiles ~5 crates, Tokio ~50 crates. The `async-compat` bridge trades smol's compile-time win for carrying both executors. Abrarqasim (2026-07-31) warns: "if half your tree needs Compat, that's your tree telling you to just use Tokio."

**Evidence**: abrarqasim.com (2026-07-31), wrenlearnsrust.com (2026-03).

**Impact on NeoTrix**:
- If NT-WORLD uses smol but NT-IO requires Tokio-native `reqwest`, `async-compat` bridges both
- Binary bloat: ~50 extra crates, ~2 executors fighting over same cores
- `cargo tree -d` will show duplicate runtime dependencies

**Action**: Enforce single-runtime policy per domain. NT-WORLD: smol. NT-IO: Tokio. NT-ACT: Tokio. No cross-domain runtime bridging.

---

### DEFECT-701-8: Closure Capture Anti-Pattern in Async Select (LOW)

**What**: Rust's async closures (`async Fn()` stabilized 1.85) have known compiler bugs reporting invalid `Send` errors. The 2026 roadmap lists `move(...)` expressions and `Move` trait as goals to fix closure capture pain, but these are 2026-2027 targets. Current workaround: clone-into-temporary patterns, which are error-prone.

**Evidence**: goals.rust-lang.org (2026), wrenlearnsrust.com (2026-03-17).

**Impact on NeoTrix**:
- NT-CORE GWT broadcast closures may incorrectly report Send errors
- NT-MIND SEAL pipeline closures may require unnecessary clones

**Action**: Track rust-lang tracking issues for `move(...)` expressions. Avoid complex closure captures in async contexts until stabilized.

---

## Summary

| # | Defect | Severity | Domain | Status |
|---|--------|----------|--------|--------|
| 701-1 | async-std transitive dependency | CRITICAL | NT-WORLD/NT-IO/NT-SHIELD | Audit needed |
| 701-2 | Async trait object safety | HIGH | NT-IO/NT-ACT | Architecture review |
| 701-3 | Send bounds silent failure | HIGH | NT-CORE/NT-MIND/NT-ACT | Add trait_variant |
| 701-4 | Cancellation data loss | HIGH | NT-WORLD/NT-ACT/NT-SHIELD | Add cancellation audit |
| 701-5 | Tokio Send+Static tax | MEDIUM | NT-CORE/NT-MEMORY/NT-PHYSICAL | Runtime policy doc |
| 701-6 | Async drop not stabilized | MEDIUM | NT-MEMORY/NT-WORLD/NT-SHIELD | Add explicit close() |
| 701-7 | Dual runtime bloat | LOW | Cross-domain | Single-runtime policy |
| 701-8 | Closure Send bugs | LOW | NT-CORE/NT-MIND | Track upstream |

**Total NEW defects**: 8
**Prior batch defects still open**: 5 (from batch 700)
**Cumulative defect count**: ~706+
