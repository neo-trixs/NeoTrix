# Iteration Batch 760 — Rust Ecosystem Research

**Date**: 2026-09-07
**Predecessor**: Batch 759 (config-file trust escalation, git subprocess bypass, sandbox canonicalization, dual-agent self-review blind spot, retrieval poisoning)

---

## 1. Rust Compiler (rustc) — NEW FINDINGS

### 1a. RFC 3943: MIR Move Elimination (CRITICAL)
- **Source**: https://github.com/rust-lang/rfcs/pull/3943 (Amanieu, 2026-04-03)
- **Status**: Open RFC, implementation in PR #157943 (4431 lines, 64 files)
- **What's NEW**: Changes MIR operational semantics so that accessing memory after a move is UB. Local variable allocation lifetime tied to initialized state (not lexical scope). Three-state model: dead → live → allocated.
- **PreciseLiveness**: Sub-statement granularity liveness analysis, replaces `MaybeStorageLive`. More fine-grained than previous liveness tracking.
- **MoveElimination pass**: Supersedes `DestinationPropagation`. Unifies move-connected locals with disjoint live ranges, eliminating copies. Prototype shows 2-5% binary speedup on debug builds.
- **NEOTRIX DEFECT IDENTIFIED**: NeoTrix's `#![forbid(unsafe_code)]` (R-P1) guards against unsafe, but this RFC introduces *new UB semantics for safe code* — accessing moved locals in MIR becomes UB even without `unsafe`. Any NeoTrix code relying on the old "moved locals are simply bitwise copies" mental model could break. **Action**: Audit `neotrix-core` for patterns where moved locals are used after move (currently valid, will become UB).

### 1b. Miri Support for Move Elimination (CRITICAL)
- **Source**: https://github.com/rust-lang/rust/pull/162048 (Amanieu, 2026-08-31, DRAFT)
- **What's NEW**: Adds `LocalValue::LiveUnallocated` state to Miri. `eval_place_for_write` allocates on first write. `move_out_local` copies to temp then frees original. ZSTs always allocated at `StorageLive`.
- **NEOTRIX DEFECT**: NeoTrix uses Miri in CI for unsafe detection (implied by `#![forbid(unsafe_code)]` and tree Borrows). With new semantics, Miri's `-Zmir-move-elimination` flag will detect *new categories of UB* in existing code. NeoTrix CI should enable this flag proactively to catch issues before they land in stable rustc.

### 1c. Post-Mono MIR Optimizations
- **Source**: https://github.com/rust-lang/rust/pull/156858 (cjgillot, 2026-05-23)
- **What's NEW**: New `build_codegen_mir` query with `Steal<Cow<Body>>` return type. Enables monomorphization-time MIR transforms. Removes JIT monomorphization from codegen.
- **Impact on NeoTrix**: 0.8% instruction count regression, 12.1% max RSS regression (up to 53.9% on some benchmarks). NeoTrix build times may increase. Monitor `cargo build -p neotrix` for regressions.

### 1d. Rust 1.98.0 (2026-08-20) Stable
- **Source**: https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/
- **Key changes**:
  - `&mut` lifetime shortening in invariant positions (e.g., `Cell<&'long mut i32>` → `Cell<&'short mut dyn Send>`)
  - `invalid_runtime_symbol_definitions` (deny-by-default) — targets `core` runtime symbols like `memcmp`, `memset`, `strlen`
  - `suspicious_runtime_symbol_definitions` (warn-by-default)
  - `c_void_returns` lint
  - Algebraic floating-point methods (`algebraic_add`, etc.)
- **NEOTRIX DEFECT**: `invalid_runtime_symbol_definitions` is deny-by-default. If NeoTrix overrides or re-exports `memcmp`/`memset`/`strlen` symbols (e.g., in NT-SHIELD sandbox or NT-ACT FFI), compilation will fail. Audit for symbol re-definitions.

---

## 2. MIRI — NEW FINDINGS

