# Unsafe Code Audit

**Date**: 2026-06-16
**Scope**: `neotrix-core` crate (`/Users/neo/Downloads/neotrix/neotrix-core/src/`)
**Total source files**: 1,321 `.rs` files
**Total lines of code**: ~423,955

---

## Forbid Policy

| Location | `#![forbid(unsafe_code)]` |
|---|---|
| `lib.rs:18` | ✅ **YES** — crate root |
| `core/mod.rs` | ❌ No (inherits from crate root) |
| `neotrix/mod.rs` | ❌ No (inherits from crate root) |
| Individual module files | **60 files** with per-file `#![forbid(unsafe_code)]` |
| Binary entry points | `ne_dialog.rs`, `ne_bootstrap_proof.rs`, `stage0_seed.rs` — all ✅ |

The crate root `lib.rs:18` applies `#![forbid(unsafe_code)]` globally, making any `unsafe { }` block, `unsafe fn`, or `unsafe impl` a **compile error** across the entire crate. The additional 60 per-file declarations are redundant but harmless (defense-in-depth).

Additionally, `lib.rs:19` has `#![deny(unsafe_op_in_unsafe_fn)]`, which would catch any `unsafe fn` whose body performs unsafe operations without an explicit `unsafe` block — though this is moot since no `unsafe fn` exists.

---

## Summary

| Category | Count |
|---|---|
| Actual `unsafe { }` blocks (production code) | **0** |
| Actual `unsafe fn` declarations | **0** |
| Actual `unsafe trait` / `unsafe impl` | **0** |
| `#[allow(unsafe_code)]` annotations | **0** |
| `#![forbid(unsafe_code)]` files | **60** (+ 3 binaries) |
| `unsafe` keyword in strings/comments only | 3 (all test fixtures) |

---

## Detailed Occurrences

### Pseudo-Occurrences (not actual unsafe code)

These match the text pattern `unsafe {` but are inside **string literals** used as test input to code-analysis functions. They are never compiled as Rust code — they are data fed to parsers/scanners.

#### 1. `cli/shield_enforcer.rs:451`
```rust
let content = r#"
    unsafe { transmute(x) }
"#;
```
- **Category**: String literal (test fixture)
- **Risk**: None
- **Justification**: This is input to `ShieldEnforcer::check_laws()` — it tests that the enforcer **detects** unsafe code in third-party input. The string is never compiled as Rust.
- **Elimination**: N/A — not actual unsafe code.

#### 2. `neotrix/nt_act_code/code_review_pipeline.rs:1366`
```rust
"+    unsafe {
+        let ptr = std::ptr::null();
+    }
"
```
- **Category**: String literal (test fixture for diff parser)
- **Risk**: None
- **Justification**: Part of a test diff string to verify the `DiffParser` correctly parses a hunk containing unsafe code. The string represents what a git diff might look like.
- **Elimination**: N/A — not actual unsafe code.

#### 3. `neotrix/nt_act_code/code_review_pipeline.rs:1584`
```rust
let diff_str = "diff --git a/src/lib.rs b/src/lib.rs
@@ -1,3 +1,5 @@
 fn compute() -> i32 {
+    let x = get_value().unwrap();
+    unsafe { std::ptr::read(0 as *const i32) };
     42
 }
";
```
- **Category**: String literal (test fixture for pipeline review)
- **Risk**: None
- **Justification**: Tests the `CodeReviewPipeline`'s ability to flag unsafe code in a code diff. The string is parsed as a diff, then fed to the reviewer — never executed.
- **Elimination**: N/A — not actual unsafe code.

### Metadata / Analysis References (not unsafe code)

These use `unsafe` as a **noun** (count/measure/property), not as a Rust keyword:

