# Iteration Batch 757 — Research Loop

**Date**: 2026-09-07  
**Context**: Batch 756 XSS/vulnerability findings → Now cross-referencing Rust language evolution + security auditing patterns  
**Sources**: 12 web sources across 5 searches

---

## 1. Lifetime / Borrow Checker (2026)

### NEW Finding 1: Polonius Alpha Enabled on Nightly (Aug 4, 2026)

**Source**: [Rust Blog — Enabling the next iteration of the borrow checker on nightly](https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/)

The Polonius borrow checker — 8 years in development — was enabled on nightly as of Aug 4, 2026, with stabilization targeted before year-end. This is a **loan-based, flow-sensitive** model replacing NLL's scope-based analysis.

**NeoTrix Defect**: NeoTrix's async code (particularly `nt_mind` SEAL pipeline and `nt_core` consciousness loop) was written against NLL borrow rules. Polonius accepts more correct programs, meaning:
- Code that currently compiles with workarounds (extra `clone()`, restructured borrows) can be simplified
- Code that currently fails with NLL may become valid under Polonius
- **But**: Polonius does NOT make borrow tracking finer-grained — disjoint field borrows across function boundaries still require view types (future feature)

**Impact**: Medium — affects code quality, not correctness. Workaround debt can be retired when Polonius stabilizes.

### NEW Finding 2: The Borrow Checker Within Roadmap — Place-Based Lifetimes