### 2a. Miri POPL 2026 Paper (Keynote)
- **Source**: https://doi.org/10.1145/3776690 (Jung et al., POPL 2026)
- **What's NEW**: Formal publication of Miri's architecture. 100K+ Rust libraries evaluated, 70%+ test execution rate. Key claims: Miri finds *all de-facto UB in deterministic Rust programs*.
- **NEOTRIX RELEVANCE**: Miri's formal model (provenance tracking, type invariant validation, data-race detection) is now peer-reviewed. NeoTrix's `#![forbid(unsafe_code)]` is good but not sufficient — Miri can find UB in safe code patterns involving uninitialized memory, invalid enum discriminants, and pointer provenance violations.

### 2b. GenMC Integration (In Progress)
- **Source**: ralfj.de/blog/2025/12/22/miri.html
- **What's NEW**: Work underway to integrate GenMC (weak memory model checker) into Miri. This enables exhaustive exploration of concurrent program behaviors, not just one interleaving.
- **NEOTRIX DEFECT**: NeoTrix's `nt_shield` (stealth net, proxy pool) and `nt_physical` (sensors, motors) likely use concurrency. Current Miri only explores one thread interleaving. GenMC integration will enable exhaustive concurrent UB detection. NeoTrix should run `MIRIFLAGS="-Zmiri-many-seeds"` in CI as interim measure.

### 2c. `-Zmiri-deterministic-concurrency` Flag
- **Source**: Miri README
- **What's NEW**: New flag makes concurrency behavior fully deterministic by disabling RNG for scheduling. Equivalent to `-Zmiri-fixed-schedule -Zmiri-compare-exchange-weak-failure-rate=0.0 -Zmiri-address-reuse-cross-thread-rate=0.0 -Zmiri-disable-weak-memory-emulation`.
- **NEOTRIX DEFECT**: NeoTrix's CI may use non-deterministic Miri runs. Use `-Zmiri-deterministic-concurrency` for reproducible results, or use `-Zmiri-seed=N` with different seeds for each CI job to maximize coverage.

---

## 3. Clippy — NEW FINDINGS

### 3a. `nonnull_unchecked_on_box_ptr` (Merged, Rust 1.98+)
- **Source**: https://github.com/rust-lang/rust-clippy/pull/17336 (ArhanChaudhary, 2026-07-24)
- **What's NEW**: Flags `unsafe { NonNull::new_unchecked(Box::into_raw(x)) }` and suggests `NonNull::from_mut(Box::leak(x))`.
- **NEOTRIX DEFECT**: If NeoTrix uses `NonNull::new_unchecked(Box::into_raw(...))` pattern (common in custom allocators or FFI), this lint will fire. The lint is warn-by-default. NeoTrix should audit for this pattern and migrate to `NonNull::from_mut`.

### 3b. `option_zip_none` (Open)
- **Source**: https://github.com/rust-lang/rust-clippy/pull/17465 (Amit5601, 2026-07-27)
- **What's NEW**: Flags `Option::zip(None)` which always yields `None` — almost certainly a logic bug.
- **NEOTRIX DEFECT**: Logic bug detection. If NeoTrix has `some_option.zip(None)` anywhere, it's dead code or a bug. Run Clippy with this lint enabled.

### 3c. `definition_in_module_root` (Merged, Rust 1.98+)
- **Source**: https://github.com/rust-lang/rust-clippy/pull/16965 (corygabrielsen, 2026-07-11)
- **What's NEW**: Restriction lint flagging item definitions in `mod.rs` files. Enforces `mod.rs` for declarations/re-exports only, definitions in named files. 7556 hits across Clippy ecosystem.
- **NEOTRIX DEFECT**: NeoTrix has massive module hierarchy (`neotrix-core/src/`). If any `mod.rs` files contain function/struct definitions (not just `mod` declarations), this lint fires. **Action**: Run `cargo clippy -- -W clippy::definition_in_module_root` on neotrix-core. This enforces canonical file layout.

### 3d. `inline_trait_bounds` (Merged, Rust 1.98+)
- **Source**: https://github.com/rust-lang/rust-clippy/pull/16486 (moses7054, 2026-05-12)
- **What's NEW**: Restriction lint enforcing `where` clauses instead of inline trait bounds on functions. 2430 hits across ecosystem.
- **NEOTRIX DEFECT**: NeoTrix code style may use inline bounds (`fn foo<T: Clone>(x: T)`). This lint prefers `fn foo<T>(x: T) where T: Clone`. Apply if desired for consistency.