| File | Usage |
|---|---|
| `nt_core_meta/scanner.rs:91-104` | `unsafe_count += content.matches("unsafe").count()` — counts occurrences in scanned files |
| `nt_core_meta/self_model.rs:86` | `pub unsafe_count: usize` — struct field tracking unsafe usage in modules |
| `nt_core_meta/self_model.rs:99` | `pub has_unsafe: bool` — struct field |
| `nt_core_meta/weakness.rs:10` | `pub unsafe_threshold: usize` — config threshold for flagging high-unsafe modules |
| `nt_core_meta/monitor.rs:55-61` | `module.unsafe_count > 5` → alert — monitors for excessive unsafe |
| `nt_act_code/code_writer.rs:357` | `if line.contains("unsafe {")` — text analysis of generated code |
| `nt_act_code/verifier_stage.rs:69-72` | `if content.contains("unsafe")` → warning — safety verifier flags unsafe |
| `nt_act_code/code_review_pipeline.rs:1155` | `"unsafe "` — review rule keyword list |
| `nt_act_code/agentic_reasoning.rs:198` | `has_unsafe = lines.iter().any(\|l\| l.contains("unsafe"))` — code analysis |
| `nt_mind_evolution_loop.rs:127` | `if t.contains("unsafe {")` — evolution loop safe-code filter |
| `cli/laws.rs:70-75` | `L002: Forbid unsafe code blocks` — coding law enforcement |
| `nt_core_experience/code_quality.rs:145` | `code.contains("unsafe {")` → quality metric |

---

## Overall Assessment

### Unsafe code percentage: **0.000%**

The neotrix-core crate is **entirely safe Rust**. There is not a single `unsafe` keyword used as an actual Rust operation anywhere in 423,955 lines of code across 1,321 source files.

### Risk Level: **Lowest possible (none)**

### Why this matters

| Property | Status |
|---|---|
| No raw pointer dereference | ✅ |
| No FFI calls | ✅ |
| No mutable statics | ✅ |
| No union field access | ✅ |
| No inline assembly | ✅ |
| No `#[allow(unsafe_code)]` escaping the forbid | ✅ |
| No `unsafe fn` exported to other crates | ✅ |
| `#![forbid(unsafe_code)]` at crate root | ✅ |
| `#![deny(unsafe_op_in_unsafe_fn)]` | ✅ |
| `#[forbid(unsafe_code)]` pervasively on individual files | ✅ (60 files) |

### Architectural reason

NeoTrix's architecture mandates that all subsystems operate on VSA 4096-bit vectors through pure mathematical operations. There is no need for:
- **FFI**: No C/OS library bindings. All processing is pure Rust.
- **Raw pointers**: All data access uses safe Rust references, `Arc`, `Mutex`, `RwLock`.
- **Mutable statics**: Global state is managed through `OnceLock`, `LazyLock`, or explicit dependency injection.
- **Inline asm**: All compute (VSA operations, Walsh-Hadamard transforms, FWHT) is done via safe Rust loops.

The `unsafe` keyword only appears in this codebase as:
1. A **detection target** — the codebase scans its own (`scanner.rs`) and third-party code (`verifier_stage.rs`, `code_review_pipeline.rs`) for the presence of `unsafe` blocks.
2. A **policy boundary** — `L002` in the coding laws explicitly forbids `unsafe` blocks.
3. A **test fixture** — string literals simulating the presence of unsafe code to verify the analysis pipeline works correctly.

### Recommendations

1. **Maintain status quo** — Keep `#![forbid(unsafe_code)]` at crate root. It is the strongest enforcement possible and has zero cost.
2. **Remove redundant per-file declarations** — The 60 individual `#![forbid(unsafe_code)]` annotations are technically redundant given the crate root forbid. They add noise. Consider removing them for cleaner code, or keep them as defense-in-depth.
3. **Extend forbid policy to remaining workspace crates** — If any other crate in the workspace (e.g., external tools) does not have `#![forbid(unsafe_code)]`, consider adding it for consistent enforcement.
4. **Preserve meta-analysis tooling** — The `scanner.rs` / `monitor.rs` / `weakness.rs` code that counts and alerts on unsafe usage is valuable oversight infrastructure. Keep it even though current count is 0 — it acts as a regression detector.