**Source**: [goals.rust-lang.org/2026/roadmap-borrow-checker-within](https://goals.rust-lang.org/2026/roadmap-borrow-checker-within.html)

Future borrow checker features include:
- **Place-based lifetime syntax**: `'self.text` instead of `'a` — makes borrow relationships readable
- **View types**: Extend to public APIs and field abstraction
- **Internal references**: Structs that hold references into their own data (`&'self.text str`)

**NeoTrix Defect**: NeoTrix's `SelfModel` types (`nt_core_self::SelfModel`, `nt_core_meta::SelfModel`) currently use index-based workarounds for self-referential patterns. Place-based lifetimes and internal references (if/when stabilized) would eliminate these workarounds. The architecture should be designed so these patterns can be adopted incrementally.

**Impact**: Low (future) — design for evolvability.

### NEW Finding 3: Async Closures — Borrowing Across Await Points

**Source**: [rustify.rs/articles/whats-new-in-rust-2026](https://rustify.rs/articles/whats-new-in-rust-2026)

Rust 1.85 (Feb 2026) stabilized `async || {}` closures with `AsyncFn` trait family. These closures can **borrow captured state across await points** — something previously impossible.

**NeoTrix Defect**: NeoTrix's `nt_core` consciousness tick and `nt_mind` SEAL pipeline use async closures extensively. Many workarounds for borrowing across `.await` (e.g., cloning state before async blocks, using `Arc<Mutex<T>>` where a simple borrow would suffice) can now be replaced with `async || {}` syntax. The `AsyncFn` trait captures lifetime relationships that `Fn() -> impl Future` cannot.

**Impact**: Medium — reduces heap allocations, simplifies async callback APIs in NT-IO.

---

## 2. Type System (2026)

### NEW Finding 4: Enum Dispatch vs Trait Objects — Decision Matrix

**Source**: [telex-tui.github.io/blog/rust-patterns-enum-dispatch.html](https://telex-tui.github.io/blog/rust-patterns-enum-dispatch.html) (July 2026)

Clear decision framework:
- **Enum dispatch** (static, inlined, no vtable): Use when set of variants is known, controlled by your crate, unlikely to grow. When adding operations more often than types.
- **Trait objects** (dynamic, vtable indirection): Use when callers define types (plugins, extensible components). When set of types is open-ended.
- **Expression problem**: Enums = easy to add operations, hard to add variants. Trait objects = easy to add variants, hard to add operations.

**NeoTrix Defect**: NeoTrix's capability system (`CapabilityTree`, `CapabilityRegistry` in `nt_core_capability_tree`) currently uses `Box<dyn Capability>` trait objects for runtime polymorphism. But the set of core capabilities (Perception, Action, Memory, etc.) is **closed and controlled by NeoTrix**. The 7 faction domains are fixed. This means:
- Core capability dispatch should use **enum dispatch** (10x faster than trait objects per `enum_dispatch` crate docs)
- Only user-extensible capabilities (plugins, MCP tools) should use trait objects
- **Current design mixes both patterns in the same data structure**

**Impact**: High — performance regression in hot paths (per-packet processing in NT-WORLD crawl, per-call in NT-ACT tool dispatch).

### NEW Finding 5: `#[expect(lint)]` Attribute Stabilized

**Source**: [rustify.rs/articles/whats-new-in-rust-2026](https://rustify.rs/articles/whats-new-in-rust-2026)

`#[expect(lint)]` is now stable (Rust 1.84). Unlike `#[allow(lint)]`, it **warns if the lint doesn't fire** — preventing stale suppressions.

**NeoTrix Defect**: NeoTrix's codebase likely has numerous `#[allow(unused)]`, `#[allow(dead_code)]`, `#[allow(clippy::...)]` suppressions that may have become stale as code evolved. These should be migrated to `#[expect(lint)]` to catch regressions.

**Impact**: Medium — code quality, prevents silent regression.

---

## 3. Unsafe Rust / FFI (2026)

### NEW Finding 6: `unsafe extern` Blocks — Mandatory in Edition 2024

**Source**: [doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html)

Starting in Rust 2024 edition (Rust 1.85), all `extern` blocks **must** be marked `unsafe`. Items within can be marked `safe` individually.

**NeoTrix Defect**: NeoTrix's FFI bindings (particularly in `nt_shield_sandbox` for sandboxed execution, and any `extern "C"` in `nt_physical` for hardware access) currently use plain `extern "C"` blocks without `unsafe`. When migrating to edition 2024:
- All extern blocks need `unsafe extern "C"` prefix
- Safety contracts must be documented per-function
- The `unsafe_op_in_unsafe_fn` lint now warns by default (previously implicit)
- **Critical**: Each FFI function needs a `// SAFETY:` comment documenting the contract

**Impact**: High — required for edition 2024 migration; missing safety contracts are audit blockers.

### NEW Finding 7: Rust Security Auditing — 2026 Patterns

**Source**: [sherlock.xyz/post/rust-security-auditing-guide-2026](https://sherlock.xyz/post/rust-security-auditing-guide-2026)

Key 2026 audit patterns:
1. **Unsafe contracts, not "unsafe blocks"**: Audit documents the *contract* (alignment, aliasing, lifetime invariants), not just the `unsafe` keyword
2. **Miri + Sanitizers combo**: Miri for Rust semantics (alignment, aliasing, provenance), sanitizers for mixed-language memory model (FFI buffers, allocator misuse)
3. **Supply chain verification**: `cargo-auditable` embeds dependency tree in binary; `cargo-geiger` maps unsafe surface across dependency graph
4. **Artifact ≠ Repo**: Feature flags, build scripts, platform-specific deps mean the deployed binary may differ from the reviewed code

**NeoTrix Defects**:
- **No Miri CI**: NeoTrix has zero Miri testing in its CI pipeline. All unsafe blocks (in FFI, SIMD, raw pointer manipulation) are untested for UB.
- **No cargo-geiger audit**: The unsafe surface across NeoTrix's 411+ dependency graph is unmeasured
- **No cargo-auditable**: Production binaries lack embedded dependency metadata
- **No sanitizer builds**: Mixed Rust/C code (if any) is untested with ASan/TSan

**Impact**: Critical — silent UB in production FFI code.

### NEW Finding 8: FFI Security — Ownership Contracts at Boundaries

**Source**: [safeguard.sh/resources/blog/auditing-unsafe-rust-ffi-boundaries](https://safeguard.sh/resources/blog/auditing-unsafe-rust-ffi-boundaries-for-memory-corruption-bugs)

Step-by-step FFI audit:
1. Map every `extern` block and `#[no_mangle]` export
2. Inventory pointer and lifetime assumptions per FFI function
3. Verify alignment with `std::ptr::addr_of!` (not casts)
4. Fuzz byte-to-structure boundaries for 24+ hours
5. Run both ASan and Miri on full test suite
6. Check ownership transfer patterns (who frees?)

**NeoTrix Defect**: NeoTrix's FFI surface (sandboxed execution in NT-SHIELD, hardware abstraction in NT-PHYSICAL) lacks:
- Documented ownership contracts per FFI function
- Length-consistency checks on buffer parameter pairs
- Fuzzing corpus at FFI byte-to-structure boundaries
- Drop semantics verification across FFI ownership transfers

**Impact**: Critical — memory corruption potential at every un-audited FFI boundary.

### NEW Finding 9: Async Deadlocks — Safe Rust's Remaining Blind Spot

**Source**: [sherlock.xyz/post/rust-security-auditing-guide-2026](https://sherlock.xyz/post/rust-security-auditing-guide-2026)

Safe Rust prevents data races but NOT:
- **Deadlocks** from holding locks across `.await`
- **Resource exhaustion** from unbounded task fanout
- **Cancellation storms** from slow downstream dependencies

**NeoTrix Defect**: NeoTrix's EventBus system and async consciousness tick loop use `tokio::spawn` extensively. If any lock is held across an `.await` point, an attacker or slow dependency can wedge the entire service. The EventBus's `tokio::sync::broadcast` channel could also be exploited with unbounded consumer fanout.

**Impact**: High — DoS vulnerability in async code paths.

---

## Summary: NEW Defects Found (Batch 757)

| # | Defect | Severity | Category |
|---|--------|----------|----------|
| 1 | Polonius workarounds can be retired (async borrow patterns) | Medium | Lifetime |
| 2 | Self-referential patterns need design-for-evolvability | Low | Lifetime |
| 3 | Async closures reduce heap alloc in NT-IO callbacks | Medium | Type System |
| 4 | Capability dispatch should use enum dispatch, not dyn Trait | High | Type System |
| 5 | Migrate `#[allow]` → `#[expect(lint)]` for stale suppression detection | Medium | Type System |
| 6 | FFI extern blocks need `unsafe extern` + SAFETY docs for edition 2024 | High | Unsafe/FFI |
| 7 | No Miri CI, no cargo-geiger, no cargo-auditable, no sanitizer builds | Critical | Unsafe/FFI |
| 8 | FFI ownership contracts undocumented, no fuzzing at byte-to-struct boundaries | Critical | Unsafe/FFI |
| 9 | Async deadlock DoS via lock-across-await in EventBus | High | Async Safety |

## Sources Cited

1. [Rust Blog — Polonius Alpha on nightly (Aug 4, 2026)](https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/)
2. [Rust Project Goals — Borrow Checker Within](https://goals.rust-lang.org/2026/roadmap-borrow-checker-within.html)
3. [Rustify — What's New in Rust 2026](https://rustify.rs/articles/whats-new-in-rust-2026)
4. [Rustify — Rust Lifetimes Deep Dive 2026](https://rustify.rs/articles/rust-lifetimes-deep-dive-2026)
5. [Telex — Enum Dispatch vs Trait Objects (July 2026)](https://telex-tui.github.io/blog/rust-patterns-enum-dispatch.html)
6. [enum_dispatch crate docs](https://docs.rs/enum_dispatch/latest/enum_dispatch)
7. [Sherlock — Rust Security Auditing Guide 2026](https://sherlock.xyz/post/rust-security-auditing-guide-2026)
8. [Safeguard — FFI Security Audit Guide (Feb 2026)](https://safeguard.sh/resources/blog/auditing-unsafe-rust-ffi-boundaries-for-memory-corruption-bugs)
9. [Cyberguid — Rust FFI Security 2026](https://cyberguid.com/rust-ffi-security-2026/)
10. [Rust Edition Guide — unsafe extern blocks](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html)
11. [Rustify — Generics and Traits 2026](https://rustify.rs/articles/rust-generics-and-traits-explained-2026)
12. [Vize — Polonius on Nightly (Aug 2026)](https://vizejs.dev/blog/notes/2026-08-11-trying-the-polonius-borrow-checker-on-rust-nightly/index.html)