### 3e. `manual_slice_match` (Open)
- **Source**: https://github.com/rust-lang/rust-clippy/pull/17232 (sylvestre, 2026-06-13)
- **What's NEW**: Pedantic lint detecting `if/else if` chains on `.is_empty()` / `.len()` and suggesting slice pattern matching.
- **NEOTRIX RELEVANCE**: NeoTrix processes many byte slices (KB embeddings, VSA vectors). Check for length-check chains that could be slice patterns.

### 3f. `unused_import_prefixes` (Open, nursery)
- **Source**: https://github.com/rust-lang/rust-clippy/pull/17246 (hafihaf123, 2026-06-14)
- **What's NEW**: Flags redundant `crate::...` prefixes in `use` statements when importing from current module's descendants.
- **NEOTRIX RELEVANCE**: NeoTrix uses deep `crate::` paths extensively. Run this lint to clean up imports.

---

## 4. CRITICAL DEFECTS SUMMARY

| # | Severity | Component | Defect | Batch |
|---|----------|-----------|--------|-------|
| 1 | **CRITICAL** | neotrix-core | RFC 3943 new UB semantics — moved locals become UB if accessed. Audit all `move` patterns. | 760 |
| 2 | **CRITICAL** | nt_shield/nt_physical | Concurrency UB detection gap — Miri only explores one interleaving. Use `-Zmiri-many-seeds` or wait for GenMC. | 760 |
| 3 | **HIGH** | neotrix-core | `invalid_runtime_symbol_definitions` (deny-by-default in 1.98) — audit symbol re-exports. | 760 |
| 4 | **HIGH** | neotrix-core | `nonnull_unchecked_on_box_ptr` — audit `NonNull::new_unchecked(Box::into_raw(...))` patterns. | 760 |
| 5 | **MEDIUM** | neotrix-core | `definition_in_module_root` — enforce mod.rs as declarations-only across 7556 hit ecosystem. | 760 |
| 6 | **MEDIUM** | neotrix-core | Post-mono MIR optimization 12% RSS regression — monitor build times. | 760 |
| 7 | **LOW** | neotrix-core | `option_zip_none` — dead code/logic bug detection for `Option::zip(None)`. | 760 |

---

## 5. SOURCES CITED

| # | Source | URL |
|---|--------|-----|
| 1 | RFC 3943: MIR Move Elimination | https://github.com/rust-lang/rfcs/pull/3943 |
| 2 | PR #157943: Implement RFC 3943 | https://github.com/rust-lang/rust/pull/157943 |
| 3 | PR #162048: Miri support for move elimination | https://github.com/rust-lang/rust/pull/162048 |
| 4 | PR #156858: Post-mono MIR optimizations | https://github.com/rust-lang/rust/pull/156858 |
| 5 | Rust 1.98.0 Announcement | https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/ |
| 6 | Miri POPL 2026 Paper | https://doi.org/10.1145/3776690 |
| 7 | Miri blog (GenMC, deterministic concurrency) | https://www.ralfj.de/blog/2025/12/22/miri.html |
| 8 | PR #17336: nonnull_unchecked_on_box_ptr | https://github.com/rust-lang/rust-clippy/pull/17336 |
| 9 | PR #17465: option_zip_none | https://github.com/rust-lang/rust-clippy/pull/17465 |
| 10 | PR #16965: definition_in_module_root | https://github.com/rust-lang/rust-clippy/pull/16965 |
| 11 | PR #16486: inline_trait_bounds | https://github.com/rust-lang/rust-clippy/pull/16486 |
| 12 | PR #17232: manual_slice_match | https://github.com/rust-lang/rust-clippy/pull/17232 |
| 13 | PR #17246: unused_import_prefixes | https://github.com/rust-lang/rust-clippy/pull/17246 |
| 14 | Rust Project Goals: MIR Move Elimination | https://goals.rust-lang.org/2026/mir-move-elimination.html |
| 15 | MIR Transformation DeepWiki | https://deepwiki.com/rust-lang/rust/2.7-mir-transformation-and-optimization |
